use super::*;
use crate::io::api::gitlab::bot::{Config, HookActor, HookPayload, Server, WebhookDelivery};
use crate::io::api::gitlab::jsonl::{read_snapshot, split_snapshot, JsonlLine, Snapshot};
use crate::io::api::powerautomate::PowerAutomateProvider;
use crate::io::api::webhooks::store::OperationQueue;
use crate::io::config::application::PowerAutomateIntake;
use crate::io::config::ApplicationConfiguration;
use crate::test::server::TestServer;
use crate::test::utils::review_server;
use acorn_core::time::{Seconds, UnitOfTime};
use acorn_schema::hardware::memory::Memory;
use acorn_schema::validation::Validate;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

fn intake(branch: &str, max_file: Memory, max_record: Memory) -> PowerAutomateIntake {
    PowerAutomateIntake {
        branch: branch.to_string(),
        path: ".acorn/intake/logbook.jsonl".to_string(),
        poll_interval: Seconds::new(30),
        max_file,
        max_record,
    }
}
async fn read_from(content: Option<String>, intake: &PowerAutomateIntake) -> crate::io::ApiResult<crate::io::api::gitlab::jsonl::Snapshot> {
    let server = snapshot_server(content).await;
    let options = Options::with_token("test").domain(&server.base_url).identifier("30").build();
    let snapshot = read_snapshot(intake, &options).await;
    server.stop().await.unwrap();
    snapshot
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
                            "last_commit_id": "0000000000000000000000000000000000000001",
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
fn standard_intake() -> PowerAutomateIntake {
    intake("pilot", Memory::mb(10), Memory::kb(64))
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
async fn test_manual_review_worker_graduates_an_authorized_note_once() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/automation/powerautomate/logbook.jsonl"
    ));
    let inbox = format!(
        "{}\n",
        fixture
            .lines()
            .next()
            .unwrap()
            .replace("\"category\":\"research\"", "\"category\":\"personnel\"")
    );
    let (server, state) = review_server(inbox).await;
    let document: ApplicationConfiguration = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/automation/powerautomate/multi-config.json"
    )))
    .unwrap();
    let queue = OperationQueue::from(std::env::temp_dir().join(format!("acorn-manual-graduate-{}.db", nanoid::nanoid!())));
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let mut config = Config::new(options, "127.0.0.1:0".parse().unwrap()).with_operation_queue(queue.clone());
    config.powerautomate = Some(PowerAutomateProvider::new(document.power_automate().unwrap().clone(), document.registry(), "test").unwrap());
    let worker = Server::new(config);
    assert_eq!(worker.poll_jsonl_once().await.unwrap().admitted_count, 1);
    assert!(worker.process_next().await.unwrap());
    let (branch, identifier) = {
        let snapshot = state.lock().unwrap();
        let branch = snapshot.merge_requests[0]["source_branch"].as_str().unwrap().to_string();
        let content = snapshot.content.get(&(branch.clone(), "acorn/index.json".to_string())).unwrap();
        let activity: acorn_schema::research_activity::ResearchActivity = serde_json::from_str(content).unwrap();
        (branch, activity.logbook.unwrap().entries[0].identifier.clone())
    };
    let delivery = WebhookDelivery {
        delivery_id: "manual-graduation-note".to_string(),
        event: HookPayload::Note {
            actor: HookActor::new(42, 1, "reviewer", false),
            note_id: 10,
            body: format!("/acorn graduate {identifier}"),
            noteable_type: "MergeRequest".to_string(),
            noteable_id: Some(1),
            noteable_iid: Some(1),
            system: false,
            confidential: false,
            internal: false,
        },
    };
    queue
        .enqueue_spec(&delivery.delivery_id, &delivery.operation().unwrap().unwrap())
        .unwrap();
    assert!(worker.process_next().await.unwrap());
    assert!(!worker.process_next().await.unwrap());
    {
        let snapshot = state.lock().unwrap();
        let content = snapshot.content.get(&(branch, "acorn/index.json".to_string())).unwrap();
        let activity: acorn_schema::research_activity::ResearchActivity = serde_json::from_str(content).unwrap();
        assert!(activity.logbook.unwrap().entries[0].archived);
        assert_eq!(snapshot.merge_requests.len(), 1);
        assert_eq!(snapshot.review_writes.len(), 2);
    }
    server.stop().await.unwrap();
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
    assert_eq!(snapshot.content, content);
    assert_eq!(snapshot.last_commit_id.as_deref(), Some("0000000000000000000000000000000000000001"));
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
async fn test_single_and_mixed_inbox_publish_only_selected_review_indexes() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/automation/powerautomate/logbook.jsonl"
    ));
    let config_fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/automation/powerautomate/multi-config.json"
    ));
    for record_count in [1, 3] {
        let inbox = fixture.lines().take(record_count).collect::<Vec<_>>().join("\n") + "\n";
        let (server, state) = review_server(inbox.clone()).await;
        let document: ApplicationConfiguration = serde_json::from_str(config_fixture).unwrap();
        let queue = OperationQueue::from(std::env::temp_dir().join(format!("acorn-mixed-jsonl-{}.db", nanoid::nanoid!())));
        let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
        let mut config = Config::new(options, "127.0.0.1:0".parse().unwrap())
            .with_operation_queue(queue.clone())
            .with_polling_enabled(false);
        config.powerautomate = Some(PowerAutomateProvider::new(document.power_automate().unwrap().clone(), document.registry(), "test").unwrap());
        let bot = Server::new(config.clone());
        let summary = bot.poll_jsonl_once().await.unwrap();
        assert_eq!(summary.admitted_count, record_count);
        assert_eq!(summary.removed_count, record_count);
        assert!(state.lock().unwrap().inbox.is_empty());
        if record_count == 3 {
            let mut changed = serde_json::to_value(&document).unwrap();
            changed["folders"][0]["path"] = serde_json::json!("moved/index.json");
            changed["folders"][1]["path"] = serde_json::json!("other/index.json");
            let changed: ApplicationConfiguration = serde_json::from_value(changed).unwrap();
            config.powerautomate = Some(PowerAutomateProvider::new(changed.power_automate().unwrap().clone(), changed.registry(), "test").unwrap());
        }
        let worker = Server::new(config);
        for _ in 0..record_count {
            assert!(worker.process_next().await.unwrap());
        }
        assert!(!worker.process_next().await.unwrap());
        let state_before_replay = state.lock().unwrap().clone();
        assert_eq!(state_before_replay.review_writes.len(), record_count);
        assert_eq!(state_before_replay.merge_requests.len(), if record_count == 1 { 1 } else { 2 });
        assert!(state_before_replay.review_writes.iter().all(|(branch, path)| {
            (branch.starts_with("acorn/powerautomate/acorn/") && path == "acorn/index.json")
                || (branch.starts_with("acorn/powerautomate/research-data/") && path == "research-data/index.json")
        }));
        let entries = |branch: &str, path: &str| -> usize {
            let content = state_before_replay.content.get(&(branch.to_string(), path.to_string()));
            content
                .map(|content| serde_json::from_str::<acorn_schema::research_activity::ResearchActivity>(content).unwrap())
                .and_then(|activity| activity.logbook)
                .map_or(0, |logbook| logbook.entries.len())
        };
        assert_eq!(
            entries("acorn/powerautomate/acorn/2026-10", "acorn/index.json"),
            if record_count == 1 { 1 } else { 2 }
        );
        assert_eq!(
            entries("acorn/powerautomate/research-data/2026-10", "research-data/index.json"),
            if record_count == 1 { 0 } else { 1 }
        );
        assert_eq!(entries("main", "acorn/index.json"), 0);
        assert_eq!(entries("main", "research-data/index.json"), 0);
        state.lock().unwrap().inbox = inbox;
        bot.poll_jsonl_once().await.unwrap();
        assert!(!worker.process_next().await.unwrap());
        {
            let replayed = state.lock().unwrap();
            assert_eq!(replayed.review_writes, state_before_replay.review_writes);
            assert_eq!(replayed.merge_requests.len(), state_before_replay.merge_requests.len());
        }
        server.stop().await.unwrap();
    }
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
    assert!(Snapshot {
        content: String::new(),
        last_commit_id: None,
        commit: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1".to_string(),
        lines: vec![valid.clone()]
    }
    .validate()
    .is_ok());
    assert!(Snapshot {
        content: String::new(),
        last_commit_id: None,
        commit: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1".to_string(),
        lines: vec![JsonlLine {
            digest: "not-a-digest".to_string(),
            ..valid.clone()
        }]
    }
    .validate()
    .is_err());
    assert!(Snapshot {
        content: String::new(),
        last_commit_id: None,
        commit: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1".to_string(),
        lines: vec![JsonlLine {
            raw: String::new(),
            ..valid.clone()
        }]
    }
    .validate()
    .is_err());
    assert!(Snapshot {
        content: String::new(),
        last_commit_id: None,
        commit: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1".to_string(),
        lines: vec![JsonlLine { line: 0, ..valid.clone() }]
    }
    .validate()
    .is_err());
    assert!(Snapshot {
        content: String::new(),
        last_commit_id: None,
        commit: "xyz".to_string(),
        lines: vec![valid]
    }
    .validate()
    .is_err());
}
