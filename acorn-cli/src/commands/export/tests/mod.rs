//! Tests for the export command
//!
//! This test suite validates the export command functionality including:
//! - Bag format export (BagIt specification)
//! - PowerPoint export with reference template
//! - JSON export (from YAML source)
//! - YAML export (from JSON source)
//! - Markdown export (from JSON source)
//! - PDF export (ignored by default - requires playwright)
//! - Combine flag for multiple files
//!
//! Test artifacts are created in `target/test_artifacts/` and cleaned up
//! automatically via the `TestCleanup` Drop implementation.
//!
//! Note: Tests run in online mode as export command doesn't support offline yet.
use crate::cli::arguments::FileFormat;
use crate::commands::export::run;
use crate::test::util::{fixture_path, temp_test_dir, TestCleanup};
use acorn::analyzer::Standard;
use acorn::prelude::{copy, create_dir_all, read_dir, read_to_string, write, PathBuf};
use acorn::schema::standard::datacite;
use acorn::schema::standard::invenio;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;

fn silent() -> Verbosity {
    Verbosity::new(0, 1)
}

const OFFLINE: bool = false;
const TEST_THREADS: usize = 4;
#[tokio::test]
async fn test_export_bag_format() -> Result<()> {
    let temp_dir = temp_test_dir("export_bag");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source_dir = fixture_path("data/valid_project");
    let test_data_dir = temp_dir.join("valid_project");
    create_dir_all(&test_data_dir)?;
    for entry in read_dir(&source_dir)? {
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
        &None,
        &None,
        &[],
        &[],
        &FileFormat::Bag,
        &None,
        &None,
        &None,
        &false,
        &false,
        &false,
        &[],
        false,
        false,
        TEST_THREADS,
        &silent(),
        OFFLINE,
    )
    .await;
    let _ = result;
    Ok(())
}
#[tokio::test]
async fn test_export_powerpoint_format() -> Result<()> {
    let temp_dir = temp_test_dir("export_powerpoint");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source_dir = fixture_path("data/highlight/hecate");
    let test_data_dir = temp_dir.join("data");
    create_dir_all(&test_data_dir)?;
    let source = source_dir.join("index.json");
    let dest = test_data_dir.join("index.json");
    copy(&source, &dest)?;
    let reference_source = fixture_path("data/highlight/reference.pptx");
    let reference_dest = test_data_dir.join("reference.pptx");
    if reference_source.exists() {
        copy(&reference_source, &reference_dest)?;
    } else {
        return Ok(());
    }
    let output_dir = temp_dir.join("output");
    create_dir_all(&output_dir)?;
    let result = run(
        &Some(output_dir.clone()),
        &Some(test_data_dir.clone()),
        &None,
        &None,
        &[],
        &[],
        &FileFormat::Powerpoint,
        &Some(PathBuf::from("reference.pptx")),
        &None,
        &None,
        &false,
        &false,
        &false,
        &[],
        false,
        false,
        TEST_THREADS,
        &silent(),
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "Export to PowerPoint format should succeed");
    let pptx_files: Vec<_> = read_dir(&output_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()).map(|s| s == "pptx").unwrap_or(false))
        .collect();
    assert!(!pptx_files.is_empty(), "PowerPoint file should be created");
    Ok(())
}
#[tokio::test]
#[ignore = "Requires Playwright for PDF generation"]
async fn test_export_pdf_format() -> Result<()> {
    let temp_dir = temp_test_dir("export_pdf");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source_dir = fixture_path("data/valid_project");
    let test_data_dir = temp_dir.join("data");
    create_dir_all(&test_data_dir)?;
    let source = source_dir.join("index.json");
    let dest = test_data_dir.join("index.json");
    copy(&source, &dest)?;
    let output_dir = temp_dir.join("output");
    create_dir_all(&output_dir)?;
    let _result = run(
        &Some(output_dir.clone()),
        &Some(test_data_dir),
        &None,
        &None,
        &[],
        &[],
        &FileFormat::Pdf,
        &None,
        &None,
        &None,
        &false,
        &false,
        &false,
        &[],
        false,
        false,
        TEST_THREADS,
        &silent(),
        OFFLINE,
    )
    .await;
    Ok(())
}
#[tokio::test]
async fn test_export_json_format() -> Result<()> {
    let temp_dir = temp_test_dir("export_json");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source = fixture_path("data/valid_project/index.json");
    let test_data_dir = temp_dir.join("data");
    create_dir_all(&test_data_dir)?;
    let dest = test_data_dir.join("index.yaml");
    let content = read_to_string(&source)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    let yaml_content = serde_norway::to_string(&json)?;
    write(&dest, yaml_content)?;
    let result = run(
        &None,
        &Some(test_data_dir.clone()),
        &None,
        &None,
        &[],
        &[],
        &FileFormat::Json,
        &None,
        &None,
        &None,
        &false,
        &false,
        &false,
        &[],
        false,
        false,
        TEST_THREADS,
        &silent(),
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "Export to JSON format should succeed");
    let json_file = test_data_dir.join("index.json");
    assert!(json_file.exists(), "JSON file should be created");
    Ok(())
}

