//! Shared HTTP policy utilities for outbound and inbound service middleware
use crate::io::http::HttpMethod;
use crate::util::constants::app::{HTTP_RETRY_INITIAL_BACKOFF_MILLISECONDS, HTTP_RETRY_MAX_BACKOFF_SECONDS};
use axum::http::header::RETRY_AFTER;
use axum::http::HeaderMap;
use jiff::{SignedDuration, Timestamp};
use tower::layer::util::{Identity, Stack};
use tower::timeout::TimeoutLayer;
use tower::ServiceBuilder;

/// Shared HTTP policy configuration for Tower-based middleware stacks.
#[derive(Clone, Copy, Debug)]
pub struct HttpPolicy {
    /// Per-request middleware timeout.
    pub timeout: SignedDuration,
    /// TCP connection timeout.
    pub connect_timeout: SignedDuration,
    /// Number of retry attempts after the first request.
    pub max_retries: usize,
}
impl HttpPolicy {
    /// Returns the total number of attempts including the initial request.
    pub fn max_attempts(self) -> usize {
        self.max_retries.saturating_add(1)
    }
}
impl Default for HttpPolicy {
    fn default() -> Self {
        Self {
            timeout: SignedDuration::from_secs(30),
            connect_timeout: SignedDuration::from_secs(10),
            max_retries: 2,
        }
    }
}
/// Builds the shared HTTP timeout layer.
pub fn http_timeout_layer() -> TimeoutLayer {
    TimeoutLayer::new(shared_http_policy().timeout.unsigned_abs())
}
/// Builds the shared Tower middleware stack for HTTP services.
pub fn http_service_builder() -> ServiceBuilder<Stack<TimeoutLayer, Identity>> {
    ServiceBuilder::new().layer(http_timeout_layer())
}
pub(crate) fn retry_delay(headers: Option<&HeaderMap>, attempt: usize, now: Timestamp) -> SignedDuration {
    let retry_after = headers
        .and_then(|values| values.get(RETRY_AFTER))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value.parse::<i64>().ok().map(SignedDuration::from_secs).or_else(|| {
                httpdate::parse_http_date(value)
                    .ok()
                    .and_then(|retry_at| Timestamp::try_from(retry_at).ok())
                    .map(|retry_at| retry_at.duration_since(now).max(SignedDuration::ZERO))
            })
        });
    let exponent = u32::try_from(attempt.saturating_sub(1)).unwrap_or(u32::MAX).min(8);
    let fallback = SignedDuration::from_millis(HTTP_RETRY_INITIAL_BACKOFF_MILLISECONDS).saturating_mul(2_i32.saturating_pow(exponent));
    retry_after
        .unwrap_or(fallback)
        .min(SignedDuration::from_secs(HTTP_RETRY_MAX_BACKOFF_SECONDS))
}
/// Creates the shared HTTP policy used by IO services.
pub fn shared_http_policy() -> HttpPolicy {
    HttpPolicy::default()
}
/// Returns true when a request should be retried for the given method and status.
pub fn should_retry(method: &HttpMethod, status_code: Option<u16>) -> bool {
    if !matches!(method, HttpMethod::Get) {
        false
    } else {
        match status_code {
            | Some(code) if code == 408 || code == 429 || code >= 500 => true,
            | Some(_) => false,
            | None => true,
        }
    }
}
