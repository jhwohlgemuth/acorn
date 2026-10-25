use super::*;
use acorn::io::api::powerautomate::local::BatchSummary;
use acorn_schema::research_activity::ResearchActivity;
use axum::{
    extract::{Path as AxumPath, Query},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use data_encoding::BASE64;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{read_to_string, write};
#[cfg(unix)]
use std::os::unix::fs::{symlink, PermissionsExt};
use tempfile::tempdir;

#[tokio::test]
async fn test_gitlab_capture_pins_revision_and_preserves_offline_review() {
    let directory = tempdir().unwrap();
    let root = directory.path();
    let config = root.join("config.json");
    let index = root.join("index.json");
    write(&index, serde_json::to_string(&ResearchActivity::default()).unwrap()).unwrap();
    write(
        &config,
        json!({"schemaVersion":"2.0.0", "folders":[{
            "id":"acorn", "members":["scientist@example.org"], "path":format!("file://{}", index.display())
        }]})
        .to_string(),
    )
    .unwrap();
    let line = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/automation/powerautomate/logbook.jsonl"
    ))
    .lines()
    .next()
    .unwrap();
    let content = format!("\n{line}\n\n");
    let size = content.len();
    let encoded = BASE64.encode(content.as_bytes());
    let commit = "1234567890123456789012345678901234567890";
    let router = Router::new()
        .route(
            "/api/v4/projects/{project}/repository/commits/{revision}",
            get(move |AxumPath((project, revision)): AxumPath<(String, String)>| async move {
                assert_eq!(project, "group/nested/repo");
                assert_eq!(revision, "feature/demo");
                Json(json!({"id":commit}))
            }),
        )
        .route(
            "/api/v4/projects/{project}/repository/files/{path}",
            get(
                move |AxumPath((project, path)): AxumPath<(String, String)>, Query(query): Query<HashMap<String, String>>| {
                    let encoded = encoded.clone();
                    async move {
                        assert_eq!(project, "group/nested/repo");
                        assert_eq!(path, ".acorn/intake/logbook.jsonl");
                        assert_eq!(query.get("ref").map(String::as_str), Some(commit));
                        Json(json!({"file_path":path,"size":size,"last_commit_id":commit,"encoding":"base64","content":encoded}))
                    }
                },
            ),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let source = Source::parse(&format!("{origin}/group/nested/repo/-/raw/feature%2Fdemo/.acorn/intake/logbook.jsonl")).unwrap();
    let options = Options::with_token("test").domain(&origin).build();
    let path = source.capture(&config, options.clone()).await.unwrap();
    assert_eq!(read_to_string(&path).unwrap(), content);
    #[cfg(unix)]
    {
        assert_eq!(path.metadata().unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(path.parent().unwrap().metadata().unwrap().permissions().mode() & 0o777, 0o700);
    }
    assert_eq!(source.capture(&config, options).await.unwrap(), path);
    let receipt: Value = serde_json::from_str(&read_to_string(path.with_file_name("source.json")).unwrap()).unwrap();
    assert_eq!(receipt.get("commit").unwrap(), commit);
    assert_eq!(receipt.get("branch").unwrap(), "feature/demo");
    assert_eq!(receipt.get("project").unwrap(), "group/nested/repo");
    assert_eq!(receipt.get("sourceDigest").unwrap(), &json!(format!("sha256:{}", checksum(&content))));
    assert!(!receipt.to_string().contains("token"));
    let batch = BatchSummary::stage(&config, &path).unwrap();
    assert!(batch.can_apply);
    server.abort();
    let (configuration, config) = ApplicationConfiguration::load_local_intake(&config).unwrap();
    let applied = configuration.apply_batch(&config, &batch.batch_id).unwrap();
    assert!(applied.error.is_none());
    let after: Value = serde_json::from_str(&read_to_string(index).unwrap()).unwrap();
    assert_eq!(after.get("logbook").unwrap().get("entries").unwrap().as_array().unwrap().len(), 1);
    write(&path, "changed\n").unwrap();
    assert!(configuration.apply_batch(&config, &batch.batch_id).is_err());
}
#[test]
fn test_gitlab_file_links_decode_components_once() {
    for view in ["blob", "raw"] {
        let source = Source::parse(&format!(
            "https://gitlab.example/group/nested/repo/-/{view}/feature%2Fdemo/path/hello%20world.jsonl"
        ))
        .unwrap();
        assert!(matches!(source, Source::GitLab { project, branch, path, .. }
            if project == "group/nested/repo" && branch == "feature/demo" && path == "path/hello world.jsonl"));
    }
    assert!(matches!(Source::parse("https://gitlab.example/42/-/raw/main/inbox.jsonl").unwrap(), Source::GitLab { project, .. } if project == "42"));
}
#[tokio::test]
async fn test_gitlab_origin_and_offline_fail_before_network_or_storage() {
    let directory = tempdir().unwrap();
    let config = directory.path().join("missing-config.json");
    let uri = "https://gitlab.example/group/repo/-/raw/main/logbook.jsonl";
    assert!(prepare(&config, uri, true).await.unwrap_err().to_string().contains("--offline"));
    let source = Source::parse(uri).unwrap();
    let options = Options::with_token("test-secret").domain("other.example").build();
    let error = source.capture(&config, options).await.unwrap_err().to_string();
    assert!(error.contains("CI_SERVER_HOST"));
    assert!(!error.contains("test-secret"));
    assert!(!directory.path().join(".acorn").exists());
}
#[tokio::test]
async fn test_missing_remote_file_is_an_error_not_an_empty_batch() {
    let router = Router::new()
        .route(
            "/api/v4/projects/42/repository/commits/main",
            get(|| async { Json(json!({"id":"1234567890123456789012345678901234567890"})) }),
        )
        .route(
            "/api/v4/projects/42/repository/files/{path}",
            get(|| async { (StatusCode::NOT_FOUND, Json(json!({"message":"404 File Not Found"}))) }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let directory = tempdir().unwrap();
    let source = Source::parse(&format!("{origin}/42/-/raw/main/missing.jsonl")).unwrap();
    let error = source
        .capture(&directory.path().join("config.json"), Options::with_token("test").domain(origin).build())
        .await
        .unwrap_err();
    assert!(error.to_string().contains("does not exist"));
    assert!(!directory.path().join(".acorn").exists());
    server.abort();
}
#[test]
fn test_paths_and_file_uris_preserve_the_same_source() {
    assert!(matches!(Source::parse("inbox with spaces.jsonl").unwrap(), Source::File(path) if path == Path::new("inbox with spaces.jsonl")));
    #[cfg(not(windows))]
    let (uris, expected) = (
        [
            "file:///tmp/inbox%20one%252F.jsonl",
            "file:/tmp/inbox%20one%252F.jsonl",
            "file://localhost/tmp/inbox%20one%252F.jsonl",
        ],
        Path::new("/tmp/inbox one%2F.jsonl"),
    );
    #[cfg(windows)]
    let (uris, expected) = (
        [
            "file:///C:/tmp/inbox%20one%252F.jsonl",
            "file:/C:/tmp/inbox%20one%252F.jsonl",
            "file://localhost/C:/tmp/inbox%20one%252F.jsonl",
        ],
        Path::new(r"C:\tmp\inbox one%2F.jsonl"),
    );
    for uri in uris {
        assert!(matches!(Source::parse(uri).unwrap(), Source::File(path) if path == expected));
    }
}
#[test]
fn test_rejects_ambiguous_unsafe_or_unsupported_sources() {
    for uri in [
        "",
        "s3://bucket/inbox.jsonl",
        "gitlab://host/project/file",
        "file://remote/tmp/inbox.jsonl",
        "file:relative.jsonl",
        "file:///tmp/inbox.jsonl?token=secret",
        "file:///tmp/inbox.jsonl#fragment",
        "https://gitlab.example/group/repo",
        "http://gitlab.example/group/repo/-/raw/main/inbox.jsonl",
        "https://user:secret@gitlab.example/group/repo/-/raw/main/inbox.jsonl",
        "https://gitlab.example/group/repo/-/raw/main/inbox.jsonl?token=secret",
        "https://gitlab.example/group/repo/-/blob/main/inbox.jsonl#L1",
        "https://gitlab.example/group/repo/-/raw/main/../inbox.jsonl",
        "https://gitlab.example/group/repo/-/raw/main/%2e%2e/inbox.jsonl",
        "https://gitlab.example/group/repo/-/raw/main/%FF",
        "https://gitlab.example/group/repo/-/raw/main/%00",
        "https://gitlab.example/group/repo/-/raw/main/",
    ] {
        let error = Source::parse(uri).unwrap_err().to_string();
        assert!(!error.contains("secret"), "{uri}");
    }
}
#[test]
fn test_snapshot_and_provenance_cannot_be_overwritten() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("snapshot.jsonl");
    preserve(&path, "original\n").unwrap();
    preserve(&path, "original\n").unwrap();
    assert!(preserve(&path, "replacement\n").is_err());
    assert_eq!(read_to_string(&path).unwrap(), "original\n");
}
#[cfg(unix)]
#[test]
fn test_snapshot_symlinks_are_rejected() {
    let directory = tempdir().unwrap();
    let original = directory.path().join("original");
    let link = directory.path().join("snapshot.jsonl");
    write(&original, "original\n").unwrap();
    symlink(&original, &link).unwrap();
    assert!(preserve(&link, "original\n").is_err());
    assert_eq!(read_to_string(&original).unwrap(), "original\n");
}
