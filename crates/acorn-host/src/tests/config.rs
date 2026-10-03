use crate::config::{read, read_jsonc, resolve_config_path, ConfigError};
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Example {
    name: String,
}
#[test]
fn test_read_jsonc_enforces_shared_syntax() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("config.jsonc");
    [
        r#"{"name":"value"}"#,
        "{// line comment\n\"name\":\"value\"}",
        r#"{/* block comment */"name":"value"}"#,
        r#"{"name":"value",}"#,
    ]
    .into_iter()
    .for_each(|content| {
        fs::write(&path, content).expect("write valid JSONC fixture");
        assert_eq!(read_jsonc::<Example>(&path).expect("read valid JSONC").name, "value");
    });
    [
        r#"{name:"value"}"#,
        r#"{"name":"value" "other":true}"#,
        r#"{'name':'value'}"#,
        r#"{"name":0x10}"#,
        r#"{"name":+1}"#,
    ]
    .into_iter()
    .for_each(|content| {
        fs::write(&path, content).expect("write invalid JSONC fixture");
        assert!(matches!(read_jsonc::<Example>(&path), Err(ConfigError::JsoncParse { .. })));
    });
}
#[test]
fn test_read_jsonc_reports_deserialize_errors() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let deserialize_error = directory.path().join("deserialize.jsonc");
    fs::write(&deserialize_error, r#"{"name":1}"#).expect("write deserialize fixture");
    assert!(matches!(
        read_jsonc::<Example>(&deserialize_error),
        Err(ConfigError::JsoncDeserialize { .. })
    ));
}
#[test]
fn test_reads_json_jsonc_yaml_and_yml() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let fixtures = [
        ("config.json", r#"{"name":"json"}"#, "json"),
        ("config.jsonc", "{// comment\n\"name\":\"jsonc\",}\n", "jsonc"),
        ("config.yaml", "name: yaml\n", "yaml"),
        ("config.yml", "name: yml\n", "yml"),
    ];
    fixtures.into_iter().for_each(|(name, content, expected)| {
        let path = directory.path().join(name);
        fs::write(&path, content).expect("write fixture");
        assert_eq!(read::<Example>(&path).expect("read configuration").name, expected);
    });
}
#[test]
fn test_resolve_config_path_prefers_explicit_paths() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let default = directory.path().join("first.json");
    let existing = directory.path().join("explicit.jsonc");
    fs::write(&default, "{}").expect("write default fixture");
    fs::write(&existing, "{}").expect("write explicit fixture");
    let relative = Path::new("missing.jsonc");
    let absolute = directory.path().join("missing.yaml");
    assert_eq!(resolve_config_path(Some(&existing), directory.path(), &["first.json"]), Some(existing));
    assert_eq!(
        resolve_config_path(Some(relative), directory.path(), &["first.json"]),
        Some(relative.to_path_buf())
    );
    assert_eq!(resolve_config_path(Some(&absolute), directory.path(), &["first.json"]), Some(absolute));
}
#[test]
fn test_resolve_config_path_uses_first_regular_candidate() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let candidates = ["first.json", "second.jsonc", "third.yaml"];
    fs::create_dir(directory.path().join(candidates[0])).expect("create directory candidate");
    fs::write(directory.path().join(candidates[1]), "{}").expect("write second candidate");
    fs::write(directory.path().join(candidates[2]), "{}").expect("write third candidate");
    assert_eq!(
        resolve_config_path(None, directory.path(), &candidates),
        Some(directory.path().join(candidates[1]))
    );
}
#[test]
fn test_resolve_config_path_without_match_returns_none() {
    let directory = tempfile::tempdir().expect("temporary directory");
    assert_eq!(resolve_config_path(None, directory.path(), &[]), None);
    assert_eq!(resolve_config_path(None, directory.path(), &["missing.json"]), None);
}
