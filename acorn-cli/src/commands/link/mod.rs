use crate::cli::{paths_from_options, CommandLineOptions};
use acorn::io::{FromPath, InputOutput};
use acorn::prelude::{exit, write, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{print_changes, Label, MimeType, StringConversion};
use clap_verbosity_flag::{log::Level, Verbosity};
use color_eyre::eyre::{Report, Result};
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
    let options = CommandLineOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .maybe_ignore(ignore.clone())
        .merge_request(merge_request)
        .build();

    let paths = paths_from_options(path, &Some(options)).await;
    paths.par_iter().for_each(|path| match ResearchActivity::read(path.clone()) {
        | Ok(data) => {
            let mime = MimeType::from_path(path);
            let old_content = match mime {
                | MimeType::Json => serde_json::to_string_pretty(&data).unwrap(),
                | MimeType::Yaml => serde_yml::to_string(&data).unwrap(),
                | _ => unimplemented!("Unsupported file type"),
            };
            let context = Some(path.clone());
            let formatted = data.format_with(context);
            let new_content = match mime {
                | MimeType::Json => serde_json::to_string_pretty(&formatted).unwrap(),
                | MimeType::Yaml => serde_yml::to_string(&formatted).unwrap(),
                | _ => unimplemented!("Unsupported file type"),
            };
            if dry_run {
                println!("{} Link {}", Label::dry_run(), path.clone().to_absolute_string().yellow());
            }
            if dry_run || verbose.log_level().unwrap() > Level::Error {
                print_changes(&old_content, &new_content);
            }
            if !dry_run {
                let filepath = path.clone().with_extension("jsonld").to_absolute_string();
                match write(&filepath, new_content.as_bytes()) {
                    | Ok(_) => {
                        info!(file = filepath, "=> {} Linked", Label::pass());
                    }
                    | Err(why) => {
                        error!(file = filepath, "=> {} Write - {why}", Label::fail());
                        exit(exitcode::UNAVAILABLE);
                    }
                }
            }
        }
        | Err(why) => {
            error!("=> {} Read research activity data - {why}", Label::fail());
            exit(exitcode::DATAERR);
        }
    });
    Ok(())
}
