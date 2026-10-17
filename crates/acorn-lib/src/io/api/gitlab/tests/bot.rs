use super::*;
use crate::io::api::gitlab::jsonl::{split_snapshot, Snapshot};
use crate::io::api::gitlab::{bot::*, NoteMetadata, UserMetadata};
use crate::io::api::powerautomate::{jsonl::admit_snapshot, PowerAutomateProvider};
use crate::io::api::webhooks::store::{OperationQueue, OperationState};
use crate::test::server::TestServer;
use crate::test::utils;
use acorn_schema::hardware::memory::Memory;
use axum::{extract, http::StatusCode};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::env::temp_dir;

#[derive(Clone)]
enum JsonlRace {
    None,
    Append(String),
    Ambiguous,
    Cleaned,
    Conflict,
    MissingMetadata,
}
#[derive(Clone)]
struct JsonlRepository {
    content: Option<String>,
    head: u64,
    file_commit: u64,
    race: JsonlRace,
    writes: Vec<serde_json::Value>,
}
fn event(created_at: &str, noteable_type: &str) -> EventDetails {
    EventDetails {
        identifier: 100,
        project_id: 30,
        action_name: EventAction::Commented,
        target_id: Some(10),
        target_iid: Some(11),
        target_type: TargetType::Note,
        author_id: 1,
        target_title: "note".to_string(),
        created_at: created_at.to_string(),
        author: user(),
        imported: false,
        imported_from: "none".to_string(),
        push_data: None,
        author_username: "tester".to_string(),
        note: Some(note(noteable_type, 42)),
    }
}
async fn jsonl_bot(content: Option<String>, race: JsonlRace) -> (TestServer, Server, Arc<Mutex<JsonlRepository>>, Config) {
    let repository = Arc::new(Mutex::new(JsonlRepository {
        content,
        head: 99,
        file_commit: 1,
        race,
        writes: vec![],
    }));
    let router = Router::new()
        .route("/api/v4/projects/42/packages", get(|| async { Json(serde_json::json!([])) }))
        .route("/api/v4/projects/42/repository/commits/{revision}", get(|State(repository): State<Arc<Mutex<JsonlRepository>>>| async move {
            Json(serde_json::json!({"id":format!("{:040x}", repository.lock().unwrap().head)}))
        }))
        .route("/api/v4/projects/42/repository/files/{*path}", get(|State(repository): State<Arc<Mutex<JsonlRepository>>>| async move {
            let repository = repository.lock().unwrap();
            match &repository.content {
                | None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"message":"404 File Not Found"}))),
                | Some(content) => (StatusCode::OK, Json(serde_json::json!({"file_path":".acorn/intake/logbook.jsonl", "encoding":"base64", "content":data_encoding::BASE64.encode(content.as_bytes()), "size":content.len(), "last_commit_id":(!matches!(repository.race, JsonlRace::MissingMetadata)).then(|| format!("{:040x}", repository.file_commit))}))),
            }
        }))
        .route("/api/v4/projects/42/repository/commits", post(|State(repository): State<Arc<Mutex<JsonlRepository>>>, Json(body): Json<serde_json::Value>| async move {
            let mut repository = repository.lock().unwrap();
            repository.writes.push(body.clone());
            let action = &body["actions"][0];
            assert_eq!(body["branch"], "main");
            assert_eq!(action["action"], "update");
            assert_eq!(action["file_path"], ".acorn/intake/logbook.jsonl");
            assert_eq!(action["last_commit_id"], format!("{:040x}", repository.file_commit));
            match repository.race.clone() {
                | JsonlRace::Append(appended) => {
                    repository.content.as_mut().unwrap().push_str(&appended);
                    repository.file_commit += 1;
                    repository.head += 1;
                    repository.race = JsonlRace::None;
                    (StatusCode::BAD_REQUEST, Json(serde_json::json!({"message":"concurrent append"})))
                },
                | JsonlRace::Conflict => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"message":"conflict"}))),
                | race => {
                    repository.content = Some(action["content"].as_str().unwrap().to_string());
                    repository.file_commit += 1;
                    repository.head += 1;
                    repository.race = JsonlRace::None;
                    match race {
                        | JsonlRace::Ambiguous => (StatusCode::OK, Json(serde_json::json!({"unexpected":"commit happened but response is unusable"}))),
                        | JsonlRace::Cleaned => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"message":"another bot already cleaned the inbox"}))),
                        | _ => (StatusCode::CREATED, Json(serde_json::json!({"id":format!("{:040x}", repository.head)}))),
                    }
                },
            }
        }))
        .with_state(Arc::clone(&repository));
    let server = TestServer::start(router).await.unwrap();
    let document: ApplicationConfiguration = serde_json::from_value(serde_json::json!({
        "folders":[{"id":"pilot", "members":["scientist@example.org"]}],
        "options":{"powerAutomate":{"callback_destination":"flow", "gitlab_project":"42", "package":"reports", "intake":{"branch":"main", "poll_interval":1}}}
    })).unwrap();
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let mut config = Config::new(options, "127.0.0.1:0".parse().unwrap())
        .with_operation_queue(operation_queue())
        .with_polling_enabled(false);
    config.powerautomate = Some(PowerAutomateProvider::new(document.power_automate().unwrap().clone(), document.registry(), "test").unwrap());
    (server, Server::new(config.clone()), repository, config)
}
fn jsonl_form(id: u8) -> String {
    serde_json::json!({"submissionId":format!("123e4567-e89b-12d3-a456-4266141740{id:02}"), "projectId":"pilot", "submittedAt":"2026-09-18T12:01:00Z", "submitter":{"email":"scientist@example.org"}, "entry":{"timestamp":"2026-09-18T12:00:00Z", "category":"software", "state":"completed", "summary":format!("New functionality {id}")}}).to_string()
}
fn mr_payload(project_id: u64, action: &str) -> serde_json::Value {
    serde_json::json!({
        "object_kind": "merge_request",
        "user": { "id": 1, "username": "alice", "bot": false },
        "project": { "id": project_id },
        "object_attributes": {
            "iid": 5,
            "title": "Add feature",
            "description": "A description",
            "action": action,
            "last_commit": { "id": "abc123" }
        }
    })
}
fn note(noteable_type: &str, noteable_iid: u64) -> NoteMetadata {
    NoteMetadata {
        identifier: 10,
        note_type: None,
        body: "hello".to_string(),
        author: user(),
        created_at: "2026-07-06T00:00:01Z".to_string(),
        updated_at: "2026-07-06T00:00:01Z".to_string(),
        system: false,
        noteable_id: Some(20),
        noteable_iid: Some(noteable_iid),
        noteable_type: noteable_type.to_string(),
        project_id: 30,
        resolvable: false,
        confidential: false,
        internal: false,
        imported: false,
        imported_from: "none".to_string(),
        commands_changes: serde_json::json!({}),
    }
}
fn note_payload(project_id: u64, noteable_type: &str, system: bool, bot: bool) -> serde_json::Value {
    serde_json::json!({
        "object_kind": "note",
        "user": { "id": 2, "username": "bob", "bot": bot },
        "project": { "id": project_id },
        "object_attributes": {
            "id": 10,
            "note": "/acorn check",
            "noteable_type": noteable_type,
            "noteable_id": 20,
            "system": system,
            "confidential": false,
            "internal": false
        },
        "issue": { "iid": 3 }
    })
}
fn operation_queue() -> OperationQueue {
    OperationQueue::from(temp_dir().join(format!("acorn-webhook-test-{}.db", nanoid::nanoid!())))
}
async fn post_webhook(server_url: &str, payload: &serde_json::Value, headers: &[(&str, &str)]) -> reqwest::Response {
    let client = reqwest::Client::new();
    let body = serde_json::to_string(payload).unwrap();
    let mut req = client
        .post(format!("{server_url}/webhooks/gitlab"))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Event", "Merge Request Hook")
        .header("X-Gitlab-Webhook-UUID", "legacy-delivery-id")
        .body(body);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    req.send().await.unwrap()
}
fn settlement_failure() -> ApiResult<()> {
    Err(color_eyre::eyre::eyre!("injected settlement failure"))
}
fn user() -> UserMetadata {
    UserMetadata {
        avatar_url: "https://example.com/avatar.png".to_string(),
        identifier: 1,
        is_bot: false,
        locked: false,
        name: "Test User".to_string(),
        email: None,
        state: "active".to_string(),
        username: "tester".to_string(),
        url: "https://example.com/tester".to_string(),
    }
}
#[test]
fn test_bot_authored_completion_events_do_not_create_operations() {
    let merge = WebhookDelivery {
        delivery_id: "bot-merge".to_string(),
        event: HookPayload::MergeRequest {
            actor: HookActor::new(30, 99, "acorn-bot", true),
            iid: 7,
            head_sha: Some("abc123".to_string()),
            action: MergeRequestAction::Merge,
            title: "Citation".to_string(),
            description: String::new(),
        },
    };
    let note = WebhookDelivery {
        delivery_id: "bot-note".to_string(),
        event: HookPayload::Note {
            actor: HookActor::new(30, 99, "acorn-bot", true),
            body: "/acorn process repository-quality".to_string(),
            confidential: false,
            internal: false,
            note_id: 10,
            noteable_id: Some(20),
            noteable_iid: Some(7),
            noteable_type: "MergeRequest".to_string(),
            system: false,
        },
    };
    assert!(merge.key().is_none());
    assert!(note.key().is_none());
    assert!(merge.operation().unwrap().is_none());
    assert!(note.operation().unwrap().is_none());
}
#[tokio::test]
async fn test_default_worker_dispatches_merge_request_analysis() {
    let requests = Arc::new(AtomicUsize::new(0));
    let router = Router::new()
        .route(
            "/api/v4/projects/30/merge_requests/7",
            get(|State(requests): State<Arc<AtomicUsize>>| async move {
                requests.fetch_add(1, Ordering::SeqCst);
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
        )
        .with_state(Arc::clone(&requests));
    let server = TestServer::start(router).await.unwrap();
    let queue = operation_queue();
    let delivery = WebhookDelivery {
        delivery_id: "delivery-stale".to_string(),
        event: HookPayload::MergeRequest {
            actor: HookActor {
                project_id: 30,
                user_id: 4,
                username: "scientist".to_string(),
                is_bot: false,
            },
            iid: 7,
            head_sha: Some("old-sha".to_string()),
            action: MergeRequestAction::Update,
            title: "Previous".to_string(),
            description: String::new(),
        },
    };
    let operation_key = delivery.key().unwrap();
    queue
        .enqueue_spec(&delivery.delivery_id, &delivery.operation().unwrap().unwrap())
        .unwrap();
    let options = Options::with_token("outbound").domain(&server.base_url).identifier("30").build();
    let config = Config::new(options, "127.0.0.1:0".parse().unwrap()).with_operation_queue(queue.clone());
    assert!(Server::new(config).process_next().await.unwrap());
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert_eq!(queue.state(&format!("gitlab:{operation_key}")).unwrap(), Some(OperationState::Succeeded));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_default_worker_dispatches_work_item_intake() {
    let written = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
    let router = Router::new()
        .route("/api/v4/user", get(|| async { Json(serde_json::json!({"id": 99})) }))
        .route(
            "/api/v4/projects/30/issues/7",
            get(|| async {
                Json(serde_json::json!({
                    "id": 70,
                    "iid": 7,
                    "project_id": 30,
                    "title": "Result",
                    "description": "https://example.org/artifact",
                    "author": {"id": 4, "username": "scientist"},
                    "issue_type": "issue"
                }))
            }),
        )
        .route(
            "/api/v4/projects/30/issues/7/notes",
            get(|| async {
                Json(serde_json::json!([{
                    "id": 10,
                    "body": "/acorn check",
                    "author": {"id": 4, "username": "scientist", "bot": false}
                }]))
            })
            .post({
                let written = Arc::clone(&written);
                move |Json(body): Json<serde_json::Value>| {
                    let written = Arc::clone(&written);
                    async move {
                        written.lock().unwrap().push(body.clone());
                        Json(serde_json::json!({
                            "id": 80,
                            "body": body.get("body").and_then(serde_json::Value::as_str).unwrap_or_default(),
                            "author": {"id": 99, "username": "acorn-bot", "bot": true}
                        }))
                    }
                }
            }),
        );
    let server = TestServer::start(router).await.unwrap();
    let queue = operation_queue();
    let delivery = WebhookDelivery {
        delivery_id: "delivery-work-item".to_string(),
        event: HookPayload::Note {
            actor: HookActor {
                project_id: 30,
                user_id: 4,
                username: "scientist".to_string(),
                is_bot: false,
            },
            note_id: 10,
            body: "/acorn check".to_string(),
            noteable_type: "Issue".to_string(),
            noteable_id: Some(70),
            noteable_iid: Some(7),
            system: false,
            confidential: false,
            internal: false,
        },
    };
    let operation_key = delivery.key().unwrap();
    queue
        .enqueue_spec(&delivery.delivery_id, &delivery.operation().unwrap().unwrap())
        .unwrap();
    let options = Options::with_token("outbound").domain(&server.base_url).identifier("30").build();
    let config = Config::new(options, "127.0.0.1:0".parse().unwrap()).with_operation_queue(queue.clone());
    assert!(Server::new(config).process_next().await.unwrap());
    assert_eq!(queue.state(&format!("gitlab:{operation_key}")).unwrap(), Some(OperationState::Succeeded));
    assert_eq!(written.lock().unwrap().len(), 1);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_default_worker_rejects_disallowed_review_effects_before_requests() {
    let requests = Arc::new(AtomicUsize::new(0));
    let router = Router::new()
        .fallback(|State(requests): State<Arc<AtomicUsize>>| async move {
            requests.fetch_add(1, Ordering::SeqCst);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })
        .with_state(Arc::clone(&requests));
    let server = TestServer::start(router).await.unwrap();
    let queue = operation_queue();
    let delivery = WebhookDelivery {
        delivery_id: "delivery-policy".to_string(),
        event: HookPayload::MergeRequest {
            actor: HookActor::new(30, 4, "scientist", false),
            iid: 7,
            head_sha: Some("abc123".to_string()),
            action: MergeRequestAction::Open,
            title: "Policy".to_string(),
            description: String::new(),
        },
    };
    let operation_key = delivery.key().unwrap();
    queue
        .enqueue_spec(&delivery.delivery_id, &delivery.operation().unwrap().unwrap())
        .unwrap();
    let deployment = DeploymentConfig::from_yaml(
        "workflows:\n  repository-quality:\n    paths: ['**/*.json']\n    formats: [json]\n    actions: [validate]\n    outputs: {mode: source-and-artifacts}\n    effects: [note]\n",
    )
    .unwrap();
    let options = Options::with_token("outbound").domain(&server.base_url).identifier("30").build();
    let config = Config::new(options, "127.0.0.1:0".parse().unwrap())
        .with_operation_queue(queue.clone())
        .with_deployment(deployment)
        .unwrap();
    let error = Server::new(config).process_next().await.unwrap_err().to_string();
    assert!(error.contains("Operation 'acorn.webhooks.gitlab.merge-request' failed"));
    assert_eq!(requests.load(Ordering::SeqCst), 0);
    assert_eq!(
        queue.state(&format!("gitlab:{operation_key}")).unwrap(),
        Some(OperationState::RetryableFailure)
    );
    server.stop().await.unwrap();
}
#[test]
fn test_deserializes_event_with_null_target_ids() {
    let event = serde_json::json!({
        "id": 100,
        "project_id": 30,
        "action_name": "commented on",
        "target_id": null,
        "target_iid": null,
        "target_type": "Note",
        "author_id": 1,
        "target_title": "note",
        "created_at": "2026-07-06T00:00:01Z",
        "author": {
            "id": 1,
            "username": "tester",
            "public_email": null,
            "name": "Test User",
            "state": "active",
            "locked": false,
            "avatar_url": "https://example.com/avatar.png",
            "web_url": "https://example.com/tester"
        },
        "imported": false,
        "imported_from": "none",
        "push_data": null,
        "author_username": "tester",
        "note": {
            "id": 10,
            "type": null,
            "body": "hello",
            "author": {
                "id": 1,
                "username": "tester",
                "public_email": null,
                "name": "Test User",
                "state": "active",
                "locked": false,
                "avatar_url": "https://example.com/avatar.png",
                "web_url": "https://example.com/tester"
            },
            "created_at": "2026-07-06T00:00:01Z",
            "updated_at": "2026-07-06T00:00:01Z",
            "system": false,
            "noteable_id": null,
            "noteable_iid": null,
            "noteable_type": "MergeRequest",
            "project_id": 30,
            "resolvable": false,
            "confidential": false,
            "internal": false,
            "imported": false,
            "imported_from": "none",
            "commands_changes": {}
        }
    });

    let event: EventDetails = serde_json::from_value(event).unwrap();
    assert_eq!(event.target_id, None);
    assert_eq!(event.target_iid, None);
    assert!(MergeRequestNoteEvent::try_from(&event).is_err());
}
#[test]
fn test_extracts_merge_request_iid_from_note_event() {
    let extracted = MergeRequestNoteEvent::try_from(&event("2026-07-06T00:00:01Z", "MergeRequest")).unwrap();
    assert_eq!(extracted.merge_request_iid, 42);
}
#[test]
fn test_graduation_command_selects_durable_merge_request_operation() {
    let delivery = WebhookDelivery {
        delivery_id: "graduation-command".to_string(),
        event: HookPayload::Note {
            actor: HookActor::new(30, 2, "bob", false),
            body: "/acorn graduate form-b form-a form-a".to_string(),
            confidential: false,
            internal: false,
            note_id: 11,
            noteable_id: Some(20),
            noteable_iid: Some(3),
            noteable_type: "MergeRequest".to_string(),
            system: false,
        },
    };
    assert_eq!(delivery.key().as_deref(), Some("logbook-graduate:30:3:11"));
    let operation = delivery.operation().unwrap().unwrap();
    assert_eq!(
        operation.notification.method,
        crate::io::api::json_rpc::MethodName::from(["webhooks", "gitlab", "note"])
    );
}
#[test]
fn test_ignores_merge_request_note_without_iid() {
    let mut event = event("2026-07-06T00:00:01Z", "MergeRequest");
    if let Some(note) = event.note.as_mut() {
        note.noteable_iid = None;
    }
    let extracted = MergeRequestNoteEvent::try_from(&event);
    assert!(extracted.is_err());
}
#[test]
fn test_ignores_non_merge_request_note_event() {
    let extracted = MergeRequestNoteEvent::try_from(&event("2026-07-06T00:00:01Z", "Issue"));
    assert!(extracted.is_err());
}
#[tokio::test]
async fn test_jsonl_cleanup_preserves_append_racing_with_removal() {
    let first = format!("{}\n", jsonl_form(0));
    let appended = format!("{}\n", jsonl_form(1));
    let (server, bot, repository, config) = jsonl_bot(Some(first), JsonlRace::Append(appended.clone())).await;
    let summary = bot.poll_jsonl_once().await.unwrap();
    assert_eq!(summary.admitted_count, 1);
    assert_eq!(repository.lock().unwrap().content.as_deref(), Some(appended.as_str()));
    assert_eq!(repository.lock().unwrap().writes.len(), 2);
    let second = bot.poll_jsonl_once().await.unwrap();
    assert_eq!(second.admitted_count, 1);
    assert_eq!(repository.lock().unwrap().content.as_deref(), Some(""));
    assert_eq!(config.operation_queue.counts().unwrap().queued, 2);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_jsonl_cleanup_preserves_rejections_blank_lines_and_unicode_bytes() {
    let invalid = "{malformed ✓}\n";
    let unknown = jsonl_form(1).replace("pilot", "unknown");
    let content = format!("\n{}\n\n{invalid}{unknown}\n", jsonl_form(0));
    let expected = format!("\n\n{invalid}{unknown}\n");
    let (server, bot, repository, _config) = jsonl_bot(Some(content), JsonlRace::None).await;
    let summary = bot.poll_jsonl_once().await.unwrap();
    assert_eq!(summary.rejected_count, 2);
    assert_eq!(summary.removed_count, 1);
    assert_eq!(repository.lock().unwrap().content.as_deref(), Some(expected.as_str()));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_jsonl_cleanup_reconciles_ambiguous_success_without_duplicate_work() {
    let content = format!("{}\n", jsonl_form(0));
    let (server, bot, repository, config) = jsonl_bot(Some(content.clone()), JsonlRace::Ambiguous).await;
    bot.poll_jsonl_once().await.unwrap();
    assert_eq!(repository.lock().unwrap().content.as_deref(), Some(""));
    assert_eq!(repository.lock().unwrap().writes.len(), 1);
    repository.lock().unwrap().content = Some(content);
    bot.poll_jsonl_once().await.unwrap();
    assert_eq!(config.operation_queue.counts().unwrap().queued, 1);
    assert_eq!(repository.lock().unwrap().content.as_deref(), Some(""));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_jsonl_cleanup_reconciles_another_bots_removal() {
    let (server, bot, repository, _config) = jsonl_bot(Some(format!("{}\n", jsonl_form(0))), JsonlRace::Cleaned).await;
    bot.poll_jsonl_once().await.unwrap();
    assert_eq!(repository.lock().unwrap().content.as_deref(), Some(""));
    assert_eq!(repository.lock().unwrap().writes.len(), 1);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_jsonl_conflicting_uuid_is_retained_after_original_was_removed() {
    let raw = jsonl_form(0);
    let (server, bot, repository, config) = jsonl_bot(Some(format!("{raw}\n")), JsonlRace::None).await;
    bot.poll_jsonl_once().await.unwrap();
    let changed = format!("{}\n", raw.replace("New functionality", "Different content"));
    repository.lock().unwrap().content = Some(changed.clone());
    let summary = bot.poll_jsonl_once().await.unwrap();
    assert_eq!(summary.rejected_count, 1);
    assert_eq!(summary.removed_count, 0);
    assert_eq!(repository.lock().unwrap().content.as_deref(), Some(changed.as_str()));
    assert_eq!(config.operation_queue.counts().unwrap().queued, 1);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_jsonl_conflicts_are_bounded_and_admitted_work_survives_restart() {
    let (server, bot, repository, config) = jsonl_bot(Some(format!("{}\n", jsonl_form(0))), JsonlRace::Conflict).await;
    assert!(bot.poll_jsonl_once().await.unwrap_err().to_string().contains("three attempts"));
    assert_eq!(repository.lock().unwrap().writes.len(), 3);
    assert_eq!(config.operation_queue.counts().unwrap().queued, 1);
    repository.lock().unwrap().race = JsonlRace::None;
    let restarted = Server::new(config.clone());
    restarted.poll_jsonl_once().await.unwrap();
    assert_eq!(repository.lock().unwrap().content.as_deref(), Some(""));
    assert_eq!(config.operation_queue.counts().unwrap().queued, 1);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_jsonl_drain_stops_intake_without_consuming_records() {
    let content = format!("{}\n", jsonl_form(0));
    let (server, bot, repository, _config) = jsonl_bot(Some(content.clone()), JsonlRace::None).await;
    bot.drain().await.unwrap();
    assert_eq!(bot.poll_jsonl_once().await.unwrap(), JsonlPollSummary::default());
    assert_eq!(repository.lock().unwrap().content.as_deref(), Some(content.as_str()));
    assert!(repository.lock().unwrap().writes.is_empty());
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_jsonl_missing_file_is_idle_and_drained_file_is_kept() {
    let (server, bot, repository, _config) = jsonl_bot(None, JsonlRace::None).await;
    assert_eq!(bot.poll_jsonl_once().await.unwrap(), JsonlPollSummary::default());
    assert!(repository.lock().unwrap().content.is_none());
    repository.lock().unwrap().content = Some(format!("{}\n", jsonl_form(0)));
    bot.poll_jsonl_once().await.unwrap();
    assert_eq!(repository.lock().unwrap().content.as_deref(), Some(""));
    assert_eq!(bot.poll_jsonl_once().await.unwrap(), JsonlPollSummary::default());
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_jsonl_missing_metadata_keeps_durable_admission_and_inbox() {
    let content = format!("{}\n", jsonl_form(0));
    let (server, bot, repository, config) = jsonl_bot(Some(content.clone()), JsonlRace::MissingMetadata).await;
    assert!(bot.poll_jsonl_once().await.unwrap_err().to_string().contains("last_commit_id"));
    assert_eq!(repository.lock().unwrap().content.as_deref(), Some(content.as_str()));
    assert!(repository.lock().unwrap().writes.is_empty());
    assert_eq!(config.operation_queue.counts().unwrap().queued, 1);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_jsonl_size_limit_fails_without_admission_or_removal() {
    let content = format!("{}\n", "x".repeat(70_000));
    let (server, bot, repository, config) = jsonl_bot(Some(content.clone()), JsonlRace::None).await;
    assert!(bot.poll_jsonl_once().await.unwrap_err().to_string().contains("record size limit"));
    assert_eq!(repository.lock().unwrap().content.as_deref(), Some(content.as_str()));
    assert!(repository.lock().unwrap().writes.is_empty());
    assert_eq!(config.operation_queue.counts().unwrap().queued, 0);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_jsonl_storage_failure_never_removes_source() {
    let content = format!("{}\n", jsonl_form(0));
    let (server, _bot, repository, config) = jsonl_bot(Some(content.clone()), JsonlRace::None).await;
    let bot = Server::new(config.with_operation_queue(OperationQueue::from(temp_dir())));
    assert!(bot.poll_jsonl_once().await.is_err());
    assert_eq!(repository.lock().unwrap().content.as_deref(), Some(content.as_str()));
    assert!(repository.lock().unwrap().writes.is_empty());
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_jsonl_unconfigured_intake_is_idle_without_database_access() {
    let bot = Server::new(Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_operation_queue(OperationQueue::from(temp_dir())));
    assert_eq!(bot.poll_jsonl_once().await.unwrap(), JsonlPollSummary::default());
}
#[tokio::test]
async fn test_jsonl_worker_uses_admitted_route_after_configuration_changes() {
    let reads = Arc::new(AtomicUsize::new(0));
    let captured = Arc::clone(&reads);
    let server = TestServer::start(
        Router::new()
            .route("/api/v4/projects/42/packages", get(|| async { Json(serde_json::json!([])) }))
            .route(
                "/api/v4/projects/42",
                get(|| async { Json(serde_json::json!({"id":42,"default_branch":"main"})) }),
            )
            .route(
                "/api/v4/projects/42/repository/branches/{*branch}",
                get(|| async { Json(serde_json::json!({"name":"review", "commit":{"id":"abc123"}})) }),
            )
            .route(
                "/api/v4/projects/42/repository/files/{*path}",
                get(move |extract::Path(path): extract::Path<String>| {
                    let reads = Arc::clone(&captured);
                    async move {
                        assert_eq!(path, "pilot/index.json");
                        reads.fetch_add(1, Ordering::SeqCst);
                        StatusCode::SERVICE_UNAVAILABLE
                    }
                }),
            ),
    )
    .await
    .unwrap();
    let original: ApplicationConfiguration = serde_json::from_value(serde_json::json!({
        "folders":[{"id":"pilot", "aliases":["folder-name"], "members":["scientist@example.org"]}],
        "options":{"powerAutomate":{"callback_destination":"flow", "gitlab_project":"42", "package":"reports", "intake":{"branch":"main"}}}
    }))
    .unwrap();
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let queue = operation_queue();
    let raw = serde_json::json!({"submissionId":"123e4567-e89b-12d3-a456-426614174000", "projectId":"folder-name", "submittedAt":"2026-09-18T12:01:00Z", "submitter":{"email":"scientist@example.org"}, "entry":{"timestamp":"2026-09-18T12:00:00Z", "category":"software", "state":"completed", "summary":"New functionality"}}).to_string();
    let snapshot = Snapshot {
        content: format!("{raw}\n"),
        last_commit_id: Some("abc123".to_string()),
        commit: "abc123".to_string(),
        lines: split_snapshot(&format!("{raw}\n"), &Memory::kb(64)).unwrap(),
    };
    assert!(
        admit_snapshot(&snapshot, original.power_automate().unwrap(), &original.registry(), &queue, &options)
            .await
            .unwrap()[0]
            .as_ref()
            .unwrap()
            .admission
            .is_some()
    );
    let changed: ApplicationConfiguration = serde_json::from_value(serde_json::json!({
        "folders":[{"id":"pilot", "path":"moved/index.json"}],
        "options":{"powerAutomate":{"callback_destination":"flow", "gitlab_project":"43", "package":"reports"}}
    }))
    .unwrap();
    let mut config = Config::new(options.with_identifier("43"), "127.0.0.1:0".parse().unwrap()).with_operation_queue(queue);
    config.powerautomate = Some(PowerAutomateProvider::new(changed.power_automate().unwrap().clone(), changed.registry(), "test").unwrap());
    assert!(Server::new(config).process_next().await.is_err());
    assert!(reads.load(Ordering::SeqCst) > 0);
    server.stop().await.unwrap();
}
#[test]
fn test_merge_request_actions_share_head_specific_operation_key() {
    let delivery = |action| WebhookDelivery {
        delivery_id: format!("{action:?}"),
        event: HookPayload::MergeRequest {
            actor: HookActor {
                project_id: 30,
                user_id: 4,
                username: "scientist".to_string(),
                is_bot: false,
            },
            iid: 7,
            head_sha: Some("abc123".to_string()),
            action,
            title: "Result".to_string(),
            description: String::new(),
        },
    };
    let keys = [
        delivery(MergeRequestAction::Open).key(),
        delivery(MergeRequestAction::Reopen).key(),
        delivery(MergeRequestAction::Update).key(),
    ];
    assert_eq!(
        keys,
        [
            Some("mr-check:30:7:abc123".to_string()),
            Some("mr-check:30:7:abc123".to_string()),
            Some("mr-check:30:7:abc123".to_string()),
        ]
    );
}
#[test]
fn test_merge_request_process_command_selects_named_workflow() {
    let delivery = WebhookDelivery {
        delivery_id: "process-command".to_string(),
        event: HookPayload::Note {
            actor: HookActor::new(30, 2, "bob", false),
            body: "/acorn process repository-quality".to_string(),
            confidential: false,
            internal: false,
            note_id: 10,
            noteable_id: Some(20),
            noteable_iid: Some(3),
            noteable_type: "MergeRequest".to_string(),
            system: false,
        },
    };
    assert_eq!(delivery.key().as_deref(), Some("repository-process:30:3:10"));
    let operation = delivery.operation().unwrap().unwrap();
    assert_eq!(
        operation.notification.method,
        crate::io::api::json_rpc::MethodName::from(["webhooks", "gitlab", "note"])
    );
}
#[test]
fn test_note_poll_queues_authorized_graduation_shape_once() {
    let queue = operation_queue();
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_operation_queue(queue.clone());
    let bot = Server::new(config);
    let mut source = event("2026-07-06T00:00:01Z", "MergeRequest");
    source.note.as_mut().unwrap().body = "/acorn graduate".to_string();
    bot.process_events(vec![source.clone()]).unwrap();
    assert_eq!(queue.counts().unwrap().queued, 1);
    bot.process_events(vec![source.clone()]).unwrap();
    assert_eq!(queue.counts().unwrap().queued, 1);
    source.identifier += 1;
    source.note.as_mut().unwrap().author.is_bot = true;
    bot.process_events(vec![source]).unwrap();
    assert_eq!(queue.counts().unwrap().queued, 1);
}
#[tokio::test]
async fn test_note_poll_worker_rejects_graduation_without_developer_access() {
    let requests = Arc::new(AtomicUsize::new(0));
    let router = Router::new()
        .route(
            "/api/v4/projects/30/members/all/1",
            get(|State(requests): State<Arc<AtomicUsize>>| async move {
                requests.fetch_add(1, Ordering::SeqCst);
                Json(serde_json::json!({"id":1,"access_level":20}))
            }),
        )
        .with_state(Arc::clone(&requests));
    let server = TestServer::start(router).await.unwrap();
    let queue = operation_queue();
    let options = Options::with_token("test").domain(&server.base_url).identifier("30").build();
    let bot = Server::new(Config::new(options, "127.0.0.1:0".parse().unwrap()).with_operation_queue(queue.clone()));
    let mut source = event("2026-07-06T00:00:01Z", "MergeRequest");
    source.note.as_mut().unwrap().body = "/acorn graduate".to_string();
    bot.process_events(vec![source]).unwrap();
    assert!(bot.process_next().await.is_err());
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert_eq!(queue.counts().unwrap().succeeded, 0);
    server.stop().await.unwrap();
}
#[test]
fn test_processing_updates_after_to_latest_event_timestamp() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap());
    let bot = Server::new(config);
    let summary = bot
        .process_events(vec![
            event("2026-07-06T00:00:01Z", "MergeRequest"),
            event("2026-07-06T00:00:03Z", "Issue"),
        ])
        .unwrap();
    assert_eq!(summary.latest_after, Some("2026-07-06T00:00:03Z".to_string()));
    assert_eq!(summary.processed_count, 1);
    assert_eq!(bot.snapshot().unwrap().after, "2026-07-06T00:00:03Z");
}
#[tokio::test]
async fn test_router_health_endpoint_returns_ok() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap());
    let bot = Server::new(config);
    let server = TestServer::start(bot.router()).await.unwrap();
    let response = reqwest::get(format!("{}/health", server.base_url)).await.unwrap();
    let status = response.status();
    let body = response.text().await.unwrap();
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(body, "ok");
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_router_liveness_remains_up_while_readiness_and_ingress_stop_during_drain() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
        .with_operation_queue(operation_queue())
        .with_shutdown_grace(core::time::Duration::ZERO)
        .with_webhook_token("secret");
    let bot = Server::new(config);
    let server = TestServer::start(bot.router()).await.unwrap();
    assert_eq!(
        reqwest::get(format!("{}/ready", server.base_url)).await.unwrap().status(),
        reqwest::StatusCode::OK
    );
    bot.drain().await.unwrap();
    assert_eq!(
        reqwest::get(format!("{}/health", server.base_url)).await.unwrap().status(),
        reqwest::StatusCode::OK
    );
    assert_eq!(
        reqwest::get(format!("{}/ready", server.base_url)).await.unwrap().status(),
        reqwest::StatusCode::SERVICE_UNAVAILABLE
    );
    let response = post_webhook(&server.base_url, &mr_payload(30, "open"), &[("X-Gitlab-Token", "secret")]).await;
    assert_eq!(response.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_router_state_endpoint_returns_initial_snapshot() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_after("2026-07-06T00:00:00Z");
    let bot = Server::new(config);
    let server = TestServer::start(bot.router()).await.unwrap();
    let response = reqwest::get(format!("{}/state", server.base_url)).await.unwrap();
    let status = response.status();
    let json = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(json.get("after").and_then(serde_json::Value::as_str), Some("2026-07-06T00:00:00Z"));
    assert_eq!(json.get("poll_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(json.get("processed_count").and_then(serde_json::Value::as_u64), Some(0));
    assert!(json.get("last_error").is_some_and(serde_json::Value::is_null));
    assert_eq!(json.pointer("/webhook/accepting").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(json.pointer("/webhook/ready").and_then(serde_json::Value::as_bool), Some(true));
    server.stop().await.unwrap();
}
#[test]
fn test_standard_webhook_signature_matches_fixed_vector() {
    let signature = utils::compute_standard_webhook_signature("whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==", "wh-id-1", "1700000000", br#"{"hello":"world"}"#);
    assert_eq!(signature, "v1,GvvDAfrKjm6zfPxjHVFe3fLaHJSqQi8ZDA7gRiqVHsY=");
}
#[tokio::test]
async fn test_success_settlement_failure_is_not_converted_to_operation_failure() {
    let queue = operation_queue().with_settlement_hook(settlement_failure);
    let delivery = WebhookDelivery {
        delivery_id: "delivery-settlement-failure".to_string(),
        event: HookPayload::MergeRequest {
            actor: HookActor {
                project_id: 30,
                user_id: 7,
                username: "scientist".to_string(),
                is_bot: false,
            },
            iid: 5,
            head_sha: Some("abc123".to_string()),
            action: MergeRequestAction::Open,
            title: "Result".to_string(),
            description: String::new(),
        },
    };
    let operation_key = delivery.key().unwrap();
    queue
        .enqueue_spec(&delivery.delivery_id, &delivery.operation().unwrap().unwrap())
        .unwrap();
    let handler: WebhookOperationHandler = Arc::new(|_| Box::pin(async { Ok(()) }));
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
        .with_operation_queue(queue.clone())
        .with_operation_handler(handler);
    let error = Server::new(config).process_next().await.unwrap_err().to_string();
    assert!(error.contains("Failed to settle successful webhook operation"));
    assert!(error.contains("injected settlement failure"));
    assert_eq!(queue.state(&format!("gitlab:{operation_key}")).unwrap(), Some(OperationState::Running));
}
#[tokio::test]
async fn test_webhook_accepts_bot_author_with_flag_set() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
        .with_webhook_token("test-token")
        .with_project_id(30);
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let body_val = note_payload(30, "Issue", false, true);
    let body_str = serde_json::to_string(&body_val).unwrap();
    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/gitlab", server.base_url))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Event", "Note Hook")
        .header("X-Gitlab-Token", "test-token")
        .header("X-Gitlab-Webhook-UUID", "bot-note-id")
        .body(body_str)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 202);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_accepts_system_note_with_flag_set() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
        .with_webhook_token("test-token")
        .with_project_id(30);
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let body_val = note_payload(30, "MergeRequest", true, false);
    let body_str = serde_json::to_string(&body_val).unwrap();
    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/gitlab", server.base_url))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Event", "Note Hook")
        .header("X-Gitlab-Token", "test-token")
        .header("X-Gitlab-Webhook-UUID", "system-note-id")
        .body(body_str)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 202);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_accepts_valid_hmac_signature() {
    let signing_token_base64 = "whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==";
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
        .with_webhook_signing_token(signing_token_base64)
        .with_project_id(30)
        .with_operation_queue(operation_queue());
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
    let timestamp = utils::current_unix_timestamp();
    let sig = utils::compute_standard_webhook_signature(signing_token_base64, "wh-id-1", &timestamp, body.as_bytes());
    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/gitlab", server.base_url))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Event", "Merge Request Hook")
        .header("webhook-id", "wh-id-1")
        .header("webhook-timestamp", timestamp)
        .header("webhook-signature", sig)
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 202);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_accepts_valid_legacy_token() {
    let queue = operation_queue();
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
        .with_webhook_token("secret")
        .with_project_id(30)
        .with_operation_queue(queue.clone());
    let bot = Server::new(config);
    let server = TestServer::start(bot.router()).await.unwrap();
    let res = post_webhook(&server.base_url, &mr_payload(30, "open"), &[("X-Gitlab-Token", "secret")]).await;
    assert_eq!(res.status(), 202);
    assert_eq!(queue.state("gitlab:mr-check:30:5:abc123").unwrap(), Some(OperationState::Queued));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_hmac_takes_precedence_over_legacy_token() {
    let signing_token_base64 = "whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==";
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
        .with_webhook_token("legacy-token")
        .with_webhook_signing_token(signing_token_base64)
        .with_project_id(30)
        .with_operation_queue(operation_queue());
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
    let timestamp = utils::current_unix_timestamp();
    let sig = utils::compute_standard_webhook_signature(signing_token_base64, "wh-id-1", &timestamp, body.as_bytes());
    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/gitlab", server.base_url))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Event", "Merge Request Hook")
        .header("webhook-id", "wh-id-1")
        .header("webhook-timestamp", timestamp)
        .header("webhook-signature", sig)
        .header("X-Gitlab-Token", "wrong-legacy-token")
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 202);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_rejects_invalid_hmac_even_with_valid_legacy_token() {
    let signing_token_base64 = "whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==";
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
        .with_webhook_token("legacy-token")
        .with_webhook_signing_token(signing_token_base64);
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
    let timestamp = utils::current_unix_timestamp();
    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/gitlab", server.base_url))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Event", "Merge Request Hook")
        .header("webhook-id", "wh-id-1")
        .header("webhook-timestamp", timestamp)
        .header("webhook-signature", "v1,invalidsig")
        .header("X-Gitlab-Token", "legacy-token")
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_rejects_invalid_hmac_signature() {
    let signing_token_base64 = "whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==";
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_signing_token(signing_token_base64);
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
    let timestamp = utils::current_unix_timestamp();
    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/gitlab", server.base_url))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Event", "Merge Request Hook")
        .header("webhook-id", "wh-id-1")
        .header("webhook-timestamp", timestamp)
        .header("webhook-signature", "v1,badsignature")
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_rejects_invalid_legacy_token() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_token("secret");
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let res = post_webhook(&server.base_url, &mr_payload(30, "open"), &[("X-Gitlab-Token", "wrong")]).await;
    assert_eq!(res.status(), 401);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_rejects_malformed_json() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_token("tok");
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/gitlab", server.base_url))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Event", "Merge Request Hook")
        .header("X-Gitlab-Token", "tok")
        .header("X-Gitlab-Webhook-UUID", "malformed-json-id")
        .body("{not valid json}")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_rejects_missing_delivery_identifier() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_token("secret");
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/gitlab", server.base_url))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Event", "Merge Request Hook")
        .header("X-Gitlab-Token", "secret")
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_rejects_missing_event_header() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_token("test-token");
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/gitlab", server.base_url))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Token", "test-token")
        .body(serde_json::to_string(&mr_payload(30, "open")).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_rejects_missing_legacy_token_when_configured() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_token("secret");
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let res = post_webhook(&server.base_url, &mr_payload(30, "open"), &[]).await;
    assert_eq!(res.status(), 401);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_rejects_missing_webhook_id_for_hmac() {
    let signing_token_base64 = "whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==";
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_signing_token(signing_token_base64);
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
    let timestamp = utils::current_unix_timestamp();
    let sig = utils::compute_standard_webhook_signature(signing_token_base64, "wh-id-1", &timestamp, body.as_bytes());
    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/gitlab", server.base_url))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Event", "Merge Request Hook")
        .header("webhook-timestamp", timestamp)
        .header("webhook-signature", sig)
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_rejects_no_auth_when_not_configured() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap());
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let res = post_webhook(&server.base_url, &mr_payload(30, "open"), &[]).await;
    assert_eq!(res.status(), 401);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_rejects_oversized_body() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap());
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let big_body = "x".repeat(1024 * 1024 + 1);
    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/gitlab", server.base_url))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Event", "Merge Request Hook")
        .body(big_body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 413);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_rejects_stale_timestamp() {
    let signing_token_base64 = "whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==";
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_signing_token(signing_token_base64);
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
    let timestamp = utils::stale_unix_timestamp(400);
    let sig = utils::compute_standard_webhook_signature(signing_token_base64, "wh-id-1", &timestamp, body.as_bytes());
    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/gitlab", server.base_url))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Event", "Merge Request Hook")
        .header("webhook-id", "wh-id-1")
        .header("webhook-timestamp", timestamp)
        .header("webhook-signature", sig)
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_rejects_wrong_project() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
        .with_webhook_token("test-token")
        .with_project_id(99);
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let res = post_webhook(&server.base_url, &mr_payload(30, "open"), &[("X-Gitlab-Token", "test-token")]).await;
    assert_eq!(res.status(), 403);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_returns_ok_for_unsupported_event_type() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_token("test-token");
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/gitlab", server.base_url))
        .header("Content-Type", "application/json")
        .header("X-Gitlab-Event", "Push Hook")
        .header("X-Gitlab-Token", "test-token")
        .header("X-Gitlab-Webhook-UUID", "unsupported-event-id")
        .body("{}")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_returns_service_unavailable_when_durable_persistence_fails() {
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
        .with_webhook_token("secret")
        .with_project_id(30)
        .with_operation_queue(OperationQueue::from(temp_dir()));
    let server = TestServer::start(Server::new(config).router()).await.unwrap();
    let res = post_webhook(&server.base_url, &mr_payload(30, "open"), &[("X-Gitlab-Token", "secret")]).await;
    assert_eq!(res.status(), 503);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_webhook_worker_awaits_operation_before_marking_success() {
    let queue = operation_queue();
    let processed = Arc::new(AtomicBool::new(false));
    let processed_by_handler = Arc::clone(&processed);
    let handler: WebhookOperationHandler = Arc::new(move |_| {
        let processed = Arc::clone(&processed_by_handler);
        Box::pin(async move {
            tokio::task::yield_now().await;
            processed.store(true, Ordering::SeqCst);
            Ok(())
        })
    });
    let delivery = WebhookDelivery {
        delivery_id: "delivery-async".to_string(),
        event: HookPayload::MergeRequest {
            actor: HookActor {
                project_id: 30,
                user_id: 7,
                username: "scientist".to_string(),
                is_bot: false,
            },
            iid: 5,
            head_sha: Some("abc123".to_string()),
            action: MergeRequestAction::Open,
            title: "Result".to_string(),
            description: String::new(),
        },
    };
    let operation_key = delivery.key().unwrap();
    queue
        .enqueue_spec(&delivery.delivery_id, &delivery.operation().unwrap().unwrap())
        .unwrap();
    let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
        .with_operation_queue(queue.clone())
        .with_operation_handler(handler);
    assert!(Server::new(config).process_next().await.unwrap());
    assert!(processed.load(Ordering::SeqCst));
    assert_eq!(queue.state(&format!("gitlab:{operation_key}")).unwrap(), Some(OperationState::Succeeded));
}
