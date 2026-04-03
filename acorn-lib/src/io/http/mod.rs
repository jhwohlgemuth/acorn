//! HTTP client utilities for ACORN IO operations
use super::ApiResult;
use crate::util::Label;
use async_trait::async_trait;
use axum::http::HeaderMap;
use color_eyre::eyre::eyre;
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use tower::{service_fn, ServiceExt};
use tracing::{debug, warn};

pub mod policy;

/// See recommendation at <https://support.datacite.org/docs/api#api-versions>
#[cfg(feature = "std")]
const ACORN_USER_AGENT: &str = concat!("ACORN/", env!("CARGO_PKG_VERSION"), " (https://acorn.ornl.gov; mailto:research@ornl.gov)");
#[cfg(not(feature = "std"))]
const ACORN_USER_AGENT: &str = "ACORN (https://acorn.ornl.gov; mailto:research@ornl.gov)";
/// Async HTTP service abstraction for ACORN I/O.
#[async_trait]
pub trait HttpService {
    /// Execute an HTTP request.
    async fn execute(&self, request: HttpRequest) -> ApiResult<HttpResponse>;
}
/// Blocking HTTP service abstraction for ACORN I/O.
pub trait SyncHttpService {
    /// Execute a blocking HTTP request.
    fn execute(&self, request: HttpRequest) -> ApiResult<SyncHttpResponse>;
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
#[derive(Clone)]
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
#[derive(Clone)]
pub struct ReqwestHttpService {
    client: reqwest::Client,
}
/// Reqwest-backed blocking HTTP service adapter.
#[derive(Clone)]
pub struct ReqwestSyncHttpService {
    client: reqwest::blocking::Client,
}
/// Fluent blocking request builder backed by [`SyncHttpService`].
#[derive(Clone)]
pub struct SyncHttpRequestBuilder {
    request: HttpRequest,
    service: ReqwestSyncHttpService,
}
/// Internal blocking HTTP response representation.
#[derive(Clone, Debug)]
pub struct SyncHttpResponse {
    /// Response body bytes
    pub body: Vec<u8>,
    /// Response headers
    pub headers: HeaderMap,
    /// HTTP status code
    pub status_code: u16,
}
impl Default for ReqwestHttpService {
    fn default() -> Self {
        Self {
            client: create_http_client(),
        }
    }
}
impl Default for ReqwestSyncHttpService {
    fn default() -> Self {
        Self {
            client: create_sync_http_client(),
        }
    }
}
impl HttpRequestBuilder {
    /// Add headers to the request.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.request.headers.extend(headers);
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
        Ok(String::from_utf8_lossy(&self.body).into_owned())
    }
}
#[async_trait]
impl HttpService for ReqwestHttpService {
    async fn execute(&self, request: HttpRequest) -> ApiResult<HttpResponse> {
        execute_async_with_policy(self.client.clone(), request).await
    }
}
impl SyncHttpRequestBuilder {
    /// Add headers to the request.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.request.headers.extend(headers);
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
            service: ReqwestSyncHttpService::default(),
        }
    }
    /// Send the request.
    pub fn send(self) -> ApiResult<SyncHttpResponse> {
        self.service.execute(self.request)
    }
}
impl SyncHttpResponse {
    /// Read response body as bytes.
    pub fn bytes(self) -> ApiResult<Vec<u8>> {
        Ok(self.body)
    }
    /// Read response body as text.
    pub fn text(self) -> ApiResult<String> {
        Ok(String::from_utf8_lossy(&self.body).into_owned())
    }
}
impl SyncHttpService for ReqwestSyncHttpService {
    fn execute(&self, request: HttpRequest) -> ApiResult<SyncHttpResponse> {
        let builder = self
            .client
            .request(reqwest_method(&request.method), request.url)
            .header(USER_AGENT, ACORN_USER_AGENT)
            .headers(request.headers);
        let builder = match request.json_body {
            | Some(value) => builder.json(&value),
            | None => builder,
        };
        match builder.send() {
            | Ok(response) => {
                let status_code = response.status().as_u16();
                let headers = response.headers().clone();
                match response.bytes() {
                    | Ok(body) => Ok(SyncHttpResponse {
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
}
/// Create a configured async HTTP client with proxy support and appropriate timeouts
fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(core::time::Duration::from_secs(30))
        .connect_timeout(core::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}
/// Create a configured HTTP client with proxy support and appropriate timeouts
fn create_sync_http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(core::time::Duration::from_secs(30))
        .connect_timeout(core::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}
async fn execute_async_with_policy(client: reqwest::Client, request: HttpRequest) -> ApiResult<HttpResponse> {
    let policy = policy::shared_http_policy();
    let method = request.method.clone();
    let url = request.url.clone();
    let max_attempts = policy.max_attempts();
    for attempt in 1..=max_attempts {
        let started = std::time::Instant::now();
        let result = execute_async_with_timeout(client.clone(), request.clone()).await;
        let elapsed_ms = started.elapsed().as_millis();
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
async fn execute_async_with_timeout(client: reqwest::Client, request: HttpRequest) -> ApiResult<HttpResponse> {
    let service = policy::http_service_builder().service(service_fn(move |value: HttpRequest| {
        let client = client.clone();
        async move { send_async_request(client, value).await }
    }));
    service
        .oneshot(request)
        .await
        .map_err(|why| eyre!("HTTP service timeout or middleware error — {why}"))
}
/// Utility method to employ best practices when using a Reqwest client to make async HTTP GET requests.
pub fn get(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Get, url)
}
/// Utility method to employ best practices when using a Reqwest client to make async HTTP POST requests.
pub fn post(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Post, url)
}
/// Utility method to employ best practices when using a Reqwest client to make async HTTP PUT requests.
pub fn put(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Put, url)
}
impl From<&str> for HttpMethod {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            | "DELETE" => HttpMethod::Delete,
            | "GET" => HttpMethod::Get,
            | "PATCH" => HttpMethod::Patch,
            | "POST" => HttpMethod::Post,
            | "PUT" => HttpMethod::Put,
            | _ => HttpMethod::Get,
        }
    }
}
fn reqwest_method(value: &HttpMethod) -> reqwest::Method {
    match value {
        | HttpMethod::Delete => reqwest::Method::DELETE,
        | HttpMethod::Get => reqwest::Method::GET,
        | HttpMethod::Patch => reqwest::Method::PATCH,
        | HttpMethod::Post => reqwest::Method::POST,
        | HttpMethod::Put => reqwest::Method::PUT,
    }
}
async fn send_async_request(client: reqwest::Client, request: HttpRequest) -> ApiResult<HttpResponse> {
    let builder = client
        .request(reqwest_method(&request.method), request.url)
        .header(USER_AGENT, ACORN_USER_AGENT)
        .headers(request.headers);
    let builder = match request.json_body {
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
pub(crate) fn should_retry(method: &HttpMethod, status_code: Option<u16>) -> bool {
    policy::should_retry(method, status_code)
}
/// Utility method to employ best practices when using a Reqwest client to make HTTP GET requests.
pub fn sync_get(url: impl Into<String>) -> SyncHttpRequestBuilder {
    SyncHttpRequestBuilder::new(HttpMethod::Get, url)
}
/// Utility method to employ best practices when using a Reqwest client to make HTTP POST requests.
pub fn sync_post(url: impl Into<String>) -> SyncHttpRequestBuilder {
    SyncHttpRequestBuilder::new(HttpMethod::Post, url)
}
/// Utility method to employ best practices when using a Reqwest client to make HTTP PUT requests.
pub fn sync_put(url: impl Into<String>) -> SyncHttpRequestBuilder {
    SyncHttpRequestBuilder::new(HttpMethod::Put, url)
}
