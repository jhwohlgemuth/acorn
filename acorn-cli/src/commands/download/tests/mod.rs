//! Tests for the download command
//!
//! These tests validate local-bucket download execution with silent verbosity.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
extern crate alloc;
use super::run;
use crate::cli::arguments::OutputFormat;
use crate::test::server::{spawn_ignored_range_server, spawn_no_range_server, spawn_range_server, spawn_retry_resume_server, spawn_sidecar_server};
use crate::test::util::{fixture_path, has_any_file, temp_test_dir, TestCleanup};
use acorn::io::api::huggingface::{FileSelectionPolicy, HuggingFaceError, HuggingFaceRepository, HuggingFaceRepositoryFile};
use acorn::io::database::schema::{ActivityRow, Table};
use acorn::io::database::{Database, Row};
use acorn::io::download::{DownloadItem, DownloadTask};
use acorn::io::{config::ApplicationConfiguration, config::AuthenticationRequirement, config::ModelEntry, SourceAction};
use acorn::prelude::{create_dir_all, env, read_to_string, write};
use acorn::util::merge;
use alloc::sync::Arc;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;
use std::io::Write;
use std::sync::Mutex;
use temp_env::with_var;
use tracing_subscriber::fmt::MakeWriter;

const OFFLINE: bool = false;
const TEST_THREADS: usize = 2;

fn candidate_file(path: &str) -> HuggingFaceRepositoryFile {
    HuggingFaceRepositoryFile {
        path: path.to_string(),
        size: None,
    }
}

