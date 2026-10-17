#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
//! Shared test utilities for `acorn-lib` unit tests.
use core::time::Duration;
#[cfg(feature = "agentic")]
use data_encoding::BASE64;
#[cfg(feature = "agentic")]
pub(crate) use gitlab_review::review_server;
use jiff::Timestamp;
#[cfg(feature = "agentic")]
use ring::hmac as ring_hmac;
use std::{path::PathBuf, time::SystemTime};

/// Computes a Standard Webhooks HMAC-SHA256 signature for the given message components.
#[cfg(feature = "agentic")]
pub(crate) fn compute_standard_webhook_signature(signing_token_base64: &str, webhook_id: &str, timestamp: &str, body: &[u8]) -> String {
    let key_bytes = BASE64.decode(signing_token_base64.strip_prefix("whsec_").unwrap().as_bytes()).unwrap();
    let key = ring_hmac::Key::new(ring_hmac::HMAC_SHA256, &key_bytes);
    let mut message = format!("{webhook_id}.{timestamp}.").into_bytes();
    message.extend_from_slice(body);
    format!("v1,{}", BASE64.encode(ring_hmac::sign(&key, &message).as_ref()))
}
/// Returns the current UTC time as a Unix timestamp string.
#[cfg(feature = "agentic")]
pub(crate) fn current_unix_timestamp() -> String {
    Timestamp::now().as_second().to_string()
}
/// Get path to test fixtures relative to workspace root
pub fn fixture_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join(path)
}
/// Returns a Unix timestamp string that is `stale_by_secs` seconds in the past.
#[cfg(feature = "agentic")]
pub(crate) fn stale_unix_timestamp(stale_by_secs: u64) -> String {
    (Timestamp::now().as_second() - stale_by_secs as i64).to_string()
}
/// Build a unique temporary directory path under `target/test_artifacts`.
pub(crate) fn temp_dir(name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_nanos(0))
        .as_nanos();
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target")
        .join("test_artifacts")
        .join(format!("{name}-{timestamp}"))
}
/// Build a unique path under `target/test_artifacts`.
pub(crate) fn unique_path(name: &str, extension: &str) -> PathBuf {
    let timestamp = Timestamp::now().as_nanosecond();
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target")
        .join("test_artifacts")
        .join(format!("{name}-{timestamp}.{extension}"))
}