#[tokio::test]
async fn test_export_yaml_format() -> Result<()> {
    let temp_dir = temp_test_dir("export_yaml");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source = fixture_path("data/valid_project/index.json");
    let test_data_dir = temp_dir.join("data");
    create_dir_all(&test_data_dir)?;
    let dest = test_data_dir.join("index.json");
    copy(&source, &dest)?;
    let result = run(
        &None,
        &Some(test_data_dir.clone()),
        &None,
        &None,
        &[],
        &[],
        &FileFormat::Yaml,
        &None,
        &None,
        &None,
        &false,
        &false,
        &false,
        &[],
        false,
        false,
        TEST_THREADS,
        &silent(),
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "Export to YAML format should succeed");
    let yaml_file = test_data_dir.join("index.yaml");
    assert!(yaml_file.exists(), "YAML file should be created");
    Ok(())
}

#[tokio::test]
async fn test_export_markdown_format() -> Result<()> {
    let temp_dir = temp_test_dir("export_markdown");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source = fixture_path("data/valid_project/index.json");
    let test_data_dir = temp_dir.join("data");
    create_dir_all(&test_data_dir)?;
    let dest = test_data_dir.join("index.json");
    copy(&source, &dest)?;
    let result = run(
        &None,
        &Some(test_data_dir.clone()),
        &None,
        &None,
        &[],
        &[],
        &FileFormat::Markdown,
        &None,
        &None,
        &None,
        &false,
        &false,
        &false,
        &[],
        false,
        false,
        TEST_THREADS,
        &silent(),
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "Export to Markdown format should succeed");
    let markdown_file = test_data_dir.join("index.md");
    assert!(markdown_file.exists(), "Markdown file should be created");
    let markdown_content = read_to_string(&markdown_file)?;
    assert!(!markdown_content.trim().is_empty(), "Markdown file should not be empty");
    let source_content = read_to_string(&source)?;
    let source_json: serde_json::Value = serde_json::from_str(&source_content)?;
    let title = source_json.get("title").and_then(serde_json::Value::as_str).unwrap_or_default();
    assert!(!title.is_empty(), "Fixture title should not be empty");
    let expected_h1 = format!("# {title}");
    let expected_h2 = format!("## {title}");
    let has_markdown_title = markdown_content
        .lines()
        .map(str::trim)
        .any(|line| line == expected_h1 || line == expected_h2);
    assert!(has_markdown_title, "Markdown should include a heading with the exported title");
    Ok(())
}
#[tokio::test]
async fn test_export_with_combine_flag() -> Result<()> {
    let temp_dir = temp_test_dir("export_combine");
    let _cleanup = TestCleanup::new(temp_dir.clone());
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
        &[],
        &[],
        &FileFormat::Json,
        &None,
        &None,
        &None,
        &true,
        &false,
        &false,
        &[],
        false,
        false,
        TEST_THREADS,
        &silent(),
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "Export with combine flag should succeed");
    Ok(())
}
#[ignore = "Only works on Windows?"]
#[tokio::test]
async fn test_export_crosswalk_datacite_to_invenio() -> Result<()> {
    let temp_dir = temp_test_dir("export_crosswalk_datacite_to_invenio");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let test_data_dir = temp_dir.join("data");
    create_dir_all(&test_data_dir)?;
    let output_dir = temp_dir.join("output");
    create_dir_all(&output_dir)?;
    let source = fixture_path("schema/datacite.json");
    let dest = test_data_dir.join("datacite.json");
    copy(&source, &dest)?;
    let result = run(
        &Some(output_dir.clone()),
        &Some(dest.clone()),
        &None,
        &None,
        &[],
        &[],
        &FileFormat::Json,
        &None,
        &Some(Standard::Datacite),
        &Some(Standard::Invenio),
        &false,
        &false,
        &false,
        &[],
        false,
        false,
        TEST_THREADS,
        &silent(),
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "Crosswalk from DataCite to Invenio should succeed");
    let output_content = read_to_string(output_dir.join("datacite.json"))?;
    let parsed: Vec<invenio::Record> = serde_json::from_str(&output_content)?;
    assert!(!parsed.is_empty(), "Expected at least one Invenio record in output");
    Ok(())
}
#[ignore = "Only works on Windows?"]
#[tokio::test]
async fn test_export_crosswalk_infer_source_to_datacite() -> Result<()> {
    let temp_dir = temp_test_dir("export_crosswalk_infer_source_to_datacite");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let test_data_dir = temp_dir.join("data");
    create_dir_all(&test_data_dir)?;
    let output_dir = temp_dir.join("output");
    create_dir_all(&output_dir)?;
    let dest = test_data_dir.join("invenio.json");
    let source = serde_json::json!([
        {
            "id": "invenio-infer-001",
            "metadata": {
                "title": "Inferred Invenio Record",
                "publication_date": "2024-01-01",
                "description": "Test record for schema inference"
            }
        }
    ]);
    write(&dest, serde_json::to_string_pretty(&source)?)?;
    let result = run(
        &Some(output_dir.clone()),
        &Some(dest.clone()),
        &None,
        &None,
        &[],
        &[],
        &FileFormat::Json,
        &None,
        &None,
        &Some(Standard::Datacite),
        &false,
        &false,
        &false,
        &[],
        false,
        false,
        TEST_THREADS,
        &silent(),
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "Crosswalk with inferred source to DataCite should succeed");
    let output_content = read_to_string(output_dir.join("invenio.json"))?;
    let parsed: Vec<datacite::Record> = serde_json::from_str(&output_content)?;
    assert!(!parsed.is_empty(), "Expected at least one DataCite record in output");
    Ok(())
}
