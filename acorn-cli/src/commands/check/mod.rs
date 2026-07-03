use crate::cli::{resolve_paths, CommandOptions};
use crate::commands::preflight;
use crate::io::{is_stdout_piped, write_stdout};
use acorn::analyzer::readability::ReadabilityType;
use acorn::analyzer::{self, checks_to_csv, summary, Analysis, Check, CheckCategory, CheckOptions, Standard};
use acorn::io::unique_file_extensions;
use acorn::prelude::PathBuf;
use acorn::schema::pid::raid;
use acorn::schema::research_activity::ResearchActivity;
use acorn::schema::standard::cff::Cff;
use acorn::schema::standard::text::{Docx, Text};
use acorn::schema::standard::{datacite, dcat, huwise, invenio};
use acorn::util::constants::env::READABILITY_METRIC;
use acorn::util::{print_values_as_table, regex_join, suffix, Label};
use acorn::{fail, skip};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{eyre, Report, Result};
use color_eyre::owo_colors::OwoColorize;
use core::iter::once;
use dotenvy::dotenv;
use futures::stream::{self, StreamExt};
use strum::IntoEnumIterator;
use tracing::{debug, error, warn};

#[allow(clippy::too_many_arguments)]
pub async fn run(
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    filter: &[String],
    ignore: &[String],
    skip: &[analyzer::CheckCategory],
    disable_website_checks: &bool,
    all: &bool,
    exit_on_first_error: &bool,
    merge_request: &bool,
    skip_verify_checksum: &bool,
    no_fail: &bool,
    raw: &bool,
    terse: &bool,
    standard: &Standard,
    readability_metric: &ReadabilityType,
    verbose: &Verbosity,
    offline: bool,
) -> Result<(), Report> {
    let is_silent = verbose.is_silent();
    let command_options = CommandOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .maybe_filter(regex_join(filter))
        .maybe_ignore(regex_join(ignore))
        .merge_request(*merge_request)
        .offline(offline)
        .quiet(is_silent)
        .supported(true)
        .build();
    preflight!(&command_options);
    let result: Result<(), Report> = match resolve_paths(path, &command_options).await {
        | Ok(paths) => {
            let options = CheckOptions::init()
                .all(*all)
                .disable_website_checks(*disable_website_checks)
                .exit_on_first_error(*exit_on_first_error)
                .offline(offline)
                .no_fail(*no_fail)
                .quiet(is_silent || *raw)
                .skip(skip.iter().map(|value| value.to_string()).collect::<Vec<_>>())
                .skip_verify_checksum(*skip_verify_checksum)
                .standard(resolve_standard(&paths, standard))
                .terse(*terse)
                .readability_metric(resolve_readability_metric(readability_metric))
                .build();
            let issues = filter_by_visibility(&collect(&paths, &options).await, options.all);
            handle(&issues, &paths, &options)
        }
        | Err(why) => Err(why),
    };
    result
}
fn apply_early_exit_policy(results: Vec<Check>, category: &CheckCategory, options: &CheckOptions) -> Vec<Check> {
    let CheckOptions {
        all,
        exit_on_first_error,
        quiet,
        terse,
        ..
    } = options;
    let filtered_results = filter_by_visibility(&results, *all);
    let issue_count = failure_count(&filtered_results);
    if *exit_on_first_error && issue_count > 0 && !*quiet {
        fail!("ACORN found {} {} issues", issue_count, category);
        render(&filtered_results, *terse);
    }
    filtered_results
}
async fn collect(paths: &[PathBuf], check_options: &CheckOptions) -> Vec<Check> {
    match check_options.standard {
        | Standard::CitationFileFormat => collect_checks_for::<Cff>(paths, check_options).await,
        | Standard::Datacite => collect_checks_for::<datacite::Record>(paths, check_options).await,
        | Standard::Dcat => collect_checks_for::<dcat::Dataset>(paths, check_options).await,
        | Standard::Docx => collect_checks_for::<Docx>(paths, check_options).await,
        | Standard::Huwise => collect_checks_for::<huwise::Dataset>(paths, check_options).await,
        | Standard::Invenio => collect_checks_for::<invenio::Record>(paths, check_options).await,
        | Standard::ResearchActivityData => collect_checks_for::<ResearchActivity>(paths, check_options).await,
        | Standard::Raid => collect_checks_for::<raid::Metadata>(paths, check_options).await,
        | Standard::Text => collect_checks_for::<Text>(paths, check_options).await,
        | _ => {
            warn!("Unsupported standard for checks");
            vec![]
        }
    }
}
async fn collect_checks_for<T: Analysis + Send + Sync>(paths: &[PathBuf], options: &CheckOptions) -> Vec<Check> {
    let skipped_categories = resolve_skipped_categories(options);
    let (skipped, non_skipped): (Vec<CheckCategory>, Vec<CheckCategory>) = CheckCategory::iter().partition(|cat| skipped_categories.contains(cat));
    skipped.iter().for_each(|category| skip!("{} checks", category));
    stream::iter(non_skipped)
        .fold(Vec::new(), |acc, category| async move {
            let results = T::check(category.clone(), paths, Some(options)).await;
            let results = apply_early_exit_policy(results, &category, options);
            acc.into_iter().chain(results).collect()
        })
        .await
}
fn handle(issues: &[Check], paths: &[PathBuf], options: &CheckOptions) -> Result<(), Report> {
    let CheckOptions { quiet, no_fail, terse, .. } = options;
    let print_summary = || {
        let file_count = paths.len();
        let headers = vec!["", "Count"];
        let rows = (file_count > 1)
            .then(|| vec!["Files checked".to_string(), file_count.to_string()])
            .into_iter()
            .chain(summary(issues.to_vec()))
            .collect();
        print_values_as_table::<String>(headers, rows, None);
    };
    if !(*quiet) {
        render(issues, *terse);
        if !(*no_fail || *terse || issues.is_empty()) {
            print_summary();
        }
    }
    if is_stdout_piped() {
        match checks_to_csv(issues, false) {
            | Ok(csv) => {
                write_stdout(&csv);
                Ok(())
            }
            | Err(why) => {
                error!("=> {} Output CSV results to stdout — {why}", Label::fail());
                Err(why.into())
            }
        }
    } else if !*no_fail && has_failures(issues) {
        let count = issues.len();
        Err(eyre!("ACORN found {count} issue{}", suffix(count)))
    } else {
        Ok(())
    }
}
fn failure_count(issues: &[Check]) -> usize {
    issues.iter().filter(|issue| issue.is_failure()).map(Check::issue_count).sum::<usize>()
}
fn filter_by_visibility(issues: &[Check], all: bool) -> Vec<Check> {
    issues.iter().filter(|issue| all || issue.is_failure()).cloned().collect::<Vec<Check>>()
}
fn has_failures(issues: &[Check]) -> bool {
    issues.iter().any(Check::is_failure)
}
fn infer_standard(paths: &[PathBuf]) -> Standard {
    let extensions = unique_file_extensions(paths);
    if extensions.len() == 1 && extensions.contains(&"cff".to_string()) {
        warn!("=> {} Inferred standard (CFF)", Label::using());
        Standard::CitationFileFormat
    } else if extensions.len() == 1 && extensions.contains(&"docx".to_string()) {
        warn!("=> {} Inferred standard (DOCX)", Label::using());
        Standard::Docx
    } else if extensions.len() == 1 && extensions.contains(&"txt".to_string()) {
        warn!("=> {} Inferred standard (Text)", Label::using());
        Standard::Text
    } else {
        Standard::ResearchActivityData
    }
}
fn render(issues: &[Check], terse: bool) {
    let mut sorted_issues = issues.to_vec();
    sorted_issues.sort_by_cached_key(|issue| {
        (
            u8::from(&issue.severity),
            u8::from(&issue.category),
            issue.locator.clone().unwrap_or_default().to_lowercase(),
        )
    });
    if terse {
        sorted_issues.iter().for_each(|issue| {
            let uri = issue.uri.clone().unwrap_or_default();
            println!("{issue}{}", format!(", path={uri}").dimmed());
        });
    } else {
        sorted_issues.into_iter().enumerate().for_each(|(index, issue)| {
            issue.with_index(index).report();
        });
    }
}
fn resolve_readability_metric(readability_metric: &ReadabilityType) -> ReadabilityType {
    match dotenv() {
        | Ok(_) => match dotenvy::var(READABILITY_METRIC) {
            | Ok(value) if !value.is_empty() => {
                debug!(value, "=> {} Readability metric from .env", Label::using());
                ReadabilityType::from_string(&value)
            }
            | _ => *readability_metric,
        },
        | _ => *readability_metric,
    }
}
fn resolve_skipped_categories(options: &CheckOptions) -> Vec<CheckCategory> {
    let CheckOptions {
        disable_website_checks,
        offline,
        skip,
        ..
    } = options;
    skip.iter()
        .map(CheckCategory::from)
        .chain(once(CheckCategory::Quality))
        .chain((*offline || *disable_website_checks).then_some(CheckCategory::Link))
        .collect()
}
fn resolve_standard(paths: &[PathBuf], standard: &Standard) -> Standard {
    match standard {
        | Standard::ResearchActivityData => infer_standard(paths),
        | value => *value,
    }
}
#[cfg(test)]
mod tests;
