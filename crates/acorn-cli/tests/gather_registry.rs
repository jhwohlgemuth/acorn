//! Direct gather extraction compared with the bounded native operation.
use acorn::io::mcp::{GatherInput, GatherReportInput, InputReference};
use assert_cmd::Command;
use core::time::Duration;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_direct_document_discoveries_match_native_records() {
    let root = tempdir().expect("temporary root");
    fs::write(root.path().join("source.txt"), "doi:10.1234/example").expect("fixture");
    let native = GatherInput {
        input: InputReference::File(PathBuf::from("source.txt")),
    }
    .gather(root.path(), true)
    .await
    .expect("native discovery");
    let output = Command::cargo_bin("acorn")
        .expect("binary")
        .current_dir(root.path())
        .args(["--offline", "--no-local-database", "gather", "source.txt", "--format", "json"])
        .output()
        .expect("direct gather");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).expect("direct report");
    let direct = report.get("discoveries").and_then(Value::as_array).expect("discoveries");
    assert_eq!(direct.len(), native.discoveries.len());
    let first = direct.first().expect("first discovery");
    let native_first = native.discoveries.first().expect("first native discovery");
    assert_eq!(first.get("identifier").and_then(Value::as_str), Some(native_first.identifier.as_str()));
    assert_eq!(first.get("source").and_then(Value::as_str), Some(native_first.source.as_str()));
}
#[tokio::test]
async fn test_direct_report_matches_native_report_artifact() {
    let root = tempdir().expect("temporary root");
    let native = GatherReportInput {
        input: InputReference::Text("doi:10.1234/example".into()),
        destination: PathBuf::from("native.json"),
        overwrite: false,
    }
    .write(root.path(), true)
    .await
    .expect("native report");
    let output = Command::cargo_bin("acorn")
        .expect("binary")
        .current_dir(root.path())
        .args([
            "--offline",
            "--no-local-database",
            "gather",
            "--text",
            "doi:10.1234/example",
            "--format",
            "json",
            "--output",
            "direct.json",
        ])
        .output()
        .expect("direct gather");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let direct: Value = serde_json::from_slice(&fs::read(root.path().join("direct.json")).expect("direct report")).expect("direct JSON");
    let routed: Value = serde_json::from_slice(&fs::read(native.destination).expect("native report")).expect("native JSON");
    assert_eq!(direct.get("discoveries"), routed.get("discoveries"));
    assert_eq!(direct.pointer("/summary/inputs"), routed.pointer("/summary/inputs"));
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_direct_url_discoveries_match_native_records() {
    let root = tempdir().expect("temporary root");
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("local listener");
    let address = listener.local_addr().expect("listener address");
    let server = tokio::spawn(async move {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().await.expect("request");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await.expect("read request");
            let body = "doi:10.1234/example";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.expect("response");
        }
    });
    let url = format!("http://{address}/source.txt");
    let native = GatherInput {
        input: InputReference::Url(url.clone()),
    }
    .gather(root.path(), false)
    .await
    .expect("native URL discovery");
    let output = Command::cargo_bin("acorn")
        .expect("binary")
        .current_dir(root.path())
        .args(["--no-local-database", "gather", &url, "--format", "json"])
        .output()
        .expect("direct URL gather");
    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .expect("server timeout")
        .expect("server task");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).expect("direct report");
    let direct = report.get("discoveries").and_then(Value::as_array).expect("discoveries");
    assert_eq!(direct.len(), native.discoveries.len());
    let first = direct.first().expect("first discovery");
    let native_first = native.discoveries.first().expect("first native discovery");
    assert_eq!(first.get("identifier").and_then(Value::as_str), Some(native_first.identifier.as_str()));
    assert_eq!(first.get("source").and_then(Value::as_str), Some(native_first.source.as_str()));
}
