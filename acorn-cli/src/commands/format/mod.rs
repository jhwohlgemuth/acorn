use crate::cli::{paths_from_options, CommandLineOptions};
use acorn_lib::io::FromPath;
use acorn_lib::prelude::{File, PathBuf, Write};
use acorn_lib::schema::research_activity::ResearchActivity;
use acorn_lib::util::{print_changes, Label, MimeType, ToAbsoluteString};
use clap_verbosity_flag::{log::Level, Verbosity};
use color_eyre::eyre::{Report, Result};
use owo_colors::OwoColorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::process::exit;
use tracing::{error, info};

#[allow(clippy::too_many_arguments)]
pub fn run(
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    ignore: &Option<String>,
    dry_run: bool,
    merge_request: &bool,
    verbose: &Verbosity,
    offline: &bool,
) -> Result<(), Report> {
    if *offline {
        println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
        unimplemented!("Offline mode is not implemented yet");
    }
    let options = CommandLineOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .maybe_ignore(ignore.clone())
        .merge_request(*merge_request)
        .build();
    let paths = paths_from_options(path, &Some(options));
    paths.par_iter().for_each(|path| match ResearchActivity::read(path.clone()) {
        | Some(data) => {
            let mime = MimeType::from_path(path);
            let old_content = match mime {
                | MimeType::Json => serde_json::to_string_pretty(&data).unwrap(),
                | MimeType::Yaml => serde_yml::to_string(&data).unwrap(),
                | _ => unimplemented!("Unsupported file type"),
            };
            let new_content = match mime {
                | MimeType::Json => serde_json::to_string_pretty(&data.format(Some(path.clone()))).unwrap(),
                | MimeType::Yaml => serde_yml::to_string(&data.format(Some(path.clone()))).unwrap(),
                | _ => unimplemented!("Unsupported file type"),
            };
            if dry_run {
                println!("{} Format {}", Label::dry_run(), path.clone().to_absolute_string().yellow());
            }
            match verbose.log_level() {
                | Some(Level::Warn) | Some(Level::Info) | Some(Level::Debug) | Some(Level::Trace) | _ if dry_run => {
                    print_changes(&old_content, &new_content);
                }
                | _ => (),
            }
            if dry_run || verbose.log_level().unwrap() > Level::Error {
                print_changes(&old_content, &new_content);
            }
            if !dry_run {
                let filepath = path.clone().to_absolute_string();
                let mut file = match File::create(path) {
                    | Ok(file) => file,
                    | Err(why) => {
                        error!(file = filepath, "=> {} Create - {}", Label::fail(), why);
                        exit(exitcode::UNAVAILABLE);
                    }
                };
                match file.write_all(new_content.as_bytes()) {
                    | Ok(_) => {
                        info!(file = filepath, "=> {} Formatted", Label::pass());
                    }
                    | Err(why) => {
                        error!(file = filepath, "=> {} Write - {}", Label::fail(), why);
                        exit(exitcode::UNAVAILABLE);
                    }
                }
            }
        }
        | None => {
            error!("=> {} Read research activity data", Label::fail());
            exit(exitcode::DATAERR);
        }
    });
    Ok(())
}
