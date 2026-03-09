//! SQLite database utilities for ACORN.
//!
//! This module provides a lazily initialized, process-global SQLite connection
//! guarded by a mutex and a helper to execute callbacks against that connection.
use crate::prelude::{create_dir_all, env, Mutex, OnceLock, Path, PathBuf};
use crate::util::constants::{APPLICATION, DATABASE_NAME, DEFAULT_CACHE_TTL_SECONDS, ENV_CACHE_TTL, ORGANIZATION, QUALIFIER};
use crate::util::Label;
use chrono::{Duration, Utc};
use color_eyre::eyre::{self, eyre};
use core::marker::PhantomData;
use directories::ProjectDirs;
use owo_colors::OwoColorize;
use rusqlite::{params, Connection, Params, Row as SqliteRow};
use tracing::{debug, info};

pub mod schema;

use schema::Table;

static CONNECTION: OnceLock<Mutex<Connection>> = OnceLock::new();

/// Utility type alias for database operation results
pub type DatabaseResult<T> = eyre::Result<T, eyre::Report>;
/// Persist and access data in a database
pub trait DatabaseOperations {
    /// Select a value from the database
    fn select<T, P>(&self, value: P) -> DatabaseResult<Option<T>>
    where
        T: Into<Table>,
        P: Params;
}
/// Insert a row into the database
// pub(crate) fn insert_row<R>(row: impl Into<R>) -> DatabaseResult<i64>
// where
//     R: Row,
// {
//     let row = row.into();
//     Database::<Table>::default().with_connection(|conn| row.insert(conn))
// }
pub(crate) fn select<R, P>(context: &str, query: &str, params: P, mapper: impl FnMut(&SqliteRow<'_>) -> rusqlite::Result<R>) -> DatabaseResult<Vec<R>>
where
    P: Params,
{
    Database::<Table>::default().with_connection(|conn| {
        conn.prepare(query)
            .map_err(|why| eyre!("Failed to prepare {} query: {}", context, why))
            .and_then(|mut stmt| {
                stmt.query_map(params, mapper)
                    .map_err(|why| eyre!("Failed to query {}: {}", context, why))
                    .and_then(|rows| {
                        rows.collect::<Result<Vec<_>, _>>()
                            .map_err(|why| eyre!("Failed to collect {}: {}", context, why))
                    })
            })
    })
}

pub(crate) fn select_optional<R, P>(
    context: &str,
    query: &str,
    params: P,
    mapper: impl FnOnce(&SqliteRow<'_>) -> rusqlite::Result<R>,
) -> DatabaseResult<Option<R>>
where
    P: Params,
{
    Database::<Table>::default().with_connection(|conn| {
        conn.prepare(query)
            .map_err(|why| eyre!("Failed to prepare {} query: {}", context, why))
            .and_then(|mut stmt| {
                let result = stmt.query_row(params, mapper);
                match result {
                    | Ok(row) => Ok(Some(row)),
                    | Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    | Err(why) => Err(eyre!("Failed to query {}: {}", context, why)),
                }
            })
    })
}
/// Table schema provider for database operations.
pub trait TableSchemaProvider: Copy + Into<Table> + 'static {
    /// Returns all variants for the table type.
    fn all() -> &'static [Self];
}
/// Database handle for a given table schema provider type.
#[derive(Debug, Clone)]
pub struct Database<T>
where
    T: TableSchemaProvider,
{
    path: Option<PathBuf>,
    marker: PhantomData<T>,
}
impl<T> Default for Database<T>
where
    T: TableSchemaProvider,
{
    fn default() -> Self {
        Self {
            path: None,
            marker: PhantomData,
        }
    }
}
impl<T> Database<T>
where
    T: TableSchemaProvider,
{
    /// Create a new database handle from an optional database path override.
    pub fn from_path(path: Option<PathBuf>) -> Self {
        Self { path, marker: PhantomData }
    }

    /// Executes a callback using the database path configured on this handle.
    pub fn with_connection<U>(&self, callback: impl FnOnce(&Connection) -> DatabaseResult<U>) -> DatabaseResult<U> {
        Self::sqlite_connection(self.path.as_ref())
            .and_then(|connection| connection.lock().map_err(|why| eyre!("Failed to acquire SQLite connection lock: {why}")))
            .and_then(|guard| callback(&guard))
    }

    fn sqlite_connection(path: Option<&PathBuf>) -> DatabaseResult<&'static Mutex<Connection>> {
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
    /// Get the cache TTL value from environment or use default.
    pub fn cache_ttl(&self) -> Duration {
        #[cfg(feature = "std")]
        {
            if let Ok(env_ttl) = dotenvy::var(ENV_CACHE_TTL) {
                if let Ok(seconds) = env_ttl.parse::<i64>() {
                    if seconds > 0 {
                        return Duration::seconds(seconds);
                    }
                }
            }
        }
        let default_ttl = Duration::seconds(DEFAULT_CACHE_TTL_SECONDS as i64);
        if let Ok(env_ttl) = env::var(ENV_CACHE_TTL) {
            if let Ok(seconds) = env_ttl.parse::<i64>() {
                if seconds > 0 {
                    return Duration::seconds(seconds);
                }
            }
        }
        default_ttl
    }
    /// Create one table if it does not exist.
    pub fn migrate_table(&self, table: impl Into<Table>) -> eyre::Result<(), eyre::Report> {
        let table = table.into();
        self.with_connection(|conn| {
            conn.execute(table.create_sql(), params![])
                .map_err(|why| eyre!("Failed to create table {}: {}", table.name(), why))
                .map(|_| ())
        })
    }
    /// Create all tables defined by the variant provider type.
    pub fn migrate(&self) -> DatabaseResult<()> {
        T::all().iter().copied().try_fold((), |_, table| self.migrate_table(table)).inspect(|_| {
            info!("{} Database migration complete", Label::pass());
        })
    }
    /// Delete all rows from a table.
    pub fn clear(&self, table: impl Into<Table>) -> DatabaseResult<usize> {
        let table = table.into();
        self.with_connection(|conn| {
            conn.execute(&format!("DELETE FROM {}", table.name()), params![])
                .map_err(|why| eyre!("Failed to clear {}: {}", table.name(), why))
        })
    }
    /// Delete rows where a timestamp/text column is older than a cutoff value.
    pub fn delete_expired_before(&self, table: impl Into<Table>, column: &str, cutoff: &str) -> DatabaseResult<usize> {
        let table = table.into();
        let sql = format!("DELETE FROM {} WHERE {} < ?", table.name(), column);
        self.with_connection(|conn| {
            conn.execute(&sql, params![cutoff])
                .map_err(|why| eyre!("Failed to clean up {}: {}", table.name(), why))
        })
    }
    /// Keep only the most recent `limit` rows using an id/order column pair.
    pub fn trim_to_recent(&self, table: impl Into<Table>, id_column: &str, order_column: &str, limit: usize) -> DatabaseResult<usize> {
        let table = table.into();
        let sql = format!(
            "DELETE FROM {table} WHERE {id_col} NOT IN (SELECT {id_col} FROM {table} ORDER BY {order_col} DESC LIMIT ?)",
            table = table.name(),
            id_col = id_column,
            order_col = order_column,
        );
        self.with_connection(|conn| {
            conn.execute(&sql, params![limit as i64])
                .map_err(|why| eyre!("Failed to trim {}: {}", table.name(), why))
        })
    }
    /// Clean up expired link cache entries and trim validation history.
    pub fn cleanup_expired_cache(&self) -> DatabaseResult<usize> {
        let ttl = self.cache_ttl();
        let now = Utc::now();
        let cutoff = (now - ttl).to_rfc3339();
        self.delete_expired_before(Table::LinkCache, "expires_at", &cutoff)
            .and_then(|link_count| {
                self.trim_to_recent(Table::ValidationHistory, "id", "checked_at", 1000).map(|_| {
                    if link_count > 0 {
                        info!("{} Cleaned up {} expired cache entries", Label::run(), link_count);
                    }
                    link_count
                })
            })
    }
    /// Clear all cache-related tables.
    pub fn clear_cache(&self) -> DatabaseResult<usize> {
        self.clear(Table::LinkCache).and_then(|link_count| {
            self.clear(Table::ValidationHistory).and_then(|validation_count| {
                self.clear(Table::ResearchActivityCache).map(|cache_count| {
                    let total = link_count + validation_count + cache_count;
                    info!("{} Cleared {} entries from cache tables", Label::pass(), total);
                    total
                })
            })
        })
    }
    /// Clear (reset) all data from all tables.
    pub fn reset(&self) -> DatabaseResult<usize> {
        T::all()
            .iter()
            .copied()
            .try_fold(0usize, |total, table| self.clear(table).map(|count| total + count))
            .inspect(|&total| {
                info!("{} Cleared {} entries from all tables", Label::pass(), total);
            })
    }
}
impl TableSchemaProvider for Table {
    fn all() -> &'static [Self] {
        Table::all()
    }
}
/// Queries and prints recent usage history from the database,
/// optionally overriding the default database path.
pub fn print_usage(path: Option<&PathBuf>) {
    Database::<Table>::from_path(path.cloned())
        .with_connection(|conn| {
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
        })
        .unwrap_or_else(|why| {
            debug!("Database connection failed: {why}");
        });
}
/// Saves usage information (date and path) to the database,
/// optionally overriding the default database path.
pub fn save_usage(path: &Path, database_path_override: Option<&PathBuf>) {
    Database::<Table>::from_path(database_path_override.cloned())
        .with_connection(|conn| {
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
        })
        .unwrap_or_else(|why| {
            debug!("Database connection failed: {why}");
        });
}
/// Insert or update a catalog entry
pub fn upsert_catalog(bucket_name: &str, bucket_url: &str, identifier: &str, title: &str) -> DatabaseResult<()> {
    let now = Utc::now().to_rfc3339();
    Database::<Table>::default().with_connection(|conn| {
        conn.execute(
            "INSERT INTO catalog (bucket_name, bucket_url, identifier, title, updated_at)
                 VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT(bucket_name, identifier) DO UPDATE SET
                    bucket_url = excluded.bucket_url,
                    title = excluded.title,
                    updated_at = excluded.updated_at",
            params![bucket_name, bucket_url, identifier, title, now],
        )
        .map_err(|why| eyre!("Failed to upsert catalog: {}", why))
        .map(|_| ())
    })
}
/// Insert or update a link cache entry
pub fn upsert_link_cache(url: &str, status_code: i32, is_reachable: bool) -> DatabaseResult<()> {
    let ttl = Database::<Table>::default().cache_ttl();
    let now = Utc::now();
    let expires = (now + ttl).to_rfc3339();
    Database::<Table>::default().with_connection(|conn| {
        conn.execute(
            "INSERT INTO link_cache (url, status_code, is_reachable, checked_at, expires_at)
                 VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT(url) DO UPDATE SET
                    status_code = excluded.status_code,
                    is_reachable = excluded.is_reachable,
                    checked_at = excluded.checked_at,
                    expires_at = excluded.expires_at",
            params![url, status_code, is_reachable as i32, now.to_rfc3339(), expires],
        )
        .map_err(|why| eyre!("Failed to upsert link cache: {}", why))
        .map(|_| ())
    })
}
/// Insert or update a research activity cache entry
pub fn upsert_research_activity_cache(identifier: &str, source_bucket: &str, title: &str, file_path: &str) -> DatabaseResult<()> {
    let now = Utc::now().to_rfc3339();
    Database::<Table>::default().with_connection(|conn| {
        conn.execute(
            "INSERT INTO research_activity_cache (identifier, source_bucket, title, downloaded_at, file_path)
                 VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT(identifier) DO UPDATE SET
                    source_bucket = excluded.source_bucket,
                    title = excluded.title,
                    downloaded_at = excluded.downloaded_at,
                    file_path = excluded.file_path",
            params![identifier, source_bucket, title, now, file_path],
        )
        .map_err(|why| eyre!("Failed to upsert research activity cache: {}", why))
        .map(|_| ())
    })
}
fn database_path(path: Option<&PathBuf>) -> DatabaseResult<PathBuf> {
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
