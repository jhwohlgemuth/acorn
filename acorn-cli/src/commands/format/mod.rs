use crate::cli::{resolve_paths, CommandOptions};
use crate::commands::preflight;
use acorn::fail;
use acorn::io::{FromPath, InputOutput};
use acorn::prelude::{write, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{print_changes, Label, MimeType, StringConversion};
use clap_verbosity_flag::{log::Level, Verbosity};
use color_eyre::eyre::{eyre, Report, Result};
use owo_colors::OwoColorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tracing::info;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    filter: &Option<String>,
    ignore: &Option<String>,
    dry_run: bool,
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
        .quiet(verbose.is_silent())
        .build();
    preflight!(&options);
    resolve_paths(path, &options).await.and_then(|paths| {
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
                            if dry_run {
                                println!("{} Format {}", Label::dry_run(), path.clone().to_absolute_string().yellow());
                            }
                            match verbose.log_level() {
                                | Some(Level::Warn) | Some(Level::Info) | Some(Level::Debug) | Some(Level::Trace) => {
                                    print_changes(&old_content, &new_content);
                                }
                                | _ => {
                                    if dry_run {
                                        print_changes(&old_content, &new_content);
                                    }
                                }
                            }
                            if !dry_run {
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
