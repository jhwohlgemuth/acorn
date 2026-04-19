use crate::cli::arguments::{CheckCategory, ReadabilityTypeArgument, Standard};
use crate::cli::{paths_from_options, CommandLineOptions};
use crate::commands::preflight;
use acorn::analyzer::readability::ReadabilityType;
use acorn::analyzer::{summary, Analysis, Check, CheckOptions};
use acorn::io::unique_file_extensions;
use acorn::prelude::PathBuf;
use acorn::schema::research_activity::ResearchActivity;
use acorn::schema::standard::cff::Cff;
use acorn::schema::standard::text::Text;
use acorn::util::constants::ENV_READABILITY_METRIC;
use acorn::util::{print_values_as_table, Label};
use acorn::{fail, skip};
use clap_verbosity_flag::{log::Level, Verbosity};
use color_eyre::eyre::{Report, Result};
use color_eyre::owo_colors::OwoColorize;
use dotenvy::dotenv;
use std::process::exit;
use tracing::{debug, warn};

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
    raw: &bool,
    terse: &bool,
    standard: &Standard,
    readability_metric: &ReadabilityTypeArgument,
    verbose: &Verbosity,
) -> Result<(), Report> {
    let level = verbose.log_level();
    let is_silent = verbose.is_silent();
    preflight(*offline, is_silent, true, "check");
    let cli_options = CommandLineOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .maybe_ignore(ignore.clone())
        .merge_request(*merge_request)
        .build();
    let check_options = CheckOptions::init()
        .exit_on_first_error(*exit_on_first_error)
        .offline(*offline)
        .quiet(is_silent || *raw)
        .skip_verify_checksum(*skip_verify_checksum)
        .readability_metric(resolve_readability_metric(readability_metric))
        .build();
    let paths = paths_from_options(path, &Some(cli_options)).await;
    let extensions = unique_file_extensions(&paths);
    let inferred_standard = if extensions.len() == 1 && extensions.contains(&"cff".to_string()) {
        warn!("=> {} Inferred standard (CFF)", Label::using());
        Standard::Cff
    } else if extensions.len() == 1 && extensions.contains(&"txt".to_string()) {
        warn!("=> {} Inferred standard (Text)", Label::using());
        Standard::Text
    } else {
        *standard
    };
    let all_issues = match inferred_standard {
        | Standard::Cff => collect_failed_checks::<Cff>(&paths, skip, &check_options, *disable_website_checks, level).await,
        | Standard::Rads => collect_failed_checks::<ResearchActivity>(&paths, skip, &check_options, *disable_website_checks, level).await,
        | Standard::Text => collect_failed_checks::<Text>(&paths, skip, &check_options, *disable_website_checks, level).await,
        | _ => unimplemented!(),
    };
    if level.is_some() {
        if *terse {
            print_compact(&all_issues);
        } else {
            print_detailed(&all_issues);
        }
        if !(*no_fail || *terse) {
            print_summary(&all_issues, &paths);
        }
    }
    // let df = acorn::analyzer::checks_to_dataframe(&all_issues);
    // dbg!(df);
    if !*no_fail && all_issues.iter().any(|Check { severity, .. }| severity.is_failure()) {
        exit(exitcode::DATAERR);
    }
    Ok(())
}
async fn collect_failed_checks<T: Analysis>(
    paths: &[PathBuf],
    skip: &[CheckCategory],
    options: &CheckOptions,
    disable_website_checks: bool,
    level: Option<Level>,
) -> Vec<Check> {
    let maybe_early_exit = |results: Vec<Check>, issue_label: &str, exit_on_first_error: bool| -> Vec<Check> {
        let issue_count = results
            .iter()
            .filter(|Check { severity, .. }| severity.is_failure())
            .map(|issue| issue.issue_count())
            .sum::<usize>();
        if exit_on_first_error && issue_count > 0 {
            if level.is_some() {
                fail!("ACORN found {} {}", issue_count, issue_label);
                results
                    .iter()
                    .enumerate()
                    .for_each(|(index, issue)| issue.clone().with_index(index).report());
            }
            exit(exitcode::DATAERR);
        }
        results
    };
    let schema_checks = if skip.contains(&CheckCategory::Schema) {
        skip!("Schema validation");
        vec![]
    } else {
        maybe_early_exit(T::check_schema(paths), "schema validation errors", options.exit_on_first_error)
    };
    let prose_checks = if skip.contains(&CheckCategory::Prose) {
        skip!("Prose analysis");
        vec![]
    } else {
        maybe_early_exit(T::check_prose(paths, Some(options)).await, "prose issues", options.exit_on_first_error)
    };
    let readability_checks = if skip.contains(&CheckCategory::Readability) {
        skip!("Readability analysis");
        vec![]
    } else {
        maybe_early_exit(
            T::check_readability(paths, Some(options)),
            "readability issues",
            options.exit_on_first_error,
        )
    };
    let quality_checks = if skip.contains(&CheckCategory::Quality) {
        skip!("Quality control and data consistency checks");
        vec![]
    } else {
        vec![]
    };
    let website_checks = if options.offline || disable_website_checks {
        skip!("Website checks");
        vec![]
    } else {
        T::check_websites(paths, None).await
    };
    let should_be_visible = Check::is_visible_at(level.map(|l| l as u8));
    schema_checks
        .into_iter()
        .chain(prose_checks)
        .chain(website_checks)
        .chain(readability_checks)
        .chain(quality_checks)
        .filter(should_be_visible)
        .collect()
}
fn print_detailed(issues: &[Check]) {
    issues.iter().enumerate().for_each(|(index, issue)| {
        issue.clone().with_index(index).report();
    });
}
fn print_compact(issues: &[Check]) {
    issues.iter().for_each(|issue| {
        let uri = issue.clone().uri.unwrap_or_default();
        println!("{issue}{}", format!(", path={uri}").dimmed());
    });
}
fn print_summary(issues: &[Check], paths: &[PathBuf]) {
    let file_count = paths.len();
    let headers = vec!["Item", "Count"];
    let rows = (file_count > 1)
        .then(|| vec!["Files checked".to_string(), file_count.to_string()])
        .into_iter()
        .chain(summary(issues.to_vec()))
        .collect();
    print_values_as_table::<String>(headers, rows, None);
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
