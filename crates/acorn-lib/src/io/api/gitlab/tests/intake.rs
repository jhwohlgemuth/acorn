use super::*;
use crate::analyzer::discovery::ArtifactCandidate;
use crate::io::api::citeas;
use crate::io::api::gitlab::intake::{analyze_work_item, verified_command_author, Authorization};
use crate::io::api::gitlab::{webhook, HookActor, Note, WorkItem, WorkItemUser};
use crate::prelude::Mutex;
use crate::schema::pid::Identifier;
use crate::schema::standard::cff::Cff;
use crate::test::server::TestServer;
use crate::util::constants::app::WORK_ITEM_REPORT_MARKER;
use alloc::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};

#[derive(Clone, Default)]
struct TestState {
    written: Arc<Mutex<Vec<serde_json::Value>>>,
    repository_url: Arc<Mutex<String>>,
}

fn note_json(identifier: u64, body: &str, author_id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": identifier,
        "body": body,
        "author": {"id": author_id, "username": format!("user-{author_id}"), "bot": author_id == 99},
        "system": false,
        "confidential": false,
        "internal": false
    })
}

fn written_note(body: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "id": 80,
        "body": body.get("body").and_then(serde_json::Value::as_str).unwrap_or_default(),
        "author": {"id": 99, "username": "acorn-bot", "bot": true},
        "system": false,
        "confidential": false,
        "internal": false
    })
}
#[test]
fn content_excludes_commands_and_non_human_or_later_notes() {
    let item = WorkItem {
        identifier: 10,
        iid: 7,
        project_id: 30,
        title: "Result".to_string(),
        description: "doi:10.1234/example".to_string(),
        author: WorkItemUser::new(1, "user-1", false),
        issue_type: "issue".to_string(),
        confidential: false,
    };
    let note = |identifier, body: &str, author, system| Note::WorkItem {
        identifier,
        body: body.to_string(),
        author,
        system,
        confidential: false,
        internal: false,
    };
    let content = item.content(
        &[
            note(2, "Human context", WorkItemUser::new(2, "user-2", false), false),
            note(3, "/acorn check", WorkItemUser::new(2, "user-2", false), false),
            note(4, "Bot context", WorkItemUser::new(9, "user-9", true), false),
            note(5, "System context", WorkItemUser::new(2, "user-2", false), true),
            note(11, "Later context", WorkItemUser::new(2, "user-2", false), false),
        ],
        10,
        9,
    );
    assert!(content.contains("Human context"));
    assert!(!content.contains("/acorn check"));
    assert!(!content.contains("Bot context"));
    assert!(!content.contains("System context"));
    assert!(!content.contains("Later context"));
}
#[test]
fn embedded_cff_is_complete_and_identifier_only_candidate_is_excluded() {
    let content = "```cff\ncff-version: 1.2.0\ntitle: Existing result\nmessage: Cite this\nauthors:\n  - family-names: Example\n    given-names: Alice\ndoi: 10.1234/example\n```";
    let candidate = ArtifactCandidate::from(Cff::embedded(content).into_iter().next().unwrap());
    assert!(candidate.classify(None).cff.is_some());
    let identifier = Identifier::new("https://example.org/artifact").normalize().unwrap();
    let excluded = ArtifactCandidate {
        identifiers: vec![identifier],
        ..ArtifactCandidate::default()
    }
    .classify(Some("metadata unavailable".to_string()));
    assert_eq!(excluded.missing, ["Title", "Authors"]);
    assert!(excluded.cff.is_none());
}
#[test]
fn citeas_metadata_completes_a_doi_candidate() {
    let identifier = Identifier::new("doi:10.1234/example").normalize().unwrap();
    let candidate = ArtifactCandidate {
        identifiers: vec![identifier],
        ..ArtifactCandidate::default()
    };
    let citations = citeas::Citations {
        metadata: citeas::Metadata {
            title: "Enriched result".to_string(),
            author: vec![citeas::Author {
                given: "Alice".to_string(),
                family: "Example".to_string(),
            }],
            url: "https://doi.org/10.1234/example".to_string(),
            ..citeas::Metadata::default()
        },
        ..citeas::Citations::default()
    };
    let enriched = candidate.apply_citeas(citations);
    assert_eq!(enriched.title.as_deref(), Some("Enriched result"));
    assert_eq!(enriched.authors, ["Alice Example"]);
    assert!(enriched.classify(None).cff.is_some());
}
#[test]
fn command_matching_requires_an_exact_trimmed_line() {
    assert!(webhook::check_requested(" /acorn check \n"));
    assert!(!webhook::check_requested("Please /acorn check"));
    assert!(!webhook::check_requested("/acorn check now"));
    let command = Note::WorkItem {
        identifier: 10,
        body: "/acorn check".to_string(),
        author: WorkItemUser::new(1, "user-1", false),
        system: false,
        confidential: false,
        internal: false,
    };
    assert_eq!(
        verified_command_author(core::slice::from_ref(&command), 10, &HookActor::new(30, 1, "user-1", false)),
        Some(1)
    );
    assert_eq!(verified_command_author(&[command], 10, &HookActor::new(30, 2, "user-2", false)), None);
}
#[tokio::test]
async fn creator_issue_intake_reports_multiple_incomplete_artifacts() {
    let state = TestState::default();
    let router = Router::new()
        .route("/api/v4/user", get(|| async { Json(serde_json::json!({"id": 99})) }))
        .route(
            "/api/v4/projects/30/issues/7",
            get(|| async {
                Json(serde_json::json!({
                    "id": 70,
                    "iid": 7,
                    "project_id": 30,
                    "title": "Research outputs",
                    "description": "https://example.org/first https://example.org/second",
                    "author": {"id": 1, "username": "scientist"},
                    "issue_type": "issue"
                }))
            }),
        )
        .route(
            "/api/v4/projects/30/issues/7/notes",
            get(|| async { Json(serde_json::json!([note_json(2, "Human context", 3), note_json(10, "/acorn check", 1)])) }).post(
                |State(state): State<TestState>, Json(body): Json<serde_json::Value>| async move {
                    state.written.lock().unwrap().push(body.clone());
                    Json(written_note(&body))
                },
            ),
        )
        .with_state(state.clone());
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound")
        .domain(&server.base_url)
        .identifier("30")
        .internal_identifier("7")
        .build();
    let report = analyze_work_item(&options, &HookActor::new(30, 1, "user-1", false), 10).await.unwrap();
    assert_eq!(report.authorization, Authorization::Creator);
    assert_eq!(report.candidates.len(), 2);
    assert!(report.candidates.iter().all(|candidate| candidate.cff.is_none()));
    assert_eq!(state.written.lock().unwrap().len(), 1);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn existing_gitlab_repository_cff_completes_candidate() {
    let state = TestState::default();
    let cff = "cff-version: 1.2.0\ntitle: Repository result\nmessage: Cite this\nauthors:\n  - family-names: Example\n    given-names: Alice\n";
    let encoded_cff = data_encoding::BASE64.encode(cff.as_bytes());
    let router = Router::new()
        .route("/api/v4/user", get(|| async { Json(serde_json::json!({"id": 99})) }))
        .route(
            "/api/v4/projects/30/issues/7",
            get(|State(state): State<TestState>| async move {
                let repository_url = state.repository_url.lock().unwrap().clone();
                Json(serde_json::json!({
                    "id": 70,
                    "iid": 7,
                    "project_id": 30,
                    "title": "Repository output",
                    "description": repository_url,
                    "author": {"id": 1, "username": "scientist"},
                    "issue_type": "issue"
                }))
            }),
        )
        .route(
            "/api/v4/projects/group%2Fproject/repository/files/CITATION.cff",
            get(move || {
                let encoded_cff = encoded_cff.clone();
                async move {
                    Json(serde_json::json!({
                        "file_path": "CITATION.cff",
                        "size": cff.len(),
                        "encoding": "base64",
                        "content": encoded_cff
                    }))
                }
            }),
        )
        .route(
            "/api/v4/projects/30/issues/7/notes",
            get(|| async { Json(serde_json::json!([note_json(10, "/acorn check", 1)])) }).post(
                |State(state): State<TestState>, Json(body): Json<serde_json::Value>| async move {
                    state.written.lock().unwrap().push(body.clone());
                    Json(written_note(&body))
                },
            ),
        )
        .with_state(state.clone());
    let server = TestServer::start(router).await.unwrap();
    *state.repository_url.lock().unwrap() = format!("{}/group/project", server.base_url);
    let options = Options::with_token("outbound")
        .domain(&server.base_url)
        .identifier("30")
        .internal_identifier("7")
        .build();
    let report = analyze_work_item(&options, &HookActor::new(30, 1, "user-1", false), 10).await.unwrap();
    assert_eq!(report.candidates.len(), 1);
    let candidate = report.candidates.first().unwrap();
    assert_eq!(candidate.artifact.title.as_deref(), Some("Repository result"));
    assert!(candidate.cff.is_some());
    server.stop().await.unwrap();
}
#[tokio::test]
async fn maintainer_task_intake_updates_existing_report_with_embedded_cff() {
    let state = TestState::default();
    let cff = "```cff\ncff-version: 1.2.0\ntitle: Existing result\nmessage: Cite this\nauthors:\n  - family-names: Example\n    given-names: Alice\ndoi: 10.1234/example\n```";
    let notes = vec![
        note_json(2, cff, 1),
        note_json(10, "/acorn check", 2),
        note_json(80, WORK_ITEM_REPORT_MARKER, 99),
    ];
    let router = Router::new()
        .route("/api/v4/user", get(|| async { Json(serde_json::json!({"id": 99})) }))
        .route(
            "/api/v4/projects/30/issues/7",
            get(|| async {
                Json(serde_json::json!({
                    "id": 70,
                    "iid": 7,
                    "project_id": 30,
                    "title": "Task result",
                    "description": "",
                    "author": {"id": 1, "username": "scientist"},
                    "issue_type": "task"
                }))
            }),
        )
        .route(
            "/api/v4/projects/30/members/all/2",
            get(|| async { Json(serde_json::json!({"id": 2, "access_level": 40})) }),
        )
        .route(
            "/api/v4/projects/30/issues/7/notes",
            get(move || {
                let notes = notes.clone();
                async move { Json(notes) }
            }),
        )
        .route(
            "/api/v4/projects/30/issues/7/notes/80",
            put(|State(state): State<TestState>, Json(body): Json<serde_json::Value>| async move {
                state.written.lock().unwrap().push(body.clone());
                Json(written_note(&body))
            }),
        )
        .with_state(state.clone());
    let server = TestServer::start(router).await.unwrap();
    let options = Options::with_token("outbound")
        .domain(&server.base_url)
        .identifier("30")
        .internal_identifier("7")
        .build();
    let report = analyze_work_item(&options, &HookActor::new(30, 2, "user-2", false), 10).await.unwrap();
    assert_eq!(report.authorization, Authorization::Maintainer);
    assert_eq!(report.candidates.len(), 1);
    assert!(report.candidates.first().unwrap().cff.is_some());
    {
        let written = state.written.lock().unwrap();
        assert_eq!(written.len(), 1);
        assert!(written
            .first()
            .unwrap()
            .get("body")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|body| body.contains("Existing result")));
    }
    server.stop().await.unwrap();
}
#[tokio::test]
async fn reporter_and_membership_lookup_failure_are_denied_safely() {
    for member_response in [
        (StatusCode::OK, serde_json::json!({"id": 2, "access_level": 20})),
        (StatusCode::INTERNAL_SERVER_ERROR, serde_json::json!({"message": "lookup failed"})),
    ] {
        let state = TestState::default();
        let router = Router::new()
            .route("/api/v4/user", get(|| async { Json(serde_json::json!({"id": 99})) }))
            .route(
                "/api/v4/projects/30/issues/7",
                get(|| async {
                    Json(serde_json::json!({
                        "id": 70,
                        "iid": 7,
                        "project_id": 30,
                        "title": "Restricted result",
                        "description": "https://example.org/private",
                        "author": {"id": 1, "username": "scientist"},
                        "issue_type": "issue"
                    }))
                }),
            )
            .route(
                "/api/v4/projects/30/members/all/2",
                get(move || {
                    let (status, body) = member_response.clone();
                    async move { (status, Json(body)) }
                }),
            )
            .route(
                "/api/v4/projects/30/issues/7/notes",
                get(|| async { Json(serde_json::json!([note_json(10, "/acorn check", 2)])) }).post(
                    |State(state): State<TestState>, Json(body): Json<serde_json::Value>| async move {
                        state.written.lock().unwrap().push(body.clone());
                        Json(written_note(&body))
                    },
                ),
            )
            .with_state(state.clone());
        let server = TestServer::start(router).await.unwrap();
        let options = Options::with_token("outbound")
            .domain(&server.base_url)
            .identifier("30")
            .internal_identifier("7")
            .build();
        let report = analyze_work_item(&options, &HookActor::new(30, 2, "user-2", false), 10).await.unwrap();
        assert!(matches!(report.authorization, Authorization::Denied { .. }));
        assert!(report.candidates.is_empty());
        assert!(state
            .written
            .lock()
            .unwrap()
            .first()
            .unwrap()
            .get("body")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|body| body.contains("No repository changes were made")));
        server.stop().await.unwrap();
    }
}
