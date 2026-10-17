use super::*;
use crate::io::api::gitlab::{
    jsonl::{split_snapshot, Snapshot},
    Options,
};
use crate::io::api::powerautomate::jsonl::{admit_snapshot, is_admitted, source_identity, AdmittedSubmission};
use crate::io::api::{webhooks::store, Configuration};
use crate::io::config::application;
use crate::io::ApiResult;
use crate::test::server::TestServer;
use acorn_schema::hardware::memory::Memory;
use axum::{routing::get, Json, Router};

async fn admit(
    content: &str,
    commit: &str,
    configuration: &ApplicationConfiguration,
    queue: &OperationQueue,
    options: &Options,
) -> Vec<ApiResult<store::IntakeRecord>> {
    admit_snapshot(
        &snapshot(content, commit),
        configuration.power_automate().unwrap(),
        &configuration.registry(),
        queue,
        options,
    )
    .await
    .unwrap()
}
fn configuration() -> ApplicationConfiguration {
    serde_json::from_value(serde_json::json!({
        "folders": [{"id":"pilot", "aliases":["folder-name"], "members":["Scientist@Example.org"]},
                    {"id":"other", "members":["Scientist@Example.org"]}],
        "options":{"powerAutomate":{"callback_destination":"pilot-flow", "gitlab_project":"42", "package":"automated-artifacts",
            "intake":{"branch":"main"}}}
    }))
    .unwrap()
}
fn multi_configuration() -> ApplicationConfiguration {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/automation/powerautomate/multi-config.json"
    )))
    .unwrap()
}
fn queue() -> OperationQueue {
    OperationQueue::from(temp_dir().join(format!("acorn-jsonl-{}.db", nanoid::nanoid!())))
}
fn raw(selector: &str, uuid: &str) -> String {
    let form = Submission {
        project_id: selector.to_string(),
        submission_id: uuid.to_string(),
        ..submission()
    };
    serde_json::to_string(&form).unwrap()
}
async fn server() -> TestServer {
    TestServer::start(Router::new().route("/api/v4/projects/42/packages", get(|| async { Json(serde_json::json!([])) })))
        .await
        .unwrap()
}
fn snapshot(content: &str, commit: &str) -> Snapshot {
    Snapshot {
        content: content.to_string(),
        last_commit_id: Some(commit.to_string()),
        commit: commit.to_string(),
        lines: split_snapshot(content, &Memory::kb(64)).unwrap(),
    }
}

