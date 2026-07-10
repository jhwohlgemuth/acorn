#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
use crate::schema::agent::{opencode, PromptFileAsset, PromptTemplate, PromptTemplateConfiguration};
use std::path::PathBuf;

fn all_known_assets() -> [PromptFileAsset; 6] {
    [
        PromptFileAsset::Eli5,
        PromptFileAsset::ExtractClaim,
        PromptFileAsset::FindGaps,
        PromptFileAsset::Summarize,
        PromptFileAsset::Teach,
        PromptFileAsset::Translate,
    ]
}

#[test]
fn prompt_file_asset_from_str_round_trips_for_known_assets() {
    all_known_assets().iter().for_each(|asset| {
        let name = asset.to_string();
        let mapped = PromptFileAsset::from(name.as_str());
        assert_eq!(mapped.to_string(), name);
    });
}
#[test]
fn prompt_file_asset_unknown_from_str_round_trips() {
    let value = "does-not-exist.prompt";
    let mapped = PromptFileAsset::from(value);
    assert_eq!(mapped.to_string(), value);
}
#[test]
fn from_asset_returns_some_for_all_known_assets() {
    all_known_assets().iter().for_each(|asset| {
        let content = PromptTemplate::from_asset(&asset.to_string());
        assert!(content.is_some(), "Expected embedded asset to exist: {}", asset);
    });
}
#[test]
fn render_accepts_prompt_asset_str_and_string_inputs() {
    let config = PromptTemplateConfiguration::init().build();
    let from_enum = PromptTemplate::render(PromptFileAsset::Summarize, &config);
    let from_str = PromptTemplate::render("summarize", &config);
    let from_string = PromptTemplate::render("summarize.prompt".to_string(), &config);
    assert_eq!(from_enum.is_ok(), from_str.is_ok());
    assert_eq!(from_str.is_ok(), from_string.is_ok());
}
#[test]
fn render_unknown_asset_returns_error() {
    let config = PromptTemplateConfiguration::init().build();
    let result = PromptTemplate::render("unknown.prompt", &config);
    assert!(result.is_err());
}
#[test]
fn snapshot_prompt_asset_file_names() {
    let names: Vec<String> = all_known_assets().iter().map(ToString::to_string).collect();
    insta::assert_yaml_snapshot!("prompt_asset_file_names", names);
}
#[test]
fn snapshot_prompt_asset_template_headers() {
    let headers: Vec<String> = all_known_assets()
        .iter()
        .filter_map(|asset| PromptTemplate::from_asset(&asset.to_string()).map(|content| content.lines().take(6).collect::<Vec<&str>>().join("\n")))
        .collect();
    insta::assert_yaml_snapshot!("prompt_asset_template_headers", headers);
}
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("tests/fixtures")
}
#[test]
fn test_opencode_config_from_jsonc_file() {
    let path = fixtures_dir().join("config").join("opencode.jsonc");
    let config = opencode::Config::from_path(&path).unwrap();
    assert_eq!(config.model.as_deref(), Some("anthropic/claude-sonnet-4-5"));
    let agents = config.agent.unwrap();
    let build = agents.get("build").unwrap();
    assert_eq!(build.steps, Some(50));
}
#[test]
fn test_opencode_config_from_jsonc_string() {
    let content = r#"{
        // MCP server config
        "mcp": {
            "local-tool": {
                "type": "local",
                "command": ["npx", "my-tool"],
                "enabled": true
            }
        }
    }"#;
    let config = opencode::Config::parse_jsonc(content).unwrap();
    let mcp = config.mcp.unwrap();
    assert!(mcp.contains_key("local-tool"));
}
#[test]
fn test_opencode_config_from_jsonc_with_trailing_commas() {
    let content = r#"{
        "model": "openai/gpt-4o",
        "logLevel": "DEBUG",
    }"#;
    let config = opencode::Config::parse_jsonc(content).unwrap();
    assert_eq!(config.model.as_deref(), Some("openai/gpt-4o"));
}
#[test]
fn test_opencode_config_from_jsonc_rejects_invalid() {
    let content = r#"{ model: invalid }"#;
    assert!(opencode::Config::parse_jsonc(content).is_err());
}
