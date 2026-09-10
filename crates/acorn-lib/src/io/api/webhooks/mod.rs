//! Provider-neutral webhook verification and callback transport contracts
use crate::io::api;
use crate::io::api::json_rpc::Notification;
use crate::io::http::{HeaderMapExt, HttpMethod, HttpRequest, HttpResponseTooLarge, HttpService};
use crate::io::ApiResult;
use crate::util::constants::app::MAX_CALLBACK_RESPONSE_BYTES;
use acorn_core::prelude::{Box, String, Vec};
use acorn_core::util::constant_time_eq;
use alloc::sync::Arc;
use color_eyre::eyre::eyre;
use core::{fmt, future::Future, pin::Pin, time::Duration};
use data_encoding::BASE64;
use http::header::{HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use http::HeaderMap;
use jiff::{SignedDuration, Timestamp};
use ring::hmac;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tokio::time::Instant;

/// Durable provider-neutral webhook operation storage.
pub mod store;

/// Awaitable result produced by a durable webhook operation.
pub type Operation = Pin<Box<dyn Future<Output = ApiResult<()>> + Send>>;
/// Callback used by durable webhook workers.
pub type OperationHandler<T> = Arc<dyn Fn(Delivery<T>) -> Operation + Send + Sync + 'static>;
/// Deployment-owned resolver for callback destination names.
pub trait CallbackResolver: Send + Sync {
    /// Resolve a persisted destination name into its current URL and credentials.
    fn resolve(&self, name: &str) -> Result<CallbackDestination, WebhookError>;
}
/// Authentication contract for inbound webhook requests.
pub trait Verifier {
    /// Verify exact request bytes using an explicit clock value.
    fn verify<'a>(&self, request: InboundRequest<'a>, now: Timestamp) -> Result<VerifiedRequest<'a>, WebhookError>;
}
/// Provider adapter that authenticates, normalizes, and classifies webhook deliveries.
pub trait WebhookProvider: Send + Sync {
    /// Provider-normalized event type.
    type Event: Clone + for<'de> Deserialize<'de> + Send + Serialize + Sync + 'static;

