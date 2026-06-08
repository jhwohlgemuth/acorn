use crate::schema::agent::{PromptFileAsset, PromptTemplate, PromptTemplateConfiguration};

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
