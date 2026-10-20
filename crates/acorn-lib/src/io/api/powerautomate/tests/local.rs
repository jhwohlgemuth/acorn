//! Standalone local JSONL proposal tests.
use super::super::local::{
    self, ApplyStatus, BatchApplyStatus, BatchInspection, BatchProjectApplyStatus, BatchProjectState, BatchSummary, StageSummary,
};
use crate::io::config::ApplicationConfiguration;
use crate::io::ApiResult;
use acorn_schema::research_activity::ResearchActivity;
use std::fs;
use std::path::{Path, PathBuf};

const ID: &str = "11111111-1111-4111-8111-111111111111";
const LINE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/automation/powerautomate/logbook.jsonl"
));
fn apply(config_path: &Path, submission_id: &str) -> ApiResult<ApplyStatus> {
    ApplicationConfiguration::load_local_intake(config_path).and_then(|(configuration, path)| configuration.apply(&path, submission_id))
}
fn apply_batch(config_path: &Path, batch_id: &str) -> ApiResult<local::BatchApplySummary> {
    ApplicationConfiguration::load_local_intake(config_path).and_then(|(configuration, path)| configuration.apply_batch(&path, batch_id))
}
fn batch_setup() -> (tempfile::TempDir, PathBuf, PathBuf, Vec<PathBuf>) {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join(".acorn.json");
    let inbox = directory.path().join("batch.jsonl");
    let indexes = ["alpha", "beta", "gamma"]
        .into_iter()
        .map(|id| {
            let path = directory.path().join(format!("{id}.json"));
            fs::write(&path, serde_json::to_vec(&ResearchActivity::default()).unwrap()).unwrap();
            path
        })
        .collect::<Vec<_>>();
    let folders = indexes
        .iter()
        .zip(["alpha", "beta", "gamma"])
        .map(|(path, id)| {
            serde_json::json!({
                "id": id,
                "aliases": if id == "alpha" { vec!["Alpha Project"] } else { vec![] },
                "members": ["scientist@example.org"],
                "path": format!("file://{}", path.display())
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        &config,
        serde_json::to_vec(&serde_json::json!({"schemaVersion":"2.0.0", "folders": folders})).unwrap(),
    )
    .unwrap();
    fs::write(
        &inbox,
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/automation/powerautomate/local-batch.jsonl"
        )),
    )
    .unwrap();
    (directory, config, inbox, indexes)
}
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
fn stage(config_path: &Path, inbox_path: &Path) -> ApiResult<StageSummary> {
    ApplicationConfiguration::load_local_intake(config_path).and_then(|(configuration, path)| configuration.stage(&path, inbox_path))
}

#[test]
fn test_batch_applies_three_projects_and_folds_interleaved_lines() {
    let (_directory, config, inbox, indexes) = batch_setup();
    let originals = indexes.iter().map(|path| fs::read_to_string(path).unwrap()).collect::<Vec<_>>();
    let staged = BatchSummary::stage(&config, &inbox).unwrap();
    assert!(staged.can_apply);
    assert_eq!((staged.accepted, staged.replayed, staged.rejected), (4, 0, 0));
    let inspection = BatchInspection::inspect(&config, &staged.batch_id).unwrap();
    assert_eq!(inspection.projects.len(), 3);
    assert_eq!(inspection.lines[0].project_id.as_deref(), Some("alpha"));
    assert_eq!(
        indexes.iter().map(|path| fs::read_to_string(path).unwrap()).collect::<Vec<_>>(),
        originals
    );
    let applied = apply_batch(&config, &staged.batch_id).unwrap();
    assert_eq!(applied.status, BatchApplyStatus::Complete);
    assert_eq!(
        applied
            .projects
            .iter()
            .filter(|project| project.status == BatchProjectApplyStatus::Applied)
            .count(),
        3
    );
    let counts = indexes
        .iter()
        .map(|path| {
            serde_json::from_str::<ResearchActivity>(&fs::read_to_string(path).unwrap())
                .unwrap()
                .logbook
                .unwrap()
                .entries
                .len()
        })
        .collect::<Vec<_>>();
    assert_eq!(counts, vec![2, 1, 1]);
    assert_eq!(
        apply_batch(&config, &staged.batch_id)
            .unwrap()
            .projects
            .iter()
            .filter(|project| project.status == BatchProjectApplyStatus::AlreadyApplied)
            .count(),
        3
    );
    assert_eq!(BatchSummary::stage(&config, &inbox).unwrap().batch_id, staged.batch_id);
    fs::write(
        &inbox,
        format!("{}{}", fs::read_to_string(&inbox).unwrap(), fs::read_to_string(&inbox).unwrap()),
    )
    .unwrap();
    let replay = BatchSummary::stage(&config, &inbox).unwrap();
    assert_eq!((replay.accepted, replay.replayed), (0, 8));
}
#[test]
fn test_batch_changed_uuid_bytes_and_invalid_line_prevent_all_writes() {
    let (_directory, config, inbox, indexes) = batch_setup();
    let originals = indexes.iter().map(|path| fs::read_to_string(path).unwrap()).collect::<Vec<_>>();
    let first = fs::read_to_string(&inbox).unwrap();
    let mut changed = serde_json::from_str::<serde_json::Value>(first.lines().next().unwrap()).unwrap();
    changed["entry"]["summary"] = serde_json::json!("Conflicting update");
    fs::write(&inbox, format!("{}{}\n{{}}\n", first, serde_json::to_string(&changed).unwrap())).unwrap();
    let staged = BatchSummary::stage(&config, &inbox).unwrap();
    assert!(!staged.can_apply);
    assert_eq!(staged.rejected, 2);
    assert!(apply_batch(&config, &staged.batch_id).is_err());
    assert_eq!(
        indexes.iter().map(|path| fs::read_to_string(path).unwrap()).collect::<Vec<_>>(),
        originals
    );
}
#[cfg(unix)]
#[test]
fn test_batch_manifest_is_owner_readable() {
    use std::os::unix::fs::PermissionsExt;
    let (_directory, config, inbox, _indexes) = batch_setup();
    let staged = BatchSummary::stage(&config, &inbox).unwrap();
    let directory = config.parent().unwrap().join(".acorn/intake/batches");
    let manifest = directory.join(format!("{}.json", staged.batch_id));
    assert_eq!(fs::metadata(directory).unwrap().permissions().mode() & 0o777, 0o700);
    assert_eq!(fs::metadata(manifest).unwrap().permissions().mode() & 0o777, 0o600);
}
#[test]
fn test_batch_preflight_stops_before_first_write_on_stale_index_or_source() {
    let (_directory, config, inbox, indexes) = batch_setup();
    let staged = BatchSummary::stage(&config, &inbox).unwrap();
    let original_alpha = fs::read_to_string(&indexes[0]).unwrap();
    fs::write(&indexes[2], "{}\n").unwrap();
    assert!(apply_batch(&config, &staged.batch_id).is_err());
    assert_eq!(fs::read_to_string(&indexes[0]).unwrap(), original_alpha);
    fs::write(&indexes[2], &original_alpha).unwrap();
    fs::write(&inbox, format!("{}{{}}\n", fs::read_to_string(&inbox).unwrap())).unwrap();
    assert!(apply_batch(&config, &staged.batch_id).is_err());
    assert_eq!(fs::read_to_string(&indexes[0]).unwrap(), original_alpha);
}
#[test]
fn test_batch_recovers_index_replacement_and_rejects_altered_receipt() {
    let (_directory, config, inbox, indexes) = batch_setup();
    let staged = BatchSummary::stage(&config, &inbox).unwrap();
    let manifest_path = config
        .parent()
        .unwrap()
        .join(".acorn/intake/batches")
        .join(format!("{}.json", staged.batch_id));
    let mut manifest = serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    fs::write(&indexes[0], manifest["projects"][0]["proposed_content"].as_str().unwrap()).unwrap();
    let inspected = BatchInspection::inspect(&config, &staged.batch_id).unwrap();
    assert_eq!(inspected.projects[0].state, BatchProjectState::Final);
    let resumed = apply_batch(&config, &staged.batch_id).unwrap();
    assert_eq!(resumed.status, BatchApplyStatus::Complete);
    assert_eq!(resumed.projects[0].status, BatchProjectApplyStatus::AlreadyApplied);
    manifest["projects"][0]["final_digest"] = serde_json::json!("sha256:changed");
    fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert!(apply_batch(&config, &staged.batch_id).is_err());
}
#[test]
fn test_batch_rejected_uuid_can_be_corrected_in_later_snapshot() {
    let (_directory, config, inbox, _indexes) = batch_setup();
    let valid = fs::read_to_string(&inbox).unwrap().lines().next().unwrap().to_string();
    let mut invalid = serde_json::from_str::<serde_json::Value>(&valid).unwrap();
    invalid["entry"]["summary"] = serde_json::json!("");
    fs::write(&inbox, format!("{}\n", serde_json::to_string(&invalid).unwrap())).unwrap();
    let rejected = BatchSummary::stage(&config, &inbox).unwrap();
    assert_eq!(rejected.rejected, 1);
    fs::write(&inbox, format!("{valid}\n")).unwrap();
    let corrected = BatchSummary::stage(&config, &inbox).unwrap();
    assert_eq!((corrected.accepted, corrected.rejected), (1, 0));
}
#[test]
fn test_batch_rejects_changed_route_and_competing_index_writer() {
    let (_directory, config, inbox, indexes) = batch_setup();
    let staged = BatchSummary::stage(&config, &inbox).unwrap();
    let original_alpha = fs::read_to_string(&indexes[0]).unwrap();
    let lock_path = indexes[0].with_extension("json.acorn-intake.lock");
    let lock = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .unwrap();
    lock.try_lock().unwrap();
    assert!(apply_batch(&config, &staged.batch_id).is_err());
    assert_eq!(fs::read_to_string(&indexes[0]).unwrap(), original_alpha);
    drop(lock);
    let mut changed = serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&config).unwrap()).unwrap();
    changed["folders"][0]["path"] = serde_json::json!("https://example.org/index.json");
    fs::write(&config, serde_json::to_vec(&changed).unwrap()).unwrap();
    assert!(apply_batch(&config, &staged.batch_id).is_err());
    assert_eq!(fs::read_to_string(&indexes[0]).unwrap(), original_alpha);
}
#[test]
fn test_batch_replays_applied_single_proposal_by_exact_bytes() {
    let (_directory, config, inbox, index) = setup("local");
    stage(&config, &inbox).unwrap();
    assert_eq!(apply(&config, ID).unwrap(), ApplyStatus::Applied);
    let final_content = fs::read_to_string(&index).unwrap();
    let staged = BatchSummary::stage(&config, &inbox).unwrap();
    assert_eq!((staged.accepted, staged.replayed, staged.rejected), (0, 1, 0));
    assert!(BatchInspection::inspect(&config, &staged.batch_id).unwrap().projects.is_empty());
    assert_eq!(apply_batch(&config, &staged.batch_id).unwrap().status, BatchApplyStatus::Complete);
    assert_eq!(fs::read_to_string(&index).unwrap(), final_content);
    fs::write(&inbox, format!(" {}", fs::read_to_string(&inbox).unwrap())).unwrap();
    let changed = BatchSummary::stage(&config, &inbox).unwrap();
    assert_eq!(changed.rejected, 1);
}
#[test]
fn test_changed_base_requires_restage() {
    let (_directory, config, inbox, index) = setup("local");
    stage(&config, &inbox).unwrap();
    let original = fs::read_to_string(&index).unwrap();
    fs::write(&index, format!("{original}\n")).unwrap();
    assert!(apply(&config, ID).unwrap_err().to_string().contains("restage"));
    assert_eq!(stage(&config, &inbox).unwrap().staged, vec![ID]);
    assert_eq!(apply(&config, ID).unwrap(), ApplyStatus::Applied);
}
#[test]
fn test_invalid_line_does_not_block_valid_line_and_changed_uuid_bytes_conflict() {
    let (_directory, config, inbox, index) = setup("local");
    let original = fs::read_to_string(&index).unwrap();
    let valid = LINE.lines().next().unwrap();
    fs::write(&inbox, format!("{{}}\n{valid}\n")).unwrap();
    let staged = stage(&config, &inbox).unwrap();
    assert_eq!(staged.staged, vec![ID]);
    assert_eq!(staged.rejected.len(), 1);
    assert_eq!(staged.rejected.first().map(|rejection| rejection.line), Some(1));
    assert_eq!(fs::read_to_string(&index).unwrap(), original);
    fs::write(&inbox, format!(" {valid}\n")).unwrap();
    let conflict = stage(&config, &inbox).unwrap();
    assert!(conflict.staged.is_empty());
    assert!(conflict
        .rejected
        .first()
        .is_some_and(|rejection| rejection.reason.contains("different bytes")));
}
#[test]
fn test_invalid_proposal_fields_are_rejected() {
    let (_directory, config, inbox, _index) = setup("local");
    stage(&config, &inbox).unwrap();
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
    stage(&config, &inbox).unwrap();
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
    stage(&config, &inbox).unwrap();
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
    let staged = stage(&config, &inbox).unwrap();
    assert_eq!(staged.staged, vec![ID]);
    assert_eq!(staged.to_string(), "1 staged, 0 existing, 0 rejected");
    assert_eq!(fs::read_to_string(&index).unwrap(), original);
    let proposal = config.parent().unwrap().join(".acorn/intake/proposals").join(format!("{ID}.json"));
    assert!(proposal.is_file());
    assert_eq!(stage(&config, &inbox).unwrap().existing, vec![ID]);
    assert_eq!(apply(&config, ID).unwrap(), ApplyStatus::Applied);
    let updated = fs::read_to_string(&index).unwrap();
    assert_ne!(updated, original);
    assert_eq!(apply(&config, ID).unwrap(), ApplyStatus::AlreadyApplied);
    assert_eq!(fs::read_to_string(&index).unwrap(), updated);
    assert_eq!(stage(&config, &inbox).unwrap().existing, vec![ID]);
}
#[test]
fn test_unsupported_destination_is_rejected_without_mutation() {
    let (_directory, config, inbox, index) = setup("remote");
    let original = fs::read_to_string(&index).unwrap();
    let staged = stage(&config, &inbox).unwrap();
    assert!(staged.staged.is_empty());
    assert_eq!(staged.rejected.len(), 1);
    assert!(staged
        .rejected
        .first()
        .is_some_and(|rejection| rejection.reason.contains("no local review adapter")));
    assert_eq!(fs::read_to_string(&index).unwrap(), original);
}
