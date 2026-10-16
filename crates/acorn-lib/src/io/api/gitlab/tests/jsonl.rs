use super::*;
use crate::io::api::gitlab::jsonl::{read_snapshot, split_snapshot, JsonlLine, JsonlSnapshot};
use crate::io::config::application::PowerAutomateIntake;
use crate::test::server::TestServer;
use acorn_schema::hardware::memory::Memory;
use acorn_schema::validation::Validate;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

fn intake(branch: &str, max_file: Memory, max_record: Memory) -> PowerAutomateIntake {
    PowerAutomateIntake {
        branch: branch.to_string(),
        path: ".acorn/intake/logbook.jsonl".to_string(),
        poll_interval_secs: 30,
        max_file,
        max_record,
    }
}
fn standard_intake() -> PowerAutomateIntake {
    intake("pilot", Memory::mb(10), Memory::kb(64))
}
async fn snapshot_server(content: Option<String>) -> TestServer {
    let router = Router::new()
        .route(
            "/api/v4/projects/30/repository/commits/{revision}",
            get(|| async { Json(serde_json::json!({"id": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1"})) }),
        )
        .route(
            "/api/v4/projects/30/repository/files/{*path}",
            get(|| async move {
                match content {
                    | Some(content) => (
                        axum::http::StatusCode::OK,
                        Json(serde_json::json!({
                            "file_path": ".acorn/intake/logbook.jsonl",
                            "size": content.len(),
                            "encoding": "base64",
                            "content": data_encoding::BASE64.encode(content.as_bytes())
                        })),
                    )
                        .into_response(),
                    | None => (
                        axum::http::StatusCode::NOT_FOUND,
                        Json(serde_json::json!({"message": "404 File Not Found"})),
                    )
                        .into_response(),
                }
            }),
        );
    TestServer::start(router).await.unwrap()
}
async fn read_from(content: Option<String>, intake: &PowerAutomateIntake) -> crate::io::ApiResult<crate::io::api::gitlab::jsonl::JsonlSnapshot> {
    let server = snapshot_server(content).await;
    let options = Options::with_token("test").domain(&server.base_url).identifier("30").build();
    let snapshot = read_snapshot(intake, &options).await;
    server.stop().await.unwrap();
    snapshot
}

#[test]
fn test_empty_file_yields_no_lines() {
    let snapshot = split_snapshot("", &Memory::kb(64)).unwrap();
    assert!(snapshot.is_empty());
}
#[tokio::test]
async fn test_empty_inbox_file_yields_empty_snapshot() {
    let snapshot = read_from(Some(String::new()), &standard_intake()).await.unwrap();
    assert_eq!(snapshot.commit, "9f86d081884c7d659a2feaa0c55ad015a3bf4f1");
    assert!(snapshot.is_empty());
    assert_eq!(snapshot.len(), 0);
}
#[tokio::test]
async fn test_missing_file_yields_empty_snapshot() {
    let snapshot = read_from(None, &standard_intake()).await.unwrap();
    assert_eq!(snapshot.commit, "9f86d081884c7d659a2feaa0c55ad015a3bf4f1");
    assert!(snapshot.is_empty());
}
#[tokio::test]
async fn test_oversize_file_is_rejected() {
    let intake = intake("pilot", Memory::kb(1), Memory::kb(1));
    let content = format!("{}\n", "x".repeat(2048));
    let error = read_from(Some(content), &intake).await.unwrap_err().to_string();
    assert!(error.contains("file size limit"));
}
#[test]
fn test_oversize_record_is_rejected() {
    let content = format!("{}\n", "x".repeat(2048));
    let error = split_snapshot(&content, &Memory::kb(1)).unwrap_err().to_string();
    assert!(error.contains("record size limit"));
}
#[tokio::test]
async fn test_reads_mixed_lines_with_digests_and_offsets() {
    let content = include_str!("../../../../../../../tests/fixtures/automation/powerautomate/logbook.jsonl").to_string();
    let snapshot = read_from(Some(content.clone()), &standard_intake()).await.unwrap();
    assert_eq!(snapshot.commit, "9f86d081884c7d659a2feaa0c55ad015a3bf4f1");
    assert_eq!(snapshot.len(), 3);
    let raws: Vec<&str> = content.strip_suffix('\n').unwrap().split('\n').collect();
    assert_eq!(snapshot.lines[0].line, 1);
    assert_eq!(snapshot.lines[1].line, 2);
    assert_eq!(snapshot.lines[2].line, 3);
    assert_eq!(snapshot.lines[0].byte_offset, 0);
    assert_eq!(snapshot.lines[1].byte_offset, (raws[0].len() + 1) as u64);
    assert_eq!(snapshot.lines[2].byte_offset, (raws[0].len() + 1 + raws[1].len() + 1) as u64);
    assert_eq!(snapshot.lines[0].raw, raws[0]);
    assert!(snapshot.lines[0].raw.contains("\"projectId\":\"acorn\""));
    assert!(snapshot.lines[1].raw.contains("\"projectId\":\"rad\""));
    assert!(snapshot.lines[2].raw.contains("\"projectId\":\"acorn-tooling\""));
    assert!(snapshot.lines[1].raw.contains("✓"));
    for line in &snapshot.lines {
        assert_eq!(line.digest.len(), 7 + 64);
        assert!(line.digest.starts_with("sha256:"));
        assert!(line.digest.strip_prefix("sha256:").is_some_and(|hash| hash
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())));
    }
    assert_ne!(snapshot.lines[0].digest, snapshot.lines[1].digest);
    assert_ne!(snapshot.lines[1].digest, snapshot.lines[2].digest);
}
#[test]
fn test_rejects_nonterminated_tail() {
    let error = split_snapshot("{\"a\":1}\n{\"b\":2}", &Memory::kb(64)).unwrap_err().to_string();
    assert!(error.contains("nonterminated"));
}
#[tokio::test]
async fn test_repeated_reads_are_identical() {
    let content = include_str!("../../../../../../../tests/fixtures/automation/powerautomate/logbook.jsonl").to_string();
    let intake = standard_intake();
    let first = read_from(Some(content.clone()), &intake).await.unwrap();
    let second = read_from(Some(content), &intake).await.unwrap();
    assert_eq!(first, second);
}
#[tokio::test]
async fn test_snapshot_validation_accepts_reader_output() {
    let content = include_str!("../../../../../../../tests/fixtures/automation/powerautomate/logbook.jsonl").to_string();
    let snapshot = read_from(Some(content), &standard_intake()).await.unwrap();
    assert!(snapshot.validate().is_ok());
}
#[test]
fn test_snapshot_validation_rejects_tampered_lines() {
    let valid = split_snapshot("{\"a\":1}\n", &Memory::kb(64)).unwrap().pop().unwrap();
    assert!(JsonlSnapshot {
        commit: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1".to_string(),
        lines: vec![valid.clone()]
    }
    .validate()
    .is_ok());
    assert!(JsonlSnapshot {
        commit: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1".to_string(),
        lines: vec![JsonlLine {
            digest: "not-a-digest".to_string(),
            ..valid.clone()
        }]
    }
    .validate()
    .is_err());
    assert!(JsonlSnapshot {
        commit: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1".to_string(),
        lines: vec![JsonlLine {
            raw: String::new(),
            ..valid.clone()
        }]
    }
    .validate()
    .is_err());
    assert!(JsonlSnapshot {
        commit: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1".to_string(),
        lines: vec![JsonlLine { line: 0, ..valid.clone() }]
    }
    .validate()
    .is_err());
    assert!(JsonlSnapshot {
        commit: "xyz".to_string(),
        lines: vec![valid]
    }
    .validate()
    .is_err());
}
