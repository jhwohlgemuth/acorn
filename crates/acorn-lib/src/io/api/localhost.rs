//! Restricted HTTP client for APIs bound to the local machine
use crate::io::{http, ApiResult};
use acorn_core::prelude::String;
use acorn_core::{Location, Scheme};
use color_eyre::eyre::eyre;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::net::TcpListener;

/// Default maximum response body accepted from a localhost API
pub const DEFAULT_MAX_RESPONSE_BYTES: usize = 1024 * 1024;

/// HTTP client that refuses non-loopback and encrypted remote endpoints
#[derive(Clone, Debug)]
pub struct Client {
    base_url: String,
    max_response_bytes: usize,
}
impl Client {
    /// Create a loopback-only HTTP client
    pub fn new(base_url: impl Into<String>) -> ApiResult<Self> {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        let location = Location::from(base_url.as_str());
        let host = location.host().unwrap_or_default();
        let loopback = matches!(host.to_ascii_lowercase().as_str(), "localhost" | "127.0.0.1" | "::1" | "[::1]");
        match (location.scheme(), loopback, location.port()) {
            | (Scheme::HTTP, true, Some(_)) => Ok(Self {
                base_url,
                max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            }),
            | (Scheme::HTTP, true, None) => Err(eyre!("Localhost API endpoint requires an explicit port")),
            | _ => Err(eyre!(
                "Localhost API endpoint must use http://localhost, 127.0.0.1, or ::1 with an explicit port"
            )),
        }
    }
    /// Override the maximum accepted response size
    pub fn with_max_response_bytes(self, bytes: usize) -> Self {
        Self {
            max_response_bytes: bytes.max(1),
            ..self
        }
    }
    /// Return the validated base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    /// POST JSON to a relative endpoint and decode a bounded JSON response
    pub async fn post_json<I, O>(&self, path: &str, input: &I) -> ApiResult<O>
    where
        I: Serialize + ?Sized,
        O: DeserializeOwned,
    {
        match path.starts_with('/') && !path.contains("://") && !path.split('/').any(|segment| segment == "..") {
            | false => Err(eyre!("Localhost API request path must be a safe absolute path")),
            | true => match serde_json::to_value(input).map_err(|why| eyre!("Failed to encode localhost API request — {why}")) {
                | Ok(body) => match http::loopback_post(format!("{}{path}", self.base_url)).map(|request| request.json(&body)) {
                    | Ok(request) => match request.send().await {
                        | Ok(response) => match (200..=299).contains(&response.status_code) {
                            | true => decode(response.body, self.max_response_bytes),
                            | false => Err(eyre!(
                                "Localhost API returned HTTP {}: {}",
                                response.status_code,
                                bounded_text(response.body, self.max_response_bytes)
                            )),
                        },
                        | Err(why) => Err(eyre!("Localhost API request failed — {why}")),
                    },
                    | Err(why) => Err(why),
                },
                | Err(why) => Err(why),
            },
        }
    }
}
/// Allocate an ephemeral loopback TCP port for a local API process.
pub fn available_port() -> ApiResult<u16> {
    TcpListener::bind("127.0.0.1:0")
        .and_then(|listener| listener.local_addr())
        .map(|address| address.port())
        .map_err(|why| eyre!("Failed to allocate a private localhost port — {why}"))
}
fn bounded_text(body: Vec<u8>, maximum: usize) -> String {
    let visible = body.get(..body.len().min(maximum)).unwrap_or(&body);
    String::from_utf8_lossy(visible).into_owned()
}
pub(super) fn decode<T: DeserializeOwned>(body: Vec<u8>, maximum: usize) -> ApiResult<T> {
    match body.len() <= maximum {
        | true => serde_json::from_slice(&body).map_err(|why| eyre!("Failed to decode localhost API response — {why}")),
        | false => Err(eyre!("Localhost API response exceeded the {maximum}-byte limit")),
    }
}
