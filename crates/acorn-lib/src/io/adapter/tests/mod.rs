#![allow(clippy::indexing_slicing, clippy::unwrap_used)]
mod robustness;

use super::migrate::Migration;
use super::*;
use crate::test::utils::fixture_path;
use acorn_schema::agent::opencode;
use acorn_schema::research_activity::Sections;
use acorn_schema::standard::cff::IdentifierType;
use acorn_schema::standard::text::Document;
use acorn_schema::standard::zon;
use std::fs::{read, write};
use tempfile::tempdir;

#[test]
fn test_cff_input_output_read_and_write_json() {
    let directory = tempdir().expect("temporary directory should be created");
    let path = directory.path().join("CITATION.json");
    let cff = Cff {
        title: "Roundtrip JSON".to_string(),
        message: "Message".to_string(),
        cff_version: "1.2.0".to_string(),
        ..Cff::default()
    };
    cff.write_json(&path).expect("failed to write CFF JSON");
    let read_back = Cff::read_json(path).expect("failed to read CFF JSON");
    assert_eq!(read_back.title, "Roundtrip JSON");
    assert_eq!(read_back.message, "Message");
}
#[test]
fn test_cff_input_output_read_and_write_yaml() {
    let directory = tempdir().expect("temporary directory should be created");
    let path = directory.path().join("CITATION.yaml");
    let cff = Cff {
        title: "Roundtrip YAML".to_string(),
        message: "Message".to_string(),
        cff_version: "1.2.0".to_string(),
        ..Cff::default()
    };
    cff.write_yaml(&path).expect("failed to write CFF YAML");
    let read_back = Cff::read_yaml(path).expect("failed to read CFF YAML");
    assert_eq!(read_back.title, "Roundtrip YAML");
    assert_eq!(read_back.message, "Message");
}
#[test]
fn test_document_input_output_reads_docx_fixture() {
    let result = Document::read(fixture_path("acorn.docx")).expect("failed to read DOCX file");
    assert!(result.content.contains("ACORN"));
}
#[test]
fn test_document_input_output_rejects_docx_write() {
    let directory = tempdir().expect("temporary directory should be created");
    let source = Document {
        content: "plain text content".to_string(),
    };
    assert!(source.write(directory.path().join("output.docx")).is_err());
}
#[test]
fn test_field_migrations_detect_deprecated_rad_paths() {
    let value = serde_json::json!({
        "meta": {"media": [{"href": "image.png"}, {"url": "video.mp4"}]},
        "contact": {"profile": "https://example.com"},
        "sections": {"outcomes": ["Outcome"]}
    });
    assert_eq!(
        ResearchActivity::field_migrations(&value),
        vec![
            ("meta.media[].href", "meta.media[].contentUrl"),
            ("meta.media[].url", "meta.media[].contentUrl"),
            ("contact.profile", "contact.url"),
            ("sections.outcomes", "sections.impact")
        ]
    );
}
#[test]
fn test_logbook_input_output_round_trips_supported_formats() {
    let directory = tempdir().expect("temporary directory should be created");
    let entry = LogbookEntry::init()
        .identifier("first-entry")
        .timestamp("2026-09-01T12:00:00Z")
        .category("other")
        .state("completed")
        .summary("First entry")
        .build();
    let logbook = Logbook::init().entries(vec![entry.clone()]).build();
    [
        directory.path().join("logbook.json"),
        directory.path().join("logbook.md"),
        directory.path().join("logbook.yaml"),
    ]
    .into_iter()
    .for_each(|path| {
        logbook.write(&path).expect("logbook should write");
        let parsed = Logbook::read(path).expect("logbook should read");
        assert_eq!(parsed.entries[0].identifier, "first-entry");
    });
    let path = directory.path().join("entry.md");
    entry.write(&path).expect("entry should write");
    let parsed = LogbookEntry::read(path).expect("entry should read");
    assert_eq!(parsed.identifier, "first-entry");
}
#[test]
fn test_malformed_jsonc_reports_error() {
    let content = r#"{
        // Valid comment
        "title": 123,
        "sections": "not an object"
    }"#;
    let value = jsonc_parse_value(content).expect("valid JSONC syntax should parse");
    let json = serde_json::to_string(&value).expect("JSONC value should serialize");
    assert!(eserde::json::from_str::<ResearchActivity>(&json).is_err());
}
#[test]
fn test_migrate_allows_strict_read() {
    let activity = ResearchActivity::init().sections(Sections::default()).build();
    let mut value = serde_json::to_value(activity).expect("RAD should serialize");
    value["meta"]["media"] = serde_json::json!([{"caption": "Caption", "href": "image.png"}]);
    let impact = value["sections"].as_object_mut().expect("sections is an object").remove("impact");
    value["sections"]["outcomes"] = impact.expect("serialized sections contain impact");
    let directory = tempdir().expect("temporary directory should be created");
    let path = directory.path().join("legacy.json");
    write(&path, serde_json::to_vec(&value).expect("legacy RAD should serialize")).expect("legacy RAD fixture should be written");
    assert!(ResearchActivity::read(path.clone()).is_ok());
    let why = ResearchActivity::read_strict(path.clone()).expect_err("strict reads should reject deprecated fields");
    assert!(why.to_string().contains("meta.media[].href"));
    assert!(why.to_string().contains("meta.media[].contentUrl"));
    assert!(ResearchActivity::migrate(path.clone()).expect("legacy RAD should migrate"));
    assert!(ResearchActivity::read_strict(path.clone()).is_ok());
    let canonical: serde_json::Value =
        serde_json::from_slice(&read(&path).expect("canonical RAD should be read")).expect("canonical RAD should deserialize as JSON");
    assert_eq!(canonical["meta"]["media"][0]["contentUrl"], "image.png");
    assert!(canonical["meta"]["media"][0].get("href").is_none());
    assert!(canonical["sections"].get("impact").is_some());
    assert!(canonical["sections"].get("outcomes").is_none());
    assert!(!ResearchActivity::migrate(path).expect("canonical RAD should remain unchanged"));
    let rad_with_handle = ResearchActivity::init()
        .meta(ResearchActivityMetadata::init().handle(vec!["hdl:10013/epic.10033".to_string()]).build())
        .build();
    let cff_with_handle: Cff = rad_with_handle.into();
    let identifier = cff_with_handle
        .identifiers
        .and_then(|identifiers| identifiers.into_iter().next())
        .expect("Handle should be exported as a CFF identifier");
    assert_eq!(identifier.kind, IdentifierType::Other);
    assert_eq!(identifier.value, "10013/epic.10033");
    assert_eq!(identifier.description.as_deref(), Some("Handle identifier"));
}
#[test]
fn test_opencode_config_from_jsonc_file() {
    let config = opencode::Config::read(fixture_path("config/opencode.jsonc")).expect("JSONC fixture should load");
    assert_eq!(config.model.as_deref(), Some("anthropic/claude-sonnet-4-5"));
    assert_eq!(
        config.agent.and_then(|agents| agents.get("build").cloned()).and_then(|agent| agent.steps),
        Some(50)
    );
}

