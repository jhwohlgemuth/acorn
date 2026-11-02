use super::*;
use crate::io::mcp::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome};
use acorn_core::similarity::SimilarityMethod;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_registered_diff_matches_direct_result_without_effects() {
    let root = tempdir().expect("temporary root");
    let arguments = json!({
        "before": {"kind": "text", "value": "kitten"},
        "after": {"kind": "text", "value": "sitting"},
        "operation": {"kind": "similarity", "method": "levenshtein"}
    });
    let direct: DiffOperationInput = serde_json::from_value(arguments.clone()).expect("typed input");
    let expected = serde_json::to_value(direct.compare(root.path()).expect("direct comparison")).expect("output JSON");
    let registry = ToolRegistry::default()
        .register_diff_capability(root.path().to_path_buf())
        .expect("registered diff");
    let selected = registry
        .select_capability_binding(
            "acorn.diff",
            None,
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: false,
            },
        )
        .expect("selection");
    let routed = registry
        .execute_selected(
            &selected,
            arguments,
            ExecutionContext {
                grounded: true,
                selection: CapabilitySelectionContext {
                    offline: true,
                    allow_remote: false,
                },
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(routed.outcome, ExecutionOutcome::Completed);
    assert_eq!(routed.content, Some(expected));
}
#[test]
fn test_semantic_patch_and_unified_diff_share_direct_operation() {
    let root = tempdir().expect("temporary root");
    let original = include_str!("../../../../../../../tests/fixtures/CITATION.cff");
    let revised = original.replace("title: ACORN", "title: Revised ACORN");
    fs::write(root.path().join("before.cff"), original).expect("original file");
    fs::write(root.path().join("after.cff"), &revised).expect("revised file");
    let before = DiffOperand::File(PathBuf::from("before.cff"));
    let after = DiffOperand::File(PathBuf::from("after.cff"));
    let patch = DiffOperationInput {
        before: before.clone(),
        after: after.clone(),
        operation: DiffOperation::Semantic {
            json_patch: true,
            standard: None,
        },
    }
    .compare(root.path())
    .expect("patch");
    assert_eq!(patch.format, "json-patch");
    assert!(patch.result.as_array().is_some_and(|operations| !operations.is_empty()));
    let unified = DiffOperationInput {
        before,
        after,
        operation: DiffOperation::Semantic {
            json_patch: false,
            standard: None,
        },
    }
    .compare(root.path())
    .expect("unified diff");
    assert_eq!(unified.format, "unified");
    assert!(unified.result.as_str().is_some_and(|text| text.contains("Revised ACORN")));
}
#[test]
fn test_similarity_literals_and_files_have_matching_scores_and_limits() {
    let root = tempdir().expect("temporary root");
    fs::write(root.path().join("left.txt"), "kitten").expect("left file");
    fs::write(root.path().join("right.txt"), "sitting").expect("right file");
    let operation = DiffOperation::Similarity {
        method: "levenshtein".to_string(),
        cutoff: None,
        length: None,
        max_collection_items: None,
        max_input_scalars: None,
        max_work_units: None,
        ngram_width: None,
        preprocessing: None,
        threshold: None,
    };
    let literal = DiffOperationInput {
        before: DiffOperand::Text("kitten".to_string()),
        after: DiffOperand::Text("sitting".to_string()),
        operation: operation.clone(),
    }
    .compare(root.path())
    .expect("literal comparison");
    let file = DiffOperationInput {
        before: DiffOperand::File(PathBuf::from("left.txt")),
        after: DiffOperand::File(PathBuf::from("right.txt")),
        operation,
    }
    .compare(root.path())
    .expect("file comparison");
    assert_eq!(literal.result, file.result);
    assert_eq!(literal.result.get("score"), Some(&json!(0.5714285714285714)));
    let limited = DiffOperationInput {
        before: DiffOperand::Text("kitten".to_string()),
        after: DiffOperand::Text("sitting".to_string()),
        operation: DiffOperation::Similarity {
            method: "levenshtein".to_string(),
            cutoff: None,
            length: None,
            max_collection_items: None,
            max_input_scalars: Some(3),
            max_work_units: None,
            ngram_width: None,
            preprocessing: None,
            threshold: None,
        },
    };
    assert!(limited.compare(root.path()).is_err());
    assert!(matches!(SimilarityMethod::try_from("osa"), Ok(SimilarityMethod::OptimalStringAlignment)));
    assert!(matches!(SimilarityMethod::try_from("dice"), Ok(SimilarityMethod::SorensenDice)));
}
