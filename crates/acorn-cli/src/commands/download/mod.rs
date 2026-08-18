use crate::cli::CommandOptions;
use crate::commands::preflight;
use acorn::io::config::{ApplicationConfiguration, Bucket, BucketOptions};
use acorn::io::database::schema::{ActivityRow, Table};
use acorn::io::database::{Database, Operations};
use acorn::io::InputOutput;
use acorn::prelude::PathBuf;
use acorn::util::constants::app::DEFAULT_CONFIG_FILENAMES;
use acorn::util::{suffix, Label, StringConversion};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{eyre, Report, Result};
use jiff::Timestamp;
use owo_colors::OwoColorize;
use tracing::error;

pub mod model;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    config: &Option<PathBuf>,
    url: &[String],
    filter: &[String],
    ignore: &[String],
    output: &Option<PathBuf>,
    flatten: bool,
    clobber: bool,
    database_path: &Option<PathBuf>,
    threads: usize,
    verbose: &Verbosity,
    offline: bool,
    no_local_database: bool,
) -> Result<(), Report> {
    let quiet = verbose.is_silent();
    preflight!(&CommandOptions::init()
        .offline(offline)
        .quiet(quiet)
        .threads(threads)
        .supported(true)
        .build());
    let options = BucketOptions::init()
        .filter(filter.to_vec())
        .ignore(ignore.to_vec())
        .flatten(flatten)
        .clobber(clobber)
        .quiet(quiet)
        .threads(threads)
        .build();
    let result: Result<(), Report> = if let Some(path) = ApplicationConfiguration::resolve(config) {
        match ApplicationConfiguration::read(path.clone()) {
            | Ok(content) => {
                let mut total: usize = 0;
                let mut branch_result: Result<(), Report> = Ok(());
                if let Some(buckets) = content.buckets {
                    if let Some(output_path) = output.clone() {
                        let bucket_options = options.clone().with_output(output_path);
                        for bucket in buckets {
                            let is_local = offline || bucket.code_repository.is_local();
                            let bucket_result = if is_local {
                                bucket.copy_files(&bucket_options).await
                            } else {
                                bucket.download_files(&bucket_options).await
                            };
                            match bucket_result {
                                | Ok(manifest) => match manifest.ingest(&bucket_options, database_path, no_local_database) {
                                    | Ok(()) => total = total.saturating_add(manifest.count()),
                                    | Err(why) => {
                                        branch_result = Err(why);
                                        break;
                                    }
                                },
                                | Err(why) => {
                                    if is_local {
                                        error!("=> {} Copy data — {why}", Label::fail());
                                    } else {
                                        error!("=> {} Download data — {why}", Label::fail());
                                    }
                                    branch_result = Err(why);
                                    break;
                                }
                            }
                        }
                    }
                }
                if branch_result.is_ok() {
                    if let Some(path) = output {
                        if !no_local_database {
                            log_activity("download", path.to_absolute_path(), database_path, true);
                        }
                    }
                    if !quiet {
                        print_result(total);
                    }
                }
                branch_result
            }
            | Err(why) => {
                error!("=> {} Download data — {why}", Label::fail());
                if !no_local_database {
                    log_activity("download", path.file_name_with_parent(), database_path, false);
                }
                return Err(eyre!("Failed to process download configuration"));
            }
        }
    } else if !url.is_empty() {
        let mut total: usize = 0;
        let mut branch_result: Result<(), Report> = Ok(());
        if let Some(path) = output.clone() {
            let bucket_options = options.clone().with_output(path.clone());
            for value in url {
                if branch_result.is_ok() {
                    let bucket = Bucket::from(value.as_ref());
                    let is_local = offline || bucket.code_repository.is_local();
                    let bucket_result = if is_local {
                        bucket.copy_files(&bucket_options).await
                    } else {
                        bucket.download_files(&bucket_options).await
                    };
                    match bucket_result {
                        | Ok(manifest) => match manifest.ingest(&bucket_options, database_path, no_local_database) {
                            | Ok(()) => total = total.saturating_add(manifest.count()),
                            | Err(why) => branch_result = Err(why),
                        },
                        | Err(why) => {
                            if is_local {
                                error!("=> {} Copy data — {why}", Label::fail());
                            } else {
                                error!("=> {} Download data — {why}", Label::fail());
                            }
                            branch_result = Err(why);
                        }
                    }
                }
            }
            if branch_result.is_ok() {
                if !no_local_database {
                    log_activity("download", path.to_absolute_path(), database_path, true);
                }
                if !quiet {
                    print_result(total);
                }
            }
        }
        branch_result
    } else if !quiet {
        println!(
            "\n\n   => {} No URL or configuration file found. Provide a URL or create one of: {}\n",
            Label::fail(),
            DEFAULT_CONFIG_FILENAMES.join(", ")
        );
        Ok(())
    } else {
        error!("=> {} No URL or configuration file found for download command", Label::fail());
        Ok(())
    };
    result
}
fn log_activity(command: &str, path: impl Into<String>, database_path: &Option<PathBuf>, success: bool) {
    let database = Database::<Table>::from_path(database_path.clone());
    let activity = ActivityRow::init()
        .command(command)
        .executed_at(Timestamp::now())
        .user_path(path)
        .success(success)
        .build();
    if let Err(why) = database.insert(activity) {
        error!("=> {} Save {command} activity — {why}", Label::fail());
    }
}
fn print_result(total: usize) {
    let (label, count) = match total {
        | 0 => (Label::warning(), total.yellow().to_string()),
        | _ => (Label::pass(), total.green().to_string()),
    };
    println!("\n\n   => {label} Processed {count} item{}\n", suffix(total));
}

#[cfg(test)]
mod tests;
