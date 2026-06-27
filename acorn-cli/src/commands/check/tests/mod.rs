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
use crate::commands::check::run;
use crate::test::util::fixture_path;
use acorn::analyzer::readability::ReadabilityType;
use acorn::analyzer::{Analysis, Check, CheckCategory, Standard};
use acorn::schema::standard::{datacite, dcat, huwise, invenio};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;

const OFFLINE: bool = true;

async fn run_standard_check(path: &str, standard: Standard) -> Result<()> {
    let path = Some(fixture_path(path));
    let result = run(
        &path,
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
    )
    .await;
    assert!(result.is_ok(), "Check should succeed for {standard}");
    Ok(())
}
async fn assert_schema_issue_count<T>(path: &str, expected: usize) -> Result<()>
where
    T: Analysis,
{
    let path = fixture_path(path);
    let checks = T::check_schema(core::slice::from_ref(&path), None).await;
    let issue_count = checks.iter().map(Check::issue_count).sum::<usize>();
    assert_eq!(issue_count, expected, "Unexpected schema issue count for {path:?}");
    Ok(())
}

#[tokio::test]
async fn test_check_valid_project() -> Result<()> {
    let path = Some(fixture_path("data/valid_project/index.json"));
    let result = run(
        &path,
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
    )
    .await;
    assert!(result.is_ok(), "Valid project should pass check");
    Ok(())
}
#[tokio::test]
async fn test_check_valid_highlight() -> Result<()> {
    let path = Some(fixture_path("data/valid_highlight/index.json"));
    let result = run(
        &path,
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
    )
    .await;
    assert!(result.is_ok(), "Valid highlight should pass check");
    Ok(())
}
#[tokio::test]
async fn test_check_invalid_project_with_no_fail() -> Result<()> {
    let path = Some(fixture_path("data/invalid_project_a/index.json"));
    let result = run(
        &path,
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
    )
    .await;
    assert!(result.is_ok(), "Check with no_fail should return Ok even with failures");
    Ok(())
}
#[tokio::test]
async fn test_check_with_skip_categories() -> Result<()> {
    let path = Some(fixture_path("data/valid_project/index.json"));
    let result = run(
        &path,
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
    )
    .await;
    assert!(result.is_ok(), "Check with skip categories should succeed");
    Ok(())
}
#[tokio::test]
async fn test_check_with_different_readability_metrics() -> Result<()> {
    let path = Some(fixture_path("data/valid_project/index.json"));
    // Test with different readability metrics
    for metric in [ReadabilityType::FKGL, ReadabilityType::ARI, ReadabilityType::CLI] {
        let result = run(
            &path,
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
        )
        .await;
        assert!(result.is_ok(), "Check with {:?} metric should succeed", metric);
    }
    Ok(())
}
#[tokio::test]
async fn test_check_terse_mode() -> Result<()> {
    let path = Some(fixture_path("data/valid_project/index.json"));
    let result = run(
        &path,
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
    )
    .await;
    assert!(result.is_ok(), "Check in terse mode should succeed");
    Ok(())
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
async fn test_check_invenio_standard() -> Result<()> {
    run_standard_check("schema/invenio-single.json", Standard::Invenio).await
}
#[tokio::test]
async fn test_check_huwise_standard() -> Result<()> {
    run_standard_check("schema/huwise.json", Standard::Huwise).await
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
async fn test_invenio_invalid_fixture_has_two_schema_errors() -> Result<()> {
    assert_schema_issue_count::<invenio::Record>("schema/invenio-invalid-2-errors.json", 2).await
}
#[tokio::test]
async fn test_huwise_invalid_fixture_has_two_schema_errors() -> Result<()> {
    assert_schema_issue_count::<huwise::Dataset>("schema/huwise-invalid-2-errors.json", 2).await
}
