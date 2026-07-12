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
use super::run;
use crate::test::util::{fixture_path, has_any_file, temp_test_dir, TestCleanup};
use acorn::io::config::ApplicationConfiguration;
use acorn::prelude::{create_dir_all, env, read_to_string, write};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;

const OFFLINE: bool = false;
const TEST_THREADS: usize = 2;

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

#[test]
fn test_config_parses_models_field_json() {
    let content = r#"{"models": ["meta-llama/Llama-2-7b-hf", "microsoft/phi-2"]}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    assert_eq!(models.len(), 2);
    assert_eq!(models[0], "meta-llama/Llama-2-7b-hf");
    assert_eq!(models[1], "microsoft/phi-2");
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
    assert_eq!(models[0], "meta-llama/Llama-2-7b-hf");
    assert_eq!(models[1], "microsoft/phi-2");
}
#[test]
fn test_config_parses_models_field_empty() {
    let content = r#"{"buckets": []}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    assert!(config.models.is_none());
}
#[tokio::test]
async fn test_download_model_offline_rejects() -> Result<()> {
    let verbose = Verbosity::new(0, 1);
    let result = super::model::run(&["some-model".to_string()], &[], &[], &None, &None, &None, &verbose, true).await;
    assert!(result.is_err(), "offline mode should reject model download");
    Ok(())
}
#[tokio::test]
async fn test_download_model_no_models_errors() -> Result<()> {
    let verbose = Verbosity::new(0, 1);
    let result = super::model::run(&[], &[], &[], &None, &None, &None, &verbose, false).await;
    assert!(result.is_err(), "empty model list should return an error");
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
    let result = super::model::run(&[], &[], &[], &Some(config_path), &Some(output_dir), &None, &verbose, false).await;
    assert!(result.is_ok(), "config-only model resolution should succeed");
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
    )
    .await;
    assert!(result.is_ok(), "positional + config with same model should succeed (deduplicated)");
    Ok(())
}
