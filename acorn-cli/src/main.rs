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
use acorn::io::database::{backend, resolve_database_path, schema::Table, Database, Operations};
use acorn::io::env_var_is_truthy;
use acorn::prelude::env;
use acorn::util::constants::env::{CACHE_TTL, DATABASE_BACKEND, DATABASE_PATH, NO_LOCAL_DATABASE};
use acorn::util::Label;
use clap::{crate_version, Parser};
use dotenvy::dotenv;
use futures::future::ready;
use futures::TryFutureExt;
use owo_colors::OwoColorize;
use std::process::exit;
use tracing::instrument;
use tracing::{debug, info};
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

use cli::{Arguments, Void};
use commands::run;

// Environment variables
const LOG_LEVEL: &str = "ACORN_LOG_LEVEL";
const DISPLAY_LEVEL: bool = true;
const DISPLAY_LINE_NUMBER: bool = false;
const DISPLAY_TARGET: bool = false;
const DISPLAY_THREAD_ID: bool = false;
const DISPLAY_THREAD_NAME: bool = false;

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
                .with_ansi_sanitization(false)
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
                .with_ansi_sanitization(false)
                .with_max_level(args.verbose.log_level_filter().as_trace())
                .event_format(format)
                .init();
        }
    }
    let Arguments {
        offline,
        threads,
        no_local_database: no_local_database_flag,
        database_backend: ref selected_database_backend,
        database_path: ref configured_database_path,
        clear_cache,
        reset_database,
        no_clear_cache,
        cache_ttl,
        ..
    } = args;
    let no_local_database = no_local_database_flag || env_var_is_truthy(NO_LOCAL_DATABASE).unwrap_or(false);
    let mut database_path = configured_database_path.clone();
    if no_local_database && (reset_database || clear_cache) {
        eprintln!("{} Cannot use --reset-database or --clear-cache with --no-local-database", Label::fail());
        exit(exitcode::USAGE);
    }
    if let Some(ttl) = cache_ttl {
        env::set_var(CACHE_TTL, ttl.to_string());
    }
    if threads > 0 {
        match rayon::ThreadPoolBuilder::new().num_threads(threads).build_global() {
            | Ok(()) => {
                debug!("{} {} threads", Label::using(), threads.cyan().bold());
            }
            | Err(why) => {
                eprintln!("=> {} Initialize thread pool — {why}", Label::fail());
                exit(exitcode::SOFTWARE);
            }
        }
    }
    let database_initialization = if no_local_database {
        Ok(())
    } else {
        let resolved_database_path = match resolve_database_path(database_path.as_ref()) {
            | Ok(path) => path,
            | Err(why) => {
                eprintln!("=> {} Failed to determine database path — {why}", Label::fail());
                exit(exitcode::SOFTWARE);
            }
        };
        env::set_var(DATABASE_PATH, resolved_database_path.as_os_str());
        if database_path.is_none() {
            database_path = Some(resolved_database_path);
        }
        if let Some(selected) = *selected_database_backend {
            env::set_var(DATABASE_BACKEND, selected.to_string());
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
        .and_then(|_| database.populate(Table::ProgrammingLanguages))
        .await
        .map(|_| ())
    };
    if let Err(why) = database_initialization {
        eprintln!("=> {} Failed to initialize database — {why}", Label::fail());
        exit(exitcode::SOFTWARE);
    }
    Box::pin(run(&args, offline, &database_path, threads)).await
}

#[cfg(test)]
mod test;
