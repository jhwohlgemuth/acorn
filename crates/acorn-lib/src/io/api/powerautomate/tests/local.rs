//! Standalone local JSONL proposal tests.
use super::super::local::{self, ApplyStatus, StageSummary};
use acorn_schema::research_activity::ResearchActivity;
use local::apply;
use std::fs;
use std::path::PathBuf;

const ID: &str = "11111111-1111-4111-8111-111111111111";
const LINE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/automation/powerautomate/logbook.jsonl"
));

fn setup(destination: &str) -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join(".acorn.json");
    let inbox = directory.path().join("logbook.jsonl");
    let index = directory.path().join("index.json");
    fs::write(&index, serde_json::to_string(&ResearchActivity::default()).unwrap()).unwrap();
    fs::write(&inbox, LINE.lines().next().unwrap().to_string() + "\n").unwrap();
    let path = match destination {
        | "local" => format!("file://{}", index.display()),
        | "remote" => "https://example.org/index.json".to_string(),
        | _ => "index.json".to_string(),
    };
    let config_content = serde_json::json!({
        "schemaVersion":"2.0.0",
        "folders":[{"id":"acorn","members":["scientist@example.org"],"path":path}]
    });
    fs::write(&config, serde_json::to_vec(&config_content).unwrap()).unwrap();
    (directory, config, inbox, index)
}

#[test]
fn test_changed_base_requires_restage() {
    let (_directory, config, inbox, index) = setup("local");
    StageSummary::stage(&config, &inbox).unwrap();
    let original = fs::read_to_string(&index).unwrap();
    fs::write(&index, format!("{original}\n")).unwrap();
    assert!(apply(&config, ID).unwrap_err().to_string().contains("restage"));
    assert_eq!(StageSummary::stage(&config, &inbox).unwrap().staged, vec![ID]);
    assert_eq!(apply(&config, ID).unwrap(), ApplyStatus::Applied);
}
#[test]
fn test_invalid_line_does_not_block_valid_line_and_changed_uuid_bytes_conflict() {
    let (_directory, config, inbox, index) = setup("local");
    let original = fs::read_to_string(&index).unwrap();
    let valid = LINE.lines().next().unwrap();
    fs::write(&inbox, format!("{{}}\n{valid}\n")).unwrap();
    let staged = StageSummary::stage(&config, &inbox).unwrap();
    assert_eq!(staged.staged, vec![ID]);
    assert_eq!(staged.rejected.len(), 1);
    assert_eq!(staged.rejected.first().map(|rejection| rejection.line), Some(1));
    assert_eq!(fs::read_to_string(&index).unwrap(), original);
    fs::write(&inbox, format!(" {valid}\n")).unwrap();
    let conflict = StageSummary::stage(&config, &inbox).unwrap();
    assert!(conflict.staged.is_empty());
    assert!(conflict
        .rejected
        .first()
        .is_some_and(|rejection| rejection.reason.contains("different bytes")));
}
#[test]
fn test_invalid_proposal_fields_are_rejected() {
    let (_directory, config, inbox, _index) = setup("local");
    StageSummary::stage(&config, &inbox).unwrap();
    let path = config.parent().unwrap().join(".acorn/intake/proposals").join(format!("{ID}.json"));
    let mut proposal: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    proposal["source_line"] = serde_json::json!(0);
    proposal["base_digest"] = serde_json::json!("bad");
    fs::write(&path, serde_json::to_vec(&proposal).unwrap()).unwrap();
    assert!(apply(&config, ID).unwrap_err().to_string().contains("Invalid local proposal"));
}
#[test]
fn test_repeated_apply_recovers_after_index_replacement() {
    let (_directory, config, inbox, index) = setup("local");
    StageSummary::stage(&config, &inbox).unwrap();
    let path = config.parent().unwrap().join(".acorn/intake/proposals").join(format!("{ID}.json"));
    let proposal: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    fs::write(&index, proposal["proposed_content"].as_str().unwrap()).unwrap();
    assert_eq!(apply(&config, ID).unwrap(), ApplyStatus::AlreadyApplied);
    let applied: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(applied["applied"], true);
}
#[test]
fn test_route_change_blocks_apply() {
    let (_directory, config, inbox, _index) = setup("local");
    StageSummary::stage(&config, &inbox).unwrap();
    let changed = serde_json::json!({
        "schemaVersion":"2.0.0",
        "folders":[{"id":"acorn","members":["scientist@example.org"],"path":"https://example.org/index.json"}]
    });
    fs::write(&config, serde_json::to_vec(&changed).unwrap()).unwrap();
    assert!(apply(&config, ID).unwrap_err().to_string().contains("no local review adapter"));
}
#[test]
fn test_stage_requires_explicit_apply_and_double_apply_is_idempotent() {
    let (_directory, config, inbox, index) = setup("local");
    let original = fs::read_to_string(&index).unwrap();
    let staged = StageSummary::stage(&config, &inbox).unwrap();
    assert_eq!(staged.staged, vec![ID]);
    assert_eq!(staged.to_string(), "1 staged, 0 existing, 0 rejected");
    assert_eq!(fs::read_to_string(&index).unwrap(), original);
    let proposal = config.parent().unwrap().join(".acorn/intake/proposals").join(format!("{ID}.json"));
    assert!(proposal.is_file());
    assert_eq!(StageSummary::stage(&config, &inbox).unwrap().existing, vec![ID]);
    assert_eq!(apply(&config, ID).unwrap(), ApplyStatus::Applied);
    let updated = fs::read_to_string(&index).unwrap();
    assert_ne!(updated, original);
    assert_eq!(apply(&config, ID).unwrap(), ApplyStatus::AlreadyApplied);
    assert_eq!(fs::read_to_string(&index).unwrap(), updated);
    assert_eq!(StageSummary::stage(&config, &inbox).unwrap().existing, vec![ID]);
}
#[test]
fn test_unsupported_destination_is_rejected_without_mutation() {
    let (_directory, config, inbox, index) = setup("remote");
    let original = fs::read_to_string(&index).unwrap();
    let staged = StageSummary::stage(&config, &inbox).unwrap();
    assert!(staged.staged.is_empty());
    assert_eq!(staged.rejected.len(), 1);
    assert!(staged
        .rejected
        .first()
        .is_some_and(|rejection| rejection.reason.contains("no local review adapter")));
    assert_eq!(fs::read_to_string(&index).unwrap(), original);
}
