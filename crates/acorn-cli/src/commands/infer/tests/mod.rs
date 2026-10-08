use super::{resolve_prompt, Backend, ConfiguredInference, InferenceArgs, InferenceBackendKind, Milliseconds};
use crate::cli::arguments::AcpAgentKind;
use acorn_core::time::UnitOfTime;
use acorn_schema::software::Software;
use core::cell::Cell;
use std::path::PathBuf;

mod prompt;

fn arguments(backend: InferenceBackendKind) -> InferenceArgs {
    InferenceArgs::init().backend(backend).working_directory(PathBuf::from(".")).build()
}

#[test]
fn test_acp_is_the_default_backend_shape() {
    let configured = ConfiguredInference::from_args(&arguments(InferenceBackendKind::Acp), false).expect("configure ACP");
    assert_eq!(configured.agent.as_deref(), Some(Software::OpenCode.command()));
    assert_eq!(configured.timeout, Milliseconds::new(30_000));
}
#[test]
fn test_acp_selects_codex_profile_and_executable_override() {
    let arguments = InferenceArgs::init()
        .backend(InferenceBackendKind::Acp)
        .agent(AcpAgentKind::Codex)
        .agent_executable(PathBuf::from("custom/codex-acp"))
        .working_directory(PathBuf::from("."))
        .build();
    let configured = ConfiguredInference::from_args(&arguments, false).expect("configure Codex ACP");
    assert_eq!(configured.agent.as_deref(), Some(Software::Codex.command()));
    assert!(matches!(
        configured.backend,
        Backend::Acp(ref client) if client.config().executable() == std::path::Path::new("custom/codex-acp")
    ));
}
#[test]
fn test_openai_requires_a_model_and_rejects_acp_flags() {
    let missing = ConfiguredInference::from_args(&arguments(InferenceBackendKind::Openai), false);
    assert!(missing.is_err());
    let configured = InferenceArgs::init()
        .backend(InferenceBackendKind::Openai)
        .model("model".to_string())
        .build();
    assert!(ConfiguredInference::from_args(&configured, false).is_ok());
}
#[test]
fn test_resolve_prompt_prefers_the_argument() {
    let read = Cell::new(false);
    let prompt = resolve_prompt(Some(" argument "), || {
        read.set(true);
        None
    })
    .expect("argument prompt");
    assert_eq!(prompt, "argument");
    assert!(!read.get());
}
#[test]
fn test_resolve_prompt_reads_standard_input() {
    let prompt = resolve_prompt(None, || Some(" piped prompt\n".to_string())).expect("piped prompt");
    assert_eq!(prompt, "piped prompt");
}
#[test]
fn test_resolve_prompt_rejects_missing_input() {
    assert!(resolve_prompt(None, || None).is_err());
    assert!(resolve_prompt(None, || Some(" \n".to_string())).is_err());
}
