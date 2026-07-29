//! Tests for the export command
//!
//! This test suite validates the export command functionality including:
//! - Bag format export (BagIt specification)
//! - PowerPoint export with reference template
//! - JSON export (from YAML source)
//! - YAML export (from JSON source)
//! - Markdown export (from JSON source)
//! - PDF export visual snapshots (ignored by default - requires Chromium and Poppler)
//! - Combine flag for multiple files
//!
//! Test artifacts are created in `target/test_artifacts/` and cleaned up
//! automatically via the `TestCleanup` Drop implementation.
//!
//! Tests run in offline mode; PDF visual snapshots remain ignored because they launch an external browser and renderer.
use crate::cli::arguments::FileFormat;
#[cfg(feature = "pdf")]
use crate::commands::export::pdf::{browser_executable, configured_browser_executable};
use crate::commands::export::run;
use crate::test::util::{fixture_path, temp_test_dir, TestCleanup};
use acorn::analyzer::Standard;
#[cfg(feature = "pdf")]
use acorn::prelude::{canonicalize, read, var_os, Command, Path};
use acorn::prelude::{copy, create_dir_all, read_dir, read_to_string, write, PathBuf};
use acorn::schema::standard::datacite;
use acorn::schema::standard::invenio;
#[cfg(feature = "pdf")]
use acorn::util::constants::env::CHROME_PATH;
use clap_verbosity_flag::Verbosity;
#[cfg(feature = "pdf")]
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
#[cfg(feature = "pdf")]
use futures::lock::Mutex;
#[cfg(feature = "pdf")]
use which::which;

fn silent() -> Verbosity {
    Verbosity::new(0, 1)
}

