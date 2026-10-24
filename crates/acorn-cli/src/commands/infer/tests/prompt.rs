use super::super::prompt::{apply_sets, decode, run, source, Options, PromptSource};
use crate::cli::arguments::{
    AcpAgentKind, InferenceArgs, InferenceBackendKind, InferenceEndpointMode, InferenceOutputFormat, PromptDataFormatArg, PromptRun,
};
use acorn_schema::validation::Validate;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;
use serde_json::{json, Map, Value};
use std::path::PathBuf;

fn options(stdin_piped: bool, read_clipboard: fn() -> Result<String>, read_stdin: fn() -> Option<String>) -> Options {
    Options {
        read_clipboard,
        read_stdin,
        stdin_piped,
    }
}
fn prompt_run(data: Option<Option<&str>>, paste: bool) -> PromptRun {
    PromptRun {
        asset: "summarize".to_string(),
        data: data.map(|data| data.map(str::to_string)),
        dry_run: true,
        format: PromptDataFormatArg::Zon,
        inference: InferenceArgs::default(),
        output: InferenceOutputFormat::Json,
        paste,
        paths: Vec::new(),
        set: Vec::new(),
        verbose: Verbosity::default(),
    }
}
#[test]
fn test_data_accepts_only_inline_json() {
    let arguments = prompt_run(Some(Some(r#"{"text":"inline"}"#)), false);
    assert_eq!(
        arguments
            .load(Some(options(false, || Ok(String::new()), || None)))
            .expect("load inline JSON")
            .get("text"),
        Some(&Value::String("inline".to_string()))
    );
    assert!(prompt_run(Some(Some("-")), false)
        .load(Some(options(false, || Ok(String::new()), || None)))
        .is_err());
    assert!(prompt_run(Some(Some("text: yaml")), false)
        .load(Some(options(false, || Ok(String::new()), || None)))
        .is_err());
}
#[test]
fn test_data_merges_inline_after_file_input() {
    let directory = tempfile::tempdir().expect("create prompt data directory");
    let path = directory.path().join("input.json");
    std::fs::write(&path, r#"{"items":[1],"text":"file"}"#).expect("write prompt data");
    let arguments = PromptRun {
        paths: vec![path],
        ..prompt_run(Some(Some(r#"{"items":[2],"text":"data"}"#)), false)
    };
    let loaded = arguments
        .load(Some(options(false, || Ok(String::new()), || None)))
        .expect("load file and inline JSON");
    assert_eq!(loaded.get("items"), Some(&json!([1, 2])));
    assert_eq!(loaded.get("text"), Some(&Value::String("file\n\ndata".to_string())));
}
#[test]
fn test_data_merges_stdin_after_file_input() {
    let directory = tempfile::tempdir().expect("create prompt data directory");
    let path = directory.path().join("input.json");
    std::fs::write(&path, r#"{"items":[1],"text":"file"}"#).expect("write prompt data");
    let arguments = PromptRun {
        paths: vec![path],
        ..prompt_run(Some(None), false)
    };
    let loaded = arguments
        .load(Some(options(
            true,
            || Ok(String::new()),
            || Some(r#"{"items":[2],"text":"stdin"}"#.to_string()),
        )))
        .expect("load file and standard-input JSON");
    assert_eq!(loaded.get("items"), Some(&json!([1, 2])));
    assert_eq!(loaded.get("text"), Some(&Value::String("file\n\nstdin".to_string())));
}
#[test]
fn test_data_paste_rejects_piped_stdin() {
    let error = prompt_run(Some(None), true)
        .load(Some(options(
            true,
            || Ok(r#"{"text":"clipboard"}"#.to_string()),
            || Some(r#"{"text":"stdin"}"#.to_string()),
        )))
        .expect_err("reject simultaneous clipboard and standard input");
    assert!(error.to_string().contains("--paste cannot be used while standard input is piped"));
}
#[test]
fn test_data_reads_json_from_clipboard_only_with_paste() {
    let arguments = prompt_run(Some(None), true);
    assert_eq!(
        arguments
            .load(Some(options(false, || Ok(r#"{"text":"clipboard"}"#.to_string()), || None)))
            .expect("load clipboard JSON")
            .get("text"),
        Some(&Value::String("clipboard".to_string()))
    );
    assert!(prompt_run(Some(Some(r#"{"text":"inline"}"#)), true)
        .load(Some(options(false, || Ok(String::new()), || None)))
        .is_err());
}
#[test]
fn test_data_reads_json_from_stdin() {
    let arguments = prompt_run(Some(None), false);
    assert_eq!(
        arguments
            .load(Some(options(true, || Ok(String::new()), || Some(r#"{"text":"stdin"}"#.to_string()),)))
            .expect("load standard-input JSON")
            .get("text"),
        Some(&Value::String("stdin".to_string()))
    );
}
#[test]
fn test_directory_input_expands_supported_files_recursively() {
    let directory = tempfile::tempdir().expect("create prompt data directory");
    let nested = directory.path().join("nested");
    std::fs::create_dir(&nested).expect("create nested prompt data directory");
    std::fs::write(directory.path().join("a.json"), r#"{"text":"first"}"#).expect("write JSON input");
    std::fs::write(nested.join("b.yaml"), "text: second\n").expect("write YAML input");
    std::fs::write(nested.join("ignored.txt"), "text: ignored\n").expect("write unsupported input");
    let arguments = PromptRun {
        paths: vec![directory.path().to_path_buf()],
        ..prompt_run(None, false)
    };
    assert_eq!(
        arguments.load(None).expect("load directory prompt data").get("text"),
        Some(&Value::String("first\n\nsecond".to_string()))
    );
}
#[tokio::test]
async fn test_dry_run_does_not_require_the_agent_executable() {
    let directory = tempfile::tempdir().expect("create prompt data directory");
    let path = directory.path().join("input.json");
    std::fs::write(&path, r#"{"text":"hello"}"#).expect("write prompt data");
    let arguments = PromptRun {
        asset: "summarize".to_string(),
        data: None,
        dry_run: true,
        format: PromptDataFormatArg::Zon,
        inference: InferenceArgs::init()
            .backend(InferenceBackendKind::Acp)
            .agent(AcpAgentKind::OpenCode)
            .agent_executable(PathBuf::from("/definitely/missing/opencode"))
            .build(),
        output: InferenceOutputFormat::Json,
        paste: false,
        paths: vec![path],
        set: Vec::new(),
        verbose: Verbosity::default(),
    };
    assert!(run(&arguments, false).await.is_ok());
}
#[test]
fn test_dry_run_still_rejects_incompatible_backend_flags() {
    let arguments = InferenceArgs::init()
        .backend(InferenceBackendKind::Acp)
        .base_url("http://localhost:8080".to_string())
        .endpoint_mode(InferenceEndpointMode::Responses)
        .build();
    assert!(arguments.validate().is_err());
}
#[tokio::test]
async fn test_script_dry_run_works_without_agent_or_data() {
    let directory = tempfile::tempdir().expect("script directory");
    let script = directory.path().join("static.rhai");
    std::fs::write(&script, "// @acorn-exec-v0.2.0\nfn evaluate(context) { \"Return one sentence.\" }").expect("script");
    let arguments = PromptRun {
        asset: script.to_string_lossy().to_string(),
        inference: InferenceArgs::init()
            .agent_executable(PathBuf::from("/definitely/missing/opencode"))
            .build(),
        ..prompt_run(None, false)
    };
    assert!(run(&arguments, false).await.is_ok());
}
#[test]
fn test_script_source_requires_rhai_path() {
    assert!(matches!(source("./prompt.rhai"), Ok(PromptSource::Script(_))));
    assert!(matches!(source("summarize"), Ok(PromptSource::Embedded(_))));
    assert!(source("unknown").is_err());
}
#[test]
fn test_globbed_inputs_are_sorted_and_merged() {
    let directory = tempfile::tempdir().expect("create prompt data directory");
    std::fs::write(directory.path().join("b.json"), r#"{"text":"second"}"#).expect("write second input");
    std::fs::write(directory.path().join("a.json"), r#"{"text":"first"}"#).expect("write first input");
    let arguments = PromptRun {
        asset: "summarize".to_string(),
        data: None,
        dry_run: true,
        format: PromptDataFormatArg::Zon,
        inference: InferenceArgs::default(),
        output: InferenceOutputFormat::Json,
        paste: false,
        paths: vec![directory.path().join("*.json")],
        set: Vec::new(),
        verbose: Verbosity::default(),
    };
    assert_eq!(
        arguments.load(None).expect("load prompt data").get("text"),
        Some(&Value::String("first\n\nsecond".to_string()))
    );
}
#[test]
fn test_set_accepts_typed_scalars_and_rejects_type_changes() {
    let mut data = Map::from_iter([("count".to_string(), Value::from(1))]);
    apply_sets(&mut data, &["enabled=true".to_string(), "count=2".to_string()]).expect("apply typed scalars");
    assert_eq!(data.get("enabled"), Some(&Value::Bool(true)));
    assert_eq!(data.get("count"), Some(&Value::from(2)));
    assert!(apply_sets(&mut data, &["count=two".to_string()]).is_err());
}
#[test]
fn test_set_rejects_reserved_keys_and_compound_values() {
    let mut data = Map::new();
    assert!(apply_sets(&mut data, &["config=true".to_string()]).is_err());
    assert!(apply_sets(&mut data, &["items=[]".to_string()]).is_err());
}
#[test]
fn test_stdin_decode_precedence_accepts_each_supported_format() {
    assert!(decode(r#"{"name":"json"}"#, None).is_ok());
    assert!(decode("name:zon", None).is_ok());
    assert!(decode("name: yaml\n", None).is_ok());
}
