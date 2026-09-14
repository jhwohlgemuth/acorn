use crate::cli::{resolve_paths, CommandOptions};
use crate::commands::preflight;
use acorn::fail;
use acorn::io::config::application::{ApplicationConfiguration, Hook};
use acorn::io::enrichment::Pipeline;
#[cfg(feature = "exec")]
use acorn::io::workflow::WorkflowAction;
use acorn::io::workflow::{self, Workflow, WorkflowExtension};
use acorn::io::{slice_path, FromPath, InputOutput};
use acorn::util::constants::app::DISPLAY_PATH_CHARACTER_COUNT;
use acorn::util::{print_changes_with_color, Label, StringConversion};
use acorn_core::options::CommonRuntime;
use acorn_core::util::MimeType;
use acorn_core::Location;
use acorn_schema::enrichment::Provider;
use acorn_schema::research_activity::ResearchActivity;
use clap_verbosity_flag::{log::Level, Verbosity};
use color_eyre::eyre::{eyre, Report, Result};
use owo_colors::OwoColorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Serialize;
use std::fs::write;
use std::path::{Path, PathBuf};
use tracing::info;

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
        .build();
    preflight!(&options);
    match enrich && offline {
        | true => Err(eyre!("--enrich cannot be used with --offline")),
        | false => match ApplicationConfiguration::load_with_path(config) {
            | Err(why) => Err(why),
            | Ok((configuration, resolved)) => {
                let root = ApplicationConfiguration::root(&resolved);
                match (config.is_some(), enrich, configuration.has_hook(Hook::Format)) {
                    #[cfg(not(feature = "exec"))]
                    | (true, _, _) => Err(eyre!("--config requires the `exec` feature")),
                    #[cfg(not(feature = "exec"))]
                    | (false, _, true) => Err(eyre!("configured Rhai format hooks require the `exec` feature")),
                    #[cfg(feature = "exec")]
                    | (_, true, _) | (_, false, true) => run_workflow(&options, providers, enrich, &configuration, &root, path).await,
                    #[cfg(not(feature = "exec"))]
                    | (false, true, false) => run_workflow(&options, providers, enrich, &configuration, &root, path).await,
                    | (_, false, false) => format(path, &options).await,
                }
            }
        },
    }
}
async fn run_workflow(
    options: &CommandOptions,
    providers: &[Provider],
    enrich: bool,
    configuration: &ApplicationConfiguration,
    root: &Path,
    path: &Option<PathBuf>,
) -> Result<(), Report> {
    let pipeline = match providers.is_empty() {
        | true => Pipeline::default(),
        | false => Pipeline::new(providers.to_vec()),
    };
    let workflow_options = workflow::Options::init()
        .common(CommonRuntime {
            dry_run: options.dry_run,
            no_color: options.no_color,
            offline: options.offline,
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
            match Workflow::<ResearchActivity>::plan(&sources, workflow_options.clone(), Some(&pipeline)).await {
                #[cfg(feature = "exec")]
                | Ok(plan) => {
                    let hooks = match enrich {
                        | true => vec![WorkflowAction::Format, WorkflowAction::Enrich],
                        | false => vec![WorkflowAction::Format],
                    };
                    plan.apply(workflow_options, configuration, root, &hooks)
                }
                #[cfg(not(feature = "exec"))]
                | Ok(plan) => {
                    let _ = (configuration, root);
                    plan.apply_enriched(workflow_options)
                }
                | Err(why) => Err(why),
            }
        }
        | Err(why) => Err(why),
    }
}
async fn format(path: &Option<PathBuf>, options: &CommandOptions) -> Result<(), Report> {
    resolve_paths(path, options)
        .await
        .and_then(|paths| paths.par_iter().try_for_each(|path| format_file::<ResearchActivity>(path, options)))
}
fn format_file<T>(path: &Path, options: &CommandOptions) -> Result<(), Report>
where
    T: InputOutput + Serialize,
{
    T::read(path.to_path_buf())
        .map_err(|why| {
            fail!("Read research activity data — {why}");
            eyre!("Read research activity data — {why}")
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
                .and_then(|(old_content, new_content)| match old_content == new_content {
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
                            print_changes_with_color(&old_content, &new_content, !options.no_color);
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
                })
        })
}

#[cfg(test)]
mod tests;
