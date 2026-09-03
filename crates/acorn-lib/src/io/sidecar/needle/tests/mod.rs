use super::{assets, write_tools};
use crate::io::sidecar::{ModelKitManifest, ResolvedAssets};
use acorn_schema::schema::agent::tools::NeedleToolDefinition;

#[test]
fn test_custom_catalog_must_match_exactly() {
    let fixture = tempfile::tempdir().expect("fixture");
    let tools = fixture.path().join("tools.json");
    std::fs::write(&tools, "[]").expect("write tools");
    let assets = ResolvedAssets {
        runner: fixture.path().join("needle"),
        tools: Some(tools),
        root: fixture.path().to_path_buf(),
    };
    assert!(assets.clone().validate_tools(&[]).is_ok());
    assert!(assets
        .validate_tools(&[NeedleToolDefinition {
            name: "acorn.version".to_string(),
            description: "Version".to_string(),
            parameters: serde_json::json!({ "type": "object" }),
        }])
        .is_err());
}
#[test]
fn test_modelkit_rejects_unsafe_paths() {
    let manifest = ModelKitManifest {
        schema_version: 1,
        source_revision: "revision".to_string(),
        platform: "platform".to_string(),
        runner: "../needle".to_string(),
        runner_sha256: "checksum".to_string(),
        tools: None,
    };
    assert!(assets::download_items("https://example.com/modelkit.json", &manifest).is_err());
}
#[test]
fn test_official_asset_is_digest_pinned() {
    let asset = assets::EngineAsset::current().expect("supported test platform");
    assert_eq!(asset.sha256.len(), 64);
    let url = asset.url().expect("configured Hugging Face endpoint");
    assert!(url.contains(crate::util::constants::app::NEEDLE_REVISION));
    assert!(!url.contains("/main/"));
}
#[test]
fn test_tools_export_is_deterministic_and_native() {
    let fixture = tempfile::tempdir().expect("fixture");
    let first_path = fixture.path().join("first.json");
    let second_path = fixture.path().join("second.json");
    write_tools(first_path.clone()).expect("first export");
    write_tools(second_path.clone()).expect("second export");
    let first = std::fs::read_to_string(first_path).expect("first catalog");
    let second = std::fs::read_to_string(second_path).expect("second catalog");
    assert_eq!(first, second);
    let value: serde_json::Value = serde_json::from_str(&first).expect("tools JSON");
    assert!(value.as_array().is_some_and(|tools| {
        tools
            .iter()
            .all(|tool| tool.get("parameters").is_some() && tool.get("input_schema").is_none())
    }));
}
