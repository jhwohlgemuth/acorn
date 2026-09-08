use crate::io::api::swhid::resolve_with;
use crate::test::server::TestServer;
use acorn_schema::pid::{PersistentIdentifierParse, SWHID};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

fn swhid(value: &str) -> SWHID {
    SWHID::from_string(value)
}

#[tokio::test]
async fn test_resolve_rejects_empty_identifier() {
    let err = resolve_with("https://example.invalid", &SWHID::default()).await.unwrap_err().to_string();
    assert!(err.contains("valid SWHID"), "{err}");
}
#[tokio::test]
async fn test_resolve_status_codes_are_distinct() {
    let router = Router::new()
        .route("/bad_request/{*path}", get(|| async { (StatusCode::BAD_REQUEST, "bad") }))
        .route("/unauthorized/{*path}", get(|| async { (StatusCode::UNAUTHORIZED, "unauthorized") }))
        .route("/forbidden/{*path}", get(|| async { (StatusCode::FORBIDDEN, "forbidden") }))
        .route("/not_found/{*path}", get(|| async { (StatusCode::NOT_FOUND, "not found") }))
        .route("/too_many/{*path}", get(|| async { (StatusCode::TOO_MANY_REQUESTS, "rate limited") }))
        .route(
            "/malformed/{*path}",
            get(|| async { Json(serde_json::json!("not-an-object-but-valid-json")) }),
        )
        .route("/invalid_json/{*path}", get(|| async { "not-json" }));
    let server = TestServer::start(router).await.expect("server should start");
    let cases = [
        ("/bad_request", "HTTP 400"),
        ("/unauthorized", "HTTP 401"),
        ("/forbidden", "HTTP 403"),
        ("/not_found", "HTTP 404"),
        ("/too_many", "HTTP 429"),
    ];
    for (prefix, fragment) in cases {
        let base = format!("{}{prefix}", server.base_url);
        let err = resolve_with(&base, &swhid("swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2"))
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains(fragment), "expected {fragment} in {err} for {prefix}");
        assert!(err.contains("Software Heritage could not resolve"), "{err}");
    }
    // malformed JSON that is valid JSON but not the expected shape still parses (serde_json::Value accepts any) — check invalid JSON
    let invalid_base = format!("{}/invalid_json", server.base_url);
    let err = resolve_with(&invalid_base, &swhid("swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2"))
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("invalid resolver metadata"), "{err}");
    server.stop().await.expect("server should stop");
}
#[tokio::test]
async fn test_resolve_success_returns_json() {
    let router = Router::new().route(
        "/{*path}",
        get(|| async { Json(serde_json::json!({"swhid": "swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2", "origin": "https://example.org"})) }),
    );
    let server = TestServer::start(router).await.expect("server should start");
    let value = resolve_with(&server.base_url, &swhid("swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2"))
        .await
        .expect("should resolve");
    assert_eq!(
        value.get("swhid").and_then(|v| v.as_str()),
        Some("swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2")
    );
    server.stop().await.expect("server should stop");
}
#[tokio::test]
async fn test_resolve_transport_failure_is_surfaced() {
    let err = resolve_with("http://127.0.0.1:1", &swhid("swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2"))
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("Software Heritage could not resolve"), "{err}");
}
