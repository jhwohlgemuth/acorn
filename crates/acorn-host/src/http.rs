//! Policy-aware asynchronous HTTP transport.
use core::time::Duration;
use futures::StreamExt;
use reqwest::{Method, Response, StatusCode};
use thiserror::Error;
use url::Url;

/// HTTP transport failure with credential-safe diagnostics.
#[derive(Debug, Error)]
pub enum HttpError {
    /// An allow-list entry is malformed.
    #[error("invalid allowed authority: {0}")]
    InvalidAuthority(String),
    /// A request URL is malformed.
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    /// The request authority is not allowed.
    #[error("remote authority is not allowed: {0}")]
    AuthorityDenied(String),
    /// The native HTTP client could not be constructed.
    #[error("failed to construct HTTP client")]
    ClientBuild,
    /// A request failed before a response was available.
    #[error("HTTP request failed for {url}: {kind}")]
    Request {
        /// Redacted request URL.
        url: String,
        /// Stable failure category.
        kind: &'static str,
    },
    /// A response used a non-success status.
    #[error("HTTP {status} for {url}")]
    Status {
        /// HTTP response status.
        status: StatusCode,
        /// Redacted request URL.
        url: String,
    },
    /// A response exceeded its configured byte limit.
    #[error("HTTP response exceeded {maximum} bytes for {url}")]
    ResponseTooLarge {
        /// Maximum response size.
        maximum: usize,
        /// Redacted request URL.
        url: String,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Authority {
    host: String,
    port: Option<u16>,
}
/// Exact normalized remote-authority allow list.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AllowedAuthorities(Vec<Authority>);
impl AllowedAuthorities {
    /// Parse `HOST` and `HOST:PORT` allow-list entries.
    pub fn parse<I, S>(entries: I) -> Result<Self, HttpError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        entries
            .into_iter()
            .map(|entry| parse_authority(entry.as_ref()))
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
    /// Return an unrestricted allow list.
    pub fn unrestricted() -> Self {
        Self::default()
    }
    /// Return whether a URL has an exactly allowed authority.
    pub fn allows(&self, url: &Url) -> bool {
        match (self.0.is_empty(), url.host_str().map(normalize_host)) {
            | (true, _) => true,
            | (false, Some(host)) => self.0.iter().any(|allowed| {
                allowed.host == host
                    && match allowed.port {
                        | Some(port) => url.port_or_known_default() == Some(port),
                        | None => url.port().is_none(),
                    }
            }),
            | (false, None) => false,
        }
    }
}
/// Bounded HTTP transport policy.
#[derive(Clone, Debug)]
pub struct HttpPolicy {
    allowed_authorities: AllowedAuthorities,
    connect_timeout: Duration,
    max_redirects: usize,
    max_response_bytes: usize,
    timeout: Option<Duration>,
    user_agent: String,
}
impl Default for HttpPolicy {
    fn default() -> Self {
        Self {
            allowed_authorities: AllowedAuthorities::unrestricted(),
            connect_timeout: Duration::from_secs(10),
            max_redirects: 5,
            max_response_bytes: 8 * 1024 * 1024,
            timeout: Some(Duration::from_secs(30)),
            user_agent: format!("acorn-host/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}
impl HttpPolicy {
    /// Create the default bounded, unrestricted policy.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the exact allowed authorities.
    pub fn with_allowed_authorities(mut self, allowed: AllowedAuthorities) -> Self {
        self.allowed_authorities = allowed;
        self
    }
    /// Set the connection-establishment timeout.
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }
    /// Set the maximum number of redirects.
    pub fn with_max_redirects(mut self, maximum: usize) -> Self {
        self.max_redirects = maximum;
        self
    }
    /// Set the maximum response body size.
    pub fn with_max_response_bytes(mut self, maximum: usize) -> Self {
        self.max_response_bytes = maximum;
        self
    }
    /// Set the total request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    /// Disable the total request timeout for a caller that enforces its own streaming deadline.
    pub fn without_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }
    /// Set the request user-agent value.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }
}
/// HTTP method used by a successful reachability probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeMethod {
    /// The server accepted a HEAD request.
    Head,
    /// The server required a bounded GET fallback.
    Get,
}
/// Successful bounded reachability probe result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Probe {
    /// Probe method that produced the successful response.
    pub method: ProbeMethod,
    /// Successful response status.
    pub status: StatusCode,
}
/// Reqwest-backed HTTP client with immutable request policy.
#[derive(Clone, Debug)]
pub struct HttpClient {
    client: reqwest::Client,
    policy: HttpPolicy,
}
impl HttpClient {
    /// Construct a policy-aware HTTP client.
    pub fn new(policy: HttpPolicy) -> Result<Self, HttpError> {
        build_native_client(policy.clone(), true).map(|client| Self { client, policy })
    }
    /// Download a bounded response body into memory.
    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, HttpError> {
        match self.parse_and_authorize(url) {
            | Ok(parsed) => {
                let diagnostic = redact_url(parsed.as_str());
                match self.response(Method::GET, parsed).await {
                    | Ok(response) => collect_response_bytes(response, diagnostic, self.policy.max_response_bytes).await,
                    | Err(error) => Err(error),
                }
            }
            | Err(error) => Err(error),
        }
    }
    /// Probe a URL with HEAD and a one-byte GET fallback for unsupported HEAD requests.
    pub async fn probe(&self, url: &str) -> Result<Probe, HttpError> {
        match self.parse_and_authorize(url) {
            | Ok(parsed) => {
                let diagnostic = redact_url(parsed.as_str());
                match self.response(Method::HEAD, parsed.clone()).await {
                    | Ok(response) if response.status().is_success() => Ok(Probe {
                        method: ProbeMethod::Head,
                        status: response.status(),
                    }),
                    | Ok(response) if matches!(response.status(), StatusCode::METHOD_NOT_ALLOWED | StatusCode::NOT_IMPLEMENTED) => {
                        match self.client.get(parsed).header(reqwest::header::RANGE, "bytes=0-0").send().await {
                            | Ok(response) if response.status().is_success() => Ok(Probe {
                                method: ProbeMethod::Get,
                                status: response.status(),
                            }),
                            | Ok(response) => Err(HttpError::Status {
                                status: response.status(),
                                url: diagnostic,
                            }),
                            | Err(error) => Err(request_error(error, &diagnostic)),
                        }
                    }
                    | Ok(response) => Err(HttpError::Status {
                        status: response.status(),
                        url: diagnostic,
                    }),
                    | Err(error) => Err(error),
                }
            }
            | Err(error) => Err(error),
        }
    }
    pub(crate) fn parse_and_authorize(&self, url: &str) -> Result<Url, HttpError> {
        Url::parse(url)
            .map_err(|_| HttpError::InvalidUrl(redact_url(url)))
            .and_then(|parsed| match parsed.scheme() {
                | "http" | "https" if self.policy.allowed_authorities.allows(&parsed) => Ok(parsed),
                | "http" | "https" => Err(HttpError::AuthorityDenied(redact_url(url))),
                | _ => Err(HttpError::InvalidUrl(redact_url(url))),
            })
    }
    pub(crate) async fn response(&self, method: Method, url: Url) -> Result<Response, HttpError> {
        let diagnostic = redact_url(url.as_str());
        self.client
            .request(method, url)
            .send()
            .await
            .map_err(|error| request_error(error, &diagnostic))
    }
}
/// Construct the native client used by higher-level ACORN HTTP services.
pub fn build_native_client(policy: HttpPolicy, follow_redirects: bool) -> Result<reqwest::Client, HttpError> {
    let allowed = policy.allowed_authorities.clone();
    let max_redirects = policy.max_redirects;
    let redirect = match follow_redirects {
        | true => reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.previous().len() >= max_redirects {
                attempt.error("redirect limit exceeded")
            } else if allowed.allows(attempt.url()) {
                attempt.follow()
            } else {
                attempt.error("redirect authority is not allowed")
            }
        }),
        | false => reqwest::redirect::Policy::none(),
    };
    let builder = reqwest::Client::builder()
        .redirect(redirect)
        .connect_timeout(policy.connect_timeout)
        .user_agent(&policy.user_agent);
    match policy.timeout {
        | Some(timeout) => builder.timeout(timeout),
        | None => builder,
    }
    .build()
    .map_err(|_| HttpError::ClientBuild)
}
/// Remove credentials, query parameters, and fragments from a URL diagnostic.
pub fn redact_url(value: &str) -> String {
    Url::parse(value).map_or_else(
        |_| redact_unparsed(value),
        |mut url| {
            let _ = url.set_username("");
            let _ = url.set_password(None);
            url.set_query(None);
            url.set_fragment(None);
            url.to_string()
        },
    )
}
fn normalize_host(host: &str) -> String {
    host.trim_matches(['[', ']']).to_ascii_lowercase()
}
fn parse_authority(value: &str) -> Result<Authority, HttpError> {
    let value = value.trim();
    let invalid = || HttpError::InvalidAuthority(redact_unparsed(value));
    match value.is_empty() || value.contains("://") || value.contains(['/', '?', '#', '@']) {
        | true => Err(invalid()),
        | false => Url::parse(&format!("http://{value}/")).map_err(|_| invalid()).and_then(|parsed| {
            parsed
                .host_str()
                .map(normalize_host)
                .filter(|host| !host.is_empty())
                .ok_or_else(invalid)
                .map(|host| Authority { host, port: parsed.port() })
        }),
    }
}
fn redact_unparsed(value: &str) -> String {
    let bounded = value.split(['?', '#']).next().unwrap_or_default();
    match bounded.split_once("://") {
        | Some((scheme, rest)) => format!("{scheme}://{}", rest.rsplit_once('@').map_or(rest, |(_, safe)| safe)),
        | None => bounded.rsplit_once('@').map_or(bounded, |(_, safe)| safe).to_string(),
    }
}
fn request_error(error: reqwest::Error, url: &str) -> HttpError {
    let kind = if error.is_timeout() {
        "request timed out"
    } else if error.is_redirect() {
        "redirect rejected"
    } else if error.is_connect() {
        "connection failed"
    } else {
        "transport failed"
    };
    HttpError::Request { url: url.to_string(), kind }
}
async fn collect_response_bytes(response: Response, diagnostic: String, maximum: usize) -> Result<Vec<u8>, HttpError> {
    match response.status().is_success() {
        | false => Err(HttpError::Status {
            status: response.status(),
            url: diagnostic,
        }),
        | true => {
            let mut bytes = Vec::new();
            let mut stream = response.bytes_stream();
            loop {
                match stream.next().await {
                    | Some(Ok(chunk)) if bytes.len().saturating_add(chunk.len()) > maximum => {
                        break Err(HttpError::ResponseTooLarge { maximum, url: diagnostic });
                    }
                    | Some(Ok(chunk)) => bytes.extend_from_slice(&chunk),
                    | Some(Err(error)) => break Err(request_error(error, &diagnostic)),
                    | None => break Ok(bytes),
                }
            }
        }
    }
}
