//! Provider-neutral webhook verification and callback transport contracts
use crate::io::http::{HeaderMapExt, HttpMethod, HttpRequest, HttpResponseTooLarge, HttpService};
use crate::io::ApiResult;
use crate::util::constants::app::MAX_CALLBACK_RESPONSE_BYTES;
use acorn_core::prelude::{Box, String, Vec};
use acorn_core::util::constant_time_eq;
use alloc::sync::Arc;
use core::{fmt, future::Future, pin::Pin};
use data_encoding::BASE64;
use http::header::{HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use http::HeaderMap;
use jiff::Timestamp;
use ring::hmac;
use serde::{Deserialize, Serialize};

/// Awaitable result produced by a durable webhook operation.
pub type Operation = Pin<Box<dyn Future<Output = ApiResult<()>> + Send>>;
/// Callback used by durable webhook workers.
pub type OperationHandler<T> = Arc<dyn Fn(Delivery<T>) -> Operation + Send + Sync + 'static>;
/// Authentication contract for inbound webhook requests.
pub trait Verifier {
    /// Verify exact request bytes using an explicit clock value.
    fn verify<'a>(&self, request: InboundRequest<'a>, now: Timestamp) -> Result<VerifiedRequest<'a>, WebhookError>;
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
    secret: String,
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
    pub fn new(secret_header: &str, secret: impl Into<String>, delivery_headers: &[&str]) -> Result<Self, WebhookError> {
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
                match secret.is_empty() {
                    | true => Err(WebhookError::new(ErrorKind::Unauthorized, "Invalid webhook authentication secret")),
                    | false => Ok(Self {
                        delivery_headers,
                        secret,
                        secret_header,
                    }),
                }
            })
    }
}
impl Verifier for HeaderSecretVerifier {
    fn verify<'a>(&self, request: InboundRequest<'a>, _now: Timestamp) -> Result<VerifiedRequest<'a>, WebhookError> {
        request
            .headers
            .get(&self.secret_header)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| WebhookError::new(ErrorKind::Unauthorized, "Missing webhook authentication header"))
            .and_then(|received| match constant_time_eq(self.secret.as_bytes(), received.as_bytes()) {
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
            })
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
    pub fn new(signing_secret: &str, tolerance_seconds: u64) -> Result<Self, WebhookError> {
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
#[cfg(test)]
mod tests;
