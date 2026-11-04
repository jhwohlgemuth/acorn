//! Direct OpenAPI import equivalence for the bounded native route.
use acorn::io::database::{schema::ModelRow, Row};
use acorn::io::mcp::{InputReference, ModelImportApplyInput, ModelImportInput, SpecImportApplyInput, SpecImportInput};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const SPEC: &str = "openapi: 3.0.0\npaths:\n  /projects/{id}:\n    get:\n      operationId: getProject\n";
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_model_metadata_preview_and_persistence_match_direct_command() {
    let root = tempdir().expect("model metadata root");
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("mock Hugging Face listener");
    let domain = format!("http://{}", listener.local_addr().expect("listener address"));
    let server = tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.expect("metadata request");
            let mut request = [0_u8; 2048];
            let count = stream.read(&mut request).await.expect("read metadata request");
            assert!(String::from_utf8_lossy(request.get(..count).expect("request bytes")).contains("/api/models/acme/tiny-gguf/tree/main"));
            let body = r#"[{"path":"tiny-Q4_K_M.gguf","type":"file","size":123}]"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.expect("metadata response");
        }
    });
    let request = ModelImportInput {
        identifier: "acme/tiny-gguf".to_string(),
    };
    let preview = request.clone().preview(&domain, false).await.expect("native metadata preview");
    assert_eq!(preview.requested, "acme/tiny-gguf");
    assert_eq!(preview.resolved, "acme/tiny-gguf");
    assert_eq!(preview.weight_count, 1);
    let direct_preview = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .env("HF_ENDPOINT", &domain)
        .args(["--no-local-database", "import", "model", "acme/tiny-gguf", "--no-fallback", "--dry-run"])
        .output()
        .expect("direct metadata preview");
    assert!(direct_preview.status.success(), "{}", String::from_utf8_lossy(&direct_preview.stderr));
    assert!(String::from_utf8_lossy(&direct_preview.stdout).contains("Resolved metadata for 1 model"));
    assert!(!root.path().join("native.sqlite").exists());

    let direct_path = root.path().join("direct.sqlite");
    let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .env("HF_ENDPOINT", &domain)
        .arg("--database-path")
        .arg(&direct_path)
        .args(["import", "model", "acme/tiny-gguf", "--no-fallback"])
        .output()
        .expect("direct metadata import");
    assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
    let native = ModelImportApplyInput {
        request,
        destination: PathBuf::from("native.sqlite"),
        overwrite: false,
    }
    .apply(root.path(), &domain, false)
    .await
    .expect("native metadata import");
    assert_eq!(native.metadata.weight_count, 1);
    let direct_row = ModelRow::init()
        .model_id("acme/tiny-gguf")
        .build()
        .select(Some(direct_path), |row| row.model_id.as_deref() == Some("acme/tiny-gguf"))
        .expect("direct model row")
        .expect("model present");
    let native_row = ModelRow::init()
        .model_id("acme/tiny-gguf")
        .build()
        .select(Some(native.destination), |row| row.model_id.as_deref() == Some("acme/tiny-gguf"))
        .expect("native model row")
        .expect("model present");
    assert_eq!(direct_row.weights, native_row.weights);
    server.abort();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_remote_spec_preview_matches_direct_command_and_rejects_offline() {
    let root = tempdir().expect("temporary remote specification root");
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("local HTTP listener");
    let address = listener.local_addr().expect("listener address");
    let server = tokio::spawn(async move {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().await.expect("request");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await.expect("read request");
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/yaml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{SPEC}",
                SPEC.len()
            );
            stream.write_all(response.as_bytes()).await.expect("response");
        }
    });
    let url = format!("http://{address}/openapi.yaml");
    let input = SpecImportInput {
        source: InputReference::Url(url.clone()),
        name: "example::api".to_string(),
        domain: "api.example.com".to_string(),
        api_root: None,
    };
    assert!(input.clone().preview(root.path(), true).await.is_err());
    let native = input.preview(root.path(), false).await.expect("remote native preview");
    let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args([
            "--no-local-database",
            "import",
            "spec",
            &url,
            "--name",
            "example::api",
            "--domain",
            "api.example.com",
            "--dry-run",
        ])
        .output()
        .expect("direct remote preview");
    assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
    assert_eq!(native.content.trim_end(), String::from_utf8_lossy(&direct.stdout).trim_end());
    server.await.expect("HTTP fixture served both requests");
}
#[tokio::test]
async fn test_spec_preview_and_apply_match_direct_command() {
    let root = tempdir().expect("temporary specification root");
    let source = root.path().join("openapi.yaml");
    fs::write(&source, SPEC).expect("write specification");
    let input = SpecImportInput {
        source: InputReference::File(PathBuf::from("openapi.yaml")),
        name: "example::api".to_string(),
        domain: "api.example.com".to_string(),
        api_root: Some("v1".to_string()),
    };
    let direct_preview = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["--no-local-database", "import", "spec"])
        .arg(&source)
        .args(["--name", "example::api", "--domain", "api.example.com", "--root", "v1", "--dry-run"])
        .output()
        .expect("direct specification preview");
    assert!(direct_preview.status.success(), "{}", String::from_utf8_lossy(&direct_preview.stderr));
    let native_preview = input.clone().preview(root.path(), true).await.expect("native specification preview");
    assert_eq!(
        native_preview.content.trim_end(),
        String::from_utf8_lossy(&direct_preview.stdout).trim_end()
    );
    let direct_destination = root.path().join("direct.json");
    let direct_apply = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["--no-local-database", "import", "spec"])
        .arg(&source)
        .args(["--name", "example::api", "--domain", "api.example.com", "--root", "v1", "--output"])
        .arg(&direct_destination)
        .output()
        .expect("direct specification import");
    assert!(direct_apply.status.success(), "{}", String::from_utf8_lossy(&direct_apply.stderr));
    let native = SpecImportApplyInput {
        request: input,
        destination: PathBuf::from("native.json"),
        overwrite: false,
    }
    .apply(root.path(), true)
    .await
    .expect("native specification import");
    assert_eq!(
        fs::read(direct_destination).expect("direct endpoint"),
        fs::read(native.destination).expect("native endpoint")
    );
    assert_eq!(fs::read_to_string(source).expect("unchanged source"), SPEC);
}
