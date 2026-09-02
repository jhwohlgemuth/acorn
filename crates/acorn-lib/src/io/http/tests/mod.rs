use super::*;
use crate::test::server::TestServer;
use crate::util::constants::env::{HTTP_MAX_RETRIES, HTTP_STALL_TIMEOUT};
use acorn_core::prelude::Arc;
use axum::body::{Body, Bytes};
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use core::sync::atomic::{AtomicUsize, Ordering};
use futures::{stream, StreamExt};
use jiff::SignedDuration;
use std::env;
use std::sync::{Mutex, MutexGuard};

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct RestoreEnv {
    _lock: MutexGuard<'static, ()>,
    max_retries: Option<String>,
    stall_timeout: Option<String>,
}
impl RestoreEnv {
    fn apply(max_retries: Option<&str>, stall_timeout: Option<&str>) -> Self {
        let lock = ENV_LOCK.lock().unwrap();
        let backup = Self {
            _lock: lock,
            max_retries: env::var(HTTP_MAX_RETRIES).ok(),
            stall_timeout: env::var(HTTP_STALL_TIMEOUT).ok(),
        };
        match max_retries {
            | Some(value) => env::set_var(HTTP_MAX_RETRIES, value),
            | None => env::remove_var(HTTP_MAX_RETRIES),
        }
        match stall_timeout {
            | Some(value) => env::set_var(HTTP_STALL_TIMEOUT, value),
            | None => env::remove_var(HTTP_STALL_TIMEOUT),
        }
        backup
    }
}
impl Drop for RestoreEnv {
    fn drop(&mut self) {
        match &self.max_retries {
            | Some(value) => env::set_var(HTTP_MAX_RETRIES, value),
            | None => env::remove_var(HTTP_MAX_RETRIES),
        }
        match &self.stall_timeout {
            | Some(value) => env::set_var(HTTP_STALL_TIMEOUT, value),
            | None => env::remove_var(HTTP_STALL_TIMEOUT),
        }
    }
}

#[test]
fn test_shared_http_policy_defaults() {
    let _backup = RestoreEnv::apply(None, None);
    let policy = policy::shared_http_policy();
    assert_eq!(policy.timeout.as_secs(), 30);
    assert_eq!(policy.max_retries, 2);
    assert_eq!(policy.max_attempts(), 3);
}
#[test]
fn test_shared_http_policy_reads_env_overrides_with_min_stall_clamp() {
    let _backup = RestoreEnv::apply(Some("4"), Some("0"));
    let policy = policy::shared_http_policy();
    assert_eq!(policy.max_retries, 4);
    assert_eq!(policy.stall_timeout, SignedDuration::from_secs(1));
}
#[tokio::test]
async fn test_http_get_retries_anonymously_when_configured_credentials_are_rejected() {
    let requests = Arc::new(AtomicUsize::new(0));
    let state = Arc::clone(&requests);
    let router = Router::new().route(
        "/public",
        get(move |headers: HeaderMap| {
            let state = Arc::clone(&state);
            async move {
                state.fetch_add(1, Ordering::SeqCst);
                match headers.contains_key("private-token") {
                    | true => (StatusCode::UNAUTHORIZED, "bad credential"),
                    | false => (StatusCode::OK, "public content"),
                }
            }
        }),
    );
    let server = TestServer::start(router).await.unwrap();
    let response = super::get(format!("{}/public", server.base_url))
        .headers(super::headers([("PRIVATE-TOKEN", "invalid")]))
        .send()
        .await
        .expect("anonymous fallback should succeed");
    assert_eq!(response.status_code, StatusCode::OK.as_u16());
    assert_eq!(response.text().await.unwrap(), "public content");
    assert_eq!(requests.load(Ordering::SeqCst), 2);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_streaming_download_retries_anonymously_when_configured_credentials_are_rejected() {
    let requests = Arc::new(AtomicUsize::new(0));
    let state = Arc::clone(&requests);
    let router = Router::new().route(
        "/public",
        get(move |headers: HeaderMap| {
            let state = Arc::clone(&state);
            async move {
                state.fetch_add(1, Ordering::SeqCst);
                match headers.contains_key("private-token") {
                    | true => (StatusCode::FORBIDDEN, "bad credential"),
                    | false => (StatusCode::OK, "public content"),
                }
            }
        }),
    );
    let server = TestServer::start(router).await.unwrap();
    let output = std::env::temp_dir().join(format!("acorn-auth-fallback-{}.txt", std::process::id()));
    download_with_progress(
        &format!("{}/public", server.base_url),
        &output,
        |_, _| {},
        Some(super::headers([("PRIVATE-TOKEN", "invalid")])),
        None,
        None,
        None,
    )
    .await
    .expect("anonymous streaming fallback should succeed");
    assert_eq!(std::fs::read_to_string(&output).unwrap(), "public content");
    assert_eq!(requests.load(Ordering::SeqCst), 2);
    let _ = std::fs::remove_file(output);
    server.stop().await.unwrap();
}
#[tokio::test]
async fn test_streaming_download_reports_credential_and_anonymous_failures() {
    let requests = Arc::new(AtomicUsize::new(0));
    let state = Arc::clone(&requests);
    let router = Router::new().route(
        "/private",
        get(move || {
            let state = Arc::clone(&state);
            async move {
                state.fetch_add(1, Ordering::SeqCst);
                (StatusCode::UNAUTHORIZED, "authentication required")
            }
        }),
    );
    let server = TestServer::start(router).await.unwrap();
    let output = std::env::temp_dir().join(format!("acorn-auth-fallback-failure-{}.txt", std::process::id()));
    let why = download_with_progress(
        &format!("{}/private", server.base_url),
        &output,
        |_, _| {},
        Some(super::headers([("PRIVATE-TOKEN", "invalid")])),
        None,
        Some("configured credential rejected"),
        None,
    )
    .await
    .expect_err("private content should reject both attempts");
    let message = why.to_string();
    assert!(message.contains("configured credential rejected"));
    assert!(message.contains("anonymous fallback also failed"));
    assert!(!message.contains("invalid"));
    assert_eq!(requests.load(Ordering::SeqCst), 2);
    let _ = std::fs::remove_file(output);
    server.stop().await.unwrap();
}
#[tokio::test(flavor = "current_thread")]
async fn test_streaming_download_aborts_when_server_stalls_between_bytes() {
    let _backup = RestoreEnv::apply(Some("0"), Some("1"));
    let router = Router::new().route(
        "/stall",
        get(|| async {
            Body::from_stream(stream::once(async { Ok::<_, std::io::Error>(Bytes::from_static(b"incomplete")) }).chain(stream::pending()))
        }),
    );
    let server = TestServer::start(router).await.unwrap();
    let output = std::env::temp_dir().join("acorn-stall-download-test.bin");
    let error = download_with_progress(&format!("{}/stall", server.base_url), &output, |_, _| {}, None, None, None, None)
        .await
        .expect_err("stalled download should fail");
    assert!(error.to_string().contains("stalled for 1 seconds"));
    let _ = std::fs::remove_file(output);
    server.stop().await.unwrap();
}
