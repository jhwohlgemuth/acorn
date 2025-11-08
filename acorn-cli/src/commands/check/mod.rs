use crate::cli::arguments::{CheckCategory, ReadabilityTypeArgument};
use crate::cli::{paths_from_options, CommandLineOptions};
use acorn_lib::analyzer::readability::ReadabilityType;
use acorn_lib::analyzer::{summary, Check};
use acorn_lib::prelude::PathBuf;
use acorn_lib::schema::ResearchActivity;
use acorn_lib::util::{print_values_as_table, Label};
use color_eyre::eyre::{Report, Result};
use dotenvy::dotenv;
use std::process::exit;
use tracing::{debug, error, warn};

const READABILITY_METRIC: &str = "READABILITY_METRIC";

fn count(issues: Vec<Check>) -> usize {
    issues
        .into_iter()
        .filter(|Check { success, .. }| !success)
        .map(|issue| issue.issue_count())
        .sum()
}
#[allow(clippy::too_many_arguments)]
pub fn run(
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
    readability_metric: &ReadabilityTypeArgument,
) -> Result<(), Report> {
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
    let paths = paths_from_options(path, &Some(options));
    let schema_issues = if skip.contains(&CheckCategory::Schema) {
        warn!("=> {} Schema validation", Label::skip());
        vec![]
    } else {
        let results = ResearchActivity::check(paths.clone(), *offline || *disable_website_checks);
        let issues = count(results.clone());
        if issues > 0 && exit_on_first_error {
            error!("=> {} Found {} schema validation errors", Label::fail(), issues);
            results.iter().for_each(|issue| issue.clone().print());
            exit(exitcode::DATAERR);
        }
        results
    };
    let prose_issues = if skip.contains(&CheckCategory::Prose) {
        warn!("=> {} Prose analysis", Label::skip());
        vec![]
    } else {
        let results = ResearchActivity::analyze_prose(paths.clone(), *offline, *skip_verify_checksum);
        let issues = count(results.clone());
        if issues > 0 && exit_on_first_error {
            error!("=> {} Found {} prose issues", Label::fail(), issues);
            results.iter().for_each(|issue| issue.clone().print());
            exit(exitcode::DATAERR);
        }
        results
    };
    let readability_issues = if skip.contains(&CheckCategory::Readability) {
        warn!("=> {} Readability analysis", Label::skip());
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
        let results = ResearchActivity::calculate_readability(paths, metric);
        let issues = count(results.clone());
        if issues > 0 && exit_on_first_error {
            error!("=> {} Found {} readability issues", Label::fail(), issues);
            results.iter().for_each(|issue| issue.clone().print());
            exit(exitcode::DATAERR);
        }
        results
    };
    let conventions_issues = if skip.contains(&CheckCategory::Conventions) {
        warn!("=> {} Conventions verification", Label::skip());
        vec![]
    } else {
        vec![]
    };
    let all_issues = schema_issues
        .clone()
        .into_iter()
        .chain(prose_issues.clone())
        .chain(readability_issues.clone())
        .chain(conventions_issues.clone())
        .inspect(|issue| issue.clone().print())
        .filter(|Check { success, .. }| !success)
        .collect::<Vec<Check>>();
    if count(all_issues.clone()) > 0 {
        let headers = vec!["Issue Type", "Issue Count"];
        let rows = summary(all_issues);
        println!("\n");
        print_values_as_table(&Label::fmt_fail("Issues Overview"), headers, rows);
        exit(exitcode::DATAERR);
    }
    Ok(())
}
