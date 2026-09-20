//! Configured authenticated gRPC client for ACORN operations.
use super::proto::acorn_rpc_client::AcornRpcClient;
use super::proto::{BatchRequest, BatchResponse, InvokeRequest, InvokeResponse, NotifyRequest};
use super::{DEFAULT_MAX_MESSAGE_BYTES, DEFAULT_REQUEST_TIMEOUT_SECONDS};
use crate::io::api::Secret;
use crate::io::ApiResult;
use color_eyre::eyre::eyre;
use core::net::IpAddr;
use core::str::FromStr;
use core::time::Duration;
use secrecy::ExposeSecret;
use tonic::metadata::AsciiMetadataValue;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};
use tonic::{Request, Status};

const AUTHORIZATION: &str = "authorization";
/// Authenticated client for the versioned ACORN gRPC service.
pub struct GrpcClient {
    authorization: AsciiMetadataValue,
    inner: AcornRpcClient<Channel>,
    request_timeout: Duration,
}
/// Connection policy for a remote ACORN gRPC service.
pub struct GrpcClientConfig {
    ca_certificate: Option<Vec<u8>>,
    endpoint: String,
    max_message_bytes: usize,
    offline: bool,
    request_timeout: Duration,
    token: Secret,
}
impl GrpcClient {
    /// Invoke an ordered batch of operations.
    pub async fn batch(&mut self, batch: BatchRequest) -> Result<BatchResponse, Status> {
        let request = self.request(batch);
        self.inner.batch(request).await.map(tonic::Response::into_inner)
    }
    /// Connect using the supplied endpoint policy.
    pub async fn connect(config: GrpcClientConfig) -> ApiResult<Self> {
        let token_empty = ExposeSecret::expose_secret(&config.token).trim().is_empty();
        match (token_empty, config.endpoint()) {
            | (true, _) => Err(eyre!("A non-empty gRPC bearer token is required")),
            | (_, Err(why)) => Err(why),
            | (false, Ok(endpoint)) => {
                let authorization = format!("Bearer {}", ExposeSecret::expose_secret(&config.token))
                    .parse::<AsciiMetadataValue>()
                    .map_err(|_| eyre!("gRPC bearer token contains invalid metadata characters"));
                match authorization {
                    | Err(why) => Err(why),
                    | Ok(authorization) => endpoint
                        .connect()
                        .await
                        .map_err(|why| eyre!("Failed to connect to gRPC endpoint — {why}"))
                        .map(|channel| Self {
                            authorization,
                            inner: AcornRpcClient::new(channel)
                                .max_decoding_message_size(config.max_message_bytes)
                                .max_encoding_message_size(config.max_message_bytes),
                            request_timeout: config.request_timeout,
                        }),
                }
            }
        }
    }
    /// Invoke one operation.
    pub async fn invoke(&mut self, invocation: InvokeRequest) -> Result<InvokeResponse, Status> {
        let request = self.request(invocation);
        self.inner.invoke(request).await.map(tonic::Response::into_inner)
    }
    /// Dispatch one operation notification.
    pub async fn notify(&mut self, notification: NotifyRequest) -> Result<(), Status> {
        let request = self.request(notification);
        self.inner.notify(request).await.map(|_| ())
    }
    fn request<T>(&self, message: T) -> Request<T> {
        let mut request = Request::new(message);
        request.metadata_mut().insert(AUTHORIZATION, self.authorization.clone());
        request.set_timeout(self.request_timeout);
        request
    }
}
impl GrpcClientConfig {
    /// Construct client policy for `endpoint` and a dedicated bearer token.
    pub fn new(endpoint: impl Into<String>, token: Secret) -> Self {
        Self {
            ca_certificate: None,
            endpoint: endpoint.into(),
            max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
            offline: false,
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECONDS),
            token,
        }
    }
    pub(crate) fn endpoint(&self) -> ApiResult<Endpoint> {
        Endpoint::from_shared(self.endpoint.clone())
            .map_err(|why| eyre!("Invalid gRPC endpoint — {why}"))
            .and_then(|endpoint| {
                let uri = endpoint.uri();
                let loopback = uri.host().is_some_and(is_loopback_host);
                let secure = uri.scheme_str().is_some_and(|scheme| scheme.eq_ignore_ascii_case("https"));
                let valid_scheme = uri.scheme_str().is_some_and(|scheme| matches!(scheme, "http" | "https"));
                match (valid_scheme, loopback, secure, self.offline) {
                    | (false, _, _, _) => Err(eyre!("gRPC endpoint must use http or https")),
                    | (_, false, _, true) => Err(eyre!("Remote gRPC clients are unavailable while offline")),
                    | (_, false, false, _) => Err(eyre!("TLS is required for non-loopback gRPC endpoints")),
                    | (_, _, true, _) => {
                        let tls = self.ca_certificate.as_ref().map_or_else(
                            || ClientTlsConfig::new().with_native_roots(),
                            |certificate| {
                                ClientTlsConfig::new()
                                    .with_native_roots()
                                    .ca_certificate(Certificate::from_pem(certificate))
                            },
                        );
                        endpoint
                            .tls_config(tls)
                            .map_err(|why| eyre!("Invalid gRPC client TLS configuration — {why}"))
                    }
                    | _ => Ok(endpoint),
                }
            })
            .map(|endpoint| endpoint.connect_timeout(self.request_timeout).timeout(self.request_timeout))
    }
    /// Set a PEM-encoded certificate authority for TLS verification.
    pub fn with_ca_certificate(self, ca_certificate: impl Into<Vec<u8>>) -> Self {
        Self {
            ca_certificate: Some(ca_certificate.into()),
            ..self
        }
    }
    /// Set maximum encoded and decoded message size.
    pub fn with_max_message_bytes(self, max_message_bytes: usize) -> Self {
        Self {
            max_message_bytes: max_message_bytes.max(1),
            ..self
        }
    }
    /// Apply offline policy to endpoint selection.
    pub fn with_offline(self, offline: bool) -> Self {
        Self { offline, ..self }
    }
    /// Set connection and request deadlines.
    pub fn with_request_timeout(self, request_timeout: Duration) -> Self {
        Self {
            request_timeout: request_timeout.max(Duration::from_millis(1)),
            ..self
        }
    }
}
pub(crate) fn is_loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost") || IpAddr::from_str(host.trim_matches(['[', ']'])).is_ok_and(|address| address.is_loopback())
}
