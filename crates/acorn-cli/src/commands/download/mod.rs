use crate::{cli::CommandOptions, commands::preflight};
use acorn::{
    io::{
        config::{ApplicationConfiguration, Bucket, BucketExtension, BucketOptions, Buckets, TransferPolicy},
        database::{
            schema::{ActivityRow, Table},
            Database, Operations,
        },
        print_with_border,
        sidecar::{needle::Needle, SidecarAssets},
        slice_path, InputOutput,
    },
    util::{
        constants::app::{DEFAULT_CONFIG_FILENAMES, DISPLAY_PATH_CHARACTER_COUNT},
        StringConversion,
    },
};
use acorn_core::{
    options::{CommonRuntime, RuntimeOptions},
    util::suffix,
};
use acorn_host::terminal::Label;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{eyre, Report, Result};
use jiff::Timestamp;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use tracing::{error, warn};

pub mod model;
fn finish_transfer(total: usize, options: &BucketOptions) -> Result<(), Report> {
    let RuntimeOptions { common, extension, output } = options;
    let CommonRuntime { quiet, .. } = common;
    let BucketExtension {
        database_path,
        no_local_database,
        ..
    } = extension;
    if let Some(path) = output {
        if !*no_local_database {
            log_activity("download", path.to_absolute_path(), database_path, true);
        }
    }
    match quiet {
        | true => Ok(()),
        | false => {
            print_result(total, output.as_ref());
            Ok(())
        }
    }
}
pub(crate) fn log_activity(command: &str, path: impl Into<String>, database_path: &Option<PathBuf>, success: bool) {
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
/// Download or verify the pinned Needle 3 engine, model, and license, then print the engine path.
pub async fn needle(verbose: &Verbosity, offline: bool) -> Result<(), Report> {
    Needle::resolve_assets(None, offline, verbose.is_silent())
        .await
        .map(|assets| println!("{}", assets.runner.display()))
}
pub(crate) fn print_result(total: usize, output: Option<&PathBuf>) {
    let dest = output
        .map(|path| slice_path(path, DISPLAY_PATH_CHARACTER_COUNT * 3))
        .unwrap_or_else(|| ".".to_string());
    let dest_styled = dest.cyan().to_string();
    let message = match total {
        | 0 => format!(
            "{} No items matched — 0 processed to {}",
            format!("{}CAUTION", Label::CAUTION).yellow(),
            dest_styled
        ),
        | n => format!(
            "{} Downloaded {} item{} to {}",
            Label::pass(),
            n.to_string().green().bold(),
            suffix(n),
            dest_styled
        ),
    };
    print_with_border(message, Some("Download complete"));
    if total == 0 {
        eprintln!(
            "  {} {}",
            "hint:".dimmed(),
            "no files passed --filter/--ignore or bucket was empty; try without --filter or check .acorn.json buckets".dimmed()
        );
    }
}
#[allow(clippy::too_many_arguments)]
pub async fn run(
    config: &Option<PathBuf>,
    url: &[String],
    filter: &[String],
    ignore: &[String],
    output: &Option<PathBuf>,
    flatten: bool,
    clobber: bool,
    strict: bool,
    canonicalize: bool,
    database_path: &Option<PathBuf>,
    threads: usize,
    verbose: &Verbosity,
    offline: bool,
    no_local_database: bool,
) -> Result<(), Report> {
    let quiet = verbose.is_silent();
    match offline {
        | true => Err(eyre!("Offline mode is not supported for bucket downloads")),
        | false => {
            preflight!(&CommandOptions::init()
                .offline(offline)
                .quiet(quiet)
                .threads(threads)
                .supported(true)
                .build());
            let options = BucketOptions::init()
                .common(CommonRuntime {
                    filter: filter.to_vec(),
                    ignore: ignore.to_vec(),
                    offline,
                    quiet,
                    threads,
                    ..CommonRuntime::default()
                })
                .extension(BucketExtension {
                    canonicalize,
                    clobber,
                    database_path: database_path.clone(),
                    flatten,
                    no_local_database,
                    strict,
                })
                .maybe_output(output.clone())
                .build();
            match ApplicationConfiguration::resolve(config) {
                | Some(path) => match ApplicationConfiguration::read(path.clone()) {
                    | Ok(content) => Buckets::from(content.buckets.unwrap_or_default())
                        .transfer(&options, TransferPolicy::Download)
                        .await
                        .inspect_err(|why| error!("=> {} Download data — {why}", Label::fail()))
                        .and_then(|total| finish_transfer(total, &options)),
                    | Err(why) => {
                        error!("=> {} Download data — {why}", Label::fail());
                        let BucketExtension {
                            database_path,
                            no_local_database,
                            ..
                        } = &options.extension;
                        if !*no_local_database {
                            log_activity("download", path.file_name_with_parent(), database_path, false);
                        }
                        Err(eyre!("Failed to process download configuration"))
                    }
                },
                | None if !url.is_empty() => {
                    let buckets = url.iter().map(|value| Bucket::from(value.as_str())).collect::<Vec<_>>();
                    Buckets::from(buckets)
                        .transfer(&options, TransferPolicy::Download)
                        .await
                        .inspect_err(|why| error!("=> {} Download data — {why}", Label::fail()))
                        .and_then(|total| finish_transfer(total, &options))
                }
                | None if !quiet => {
                    warn!(
                        "=> {} No URL or configuration file found; provide a URL or create one of: {}",
                        Label::warning(),
                        DEFAULT_CONFIG_FILENAMES.join(", ")
                    );
                    Ok(())
                }
                | None => {
                    error!("=> {} No URL or configuration file found for download command", Label::fail());
                    Ok(())
                }
            }
        }
    }
}
#[cfg(test)]
mod tests;
