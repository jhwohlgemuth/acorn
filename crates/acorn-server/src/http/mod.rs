//! Authenticated JSON-RPC HTTP and unauthenticated health endpoints.
use acorn_core::util::constant_time_eq;
use acorn_service::{InvocationContext, InvocationOrigin, OperationRegistry};
use alloc::sync::Arc;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use std::time::Instant;

/// Shared HTTP adapter state.
#[derive(Clone)]
pub struct HttpState {
    context: InvocationContext,
    ready: Arc<AtomicBool>,
    registry: OperationRegistry,
    request_timeout: Duration,
    token: Arc<str>,
}
impl HttpState {
    /// Construct authenticated JSON-RPC state.
    pub fn new(
        registry: OperationRegistry,
        token: impl Into<Arc<str>>,
        context: InvocationContext,
        ready: Arc<AtomicBool>,
        request_timeout: Duration,
    ) -> Self {
        Self {
            context,
            ready,
            registry,
            request_timeout,
            token: token.into(),
        }
    }
    /// Build the HTTP router with explicit body bounds.
    pub fn router(self, max_request_bytes: usize) -> Router {
        Router::new()
            .route("/healthz", get(health))
            .route("/readyz", get(readiness))
            .route("/rpc", post(receive))
            .layer(DefaultBodyLimit::max(max_request_bytes))
            .with_state(Arc::new(self))
    }
    fn authenticated(&self, headers: &HeaderMap) -> bool {
        headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .is_some_and(|received| constant_time_eq(received.as_bytes(), self.token.as_bytes()))
    }
}
async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}
async fn readiness(State(state): State<Arc<HttpState>>) -> StatusCode {
    match state.ready.load(Ordering::Acquire) {
        | true => StatusCode::NO_CONTENT,
        | false => StatusCode::SERVICE_UNAVAILABLE,
    }
}
async fn receive(State(state): State<Arc<HttpState>>, headers: HeaderMap, body: Bytes) -> Response {
    let started = Instant::now();
    let correlation_id = headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.len() <= 128)
        .unwrap_or("unassigned");
    let operation = serde_json::from_slice::<serde_json::Value>(&body)
        .ok()
        .and_then(|value| match value {
            | serde_json::Value::Array(_) => Some("batch".to_string()),
            | serde_json::Value::Object(object) => object.get("method").and_then(serde_json::Value::as_str).map(str::to_string),
            | _ => None,
        })
        .unwrap_or_else(|| "invalid".to_string());
    let supported = headers.get(CONTENT_TYPE).and_then(|value| value.to_str().ok()).is_some_and(|value| {
        value
            .split(';')
            .next()
            .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("application/json"))
    });
    let response = match (state.authenticated(&headers), supported) {
        | (false, _) => StatusCode::UNAUTHORIZED.into_response(),
        | (true, false) => StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response(),
        | (true, true) => {
            let context = InvocationContext {
                origin: InvocationOrigin::JsonRpc,
                ..state.context.clone()
            };
            match tokio::time::timeout(state.request_timeout, state.registry.dispatch(&body, context)).await {
                | Err(_) => StatusCode::REQUEST_TIMEOUT.into_response(),
                | Ok(dispatch) => match dispatch.body {
                    | Some(body) => {
                        let status = StatusCode::from_u16(dispatch.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                        (status, [(CONTENT_TYPE, HeaderValue::from_static("application/json"))], body).into_response()
                    }
                    | None => StatusCode::NO_CONTENT.into_response(),
                },
            }
        }
    };
    trace_request(&operation, correlation_id, body.len(), response.status(), started);
    response
}
fn trace_request(operation: &str, correlation_id: &str, request_bytes: usize, status: StatusCode, started: Instant) {
    tracing::info!(
        protocol = "json-rpc",
        operation,
        correlation_id,
        outcome = status.as_u16(),
        latency_ms = started.elapsed().as_millis(),
        request_bytes,
        "ACORN service request completed"
    );
}

#[cfg(test)]
mod tests;
