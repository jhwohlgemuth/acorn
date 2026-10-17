use super::{export_admission, receipt_path, render_review_diff};
use crate::cli::{Arguments, Commands, IntakeCommands};
use acorn::io::api::gitlab::{self, Options};
use acorn::io::api::powerautomate::{jsonl::AdmittedSubmission, Submission};
use acorn_schema::research_activity::LogbookEntry;
use axum::{
    extract::{Path as AxumPath, Query},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use clap::Parser;
use data_encoding::BASE64;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[tokio::test]
async fn test_export_admission_uses_exact_gitlab_revisions_and_replays_identically() {
    let entry: LogbookEntry = serde_json::from_value(serde_json::json!({
        "identifier":"entry-1", "projectId":"pilot", "timestamp":"2026-09-18T12:00:00Z",
        "category":"software", "state":"completed", "summary":"New functionality"
    }))
    .unwrap();
    let entry_value = serde_json::to_value(&entry).unwrap();
    let before = serde_json::json!({"logbook":{"entries":[]}}).to_string();
    let after = serde_json::json!({"logbook":{"entries":[entry_value]}}).to_string();
    let before_encoded = BASE64.encode(before.as_bytes());
    let after_encoded = BASE64.encode(after.as_bytes());
    let stale_head = Arc::new(AtomicBool::new(false));
    let stale_head_for_route = Arc::clone(&stale_head);
    let app = Router::new()
        .route(
            "/api/v4/projects/42",
            get(|| async { Json(serde_json::json!({"id":42,"default_branch":"main"})) }),
        )
        .route(
            "/api/v4/projects/42/merge_requests",
            get(|| async {
                Json(serde_json::json!([{
                    "iid":7, "source_branch":"review", "state":"opened", "web_url":"https://gitlab.example.test/merge_requests/7"
                }]))
            }),
        )
        .route(
            "/api/v4/projects/42/merge_requests/7",
            get(|| async {
                Json(serde_json::json!({
                    "iid":7, "project_id":42, "source_project_id":42, "sha":"head456",
                    "source_branch":"review", "target_branch":"main", "title":"Review", "description":"",
                    "web_url":"https://gitlab.example.test/merge_requests/7",
                    "diff_refs":{"base_sha":"base123","head_sha":"head456","start_sha":"newbase999"}
                }))
            }),
        )
        .route(
            "/api/v4/projects/42/repository/branches/{*branch}",
            get(move |AxumPath(branch): AxumPath<String>| {
                let stale_head = Arc::clone(&stale_head_for_route);
                async move {
                    let revision = match branch.as_str() {
                        | "main" => "newbase999",
                        | "review" if stale_head.load(Ordering::SeqCst) => "moved789",
                        | "review" => "head456",
                        | _ => "",
                    };
                    if revision.is_empty() {
                        (StatusCode::NOT_FOUND, Json(serde_json::json!({"message":"404 Branch Not Found"})))
                    } else {
                        (StatusCode::OK, Json(serde_json::json!({"name":branch,"commit":{"id":revision}})))
                    }
                }
            }),
        )
        .route(
            "/api/v4/projects/42/repository/files/{*path}",
            get(move |Query(params): Query<HashMap<String, String>>| {
                let before_encoded = before_encoded.clone();
                let after_encoded = after_encoded.clone();
                async move {
                    let content = match params.get("ref").map(String::as_str) {
                        | Some("base123") => before_encoded,
                        | Some("head456") => after_encoded,
                        | _ => String::new(),
                    };
                    if content.is_empty() {
                        (StatusCode::NOT_FOUND, Json(serde_json::json!({"message":"404 File Not Found"})))
                    } else {
                        (
                            StatusCode::OK,
                            Json(serde_json::json!({"file_path":"pilot/index.json","size":content.len(),"encoding":"base64","content":content})),
                        )
                    }
                }
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let options = Options::with_token("test").domain(&format!("http://{address}")).identifier("42").build();
    let submission: Submission = serde_json::from_str(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/automation/powerautomate/logbook.jsonl"
        ))
        .lines()
        .next()
        .unwrap(),
    )
    .unwrap();
    let admitted = AdmittedSubmission {
        submission,
        entry,
        domain: gitlab::service_url(&options).unwrap(),
        project: 42,
        path: "pilot/index.json".to_string(),
        branch: "review".to_string(),
        bucket: None,
    };
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("change.diff");
    export_admission(&options, &admitted, "sha256:source", &output).await.unwrap();
    let original = std::fs::read(&output).unwrap();
    let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(receipt_path(&output)).unwrap()).unwrap();
    assert_eq!(receipt["baseRevision"], "base123");
    assert_eq!(receipt["headRevision"], "head456");
    assert_eq!(receipt["mergeRequestIid"], 7);
    assert_eq!(receipt["targetBranch"], "main");
    assert_eq!(receipt["sourceDigest"], "sha256:source");
    assert_eq!(receipt["path"], "pilot/index.json");
    assert_eq!(receipt["diffSha256"], super::file_checksum(&output, None).unwrap().checksum_value);
    export_admission(&options, &admitted, "sha256:source", &output).await.unwrap();
    assert_eq!(std::fs::read(&output).unwrap(), original);
    stale_head.store(true, Ordering::SeqCst);
    let moved_output = directory.path().join("moved.diff");
    assert!(export_admission(&options, &admitted, "sha256:source", &moved_output).await.is_err());
    assert!(!moved_output.exists());
    stale_head.store(false, Ordering::SeqCst);
    let invalid = AdmittedSubmission {
        branch: "missing".to_string(),
        ..admitted
    };
    let missing_output = directory.path().join("missing.diff");
    assert!(export_admission(&options, &invalid, "sha256:source", &missing_output).await.is_err());
    assert!(!missing_output.exists());
    let mismatched = AdmittedSubmission {
        domain: "https://other.example.org".to_string(),
        ..invalid
    };
    let mismatched_output = directory.path().join("mismatched.diff");
    assert!(export_admission(&options, &mismatched, "sha256:source", &mismatched_output)
        .await
        .is_err());
    assert!(!mismatched_output.exists());
    server.abort();
}
#[test]
fn test_manual_gitlab_commands_parse_with_explicit_paths() {
    for command in ["run", "deliver"] {
        let parsed = Arguments::try_parse_from(["acorn", "intake", command, "--config", "config.json"]).unwrap();
        assert!(matches!(parsed.command, Some(Commands::Intake { .. })));
    }
    let parsed = Arguments::try_parse_from([
        "acorn",
        "intake",
        "export",
        "--config",
        "config.json",
        "--submission-id",
        "10000000-0000-4000-8000-000000000001",
        "--output",
        "change.diff",
    ])
    .unwrap();
    assert!(matches!(
        parsed.command,
        Some(Commands::Intake {
            command: IntakeCommands::Export { .. }
        })
    ));
}
#[test]
fn test_review_diff_requires_admitted_entry_and_exact_revision_labels() {
    let entry = serde_json::json!({"id":"entry-1"});
    let before = "{\"logbook\":{\"entries\":[]}}\n";
    let after = "{\"logbook\":{\"entries\":[{\"id\":\"entry-1\"}]}}\n";
    let diff = render_review_diff("pilot/index.json", &entry, before, after, "base123", "head456").unwrap();
    assert!(diff.contains("pilot/index.json@base123"));
    assert!(diff.contains("pilot/index.json@head456"));
    assert_eq!(
        diff,
        render_review_diff("pilot/index.json", &entry, before, after, "base123", "head456").unwrap()
    );
    let graduated = "{\"logbook\":{\"entries\":[{\"id\":\"entry-1\",\"archived\":true}]}}\n";
    assert!(render_review_diff("pilot/index.json", &entry, before, graduated, "base123", "head456").is_ok());
    assert!(render_review_diff("pilot/index.json", &entry, before, before, "base123", "head456").is_err());
    assert_eq!(
        receipt_path(std::path::Path::new("change.diff")),
        std::path::PathBuf::from("change.diff.receipt.json")
    );
}
#[test]
fn test_stage_and_apply_parse_with_explicit_paths() {
    let stage = Arguments::try_parse_from(["acorn", "intake", "stage", "--config", "config.json", "--inbox", "inbox.jsonl"]).unwrap();
    assert!(matches!(
        stage.command,
        Some(Commands::Intake {
            command: IntakeCommands::Stage { .. }
        })
    ));
    let apply = Arguments::try_parse_from([
        "acorn",
        "intake",
        "apply",
        "--config",
        "config.json",
        "--submission-id",
        "10000000-0000-4000-8000-000000000001",
    ])
    .unwrap();
    assert!(matches!(
        apply.command,
        Some(Commands::Intake {
            command: IntakeCommands::Apply { .. }
        })
    ));
}
