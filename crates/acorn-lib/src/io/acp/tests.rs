use super::{AgentProcessConfig, Client, Error, OPENCODE_AGENT_ID};
use crate::agent::{HydratedPrompt, InferencePermissionPolicy, InferenceRequest};
use acorn_cmd::{find_executable, CommandSpec};
use acorn_schema::agent::{FrontMatter, PromptFileAsset};
use core::time::Duration;
use serde_json::json;
use std::{ffi::OsStr, path::PathBuf};
use tempfile::TempDir;

struct FakeAgent {
    _directory: TempDir,
    executable: PathBuf,
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
        timeout_ms: 10_000,
    }
}
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("build test runtime")
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
    timed.timeout_ms = 50;
    let result = runtime().block_on(client.infer_checked(&timed));
    assert!(matches!(result, Err(Error::Timeout { .. })));
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
fn test_agent_process_config_opencode_uses_thin_command_defaults() {
    let working_directory = std::env::current_dir().expect("current directory");
    let config = AgentProcessConfig::opencode(None, working_directory.clone());
    assert_eq!(config.agent_id(), OPENCODE_AGENT_ID);
    assert_eq!(config.executable(), &PathBuf::from("opencode"));
    assert_eq!(config.acp_arguments(), ["acp"]);
    assert_eq!(config.permission_policy(), InferencePermissionPolicy::DenyMutation);
    assert_eq!(config.version_arguments(), ["--version"]);
    assert_eq!(config.working_directory(), &working_directory);
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
#[cfg(unix)]
#[test]
fn test_installation_preflight_rejects_non_executable_file() {
    let directory = tempfile::tempdir().expect("create fixture directory");
    let executable = directory.path().join("not-executable");
    std::fs::write(&executable, "fixture").expect("write fixture");
    let config = AgentProcessConfig::new("fixture", executable, directory.path());
    assert!(matches!(config.check_installation(), Err(Error::ExecutableNotUsable { .. })));
}
