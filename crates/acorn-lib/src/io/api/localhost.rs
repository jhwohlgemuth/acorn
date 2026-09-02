//! Restricted HTTP client for APIs bound to the local machine.

use crate::io::ApiResult;
use acorn_core::prelude::String;
use acorn_core::{Location, Scheme};
use color_eyre::eyre::eyre;
use core::time::Duration;
use reqwest::redirect::Policy;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Default maximum response body accepted from a localhost API.
pub const DEFAULT_MAX_RESPONSE_BYTES: usize = 1024 * 1024;

/// HTTP client that refuses non-loopback and encrypted remote endpoints.
#[derive(Clone, Debug)]
pub struct Client {
    base_url: String,
    client: reqwest::Client,
    max_response_bytes: usize,
}

impl Client {
    /// Create a loopback-only HTTP client.
    pub fn new(base_url: impl Into<String>) -> ApiResult<Self> {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        let location = Location::from(base_url.as_str());
        let host = location.host().unwrap_or_default();
        let loopback = matches!(host.to_ascii_lowercase().as_str(), "localhost" | "127.0.0.1" | "::1" | "[::1]");
        match (location.scheme(), loopback, location.port()) {
            | (Scheme::HTTP, true, Some(_)) => reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(2))
                .timeout(Duration::from_secs(30))
                .redirect(Policy::none())
                .build()
                .map(|client| Self {
                    base_url,
                    client,
                    max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
                })
                .map_err(|why| eyre!("Failed to construct localhost API client — {why}")),
            | (Scheme::HTTP, true, None) => Err(eyre!("Localhost API endpoint requires an explicit port")),
            | _ => Err(eyre!(
                "Localhost API endpoint must use http://localhost, 127.0.0.1, or ::1 with an explicit port"
            )),
        }
    }

    /// Override the maximum accepted response size.
    pub fn with_max_response_bytes(self, max_response_bytes: usize) -> Self {
        Self {
            max_response_bytes: max_response_bytes.max(1),
            ..self
        }
    }

    /// Return the validated base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// POST JSON to a relative endpoint and decode a bounded JSON response.
    pub async fn post_json<I, O>(&self, path: &str, input: &I) -> ApiResult<O>
    where
        I: Serialize + ?Sized,
        O: DeserializeOwned,
    {
        match path.starts_with('/') && !path.contains("://") && !path.split('/').any(|segment| segment == "..") {
            | false => Err(eyre!("Localhost API request path must be a safe absolute path")),
            | true => match self.client.post(format!("{}{path}", self.base_url)).json(input).send().await {
                | Ok(response) => {
                    let status = response.status().as_u16();
                    read_bounded(response, self.max_response_bytes)
                        .await
                        .and_then(|body| match (200..=299).contains(&status) {
                            | true => decode(body, self.max_response_bytes),
                            | false => Err(eyre!(
                                "Localhost API returned HTTP {status}: {}",
                                bounded_text(body, self.max_response_bytes)
                            )),
                        })
                }
                | Err(why) => Err(eyre!("Localhost API request failed — {why}")),
            },
        }
    }
}

async fn read_bounded(mut response: reqwest::Response, maximum: usize) -> ApiResult<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        match response.chunk().await {
            | Ok(Some(chunk)) if body.len().saturating_add(chunk.len()) <= maximum => body.extend_from_slice(&chunk),
            | Ok(Some(_)) => break Err(eyre!("Localhost API response exceeded the {maximum}-byte limit")),
            | Ok(None) => break Ok(body),
            | Err(why) => break Err(eyre!("Failed to read localhost API response — {why}")),
        }
    }
}

fn decode<T: DeserializeOwned>(body: Vec<u8>, maximum: usize) -> ApiResult<T> {
    match body.len() <= maximum {
        | true => serde_json::from_slice(&body).map_err(|why| eyre!("Failed to decode localhost API response — {why}")),
        | false => Err(eyre!("Localhost API response exceeded the {maximum}-byte limit")),
    }
}

fn bounded_text(body: Vec<u8>, maximum: usize) -> String {
    let visible = body.get(..body.len().min(maximum)).unwrap_or(&body);
    String::from_utf8_lossy(visible).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn accepts_only_explicit_loopback_http_endpoints() {
        assert!(Client::new("http://127.0.0.1:8080").is_ok());
        assert!(Client::new("http://localhost:8080").is_ok());
        assert!(Client::new("http://[::1]:8080").is_ok());
        assert!(Client::new("http://localhost").is_err());
        assert!(Client::new("https://localhost:8080").is_err());
        assert!(Client::new("http://example.com:8080").is_err());
    }

    #[test]
    fn bounded_decoder_rejects_large_responses() {
        assert!(decode::<serde_json::Value>(br#"{}"#.to_vec(), 2).is_ok());
        assert!(decode::<serde_json::Value>(br#"{}"#.to_vec(), 1).is_err());
    }

    #[tokio::test]
    async fn posts_json_to_loopback_and_decodes_response() {
        let (endpoint, server) = one_response(br#"{"ok":true}"#.to_vec());
        let response = Client::new(endpoint)
            .expect("client")
            .post_json::<_, serde_json::Value>("/complete", &serde_json::json!({ "input": "hello" }))
            .await
            .expect("response");
        server.join().expect("server");
        assert_eq!(response.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    }

    #[tokio::test]
    async fn rejects_streamed_response_above_limit() {
        let (endpoint, server) = one_response(br#"{"value":"too large"}"#.to_vec());
        let client = Client::new(endpoint).expect("client").with_max_response_bytes(4);
        let input = serde_json::json!({});
        assert!(client.post_json::<_, serde_json::Value>("/complete", &input).await.is_err());
        server.join().expect("server");
    }

    fn one_response(body: Vec<u8>) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let endpoint = format!("http://{}", listener.local_addr().expect("address"));
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut request = [0_u8; 4096];
            let _ = stream.read(&mut request);
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(headers.as_bytes()).expect("headers");
            stream.write_all(&body).expect("body");
        });
        (endpoint, server)
    }
}
