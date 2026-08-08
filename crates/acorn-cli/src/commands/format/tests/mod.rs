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
use crate::test::util::{fixture_path, temp_test_dir, TestCleanup};
use acorn::prelude::copy;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;
use std::fs;

const OFFLINE: bool = false;

#[tokio::test]
async fn test_format_dry_run() -> Result<()> {
    let path = Some(fixture_path("data/format/unresolved_changes/index.json"));
    let result = run(&path, &None, &None, &None, &None, true, false, &false, &Verbosity::new(0, 0), OFFLINE).await;
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
    let result = run(&path, &None, &None, &None, &None, false, false, &false, &Verbosity::new(0, 0), OFFLINE).await;
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
    let result = run(&path, &None, &None, &None, &None, false, false, &false, &Verbosity::new(0, 0), OFFLINE).await;
    assert!(result.is_ok(), "Format with changes should succeed");
    let formatted_content = fs::read_to_string(&dest_file)?;
    let parsed: serde_json::Value = serde_json::from_str(&formatted_content)?;
    assert!(parsed.is_object(), "Formatted file should be valid JSON");
    Ok(())
}
#[tokio::test]
async fn test_format_multiple_files() -> Result<()> {
    let temp_dir = temp_test_dir("format_multiple");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    for (i, fixture) in ["data/valid_project/index.json", "data/valid_highlight/index.json"].iter().enumerate() {
        let source = fixture_path(fixture);
        let dest_file = temp_dir.join(format!("file_{}.json", i));
        copy(&source, &dest_file)?;
    }
    let path = Some(temp_dir);
    let result = run(&path, &None, &None, &None, &None, false, false, &false, &Verbosity::new(0, 0), OFFLINE).await;
    assert!(result.is_ok(), "Format multiple files should succeed");
    Ok(())
}