#[test]
fn test_opencode_config_from_jsonc_rejects_invalid() {
    assert!(parse_jsonc_cst::<opencode::Config>(r#"{ model: invalid }"#).is_err());
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
    let (config, _) = parse_jsonc_cst::<opencode::Config>(content).expect("JSONC should parse");
    assert!(config.mcp.is_some_and(|mcp| mcp.contains_key("local-tool")));
}
#[test]
fn test_opencode_config_from_jsonc_with_trailing_commas() {
    let content = r#"{
        "model": "openai/gpt-4o",
        "logLevel": "DEBUG",
    }"#;
    let (config, _) = parse_jsonc_cst::<opencode::Config>(content).expect("JSONC should parse");
    assert_eq!(config.model.as_deref(), Some("openai/gpt-4o"));
}
#[test]
fn test_opencode_config_read_json_round_trip() {
    let directory = tempdir().expect("temporary directory should be created");
    let path = directory.path().join("opencode.json");
    let config: opencode::Config =
        serde_json::from_str(r#"{"model":"anthropic/claude-sonnet-4-5","logLevel":"INFO"}"#).expect("OpenCode JSON should parse");
    config.write_json(&path).expect("OpenCode JSON should write");
    let loaded = opencode::Config::read_json(path).expect("OpenCode JSON should read");
    assert_eq!(loaded.model.as_deref(), Some("anthropic/claude-sonnet-4-5"));
    assert!(loaded.log_level.is_some());
}
#[test]
fn test_opencode_config_read_jsonc_file() {
    let config = opencode::Config::read(fixture_path("config/opencode.jsonc")).expect("JSONC fixture should load");
    assert_eq!(config.model.as_deref(), Some("anthropic/claude-sonnet-4-5"));
}
#[test]
fn test_opencode_config_write_dispatches_by_extension() {
    let directory = tempdir().expect("temporary directory should be created");
    let json = directory.path().join("opencode.json");
    let jsonc = directory.path().join("opencode.jsonc");
    let config: opencode::Config = serde_json::from_str(r#"{"model":"test"}"#).expect("OpenCode JSON should parse");
    config.write(&json).expect("JSON should write");
    config.write(&jsonc).expect("JSONC should write");
    assert_eq!(opencode::Config::read(json).expect("JSON should read").model.as_deref(), Some("test"));
    assert_eq!(opencode::Config::read(jsonc).expect("JSONC should read").model.as_deref(), Some("test"));
}
#[test]
fn test_read_jsonc_rejects_single_quotes() {
    assert!(jsonc_parse_value(r#"{'title': 'single quotes'}"#).is_err());
}
#[test]
fn test_read_jsonc_rejects_unquoted_keys() {
    assert!(jsonc_parse_value(r#"{ title: 'unquoted' }"#).is_err());
}
#[test]
fn test_read_jsonc_with_comments() {
    let value = jsonc_parse_value(
        r#"{
            // This is a comment
            "title": "JSONC Project",
            "sections": {"mission": "Mission statement"}
        }"#,
    )
    .expect("JSONC comments should parse");
    assert_eq!(value["title"], "JSONC Project");
    assert_eq!(value["sections"]["mission"], "Mission statement");
}
#[test]
fn test_read_jsonc_with_trailing_commas() {
    let value = jsonc_parse_value(
        r#"{
            "title": "Trailing Commas",
            "sections": {"approach": ["Approach",],},
        }"#,
    )
    .expect("JSONC trailing commas should parse");
    assert_eq!(value["title"], "Trailing Commas");
}
#[test]
fn test_research_activity_format() {
    let changed = fixture_path("data/format/changes");
    let pre = ResearchActivity::read(changed.join("index.json")).expect("changed fixture should read");
    assert!(pre.meta.media.is_none());
    assert!(pre.contact.affiliation.is_none());
    let post = pre.format_with(Some(changed));
    assert_eq!(
        post.meta
            .media
            .and_then(|media| media.first().cloned())
            .and_then(|media| media.content_url()),
        Some("42.png".to_string())
    );
    assert_eq!(post.contact.affiliation.as_deref(), Some("National Security Sciences Directorate"));
    let unresolved = ResearchActivity::read(fixture_path("data/format/unresolved_changes/index.json")).expect("unresolved fixture should read");
    assert!(unresolved.meta.media.is_some());
    assert_eq!(unresolved.contact.affiliation.as_deref(), Some("Not an actual affiliation"));
    let post = unresolved.format_with(None);
    assert!(post
        .meta
        .media
        .as_ref()
        .and_then(|media| media.first())
        .cloned()
        .and_then(MediaObject::content_url)
        .is_none());
    assert_eq!(
        post.meta
            .media
            .as_ref()
            .and_then(|media| media.first())
            .cloned()
            .map(MediaObject::description),
        Some(String::new())
    );
    assert_eq!(post.contact.affiliation.as_deref(), Some("Oak Ridge National Laboratory"));
    let mut pre = post.clone();
    pre.contact.organization = "Not an actual organization".to_string();
    pre.contact.affiliation = None;
    let post = pre.format_with(None);
    assert_eq!(post.contact.organization, String::new());
    assert_eq!(post.contact.affiliation.as_deref(), Some("Oak Ridge National Laboratory"));
    let mut pre = post.clone();
    pre.contact.organization = "Oak Ridge National Laboratory".to_string();
    pre.contact.affiliation = None;
    let post = pre.format_with(None);
    assert_eq!(post.contact.organization, "Oak Ridge National Laboratory");
    assert_eq!(post.contact.affiliation.as_deref(), Some("Oak Ridge National Laboratory"));
    let unchanged = fixture_path("data/format/no_changes");
    let pre = ResearchActivity::read(unchanged.join("index.json")).expect("unchanged fixture should read");
    assert_eq!(
        pre.meta
            .media
            .as_ref()
            .and_then(|media| media.first())
            .cloned()
            .and_then(MediaObject::content_url),
        Some("00.png".to_string())
    );
    let post = pre.format_with(Some(unchanged));
    assert_eq!(
        post.meta
            .media
            .as_ref()
            .and_then(|media| media.first())
            .cloned()
            .and_then(MediaObject::content_url),
        Some("42.png".to_string())
    );
    assert_eq!(post.contact.affiliation.as_deref(), Some("National Security Sciences Directorate"));
}
#[test]
fn test_text_input_output_rejects_unsupported_extension() {
    let directory = tempdir().expect("temporary directory should be created");
    assert!(Text::read(directory.path().join("text.pptx")).is_err());
}
#[test]
fn test_text_input_output_round_trip_txt() {
    let directory = tempdir().expect("temporary directory should be created");
    let path = directory.path().join("text.txt");
    let source = Text {
        content: "plain text content\nwith two lines".to_string(),
    };
    source.write(&path).expect("text should write");
    assert_eq!(Text::read(path).expect("text should read"), source);
}
#[test]
fn test_text_try_from_path_uses_supported_extensions() {
    let result = Text::read(fixture_path("gather/depth/direct.md")).expect("supported Markdown path should load");
    assert!(!result.content.is_empty());
    let directory = tempdir().expect("temporary directory should be created");
    assert!(Text::read(directory.path().join("text.pptx")).is_err());
}
#[test]
fn test_zon_rad_read_preserves_multiple_eserde_errors() {
    let directory = tempdir().expect("temporary directory should be created");
    let value: serde_json::Value = serde_json::from_slice(&read(fixture_path("malformed-rad.json")).unwrap()).unwrap();
    let path = directory.path().join("malformed.zonf");
    write(&path, zon::encode(&value).unwrap()).unwrap();
    let error = ResearchActivity::read_zon(path).expect_err("malformed ZON RAD should fail");
    let details = error.to_string();
    assert!(details.lines().count() >= 2, "expected multiple schema errors: {details}");
}
