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
extern crate alloc;
use crate::cli::CommandOptions;
use crate::commands::format::{format_file, run};
use crate::test::util::{fixture_path, temp_test_dir, TestCleanup};
use acorn::analyzer::Standard;
use acorn::io::slice_path;
use acorn::util::constants::app::DISPLAY_PATH_CHARACTER_COUNT;
use acorn_schema::enrichment::Provider;
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::standard::cff::Cff;
use alloc::sync::Arc;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;
use std::fs;
use std::fs::copy;
use std::io::Write;
use std::sync::Mutex;
use tracing_subscriber::fmt::MakeWriter;

const OFFLINE: bool = false;
struct SharedWriter(Arc<Mutex<Vec<u8>>>);
struct SharedWriterGuard(Arc<Mutex<Vec<u8>>>);
impl<'a> MakeWriter<'a> for SharedWriter {
    type Writer = SharedWriterGuard;
    fn make_writer(&'a self) -> Self::Writer {
        SharedWriterGuard(self.0.clone())
    }
}
impl Write for SharedWriterGuard {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        match self.0.lock() {
            | Ok(mut logs) => {
                logs.extend_from_slice(buffer);
                Ok(buffer.len())
            }
            | Err(why) => Err(std::io::Error::other(why.to_string())),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn captured_logs(operation: impl FnOnce()) -> String {
    let logs = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(SharedWriter(logs.clone()))
        .finish();
    tracing::subscriber::with_default(subscriber, operation);
    let bytes = logs.lock().expect("lock logs").clone();
    String::from_utf8(bytes).unwrap_or_default()
}
#[cfg(feature = "exec")]
#[tokio::test]
async fn test_configured_format_gate_prevents_write() -> Result<()> {
    let temp_dir = temp_test_dir("format_configured_gate");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source = fixture_path("data/format/unresolved_changes/index.json");
    let dest_file = temp_dir.join("index.json");
    copy(&source, &dest_file)?;
    let original = fs::read_to_string(&dest_file)?;
    let script_root = temp_dir.join(".acorn");
    fs::create_dir(&script_root)?;
    fs::write(
        script_root.join("quality.rhai"),
        "// @acorn-exec-v0.2.0\nfn evaluate(context) { #{findings: [#{rule_id: \"format-gate\", severity: \"error\", message: \"blocked\"}]} }",
    )?;
    let config = temp_dir.join(".acorn.json");
    fs::write(&config, r#"{"scripts":[{"path":"quality","hooks":["format"]}]}"#)?;
    let result = run(
        &Some(dest_file.clone()),
        &None,
        &None,
        &None,
        &None,
        &Some(config),
        false,
        false,
        true,
        &[],
        &false,
        Standard::ResearchActivityData,
        1,
        &Verbosity::new(0, 0),
        OFFLINE,
    )
    .await;
    assert!(result.is_err(), "error finding must block the format proposal");
    assert_eq!(fs::read_to_string(dest_file)?, original);
    Ok(())
}
#[cfg(feature = "exec")]
#[tokio::test]
async fn test_configured_format_missing_path_prevents_write() {
    let temp_dir = temp_test_dir("format_missing_config");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source = fixture_path("data/format/unresolved_changes/index.json");
    let dest_file = temp_dir.join("index.json");
    copy(&source, &dest_file).expect("copy format fixture");
    let original = fs::read_to_string(&dest_file).expect("read original fixture");
    let result = run(
        &Some(dest_file.clone()),
        &None,
        &None,
        &None,
        &None,
        &Some(temp_dir.join("missing.json")),
        false,
        false,
        true,
        &[],
        &false,
        Standard::ResearchActivityData,
        1,
        &Verbosity::new(0, 0),
        OFFLINE,
    )
    .await;
    assert!(
        result.is_err_and(|why| why.to_string().contains("Configuration file does not exist")),
        "missing explicit configuration must fail"
    );
    assert_eq!(fs::read_to_string(dest_file).expect("read unchanged fixture"), original);
}
#[cfg(not(feature = "exec"))]
#[tokio::test]
async fn test_configured_format_requires_exec_feature() {
    let temp_dir = temp_test_dir("format_config_requires_exec");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source = fixture_path("data/format/unresolved_changes/index.json");
    let dest_file = temp_dir.join("index.json");
    copy(&source, &dest_file).expect("copy format fixture");
    let original = fs::read_to_string(&dest_file).expect("read original fixture");
    let config = temp_dir.join(".acorn.json");
    fs::write(&config, "{}").expect("write application configuration");
    let result = run(
        &Some(dest_file.clone()),
        &None,
        &None,
        &None,
        &None,
        &Some(config),
        false,
        false,
        true,
        &[],
        &false,
        Standard::ResearchActivityData,
        1,
        &Verbosity::new(0, 0),
        OFFLINE,
    )
    .await;
    assert!(
        result.is_err_and(|why| why.to_string().contains("--config requires the `exec` feature")),
        "configuration must not be ignored without Rhai support"
    );
    assert_eq!(fs::read_to_string(dest_file).expect("read unchanged fixture"), original);
}
#[tokio::test]
async fn test_format_dispatches_cff_and_rejects_unsupported_provider() {
    let temp_dir = temp_test_dir("format_cff");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let path = temp_dir.join("CITATION.cff");
    let cff = Cff {
        title: "Example".to_string(),
        ..Cff::default()
    };
    fs::write(&path, serde_norway::to_string(&cff).expect("serialize CFF")).expect("write CFF");
    let formatted = run(
        &Some(path.clone()),
        &None,
        &None,
        &None,
        &None,
        &None,
        false,
        false,
        true,
        &[],
        &false,
        Standard::CitationFileFormat,
        1,
        &Verbosity::new(0, 0),
        OFFLINE,
    )
    .await;
    assert!(formatted.is_ok(), "CFF formatting should use the selected standard");
    let unsupported = run(
        &Some(path),
        &None,
        &None,
        &None,
        &None,
        &None,
        true,
        true,
        true,
        &[Provider::Ror],
        &false,
        Standard::CitationFileFormat,
        1,
        &Verbosity::new(0, 0),
        OFFLINE,
    )
    .await;
    assert!(unsupported.is_err_and(|why| why.to_string().contains("Only the ORCID provider")));
}
#[tokio::test]
async fn test_format_dry_run() -> Result<()> {
    let temp_dir = temp_test_dir("format_dry_run");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source = fixture_path("data/format/unresolved_changes/index.json");
    let dest_file = temp_dir.join("index.json");
    copy(&source, &dest_file)?;
    let original = fs::read_to_string(&dest_file)?;
    let path = Some(dest_file.clone());
    let result = run(
        &path,
        &None,
        &None,
        &None,
        &None,
        &None,
        true,
        false,
        false,
        &[],
        &false,
        Standard::ResearchActivityData,
        1,
        &Verbosity::new(0, 0),
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "Format dry run should succeed");
    assert_eq!(fs::read_to_string(dest_file)?, original);
    Ok(())
}
#[tokio::test]
async fn test_format_enrich_dry_run_does_not_write() -> Result<()> {
    let temp_dir = temp_test_dir("format_enrich_dry_run");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let dest_file = temp_dir.join("index.json");
    let mut data = ResearchActivity::default();
    data.meta.ror = Some(vec!["https://ror.org/01qz5mb56".to_string()].into());
    let original = serde_json::to_string_pretty(&data)?;
    fs::write(&dest_file, &original)?;
    let result = run(
        &Some(dest_file.clone()),
        &None,
        &None,
        &None,
        &None,
        &None,
        true,
        true,
        true,
        &[Provider::Ror],
        &false,
        Standard::ResearchActivityData,
        1,
        &Verbosity::new(0, 0),
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "Enriched dry run should succeed");
    assert_eq!(fs::read_to_string(dest_file)?, original);
    Ok(())
}
#[tokio::test]
async fn test_format_enrich_offline_fails_without_writing() -> Result<()> {
    let temp_dir = temp_test_dir("format_enrich_offline");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source = fixture_path("data/format/unresolved_changes/index.json");
    let dest_file = temp_dir.join("index.json");
    copy(&source, &dest_file)?;
    let original = fs::read_to_string(&dest_file)?;
    let result = run(
        &Some(dest_file.clone()),
        &None,
        &None,
        &None,
        &None,
        &None,
        false,
        true,
        false,
        &[Provider::Ror],
        &false,
        Standard::ResearchActivityData,
        1,
        &Verbosity::new(0, 0),
        true,
    )
    .await;
    assert!(result.is_err(), "Offline enrichment should fail");
    assert_eq!(fs::read_to_string(dest_file)?, original);
    Ok(())
}
#[test]
fn test_format_file_logs_path_for_changes() -> Result<()> {
    let temp_dir = temp_test_dir("format_logs_changes");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source = fixture_path("data/format/unresolved_changes/index.json");
    let dest_file = temp_dir.join("index.json");
    copy(&source, &dest_file)?;
    let options = CommandOptions::init().no_color(true).build();
    let logs = captured_logs(|| format_file::<ResearchActivity>(&dest_file, &options).expect("format changed file"));
    assert!(logs.contains("Formatted"));
    assert!(logs.contains(&dest_file.display().to_string()));
    Ok(())
}
#[test]
fn test_format_file_logs_path_for_no_changes() -> Result<()> {
    let temp_dir = temp_test_dir("format_logs_no_changes");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let source = fixture_path("data/format/no_changes/index.json");
    let dest_file = temp_dir.join("index.json");
    copy(&source, &dest_file)?;
    let options = CommandOptions::init().no_color(true).build();
    let logs = captured_logs(|| format_file::<ResearchActivity>(&dest_file, &options).expect("format unchanged file"));
    let message_index = logs.find("No format changes for").expect("no-change message");
    assert_eq!(logs.as_bytes().get(message_index.saturating_sub(1)), Some(&b' '));
    assert!(logs.contains("INFO"));
    assert!(logs.contains("SKIP"));
    assert!(!logs.contains("WARN"));
    assert!(logs.contains(&slice_path(&dest_file, DISPLAY_PATH_CHARACTER_COUNT)));
    assert!(!logs.contains(&dest_file.display().to_string()));
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
    let result = run(
        &path,
        &None,
        &None,
        &None,
        &None,
        &None,
        false,
        false,
        false,
        &[],
        &false,
        Standard::ResearchActivityData,
        1,
        &Verbosity::new(0, 0),
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "Format multiple files should succeed");
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
    let original_permissions = fs::metadata(&dest_file)?.permissions();
    let mut read_only_permissions = original_permissions.clone();
    read_only_permissions.set_readonly(true);
    fs::set_permissions(&dest_file, read_only_permissions)?;
    let path = Some(dest_file.clone());
    let result = run(
        &path,
        &None,
        &None,
        &None,
        &None,
        &None,
        false,
        false,
        false,
        &[],
        &false,
        Standard::ResearchActivityData,
        1,
        &Verbosity::new(0, 0),
        OFFLINE,
    )
    .await;
    fs::set_permissions(dest_file, original_permissions)?;
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
        &None,
        &None,
        false,
        false,
        false,
        &[],
        &false,
        Standard::ResearchActivityData,
        1,
        &Verbosity::new(0, 0),
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "Format with changes should succeed");
    let formatted_content = fs::read_to_string(&dest_file)?;
    let parsed: serde_json::Value = serde_json::from_str(&formatted_content)?;
    assert!(parsed.is_object(), "Formatted file should be valid JSON");
    Ok(())
}
