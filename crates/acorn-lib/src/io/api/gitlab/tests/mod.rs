#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::automation::ProcessRepositoryResponseExt;
use super::*;
use crate::io::api::gitlab::{
    commit_sha, handle_tree_paths_response, project_details, repository_file, repository_tree, PaginationKey, ProgrammingLanguageDetails,
    ProgrammingLanguageUseResponse, ProgrammingLanguagesResponse, TreeResponse,
};
use crate::io::api::webhooks::store::OperationQueue;
use crate::io::api::{self, sluggify, IntoHeaders};
use crate::io::config::ApplicationConfiguration;
use crate::param;
use crate::test::server::TestServer;
use crate::util::constants::app::{
    APPLICATION, GITLAB_CREDENTIAL_VARIABLE_PAIRS, GITLAB_JOB_TOKEN_HEADER, GITLAB_PRIVATE_TOKEN_HEADER, MERGE_REQUEST_REPORT_MARKER,
};
use crate::util::constants::env::GITLAB_TOKEN_VARIABLE_NAMES;
use acorn_core::prelude::HashMap;
use alloc::sync::Arc;
use axum::extract::{Path as AxumPath, Request, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use core::sync::atomic::{AtomicUsize, Ordering};
use secrecy::ExposeSecret;
use std::sync::Mutex;

#[derive(Clone, Default)]
struct CitationCompletionState {
    label_writes: Arc<Mutex<u64>>,
    labels: Arc<Mutex<Vec<String>>>,
    note_writes: Arc<Mutex<u64>>,
    notes: Arc<Mutex<Vec<serde_json::Value>>>,
}
#[derive(Clone, Default)]
struct CommitStatusEffectState {
    posts: Arc<Mutex<u64>>,
    statuses: Arc<Mutex<Vec<serde_json::Value>>>,
}
#[derive(Clone, Default)]
struct MergeRequestWorkflowState {
    automation_requests: Arc<Mutex<Vec<serde_json::Value>>>,
    file_queries: Arc<Mutex<Vec<String>>>,
    notes: Arc<Mutex<Vec<serde_json::Value>>>,
    statuses: Arc<Mutex<Vec<serde_json::Value>>>,
}
#[derive(Clone, Default)]
struct ReconciledPublicationState {
    branch_creates: Arc<Mutex<u64>>,
    branch_head: Arc<Mutex<Option<String>>>,
    commit_creates: Arc<Mutex<u64>>,
    content: Arc<Mutex<Option<String>>>,
    merge_request: Arc<Mutex<Option<serde_json::Value>>>,
    merge_request_creates: Arc<Mutex<u64>>,
}
#[test]
fn test_application_configuration_deserializes_workflow_policy() {
    let configuration = serde_json::from_value::<ApplicationConfiguration>(serde_json::json!({
        "workflow": {
            "allow-forks": false,
            "workflows": {
                "repository-quality": {
                    "actions": ["validate"],
                    "effects": ["note"],
                    "formats": ["json"],
                    "outputs": {"mode": "source-and-artifacts"},
                    "paths": ["**/*.json"]
                }
            }
        }
    }))
    .unwrap();
    let deployment = configuration.workflow.unwrap();
    assert!(deployment.validate().is_ok());
    assert!(deployment.workflow("repository-quality").unwrap().accepts("metadata.json").unwrap());
}
#[tokio::test]
async fn test_citation_merge_completion_reconciles_label_and_comment_after_ambiguous_failures() {
    let state = CitationCompletionState::default();
    let router = Router::new()
        .route("/api/v4/user", get(|| async { Json(serde_json::json!({"id":99})) }))
        .route(
            "/api/v4/projects/30/merge_requests/9",
            get(|| async {
                Json(serde_json::json!({
                    "iid":9,
                    "author":{"id":99},
                    "project_id":30,
                    "source_project_id":30,
                    "sha":"citation-sha",
                    "source_branch":"acorn/citation-work-item-7",
                    "target_branch":"main",
                    "title":"Citation",
                    "description":"<!-- acorn:repository-workflow -->\nWorkflow: `citation-work-item-7`",
                    "web_url":"https://gitlab.example/merge_requests/9"
                }))
            }),
        )
        .route(
            "/api/v4/projects/30/issues/7",
            get(|State(state): State<CitationCompletionState>| async move {
                Json(serde_json::json!({
                    "id":70,
                    "iid":7,
                    "project_id":30,
                    "title":"Result",
                    "description":"",
                    "author":{"id":1,"username":"scientist"},
                    "issue_type":"issue",
                    "labels":state.labels.lock().unwrap().clone()
                }))
            })
            .put(
                |State(state): State<CitationCompletionState>, Json(body): Json<serde_json::Value>| async move {
                    *state.label_writes.lock().unwrap() += 1;
                    state
                        .labels
                        .lock()
                        .unwrap()
                        .push(body["add_labels"].as_str().unwrap_or_default().to_string());
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"message":"response lost after label update"})),
                    )
                },
            ),
        )
        .route(
            "/api/v4/projects/30/issues/7/notes",
            get(|State(state): State<CitationCompletionState>| async move { Json(state.notes.lock().unwrap().clone()) }).post(
                |State(state): State<CitationCompletionState>, Json(body): Json<serde_json::Value>| async move {
                    *state.note_writes.lock().unwrap() += 1;
                    state.notes.lock().unwrap().push(serde_json::json!({
                        "id":80,
                        "body":body["body"],
                        "author":{"id":99,"username":"acorn-bot","bot":true}
                    }));
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"message":"response lost after comment creation"})),
                    )
                },
            ),
        )
        .with_state(state.clone());
    let server = TestServer::start(router).await.unwrap();
    let queue = OperationQueue::from(std::env::temp_dir().join(format!("acorn-citation-completion-{}.db", nanoid::nanoid!())));
    let options = Options::with_token("outbound")
        .domain(&server.base_url)
        .identifier("30")
        .build()
        .with_effects(queue)
        .with_merge_request_iid(9);
    assert!(super::intake::complete_citation_merge(&options).await.is_err());
    assert!(super::intake::complete_citation_merge(&options).await.is_err());
    assert!(super::intake::complete_citation_merge(&options).await.unwrap());
    assert_eq!(state.labels.lock().unwrap().as_slice(), [format!("{APPLICATION}::collected")]);
    assert_eq!(*state.label_writes.lock().unwrap(), 1);
    assert_eq!(*state.note_writes.lock().unwrap(), 1);
    {
        let notes = state.notes.lock().unwrap();
        assert_eq!(notes.len(), 1);
        assert!(notes[0]["body"].as_str().unwrap().contains("merge request !9"));
    }
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_citation_merge_completion_rejects_spoofed_human_merge_request() {
    let router = Router::new()
        .route("/api/v4/user", get(|| async { Json(serde_json::json!({"id":99})) }))
        .route(
            "/api/v4/projects/30/merge_requests/9",
            get(|| async {
                Json(serde_json::json!({
                    "iid":9,
                    "author":{"id":4},
                    "project_id":30,
                    "source_project_id":30,
                    "sha":"citation-sha",
                    "source_branch":"acorn/citation-work-item-7",
                    "target_branch":"main",
                    "title":"Citation",
                    "description":"<!-- acorn:repository-workflow -->\nWorkflow: `citation-work-item-7`",
                    "web_url":"https://gitlab.example/merge_requests/9"
                }))
            }),
        );
    let server = TestServer::start(router).await.unwrap();
    let queue = OperationQueue::from(std::env::temp_dir().join(format!("acorn-citation-spoof-{}.db", nanoid::nanoid!())));
    let options = Options::with_token("outbound")
        .domain(&server.base_url)
        .identifier("30")
        .build()
        .with_effects(queue)
        .with_merge_request_iid(9);
    assert!(!super::intake::complete_citation_merge(&options).await.unwrap());
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_commit_sha_resolves_an_exact_reference() {
    let router = Router::new().route(
        "/api/v4/projects/30/repository/commits/{revision}",
        get(|AxumPath(revision): AxumPath<String>| async move {
            assert_eq!(revision, "release/v1");
            Json(serde_json::json!({"id":"abc123"}))
        }),
    );
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("").domain(&server.base_url).identifier("30").build();
    assert_eq!(commit_sha(&options, "release/v1").await.unwrap(), "abc123");
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_commit_status_effect_reconciles_after_an_ambiguous_failure() {
    let state = CommitStatusEffectState::default();
    let router = Router::new()
        .route(
            "/api/v4/projects/30/repository/commits/abc123/statuses",
            get(|State(state): State<CommitStatusEffectState>| async move { Json(state.statuses.lock().unwrap().clone()) }),
        )
        .route(
            "/api/v4/projects/30/statuses/abc123",
            post(
                |State(state): State<CommitStatusEffectState>, Json(body): Json<serde_json::Value>| async move {
                    *state.posts.lock().unwrap() += 1;
                    state.statuses.lock().unwrap().push(serde_json::json!({
                        "name":format!("{APPLICATION}/check"),
                        "sha":"abc123",
                        "status":body["state"],
                        "description":body["description"],
                        "target_url":body["target_url"]
                    }));
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"message":"response lost after status creation"})),
                    )
                },
            ),
        )
        .with_state(state.clone());
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound")
        .domain(&server.base_url)
        .identifier("30")
        .sha("abc123")
        .build();
    let queue = OperationQueue::from(std::env::temp_dir().join(format!("acorn-status-effect-{}.db", nanoid::nanoid!())));
    let publish = || async {
        effects::Effect::new(
            ["gitlab", "30", "commit", "abc123", "status", "success"],
            "gitlab-commit-status",
            Some(&queue),
        )
        .commit_status(
            &options,
            CommitStatusState::Success,
            "ACORN merge request analysis passed",
            Some("https://gitlab.example/merge_requests/7"),
        )
        .await
    };
    assert!(publish().await.is_err());
    assert_eq!(publish().await.unwrap().status, "success");
    assert_eq!(*state.posts.lock().unwrap(), 1);
    server.stop().await.unwrap();
}
#[test]
fn test_creator_slug_normalizes_and_falls_back() {
    assert_eq!(sluggify(" Alice.Example ", 42), "alice-example");
    assert_eq!(sluggify("---", 42), "user-42");
}
#[test]
fn test_deployment_policy_accepts_declared_formats_and_brace_patterns() {
    let config = DeploymentConfig::default();
    let policy = config.workflow("repository-quality").unwrap();
    assert!(policy.accepts("metadata/CITATION.cff").unwrap());
    assert!(policy.accepts("metadata/activity.jsonc").unwrap());
    assert!(policy.accepts("metadata/activity.zonf").unwrap());
    assert!(!policy.accepts("src/main.rs").unwrap());
}
#[test]
fn test_deployment_policy_rejects_unknown_values_and_unsafe_paths() {
    let unknown = "workflows:\n  unknown:\n    paths: ['**/*.json']\n    formats: [json]\n    actions: [validate]\n    outputs: {mode: source-and-artifacts}\n    effects: [note]\n";
    assert!(DeploymentConfig::from_yaml(unknown)
        .unwrap_err()
        .to_string()
        .contains("Unknown repository workflow"));
    let unsafe_path = "workflows:\n  repository-quality:\n    paths: ['../*.json']\n    formats: [json]\n    actions: [validate]\n    outputs: {mode: source-and-artifacts}\n    effects: [note]\n";
    assert!(DeploymentConfig::from_yaml(unsafe_path)
        .unwrap_err()
        .to_string()
        .contains("Unsafe repository path"));
    let unknown_effect = "workflows:\n  repository-quality:\n    paths: ['**/*.json']\n    formats: [json]\n    actions: [validate]\n    outputs: {mode: source-and-artifacts}\n    effects: [release]\n";
    assert!(DeploymentConfig::from_yaml(unknown_effect)
        .unwrap_err()
        .to_string()
        .contains("unknown variant"));
    let unsupported_format = "workflows:\n  repository-quality:\n    paths: ['**/*.pdf']\n    formats: [pdf]\n    actions: [validate]\n    outputs: {mode: source-and-artifacts}\n    effects: [note]\n";
    assert!(DeploymentConfig::from_yaml(unsupported_format)
        .unwrap_err()
        .to_string()
        .contains("Unsupported repository workflow format"));
}
#[test]
fn test_effect_key_uses_application_namespace_and_parts() {
    let effect = effects::Effect::new(["gitlab", "30", "commit", "abc123"], "gitlab-commit", None);
    assert_eq!(effect.to_string(), format!("{APPLICATION}:gitlab:30:commit:abc123"));
}
#[test]
fn test_fork_policy_requires_an_explicit_destination() {
    let config = "allow-forks: true\nworkflows:\n  repository-quality:\n    paths: ['**/*.json']\n    formats: [json]\n    actions: [validate]\n    outputs: {mode: source-and-artifacts}\n    effects: [note]\n";
    assert!(DeploymentConfig::from_yaml(config)
        .unwrap_err()
        .to_string()
        .contains("requires an explicit destination"));
}
#[test]
fn test_gitlab_credential_variables_match_precedence_and_headers() {
    assert_eq!(
        GITLAB_CREDENTIAL_VARIABLE_PAIRS,
        [
            ("GITLAB_TOKEN", GITLAB_PRIVATE_TOKEN_HEADER),
            ("PRIVATE_TOKEN", GITLAB_PRIVATE_TOKEN_HEADER),
            ("CI_JOB_TOKEN", GITLAB_JOB_TOKEN_HEADER),
        ]
    );
    assert_eq!(GITLAB_CREDENTIAL_VARIABLE_PAIRS.map(|(name, _)| name), GITLAB_TOKEN_VARIABLE_NAMES);
}
#[tokio::test]
async fn test_gitlab_effects_reconcile_ambiguous_publication_and_corrected_content() {
    let state = ReconciledPublicationState::default();
    let router = Router::new()
        .route(
            "/api/v4/projects/30/repository/branches/{*branch}",
            get(|State(state): State<ReconciledPublicationState>| async move {
                match state.branch_head.lock().unwrap().clone() {
                    | Some(head) => (
                        axum::http::StatusCode::OK,
                        Json(serde_json::json!({"name":"acorn/citation-work-item-7","commit":{"id":head}})),
                    ),
                    | None => (
                        axum::http::StatusCode::NOT_FOUND,
                        Json(serde_json::json!({"message":"404 Branch Not Found"})),
                    ),
                }
            }),
        )
        .route(
            "/api/v4/projects/30/repository/branches",
            post(
                |State(state): State<ReconciledPublicationState>, Json(body): Json<serde_json::Value>| async move {
                    *state.branch_creates.lock().unwrap() += 1;
                    *state.branch_head.lock().unwrap() = Some("base-sha".to_string());
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"message":"response lost after branch creation", "branch":body["branch"]})),
                    )
                },
            ),
        )
        .route(
            "/api/v4/projects/30/repository/files/{*path}",
            get(|State(state): State<ReconciledPublicationState>| async move {
                match state.content.lock().unwrap().clone() {
                    | Some(content) => (
                        axum::http::StatusCode::OK,
                        Json(serde_json::json!({
                            "file_path":"citations/work-items/7/result/CITATION.cff",
                            "size":content.len(),
                            "encoding":"base64",
                            "content":data_encoding::BASE64.encode(content.as_bytes())
                        })),
                    ),
                    | None => (
                        axum::http::StatusCode::NOT_FOUND,
                        Json(serde_json::json!({"message":"404 File Not Found"})),
                    ),
                }
            }),
        )
        .route(
            "/api/v4/projects/30/repository/commits",
            post(
                |State(state): State<ReconciledPublicationState>, Json(body): Json<serde_json::Value>| async move {
                    let count = {
                        let mut count = state.commit_creates.lock().unwrap();
                        *count += 1;
                        *count
                    };
                    let content = body["actions"][0]["content"].as_str().unwrap_or_default().to_string();
                    *state.content.lock().unwrap() = Some(content);
                    *state.branch_head.lock().unwrap() = Some(format!("commit-{count}"));
                    let status = if count == 1 {
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR
                    } else {
                        axum::http::StatusCode::OK
                    };
                    let response = match count {
                        | 1 => serde_json::json!({"message":"response lost after commit creation"}),
                        | _ => serde_json::json!({"id":format!("commit-{count}"),"web_url":format!("https://gitlab.example/commit-{count}")}),
                    };
                    (status, Json(response))
                },
            ),
        )
        .route(
            "/api/v4/projects/30/merge_requests",
            get(|State(state): State<ReconciledPublicationState>| async move {
                Json(state.merge_request.lock().unwrap().clone().into_iter().collect::<Vec<_>>())
            })
            .post(
                |State(state): State<ReconciledPublicationState>, Json(body): Json<serde_json::Value>| async move {
                    *state.merge_request_creates.lock().unwrap() += 1;
                    *state.merge_request.lock().unwrap() = Some(serde_json::json!({
                        "iid":9,
                        "source_branch":body["source_branch"],
                        "state":"opened",
                        "web_url":"https://gitlab.example/merge_requests/9"
                    }));
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"message":"response lost after merge request creation"})),
                    )
                },
            ),
        )
        .with_state(state.clone());
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound")
        .branch("acorn/citation-work-item-7")
        .domain(&server.base_url)
        .identifier("30")
        .workflow("citation-work-item-7")
        .build();
    let queue = OperationQueue::from(std::env::temp_dir().join(format!("acorn-gitlab-effects-{}.db", nanoid::nanoid!())));
    let changes = |content: &str| crate::io::workflow::ProcessRepositoryResponse {
        base_revision: "base-sha".to_string(),
        files: vec![crate::io::workflow::FileChange {
            content: content.to_string(),
            kind: crate::io::workflow::FileChangeKind::Create,
            path: "citations/work-items/7/result/CITATION.cff".to_string(),
        }],
        reports: vec![],
    };
    for _ in 0..2 {
        let _ = changes("version: 1").publish_reconciled("main", &options, &queue).await.unwrap_err();
    }
    assert!(state.branch_head.lock().unwrap().is_some());
    let first = changes("version: 1").publish_reconciled("main", &options, &queue).await.unwrap();
    let corrected = changes("version: 2").publish_reconciled("main", &options, &queue).await.unwrap();
    assert_eq!(first.branch, corrected.branch);
    assert_eq!(first.merge_request.iid, corrected.merge_request.iid);
    assert_eq!(*state.branch_creates.lock().unwrap(), 1);
    assert_eq!(*state.commit_creates.lock().unwrap(), 2);
    assert_eq!(*state.merge_request_creates.lock().unwrap(), 1);
    assert_eq!(state.content.lock().unwrap().as_deref(), Some("version: 2"));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_merge_request_analysis_does_not_publish_for_stale_sha() {
    let router = Router::new().route(
        "/api/v4/projects/30/merge_requests/7",
        get(|| async {
            Json(serde_json::json!({
                "iid": 7,
                "project_id": 30,
                "source_project_id": 30,
                "sha": "new-sha",
                "title": "Updated",
                "description": "",
                "web_url": "https://gitlab.example.test/project/-/merge_requests/7"
            }))
        }),
    );
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound")
        .domain(&server.base_url)
        .identifier("30")
        .internal_identifier("7")
        .sha("old-sha")
        .build();
    let outcome = review::analyze_merge_request(&options, &crate::analyzer::host::context::AnalysisContext::default())
        .await
        .unwrap();
    assert!(matches!(
        outcome,
        review::MergeRequestAnalysisOutcome::Stale {
            queued_sha,
            current_sha
        } if queued_sha == "old-sha" && current_sha == "new-sha"
    ));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_merge_request_analysis_fails_status_for_oversized_supported_file() {
    let state = MergeRequestWorkflowState::default();
    let router = Router::new()
        .route("/api/v4/user", get(|| async { Json(serde_json::json!({"id": 500})) }))
        .route(
            "/api/v4/projects/30/merge_requests/7",
            get(|| async {
                Json(serde_json::json!({
                    "iid": 7,
                    "project_id": 30,
                    "source_project_id": 30,
                    "sha": "large-sha",
                    "title": "Large metadata",
                    "description": "",
                    "web_url": "https://gitlab.example.test/project/-/merge_requests/7"
                }))
            }),
        )
        .route(
            "/api/v4/projects/30/merge_requests/7/diffs",
            get(|| async { Json(serde_json::json!([{"old_path":"large.json","new_path":"large.json"}])) }),
        )
        .route(
            "/api/v4/projects/30/repository/files/large.json",
            get(|| async {
                Json(serde_json::json!({
                    "file_path": "large.json",
                    "size": 1_048_577,
                    "encoding": "base64",
                    "content": ""
                }))
            }),
        )
        .route(
            "/api/v4/projects/30/statuses/large-sha",
            post(
                |State(state): State<MergeRequestWorkflowState>, Json(body): Json<serde_json::Value>| async move {
                    state.statuses.lock().unwrap().push(body.clone());
                    Json(serde_json::json!({
                        "name": format!("{APPLICATION}/check"),
                        "sha": "large-sha",
                        "status": body.get("state").and_then(serde_json::Value::as_str).unwrap_or_default(),
                        "description": body.get("description").cloned(),
                        "target_url": body.get("target_url").cloned()
                    }))
                },
            ),
        )
        .route(
            "/api/v4/projects/30/merge_requests/7/notes",
            get(|| async { Json(serde_json::json!([])) }).post(
                |State(state): State<MergeRequestWorkflowState>, Json(body): Json<serde_json::Value>| async move {
                    state.notes.lock().unwrap().push(body.clone());
                    Json(serde_json::json!({"id": 89, "body": body.get("body").and_then(serde_json::Value::as_str).unwrap_or_default()}))
                },
            ),
        )
        .with_state(state.clone());
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound")
        .domain(&server.base_url)
        .identifier("30")
        .internal_identifier("7")
        .sha("large-sha")
        .build();
    let check_options = crate::analyzer::host::context::AnalysisContext {
        analysis: crate::analyzer::host::context::AnalysisOptions {
            skip: vec![
                "schema".to_string(),
                "link".to_string(),
                "prose".to_string(),
                "quality".to_string(),
                "fair".to_string(),
                "readability".to_string(),
                "crosswalk".to_string(),
            ],
            ..crate::analyzer::host::context::AnalysisOptions::default()
        },
        execution: crate::analyzer::host::context::ExecutionOptions {
            common: acorn_core::options::CommonRuntime {
                offline: true,
                ..Default::default()
            },
            ..crate::analyzer::host::context::ExecutionOptions::default()
        },
        ..crate::analyzer::host::context::AnalysisContext::default()
    };
    let outcome = review::analyze_merge_request(&options, &check_options).await.unwrap();
    let review::MergeRequestAnalysisOutcome::Published(report) = outcome else {
        panic!("expected published analysis");
    };
    assert!(report.failed());
    assert!(!report.requires_retry());
    assert_eq!(report.inputs.iter().filter(|input| input.is_failed()).count(), 1);
    let statuses = state.statuses.lock().unwrap().clone();
    assert_eq!(statuses[0].get("state").and_then(serde_json::Value::as_str), Some("running"));
    assert_eq!(statuses[1].get("state").and_then(serde_json::Value::as_str), Some("failed"));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_merge_request_analysis_fetches_fork_files_and_publishes_one_report() {
    let state = MergeRequestWorkflowState::default();
    let router = Router::new()
        .route("/api/v4/user", get(|| async { Json(serde_json::json!({"id": 500})) }))
        .route(
            "/api/v4/projects/30/merge_requests/7",
            get(|| async {
                Json(serde_json::json!({
                    "iid": 7,
                    "project_id": 30,
                    "source_project_id": 31,
                    "sha": "abc123",
                    "title": "Document the result",
                    "description": "Artifact doi:10.1234/example",
                    "web_url": "https://gitlab.example.test/project/-/merge_requests/7"
                }))
            }),
        )
        .route(
            "/api/v4/projects/30/merge_requests/7/diffs",
            get(|| async {
                Json(serde_json::json!([
                    {"old_path":"old-activity.json","new_path":"activity.json","renamed_file":true},
                    {"old_path":"notes.md","new_path":"notes.md","collapsed":true},
                    {"old_path":"CITATION.cff","new_path":"CITATION.cff","too_large":true},
                    {"old_path":"removed.cff","new_path":"removed.cff","deleted_file":true},
                    {"old_path":"generated.md","new_path":"generated.md","generated_file":true},
                    {"old_path":"src/main.rs","new_path":"src/main.rs"}
                ]))
            }),
        )
        .route(
            "/api/v4/projects/31/repository/files/activity.json",
            get(|State(state): State<MergeRequestWorkflowState>, request: Request| async move {
                state
                    .file_queries
                    .lock()
                    .unwrap()
                    .push(request.uri().query().unwrap_or_default().to_string());
                Json(serde_json::json!({
                    "file_path": "activity.json",
                    "size": 2,
                    "encoding": "base64",
                    "content": data_encoding::BASE64.encode(b"{}")
                }))
            }),
        )
        .route(
            "/api/v4/projects/31/statuses/abc123",
            post(
                |State(state): State<MergeRequestWorkflowState>, Json(body): Json<serde_json::Value>| async move {
                    state.statuses.lock().unwrap().push(body.clone());
                    Json(serde_json::json!({
                        "name": format!("{APPLICATION}/check"),
                        "sha": "abc123",
                        "status": body.get("state").and_then(serde_json::Value::as_str).unwrap_or_default(),
                        "description": body.get("description").cloned(),
                        "target_url": body.get("target_url").cloned()
                    }))
                },
            ),
        )
        .route(
            "/api/v4/projects/31/repository/files/notes.md",
            get(|| async {
                Json(serde_json::json!({
                    "file_path": "notes.md",
                    "size": 14,
                    "encoding": "base64",
                    "content": data_encoding::BASE64.encode(b"Project notes")
                }))
            }),
        )
        .route(
            "/api/v4/projects/31/repository/files/CITATION.cff",
            get(|| async {
                Json(serde_json::json!({
                    "file_path": "CITATION.cff",
                    "size": 31,
                    "encoding": "base64",
                    "content": data_encoding::BASE64.encode(b"title: Result\nauthors:\n- name: A")
                }))
            }),
        )
        .route(
            "/api/v4/projects/30/merge_requests/7/notes",
            get(|| async { Json(serde_json::json!([])) }).post(
                |State(state): State<MergeRequestWorkflowState>, Json(body): Json<serde_json::Value>| async move {
                    state.notes.lock().unwrap().push(body.clone());
                    Json(serde_json::json!({"id": 88, "body": body.get("body").and_then(serde_json::Value::as_str).unwrap_or_default()}))
                },
            ),
        )
        .with_state(state.clone());
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound")
        .domain(&server.base_url)
        .identifier("30")
        .internal_identifier("7")
        .sha("abc123")
        .build();
    let check_options = crate::analyzer::host::context::AnalysisContext {
        analysis: crate::analyzer::host::context::AnalysisOptions {
            skip: vec![
                "schema".to_string(),
                "link".to_string(),
                "prose".to_string(),
                "quality".to_string(),
                "fair".to_string(),
                "readability".to_string(),
                "crosswalk".to_string(),
            ],
            ..crate::analyzer::host::context::AnalysisOptions::default()
        },
        execution: crate::analyzer::host::context::ExecutionOptions {
            common: acorn_core::options::CommonRuntime {
                offline: true,
                ..Default::default()
            },
            ..crate::analyzer::host::context::ExecutionOptions::default()
        },
        ..crate::analyzer::host::context::AnalysisContext::default()
    };
    let outcome = review::analyze_merge_request(&options, &check_options).await.unwrap();
    let review::MergeRequestAnalysisOutcome::Published(report) = outcome else {
        panic!("expected published analysis");
    };
    assert_eq!(report.inputs.iter().filter(|input| input.is_checked()).count(), 3);
    assert_eq!(report.inputs.iter().filter(|input| input.is_skipped()).count(), 3);
    assert_eq!(report.citation_candidates.len(), 1);
    assert_eq!(state.file_queries.lock().unwrap().as_slice(), ["ref=abc123"]);
    let statuses = state.statuses.lock().unwrap().clone();
    assert_eq!(statuses.len(), 2);
    assert_eq!(statuses[0].get("state").and_then(serde_json::Value::as_str), Some("running"));
    assert_eq!(statuses[1].get("state").and_then(serde_json::Value::as_str), Some("success"));
    let notes = state.notes.lock().unwrap().clone();
    assert_eq!(notes.len(), 1);
    let note = notes[0].get("body").and_then(serde_json::Value::as_str).unwrap_or_default();
    assert!(note.contains(MERGE_REQUEST_REPORT_MARKER));
    assert!(note.contains("`abc123`"));
    assert!(note.contains("doi"));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_merge_request_analysis_retries_inaccessible_supported_content_after_reporting_failure() {
    let state = MergeRequestWorkflowState::default();
    let router = Router::new()
        .route("/api/v4/user", get(|| async { Json(serde_json::json!({"id": 500})) }))
        .route(
            "/api/v4/projects/30/merge_requests/7",
            get(|| async {
                Json(serde_json::json!({
                    "iid": 7,
                    "project_id": 30,
                    "source_project_id": 30,
                    "sha": "missing-sha",
                    "title": "Missing metadata",
                    "description": "",
                    "web_url": "https://gitlab.example.test/project/-/merge_requests/7"
                }))
            }),
        )
        .route(
            "/api/v4/projects/30/merge_requests/7/diffs",
            get(|| async { Json(serde_json::json!([{"old_path":"missing.json","new_path":"missing.json"}])) }),
        )
        .route(
            "/api/v4/projects/30/repository/files/missing.json",
            get(|| async {
                (
                    axum::http::StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"message":"404 File Not Found"})),
                )
            }),
        )
        .route(
            "/api/v4/projects/30/statuses/missing-sha",
            post(
                |State(state): State<MergeRequestWorkflowState>, Json(body): Json<serde_json::Value>| async move {
                    state.statuses.lock().unwrap().push(body.clone());
                    Json(serde_json::json!({
                        "name": format!("{APPLICATION}/check"),
                        "sha": "missing-sha",
                        "status": body.get("state").and_then(serde_json::Value::as_str).unwrap_or_default(),
                        "description": body.get("description").cloned(),
                        "target_url": body.get("target_url").cloned()
                    }))
                },
            ),
        )
        .route(
            "/api/v4/projects/30/merge_requests/7/notes",
            get(|| async { Json(serde_json::json!([])) }).post(
                |State(state): State<MergeRequestWorkflowState>, Json(body): Json<serde_json::Value>| async move {
                    state.notes.lock().unwrap().push(body.clone());
                    Json(serde_json::json!({"id": 90, "body": body.get("body").and_then(serde_json::Value::as_str).unwrap_or_default()}))
                },
            ),
        )
        .with_state(state.clone());
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound")
        .domain(&server.base_url)
        .identifier("30")
        .internal_identifier("7")
        .sha("missing-sha")
        .build();
    let check_options = crate::analyzer::host::context::AnalysisContext {
        analysis: crate::analyzer::host::context::AnalysisOptions {
            skip: vec![
                "schema".to_string(),
                "link".to_string(),
                "prose".to_string(),
                "quality".to_string(),
                "fair".to_string(),
                "readability".to_string(),
                "crosswalk".to_string(),
            ],
            ..crate::analyzer::host::context::AnalysisOptions::default()
        },
        execution: crate::analyzer::host::context::ExecutionOptions {
            common: acorn_core::options::CommonRuntime {
                offline: true,
                ..Default::default()
            },
            ..crate::analyzer::host::context::ExecutionOptions::default()
        },
        ..crate::analyzer::host::context::AnalysisContext::default()
    };
    let error = review::analyze_merge_request(&options, &check_options).await.unwrap_err().to_string();
    assert!(error.contains("could not be fetched"));
    let statuses = state.statuses.lock().unwrap().clone();
    assert_eq!(statuses[0].get("state").and_then(serde_json::Value::as_str), Some("running"));
    assert_eq!(statuses[1].get("state").and_then(serde_json::Value::as_str), Some("failed"));
    assert_eq!(state.notes.lock().unwrap().len(), 1);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_merge_request_diffs_reads_every_page() {
    let router = Router::new().route(
        "/api/v4/projects/30/merge_requests/7/diffs",
        get(|request: Request| async move {
            let page = request
                .uri()
                .query()
                .and_then(|query| query.split('&').find_map(|part| part.strip_prefix("page=")))
                .unwrap_or("1");
            let diffs = if page == "1" {
                (0..100)
                    .map(|index| serde_json::json!({"old_path":format!("old-{index}.json"),"new_path":format!("new-{index}.json")}))
                    .collect::<Vec<_>>()
            } else {
                vec![serde_json::json!({"old_path":"last.json","new_path":"last.json"})]
            };
            Json(diffs)
        }),
    );
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound")
        .domain(&server.base_url)
        .identifier("30")
        .internal_identifier("7")
        .build();
    assert_eq!(merge_request_diffs(&options).await.unwrap().len(), 101);
    server.stop().await.unwrap();
}
#[test]
fn test_options_send_configured_token_header() {
    let options = Options::with_token("job-token").token_header(GITLAB_JOB_TOKEN_HEADER).build();
    let headers = Params::new()
        .with_auth(ExposeSecret::expose_secret(options.token()), Some(options.token_header()))
        .build()
        .into_headers();
    assert_eq!(
        headers.get(GITLAB_JOB_TOKEN_HEADER).and_then(|value| value.to_str().ok()),
        Some("job-token")
    );
    assert!(!headers.contains_key(GITLAB_PRIVATE_TOKEN_HEADER));
}
#[test]
fn test_params_to_query_string() {
    let params = vec![param!(KeyValuePair, "per_page", "100"), param!(KeyValuePair, "page", "2")];
    let query = Param::to_query_string::<PaginationKey, api::EmptyField>(params);
    assert_eq!(query, "?per_page=100&page=2");
}
#[test]
fn test_params_to_query_string_with_invalid_fields() {
    let params = vec![param!(KeyValuePair, "every_page", "100"), param!(KeyValuePair, "page", "42")];
    let query = Param::to_query_string::<PaginationKey, api::EmptyField>(params);
    assert_eq!(query, "?page=42");
}
#[test]
fn test_params_to_query_string_with_invalid_values() {
    let params = vec![param!(KeyValuePair, "per_page", "100"), param!(KeyValuePair, "page", "not a number")];
    let query = Param::to_query_string::<PaginationKey, api::EmptyField>(params);
    assert_eq!(query, "?per_page=100");
    let params = vec![param!(KeyValuePair, "per_page", "{}"), param!(KeyValuePair, "page", "not a number")];
    let query = Param::to_query_string::<PaginationKey, api::EmptyField>(params);
    assert!(query.is_empty());
}
#[test]
fn test_parse_tree_paths_response_filters_blob_entries() {
    let json = r#"[
        {"id":"1","name":"README.md","type":"blob","path":"README.md","mode":"100644"},
        {"id":"2","name":"content","type":"tree","path":"content","mode":"040000"}
    ]"#;
    let response: TreeResponse = serde_json::from_str(json).expect("tree entries should parse");
    assert_eq!(response.paths, vec!["README.md".to_string()]);
    assert_eq!(response.entry_count, 2);
}
#[test]
fn test_parse_tree_paths_response_retains_directory_only_page_count() {
    let json = r#"[
        {"id":"1","name":"content","type":"tree","path":"content","mode":"040000"},
        {"id":"2","name":"docs","type":"tree","path":"docs","mode":"040000"}
    ]"#;
    let response: TreeResponse = serde_json::from_str(json).expect("tree entries should parse");
    assert!(response.paths.is_empty());
    assert_eq!(response.entry_count, 2);
}
#[test]
fn test_parse_tree_paths_response_returns_actionable_error_for_non_json() {
    let html = "<!doctype html><html><body>403 Forbidden</body></html>";
    let why = serde_json::from_str::<TreeResponse>(html).expect_err("non-json response should fail with parse error");
    assert!(why.to_string().contains("expected value"));
}
#[test]
fn test_parse_tree_paths_response_returns_error_on_first_page_403() {
    let json = r#"{"message":"403 Forbidden"}"#;
    let response: TreeResponse = serde_json::from_str(json).expect("error payload should deserialize as tree response");
    let why = handle_tree_paths_response(Ok(response), 1).expect_err("first-page forbidden should fail");
    assert!(why.to_string().contains("403 Forbidden"));
}
#[test]
fn test_parse_tree_paths_response_treats_later_page_403_as_terminal() {
    let json = r#"{"message":"403 Forbidden"}"#;
    let response: TreeResponse = serde_json::from_str(json).expect("error payload should deserialize as tree response");
    assert!(response.error.is_some());
    let response = handle_tree_paths_response(Ok(response), 3).expect("later-page forbidden should be terminal");
    assert!(response.paths.is_empty());
}
#[test]
fn test_programming_language_use_response_deserializes_map() {
    let json = r#"{"Rust":98.12,"Makefile":0.5,"Python":0.49}"#;
    let response: ProgrammingLanguageUseResponse = serde_json::from_str(json).expect("should deserialize language usage map");
    assert_eq!(response.languages.len(), 3);
    assert_eq!(response.languages[0].name, "Makefile");
    assert_eq!(response.languages[0].percentage, 0.5);
    assert_eq!(response.languages[1].name, "Python");
    assert_eq!(response.languages[2].name, "Rust");
}
#[test]
fn test_programming_languages_response_parse_filters_programming_only() {
    let data = HashMap::from_iter([
        (
            "Python".to_string(),
            ProgrammingLanguageDetails {
                language_id: Some(303),
                language_type: Some("programming".to_string()),
                color: Some("#3572A5".to_string()),
                group: None,
            },
        ),
        (
            "YAML".to_string(),
            ProgrammingLanguageDetails {
                language_id: Some(407),
                language_type: Some("data".to_string()),
                color: Some("#cb171e".to_string()),
                group: None,
            },
        ),
    ]);
    let response = ProgrammingLanguagesResponse::parse(data);
    assert_eq!(response.languages.len(), 1);
    assert_eq!(response.languages[0].name, "Python");
    assert_eq!(response.languages[0].language_id, Some(303));
}
#[tokio::test]
async fn test_project_details_accepts_missing_default_branch() {
    let router = Router::new().route("/api/v4/projects/30", get(|| async { Json(serde_json::json!({"id": 30})) }));
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("").domain(&server.base_url).identifier("30").build();
    let details = project_details(&options)
        .await
        .expect("project details without default branch should resolve");
    assert_eq!(details.identifier, 30);
    assert_eq!(details.default_branch, None);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_project_details_requires_identifier() {
    let options = Options::with_token("").domain("gitlab.com").build();
    let why = project_details(&options)
        .await
        .expect_err("project details without identifier should fail");
    assert!(why.to_string().contains("project identifier is required"));
}
#[tokio::test]
async fn test_project_details_resolves_identifier_and_default_branch() {
    let router = Router::new().route(
        "/api/v4/projects/30",
        get(|| async { Json(serde_json::json!({"id": 30, "default_branch": "main", "path_with_namespace": "group/quest"})) }),
    );
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("").domain(&server.base_url).identifier("30").build();
    let details = project_details(&options).await.expect("project details should resolve");
    assert_eq!(details.identifier, 30);
    assert_eq!(details.default_branch.as_deref(), Some("main"));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_project_webhook_registration_refreshes_exact_existing_hook_credential() {
    let bodies = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
    let router = Router::new()
        .route("/api/v4/version", get(|| async { Json(serde_json::json!({"version":"19.1.0"})) }))
        .route(
            "/api/v4/projects/30/hooks",
            get(|| async {
                Json(serde_json::json!([{
                    "id": 9,
                    "url": "https://bot.example.org/webhooks/gitlab",
                    "merge_requests_events": true,
                    "note_events": true,
                    "enable_ssl_verification": true,
                    "signing_token_present": true
                }]))
            }),
        )
        .route(
            "/api/v4/projects/30/hooks/9",
            put(
                |State(bodies): State<Arc<Mutex<Vec<serde_json::Value>>>>, Json(body): Json<serde_json::Value>| async move {
                    bodies.lock().unwrap().push(body);
                    Json(serde_json::json!({
                        "id": 9,
                        "url": "https://bot.example.org/webhooks/gitlab",
                        "merge_requests_events": true,
                        "note_events": true,
                        "enable_ssl_verification": true,
                        "signing_token_present": true
                    }))
                },
            ),
        )
        .with_state(Arc::clone(&bodies));
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound").domain(&server.base_url).identifier("30").build();
    let webhook_options = WebhookOptions::new(Some("https://bot.example.org/"), Some("legacy-secret"), Some("whsec_c2lnbmluZw=="));
    let result = upsert_project_webhook(&options, &webhook_options).await.unwrap();
    assert!(!result.created);
    assert_eq!(result.hook.id, 9);
    assert_eq!(
        bodies.lock().unwrap()[0].get("signing_token").and_then(serde_json::Value::as_str),
        Some("whsec_c2lnbmluZw==")
    );
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_project_webhook_registration_surfaces_permission_error_without_credentials() {
    let router = Router::new()
        .route("/api/v4/version", get(|| async { Json(serde_json::json!({"version":"18.11.2"})) }))
        .route(
            "/api/v4/projects/30/hooks",
            get(|| async { (axum::http::StatusCode::FORBIDDEN, Json(serde_json::json!({"message":"403 Forbidden"}))) }),
        );
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound-secret").domain(&server.base_url).identifier("30").build();
    let webhook_options = WebhookOptions::new(Some("https://bot.example.org"), Some("inbound-secret"), Some("whsec_c2lnbmluZw=="));
    let error = upsert_project_webhook(&options, &webhook_options).await.unwrap_err().to_string();
    assert!(!error.contains("outbound-secret"));
    assert!(!error.contains("inbound-secret"));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_project_webhook_registration_updates_mismatched_hook_without_duplication() {
    let bodies = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
    let router = Router::new()
        .route("/api/v4/version", get(|| async { Json(serde_json::json!({"version":"19.1.0"})) }))
        .route(
            "/api/v4/projects/30/hooks",
            get(|| async {
                Json(serde_json::json!([{
                    "id": 10,
                    "url": "https://bot.example.org/webhooks/gitlab",
                    "merge_requests_events": false,
                    "note_events": true,
                    "enable_ssl_verification": true
                }]))
            }),
        )
        .route(
            "/api/v4/projects/30/hooks/10",
            put(
                |State(bodies): State<Arc<Mutex<Vec<serde_json::Value>>>>, Json(body): Json<serde_json::Value>| async move {
                    bodies.lock().unwrap().push(body);
                    Json(serde_json::json!({
                        "id": 10,
                        "url": "https://bot.example.org/webhooks/gitlab",
                        "merge_requests_events": true,
                        "note_events": true,
                        "enable_ssl_verification": true,
                        "signing_token_present": true
                    }))
                },
            ),
        )
        .with_state(Arc::clone(&bodies));
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound").domain(&server.base_url).identifier("30").build();
    let webhook_options = WebhookOptions::new(Some("https://bot.example.org"), Some("legacy-secret"), Some("whsec_c2lnbmluZw=="));
    let result = upsert_project_webhook(&options, &webhook_options).await.unwrap();
    assert!(!result.created);
    assert_eq!(bodies.lock().unwrap().len(), 1);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_project_webhook_registration_uses_legacy_token_before_gitlab_19() {
    let bodies = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
    let router = Router::new()
        .route(
            "/api/v4/version",
            get(|| async { Json(serde_json::json!({"version":"18.11.2","revision":"abc"})) }),
        )
        .route(
            "/api/v4/projects/30/hooks",
            get(|| async { Json(serde_json::json!([])) }).post(
                |State(bodies): State<Arc<Mutex<Vec<serde_json::Value>>>>, Json(body): Json<serde_json::Value>| async move {
                    bodies.lock().unwrap().push(body);
                    Json(serde_json::json!({
                        "id": 8,
                        "url": "https://bot.example.org/webhooks/gitlab",
                        "merge_requests_events": true,
                        "note_events": true,
                        "enable_ssl_verification": true,
                        "token_present": true
                    }))
                },
            ),
        )
        .with_state(Arc::clone(&bodies));
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound").domain(&server.base_url).identifier("30").build();
    let webhook_options = WebhookOptions::new(Some("https://bot.example.org"), Some("legacy-secret"), Some("whsec_c2lnbmluZw=="));
    upsert_project_webhook(&options, &webhook_options).await.unwrap();
    let body = bodies.lock().unwrap()[0].clone();
    assert_eq!(body.get("token").and_then(serde_json::Value::as_str), Some("legacy-secret"));
    assert!(body.get("signing_token").is_none());
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_project_webhook_registration_uses_signing_token_on_gitlab_19() {
    let bodies = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
    let router = Router::new()
        .route(
            "/api/v4/version",
            get(|| async { Json(serde_json::json!({"version":"19.1.0","revision":"abc"})) }),
        )
        .route(
            "/api/v4/projects/30/hooks",
            get(|| async { Json(serde_json::json!([])) }).post(
                |State(bodies): State<Arc<Mutex<Vec<serde_json::Value>>>>, Json(body): Json<serde_json::Value>| async move {
                    bodies.lock().unwrap().push(body);
                    Json(serde_json::json!({
                        "id": 7,
                        "url": "https://bot.example.org/webhooks/gitlab",
                        "merge_requests_events": true,
                        "note_events": true,
                        "enable_ssl_verification": true,
                        "signing_token_present": true
                    }))
                },
            ),
        )
        .with_state(Arc::clone(&bodies));
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound").domain(&server.base_url).identifier("30").build();
    let webhook_options = WebhookOptions::new(Some("https://bot.example.org"), Some("legacy-secret"), Some("whsec_c2lnbmluZw=="));
    let result = upsert_project_webhook(&options, &webhook_options).await.unwrap();
    let body = bodies.lock().unwrap()[0].clone();
    assert!(result.created);
    assert_eq!(body.get("signing_token").and_then(serde_json::Value::as_str), Some("whsec_c2lnbmluZw=="));
    assert!(body.get("token").is_none());
    assert_eq!(body.get("merge_requests_events").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(body.get("note_events").and_then(serde_json::Value::as_bool), Some(true));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_project_webhooks_reads_every_page() {
    let router = Router::new().route(
        "/api/v4/projects/30/hooks",
        get(|request: Request| async move {
            let page = request
                .uri()
                .query()
                .and_then(|query| query.split('&').find_map(|part| part.strip_prefix("page=")))
                .unwrap_or("1");
            let hooks = if page == "1" {
                (0..100)
                    .map(|id| {
                        serde_json::json!({
                            "id": id,
                            "url": format!("https://bot.example.org/{id}"),
                            "merge_requests_events": true,
                            "note_events": true,
                            "enable_ssl_verification": true
                        })
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![serde_json::json!({
                    "id": 100,
                    "url": "https://bot.example.org/webhooks/gitlab",
                    "merge_requests_events": true,
                    "note_events": true,
                    "enable_ssl_verification": true
                })]
            };
            Json(hooks)
        }),
    );
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound").domain(&server.base_url).identifier("30").build();
    assert_eq!(project_webhooks(&options).await.unwrap().len(), 101);
    server.stop().await.unwrap();
}
#[test]
fn test_query_string() {
    let param = param!(KeyValuePair, "per_page", "100");
    let query = param.to_string::<PaginationKey, api::EmptyField>();
    assert_eq!(query, "per_page=100");
}
#[tokio::test]
async fn test_repository_change_set_publishes_bot_branch_commit_and_draft_merge_request() {
    let state = MergeRequestWorkflowState::default();
    let router = Router::new()
        .route(
            "/api/v4/projects/30/repository/branches",
            post(
                |State(state): State<MergeRequestWorkflowState>, Json(body): Json<serde_json::Value>| async move {
                    state.automation_requests.lock().unwrap().push(body.clone());
                    Json(serde_json::json!({
                        "name": body["branch"],
                        "commit": {"id": "abcdef0123456789"},
                        "protected": false
                    }))
                },
            ),
        )
        .route(
            "/api/v4/projects/30/repository/commits",
            post(
                |State(state): State<MergeRequestWorkflowState>, Json(body): Json<serde_json::Value>| async move {
                    state.automation_requests.lock().unwrap().push(body);
                    Json(serde_json::json!({"id": "commit-1", "web_url": "https://gitlab.example/commit-1"}))
                },
            ),
        )
        .route(
            "/api/v4/projects/30/merge_requests",
            get(|| async { Json(serde_json::json!([])) }).post(
                |State(state): State<MergeRequestWorkflowState>, Json(body): Json<serde_json::Value>| async move {
                    state.automation_requests.lock().unwrap().push(body.clone());
                    Json(serde_json::json!({
                        "iid": 9,
                        "source_branch": body["source_branch"],
                        "state": "opened",
                        "web_url": "https://gitlab.example/merge_requests/9"
                    }))
                },
            ),
        )
        .with_state(state.clone());
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound")
        .domain(&server.base_url)
        .identifier("30")
        .workflow("repository-quality")
        .build();
    let changes = crate::io::workflow::ProcessRepositoryResponse {
        base_revision: "abcdef0123456789".to_string(),
        files: vec![crate::io::workflow::FileChange {
            content: "{\n  \"name\": \"example\"\n}\n".to_string(),
            kind: crate::io::workflow::FileChangeKind::Update,
            path: "metadata.json".to_string(),
        }],
        reports: vec![],
    };
    let result = changes.publish("feature", &options).await.unwrap();
    assert_eq!(result.branch, "acorn/repository-quality/abcdef012345");
    assert_eq!(result.commit.id, "commit-1");
    assert_eq!(result.merge_request.iid, 9);
    let requests = state.automation_requests.lock().unwrap().clone();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[1]["actions"][0]["last_commit_id"], "abcdef0123456789");
    assert!(requests[2]["description"].as_str().unwrap().contains("- `metadata.json` (update)"));
    assert_eq!(requests[2]["title"], "Draft: Apply ACORN repository-quality");
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_repository_file_reports_non_json_error_status() {
    let router = Router::new().route(
        "/api/v4/projects/30/repository/files/activity.json",
        get(|| async { (axum::http::StatusCode::FORBIDDEN, "Access is restricted") }),
    );
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("")
        .domain(&server.base_url)
        .identifier("30")
        .path("activity.json")
        .sha("HEAD")
        .build();
    let why = repository_file(&options).await.expect_err("forbidden repository file should fail");
    assert!(why.to_string().contains("HTTP 403: Access is restricted"));
    assert!(!why.to_string().contains("missing field `file_path`"));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_repository_tree_reports_first_page_empty_403_status() {
    let router = Router::new().route("/api/v4/projects/30/repository/tree", get(|| async { axum::http::StatusCode::FORBIDDEN }));
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("").domain(&server.base_url).identifier("30").build();
    let why = repository_tree(&options).await.expect_err("first-page forbidden should fail");
    assert!(why.to_string().contains("HTTP 403"));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_repository_tree_reports_later_page_403_with_reason() {
    let router = Router::new().route(
        "/api/v4/projects/30/repository/tree",
        get(|| async { (axum::http::StatusCode::FORBIDDEN, "Access is restricted") }),
    );
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("").domain(&server.base_url).identifier("30").page(2).build();
    let why = repository_tree(&options)
        .await
        .expect_err("later-page forbidden with a reason should fail");
    assert!(why.to_string().contains("HTTP 403: Access is restricted"));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_repository_tree_treats_later_page_empty_403_as_terminal() {
    let router = Router::new().route("/api/v4/projects/30/repository/tree", get(|| async { axum::http::StatusCode::FORBIDDEN }));
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("").domain(&server.base_url).identifier("30").page(2).build();
    let response = repository_tree(&options).await.expect("later-page forbidden should be terminal");
    assert!(response.paths.is_empty());
    assert_eq!(response.entry_count, 0);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_repository_tree_uses_exact_version_reference() {
    let query = Arc::new(Mutex::new(String::new()));
    let router = Router::new()
        .route(
            "/api/v4/projects/30/repository/tree",
            get(|State(query): State<Arc<Mutex<String>>>, request: Request| async move {
                *query.lock().unwrap() = request.uri().query().unwrap_or_default().to_string();
                Json(serde_json::json!([]))
            }),
        )
        .with_state(query.clone());
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("").domain(&server.base_url).identifier("30").sha("abc123").build();
    repository_tree(&options).await.expect("tree request should succeed");
    assert!(query.lock().unwrap().contains("ref=abc123"));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_repository_workflow_policy_denies_forks_before_requests() {
    let details = MergeRequestDetails {
        iid: 7,
        author: None,
        project_id: 30,
        source_project_id: Some(31),
        sha: "abc123".to_string(),
        source_branch: "feature".to_string(),
        target_branch: "main".to_string(),
        title: "Fork".to_string(),
        description: String::new(),
        web_url: "https://gitlab.example/merge_requests/7".to_string(),
        diff_refs: None,
    };
    let options = Options::with_token("outbound")
        .domain("http://127.0.0.1:1")
        .identifier("30")
        .internal_identifier("7")
        .build();
    let error = details
        .process("repository-quality", &options, &DeploymentConfig::default(), None)
        .await
        .unwrap_err()
        .to_string();
    assert!(error.contains("requires an explicit destination and fork policy"));
}
#[tokio::test]
async fn test_repository_workflow_policy_rejects_paths_and_effects_before_mutation() {
    for (path, effects, expected) in [
        ("metadata.json", "[note]", "not allowed to produce"),
        ("private/data.json", "[branch, commit, draft-merge-request]", "path is not declared"),
    ] {
        let mutations = Arc::new(AtomicUsize::new(0));
        let diff_path = path.to_string();
        let content = data_encoding::BASE64.encode(br#"{"z":1,"a":2}"#);
        let router = Router::new()
            .route(
                "/api/v4/projects/30/merge_requests/7/diffs",
                get(move || {
                    let path = diff_path.clone();
                    async move { Json(serde_json::json!([{"old_path":path,"new_path":path}])) }
                }),
            )
            .route(
                "/api/v4/projects/30/repository/files/{*path}",
                get(move || {
                    let content = content.clone();
                    async move {
                        Json(serde_json::json!({
                            "file_path":"metadata.json",
                            "size":13,
                            "encoding":"base64",
                            "content":content
                        }))
                    }
                }),
            )
            .route(
                "/api/v4/projects/30/repository/branches",
                post(|State(mutations): State<Arc<AtomicUsize>>| async move {
                    mutations.fetch_add(1, Ordering::SeqCst);
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR
                }),
            )
            .route(
                "/api/v4/projects/30/repository/commits",
                post(|State(mutations): State<Arc<AtomicUsize>>| async move {
                    mutations.fetch_add(1, Ordering::SeqCst);
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR
                }),
            )
            .route(
                "/api/v4/projects/30/merge_requests",
                post(|State(mutations): State<Arc<AtomicUsize>>| async move {
                    mutations.fetch_add(1, Ordering::SeqCst);
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR
                }),
            )
            .with_state(Arc::clone(&mutations));
        let server = TestServer::start(router).await.unwrap();
        let config = DeploymentConfig::from_yaml(&format!(
            "workflows:\n  repository-quality:\n    paths: ['metadata/*.json', 'metadata.json']\n    formats: [json]\n    actions: [validate, format]\n    outputs: {{mode: source-and-artifacts}}\n    effects: {effects}\n"
        ))
        .unwrap();
        let details = MergeRequestDetails {
            iid: 7,
            author: None,
            project_id: 30,
            source_project_id: Some(30),
            sha: "abc123".to_string(),
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            title: "Policy".to_string(),
            description: String::new(),
            web_url: "https://gitlab.example/merge_requests/7".to_string(),
            diff_refs: None,
        };
        let options = Options::with_token("outbound")
            .domain(&server.base_url)
            .identifier("30")
            .internal_identifier("7")
            .build();
        let error = details
            .process("repository-quality", &options, &config, None)
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected));
        assert_eq!(mutations.load(Ordering::SeqCst), 0);
        server.stop().await.unwrap();
    }
}
#[tokio::test]
async fn test_upsert_merge_request_note_updates_existing_marker() {
    let bodies = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
    let router = Router::new()
        .route("/api/v4/user", get(|| async { Json(serde_json::json!({"id": 500})) }))
        .route(
            "/api/v4/projects/30/merge_requests/7/notes",
            get(|| async {
                Json(serde_json::json!([{
                    "id": 91,
                    "body": "<!-- acorn:merge-request-analysis -->\nold",
                    "author": {"id": 500}
                }]))
            }),
        )
        .route(
            "/api/v4/projects/30/merge_requests/7/notes/91",
            put(
                |State(bodies): State<Arc<Mutex<Vec<serde_json::Value>>>>, Json(body): Json<serde_json::Value>| async move {
                    bodies.lock().unwrap().push(body.clone());
                    Json(serde_json::json!({"id": 91, "body": body.get("body").and_then(serde_json::Value::as_str).unwrap_or_default()}))
                },
            ),
        )
        .with_state(Arc::clone(&bodies));
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound")
        .domain(&server.base_url)
        .identifier("30")
        .internal_identifier("7")
        .build();
    let result = upsert_merge_request_note(&options, MERGE_REQUEST_REPORT_MARKER, "<!-- acorn:merge-request-analysis -->\nnew")
        .await
        .unwrap();
    assert_eq!(result.identifier(), 91);
    assert_eq!(bodies.lock().unwrap().len(), 1);
    server.stop().await.unwrap();
}
#[test]
fn test_webhook_options_validate_reachability_and_credentials() {
    assert!(WebhookOptions::new(Some("not a URL"), Some("legacy-secret"), Some("signing-secret"))
        .validate()
        .is_err());
    assert!(WebhookOptions::new(Some("https://bot.example.org"), None, None).validate().is_err());
    assert!(
        WebhookOptions::new(Some("https://bot.example.org"), Some("legacy-secret"), Some("signing-secret"))
            .validate()
            .is_ok()
    );
}
#[cfg(test)]
mod bot;
mod database;
#[cfg(feature = "analysis")]
mod intake;
mod jsonl;
mod package;