#[cfg(feature = "agentic")]
mod gitlab_review {
    use crate::test::server::TestServer;
    use alloc::collections::BTreeMap;
    use alloc::sync::Arc;
    use axum::routing::{get, post};
    use axum::{extract::State, http::StatusCode, Json, Router};
    use std::sync::Mutex;
    #[derive(Clone)]
    pub(crate) struct ReviewState {
        pub(crate) branches: BTreeMap<String, String>,
        pub(crate) content: BTreeMap<(String, String), String>,
        pub(crate) inbox: String,
        pub(crate) merge_requests: Vec<serde_json::Value>,
        pub(crate) review_writes: Vec<(String, String)>,
        pub(crate) sequence: u64,
    }
    impl ReviewState {
        fn new(inbox: String) -> Self {
            let base = format!("{:040x}", 1);
            let activity = serde_json::to_string(&acorn_schema::research_activity::ResearchActivity::default()).unwrap();
            Self {
                branches: BTreeMap::from([("main".to_string(), base.clone())]),
                content: BTreeMap::from([
                    (("main".to_string(), "acorn/index.json".to_string()), activity.clone()),
                    (("main".to_string(), "research-data/index.json".to_string()), activity),
                ]),
                inbox,
                merge_requests: Vec::new(),
                review_writes: Vec::new(),
                sequence: 1,
            }
        }
        fn branch_for(&self, reference: &str) -> String {
            self.branches
                .iter()
                .find(|(_, head)| head.as_str() == reference)
                .map_or_else(|| reference.to_string(), |(branch, _)| branch.clone())
        }
    }
    pub(crate) async fn review_server(inbox: String) -> (TestServer, Arc<Mutex<ReviewState>>) {
        let state = Arc::new(Mutex::new(ReviewState::new(inbox)));
        let router = Router::new()
            .route("/api/v4/user", get(|| async { Json(serde_json::json!({"id":99})) }))
            .route("/api/v4/projects/42", get(|| async { Json(serde_json::json!({"id":42,"default_branch":"main"})) }))
            .route("/api/v4/projects/42/packages", get(|| async { Json(serde_json::json!([])) }))
            .route("/api/v4/projects/42/repository/commits/{revision}", get(|State(state): State<Arc<Mutex<ReviewState>>>| async move {
                Json(serde_json::json!({"id":state.lock().unwrap().branches["main"]}))
            }))
            .route("/api/v4/projects/42/repository/branches/{*branch}", get(|State(state): State<Arc<Mutex<ReviewState>>>, axum::extract::Path(branch): axum::extract::Path<String>| async move {
                let state = state.lock().unwrap();
                match state.branches.get(branch.trim_start_matches('/')) {
                    | Some(head) => (StatusCode::OK, Json(serde_json::json!({"name":branch.trim_start_matches('/'),"commit":{"id":head}}))),
                    | None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"message":"404 Branch Not Found"}))),
                }
            }))
            .route("/api/v4/projects/42/repository/branches", post(|State(state): State<Arc<Mutex<ReviewState>>>, Json(body): Json<serde_json::Value>| async move {
                let mut state = state.lock().unwrap();
                let branch = body["branch"].as_str().unwrap().to_string();
                let head = body["ref"].as_str().unwrap().to_string();
                state.branches.insert(branch.clone(), head.clone());
                (StatusCode::CREATED, Json(serde_json::json!({"name":branch,"commit":{"id":head}})))
            }))
            .route("/api/v4/projects/42/repository/files/{*path}", get(|State(state): State<Arc<Mutex<ReviewState>>>, axum::extract::Path(path): axum::extract::Path<String>, uri: axum::http::Uri| async move {
                let state = state.lock().unwrap();
                let path = path.trim_start_matches('/');
                let reference = uri.query().unwrap_or("").split('&').find_map(|item| item.strip_prefix("ref=")).and_then(|item| urlencoding::decode(item).ok()).map_or("main".to_string(), |item| item.into_owned());
                let content = if path == ".acorn/intake/logbook.jsonl" {
                    Some(state.inbox.as_str())
                } else {
                    state.content.get(&(state.branch_for(&reference), path.to_string())).map(String::as_str)
                        .or_else(|| state.content.get(&("main".to_string(), path.to_string())).map(String::as_str))
                };
                match content {
                    | Some(content) => (StatusCode::OK, Json(serde_json::json!({"file_path":path,"encoding":"base64","content":data_encoding::BASE64.encode(content.as_bytes()),"size":content.len(),"last_commit_id":format!("{:040x}", state.sequence)}))),
                    | None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"message":"404 File Not Found"}))),
                }
            }))
            .route("/api/v4/projects/42/repository/commits", post(|State(state): State<Arc<Mutex<ReviewState>>>, Json(body): Json<serde_json::Value>| async move {
                let mut state = state.lock().unwrap();
                let branch = body["branch"].as_str().unwrap().to_string();
                let path = body["actions"][0]["file_path"].as_str().unwrap().to_string();
                let content = body["actions"][0]["content"].as_str().unwrap().to_string();
                state.sequence += 1;
                let head = format!("{:040x}", state.sequence);
                state.branches.insert(branch.clone(), head.clone());
                if path == ".acorn/intake/logbook.jsonl" {
                    state.inbox = content;
                } else {
                    state.review_writes.push((branch.clone(), path.clone()));
                    state.content.insert((branch, path), content);
                }
                (StatusCode::CREATED, Json(serde_json::json!({"id":head})))
            }))
            .route("/api/v4/projects/42/merge_requests", get(|State(state): State<Arc<Mutex<ReviewState>>>| async move {
                Json(state.lock().unwrap().merge_requests.clone())
            }).post(|State(state): State<Arc<Mutex<ReviewState>>>, Json(body): Json<serde_json::Value>| async move {
                let mut state = state.lock().unwrap();
                let iid = state.merge_requests.len() + 1;
                let request = serde_json::json!({"iid":iid,"source_branch":body["source_branch"],"state":"opened","web_url":format!("https://gitlab.example/merge_requests/{iid}")});
                state.merge_requests.push(request.clone());
                (StatusCode::CREATED, Json(request))
            }))
            .route("/api/v4/projects/42/merge_requests/{iid}/notes", get(|| async { Json(serde_json::json!([])) }).post(|Json(body): Json<serde_json::Value>| async move {
                (StatusCode::CREATED, Json(serde_json::json!({"id":1,"body":body["body"],"author":{"id":99}})))
            }))
            .with_state(Arc::clone(&state));
        (TestServer::start(router).await.unwrap(), state)
    }
}
