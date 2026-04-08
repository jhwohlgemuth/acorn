//! # 🌱 ACORN CLI
//! "Plant an ACORN and grow your science"
//!
//!⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣤⣄⣀⠀⠀⠀
//!⠀⠀⠀⠀⠀⠀⠀⣀⠀⢴⣶⠀⢶⣦⠀⢄⣀⠀⠠⢾⣿⠿⠿⠿⠿⢦⠀
//!⠀⠀⠀⠀⠀⠀⠺⠿⠇⢸⣿⣇⠘⣿⣆⠘⣿⡆⠠⣄⡀⠀⠀⠀⠀⠀⠀    
//!⠀⠀⠀⠀⢀⣴⣶⣶⣤⣄⡉⠛⠀⢹⣿⡄⢹⣿⡀⢻⣧⠀⡀⠀⠀⠀⠀    
//!⠀⠀⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣶⣤⡈⠓⠀⣿⣧⠈⢿⡆⠸⡄⠀⠀⠀   █████████     █████████     ███████    ███████████   ██████   █████
//!⠀⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣦⣈⠙⢆⠘⣿⡀⢻⠀⠀   ███▒▒▒▒▒███   ███▒▒▒▒▒███  ███▒▒▒▒▒███ ▒▒███▒▒▒▒▒███ ▒▒██████ ▒▒███
//!⠀⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠀⠹⣧⠈⠀⠀  ▒███    ▒███  ███     ▒▒▒  ███     ▒▒███ ▒███    ▒███  ▒███▒███ ▒███
//!⠀⣸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠈⠃⠀⠀⠀ ▒███████████ ▒███         ▒███      ▒███ ▒██████████   ▒███▒▒███▒███
//!⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⠀⠀ ▒███▒▒▒▒▒███ ▒███         ▒███      ▒███ ▒███▒▒▒▒▒███  ▒███ ▒▒██████⠀
//!⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⠀⠀⠀ ▒███    ▒███ ▒▒███     ███▒▒███     ███  ▒███    ▒███  ▒███  ▒▒█████
//!⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠃⠀⠀⠀⠀⠀⠀ █████   █████ ▒▒█████████  ▒▒▒███████▒   █████   █████ █████  ▒▒█████⠀
//!⠀⢹⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀▒▒▒▒▒   ▒▒▒▒▒   ▒▒▒▒▒▒▒▒▒     ▒▒▒▒▒▒▒    ▒▒▒▒▒   ▒▒▒▒▒ ▒▒▒▒▒    ▒▒▒▒▒
//!⠀⠈⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠟⠉⠀⠀⠀⠀⠀⠀⠀    
//! ⠀ ⣿⣿⠿⠿⠿⠿⠿⠿⠟⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀~ Accessible Content Optimization for Research Needs ~⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//!
//!
//! The ACORN command line interface (CLI) is a metascience multi-tool made for cross-domain research.
//!
//! See [the ACORN project README](https://code.ornl.gov/research-enablement/acorn#installation) for installation instructions.
use crate::io::{is_stdout_piped, read_stdin, write_stdout};
use acorn::io::database::{backend, schema::Table, Database, Operations};
use acorn::io::env_var_is_truthy;
use acorn::prelude::env;
use acorn::schema::pid::raid::Metadata;
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::constants::{ENV_CACHE_TTL, ENV_DATABASE_BACKEND, ENV_DATABASE_PATH, ENV_NO_LOCAL_DATABASE};
use acorn::util::Label;
use clap::{crate_version, Parser};
use color_eyre::eyre;
use dotenvy::dotenv;
use futures::future::ready;
use futures::TryFutureExt;
use owo_colors::OwoColorize;
use std::process::exit;
use tracing::info;
use tracing::instrument;
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
#[tokio::main]
async fn main() -> Void {
    color_eyre::install()?;
    let args = Arguments::parse();
    if args.markdown_help {
        clap_markdown::print_help_markdown::<Arguments>();
        exit(exitcode::OK);
    }
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
    let Arguments {
        offline,
        threads,
        no_local_database: no_local_database_flag,
        database_backend,
        database_path,
        clear_cache,
        reset_database,
        no_clear_cache,
        cache_ttl,
        ..
    } = args;
    let no_local_database = no_local_database_flag || env_var_is_truthy(ENV_NO_LOCAL_DATABASE).unwrap_or(false);
    if no_local_database && (reset_database || clear_cache) {
        eprintln!("{} Cannot use --reset-database or --clear-cache with --no-local-database", Label::fail());
        exit(exitcode::USAGE);
    }
    if let Some(ttl) = cache_ttl {
        env::set_var(ENV_CACHE_TTL, ttl.to_string());
    }
    if threads > 0 {
        info!("{} {} threads", Label::using(), threads.cyan().bold());
        rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();
    }
    let database_initialization = if no_local_database {
        Ok(())
    } else {
        if let Some(path) = database_path.as_ref() {
            env::set_var(ENV_DATABASE_PATH, path.as_os_str());
        }
        if let Some(selected) = database_backend {
            env::set_var(ENV_DATABASE_BACKEND, selected.to_string());
        }
        if let Err(why) = backend::validate_backend_selection() {
            eprintln!("=> {} {why}", Label::fail());
            exit(exitcode::USAGE);
        }
        let database = Database::<Table>::from_path(database_path.clone());
        if reset_database {
            match database.reset() {
                | Ok(count) => {
                    info!("{} Reset database - cleared {count} entries from all tables", Label::pass());
                }
                | Err(why) => {
                    eprintln!("=> {} Failed to clear all data — {why}", Label::fail());
                    exit(exitcode::SOFTWARE);
                }
            }
        }
        if clear_cache {
            match database.clear_cache() {
                | Ok(count) => {
                    info!("{} Cleared {count} entries from cache tables", Label::pass());
                }
                | Err(why) => {
                    eprintln!("=> {} Failed to clear cache — {why}", Label::fail());
                    exit(exitcode::SOFTWARE);
                }
            }
        }
        ready(database.migrate().and_then(|_| {
            if !no_clear_cache {
                if let Err(why) = database.cleanup_expired_cache() {
                    info!("Cache cleanup warning — {why}");
                }
            }
            database.with_connection(|_| Ok(())).map(|_| ())
        }))
        .and_then(|_| database.populate(Table::Licenses))
        .await
        .map(|_| ())
    };
    if let Err(why) = database_initialization {
        eprintln!("=> {} Failed to initialize database — {why}", Label::fail());
        exit(exitcode::SOFTWARE);
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
            no_fail,
            terse,
            standard,
            readability_metric,
            verbose,
            ..
        }) => {
            commands::check::run(
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
                no_fail,
                terse,
                standard,
                readability_metric,
                verbose,
            )
            .await
        }
        | Some(Commands::Doctor { fix, interactive, check, .. }) => commands::doctor::run(fix, interactive, check, &offline).await,
        | Some(Commands::Download {
            config,
            url,
            output,
            verbose,
            ..
        }) => commands::download::run(config, url, output, &offline, &database_path, threads, verbose).await,
        | Some(Commands::Export {
            output,
            path,
            branch,
            commit,
            format,
            reference,
            combine,
            merge_request,
            verbose,
            ..
        }) => {
            commands::export::run(
                output,
                path,
                branch,
                commit,
                format,
                reference,
                combine,
                merge_request,
                &offline,
                threads,
                verbose,
            )
            .await
        }
        | Some(Commands::Format {
            path,
            branch,
            commit,
            ignore,
            dry_run,
            merge_request,
            verbose,
            ..
        }) => commands::format::run(path, branch, commit, ignore, *dry_run, merge_request, verbose, &offline).await,
        | Some(Commands::Gather { path, ignore, verbose, .. }) => commands::gather::run(path, ignore, &offline, &database_path, verbose).await,
        | Some(Commands::Link {
            path,
            branch,
            commit,
            ignore,
            dry_run,
            merge_request,
            verbose,
            ..
        }) => commands::link::run(path, branch, commit, ignore, *dry_run, *merge_request, verbose).await,
        | Some(Commands::Schema { command, .. }) => match command {
            | Some(SchemaCommands::Rad {}) | None => print_rad_schema(),
            | Some(SchemaCommands::Raid {}) => print_raid_schema(),
        },
        | None => {
            let name = "JASON";
            acorn::fail!("FOOBAR {} RABOOF", name);
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

#[cfg(test)]
mod test_utils;
