//! # 🌱 ACORN CLI
use acorn::io::env_var_is_truthy;
use acorn::prelude::env;
use acorn::util::constants::env::{CACHE_TTL, NO_LOCAL_DATABASE};
use acorn::util::Label;
use clap::Parser;
use color_eyre::eyre::eyre;
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
#[cfg(feature = "pdf")]
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
    dotenv().ok();
    let args = Arguments::parse();
    if args.markdown_help {
        clap_markdown::print_help_markdown::<Arguments>();
        return Ok(());
    }
    init_tracing(&args);
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
    if let Some(ttl) = cache_ttl {
        env::set_var(CACHE_TTL, ttl.to_string());
    }
    let pool_result = if threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .map(|_| debug!("{} {} threads", Label::using(), threads.cyan().bold()))
            .map_err(|why| eyre!("Initialize thread pool — {why}"))
    } else {
        Ok(())
    };
    let config = DatabaseConfig {
        offline,
        no_local_database,
        database_backend: selected_database_backend.as_ref().map(|b| b.to_string()),
        database_path,
        clear_cache,
        reset_database,
        no_clear_cache,
        cache_ttl,
        initial_download: true,
    };
    let pre_check = if no_local_database && (reset_database || clear_cache) {
        Err(eyre!("Cannot use --reset-database or --clear-cache with --no-local-database"))
    } else {
        Ok(())
    };
    match pre_check.and(pool_result).and_then(|_| run(&args, offline, &config, threads)) {
        | Ok(()) => exit(exitcode::OK),
        | Err(report) => {
            eprintln!("Error: {report:?}");
            exit(exitcode::SOFTWARE);
        }
    }
}
fn init_tracing(args: &Arguments) {
    let indicatif_layer = IndicatifLayer::new();
    let writer = indicatif_layer.get_stderr_writer();
    let filter = dotenvy::var(LOG_LEVEL)
        .ok()
        .filter(|value| !value.is_empty())
        .map_or_else(|| EnvFilter::new(args.verbose.log_level_filter().as_trace().to_string()), EnvFilter::new);
    let format = fmt::layer()
        .with_ansi(true)
        .with_ansi_sanitization(false)
        .with_level(DISPLAY_LEVEL)
        .with_line_number(DISPLAY_LINE_NUMBER)
        .with_target(DISPLAY_TARGET)
        .with_thread_ids(DISPLAY_THREAD_ID)
        .with_thread_names(DISPLAY_THREAD_NAME)
        .without_time()
        .compact()
        .with_writer(writer);
    tracing_subscriber::registry().with(filter).with(indicatif_layer).with(format).init();
}

#[cfg(test)]
mod test;
