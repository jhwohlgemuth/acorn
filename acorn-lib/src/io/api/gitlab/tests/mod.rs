#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::*;
use crate::io::api::gitlab::{
    handle_tree_paths_response, PaginationKey, ProgrammingLanguageDetails, ProgrammingLanguageUseResponse, ProgrammingLanguagesResponse, TreeResponse,
};
use crate::io::api::{self, sluggify};
use crate::param;
use crate::prelude::{HashMap, Mutex};
use crate::test::server::TestServer;
use crate::util::constants::app::MERGE_REQUEST_REPORT_MARKER;
use alloc::sync::Arc;
use axum::extract::{Request, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};

#[derive(Clone, Default)]
struct MergeRequestWorkflowState {
    statuses: Arc<Mutex<Vec<serde_json::Value>>>,
    notes: Arc<Mutex<Vec<serde_json::Value>>>,
    file_queries: Arc<Mutex<Vec<String>>>,
}

#[test]
fn test_creator_slug_normalizes_and_falls_back() {
    assert_eq!(sluggify(" Alice.Example ", 42), "alice-example");
    assert_eq!(sluggify("---", 42), "user-42");
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
#[test]
fn test_query_string() {
    let param = param!(KeyValuePair, "per_page", "100");
    let query = param.to_string::<PaginationKey, api::EmptyField>();
    assert_eq!(query, "per_page=100");
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
fn test_parse_tree_paths_response_filters_blob_entries() {
    let json = r#"[
        {"id":"1","name":"README.md","type":"blob","path":"README.md","mode":"100644"},
        {"id":"2","name":"content","type":"tree","path":"content","mode":"040000"}
    ]"#;
    let response: TreeResponse = serde_json::from_str(json).expect("tree entries should parse");
    assert_eq!(response.paths, vec!["README.md".to_string()]);
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
fn test_parse_tree_paths_response_returns_error_on_first_page_403() {
    let json = r#"{"message":"403 Forbidden"}"#;
    let response: TreeResponse = serde_json::from_str(json).expect("error payload should deserialize as tree response");
    let why = handle_tree_paths_response(Ok(response), 1).expect_err("first-page forbidden should fail");
    assert!(why.to_string().contains("403 Forbidden"));
}
#[test]
fn test_parse_tree_paths_response_returns_actionable_error_for_non_json() {
    let html = "<!doctype html><html><body>403 Forbidden</body></html>";
    let why = serde_json::from_str::<TreeResponse>(html).expect_err("non-json response should fail with parse error");
    assert!(why.to_string().contains("expected value"));
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
                        "name": "acorn/check",
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
    let check_options = crate::analyzer::CheckOptions {
        offline: true,
        skip: vec![
            "schema".to_string(),
            "link".to_string(),
            "prose".to_string(),
            "quality".to_string(),
            "readability".to_string(),
            "crosswalk".to_string(),
        ],
        ..crate::analyzer::CheckOptions::default()
    };
    let outcome = review::analyze_merge_request(&options, &check_options).await.unwrap();
    let review::MergeRequestAnalysisOutcome::Published(report) = outcome else {
        panic!("expected published analysis");
    };
    assert_eq!(
        report
            .inputs
            .iter()
            .filter(|input| matches!(input, review::Input::Checked { .. }))
            .count(),
        3
    );
    assert_eq!(
        report
            .inputs
            .iter()
            .filter(|input| matches!(input, review::Input::Skipped { .. }))
            .count(),
        3
    );
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
    let outcome = review::analyze_merge_request(&options, &crate::analyzer::CheckOptions::default())
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
                        "name": "acorn/check",
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
    let check_options = crate::analyzer::CheckOptions {
        offline: true,
        skip: vec![
            "schema".to_string(),
            "link".to_string(),
            "prose".to_string(),
            "quality".to_string(),
            "readability".to_string(),
            "crosswalk".to_string(),
        ],
        ..crate::analyzer::CheckOptions::default()
    };
    let outcome = review::analyze_merge_request(&options, &check_options).await.unwrap();
    let review::MergeRequestAnalysisOutcome::Published(report) = outcome else {
        panic!("expected published analysis");
    };
    assert!(report.failed());
    assert!(!report.requires_retry());
    assert_eq!(
        report.inputs.iter().filter(|input| matches!(input, review::Input::Failed { .. })).count(),
        1
    );
    let statuses = state.statuses.lock().unwrap().clone();
    assert_eq!(statuses[0].get("state").and_then(serde_json::Value::as_str), Some("running"));
    assert_eq!(statuses[1].get("state").and_then(serde_json::Value::as_str), Some("failed"));
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
                        "name": "acorn/check",
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
    let check_options = crate::analyzer::CheckOptions {
        offline: true,
        skip: vec![
            "schema".to_string(),
            "link".to_string(),
            "prose".to_string(),
            "quality".to_string(),
            "readability".to_string(),
            "crosswalk".to_string(),
        ],
        ..crate::analyzer::CheckOptions::default()
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
    assert_eq!(result.identifier, 91);
    assert_eq!(bodies.lock().unwrap().len(), 1);
    server.stop().await.unwrap();
}
#[cfg(test)]
mod bot;
