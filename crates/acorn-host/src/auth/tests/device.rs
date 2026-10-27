use super::super::{
    config::{GrantFlow, Persistence, Resource},
    device::DeviceAuthorization,
    entra::{Cloud, Profile},
    AuthError, TokenRequest,
};
use super::identity::{signed_token, PRIVATE_KEY};
use alloc::{collections::VecDeque, sync::Arc};
use chrono::Duration as ChronoDuration;
use core::{
    future::{ready, Ready},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use oauth2::{http, AsyncHttpClient, HttpRequest, HttpResponse};
use openidconnect::{
    core::{CoreJsonWebKeySet, CoreRsaPrivateSigningKey},
    IssuerUrl, PrivateSigningKey,
};
use secrecy::ExposeSecret;
use std::{io, sync::Mutex};
use tokio::sync::watch;

#[derive(Clone)]
struct FakeHttp {
    calls: Arc<AtomicUsize>,
    responses: Arc<Mutex<VecDeque<HttpResponse>>>,
}
impl<'a> AsyncHttpClient<'a> for FakeHttp {
    type Error = io::Error;
    type Future = Ready<Result<HttpResponse, io::Error>>;
    fn call(&'a self, _request: HttpRequest) -> Self::Future {
        self.calls.fetch_add(1, Ordering::SeqCst);
        ready(
            self.responses
                .lock()
                .expect("fake response lock")
                .pop_front()
                .ok_or_else(|| io::Error::other("no fake response")),
        )
    }
}
impl FakeHttp {
    fn new(responses: impl IntoIterator<Item = HttpResponse>) -> Self {
        Self {
            calls: Arc::new(AtomicUsize::new(0)),
            responses: Arc::new(Mutex::new(responses.into_iter().collect())),
        }
    }
    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}
fn challenge(verification_uri: &str) -> HttpResponse {
    response(
        200,
        &include_str!("../../../../../tests/fixtures/oauth2/device/challenge.json").replace("https://microsoft.com/devicelogin", verification_uri),
    )
}
fn profile_and_request() -> (Profile, TokenRequest) {
    let resource = Resource::try_new("https://graph.microsoft.com", ["Files.Read".to_string()]).expect("resource");
    let profile = Profile::try_new(
        Cloud::Public,
        "00000000-0000-0000-0000-000000000002".to_string(),
        GrantFlow::Device,
        Persistence::Disabled,
        "00000000-0000-0000-0000-000000000001".to_string(),
        vec![resource.clone()],
    )
    .expect("profile");
    (profile, TokenRequest { account: None, resource })
}
fn response(status: u16, body: &str) -> HttpResponse {
    http::Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(body.as_bytes().to_vec())
        .expect("valid fake response")
}
#[tokio::test]
async fn test_device_consent_boundary_and_private_code_redaction() {
    let (profile, request) = profile_and_request();
    let http = FakeHttp::new([challenge("https://microsoft.com/devicelogin")]);
    let device = DeviceAuthorization;
    assert_eq!(http.calls(), 0);
    let pending = device.begin_with(&profile, &request, &http).await.expect("device code");
    assert_eq!(http.calls(), 1);
    assert_eq!(pending.challenge().user_code, "ABCD-EFGH");
    assert_eq!(pending.challenge().verification_uri, "https://microsoft.com/devicelogin");
    assert!(!format!("{pending:?}").contains("private-device-code"));
}
#[tokio::test]
async fn test_device_denial_and_cancellation_stop_polling() {
    let (profile, request) = profile_and_request();
    let http = FakeHttp::new([
        challenge("https://microsoft.com/devicelogin"),
        response(400, include_str!("../../../../../tests/fixtures/oauth2/device/denied.json")),
    ]);
    let device = DeviceAuthorization;
    let pending = device.begin_with(&profile, &request, &http).await.expect("device code");
    let (_sender, mut cancellation) = watch::channel(false);
    let error = device.poll_token_with(&pending, &mut cancellation, &http).await.expect_err("denied");
    assert!(matches!(error, AuthError::Denied));
    assert!(!error.to_string().contains("private provider detail"));
    assert_eq!(http.calls(), 2);
    let http = FakeHttp::new([challenge("https://microsoft.com/devicelogin")]);
    let pending = device.begin_with(&profile, &request, &http).await.expect("device code");
    let (sender, mut cancellation) = watch::channel(false);
    sender.send(true).expect("cancel");
    assert!(matches!(
        device.poll_token_with(&pending, &mut cancellation, &http).await,
        Err(AuthError::Cancelled)
    ));
    assert_eq!(http.calls(), 1);
}
#[tokio::test]
async fn test_device_expired_and_provider_errors_are_typed_and_redacted() {
    let (profile, request) = profile_and_request();
    for (body, expired) in [
        (include_str!("../../../../../tests/fixtures/oauth2/device/expired.json"), true),
        (include_str!("../../../../../tests/fixtures/oauth2/device/provider-error.json"), false),
    ] {
        let http = FakeHttp::new([challenge("https://microsoft.com/devicelogin"), response(400, body)]);
        let device = DeviceAuthorization;
        let pending = device.begin_with(&profile, &request, &http).await.expect("device code");
        let (_sender, mut cancellation) = watch::channel(false);
        let error = device
            .poll_token_with(&pending, &mut cancellation, &http)
            .await
            .expect_err("provider failure");
        assert_eq!(matches!(error, AuthError::Expired), expired);
        assert!(!error.to_string().contains("private provider detail"));
    }
}
#[tokio::test]
async fn test_device_grant_returns_verified_account_and_resource_scopes() {
    let (profile, request) = profile_and_request();
    let issuer = IssuerUrl::new(profile.issuer().to_string()).expect("issuer");
    let (id_token, _) = signed_token(&issuer, profile.client_id(), ChronoDuration::minutes(5), None);
    let encoded = serde_json::to_value(&id_token).expect("signed token encoding");
    let body = serde_json::json!({
        "access_token": "synthetic-access-token",
        "token_type": "Bearer",
        "expires_in": 3600,
        "scope": "Files.Read openid",
        "id_token": encoded,
    })
    .to_string();
    let metadata = serde_json::json!({
        "issuer": issuer.as_str(),
        "authorization_endpoint": "https://login.microsoftonline.com/tenant/oauth2/v2.0/authorize",
        "jwks_uri": "https://login.microsoftonline.com/common/discovery/v2.0/keys",
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"]
    })
    .to_string();
    let signing = CoreRsaPrivateSigningKey::from_pem(PRIVATE_KEY, None).expect("synthetic signing key");
    let jwks = serde_json::to_string(&CoreJsonWebKeySet::new(vec![signing.as_verification_key()])).expect("JWKS");
    let http = FakeHttp::new([
        challenge("https://microsoft.com/devicelogin"),
        response(200, &body),
        response(200, &metadata),
        response(200, &jwks),
    ]);
    let device = DeviceAuthorization;
    let pending = device.begin_with(&profile, &request, &http).await.expect("device code");
    let (_sender, mut cancellation) = watch::channel(false);
    let grant = device.poll_with(pending, &mut cancellation, &http).await.expect("verified grant");
    assert_eq!(grant.account.issuer(), issuer.as_str());
    assert_eq!(grant.account.subject(), "opaque-subject");
    assert_eq!(grant.scopes, ["Files.Read"]);
    assert_eq!(grant.access_token.expose_secret(), "synthetic-access-token");
    assert!(!format!("{grant:?}").contains("synthetic-access-token"));
    assert_eq!(http.calls(), 4);
}
#[tokio::test]
async fn test_device_pending_respects_interval() {
    let (profile, request) = profile_and_request();
    let http = FakeHttp::new([
        challenge("https://microsoft.com/devicelogin"),
        response(400, include_str!("../../../../../tests/fixtures/oauth2/device/pending.json")),
    ]);
    let device = DeviceAuthorization;
    let pending = device.begin_with(&profile, &request, &http).await.expect("device code");
    let (_sender, mut cancellation) = watch::channel(false);
    let timed = tokio::time::timeout(Duration::from_millis(20), device.poll_token_with(&pending, &mut cancellation, &http)).await;
    assert!(timed.is_err());
    assert_eq!(http.calls(), 2);
}

#[tokio::test(start_paused = true)]
async fn test_device_rate_limit_backs_off_within_deadline() {
    let (profile, request) = profile_and_request();
    let http = FakeHttp::new([
        challenge("https://microsoft.com/devicelogin"),
        response(429, ""),
        response(400, include_str!("../../../../../tests/fixtures/oauth2/device/denied.json")),
    ]);
    let device = DeviceAuthorization;
    let pending = device.begin_with(&profile, &request, &http).await.expect("device code");
    let (_sender, mut cancellation) = watch::channel(false);
    let inspect = async {
        tokio::task::yield_now().await;
        assert_eq!(http.calls(), 2);
        tokio::time::advance(Duration::from_secs(1)).await;
        tokio::task::yield_now().await;
        assert_eq!(http.calls(), 2);
        tokio::time::advance(Duration::from_secs(1)).await;
    };
    let (result, ()) = tokio::join!(device.poll_token_with(&pending, &mut cancellation, &http), inspect);
    assert!(matches!(result, Err(AuthError::Denied)));
    assert_eq!(http.calls(), 3);
}

#[tokio::test]
async fn test_device_rejects_malicious_verification_url() {
    let (profile, request) = profile_and_request();
    for uri in [
        "http://microsoft.com/devicelogin",
        "https://user:pass@microsoft.com/devicelogin",
        "https://microsoft.com/devicelogin?token=secret",
        "https://evil.example/devicelogin",
    ] {
        let http = FakeHttp::new([challenge(uri)]);
        assert!(matches!(
            DeviceAuthorization.begin_with(&profile, &request, &http).await,
            Err(AuthError::Provider("verification URL"))
        ));
    }
}
#[tokio::test(start_paused = true)]
async fn test_device_slow_down_extends_poll_interval() {
    let (profile, request) = profile_and_request();
    let http = FakeHttp::new([
        challenge("https://microsoft.com/devicelogin"),
        response(400, include_str!("../../../../../tests/fixtures/oauth2/device/slow-down.json")),
        response(400, include_str!("../../../../../tests/fixtures/oauth2/device/denied.json")),
    ]);
    let device = DeviceAuthorization;
    let pending = device.begin_with(&profile, &request, &http).await.expect("device code");
    let (_sender, mut cancellation) = watch::channel(false);
    let inspect = async {
        tokio::task::yield_now().await;
        assert_eq!(http.calls(), 2);
        tokio::time::advance(Duration::from_secs(5)).await;
        tokio::task::yield_now().await;
        assert_eq!(http.calls(), 2);
        tokio::time::advance(Duration::from_secs(1)).await;
    };
    let (result, ()) = tokio::join!(device.poll_token_with(&pending, &mut cancellation, &http), inspect);
    assert!(matches!(result, Err(AuthError::Denied)));
    assert_eq!(http.calls(), 3);
}
