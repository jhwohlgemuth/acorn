use crate::{
    cli::{
        arguments::{CheckOutputFormat, InferenceArgs},
        resolve_paths_including, CommandOptions, WatchOptions,
    },
    commands::{
        inference::ConfiguredInference,
        preflight,
        watch::{self, Prepared},
    },
    io::{is_stdout_piped, read_stdin, write_stdout},
};
#[cfg(feature = "exec")]
use acorn::exec::config::ExecutionOrigin;
#[cfg(feature = "exec")]
use acorn::io::workflow::{Workflow, WorkflowAction};
use acorn::{
    analyzer::{
        diff::DiffFormat,
        fix::FixReport,
        host::discovery::Record,
        host::{
            context::{AnalysisContext, AnalysisOptions, ExecutionExtension, ExecutionOptions, OutputFormat, ReportExtension, ReportOptions},
            render::{Render, RenderContext},
            runtime::StaticAnalyzer,
            service::{analyze_paths, AnalysisReport, ExecutionStatus},
            tabular::checks_to_csv,
            vale::{Vale, ValeConfig},
        },
    },
    fail,
    io::{
        config::application::{ApplicationConfiguration, Hook},
        read_clipboard, select_source, standard_project_folder, Fingerprint, InputSource, Source, TextOrigin,
    },
    skip,
    util::{constants::env::READABILITY_METRIC, print_values_as_table},
    LocationExt,
};
use acorn_analysis::readability::ReadabilityType;
use acorn_analysis::{Check, CheckBatch, CheckCategory, CheckSummary, FairProfile};
use acorn_core::{
    options::CommonRuntime,
    util::{regex_join, suffix, MimeType},
    Location,
};
use acorn_host::terminal::Label;
#[cfg(feature = "exec")]
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::standard::Standard;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{eyre, Report, Result};
use dotenvy::dotenv;
use jiff::SignedDuration;
use serde::Serialize;
use std::{
    env::temp_dir,
    fs::{create_dir_all, remove_dir_all, remove_file, write},
    path::{Path, PathBuf},
};
use tracing::{debug, error};
trait PrepareLocation {
    async fn prepare(&self, options: &CommandOptions) -> Result<Prepared<Input>, Report>;
}
trait PrepareText {
    fn prepare(self, content: &str, format: Option<&MimeType>) -> Result<Prepared<Input>, Report>;
}
enum Temporary {
    Directory(PathBuf),
    File(PathBuf),
}
#[derive(Serialize)]
struct CheckJsonOutput<'a> {
    checks: Vec<Record>,
    fixes: &'a FixReport,
}
#[derive(Clone, Copy)]
struct HookConfiguration<'a> {
    application: &'a ApplicationConfiguration,
    explicit: bool,
    project_root: &'a Path,
}
struct Input {
    paths: Vec<PathBuf>,
    temporary: Option<Temporary>,
}
#[derive(Clone)]
struct SuggestionConfiguration {
    enabled: bool,
    inference: InferenceArgs,
    offline: bool,
    output: CheckOutputFormat,
}
impl Drop for Input {
    fn drop(&mut self) {
        self.temporary.iter().for_each(|temporary| match temporary {
            | Temporary::Directory(path) => {
                let _ = remove_dir_all(path);
            }
            | Temporary::File(path) => {
                let _ = remove_file(path);
            }
        });
    }
}
impl PrepareLocation for Location {
    async fn prepare(&self, options: &CommandOptions) -> Result<Prepared<Input>, Report> {
        let scheme = self.scheme();
        match self {
            | _ if scheme.is_http() || scheme.is_https() => {
                let uri = self.uri().unwrap_or_default();
                Source::read_bytes(&uri, options.offline).await.and_then(|content| {
                    let fingerprint = Fingerprint::from_entries([(uri, content.clone())]);
                    let extension = self.resolve_file_type(&content);
                    let path = standard_project_folder("check", Some(temp_dir())).with_extension(extension);
                    write(&path, content)
                        .map(|()| {
                            Prepared::new(
                                Input {
                                    paths: vec![path.clone()],
                                    temporary: Some(Temporary::File(path.clone())),
                                },
                                fingerprint,
                            )
                        })
                        .map_err(|why| eyre!("Materialize remote check source at {} — {why}", path.display()))
                })
            }
            | _ if scheme.is_file() => resolve_paths_including(&Some(self.uri_as_path()), options, &["Kitfile"])
                .await
                .and_then(|paths| {
                    Fingerprint::from_files(paths.clone()).map(|fingerprint| Prepared::new(Input { paths, temporary: None }, fingerprint))
                }),
            | Location::Simple(value) if scheme.is_unsupported() => resolve_paths_including(&Some(PathBuf::from(value)), options, &["Kitfile"])
                .await
                .and_then(|paths| {
                    Fingerprint::from_files(paths.clone()).map(|fingerprint| Prepared::new(Input { paths, temporary: None }, fingerprint))
                }),
            | Location::Detailed { uri, .. } => Err(eyre!("Unsupported check source URI scheme — {uri}")),
            | Location::Simple(value) => Err(eyre!("Unsupported check source — {value}")),
        }
    }
}
impl PrepareText for TextOrigin {
    fn prepare(self, content: &str, format: Option<&MimeType>) -> Result<Prepared<Input>, Report> {
        let root = standard_project_folder("check", Some(temp_dir()));
        let input = root.join("input");
        let format = format.cloned().unwrap_or_else(|| MimeType::infer_from(content));
        let path = input.join(self.filename(&format));
        let cleanup = || {
            let _ = remove_dir_all(&root);
        };
        match create_dir_all(&input).and_then(|()| write(&path, content)) {
            | Ok(()) => Ok(Prepared::new(
                Input {
                    paths: vec![path],
                    temporary: Some(Temporary::Directory(root)),
                },
                Fingerprint::from_entries([(self.label(), content.as_bytes())]),
            )),
            | Err(why) => {
                cleanup();
                Err(eyre!("Materialize {} check source — {why}", self.label()))
            }
        }
    }
}
fn apply_early_exit_policy(results: CheckBatch, category: &CheckCategory, context: RenderContext<'_>) -> CheckBatch {
    let ReportExtension {
        all, exit_on_first_error, ..
    } = &context.options.extension;
    let quiet = context.options.common.quiet;
    let filtered_results = filter_by_visibility(results.as_slice(), *all);
    let issue_count = failure_count(filtered_results.as_slice());
    if *exit_on_first_error && issue_count > 0 && !quiet {
        fail!("Found {issue_count} {category} issues");
        filtered_results.as_slice().render(context);
    }
    filtered_results
}
#[allow(clippy::too_many_arguments)]
fn build_analysis_context(
    skip: &[CheckCategory],
    disable_website_checks: bool,
    all: bool,
    exit_on_first_error: bool,
    skip_verify_checksum: bool,
    no_fail: bool,
    raw: bool,
    terse: bool,
    standard: Standard,
    readability_metric: ReadabilityType,
    silent: bool,
    offline: bool,
    ignore_sync_failure: bool,
    sync_retry_interval: SignedDuration,
    vale_directory: Option<PathBuf>,
) -> AnalysisContext {
    let analysis = AnalysisOptions::init()
        .readability_metric(readability_metric)
        .maybe_readability_threshold(resolve_readability_threshold(readability_metric))
        .skip(skip.iter().map(|value| value.to_string()).collect::<Vec<_>>())
        .standard(standard)
        .build();
    let execution = ExecutionOptions::init()
        .common(CommonRuntime {
            offline,
            quiet: silent || raw,
            ..Default::default()
        })
        .extension(
            ExecutionExtension::init()
                .disable_website_checks(disable_website_checks)
                .ignore_sync_failure(ignore_sync_failure)
                .skip_verify_checksum(skip_verify_checksum)
                .sync_retry_interval(sync_retry_interval)
                .maybe_vale_config(vale_directory.map(|directory| ValeConfig::default().with_directory(directory)))
                .build(),
        )
        .build();
    let report = ReportOptions::init()
        .common(CommonRuntime {
            quiet: silent || raw,
            ..Default::default()
        })
        .extension(
            ReportExtension::init()
                .all(all)
                .exit_on_first_error(exit_on_first_error)
                .format(OutputFormat::Text)
                .no_fail(no_fail)
                .terse(terse)
                .build(),
        )
        .build();
    AnalysisContext::init().analysis(analysis).execution(execution).report(report).build()
}
fn collect(report: &AnalysisReport, options: &AnalysisContext) -> CheckBatch {
    report.skipped_categories.iter().for_each(|category| skip!("{} checks", category));
    report.batches.iter().for_each(|batch| {
        batch.categories.iter().for_each(|category| match &category.status {
            | ExecutionStatus::Executed => {}
            | ExecutionStatus::Skipped { reason } | ExecutionStatus::Unsupported { reason } => {
                skip!("{} {} checks — {}", batch.standard, category.category, reason);
            }
        });
    });
    report
        .batches
        .iter()
        .flat_map(|batch| batch.categories.iter())
        .flat_map(|category| apply_early_exit_policy(category.checks.clone(), &category.category, report.render_context(&options.report)))
        .collect()
}
#[cfg(feature = "exec")]
async fn configured_checks(
    paths: &[PathBuf],
    options: &AnalysisContext,
    configuration: &ApplicationConfiguration,
    project_root: &Path,
    explicit_configuration: bool,
) -> Result<Vec<Check>, Report> {
    let should_validate = !options
        .analysis
        .skip
        .iter()
        .any(|category| CheckCategory::from(category) == CheckCategory::Schema);
    let lifecycle_hooks = [
        configuration.has_hook(Hook::Check).then_some(WorkflowAction::Check),
        (should_validate && configuration.has_hook(Hook::Validate)).then_some(WorkflowAction::Validate),
    ];
    let hooks = lifecycle_hooks.into_iter().flatten().collect::<Vec<_>>();
    match hooks.is_empty() {
        | true => Ok(Vec::new()),
        | false => {
            let sources = paths.iter().map(|path| Location::from(&path.display().to_string())).collect::<Vec<_>>();
            Workflow::<ResearchActivity>::plan(&sources, Default::default(), None)
                .await
                .and_then(|plan| {
                    configuration.check_script_hooks(
                        &plan.changes,
                        project_root,
                        &hooks,
                        ExecutionOrigin::Local,
                        explicit_configuration,
                        options.execution.common.offline,
                    )
                })
        }
    }
}
async fn execute(
    input: Input,
    options: &AnalysisContext,
    configuration: &ApplicationConfiguration,
    project_root: &Path,
    explicit_configuration: bool,
    suggestions: &SuggestionConfiguration,
) -> Result<(), Report> {
    let report = analyze_paths(&input.paths, options).await;
    let fair_profiles = Vec::<FairProfile>::from(&report);
    let collected = collect(&report, options);
    #[cfg(feature = "exec")]
    let collected = configured_checks(&input.paths, options, configuration, project_root, explicit_configuration)
        .await
        .map(|checks| collected.combine(checks.into()));
    #[cfg(not(feature = "exec"))]
    let collected = {
        let _ = (configuration, explicit_configuration, project_root);
        Ok::<CheckBatch, Report>(collected)
    };
    match collected {
        | Err(why) => Err(why),
        | Ok(collected) => {
            let issues = filter_by_visibility(collected.as_slice(), options.report.extension.all);
            let fixes = match suggestions.enabled {
                | false => Ok(FixReport::default()),
                | true => match ConfiguredInference::from_args(&suggestions.inference, suggestions.offline) {
                    | Err(why) => Err(why),
                    | Ok(configured) => configured.suggest_fixes(&report).await,
                },
            };
            match (suggestions.output, fixes) {
                | (CheckOutputFormat::Diff, Ok(fixes)) => fixes.build_preview(&report, input.temporary.is_some()).render(
                    &issues,
                    options,
                    report.render_context(&options.report),
                    DiffFormat::Unified,
                ),
                | (CheckOutputFormat::JsonPatch, Ok(fixes)) => fixes.build_preview(&report, input.temporary.is_some()).render(
                    &issues,
                    options,
                    report.render_context(&options.report),
                    DiffFormat::JsonPatch,
                ),
                | (CheckOutputFormat::Diff | CheckOutputFormat::Json | CheckOutputFormat::JsonPatch, Err(why)) => Err(why),
                | (CheckOutputFormat::Json, Ok(fixes)) => handle_json(&issues, &fixes, options),
                | (CheckOutputFormat::Human, Ok(fixes)) => {
                    let checked = handle(&issues, &fair_profiles, &input.paths, options, report.render_context(&options.report));
                    print!("{fixes}");
                    checked
                }
                | (CheckOutputFormat::Human, Err(why)) => {
                    let checked = handle(&issues, &fair_profiles, &input.paths, options, report.render_context(&options.report));
                    checked.and(Err(why))
                }
            }
        }
    }
}
fn failure_count(issues: &[Check]) -> usize {
    issues.iter().filter(|issue| issue.is_failure()).map(Check::issue_count).sum::<usize>()
}
fn filter_by_visibility(issues: &[Check], all: bool) -> CheckBatch {
    issues.iter().filter(|issue| all || issue.is_failure()).cloned().collect()
}
fn handle(
    issues: &CheckBatch,
    profiles: &[FairProfile],
    paths: &[PathBuf],
    options: &AnalysisContext,
    render_context: RenderContext<'_>,
) -> Result<(), Report> {
    let quiet = options.report.common.quiet;
    let ReportExtension { no_fail, terse, .. } = &options.report.extension;
    let print_summary = || {
        let file_count = paths.len();
        let headers = vec!["", "Count"];
        let rows = (file_count > 1)
            .then(|| vec!["Files checked".to_string(), file_count.to_string()])
            .into_iter()
            .chain(CheckSummary::from(issues).rows())
            .collect();
        print_values_as_table::<String>(headers, rows, None);
    };
    if !quiet {
        issues.as_slice().render(render_context);
        if !*terse && !profiles.is_empty() {
            let headers = vec!["Resource", "Findable", "Accessible", "Interoperable", "Reusable", "Overall"];
            let rows = profiles
                .iter()
                .map(|profile| {
                    vec![
                        profile.resource.clone(),
                        profile.findable.format(),
                        profile.accessible.format(),
                        profile.interoperable.format(),
                        profile.reusable.format(),
                        profile.total.format(),
                    ]
                })
                .collect();
            print_values_as_table::<String>(headers, rows, None);
        }
        if !(*no_fail || *terse || issues.is_empty()) {
            print_summary();
        }
    }
    if is_stdout_piped() && !issues.is_empty() {
        match checks_to_csv(issues.as_slice(), false) {
            | Ok(csv) => {
                write_stdout(&csv);
                Ok(())
            }
            | Err(why) => {
                error!("=> {} Output CSV results to stdout — {why}", Label::fail());
                Err(why.into())
            }
        }
    } else {
        issues
            .finish_status(*no_fail)
            .map_err(|count| eyre!("ACORN found {count} issue{}", suffix(count)))
    }
}
fn handle_json(issues: &CheckBatch, fixes: &FixReport, options: &AnalysisContext) -> Result<(), Report> {
    serde_json::to_string_pretty(&CheckJsonOutput {
        checks: issues.iter().map(Record::from).collect(),
        fixes,
    })
    .map_err(Report::new)
    .map(|output| println!("{output}"))
    .and_then(|()| {
        issues
            .finish_status(options.report.extension.no_fail)
            .map_err(|count| eyre!("ACORN found {count} issue{}", suffix(count)))
    })
}
async fn prepare(source: &InputSource, options: &CommandOptions, format: Option<&MimeType>) -> Result<Prepared<Input>, Report> {
    match source {
        | InputSource::Location(location) => location.prepare(options).await,
        | InputSource::Text { content, origin } => origin.prepare(content, format),
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
fn resolve_readability_threshold(readability_metric: ReadabilityType) -> Option<f64> {
    let variable = match readability_metric {
        | ReadabilityType::ARI => "MAX_ALLOWED_ARI",
        | ReadabilityType::CLI => "MAX_ALLOWED_CLI",
        | ReadabilityType::FKGL => "MAX_ALLOWED_FKGL",
        | ReadabilityType::FRES => "MAX_ALLOWED_FRES",
        | ReadabilityType::GFI => "MAX_ALLOWED_GFI",
        | ReadabilityType::Lix => "MAX_ALLOWED_LIX",
        | ReadabilityType::SMOG => "MAX_ALLOWED_SMOG",
    };
    let _ = dotenv();
    dotenvy::var(variable)
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .inspect(|value| debug!(variable, value, "=> {} Readability threshold from .env", Label::using()))
}
#[allow(clippy::too_many_arguments)]
pub async fn run(
    path: &Option<PathBuf>,
    paste: &bool,
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
    vale_directory: &Option<PathBuf>,
    format: &Option<MimeType>,
    config: &Option<PathBuf>,
    suggest_fixes: bool,
    output: CheckOutputFormat,
    inference: &InferenceArgs,
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
    let suggestions = SuggestionConfiguration {
        enabled: suggest_fixes,
        inference: inference.clone(),
        offline,
        output,
    };
    let filesystem_selected = path.is_some() || branch.is_some() || commit.is_some() || !filter.is_empty() || !ignore.is_empty() || *merge_request;
    match select_source(path, *paste, filesystem_selected, watch.is_some(), read_stdin, read_clipboard) {
        | Ok(InputSource::Location(_)) if format.is_some() => Err(eyre!("--format requires piped standard input or --paste")),
        | Ok(source) => {
            let readability_metric = resolve_readability_metric(readability_metric);
            let options = build_analysis_context(
                skip,
                *disable_website_checks,
                *all,
                *exit_on_first_error,
                *skip_verify_checksum,
                *no_fail,
                *raw,
                *terse,
                *standard,
                readability_metric,
                is_silent,
                offline,
                ignore_sync_failure,
                sync_retry_interval,
                vale_directory.clone(),
            );
            match ApplicationConfiguration::load_with_path(config) {
                | Err(why) => Err(why),
                | Ok((configuration, resolved)) => {
                    let project_root = ApplicationConfiguration::root(&resolved);
                    #[cfg(feature = "exec")]
                    let supported = Ok::<(), Report>(());
                    #[cfg(not(feature = "exec"))]
                    let supported = match config.is_some() || configuration.has_hook(Hook::Check) || configuration.has_hook(Hook::Validate) {
                        | true => Err(eyre!("configured Rhai check hooks require the `exec` feature")),
                        | false => Ok(()),
                    };
                    match supported {
                        | Ok(()) => {
                            run_source(
                                source,
                                &command_options,
                                &options,
                                HookConfiguration {
                                    application: &configuration,
                                    explicit: config.is_some(),
                                    project_root: &project_root,
                                },
                                format.as_ref(),
                                watch,
                                &suggestions,
                            )
                            .await
                        }
                        | Err(why) => Err(why),
                    }
                }
            }
        }
        | Err(why) => Err(why),
    }
}
async fn run_source(
    source: InputSource,
    command_options: &CommandOptions,
    options: &AnalysisContext,
    hooks: HookConfiguration<'_>,
    format: Option<&MimeType>,
    watch: Option<WatchOptions>,
    suggestions: &SuggestionConfiguration,
) -> Result<(), Report> {
    let HookConfiguration {
        application,
        explicit,
        project_root,
    } = hooks;
    match watch {
        | Some(watch) => match &source {
            | InputSource::Location(location) => {
                watch_run(
                    location,
                    command_options,
                    options,
                    watch,
                    application,
                    project_root,
                    explicit,
                    suggestions,
                )
                .await
            }
            | InputSource::Text { .. } => Err(eyre!("Standard input and clipboard text cannot be used with --watch")),
        },
        | None => match sync_analyzer_before_check(options).await {
            | Ok(run_options) => match prepare(&source, command_options, format).await {
                | Ok(prepared) => execute(prepared.value, &run_options, application, project_root, explicit, suggestions).await,
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        },
    }
}
fn should_sync_analyzer(options: &AnalysisContext) -> bool {
    options.analysis.standard != Standard::ApplicationConfigurationResearchEnablement
        && !options
            .analysis
            .skip
            .iter()
            .any(|category| CheckCategory::from(category) == CheckCategory::Prose)
}
async fn sync_analyzer_before_check(options: &AnalysisContext) -> Result<AnalysisContext> {
    let should_sync = should_sync_analyzer(options);
    let sync_result = match should_sync {
        | false => Ok(()),
        | true => match options
            .execution
            .extension
            .resolve_analyzer::<Vale, _>(
                &options.execution.common,
                options.execution.extension.vale_config.clone().unwrap_or_default(),
            )
            .await
        {
            | Ok(analyzer) => analyzer.retry_sync(&options.execution).await,
            | Err(why) => Err(why),
        },
    };
    match sync_result {
        | Ok(()) => Ok(AnalysisContext {
            execution: ExecutionOptions {
                extension: options.execution.extension.clone().without_sync(),
                ..options.execution.clone()
            },
            ..options.clone()
        }),
        | Err(why) => Err(why),
    }
}
#[allow(clippy::too_many_arguments)]
async fn watch_run(
    location: &Location,
    command_options: &CommandOptions,
    options: &AnalysisContext,
    watch: WatchOptions,
    configuration: &ApplicationConfiguration,
    project_root: &Path,
    explicit_configuration: bool,
    suggestions: &SuggestionConfiguration,
) -> Result<(), Report> {
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
                || location.prepare(command_options),
                |input| execute(input, &run_options, configuration, project_root, explicit_configuration, suggestions),
            )
            .await
        }
        | Err(why) => Err(why),
    }
}
#[cfg(test)]
mod tests;
