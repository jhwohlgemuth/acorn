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
use acorn::prelude::{create_dir_all, env, write};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;
use std::fs;

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
    let config = fs::read_to_string(fixture_path("config/with_local_buckets.json"))?.replace("file:./tests/fixtures/data/bucket/", &bucket_uri);
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
