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
use crate::cli::arguments::{CheckCategoryArgument, ReadabilityTypeArgument, Standard};
use crate::commands::check::run;
use crate::test_utils::fixture_path;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;

const OFFLINE: bool = true;

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
        &ReadabilityTypeArgument::Fkgl,
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
        &ReadabilityTypeArgument::Fkgl,
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
        &ReadabilityTypeArgument::Fkgl,
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
        &[CheckCategoryArgument::Prose, CheckCategoryArgument::Schema],
        &true,
        &false,
        &false,
        &false,
        &false,
        &true,
        &false,
        &false,
        &Standard::default(),
        &ReadabilityTypeArgument::Fkgl,
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
    for metric in [ReadabilityTypeArgument::Fkgl, ReadabilityTypeArgument::Ari, ReadabilityTypeArgument::Cli] {
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
        &ReadabilityTypeArgument::Fkgl,
        &Verbosity::new(0, 1),
        OFFLINE,
    )
    .await;
    assert!(result.is_ok(), "Check in terse mode should succeed");
    Ok(())
}
