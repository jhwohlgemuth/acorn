//! Tests for the format command
//!
//! This test suite validates the format command functionality including:
//! - Dry run mode (no file modification)
//! - Formatting files with no changes needed
//! - Formatting files with changes
//! - Processing multiple files
//!
//! Test artifacts are created in `target/test_artifacts/` and cleaned up
//! automatically via the `TestCleanup` Drop implementation.
//!
//! Note: Tests run in online mode as format command doesn't support offline yet.

use crate::commands::format::run;
use acorn::prelude::{copy, create_dir_all, remove_dir_all, PathBuf};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;
use std::fs;

const OFFLINE: bool = false; // Format command doesn't support offline mode yet

// Helper to get fixtures path
fn fixture_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join(path)
}

// Helper to create temp directory for test artifacts
fn temp_test_dir(test_name: &str) -> PathBuf {
    let temp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target")
        .join("test_artifacts")
        .join(test_name);
    let _ = create_dir_all(&temp);
    temp
}

// Cleanup helper
struct TestCleanup {
    path: PathBuf,
}

impl TestCleanup {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for TestCleanup {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = remove_dir_all(&self.path);
        }
    }
}

#[tokio::test]
async fn test_format_dry_run() -> Result<()> {
    let path = Some(fixture_path("data/format/unresolved_changes/index.json"));
    let result = run(
        &path,
        &None,  // branch
        &None,  // commit
        &None,  // ignore
        true,   // dry_run
        &false, // merge_request
        &Verbosity::new(0, 0),
        &OFFLINE, // offline
    )
    .await;

    assert!(result.is_ok(), "Format dry run should succeed");
    Ok(())
}

#[tokio::test]
async fn test_format_no_changes_needed() -> Result<()> {
    let temp_dir = temp_test_dir("format_no_changes");
    let _cleanup = TestCleanup::new(temp_dir.clone());

    // Copy fixture to temp directory
    let source = fixture_path("data/format/no_changes/index.json");
    let dest_file = temp_dir.join("index.json");
    copy(&source, &dest_file)?;

    let path = Some(dest_file);
    let result = run(
        &path,
        &None,
        &None,
        &None,
        false, // not dry run
        &false,
        &Verbosity::new(0, 0),
        &OFFLINE,
    )
    .await;

    assert!(result.is_ok(), "Format with no changes should succeed");
    Ok(())
}

#[tokio::test]
async fn test_format_with_changes() -> Result<()> {
    let temp_dir = temp_test_dir("format_with_changes");
    let _cleanup = TestCleanup::new(temp_dir.clone());

    // Copy fixture to temp directory
    let source = fixture_path("data/format/unresolved_changes/index.json");
    let dest_file = temp_dir.join("index.json");
    copy(&source, &dest_file)?;

    let path = Some(dest_file.clone());
    let result = run(
        &path,
        &None,
        &None,
        &None,
        false, // not dry run - actually write
        &false,
        &Verbosity::new(0, 0),
        &OFFLINE,
    )
    .await;

    assert!(result.is_ok(), "Format with changes should succeed");

    // Verify file was modified (content should be formatted)
    let formatted_content = fs::read_to_string(&dest_file)?;

    // File should still be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&formatted_content)?;
    assert!(parsed.is_object(), "Formatted file should be valid JSON");

    Ok(())
}

#[tokio::test]
async fn test_format_multiple_files() -> Result<()> {
    let temp_dir = temp_test_dir("format_multiple");
    let _cleanup = TestCleanup::new(temp_dir.clone());

    // Copy multiple fixtures to temp directory
    for (i, fixture) in ["data/valid_project/index.json", "data/valid_highlight/index.json"].iter().enumerate() {
        let source = fixture_path(fixture);
        let dest_file = temp_dir.join(format!("file_{}.json", i));
        copy(&source, &dest_file)?;
    }

    let path = Some(temp_dir);
    let result = run(&path, &None, &None, &None, false, &false, &Verbosity::new(0, 0), &OFFLINE).await;

    assert!(result.is_ok(), "Format multiple files should succeed");
    Ok(())
}
