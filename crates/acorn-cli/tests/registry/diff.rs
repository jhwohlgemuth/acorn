//! Direct diff command equivalence against the native routed comparison operation.
use acorn::io::mcp::{DiffOperand, DiffOperation, DiffOperationInput};
use assert_cmd::Command;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_direct_json_patch_matches_native_operation() {
    let root = tempdir().expect("temporary root");
    let original = include_str!("../../../../tests/fixtures/CITATION.cff");
    fs::write(root.path().join("before.cff"), original).expect("original file");
    fs::write(root.path().join("after.cff"), original.replace("title: ACORN", "title: Revised ACORN")).expect("revised file");
    let expected = DiffOperationInput {
        before: DiffOperand::File(PathBuf::from("before.cff")),
        after: DiffOperand::File(PathBuf::from("after.cff")),
        operation: DiffOperation::Semantic {
            json_patch: true,
            standard: None,
        },
    }
    .compare(root.path())
    .expect("native patch");
    let output = Command::cargo_bin("acorn")
        .expect("ACORN binary")
        .current_dir(root.path())
        .args(["diff", "before.cff", "after.cff", "--format", "json-patch"])
        .output()
        .expect("direct diff");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let direct: Value = serde_json::from_slice(&output.stdout).expect("direct JSON Patch");
    assert_eq!(direct, expected.result);
}
#[test]
fn test_direct_similarity_files_match_native_operation() {
    let root = tempdir().expect("temporary root");
    fs::write(root.path().join("left.txt"), "kitten").expect("left file");
    fs::write(root.path().join("right.txt"), "sitting").expect("right file");
    let request: DiffOperationInput = serde_json::from_value(json!({
        "before": {"kind": "file", "value": "left.txt"},
        "after": {"kind": "file", "value": "right.txt"},
        "operation": {"kind": "similarity", "method": "levenshtein"}
    }))
    .expect("native request");
    let expected = request.compare(root.path()).expect("native similarity");
    let output = Command::cargo_bin("acorn")
        .expect("ACORN binary")
        .current_dir(root.path())
        .args(["diff", "left.txt", "right.txt", "--method", "levenshtein", "--format", "json"])
        .output()
        .expect("direct file similarity");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let direct: Value = serde_json::from_slice(&output.stdout).expect("direct similarity JSON");
    assert_eq!(direct, expected.result);
}
#[test]
fn test_direct_similarity_values_match_native_operation() {
    let root = tempdir().expect("temporary root");
    let expected = DiffOperationInput {
        before: DiffOperand::Text("kitten".to_string()),
        after: DiffOperand::Text("sitting".to_string()),
        operation: DiffOperation::Similarity {
            method: "levenshtein".to_string(),
            cutoff: None,
            length: None,
            max_collection_items: None,
            max_input_scalars: None,
            max_work_units: None,
            ngram_width: None,
            preprocessing: None,
            threshold: None,
        },
    }
    .compare(root.path())
    .expect("native similarity");
    let output = Command::cargo_bin("acorn")
        .expect("ACORN binary")
        .current_dir(root.path())
        .args(["diff", "kitten", "sitting", "--values", "--method", "levenshtein", "--format", "json"])
        .output()
        .expect("direct similarity");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let direct: Value = serde_json::from_slice(&output.stdout).expect("direct similarity JSON");
    assert_eq!(direct, expected.result);
}
#[test]
fn test_direct_unified_diff_matches_native_operation() {
    let root = tempdir().expect("temporary root");
    let original = include_str!("../../../../tests/fixtures/CITATION.cff");
    fs::write(root.path().join("before.cff"), original).expect("original file");
    fs::write(root.path().join("after.cff"), original.replace("title: ACORN", "title: Revised ACORN")).expect("revised file");
    let expected = DiffOperationInput {
        before: DiffOperand::File(PathBuf::from("before.cff")),
        after: DiffOperand::File(PathBuf::from("after.cff")),
        operation: DiffOperation::Semantic {
            json_patch: false,
            standard: None,
        },
    }
    .compare(root.path())
    .expect("native unified diff");
    let output = Command::cargo_bin("acorn")
        .expect("ACORN binary")
        .current_dir(root.path())
        .args(["diff", "before.cff", "after.cff"])
        .output()
        .expect("direct unified diff");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 diff"),
        expected.result.as_str().expect("unified text")
    );
}
