use super::super::model::ModelDownloadInputExt;
use crate::test::server::spawn_file_server;
use acorn::io::mcp::ModelDownloadInput;
use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_configured_local_models_outside_root_are_rejected() {
    let root = tempdir().expect("routed model root");
    let outside = tempdir().expect("outside model root");
    let weights = outside.path().join("weights.gguf");
    fs::write(&weights, b"outside model").expect("outside weights");
    let list = root.path().join("models.txt");
    fs::write(&list, weights.display().to_string()).expect("model list");
    let config = root.path().join("acorn.json");
    fs::write(&config, format!(r#"{{"models":["{}"]}}"#, weights.display())).expect("model configuration");
    for (config, model_file) in [(Some(config), None), (None, Some(list.display().to_string()))] {
        let input = ModelDownloadInput {
            models: Vec::new(),
            config,
            model_file,
            destination: PathBuf::from("native"),
            copy: true,
            symlink: false,
            filter: Vec::new(),
            ignore: Vec::new(),
            whitelist: Vec::new(),
            overwrite: false,
        };
        for preview in [true, false] {
            let result = input
                .clone()
                .execute_routed(root.path().to_path_buf(), root.path().join("native"), preview, true)
                .await;
            assert!(result.is_err(), "outside model was accepted for preview={preview}");
            assert!(!root.path().join("native").exists());
        }
    }
    assert_eq!(fs::read(weights).expect("outside source unchanged"), b"outside model");
}
#[tokio::test]
async fn test_local_model_preview_and_copy_match_direct_command() {
    let root = tempdir().expect("model comparison root");
    let source = root.path().join("weights.gguf");
    let source_arg = source.to_str().expect("model source path");
    fs::write(&source, b"local model fixture").expect("local model");
    let preview = Command::cargo_bin("acorn")
        .expect("resolve acorn binary")
        .current_dir(root.path())
        .args([
            "--offline",
            "--no-local-database",
            "download",
            "model",
            source_arg,
            "--copy",
            "--dry-run",
            "--raw",
        ])
        .output()
        .expect("direct model preview");
    assert!(preview.status.success(), "{}", String::from_utf8_lossy(&preview.stderr));
    let input = ModelDownloadInput {
        models: vec![source.display().to_string()],
        config: None,
        model_file: None,
        destination: PathBuf::from("native"),
        copy: true,
        symlink: false,
        filter: Vec::new(),
        ignore: Vec::new(),
        whitelist: Vec::new(),
        overwrite: false,
    };
    let native_preview = input
        .clone()
        .execute_routed(root.path().to_path_buf(), root.path().join("native-preview"), true, true)
        .await
        .expect("native model preview");
    assert_eq!(native_preview.model_ids.len(), 1);
    assert!(native_preview.artifacts.is_empty());
    assert!(!root.path().join("native-preview").exists());
    let direct = Command::cargo_bin("acorn")
        .expect("resolve acorn binary")
        .current_dir(root.path())
        .args([
            "--offline",
            "--no-local-database",
            "download",
            "model",
            source_arg,
            "--copy",
            "--local-dir",
            "direct",
        ])
        .output()
        .expect("direct model copy");
    assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
    let native = input
        .execute_routed(root.path().to_path_buf(), root.path().join("native"), false, true)
        .await
        .expect("native model copy");
    assert_eq!(native.model_ids.len(), 1);
    assert_eq!(native.artifacts.len(), 1);
    assert_eq!(
        fs::read(root.path().join("direct/weights/weights.gguf")).expect("direct copied weights"),
        fs::read(root.path().join("native/weights/weights.gguf")).expect("native copied weights")
    );
    assert_eq!(fs::read(source).expect("source preserved"), b"local model fixture");
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mocked_model_url_matches_direct_preview_and_acquisition() {
    let root = tempdir().expect("mocked model root");
    let (direct_url, direct_server) = spawn_file_server("model.gguf", b"mock model weights".to_vec());
    let direct_preview = Command::cargo_bin("acorn")
        .expect("resolve acorn binary")
        .current_dir(root.path())
        .args(["--no-local-database", "download", "model", &direct_url, "--dry-run", "--raw"])
        .output()
        .expect("direct remote model preview");
    assert!(direct_preview.status.success(), "{}", String::from_utf8_lossy(&direct_preview.stderr));
    let input = ModelDownloadInput {
        models: Vec::new(),
        config: None,
        model_file: None,
        destination: PathBuf::from("native"),
        copy: false,
        symlink: false,
        filter: Vec::new(),
        ignore: Vec::new(),
        whitelist: Vec::new(),
        overwrite: false,
    };
    let (native_url, native_server) = spawn_file_server("model.gguf", b"mock model weights".to_vec());
    let native_input = ModelDownloadInput {
        models: vec![native_url],
        ..input.clone()
    };
    let native_preview = native_input
        .clone()
        .execute_routed(root.path().to_path_buf(), root.path().join("native-preview"), true, false)
        .await
        .expect("native remote model preview");
    assert_eq!(native_preview.model_ids.len(), 1);
    assert!(!root.path().join("native-preview").exists());
    let direct = Command::cargo_bin("acorn")
        .expect("resolve acorn binary")
        .current_dir(root.path())
        .args(["--no-local-database", "download", "model", &direct_url, "--local-dir", "direct"])
        .output()
        .expect("direct remote model acquisition");
    assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
    direct_server.join().expect("direct model server");
    let native = native_input
        .execute_routed(root.path().to_path_buf(), root.path().join("native"), false, false)
        .await
        .expect("native remote model acquisition");
    assert_eq!(native.model_ids.len(), 1);
    assert_eq!(
        fs::read(root.path().join("direct/model/model.gguf")).expect("direct weights"),
        fs::read(root.path().join("native/model/model.gguf")).expect("native weights")
    );
    native_server.join().expect("native model server");
}
#[tokio::test]
async fn test_offline_remote_model_preview_matches_direct_rejection() {
    let root = tempdir().expect("offline model root");
    let direct = Command::cargo_bin("acorn")
        .expect("resolve acorn binary")
        .current_dir(root.path())
        .args([
            "--offline",
            "--no-local-database",
            "download",
            "model",
            "https://example.invalid/model.gguf",
            "--dry-run",
        ])
        .output()
        .expect("direct offline model preview");
    assert!(!direct.status.success());
    let native = ModelDownloadInput {
        models: vec!["https://example.invalid/model.gguf".to_string()],
        config: None,
        model_file: None,
        destination: PathBuf::from("native"),
        copy: false,
        symlink: false,
        filter: Vec::new(),
        ignore: Vec::new(),
        whitelist: Vec::new(),
        overwrite: false,
    }
    .execute_routed(root.path().to_path_buf(), root.path().join("native"), true, true)
    .await;
    assert!(native.is_err());
    assert!(!root.path().join("native").exists());
}
