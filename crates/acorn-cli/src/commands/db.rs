#![allow(clippy::std_instead_of_alloc)]
use crate::io::chromium_cache_dir;
use acorn::{
    io::database::{backend, resolve_database_path, schema::Table, Database, Operations},
    util::constants::env::{CACHE_TTL, DATABASE_BACKEND, DATABASE_PATH},
};
use acorn_core::prelude::Arc;
use acorn_host::terminal::Label;
use color_eyre::eyre::Report;
use futures::{future::ready, TryFutureExt};
use std::{
    env,
    fs::remove_dir_all,
    path::{Path, PathBuf},
};
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct DatabaseConfig {
    pub offline: bool,
    pub no_local_database: bool,
    pub database_backend: Option<String>,
    pub database_path: Option<PathBuf>,
    pub clear_cache: bool,
    pub reset_database: bool,
    pub no_clear_cache: bool,
    pub cache_ttl: Option<u64>,
    pub initial_download: bool,
}
/// Determine whether a database error is a transient lock conflict.
fn is_database_lock_error(report: &Report) -> bool {
    let message = report.to_string();
    message.contains("Conflicting lock is held")
        || message.contains("Could not set lock")
        || message.contains("database is locked")
        || (message.contains("IO Error") && message.to_ascii_lowercase().contains("lock"))
}

pub async fn initialize_database(config: &DatabaseConfig) -> Result<(), Report> {
    initialize_database_with_policy(config, false).await
}

/// Initialize the database, optionally warning instead of failing on lock conflicts.
///
/// When `warn_on_lock` is true, a lock conflict is logged and treated as success
/// so that validation-only commands (`check`, `format`, `link`) can proceed
/// without cache while `download`/`sync` remain fail-closed.
pub async fn initialize_database_with_policy(config: &DatabaseConfig, warn_on_lock: bool) -> Result<(), Report> {
    let result = initialize_database_inner(config).await;
    match result {
        | Ok(()) => Ok(()),
        | Err(report) if warn_on_lock && is_database_lock_error(&report) => {
            warn!("=> {} Database lock unavailable — proceeding without cache ({report})", Label::warning());
            eprintln!("=> {} Database lock unavailable — proceeding without cache", Label::warning());
            Ok(())
        }
        | Err(report) => Err(report),
    }
}

async fn initialize_database_inner(config: &DatabaseConfig) -> Result<(), Report> {
    if config.no_local_database {
        Ok(())
    } else {
        let mut database_path = config.database_path.clone();
        ready(
            resolve_database_path(database_path.as_ref())
                .map_err(|why| {
                    eprintln!("=> {} Failed to determine database path — {why}", Label::fail());
                    Report::msg(format!("Failed to determine database path — {why}"))
                })
                .inspect(|resolved| {
                    env::set_var(DATABASE_PATH, resolved.as_os_str());
                    if database_path.is_none() {
                        database_path = Some(resolved.clone());
                    }
                })
                .and_then(|_| {
                    if let Some(selected) = &config.database_backend {
                        env::set_var(DATABASE_BACKEND, selected);
                    }
                    backend::validate_backend_selection()
                        .map_err(|why| {
                            eprintln!("=> {} {why}", Label::fail());
                            Report::msg(format!("{why}"))
                        })
                        .map(|_| {
                            if let Some(ttl) = config.cache_ttl {
                                env::set_var(CACHE_TTL, ttl.to_string());
                            }
                            Database::<Table>::from_path(database_path.clone())
                        })
                }),
        )
        .and_then(|db| {
            let db = Arc::new(db);
            let reset = if config.reset_database {
                db.reset()
                    .map_err(|why| {
                        eprintln!("=> {} Failed to clear all data — {why}", Label::fail());
                        Report::msg(format!("Failed to clear all data — {why}"))
                    })
                    .inspect(|count| {
                        info!("=> {} Reset database - cleared {count} entries from all tables", Label::pass());
                    })
                    .map(|_| ())
            } else {
                Ok(())
            };
            let cleared = reset.and_then(|_| {
                if config.clear_cache {
                    db.clear_cache()
                        .and_then(|count| clear_chromium_cache().map(|_| count))
                        .map_err(|why| {
                            eprintln!("=> {} Failed to clear cache — {why}", Label::fail());
                            Report::msg(format!("Failed to clear cache — {why}"))
                        })
                        .inspect(|count| {
                            debug!("=> {} Cleared {count} entries from cache tables", Label::pass());
                        })
                        .map(|_| ())
                } else {
                    Ok(())
                }
            });
            let db_for_migrate = db.clone();
            ready(cleared)
                .and_then(move |_| {
                    let db = db_for_migrate.clone();
                    ready(db.migrate().and_then(|_| {
                        if !config.no_clear_cache {
                            if let Err(why) = db.cleanup_expired_cache() {
                                debug!("=> Cache cleanup warning — {why}");
                            }
                        }
                        db.with_connection(|_| Ok(())).map(|_| ())
                    }))
                })
                .and_then({
                    let db = db.clone();
                    move |_| async move {
                        match config.offline || !config.initial_download {
                            | true => Ok(()),
                            | false => db.populate(Table::Licenses).await.map(|_| ()),
                        }
                    }
                })
                .and_then({
                    let db = db.clone();
                    move |_| async move {
                        match config.offline || !config.initial_download {
                            | true => Ok(()),
                            | false => db.populate(Table::ProgrammingLanguages).await.map(|_| ()),
                        }
                    }
                })
                .and_then({
                    let db = db.clone();
                    move |_| async move {
                        match config.offline || !config.initial_download {
                            | true => Ok(()),
                            | false => db.populate(Table::Models).await.map(|_| ()),
                        }
                    }
                })
                .and_then({
                    let db = db.clone();
                    move |_| async move {
                        match config.offline || !config.initial_download {
                            | true => Ok(()),
                            | false => db.populate(Table::Providers).await.map(|_| ()),
                        }
                    }
                })
        })
        .await
        .map(|_| ())
    }
}
fn clear_cache_directory(path: &Path) -> Result<(), Report> {
    match path.exists() {
        | true => remove_dir_all(path).map_err(|why| Report::msg(format!("Failed to clear Chromium cache — {why}"))),
        | false => Ok(()),
    }
}
fn clear_chromium_cache() -> Result<(), Report> {
    chromium_cache_dir()
        .ok_or_else(|| Report::msg("Failed to resolve Chromium cache directory"))
        .and_then(|path| clear_cache_directory(&path))
}
#[cfg(test)]
mod tests;
