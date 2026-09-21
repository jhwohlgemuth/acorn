use crate::http::{redact_url, AllowedAuthorities, HttpClient, HttpError, HttpPolicy, ProbeMethod};
use axum::{
    http::{header::LOCATION, StatusCode},
    routing::{get, head},
    Router,
};
use core::time::Duration;
use url::Url;

async fn serve(app: Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind test server");
    let address = listener.local_addr().expect("test server address");
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve test application");
    });
    (format!("http://{address}"), task)
}

#[test]
fn authority_matching_is_exact_and_port_aware() {
    let allowed = AllowedAuthorities::parse(["example.org", "api.example.org:8443", "[::1]:8080"]).expect("allow list");
    assert!(allowed.allows(&Url::parse("https://example.org/path").expect("URL")));
    assert!(allowed.allows(&Url::parse("https://api.example.org:8443/path").expect("URL")));
    assert!(allowed.allows(&Url::parse("http://[::1]:8080/path").expect("URL")));
    assert!(!allowed.allows(&Url::parse("https://example.org.evil.test/path").expect("URL")));
    assert!(!allowed.allows(&Url::parse("https://api.example.org/path").expect("URL")));
}
#[test]
fn diagnostics_remove_secrets() {
    let value = redact_url("https://user:secret@example.org/path?token=secret#fragment");
    assert_eq!(value, "https://example.org/path");
    assert!(!value.contains("secret"));
    assert!(!value.contains("token"));
}
#[test]
fn explicit_default_port_matches_an_implicit_url_port() {
    let allowed = AllowedAuthorities::parse(["example.org:443"]).expect("allow list");
    assert!(allowed.allows(&Url::parse("https://example.org/path").expect("URL")));
}
#[tokio::test]
async fn probe_falls_back_to_a_bounded_get() {
    let app = Router::new().route("/probe", head(|| async { StatusCode::METHOD_NOT_ALLOWED }).get(|| async { "available" }));
    let (base, task) = serve(app).await;
    let authority = base.trim_start_matches("http://");
    let allowed = AllowedAuthorities::parse([authority]).expect("allow list");
    let client = HttpClient::new(HttpPolicy::new().with_allowed_authorities(allowed)).expect("HTTP client");
    let probe = client.probe(&format!("{base}/probe")).await.expect("probe");
    assert_eq!(probe.method, ProbeMethod::Get);
    task.abort();
}
#[tokio::test]
async fn redirect_targets_are_revalidated() {
    let app = Router::new().route(
        "/redirect",
        get(|| async { (StatusCode::FOUND, [(LOCATION, "http://localhost.evil.invalid/secret?token=value")]) }),
    );
    let (base, task) = serve(app).await;
    let authority = base.trim_start_matches("http://");
    let allowed = AllowedAuthorities::parse([authority]).expect("allow list");
    let client = HttpClient::new(HttpPolicy::new().with_allowed_authorities(allowed)).expect("HTTP client");
    let error = client
        .get_bytes(&format!("{base}/redirect"))
        .await
        .expect_err("redirect must be rejected");
    assert!(matches!(
        error,
        HttpError::Request {
            kind: "redirect rejected",
            ..
        }
    ));
    assert!(!error.to_string().contains("token=value"));
    task.abort();
}
#[tokio::test]
async fn request_timeout_is_enforced() {
    let app = Router::new().route(
        "/slow",
        get(|| async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            "late"
        }),
    );
    let (base, task) = serve(app).await;
    let client = HttpClient::new(HttpPolicy::new().with_timeout(Duration::from_millis(20))).expect("HTTP client");
    let result = client.get_bytes(&format!("{base}/slow")).await;
    assert!(matches!(
        result,
        Err(HttpError::Request {
            kind: "request timed out",
            ..
        })
    ));
    task.abort();
}
#[tokio::test]
async fn response_limit_is_enforced() {
    let (base, task) = serve(Router::new().route("/large", get(|| async { "too large" }))).await;
    let client = HttpClient::new(HttpPolicy::new().with_max_response_bytes(3)).expect("HTTP client");
    let result = client.get_bytes(&format!("{base}/large")).await;
    assert!(matches!(result, Err(HttpError::ResponseTooLarge { .. })));
    task.abort();
}
