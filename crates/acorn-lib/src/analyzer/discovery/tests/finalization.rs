use super::super::*;
use crate::check_err;
use std::fs::read_to_string;
use tempfile::tempdir;

fn policy<'a>(format: Option<OutputFormat>, output: &'a Option<PathBuf>, quiet: bool, terse: bool, verbosity: Option<u8>) -> GatherOutputPolicy<'a> {
    GatherOutputPolicy {
        format,
        output,
        quiet,
        terse,
        verbosity,
    }
}
fn report(checks: &[Check]) -> Report {
    Report::new(checks, Records::default(), 1)
}

#[test]
fn test_finalize_report_check_and_additional_failures_are_independent() {
    let output = None;
    let failing = [check_err!(CheckCategory::Link, message: "failed")];
    for (checks, additional_failure, expected_failure) in [
        (failing.as_slice(), false, true),
        ([].as_slice(), true, true),
        (failing.as_slice(), true, true),
        ([].as_slice(), false, false),
    ] {
        let result = finalize_report(
            checks,
            &report(checks),
            policy(Some(OutputFormat::Json), &output, true, false, None),
            additional_failure,
        );
        assert_eq!(result.is_err(), expected_failure);
    }
}
#[test]
fn test_finalize_report_explicit_format_overrides_terminal_inference() {
    let output = None;
    assert_eq!(
        policy(Some(OutputFormat::Markdown), &output, false, false, None).format(),
        OutputFormat::Markdown
    );
}
#[test]
fn test_finalize_report_output_write_error_precedes_analysis_failure() {
    let directory = tempdir().expect("temporary directory should be created");
    let output = Some(directory.path().to_path_buf());
    let error = finalize_report(&[], &report(&[]), policy(Some(OutputFormat::Json), &output, true, false, None), true)
        .expect_err("directory output should fail");
    assert!(error.downcast_ref::<OutputFailure>().is_none());
}
#[test]
fn test_finalize_report_serializes_json_and_markdown_files() {
    let directory = tempdir().expect("temporary directory should be created");
    for (format, extension, expected) in [
        (OutputFormat::Json, "json", "\"summary\""),
        (OutputFormat::Markdown, "md", "# ACORN gather"),
    ] {
        let path = directory.path().join(format!("report.{extension}"));
        let output = Some(path.clone());
        finalize_report(&[], &report(&[]), policy(Some(format), &output, true, false, None), false).expect("report should finalize");
        assert!(read_to_string(path).expect("report should be readable").contains(expected));
    }
}
#[test]
fn test_finalize_report_silent_failure_respects_raw_verbosity_boundary() {
    for (format, verbosity, expected_silent) in [
        (OutputFormat::Raw, None, true),
        (OutputFormat::Raw, Some(0), true),
        (OutputFormat::Raw, Some(1), true),
        (OutputFormat::Raw, Some(2), false),
        (OutputFormat::Json, None, false),
    ] {
        let output = None;
        let error = finalize_report(&[], &report(&[]), policy(Some(format), &output, true, false, verbosity), true)
            .expect_err("additional failure should fail finalization");
        assert_eq!(
            error.downcast_ref::<OutputFailure>().map(|value| value.is_silent()),
            Some(expected_silent)
        );
    }
}
#[test]
fn test_finalize_report_terse_console_skips_output_file_and_preserves_failure() {
    let directory = tempdir().expect("temporary directory should be created");
    let path = directory.path().join("report.txt");
    let output = Some(path.clone());
    let error = finalize_report(&[], &report(&[]), policy(Some(OutputFormat::Console), &output, false, true, None), true)
        .expect_err("additional failure should fail finalization");
    assert!(error.downcast_ref::<OutputFailure>().is_some());
    assert!(!path.exists());
}
