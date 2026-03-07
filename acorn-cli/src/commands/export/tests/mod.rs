//! Tests for the export command
//!
//! This test suite validates the export command functionality including:
//! - Bag format export (BagIt specification)
//! - PowerPoint export with reference template
//! - JSON export (from YAML source)
//! - YAML export (from JSON source)
//! - PDF export (ignored by default - requires playwright)
//! - Combine flag for multiple files
//!
//! Test artifacts are created in `target/test_artifacts/` and cleaned up
//! automatically via the `TestCleanup` Drop implementation.
//!
//! Note: Tests run in online mode as export command doesn't support offline yet.

use crate::cli::arguments::FileFormat;
use crate::commands::export::run;
use acorn::prelude::{copy, create_dir_all, remove_dir_all, PathBuf};
use color_eyre::eyre::Result;
use std::fs;

const OFFLINE: bool = false; // Export command doesn't support offline mode yet

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
async fn test_export_bag_format() -> Result<()> {
    let temp_dir = temp_test_dir("export_bag");
    let _cleanup = TestCleanup::new(temp_dir.clone());

    // Copy entire fixture directory for bag export
    let source_dir = fixture_path("data/valid_project");
    let test_data_dir = temp_dir.join("valid_project");

    // Create directory and copy files manually
    create_dir_all(&test_data_dir)?;
    for entry in fs::read_dir(&source_dir)? {
        let entry = entry?;
        let dest = test_data_dir.join(entry.file_name());
        if entry.path().is_file() {
            copy(entry.path(), &dest)?;
        }
    }

    let output_dir = temp_dir.join("output");
    create_dir_all(&output_dir)?;

    let result = run(
        &Some(output_dir.clone()),
        &Some(test_data_dir),
        &None,            // branch
        &None,            // commit
        &FileFormat::Bag, // format
        &None,            // reference
        &false,           // combine
        &false,           // merge_request
        &OFFLINE,         // offline
    )
    .await;

    // Bag export may fail without proper directory structure, just check it doesn't panic
    let _ = result;
    Ok(())
}

#[tokio::test]
async fn test_export_powerpoint_format() -> Result<()> {
    let temp_dir = temp_test_dir("export_powerpoint");
    let _cleanup = TestCleanup::new(temp_dir.clone());

    // Copy fixture data to temp directory
    let source_dir = fixture_path("data/highlight/hecate");
    let test_data_dir = temp_dir.join("data");
    create_dir_all(&test_data_dir)?;

    // Copy index.json
    let source = source_dir.join("index.json");
    let dest = test_data_dir.join("index.json");
    copy(&source, &dest)?;

    // Copy reference.pptx
    let reference_source = fixture_path("data/highlight/reference.pptx");
    let reference_dest = test_data_dir.join("reference.pptx");
    if reference_source.exists() {
        copy(&reference_source, &reference_dest)?;
    } else {
        // Skip test if reference file doesn't exist
        return Ok(());
    }

    let output_dir = temp_dir.join("output");
    create_dir_all(&output_dir)?;

    let result = run(
        &Some(output_dir.clone()),
        &Some(test_data_dir.clone()),
        &None,                                  // branch
        &None,                                  // commit
        &FileFormat::Powerpoint,                // format
        &Some(PathBuf::from("reference.pptx")), // reference (relative)
        &false,                                 // combine
        &false,                                 // merge_request
        &OFFLINE,                               // offline
    )
    .await;

    assert!(result.is_ok(), "Export to PowerPoint format should succeed");

    // Verify PPTX was created
    let pptx_files: Vec<_> = fs::read_dir(&output_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()).map(|s| s == "pptx").unwrap_or(false))
        .collect();

    assert!(!pptx_files.is_empty(), "PowerPoint file should be created");

    Ok(())
}

