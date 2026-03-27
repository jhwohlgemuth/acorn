//! Tests for the download command
//!
//! These tests validate local-bucket download execution with silent verbosity.
use crate::commands::download::run;
use crate::test_utils::{fixture_path, temp_test_dir, TestCleanup};
use acorn::prelude::{create_dir_all, PathBuf};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;
use std::fs;

const OFFLINE: bool = false;
const TEST_THREADS: usize = 2;

fn has_any_file(path: &PathBuf) -> bool {
    match fs::read_dir(path) {
        | Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .any(|entry_path| match entry_path.is_file() {
                | true => true,
                | false => has_any_file(&entry_path),
            }),
        | Err(_) => false,
    }
}
#[tokio::test]
async fn test_download_local_url_silent() -> Result<()> {
    let temp_dir = temp_test_dir("download_local_url_silent");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("output");
    create_dir_all(&output_dir)?;
    let url = Some("file:./tests/fixtures/data/bucket/".to_string());
    let verbose = Verbosity::new(0, 1);
    let result = run(&None, &url, &Some(output_dir.clone()), &OFFLINE, &None, TEST_THREADS, &verbose).await;
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
    let config = fs::read_to_string(fixture_path("config/with_local_buckets.json"))?;
    fs::write(&config_path, config)?;
    let verbose = Verbosity::new(0, 1);
    let result = run(
        &Some(config_path),
        &None,
        &Some(output_dir.clone()),
        &OFFLINE,
        &None,
        TEST_THREADS,
        &verbose,
    )
    .await;
    assert!(result.is_ok(), "download from local config should succeed");
    assert!(has_any_file(&output_dir), "output should contain copied files");
    Ok(())
}
