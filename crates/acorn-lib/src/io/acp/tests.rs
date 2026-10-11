use super::{AcpSessionEvent, AgentProcessConfig, AgentProfile, Client, Codex, Error, OpenCode, SessionObserver};
use crate::agent::{HydratedPrompt, InferencePermissionPolicy, InferenceRequest};
use crate::analyzer::fix::{CheckReference, CheckReferences, FixOptions, IntoFixes};
use acorn_cmd::{find_executable, CommandSpec};
use acorn_core::Noul;
use acorn_schema::{
    agent::{FrontMatter, PromptFileAsset},
    software::Software,
};
use alloc::sync::Arc;
use core::time::Duration;
use serde_json::json;
use std::{ffi::OsStr, path::PathBuf, sync::Mutex};
use tempfile::TempDir;

struct FakeAgent {
    _directory: TempDir,
    executable: PathBuf,
}
#[derive(Default)]
struct Observer {
    events: Mutex<Vec<AcpSessionEvent>>,
}
impl FakeAgent {
    fn compile() -> Self {
        let directory = tempfile::tempdir().expect("create fake agent directory");
        let source = directory.path().join("fake_agent.rs");
        let executable = directory.path().join(format!("fake-agent{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(
            &source,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/acp/fake_agent.rs")),
        )
        .expect("write fake agent source");
        let rustc = find_executable("rustc").expect("resolve rustc");
        let output = CommandSpec::new(rustc)
            .args([source.as_os_str(), OsStr::new("-o"), executable.as_os_str()])
            .output()
            .expect("compile fake ACP agent");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        Self {
            _directory: directory,
            executable,
        }
    }
    fn config(&self, agent_id: &str) -> AgentProcessConfig {
        AgentProcessConfig::new(agent_id, self.executable.clone(), self._directory.path())
            .with_acp_arguments(["acp"])
            .with_timeout(Duration::from_secs(10))
    }
}
impl SessionObserver for Observer {
    fn observe(&self, event: &AcpSessionEvent) {
        self.events.lock().expect("lock observer").push(event.clone());
    }
}
fn fix_reference() -> CheckReference {
    CheckReference {
        category: "schema".to_string(),
        context: Some("Old title".to_string()),
        fingerprint: "a".repeat(64),
        id: "check-0001".to_string(),
        locator: Some("title".to_string()),
        message: "Title is invalid".to_string(),
        rule_id: None,
        severity: "error".to_string(),
        standard: "research-activity-data".to_string(),
        uri: Some("activity.json".to_string()),
        vocabulary_advisory: None,
    }
}
fn request(body: &str) -> InferenceRequest {
    InferenceRequest {
        agent: Some("fake".to_string()),
        model: Some("fake-model".to_string()),
        permission_policy: InferencePermissionPolicy::DenyMutation,
        prompt: HydratedPrompt {
            asset: PromptFileAsset::Summarize,
            body: body.to_string(),
            metadata: FrontMatter::init().name("test".to_string()).build(),
        },
        response_schema: None,
        timeout: 10_000.into(),
    }
}
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("build test runtime")
}

