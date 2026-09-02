use acorn::io::write_file;
use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn test_gather_defaults_to_json_when_stdout_is_piped() {
    let output = Command::cargo_bin("acorn")
        .expect("acorn binary should resolve")
        .args(["--offline", "--no-local-database", "gather", "--text", "doi:10.1234/example"])
        .output()
        .expect("gather command should run");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("piped output should be JSON");
    assert_eq!(report["discoveries"][0]["identifier"], "10.1234/example");
}

#[test]
fn test_gather_mixed_load_failure_and_successful_discovery_returns_failure() {
    let directory = tempdir().expect("temporary directory should be created");
    let invalid = directory.path().join("invalid.pdf");
    write_file(invalid.clone(), "not a PDF".to_string()).expect("invalid PDF fixture should be written");
    let output = Command::cargo_bin("acorn")
        .expect("acorn binary should resolve")
        .args([
            "-v",
            "--offline",
            "--no-local-database",
            "gather",
            "--format",
            "json",
            "--text",
            "doi:10.1234/example",
            invalid.to_str().expect("temporary path should be UTF-8"),
        ])
        .output()
        .expect("gather command should run");
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("mixed result should still emit JSON");
    assert_eq!(report["discoveries"][0]["identifier"], "10.1234/example");
    assert_eq!(report["summary"]["failures"], 1);
}
