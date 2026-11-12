use super::resolve::{DocumentPath, DocumentResolver, ResolvedDocument};
use crate::io::config::application::auth::SourceProvider;
use crate::io::{api::graph::drive::DriveItem, config::application::auth::SharePointSource, ApiResult};
use crate::test::server::TestServer;
use acorn_host::auth::{config::Resource, AuthResult, TokenAcquisition};
use acorn_host::auth::{AccessTokenProvider, TokenRequest};
use acorn_schema::validation::Validate;
use alloc::sync::Arc;
use async_trait::async_trait;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, Uri},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::{Mutex, PoisonError};

#[derive(Clone, Copy)]
enum Case {
    Healthy,
    Ambiguous,
    Denied,
    HostileContinuation,
    Paginated,
    TooManyPages,
    Moved,
    Deleted,
}
#[derive(Clone)]
struct Mock {
    case: Case,
    requests: Arc<Mutex<Vec<String>>>,
}
struct InteractionOnly;
fn set_field(value: &mut Value, key: &str, replacement: Value) {
    value.as_object_mut().expect("JSON object fixture").insert(key.to_string(), replacement);
}
#[async_trait]
impl AccessTokenProvider for InteractionOnly {
    async fn acquire(&self, request: &TokenRequest) -> AuthResult<TokenAcquisition> {
        Ok(TokenAcquisition::InteractionNeeded(acorn_host::auth::InteractionNeeded {
            authority: "https://login.microsoftonline.com/test".to_string(),
            flow: acorn_host::auth::config::GrantFlow::Device,
            profile: "test".to_string(),
            resource: request.resource.clone(),
        }))
    }
}
fn source() -> DocumentPath {
    DocumentPath {
        host: "example.sharepoint.com".to_string(),
        item_path: "a%20b/%C3%A9.jsonl".to_string(),
        library_path: "/Shared%20Documents".to_string(),
        pinned: None,
        request: TokenRequest {
            account: None,
            resource: Resource::try_new("https://graph.microsoft.com", ["Files.Read".to_string()]).expect("resource"),
        },
        site_path: "/sites/Example".to_string(),
    }
}
fn configured_source(scopes: Option<Vec<&str>>) -> SharePointSource {
    SharePointSource {
        drive_id: None,
        host: "example.sharepoint.com".to_string(),
        id: "source".to_string(),
        item_id: None,
        item_path: "a%20b/%C3%A9.jsonl".to_string(),
        library_path: "/Shared%20Documents".to_string(),
        profile: "default".to_string(),
        provider: SourceProvider::Sharepoint,
        resource: "graph".to_string(),
        scopes: scopes.map(|scopes| scopes.into_iter().map(str::to_string).collect()),
        site_id: None,
        site_path: "/sites/Example".to_string(),
    }
}
fn request(scopes: &[&str]) -> TokenRequest {
    TokenRequest {
        account: None,
        resource: Resource::try_new("https://graph.microsoft.com", scopes.iter().map(|scope| (*scope).to_string())).expect("valid request resource"),
    }
}
async fn handler(State(mock): State<Mock>, uri: Uri, headers: HeaderMap) -> (StatusCode, Json<Value>) {
    let path = uri.path().to_string();
    let auth = headers.get("authorization").and_then(|value| value.to_str().ok());
    mock.requests.lock().unwrap_or_else(PoisonError::into_inner).push(format!(
        "{} {}",
        uri.path_and_query().map_or(path.as_str(), |value| value.as_str()),
        auth.unwrap_or_default()
    ));
    if auth != Some("Bearer test-token") {
        (StatusCode::UNAUTHORIZED, Json(json!({})))
    } else if path.contains("/sites/example.sharepoint.com:") || path.ends_with("/sites/site-1") || path.ends_with("/sites/wrong") {
        (
            StatusCode::OK,
            Json(
                serde_json::from_str(include_str!("../../../../../../../tests/fixtures/sharepoint-intake/resolve/site.json")).expect("site fixture"),
            ),
        )
    } else if path.ends_with("/drives/drive-1") {
        (
            StatusCode::OK,
            Json(
                serde_json::from_str(include_str!("../../../../../../../tests/fixtures/sharepoint-intake/resolve/drive.json"))
                    .expect("drive fixture"),
            ),
        )
    } else if path.contains("/drives/drive-1/root:") || path.ends_with("/items/item-1") {
        let mut item: Value =
            serde_json::from_str(include_str!("../../../../../../../tests/fixtures/sharepoint-intake/resolve/item.json")).expect("item fixture");
        if matches!(mock.case, Case::Moved) {
            set_field(
                &mut item,
                "webUrl",
                json!("https://example.sharepoint.com/sites/Example/Shared%20Documents/elsewhere.jsonl"),
            );
        }
        if matches!(mock.case, Case::Deleted) {
            set_field(&mut item, "deleted", json!({}));
        }
        (StatusCode::OK, Json(item))
    } else if path.ends_with("/sites/site-1/drives") {
        if matches!(mock.case, Case::Denied) {
            (StatusCode::FORBIDDEN, Json(json!({})))
        } else {
            let drive: Value = serde_json::from_str(include_str!("../../../../../../../tests/fixtures/sharepoint-intake/resolve/drive.json"))
                .expect("drive fixture");
            let value = if matches!(mock.case, Case::Ambiguous) {
                vec![drive.clone(), drive]
            } else {
                vec![drive]
            };
            let mut page = json!({ "value": value });
            if matches!(mock.case, Case::Paginated) && uri.query() == Some("%24top=100") {
                let host = headers.get("host").and_then(|value| value.to_str().ok()).unwrap_or_default();
                page = json!({
                    "value": [],
                    "@odata.nextLink": format!("http://{host}/v1.0/sites/site-1/drives?$skiptoken=a%2Bb%3D&$top=10")
                });
            }
            if matches!(mock.case, Case::HostileContinuation) {
                set_field(&mut page, "@odata.nextLink", json!("https://attacker.example/v1.0/sites/site-1/drives"));
            }
            if matches!(mock.case, Case::TooManyPages) {
                let host = headers.get("host").and_then(|value| value.to_str().ok()).unwrap_or_default();
                set_field(
                    &mut page,
                    "@odata.nextLink",
                    json!(format!("http://{host}/v1.0/sites/site-1/drives?$skiptoken=again")),
                );
            }
            (StatusCode::OK, Json(page))
        }
    } else {
        (StatusCode::NOT_FOUND, Json(json!({})))
    }
}
async fn run(case: Case, source: &DocumentPath) -> (ApiResult<ResolvedDocument>, Vec<String>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let mock = Mock {
        case,
        requests: Arc::clone(&requests),
    };
    let server = TestServer::start(Router::new().fallback(get(handler)).with_state(mock))
        .await
        .expect("mock server");
    let result = DocumentResolver
        .resolve_with_base(source, &format!("{}/v1.0", server.base_url), "test-token")
        .await;
    server.stop().await.expect("stop mock server");
    let logged = requests.lock().unwrap_or_else(PoisonError::into_inner).clone();
    (result, logged)
}
#[test]
fn test_document_path_collects_field_validation_errors() {
    let source = DocumentPath {
        host: "user@example.sharepoint.com".into(),
        site_path: "/sites/..".into(),
        library_path: "Shared Documents".into(),
        item_path: "a%2Fb.jsonl".into(),
        ..source()
    };
    let report = source.validate().expect_err("all invalid paths should be reported");
    let paths = report.iter().map(|issue| issue.path.as_str()).collect::<Vec<_>>();
    assert_eq!(paths, ["host", "site_path", "library_path", "item_path"]);
}
#[tokio::test]
async fn test_pagination_preserves_encoded_query_and_scoped_authorization() {
    let (result, requests) = run(Case::Paginated, &source()).await;
    assert_eq!(result.expect("paginated resolution").item.item_id, "item-1");
    assert_eq!(requests.len(), 4);
    assert!(requests
        .iter()
        .any(|request| request == "/v1.0/sites/site-1/drives?$skiptoken=a%2Bb%3D&$top=10 Bearer test-token"));
}
#[tokio::test]
async fn test_pinned_ids_use_direct_metadata_without_discovery() {
    let mut pinned = source();
    pinned.pinned = Some(DriveItem {
        site_id: "site-1".to_string(),
        drive_id: "drive-1".to_string(),
        item_id: "item-1".to_string(),
    });
    let (result, requests) = run(Case::Healthy, &pinned).await;
    assert_eq!(result.expect("pinned resolution").item.item_id, "item-1");
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().any(|request| request.contains("/sites/site-1 ")));
    assert!(requests.iter().any(|request| request.contains("/drives/drive-1 ")));
    assert!(requests.iter().any(|request| request.contains("/items/item-1 ")));
}
#[tokio::test]
async fn test_rejects_ambiguous_denied_and_unbounded_discovery() {
    for (case, expected) in [
        (Case::Ambiguous, "ambiguous"),
        (Case::Denied, "denied"),
        (Case::TooManyPages, "page limit"),
    ] {
        let (result, _) = run(case, &source()).await;
        assert!(result.expect_err("must reject").to_string().contains(expected));
    }
}
#[tokio::test]
async fn test_rejects_hostile_continuation_without_sending_token() {
    let (result, requests) = run(Case::HostileContinuation, &source()).await;
    assert!(result.expect_err("must reject").to_string().contains("continuation"));
    assert_eq!(requests.len(), 2);
}
#[tokio::test]
async fn test_rejects_moved_deleted_and_mismatched_pins() {
    for case in [Case::Moved, Case::Deleted] {
        assert!(run(case, &source()).await.0.is_err());
    }
    let mut pinned = source();
    pinned.pinned = Some(DriveItem {
        site_id: "wrong".to_string(),
        drive_id: "drive-1".to_string(),
        item_id: "item-1".to_string(),
    });
    assert!(run(Case::Healthy, &pinned)
        .await
        .0
        .expect_err("pin mismatch")
        .to_string()
        .contains("site ID"));
}
#[tokio::test]
async fn test_requires_scoped_graph_sign_in_before_network() {
    let result = DocumentResolver.resolve(&source(), &InteractionOnly).await;
    assert!(result.expect_err("interaction needed").to_string().contains("sign-in"));
    let mut hostile = source();
    hostile.item_path = "..%2Fescape.jsonl".to_string();
    assert!(DocumentResolver.resolve(&hostile, &InteractionOnly).await.is_err());
}
#[tokio::test]
async fn test_resolves_encoded_spaces_and_unicode_without_token_redirect() {
    let (result, requests) = run(Case::Healthy, &source()).await;
    let resolved = result.expect("resolved document");
    assert_eq!(resolved.item.site_id, "site-1");
    assert_eq!(resolved.item.drive_id, "drive-1");
    assert_eq!(resolved.item.item_id, "item-1");
    assert_eq!(resolved.revision, "etag-1");
    assert!(requests.iter().any(|request| request.contains("a%20b/%C3%A9.jsonl")));
    assert_eq!(requests.len(), 3);
}
#[test]
fn test_source_scope_restriction_cannot_be_bypassed_by_request() {
    let cases = [
        (Some(vec!["Files.Read"]), vec!["Files.Read", "Sites.Read.All"], false),
        (Some(vec!["Files.Read"]), vec!["Sites.Read.All"], false),
        (Some(vec!["Files.Read"]), vec!["Files.Read"], true),
        (Some(vec!["Sites.Read.All", "Files.Read"]), vec!["Files.Read"], true),
        (Some(vec!["Sites.Read.All", "Files.Read"]), vec!["Files.Read", "Sites.Read.All"], true),
        (None, vec!["Files.Read", "Sites.Read.All"], true),
        (Some(vec![]), vec!["Files.Read"], false),
    ];
    for (allowed, requested, accepted) in cases {
        let result = DocumentPath::from_source(&configured_source(allowed), request(&requested));
        assert_eq!(result.is_ok(), accepted, "requested scopes: {requested:?}");
    }
}
