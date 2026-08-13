#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
//! Integration tests for `acorn check`
use acorn::prelude::PathBuf;
use assert_cmd::Command;

fn dcat_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("schema")
        .join("dcat")
        .join(name)
}
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn test_check_diagnostics_remain_on_stdout() {
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .args([
            "--offline",
            "check",
            &fixture("malformed-rad.json").to_string_lossy(),
            "--no-fail",
            "--skip",
            "prose,readability",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Deserialization error"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Deserialization error"));
}

#[ignore = "Broken for reasons unknown"]
#[test]
fn test_dcat_check_valid_dataset() {
    Command::cargo_bin("acorn")
        .unwrap()
        .args(["check", "--standard", "dcat", &dcat_fixture("dcat-us-dataset.json").to_string_lossy()])
        .assert()
        .success();
}
#[ignore = "Broken for reasons unknown"]
#[test]
fn test_dcat_check_valid_catalog() {
    Command::cargo_bin("acorn")
        .unwrap()
        .args(["check", "--standard", "dcat", &dcat_fixture("dcat-us-catalog.json").to_string_lossy()])
        .assert()
        .success();
}
#[ignore = "Broken for reasons unknown"]
#[test]
fn test_dcat_check_invalid_dataset() {
    Command::cargo_bin("acorn")
        .unwrap()
        .args([
            "check",
            "--standard",
            "dcat",
            &dcat_fixture("dcat-invalid-2-errors.json").to_string_lossy(),
        ])
        .assert()
        .success();
}
