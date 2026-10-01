mod acre;
use crate::{
    cli::{resolve_paths, CommandOptions},
    commands::preflight,
};
#[cfg(feature = "exec")]
use acorn::exec::config::ExecutionOrigin;
#[cfg(feature = "exec")]
use acorn::io::workflow::WorkflowAction;
use acorn::{
    analyzer::Standard,
    fail,
    io::{
        config::application::{ApplicationConfiguration, Hook},
        enrichment::{EnrichmentMetadata, Pipeline},
        slice_path,
        workflow::{self, Workflow, WorkflowExtension},
        FromPath, InputOutput,
    },
    util::{constants::app::DISPLAY_PATH_CHARACTER_COUNT, StringConversion},
};
use acorn_core::{options::CommonRuntime, util::MimeType, Location};
use acorn_diff::unified_diff;
use acorn_host::terminal::Label;
use acorn_schema::{
    enrichment::Provider,
    research_activity::ResearchActivity,
    standard::{cff::Cff, datacite, invenio},
    validation::Validate,
};
use clap_verbosity_flag::{log::Level, Verbosity};
use color_eyre::eyre::{eyre, Report, Result};
use owo_colors::OwoColorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::write,
    path::{Path, PathBuf},
};
use strum::IntoEnumIterator;
use tracing::info;

