//! Rhai prompt hydration tests.
use crate::agent::{PromptContext, PromptDataFormat, PromptScriptHydrator};
use acorn_core::util::constants::app::MAX_EXEC_BYTES;
use acorn_schema::agent::{PromptFileAsset, PromptTemplateConfiguration};
use serde_json::{json, Value};

fn context() -> PromptContext<Value> {
    PromptContext {
        config: PromptTemplateConfiguration::default(),
        data: json!({"title": "Research notes"}),
    }
}

#[test]
fn test_hydrates_script_as_complete_prompt() {
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { \"Review: \" + context.data.title }";
    let prompt = PromptScriptHydrator::hydrate("review.rhai", source, &context()).expect("prompt");
    assert_eq!(prompt.body, "Review: Research notes");
    assert_eq!(prompt.asset, PromptFileAsset::Script("review.rhai".to_string()));
    assert_eq!(prompt.metadata.name, "review");
    let preview = serde_json::to_value(&prompt).expect("preview JSON");
    assert_eq!(
        preview.get("asset").and_then(|asset| asset.get("Script")).and_then(Value::as_str),
        Some("review.rhai")
    );
}
#[test]
fn test_json_data_block_is_available_to_script() {
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { context.data_block }";
    let prompt = PromptScriptHydrator::hydrate_with_format("data.rhai", source, &context(), PromptDataFormat::Json).expect("JSON data block");
    assert!(prompt.body.contains("# Input Data (JSON)"));
    assert!(prompt.body.contains("\"title\": \"Research notes\""));
}
#[test]
fn test_rejects_host_access_and_oversized_source() {
    let host = "// @acorn-exec-v0.2.0\nfn evaluate(context) { read_text(\"secret\") }";
    assert!(PromptScriptHydrator::hydrate("host.rhai", host, &context()).is_err());
    let huge = format!("// @acorn-exec-v0.2.0\n{}", " ".repeat(MAX_EXEC_BYTES));
    assert!(PromptScriptHydrator::hydrate("huge.rhai", &huge, &context()).is_err());
}
#[test]
fn test_rejects_invalid_entry_and_result() {
    for source in [
        "fn evaluate(context) { \"text\" }",
        "// @acorn-prompt-v1\nfn evaluate(context) { \"text\" }",
        "// @acorn-exec-v2.0.0\nfn evaluate(context) { \"text\" }",
        "// @acorn-exec-v0.2.0\nfn other(context) { \"text\" }",
        "// @acorn-exec-v0.2.0\nfn evaluate() { \"text\" }",
        "// @acorn-exec-v0.2.0\nfn evaluate(context) { 42 }",
        "// @acorn-exec-v0.2.0\nfn evaluate(context) { \"  \" }",
    ] {
        assert!(
            PromptScriptHydrator::hydrate("invalid.rhai", source, &context()).is_err(),
            "accepted {source}"
        );
    }
}
#[test]
fn test_rejects_nonterminating_script() {
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { loop {} }";
    assert!(PromptScriptHydrator::hydrate("loop.rhai", source, &context()).is_err());
}
