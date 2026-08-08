//! Shared HTTP policy utilities for outbound and inbound service middleware
use crate::io::http::HttpMethod;
use core::time::Duration;
use tower::layer::util::{Identity, Stack};
use tower::timeout::TimeoutLayer;
use tower::ServiceBuilder;

/// Shared HTTP policy configuration for Tower-based middleware stacks.
#[derive(Clone, Copy, Debug)]
pub struct HttpPolicy {
    /// Per-request middleware timeout.
    pub timeout: Duration,
    /// TCP connection timeout.
    pub connect_timeout: Duration,
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
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_retries: 2,
        }
    }
}
/// Builds the shared HTTP timeout layer.
pub fn http_timeout_layer() -> TimeoutLayer {
    TimeoutLayer::new(shared_http_policy().timeout)
}
/// Builds the shared Tower middleware stack for HTTP services.
pub fn http_service_builder() -> ServiceBuilder<Stack<TimeoutLayer, Identity>> {
    ServiceBuilder::new().layer(http_timeout_layer())
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
