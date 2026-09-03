use super::super::localhost::{self, Client};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

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

#[test]
fn test_accepts_only_explicit_loopback_http_endpoints() {
    assert!(Client::new("http://127.0.0.1:8080").is_ok());
    assert!(Client::new("http://localhost:8080").is_ok());
    assert!(Client::new("http://[::1]:8080").is_ok());
    assert!(Client::new("http://localhost").is_err());
    assert!(Client::new("https://localhost:8080").is_err());
    assert!(Client::new("http://example.com:8080").is_err());
}
#[test]
fn test_allocates_an_ephemeral_port() {
    assert_ne!(localhost::available_port().expect("port"), 0);
}
#[test]
fn test_bounded_decoder_rejects_large_responses() {
    assert!(localhost::decode::<serde_json::Value>(br#"{}"#.to_vec(), 2).is_ok());
    assert!(localhost::decode::<serde_json::Value>(br#"{}"#.to_vec(), 1).is_err());
}
#[tokio::test]
async fn test_posts_json_to_loopback_and_decodes_response() {
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
async fn test_rejects_streamed_response_above_limit() {
    let (endpoint, server) = one_response(br#"{"value":"too large"}"#.to_vec());
    let client = Client::new(endpoint).expect("client").with_max_response_bytes(4);
    let input = serde_json::json!({});
    assert!(client.post_json::<_, serde_json::Value>("/complete", &input).await.is_err());
    server.join().expect("server");
}
