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
use super::model::resolve_models;
use super::run;
use crate::test::util::{fixture_path, has_any_file, temp_test_dir, TestCleanup};
use acorn::io::{config::ApplicationConfiguration, SourceAction};
use acorn::prelude::{create_dir_all, env, read_to_string, write};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;
use std::fs;

const OFFLINE: bool = false;
const TEST_THREADS: usize = 2;

#[test]
fn test_config_parses_hugging_face_location_model_entry() {
    let content = r#"{"models": [{"name": "tiny-bert", "source": {"provider": "huggingface", "location": {"scheme": "https", "uri": "https://huggingface.co/hf-internal-testing/tiny-random-bert", "revision": "main"}}}]}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    assert_eq!(models.len(), 1);
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
    write(&config_path, r#"{"models": ["hf-internal-testing/tiny-random-bert"]}"#)?;
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
        &[],
    )
    .await;
    assert!(result.is_ok(), "config-only model resolution should succeed");
    Ok(())
}
#[tokio::test]
async fn test_download_model_local_file_copy() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_local_copy");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    fs::write(&model_file, b"fake model data")?;
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
        &[],
    )
    .await;
    assert!(result.is_ok(), "local file copy should succeed");
    let model_dir = output_dir.join("tiny");
    assert!(model_dir.join("tiny.gguf").exists(), "copy mode should place files in output dir");
    Ok(())
}
#[tokio::test]
async fn test_download_model_local_file_reference() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_local_reference");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    fs::write(&model_file, b"fake model data")?;
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
        &[],
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
        &[],
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
    fs::write(&model_file, b"fake model data")?;
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
        &[],
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
    fs::write(&model_file, b"fake model data")?;
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
        &[],
    )
    .await;
    assert!(result.is_ok(), "local model should work in offline mode");
    Ok(())
}
#[tokio::test]
async fn test_download_model_no_models_errors() -> Result<()> {
    let verbose = Verbosity::new(0, 1);
    let result = super::model::run(&[], &[], &[], &None, &None, &None, &verbose, false, false, false, &[]).await;
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
        &[],
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
    write(&config_path, r#"{"models": ["hf-internal-testing/tiny-random-bert"]}"#)?;
    let verbose = Verbosity::new(0, 1);
    let output_dir = temp_dir.join("models");
    let result = super::model::run(
        &["hf-internal-testing/tiny-random-bert".to_string()],
        &[],
        &[],
        &Some(config_path),
        &Some(output_dir),
        &None,
        &verbose,
        false,
        false,
        false,
        &[],
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
        &["missing".to_string()],
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
    write(
        &config_path,
        r#"{"models": [
            {"name": "keep", "source": {"provider": "huggingface", "location": "https://huggingface.co/hf-internal-testing/tiny-random-bert"}},
            {"name": "drop", "source": {"provider": "huggingface", "location": "https://huggingface.co/hf-internal-testing/tiny-random-gpt2"}}
        ]}"#,
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
        &["keep".to_string()],
    )
    .await;
    assert!(result.is_ok(), "matching whitelist entry should succeed");
    assert!(output_dir.join("hf-internal-testing/tiny-random-bert").exists());
    assert!(!output_dir.join("hf-internal-testing/tiny-random-gpt2").exists());
    Ok(())
}
#[test]
fn test_resolve_cli_action() {
    assert_eq!(super::model::resolve_cli_action(false, false), None);
    assert_eq!(super::model::resolve_cli_action(true, false), Some(SourceAction::Copy));
    assert_eq!(super::model::resolve_cli_action(false, true), Some(SourceAction::Symlink));
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
fn test_resolve_models_all_empty_returns_empty() {
    let positional = vec!["".to_string(), "  ".to_string()];
    let config = Some(vec!["".to_string()]);
    let result = resolve_models(&positional, config.as_deref());
    assert!(result.is_empty());
}
#[test]
fn test_resolve_models_config_only() {
    let positional: Vec<String> = vec![];
    let config = Some(vec!["x".to_string(), "y".to_string()]);
    let result = resolve_models(&positional, config.as_deref());
    assert_eq!(result, vec!["x", "y"]);
}
#[test]
fn test_resolve_models_dedup_preserves_positional_first() {
    let positional = vec!["m1".to_string(), "m2".to_string()];
    let config = Some(vec!["m2".to_string(), "m3".to_string(), "m1".to_string()]);
    let result = resolve_models(&positional, config.as_deref());
    assert_eq!(result, vec!["m1", "m2", "m3"]);
}
#[test]
fn test_resolve_models_positional_only() {
    let positional = vec!["a".to_string(), "b".to_string()];
    let result = resolve_models(&positional, None);
    assert_eq!(result, vec!["a", "b"]);
}
#[test]
fn test_resolve_models_positional_overrides_config() {
    let positional = vec!["b".to_string()];
    let config = Some(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    let result = resolve_models(&positional, config.as_deref());
    assert_eq!(result, vec!["b", "a", "c"]);
}
#[test]
fn test_resolve_models_skips_empty_entries() {
    let positional = vec!["".to_string(), "  ".to_string(), "a".to_string()];
    let config = Some(vec!["".to_string(), "b".to_string()]);
    let result = resolve_models(&positional, config.as_deref());
    assert_eq!(result, vec!["a", "b"]);
}
