use crate::cli::arguments::{CheckCategoryArgument, ReadabilityTypeArgument, Standard};
use crate::cli::{resolve_paths, CommandOptions};
use crate::commands::preflight;
use crate::io::{is_stdout_piped, write_stdout};
use acorn::analyzer::readability::ReadabilityType;
use acorn::analyzer::{checks_to_csv, summary, Analysis, Check, CheckCategory, CheckOptions};
use acorn::io::unique_file_extensions;
use acorn::prelude::{exit, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::schema::standard::cff::Cff;
use acorn::schema::standard::text::Text;
use acorn::util::constants::ENV_READABILITY_METRIC;
use acorn::util::{print_values_as_table, regex_join, Label};
use acorn::{fail, skip};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};
use color_eyre::owo_colors::OwoColorize;
use dotenvy::dotenv;
use tracing::{debug, error, warn};

#[allow(clippy::too_many_arguments)]
pub async fn run(
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    ignore: &[String],
    skip: &[CheckCategoryArgument],
    disable_website_checks: &bool,
    all: &bool,
    exit_on_first_error: &bool,
    merge_request: &bool,
    skip_verify_checksum: &bool,
    no_fail: &bool,
    raw: &bool,
    terse: &bool,
    _standard: &Standard,
    readability_metric: &ReadabilityTypeArgument,
    verbose: &Verbosity,
    offline: bool,
) -> Result<(), Report> {
    let is_silent = verbose.is_silent();
    let command_options = CommandOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .maybe_ignore(regex_join(ignore))
        .merge_request(*merge_request)
        .offline(offline)
        .quiet(is_silent)
        .supported(true)
        .build();
    preflight!(&command_options);
    let options = CheckOptions::init()
        .all(*all)
        .disable_website_checks(*disable_website_checks)
        .exit_on_first_error(*exit_on_first_error)
        .offline(offline)
        .no_fail(*no_fail)
        .quiet(is_silent || *raw)
        .skip(skip.iter().map(|value| value.to_string()).collect::<Vec<_>>())
        .skip_verify_checksum(*skip_verify_checksum)
        .terse(*terse)
        .readability_metric(resolve_readability_metric(readability_metric))
        .build();
    let paths = resolve_paths(path, &command_options).await;
    let issues = collect(&paths, &options)
        .await
        .iter()
        .filter(|issue| options.all || issue.severity.is_failure())
        .cloned()
        .collect::<Vec<Check>>();
    handle(&issues, &paths, &options)
}
async fn collect(paths: &[PathBuf], check_options: &CheckOptions) -> Vec<Check> {
    let extensions = unique_file_extensions(paths);
    let inferred_standard = if extensions.len() == 1 && extensions.contains(&"cff".to_string()) {
        warn!("=> {} Inferred standard (CFF)", Label::using());
        Standard::Cff
    } else if extensions.len() == 1 && extensions.contains(&"txt".to_string()) {
        warn!("=> {} Inferred standard (Text)", Label::using());
        Standard::Text
    } else {
        Standard::Rads
    };
    match inferred_standard {
        | Standard::Cff => collect_checks_for::<Cff>(paths, check_options).await,
        | Standard::Rads => collect_checks_for::<ResearchActivity>(paths, check_options).await,
        | Standard::Text => collect_checks_for::<Text>(paths, check_options).await,
        | _ => unimplemented!(),
    }
}
async fn collect_checks_for<T: Analysis>(paths: &[PathBuf], options: &CheckOptions) -> Vec<Check> {
    let is_skipped = |category: CheckCategory| -> bool { options.skip.iter().cloned().any(|value| CheckCategory::from(value) == category) };
    let maybe_early_exit = |results: Vec<Check>, issue_label: &str, exit_on_first_error: bool| -> Vec<Check> {
        let issue_count = results
            .iter()
            .filter(|Check { severity, .. }| severity.is_failure())
            .map(|issue| issue.issue_count())
            .sum::<usize>();
        if exit_on_first_error && issue_count > 0 {
            if !options.quiet {
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
    let schema_checks = if is_skipped(CheckCategory::Schema) {
        skip!("{} checks", CheckCategory::Schema);
        vec![]
    } else {
        maybe_early_exit(T::check_schema(paths), "schema", options.exit_on_first_error)
    };
    let prose_checks = if is_skipped(CheckCategory::Prose) {
        skip!("{} checks", CheckCategory::Prose);
        vec![]
    } else {
        maybe_early_exit(T::check_prose(paths, Some(options)).await, "prose", options.exit_on_first_error)
    };
    let readability_checks = if is_skipped(CheckCategory::Readability) {
        skip!("{} checks", CheckCategory::Readability);
        vec![]
    } else {
        maybe_early_exit(T::check_readability(paths, Some(options)), "readability", options.exit_on_first_error)
    };
    let quality_checks = if is_skipped(CheckCategory::Quality) {
        skip!("{} checks", CheckCategory::Quality);
        vec![]
    } else {
        vec![]
    };
    let website_checks = if is_skipped(CheckCategory::Link) || options.offline || options.disable_website_checks {
        skip!("{} checks", CheckCategory::Link);
        vec![]
    } else {
        T::check_websites(paths, None).await
    };
    schema_checks
        .into_iter()
        .chain(prose_checks)
        .chain(website_checks)
        .chain(readability_checks)
        .chain(quality_checks)
        .collect()
}
fn handle(issues: &[Check], paths: &[PathBuf], options: &CheckOptions) -> Result<(), Report> {
    let CheckOptions { quiet, no_fail, terse, .. } = options.clone();
    let print_compact = || {
        issues.iter().for_each(|issue| {
            let uri = issue.clone().uri.unwrap_or_default();
            println!("{issue}{}", format!(", path={uri}").dimmed());
        });
    };
    let print_detailed = || {
        issues.iter().enumerate().for_each(|(index, issue)| {
            issue.clone().with_index(index).report();
        });
    };
    let print_summary = || {
        let file_count = paths.len();
        let headers = vec!["Item", "Count"];
        let rows = (file_count > 1)
            .then(|| vec!["Files checked".to_string(), file_count.to_string()])
            .into_iter()
            .chain(summary(issues.to_vec()))
            .collect();
        print_values_as_table::<String>(headers, rows, None);
    };
    if !quiet {
        if terse {
            print_compact();
        } else {
            print_detailed();
        }
        if !(no_fail || terse) {
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
    } else if !no_fail && issues.iter().any(|Check { severity, .. }| severity.is_failure()) {
        exit(exitcode::DATAERR)
    } else {
        Ok(())
    }
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
