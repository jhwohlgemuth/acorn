#![allow(clippy::indexing_slicing)]
use super::*;
use acorn_service::AcornService;
use axum::body::{to_bytes, Body};
use axum::http::Request;
use tower::ServiceExt;

fn app(ready: bool) -> Router {
    let registry = OperationRegistry::acorn(AcornService, "1.2.3", None).expect("registry");
    let ready = Arc::new(AtomicBool::new(ready));
    HttpState::new(
        registry,
        Arc::<str>::from("secret"),
        InvocationContext::default(),
        ready,
        Duration::from_secs(1),
    )
    .router(1024)
}

#[tokio::test]
async fn test_health_is_public_and_readiness_tracks_state() {
    let health = app(false)
        .oneshot(Request::builder().uri("/healthz").body(Body::empty()).expect("request"))
        .await
        .expect("health response");
    assert_eq!(health.status(), StatusCode::NO_CONTENT);
    let readiness = app(false)
        .oneshot(Request::builder().uri("/readyz").body(Body::empty()).expect("request"))
        .await
        .expect("readiness response");
    assert_eq!(readiness.status(), StatusCode::SERVICE_UNAVAILABLE);
}
#[tokio::test]
async fn test_rpc_dispatches_the_shared_registry() {
    let response = app(true)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header(AUTHORIZATION, "Bearer secret")
                .header(CONTENT_TYPE, "application/json; charset=utf-8")
                .body(Body::from(r#"{"jsonrpc":"2.0","id":1,"method":"acorn.version","params":{}}"#))
                .expect("request"),
        )
        .await
        .expect("RPC response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024).await.expect("response body");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("JSON-RPC response");
    assert_eq!(value["result"]["version"], "1.2.3");
}
#[tokio::test]
async fn test_rpc_requires_bearer_and_json_media_type() {
    let unauthorized = app(true)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"jsonrpc":"2.0","id":1,"method":"acorn.version"}"#))
                .expect("request"),
        )
        .await
        .expect("unauthorized response");
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
    let unsupported = app(true)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header(AUTHORIZATION, "Bearer secret")
                .header(CONTENT_TYPE, "text/plain")
                .body(Body::from("{}"))
                .expect("request"),
        )
        .await
        .expect("media response");
    assert_eq!(unsupported.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
