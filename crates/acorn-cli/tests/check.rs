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

fn normalized_process(args: &[String]) -> String {
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .env("TERM", "dumb")
        .args(args)
        .output()
        .unwrap();
    let normalize = |bytes: &[u8]| {
        let text = String::from_utf8_lossy(bytes).replace('\r', "").replace('\\', "/");
        let workspace = fixture("").parent().unwrap().parent().unwrap().to_string_lossy().replace('\\', "/");
        let without_workspace = text.replace(workspace.as_str(), "<WORKSPACE>");
        let mut escaped = false;
        without_workspace
            .chars()
            .filter(|character| {
                if escaped {
                    if character.is_ascii_alphabetic() {
                        escaped = false;
                    }
                    false
                } else if *character == '\u{1b}' {
                    escaped = true;
                    false
                } else {
                    true
                }
            })
            .collect::<String>()
    };
    format!(
        "status: {}\nstdout:\n{}stderr:\n{}",
        output.status.code().map_or_else(|| "signal".to_string(), |code| code.to_string()),
        normalize(&output.stdout),
        normalize(&output.stderr)
    )
}

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

#[test]
fn test_check_validation_compatibility_snapshots() {
    macro_rules! args {
        ($($value:expr),+ $(,)?) => {
            vec![$($value.to_string()),+]
        };
    }
    let fixtures = [
        (
            "check_valid_acorn",
            args![
                "--offline",
                "check",
                fixture("data/valid_project/index.json").to_string_lossy(),
                "--no-fail",
                "--all",
                "--skip",
                "link,prose,readability,quality",
            ],
        ),
        (
            "check_invalid_acorn",
            args![
                "--offline",
                "check",
                fixture("data/invalid_project_a/index.json").to_string_lossy(),
                "--skip",
                "link,prose,readability,quality",
            ],
        ),
        (
            "check_invalid_acorn_no_fail",
            args![
                "--offline",
                "check",
                fixture("data/invalid_project_a/index.json").to_string_lossy(),
                "--no-fail",
                "--skip",
                "link,prose,readability,quality",
            ],
        ),
        (
            "check_invalid_acorn_terse_raw",
            args![
                "--offline",
                "check",
                fixture("data/invalid_project_a/index.json").to_string_lossy(),
                "--no-fail",
                "--terse",
                "--raw",
                "--skip",
                "link,prose,readability,quality",
            ],
        ),
        (
            "check_malformed_acorn_failure",
            args![
                "--offline",
                "check",
                fixture("malformed-rad.json").to_string_lossy(),
                "--skip",
                "link,prose,readability,quality",
            ],
        ),
        (
            "check_malformed_acorn",
            args![
                "--offline",
                "check",
                fixture("malformed-rad.json").to_string_lossy(),
                "--no-fail",
                "--skip",
                "link,prose,readability,quality",
            ],
        ),
        (
            "check_invalid_datacite",
            args![
                "--offline",
                "check",
                fixture("schema/datacite-invalid-2-errors.json").to_string_lossy(),
                "--standard",
                "datacite",
                "--no-fail",
                "--skip",
                "link,prose,readability,quality",
            ],
        ),
        (
            "check_invalid_dcat",
            args![
                "--offline",
                "check",
                fixture("schema/dcat-invalid-2-errors.json").to_string_lossy(),
                "--standard",
                "dcat",
                "--no-fail",
                "--skip",
                "link,prose,readability,quality",
            ],
        ),
        (
            "check_invalid_huwise",
            args![
                "--offline",
                "check",
                fixture("schema/huwise-invalid-2-errors.json").to_string_lossy(),
                "--standard",
                "huwise",
                "--no-fail",
                "--skip",
                "link,prose,readability,quality",
            ],
        ),
        (
            "check_invalid_invenio",
            args![
                "--offline",
                "check",
                fixture("schema/invenio-invalid-2-errors.json").to_string_lossy(),
                "--standard",
                "invenio",
                "--no-fail",
                "--skip",
                "link,prose,readability,quality",
            ],
        ),
    ];
    fixtures
        .into_iter()
        .for_each(|(name, args)| insta::assert_snapshot!(name, normalized_process(&args)));
}

#[test]
fn test_check_pdf_as_document_text() {
    Command::cargo_bin("acorn")
        .unwrap()
        .args([
            "--offline",
            "check",
            &fixture("gather/resume.pdf").to_string_lossy(),
            "--skip",
            "prose,readability,schema",
            "--all",
        ])
        .assert()
        .success();
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
