use super::super::{
    config::{GrantFlow, Persistence, Resource},
    entra::{Cloud, Profile},
    identity::{verify_core_with, verify_issue_time, verify_token, BoundedHttp},
    AuthError,
};
use chrono::{Duration, Utc};
use oauth2::{http, AsyncHttpClient, HttpRequest};
use openidconnect::{
    core::{CoreIdToken, CoreIdTokenClaims, CoreIdTokenVerifier, CoreJsonWebKeySet, CoreJwsSigningAlgorithm, CoreRsaPrivateSigningKey},
    Audience, ClientId, IssuerUrl, Nonce, PrivateSigningKey, StandardClaims, SubjectIdentifier,
};
use std::io;

pub(super) const PRIVATE_KEY: &str = include_str!("../../../../../tests/fixtures/oauth2/identity/test-rsa.pem");

pub(super) fn signed_token(
    issuer: &IssuerUrl,
    client_id: &str,
    expires: Duration,
    nonce: Option<&str>,
) -> (CoreIdToken, CoreIdTokenVerifier<'static>) {
    let signing = CoreRsaPrivateSigningKey::from_pem(PRIVATE_KEY, None).expect("synthetic signing key");
    let jwks = CoreJsonWebKeySet::new(vec![signing.as_verification_key()]);
    let claims = CoreIdTokenClaims::new(
        issuer.clone(),
        vec![Audience::new(client_id.to_string())],
        Utc::now().checked_add_signed(expires).expect("test duration in range"),
        Utc::now(),
        StandardClaims::new(SubjectIdentifier::new("opaque-subject".to_string())),
        Default::default(),
    )
    .set_nonce(nonce.map(|value| Nonce::new(value.to_string())));
    let token = CoreIdToken::new(claims, &signing, CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256, None, None).expect("signed token");
    let verifier = CoreIdTokenVerifier::new_public_client(ClientId::new(client_id.to_string()), issuer.clone(), jwks);
    (token, verifier)
}

#[tokio::test]
async fn test_identity_discovery_uses_signed_jwks_and_rejects_unknown_key() {
    let client_id = "00000000-0000-0000-0000-000000000002";
    let resource = Resource::try_new("https://graph.microsoft.com", ["Files.Read".to_string()]).expect("resource");
    let profile = Profile::try_new(
        Cloud::Public,
        client_id.to_string(),
        GrantFlow::Device,
        Persistence::Disabled,
        "00000000-0000-0000-0000-000000000001".to_string(),
        vec![resource],
    )
    .expect("profile");
    let issuer = IssuerUrl::new(profile.issuer().to_string()).expect("issuer");
    let (token, _) = signed_token(&issuer, client_id, Duration::minutes(5), None);
    let signing = CoreRsaPrivateSigningKey::from_pem(PRIVATE_KEY, None).expect("synthetic signing key");
    let jwks = serde_json::to_vec(&CoreJsonWebKeySet::new(vec![signing.as_verification_key()])).expect("JWKS");
    let metadata = serde_json::json!({
        "issuer": issuer.as_str(),
        "authorization_endpoint": "https://login.microsoftonline.com/tenant/oauth2/v2.0/authorize",
        "jwks_uri": "https://login.microsoftonline.com/common/discovery/v2.0/keys",
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"]
    })
    .to_string()
    .into_bytes();
    let http = |request: HttpRequest| {
        let body = if request.uri().path().ends_with("/keys") {
            jwks.clone()
        } else {
            metadata.clone()
        };
        async move {
            Ok::<_, io::Error>(
                http::Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(body)
                    .expect("fake OIDC response"),
            )
        }
    };
    let account = verify_core_with(&profile, &token, None, &http).await.expect("verified discovery result");
    assert_eq!(account.subject(), "opaque-subject");
    let empty_jwks = serde_json::to_vec(&CoreJsonWebKeySet::new(vec![])).expect("empty JWKS");
    let no_key = |request: HttpRequest| {
        let body = if request.uri().path().ends_with("/keys") {
            empty_jwks.clone()
        } else {
            metadata.clone()
        };
        async move {
            Ok::<_, io::Error>(
                http::Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(body)
                    .expect("fake OIDC response"),
            )
        }
    };
    assert!(matches!(
        verify_core_with(&profile, &token, None, &no_key).await,
        Err(AuthError::InvalidIdentity)
    ));
}
#[tokio::test]
async fn test_identity_http_rejects_untrusted_metadata_destination() {
    let resource = Resource::try_new("https://graph.microsoft.com", ["Files.Read".to_string()]).expect("resource");
    let profile = Profile::try_new(
        Cloud::Public,
        "00000000-0000-0000-0000-000000000002".to_string(),
        GrantFlow::Device,
        Persistence::Disabled,
        "00000000-0000-0000-0000-000000000001".to_string(),
        vec![resource],
    )
    .expect("profile");
    let http = BoundedHttp::new(&profile).expect("bounded provider client");
    let request = http::Request::builder()
        .uri("https://evil.example/keys")
        .body(Vec::new())
        .expect("request");
    assert!(http.call(request).await.is_err());
}
#[test]
fn test_identity_rejects_future_issue_time() {
    assert!(verify_issue_time(Utc::now().checked_add_signed(Duration::minutes(10)).expect("test time in range")).is_err());
    assert!(verify_issue_time(Utc::now()).is_ok());
}
#[test]
fn test_signed_identity_accepts_device_without_nonce_and_browser_with_nonce() {
    let issuer = IssuerUrl::new("https://login.example.test/tenant/v2.0".to_string()).expect("issuer");
    let (token, verifier) = signed_token(&issuer, "client-one", Duration::minutes(5), None);
    let account = verify_token(&token, &verifier, None).expect("device identity");
    assert_eq!(account.subject(), "opaque-subject");
    assert_eq!(account.issuer(), issuer.as_str());
    let (token, verifier) = signed_token(&issuer, "client-one", Duration::minutes(5), Some("browser-nonce"));
    assert!(verify_token(&token, &verifier, Some("browser-nonce")).is_ok());
    assert!(matches!(
        verify_token(&token, &verifier, Some("wrong-nonce")),
        Err(AuthError::InvalidIdentity)
    ));
}
#[test]
fn test_signed_identity_rejects_wrong_issuer_audience_and_expiry() {
    let issuer = IssuerUrl::new("https://login.example.test/tenant/v2.0".to_string()).expect("issuer");
    let other = IssuerUrl::new("https://login.example.test/other/v2.0".to_string()).expect("issuer");
    let (token, verifier) = signed_token(&issuer, "client-one", Duration::minutes(5), None);
    let signing = CoreRsaPrivateSigningKey::from_pem(PRIVATE_KEY, None).expect("synthetic signing key");
    let jwks = CoreJsonWebKeySet::new(vec![signing.as_verification_key()]);
    let wrong_issuer = CoreIdTokenVerifier::new_public_client(ClientId::new("client-one".to_string()), other, jwks.clone());
    let wrong_audience = CoreIdTokenVerifier::new_public_client(ClientId::new("client-two".to_string()), issuer.clone(), jwks);
    assert!(matches!(verify_token(&token, &wrong_issuer, None), Err(AuthError::InvalidIdentity)));
    assert!(matches!(verify_token(&token, &wrong_audience, None), Err(AuthError::InvalidIdentity)));
    let (expired, _) = signed_token(&issuer, "client-one", Duration::seconds(-30), None);
    assert!(matches!(verify_token(&expired, &verifier, None), Err(AuthError::InvalidIdentity)));
}
