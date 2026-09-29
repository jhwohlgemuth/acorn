use super::*;
use std::path::PathBuf;
fn arguments(backend: InferenceBackendKind) -> InferenceArgs {
    InferenceArgs::init().backend(backend).working_directory(PathBuf::from(".")).build()
}
#[test]
fn acp_is_the_default_backend_shape() {
    let configured = ConfiguredInference::from_args(&arguments(InferenceBackendKind::Acp), false).expect("configure ACP");
    assert_eq!(configured.agent.as_deref(), Some(OPENCODE_AGENT_ID));
    assert_eq!(configured.timeout, Milliseconds::new(30_000));
}
#[test]
fn openai_requires_a_model_and_rejects_acp_flags() {
    let missing = ConfiguredInference::from_args(&arguments(InferenceBackendKind::Openai), false);
    assert!(missing.is_err());
    let configured = InferenceArgs::init()
        .backend(InferenceBackendKind::Openai)
        .model("model".to_string())
        .build();
    assert!(ConfiguredInference::from_args(&configured, false).is_ok());
}