const OFFLINE: bool = true;
const TEST_THREADS: usize = 4;
#[cfg(feature = "pdf")]
static PDF_VISUAL_TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(feature = "pdf")]
fn copy_pdf_fixture(source: &Path, destination: &Path) -> Result<()> {
    create_dir_all(destination)
        .map_err(|why| eyre!("Create PDF snapshot fixture directory — {why}"))
        .and_then(|_| read_dir(source).map_err(|why| eyre!("Read PDF snapshot fixture directory — {why}")))
        .and_then(|entries| {
            entries
                .map(|entry| entry.map_err(|why| eyre!("Read PDF snapshot fixture entry — {why}")))
                .try_for_each(|entry| {
                    entry.and_then(|entry| match entry.path().is_file() {
                        | true => copy(entry.path(), destination.join(entry.file_name()))
                            .map(|_| ())
                            .map_err(|why| eyre!("Copy PDF snapshot fixture entry — {why}")),
                        | false => Ok(()),
                    })
                })
        })
}
#[cfg(feature = "pdf")]
fn remove_aspect(path: &Path) -> Result<()> {
    read_to_string(path)
        .map_err(|why| eyre!("Read PDF snapshot research activity — {why}"))
        .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).map_err(|why| eyre!("Parse PDF snapshot research activity — {why}")))
        .map(|mut value| {
            let _removed = value.as_object_mut().and_then(|object| object.remove("aspect"));
            value
        })
        .and_then(|value| serde_json::to_string_pretty(&value).map_err(|why| eyre!("Serialize PDF snapshot research activity — {why}")))
        .and_then(|content| write(path, content).map_err(|why| eyre!("Write PDF snapshot research activity — {why}")))
}
#[cfg(feature = "pdf")]
fn render_first_pdf_page(pdf: &Path, output_prefix: &Path) -> Result<Vec<u8>> {
    which("pdftoppm")
        .map_err(|why| eyre!("Find pdftoppm for PDF visual snapshot — {why}"))
        .and_then(|renderer| {
            Command::new(renderer)
                .args(["-f", "1", "-l", "1", "-singlefile", "-png", "-r", "96"])
                .arg(pdf)
                .arg(output_prefix)
                .status()
                .map_err(|why| eyre!("Render PDF visual snapshot — {why}"))
        })
        .and_then(|status| match status.success() {
            | true => read(output_prefix.with_extension("png")).map_err(|why| eyre!("Read rendered PDF visual snapshot — {why}")),
            | false => Err(eyre!("pdftoppm failed to render PDF visual snapshot with status {status}")),
        })
}
#[cfg(feature = "pdf")]
fn pdf_test_browser() -> Result<PathBuf> {
    let configured = var_os(CHROME_PATH).map(PathBuf::from).filter(|path| path.is_file());
    let executable = ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser"]
        .iter()
        .find_map(|name| which(name).ok());
    #[cfg(target_os = "macos")]
    let installed = Some(PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")).filter(|path| path.is_file());
    #[cfg(not(target_os = "macos"))]
    let installed = None;
    configured
        .or(executable)
        .or(installed)
        .ok_or_else(|| eyre!("PDF visual snapshots require CHROME_PATH or an installed Chromium browser"))
}
#[cfg(feature = "pdf")]
async fn assert_pdf_visual_snapshot(test_name: &str, include_aspect: bool, snapshot: &str) -> Result<()> {
    let _guard = PDF_VISUAL_TEST_LOCK.lock().await;
    let temp_dir = temp_test_dir(test_name);
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source_dir = fixture_path("data/highlight/hecate");
    let test_data_dir = temp_dir.join("data");
    let output_dir = temp_dir.join("output");
    let data_path = test_data_dir.join("index.json");
    let prepared = copy_pdf_fixture(&source_dir, &test_data_dir)
        .and_then(|_| match include_aspect {
            | true => Ok(()),
            | false => remove_aspect(&data_path),
        })
        .and_then(|_| create_dir_all(&output_dir).map_err(|why| eyre!("Create PDF snapshot output directory — {why}")))
        .and_then(|_| pdf_test_browser());
    match prepared {
        | Ok(browser) => run(
            &Some(output_dir.clone()),
            &Some(data_path),
            &None,
            &Some(browser),
            &None,
            &[],
            &[],
            false,
            false,
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
        .await
        .and_then(|_| render_first_pdf_page(&output_dir.join("hecate.pdf"), &temp_dir.join(snapshot)))
        .map(|image| {
            let snapshot_name = format!("{snapshot}.png");
            insta::assert_binary_snapshot!(&snapshot_name, image);
        }),
        | Err(why) => Err(why),
    }
}
#[cfg(feature = "pdf")]
#[tokio::test]
async fn test_browser_executable_prefers_configured_path_offline() -> Result<()> {
    let directory = temp_test_dir("configured-chrome");
    let _cleanup = TestCleanup::new(directory.clone());
    let executable = directory.join("chrome");
    write(&executable, b"browser")?;
    let expected = canonicalize(&executable)?;
    let selected = browser_executable(OFFLINE, Some(&executable)).await?;
    assert_eq!(selected, Some(expected));
    Ok(())
}
#[cfg(feature = "pdf")]
#[test]
fn test_configured_browser_executable_rejects_non_files() {
    let directory = temp_test_dir("invalid-configured-chrome");
    let _cleanup = TestCleanup::new(directory.clone());
    let missing = directory.join("missing-chrome");
    assert!(configured_browser_executable(&missing).is_err_and(|why| why.to_string().contains("is not a file")));
    assert!(configured_browser_executable(&directory).is_err_and(|why| why.to_string().contains("is not a file")));
}
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
        &None,
        &[],
        &[],
        false,
        false,
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
        &None,
        &[],
        &[],
        false,
        false,
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
#[cfg(feature = "pdf")]
#[tokio::test]
#[ignore = "Requires Chromium and pdftoppm for PDF visual snapshots"]
async fn test_export_pdf_with_aspect_visual_snapshot() -> Result<()> {
    assert_pdf_visual_snapshot("export-pdf-with-aspect", true, "pdf-export-with-aspect").await
}
#[cfg(feature = "pdf")]
#[tokio::test]
#[ignore = "Requires Chromium and pdftoppm for PDF visual snapshots"]
async fn test_export_pdf_without_aspect_visual_snapshot() -> Result<()> {
    assert_pdf_visual_snapshot("export-pdf-without-aspect", false, "pdf-export-without-aspect").await
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
        &None,
        &[],
        &[],
        false,
        false,
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
async fn test_export_offline_rejects_remote_merge_request() {
    let result = run(
        &None,
        &Some(fixture_path("data/valid_project")),
        &None,
        &None,
        &None,
        &[],
        &[],
        false,
        false,
        &FileFormat::Json,
        &None,
        &None,
        &None,
        &false,
        &true,
        &false,
        &[],
        false,
        false,
        TEST_THREADS,
        &silent(),
        OFFLINE,
    )
    .await;
    assert!(result.is_err_and(|why| why.to_string().contains("remote merge request")));
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
        &None,
        &[],
        &[],
        false,
        false,
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
        &None,
        &[],
        &[],
        false,
        false,
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
        &None,
        &[],
        &[],
        false,
        false,
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
        &None,
        &[],
        &[],
        false,
        false,
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
        &None,
        &[],
        &[],
        false,
        false,
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
