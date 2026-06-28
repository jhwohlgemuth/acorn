//! # 🌱 ACORN CLI
use acorn::io::env_var_is_truthy;
use acorn::prelude::env;
use acorn::util::constants::env::{CACHE_TTL, NO_LOCAL_DATABASE};
use acorn::util::Label;
use clap::Parser;
use dotenvy::dotenv;
use owo_colors::OwoColorize;
use std::process::exit;
use tracing::debug;
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
use commands::{run, DatabaseConfig};

// Environment variables
const LOG_LEVEL: &str = "ACORN_LOG_LEVEL";
const DISPLAY_LEVEL: bool = true;
const DISPLAY_LINE_NUMBER: bool = false;
const DISPLAY_TARGET: bool = false;
const DISPLAY_THREAD_ID: bool = false;
const DISPLAY_THREAD_NAME: bool = false;

fn main() -> Void {
    color_eyre::install()?;
    let args = Arguments::parse();
    if args.markdown_help {
        clap_markdown::print_help_markdown::<Arguments>();
        exit(exitcode::OK);
    }
    match dotenv() {
        | Ok(_) if !dotenvy::var(LOG_LEVEL).unwrap_or_default().is_empty() => {
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
    let database_path = configured_database_path.clone();
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
    let config = DatabaseConfig {
        no_local_database,
        database_backend: selected_database_backend.as_ref().map(|b| b.to_string()),
        database_path,
        clear_cache,
        reset_database,
        no_clear_cache,
        cache_ttl,
    };
    run(&args, offline, &config, threads)
}

#[cfg(test)]
mod test;
