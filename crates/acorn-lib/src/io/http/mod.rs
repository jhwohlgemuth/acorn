//! HTTP client utilities for ACORN IO operations
use super::ApiResult;
use crate::prelude::{File, OpenOptions, Path, Write};
use crate::util::constants::app::ACORN_USER_AGENT;
use crate::util::Label;
use async_trait::async_trait;
use axum::http::header::{HeaderName, HeaderValue, ACCEPT_RANGES, AUTHORIZATION, RANGE, USER_AGENT};
use axum::http::HeaderMap;
use color_eyre::eyre::eyre;
use core::fmt;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use tower::{service_fn, ServiceExt};
use tracing::{debug, warn};

pub mod policy;

struct DownloadError {
    status_code: Option<u16>,
    report: color_eyre::Report,
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
/// Internal HTTP request representation.
#[derive(Clone, Debug)]
pub struct HttpRequest {
    /// Request headers
    pub headers: HeaderMap,
    /// Optional JSON body
    pub json_body: Option<serde_json::Value>,
    /// HTTP method
    pub method: HttpMethod,
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
/// Reqwest-backed async HTTP service adapter.
#[derive(Clone, Debug)]
pub struct ReqwestHttpService {
    client: reqwest::Client,
}
impl Default for ReqwestHttpService {
    fn default() -> Self {
        let policy = policy::shared_http_policy();
        let client = reqwest::Client::builder()
            .timeout(policy.timeout)
            .connect_timeout(policy.connect_timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }
}
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
impl fmt::Display for HttpRequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.request.method, self.request.url)
    }
}
impl HttpRequestBuilder {
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
            request: HttpRequest {
                headers: HeaderMap::new(),
                json_body: None,
                method,
                url: url.into(),
            },
            service: ReqwestHttpService::default(),
        }
    }
    /// Send the request.
    pub async fn send(self) -> ApiResult<HttpResponse> {
        self.service.execute(self.request).await
    }
}
impl HttpResponse {
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
#[async_trait]
impl HttpService for ReqwestHttpService {
    async fn execute(&self, request: HttpRequest) -> ApiResult<HttpResponse> {
        execute_with_policy(self.client.clone(), request).await
    }
}
impl ReqwestHttpService {
    fn streaming() -> Self {
        let policy = policy::shared_http_policy();
        let client = reqwest::Client::builder()
            .connect_timeout(policy.connect_timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }
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
    let request = HttpRequest {
        headers: headers.unwrap_or_default(),
        json_body: None,
        method: HttpMethod::Get,
        url: url.to_string(),
    };
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
async fn stream_request_with_policy(
    client: reqwest::Client,
    request: HttpRequest,
    output: &Path,
    progress: &mut impl FnMut(u64, Option<u64>),
    resume_from: Option<u64>,
    error_message: Option<&str>,
    auth_error_message: Option<&str>,
) -> ApiResult<()> {
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
            | Err(DownloadError { status_code, report }) => {
                let retry = should_retry(&method, status_code);
                if retry && attempt < max_attempts {
                    warn!(attempt, elapsed_ms, url, "=> {} Retrying HTTP download — {report}", Label::using());
                } else {
                    warn!(attempt, elapsed_ms, url, "=> {} HTTP download failed — {report}", Label::fail());
                    outcome = Some(Err(report));
                    break;
                }
            }
        }
    }
    match outcome {
        | Some(result) => result,
        | None => Err(eyre!("HTTP download failed after retry attempts")),
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
            Err(DownloadError {
                status_code: Some(status_code),
                report: eyre!("{}", auth_error_message.unwrap_or("Failed to download file — authentication required")),
            })
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
                    progress(downloaded, total);
                    loop {
                        match response.chunk().await {
                            | Ok(Some(chunk)) => match file.write_all(&chunk) {
                                | Ok(_) => {
                                    downloaded = downloaded.saturating_add(chunk.len() as u64);
                                    progress(downloaded, total);
                                }
                                | Err(why) => break Err(eyre!("{error_message} — failed to write download chunk — {why}")),
                            },
                            | Ok(None) => break Ok(()),
                            | Err(why) => break Err(eyre!("{error_message} — {why}")),
                        }
                    }
                }
                | Err(why) => Err(eyre!("{error_message} — failed to create output file {} — {why}", output.display())),
            }
            .map_err(|report| DownloadError { status_code: None, report })
        }
        | Ok(response) => Err(DownloadError {
            status_code: Some(response.status().as_u16()),
            report: eyre!("{error_message} — HTTP {}", response.status()),
        }),
        | Err(why) => Err(DownloadError {
            status_code: None,
            report: eyre!("{error_message} — {why}"),
        }),
    }
}
async fn execute_with_policy(client: reqwest::Client, request: HttpRequest) -> ApiResult<HttpResponse> {
    let policy = policy::shared_http_policy();
    let method = request.method.clone();
    let url = request.url.clone();
    let max_attempts = policy.max_attempts();
    for attempt in 1..=max_attempts {
        let started = Timestamp::now();
        let result = execute_with_timeout(client.clone(), request.clone()).await;
        let elapsed_ms = Timestamp::now().duration_since(started).as_millis();
        match result {
            | Ok(response) => {
                let retry = should_retry(&method, Some(response.status_code));
                if retry && attempt < max_attempts {
                    warn!(
                        attempt,
                        status_code = response.status_code,
                        elapsed_ms,
                        url,
                        "=> {} Retrying HTTP request",
                        Label::using()
                    );
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
                    warn!(attempt, elapsed_ms, url, "=> {} Retrying HTTP request — {why}", Label::using());
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
/// Build a header map from string name-value pairs, skipping invalid entries.
pub fn headers<'a>(values: impl IntoIterator<Item = (&'a str, &'a str)>) -> HeaderMap {
    values.into_iter().fold(HeaderMap::new(), |mut headers, (name, value)| {
        if let (Ok(name), Ok(mut value)) = (HeaderName::from_bytes(name.as_bytes()), HeaderValue::from_str(value)) {
            value.set_sensitive(name == AUTHORIZATION);
            headers.append(name, value);
        }
        headers
    })
}
/// Build a one-entry header map, skipping invalid names or values.
pub fn header(name: &str, value: &str) -> HeaderMap {
    headers([(name, value)])
}
/// Build a Bearer authorization header when `token` is non-empty.
pub fn bearer_auth(token: &str) -> HeaderMap {
    match token.trim() {
        | "" => HeaderMap::new(),
        | value => header(AUTHORIZATION.as_str(), format!("Bearer {value}").as_str()),
    }
}
/// Build an HTTP request for a dynamic method.
pub fn request(method: HttpMethod, url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(method, url)
}
/// Utility method to employ best practices when making async HTTP GET requests.
pub fn get(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Get, url)
}
/// Utility method to employ best practices when making async HTTP DELETE requests.
pub fn delete(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Delete, url)
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
async fn invoke_request(client: reqwest::Client, request: HttpRequest) -> ApiResult<HttpResponse> {
    let HttpRequest {
        headers,
        json_body,
        method,
        url,
    } = request;
    let builder = client.request(method.into(), url).header(USER_AGENT, ACORN_USER_AGENT).headers(headers);
    let builder = match json_body {
        | Some(value) => builder.json(&value),
        | None => builder,
    };
    match builder.send().await {
        | Ok(response) => {
            let status_code = response.status().as_u16();
            let headers = response.headers().clone();
            match response.bytes().await {
                | Ok(body) => Ok(HttpResponse {
                    body: body.to_vec(),
                    headers,
                    status_code,
                }),
                | Err(why) => Err(eyre!(why)),
            }
        }
        | Err(why) => Err(eyre!(why)),
    }
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