#[test]
fn test_acp_inference_forwards_hydrated_zon_prompt_body() {
    let fake = FakeAgent::compile();
    let client = Client::new(fake.config("fake"));
    let body = "Instructions\n\n# Input Data (ZON)\n\n```zon\ntext:ZON-PASSTHROUGH\n```";
    let result = runtime()
        .block_on(client.infer_checked(&request(body)))
        .expect("forward hydrated ZON prompt");
    assert_eq!(result.text, "zon prompt received");
}
#[test]
fn test_acp_inference_rejects_offline_and_wrong_agent_without_starting() {
    let config = AgentProcessConfig::new("fake", "missing-agent", std::env::current_dir().expect("current directory"));
    let offline = runtime().block_on(Client::new(config.clone()).with_offline(true).infer_checked(&request("prompt")));
    assert!(matches!(offline, Err(Error::Offline { .. })));
    let mut mismatched = request("prompt");
    mismatched.agent = Some("different".to_string());
    let mismatch = runtime().block_on(Client::new(config).infer_checked(&mismatched));
    assert!(matches!(mismatch, Err(Error::AgentMismatch { .. })));
}
#[test]
fn test_acp_inference_rejects_unadvertised_model() {
    let fake = FakeAgent::compile();
    let mut invalid = request("prompt");
    invalid.model = Some("invented-model".to_string());
    let result = runtime().block_on(Client::new(fake.config("fake")).infer_checked(&invalid));
    assert!(matches!(result, Err(Error::ModelUnavailable { .. })));
}
#[test]
fn test_acp_inference_reports_early_child_exit() {
    let fake = FakeAgent::compile();
    let result = runtime().block_on(Client::new(fake.config("fake")).infer_checked(&request("early-exit")));
    assert!(matches!(&result, Err(Error::ChildExit { .. })), "{result:?}");
}
#[test]
fn test_acp_inference_streams_text_and_structured_output() {
    let fake = FakeAgent::compile();
    let client = Client::new(fake.config("fake"));
    let text = runtime().block_on(client.infer_checked(&request("plain"))).expect("stream text");
    assert_eq!(text.text, "fake response");
    assert_eq!(text.provenance.agent.as_deref(), Some("fake"));
    assert_eq!(text.provenance.request_id.as_deref(), Some("fake-session"));
    let mut structured_request = request("structured");
    structured_request.response_schema = Some(json!({"type": "object"}));
    let structured = runtime()
        .block_on(client.infer_checked(&structured_request))
        .expect("stream structured output");
    assert_eq!(structured.structured, Some(json!({"answer": 42})));
}
#[test]
fn test_acp_inference_times_out() {
    let fake = FakeAgent::compile();
    let client = Client::new(fake.config("fake"));
    let mut timed = request("hang");
    timed.timeout = 50.into();
    let result = runtime().block_on(client.infer_checked(&timed));
    assert!(matches!(result, Err(Error::Timeout { .. })));
}
#[test]
fn test_acp_observer_receives_text_and_cannot_change_permissions() {
    let fake = FakeAgent::compile();
    let observer = Arc::new(Observer::default());
    let client = Client::new(fake.config("fake")).with_observer(observer.clone() as Arc<dyn SessionObserver>);
    let text = runtime().block_on(client.infer_checked(&request("plain"))).expect("stream text");
    assert_eq!(text.text, "fake response");
    assert!(observer
        .events
        .lock()
        .expect("lock observer")
        .iter()
        .any(|event| matches!(event, AcpSessionEvent::TextDelta { .. })));

    let denied = runtime().block_on(client.infer_checked(&request("permission")));
    assert!(matches!(denied, Err(Error::PermissionDenied { .. })));
    assert!(observer
        .events
        .lock()
        .expect("lock observer")
        .iter()
        .any(|event| matches!(event, AcpSessionEvent::PermissionRequested { option_count } if *option_count > 0)));
}
#[test]
fn test_acp_permission_requests_are_denied() {
    let fake = FakeAgent::compile();
    let result = runtime().block_on(Client::new(fake.config("fake")).infer_checked(&request("permission")));
    assert!(matches!(result, Err(Error::PermissionDenied { .. })));
}
#[test]
fn test_acp_protocol_readiness_reports_negotiated_capabilities() {
    let fake = FakeAgent::compile();
    let readiness = runtime()
        .block_on(Client::new(fake.config("fake")).protocol_readiness())
        .expect("protocol readiness");
    assert_eq!(readiness.agent_name.as_deref(), Some("fake-acp"));
    assert_eq!(readiness.agent_version.as_deref(), Some("1.0.0"));
    assert!(readiness.capabilities.attachments);
    assert!(readiness.capabilities.streaming);
    assert!(readiness.capabilities.tools);
    assert_eq!(readiness.protocol_version, 1);
}
#[test]
fn test_acp_suggestion_service_returns_validated_fix_report() {
    let fake = FakeAgent::compile();
    let report = runtime()
        .block_on(vec![fix_reference()].suggest(
            &Client::new(fake.config("fake")),
            &FixOptions {
                agent: Some("fake".to_string()),
                min_confidence: Noul::new(0.5).expect("valid unit value"),
                model: Some("fake-model".to_string()),
                ..FixOptions::default()
            },
        ))
        .expect("suggest fixes through ACP");
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    let fixes = report.fixes();
    let fix = fixes.first().expect("one ACP fix proposal");
    assert_eq!(fix.id, "fix-0001");
    assert_eq!(fix.provenance.backend, "acp");
}
#[test]
fn test_agent_process_config_profiles_use_thin_command_defaults() {
    let working_directory = std::env::current_dir().expect("current directory");
    let codex = AgentProcessConfig::codex(None, working_directory.clone());
    assert_eq!(codex, Codex::process_config(None, working_directory.clone()));
    assert_eq!(codex.agent_id(), Software::Codex.command());
    assert_eq!(codex.executable(), &PathBuf::from("codex-acp"));
    assert!(codex.acp_arguments().is_empty());
    assert_eq!(
        codex.environment().get(OsStr::new("INITIAL_AGENT_MODE")).map(|value| value.as_os_str()),
        Some(OsStr::new("read-only"))
    );
    assert_eq!(codex.permission_policy(), InferencePermissionPolicy::DenyMutation);
    assert_eq!(codex.version_arguments(), ["--version"]);
    assert_eq!(codex.working_directory(), &working_directory);
    let opencode = AgentProcessConfig::opencode(None, working_directory.clone());
    assert_eq!(opencode, OpenCode::process_config(None, working_directory.clone()));
    assert_eq!(opencode.agent_id(), Software::OpenCode.command());
    assert_eq!(opencode.executable(), &PathBuf::from("opencode"));
    assert_eq!(opencode.acp_arguments(), ["acp"]);
    assert!(opencode.environment().is_empty());
    assert_eq!(opencode.permission_policy(), InferencePermissionPolicy::DenyMutation);
    assert_eq!(opencode.version_arguments(), ["--version"]);
    assert_eq!(opencode.working_directory(), &working_directory);
    let override_path = PathBuf::from("custom/codex-acp");
    assert_eq!(
        AgentProcessConfig::codex(Some(override_path.clone()), working_directory).executable(),
        &override_path
    );
}
#[test]
fn test_installation_preflight_distinguishes_failure_modes_and_agents() {
    let fake = FakeAgent::compile();
    let first = fake.config("first").check_installation().expect("first installation");
    let second = fake.config("second").check_installation().expect("second installation");
    assert_eq!(first.agent_id, "first");
    assert_eq!(second.agent_id, "second");
    assert_eq!(first.version.map(|version| version.to_string()).as_deref(), Some("1.2.3"));
    let missing = AgentProcessConfig::new("missing", fake._directory.path().join("absent"), fake._directory.path());
    assert!(matches!(missing.check_installation(), Err(Error::ExecutableNotFound { .. })));
    let failing = fake.config("failing").with_version_arguments(["fail-version"]);
    assert!(matches!(failing.check_installation(), Err(Error::VersionProbe { .. })));
    let slow = fake
        .config("slow")
        .with_timeout(Duration::from_millis(50))
        .with_version_arguments(["slow-version"]);
    assert!(matches!(slow.check_installation(), Err(Error::VersionProbeTimeout { .. })));
}
#[test]
fn test_installation_preflight_explains_how_to_install_missing_codex_acp() {
    let directory = tempfile::tempdir().expect("create fixture directory");
    let missing = directory.path().join("missing-codex-acp");
    let error = AgentProcessConfig::codex(Some(missing), directory.path().to_path_buf())
        .check_installation()
        .expect_err("missing Codex ACP executable should fail");
    let message = error.to_string();
    assert!(message.contains("npm install -g @agentclientprotocol/codex-acp"));
    assert!(message.contains("does not require `npx`"));
    assert!(message.contains("https://github.com/agentclientprotocol/codex-acp/releases"));
}
#[cfg(unix)]
#[test]
fn test_installation_preflight_rejects_non_executable_file() {
    let directory = tempfile::tempdir().expect("create fixture directory");
    let executable = directory.path().join("not-executable");
    std::fs::write(&executable, "fixture").expect("write fixture");
    let config = AgentProcessConfig::new("fixture", executable, directory.path());
    assert!(matches!(config.check_installation(), Err(Error::ExecutableNotUsable { .. })));
}
