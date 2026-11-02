use super::*;
use crate::agent::{
    InferenceBackend, InferenceCapabilities, InferenceFinishReason, InferenceFuture, InferenceProvenance, InferenceRequest, InferenceResult,
};
use crate::io::config::ApplicationConfiguration;
use crate::test::server::TestServer;
use alloc::sync::Arc;
use axum::{routing::post, Json, Router};
use core::sync::atomic::{AtomicBool, Ordering};
use serde_json::{json, Value};
use std::sync::Mutex;

struct DropFlag(Arc<AtomicBool>);
#[derive(Clone)]
struct FakeBackend {
    requests: Arc<Mutex<Vec<InferenceRequest>>>,
    result: InferenceResult,
}
struct PendingBackend(Arc<AtomicBool>);
impl Drop for DropFlag {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}
impl InferenceBackend for FakeBackend {
    fn capabilities(&self) -> InferenceCapabilities {
        InferenceCapabilities::default()
    }
    fn infer<'a>(&'a self, request: &'a InferenceRequest) -> InferenceFuture<'a> {
        self.requests.lock().expect("request log").push(request.clone());
        Box::pin(core::future::ready(Ok(self.result.clone())))
    }
}
impl InferenceBackend for PendingBackend {
    fn capabilities(&self) -> InferenceCapabilities {
        InferenceCapabilities::default()
    }
    fn infer<'a>(&'a self, _request: &'a InferenceRequest) -> InferenceFuture<'a> {
        let flag = self.0.clone();
        Box::pin(async move {
            let _drop = DropFlag(flag);
            core::future::pending().await
        })
    }
}
fn configuration(name: &str, domain: &str, scheme: &str, port: Option<u16>, policy: &str) -> ApplicationConfiguration {
    let document = json!({
        "endpoints": [{"name":name,"domain":domain,"scheme":scheme,"port":port,"resources":[]}],
        "executors": [{
            "id":"endpoint:summary",
            "capabilityId":"acorn.summary",
            "executor":{"type":"endpoint","endpoint":name,"model":"selected-model","data_policy":policy}
        }]
    });
    ApplicationConfiguration::parse(document.to_string()).expect("endpoint configuration")
}
fn fake(reason: InferenceFinishReason, text: &str) -> FakeBackend {
    FakeBackend {
        requests: Arc::new(Mutex::new(Vec::new())),
        result: InferenceResult {
            finish_reason: reason,
            provenance: InferenceProvenance {
                backend: "openai".to_string(),
                agent: None,
                model: Some("selected-model".to_string()),
                request_id: Some("request-1".to_string()),
            },
            structured: None,
            text: text.to_string(),
            usage: None,
        },
    }
}
fn input() -> Value {
    json!({"asset":"summarize","data":{"text":"A short source"},"responseSchema":null})
}

