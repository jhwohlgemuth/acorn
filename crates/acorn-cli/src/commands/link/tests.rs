#![cfg(feature = "exec")]
use crate::test::util::fixture_path;
use acorn::io::InputOutput;
use acorn_schema::research_activity::ResearchActivity;
use assert_cmd::Command;
use std::fs::{copy, create_dir, read_to_string, write};
use tempfile::tempdir;

#[test]
fn test_configured_link_gate_prevents_every_write() {
    let temporary = tempdir().expect("temporary project");
    let input = temporary.path().join("index.json");
    copy(fixture_path("data/valid_project/index.json"), &input).expect("copy fixture");
    let original = read_to_string(&input).expect("read fixture");
    create_dir(temporary.path().join(".acorn")).expect("script directory");
    write(
        temporary.path().join(".acorn/link.rhai"),
        "// @acorn-exec-v0.2.0\nfn evaluate(context) { #{findings: [#{rule_id: \"link-gate\", severity: \"error\", message: \"blocked\"}]} }",
    )
    .expect("script");
    let config = temporary.path().join(".acorn.json");
    write(&config, r#"{"scripts":[{"path":"link","hooks":["link"]}]}"#).expect("configuration");
    let output = Command::cargo_bin("acorn")
        .expect("resolve acorn binary")
        .args([
            "--no-local-database",
            "link",
            input.to_str().expect("input path"),
            "--config",
            config.to_str().expect("config path"),
        ])
        .output()
        .expect("run configured link hook");
    assert!(!output.status.success(), "error finding must block link");
    assert_eq!(read_to_string(&input).expect("read unchanged source"), original);
    assert!(!input.with_extension("jsonld").exists());
}
#[test]
fn test_link_matches_shared_workflow_projection_without_rewriting_source() {
    let temporary = tempdir().expect("temporary project");
    let input = temporary.path().join("index.json");
    copy(fixture_path("data/valid_project/index.json"), &input).expect("copy fixture");
    let original = read_to_string(&input).expect("read fixture");
    let expected = ResearchActivity::read(input.clone())
        .expect("fixture should deserialize")
        .format_with(Some(input.clone()))
        .linked_content()
        .expect("linked projection should succeed")
        .expect("research activity should produce linked content");
    let output = Command::cargo_bin("acorn")
        .expect("resolve acorn binary")
        .args(["--no-local-database", "link", input.to_str().expect("input path")])
        .output()
        .expect("run link command");
    assert!(
        output.status.success(),
        "link command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(read_to_string(&input).expect("read unchanged source"), original);
    let linked_path = input.with_extension("jsonld");
    assert!(linked_path.exists());
    let actual = read_to_string(linked_path).expect("read linked artifact");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&actual).expect("linked artifact should be JSON"),
        serde_json::from_str::<serde_json::Value>(&expected).expect("shared projection should be JSON")
    );
}
