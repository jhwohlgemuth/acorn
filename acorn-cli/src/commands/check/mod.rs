use crate::cli::arguments::{CheckCategory, ReadabilityTypeArgument};
use crate::cli::{paths_from_options, CommandLineOptions};
use acorn::analyzer::readability::ReadabilityType;
use acorn::analyzer::{summary, Analysis, Check, Validation};
use acorn::prelude::PathBuf;
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{print_values_as_table, Label};
use acorn::{fail, skip};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};
use dotenvy::dotenv;
use std::process::exit;
use tracing::debug;

const READABILITY_METRIC: &str = "READABILITY_METRIC";

fn count(issues: Vec<Check>) -> usize {
    issues
        .into_iter()
        .filter(|Check { success, .. }| !success)
        .map(|issue| issue.issue_count())
        .sum()
}
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
    if *offline {
        println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
    }
    let options = CommandLineOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .maybe_ignore(ignore.clone())
        .exit_on_first_error(*exit_on_first_error)
        .merge_request(*merge_request)
        .build();
    let CommandLineOptions { exit_on_first_error, .. } = options;
    let paths = paths_from_options(path, &Some(options)).await;
    let is_offline = *offline || *disable_website_checks;
    let schema_issues = if skip.contains(&CheckCategory::Schema) {
        skip!("Schema validation");
        vec![]
    } else {
        let results = ResearchActivity::check(paths.clone());
        let issues = count(results.clone());
        if issues > 0 && exit_on_first_error {
            if level.is_some() {
                fail!("ACORN found {} schema validation errors", issues);
                results.iter().for_each(|issue| issue.clone().report());
            }
            exit(exitcode::DATAERR);
        }
        results
    };
    let prose_issues = if skip.contains(&CheckCategory::Prose) {
        skip!("Prose analysis");
        vec![]
    } else {
        let results = ResearchActivity::prose_analysis(paths.clone(), *offline, *skip_verify_checksum).await;
        let issues = count(results.clone());
        if issues > 0 && exit_on_first_error {
            if level.is_some() {
                fail!("ACORN found {} prose issues", issues);
                results.iter().for_each(|issue| issue.clone().report());
            }
            exit(exitcode::DATAERR);
        }
        results
    };
    let readability_issues = if skip.contains(&CheckCategory::Readability) {
        skip!("Readability analysis");
        vec![]
    } else {
        let metric_value_from_cli = ReadabilityType::from_string(&readability_metric.to_string());
        let metric = match dotenv() {
            | Ok(_) => match dotenvy::var(READABILITY_METRIC) {
                | Ok(value) if !value.is_empty() => {
                    debug!(value, "=> {} Readability metric from .env", Label::using());
                    ReadabilityType::from_string(&value)
                }
                | _ => metric_value_from_cli,
            },
            | _ => metric_value_from_cli,
        };
        let results = ResearchActivity::readability_analysis(paths.clone(), metric);
        let issues = count(results.clone());
        if issues > 0 && exit_on_first_error {
            if level.is_some() {
                fail!("ACORN found {} readability issues", issues);
                results.iter().for_each(|issue| issue.clone().report());
            }
            exit(exitcode::DATAERR);
        }
        results
    };
    let conventions_issues = if skip.contains(&CheckCategory::Conventions) {
        skip!("Conventions validation");
        vec![]
    } else {
        vec![]
    };
    let website_checks = ResearchActivity::check_websites(paths.clone(), is_offline).await;
    let all_issues = vec![]
        .into_iter()
        .chain(schema_issues.clone())
        .chain(prose_issues.clone())
        .chain(readability_issues.clone())
        .chain(conventions_issues.clone())
        .chain(website_checks.clone())
        .filter(|Check { success, .. }| !success)
        .collect::<Vec<Check>>();
    if *terse {
        print_compact(&all_issues);
    } else {
        print_detailed(&all_issues);
    }
    if !(*no_fail || *terse) {
        if level.is_some() {
            print_summary(&all_issues);
        }
        exit(exitcode::DATAERR);
    }
    Ok(())
}
fn print_detailed(issues: &[Check]) {
    issues.iter().for_each(|issue| {
        issue.clone().report();
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
