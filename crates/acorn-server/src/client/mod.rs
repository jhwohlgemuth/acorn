//! Bounded authenticated JSON-RPC client for the canonical ACORN operations.
use acorn_service::{
    CheckReadabilityRequest, CheckReadabilityResponse, ProcessRepositoryRequest, ProcessRepositoryResponse, RequestId, RpcError,
    ValidateResearchActivityRequest, ValidateResearchActivityResponse, ValidateStandardRequest,
};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicI64, Ordering};
use core::time::Duration;
use futures::StreamExt;
use reqwest::{Client, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};

/// One entry in an ordered JSON-RPC batch.
#[derive(Clone, Debug)]
pub enum JsonRpcBatchItem {
    /// A call that expects a result.
    Call {
        /// Registered method name.
        method: String,
        /// Structured method parameters.
        params: Value,
    },
    /// A notification that produces no result.
    Notification {
        /// Registered method name.
        method: String,
        /// Structured method parameters.
        params: Value,
    },
}
/// Failures returned by the JSON-RPC client before or after remote dispatch.
#[derive(Debug, thiserror::Error)]
pub enum JsonRpcClientError {
    /// The endpoint is invalid or violates the plaintext policy.
    #[error("Invalid JSON-RPC endpoint: {0}")]
    Endpoint(String),
    /// The request or response exceeded the configured bound.
    #[error("JSON-RPC payload exceeds the configured {0}-byte bound")]
    PayloadBounds(usize),
    /// The remote endpoint returned a JSON-RPC error.
    #[error("JSON-RPC error {code}: {message}", code = .0.code, message = .0.message)]
    Remote(RpcError),
    /// The HTTP exchange failed.
    #[error("JSON-RPC transport failed: {0}")]
    Transport(#[from] reqwest::Error),
    /// The remote document was malformed or did not match the request.
    #[error("Invalid JSON-RPC response: {0}")]
    InvalidResponse(String),
    /// Request or response serialization failed.
    #[error("JSON-RPC codec failed: {0}")]
    Codec(#[from] serde_json::Error),
}
/// One call result from an ordered mixed batch.
#[derive(Clone, Debug, PartialEq)]
pub struct JsonRpcBatchResult {
    /// Identifier assigned by this client.
    pub id: RequestId,
    /// Successful value or public remote error.
    pub result: Result<Value, RpcError>,
}
/// Authenticated bounded JSON-RPC client.
#[derive(Clone)]
pub struct JsonRpcClient {
    endpoint: Url,
    http: Client,
    max_payload_bytes: usize,
    next_id: Arc<AtomicI64>,
    token: Arc<str>,
}
impl JsonRpcClient {
    /// Construct a client, allowing plaintext only for loopback endpoints.
    pub fn new(endpoint: &str, token: impl Into<Arc<str>>, timeout: Duration, max_payload_bytes: usize) -> Result<Self, JsonRpcClientError> {
        match max_payload_bytes {
            | 0 => Err(JsonRpcClientError::PayloadBounds(0)),
            | _ => Url::parse(endpoint)
                .map_err(|error| JsonRpcClientError::Endpoint(error.to_string()))
                .and_then(|endpoint| {
                    let supported = matches!(endpoint.scheme(), "http" | "https");
                    let loopback = endpoint.host_str().is_some_and(|host| {
                        host.eq_ignore_ascii_case("localhost") || host.parse::<core::net::IpAddr>().is_ok_and(|address| address.is_loopback())
                    });
                    match supported && (endpoint.scheme() != "http" || loopback) {
                        | false => Err(JsonRpcClientError::Endpoint(
                            "use HTTPS, or HTTP with an explicit loopback host".to_string(),
                        )),
                        | true => Client::builder()
                            .timeout(timeout)
                            .build()
                            .map_err(JsonRpcClientError::from)
                            .map(|http| Self {
                                endpoint,
                                http,
                                max_payload_bytes,
                                next_id: Arc::new(AtomicI64::new(1)),
                                token: token.into(),
                            }),
                    }
                }),
        }
    }
    /// Call one arbitrary method and decode its typed result.
    pub async fn call<I, O>(&self, method: &str, params: &I) -> Result<O, JsonRpcClientError>
    where
        I: Serialize + ?Sized,
        O: DeserializeOwned,
    {
        let id = self.id();
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))
            .await
            .and_then(|response| response.ok_or_else(|| JsonRpcClientError::InvalidResponse("call returned no document".to_string())))
            .and_then(|response| decode_response(response, &id))
            .and_then(|response| serde_json::from_value(response).map_err(JsonRpcClientError::from))
    }
    /// Send one arbitrary notification.
    pub async fn notify<I>(&self, method: &str, params: &I) -> Result<(), JsonRpcClientError>
    where
        I: Serialize + ?Sized,
    {
        match self.send(json!({"jsonrpc":"2.0","method":method,"params":params})).await {
            | Err(error) => Err(error),
            | Ok(None) => Ok(()),
            | Ok(Some(_)) => Err(JsonRpcClientError::InvalidResponse("notification returned a document".to_string())),
        }
    }
    /// Send an ordered mixed batch and return call results in call order.
    pub async fn batch(&self, items: Vec<JsonRpcBatchItem>) -> Result<Vec<JsonRpcBatchResult>, JsonRpcClientError> {
        let mut ids = Vec::new();
        let documents = items
            .into_iter()
            .map(|item| match item {
                | JsonRpcBatchItem::Call { method, params } => {
                    let id = self.id();
                    ids.push(id.clone());
                    json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
                }
                | JsonRpcBatchItem::Notification { method, params } => json!({"jsonrpc":"2.0","method":method,"params":params}),
            })
            .collect::<Vec<_>>();
        match self.send(Value::Array(documents)).await {
            | Err(error) => Err(error),
            | Ok(response) => {
                match ids.is_empty() {
                    | true => response.map_or(Ok(Vec::new()), |_| {
                        Err(JsonRpcClientError::InvalidResponse(
                            "notification-only batch returned a document".to_string(),
                        ))
                    }),
                    | false => response
                        .and_then(|value| value.as_array().cloned())
                        .ok_or_else(|| JsonRpcClientError::InvalidResponse("batch response must be an array".to_string()))
                        .and_then(|values| {
                            ids.into_iter()
                                .map(|id| {
                                    serde_json::to_value(&id)
                                        .map_err(JsonRpcClientError::from)
                                        .and_then(|expected| {
                                            values.iter().find(|value| value.get("id") == Some(&expected)).cloned().ok_or_else(|| {
                                                JsonRpcClientError::InvalidResponse("batch response omitted an identifier".to_string())
                                            })
                                        })
                                        .and_then(|value| match decode_response(value, &id) {
                                            | Ok(value) => Ok(JsonRpcBatchResult { id, result: Ok(value) }),
                                            | Err(JsonRpcClientError::Remote(error)) => Ok(JsonRpcBatchResult { id, result: Err(error) }),
                                            | Err(error) => Err(error),
                                        })
                                })
                                .collect()
                        }),
                }
            }
        }
    }
    /// Validate supplied research activity data.
    pub async fn validate_research_activity(
        &self,
        request: &ValidateResearchActivityRequest,
    ) -> Result<ValidateResearchActivityResponse, JsonRpcClientError> {
        self.call("acorn.research-activity.validate", request).await
    }
    /// Validate supplied data against a selected metadata standard.
    pub async fn validate_standard(&self, request: &ValidateStandardRequest) -> Result<ValidateResearchActivityResponse, JsonRpcClientError> {
        self.call("acorn.standard.validate", request).await
    }
    /// Check readability of supplied text.
    pub async fn check_readability(&self, request: &CheckReadabilityRequest) -> Result<CheckReadabilityResponse, JsonRpcClientError> {
        self.call("acorn.readability.check", request).await
    }
    /// Process a supplied repository snapshot.
    pub async fn process_repository(&self, request: &ProcessRepositoryRequest) -> Result<ProcessRepositoryResponse, JsonRpcClientError> {
        self.call("acorn.workflows.process", request).await
    }
    fn id(&self) -> RequestId {
        RequestId::Number(self.next_id.fetch_add(1, Ordering::Relaxed))
    }
    async fn send(&self, document: Value) -> Result<Option<Value>, JsonRpcClientError> {
        match serde_json::to_vec(&document).map_err(JsonRpcClientError::from) {
            | Err(error) => Err(error),
            | Ok(encoded) if encoded.len() > self.max_payload_bytes => Err(JsonRpcClientError::PayloadBounds(self.max_payload_bytes)),
            | Ok(encoded) => match self
                .http
                .post(self.endpoint.clone())
                .bearer_auth(self.token.as_ref())
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(encoded)
                .send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .map_err(JsonRpcClientError::from)
            {
                | Err(error) => Err(error),
                | Ok(response) => match (
                    response.status() == reqwest::StatusCode::NO_CONTENT,
                    response.content_length().is_some_and(|length| length > self.max_payload_bytes as u64),
                ) {
                    | (true, _) => Ok(None),
                    | (false, true) => Err(JsonRpcClientError::PayloadBounds(self.max_payload_bytes)),
                    | (false, false) => {
                        let mut bytes = Vec::new();
                        let mut stream = response.bytes_stream();
                        loop {
                            match stream.next().await {
                                | None => break serde_json::from_slice(&bytes).map(Some).map_err(JsonRpcClientError::from),
                                | Some(Err(error)) => break Err(JsonRpcClientError::from(error)),
                                | Some(Ok(chunk)) if bytes.len().saturating_add(chunk.len()) > self.max_payload_bytes => {
                                    break Err(JsonRpcClientError::PayloadBounds(self.max_payload_bytes));
                                }
                                | Some(Ok(chunk)) => bytes.extend_from_slice(&chunk),
                            }
                        }
                    }
                },
            },
        }
    }
}
fn decode_response(value: Value, expected_id: &RequestId) -> Result<Value, JsonRpcClientError> {
    serde_json::to_value(expected_id).map_err(JsonRpcClientError::from).and_then(|expected| {
        match value.get("jsonrpc") == Some(&Value::String("2.0".to_string())) && value.get("id") == Some(&expected) {
            | false => Err(JsonRpcClientError::InvalidResponse("version or identifier mismatch".to_string())),
            | true => match (value.get("result"), value.get("error")) {
                | (Some(result), None) => Ok(result.clone()),
                | (None, Some(error)) => serde_json::from_value(error.clone())
                    .map(JsonRpcClientError::Remote)
                    .map_err(JsonRpcClientError::from)
                    .and_then(Err),
                | _ => Err(JsonRpcClientError::InvalidResponse(
                    "response must contain exactly one of result or error".to_string(),
                )),
            },
        }
    })
}

#[cfg(test)]
mod tests;
