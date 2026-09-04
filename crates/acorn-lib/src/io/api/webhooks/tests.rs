use super::*;
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use core::sync::atomic::{AtomicUsize, Ordering};
use data_encoding::BASE64;
use ring::hmac;
use std::sync::Mutex;

#[derive(Default)]
struct RecordingService {
    calls: AtomicUsize,
    request: Mutex<Option<HttpRequest>>,
    error: bool,
}
#[async_trait]
impl HttpService for RecordingService {
    async fn execute(&self, request: HttpRequest) -> crate::io::ApiResult<crate::io::http::HttpResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        *self.request.lock().unwrap() = Some(request);
        match self.error {
            | true => Err(eyre!("failure containing https://example.test/secret?sig=leak")),
            | false => Ok(crate::io::http::HttpResponse {
                body: Vec::new(),
                headers: HeaderMap::new(),
                status_code: 202,
            }),
        }
    }
}

#[test]
fn callback_headers_make_diagnostic_secrecy_explicit() {
    let public = CallbackHeader::public("x-event-kind", "created").unwrap();
    let secret = CallbackHeader::secret("x-callback-key", "custom-secret").unwrap();
    let signature = CallbackHeader::public("webhook-signature", "signature-secret").unwrap();
    assert!(format!("{public:?}").contains("created"));
    assert!(!format!("{secret:?}").contains("custom-secret"));
    assert!(!format!("{signature:?}").contains("signature-secret"));
}
#[test]
fn callback_target_requires_https_and_rejects_userinfo_or_fragments() {
    for url in [
        "http://example.test/hook",
        "https://user@example.test/hook",
        "https://example.test/hook#fragment",
    ] {
        assert_eq!(CallbackTarget::new(url).unwrap_err().kind(), ErrorKind::InvalidTarget);
    }
}
#[tokio::test]
async fn dispatch_discards_transport_details_that_may_contain_secrets() {
    let service = RecordingService {
        error: true,
        ..RecordingService::default()
    };
    let target = CallbackTarget::new("https://example.test/hooks/secret?sig=value").unwrap();
    let error = dispatch(&service, &target, &serde_json::json!({}), &[]).await.unwrap_err();
    let message = error.to_string();
    assert_eq!(message, "Webhook callback transport failed");
    assert!(!message.contains("secret"));
    assert!(!message.contains("sig"));
}
#[tokio::test]
async fn dispatch_is_one_shot_and_applies_callback_safety_policy() {
    let service = RecordingService::default();
    let target = CallbackTarget::new("https://example.test/hooks/secret?sig=value").unwrap();
    let header = CallbackHeader::bearer("bearer-secret").unwrap();
    let result = dispatch(&service, &target, &serde_json::json!({"ok": true}), &[header]).await.unwrap();
    assert_eq!(result.status_code, 202);
    assert_eq!(service.calls.load(Ordering::SeqCst), 1);
    let request = service.request.lock().unwrap();
    let request = request.as_ref().unwrap();
    assert_eq!(request.url, "https://example.test/hooks/secret?sig=value");
    assert_eq!(request.json_body, Some(serde_json::json!({"ok": true})));
    assert!(!request.allow_anonymous_fallback);
    assert!(!request.follow_redirects);
    assert!(request.sensitive_url);
    assert_eq!(request.max_response_bytes, Some(MAX_CALLBACK_RESPONSE_BYTES));
    assert!(request.headers.get(AUTHORIZATION).is_some_and(HeaderValue::is_sensitive));
    assert!(!format!("{request:?}").contains("bearer-secret"));
    assert!(!format!("{target:?}").contains("secret"));
}
#[test]
fn secret_verifier_uses_ordered_delivery_headers_and_redacts_secret() {
    let verifier = HeaderSecretVerifier::new("x-provider-token", "top-secret", &["webhook-id", "idempotency-key"]).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("x-provider-token", HeaderValue::from_static("top-secret"));
    headers.insert("idempotency-key", HeaderValue::from_static("fallback-id"));
    let verified = verifier
        .verify(
            InboundRequest {
                headers: &headers,
                raw_body: b"payload",
            },
            Timestamp::now(),
        )
        .unwrap();
    assert_eq!(verified.delivery_id, "fallback-id");
    assert!(!format!("{verifier:?}").contains("top-secret"));
}
#[test]
fn standard_webhooks_rejects_stale_and_future_timestamps() {
    let verifier = StandardWebhooksVerifier::new(format!("whsec_{}", BASE64.encode(b"key")), 300).unwrap();
    for timestamp in ["1699999699", "1700000301"] {
        let mut headers = HeaderMap::new();
        headers.insert("webhook-id", HeaderValue::from_static("delivery"));
        headers.insert("webhook-timestamp", HeaderValue::from_str(timestamp).unwrap());
        headers.insert("webhook-signature", HeaderValue::from_static("v1,bad"));
        let error = verifier
            .verify(
                InboundRequest {
                    headers: &headers,
                    raw_body: b"{}",
                },
                Timestamp::from_second(1_700_000_000).unwrap(),
            )
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unauthorized);
    }
}
#[test]
fn standard_webhooks_verifies_raw_bytes_and_rotated_signatures() {
    let secret = b"signing-key";
    let encoded_secret = format!("whsec_{}", BASE64.encode(secret));
    let verifier = StandardWebhooksVerifier::new(encoded_secret.clone(), 300).unwrap();
    assert!(!format!("{verifier:?}").contains(&encoded_secret));
    let now = Timestamp::from_second(1_700_000_000).unwrap();
    let body = br#"{"event":"created"}"#;
    let message = [b"delivery-1.1700000000.".as_slice(), body].concat();
    let valid = BASE64.encode(hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, secret), &message).as_ref());
    let mut headers = HeaderMap::new();
    headers.insert("webhook-id", HeaderValue::from_static("delivery-1"));
    headers.insert("webhook-timestamp", HeaderValue::from_static("1700000000"));
    headers.insert("webhook-signature", HeaderValue::from_str(&format!("v1,bad v1,{valid}")).unwrap());
    let verified = verifier
        .verify(
            InboundRequest {
                headers: &headers,
                raw_body: body,
            },
            now,
        )
        .unwrap();
    assert_eq!(verified.delivery_id, "delivery-1");
    assert_eq!(verified.raw_body, body);
}
#[test]
fn verifiers_reject_empty_or_malformed_configured_secrets() {
    assert_eq!(
        StandardWebhooksVerifier::new("not-a-secret", 300).unwrap_err().kind(),
        ErrorKind::Unauthorized
    );
    assert_eq!(StandardWebhooksVerifier::new("whsec_", 300).unwrap_err().kind(), ErrorKind::Unauthorized);
    assert_eq!(
        HeaderSecretVerifier::new("x-token", "", &["webhook-id"]).unwrap_err().kind(),
        ErrorKind::Unauthorized
    );
}