    /// Authenticate and normalize one request.
    fn receive(&self, request: InboundRequest<'_>, now: Timestamp) -> Result<Delivery<Self::Event>, WebhookError>;
    /// Classify one normalized delivery into registered operations.
    fn classify(&self, delivery: &Delivery<Self::Event>) -> Result<Vec<OperationSpec>, WebhookError>;
    /// Stable provider namespace.
    fn name(&self) -> &'static str;
}
/// Stable webhook verification and dispatch error categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// A required header is absent or malformed.
    BadRequest,
    /// Authentication failed.
    Unauthorized,
    /// Callback target is invalid or unsafe.
    InvalidTarget,
    /// HTTP transport failed.
    Transport,
    /// Callback endpoint returned a non-success status.
    Downstream,
    /// Callback response exceeded its configured bound.
    ResponseTooLarge,
}
/// One active claim tracked by a [`WebhookRuntime`].
pub(crate) struct ActiveClaim {
    runtime: WebhookRuntime,
}
/// Callback destination resolved at attempt time so credentials are never persisted with an intent.
pub struct CallbackDestination {
    headers: Vec<CallbackHeader>,
    target: CallbackTarget,
}
/// Callback header with an explicit public-or-secret diagnostic classification.
#[derive(Clone)]
pub struct CallbackHeader {
    name: HeaderName,
    secret: bool,
    value: HeaderValue,
}
/// Validated HTTPS callback target whose debug output never exposes its path or query.
#[derive(Clone)]
pub struct CallbackTarget(String);
/// Authenticated, provider-normalized webhook delivery.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Delivery<T> {
    /// Stable provider delivery identifier.
    pub delivery_id: String,
    /// Provider-normalized event.
    pub event: T,
}
/// Minimal result returned after one callback attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DispatchResult {
    /// Downstream HTTP status.
    pub status_code: u16,
}
/// Constant-time verifier for a shared secret carried in one header.
#[derive(Clone)]
pub struct HeaderSecretVerifier {
    delivery_headers: Vec<HeaderName>,
    secret: api::Secret,
    secret_header: HeaderName,
}
/// Borrowed inbound request material used by webhook verifiers.
#[derive(Clone, Copy, Debug)]
pub struct InboundRequest<'a> {
    /// Request headers.
    pub headers: &'a HeaderMap,
    /// Exact raw request body bytes.
    pub raw_body: &'a [u8],
}
/// Durable operation selected from one provider-normalized delivery.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationSpec {
    /// Provider-scoped canonical operation key.
    pub idempotency_key: String,
    /// JSON-RPC notification dispatched by the worker.
    pub notification: Notification,
    /// Stable provider namespace.
    pub provider: String,
}
/// Authenticated delivery and the registered operations selected from it.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedDelivery<T> {
    /// Provider-normalized delivery.
    pub delivery: Delivery<T>,
    /// Deterministically ordered operations selected from the delivery.
    pub operations: Vec<OperationSpec>,
}
#[derive(Clone, Copy, Debug)]
struct RuntimeState {
    accepting: bool,
    active_claims: u64,
    initialized: bool,
    last_error_at: Option<Timestamp>,
    last_success_at: Option<Timestamp>,
}
/// Standard Webhooks HMAC-SHA256 verifier.
#[derive(Clone)]
pub struct StandardWebhooksVerifier {
    key: hmac::Key,
    tolerance_seconds: u64,
}
/// Authenticated request material returned by a verifier.
#[derive(Clone, Copy, Debug)]
pub struct VerifiedRequest<'a> {
    /// Stable delivery identifier selected during authentication.
    pub delivery_id: &'a str,
    /// Exact authenticated request body bytes.
    pub raw_body: &'a [u8],
}
/// Secret-safe error returned by shared webhook contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebhookError {
    kind: ErrorKind,
    message: &'static str,
    status_code: Option<u16>,
}
/// Provider-neutral webhook admission, readiness, and drain coordinator.
#[derive(Clone, Debug)]
pub struct WebhookRuntime {
    state: Arc<Mutex<RuntimeState>>,
}
/// Observable provider-neutral webhook runtime state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct WebhookRuntimeState {
    /// Whether the runtime accepts new delivery and worker claims.
    pub accepting: bool,
    /// Number of claims currently being processed.
    pub active_claims: u64,
    /// Most recent failed operation or callback settlement.
    pub last_error_at: Option<Timestamp>,
    /// Most recent successful operation or callback settlement.
    pub last_success_at: Option<Timestamp>,
    /// Whether store initialization completed and new work may be accepted.
    pub ready: bool,
}
impl Drop for ActiveClaim {
    fn drop(&mut self) {
        if let Ok(mut state) = self.runtime.state.lock() {
            *state = (*state).active_released();
        }
    }
}
impl CallbackDestination {
    /// Construct a resolved callback destination.
    pub fn new(target: CallbackTarget, headers: Vec<CallbackHeader>) -> Self {
        Self { headers, target }
    }
    pub(super) fn parts(&self) -> (&CallbackTarget, &[CallbackHeader]) {
        (&self.target, &self.headers)
    }
}
impl CallbackHeader {
    fn new(name: &str, value: &str, is_secret: bool) -> Result<Self, WebhookError> {
        HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| WebhookError::new(ErrorKind::BadRequest, "Invalid callback header"))
            .and_then(|name| {
                HeaderValue::from_str(value)
                    .map_err(|_| WebhookError::new(ErrorKind::BadRequest, "Invalid callback header"))
                    .map(|mut value| {
                        let secret = Self::is_sensitive(name.as_str(), is_secret);
                        value.set_sensitive(secret);
                        Self { name, secret, value }
                    })
            })
    }
    /// Check if the given header name is considered sensitive
    pub fn is_sensitive(name: &str, is_secret: bool) -> bool {
        is_secret
            || name == AUTHORIZATION.as_str()
            || name.ends_with("-token")
            || name.ends_with("-key")
            || name.ends_with("-signature")
            || name == "apikey"
    }
    /// Construct a public callback header whose value may appear in diagnostics.
    pub fn public(name: &str, value: &str) -> Result<Self, WebhookError> {
        Self::new(name, value, false)
    }
    /// Construct a secret callback header whose value is always redacted.
    pub fn secret(name: &str, value: &str) -> Result<Self, WebhookError> {
        Self::new(name, value, true)
    }
    /// Construct a secret bearer authorization header.
    pub fn bearer(token: &str) -> Result<Self, WebhookError> {
        Self::secret(AUTHORIZATION.as_str(), &format!("Bearer {token}"))
    }
}
impl fmt::Debug for CallbackHeader {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value: &dyn fmt::Debug = match self.secret {
            | true => &"[REDACTED]",
            | false => &self.value,
        };
        formatter
            .debug_struct("CallbackHeader")
            .field("name", &self.name)
            .field("value", value)
            .finish()
    }
}
impl CallbackTarget {
    /// Validate and retain an HTTPS callback URL.
    pub fn new(url: impl Into<String>) -> Result<Self, WebhookError> {
        let url = url.into();
        reqwest::Url::parse(&url)
            .map_err(|_| WebhookError::new(ErrorKind::InvalidTarget, "Invalid webhook callback target"))
            .and_then(|parsed| {
                Self::is_valid(parsed).then_some(Self(url)).ok_or_else(|| {
                    WebhookError::new(
                        ErrorKind::InvalidTarget,
                        "Webhook callback target must be an HTTPS URL without userinfo or fragments",
                    )
                })
            })
    }
    /// Check if the given URL is a valid HTTPS callback target
    pub fn is_valid(value: reqwest::Url) -> bool {
        value.scheme() == "https"
            && value.host_str().is_some()
            && value.username().is_empty()
            && value.password().is_none()
            && value.fragment().is_none()
    }
    /// Return the exact validated URL for transport use.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for CallbackTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CallbackTarget([REDACTED])")
    }
}
impl fmt::Debug for HeaderSecretVerifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeaderSecretVerifier")
            .field("delivery_headers", &self.delivery_headers)
            .field("secret", &"[REDACTED]")
            .field("secret_header", &self.secret_header)
            .finish()
    }
}
impl HeaderSecretVerifier {
    /// Construct a shared-header verifier and ordered delivery-ID lookup.
    pub fn new(secret_header: &str, secret: impl Into<api::Secret>, delivery_headers: &[&str]) -> Result<Self, WebhookError> {
        HeaderName::from_bytes(secret_header.as_bytes())
            .map_err(|_| WebhookError::new(ErrorKind::BadRequest, "Invalid secret header name"))
            .and_then(|secret_header| {
                delivery_headers
                    .iter()
                    .map(|name| HeaderName::from_bytes(name.as_bytes()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| WebhookError::new(ErrorKind::BadRequest, "Invalid delivery header name"))
                    .map(|delivery_headers| (secret_header, delivery_headers))
            })
            .and_then(|(secret_header, delivery_headers)| {
                let secret = secret.into();
                match ExposeSecret::expose_secret(&secret).is_empty() {
                    | true => Err(WebhookError::new(ErrorKind::Unauthorized, "Invalid webhook authentication secret")),
                    | false => Ok(Self {
                        delivery_headers,
                        secret,
                        secret_header,
                    }),
                }
            })
    }
    /// Construct from a string secret (convenience for tests).
    pub fn from_string(secret_header: &str, secret: impl Into<String>, delivery_headers: &[&str]) -> Result<Self, WebhookError> {
        Self::new(secret_header, api::Secret::from(secret.into()), delivery_headers)
    }
}
impl Verifier for HeaderSecretVerifier {
    fn verify<'a>(&self, request: InboundRequest<'a>, _now: Timestamp) -> Result<VerifiedRequest<'a>, WebhookError> {
        request
            .headers
            .get(&self.secret_header)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| WebhookError::new(ErrorKind::Unauthorized, "Missing webhook authentication header"))
            .and_then(
                |received| match constant_time_eq(ExposeSecret::expose_secret(&self.secret).as_bytes(), received.as_bytes()) {
                    | true => self
                        .delivery_headers
                        .iter()
                        .find_map(|name| request.headers.get(name).and_then(|value| value.to_str().ok()))
                        .filter(|value| !value.trim().is_empty())
                        .ok_or_else(|| WebhookError::new(ErrorKind::BadRequest, "Missing webhook delivery identifier"))
                        .map(|delivery_id| VerifiedRequest {
                            delivery_id,
                            raw_body: request.raw_body,
                        }),
                    | false => Err(WebhookError::new(ErrorKind::Unauthorized, "Webhook authentication failed")),
                },
            )
    }
}
impl OperationSpec {
    /// Construct one provider-namespaced durable operation.
    pub fn new(provider: impl Into<String>, idempotency_key: impl Into<String>, notification: Notification) -> Self {
        Self {
            idempotency_key: idempotency_key.into(),
            notification,
            provider: provider.into(),
        }
    }
}
impl RuntimeState {
    fn active_claimed(self) -> Self {
        Self {
            active_claims: self.active_claims.saturating_add(1),
            ..self
        }
    }
    fn active_released(self) -> Self {
        Self {
            active_claims: self.active_claims.saturating_sub(1),
            ..self
        }
    }
    fn snapshot(&self) -> WebhookRuntimeState {
        WebhookRuntimeState {
            accepting: self.accepting,
            active_claims: self.active_claims,
            last_error_at: self.last_error_at,
            last_success_at: self.last_success_at,
            ready: self.accepting && self.initialized,
        }
    }
    fn unavailable(state: Self) -> Self {
        Self {
            initialized: false,
            last_error_at: Some(Timestamp::now()),
            ..state
        }
    }
    fn updated_at(self, timestamps: store::RuntimeTimestamps) -> Self {
        Self {
            last_error_at: timestamps.last_error_at,
            last_success_at: timestamps.last_success_at,
            ..self
        }
    }
}
impl fmt::Debug for StandardWebhooksVerifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StandardWebhooksVerifier")
            .field("key", &"[REDACTED]")
            .field("tolerance_seconds", &self.tolerance_seconds)
            .finish()
    }
}
impl StandardWebhooksVerifier {
    /// Construct a verifier from a `whsec_` Standard Webhooks secret.
    pub fn new(signing_secret: impl Into<api::Secret>, tolerance_seconds: u64) -> Result<Self, WebhookError> {
        let signing_secret = signing_secret.into();
        let signing_secret = ExposeSecret::expose_secret(&signing_secret);
        signing_secret
            .strip_prefix("whsec_")
            .ok_or_else(|| WebhookError::new(ErrorKind::Unauthorized, "Invalid webhook signing secret"))
            .and_then(|encoded| {
                BASE64
                    .decode(encoded.as_bytes())
                    .map_err(|_| WebhookError::new(ErrorKind::Unauthorized, "Invalid webhook signing secret"))
            })
            .and_then(|key| match key.is_empty() {
                | true => Err(WebhookError::new(ErrorKind::Unauthorized, "Invalid webhook signing secret")),
                | false => Ok(Self {
                    key: hmac::Key::new(hmac::HMAC_SHA256, &key),
                    tolerance_seconds,
                }),
            })
    }
    /// Construct from a string secret (convenience for tests).
    pub fn from_string(signing_secret: &str, tolerance_seconds: u64) -> Result<Self, WebhookError> {
        Self::new(api::Secret::from(signing_secret.to_string()), tolerance_seconds)
    }
}
impl Default for WebhookRuntime {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(RuntimeState {
                accepting: true,
                active_claims: 0,
                initialized: false,
                last_error_at: None,
                last_success_at: None,
            })),
        }
    }
}
impl WebhookRuntime {
    /// Stop accepting new work and wait up to `grace` for active claims.
    pub async fn drain(&self, grace: Duration) -> ApiResult<WebhookRuntimeState> {
        match self.set_accepting(false) {
            | Ok(()) => {
                let deadline = Instant::now().checked_add(grace).unwrap_or_else(Instant::now);
                loop {
                    match self.state() {
                        | Ok(state) if state.active_claims == 0 || Instant::now() >= deadline => break Ok(state),
                        | Ok(_) => tokio::time::sleep(Duration::from_millis(10)).await,
                        | Err(why) => break Err(why),
                    }
                }
            }
            | Err(why) => Err(why),
        }
    }
    /// Initialize the store, recover stale claims, and enable readiness.
    pub fn initialize(&self, queue: &store::OperationQueue, stale_after: SignedDuration) -> ApiResult<()> {
        self.state
            .lock()
            .map(|state| state.initialized)
            .map_err(|why| eyre!("Webhook runtime state lock is poisoned — {why}"))
            .and_then(|initialized| match initialized {
                | true => Ok(()),
                | false => match queue.initialize(stale_after) {
                    | Ok(timestamps) => self.update(|state| RuntimeState {
                        initialized: true,
                        ..state.updated_at(timestamps)
                    }),
                    | Err(why) => {
                        let _ = self.update(RuntimeState::unavailable);
                        Err(why)
                    }
                },
            })
    }
    /// Return whether the runtime currently accepts work.
    pub fn is_accepting(&self) -> bool {
        self.state().is_ok_and(|state| state.accepting)
    }
    /// Return whether the runtime is initialized and accepting work.
    pub fn is_ready(&self) -> bool {
        self.state().is_ok_and(|state| state.ready)
    }
    /// Return a snapshot suitable for readiness and status endpoints.
    pub fn state(&self) -> ApiResult<WebhookRuntimeState> {
        self.state
            .lock()
            .map(|state| state.snapshot())
            .map_err(|why| eyre!("Webhook runtime state lock is poisoned — {why}"))
    }
    pub(crate) fn claim<T>(&self, operation: impl FnOnce() -> ApiResult<Option<T>>) -> ApiResult<Option<(T, ActiveClaim)>> {
        self.state
            .lock()
            .map_err(|why| eyre!("Webhook runtime state lock is poisoned — {why}"))
            .and_then(|mut state| match state.accepting && state.initialized {
                | false => Ok(None),
                | true => match operation() {
                    | Ok(Some(value)) => {
                        *state = state.active_claimed();
                        Ok(Some((value, ActiveClaim { runtime: self.clone() })))
                    }
                    | Ok(None) => Ok(None),
                    | Err(why) => {
                        *state = RuntimeState::unavailable(*state);
                        Err(why)
                    }
                },
            })
    }
    pub(crate) fn mark_unavailable(&self) {
        let _ = self.update(RuntimeState::unavailable);
    }
    pub(crate) fn refresh(&self, queue: &store::OperationQueue) -> ApiResult<()> {
        match queue.runtime_timestamps() {
            | Ok(timestamps) => self.update(|state| state.updated_at(timestamps)),
            | Err(why) => {
                self.mark_unavailable();
                Err(why)
            }
        }
    }
    fn set_accepting(&self, accepting: bool) -> ApiResult<()> {
        self.update(|state| RuntimeState { accepting, ..state })
    }
    fn update(&self, operation: impl FnOnce(RuntimeState) -> RuntimeState) -> ApiResult<()> {
        self.state
            .lock()
            .map_err(|why| eyre!("Webhook runtime state lock is poisoned — {why}"))
            .map(|mut state| *state = operation(*state))
    }
}
impl Verifier for StandardWebhooksVerifier {
    fn verify<'a>(&self, request: InboundRequest<'a>, now: Timestamp) -> Result<VerifiedRequest<'a>, WebhookError> {
        request
            .headers
            .first(&["webhook-id"])
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| WebhookError::new(ErrorKind::BadRequest, "Missing webhook-id header"))
            .and_then(|delivery_id| {
                request
                    .headers
                    .first(&["webhook-timestamp"])
                    .ok_or_else(|| WebhookError::new(ErrorKind::BadRequest, "Missing webhook-timestamp header"))
                    .map(|timestamp| (delivery_id, timestamp))
            })
            .and_then(|(delivery_id, timestamp)| {
                timestamp
                    .parse::<i64>()
                    .map_err(|_| WebhookError::new(ErrorKind::BadRequest, "Invalid webhook-timestamp header"))
                    .map(|unix_seconds| (delivery_id, timestamp, unix_seconds))
            })
            .and_then(
                |(delivery_id, timestamp, unix_seconds)| match now.as_second().abs_diff(unix_seconds) > self.tolerance_seconds {
                    | true => Err(WebhookError::new(
                        ErrorKind::Unauthorized,
                        "Webhook timestamp is outside the allowed window",
                    )),
                    | false => request
                        .headers
                        .first(&["webhook-signature"])
                        .ok_or_else(|| WebhookError::new(ErrorKind::Unauthorized, "Missing webhook-signature header"))
                        .and_then(|signatures| {
                            let prefix = format!("{delivery_id}.{timestamp}.");
                            let message = [prefix.as_bytes(), request.raw_body].concat();
                            match signatures.split_whitespace().any(|part| {
                                part.strip_prefix("v1,").is_some_and(|signature| {
                                    BASE64
                                        .decode(signature.as_bytes())
                                        .is_ok_and(|tag| hmac::verify(&self.key, &message, &tag).is_ok())
                                })
                            }) {
                                | true => Ok(VerifiedRequest {
                                    delivery_id,
                                    raw_body: request.raw_body,
                                }),
                                | false => Err(WebhookError::new(ErrorKind::Unauthorized, "Webhook signature verification failed")),
                            }
                        }),
                },
            )
    }
}
impl core::error::Error for WebhookError {}
impl fmt::Display for WebhookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status_code {
            | Some(status) => write!(formatter, "{} (HTTP {status})", self.message),
            | None => formatter.write_str(self.message),
        }
    }
}
impl WebhookError {
    fn new(kind: ErrorKind, message: &'static str) -> Self {
        Self {
            kind,
            message,
            status_code: None,
        }
    }
    /// Return the stable error category.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
    /// Return the downstream HTTP status, when one was received.
    pub fn status_code(&self) -> Option<u16> {
        self.status_code
    }
}
/// Serialize and dispatch one authenticated HTTPS callback without retries or anonymous fallback.
pub async fn dispatch<T: Serialize + ?Sized>(
    service: &impl HttpService,
    target: &CallbackTarget,
    event: &T,
    callback_headers: &[CallbackHeader],
) -> Result<DispatchResult, WebhookError> {
    match serde_json::to_value(event).map_err(|_| WebhookError::new(ErrorKind::BadRequest, "Failed to encode callback payload")) {
        | Ok(json_body) => {
            let headers = core::iter::once((CONTENT_TYPE, HeaderValue::from_static("application/json")))
                .chain(callback_headers.iter().map(|header| (header.name.clone(), header.value.clone())))
                .collect();
            let request = HttpRequest::init()
                .allow_anonymous_fallback(false)
                .follow_redirects(false)
                .headers(headers)
                .json_body(json_body)
                .max_response_bytes(MAX_CALLBACK_RESPONSE_BYTES)
                .method(HttpMethod::Post)
                .sensitive_url(true)
                .url(target.as_str())
                .build();
            service
                .execute(request)
                .await
                .map_err(|why| match why.downcast_ref::<HttpResponseTooLarge>().is_some() {
                    | true => WebhookError::new(ErrorKind::ResponseTooLarge, "Webhook callback response exceeded its size limit"),
                    | false => WebhookError::new(ErrorKind::Transport, "Webhook callback transport failed"),
                })
                .and_then(|response| match response.status_code {
                    | 200..=299 => Ok(DispatchResult {
                        status_code: response.status_code,
                    }),
                    | status_code => Err(WebhookError {
                        kind: ErrorKind::Downstream,
                        message: "Webhook callback was rejected",
                        status_code: Some(status_code),
                    }),
                })
        }
        | Err(why) => Err(why),
    }
}
/// Authenticate, normalize, and classify one provider request without persistence.
pub fn prepare<P: WebhookProvider>(provider: &P, request: InboundRequest<'_>, now: Timestamp) -> Result<PreparedDelivery<P::Event>, WebhookError> {
    provider.receive(request, now).and_then(|delivery| {
        provider.classify(&delivery).map(|mut operations| {
            operations.sort_by(|left, right| left.idempotency_key.cmp(&right.idempotency_key));
            PreparedDelivery { delivery, operations }
        })
    })
}

#[cfg(test)]
mod tests;
