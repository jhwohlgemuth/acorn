#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
//! Integration tests for semantic and similarity comparisons.
use assert_cmd::Command;
use serde_json::Value;
use std::fs::write;
use tempfile::tempdir;

fn acorn_diff() -> Command {
    let mut command = Command::cargo_bin("acorn").expect("ACORN binary");
    command.args(["--offline", "--no-local-database", "diff"]);
    command
}
#[test]
fn test_diff_dublin_core_compares_rdf_syntaxes_semantically() {
    let directory = tempdir().expect("temporary directory");
    let before = directory.path().join("before.ttl");
    let after = directory.path().join("after.jsonld");
    write(
        &before,
        "@prefix dcterms: <http://purl.org/dc/terms/> .\n<https://example.org/work> dcterms:title \"Example\" .\n",
    )
    .expect("write Turtle fixture");
    write(
        &after,
        r#"[{"@id":"https://example.org/work","http://purl.org/dc/terms/title":[{"@value":"Example"}]}]"#,
    )
    .expect("write JSON-LD fixture");
    let output = acorn_diff()
        .args([before.as_os_str(), after.as_os_str()])
        .args(["--standard", "dcmi"])
        .output()
        .expect("Dublin Core semantic diff output");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
}
#[test]
fn test_diff_semantic_default_remains_unified() {
    let directory = tempdir().expect("temporary directory");
    let before = directory.path().join("before.json");
    let after = directory.path().join("after.json");
    write(&before, "{\"name\":\"before\"}\n").expect("write before fixture");
    write(&after, "{\"name\":\"after\"}\n").expect("write after fixture");
    let output = acorn_diff()
        .args([before.as_os_str(), after.as_os_str()])
        .output()
        .expect("semantic diff output");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--- a/"));
    assert!(stdout.contains("+++ b/"));
    assert!(stdout.contains("-{\"name\":\"before\"}"));
    assert!(stdout.contains("+{\"name\":\"after\"}"));
}
#[test]
fn test_diff_similarity_file_error_fails() {
    let output = acorn_diff()
        .args(["missing-left.txt", "missing-right.txt", "--method", "exact"])
        .output()
        .expect("similarity failure output");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Diff input is not a regular file"));
}
#[test]
fn test_diff_similarity_file_text_preserves_whitespace() {
    let directory = tempdir().expect("temporary directory");
    let left = directory.path().join("left.txt");
    let right = directory.path().join("right.txt");
    write(&left, "acorn\n").expect("write left fixture");
    write(&right, "acorn").expect("write right fixture");
    let output = acorn_diff()
        .args([left.as_os_str(), right.as_os_str()])
        .args(["--method", "exact", "--format", "json"])
        .output()
        .expect("similarity output");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let result: Value = serde_json::from_slice(&output.stdout).expect("JSON similarity output");
    assert_eq!(result["score"], 0.0);
}
#[test]
fn test_diff_similarity_incompatible_format_fails() {
    let output = acorn_diff()
        .args(["left", "right", "--method", "exact", "--values", "--format", "diff"])
        .output()
        .expect("similarity failure output");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Similarity comparisons support --format text or json"));
}
#[test]
fn test_diff_similarity_invalid_cutoff_fails() {
    let output = acorn_diff()
        .args(["1", "2", "--method", "absolute-difference", "--cutoff", "0", "--values"])
        .output()
        .expect("similarity failure output");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Similarity cutoff must be finite and positive"));
}
#[test]
fn test_diff_similarity_json_reports_distance_and_score() {
    let output = acorn_diff()
        .args(["kitten", "sitting", "--method", "levenshtein", "--values", "--format", "json"])
        .output()
        .expect("similarity output");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let result: Value = serde_json::from_slice(&output.stdout).expect("JSON similarity output");
    assert_eq!(result["method"], "levenshtein");
    assert_eq!(result["distance"]["kind"], "edit-count");
    assert_eq!(result["distance"]["value"], 3);
    assert_eq!(result["score"], 4.0 / 7.0);
    assert_eq!(result["scoreSemantics"], "zero-to-one-higher-is-more-similar");
    assert_eq!(result["parameters"]["limits"]["maxInputScalars"], 4096);
}
#[test]
fn test_diff_similarity_later_methods_report_reproducible_parameters() {
    let output = acorn_diff()
        .args([
            "research data",
            "research datum",
            "--method",
            "soft-tfidf",
            "--threshold",
            "0.7",
            "--values",
            "--format",
            "json",
        ])
        .output()
        .expect("Soft TF-IDF output");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let result: Value = serde_json::from_slice(&output.stdout).expect("JSON similarity output");
    assert_eq!(result["method"], "soft-tfidf");
    assert_eq!(result["parameters"]["threshold"], 0.7);
    assert!(result["parameters"]["corpusIdentity"].as_str().is_some());
}
#[test]
fn test_diff_similarity_limit_failure_is_an_error() {
    let output = acorn_diff()
        .args(["acorn", "acorn", "--method", "exact", "--values", "--max-input-scalars", "4"])
        .output()
        .expect("similarity failure output");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("exceeding the limit of 4"));
}
#[test]
fn test_diff_similarity_low_score_succeeds() {
    let output = acorn_diff()
        .args(["left", "right", "--method", "exact", "--values"])
        .output()
        .expect("similarity output");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("score: 0"));
}
#[test]
fn test_diff_similarity_numeric_files_trim_outer_whitespace() {
    let directory = tempdir().expect("temporary directory");
    let left = directory.path().join("left.txt");
    let right = directory.path().join("right.txt");
    write(&left, " 10\n").expect("write left fixture");
    write(&right, "12\n").expect("write right fixture");
    let output = acorn_diff()
        .args([left.as_os_str(), right.as_os_str()])
        .args(["--method", "absolute-difference", "--cutoff", "4", "--format", "json"])
        .output()
        .expect("similarity output");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let result: Value = serde_json::from_slice(&output.stdout).expect("JSON similarity output");
    assert_eq!(result["distance"]["value"], 2.0);
    assert_eq!(result["score"], 0.5);
}
#[test]
fn test_diff_similarity_threshold_is_required_for_fuzzy_token_methods() {
    let output = acorn_diff()
        .args(["research", "research", "--method", "extended-jaccard", "--values"])
        .output()
        .expect("similarity failure output");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires --threshold"));
}
#[test]
fn test_diff_similarity_threshold_outside_unit_interval_is_an_error() {
    ["1.1", "NaN"].into_iter().for_each(|threshold| {
        let output = acorn_diff()
            .args(["research", "research", "--method", "soft-tfidf", "--threshold", threshold, "--values"])
            .output()
            .expect("similarity failure output");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("invalid value"));
    });
}
#[test]
fn test_diff_similarity_unicode_preprocessing() {
    [("Straße", "STRASSE", "case-fold"), ("Café", "Cafe\u{301}", "nfc"), ("①", "1", "nfkc")]
        .into_iter()
        .for_each(|(left, right, preprocessing)| {
            let output = acorn_diff()
                .args([
                    left,
                    right,
                    "--method",
                    "exact",
                    "--values",
                    "--preprocess",
                    preprocessing,
                    "--format",
                    "json",
                ])
                .output()
                .expect("Unicode preprocessing output");
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            let result: Value = serde_json::from_slice(&output.stdout).expect("JSON similarity output");
            assert_eq!(result["preprocessing"], preprocessing);
            assert_eq!(result["score"], 1.0);
        });
}
