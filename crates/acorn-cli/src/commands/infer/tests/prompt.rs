use super::super::prompt::{apply_sets, decode, run};
use crate::cli::arguments::{InferenceArgs, InferenceBackendKind, InferenceEndpointMode, InferenceOutputFormat, PromptDataFormatArg, PromptRun};
use acorn_schema::validation::Validate;
use clap_verbosity_flag::Verbosity;
use serde_json::{Map, Value};
use std::path::PathBuf;
#[tokio::test]
async fn dry_run_does_not_require_the_agent_executable() {
    let directory = tempfile::tempdir().expect("create prompt data directory");
    let path = directory.path().join("input.json");
    std::fs::write(&path, r#"{"text":"hello"}"#).expect("write prompt data");
    let arguments = PromptRun {
        asset: "summarize".to_string(),
        paths: vec![path],
        dry_run: true,
        inference: InferenceArgs::init()
            .backend(InferenceBackendKind::Acp)
            .agent("opencode".to_string())
            .agent_executable(PathBuf::from("/definitely/missing/opencode"))
            .build(),
        output: InferenceOutputFormat::Json,
        format: PromptDataFormatArg::Zon,
        set: Vec::new(),
        verbose: Verbosity::default(),
    };
    assert!(run(&arguments, false).await.is_ok());
}
#[test]
fn dry_run_still_rejects_incompatible_backend_flags() {
    let arguments = InferenceArgs::init()
        .backend(InferenceBackendKind::Acp)
        .base_url("http://localhost:8080".to_string())
        .endpoint_mode(InferenceEndpointMode::Responses)
        .build();
    assert!(arguments.validate().is_err());
}
#[test]
fn globbed_inputs_are_sorted_and_merged() {
    let directory = tempfile::tempdir().expect("create prompt data directory");
    std::fs::write(directory.path().join("b.json"), r#"{"text":"second"}"#).expect("write second input");
    std::fs::write(directory.path().join("a.json"), r#"{"text":"first"}"#).expect("write first input");
    let arguments = PromptRun {
        asset: "summarize".to_string(),
        paths: vec![directory.path().join("*.json")],
        dry_run: true,
        inference: InferenceArgs::default(),
        output: InferenceOutputFormat::Json,
        format: PromptDataFormatArg::Zon,
        set: Vec::new(),
        verbose: Verbosity::default(),
    };
    assert_eq!(
        arguments.load().expect("load prompt data").get("text"),
        Some(&Value::String("first\n\nsecond".to_string()))
    );
}
#[test]
fn set_accepts_typed_scalars_and_rejects_type_changes() {
    let mut data = Map::from_iter([("count".to_string(), Value::from(1))]);
    apply_sets(&mut data, &["enabled=true".to_string(), "count=2".to_string()]).expect("apply typed scalars");
    assert_eq!(data.get("enabled"), Some(&Value::Bool(true)));
    assert_eq!(data.get("count"), Some(&Value::from(2)));
    assert!(apply_sets(&mut data, &["count=two".to_string()]).is_err());
}
#[test]
fn set_rejects_reserved_keys_and_compound_values() {
    let mut data = Map::new();
    assert!(apply_sets(&mut data, &["config=true".to_string()]).is_err());
    assert!(apply_sets(&mut data, &["items=[]".to_string()]).is_err());
}
#[test]
fn stdin_decode_precedence_accepts_each_supported_format() {
    assert!(decode(r#"{"name":"json"}"#, None).is_ok());
    assert!(decode("name:zon", None).is_ok());
    assert!(decode("name: yaml\n", None).is_ok());
}
