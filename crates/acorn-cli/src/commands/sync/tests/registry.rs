use super::super::*;
use acorn::io::mcp::{SyncInput, SyncTargetKind};
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_opencode_sync_preview_and_apply_match_direct_command() {
    let root = tempdir().expect("sync root");
    let config = root.path().join("acorn.json");
    fs::write(&config, br#"{"models":["acme/model"]}"#).expect("ACORN model config");
    let models_dir = root.path().join("models");
    let model = models_dir.join("acme/model/model-Q4_K_M.gguf");
    fs::create_dir_all(model.parent().expect("model parent")).expect("model directory");
    fs::write(&model, b"model").expect("GGUF fixture");
    let direct_path = root.path().join("direct.json");
    let args = arguments::Sync {
        config: Some(config),
        model_file: None,
        force: true,
        assume_models: false,
        no_fallback: true,
        opencode: true,
        llama_swap: false,
        vscode: false,
        goose: false,
        mcp: false,
        models_dir: Some(models_dir),
        opencode_config: Some(direct_path.clone()),
        llama_swap_config: None,
        vscode_config: None,
        vscode_mcp_config: None,
        goose_config: None,
        dry_run: true,
        no_color: true,
        prune: false,
        verbose: clap_verbosity_flag::Verbosity::new(0, 0),
    };
    run(&args, &None, true, true).await.expect("direct dry-run");
    assert!(!direct_path.exists());
    let native = SyncInput {
        models: vec!["acme/model".to_string()],
        models_dir: PathBuf::from("models"),
        target: SyncTargetKind::OpenCode,
        destination: PathBuf::from("native.json"),
        overwrite: false,
        prune: false,
    };
    let preview = native.clone().preview(root.path()).expect("native preview");
    assert!(preview.changed);
    assert!(!root.path().join("native.json").exists());
    run(&arguments::Sync { dry_run: false, ..args }, &None, true, true)
        .await
        .expect("direct apply");
    let applied = native.apply(root.path()).expect("native apply");
    assert_eq!(fs::read_to_string(direct_path).expect("direct config"), applied.content);
    assert_eq!(fs::read_to_string(applied.destination).expect("native config"), applied.content);
    assert_eq!(fs::read(&model).expect("model preserved"), b"model");
}
