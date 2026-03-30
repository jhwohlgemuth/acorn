use crate::cli::arguments::{CheckCategory, ReadabilityTypeArgument};
use crate::cli::{paths_from_options, CommandLineOptions};
use acorn::analyzer::readability::ReadabilityType;
use acorn::analyzer::{summary, Analysis, Check, CheckOptions, Validation};
use acorn::prelude::PathBuf;
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{print_values_as_table, Label};
use acorn::{fail, skip};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};
use dotenvy::dotenv;
use std::process::exit;
use tracing::debug;

const ENV_READABILITY_METRIC: &str = "READABILITY_METRIC";

#[allow(clippy::too_many_arguments)]
pub async fn run(
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    ignore: &Option<String>,
    skip: &[CheckCategory],
    disable_website_checks: &bool,
    exit_on_first_error: &bool,
    merge_request: &bool,
    skip_verify_checksum: &bool,
    offline: &bool,
    no_fail: &bool,
    terse: &bool,
    readability_metric: &ReadabilityTypeArgument,
    verbose: &Verbosity,
) -> Result<(), Report> {
    let level = verbose.log_level();
    let not_quiet = level.is_some();
    if *offline {
        println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
    }
    let cli_options = CommandLineOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .maybe_ignore(ignore.clone())
        .merge_request(*merge_request)
        .build();
    let check_options = CheckOptions::init()
        .exit_on_first_error(*exit_on_first_error)
        .offline(*offline)
        .skip_verify_checksum(*skip_verify_checksum)
        .readability_metric(resolve_readability_metric(readability_metric))
        .build();
    let paths = paths_from_options(path, &Some(cli_options)).await;
    let schema_checks = if skip.contains(&CheckCategory::Schema) {
        skip!("Schema validation");
        vec![]
    } else {
        process_checks(
            ResearchActivity::check_schema(paths.clone()),
            "schema validation errors",
            *exit_on_first_error,
            not_quiet,
        )
    };
    let prose_checks = if skip.contains(&CheckCategory::Prose) {
        skip!("Prose analysis");
        vec![]
    } else {
        process_checks(
            ResearchActivity::check_prose(paths.clone(), Some(&check_options)).await,
            "prose issues",
            *exit_on_first_error,
            not_quiet,
        )
    };
    let readability_checks = if skip.contains(&CheckCategory::Readability) {
        skip!("Readability analysis");
        vec![]
    } else {
        process_checks(
            ResearchActivity::check_readability(paths.clone(), Some(&check_options)),
            "readability issues",
            *exit_on_first_error,
            not_quiet,
        )
    };
    let quality_checks = if skip.contains(&CheckCategory::Quality) {
        skip!("Quality control and data consistency checks");
        vec![]
    } else {
        vec![]
    };
    let website_checks = if *offline || *disable_website_checks {
        skip!("Website checks");
        vec![]
    } else {
        ResearchActivity::check_websites(paths.clone(), None).await
    };
    let all_issues = vec![]
        .into_iter()
        .chain(schema_checks)
        .chain(prose_checks)
        .chain(readability_checks)
        .chain(quality_checks)
        .chain(website_checks)
        .filter(|Check { success, .. }| !success)
        .collect::<Vec<Check>>();
    if not_quiet {
        if *terse {
            print_compact(&all_issues);
        } else {
            print_detailed(&all_issues);
        }
        if !(*no_fail || *terse) {
            print_summary(&all_issues);
        }
    }
    if !(all_issues.is_empty() || *no_fail) {
        exit(exitcode::DATAERR);
    }
    Ok(())
}
fn count_failed_issues(issues: &[Check]) -> usize {
    issues
        .iter()
        .filter(|Check { success, .. }| !success)
        .map(|issue| issue.issue_count())
        .sum()
}
fn print_detailed(issues: &[Check]) {
    issues.iter().for_each(|issue| {
        issue.report();
    });
}
fn print_compact(issues: &[Check]) {
    issues.iter().for_each(|issue| {
        println!("{issue}");
    });
}
fn print_summary(issues: &[Check]) {
    let headers = vec!["Issue Type", "Issue Count"];
    let rows = summary(issues.to_vec());
    print_values_as_table::<String>(headers, rows, None);
}
fn process_checks(results: Vec<Check>, issue_label: &str, exit_on_first_error: bool, not_quiet: bool) -> Vec<Check> {
    let issue_count = count_failed_issues(&results);
    if issue_count > 0 && exit_on_first_error {
        if not_quiet {
            fail!("ACORN found {} {}", issue_count, issue_label);
            results.iter().for_each(Check::report);
        }
        exit(exitcode::DATAERR);
    }
    results
}
fn resolve_readability_metric(readability_metric: &ReadabilityTypeArgument) -> ReadabilityType {
    let metric_from_cli = ReadabilityType::from_string(&readability_metric.to_string());
    match dotenv() {
        | Ok(_) => match dotenvy::var(ENV_READABILITY_METRIC) {
            | Ok(value) if !value.is_empty() => {
                debug!(value, "=> {} Readability metric from .env", Label::using());
                ReadabilityType::from_string(&value)
            }
            | _ => metric_from_cli,
        },
        | _ => metric_from_cli,
    }
}

#[cfg(test)]
mod tests;
