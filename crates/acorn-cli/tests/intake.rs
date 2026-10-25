#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
//! Exercise explicit approval at the local intake command boundary.
use acorn_schema::research_activity::ResearchActivity;
use assert_cmd::Command;
use serde_json::{json, Value};
use std::fs::{read, read_dir, write};
use std::path::Path;
use std::process::Output;
use tempfile::tempdir;

fn intake(root: &Path, arguments: &[&str]) -> Output {
    Command::cargo_bin("acorn")
        .unwrap()
        .current_dir(root)
        .args(["--offline", "--no-local-database", "intake"])
        .args(arguments)
        .output()
        .unwrap()
}
fn stage(root: &Path, batch: bool) -> String {
    write(root.join("index.json"), serde_json::to_vec(&ResearchActivity::default()).unwrap()).unwrap();
    write(
        root.join("config.json"),
        json!({"schemaVersion":"2.0.0", "folders":[{
            "id":"acorn", "members":["scientist@example.org"],
            "path":format!("file://{}", root.join("index.json").display())
        }]})
        .to_string(),
    )
    .unwrap();
    let mut submission: Value = serde_json::from_str(
        include_str!("../../../tests/fixtures/automation/powerautomate/logbook.jsonl")
            .lines()
            .next()
            .unwrap(),
    )
    .unwrap();
    submission["entry"]["category"] = json!("personnel");
    submission["entry"]["summary"] = json!("Example, Avery");
    write(root.join("inbox.jsonl"), format!("{submission}\n")).unwrap();
    let mut arguments = vec!["stage", "--config", "config.json", "--source", "inbox.jsonl"];
    if batch {
        arguments.push("--batch");
    }
    let output = intake(root, &arguments);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    if batch {
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["batchId"]
            .as_str()
            .unwrap()
            .to_string()
    } else {
        submission["submissionId"].as_str().unwrap().to_string()
    }
}
#[test]
fn test_apply_requires_explicit_approval_when_redirected() {
    for batch in [false, true] {
        let directory = tempdir().unwrap();
        let root = directory.path();
        let id = stage(root, batch);
        let index = root.join("index.json");
        let before = read(&index).unwrap();
        let selector = if batch { "--batch-id" } else { "--submission-id" };
        let arguments = ["apply", "--config", "config.json", selector, &id];
        let declined = intake(root, &arguments);
        assert!(!declined.status.success());
        assert!(String::from_utf8_lossy(&declined.stderr).contains("use --yes"));
        assert_eq!(read(&index).unwrap(), before);
        let approved = [&arguments[..], &["--yes"]].concat();
        let mut changed: Value = serde_json::from_slice(&before).unwrap();
        changed["title"] = json!("Changed after review");
        let changed = serde_json::to_vec(&changed).unwrap();
        write(&index, &changed).unwrap();
        assert!(!intake(root, &approved).status.success(), "--yes must preserve stale index checks");
        assert_eq!(read(&index).unwrap(), changed);
        write(&index, &before).unwrap();
        let output = intake(root, &approved);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let after = read(&index).unwrap();
        assert_ne!(after, before);
        assert_eq!(
            serde_json::from_slice::<Value>(&after).unwrap()["logbook"]["entries"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(intake(root, &approved).status.success());
        assert_eq!(read(&index).unwrap(), after);
    }
}
#[test]
fn test_graduation_requires_reviewed_selection_before_backup_or_replacement() {
    let directory = tempdir().unwrap();
    let root = directory.path();
    let id = stage(root, true);
    assert!(intake(root, &["apply", "--config", "config.json", "--batch-id", &id, "--yes"])
        .status
        .success());
    let output = intake(root, &["run", "--index", "index.json", "--dry-run"]);
    assert!(output.status.success());
    let preview: Value = serde_json::from_slice(&output.stdout).unwrap();
    let arguments = [
        "run",
        "--index",
        "index.json",
        "--accept",
        preview["candidates"][0]["entryIdentifier"].as_str().unwrap(),
        "--expected-digest",
        preview["baseDigest"].as_str().unwrap(),
    ];
    let index = root.join("index.json");
    let before = read(&index).unwrap();
    let files_before = read_dir(root).unwrap().count();
    assert_eq!(read(&index).unwrap(), before);
    assert_eq!(read_dir(root).unwrap().count(), files_before);
    let output = intake(root, &arguments);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(read(receipt["backup"].as_str().unwrap()).unwrap(), before);
    let after = read(&index).unwrap();
    let graduated: Value = serde_json::from_slice(&after).unwrap();
    assert_eq!(graduated["meta"]["contributors"][0]["name"], "Example, Avery");
    assert_eq!(graduated["logbook"]["entries"][0]["archived"], true);
    assert!(
        !intake(root, &arguments).status.success(),
        "reviewed selection must reject a stale digest"
    );
    assert_eq!(read(&index).unwrap(), after);
}
#[test]
fn test_run_dry_run_force_and_reviewed_selection() {
    for force in [false, true] {
        let directory = tempdir().unwrap();
        let root = directory.path();
        let id = stage(root, true);
        assert!(intake(root, &["apply", "--config", "config.json", "--batch-id", &id, "--yes"])
            .status
            .success());
        let index = root.join("index.json");
        let before = read(&index).unwrap();
        let count = read_dir(root).unwrap().count();
        let preview = intake(root, &["run", "--index", "index.json", "--dry-run", "--force"]);
        assert!(preview.status.success(), "{}", String::from_utf8_lossy(&preview.stderr));
        let preview: Value = serde_json::from_slice(&preview.stdout).unwrap();
        assert_eq!(preview["status"], "dry-run");
        assert_eq!(read(&index).unwrap(), before);
        assert_eq!(read_dir(root).unwrap().count(), count);
        let no_approval = intake(root, &["run", "--index", "index.json"]);
        assert!(!no_approval.status.success());
        assert!(String::from_utf8_lossy(&no_approval.stderr).contains("Graduation requires approval"));
        let interactive = intake(root, &["run", "--index", "index.json", "--interactive", "--dry-run"]);
        assert!(!interactive.status.success());
        assert!(String::from_utf8_lossy(&interactive.stderr).contains("--interactive requires terminal"));
        assert_eq!(read(&index).unwrap(), before);
        let approved = if force {
            vec![
                "run",
                "--index",
                "index.json",
                "--force",
                "--expected-digest",
                preview["baseDigest"].as_str().unwrap(),
            ]
        } else {
            vec![
                "run",
                "--index",
                "index.json",
                "--accept",
                preview["candidates"][0]["entryIdentifier"].as_str().unwrap(),
                "--expected-digest",
                preview["baseDigest"].as_str().unwrap(),
            ]
        };
        let result = intake(root, &approved);
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let receipt: Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(receipt["status"], "graduated");
        assert_eq!(read(receipt["backup"].as_str().unwrap()).unwrap(), before);
        let after = read(&index).unwrap();
        assert_ne!(after, before);
        assert!(!intake(root, &approved).status.success());
        let replay = intake(root, &["run", "--index", "index.json", "--force"]);
        assert!(replay.status.success());
        assert_eq!(serde_json::from_slice::<Value>(&replay.stdout).unwrap()["status"], "unchanged");
        assert_eq!(read(&index).unwrap(), after);
    }
}
#[test]
fn test_run_gitlab_polling_respects_offline_before_configuration() {
    let directory = tempdir().unwrap();
    let result = intake(directory.path(), &["run", "--config", "missing.json"]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("GitLab intake polling is unavailable with --offline"));
    assert_eq!(read_dir(directory.path()).unwrap().count(), 0);
}
#[test]
fn test_stage_source_accepts_file_uri() {
    let directory = tempdir().unwrap();
    let root = directory.path();
    let expected = stage(root, true);
    let uri = format!("file://{}/inbox.jsonl", root.display());
    for source in ["inbox.jsonl", uri.as_str()] {
        let output = intake(root, &["stage", "--config", "config.json", "--source", source, "--batch"]);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(serde_json::from_slice::<Value>(&output.stdout).unwrap()["batchId"], expected);
    }
    let remote = intake(
        root,
        &[
            "stage",
            "--config",
            "config.json",
            "--source",
            "https://code.ornl.gov/group/repo/-/raw/main/logbook.jsonl",
            "--batch",
        ],
    );
    assert!(!remote.status.success());
    assert!(String::from_utf8_lossy(&remote.stderr).contains("Remote intake sources are unavailable with --offline"));
}