#[tokio::test]
async fn test_admission_canonicalizes_alias_and_retains_exact_evidence() {
    let server = server().await;
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let queue = queue();
    let configuration = configuration();
    let content = format!("{}\n", raw("folder-name", "123e4567-e89b-12d3-a456-426614174000"));
    let outcomes = admit(&content, "first", &configuration, &queue, &options).await;
    let outcome = outcomes[0].as_ref().unwrap();
    assert!(outcome.rejection.is_none());
    assert_eq!(outcome.selector.as_deref(), Some("folder-name"));
    assert_eq!(outcome.raw, content.trim_end_matches('\n'));
    let admitted: AdmittedSubmission = serde_json::from_str(outcome.admission.as_ref().unwrap()).unwrap();
    assert_eq!(admitted.entry.project_id, "pilot");
    assert_eq!(admitted.path, "pilot/index.json");
    assert!(admitted.branch.starts_with("acorn/powerautomate/pilot/"));
    assert_eq!(queue.counts().unwrap().queued, 1);
    let source = source_identity(configuration.power_automate().unwrap().intake.as_ref().unwrap(), &options).unwrap();
    assert_eq!(queue.intake_records(&source).unwrap().len(), 1);
    assert!(is_admitted(&snapshot(&content, "first").lines[0], &queue).unwrap());
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_admission_failure_rolls_back_queue_and_evidence() {
    let server = server().await;
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let queue = queue();
    queue.migrate().unwrap();
    queue.with_connection(|connection| connection.execute_batch("CREATE TRIGGER reject_intake BEFORE INSERT ON webhook_intake_records BEGIN SELECT RAISE(ABORT, 'injected evidence failure'); END;").map_err(Into::into)).unwrap();
    let content = format!("{}\n", raw("pilot", "123e4567-e89b-12d3-a456-426614174000"));
    assert!(admit(&content, "first", &configuration(), &queue, &options).await[0].is_err());
    assert_eq!(queue.counts().unwrap().queued, 0);
    assert!(queue
        .intake_admission("powerautomate:form:v1:123e4567-e89b-12d3-a456-426614174000")
        .unwrap()
        .is_none());
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_bucket_identity_survives_reordering_and_rejects_wrong_collection() {
    let server = TestServer::start(
        Router::new()
            .route("/api/v4/projects/42/packages", get(|| async { Json(serde_json::json!([])) }))
            .route("/api/v4/projects/42", get(|| async { Json(serde_json::json!({"id":42})) }))
            .route("/api/v4/projects/43", get(|| async { Json(serde_json::json!({"id":43})) })),
    )
    .await
    .unwrap();
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let queue = queue();
    let mut value = serde_json::to_value(configuration()).unwrap();
    value["buckets"] = serde_json::json!([
        {"name":"collection", "source":{"provider":"gitlab", "id":42, "uri":format!("{}/example/collection", server.base_url)}},
        {"name":"elsewhere", "source":{"provider":"gitlab", "id":43, "uri":format!("{}/example/elsewhere", server.base_url)}}
    ]);
    value["folders"][0]["bucket"] = serde_json::json!(0);
    let config: ApplicationConfiguration = serde_json::from_value(value.clone()).unwrap();
    let content = format!("{}\n", raw("pilot", "123e4567-e89b-12d3-a456-426614174000"));
    let first = admit(&content, "first", &config, &queue, &options).await;
    let original: AdmittedSubmission = serde_json::from_str(first[0].as_ref().unwrap().admission.as_ref().unwrap()).unwrap();
    value["buckets"].as_array_mut().unwrap().swap(0, 1);
    let changed: ApplicationConfiguration = serde_json::from_value(value).unwrap();
    let replay = admit(&content, "second", &changed, &queue, &options).await;
    let retained: AdmittedSubmission = serde_json::from_str(replay[0].as_ref().unwrap().admission.as_ref().unwrap()).unwrap();
    assert_eq!(original.bucket, retained.bucket);
    let different = format!("{}\n", raw("pilot", "123e4567-e89b-12d3-a456-426614174001"));
    assert!(admit(&different, "third", &changed, &queue, &options).await[0]
        .as_ref()
        .unwrap()
        .rejection
        .as_ref()
        .unwrap()
        .contains("different GitLab project"));
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_cutoff_rejection_is_inspectable_and_not_queued() {
    let window = ReportingWindow::new(None, "2026-10-01T00:00:00Z").unwrap();
    let manifest = ReportManifest::new(
        "automated-artifacts",
        "pilot",
        "0123456789abcdef",
        &window,
        vec![],
        "1",
        "2",
        None,
        "https://gitlab.example/package",
    )
    .unwrap();
    let server = TestServer::start(
        Router::new()
            .route(
                "/api/v4/projects/42/packages",
                get(|| async { Json(serde_json::json!([{"status":"default", "version":"test"}])) }),
            )
            .route(
                "/api/v4/projects/42/packages/generic/automated-artifacts/test/manifest.json",
                get(|| async move { Json(manifest) }),
            ),
    )
    .await
    .unwrap();
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let queue = queue();
    let content = format!("{}\n", raw("pilot", "123e4567-e89b-12d3-a456-426614174000"));
    let result = admit(&content, "first", &configuration(), &queue, &options).await;
    assert!(result[0].as_ref().unwrap().rejection.as_ref().unwrap().contains("predates"));
    assert_eq!(queue.counts().unwrap().queued, 0);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_infrastructure_failure_does_not_become_a_permanent_rejection() {
    let server = TestServer::start(Router::new()).await.unwrap();
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let queue = queue();
    let content = format!("{}\n", raw("pilot", "123e4567-e89b-12d3-a456-426614174000"));
    assert!(admit(&content, "first", &configuration(), &queue, &options).await[0].is_err());
    assert_eq!(queue.counts().unwrap().queued, 0);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_legacy_operation_collision_stays_visible_without_overwrite() {
    let server = server().await;
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let queue = queue();
    queue
        .enqueue("legacy", "powerautomate:form:v1:123e4567-e89b-12d3-a456-426614174000", "{}")
        .unwrap();
    let content = format!("{}\n", raw("pilot", "123e4567-e89b-12d3-a456-426614174000"));
    let outcomes = admit(&content, "first", &configuration(), &queue, &options).await;
    assert!(outcomes[0].as_ref().unwrap().rejection.as_ref().unwrap().contains("no verifiable"));
    assert!(!is_admitted(&snapshot(&content, "first").lines[0], &queue).unwrap());
    assert_eq!(queue.counts().unwrap().queued, 1);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_local_and_unsafe_destinations_are_rejected_without_package_reads() {
    let server = TestServer::start(Router::new()).await.unwrap();
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let queue = queue();
    let mut value = serde_json::to_value(configuration()).unwrap();
    value["folders"][0]["path"] = serde_json::json!("file:///tmp/pilot/index.json");
    value["folders"][1]["path"] = serde_json::json!("../outside/index.json");
    let config: ApplicationConfiguration = serde_json::from_value(value).unwrap();
    let content = format!(
        "{}\n{}\n",
        raw("pilot", "123e4567-e89b-12d3-a456-426614174000"),
        raw("other", "123e4567-e89b-12d3-a456-426614174001")
    );
    assert!(admit(&content, "first", &config, &queue, &options)
        .await
        .iter()
        .all(|outcome| outcome.as_ref().unwrap().rejection.is_some()));
    assert_eq!(queue.counts().unwrap().queued, 0);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_mixed_fixture_admits_only_its_canonical_routes() {
    let server = server().await;
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let queue = queue();
    let content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/automation/powerautomate/logbook.jsonl"
    ));
    let outcomes = admit(content, "first", &multi_configuration(), &queue, &options).await;
    let routes = outcomes
        .iter()
        .map(|outcome| {
            let outcome = outcome.as_ref().unwrap();
            let admitted: AdmittedSubmission = serde_json::from_str(outcome.admission.as_ref().unwrap()).unwrap();
            (outcome.selector.clone().unwrap(), admitted.entry.project_id, admitted.path)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        routes,
        [
            ("acorn".to_string(), "acorn".to_string(), "acorn/index.json".to_string()),
            ("rad".to_string(), "research-data".to_string(), "research-data/index.json".to_string()),
            ("acorn-tooling".to_string(), "acorn".to_string(), "acorn/index.json".to_string()),
        ]
    );
    assert_eq!(queue.counts().unwrap().queued, 3);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_mixed_selector_rejections_leave_valid_project_queued() {
    let server = server().await;
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let queue = queue();
    let mut config = serde_json::to_value(multi_configuration()).unwrap();
    config["folders"][1]["aliases"] = serde_json::json!(["rad", "shared"]);
    config["folders"][0]["aliases"] = serde_json::json!(["acorn-tooling", "shared"]);
    let config: ApplicationConfiguration = serde_json::from_value(config).unwrap();
    let source = raw("acorn", "123e4567-e89b-12d3-a456-426614174000");
    let missing = source.replace("\"projectId\":\"acorn\",", "");
    let unknown = raw("unknown", "123e4567-e89b-12d3-a456-426614174001");
    let ambiguous = raw("shared", "123e4567-e89b-12d3-a456-426614174002");
    let conflicting = source.replace("\"projectId\":\"acorn\"", "\"projectId\":\"acorn\",\"folderName\":\"rad\"");
    let unauthorized = raw("rad", "123e4567-e89b-12d3-a456-426614174003");
    let valid = raw("acorn-tooling", "123e4567-e89b-12d3-a456-426614174004");
    let content = [missing, unknown, ambiguous, conflicting, unauthorized, valid].join("\n") + "\n";
    let outcomes = admit(&content, "first", &config, &queue, &options).await;
    let reasons = outcomes[..5]
        .iter()
        .map(|outcome| outcome.as_ref().unwrap().rejection.as_ref().unwrap().to_lowercase())
        .collect::<Vec<_>>();
    assert!(reasons[0].contains("projectid"));
    assert!(reasons[1].contains("unknown"));
    assert!(reasons[2].contains("ambiguous"));
    assert!(reasons[3].contains("unknown field"));
    assert!(reasons[4].contains("not authorized"));
    assert!(outcomes[5].as_ref().unwrap().admission.is_some());
    assert_eq!(queue.counts().unwrap().queued, 1);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_rejected_records_do_not_block_valid_projects_or_reserve_ids() {
    let server = server().await;
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let queue = queue();
    let unauthorized = raw("pilot", "123e4567-e89b-12d3-a456-426614174000")
        .replace("Scientist@Example.org", "outsider@example.org")
        .replace("scientist@example.org", "outsider@example.org");
    let content = format!(
        "{{bad\n{}\n{}\n{}\n",
        unauthorized,
        raw("missing", "123e4567-e89b-12d3-a456-426614174001"),
        raw("other", "123e4567-e89b-12d3-a456-426614174002")
    );
    let outcomes = admit(&content, "first", &configuration(), &queue, &options).await;
    assert!(outcomes[..3].iter().all(|outcome| outcome.as_ref().unwrap().rejection.is_some()));
    assert!(outcomes[3].as_ref().unwrap().admission.is_some());
    let corrected = format!("{}\n", raw("pilot", "123e4567-e89b-12d3-a456-426614174000"));
    assert!(admit(&corrected, "second", &configuration(), &queue, &options).await[0]
        .as_ref()
        .unwrap()
        .admission
        .is_some());
    assert_eq!(queue.counts().unwrap().queued, 2);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_restart_replay_uses_original_route_and_rejects_changed_bytes() {
    let server = server().await;
    let options = Options::with_token("test").domain(&server.base_url).identifier("42").build();
    let database = temp_dir().join(format!("acorn-jsonl-restart-{}.db", nanoid::nanoid!()));
    let queue = OperationQueue::from(database.clone());
    let content = format!("{}\n", raw("folder-name", "123e4567-e89b-12d3-a456-426614174000"));
    assert!(admit(&content, "first", &configuration(), &queue, &options).await[0]
        .as_ref()
        .unwrap()
        .admission
        .is_some());
    let restarted = OperationQueue::from(database);
    let mut changed = configuration();
    changed.folders.as_mut().unwrap().iter_mut().for_each(|folder| {
        folder.path = Some("moved/index.json".to_string());
        folder.aliases = None;
    });
    let outcomes = admit(&content, "second", &changed, &restarted, &options).await;
    let payload: AdmittedSubmission = serde_json::from_str(outcomes[0].as_ref().unwrap().admission.as_ref().unwrap()).unwrap();
    assert_eq!(payload.path, "pilot/index.json");
    let formatted = format!(" {}", content);
    assert!(admit(&formatted, "third", &changed, &restarted, &options).await[0]
        .as_ref()
        .unwrap()
        .rejection
        .is_some());
    assert!(!is_admitted(&snapshot(&formatted, "third").lines[0], &restarted).unwrap());
    assert_eq!(restarted.counts().unwrap().queued, 1);
    server.stop().await.unwrap();
}
#[test]
fn test_source_identity_uses_existing_endpoint_scheme_and_removes_credentials() {
    let config = configuration();
    let intake = config.power_automate().unwrap().intake.as_ref().unwrap();
    let options = Options::with_token("secret-token").domain("gitlab.example.org").identifier("42").build();
    let source = source_identity(intake, &options).unwrap();
    assert!(source.contains("https://gitlab.example.org"));
    assert!(!source.contains("secret-token"));
    assert_ne!(source, source_identity(intake, &options.clone().with_identifier("43")).unwrap());
    let other_branch = application::PowerAutomateIntake {
        branch: "other".to_string(),
        ..intake.clone()
    };
    assert_ne!(source, source_identity(&other_branch, &options).unwrap());
}
