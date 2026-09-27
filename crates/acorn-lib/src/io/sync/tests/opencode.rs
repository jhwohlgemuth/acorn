#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::super::{
    opencode::{Config, Inner},
    McpSync, SyncTarget,
};
use crate::io::{home_directory, parse_jsonc_cst, PathConversion};
use crate::test::utils::temp_dir;
use crate::util::constants::app::DEFAULT_OPENCODE_CONFIG_PATH;
use acorn_schema::agent::opencode::{self, McpConfig, McpLocalConfig};
use serde_json::Value;
use std::env::{current_dir, current_exe};
use std::fs::{create_dir_all, remove_dir_all, write};
use std::path::{Path, PathBuf};

#[test]
fn test_default_values() {
    let config = Config::default();
    assert_eq!(config.base_url, "http://localhost:8080/v1");
    assert_eq!(config.provider_id, "llama-swap");
    assert_eq!(config.provider_name, "Local (llama-swap)");
    assert_eq!(config.inner.to_string(), "llama-swap");
}
#[test]
fn test_discover_path_prefers_calling_directory_jsonc_then_json() {
    let dir = temp_dir("opencode-calling-directory");
    let user_config = dir.join("user").join("opencode.jsonc");
    create_dir_all(user_config.parent().unwrap()).unwrap();
    write(&user_config, "{}").unwrap();
    let project_json = dir.join("opencode.json");
    write(&project_json, "{}").unwrap();
    assert_eq!(Inner::discover_path(&dir, user_config.clone()), project_json);
    let project_jsonc = dir.join("opencode.jsonc");
    write(&project_jsonc, "not valid jsonc").unwrap();
    assert_eq!(Inner::discover_path(&dir, user_config), project_jsonc);
    let _ = remove_dir_all(dir);
}
#[test]
fn test_model_json_structure() {
    let json = Config::default().model_entry("Qwen GGUF");
    assert_eq!(json.get("name").unwrap().as_str().unwrap(), "Qwen GGUF");
}
#[test]
fn test_provider_json_structure() {
    let config = Config::default();
    let json = config.provider_entry();
    assert_eq!(json.get("npm").unwrap().as_str().unwrap(), "@ai-sdk/openai-compatible");
    assert_eq!(json.get("name").unwrap().as_str().unwrap(), "Local (llama-swap)");
    let options = json.get("options").unwrap();
    assert_eq!(options.get("baseURL").unwrap().as_str().unwrap(), "http://localhost:8080/v1");
}
#[test]
fn test_render_preserves_jsonc_comments_outside_managed_provider() {
    let (existing, cst) = parse_jsonc_cst::<opencode::Config>(
        r#"{
  // root comment
  "username": "acorn",
  "provider": {
    // unrelated provider comment
    "other": {"options": {"baseURL": "https://example.test"}},
    "llama-swap": {"models": {"stale": {"name": "Stale"}}}
  }
}"#,
    )
    .unwrap();
    let config = Config::default();
    let updated = config.upsert(&existing, &[("qwen".to_string(), "Qwen".to_string())], false).unwrap();
    let rendered = config.render_with_cst(&updated, Some(&cst)).unwrap();
    assert!(rendered.contains("// root comment"));
    assert!(rendered.contains("// unrelated provider comment"));
    assert!(rendered.contains("\"stale\""));
    assert!(rendered.contains("\"qwen\""));
    assert_eq!(rendered, config.render_with_cst(&updated, Some(&cst)).unwrap());
}
#[test]
fn test_resolve_path_prefers_explicit_path() {
    let explicit = PathBuf::from(Path::new("custom/opencode.json").cross_platform_display());
    assert_eq!(Config::resolve_path(Some("custom/opencode.json")).unwrap(), explicit);
}
#[test]
fn test_resolve_path_uses_discovered_or_default_config() {
    let calling_directory = current_dir().unwrap();
    let user_config = home_directory(DEFAULT_OPENCODE_CONFIG_PATH).unwrap();
    let expected = [
        calling_directory.join("opencode.jsonc"),
        calling_directory.join("opencode.json"),
        user_config.clone(),
        user_config.with_extension("json"),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .unwrap_or(user_config);
    assert_eq!(Config::resolve_path(None).unwrap(), expected);
}
#[test]
fn test_upsert_is_additive_unless_pruned() {
    let existing = opencode::Config {
        username: Some("acorn".to_string()),
        provider: Some(
            [
                ("other".to_string(), serde_json::json!({"options": {"baseURL": "https://example.test"}})),
                (
                    "llama-swap".to_string(),
                    serde_json::json!({
                        "custom": true,
                        "options": {"apiKey": "secret"},
                        "models": {"stale": {"name": "Stale"}}
                    }),
                ),
            ]
            .into_iter()
            .collect(),
        ),
        ..Default::default()
    };
    let config = Config::default();
    let models = [("qwen".to_string(), "Qwen".to_string())];
    let additive = config.upsert(&existing, &models, false).unwrap();
    let provider = additive.provider.as_ref().unwrap().get("llama-swap").unwrap();
    assert!(provider.pointer("/models/stale").is_some());
    assert!(provider.pointer("/models/qwen").is_some());
    assert_eq!(provider.get("custom"), Some(&Value::Bool(true)));
    assert_eq!(provider.pointer("/options/apiKey").and_then(Value::as_str), Some("secret"));
    assert!(additive.provider.as_ref().unwrap().contains_key("other"));
    assert_eq!(additive.username.as_deref(), Some("acorn"));
    let pruned = config.upsert(&existing, &models, true).unwrap();
    let provider = pruned.provider.as_ref().unwrap().get("llama-swap").unwrap();
    assert!(provider.pointer("/models/stale").is_none());
    assert!(provider.pointer("/models/qwen").is_some());
}
#[test]
fn test_upsert_mcp_preserves_other_servers_and_prunes_only_acorn() {
    let executable = current_exe().expect("current test executable");
    let existing = opencode::Config {
        mcp: Some(
            [(
                "other".to_string(),
                McpConfig::Local(McpLocalConfig {
                    command: vec!["other".to_string()],
                    cwd: None,
                    environment: None,
                    enabled: None,
                    timeout: None,
                }),
            )]
            .into_iter()
            .collect(),
        ),
        ..Default::default()
    };
    let updated = Config::default().upsert_mcp(existing, &executable, false).expect("upsert MCP");
    assert!(updated
        .mcp
        .as_ref()
        .is_some_and(|servers| servers.contains_key("acorn") && servers.contains_key("other")));
    let pruned = Config::default().upsert_mcp(updated, &executable, true).expect("prune MCP");
    assert!(pruned
        .mcp
        .as_ref()
        .is_some_and(|servers| !servers.contains_key("acorn") && servers.contains_key("other")));
}
#[test]
fn test_upsert_protects_pruned_root_model() {
    let existing = opencode::Config {
        model: Some("llama-swap/stale".to_string()),
        provider: Some(
            [("llama-swap".to_string(), serde_json::json!({"models": {"stale": {"name": "Stale"}}}))]
                .into_iter()
                .collect(),
        ),
        ..Default::default()
    };
    let models = [("qwen".to_string(), "Qwen".to_string())];
    assert!(Config::default().upsert(&existing, &models, true).is_err());
    let replacement = Config {
        inner: Inner {
            default_model: Some("qwen".to_string()),
            ..Default::default()
        },
        ..Default::default()
    }
    .upsert(&existing, &models, true)
    .unwrap();
    assert_eq!(replacement.model.as_deref(), Some("llama-swap/qwen"));
}
