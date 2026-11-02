use super::*;
use crate::agent::{
    InferenceBackend, InferenceCapabilities, InferenceFinishReason, InferenceFuture, InferenceProvenance, InferenceRequest, InferenceResult,
};
use crate::io::config::ApplicationConfiguration;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use serde_json::json;
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
fn backend(finish_reason: InferenceFinishReason, text: &str) -> FakeBackend {
    FakeBackend {
        requests: Arc::new(Mutex::new(Vec::new())),
        result: InferenceResult {
            finish_reason,
            provenance: InferenceProvenance {
                agent: Some("codex".to_string()),
                backend: "acp".to_string(),
                model: Some("selected-model".to_string()),
                request_id: Some("session-1".to_string()),
            },
            structured: None,
            text: text.to_string(),
            usage: None,
        },
    }
}
fn configuration(profile: &str) -> ApplicationConfiguration {
    ApplicationConfiguration::parse(format!(
        r#"{{"executors":[{{"id":"acp:summary","capabilityId":"acorn.summary","executor":{{"type":"acp","profile":"{profile}","model":"selected-model"}}}}]}}"#
    ))
    .expect("ACP configuration")
}
fn input() -> serde_json::Value {
    json!({"asset":"summarize","data":{"text":"A short source"},"responseSchema":null})
}

#[tokio::test]
async fn test_acp_binding_cancellation_drops_inflight_session() {
    let dropped = Arc::new(AtomicBool::new(false));
    let backend = Arc::new(PendingBackend(dropped.clone()));
    let registry = ToolRegistry::default()
        .register_acp_bindings_with(&configuration("codex"), |_| Ok(backend.clone()))
        .expect("register ACP binding");
    let selected = registry
        .select_capability_binding("acorn.summary", None, None, CapabilitySelectionContext::default())
        .expect("select ACP binding");
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
async fn test_acp_binding_hydrates_prompt_and_preserves_selection() {
    let fake = backend(InferenceFinishReason::Stop, "Final summary");
    let requests = fake.requests.clone();
    let registry = ToolRegistry::default()
        .register_acp_bindings_with(&configuration("codex"), |_| Ok(Arc::new(fake.clone())))
        .expect("register ACP binding");
    assert!(requests.lock().expect("request log").is_empty());
    let selected = registry
        .select_capability_binding("acorn.summary", None, None, CapabilitySelectionContext::default())
        .expect("select ACP binding");
    let outcome = registry
        .execute_selected(
            &selected,
            input(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(outcome.outcome, ExecutionOutcome::Completed);
    assert_eq!(
        outcome.content.as_ref().and_then(|value| value.get("text")),
        Some(&json!("Final summary"))
    );
    assert_eq!(outcome.provenance.binding_id, "acp:summary");
    let requests = requests.lock().expect("request log");
    assert_eq!(requests.len(), 1);
    let request = requests.first().expect("one request");
    assert_eq!(request.agent.as_deref(), Some("codex"));
    assert_eq!(request.model.as_deref(), Some("selected-model"));
    assert!(request.prompt.body.contains("A short source"));
}
#[tokio::test]
async fn test_acp_binding_rejects_refusal_and_invalid_structured_output() {
    for (result, arguments) in [
        (backend(InferenceFinishReason::Refusal, "Denied"), input()),
        (
            backend(InferenceFinishReason::Stop, "not JSON"),
            json!({"asset":"summarize","data":{"text":"A short source"},"responseSchema":{"type":"object"}}),
        ),
    ] {
        let registry = ToolRegistry::default()
            .register_acp_bindings_with(&configuration("codex"), |_| Ok(Arc::new(result.clone())))
            .expect("register ACP binding");
        let selected = registry
            .select_capability_binding("acorn.summary", None, None, CapabilitySelectionContext::default())
            .expect("select ACP binding");
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
#[test]
fn test_acp_binding_rejects_untrusted_profile_and_offline_selection() {
    let fake = backend(InferenceFinishReason::Stop, "Final summary");
    let rejected = ToolRegistry::default().register_acp_bindings_with(&configuration("other"), |_| Ok(Arc::new(fake.clone())));
    assert!(rejected.is_err());
    let registry = ToolRegistry::default()
        .register_acp_bindings_with(&configuration("codex"), |_| Ok(Arc::new(fake.clone())))
        .expect("register ACP binding");
    assert!(registry
        .select_capability_binding(
            "acorn.summary",
            None,
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: false,
            },
        )
        .is_err());
}
#[tokio::test]
async fn test_acp_binding_uses_opencode_profile() {
    let fake = backend(InferenceFinishReason::Stop, "Final summary");
    let requests = fake.requests.clone();
    let registry = ToolRegistry::default()
        .register_acp_bindings_with(&configuration("opencode"), |_| Ok(Arc::new(fake.clone())))
        .expect("register OpenCode binding");
    let selected = registry
        .select_capability_binding("acorn.summary", None, None, CapabilitySelectionContext::default())
        .expect("select OpenCode binding");
    let outcome = registry
        .execute_selected(
            &selected,
            input(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(outcome.outcome, ExecutionOutcome::Completed);
    assert_eq!(
        requests.lock().expect("request log").first().and_then(|request| request.agent.as_deref()),
        Some("opencode")
    );
}
