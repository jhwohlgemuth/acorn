//! Authenticated gRPC service and server for registered ACORN operations.
use super::proto::acorn_rpc_server::{AcornRpc, AcornRpcServer};
use super::proto::{invoke_response, BatchRequest, BatchResponse, InvokeRequest, InvokeResponse, NotifyRequest};
use super::{empty_params, json_value, protobuf_value, DEFAULT_MAX_BATCH_LENGTH, DEFAULT_MAX_MESSAGE_BYTES, DEFAULT_REQUEST_TIMEOUT_SECONDS};
use crate::io::api::json_rpc::{InvocationContext, InvocationOrigin, MethodName, OperationRegistry, Principal, RpcError};
use crate::io::api::Secret;
use crate::io::ApiResult;
use acorn_core::util::constant_time_eq;
use color_eyre::eyre::eyre;
use core::future::Future;
use core::net::SocketAddr;
use core::time::Duration;
use futures::stream::{self, StreamExt};
use secrecy::ExposeSecret;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tonic::{Request, Response, Status};

const AUTHORIZATION: &str = "authorization";
/// Configured gRPC server.
pub struct GrpcServer {
    max_message_bytes: usize,
    request_timeout: Duration,
    service: GrpcService,
    tls: Option<TlsIdentity>,
}
/// Authenticated adapter from generated gRPC methods to the shared operation registry.
#[derive(Clone)]
pub struct GrpcService {
    context: InvocationContext,
    max_batch_length: usize,
    registry: OperationRegistry,
    token: Secret,
}
/// PEM-encoded server certificate and private key.
#[derive(Clone)]
pub struct TlsIdentity {
    certificate: Vec<u8>,
    private_key: Vec<u8>,
}
impl GrpcServer {
    /// Construct a gRPC server over `registry` using a dedicated bearer token.
    pub fn new(registry: OperationRegistry, token: Secret, context: InvocationContext) -> ApiResult<Self> {
        GrpcService::new(registry, token, context).map(|service| Self {
            max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECONDS),
            service,
            tls: None,
        })
    }
    /// Serve until the transport exits.
    pub async fn serve(self, address: SocketAddr) -> ApiResult<()> {
        self.serve_with_shutdown(address, core::future::pending()).await
    }
    /// Serve until `shutdown` completes.
    pub async fn serve_with_shutdown<F>(self, address: SocketAddr, shutdown: F) -> ApiResult<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        match validate_transport(address, self.tls.as_ref()) {
            | Err(why) => Err(why),
            | Ok(()) => {
                let builder = match self.tls {
                    | Some(identity) => Server::builder()
                        .tls_config(ServerTlsConfig::new().identity(identity.into()))
                        .map_err(|why| eyre!("Invalid gRPC TLS configuration — {why}")),
                    | None => Ok(Server::builder()),
                };
                match builder {
                    | Err(why) => Err(why),
                    | Ok(builder) => {
                        let service = AcornRpcServer::new(self.service)
                            .max_decoding_message_size(self.max_message_bytes)
                            .max_encoding_message_size(self.max_message_bytes);
                        builder
                            .timeout(self.request_timeout)
                            .add_service(service)
                            .serve_with_shutdown(address, shutdown)
                            .await
                            .map_err(|why| eyre!("gRPC server failed — {why}"))
                    }
                }
            }
        }
    }
    /// Set the maximum encoded and decoded message size.
    pub fn with_max_message_bytes(self, max_message_bytes: usize) -> Self {
        Self {
            max_message_bytes: max_message_bytes.max(1),
            ..self
        }
    }
    /// Set the server-side request timeout.
    pub fn with_request_timeout(self, request_timeout: Duration) -> Self {
        Self {
            request_timeout: request_timeout.max(Duration::from_millis(1)),
            ..self
        }
    }
    /// Configure the server TLS identity.
    pub fn with_tls(self, tls: TlsIdentity) -> Self {
        Self { tls: Some(tls), ..self }
    }
}
impl GrpcService {
    /// Construct an authenticated registry adapter.
    pub fn new(registry: OperationRegistry, token: Secret, context: InvocationContext) -> ApiResult<Self> {
        match ExposeSecret::expose_secret(&token).trim().is_empty() {
            | true => Err(eyre!("A non-empty gRPC bearer token is required")),
            | false => Ok(Self {
                context,
                max_batch_length: DEFAULT_MAX_BATCH_LENGTH,
                registry,
                token,
            }),
        }
    }
    /// Set the maximum accepted batch length.
    pub fn with_max_batch_length(self, max_batch_length: usize) -> Self {
        Self {
            max_batch_length: max_batch_length.max(1),
            ..self
        }
    }
    fn authenticate<T>(&self, request: &Request<T>) -> Result<(), Status> {
        let authenticated = request
            .metadata()
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .is_some_and(|received| constant_time_eq(received.as_bytes(), ExposeSecret::expose_secret(&self.token).as_bytes()));
        match authenticated {
            | true => Ok(()),
            | false => Err(Status::unauthenticated("A valid gRPC bearer token is required")),
        }
    }
    fn context(&self, identifier: &Option<super::proto::RequestId>) -> InvocationContext {
        InvocationContext {
            correlation_id: identifier.as_ref().and_then(|identifier| match identifier.value.as_ref() {
                | Some(super::proto::request_id::Value::Number(value)) => Some(value.to_string()),
                | Some(super::proto::request_id::Value::Text(value)) => Some(value.clone()),
                | None => None,
            }),
            origin: InvocationOrigin::Grpc,
            principal: Principal {
                identifier: "grpc".to_string(),
            },
            ..self.context.clone()
        }
    }
    async fn invoke_request(&self, request: InvokeRequest) -> InvokeResponse {
        let id = request.id;
        let identifier_valid = id.as_ref().is_some_and(|identifier| identifier.value.is_some());
        let decoded = match identifier_valid {
            | false => Err(RpcError::new(crate::io::api::json_rpc::INVALID_REQUEST, "Invalid Request")),
            | true => MethodName::try_from(request.method)
                .map_err(|_| RpcError::new(crate::io::api::json_rpc::INVALID_REQUEST, "Invalid Request"))
                .and_then(|method| {
                    request
                        .params
                        .map_or_else(|| Ok(empty_params()), json_value)
                        .map(|params| (method, params))
                }),
        };
        let outcome = match decoded {
            | Ok((method, params)) => self
                .registry
                .invoke(&method, params, self.context(&id))
                .await
                .and_then(protobuf_value)
                .map(invoke_response::Outcome::Result)
                .unwrap_or_else(|error| invoke_response::Outcome::Error(error.into())),
            | Err(error) => invoke_response::Outcome::Error(error.into()),
        };
        InvokeResponse { id, outcome: Some(outcome) }
    }
}
#[tonic::async_trait]
impl AcornRpc for GrpcService {
    async fn batch(&self, request: Request<BatchRequest>) -> Result<Response<BatchResponse>, Status> {
        match self.authenticate(&request) {
            | Err(status) => Err(status),
            | Ok(()) => {
                let requests = request.into_inner().requests;
                match requests.len() {
                    | 0 => Err(Status::invalid_argument("A gRPC batch cannot be empty")),
                    | length if length > self.max_batch_length => Err(Status::resource_exhausted("gRPC batch exceeds the configured request limit")),
                    | _ => {
                        let responses = stream::iter(requests)
                            .then(|request| self.invoke_request(request))
                            .collect::<Vec<_>>()
                            .await;
                        Ok(Response::new(BatchResponse { responses }))
                    }
                }
            }
        }
    }
    async fn invoke(&self, request: Request<InvokeRequest>) -> Result<Response<InvokeResponse>, Status> {
        match self.authenticate(&request) {
            | Err(status) => Err(status),
            | Ok(()) => Ok(Response::new(self.invoke_request(request.into_inner()).await)),
        }
    }
    async fn notify(&self, request: Request<NotifyRequest>) -> Result<Response<()>, Status> {
        match self.authenticate(&request) {
            | Err(status) => Err(status),
            | Ok(()) => {
                let notification = request.into_inner();
                let decoded = MethodName::try_from(notification.method)
                    .map_err(|_| Status::invalid_argument("Invalid operation name"))
                    .and_then(|method| {
                        notification
                            .params
                            .map_or_else(|| Ok(empty_params()), json_value)
                            .map_err(|why| Status::invalid_argument(why.message))
                            .map(|params| (method, params))
                    });
                match decoded {
                    | Err(status) => Err(status),
                    | Ok((method, params)) => {
                        let _ = self.registry.invoke(&method, params, self.context(&None)).await;
                        Ok(Response::new(()))
                    }
                }
            }
        }
    }
}
impl From<TlsIdentity> for Identity {
    fn from(identity: TlsIdentity) -> Self {
        Self::from_pem(identity.certificate, identity.private_key)
    }
}
impl TlsIdentity {
    /// Construct a server identity from PEM-encoded certificate and private-key bytes.
    pub fn from_pem(certificate: impl Into<Vec<u8>>, private_key: impl Into<Vec<u8>>) -> Self {
        Self {
            certificate: certificate.into(),
            private_key: private_key.into(),
        }
    }
}
/// Serve the built-in operation registry over authenticated gRPC.
pub async fn serve(address: SocketAddr, token: Secret, allow_mutation: bool, offline: bool, tls: Option<TlsIdentity>) -> ApiResult<()> {
    match OperationRegistry::acorn().and_then(|registry| {
        GrpcServer::new(
            registry,
            token,
            InvocationContext {
                allow_mutation,
                offline,
                ..InvocationContext::default()
            },
        )
    }) {
        | Err(why) => Err(why),
        | Ok(server) => match tls {
            | Some(tls) => server.with_tls(tls).serve(address).await,
            | None => server.serve(address).await,
        },
    }
}
pub(crate) fn validate_transport(address: SocketAddr, tls: Option<&TlsIdentity>) -> ApiResult<()> {
    match (address.ip().is_loopback(), tls.is_some()) {
        | (false, false) => Err(eyre!("TLS is required when serving gRPC on a non-loopback address")),
        | _ => Ok(()),
    }
}
