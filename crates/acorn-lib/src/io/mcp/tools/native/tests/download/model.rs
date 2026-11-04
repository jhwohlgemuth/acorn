use super::*;

#[test]
fn test_model_list_resolves_relative_to_registered_root() {
    let root = tempdir().expect("model list root");
    fs::write(root.path().join("models.txt"), "acme/tiny-gguf\n").expect("model list");
    let input = ModelDownloadInput {
        models: Vec::new(),
        config: None,
        model_file: Some("models.txt".to_string()),
        destination: PathBuf::from("models"),
        copy: false,
        symlink: false,
        filter: Vec::new(),
        ignore: Vec::new(),
        whitelist: Vec::new(),
        overwrite: false,
    };
    let (validated, _) = input.clone().validate(root.path(), false).expect("root-relative list");
    assert_eq!(
        validated.model_file,
        Some(
            root.path()
                .join("models.txt")
                .canonicalize()
                .expect("canonical list")
                .display()
                .to_string()
        )
    );
    assert!(ModelDownloadInput {
        model_file: Some("missing.txt".to_string()),
        ..input
    }
    .validate(root.path(), false)
    .is_err());
}
#[tokio::test]
async fn test_model_preview_and_acquisition_require_exact_authority() {
    let root = tempdir().expect("model root");
    fs::write(root.path().join("weights.gguf"), b"local weights").expect("local model");
    let original = fs::read(root.path().join("weights.gguf")).expect("model bytes");
    let registry = ToolRegistry::default()
        .register_download_model_capabilities(root.path(), true, |input, _root, destination, preview| async move {
            if !preview {
                fs::create_dir_all(&destination).expect("model output directory");
                fs::write(destination.join("weights.gguf"), b"local weights").expect("mock acquisition");
            }
            Ok(ModelDownloadOutput {
                model_ids: input.models,
                artifacts: match preview {
                    | true => Vec::new(),
                    | false => vec![destination.display().to_string()],
                },
                destination,
            })
        })
        .expect("register model routes");
    let selection = CapabilitySelectionContext {
        offline: true,
        allow_remote: false,
    };
    let preview = registry
        .select_capability_binding("acorn.download::model::preview", None, None, selection)
        .expect("select preview");
    let acquisition = registry
        .select_capability_binding("acorn.download::model", None, None, selection)
        .expect("select acquisition");
    let arguments = json!({"models":["weights.gguf"],"destination":"models","copy":true});
    let planned = registry
        .execute_selected(
            &preview,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(planned.outcome, ExecutionOutcome::Completed, "{planned:?}");
    assert!(planned.artifacts.is_empty());
    assert!(!root.path().join("models").exists());
    let denied = registry
        .execute_selected(
            &acquisition,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    assert!(!root.path().join("models").exists());
    let applied = registry
        .execute_selected(
            &acquisition,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.download::model".to_string(),
                    binding_id: "native:acorn.download::model".to_string(),
                    arguments,
                    destination: "models".to_string(),
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(applied.outcome, ExecutionOutcome::Completed, "{applied:?}");
    assert_eq!(applied.artifacts.len(), 1);
    assert_eq!(fs::read(root.path().join("models/weights.gguf")).expect("acquired model"), original);
    assert_eq!(fs::read(root.path().join("weights.gguf")).expect("source preserved"), original);
    let cancelled_arguments = json!({"models":["weights.gguf"],"destination":"cancelled","copy":true});
    let cancelled = registry
        .execute_selected(
            &acquisition,
            cancelled_arguments.clone(),
            ExecutionContext {
                grounded: true,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.download::model".to_string(),
                    binding_id: "native:acorn.download::model".to_string(),
                    arguments: cancelled_arguments,
                    destination: "cancelled".to_string(),
                }),
                cancellation: Some(Box::pin(async {})),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(cancelled.outcome, ExecutionOutcome::Cancelled);
    assert!(!root.path().join("cancelled").exists());
    assert!(ModelDownloadInput {
        models: vec!["weights.gguf".to_string()],
        config: None,
        model_file: None,
        destination: PathBuf::from("models"),
        copy: true,
        symlink: false,
        filter: Vec::new(),
        ignore: Vec::new(),
        whitelist: Vec::new(),
        overwrite: false,
    }
    .validate(root.path(), true)
    .is_err());
    assert!(ModelDownloadInput {
        models: vec!["../outside.gguf".to_string()],
        config: None,
        model_file: None,
        destination: PathBuf::from("other"),
        copy: true,
        symlink: false,
        filter: Vec::new(),
        ignore: Vec::new(),
        whitelist: Vec::new(),
        overwrite: false,
    }
    .validate(root.path(), true)
    .is_err());
}
