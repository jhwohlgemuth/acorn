//! SQLite database utilities for ACORN.
//!
//! This module provides a lazily initialized, process-global SQLite connection
//! guarded by a mutex and a helper to execute callbacks against that connection.

use crate::prelude::{create_dir_all, Mutex, OnceLock, Path, PathBuf};
use crate::util::constants::{APPLICATION, DATABASE_NAME, ORGANIZATION, QUALIFIER};
use crate::util::Label;
use chrono::Utc;
use color_eyre::eyre::{self, eyre};
use directories::ProjectDirs;
use owo_colors::OwoColorize;
use rusqlite::{params, Connection};
use tracing::debug;

mod schema;

static CONNECTION: OnceLock<Mutex<Connection>> = OnceLock::new();

/// Persist and access data in a database
pub trait Connect {
    /// Load data from database into the implementing type.
    fn load(data: Self) -> Self;
    /// Save the implementing type's data to database.
    fn save(&self);
}
fn database_path(path: Option<&PathBuf>) -> eyre::Result<PathBuf, eyre::Report> {
    match path {
        | Some(path) => Ok(path.clone()),
        | None => ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .ok_or_else(|| eyre!("Failed to determine project directories"))
            .map(|directories| directories.cache_dir().join("database").join(DATABASE_NAME)),
    }
    .and_then(|path| {
        path.parent()
            .map(PathBuf::from)
            .ok_or_else(|| eyre!("Failed to determine database parent directory"))
            .and_then(|parent| {
                create_dir_all(&parent)
                    .map_err(|why| eyre!("Failed to create database directory: {why}"))
                    .map(|_| path)
            })
    })
}
fn sqlite_connection(path: Option<&PathBuf>) -> eyre::Result<&'static Mutex<Connection>, eyre::Report> {
    match CONNECTION.get() {
        | Some(connection) => Ok(connection),
        | None => database_path(path)
            .and_then(|db_path| {
                Connection::open(db_path)
                    .map(Mutex::new)
                    .map_err(|why| eyre!("Failed to initialize SQLite connection: {why}"))
            })
            .and_then(|connection| {
                CONNECTION.set(connection).ok();
                CONNECTION
                    .get()
                    .ok_or_else(|| eyre!("Failed to access SQLite connection after initialization"))
            }),
    }
}
/// Executes a callback using a lazily initialized global SQLite connection,
/// optionally overriding the default database path.
///
/// # Examples
///
/// ```ignore
/// use acorn::io::database::with_connection;
///
/// let result = with_connection(|conn| {
///     conn.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT)", [])
///         .map(|_| ())
///         .map_err(|why| Err("Failed to create table: {why}"))
/// }, None);
/// ```
pub fn with_connection<T>(
    callback: impl FnOnce(&Connection) -> eyre::Result<T, eyre::Report>,
    path: Option<&PathBuf>,
) -> eyre::Result<T, eyre::Report> {
    sqlite_connection(path)
        .and_then(|connection| connection.lock().map_err(|why| eyre!("Failed to acquire SQLite connection lock: {why}")))
        .and_then(|guard| callback(&guard))
}
/// Saves usage information (date and path) to the database,
/// optionally overriding the default database path.
pub fn save_usage(path: &Path, database_path_override: Option<&PathBuf>) {
    with_connection(
        |conn| {
            conn.execute("CREATE TABLE IF NOT EXISTS usage (date TEXT NOT NULL, path TEXT NOT NULL)", [])
                .and_then(|_| {
                    let now = Utc::now().to_rfc3339();
                    let path_str = path.to_string_lossy().to_string();
                    conn.execute("INSERT INTO usage (date, path) VALUES (?, ?)", params![now, path_str])
                })
                .map(|_| ())
                .map_err(|why| {
                    debug!("Failed to save usage to database: {why}");
                    eyre!("Database error: {why}")
                })
        },
        database_path_override,
    )
    .unwrap_or_else(|why| {
        debug!("Database connection failed: {why}");
    });
}
/// Queries and prints recent usage history from the database,
/// optionally overriding the default database path.
pub fn print_usage(path: Option<&PathBuf>) {
    with_connection(
        |conn| {
            conn.prepare("SELECT date, path FROM usage ORDER BY date DESC LIMIT 10")
                .map_err(|why| eyre!("Failed to prepare query: {why}"))
                .and_then(|mut stmt| {
                    stmt.query_map([], |row| {
                        row.get::<_, String>(0).and_then(|date| row.get::<_, String>(1).map(|path| (date, path)))
                    })
                    .map_err(|why| eyre!("Query execution failed: {why}"))
                    .map(|rows| {
                        println!("\n{} Recent usage history:", Label::using());
                        rows.enumerate().for_each(|(idx, result)| {
                            result
                                .map(|(date, path)| {
                                    println!("  {}. {} - {}", idx + 1, date.cyan(), path.yellow());
                                })
                                .unwrap_or_else(|why| {
                                    debug!("Failed to read row: {why}");
                                });
                        });
                    })
                })
                .map_err(|why| {
                    debug!("Failed to query usage from database: {why}");
                    why
                })
        },
        path,
    )
    .unwrap_or_else(|why| {
        debug!("Database connection failed: {why}");
    });
}
