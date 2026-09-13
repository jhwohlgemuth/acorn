//! Tests for the check command
//!
//! This test suite validates the check command functionality including:
//! - Valid project and highlight checking
//! - Invalid data handling with no-fail mode
//! - Skip categories (prose, schema validation)
//! - Different readability metrics (FKGL, ARI, CLI)
//! - Terse output mode
//!
//! All tests run in offline mode to avoid network dependencies.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
mod context;

use super::{run, PrepareText};
use crate::test::server::spawn_source_server;
use crate::test::util::fixture_path;
use acorn::analyzer::host::service::{classify_paths, Analysis};
use acorn::analyzer::readability::ReadabilityType;
use acorn::analyzer::{Check, CheckCategory, CheckSummary, Standard};
use acorn::io::TextOrigin;
use acorn::LocationExt;
use acorn_core::util::MimeType;
use acorn_schema::standard::{datacite, dcat, huwise, invenio};
use assert_cmd::Command;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;
use jiff::SignedDuration;
use std::fs::{read, read_to_string};
use std::path::PathBuf;

const OFFLINE: bool = true;
#[test]
fn summary_rows_preserve_cli_labels_and_order() {
    let rows = CheckSummary {
        fair: 7,
        schema: 1,
        link: 2,
        prose: 3,
        quality: 4,
        readability: 5,
        crosswalk: 6,
    }
    .rows();
    assert_eq!(
        rows,
        vec![
            vec!["Schema items found", "1"],
            vec!["Link items found", "2"],
            vec!["Prose items found", "3"],
            vec!["Quality items found", "4"],
            vec!["Fair items found", "7"],
            vec!["Readability items found", "5"],
            vec!["Crosswalk items found", "6"],
        ]
    );
}
async fn assert_schema_issue_count<T>(path: &str, expected: usize) -> Result<()>
where
    T: Analysis + Send,
{
    let path = fixture_path(path);
    let checks = T::check_schema(core::slice::from_ref(&path), None).await;
    let issue_count = checks.iter().map(Check::issue_count).sum::<usize>();
    assert_eq!(issue_count, expected, "Unexpected schema issue count for {path:?}");
    Ok(())
}
async fn run_standard_check(path: &str, standard: Standard) -> Result<()> {
    let path = Some(fixture_path(path));
    let result = run(
        &path,
        &false,
        &None,
        &None,
        &[],
        &[],
        &[],
        &true,
        &false,
        &false,
        &false,
        &false,
        &true,
        &false,
        &false,
        &standard,
        &ReadabilityType::FKGL,
        &Verbosity::new(0, 1),
        OFFLINE,
        None,
        false,
        SignedDuration::from_secs(1),
        &None,
    )
    .await;
    assert!(result.is_ok(), "Check should succeed for {standard}: {result:?}");
    Ok(())
}
#[test]
fn test_check_accepts_piped_text() {
    let output = Command::cargo_bin("acorn")
        .expect("resolve acorn binary")
        .args([
            "--offline",
            "--no-local-database",
            "check",
            "--all",
            "--skip",
            "prose,link,crosswalk,schema,quality",
        ])
        .write_stdin("Text supplied through standard input. It has several words and another sentence.\n")
        .output()
        .expect("run check with standard input");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("stdin.txt"));
}
#[tokio::test]
async fn test_check_datacite_standard() -> Result<()> {
    run_standard_check("schema/datacite.json", Standard::Datacite).await
}
#[tokio::test]
async fn test_check_dcat_standard() -> Result<()> {
    run_standard_check("schema/dcat.json", Standard::Dcat).await
}
#[tokio::test]
async fn test_check_huwise_standard() -> Result<()> {
    run_standard_check("schema/huwise.json", Standard::Huwise).await
}
#[tokio::test]
async fn test_check_invalid_project_with_no_fail() -> Result<()> {
    let path = Some(fixture_path("data/invalid_project_a/index.json"));
    let result = run(
        &path,
        &false,
        &None,
        &None,
        &[],
        &[],
        &[],
        &true,
        &false,
        &false,
        &false,
        &false,
        &true,
        &false,
        &false,
        &Standard::default(),
        &ReadabilityType::FKGL,
        &Verbosity::new(0, 1),
        OFFLINE,
        None,
        false,
        SignedDuration::from_secs(1),
        &None,
    )
    .await;
    assert!(result.is_ok(), "Check with no_fail should return Ok even with failures");
    Ok(())
}
#[tokio::test]
async fn test_check_invenio_standard() -> Result<()> {
    run_standard_check("schema/invenio-single.json", Standard::Invenio).await
}
#[tokio::test]
async fn test_check_remote_project() -> Result<()> {
    let content = read(fixture_path("data/valid_project/index.json"))?;
    let (url, server) = spawn_source_server(content);
    let path = Some(PathBuf::from(url));
    let result = run(
        &path,
        &false,
        &None,
        &None,
        &[],
        &[],
        &[CheckCategory::Prose],
        &true,
        &false,
        &false,
        &false,
        &false,
        &true,
        &false,
        &false,
        &Standard::default(),
        &ReadabilityType::FKGL,
        &Verbosity::new(0, 1),
        false,
        None,
        false,
        SignedDuration::from_secs(1),
        &None,
    )
    .await;
    assert!(result.is_ok(), "Remote project should be checked: {result:?}");
    server.join().expect("join remote check server");
    Ok(())
}
#[tokio::test]
async fn test_check_remote_project_rejects_offline_mode() -> Result<()> {
    let path = Some(PathBuf::from("https://example.invalid/project.json"));
    let result = run(
        &path,
        &false,
        &None,
        &None,
        &[],
        &[],
        &[],
        &true,
        &false,
        &false,
        &false,
        &false,
        &true,
        &false,
        &false,
        &Standard::default(),
        &ReadabilityType::FKGL,
        &Verbosity::new(0, 1),
        true,
        None,
        false,
        SignedDuration::from_secs(1),
        &None,
    )
    .await;
    assert!(result.is_err());
    Ok(())
}
#[tokio::test]
async fn test_check_terse_mode() -> Result<()> {
    let path = Some(fixture_path("data/valid_project/index.json"));
    let result = run(
        &path,
        &false,
        &None,
        &None,
        &[],
        &[],
        &[],
        &true,
        &false,
        &false,
        &false,
        &false,
        &true,
        &false,
        &true, // terse mode
        &Standard::default(),
        &ReadabilityType::FKGL,
        &Verbosity::new(0, 1),
        OFFLINE,
        None,
        false,
        SignedDuration::from_secs(1),
        &None,
    )
    .await;
    assert!(result.is_ok(), "Check in terse mode should succeed");
    Ok(())
}
#[tokio::test]
async fn test_check_valid_highlight() -> Result<()> {
    let path = Some(fixture_path("data/valid_highlight/index.json"));
    let result = run(
        &path,
        &false,
        &None,
        &None,
        &[],
        &[],
        &[],
        &true,
        &false,
        &false,
        &false,
        &false,
        &true,
        &false,
        &false,
        &Standard::default(),
        &ReadabilityType::FKGL,
        &Verbosity::new(0, 1),
        OFFLINE,
        None,
        false,
        SignedDuration::from_secs(1),
        &None,
    )
    .await;
    assert!(result.is_ok(), "Valid highlight should pass check: {result:?}");
    Ok(())
}
#[tokio::test]
async fn test_check_valid_project() -> Result<()> {
    let path = Some(fixture_path("data/valid_project/index.json"));
    let result = run(
        &path,
        &false,
        &None,
        &None,
        &[],
        &[],
        &[],
        &true,
        &false,
        &false,
        &false,
        &false,
        &true,
        &false,
        &false,
        &Standard::default(),
        &ReadabilityType::FKGL,
        &Verbosity::new(0, 1),
        OFFLINE,
        None,
        false,
        SignedDuration::from_secs(1),
        &None,
    )
    .await;
    assert!(result.is_ok(), "Valid project should pass check");
    Ok(())
}
#[tokio::test]
async fn test_check_with_different_readability_metrics() -> Result<()> {
    let path = Some(fixture_path("data/valid_project/index.json"));
    // Test with different readability metrics
    for metric in [ReadabilityType::FKGL, ReadabilityType::ARI, ReadabilityType::CLI] {
        let result = run(
            &path,
            &false,
            &None,
            &None,
            &[],
            &[],
            &[],
            &true,
            &false,
            &false,
            &false,
            &false,
            &true,
            &false,
            &false,
            &Standard::default(),
            &metric,
            &Verbosity::new(0, 1),
            OFFLINE,
            None,
            false,
            SignedDuration::from_secs(1),
            &None,
        )
        .await;
        assert!(result.is_ok(), "Check with {:?} metric should succeed", metric);
    }
    Ok(())
}
#[tokio::test]
async fn test_check_with_skip_categories() -> Result<()> {
    let path = Some(fixture_path("data/valid_project/index.json"));
    let result = run(
        &path,
        &false,
        &None,
        &None,
        &[],
        &[],
        &[CheckCategory::Prose, CheckCategory::Schema],
        &true,
        &false,
        &false,
        &false,
        &false,
        &true,
        &false,
        &false,
        &Standard::default(),
        &ReadabilityType::FKGL,
        &Verbosity::new(0, 1),
        OFFLINE,
        None,
        false,
        SignedDuration::from_secs(1),
        &None,
    )
    .await;
    assert!(result.is_ok(), "Check with skip categories should succeed");
    Ok(())
}
#[tokio::test]
async fn test_datacite_invalid_fixture_has_two_schema_errors() -> Result<()> {
    assert_schema_issue_count::<datacite::Record>("schema/datacite-invalid-2-errors.json", 2).await
}
#[tokio::test]
async fn test_dcat_invalid_fixture_has_two_schema_errors() -> Result<()> {
    assert_schema_issue_count::<dcat::Dataset>("schema/dcat-invalid-2-errors.json", 2).await
}
#[tokio::test]
async fn test_huwise_invalid_fixture_has_two_schema_errors() -> Result<()> {
    assert_schema_issue_count::<huwise::Dataset>("schema/huwise-invalid-2-errors.json", 2).await
}
#[tokio::test]
async fn test_invenio_invalid_fixture_has_two_schema_errors() -> Result<()> {
    assert_schema_issue_count::<invenio::Record>("schema/invenio-invalid-2-errors.json", 2).await
}
#[test]
fn test_location_resolve_file_type_detects_extensionless_pdf_content() {
    let content = read(fixture_path("gather/resume.pdf")).unwrap();
    assert_eq!(
        acorn_core::Location::from("https://example.org/download").resolve_file_type(&content),
        MimeType::Pdf
    );
}
#[test]
fn test_text_input_format_is_inferred_and_cleaned_up() {
    [
        ("Text to check.\n".to_string(), "stdin.txt", Standard::Text),
        (
            read_to_string(fixture_path("data/invalid_project_a/index.json")).expect("read JSON fixture"),
            "stdin.json",
            Standard::ResearchActivityData,
        ),
        (
            read_to_string(fixture_path("data/valid_project/index.jsonc")).expect("read JSONC fixture"),
            "stdin.jsonc",
            Standard::ResearchActivityData,
        ),
        (
            read_to_string(fixture_path("data/invalid_project_b/index.yaml")).expect("read YAML fixture"),
            "stdin.yaml",
            Standard::ResearchActivityData,
        ),
        (
            read_to_string(fixture_path("data/invalid_project_c/index.md")).expect("read Markdown fixture"),
            "stdin.md",
            Standard::ResearchActivityData,
        ),
    ]
    .into_iter()
    .for_each(|(content, filename, standard)| {
        let prepared = TextOrigin::Stdin.prepare(&content, None).expect("prepare standard input");
        let path = prepared.value.paths[0].clone();
        let input = path.parent().expect("temporary input parent").to_path_buf();
        assert_eq!(path.file_name().and_then(|value| value.to_str()), Some(filename));
        assert_eq!(input.file_name().and_then(|value| value.to_str()), Some("input"));
        assert_eq!(read_to_string(&path).expect("read temporary input"), content);
        assert_eq!(classify_paths(core::slice::from_ref(&path), Standard::default())[0].standard, standard);
        drop(prepared.value);
        assert!(!input.exists());
    });
    [
        (MimeType::Cff, "clipboard.cff"),
        (MimeType::Json, "clipboard.json"),
        (MimeType::Jsonc, "clipboard.jsonc"),
        (MimeType::Markdown, "clipboard.md"),
        (MimeType::Text, "clipboard.txt"),
        (MimeType::Yaml, "clipboard.yaml"),
    ]
    .into_iter()
    .for_each(|(format, filename)| {
        let prepared = TextOrigin::Clipboard
            .prepare("forced format", Some(&format))
            .expect("prepare forced clipboard format");
        let path = prepared.value.paths[0].clone();
        let input = path.parent().expect("temporary input parent").to_path_buf();
        assert_eq!(path.file_name().and_then(|value| value.to_str()), Some(filename));
        drop(prepared.value);
        assert!(!input.exists());
    });
}
