use super::super::link::*;
use crate::io::mcp::tools::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome, MutationAuthority};
use crate::io::mcp::{InputReference, ToolRegistry};
use crate::io::InputOutput;
use acorn_schema::research_activity::ResearchActivity;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_link_preview_and_exact_apply_match_shared_projection() {
    let root = tempdir().expect("temporary linked-data root");
    let source = root.path().join("index.json");
    fs::copy(crate::test::utils::fixture_path("data/valid_project/index.json"), &source).expect("copy activity fixture");
    let original = fs::read_to_string(&source).expect("read source");
    let registry = ToolRegistry::default()
        .register_link_capabilities(root.path())
        .expect("register link operations");
    let preview = registry
        .select_capability_binding("acorn.link::preview", None, None, CapabilitySelectionContext::default())
        .expect("select preview");
    let input = json!({"input":{"kind":"file","value":"index.json"}});
    let result = registry
        .execute_selected(
            &preview,
            input.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::Completed);
    assert!(!source.with_extension("jsonld").exists());
    let expected = ResearchActivity::read(source.clone())
        .expect("read activity")
        .format_with(Some(source.clone()))
        .linked_content()
        .expect("linked projection")
        .expect("linked activity");
    let projection = LinkPreviewInput {
        input: InputReference::File(PathBuf::from("index.json")),
    }
    .preview(root.path())
    .expect("preview");
    assert_eq!(projection.content, expected);
    let repeated = LinkPreviewInput {
        input: InputReference::File(PathBuf::from("index.json")),
    }
    .preview(root.path())
    .expect("repeat preview");
    assert_eq!(repeated.content, projection.content);
    let arguments = json!({"input":input,"destination":"index.jsonld","overwrite":false});
    let selected = registry
        .select_capability_binding("acorn.link::apply", None, None, CapabilitySelectionContext::default())
        .expect("select apply");
    let denied = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    assert!(!source.with_extension("jsonld").exists());
    let applied = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.link::apply".to_string(),
                    binding_id: "native:acorn.link::apply".to_string(),
                    arguments: arguments.clone(),
                    destination: "index.jsonld".to_string(),
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(applied.outcome, ExecutionOutcome::Completed);
    assert_eq!(applied.artifacts.len(), 1);
    assert_eq!(
        applied.artifacts.first().expect("linked artifact").uri,
        source
            .canonicalize()
            .expect("canonical source")
            .with_extension("jsonld")
            .display()
            .to_string()
    );
    assert_eq!(fs::read_to_string(source.with_extension("jsonld")).expect("linked artifact"), expected);
    assert_eq!(fs::read_to_string(&source).expect("unchanged source"), original);
    let repeated_apply = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.link::apply".to_string(),
                    binding_id: "native:acorn.link::apply".to_string(),
                    arguments,
                    destination: "index.jsonld".to_string(),
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(repeated_apply.outcome, ExecutionOutcome::ToolFailed);
    assert_eq!(
        fs::read_to_string(source.with_extension("jsonld")).expect("unchanged linked artifact"),
        expected
    );
}
#[test]
fn test_link_preview_rejects_invalid_and_outside_inputs() {
    let root = tempdir().expect("temporary linked-data root");
    fs::write(root.path().join("invalid.json"), "{}").expect("write invalid fixture");
    assert!(LinkPreviewInput {
        input: InputReference::File(PathBuf::from("invalid.json"))
    }
    .preview(root.path())
    .is_err());
    assert!(LinkPreviewInput {
        input: InputReference::File(PathBuf::from("../outside.json"))
    }
    .preview(root.path())
    .is_err());
    assert!(LinkPreviewInput {
        input: InputReference::Text("{}".to_string())
    }
    .preview(root.path())
    .is_err());
}
