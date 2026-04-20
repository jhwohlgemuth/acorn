use crate::cli::{paths_from_options, CommandOptions};
use crate::commands::preflight;
use acorn::fail;
use acorn::io::{FromPath, InputOutput};
use acorn::prelude::{exit, write, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{print_changes, Label, MimeType, StringConversion};
use clap_verbosity_flag::{log::Level, Verbosity};
use color_eyre::eyre::{Report, Result};
use owo_colors::OwoColorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tracing::info;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    ignore: &Option<String>,
    dry_run: bool,
    merge_request: &bool,
    verbose: &Verbosity,
    offline: bool,
) -> Result<(), Report> {
    let options = CommandOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .maybe_ignore(ignore.clone())
        .merge_request(*merge_request)
        .offline(offline)
        .quiet(verbose.is_silent())
        .build();
    preflight!(&options);
    let paths = paths_from_options(path, &Some(options)).await;
    paths.par_iter().for_each(|path| match ResearchActivity::read(path.clone()) {
        | Ok(data) => {
            let mime = MimeType::from_path(path);
            let old_content = match data.serialize_as(&mime) {
                | Ok(content) => content,
                | Err(why) => {
                    fail!("Serialize original content - {}", why);
                    exit(exitcode::UNAVAILABLE);
                }
            };
            let context = Some(path.clone());
            let formatted = data.format_with(context);
            let new_content = match formatted.serialize_as(&mime) {
                | Ok(content) => content,
                | Err(why) => {
                    fail!("Serialize formatted content - {}", why);
                    exit(exitcode::UNAVAILABLE);
                }
            };
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
                match write(path, new_content.as_bytes()) {
                    | Ok(_) => {
                        info!("=> {} Formatted", Label::pass());
                    }
                    | Err(why) => {
                        fail!("Write - {}", why);
                        exit(exitcode::UNAVAILABLE);
                    }
                }
            }
        }
        | Err(why) => {
            fail!("Read research activity data - {}", why);
            exit(exitcode::DATAERR);
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests;