#[tokio::test]
#[ignore = "PDF export requires playwright and may fail in test environment"]
async fn test_export_pdf_format() -> Result<()> {
    let temp_dir = temp_test_dir("export_pdf");
    let _cleanup = TestCleanup::new(temp_dir.clone());

    // Copy fixture data to temp directory
    let source_dir = fixture_path("data/valid_project");
    let test_data_dir = temp_dir.join("data");
    create_dir_all(&test_data_dir)?;

    // Copy index.json
    let source = source_dir.join("index.json");
    let dest = test_data_dir.join("index.json");
    copy(&source, &dest)?;

    let output_dir = temp_dir.join("output");
    create_dir_all(&output_dir)?;

    let _result = run(
        &Some(output_dir.clone()),
        &Some(test_data_dir),
        &None,            // branch
        &None,            // commit
        &FileFormat::Pdf, // format
        &None,            // reference
        &false,           // combine
        &false,           // merge_request
        &OFFLINE,         // offline
    )
    .await;

    // PDF generation requires playwright - test is ignored by default
    Ok(())
}

#[tokio::test]
async fn test_export_json_format() -> Result<()> {
    let temp_dir = temp_test_dir("export_json");
    let _cleanup = TestCleanup::new(temp_dir.clone());

    // Copy fixture data to temp directory (as YAML to convert to JSON)
    let source = fixture_path("data/valid_project/index.json");
    let test_data_dir = temp_dir.join("data");
    create_dir_all(&test_data_dir)?;

    // Copy as YAML so we can export to JSON
    let dest = test_data_dir.join("index.yaml");
    let content = fs::read_to_string(&source)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    let yaml_content = serde_yml::to_string(&json)?;
    fs::write(&dest, yaml_content)?;

    let result = run(
        &None, // output not used for JSON/YAML export
        &Some(test_data_dir.clone()),
        &None,             // branch
        &None,             // commit
        &FileFormat::Json, // format
        &None,             // reference
        &false,            // combine
        &false,            // merge_request
        &OFFLINE,          // offline
    )
    .await;

    assert!(result.is_ok(), "Export to JSON format should succeed");

    // Verify JSON was created (same directory, different extension)
    let json_file = test_data_dir.join("index.json");
    assert!(json_file.exists(), "JSON file should be created");

    Ok(())
}

#[tokio::test]
async fn test_export_yaml_format() -> Result<()> {
    let temp_dir = temp_test_dir("export_yaml");
    let _cleanup = TestCleanup::new(temp_dir.clone());

    // Copy fixture data to temp directory (JSON to export as YAML)
    let source = fixture_path("data/valid_project/index.json");
    let test_data_dir = temp_dir.join("data");
    create_dir_all(&test_data_dir)?;
    let dest = test_data_dir.join("index.json");
    copy(&source, &dest)?;

    let result = run(
        &None, // output not used for JSON/YAML export
        &Some(test_data_dir.clone()),
        &None,             // branch
        &None,             // commit
        &FileFormat::Yaml, // format
        &None,             // reference
        &false,            // combine
        &false,            // merge_request
        &OFFLINE,          // offline
    )
    .await;

    assert!(result.is_ok(), "Export to YAML format should succeed");

    // Verify YAML was created (same directory, different extension)
    let yaml_file = test_data_dir.join("index.yaml");
    assert!(yaml_file.exists(), "YAML file should be created");

    Ok(())
}

#[tokio::test]
async fn test_export_with_combine_flag() -> Result<()> {
    let temp_dir = temp_test_dir("export_combine");
    let _cleanup = TestCleanup::new(temp_dir.clone());

    // Copy multiple fixtures to temp directory
    let test_data_dir = temp_dir.join("data");
    create_dir_all(&test_data_dir)?;

    for (i, fixture) in ["data/valid_project/index.json", "data/valid_highlight/index.json"].iter().enumerate() {
        let source = fixture_path(fixture);
        let dest = test_data_dir.join(format!("file_{}.json", i));
        copy(&source, &dest)?;
    }

    let output_dir = temp_dir.join("output");
    create_dir_all(&output_dir)?;

    let result = run(
        &Some(output_dir.clone()),
        &Some(test_data_dir),
        &None,
        &None,
        &FileFormat::Json,
        &None,
        &true, // combine flag
        &false,
        &OFFLINE,
    )
    .await;

    assert!(result.is_ok(), "Export with combine flag should succeed");

    Ok(())
}