struct StandardOptions<'a> {
    command: &'a CommandOptions,
    configuration: &'a ApplicationConfiguration,
    enrich: bool,
    explicit_configuration: bool,
    path: &'a Option<PathBuf>,
    providers: &'a [Provider],
    root: &'a Path,
    standard: Standard,
}
impl StandardOptions<'_> {
    fn config_is_unsupported(&self) -> Option<Report> {
        match (self.explicit_configuration, self.configuration.has_hook(Hook::Format)) {
            #[cfg(not(feature = "exec"))]
            | (true, _) => Some(eyre!("--config requires the `exec` feature")),
            #[cfg(not(feature = "exec"))]
            | (false, true) => Some(eyre!("configured Rhai format hooks require the `exec` feature")),
            | _ => None,
        }
    }
}
async fn format<T>(path: &Option<PathBuf>, options: &CommandOptions) -> Result<(), Report>
where
    T: InputOutput + Serialize,
{
    resolve_paths(path, options)
        .await
        .and_then(|paths| paths.par_iter().try_for_each(|path| format_file::<T>(path, options)))
}
fn format_diff(old: &str, new: &str, path: &Path, color: bool) -> String {
    let before_label = format!("a/{}", path.display());
    let after_label = format!("b/{}", path.display());
    let diff = unified_diff(old, new, &before_label, &after_label);
    match color {
        | false => diff,
        | true => diff
            .split_inclusive('\n')
            .map(|line| match line {
                | line if line.starts_with("--- ") || line.starts_with('-') => line.red().to_string(),
                | line if line.starts_with("+++ ") || line.starts_with('+') => line.green().to_string(),
                | line if line.starts_with("@@") => line.cyan().to_string(),
                | line => line.dimmed().to_string(),
            })
            .collect(),
    }
}
fn format_file<T>(path: &Path, options: &CommandOptions) -> Result<(), Report>
where
    T: InputOutput + Serialize,
{
    T::read(path.to_path_buf())
        .map_err(|why| {
            fail!("Read metadata — {why}");
            eyre!("Read metadata — {why}")
        })
        .and_then(|data| {
            let mime = MimeType::from_path(path);
            data.serialize_as(&mime)
                .map_err(|why| {
                    fail!("Serialize original content — {why}");
                    eyre!("Serialize original content — {why}")
                })
                .and_then(|old_content| {
                    let formatted = data.format_with(Some(path.to_path_buf()));
                    formatted
                        .serialize_as(&mime)
                        .map_err(|why| {
                            fail!("Serialize formatted content — {why}");
                            eyre!("Serialize formatted content — {why}")
                        })
                        .map(|new_content| (old_content, new_content))
                })
                .and_then(|(old_content, new_content)| write_formatted_content(path, options, &old_content, &new_content))
        })
}
fn write_formatted_content(path: &Path, options: &CommandOptions, old_content: &str, new_content: &str) -> Result<(), Report> {
    match old_content == new_content {
        | true => {
            let display_path = slice_path(path, DISPLAY_PATH_CHARACTER_COUNT);
            info!("=> {} No format changes for {display_path}", Label::skip());
            Ok(())
        }
        | false => {
            let CommandOptions {
                dry_run, no_color, verbose, ..
            } = options;
            if *dry_run {
                let absolute_path = path.to_path_buf().to_absolute_path();
                match no_color {
                    | true => println!("{} Format {absolute_path}", Label::DRY_RUN),
                    | false => println!("{} Format {}", Label::dry_run(), absolute_path.yellow()),
                }
            }
            let should_show_changes = matches!(
                verbose.log_level(),
                Some(Level::Warn) | Some(Level::Info) | Some(Level::Debug) | Some(Level::Trace)
            );
            if *dry_run || should_show_changes {
                print!("{}", format_diff(old_content, new_content, path, !options.no_color));
            }
            match *dry_run {
                | true => Ok(()),
                | false => write(path, new_content.as_bytes())
                    .map_err(|why| {
                        fail!("Write — {why}");
                        eyre!("Write — {why}")
                    })
                    .map(|_| match no_color {
                        | true => info!("=> {} Formatted {}", Label::pass(), path.display()),
                        | false => info!("=> {} Formatted {}", Label::pass(), path.display().cyan()),
                    }),
            }
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub async fn run(
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    filter: &Option<String>,
    ignore: &Option<String>,
    config: &Option<PathBuf>,
    dry_run: bool,
    enrich: bool,
    no_color: bool,
    providers: &[Provider],
    merge_request: &bool,
    standard: Standard,
    threads: usize,
    verbose: &Verbosity,
    offline: bool,
) -> Result<(), Report> {
    let options = CommandOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .maybe_filter(filter.clone())
        .maybe_ignore(ignore.clone())
        .merge_request(*merge_request)
        .offline(offline)
        .dry_run(dry_run)
        .no_color(no_color)
        .verbose(*verbose)
        .quiet(verbose.is_silent())
        .threads(threads)
        .build();
    preflight!(&options);
    match standard {
        | Standard::ApplicationConfigurationResearchEnablement => acre::run(path, &options, config, enrich, providers).await,
        | _ => match enrich && offline {
            | true => Err(eyre!("--enrich cannot be used with --offline")),
            | false => match ApplicationConfiguration::load_with_path(config) {
                | Err(why) => Err(why),
                | Ok((configuration, resolved)) => {
                    let root = ApplicationConfiguration::root(&resolved);
                    let standard_options = StandardOptions {
                        command: &options,
                        configuration: &configuration,
                        enrich,
                        explicit_configuration: config.is_some(),
                        path,
                        providers,
                        root: &root,
                        standard,
                    };
                    match standard {
                        | Standard::CitationFileFormat => run_as::<Cff>(standard_options).await,
                        | Standard::Datacite => run_as::<datacite::Record>(standard_options).await,
                        | Standard::Invenio => run_as::<invenio::Record>(standard_options).await,
                        | Standard::ResearchActivityData => run_as::<ResearchActivity>(standard_options).await,
                        | _ => Err(eyre!("Formatting is not supported for the {standard} standard")),
                    }
                }
            },
        },
    }
}
async fn run_as<T>(options: StandardOptions<'_>) -> Result<(), Report>
where
    T: Clone + DeserializeOwned + EnrichmentMetadata + InputOutput + Send + Serialize + Sync + Validate + 'static,
{
    let config_is_unsupported = options.config_is_unsupported();
    let StandardOptions {
        command,
        configuration,
        enrich,
        explicit_configuration,
        path,
        providers,
        root,
        standard,
    } = options;
    let selected = match (standard, providers.is_empty()) {
        | (Standard::ResearchActivityData, true) => Ok(Provider::iter().collect()),
        | (Standard::ResearchActivityData, false) => Ok(providers.to_vec()),
        | (_, true) => Ok(vec![Provider::Orcid]),
        | (_, false) if providers.iter().all(|provider| matches!(provider, Provider::Orcid)) => Ok(providers.to_vec()),
        | _ => Err(eyre!("Only the ORCID provider supports enrichment for the {standard} standard")),
    };
    let workflow = match (config_is_unsupported, selected) {
        | (Some(why), _) | (_, Err(why)) => Err(why),
        | (None, Ok(selected)) => match enrich || configuration.has_hook(Hook::Format) {
            | true => {
                run_workflow::<T>(
                    command,
                    Pipeline::new(selected),
                    enrich,
                    configuration,
                    root,
                    explicit_configuration,
                    path,
                )
                .await
            }
            | false => format::<T>(path, command).await,
        },
    };
    workflow
}
async fn run_workflow<T>(
    options: &CommandOptions,
    pipeline: Pipeline,
    enrich: bool,
    configuration: &ApplicationConfiguration,
    root: &Path,
    explicit_configuration: bool,
    path: &Option<PathBuf>,
) -> Result<(), Report>
where
    T: Clone + DeserializeOwned + EnrichmentMetadata + InputOutput + Send + Serialize + Sync + Validate + 'static,
{
    let workflow_options = workflow::Options::init()
        .common(CommonRuntime {
            dry_run: options.dry_run,
            no_color: options.no_color,
            offline: options.offline,
            threads: options.threads,
            ..CommonRuntime::default()
        })
        .extension(WorkflowExtension {
            check: false,
            enrich,
            format: true,
            show_changes: options.verbose.log_level().is_some(),
            ..WorkflowExtension::default()
        })
        .build();
    match resolve_paths(path, options).await {
        | Ok(paths) => {
            let sources = paths.iter().map(|path| Location::from(&path.display().to_string())).collect::<Vec<_>>();
            match Workflow::<T>::plan(&sources, workflow_options.clone(), Some(&pipeline)).await {
                #[cfg(feature = "exec")]
                | Ok(plan) => {
                    let hooks = match enrich {
                        | true => vec![WorkflowAction::Enrich, WorkflowAction::Format],
                        | false => vec![WorkflowAction::Format],
                    };
                    plan.apply(
                        workflow_options,
                        configuration,
                        root,
                        &hooks,
                        ExecutionOrigin::Local,
                        explicit_configuration,
                    )
                }
                #[cfg(not(feature = "exec"))]
                | Ok(plan) => {
                    let _ = (configuration, explicit_configuration, root);
                    plan.apply_enriched(workflow_options)
                }
                | Err(why) => Err(why),
            }
        }
        | Err(why) => Err(why),
    }
}

#[cfg(test)]
mod tests;
