use crate::cli::{resolve_paths, CommandOptions};
use acorn::io::{FromPath, InputOutput};
use acorn::prelude::{write, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{print_changes, Label, MimeType, StringConversion};
use clap_verbosity_flag::{log::Level, Verbosity};
use color_eyre::eyre::{eyre, Report, Result};
use owo_colors::OwoColorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tracing::{error, info};

pub async fn run(
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    ignore: &Option<String>,
    dry_run: bool,
    merge_request: bool,
    verbose: &Verbosity,
) -> Result<(), Report> {
    let options = CommandOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .maybe_ignore(ignore.clone())
        .merge_request(merge_request)
        .build();

    resolve_paths(path, &options).await.and_then(|paths| {
        paths.par_iter().try_for_each(|path| -> Result<(), Report> {
            ResearchActivity::read(path.clone())
                .map_err(|why| {
                    error!("=> {} Read research activity data - {why}", Label::fail());
                    eyre!("Read research activity data - {why}")
                })
                .and_then(|data| {
                    let mime = MimeType::from_path(path);
                    data.serialize_as(&mime)
                        .map_err(|why| {
                            error!("=> {} Serialize original link content - {why}", Label::fail());
                            eyre!("Serialize original link content - {why}")
                        })
                        .and_then(|old_content| {
                            let formatted = data.format_with(Some(path.clone()));
                            formatted
                                .serialize_as(&mime)
                                .map_err(|why| {
                                    error!("=> {} Serialize linked content - {why}", Label::fail());
                                    eyre!("Serialize linked content - {why}")
                                })
                                .map(|new_content| (old_content, new_content))
                        })
                        .and_then(|(old_content, new_content)| {
                            if dry_run {
                                println!("{} Link {}", Label::dry_run(), path.clone().to_absolute_string().yellow());
                            }
                            if dry_run || verbose.log_level().unwrap_or(Level::Info) > Level::Error {
                                print_changes(&old_content, &new_content);
                            }
                            if !dry_run {
                                let filepath = path.clone().with_extension("jsonld").to_absolute_string();
                                write(&filepath, new_content.as_bytes())
                                    .map_err(|why| {
                                        error!(file = filepath, "=> {} Write - {why}", Label::fail());
                                        eyre!("Write - {why}")
                                    })
                                    .map(|_| {
                                        info!(file = filepath, "=> {} Linked", Label::pass());
                                    })
                            } else {
                                Ok(())
                            }
                        })
                })
        })
    })
}
