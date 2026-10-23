use super::super::chat::ChatDriver;
use acorn::agent::{
    ChatEvent, ChatRole, InMemoryBackend, InferenceBackend, InferenceCapabilities, InferenceFinishReason, InferenceFuture, InferencePermissionPolicy,
    InferenceProvenance, InferenceRequest, InferenceResult,
};
use acorn_core::{
    time::{Milliseconds, UnitOfTime},
    Location,
};
use alloc::sync::Arc;
use std::path::PathBuf;
use std::sync::Mutex;

struct FailureBackend;
struct PendingBackend;
struct RecordingBackend {
    requests: Arc<Mutex<Vec<InferenceRequest>>>,
}
impl InferenceBackend for FailureBackend {
    fn capabilities(&self) -> InferenceCapabilities {
        InferenceCapabilities::default()
    }

    fn infer<'a>(&'a self, _request: &'a InferenceRequest) -> InferenceFuture<'a> {
        Box::pin(async { Err(color_eyre::eyre::eyre!("transport unavailable")) })
    }
}
impl InferenceBackend for PendingBackend {
    fn capabilities(&self) -> InferenceCapabilities {
        InferenceCapabilities::default()
    }

    fn infer<'a>(&'a self, _request: &'a InferenceRequest) -> InferenceFuture<'a> {
        Box::pin(core::future::pending())
    }
}
impl InferenceBackend for RecordingBackend {
    fn capabilities(&self) -> InferenceCapabilities {
        InferenceCapabilities::default()
    }

    fn infer<'a>(&'a self, request: &'a InferenceRequest) -> InferenceFuture<'a> {
        self.requests.lock().expect("request lock").push(request.clone());
        Box::pin(async { Ok(result(InferenceFinishReason::Stop, "The CFF file has one finding.")) })
    }
}
fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}
fn result(reason: InferenceFinishReason, text: &str) -> InferenceResult {
    InferenceResult {
        finish_reason: reason,
        provenance: InferenceProvenance {
            backend: "memory".to_string(),
            agent: None,
            model: None,
            request_id: None,
        },
        structured: None,
        text: text.to_string(),
        usage: None,
    }
}

#[tokio::test]
async fn test_cancellation_does_not_record_an_assistant_turn() {
    let backend = InMemoryBackend::new(result(InferenceFinishReason::Stop, "Late reply"));
    let mut driver = ChatDriver::new(&backend, fixtures(), true, None, None, Milliseconds::new(3000));
    assert!(driver.reply_with_cancel("Check this", async {}).await.is_err());
    assert!(matches!(driver.state().events().last(), Some(ChatEvent::Cancelled)));
    assert!(!driver
        .state()
        .events()
        .iter()
        .any(|event| matches!(event, ChatEvent::Turn(turn) if turn.role == ChatRole::Assistant)));
}
#[tokio::test]
async fn test_multi_turn_reply_is_grounded_in_typed_findings_and_denies_mutation() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let backend = RecordingBackend { requests: requests.clone() };
    let mut driver = ChatDriver::new(&backend, fixtures(), true, None, None, Milliseconds::new(3000));
    driver.attach(Location::from("CITATION.cff"), None).await.expect("attach");
    driver.reply("What did the check find?").await.expect("first reply");
    driver.reply("Explain that again.").await.expect("second reply");
    let requests = requests.lock().expect("request lock");
    assert_eq!(requests.len(), 2);
    assert!(requests.first().expect("first request").prompt.body.contains("citation-file-format"));
    assert!(requests.get(1).expect("second request").prompt.body.contains("What did the check find?"));
    assert_eq!(
        requests.first().expect("first request").permission_policy,
        InferencePermissionPolicy::DenyMutation
    );
    assert!(driver
        .state()
        .events()
        .iter()
        .any(|event| matches!(event, ChatEvent::Turn(turn) if turn.role == ChatRole::Tool)));
}
#[tokio::test]
async fn test_refusal_cancel_and_missing_target_are_explicit() {
    let backend = InMemoryBackend::new(result(InferenceFinishReason::Refusal, "No"));
    let mut driver = ChatDriver::new(&backend, fixtures(), true, None, None, Milliseconds::new(3000));
    assert!(driver.reply("Help me").await.is_err());
    assert!(matches!(driver.state().events().last(), Some(ChatEvent::Failed(_))));
    assert!(driver.attach(Location::from("missing.json"), None).await.is_err());
    assert!(driver.state().target().is_none());
    driver.cancel();
    assert!(matches!(driver.state().events().last(), Some(ChatEvent::Cancelled)));
    driver.clear();
    assert!(driver.state().events().is_empty());
}
#[tokio::test]
async fn test_transport_failure_and_timeout_keep_the_session_usable() {
    let failure = FailureBackend;
    let mut failed = ChatDriver::new(&failure, fixtures(), true, None, None, Milliseconds::new(3000));
    assert!(failed.reply("What is this?").await.is_err());
    assert!(matches!(failed.state().events().last(), Some(ChatEvent::Failed(_))));
    let pending = PendingBackend;
    let mut timed_out = ChatDriver::new(&pending, fixtures(), true, None, None, Milliseconds::new(1));
    assert!(timed_out.reply("What is this?").await.is_err());
    assert!(matches!(timed_out.state().events().last(), Some(ChatEvent::Failed(message)) if message.contains("timed out")));
}
