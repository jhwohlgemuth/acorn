#![allow(clippy::indexing_slicing)]

use super::*;
use alloc::sync::Arc as StdArc;
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use core::sync::atomic::{AtomicUsize, Ordering};
use data_encoding::BASE64;
use ring::hmac;
use std::env::temp_dir;
use std::sync::Mutex;

struct FakeCallbackResolver;
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct FakeEvent {
    value: u64,
}
struct FakeProvider {
    verifier: HeaderSecretVerifier,
}
#[derive(Default)]
struct RecordingService {
    calls: AtomicUsize,
    request: Mutex<Option<HttpRequest>>,
    error: bool,
}
impl CallbackResolver for FakeCallbackResolver {
    fn resolve(&self, name: &str) -> Result<CallbackDestination, WebhookError> {
        match name {
            | "audit" => CallbackTarget::new("https://example.test/callback?credential=secret")
                .and_then(|target| CallbackHeader::bearer("resolved-attempt-secret").map(|header| CallbackDestination::new(target, vec![header]))),
            | _ => Err(WebhookError::new(ErrorKind::InvalidTarget, "Unknown callback destination")),
        }
    }
}
impl WebhookProvider for FakeProvider {
    type Event = FakeEvent;

    fn receive(&self, request: InboundRequest<'_>, now: Timestamp) -> Result<Delivery<Self::Event>, WebhookError> {
        self.verifier.verify(request, now).and_then(|verified| {
            serde_json::from_slice(verified.raw_body)
                .map_err(|_| WebhookError::new(ErrorKind::BadRequest, "Invalid fake webhook payload"))
                .map(|event| Delivery {
                    delivery_id: verified.delivery_id.to_string(),
                    event,
                })
        })
    }
    fn classify(&self, delivery: &Delivery<Self::Event>) -> Result<Vec<OperationSpec>, WebhookError> {
        ["second", "first"]
            .into_iter()
            .map(|suffix| {
                crate::io::api::json_rpc::Notification::new(
                    crate::io::api::json_rpc::MethodName::from(["fake", "record"]),
                    serde_json::json!({"value": delivery.event.value}),
                )
                .map(|notification| OperationSpec::new("fake", format!("{suffix}:{}", delivery.event.value), notification))
                .map_err(|_| WebhookError::new(ErrorKind::BadRequest, "Invalid fake operation"))
            })
            .collect()
    }
    fn name(&self) -> &'static str {
        "fake"
    }
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
fn test_callback_headers_make_diagnostic_secrecy_explicit() {
    let public = CallbackHeader::public("x-event-kind", "created").unwrap();
    let secret = CallbackHeader::secret("x-callback-key", "custom-secret").unwrap();
    let signature = CallbackHeader::public("webhook-signature", "signature-secret").unwrap();
    assert!(format!("{public:?}").contains("created"));
    assert!(!format!("{secret:?}").contains("custom-secret"));
    assert!(!format!("{signature:?}").contains("signature-secret"));
}
#[test]
fn test_callback_target_requires_https_and_rejects_userinfo_or_fragments() {
    for url in [
        "http://example.test/hook",
        "https://user@example.test/hook",
        "https://example.test/hook#fragment",
    ] {
        assert_eq!(CallbackTarget::new(url).unwrap_err().kind(), ErrorKind::InvalidTarget);
    }
}
#[tokio::test]
async fn test_dispatch_discards_transport_details_that_may_contain_secrets() {
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
async fn test_dispatch_is_one_shot_and_applies_callback_safety_policy() {
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
fn test_effect_results_are_durable_typed_and_kind_safe() {
    let queue = store::OperationQueue::from(temp_dir().join(format!("acorn-webhook-effects-{}.db", nanoid::nanoid!())));
    queue.begin_effect("gitlab:30:issue:7:label", "gitlab-label").unwrap();
    assert_eq!(
        queue
            .effect_result::<serde_json::Value>("gitlab:30:issue:7:label", "gitlab-label")
            .unwrap(),
        None
    );
    let result = serde_json::json!({"label": "acorn::collected"});
    queue.succeed_effect("gitlab:30:issue:7:label", "gitlab-label", &result).unwrap();
    assert_eq!(
        queue
            .effect_result::<serde_json::Value>("gitlab:30:issue:7:label", "gitlab-label")
            .unwrap(),
        Some(result)
    );
    assert!(queue
        .begin_effect("gitlab:30:issue:7:label", "gitlab-note")
        .unwrap_err()
        .to_string()
        .contains("already assigned"));
}
#[tokio::test]
async fn test_outbound_callbacks_are_persisted_deduplicated_and_attempted_durably() {
    let queue = store::OperationQueue::from(temp_dir().join(format!("acorn-webhook-callbacks-{}.db", nanoid::nanoid!())));
    assert!(queue
        .enqueue_callback("callback-1", "audit", &serde_json::json!({"event": "complete"}))
        .unwrap());
    assert!(!queue
        .enqueue_callback("callback-1", "audit", &serde_json::json!({"event": "changed"}))
        .unwrap());
    let worker = store::CallbackWorker::new(
        queue.clone(),
        RecordingService::default(),
        FakeCallbackResolver,
        jiff::SignedDuration::from_mins(5),
    );
    assert!(worker.run_once().await.unwrap());
    assert_eq!(queue.callback_state("callback-1").unwrap(), Some(store::OperationState::Succeeded));
    assert_eq!(queue.callback_attempts("callback-1").unwrap(), 1);
    assert!(queue.runtime_timestamps().unwrap().last_success_at.is_some());
    assert!(!worker.run_once().await.unwrap());
    queue
        .with_connection(|connection| {
            connection
                .query_row(
                    "SELECT destination, payload_json FROM webhook_callbacks WHERE callback_key = ?",
                    crate::io::database::backend::params!["callback-1"],
                    |row| {
                        row.get::<_, String>(0)
                            .and_then(|destination| row.get::<_, String>(1).map(|payload| (destination, payload)))
                    },
                )
                .map_err(|why| eyre!(why))
                .map(|(destination, payload)| {
                    assert_eq!(destination, "audit");
                    assert!(!payload.contains("resolved-attempt-secret"));
                    assert!(!payload.contains("credential=secret"));
                })
        })
        .unwrap();
}
#[tokio::test]
async fn test_outbound_callback_failures_are_recorded_for_retry() {
    let queue = store::OperationQueue::from(temp_dir().join(format!("acorn-webhook-callback-retry-{}.db", nanoid::nanoid!())));
    queue
        .enqueue_callback("callback-retry", "audit", &serde_json::json!({"event": "complete"}))
        .unwrap();
    let worker = store::CallbackWorker::new(
        queue.clone(),
        RecordingService {
            error: true,
            ..RecordingService::default()
        },
        FakeCallbackResolver,
        jiff::SignedDuration::from_mins(5),
    );
    assert!(worker
        .run_once()
        .await
        .unwrap_err()
        .to_string()
        .contains("Webhook callback transport failed"));
    assert_eq!(
        queue.callback_state("callback-retry").unwrap(),
        Some(store::OperationState::RetryableFailure)
    );
    assert_eq!(queue.callback_attempts("callback-retry").unwrap(), 1);
    assert!(queue.runtime_timestamps().unwrap().last_error_at.is_some());
}
#[tokio::test]
async fn test_provider_pipeline_is_durable_sorted_and_dispatchable() {
    #[derive(Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields)]
    struct Input {
        value: u64,
    }
    #[derive(schemars::JsonSchema, Serialize)]
    #[serde(deny_unknown_fields)]
    struct Output {
        recorded: u64,
    }
    let provider = FakeProvider {
        verifier: HeaderSecretVerifier::from_string("x-token", "secret", &["webhook-id"]).unwrap(),
    };
    let mut headers = HeaderMap::new();
    headers.insert("x-token", HeaderValue::from_static("secret"));
    headers.insert("webhook-id", HeaderValue::from_static("delivery-1"));
    let prepared = prepare(
        &provider,
        InboundRequest {
            headers: &headers,
            raw_body: br#"{"value":7}"#,
        },
        Timestamp::now(),
    )
    .unwrap();
    assert_eq!(prepared.operations[0].idempotency_key, "first:7");
    assert_eq!(prepared.operations[1].idempotency_key, "second:7");
    let queue = store::OperationQueue::from(temp_dir().join(format!("acorn-webhooks-{}.db", nanoid::nanoid!())));
    assert_eq!(queue.enqueue_prepared(&prepared).unwrap(), vec![store::EnqueueStatus::Inserted; 2]);
    assert_eq!(
        queue.enqueue_prepared(&prepared).unwrap(),
        vec![store::EnqueueStatus::DuplicateDelivery; 2]
    );
    let calls = StdArc::new(AtomicUsize::new(0));
    let definition = crate::io::api::json_rpc::OperationDefinition::new::<Input, Output>(
        crate::io::api::json_rpc::MethodName::from(["fake", "record"]),
        crate::io::api::json_rpc::OperationEffects::default(),
    )
    .unwrap();
    let registry = crate::io::api::json_rpc::OperationRegistry::default()
        .register(definition, {
            let calls = StdArc::clone(&calls);
            move |input: Input, _| {
                let calls = StdArc::clone(&calls);
                async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(Output { recorded: input.value })
                }
            }
        })
        .unwrap();
    let worker = store::OperationWorker::new(
        queue.clone(),
        registry,
        crate::io::api::json_rpc::InvocationContext {
            origin: crate::io::api::json_rpc::InvocationOrigin::Webhook,
            ..crate::io::api::json_rpc::InvocationContext::default()
        },
        jiff::SignedDuration::from_mins(5),
    );
    assert!(worker.run_once().await.unwrap());
    assert!(worker.run_once().await.unwrap());
    assert!(!worker.run_once().await.unwrap());
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(queue.counts().unwrap().succeeded, 2);
    assert!(queue.runtime_timestamps().unwrap().last_success_at.is_some());
}
#[test]
fn test_runtime_callback_errors_persist_through_terminal_failure() {
    let queue = store::OperationQueue::from(temp_dir().join(format!("acorn-webhook-callback-terminal-{}.db", nanoid::nanoid!())));
    queue.enqueue_callback("callback", "audit", &serde_json::json!({})).unwrap();
    for attempt in 1..=crate::util::constants::app::MAX_WEBHOOK_OPERATION_ATTEMPTS {
        let claim = queue.claim_callback(jiff::SignedDuration::from_mins(5)).unwrap().unwrap();
        let state = queue.fail_callback(claim, "expected failure").unwrap();
        if attempt < crate::util::constants::app::MAX_WEBHOOK_OPERATION_ATTEMPTS {
            queue
                .with_connection(|connection| {
                    connection
                        .execute(
                            "UPDATE webhook_callbacks SET available_at = ? WHERE callback_key = ?",
                            crate::io::database::backend::params!["1970-01-01T00:00:00Z", "callback"],
                        )
                        .map(|_| ())
                        .map_err(|why| eyre!(why))
                })
                .unwrap();
            assert_eq!(state, store::OperationState::RetryableFailure);
        } else {
            assert_eq!(state, store::OperationState::TerminalFailure);
        }
    }
    assert!(queue.runtime_timestamps().unwrap().last_error_at.is_some());
}
#[tokio::test]
async fn test_runtime_drains_active_claims_within_grace() {
    let queue = store::OperationQueue::from(temp_dir().join(format!("acorn-webhook-drain-settle-{}.db", nanoid::nanoid!())));
    let runtime = WebhookRuntime::default();
    runtime.initialize(&queue, jiff::SignedDuration::from_mins(5)).unwrap();
    let (_, claim) = runtime.claim(|| Ok(Some(()))).unwrap().unwrap();
    tokio::spawn(async move {
        tokio::time::sleep(core::time::Duration::from_millis(20)).await;
        drop(claim);
    });
    let state = runtime.drain(core::time::Duration::from_secs(1)).await.unwrap();
    assert!(!state.accepting);
    assert_eq!(state.active_claims, 0);
    assert!(!state.ready);
}
#[tokio::test]
async fn test_runtime_drains_immediately_without_work() {
    let queue = store::OperationQueue::from(temp_dir().join(format!("acorn-webhook-drain-empty-{}.db", nanoid::nanoid!())));
    let runtime = WebhookRuntime::default();
    runtime.initialize(&queue, jiff::SignedDuration::from_mins(5)).unwrap();
    let state = runtime.drain(core::time::Duration::ZERO).await.unwrap();
    assert!(!state.accepting);
    assert_eq!(state.active_claims, 0);
    assert!(!state.ready);
}
#[test]
fn test_runtime_initialization_controls_readiness_and_reports_failure() {
    let queue = store::OperationQueue::from(temp_dir().join(format!("acorn-webhook-runtime-{}.db", nanoid::nanoid!())));
    let runtime = WebhookRuntime::default();
    assert!(!runtime.state().unwrap().ready);
    runtime.initialize(&queue, jiff::SignedDuration::from_mins(5)).unwrap();
    assert!(runtime.state().unwrap().ready);

    let invalid_queue = store::OperationQueue::from(temp_dir());
    let invalid_runtime = WebhookRuntime::default();
    assert!(invalid_runtime.initialize(&invalid_queue, jiff::SignedDuration::from_mins(5)).is_err());
    let state = invalid_runtime.state().unwrap();
    assert!(state.accepting);
    assert!(state.last_error_at.is_some());
    assert!(!state.ready);
}
#[test]
fn test_runtime_operation_errors_persist_through_terminal_failure() {
    let queue = store::OperationQueue::from(temp_dir().join(format!("acorn-webhook-terminal-{}.db", nanoid::nanoid!())));
    queue.enqueue("delivery", "operation", "{}").unwrap();
    let mut last_error_at = None;
    for attempt in 1..=crate::util::constants::app::MAX_WEBHOOK_OPERATION_ATTEMPTS {
        let claim = queue.claim_next(jiff::SignedDuration::from_mins(5)).unwrap().unwrap();
        let state = queue.fail(claim, "expected failure").unwrap();
        last_error_at = queue.runtime_timestamps().unwrap().last_error_at;
        if attempt < crate::util::constants::app::MAX_WEBHOOK_OPERATION_ATTEMPTS {
            queue
                .with_connection(|connection| {
                    connection
                        .execute(
                            "UPDATE webhook_operations SET available_at = ? WHERE operation_key = ?",
                            crate::io::database::backend::params!["1970-01-01T00:00:00Z", "operation"],
                        )
                        .map(|_| ())
                        .map_err(|why| eyre!(why))
                })
                .unwrap();
            assert_eq!(state, store::OperationState::RetryableFailure);
        } else {
            assert_eq!(state, store::OperationState::TerminalFailure);
        }
    }
    assert!(last_error_at.is_some());
}
#[tokio::test]
async fn test_runtime_timed_out_claim_remains_recoverable() {
    let queue = store::OperationQueue::from(temp_dir().join(format!("acorn-webhook-drain-timeout-{}.db", nanoid::nanoid!())));
    queue.enqueue("delivery", "operation", "{}").unwrap();
    let runtime = WebhookRuntime::default();
    runtime.initialize(&queue, jiff::SignedDuration::from_mins(5)).unwrap();
    let (operation, claim) = runtime.claim(|| queue.claim_next(jiff::SignedDuration::from_mins(5))).unwrap().unwrap();
    let state = runtime.drain(core::time::Duration::ZERO).await.unwrap();
    assert_eq!(state.active_claims, 1);
    assert_eq!(queue.state("operation").unwrap(), Some(store::OperationState::Running));
    drop(operation);
    drop(claim);
    queue.initialize(jiff::SignedDuration::from_secs(-1)).unwrap();
    assert_eq!(queue.state("operation").unwrap(), Some(store::OperationState::Queued));
}
#[test]
fn test_secret_verifier_uses_ordered_delivery_headers_and_redacts_secret() {
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
fn test_standard_webhooks_rejects_stale_and_future_timestamps() {
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
fn test_standard_webhooks_verifies_raw_bytes_and_rotated_signatures() {
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
fn test_verifiers_reject_empty_or_malformed_configured_secrets() {
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
