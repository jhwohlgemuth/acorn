#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    clippy::arithmetic_side_effects
)]
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread::{self, JoinHandle};

struct ParsedRequest {
    is_head: bool,
    path: String,
    range: Option<usize>,
}
fn parse_request(stream: &mut std::net::TcpStream) -> ParsedRequest {
    let mut request = [0_u8; 4096];
    let count = stream.read(&mut request).expect("read request");
    let text = String::from_utf8_lossy(&request[..count]);
    let first_line = text.lines().next().unwrap_or_default();
    let is_head = first_line.starts_with("HEAD ");
    let path = first_line.split_whitespace().nth(1).unwrap_or("/").to_string();
    let range = text
        .lines()
        .find(|line| line.starts_with("Range: bytes="))
        .and_then(|line| line.strip_prefix("Range: bytes="))
        .and_then(|line| line.strip_suffix('-'))
        .and_then(|value| value.parse::<usize>().ok());
    ParsedRequest { is_head, path, range }
}
fn response_200_head(content_length: usize) -> Vec<u8> {
    format!("HTTP/1.1 200 OK\r\nAccept-Ranges: bytes\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n").into_bytes()
}
fn response_200_with_body(body: &[u8]) -> Vec<u8> {
    let mut response = response_200_head(body.len());
    response.extend_from_slice(body);
    response
}
fn response_206_with_body(offset: usize, total_len: usize, body: &[u8]) -> Vec<u8> {
    let mut headers = format!(
        "HTTP/1.1 206 Partial Content\r\nAccept-Ranges: bytes\r\nContent-Length: {}\r\nContent-Range: bytes {}-{}/{}\r\nConnection: close\r\n\r\n",
        body.len(),
        offset,
        total_len.saturating_sub(1),
        total_len,
    )
    .into_bytes();
    headers.extend_from_slice(body);
    headers
}
fn response_404() -> Vec<u8> {
    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()
}
/// Spawn a local HTTP server that supports HEAD plus full and range GET responses for `/model.gguf`.
pub(crate) fn spawn_range_server(content: Vec<u8>) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let address = format!("http://{}/model.gguf", listener.local_addr().expect("local addr"));
    let handle = thread::spawn(move || {
        for stream in listener.incoming().take(2) {
            let mut stream = stream.expect("accept");
            let parsed = parse_request(&mut stream);
            let response = if parsed.is_head {
                response_200_head(content.len())
            } else if let Some(offset) = parsed.range {
                let tail = if offset < content.len() { &content[offset..] } else { &[] };
                response_206_with_body(offset, content.len(), tail)
            } else {
                response_200_with_body(&content)
            };
            stream.write_all(&response).expect("write response");
        }
    });
    (address, handle)
}
/// Spawn a local HTTP server that serves a model file and its `.sha256` sidecar.
pub(crate) fn spawn_sidecar_server(model_content: Vec<u8>, sha256_content: String) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind sidecar test server");
    let base = format!("http://{}", listener.local_addr().expect("local addr"));
    let handle = thread::spawn(move || {
        for stream in listener.incoming().take(2) {
            let mut stream = stream.expect("accept");
            let parsed = parse_request(&mut stream);
            let response = match parsed.path.as_str() {
                | "/repo/resolve/main/model.gguf.sha256" => {
                    let body = sha256_content.as_bytes();
                    if parsed.is_head {
                        response_200_head(body.len())
                    } else {
                        response_200_with_body(body)
                    }
                }
                | "/repo/resolve/main/model.gguf" => {
                    if parsed.is_head {
                        response_200_head(model_content.len())
                    } else {
                        response_200_with_body(&model_content)
                    }
                }
                | _ => response_404(),
            };
            stream.write_all(&response).expect("write response");
        }
    });
    (base, handle)
}
