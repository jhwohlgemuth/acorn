use crate::cli::{resolve_paths, CommandOptions};
use crate::commands::preflight;
use acorn::fail;
use acorn::io::enrichment::Pipeline;
use acorn::io::workflow::{self, Workflow, WorkflowExtension};
use acorn::io::{FromPath, InputOutput};
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
use std::fs::write;
use std::path::PathBuf;
use tracing::info;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    filter: &Option<String>,
    ignore: &Option<String>,
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
        | false => match format(path, &options).await {
            | Ok(()) if enrich => run_with_enrich(path, &options, providers).await,
            | result => result,
        },
    }
}
async fn run_with_enrich(path: &Option<PathBuf>, options: &CommandOptions, providers: &[Provider]) -> Result<(), Report> {
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
            enrich: true,
            format: true,
            show_changes: options.verbose.log_level().is_some(),
            ..WorkflowExtension::default()
        })
        .build();
    match resolve_paths(path, options).await {
        | Ok(paths) => {
            let sources = paths.iter().map(|path| Location::from(&path.display().to_string())).collect::<Vec<_>>();
            match Workflow::<ResearchActivity>::plan(&sources, workflow_options.clone(), Some(&pipeline)).await {
                | Ok(plan) => plan.apply_enriched(workflow_options),
                | Err(why) => Err(why),
            }
        }
        | Err(why) => Err(why),
    }
}
async fn format(path: &Option<PathBuf>, options: &CommandOptions) -> Result<(), Report> {
    resolve_paths(path, options).await.and_then(|paths| {
        paths.par_iter().try_for_each(|path| -> Result<(), Report> {
            ResearchActivity::read(path.clone())
                .map_err(|why| {
                    fail!("Read research activity data — {}", why);
                    eyre!("Read research activity data — {why}")
                })
                .and_then(|data| {
                    let mime = MimeType::from_path(path);
                    data.serialize_as(&mime)
                        .map_err(|why| {
                            fail!("Serialize original content — {}", why);
                            eyre!("Serialize original content — {why}")
                        })
                        .and_then(|old_content| {
                            let formatted = data.format_with(Some(path.clone()));
                            formatted
                                .serialize_as(&mime)
                                .map_err(|why| {
                                    fail!("Serialize formatted content — {}", why);
                                    eyre!("Serialize formatted content — {why}")
                                })
                                .map(|new_content| (old_content, new_content))
                        })
                        .and_then(|(old_content, new_content)| {
                            if options.dry_run {
                                let path = path.clone().to_absolute_path();
                                match options.no_color {
                                    | true => println!("{} Format {path}", Label::DRY_RUN),
                                    | false => println!("{} Format {}", Label::dry_run(), path.yellow()),
                                }
                            }
                            match options.verbose.log_level() {
                                | Some(Level::Warn) | Some(Level::Info) | Some(Level::Debug) | Some(Level::Trace) => {
                                    print_changes_with_color(&old_content, &new_content, !options.no_color);
                                }
                                | _ => {
                                    if options.dry_run {
                                        print_changes_with_color(&old_content, &new_content, !options.no_color);
                                    }
                                }
                            }
                            if !options.dry_run {
                                write(path, new_content.as_bytes())
                                    .map_err(|why| {
                                        fail!("Write — {}", why);
                                        eyre!("Write — {why}")
                                    })
                                    .map(|_| {
                                        info!("=> {} Formatted", Label::pass());
                                    })
                            } else {
                                Ok(())
                            }
                        })
                })
        })
    })
}

#[cfg(test)]
mod tests;
