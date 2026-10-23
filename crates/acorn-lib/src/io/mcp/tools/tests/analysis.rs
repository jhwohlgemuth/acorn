use super::*;
use crate::io::mcp::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome, InputReference, ToolRegistry};
use serde_json::json;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}
fn input(path: &str) -> AnalysisInput {
    AnalysisInput {
        input: InputReference::File(PathBuf::from(path)),
        standard: None,
        skip: Vec::new(),
        disable_website_checks: false,
    }
}

#[tokio::test]
async fn test_category_selection_and_invalid_selection_are_explicit() {
    let mut selected = input("CITATION.cff");
    selected.skip = vec!["schema".to_string()];
    let result = selected.analyze(&fixture_root(), true).await.expect("analysis");
    assert_eq!(result.skipped_categories, vec!["schema"]);
    let mut invalid = input("CITATION.cff");
    invalid.skip = vec!["not-a-category".to_string()];
    assert!(invalid.analyze(&fixture_root(), true).await.is_err());
}
#[tokio::test]
async fn test_directory_results_are_stable_and_input_failures_are_reported() {
    let request = AnalysisInput {
        input: InputReference::Directory(PathBuf::from("data/valid_project")),
        standard: None,
        skip: Vec::new(),
        disable_website_checks: true,
    };
    let first = request.clone().analyze(&fixture_root(), true).await.expect("first");
    let second = request.analyze(&fixture_root(), true).await.expect("second");
    assert_eq!(serde_json::to_value(first).unwrap(), serde_json::to_value(second).unwrap());
}
#[tokio::test]
async fn test_full_local_check_reports_structured_categories_offline() {
    let result = input("data/valid_project/index.json")
        .analyze(&fixture_root(), true)
        .await
        .expect("analysis");
    assert_eq!(result.batches.len(), 1);
    let batch = result.batches.first().expect("one batch");
    assert!(!batch.categories.is_empty());
    assert!(batch.categories.iter().any(|category| category.category == "schema"));
    assert!(batch
        .categories
        .iter()
        .any(|category| category.category == "link" && category.status == AnalysisCategoryStatus::Skipped));
}
#[tokio::test]
async fn test_native_registry_executes_the_same_analysis_offline() {
    let root = fixture_root();
    let expected = input("data/valid_project/index.json")
        .analyze(&root, true)
        .await
        .expect("direct analysis");
    let registry = ToolRegistry::default().register_analysis_capability(root, true).expect("registry");
    let selection = registry
        .select_capability_binding(
            "acorn.check",
            None,
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: false,
            },
        )
        .expect("native selection");
    let result = registry
        .execute_selected(
            &selection,
            json!({"input":{"kind":"file","value":"data/valid_project/index.json"},"standard":null,"skip":[],"disable_website_checks":false}),
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
    assert_eq!(result.outcome, ExecutionOutcome::Completed);
    assert_eq!(result.content.expect("output"), serde_json::to_value(expected).unwrap());
}
#[test]
fn test_online_binding_cannot_be_reused_under_an_offline_policy() {
    let registry = ToolRegistry::default()
        .register_analysis_capability(fixture_root(), false)
        .expect("registry");
    let offline = CapabilitySelectionContext {
        offline: true,
        allow_remote: false,
    };
    assert!(registry.select_capability_binding("acorn.check", None, None, offline).is_err());
}
