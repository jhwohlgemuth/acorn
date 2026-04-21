use crate::cli::CommandOptions;
use crate::commands::preflight;
use acorn::io::config::{ApplicationConfiguration, Bucket, BucketOptions};
use acorn::io::database::schema::{ActivityRow, Table};
use acorn::io::database::{Database, Operations};
use acorn::io::InputOutput;
use acorn::prelude::{Path, PathBuf};
use acorn::util::{suffix, Label, StringConversion};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};
use owo_colors::OwoColorize;
use std::process::exit;
use tracing::error;

const DEFAULT_CONFIG_FILENAMES: [&str; 3] = [".acorn.json", ".acorn.yaml", ".acorn.yml"];

#[allow(clippy::too_many_arguments)]
pub async fn run(
    config: &Option<PathBuf>,
    url: &Option<String>,
    ignore: &[String],
    output: &Option<PathBuf>,
    database_path: &Option<PathBuf>,
    threads: usize,
    verbose: &Verbosity,
    offline: bool,
) -> Result<(), Report> {
    let quiet = verbose.is_silent();
    preflight!(&CommandOptions::init()
        .offline(offline)
        .quiet(quiet)
        .threads(threads)
        .supported(true)
        .build());
    let database = Database::<Table>::from_path(database_path.clone());
    let options = BucketOptions::init().threads(threads).quiet(quiet).ignore(ignore.to_vec()).build();
    let config = config.clone().or_else(|| resolve_default_config_path(&PathBuf::from(".")));
    if let Some(path) = &config {
        match ApplicationConfiguration::read(path.clone()) {
            | Ok(content) => {
                let mut total: usize = 0;
                if let Some(buckets) = content.buckets {
                    for bucket in buckets {
                        if let Some(path) = output.clone() {
                            let bucket_options = options.clone().with_output(path);
                            if offline || bucket.code_repository.is_local() {
                                match bucket.copy_files(&bucket_options).await {
                                    | Ok(count) => {
                                        total += count;
                                    }
                                    | Err(why) => {
                                        error!("=> {} Copy data — {why}", Label::fail());
                                        return Err(why);
                                    }
                                }
                            } else {
                                match bucket.download_files(&bucket_options).await {
                                    | Ok(count) => {
                                        total += count;
                                    }
                                    | Err(why) => {
                                        error!("=> {} Download data — {why}", Label::fail());
                                        return Err(why);
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(path) = output {
                    let activity = ActivityRow::init()
                        .command("download")
                        .executed_at(chrono::Utc::now())
                        .user_path(path.to_absolute_string())
                        .success(true)
                        .build();
                    let _ = database.insert(activity);
                }
                if !quiet {
                    println!("\n\n   => {} Processed {} item{}\n", Label::pass(), total.green(), suffix(total));
                }
            }
            | Err(why) => {
                error!("=> {} Download data — {why}", Label::fail());
                let activity = ActivityRow::init()
                    .command("download")
                    .executed_at(chrono::Utc::now())
                    .user_path(path.file_name_with_parent())
                    .success(false)
                    .build();
                let _ = database.insert(activity);
                exit(exitcode::DATAERR);
            }
        };
    } else if let Some(value) = url {
        let mut total: usize = 0;
        let bucket = Bucket::from(value.as_ref());
        if let Some(path) = output.clone() {
            let bucket_options = options.clone().with_output(path.clone());
            if offline || bucket.code_repository.is_local() {
                match bucket.copy_files(&bucket_options).await {
                    | Ok(count) => {
                        total += count;
                    }
                    | Err(why) => {
                        error!("=> {} Copy data — {why}", Label::fail());
                        return Err(why);
                    }
                }
            } else {
                match bucket.download_files(&bucket_options).await {
                    | Ok(count) => {
                        total += count;
                    }
                    | Err(why) => {
                        error!("=> {} Download data — {why}", Label::fail());
                        return Err(why);
                    }
                }
            }
            let activity = ActivityRow::init()
                .command("download")
                .executed_at(chrono::Utc::now())
                .user_path(path.to_absolute_string())
                .success(true)
                .build();
            let _ = database.insert(activity);
            if !quiet {
                println!("\n\n   => {} Processed {} item{}\n", Label::pass(), total.green(), suffix(total));
            }
        }
    } else if !quiet {
        println!(
            "\n\n   => {} No URL or configuration file found. Provide a URL or create one of: {}\n",
            Label::fail(),
            DEFAULT_CONFIG_FILENAMES.join(", ")
        );
    } else {
        error!("=> {} No URL or configuration file found for download command", Label::fail());
    }
    Ok(())
}
fn resolve_default_config_path(base: &Path) -> Option<PathBuf> {
    DEFAULT_CONFIG_FILENAMES
        .iter()
        .map(|name| base.join(name))
        .find(|candidate| candidate.exists())
}

#[cfg(test)]
mod tests;
