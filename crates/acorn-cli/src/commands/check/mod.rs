use crate::cli::{resolve_paths, CommandOptions, WatchOptions};
use crate::commands::preflight;
use crate::commands::watch::{self, Prepared};
use crate::io::{is_stdout_piped, write_stdout};
use acorn::analyzer::readability::ReadabilityType;
use acorn::analyzer::vale::Vale;
use acorn::analyzer::{
    analyze_paths, checks_to_csv, summary, Check, CheckCategory, CheckOptions, Checks, OutputFormat, Render, Standard, StaticAnalyzer,
};
use acorn::io::{standard_project_folder, Fingerprint, Source};
use acorn::prelude::{remove_file, temp_dir, write, Path, PathBuf};
use acorn::util::constants::env::READABILITY_METRIC;
use acorn::util::{print_values_as_table, regex_join, suffix, Label};
use acorn::{fail, skip, Location};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{eyre, Report, Result};
use dotenvy::dotenv;
use jiff::SignedDuration;
use tracing::{debug, error};

#[allow(clippy::too_many_arguments)]
pub async fn run(
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    filter: &[String],
    ignore: &[String],
    skip: &[CheckCategory],
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
    watch: Option<WatchOptions>,
    ignore_sync_failure: bool,
    sync_retry_interval: SignedDuration,
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
    let input = path
        .as_ref()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| "./".to_string());
    let location = Location::from(input.as_str());
    let options = CheckOptions::init()
        .all(*all)
        .disable_website_checks(*disable_website_checks)
        .exit_on_first_error(*exit_on_first_error)
        .offline(offline)
        .no_fail(*no_fail)
        .format(OutputFormat::Console)
        .ignore_sync_failure(ignore_sync_failure)
        .quiet(is_silent || *raw)
        .skip(skip.iter().map(|value| value.to_string()).collect::<Vec<_>>())
        .skip_verify_checksum(*skip_verify_checksum)
        .standard(*standard)
        .terse(*terse)
        .sync_retry_interval(sync_retry_interval)
        .readability_metric(resolve_readability_metric(readability_metric))
        .build();
    match watch {
        | Some(watch) => watch_run(&location, &command_options, &options, watch).await,
        | None => match sync_analyzer_before_check(&options).await {
            | Ok(run_options) => match prepare(&location, &command_options).await {
                | Ok(prepared) => execute(prepared.value, &run_options).await,
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        },
    }
}
struct Input {
    paths: Vec<PathBuf>,
    temporary: Option<PathBuf>,
}
impl Drop for Input {
    fn drop(&mut self) {
        self.temporary.iter().for_each(|path| {
            let _ = remove_file(path);
        });
    }
}
async fn prepare(location: &Location, options: &CommandOptions) -> Result<Prepared<Input>, Report> {
    let scheme = location.scheme();
    match location {
        | _ if scheme.is_http() || scheme.is_https() => {
            let uri = location.uri().unwrap_or_default();
            Source::read_bytes(&uri, options.offline).await.and_then(|content| {
                let fingerprint = Fingerprint::from_entries([(uri, content.clone())]);
                let location_path = location.path();
                let extension = location_path
                    .as_deref()
                    .and_then(|path| Path::new(path).extension())
                    .and_then(|value| value.to_str())
                    .unwrap_or("json");
                let path = standard_project_folder("check", Some(temp_dir())).with_extension(extension);
                write(&path, content)
                    .map(|()| {
                        Prepared::new(
                            Input {
                                paths: vec![path.clone()],
                                temporary: Some(path.clone()),
                            },
                            fingerprint,
                        )
                    })
                    .map_err(|why| eyre!("Materialize remote check source at {} — {why}", path.display()))
            })
        }
        | _ if scheme.is_file() => resolve_paths(&Some(location.uri_as_path()), options)
            .await
            .and_then(|paths| Fingerprint::from_files(paths.clone()).map(|fingerprint| Prepared::new(Input { paths, temporary: None }, fingerprint))),
        | Location::Simple(value) if scheme.is_unsupported() => resolve_paths(&Some(PathBuf::from(value)), options)
            .await
            .and_then(|paths| Fingerprint::from_files(paths.clone()).map(|fingerprint| Prepared::new(Input { paths, temporary: None }, fingerprint))),
        | Location::Detailed { uri, .. } => Err(eyre!("Unsupported check source URI scheme — {uri}")),
        | Location::Simple(value) => Err(eyre!("Unsupported check source — {value}")),
    }
}
async fn execute(input: Input, options: &CheckOptions) -> Result<(), Report> {
    let issues = filter_by_visibility(&collect(&input.paths, options).await, options.all);
    handle(&issues, &input.paths, options)
}
async fn watch_run(location: &Location, command_options: &CommandOptions, options: &CheckOptions, watch: WatchOptions) -> Result<(), Report> {
    let scheme = location.scheme();
    let network = scheme.is_http() || scheme.is_https() || command_options.merge_request;
    let roots = match location {
        | _ if scheme.is_file() => vec![location.uri_as_path()],
        | Location::Simple(value) if scheme.is_unsupported() => vec![PathBuf::from(value)],
        | _ => Vec::new(),
    };
    match sync_analyzer_before_check(options).await {
        | Ok(run_options) => {
            watch::run(
                "check",
                &roots,
                WatchOptions { network, ..watch },
                || prepare(location, command_options),
                |input| execute(input, &run_options),
            )
            .await
        }
        | Err(why) => Err(why),
    }
}
async fn sync_analyzer_before_check(options: &CheckOptions) -> Result<CheckOptions> {
    let should_sync = !options.skip.iter().any(|category| CheckCategory::from(category) == CheckCategory::Prose);
    let sync_result = match should_sync {
        | false => Ok(()),
        | true => {
            options
                .resolve_analyzer::<Vale, _>(options.vale_config.clone().unwrap_or_default())
                .await
                .retry_sync(options)
                .await
        }
    };
    match sync_result {
        | Ok(()) => Ok(CheckOptions {
            skip_sync: true,
            ..options.clone()
        }),
        | Err(why) => Err(why),
    }
}
fn apply_early_exit_policy(results: Vec<Check>, category: &CheckCategory, options: &CheckOptions) -> Vec<Check> {
    let CheckOptions {
        all,
        exit_on_first_error,
        quiet,
        ..
    } = options;
    let filtered_results = filter_by_visibility(&results, *all);
    let issue_count = failure_count(&filtered_results);
    if *exit_on_first_error && issue_count > 0 && !*quiet {
        fail!("ACORN found {} {} issues", issue_count, category);
        filtered_results.render(options);
    }
    filtered_results
}
async fn collect(paths: &[PathBuf], options: &CheckOptions) -> Vec<Check> {
    let report = analyze_paths(paths, options).await;
    report.skipped_categories.iter().for_each(|category| skip!("{} checks", category));
    report
        .batches
        .into_iter()
        .flat_map(|batch| batch.categories)
        .flat_map(|category| apply_early_exit_policy(category.checks, &category.category, options))
        .collect()
}
fn handle(issues: &Checks, paths: &[PathBuf], options: &CheckOptions) -> Result<(), Report> {
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
        issues.render(options);
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
#[cfg(test)]
mod tests;
