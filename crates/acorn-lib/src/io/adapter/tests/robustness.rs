use super::super::*;
use acorn_schema::agent::opencode;
use acorn_schema::modelkit::kitfile::Kitfile;
use acorn_schema::standard::text::{Document, Text};
use acorn_schema::standard::zon;
use std::fs::{read_to_string, write};
use tempfile::tempdir;

#[test]
fn test_document_adapter_rejects_missing_and_unsupported_sources() {
    let directory = tempdir().expect("temporary directory should be created");
    assert!(Document::read(directory.path().join("missing.md")).is_err());
    assert!(Document::read(directory.path().join("unsupported.bin")).is_err());
    assert!(Document::default().serialize_as(&MimeType::Pdf).is_err());
}
#[test]
fn test_document_adapter_round_trips_text_formats() {
    let directory = tempdir().expect("temporary directory should be created");
    let document = Document {
        content: "document content".to_string(),
    };
    for extension in ["json", "md", "txt", "yaml", "yml"] {
        let path = directory.path().join(format!("document.{extension}"));
        document.write(&path).expect("document should write");
        assert_eq!(Document::read(path).expect("document should read"), document);
    }
}
#[test]
fn test_kitfile_adapter_round_trips_json_and_yaml() {
    let directory = tempdir().expect("temporary directory should be created");
    let kitfile = Kitfile::parse("manifestVersion: v1.0.0\ndocs:\n  - path: README.md\n").expect("Kitfile should parse");
    let yaml_path = directory.path().join("Kitfile.yaml");
    kitfile.write(&yaml_path).expect("Kitfile YAML should write");
    assert_eq!(Kitfile::read(yaml_path).expect("Kitfile YAML should read"), kitfile);
    let modelkit = Kitfile::from_resolved_json(
        br#"{"manifestVersion":"v1.0.0","docs":[{"path":"README.md","digest":"sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}]}"#,
    )
    .expect("ModelKit config should parse");
    let json_path = directory.path().join("modelkit.json");
    modelkit.write(&json_path).expect("ModelKit JSON should write");
    assert_eq!(Kitfile::read(json_path).expect("ModelKit JSON should read"), modelkit);
}
#[test]
fn test_opencode_adapter_errors_include_source_path() {
    let directory = tempdir().expect("temporary directory should be created");
    let path = directory.path().join("broken.json");
    write(&path, "{").expect("broken fixture should be written");
    let error = opencode::Config::read(&path).expect_err("malformed JSON should fail");
    assert!(error.to_string().contains(&path.display().to_string()));
}
#[test]
fn test_opencode_adapter_rejects_unsupported_and_yaml_operations() {
    let directory = tempdir().expect("temporary directory should be created");
    let config = opencode::Config::default();
    assert!(opencode::Config::read(directory.path().join("config.toml")).is_err());
    assert!(opencode::Config::read_yaml(directory.path().join("config.yaml")).is_err());
    assert!(config.write(directory.path().join("config.toml")).is_err());
    assert!(config.write_yaml(directory.path().join("config.yaml")).is_err());
}
#[test]
fn test_research_activity_adapter_round_trips_zonf() {
    let directory = tempdir().expect("temporary directory should be created");
    let path = directory.path().join("activity.zonf");
    let activity = ResearchActivity::default();
    activity.write(&path).expect("RAD should write as ZON");
    let decoded = ResearchActivity::read(path).expect("RAD should read as ZON");
    assert_eq!(serde_json::to_value(decoded).unwrap(), serde_json::to_value(activity).unwrap());
}
#[test]
fn test_research_activity_parse_accepts_json_jsonc_yaml_and_zon() {
    let value = ResearchActivity::default();
    let json = serde_json::to_string(&value).expect("RAD should serialize as JSON");
    let jsonc = json.replacen('{', "{\n// comment\n", 1);
    let yaml = serde_norway::to_string(&value).expect("RAD should serialize as YAML");
    let zon = zon::encode(&value).expect("RAD should serialize as ZON");
    assert!(ResearchActivity::parse(&json).is_some());
    assert!(ResearchActivity::parse(&jsonc).is_some());
    assert!(ResearchActivity::parse(&yaml).is_some());
    assert!(ResearchActivity::parse(&zon).is_some());
}
#[test]
fn test_research_activity_parse_rejects_malformed_and_wrong_shaped_values() {
    assert!(ResearchActivity::parse("{").is_none());
    assert!(ResearchActivity::parse("[]").is_none());
    assert!(ResearchActivity::parse("title: [unterminated").is_none());
}
#[test]
fn test_text_adapter_rejects_missing_and_unsupported_sources() {
    let directory = tempdir().expect("temporary directory should be created");
    assert!(Text::read(directory.path().join("missing.md")).is_err());
    assert!(Text::read(directory.path().join("unsupported.bin")).is_err());
    assert!(Text::default().serialize_as(&MimeType::Pdf).is_err());
}
#[test]
fn test_text_adapter_round_trips_supported_formats() {
    let directory = tempdir().expect("temporary directory should be created");
    let text = Text {
        content: "plain text content".to_string(),
    };
    for (extension, mime) in [
        ("json", MimeType::Json),
        ("md", MimeType::Markdown),
        ("txt", MimeType::Text),
        ("yaml", MimeType::Yaml),
    ] {
        let path = directory.path().join(format!("text.{extension}"));
        text.write(&path).expect("text should write");
        assert_eq!(read_to_string(&path).expect("text should be readable"), text.content);
        assert_eq!(Text::read(path).expect("text should read"), text);
        assert_eq!(text.serialize_as(&mime).expect("text should serialize"), text.content);
    }
}