#[test]
fn test_config_parses_hugging_face_location_model_entry() {
    let content = r#"{"models": [{"name": "tiny-bert", "source": {"provider": "huggingface", "location": {"scheme": "https", "uri": "https://huggingface.co/hf-internal-testing/tiny-random-bert", "revision": "main"}}}]}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    assert_eq!(models.len(), 1);
}
#[test]
fn test_config_parses_model_download_options() {
    let content = r#"{
        "models": [{
            "name": "tiny-bert",
            "source": {"provider": "huggingface", "location": "https://huggingface.co/hf-internal-testing/tiny-random-bert"},
            "revision": "refs/pr/1",
            "auth": "required",
            "filter": ["Q4_K_M.*\\.gguf$"],
            "ignore": ["Q2_", "Q3_"]
        }]
    }"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    match models.first().unwrap() {
        | ModelEntry::Entry(entry) => {
            assert_eq!(entry.revision.as_deref(), Some("refs/pr/1"));
            assert_eq!(entry.auth, Some(AuthenticationRequirement::Required));
            assert_eq!(entry.filter.as_ref().unwrap(), &["Q4_K_M.*\\.gguf$".to_string()]);
            assert_eq!(entry.ignore.as_ref().unwrap(), &["Q2_".to_string(), "Q3_".to_string()]);
        }
        | ModelEntry::Selector(_) => panic!("expected detailed model entry"),
    }
}
#[test]
fn test_config_parses_local_model_entry() {
    let content = r#"{"models": [{"name": "qwen-local", "source": {"provider": "git", "location": "file:./models/qwen.gguf"}}]}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    assert_eq!(models.len(), 1);
}
#[test]
fn test_config_parses_mixed_model_entries() {
    let content =
        r#"{"models": ["meta-llama/Llama-2-7b-hf", {"name": "qwen-local", "source": {"provider": "git", "location": "file:./models/qwen.gguf"}}]}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    assert_eq!(models.len(), 2);
}
#[test]
fn test_config_parses_models_field_empty() {
    let content = r#"{"buckets": []}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    assert!(config.models.is_none());
}
#[test]
fn test_config_parses_models_field_json() {
    let content = r#"{"models": ["meta-llama/Llama-2-7b-hf", "microsoft/phi-2"]}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    assert_eq!(models.len(), 2);
}
#[test]
fn test_config_parses_models_field_yaml() {
    let content = r#"
models:
  - meta-llama/Llama-2-7b-hf
  - microsoft/phi-2
"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    assert_eq!(models.len(), 2);
}
#[tokio::test]
async fn test_download_local_config_silent() -> Result<()> {
    let temp_dir = temp_test_dir("download_local_config_silent");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("output");
    create_dir_all(&output_dir)?;
    let config_path = temp_dir.join("config.json");
    let bucket_path = fixture_path("data/bucket");
    let bucket_uri = format!("file:{}", bucket_path.display()).replace('\\', "/");
    let config = read_to_string(fixture_path("config/with_local_buckets.json"))?.replace("file:./tests/fixtures/data/bucket/", &bucket_uri);
    write(&config_path, config)?;
    let filter: Vec<String> = vec![];
    let ignore: Vec<String> = vec![];
    let verbose = Verbosity::new(0, 1);
    let result = run(
        &Some(config_path),
        &[],
        &filter,
        &ignore,
        &Some(output_dir.clone()),
        &None,
        TEST_THREADS,
        &verbose,
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "download from local config should succeed");
    assert!(has_any_file(&output_dir), "output should contain copied files");
    Ok(())
}
#[tokio::test]
async fn test_download_local_url_silent() -> Result<()> {
    let temp_dir = temp_test_dir("download_local_url_silent");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("output");
    create_dir_all(&output_dir)?;
    let url = vec!["file:./tests/fixtures/data/bucket/".to_string()];
    let filter: Vec<String> = vec![];
    let ignore: Vec<String> = vec![];
    let verbose = Verbosity::new(0, 1);
    let result = run(
        &None,
        &url,
        &filter,
        &ignore,
        &Some(output_dir.clone()),
        &None,
        TEST_THREADS,
        &verbose,
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "download from local URL should succeed");
    assert!(output_dir.exists(), "output directory should exist");
    Ok(())
}
#[tokio::test]
async fn test_download_model_config_resolves_models() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_config_resolve");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    write(
        &config_path,
        format!(r#"{{"models": ["{}"]}}"#, model_file.display().to_string().replace('\\', "/")),
    )?;
    let verbose = Verbosity::new(0, 1);
    let output_dir = temp_dir.join("models");
    let result = super::model::run(
        &[],
        &[],
        &[],
        &Some(config_path),
        &Some(output_dir),
        &None,
        &verbose,
        false,
        true,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "config-only model resolution should succeed");
    Ok(())
}
#[tokio::test]
async fn test_download_model_local_config_copy() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_local_config_copy");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    let config_path = temp_dir.join(".acorn.json");
    write(&model_file, b"fake model data")?;
    write(
        &config_path,
        format!(
            r#"{{"models": [{{"name": "tiny", "copy": true, "source": {{"provider": "git", "location": "file:{}"}}}}]}}"#,
            model_file.display().to_string().replace('\\', "/")
        ),
    )?;
    let verbose = Verbosity::new(0, 1);
    let result = super::model::run(
        &[],
        &[],
        &[],
        &Some(config_path),
        &Some(output_dir.clone()),
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "config copy mode should succeed");
    assert!(
        output_dir.join("tiny").join("tiny.gguf").exists(),
        "config copy mode should place files in output dir"
    );
    Ok(())
}
#[tokio::test]
async fn test_download_model_local_file_copy() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_local_copy");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    let verbose = Verbosity::new(0, 1);
    let result = super::model::run(
        &[model_file.display().to_string()],
        &[],
        &[],
        &None,
        &Some(output_dir.clone()),
        &None,
        &verbose,
        false,
        true, // copy
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "local file copy should succeed");
    let model_dir = output_dir.join("tiny");
    assert!(model_dir.join("tiny.gguf").exists(), "copy mode should place files in output dir");
    Ok(())
}
#[tokio::test]
async fn test_download_model_filter_keeps_matching_local_file() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_filter_local");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    let verbose = Verbosity::new(0, 1);
    let result = super::model::run(
        &[model_file.display().to_string()],
        &["tiny\\.gguf$".to_string()],
        &[],
        &None,
        &Some(output_dir.clone()),
        &None,
        &verbose,
        false,
        true,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "matching filter should keep local model");
    assert!(output_dir.join("tiny").join("tiny.gguf").exists());
    Ok(())
}
#[tokio::test]
async fn test_download_model_ignore_removes_matching_local_file() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_ignore_local");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    let verbose = Verbosity::new(0, 1);
    let result = super::model::run(
        &[model_file.display().to_string()],
        &[],
        &["tiny\\.gguf$".to_string()],
        &None,
        &Some(output_dir),
        &None,
        &verbose,
        false,
        true,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "ignore should remove the only selected model");
    Ok(())
}
#[test]
fn test_model_select_preferred_file_prefers_marker() {
    let policy = FileSelectionPolicy {
        preferred_marker: "Q4_K_M",
        no_match_message: "No GGUF model files found",
    };
    let files = vec![candidate_file("model.Q5_K_M.gguf"), candidate_file("model.Q4_K_M.gguf")];
    let selected = files.select(&policy).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected.first().map(|file| file.path.as_str()), Some("model.Q4_K_M.gguf"));
}
#[test]
fn test_model_select_preferred_file_errors_when_ambiguous() {
    let policy = FileSelectionPolicy {
        preferred_marker: "Q4_K_M",
        no_match_message: "No GGUF model files found",
    };
    let files = vec![candidate_file("model.Q5_K_M.gguf"), candidate_file("model.Q6_K.gguf")];
    let selected = files.select(&policy);
    assert!(selected.is_err());
}
#[tokio::test]
async fn test_model_resolve_plan_matches_config_name() {
    let content = r#"{"models": [{"name": "tiny", "revision": "dev", "filter": ["\\.gguf$"], "source": {"provider": "huggingface", "location": "https://huggingface.co/acme/tiny"}}]}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let plans = super::model::resolve_plans(&["tiny".to_string()], &config.models.unwrap(), None, &None, false)
        .await
        .unwrap();
    assert_eq!(plans.len(), 1);
    assert_eq!(plans.first().unwrap().name(), "tiny");
    assert_eq!(plans.first().unwrap().revision, "dev");
    assert_eq!(plans.first().unwrap().filter, vec!["\\.gguf$".to_string()]);
}
#[test]
fn test_model_merge_patterns_keeps_config_and_cli_patterns() {
    let merged = merge(&["Q4".to_string()], Some(&["\\.gguf$".to_string(), "Q4".to_string()]));
    assert_eq!(merged, vec!["Q4".to_string(), "\\.gguf$".to_string()]);
}
#[tokio::test]
async fn test_download_model_invalid_filter_errors() -> Result<()> {
    let verbose = Verbosity::new(0, 1);
    let result = super::model::run(
        &["model.gguf".to_string()],
        &["[".to_string()],
        &[],
        &None,
        &None,
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "invalid filter regex should fail");
    Ok(())
}
#[tokio::test]
async fn test_download_model_local_file_reference() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_local_reference");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    let verbose = Verbosity::new(0, 1);
    let result = super::model::run(
        &[model_file.display().to_string()],
        &[],
        &[],
        &None,
        &Some(output_dir.clone()),
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "local file reference should succeed");
    // Reference mode: no copy into output dir
    let model_dir = output_dir.join("tiny");
    assert!(!model_dir.join("tiny.gguf").exists(), "reference mode should not copy files");
    Ok(())
}
#[tokio::test]
async fn test_download_model_local_missing_path_errors() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_local_missing");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let verbose = Verbosity::new(0, 1);
    let result = super::model::run(
        &["./nonexistent/model.gguf".to_string()],
        &[],
        &[],
        &None,
        &Some(output_dir),
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "missing local model path should return an error");
    Ok(())
}
#[tokio::test]
async fn test_download_model_local_mixed_offline_errors() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_mixed_offline");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    let verbose = Verbosity::new(0, 1);
    // Local + remote in offline mode: should fail because remote is present
    let result = super::model::run(
        &[model_file.display().to_string(), "some-remote-model".to_string()],
        &[],
        &[],
        &None,
        &Some(output_dir),
        &None,
        &verbose,
        true, // offline
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "mixed local+remote in offline mode should fail");
    Ok(())
}
#[tokio::test]
async fn test_download_model_local_offline_succeeds() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_local_offline");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    let verbose = Verbosity::new(0, 1);
    let result = super::model::run(
        &[model_file.display().to_string()],
        &[],
        &[],
        &None,
        &Some(output_dir),
        &None,
        &verbose,
        true, // offline — local should still work
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "local model should work in offline mode");
    Ok(())
}
#[tokio::test]
async fn test_download_model_no_models_errors() -> Result<()> {
    let verbose = Verbosity::new(0, 1);
    let result = super::model::run(
        &[],
        &[],
        &[],
        &None,
        &None,
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "empty model list should return an error");
    Ok(())
}
#[tokio::test]
async fn test_download_model_offline_rejects() -> Result<()> {
    let verbose = Verbosity::new(0, 1);
    let result = super::model::run(
        &["some-model".to_string()],
        &[],
        &[],
        &None,
        &None,
        &None,
        &verbose,
        true,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "offline mode should reject remote model download");
    Ok(())
}
#[tokio::test]
async fn test_download_model_positional_overrides_config() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_positional_override");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    let model_file = temp_dir.join("tiny.gguf");
    let model_path = model_file.display().to_string();
    write(&model_file, b"fake model data")?;
    write(&config_path, format!(r#"{{"models": ["{}"]}}"#, model_path.replace('\\', "/")))?;
    let verbose = Verbosity::new(0, 1);
    let output_dir = temp_dir.join("models");
    let result = super::model::run(
        &[model_path],
        &[],
        &[],
        &Some(config_path),
        &Some(output_dir),
        &None,
        &verbose,
        false,
        true,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "positional + config with same model should succeed (deduplicated)");
    Ok(())
}
#[tokio::test]
async fn test_download_model_whitelist_errors_when_no_name_matches() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_whitelist_no_match");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    write(
        &config_path,
        r#"{"models": [{"name": "keep", "source": {"provider": "huggingface", "location": "https://huggingface.co/hf-internal-testing/tiny-random-bert"}}]}"#,
    )?;
    let verbose = Verbosity::new(0, 1);
    let output_dir = temp_dir.join("models");
    let result = super::model::run(
        &[],
        &[],
        &[],
        &Some(config_path),
        &Some(output_dir),
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &["missing".to_string()],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "non-matching whitelist should fail");
    Ok(())
}
#[tokio::test]
async fn test_download_model_whitelist_keeps_matching_name() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_whitelist_match");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    let keep_file = temp_dir.join("keep.gguf");
    let drop_file = temp_dir.join("drop.gguf");
    write(&keep_file, b"fake keep model data")?;
    write(&drop_file, b"fake drop model data")?;
    write(
        &config_path,
        format!(
            r#"{{"models": [
            {{"name": "keep", "copy": true, "source": {{"provider": "git", "location": "file:{}"}}}},
            {{"name": "drop", "copy": true, "source": {{"provider": "git", "location": "file:{}"}}}}
        ]}}"#,
            keep_file.display().to_string().replace('\\', "/"),
            drop_file.display().to_string().replace('\\', "/")
        ),
    )?;
    let verbose = Verbosity::new(0, 1);
    let output_dir = temp_dir.join("models");
    let result = super::model::run(
        &[],
        &[],
        &[],
        &Some(config_path),
        &Some(output_dir.clone()),
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &["keep".to_string()],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "matching whitelist entry should succeed");
    assert!(output_dir.join("keep").join("keep.gguf").exists());
    assert!(!output_dir.join("drop").join("drop.gguf").exists());
    Ok(())
}
#[test]
fn test_resolve_cli_action() {
    assert_eq!(SourceAction::from_options(false, false), None);
    assert_eq!(SourceAction::from_options(true, false), Some(SourceAction::Copy));
    assert_eq!(SourceAction::from_options(false, true), Some(SourceAction::Symlink));
}
#[test]
fn test_resolve_default_config_path_none_when_missing() -> Result<()> {
    let temp_dir = temp_test_dir("download_default_config_missing");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let previous_dir = env::current_dir()?;
    env::set_current_dir(&temp_dir)?;
    let resolved = ApplicationConfiguration::resolve(&None);
    env::set_current_dir(previous_dir)?;
    assert!(resolved.is_none(), "resolver should return none when no default config files exist");
    Ok(())
}
#[test]
fn test_resolve_default_config_path_prefers_standard_names() -> Result<()> {
    let temp_dir = temp_test_dir("download_default_config_resolution");
    let previous_dir = env::current_dir()?;
    let expected = temp_dir.join(".acorn.json");
    write(&expected, "{}")?;
    env::set_current_dir(&temp_dir)?;
    let resolved = ApplicationConfiguration::resolve(&None);
    env::set_current_dir(previous_dir)?;
    assert_eq!(resolved, Some(expected));
    let _cleanup = TestCleanup::new(temp_dir.clone());
    Ok(())
}
#[test]
fn test_huggingface_filter_extension() {
    let files = vec![
        candidate_file("model.Q4_K_M.gguf"),
        candidate_file("model.Q4_K_S.gguf"),
        candidate_file("config.json"),
        candidate_file("tokenizer.model"),
    ];
    let result = files.filter(Some("\\.gguf$"), None).unwrap();
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|f| f.path.ends_with(".gguf")));
}
#[test]
fn test_huggingface_filter_ignore() {
    let files = vec![
        candidate_file("model.Q4_K_M.gguf"),
        candidate_file("model.Q2_K.gguf"),
        candidate_file("model.Q3_K.gguf"),
    ];
    let result = files.filter(None, Some("Q2_")).unwrap();
    assert_eq!(result.len(), 1);
    assert!(result.iter().all(|f| f.path != "model.Q2_K.gguf"));
    assert_eq!(result.first().map(|f| f.path.as_str()), Some("model.Q4_K_M.gguf"));
}
#[test]
fn test_huggingface_filter_combined() {
    let files = vec![
        candidate_file("model.Q4_K_M.gguf"),
        candidate_file("model.Q4_K_S.gguf"),
        candidate_file("model.Q2_K.gguf"),
        candidate_file("config.json"),
    ];
    let result = files.filter(Some("\\.gguf$"), Some("Q2_")).unwrap();
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|f| f.path.ends_with(".gguf") && !f.path.contains("Q2_")));
}
#[test]
fn test_huggingface_filter_works_with_complex_patterns() {
    let files = vec![candidate_file("model.gguf")];
    let result = files.filter(Some("[a-z]+"), None);
    assert!(result.is_ok());
}
#[test]
fn test_download_item_checksum_mismatch_returns_incomplete_download_error() {
    let temp_dir = temp_test_dir("download_model_checksum_mismatch");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let file_path = temp_dir.join("model.gguf.part");
    write(&file_path, b"abc").expect("write model part");
    let item = DownloadItem {
        url: "https://example.test/model.gguf".to_string(),
        path: "model.gguf".to_string(),
        size: Some(3),
        sha: Some("0000000000000000000000000000000000000000000000000000000000000000".to_string()),
    };
    let result = item.verify_checksum(&file_path, false);
    assert!(result.is_err());
    let text = format!("{}", result.expect_err("checksum error"));
    assert!(text.contains("Model download is incomplete"));
}
#[tokio::test]
async fn test_download_task_resumes_partial_file_with_http_range() {
    let payload = b"abcdefghij".to_vec();
    let (url, handle) = spawn_range_server(payload.clone());
    let temp_dir = temp_test_dir("download_model_range_resume");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let destination = temp_dir.join("models");
    match create_dir_all(&destination) {
        | Ok(()) => {}
        | Err(why) => panic!("create destination directory failed: {why}"),
    }
    let item = DownloadItem {
        url,
        path: "model.gguf".to_string(),
        size: Some(payload.len() as u64),
        sha: None,
    };
    let part = item.partial_path(&destination);
    if let Some(parent) = part.parent() {
        match create_dir_all(parent) {
            | Ok(()) => {}
            | Err(why) => panic!("create parent directory failed: {why}"),
        }
    }
    match write(&part, &payload[..4]) {
        | Ok(()) => {}
        | Err(why) => panic!("seed partial file failed: {why}"),
    }
    let task = DownloadTask::new(item, &destination, true, false);
    let result = task.download().await;
    assert!(result.is_ok(), "resume download should succeed: {result:?}");
    let final_file = destination.join("model.gguf");
    assert!(final_file.exists(), "final file should be moved into place");
    let final_bytes = match std::fs::read(final_file) {
        | Ok(bytes) => bytes,
        | Err(why) => panic!("read resumed file failed: {why}"),
    };
    assert_eq!(final_bytes, payload, "resumed download should append remaining bytes");
    handle.join().expect("join server thread");
}
#[tokio::test]
async fn test_download_task_restarts_partial_file_without_http_range_support() {
    let payload = b"abcdefghij".to_vec();
    let (url, handle) = spawn_no_range_server(payload.clone());
    let temp_dir = temp_test_dir("download_model_no_range_restart");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let destination = temp_dir.join("models");
    create_dir_all(&destination).expect("create destination directory");
    let item = DownloadItem {
        url,
        path: "model.gguf".to_string(),
        size: Some(payload.len() as u64),
        sha: None,
    };
    let part = item.partial_path(&destination);
    write(&part, b"stale").expect("seed stale partial file");
    let task = DownloadTask::new(item, &destination, true, false);
    let result = task.download().await;
    assert!(result.is_ok(), "restart download should succeed: {result:?}");
    let final_bytes = std::fs::read(destination.join("model.gguf")).expect("read restarted file");
    assert_eq!(
        final_bytes, payload,
        "download should restart from byte zero without appending stale content"
    );
    handle.join().expect("join no-range server thread");
}
#[tokio::test]
async fn test_download_task_truncates_when_range_request_returns_200() {
    let payload = b"abcdefghij".to_vec();
    let (url, handle) = spawn_ignored_range_server(payload.clone());
    let temp_dir = temp_test_dir("download_model_ignored_range_restart");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let destination = temp_dir.join("models");
    create_dir_all(&destination).expect("create destination directory");
    let item = DownloadItem {
        url,
        path: "model.gguf".to_string(),
        size: Some(payload.len() as u64),
        sha: None,
    };
    let part = item.partial_path(&destination);
    write(&part, &payload[..4]).expect("seed partial file");
    let task = DownloadTask::new(item, &destination, true, false);
    let result = task.download().await;
    assert!(result.is_ok(), "ignored range download should succeed: {result:?}");
    let final_bytes = std::fs::read(destination.join("model.gguf")).expect("read restarted file");
    assert_eq!(final_bytes, payload, "200 OK to ranged GET should truncate partial file before writing");
    handle.join().expect("join ignored-range server thread");
}
#[tokio::test]
async fn test_download_task_retry_resumes_from_updated_partial_file_length() {
    let payload = b"abcdefghij".to_vec();
    let temp_dir = temp_test_dir("download_model_retry_resume_updated_offset");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let destination = temp_dir.join("models");
    create_dir_all(&destination).expect("create destination directory");
    let part = destination.join("model.gguf.part");
    write(&part, &payload[..4]).expect("seed partial file");
    let (url, handle) = spawn_retry_resume_server(payload.clone(), part.clone(), 4, 7);
    let item = DownloadItem {
        url,
        path: "model.gguf".to_string(),
        size: Some(payload.len() as u64),
        sha: None,
    };
    let task = DownloadTask::new(item, &destination, true, false);
    let result = task.download().await;
    assert!(result.is_ok(), "retry should resume from updated partial length: {result:?}");
    let final_bytes = std::fs::read(destination.join("model.gguf")).expect("read retried file");
    assert_eq!(final_bytes, payload, "retry should not duplicate bytes from the original resume offset");
    handle.join().expect("join retry-resume server thread");
}
#[tokio::test]
async fn test_download_task_succeeds_with_real_sha256_sidecar_validation() {
    let model_bytes = b"checksum-success-payload".to_vec();
    let sidecar_dir = temp_test_dir("download_model_checksum_sidecar_seed");
    let _sidecar_cleanup = TestCleanup::new(sidecar_dir.clone());
    let sidecar_file = sidecar_dir.join("model.gguf");
    match write(&sidecar_file, &model_bytes) {
        | Ok(()) => {}
        | Err(why) => panic!("write sidecar seed file failed: {why}"),
    }
    let digest = acorn::io::file_checksum(sidecar_file.clone(), None)
        .expect("checksum should be computed")
        .checksum_value;
    let sidecar_content = format!("{digest}  model.gguf\n");
    let (base_url, handle) = spawn_sidecar_server(model_bytes.clone(), sidecar_content);

    let served_sidecar = match acorn::io::http::get(format!("{base_url}/repo/resolve/main/model.gguf.sha256"))
        .send()
        .await
    {
        | Ok(response) => response.text().await.expect("read sidecar response body"),
        | Err(why) => panic!("fetch served sidecar failed: {why}"),
    };
    let served_digest = served_sidecar
        .split_whitespace()
        .find(|token| token.len() == 64 && token.chars().all(|character| character.is_ascii_hexdigit()))
        .map(|value| value.to_ascii_lowercase())
        .expect("extract digest from sidecar");

    let destination = temp_test_dir("download_model_checksum_sidecar_dest");
    let _dest_cleanup = TestCleanup::new(destination.clone());
    match create_dir_all(&destination) {
        | Ok(()) => {}
        | Err(why) => panic!("create destination directory failed: {why}"),
    }
    let item = DownloadItem {
        url: format!("{base_url}/repo/resolve/main/model.gguf"),
        path: "model.gguf".to_string(),
        size: Some(model_bytes.len() as u64),
        sha: Some(served_digest),
    };
    let task = DownloadTask::new(item, &destination, true, false);
    let result = task.download().await;
    assert!(result.is_ok(), "download with valid sidecar checksum should succeed: {result:?}");

    let final_file = destination.join("model.gguf");
    assert!(final_file.exists(), "final file should exist after successful checksum validation");
    let final_bytes = match std::fs::read(final_file) {
        | Ok(bytes) => bytes,
        | Err(why) => panic!("read downloaded model failed: {why}"),
    };
    assert_eq!(final_bytes, model_bytes);
    handle.join().expect("join sidecar server thread");
}
#[tokio::test]
async fn test_download_task_skip_verify_checksum_allows_checksum_mismatch() {
    let payload = b"checksum-skip-payload".to_vec();
    let (url, handle) = spawn_range_server(payload.clone());
    let temp_dir = temp_test_dir("download_model_skip_checksum_mismatch");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let destination = temp_dir.join("models");
    create_dir_all(&destination).expect("create destination directory");
    let item = DownloadItem {
        url,
        path: "model.gguf".to_string(),
        size: Some(payload.len() as u64),
        sha: Some("0000000000000000000000000000000000000000000000000000000000000000".to_string()),
    };
    let part = item.partial_path(&destination);
    write(&part, &payload[..4]).expect("seed partial file");
    let task = DownloadTask::new(item, &destination, true, true);
    let result = task.download().await;
    assert!(result.is_ok(), "skip checksum should allow mismatch: {result:?}");
    let final_bytes = std::fs::read(destination.join("model.gguf")).expect("read downloaded file");
    assert_eq!(final_bytes, payload);
    handle.join().expect("join range server thread");
}
#[test]
fn test_gguf_fallback_trigger_conditions() {
    let _error = color_eyre::eyre::eyre!(HuggingFaceError::NoGgufModelFiles);
    let files = vec![candidate_file("config.json")];
    let options = acorn::io::api::huggingface::Options::init().identifier("openai/gpt-oss-2b").build();
    assert!(files.should_use_fallback(&options));
    let offline = acorn::io::api::huggingface::Options::init()
        .identifier("openai/gpt-oss-2b")
        .offline(true)
        .build();
    assert!(!files.should_use_fallback(&offline));
    let filtered = acorn::io::api::huggingface::Options::init()
        .identifier("openai/gpt-oss-2b")
        .filter("safetensors")
        .build();
    assert!(!files.should_use_fallback(&filtered));
    let disabled = acorn::io::api::huggingface::Options::init()
        .identifier("openai/gpt-oss-2b")
        .no_fallback(true)
        .build();
    assert!(!files.should_use_fallback(&disabled));
    let invalid_identifier = acorn::io::api::huggingface::Options::init().identifier("gpt-oss-2b").build();
    assert!(!files.should_use_fallback(&invalid_identifier));
    assert!(!vec![candidate_file("model.gguf")].should_use_fallback(&options));
}
#[test]
fn test_hf_token_not_exposed_in_model_download_error_display() {
    let token = "test-secret-token-12345";
    with_var("HF_TOKEN", Some(token), || {
        let config = ApplicationConfiguration::parse(
            r#"{"models":[{"name":"tiny","source":{"provider":"huggingface","location":"https://huggingface.co/hf-internal-testing/tiny-random-bert"}}]}"#,
        )
        .expect("parse config");
        let plans = futures::executor::block_on(super::model::resolve_plans(
            &["tiny".to_string()],
            &config.models.unwrap_or_default(),
            None,
            &None,
            true,
        ))
        .expect("resolve plans");
        assert!(!plans.is_empty());
        let displays = plans
            .iter()
            .map(|plan| format!("{}", super::model::ModelDownloadError::MissingAuth(plan.name())))
            .collect::<Vec<_>>();
        assert!(displays.iter().all(|line| !line.contains(token)));
        let urls = plans
            .iter()
            .map(|plan| format!("https://huggingface.co/{}/resolve/main/model.gguf", plan.selector.identifier()))
            .collect::<Vec<_>>();
        assert!(urls.iter().all(|url| !url.contains(token)));
    });
}
#[test]
fn test_hf_token_not_exposed_in_model_resolution_logs() {
    #[derive(Clone)]
    struct SharedWriter(Arc<Mutex<Vec<u8>>>);
    impl<'a> MakeWriter<'a> for SharedWriter {
        type Writer = SharedWriterGuard;
        fn make_writer(&'a self) -> Self::Writer {
            SharedWriterGuard(self.0.clone())
        }
    }
    struct SharedWriterGuard(Arc<Mutex<Vec<u8>>>);
    impl Write for SharedWriterGuard {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().expect("lock log buffer").extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let token = "test-secret-token-12345";
    let logs = Arc::new(Mutex::new(Vec::new()));
    let writer = SharedWriter(logs.clone());
    let subscriber = tracing_subscriber::fmt().with_ansi(false).with_writer(writer).finish();
    with_var("HF_TOKEN", Some(token), || {
        tracing::subscriber::with_default(subscriber, || {
            let _ = futures::executor::block_on(super::model::resolve_plans(
                &["hf-internal-testing/tiny-random-bert".to_string()],
                &[],
                None,
                &None,
                true,
            ));
        });
    });
    let text = String::from_utf8(logs.lock().expect("lock logs").clone()).unwrap_or_default();
    assert!(!text.contains(token));
}
#[test]
fn test_hf_token_not_persisted_in_activity_rows() {
    let token = "test-secret-token-12345";
    with_var("HF_TOKEN", Some(token), || {
        let temp_dir = temp_test_dir("download_model_activity_redaction");
        let _cleanup = TestCleanup::new(temp_dir.clone());
        let database_path = temp_dir.join("activity.duckdb");
        let database = Database::<Table>::from_path(Some(database_path.clone()));
        database.migrate().expect("migrate db");
        super::log_activity(
            "download model",
            temp_dir.join("models").display().to_string(),
            &Some(database_path.clone()),
            true,
        );
        let row = ActivityRow::default()
            .select(Some(database_path), |_| true)
            .expect("select activity row")
            .expect("activity row exists");
        let columns = [
            row.command.unwrap_or_default(),
            row.user_path.unwrap_or_default(),
            row.executed_at.map(|value| value.to_rfc3339()).unwrap_or_default(),
            row.success.map(|value| value.to_string()).unwrap_or_default(),
        ];
        assert!(columns.iter().all(|value| !value.contains(token)));
    });
}