#[tokio::test]
async fn test_endpoint_binding_cancellation_drops_inflight_request() {
    let configuration = configuration("ollama", "127.0.0.1", "http", Some(11434), "local-only");
    let dropped = Arc::new(AtomicBool::new(false));
    let backend = Arc::new(PendingBackend(dropped.clone()));
    let registry = ToolRegistry::default()
        .register_endpoint_bindings_with(&configuration, |_, _, _| Ok(backend.clone()))
        .expect("register local endpoint");
    let selected = registry
        .select_capability_binding("acorn.summary", None, None, CapabilitySelectionContext::default())
        .expect("select local endpoint");
    let outcome = registry
        .execute_selected(
            &selected,
            input(),
            ExecutionContext {
                grounded: true,
                cancellation: Some(Box::pin(async { tokio::time::sleep(core::time::Duration::from_millis(20)).await })),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(outcome.outcome, ExecutionOutcome::Cancelled);
    assert!(dropped.load(Ordering::SeqCst));
}
#[tokio::test]
async fn test_endpoint_binding_enforces_remote_policy_and_model() {
    let configuration = configuration("remote", "example.org", "https", None, "allow-remote");
    let fake = fake(InferenceFinishReason::Stop, "Summary");
    let requests = fake.requests.clone();
    let registry = ToolRegistry::default()
        .register_endpoint_bindings_with(&configuration, |_, model, ollama| {
            assert_eq!(model, "selected-model");
            assert!(!ollama);
            Ok(Arc::new(fake.clone()))
        })
        .expect("register remote endpoint");
    assert!(requests.lock().expect("request log").is_empty());
    assert!(registry
        .select_capability_binding("acorn.summary", None, None, CapabilitySelectionContext::default())
        .is_err());
    let context = CapabilitySelectionContext {
        offline: false,
        allow_remote: true,
    };
    let selected = registry
        .select_capability_binding("acorn.summary", None, None, context)
        .expect("select allowed remote endpoint");
    let outcome = registry
        .execute_selected(
            &selected,
            input(),
            ExecutionContext {
                grounded: true,
                selection: context,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(outcome.outcome, ExecutionOutcome::Completed);
    let requests = requests.lock().expect("request log");
    assert_eq!(requests.first().and_then(|request| request.model.as_deref()), Some("selected-model"));
    assert!(registry
        .select_capability_binding(
            "acorn.summary",
            None,
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: true
            },
        )
        .is_err());
}
#[test]
fn test_endpoint_binding_preference_selects_configured_remote() {
    let document = json!({
        "endpoints": [
            {"name":"ollama","domain":"127.0.0.1","scheme":"http","port":11434,"resources":[]},
            {"name":"remote","domain":"example.org","scheme":"https","resources":[]}
        ],
        "executors": [
            {"id":"endpoint:local","capabilityId":"acorn.summary","executor":{"type":"endpoint","endpoint":"ollama","model":"local","data_policy":"local-only"}},
            {"id":"endpoint:remote","capabilityId":"acorn.summary","executor":{"type":"endpoint","endpoint":"remote","model":"remote","data_policy":"allow-remote"}}
        ]
    });
    let configuration = ApplicationConfiguration::parse(document.to_string()).expect("two endpoint bindings");
    let fake = fake(InferenceFinishReason::Stop, "Summary");
    let registry = ToolRegistry::default()
        .register_endpoint_bindings_with(&configuration, |_, _, _| Ok(Arc::new(fake.clone())))
        .expect("register endpoint bindings");
    let context = CapabilitySelectionContext {
        offline: false,
        allow_remote: true,
    };
    let selected = registry
        .select_capability_binding("acorn.summary", None, Some("endpoint:remote"), context)
        .expect("select preferred remote endpoint");
    assert_eq!(selected.binding.id, "endpoint:remote");
}
#[test]
fn test_endpoint_binding_rejects_local_only_remote_and_insecure_remote() {
    let fake = fake(InferenceFinishReason::Stop, "Summary");
    for (scheme, policy) in [("https", "local-only"), ("http", "allow-remote")] {
        let configuration = configuration("remote", "example.org", scheme, None, policy);
        let registry = ToolRegistry::default()
            .register_endpoint_bindings_with(&configuration, |_, _, _| Ok(Arc::new(fake.clone())))
            .expect("register unavailable endpoint");
        assert!(registry
            .select_capability_binding(
                "acorn.summary",
                Some("endpoint:summary"),
                None,
                CapabilitySelectionContext {
                    offline: false,
                    allow_remote: true
                },
            )
            .is_err());
    }
}
#[tokio::test]
async fn test_endpoint_binding_rejects_refusal_and_malformed_structured_output() {
    let configuration = configuration("ollama", "127.0.0.1", "http", Some(11434), "local-only");
    for (fake, arguments) in [
        (fake(InferenceFinishReason::Refusal, "Denied"), input()),
        (
            fake(InferenceFinishReason::Stop, "not JSON"),
            json!({"asset":"summarize","data":{"text":"A short source"},"responseSchema":{"type":"object"}}),
        ),
    ] {
        let registry = ToolRegistry::default()
            .register_endpoint_bindings_with(&configuration, |_, _, ollama| {
                assert!(ollama);
                Ok(Arc::new(fake.clone()))
            })
            .expect("register local endpoint");
        let selected = registry
            .select_capability_binding("acorn.summary", None, None, CapabilitySelectionContext::default())
            .expect("select local endpoint");
        let outcome = registry
            .execute_selected(
                &selected,
                arguments,
                ExecutionContext {
                    grounded: true,
                    ..ExecutionContext::default()
                },
            )
            .await;
        assert_eq!(outcome.outcome, ExecutionOutcome::ToolFailed);
    }
}
#[tokio::test]
async fn test_endpoint_binding_rejects_unsupported_inputs_before_provider_call() {
    let configuration = configuration("ollama", "127.0.0.1", "http", Some(11434), "local-only");
    let fake = fake(InferenceFinishReason::Stop, "Summary");
    let requests = fake.requests.clone();
    let registry = ToolRegistry::default()
        .register_endpoint_bindings_with(&configuration, |_, _, _| Ok(Arc::new(fake.clone())))
        .expect("register local endpoint");
    let selected = registry
        .select_capability_binding("acorn.summary", None, None, CapabilitySelectionContext::default())
        .expect("select local endpoint");
    for (arguments, expected) in [
        (
            json!({"asset":"summarize","data":{"text":"source"},"responseSchema":{"type":"not-a-type"}}),
            ExecutionOutcome::ToolFailed,
        ),
        (
            json!({"asset":"summarize","data":{"text":"source"},"responseSchema":null,"attachments":["file"]}),
            ExecutionOutcome::InvalidInput,
        ),
    ] {
        let outcome = registry
            .execute_selected(
                &selected,
                arguments,
                ExecutionContext {
                    grounded: true,
                    ..ExecutionContext::default()
                },
            )
            .await;
        assert_eq!(outcome.outcome, expected);
    }
    assert!(requests.lock().expect("request log").is_empty());
}
#[tokio::test]
async fn test_local_ollama_binding_uses_existing_backend_while_offline() {
    let router = Router::new().route(
        "/v1/chat/completions",
        post(|| async {
            Json(json!({
                "id":"local-1",
                "model":"selected-model",
                "choices":[{"finish_reason":"stop","message":{"content":"Local summary","refusal":null}}]
            }))
        }),
    );
    let server = TestServer::start(router).await.expect("start local endpoint");
    let url = reqwest::Url::parse(&server.base_url).expect("parse test URL");
    let configuration = configuration("ollama", "127.0.0.1", "http", url.port(), "local-only");
    let registry = ToolRegistry::default()
        .register_endpoint_bindings(&configuration, true)
        .expect("register local Ollama binding");
    let policy = CapabilitySelectionContext {
        offline: true,
        allow_remote: false,
    };
    let selected = registry
        .select_capability_binding("acorn.summary", None, None, policy)
        .expect("select offline-safe local binding");
    let outcome = registry
        .execute_selected(
            &selected,
            input(),
            ExecutionContext {
                grounded: true,
                selection: policy,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(outcome.outcome, ExecutionOutcome::Completed);
    assert_eq!(
        outcome.content.as_ref().and_then(|value| value.get("text")),
        Some(&json!("Local summary"))
    );
    assert_eq!(
        outcome.content.as_ref().and_then(|value| value.pointer("/provenance/backend")),
        Some(&json!("ollama"))
    );
    server.stop().await.expect("stop local endpoint");
}
