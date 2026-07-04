//! HTTP client utilities for ACORN IO operations
use super::ApiResult;
use crate::util::Label;
use async_trait::async_trait;
use axum::http::HeaderMap;
use chrono::Utc;
use color_eyre::eyre::eyre;
use core::fmt;
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
            | "GET" => HttpMethod::Get,
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
async fn execute_with_policy(client: reqwest::Client, request: HttpRequest) -> ApiResult<HttpResponse> {
    let policy = policy::shared_http_policy();
    let method = request.method.clone();
    let url = request.url.clone();
    let max_attempts = policy.max_attempts();
    for attempt in 1..=max_attempts {
        let started = Utc::now();
        let result = execute_with_timeout(client.clone(), request.clone()).await;
        #[allow(clippy::arithmetic_side_effects)]
        let elapsed_ms = (Utc::now() - started).num_milliseconds();
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
/// Utility method to employ best practices when using a Reqwest client to make async HTTP GET requests.
pub fn get(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Get, url)
}
/// Utility method to employ best practices when using a Reqwest client to make async HTTP DELETE requests.
pub fn delete(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Delete, url)
}
/// Utility method to employ best practices when using a Reqwest client to make async HTTP PATCH requests.
pub fn patch(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Patch, url)
}
/// Utility method to employ best practices when using a Reqwest client to make async HTTP POST requests.
pub fn post(url: impl Into<String>) -> HttpRequestBuilder {
    HttpRequestBuilder::new(HttpMethod::Post, url)
}
/// Utility method to employ best practices when using a Reqwest client to make async HTTP PUT requests.
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
