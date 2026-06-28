use acorn::io::database::{backend, resolve_database_path, schema::Table, Database, Operations};
use acorn::prelude::env;
use acorn::util::constants::env::{CACHE_TTL, DATABASE_BACKEND, DATABASE_PATH};
use acorn::util::Label;
use color_eyre::eyre::Report;
use futures::future::ready;
use futures::TryFutureExt;
use std::path::PathBuf;
use std::process::exit;
use tracing::info;

pub struct DatabaseConfig {
    pub no_local_database: bool,
    pub database_backend: Option<String>,
    pub database_path: Option<PathBuf>,
    pub clear_cache: bool,
    pub reset_database: bool,
    pub no_clear_cache: bool,
    pub cache_ttl: Option<u64>,
}
pub async fn initialize_database(config: &DatabaseConfig) -> Result<(), Report> {
    if config.no_local_database {
        return Ok(());
    }
    let mut database_path = config.database_path.clone();
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
    if let Some(selected) = &config.database_backend {
        env::set_var(DATABASE_BACKEND, selected);
    }
    if let Err(why) = backend::validate_backend_selection() {
        eprintln!("=> {} {why}", Label::fail());
        exit(exitcode::USAGE);
    }
    if let Some(ttl) = config.cache_ttl {
        env::set_var(CACHE_TTL, ttl.to_string());
    }
    let database = Database::<Table>::from_path(database_path.clone());
    if config.reset_database {
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
    if config.clear_cache {
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
        if !config.no_clear_cache {
            if let Err(why) = database.cleanup_expired_cache() {
                info!("Cache cleanup warning — {why}");
            }
        }
        database.with_connection(|_| Ok(())).map(|_| ())
    }))
    .and_then(|_| database.populate(Table::Licenses))
    .and_then(|_| database.populate(Table::ProgrammingLanguages))
    .and_then(|_| database.populate(Table::Models))
    .and_then(|_| database.populate(Table::Providers))
    .await
    .map(|_| ())
}
