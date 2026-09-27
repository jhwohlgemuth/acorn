use super::super::{vscode::Config, McpSync, SyncTarget};
use crate::util::constants::app::DEFAULT_VSCODE_CONFIG_PATH;
use crate::util::StringConversion;
use acorn_schema::agent::ModelDetails;
use directories::BaseDirs;
use serde_json::Value;
use std::{env::current_exe, path::PathBuf};

#[test]
fn test_resolve_path_normalizes_explicit_path() {
    let path = "custom/Code/User/chatLanguageModels.json";
    assert_eq!(
        Config::resolve_path(Some(path)).unwrap(),
        PathBuf::from(path.to_string().to_cross_platform_path())
    );
}
#[test]
fn test_resolve_path_uses_platform_config_directory() {
    let expected = BaseDirs::new()
        .map(|directories| directories.config_dir().join(DEFAULT_VSCODE_CONFIG_PATH))
        .unwrap();
    assert_eq!(Config::resolve_path(None).unwrap(), expected);
}
#[test]
fn test_upsert_mcp_preserves_unrelated_servers_and_sandbox() {
    let executable = current_exe().expect("current test executable");
    let existing = serde_json::json!({"servers":{"other":{"command":"other"}},"sandbox":{"enabled":true}});
    let updated = Config::default().upsert_mcp(existing, &executable, false).expect("upsert MCP");
    assert_eq!(updated.pointer("/servers/acorn/type").and_then(Value::as_str), Some("stdio"));
    assert!(updated.pointer("/servers/other").is_some());
    assert_eq!(updated.pointer("/sandbox/enabled").and_then(Value::as_bool), Some(true));
}
#[test]
fn test_upsert_preserves_unrelated_provider_and_prunes_managed_models() {
    let existing = serde_json::json!([
        {"name": "Other", "vendor": "openai", "models": []},
        {"name": "Local (llama-swap)", "vendor": "customendpoint", "apiKey": "saved", "models": [{"id": "stale"}]}
    ]);
    let models = [ModelDetails::init().id("qwen").name("Qwen").build()];
    let additive = Config::default().upsert(existing.clone(), &models, false).unwrap();
    assert_eq!(additive.as_array().unwrap().len(), 2);
    assert!(additive.pointer("/1/models/0/id").is_some_and(|id| id == "stale"));
    assert_eq!(additive.pointer("/1/apiKey").and_then(Value::as_str), Some("saved"));
    let pruned = Config::default().upsert(existing, &models, true).unwrap();
    assert_eq!(pruned.pointer("/1/models/0/id").and_then(Value::as_str), Some("qwen"));
    assert_eq!(pruned.pointer("/1/apiType").and_then(Value::as_str), Some("chat-completions"));
}
