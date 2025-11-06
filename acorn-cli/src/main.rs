//! # 🌱 ACORN CLI
//! Multitool for maintaining sustainable research activity data
//!
//!⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣤⣄⣀⠀⠀⠀
//!⠀⠀⠀⠀⠀⠀⠀⣀⠀⢴⣶⠀⢶⣦⠀⢄⣀⠀⠠⢾⣿⠿⠿⠿⠿⢦⠀
//!⠀⠀⠀⠀⠀⠀⠺⠿⠇⢸⣿⣇⠘⣿⣆⠘⣿⡆⠠⣄⡀⠀⠀⠀⠀⠀⠀   .---.    .--.      .--.    ___ .-.     ___ .-.   
//!⠀⠀⠀⠀⢀⣴⣶⣶⣤⣄⡉⠛⠀⢹⣿⡄⢹⣿⡀⢻⣧⠀⡀⠀⠀⠀⠀⠀ / .-, \  /    \    /    \  (   )   \   (   )   \
//!⠀⠀⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣶⣤⡈⠓⠀⣿⣧⠈⢿⡆⠸⡄⠀⠀⠀ (__) ; | |  .-. ;  |  .-. ;  | ' .-. ;   |  .-. .
//!⠀⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣦⣈⠙⢆⠘⣿⡀⢻⠀⠀ ⠀  .'`  | |  |(___) | |  | |  |  / (___)  | |  | |
//!⠀⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠀⠹⣧⠈⠀⠀ ⠀ / .'| | |  |      | |  | |  | |         | |  | | ⠀
//!⠀⣸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠈⠃⠀⠀⠀ | /  | | |  | ___  | |  | |  | |         | |  | |
//!⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⠀⠀ ; |  ; | |  '(   ) | '  | |  | |         | |  | | ⠀
//!⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⠀⠀⠀ ' `-'  | '  `-' |  '  `-' /  | |         | |  | |
//!⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠃⠀⠀⠀⠀⠀⠀⠀`.__.'_.  `.__,'    `.__.'  (___)       (___)(___)
//!⠀⢹⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀
//!⠀⠈⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠟⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀Accessible Content Optimization for Research Needs
//! ⠀ ⣿⣿⠿⠿⠿⠿⠿⠿⠟⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//!
//!
//! The ACORN command line interface (CLI) provides a set of commands for maintaining research activity data and leveraging ACORN schemas.
//!
//! See [the ACORN project README](https://code.ornl.gov/research-enablement/acorn#installation) for installation instructions.
use crate::io::{is_stdout_piped, read_stdin, write_stdout};
use acorn_lib::prelude::exit;
use acorn_lib::schema::pid::raid::Metadata;
use acorn_lib::schema::ResearchActivity;
use acorn_lib::util::Label;
use clap::{crate_version, Parser};
use color_eyre::eyre;
use dotenvy::dotenv;
use owo_colors::OwoColorize;
use tracing::instrument;
use tracing::{info, warn};
use tracing_indicatif::IndicatifLayer;
use tracing_log::AsTrace;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::layer::SubscriberExt as __tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::EnvFilter;

mod cli;
mod commands;
mod io;
mod template;

use cli::{Arguments, Commands, SchemaCommands};

/// Return type for main function
pub type Void = eyre::Result<(), eyre::Report>;

// Environment variables
const LOG_LEVEL: &str = "ACORN_LOG_LEVEL";
const DISPLAY_LEVEL: bool = true;
const DISPLAY_LINE_NUMBER: bool = false;
const DISPLAY_TARGET: bool = false;
const DISPLAY_THREAD_ID: bool = false;
const DISPLAY_THREAD_NAME: bool = false;

fn print_rad_schema() -> Void {
    ResearchActivity::to_schema();
    exit(exitcode::OK);
}
fn print_raid_schema() -> Void {
    Metadata::to_schema();
    exit(exitcode::OK);
}
#[instrument]
fn main() -> Void {
    color_eyre::install()?;
    let args = Arguments::parse();
    if args.version {
        println!("{}", crate_version!());
        exit(exitcode::OK);
    }
    match dotenv() {
        | Ok(_) if !dotenvy::var(LOG_LEVEL).unwrap_or_default().is_empty() => {
            // Configure tracing with a .env file
            let indicatif_layer = IndicatifLayer::new();
            let format = fmt::layer()
                .with_ansi(true)
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
                .with_ansi(true)
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
    let Arguments { offline, threads, .. } = args;
    if threads > 0 {
        info!("{} {} threads", Label::using(), threads.cyan().bold());
        rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();
    }
    match &args.command {
        | Some(Commands::Check {
            path,
            branch,
            commit,
            ignore,
            skip,
            disable_website_checks,
            exit_on_first_error,
            merge_request,
            skip_verify_checksum,
            readability_metric,
            ..
        }) => commands::check::run(
            path,
            branch,
            commit,
            ignore,
            skip,
            disable_website_checks,
            exit_on_first_error,
            merge_request,
            skip_verify_checksum,
            &offline,
            readability_metric,
        ),
        | Some(Commands::Doctor { fix, interactive, check, .. }) => commands::doctor::run(fix, interactive, check, &offline),
        | Some(Commands::Download { buckets, output, .. }) => commands::download::run(buckets, output, &offline),
        | Some(Commands::Export {
            output,
            path,
            branch,
            commit,
            format,
            reference,
            size,
            target,
            merge_request,
            ..
        }) => commands::export::run(output, path, branch, commit, format, reference, size, target, merge_request, &offline),
        | Some(Commands::Format {
            path,
            branch,
            commit,
            ignore,
            dry_run,
            merge_request,
            verbose,
            ..
        }) => commands::format::run(path, branch, commit, ignore, *dry_run, merge_request, verbose, &offline),
        | Some(Commands::Link {
            path,
            branch,
            commit,
            ignore,
            dry_run,
            merge_request,
            ..
        }) => commands::link::run(path, branch, commit, ignore, *dry_run, *merge_request),
        | Some(Commands::Schema { command, .. }) => match command {
            | Some(SchemaCommands::Rad {}) | None => print_rad_schema(),
            | Some(SchemaCommands::Raid {}) => print_raid_schema(),
        },
        | None => {
            if let Some(value) = read_stdin() {
                println!("stdin: {value}");
                info!(value, "=> {} Input from stdin", Label::using());
            }
            // acorn -vv | bat
            if is_stdout_piped() {
                println!("stdout is piped");
                write_stdout("stdout is piped");
            }
            Ok(())
        }
    }
}
