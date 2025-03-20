use acorn_lib::schema::ResearchActivity;
use acorn_lib::util::{get_all_files, path_to_string, print_changes, Label};
use clap_verbosity_flag::{log::Level, Verbosity};
use color_eyre::eyre::{Report, Result};
use owo_colors::OwoColorize;
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tracing::{error, info};

#[allow(clippy::too_many_arguments)]
pub fn run(path: &Option<PathBuf>, dry_run: bool, verbose: &Verbosity) -> Result<(), Report> {
    if let Some(path) = path {
        let paths = get_all_files(path.into(), None);
        paths.par_iter().for_each(|path| match ResearchActivity::read(path.clone()) {
            | Ok(data) => {
                let old_content = serde_json::to_string_pretty(&data.clone()).unwrap();
                let new_content = serde_json::to_string_pretty(&data.clone().format(Some(path.clone()))).unwrap();
                if dry_run {
                    println!("{} Format {}", Label::dry_run(), path_to_string(path.clone()).yellow());
                }
                if dry_run || verbose.log_level().unwrap() > Level::Error {
                    print_changes(&old_content, &new_content);
                }
                if !dry_run {
                    let filepath = path_to_string(path.clone());
                    let mut file = match File::create(path) {
                        | Ok(file) => file,
                        | Err(why) => {
                            error!(file = filepath, "=> {} Create - {}", Label::fail(), why);
                            std::process::exit(exitcode::UNAVAILABLE);
                        }
                    };
                    match file.write_all(new_content.as_bytes()) {
                        | Ok(_) => {
                            info!(file = filepath, "=> {} Formatted", Label::pass());
                        }
                        | Err(why) => {
                            error!(file = filepath, "=> {} Write - {}", Label::fail(), why);
                            std::process::exit(exitcode::UNAVAILABLE);
                        }
                    }
                }
            }
            | Err(err) => {
                error!("=> {} Read research activity data at - {}", Label::fail(), err);
                std::process::exit(exitcode::DATAERR);
            }
        });
    }
    Ok(())
}
