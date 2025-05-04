// #![deny(missing_docs)]
use clap::Parser;
use color_eyre::eyre;
use dotenvy::dotenv;
use tracing::instrument;
use tracing::warn;
use tracing_indicatif::IndicatifLayer;
use tracing_log::AsTrace;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

mod cli;
mod commands;
mod template;
mod util;

use cli::{Arguments, Commands};

pub type Void = eyre::Result<(), eyre::Report>;

// Environment variables
const LOG_LEVEL: &str = "ACORN_LOG_LEVEL";
const DISPLAY_LEVEL: bool = true;
const DISPLAY_LINE_NUMBER: bool = false;
const DISPLAY_TARGET: bool = false;
const DISPLAY_THREAD_ID: bool = false;
const DISPLAY_THREAD_NAME: bool = false;

#[instrument]
fn main() -> Void {
    color_eyre::install()?;
    let args = Arguments::parse();
    match dotenv() {
        | Ok(_) if !dotenvy::var(LOG_LEVEL).unwrap_or_default().is_empty() => {
            // Configure tracing with a .env file
            let indicatif_layer = IndicatifLayer::new();
            let format = fmt::layer()
                .with_level(DISPLAY_LEVEL)
                .with_line_number(DISPLAY_LINE_NUMBER)
                .with_target(DISPLAY_TARGET)
                .with_thread_ids(DISPLAY_THREAD_ID)
                .with_thread_names(DISPLAY_THREAD_NAME)
                .compact();
            tracing_subscriber::registry()
                .with(format)
                .with(indicatif_layer)
                .with(EnvFilter::from_env(LOG_LEVEL))
                .init();
        }
        | _ => {
            // Configure tracing with command line argument
            let format = fmt::format()
                .with_level(DISPLAY_LEVEL)
                .with_line_number(DISPLAY_LINE_NUMBER)
                .with_target(DISPLAY_TARGET)
                .with_thread_ids(DISPLAY_THREAD_ID)
                .with_thread_names(DISPLAY_THREAD_NAME)
                .without_time()
                .compact();
            tracing_subscriber::fmt()
                .with_max_level(args.verbose.log_level_filter().as_trace())
                .event_format(format)
                .init();
        }
    }
    match &args.command {
        | Some(Commands::Check {
            path,
            commit,
            ignore,
            skip,
            exit_on_first_error,
            ..
        }) => commands::check::run(path, commit, ignore, skip, exit_on_first_error),
        | Some(Commands::Doctor { fix, interactive, check, .. }) => commands::doctor::run(fix, interactive, check),
        | Some(Commands::Download { buckets, output, .. }) => commands::download::run(buckets, output),
        | Some(Commands::Export {
            output,
            path,
            commit,
            format,
            size,
            target,
            schema,
            ..
        }) => commands::export::run(output, path, commit, format, size, target, schema),
        | Some(Commands::Format {
            path,
            commit,
            ignore,
            dry_run,
            verbose,
            ..
        }) => commands::format::run(path, commit, ignore, *dry_run, verbose),
        | None => Ok(()),
    }
}
