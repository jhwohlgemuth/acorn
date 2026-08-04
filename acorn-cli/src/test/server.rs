#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::std_instead_of_core,
    clippy::unwrap_used,
    clippy::arithmetic_side_effects
)]
use core::time::Duration;
use std::fs::OpenOptions;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
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
fn response_200_head_without_ranges(content_length: usize) -> Vec<u8> {
    format!("HTTP/1.1 200 OK\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n").into_bytes()
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
fn response_401() -> Vec<u8> {
    let body = br#"{"error":"Invalid username or password."}"#;
    let mut response = format!(
        "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}
fn response_500() -> Vec<u8> {
    b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()
}
/// Spawn a local HTTP server that returns source document content.
pub(crate) fn spawn_source_server(content: Vec<u8>) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind source test server");
    let address = format!("http://{}/models.yaml", listener.local_addr().expect("local addr"));
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept source request");
        let _ = parse_request(&mut stream);
        stream.write_all(&response_200_with_body(&content)).expect("write source response");
    });
    (address, handle)
}
/// Spawn a local Hugging Face API server that returns model metadata with a declared base model.
pub(crate) fn spawn_huggingface_model_info_server(identifier: &str, base_model: &str) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind metadata server");
    let endpoint = format!("http://{}", listener.local_addr().expect("metadata server address"));
    let identifier = identifier.to_string();
    let base_model = base_model.to_string();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept metadata request");
        let parsed = parse_request(&mut stream);
        assert!(parsed.path.starts_with(format!("/api/models/{identifier}").as_str()));
        let body = format!(r#"{{"id":"{identifier}","baseModels":["{base_model}"]}}"#);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).expect("write metadata response");
    });
    (endpoint, handle)
}
/// Spawn a local Hugging Face API server that reports an unavailable model.
pub(crate) fn spawn_huggingface_model_unavailable_server(identifier: &str) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind unavailable model server");
    let endpoint = format!("http://{}", listener.local_addr().expect("unavailable model server address"));
    let identifier = identifier.to_string();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept unavailable model request");
        let parsed = parse_request(&mut stream);
        assert!(
            parsed.path.starts_with(format!("/api/models/{identifier}").as_str()),
            "unexpected unavailable model request: {}",
            parsed.path
        );
        stream.write_all(&response_401()).expect("write unavailable model response");
    });
    (endpoint, handle)
}
/// Spawn a local Hugging Face server for direct GGUF and fallback model resolution.
pub(crate) fn spawn_huggingface_model_search_server(
    identifier: &str,
    base_model: &str,
    direct_gguf: Option<&str>,
    requests: usize,
) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind model search server");
    let endpoint = format!("http://{}", listener.local_addr().expect("model search server address"));
    let identifier = identifier.to_string();
    let base_model = base_model.to_string();
    let direct_gguf = direct_gguf.map(str::to_string);
    let search_term = base_model.rsplit('/').next().unwrap_or_default().to_string();
    let handle = thread::spawn(move || {
        for stream in listener.incoming().take(requests) {
            let mut stream = stream.expect("accept model search request");
            let parsed = parse_request(&mut stream);
            let body = if parsed.path.starts_with("/api/models?") && parsed.path.contains(format!("search={search_term}").as_str()) {
                format!(
                    r#"[{{"id":"{identifier}","downloads":142,"tags":["base_model:quantized:{base_model}"],"siblings":[{{"rfilename":"model-Q4_K_M.gguf"}}]}},{{"id":"{identifier}-secondary","downloads":141,"tags":["base_model:quantized:{base_model}"],"siblings":[{{"rfilename":"model-Q4_K_M.gguf"}}]}}]"#
                )
            } else if parsed.path.starts_with("/api/models?") {
                "[]".to_string()
            } else if direct_gguf
                .as_ref()
                .is_some_and(|direct| parsed.path.starts_with(format!("/api/models/{direct}").as_str()))
            {
                let direct = direct_gguf.as_deref().unwrap_or_default();
                format!(r#"{{"id":"{direct}","siblings":[{{"rfilename":"tiny-llama.gguf"}}]}}"#)
            } else {
                let model = parsed.path.trim_start_matches("/api/models/").split('?').next().unwrap_or_default();
                format!(r#"{{"id":"{model}","siblings":[]}}"#)
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).expect("write model search response");
        }
    });
    (endpoint, handle)
}
/// Spawn a local Hugging Face server where the requested base model is missing but a GGUF fallback exists.
pub(crate) fn spawn_huggingface_missing_model_search_server(
    identifier: &str,
    base_model: &str,
    declared_base_model: &str,
) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind missing model search server");
    let endpoint = format!("http://{}", listener.local_addr().expect("missing model search server address"));
    let identifier = identifier.to_string();
    let base_model = base_model.to_string();
    let declared_base_model = declared_base_model.to_string();
    let search_term = base_model.rsplit('/').next().unwrap_or_default().to_string();
    let handle = thread::spawn(move || {
        for stream in listener.incoming().take(2) {
            let mut stream = stream.expect("accept missing model search request");
            let parsed = parse_request(&mut stream);
            let response = if parsed.path.starts_with("/api/models?") && parsed.path.contains(format!("search={search_term}").as_str()) {
                let body = format!(
                    r#"[{{"id":"{identifier}","downloads":142,"tags":["base_model:quantized:{declared_base_model}"],"siblings":[{{"rfilename":"model-Q4_K_M.gguf"}}]}}]"#
                );
                response_200_with_body(body.as_bytes())
            } else {
                assert!(
                    parsed.path.starts_with(format!("/api/models/{base_model}").as_str()),
                    "unexpected missing model request: {}",
                    parsed.path
                );
                response_401()
            };
            stream.write_all(&response).expect("write missing model search response");
        }
    });
    (endpoint, handle)
}
/// Spawn a local Hugging Face API server for the basic model download configuration snapshot.
pub(crate) fn spawn_basic_model_config_server() -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind basic model config server");
    let endpoint = format!("http://{}", listener.local_addr().expect("basic model config server address"));
    let handle = thread::spawn(move || {
        for stream in listener.incoming().take(7) {
            let mut stream = stream.expect("accept basic model config request");
            let parsed = parse_request(&mut stream);
            let response = if parsed.path.contains("/api/models/openai/gpt-oss-2b")
                || parsed.path.contains("/api/models/nvidia/nemotron-3-super-120b-a12b")
            {
                response_401()
            } else {
                let body = if parsed.path.contains("/api/models/mozilla/test-llama") {
                    r#"{"id":"mozilla/test-llama","siblings":[{"rfilename":"tiny-llama.gguf"}]}"#.to_string()
                } else if parsed.path.starts_with("/api/models?") && parsed.path.contains("search=gpt-oss-20b") {
                    r#"[{"id":"unsloth/gpt-oss-20b-GGUF","downloads":142,"tags":["base_model:quantized:openai/gpt-oss-20b"],"siblings":[{"rfilename":"model-Q4_K_M.gguf"}]}]"#.to_string()
                } else if parsed.path.starts_with("/api/models?") && parsed.path.contains("search=gpt-oss-120b") {
                    r#"[{"id":"unsloth/gpt-oss-120b-GGUF","downloads":142,"tags":["base_model:quantized:openai/gpt-oss-120b"],"siblings":[{"rfilename":"model-Q4_K_M.gguf"}]}]"#.to_string()
                } else if parsed.path.starts_with("/api/models?") {
                    "[]".to_string()
                } else {
                    let model = parsed.path.trim_start_matches("/api/models/").split('?').next().unwrap_or_default();
                    format!(r#"{{"id":"{model}","siblings":[]}}"#)
                };
                response_200_with_body(body.as_bytes())
            };
            stream.write_all(&response).expect("write basic model config response");
        }
    });
    (endpoint, handle)
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
/// Spawn a local HTTP server that does not advertise byte-range support for `/model.gguf`.
pub(crate) fn spawn_no_range_server(content: Vec<u8>) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind no-range test server");
    let address = format!("http://{}/model.gguf", listener.local_addr().expect("local addr"));
    let handle = thread::spawn(move || {
        for stream in listener.incoming().take(2) {
            let mut stream = stream.expect("accept");
            let parsed = parse_request(&mut stream);
            let response = if parsed.is_head {
                response_200_head_without_ranges(content.len())
            } else {
                response_200_with_body(&content)
            };
            stream.write_all(&response).expect("write response");
        }
    });
    (address, handle)
}
/// Spawn a local HTTP server that advertises ranges but ignores Range on GET.
pub(crate) fn spawn_ignored_range_server(content: Vec<u8>) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ignored-range test server");
    let address = format!("http://{}/model.gguf", listener.local_addr().expect("local addr"));
    let handle = thread::spawn(move || {
        for stream in listener.incoming().take(2) {
            let mut stream = stream.expect("accept");
            let parsed = parse_request(&mut stream);
            let response = if parsed.is_head {
                response_200_head(content.len())
            } else {
                response_200_with_body(&content)
            };
            stream.write_all(&response).expect("write response");
        }
    });
    (address, handle)
}
/// Spawn a local HTTP server whose first ranged GET grows the part file, then fails.
pub(crate) fn spawn_retry_resume_server(
    content: Vec<u8>,
    part_path: PathBuf,
    initial_offset: usize,
    updated_offset: usize,
) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind retry-resume test server");
    let address = format!("http://{}/model.gguf", listener.local_addr().expect("local addr"));
    let handle = thread::spawn(move || {
        listener.set_nonblocking(true).expect("set nonblocking");
        let mut idle = 0;
        let mut completed = false;
        while !completed && idle < 500 {
            let mut stream = match listener.accept() {
                | Ok((stream, _)) => {
                    idle = 0;
                    stream
                }
                | Err(why) if why.kind() == ErrorKind::WouldBlock => {
                    idle += 1;
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                | Err(why) => panic!("accept failed: {why}"),
            };
            let parsed = parse_request(&mut stream);
            let (response, done) = if parsed.is_head {
                (response_200_head(content.len()), false)
            } else if parsed.range == Some(initial_offset) {
                let end = updated_offset.min(content.len());
                let mut file = OpenOptions::new().append(true).open(&part_path).expect("open partial file");
                file.write_all(&content[initial_offset..end]).expect("extend partial file");
                (response_500(), false)
            } else if let Some(offset) = parsed.range {
                let tail = if offset < content.len() { &content[offset..] } else { &[] };
                (response_206_with_body(offset, content.len(), tail), true)
            } else {
                (response_200_with_body(&content), true)
            };
            stream.write_all(&response).expect("write response");
            completed = done;
        }
        assert!(completed, "retry-resume server timed out before a successful download response");
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
