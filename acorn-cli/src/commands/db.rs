#![allow(clippy::std_instead_of_alloc)]
use acorn::io::database::{backend, resolve_database_path, schema::Table, Database, Operations};
use acorn::prelude::env;
use acorn::util::constants::env::{CACHE_TTL, DATABASE_BACKEND, DATABASE_PATH};
use acorn::util::Label;
use color_eyre::eyre::Report;
use futures::future::ready;
use futures::TryFutureExt;
use std::path::PathBuf;
use std::sync::Arc;
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
                        info!("{} Reset database - cleared {count} entries from all tables", Label::pass());
                    })
                    .map(|_| ())
            } else {
                Ok(())
            };
            let cleared = reset.and_then(|_| {
                if config.clear_cache {
                    db.clear_cache()
                        .map_err(|why| {
                            eprintln!("=> {} Failed to clear cache — {why}", Label::fail());
                            Report::msg(format!("Failed to clear cache — {why}"))
                        })
                        .inspect(|count| {
                            info!("{} Cleared {count} entries from cache tables", Label::pass());
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
                                info!("Cache cleanup warning — {why}");
                            }
                        }
                        db.with_connection(|_| Ok(())).map(|_| ())
                    }))
                })
                .and_then({
                    let db = db.clone();
                    move |_| async move { db.populate(Table::Licenses).await }
                })
                .and_then({
                    let db = db.clone();
                    move |_| async move { db.populate(Table::ProgrammingLanguages).await }
                })
                .and_then({
                    let db = db.clone();
                    move |_| async move { db.populate(Table::Models).await }
                })
                .and_then({
                    let db = db.clone();
                    move |_| async move { db.populate(Table::Providers).await }
                })
        })
        .await
        .map(|_| ())
    }
}
