#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::*;
use crate::io::api::gitlab::{ReportManifest, ReportStatus};
use crate::io::api::webhooks::store::{EnqueueStatus, OperationQueue};
use crate::io::api::webhooks::{self, CallbackResolver, ErrorKind, InboundRequest, WebhookProvider};
use crate::io::config::application::{ApplicationConfiguration, Destination, FolderRegistry};
use crate::io::workflow::{LogbookProposal, ProcessRepositoryResponse};
use acorn_core::OneOrMany;
use acorn_schema::research_activity::{Logbook, LogbookEntry, LogbookState, ReportingWindow, ResearchActivity};
use acorn_schema::validation::Validate;
use alloc::collections::BTreeSet;
use http::{HeaderMap, HeaderValue};
use jiff::Timestamp;
use std::env::temp_dir;
use tower::ServiceExt;

impl FolderRegistry {
    /// Return the routing registry derived from the shared fixture document.
    fn fixture() -> Self {
        document().registry()
    }
}
impl PowerAutomateConfig {
    /// Return the PowerAutomate options carried by the shared fixture document.
    fn fixture() -> Self {
        document().power_automate().cloned().expect("PowerAutomate options")
    }
}
fn document() -> ApplicationConfiguration {
    serde_json::from_value(serde_json::json!({
        "folders": [{
            "id": "pilot",
            "members": ["Scientist@Example.org"],
            "path": "pilot/index.json"
        }],
        "variables": {
            "custom": {
                "power_automate.callback_url": "POWER_AUTOMATE_CALLBACK_URL",
                "power_automate.inbound_secret": "POWER_AUTOMATE_INBOUND_SECRET"
            }
        },
        "options": {
            "powerAutomate": {
                "callback_destination": "pilot-flow",
                "gitlab_project": "42",
                "package": "automated-artifacts"
            }
        }
    }))
    .expect("valid configuration fixture")
}
fn intake_headers(delivery_id: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-acorn-delivery-id", HeaderValue::from_str(delivery_id).expect("delivery header"));
    headers.insert("x-acorn-webhook-secret", HeaderValue::from_static("test-secret"));
    headers
}
fn logbook_fixture() -> Logbook {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/logbook/entries.json"
    )))
    .expect("valid PowerAutomate logbook fixture")
}
fn submission() -> Submission {
    serde_json::from_value(serde_json::json!({
        "submissionId": "123e4567-e89b-12d3-a456-426614174000",
        "projectId": "pilot",
        "submittedAt": "2026-09-18T12:01:00Z",
        "submitter": {"email": "SCIENTIST@example.org"},
        "entry": {
            "timestamp": "2026-09-18T12:00:00Z",
            "category": "software",
            "state": "completed",
            "summary": "Released the pilot",
            "items": ["Tagged v1"]
        }
    }))
    .expect("valid submission fixture")
}

