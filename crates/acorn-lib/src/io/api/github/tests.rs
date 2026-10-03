use super::{commit_sha, CommitResponse, LanguagesResponse};
use crate::test::server::TestServer;
use axum::{extract::Path, routing::get, Json, Router};

#[test]
fn test_commit_response_requires_exact_sha() {
    let response: CommitResponse = serde_json::from_str(r#"{"sha":"abc123"}"#).unwrap();
    assert_eq!(response.sha, "abc123");
}
#[tokio::test]
async fn test_commit_sha_resolves_reference() {
    let router = Router::new().route(
        "/repos/acme/quest/commits/{revision}",
        get(|Path(revision): Path<String>| async move {
            assert_eq!(revision, "release/v1");
            Json(serde_json::json!({"sha": "abc123"}))
        }),
    );
    let server = TestServer::start(router).await.unwrap();
    let sha = commit_sha(&server.base_url, "acme/quest", "release/v1").await.unwrap();
    assert_eq!(sha, "abc123");
    server.stop().await.unwrap();
}
#[test]
fn test_language_response_returns_deterministic_names() {
    let response: LanguagesResponse = serde_json::from_str(r#"{"Rust": 900, "Python": 100}"#).unwrap();
    assert_eq!(response.0.into_keys().collect::<Vec<_>>(), vec!["Python", "Rust"]);
}
