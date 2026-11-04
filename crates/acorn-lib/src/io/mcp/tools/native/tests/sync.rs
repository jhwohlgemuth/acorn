use super::super::sync::*;
use crate::io::mcp::tools::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome, MutationAuthority};
use crate::io::mcp::ToolRegistry;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_sync_preview_and_apply_require_exact_target_authority() {
    let root = tempdir().expect("sync root");
    let model = root.path().join("models/acme/model/model-Q4_K_M.gguf");
    fs::create_dir_all(model.parent().expect("model parent")).expect("create model directory");
    fs::write(&model, b"model").expect("model fixture");
    let input = SyncInput {
        models: vec!["acme/model".to_string()],
        models_dir: PathBuf::from("models"),
        target: SyncTargetKind::OpenCode,
        destination: PathBuf::from("opencode.json"),
        overwrite: false,
        prune: false,
    };
    let preview = input.clone().preview(root.path()).expect("sync preview");
    assert!(preview.changed);
    assert!(preview.content.contains("acme/model"));
    assert!(preview.artifacts.is_empty());
    assert!(!root.path().join("opencode.json").exists());
    let registry = ToolRegistry::default()
        .register_sync_preview_capability(root.path())
        .expect("register preview")
        .register_sync_apply_capability(root.path())
        .expect("register apply");
    let selected_preview = registry
        .select_capability_binding(
            "acorn.sync::preview",
            None,
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: false,
            },
        )
        .expect("select offline preview");
    let arguments =
        json!({"models":["acme/model"],"models_dir":"models","target":"open-code","destination":"opencode.json","overwrite":false,"prune":false});
    let inspected = registry
        .execute_selected(
            &selected_preview,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(inspected.outcome, ExecutionOutcome::Completed, "{inspected:?}");
    assert!(!root.path().join("opencode.json").exists());
    let selected_apply = registry
        .select_capability_binding(
            "acorn.sync::apply",
            None,
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: false,
            },
        )
        .expect("select offline apply");
    let denied = registry
        .execute_selected(
            &selected_apply,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    let applied = registry
        .execute_selected(
            &selected_apply,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                selection: CapabilitySelectionContext {
                    offline: true,
                    allow_remote: false,
                },
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.sync::apply".to_string(),
                    binding_id: "native:acorn.sync::apply".to_string(),
                    destination: "opencode.json".to_string(),
                    arguments,
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(applied.outcome, ExecutionOutcome::Completed, "{applied:?}");
    assert_eq!(
        fs::read_to_string(root.path().join("opencode.json")).expect("target content"),
        preview.content
    );
    assert_eq!(fs::read(&model).expect("model preserved"), b"model");
    let unchanged = SyncInput {
        overwrite: true,
        ..input.clone()
    }
    .preview(root.path())
    .expect("unchanged preview");
    assert!(!unchanged.changed);
    assert!(SyncInput {
        destination: PathBuf::from("../escape.json"),
        ..input.clone()
    }
    .preview(root.path())
    .is_err());
    assert!(SyncInput {
        destination: PathBuf::from("models/target.json"),
        ..input.clone()
    }
    .preview(root.path())
    .is_err());
    assert!(SyncInput {
        destination: PathBuf::from("opencode.json"),
        ..input
    }
    .apply(root.path())
    .is_err());
    for (target, destination) in [
        (SyncTargetKind::Goose, "goose.yaml"),
        (SyncTargetKind::LlamaSwap, "llama-swap.yaml"),
        (SyncTargetKind::VsCode, "vscode.json"),
    ] {
        let request = SyncInput {
            models: vec!["acme/model".to_string()],
            models_dir: PathBuf::from("models"),
            target,
            destination: PathBuf::from(destination),
            overwrite: false,
            prune: false,
        };
        let output = request.preview(root.path()).expect("supported target preview");
        assert!(output.changed);
        assert!(!output.content.is_empty());
        assert!(!root.path().join(destination).exists());
    }
}
