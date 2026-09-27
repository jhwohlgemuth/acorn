use super::super::{goose::Config, McpSync};
use acorn_schema::agent::ModelDetails;
use serde_norway::Value;
use std::env::current_exe;

#[test]
fn test_upsert_mcp_preserves_providers_and_extensions() {
    let executable = current_exe().expect("current test executable");
    let existing: Value = serde_norway::from_str("providers:\n  anthropic:\n    enabled: true\nextensions:\n  other:\n    type: stdio\n").unwrap();
    let updated = Config::default().upsert_mcp(existing, &executable, false).expect("upsert MCP");
    let rendered = serde_norway::to_string(&updated).unwrap();
    assert!(rendered.contains("anthropic:"));
    assert!(rendered.contains("other:"));
    assert!(rendered.contains("acorn:"));
    assert!(rendered.contains(executable.to_string_lossy().as_ref()));
}
#[test]
fn test_upsert_preserves_unrelated_settings_and_provider_fields() {
    let existing: Value =
        serde_norway::from_str("GOOSE_MODE: approve\nproviders:\n  anthropic:\n    enabled: true\n  openai:\n    custom: retained\n").unwrap();
    let models = [ModelDetails::init().id("qwen").build()];
    let updated = Config::default().upsert(existing, &models).unwrap();
    let rendered = serde_norway::to_string(&updated).unwrap();
    assert!(rendered.contains("GOOSE_MODE: approve"));
    assert!(rendered.contains("anthropic:"));
    assert!(rendered.contains("custom: retained"));
    assert!(rendered.contains("model: qwen"));
    assert!(rendered.contains("OPENAI_BASE_PATH: v1/chat/completions"));
}
