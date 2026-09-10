//! Authenticated Axum adapter for JSON-RPC dispatch
use super::{InvocationContext, InvocationOrigin, OperationRegistry};
use crate::io::api;
use crate::io::ApiResult;
use crate::util::constants::app::DEFAULT_JSON_RPC_MAX_BODY_BYTES;
use acorn_core::util::constant_time_eq;
use alloc::sync::Arc;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use core::net::SocketAddr;
use secrecy::ExposeSecret;
use tokio::net::TcpListener;

/// Immutable state for the authenticated JSON-RPC HTTP endpoint.
#[derive(Clone)]
pub struct RpcHttpService {
    context: InvocationContext,
    max_body_bytes: usize,
    registry: OperationRegistry,
    token: api::Secret,
}
impl RpcHttpService {
    /// Construct an authenticated JSON-RPC HTTP service.
    pub fn new(registry: OperationRegistry, token: api::Secret, context: InvocationContext) -> Self {
        Self {
            context,
            max_body_bytes: DEFAULT_JSON_RPC_MAX_BODY_BYTES,
            registry,
            token,
        }
    }
    /// Set the maximum accepted request-body size.
    pub fn with_max_body_bytes(self, max_body_bytes: usize) -> Self {
        Self {
            max_body_bytes: max_body_bytes.max(1),
            ..self
        }
    }
    /// Build an Axum router exposing `POST /rpc`.
    pub fn router(self) -> Router {
        Router::new().route("/rpc", post(receive)).with_state(Arc::new(self))
    }
    fn authenticated(&self, headers: &HeaderMap) -> bool {
        headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .is_some_and(|received| constant_time_eq(received.as_bytes(), ExposeSecret::expose_secret(&self.token).as_bytes()))
    }
    fn content_type_supported(headers: &HeaderMap) -> bool {
        headers.get(CONTENT_TYPE).and_then(|value| value.to_str().ok()).is_some_and(|value| {
            value
                .split(';')
                .next()
                .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("application/json"))
        })
    }
}
async fn receive(State(service): State<Arc<RpcHttpService>>, headers: HeaderMap, body: Bytes) -> Response {
    let authenticated = service.authenticated(&headers);
    let content_type_supported = RpcHttpService::content_type_supported(&headers);
    let body_within_limit = body.len() <= service.max_body_bytes;
    match (authenticated, content_type_supported, body_within_limit) {
        | (false, _, _) => StatusCode::UNAUTHORIZED.into_response(),
        | (_, false, _) => StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response(),
        | (_, _, false) => StatusCode::PAYLOAD_TOO_LARGE.into_response(),
        | (true, true, true) => {
            let context = InvocationContext {
                origin: InvocationOrigin::JsonRpc,
                ..service.context.clone()
            };
            let response = service.registry.dispatch(&body, context).await;
            match response.body {
                | Some(body) => {
                    let status = StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                    let headers = [(CONTENT_TYPE, HeaderValue::from_static("application/json"))];
                    (status, headers, body).into_response()
                }
                | None => StatusCode::NO_CONTENT.into_response(),
            }
        }
    }
}
/// Serve the built-in operation registry over authenticated JSON-RPC HTTP.
pub async fn serve(address: SocketAddr, token: api::Secret, allow_mutation: bool, offline: bool) -> ApiResult<()> {
    match TcpListener::bind(address).await {
        | Ok(listener) => match OperationRegistry::acorn() {
            | Ok(registry) => {
                let service = RpcHttpService::new(
                    registry,
                    token,
                    InvocationContext {
                        allow_mutation,
                        offline,
                        ..InvocationContext::default()
                    },
                );
                axum::serve(listener, service.router())
                    .await
                    .map_err(|why| color_eyre::eyre::eyre!("JSON-RPC server failed — {why}"))
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(color_eyre::eyre::eyre!("Failed to bind JSON-RPC server to {address} — {why}")),
    }
}