#[test]
fn test_callback_resolution_is_allowlisted_and_attempt_time_only() {
    let resolver = PowerAutomateCallbackResolver::new(PowerAutomateConfig::fixture().callback_destination, &FolderRegistry::fixture());
    assert_eq!(
        resolver.resolve("other").err().expect("destination allowlist").kind(),
        ErrorKind::InvalidTarget
    );
    temp_env::with_var("POWER_AUTOMATE_CALLBACK_URL", Some("https://example.test/hooks/secret"), || {
        assert!(resolver.resolve("pilot-flow").is_ok());
    });
}
#[test]
fn test_flow_is_authenticated_durable_duplicate_safe_and_reportable() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/automation/powerautomate/form.json"
    ));
    let body = fixture.as_bytes();
    let now = "2026-09-18T12:02:00Z".parse::<Timestamp>().expect("test timestamp");
    let provider = PowerAutomateProvider::new(PowerAutomateConfig::fixture(), FolderRegistry::fixture(), "test-secret").expect("provider");
    let headers = intake_headers("delivery-1");
    let prepared = webhooks::prepare(
        &provider,
        InboundRequest {
            headers: &headers,
            raw_body: body,
        },
        now,
    )
    .expect("authenticated intake");
    let queue = OperationQueue::from(temp_dir().join(format!("acorn-powerautomate-{}.db", nanoid::nanoid!())));
    assert_eq!(queue.enqueue_prepared(&prepared).expect("durable enqueue"), [EnqueueStatus::Inserted]);

    let duplicate_headers = intake_headers("delivery-2");
    let duplicate = webhooks::prepare(
        &provider,
        InboundRequest {
            headers: &duplicate_headers,
            raw_body: body,
        },
        now,
    )
    .expect("duplicate authenticated intake");
    assert_eq!(
        queue.enqueue_prepared(&duplicate).expect("duplicate enqueue"),
        [EnqueueStatus::DuplicateOperation]
    );
    assert_eq!(queue.counts().expect("queue counts").queued, 1);

    let form = serde_json::from_str::<Submission>(fixture).expect("form fixture");
    let entry = form.entry(provider.routing()).expect("normalized entry");
    let content = serde_json::to_string_pretty(&ResearchActivity::default()).expect("activity fixture");
    let changes = ProcessRepositoryResponse::try_from(LogbookProposal {
        base_revision: "0123456789abcdef".to_string(),
        content,
        entry,
        path: "pilot/index.json".to_string(),
    })
    .expect("logbook proposal");
    let activity = serde_json::from_str::<ResearchActivity>(&changes.files.first().expect("updated file").content).expect("updated activity");
    assert_eq!(
        activity
            .logbook
            .as_ref()
            .expect("logbook")
            .entries
            .first()
            .expect("form entry")
            .project_id,
        "pilot"
    );
    let window = ReportingWindow::new(None, "2026-10-01T00:00:00Z").expect("reporting window");
    let updates = activity.updates_within(&window).expect("bounded updates");
    let manifest = ReportManifest::new(
        "automated-artifacts",
        "pilot",
        "0123456789abcdef",
        &window,
        updates.entries.iter().map(|entry| entry.identifier.clone()).collect(),
        "100",
        "200",
        Some(b"%PDF-1.7 report"),
        "https://gitlab.example/package",
    )
    .expect("report manifest");
    assert_eq!(manifest.entry_identifiers, ["form-123-e-4567-e-89-b-12-d-3-a-456-426614174000"]);
    assert_eq!(manifest.status, ReportStatus::Published);
}
#[test]
fn test_form_conversion_forces_effect_fields_and_rejects_late_entries() {
    let submission = submission();
    let entry = submission.entry(&FolderRegistry::fixture()).expect("authorized form");
    assert_eq!(entry.identifier, "form-123-e-4567-e-89-b-12-d-3-a-456-426614174000");
    assert_eq!(entry.origin, acorn_schema::research_activity::LogbookEntryOrigin::Manual);
    assert_eq!(entry.project_id, "pilot");
    assert!(!entry.archived);
    assert!(entry.discovery.is_none());
    assert!(submission.entry_after(&FolderRegistry::fixture(), Some("2026-09-18T12:00:01Z")).is_err());
}
#[test]
fn test_form_payload_is_strict_and_authorized() {
    assert!(submission().submitter.validate().is_ok());
    let extra = serde_json::json!({
        "submissionId": "123e4567-e89b-12d3-a456-426614174000",
        "projectId": "pilot",
        "submittedAt": "2026-09-18T12:01:00Z",
        "submitter": {"email": "scientist@example.org"},
        "entry": {
            "timestamp": "2026-09-18T12:00:00Z",
            "category": "software",
            "state": "completed",
            "summary": "Released the pilot",
            "archived": true
        }
    });
    assert!(serde_json::from_value::<Submission>(extra).is_err());
    let mut unauthorized = submission();
    unauthorized.submitter.email = serde_json::from_value(serde_json::json!("other@example.org")).expect("email");
    assert!(unauthorized.entry(&FolderRegistry::fixture()).is_err());
}
#[test]
fn test_logbook_fixture_graduates_a_project_routed_entry() {
    let mut logbook = logbook_fixture();
    logbook.entries.push(
        LogbookEntry::init()
            .category("personnel")
            .identifier("powerautomate-new-contributor")
            .project_id("fullmetal-alchemist")
            .state("in-progress")
            .summary("Example Contributor")
            .timestamp("2026-03-31T00:00:00Z")
            .build(),
    );
    let activity = ResearchActivity {
        logbook: Some(logbook),
        ..ResearchActivity::default()
    };
    let graduated = activity
        .graduate(&["powerautomate-new-contributor".to_string()])
        .expect("graduate fixture-backed entry");
    let entries = &graduated.logbook.as_ref().expect("graduated logbook").entries;
    let entry = entries
        .iter()
        .find(|entry| entry.identifier == "powerautomate-new-contributor")
        .expect("graduated entry");
    assert!(entry.archived);
    assert_eq!(entry.project_id, "fullmetal-alchemist");
    assert_eq!(entry.state, LogbookState::Completed);
    assert_eq!(entries.iter().filter(|entry| entry.archived).count(), 1);
    assert_eq!(
        graduated
            .meta
            .contributors
            .as_ref()
            .and_then(|contributors| contributors.iter().next())
            .expect("contributor")
            .name,
        "Example Contributor"
    );
}
#[test]
fn test_logbook_fixture_preserves_project_routing() {
    let logbook = logbook_fixture();
    logbook.validate().expect("valid PowerAutomate logbook fixture");
    assert_eq!(logbook.entries.len(), 22);
    assert_eq!(
        logbook.entries.iter().map(|entry| entry.project_id.as_str()).collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "akira",
            "alchemy",
            "bebop",
            "bleach",
            "dragon-ball",
            "evangelion",
            "gundam",
            "naruto",
            "one-piece",
            "sailor-moon",
            "totoro",
            "trigun",
        ])
    );
}
#[test]
fn test_provider_classifies_one_stable_operation() {
    let provider = PowerAutomateProvider::new(PowerAutomateConfig::fixture(), FolderRegistry::fixture(), "test-secret").expect("provider");
    let delivery = crate::io::api::webhooks::Delivery {
        delivery_id: "delivery-1".to_string(),
        event: submission(),
    };
    let operation = provider.classify(&delivery).expect("classification");
    assert_eq!(operation.len(), 1);
    assert_eq!(
        operation.first().expect("classified operation").idempotency_key,
        "form:v1:123e4567-e89b-12d3-a456-426614174000"
    );
}
#[test]
fn test_provider_rejects_unauthenticated_form() {
    let provider = PowerAutomateProvider::new(PowerAutomateConfig::fixture(), FolderRegistry::fixture(), "test-secret").expect("provider");
    let body = serde_json::to_vec(&submission()).expect("form JSON");
    let error = provider
        .receive(
            InboundRequest {
                headers: &HeaderMap::new(),
                raw_body: &body,
            },
            Timestamp::now(),
        )
        .expect_err("unauthenticated form must fail");
    assert_eq!(error.kind(), crate::io::api::webhooks::ErrorKind::Unauthorized);
}
#[test]
fn test_registry_normalizes_allowlists_and_rejects_unsafe_paths() {
    let document = document();
    let registry = FolderRegistry::fixture();
    assert_eq!(PowerAutomateConfig::fixture().timezone(), "UTC");
    assert!(PowerAutomateConfig::fixture().validate().is_ok());
    assert!(document.validate().is_ok());
    assert_eq!(
        registry.effective_members(Some("pilot"), &|_| None).into_iter().collect::<Vec<_>>(),
        vec!["scientist@example.org".to_string()]
    );
    assert!(registry
        .authorizes("pilot", "SCIENTIST@example.org", &|_| None)
        .expect("authorized folder"));
    assert!(!registry.authorizes("pilot", "other@example.org", &|_| None).expect("authorized folder"));
    assert!(registry.authorizes("missing", "scientist@example.org", &|_| None).is_err());
    assert_eq!(
        registry.resolve_folder("pilot").expect("resolved folder").destination,
        Destination::GitLab {
            project: "42".to_string(),
            path: "pilot/index.json".to_string()
        }
    );
    assert!(registry.resolve_folder("pilot").map(|folder| folder.destination).is_ok());
    let mut invalid = PowerAutomateConfig::fixture();
    invalid.timezone = Some("America/New_York".to_string());
    assert!(invalid.validate().is_err());
}
#[test]
fn test_registry_preserves_scalar_and_array_allowlists() {
    let scalar = serde_json::from_value::<ApplicationConfiguration>(serde_json::json!({
        "folders": { "id": "pilot", "members": "scientist@example.org" }
    }))
    .expect("scalar folder configuration");
    let folders = scalar.folders.expect("scalar folders");
    assert!(matches!(folders, OneOrMany::One(_)));
    assert!(matches!(folders[0].members, Some(OneOrMany::One(_))));
    let many = serde_json::from_value::<ApplicationConfiguration>(serde_json::json!({
        "folders": [{ "id": "pilot", "members": ["scientist@example.org"] }]
    }))
    .expect("array folder configuration");
    assert!(matches!(many.folders.expect("array folders"), OneOrMany::Many(_)));
    let empty = serde_json::from_value::<ApplicationConfiguration>(serde_json::json!({
        "folders": [{ "id": "pilot", "members": [] }]
    }))
    .expect("empty allowlist configuration");
    assert!(empty.registry().effective_members(Some("pilot"), &|_| None).is_empty());
}
#[tokio::test]
async fn test_router_accepts_authenticated_form_and_returns_canonical_receipt() {
    let body = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/automation/powerautomate/form.json"
    ))
    .as_bytes()
    .to_vec();
    let headers = intake_headers("delivery-router");
    let provider = PowerAutomateProvider::new(PowerAutomateConfig::fixture(), FolderRegistry::fixture(), "test-secret").expect("provider");
    let queue = OperationQueue::from(temp_dir().join(format!("acorn-powerautomate-router-{}.db", nanoid::nanoid!())));
    let router = provider.router(queue.clone(), crate::io::api::webhooks::WebhookRuntime::default());
    let mut request = axum::http::Request::builder()
        .method("POST")
        .uri("/webhooks/powerautomate/forms")
        .body(axum::body::Body::from(body))
        .expect("form request");
    *request.headers_mut() = headers;
    let response = router.oneshot(request).await.expect("form response");
    assert_eq!(response.status(), axum::http::StatusCode::ACCEPTED);
    let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024).await.expect("receipt body");
    let receipt = serde_json::from_slice::<serde_json::Value>(&bytes).expect("form receipt");
    assert_eq!(receipt.get("disposition"), Some(&serde_json::json!("inserted")));
    assert_eq!(
        receipt.get("operationId"),
        Some(&serde_json::json!("powerautomate:form:v1:123e4567-e89b-12d3-a456-426614174000"))
    );
    assert_eq!(queue.counts().expect("queue counts").queued, 1);
}

mod jsonl;
mod local;
