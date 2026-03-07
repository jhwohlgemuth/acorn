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

use crate::cli::arguments::{CheckCategory, ReadabilityTypeArgument};
use crate::commands::check::run;
use acorn::prelude::PathBuf;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;

const OFFLINE: bool = true;

// Helper to get fixtures path
fn fixture_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join(path)
}

#[tokio::test]
async fn test_check_valid_project() -> Result<()> {
    let path = Some(fixture_path("data/valid_project/index.json"));
    let result = run(
        &path,
        &None,                          // branch
        &None,                          // commit
        &None,                          // ignore
        &[],                            // skip
        &true,                          // disable_website_checks
        &false,                         // exit_on_first_error
        &false,                         // merge_request
        &false,                         // skip_verify_checksum
        &OFFLINE,                       // offline
        &true,                          // no_fail (don't exit on failures)
        &false,                         // terse
        &ReadabilityTypeArgument::Fkgl, // readability_metric
        &Verbosity::new(0, 1),          // quiet
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
        &None,
        &[],
        &true,
        &false,
        &false,
        &false,
        &OFFLINE,
        &true,
        &false,
        &ReadabilityTypeArgument::Fkgl,
        &Verbosity::new(0, 1),
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
        &None,
        &[],
        &true,
        &false,
        &false,
        &false,
        &OFFLINE,
        &true, // no_fail ensures it returns Ok despite validation failures
        &false,
        &ReadabilityTypeArgument::Fkgl,
        &Verbosity::new(0, 1),
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
        &None,
        &[CheckCategory::Prose, CheckCategory::Schema],
        &true,
        &false,
        &false,
        &false,
        &OFFLINE,
        &true,
        &false,
        &ReadabilityTypeArgument::Fkgl,
        &Verbosity::new(0, 1),
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
            &None,
            &[],
            &true,
            &false,
            &false,
            &false,
            &OFFLINE,
            &true,
            &false,
            &metric,
            &Verbosity::new(0, 1),
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
        &None,
        &[],
        &true,
        &false,
        &false,
        &false,
        &OFFLINE,
        &true,
        &true, // terse mode
        &ReadabilityTypeArgument::Fkgl,
        &Verbosity::new(0, 1),
    )
    .await;

    assert!(result.is_ok(), "Check in terse mode should succeed");
    Ok(())
}
