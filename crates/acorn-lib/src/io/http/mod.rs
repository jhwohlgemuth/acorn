//! HTTP client utilities for ACORN IO operations
use super::ApiResult;
use crate::util::constants::app::ACORN_USER_AGENT;
use acorn_host::{
    http::{build_native_client, HttpPolicy as HostHttpPolicy},
    terminal::Label,
};
use async_trait::async_trait;
use bon::Builder;
use color_eyre::eyre::eyre;
use core::{borrow::Borrow, fmt};
use http::{
    header::{HeaderName, HeaderValue, ACCEPT_RANGES, AUTHORIZATION, RANGE, USER_AGENT},
    HeaderMap,
};
use jiff::Timestamp;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};
use tokio::time::{sleep, timeout};
use tower::{service_fn, ServiceExt};
use tracing::{debug, warn};

pub mod policy;
/// Ordered UTF-8 header lookup helpers.
pub trait HeaderMapExt {
    /// Return the first named header whose value is valid UTF-8.
    fn first<'a>(&'a self, names: &[&str]) -> Option<&'a str>
    where
        Self: Borrow<HeaderMap>,
    {
        names
            .iter()
            .find_map(|name| <Self as Borrow<HeaderMap>>::borrow(self).get(*name).and_then(|value| value.to_str().ok()))
    }
}
/// Async HTTP service abstraction for ACORN I/O.
#[async_trait]
pub trait HttpService {
    /// Execute an HTTP request.
    async fn execute(&self, request: HttpRequest) -> ApiResult<HttpResponse>;
}
/// Supported HTTP methods for ACORN I/O requests.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    /// HTTP GET
    #[default]
    Get,
    /// HTTP DELETE
    Delete,
    /// HTTP PATCH
    Patch,
    /// HTTP POST
    Post,
    /// HTTP PUT
    Put,
}
#[derive(Builder)]
#[builder(start_fn = init)]
struct DownloadError {
    status_code: Option<u16>,
    retry_headers: Option<HeaderMap>,
    report: color_eyre::Report,
}
/// Internal HTTP request representation.
#[derive(Builder, Clone)]
#[builder(builder_type = HttpRequestInit, start_fn = init, on(String, into))]
pub struct HttpRequest {
    /// Whether rejected credentials may be retried without authentication.
    #[builder(default = true)]
    pub allow_anonymous_fallback: bool,
    /// Whether HTTP redirects may be followed.
    #[builder(default = true)]
    pub follow_redirects: bool,
    /// Optional exact request body bytes.
    pub body: Option<Vec<u8>>,
    /// Request headers
    #[builder(default)]
    pub headers: HeaderMap,
    /// Optional JSON body
    pub json_body: Option<serde_json::Value>,
    /// Maximum accepted response body size, when bounded.
    pub max_response_bytes: Option<usize>,
    /// HTTP method
    pub method: HttpMethod,
    /// Whether diagnostics must redact the URL path and query.
    #[builder(default)]
    pub sensitive_url: bool,
    /// URL string
    pub url: String,
}
/// Fluent async request builder backed by [`HttpService`].
#[derive(Clone, Debug)]
pub struct HttpRequestBuilder {
    request: HttpRequest,
    service: ReqwestHttpService,
}
/// Internal HTTP response representation.
#[derive(Clone, Debug)]
pub struct HttpResponse {
    /// Response body bytes
    pub body: Vec<u8>,
    /// Response headers
    pub headers: HeaderMap,
    /// HTTP status code
    pub status_code: u16,
}
/// Error returned when an HTTP response has a non-success status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpResponseError {
    status_code: u16,
    reason: String,
}
/// Error returned when a response body exceeds its request-level byte limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HttpResponseTooLarge {
    maximum: usize,
}
/// Reqwest-backed async HTTP service adapter.
#[derive(Clone, Debug)]
pub struct ReqwestHttpService {
    client: reqwest::Client,
}
impl HeaderMapExt for HeaderMap {}
impl From<&str> for HttpMethod {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            | "DELETE" => HttpMethod::Delete,
            | "PATCH" => HttpMethod::Patch,
            | "POST" => HttpMethod::Post,
            | "PUT" => HttpMethod::Put,
            | _ => HttpMethod::Get,
        }
    }
}
impl fmt::Debug for HttpRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HttpRequest")
            .field("allow_anonymous_fallback", &self.allow_anonymous_fallback)
            .field("follow_redirects", &self.follow_redirects)
            .field("body_length", &self.body.as_ref().map(Vec::len))
            .field("headers", &self.headers)
            .field("json_body", &self.json_body)
            .field("max_response_bytes", &self.max_response_bytes)
            .field("method", &self.method)
            .field("sensitive_url", &self.sensitive_url)
            .field("url", &self.diagnostic_url())
            .finish()
    }
}
impl HttpRequest {
    fn diagnostic_url(&self) -> &str {
        match self.sensitive_url {
            | true => "[REDACTED WEBHOOK URL]",
            | false => &self.url,
        }
    }
}
impl fmt::Display for HttpRequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.request.method, self.request.diagnostic_url())
    }
}
impl HttpRequestBuilder {
    /// Add exact request body bytes.
    pub fn body(mut self, value: impl Into<Vec<u8>>) -> Self {
        self.request.body = Some(value.into());
        self
    }
    /// Add a single header to the request, ignoring invalid names or values.
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.request.headers.extend(header(name, value));
        self
    }
    /// Add headers to the request.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.request.headers.extend(headers);
        self
    }
    /// Add a Bearer `Authorization` header to the request when `token` is non-empty.
    pub fn bearer_auth(mut self, token: &str) -> Self {
        self.request.headers.extend(bearer_auth(token));
        self
    }
    /// Add a JSON body to the request.
    pub fn json(mut self, value: &serde_json::Value) -> Self {
        self.request.json_body = Some(value.clone());
        self
    }
    fn new(method: HttpMethod, url: impl Into<String>) -> Self {
        Self {
            request: HttpRequest::init().method(method).url(url).build(),
            service: ReqwestHttpService::default(),
        }
    }
    /// Send the request.
    pub async fn send(self) -> ApiResult<HttpResponse> {
        self.service.execute(self.request).await
    }
}
impl HttpResponse {
    /// Return this response when its status is successful.
    pub fn success(self, action: &str) -> ApiResult<Self> {
        match (200..=299).contains(&self.status_code) {
            | true => Ok(self),
            | false => Err(eyre!("Failed to {action} — HTTP {}", self.status_code)),
        }
    }
    /// Read response body as bytes.
    pub async fn bytes(self) -> ApiResult<Vec<u8>> {
        Ok(self.body)
    }
    /// Read response body as text.
    pub async fn text(self) -> ApiResult<String> {
        match String::from_utf8(self.body) {
            | Ok(value) => Ok(value),
            | Err(why) => Err(eyre!("HTTP response body is not valid UTF-8 — {why}")),
        }
    }
}
impl core::error::Error for HttpResponseError {}
impl fmt::Display for HttpResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "HTTP {}: {}", self.status_code, self.reason)
    }
}
impl HttpResponseError {
    /// Create an HTTP response error from its status and response detail.
    pub fn new(status_code: u16, reason: impl Into<String>) -> Self {
        Self {
            status_code,
            reason: reason.into(),
        }
    }
    /// Return the HTTP status code.
    pub fn status_code(&self) -> u16 {
        self.status_code
    }
}
impl core::error::Error for HttpResponseTooLarge {}
impl fmt::Display for HttpResponseTooLarge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "HTTP response exceeded its configured {}-byte limit", self.maximum)
    }
}
impl From<HttpMethod> for reqwest::Method {
    fn from(value: HttpMethod) -> Self {
        match value {
            | HttpMethod::Delete => reqwest::Method::DELETE,
            | HttpMethod::Get => reqwest::Method::GET,
            | HttpMethod::Patch => reqwest::Method::PATCH,
            | HttpMethod::Post => reqwest::Method::POST,
            | HttpMethod::Put => reqwest::Method::PUT,
        }
    }
}
impl Default for ReqwestHttpService {
    fn default() -> Self {
        let client = build_native_client(host_http_policy(), true).unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }
}
#[async_trait]
impl HttpService for ReqwestHttpService {
    async fn execute(&self, request: HttpRequest) -> ApiResult<HttpResponse> {
        match request.follow_redirects {
            | true => execute_with_policy(self.client.clone(), request).await,
            | false => match no_redirect_client() {
                | Ok(client) => execute_with_policy(client, request).await,
                | Err(why) => Err(why),
            },
        }
    }
}
impl ReqwestHttpService {
    fn loopback() -> ApiResult<Self> {
        build_native_client(host_http_policy(), false)
            .map(|client| Self { client })
            .map_err(|why| eyre!("Failed to construct loopback HTTP client — {why}"))
    }
    fn streaming() -> Self {
        let client = build_native_client(host_http_policy().without_timeout(), true).unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }
}
fn anonymous_request(request: &HttpRequest) -> Option<HttpRequest> {
    let headers = request
        .headers
        .iter()
        .filter(|(name, value)| !(value.is_sensitive() && is_credential_header(name)))
        .fold(HeaderMap::new(), |mut headers, (name, value)| {
            headers.append(name.clone(), value.clone());
            headers
        });
    (headers.len() < request.headers.len()).then(|| HttpRequest { headers, ..request.clone() })
}
/// Build a Bearer authorization header when `token` is non-empty.
pub fn bearer_auth(token: &str) -> HeaderMap {
    match token.trim() {
        | "" => HeaderMap::new(),
        | value => header(AUTHORIZATION.as_str(), format!("Bearer {value}").as_str()),
    }
}
/// Utility method to employ best practices when making async HTTP DELETE requests.
pub fn delete(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Delete, url)
}
/// Streams response bytes from `url` to `output` and reports cumulative progress
pub async fn download_with_progress(
    url: &str,
    output: &Path,
    mut progress: impl FnMut(u64, Option<u64>),
    headers: Option<HeaderMap>,
    error_message: Option<&str>,
    auth_error_message: Option<&str>,
    resume_from: Option<u64>,
) -> ApiResult<()> {
    let service = ReqwestHttpService::streaming();
    let request = HttpRequest::init().maybe_headers(headers).method(HttpMethod::Get).url(url).build();
    let policy = stream_request_with_policy(
        service.client,
        request,
        output,
        &mut progress,
        resume_from,
        error_message,
        auth_error_message,
    );
    match policy.await {
        | Ok(()) => Ok(()),
        | Err(why) => Err(why),
    }
}
async fn execute_with_policy(client: reqwest::Client, request: HttpRequest) -> ApiResult<HttpResponse> {
    let anonymous = request.allow_anonymous_fallback.then(|| anonymous_request(&request)).flatten();
    let result = execute_with_policy_attempts(client.clone(), request).await;
    match (result, anonymous) {
        | (Ok(response), Some(request)) if matches!(response.status_code, 401 | 403) => {
            let credential_status = response.status_code;
            warn!(
                status_code = credential_status,
                url = request.diagnostic_url(),
                "=> {} Configured HTTP credentials were rejected; retrying without authentication",
                Label::using()
            );
            match execute_with_policy_attempts(client, request).await {
                | Ok(response) if matches!(response.status_code, 401 | 403) => Err(eyre!(
                    "Configured HTTP credentials were rejected (HTTP {credential_status}); anonymous fallback also failed (HTTP {})",
                    response.status_code
                )),
                | Err(why) => Err(eyre!(
                    "Configured HTTP credentials were rejected (HTTP {credential_status}); anonymous fallback also failed — {why}"
                )),
                | result => result,
            }
        }
        | (result, _) => result,
    }
}
async fn execute_with_policy_attempts(client: reqwest::Client, request: HttpRequest) -> ApiResult<HttpResponse> {
    let policy = policy::shared_http_policy();
    let method = request.method.clone();
    let url = request.diagnostic_url().to_string();
    let max_attempts = policy.max_attempts();
    for attempt in 1..=max_attempts {
        let started = Timestamp::now();
        let result = execute_with_timeout(client.clone(), request.clone()).await;
        let elapsed_ms = Timestamp::now().duration_since(started).as_millis();
        match result {
            | Ok(response) => {
                let retry = should_retry(&method, Some(response.status_code));
                if retry && attempt < max_attempts {
                    let delay = policy::retry_delay(Some(&response.headers), attempt, Timestamp::now());
                    warn!(
                        attempt,
                        status_code = response.status_code,
                        elapsed_ms,
                        retry_after_ms = delay.as_millis(),
                        url,
                        "=> {} Retrying HTTP request",
                        Label::using()
                    );
                    sleep(delay.unsigned_abs()).await;
                } else {
                    debug!(
                        attempt,
                        status_code = response.status_code,
                        elapsed_ms,
                        url,
                        "=> {} HTTP request",
                        Label::using()
                    );
                    return Ok(response);
                }
            }
            | Err(why) => {
                let retry = should_retry(&method, None);
                if retry && attempt < max_attempts {
                    let delay = policy::retry_delay(None, attempt, Timestamp::now());
                    warn!(
                        attempt,
                        elapsed_ms,
                        retry_after_ms = delay.as_millis(),
                        url,
                        "=> {} Retrying HTTP request — {why}",
                        Label::using()
                    );
                    sleep(delay.unsigned_abs()).await;
                } else {
                    warn!(attempt, elapsed_ms, url, "=> {} HTTP request failed — {why}", Label::fail());
                    return Err(why);
                }
            }
        }
    }
    Err(eyre!("HTTP request failed after retry attempts"))
}
async fn execute_with_timeout(client: reqwest::Client, request: HttpRequest) -> ApiResult<HttpResponse> {
    let service = policy::http_service_builder().service(service_fn(move |value: HttpRequest| {
        let client = client.clone();
        async move { invoke_request(client, value).await }
    }));
    service
        .oneshot(request)
        .await
        .map_err(|why| eyre!("HTTP service timeout or middleware error — {why}"))
}
/// Utility method to employ best practices when making async HTTP GET requests.
pub fn get(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Get, url)
}
/// Build a one-entry header map, skipping invalid names or values.
pub fn header(name: &str, value: &str) -> HeaderMap {
    headers([(name, value)])
}
/// Build a header map from string name-value pairs, skipping invalid entries.
pub fn headers<'a>(values: impl IntoIterator<Item = (&'a str, &'a str)>) -> HeaderMap {
    values.into_iter().fold(HeaderMap::new(), |mut headers, (name, value)| {
        if let (Ok(name), Ok(mut value)) = (HeaderName::from_bytes(name.as_bytes()), HeaderValue::from_str(value)) {
            value.set_sensitive(is_credential_header(&name));
            headers.append(name, value);
        }
        headers
    })
}
async fn invoke_request(client: reqwest::Client, request: HttpRequest) -> ApiResult<HttpResponse> {
    let HttpRequest {
        body,
        headers,
        json_body,
        max_response_bytes,
        method,
        sensitive_url,
        url,
        ..
    } = request;
    let builder = client.request(method.into(), url).header(USER_AGENT, ACORN_USER_AGENT).headers(headers);
    let builder = match (body, json_body) {
        | (Some(value), _) => builder.body(value),
        | (None, Some(value)) => builder.json(&value),
        | (None, None) => builder,
    };
    match builder.send().await {
        | Ok(mut response) => {
            let status_code = response.status().as_u16();
            let headers = response.headers().clone();
            let oversized = max_response_bytes
                .zip(response.content_length())
                .and_then(|(maximum, length)| u64::try_from(maximum).is_ok_and(|maximum| length > maximum).then_some(maximum));
            match oversized {
                | Some(maximum) => Err(eyre!(HttpResponseTooLarge { maximum })),
                | None => {
                    let mut body = Vec::new();
                    loop {
                        match response.chunk().await {
                            | Ok(Some(chunk)) => match max_response_bytes.filter(|maximum| body.len().saturating_add(chunk.len()) > *maximum) {
                                | Some(maximum) => break Err(eyre!(HttpResponseTooLarge { maximum })),
                                | None => body.extend_from_slice(&chunk),
                            },
                            | Ok(None) => break Ok(HttpResponse { body, headers, status_code }),
                            | Err(_why) if sensitive_url => break Err(eyre!("HTTP response body transport failed")),
                            | Err(why) => break Err(eyre!(why)),
                        }
                    }
                }
            }
        }
        | Err(_why) if sensitive_url => Err(eyre!("HTTP request transport failed")),
        | Err(why) => Err(eyre!(why)),
    }
}
fn is_credential_header(name: &HeaderName) -> bool {
    let name = name.as_str();
    name == AUTHORIZATION.as_str() || name.ends_with("-token") || name.ends_with("-key") || name == "apikey"
}
/// Build an HTTP POST request that refuses redirects from a validated loopback endpoint.
pub fn loopback_post(url: impl Into<String>) -> ApiResult<HttpRequestBuilder> {
    ReqwestHttpService::loopback().map(|service| HttpRequestBuilder {
        service,
        ..HttpRequestBuilder::new(HttpMethod::Post, url)
    })
}
fn no_redirect_client() -> ApiResult<reqwest::Client> {
    build_native_client(host_http_policy(), false).map_err(|_| eyre!("Failed to construct restricted HTTP client"))
}
fn host_http_policy() -> HostHttpPolicy {
    let policy = policy::shared_http_policy();
    HostHttpPolicy::new()
        .with_connect_timeout(policy.connect_timeout.unsigned_abs())
        .with_timeout(policy.timeout.unsigned_abs())
        .with_user_agent(ACORN_USER_AGENT)
}
/// Utility method to employ best practices when making async HTTP PATCH requests.
pub fn patch(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Patch, url)
}
/// Utility method to employ best practices when making async HTTP POST requests.
pub fn post(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Post, url)
}
/// Utility method to employ best practices when making async HTTP PUT requests.
pub fn put(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Put, url)
}
/// Build an HTTP request for a dynamic method.
pub fn request(method: HttpMethod, url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(method, url)
}
/// Reads response bytes or converts request, status, and body errors into a contextual error.
pub async fn response_body_bytes(response: ApiResult<HttpResponse>, error_message: &str) -> ApiResult<Vec<u8>> {
    match response {
        | Ok(value) => match value.status_code {
            | 200..=299 => value.bytes().await.map_err(|why| eyre!("{error_message} — {why}")),
            | status => Err(eyre!("{error_message} — HTTP {status}")),
        },
        | Err(why) => Err(eyre!("{error_message} — {why}")),
    }
}
pub(crate) fn should_retry(method: &HttpMethod, status_code: Option<u16>) -> bool {
    policy::should_retry(method, status_code)
}
async fn stream_request_attempts(
    client: reqwest::Client,
    request: HttpRequest,
    output: &Path,
    progress: &mut impl FnMut(u64, Option<u64>),
    resume_from: Option<u64>,
    error_message: Option<&str>,
    auth_error_message: Option<&str>,
) -> Result<(), DownloadError> {
    let policy = policy::shared_http_policy();
    let method = request.method.clone();
    let url = request.url.clone();
    let max_attempts = policy.max_attempts();
    let mut outcome = None;
    for attempt in 1..=max_attempts {
        let started = Timestamp::now();
        let attempt_resume_from = resume_from.and_then(|_| output.metadata().ok().map(|metadata| metadata.len()).filter(|size| *size > 0));
        let result = stream_request_once(
            client.clone(),
            request.clone(),
            output,
            &mut *progress,
            attempt_resume_from,
            error_message,
            auth_error_message,
        )
        .await;
        let elapsed_ms = Timestamp::now().duration_since(started).as_millis();
        match result {
            | Ok(()) => {
                debug!(attempt, elapsed_ms, url, "=> {} HTTP download", Label::using());
                outcome = Some(Ok(()));
                break;
            }
            | Err(DownloadError {
                status_code,
                retry_headers,
                report,
            }) => {
                let retry = should_retry(&method, status_code);
                if retry && attempt < max_attempts {
                    let delay = policy::retry_delay(retry_headers.as_ref(), attempt, Timestamp::now());
                    warn!(
                        attempt,
                        elapsed_ms,
                        retry_after_ms = delay.as_millis(),
                        url,
                        "=> {} Retrying HTTP download — {report}",
                        Label::using()
                    );
                    sleep(delay.unsigned_abs()).await;
                } else {
                    outcome = Some(Err(DownloadError {
                        status_code,
                        retry_headers,
                        report,
                    }));
                    break;
                }
            }
        }
    }
    match outcome {
        | Some(result) => result,
        | None => Err(DownloadError::init().report(eyre!("HTTP download failed after retry attempts")).build()),
    }
}
async fn stream_request_once(
    client: reqwest::Client,
    request: HttpRequest,
    output: &Path,
    progress: &mut impl FnMut(u64, Option<u64>),
    resume_from: Option<u64>,
    error_message: Option<&str>,
    auth_error_message: Option<&str>,
) -> Result<(), DownloadError> {
    let error_message = error_message.unwrap_or("Failed to download file");
    let HttpRequest { headers, url, .. } = request;
    let mut request = client.get(url).header(USER_AGENT, ACORN_USER_AGENT).headers(headers);
    if let Some(offset) = resume_from {
        request = request.header(RANGE, format!("bytes={offset}-"));
    }
    match request.send().await {
        | Ok(response) if matches!(response.status().as_u16(), 401 | 403) => {
            let status_code = response.status().as_u16();
            Err(DownloadError::init()
                .status_code(status_code)
                .retry_headers(response.headers().clone())
                .report(eyre!(
                    "{}",
                    auth_error_message.unwrap_or("Failed to download file — authentication required")
                ))
                .build())
        }
        | Ok(mut response) if response.status().is_success() => {
            let append = response.status().as_u16() == 206 && resume_from.is_some();
            let file_result = if append {
                OpenOptions::new().create(true).append(true).open(output)
            } else {
                File::create(output)
            };
            match file_result {
                | Ok(mut file) => {
                    let resumed = if append { resume_from.unwrap_or_default() } else { 0 };
                    let total = response.content_length().map(|value| value.saturating_add(resumed));
                    let mut downloaded = resumed;
                    let stall_timeout = policy::shared_http_policy().stall_timeout.unsigned_abs();
                    progress(downloaded, total);
                    loop {
                        match timeout(stall_timeout, response.chunk()).await {
                            | Ok(Ok(Some(chunk))) => match file.write_all(&chunk) {
                                | Ok(_) => {
                                    downloaded = downloaded.saturating_add(chunk.len() as u64);
                                    progress(downloaded, total);
                                }
                                | Err(why) => break Err(eyre!("{error_message} — failed to write download chunk — {why}")),
                            },
                            | Ok(Ok(None)) => break Ok(()),
                            | Ok(Err(why)) => break Err(eyre!("{error_message} — {why}")),
                            | Err(_) => {
                                break Err(eyre!(
                                    "{error_message} — download stalled for {} seconds without new bytes",
                                    stall_timeout.as_secs()
                                ))
                            }
                        }
                    }
                }
                | Err(why) => Err(eyre!("{error_message} — failed to create output file {} — {why}", output.display())),
            }
            .map_err(|report| DownloadError::init().report(report).build())
        }
        | Ok(response) => {
            let status = response.status();
            Err(DownloadError::init()
                .status_code(status.as_u16())
                .retry_headers(response.headers().clone())
                .report(eyre!("{error_message} — HTTP {status}"))
                .build())
        }
        | Err(why) => Err(DownloadError::init().report(eyre!("{error_message} — {why}")).build()),
    }
}
async fn stream_request_with_policy(
    client: reqwest::Client,
    request: HttpRequest,
    output: &Path,
    progress: &mut impl FnMut(u64, Option<u64>),
    resume_from: Option<u64>,
    error_message: Option<&str>,
    auth_error_message: Option<&str>,
) -> ApiResult<()> {
    let anonymous = anonymous_request(&request);
    let url = request.url.clone();
    let result = stream_request_attempts(client.clone(), request, output, progress, resume_from, error_message, auth_error_message).await;
    match (result, anonymous) {
        | (
            Err(DownloadError {
                status_code: Some(status_code @ (401 | 403)),
                report,
                ..
            }),
            Some(request),
        ) => {
            let follow_up_action = "(retrying download without authentication)".dimmed();
            warn!(status_code, url, "=> {} Rejected HTTP credentials {follow_up_action}", Label::using(),);
            match stream_request_attempts(client, request, output, progress, resume_from, error_message, None).await {
                | Ok(()) => Ok(()),
                | Err(anonymous_error) => {
                    let combined = eyre!("{report}; anonymous fallback also failed — {}", anonymous_error.report);
                    warn!(url, "=> {} HTTP download failed — {combined}", Label::fail());
                    Err(combined)
                }
            }
        }
        | (Ok(()), _) => Ok(()),
        | (Err(why), _) => {
            warn!(url, "=> {} HTTP download — {}", Label::fail(), why.report);
            Err(why.report)
        }
    }
}
/// Check if a URL advertises byte-range support via `Accept-Ranges: bytes`.
pub async fn supports_byte_ranges(url: &str, headers: Option<HeaderMap>) -> ApiResult<bool> {
    let service = ReqwestHttpService::streaming();
    let request = service
        .client
        .head(url)
        .header(USER_AGENT, ACORN_USER_AGENT)
        .headers(headers.unwrap_or_default());
    match request.send().await {
        | Ok(response) => Ok(response
            .headers()
            .get(ACCEPT_RANGES)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.eq_ignore_ascii_case("bytes"))),
        | Err(why) => Err(eyre!("Failed to probe HTTP range support — {why}")),
    }
}
#[cfg(test)]
mod tests;
