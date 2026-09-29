#[cfg(feature = "pdf")]
use crate::commands::export::pdf;
use crate::{
    cli::{arguments::Download, CommandOptions},
    commands::preflight,
};
use acorn::{
    analyzer::host::{
        runtime::StaticAnalyzer,
        vale::{Vale, ValeConfig},
    },
    io::{
        config::{ApplicationConfiguration, BucketExtension, BucketOptions, TransferPolicy},
        database::{
            schema::{ActivityRow, Table},
            Database, Operations,
        },
        print_with_border,
        sidecar::{needle::Needle, SidecarAssets},
        slice_path,
    },
    util::{constants::app::DISPLAY_PATH_CHARACTER_COUNT, StringConversion},
};
use acorn_core::{
    options::{CommonRuntime, RuntimeOptions},
    util::{suffix, MimeType},
};
use acorn_host::terminal::Label;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{eyre, Report, Result};
use futures::{future, stream, StreamExt, TryStreamExt};
use jiff::Timestamp;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use tracing::error;
pub mod model;
/// Install or resolve the pinned Chromium build used by PDF export.
#[cfg(feature = "pdf")]
pub async fn chromium(offline: bool) -> Result<(), Report> {
    pdf::browser_executable(offline, None)
        .await
        .and_then(|path| path.ok_or_else(|| eyre!("Chromium is unavailable for this platform")))
        .map(|path| println!("{}", path.display()))
}
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
    arguments: &Download,
    database_path: &Option<PathBuf>,
    threads: usize,
    offline: bool,
    no_local_database: bool,
) -> Result<(), Report> {
    let Download {
        url,
        config,
        filter,
        ignore,
        output,
        flatten,
        clobber,
        strict,
        canonicalize,
        archive_format,
        verbose,
        ..
    } = arguments;
    let quiet = verbose.is_silent();
    preflight!(&CommandOptions::init()
        .offline(offline)
        .quiet(quiet)
        .threads(threads)
        .supported(true)
        .build());
    let options = BucketOptions::init()
        .common(CommonRuntime {
            filter: filter.clone(),
            ignore: ignore.clone(),
            offline,
            quiet,
            threads,
            ..CommonRuntime::default()
        })
        .extension(BucketExtension {
            canonicalize: *canonicalize,
            clobber: *clobber,
            database_path: database_path.clone(),
            flatten: *flatten,
            no_local_database,
            strict: *strict,
        })
        .maybe_output(output.clone())
        .build();
    let shared = &options;
    let sources = ApplicationConfiguration::sources(config, url);
    let empty = sources.as_ref().map(|sources| sources.is_empty()).unwrap_or_default();
    let result = match empty {
        | true => Err(eyre!("No download URL or configuration file found")),
        | false => match sources {
            | Ok(sources) => {
                stream::iter(sources)
                    .then(|source| async move {
                        match source.resolve(archive_format.map(MimeType::from), offline).await {
                            | Ok(bucket) => bucket.transfer(shared, TransferPolicy::Download).await,
                            | Err(why) => Err(why),
                        }
                    })
                    .try_fold(0_usize, |total, count| future::ready(Ok(total.saturating_add(count))))
                    .await
            }
            | Err(why) => Err(why),
        },
    };
    match result {
        | Ok(total) => finish_transfer(total, &options),
        | Err(why) => {
            error!("=> {} Download data — {why}", Label::fail());
            let BucketExtension {
                database_path,
                no_local_database,
                ..
            } = &options.extension;
            match config {
                | Some(path) if !*no_local_database => {
                    log_activity("download", path.file_name_with_parent(), database_path, false);
                }
                | _ => {}
            }
            Err(why)
        }
    }
}
/// Install or resolve the pinned Vale build used by prose analysis.
pub async fn vale(offline: bool) -> Result<(), Report> {
    Vale::resolve(ValeConfig::default(), offline, false)
        .await
        .map(|vale| println!("{}", vale.command()))
}
#[cfg(test)]
mod tests;
