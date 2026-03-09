//! Database schema model
//!
use chrono::{DateTime, Utc};
use color_eyre::eyre::eyre;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use super::{select, select_optional, DatabaseResult};

/// Row metadata for database table mapping.
pub trait Row {
    /// Returns the table associated with this row type.
    fn table() -> Table;
    /// Inserts this row into its corresponding table.
    fn insert(self, _conn: &Connection) -> DatabaseResult<i64>
    where
        Self: Sized;
}
/// Database tables
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum Table {
    /// CLI activity log
    Activity = 0,
    /// Research activity index entries from buckets
    Catalog = 1,
    /// Cached link check results
    LinkCache = 2,
    /// Validation/check results history
    ValidationHistory = 3,
    /// Downloaded research activity snapshots
    ResearchActivityCache = 4,
}
/// Activity log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRow {
    /// Unique identifier for the activity entry
    pub id: i64,
    /// Command that was executed
    pub command: String,
    /// When the command was executed
    pub executed_at: DateTime<Utc>,
    /// Path provided by user (if any)
    pub user_path: Option<String>,
    /// Whether the command succeeded
    pub success: bool,
}
/// Catalog/index entry from bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogRow {
    /// Unique identifier for the catalog entry
    pub id: i64,
    /// Name of the bucket
    pub bucket_name: String,
    /// URL of the bucket repository
    pub bucket_url: String,
    /// Research activity identifier
    pub identifier: String,
    /// Research activity title
    pub title: String,
    /// When the entry was last updated
    pub updated_at: DateTime<Utc>,
}
/// Link check cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkCacheRow {
    /// Unique identifier for the cache entry
    pub id: i64,
    /// The URL that was checked
    pub url: String,
    /// HTTP status code returned
    pub status_code: i32,
    /// Whether the URL was reachable
    pub is_reachable: bool,
    /// When the URL was checked
    pub checked_at: DateTime<Utc>,
    /// When the cache entry expires
    pub expires_at: DateTime<Utc>,
}
/// Research activity cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchActivityCacheRow {
    /// Unique identifier for the cache entry
    pub id: i64,
    /// Research activity identifier
    pub identifier: String,
    /// Source bucket name
    pub source_bucket: String,
    /// Research activity title
    pub title: String,
    /// When the activity was downloaded
    pub downloaded_at: DateTime<Utc>,
    /// Local file path where the activity is stored
    pub file_path: String,
}
/// Validation/check result history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRow {
    /// Unique identifier for the validation entry
    pub id: i64,
    /// Path to the file that was validated
    pub path: String,
    /// Type of check performed (schema, link, prose, readability)
    pub check_type: String,
    /// Whether the validation passed
    pub success: bool,
    /// Validation message or error details
    pub message: String,
    /// When the validation was performed
    pub checked_at: DateTime<Utc>,
}
impl Table {
    /// Returns all table variants as a slice
    pub fn all() -> &'static [Table] {
        &[
            Table::Activity,
            Table::Catalog,
            Table::LinkCache,
            Table::ValidationHistory,
            Table::ResearchActivityCache,
        ]
    }
    /// Returns the table name as a string
    pub fn name(&self) -> &'static str {
        match self {
            | Table::Activity => "activity",
            | Table::Catalog => "catalog",
            | Table::LinkCache => "link_cache",
            | Table::ValidationHistory => "validation_history",
            | Table::ResearchActivityCache => "research_activity_cache",
        }
    }
    /// Returns CREATE TABLE SQL statement
    pub fn create_sql(&self) -> &'static str {
        match self {
            | Table::Activity => {
                r#"
                CREATE TABLE IF NOT EXISTS activity (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    command TEXT NOT NULL,
                    executed_at TEXT NOT NULL,
                    user_path TEXT,
                    success INTEGER NOT NULL DEFAULT 1
                )
                "#
            }
            | Table::Catalog => {
                r#"
                CREATE TABLE IF NOT EXISTS catalog (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    bucket_name TEXT NOT NULL,
                    bucket_url TEXT NOT NULL,
                    identifier TEXT NOT NULL,
                    title TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    UNIQUE(bucket_name, identifier)
                )
                "#
            }
            | Table::LinkCache => {
                r#"
                CREATE TABLE IF NOT EXISTS link_cache (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    url TEXT NOT NULL UNIQUE,
                    status_code INTEGER NOT NULL,
                    is_reachable INTEGER NOT NULL,
                    checked_at TEXT NOT NULL,
                    expires_at TEXT NOT NULL
                )
                "#
            }
            | Table::ValidationHistory => {
                r#"
                CREATE TABLE IF NOT EXISTS validation_history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    path TEXT NOT NULL,
                    check_type TEXT NOT NULL,
                    success INTEGER NOT NULL,
                    message TEXT NOT NULL,
                    checked_at TEXT NOT NULL
                )
                "#
            }
            | Table::ResearchActivityCache => {
                r#"
                CREATE TABLE IF NOT EXISTS research_activity_cache (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    identifier TEXT NOT NULL UNIQUE,
                    source_bucket TEXT NOT NULL,
                    title TEXT NOT NULL,
                    downloaded_at TEXT NOT NULL,
                    file_path TEXT NOT NULL
                )
                "#
            }
        }
    }
}
impl LinkCacheRow {
    /// Check if the cache entry is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
impl ActivityRow {
    /// Select recent activity entries
    pub fn select_recent(limit: usize) -> DatabaseResult<Vec<ActivityRow>> {
        select(
            "activity",
            "SELECT id, command, executed_at, user_path, success FROM activity ORDER BY executed_at DESC LIMIT ?",
            params![limit as i64],
            |row| {
                Ok(ActivityRow {
                    id: row.get(0)?,
                    command: row.get(1)?,
                    executed_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_default(),
                    user_path: row.get(3)?,
                    success: row.get::<_, i32>(4)? != 0,
                })
            },
        )
    }
}
impl Row for ActivityRow {
    fn table() -> Table {
        Table::Activity
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        conn.execute(
            "INSERT INTO activity (command, executed_at, user_path, success) VALUES (?, ?, ?, ?)",
            params![self.command, self.executed_at.to_rfc3339(), self.user_path, self.success as i32],
        )
        .map_err(|why| eyre!("Failed to insert activity: {}", why))
        .map(|_| conn.last_insert_rowid())
    }
}
impl CatalogRow {
    /// Select catalog entries by bucket name
    pub fn select_by_bucket(bucket_name: &str) -> DatabaseResult<Vec<CatalogRow>> {
        select(
            "catalog",
            "SELECT id, bucket_name, bucket_url, identifier, title, updated_at FROM catalog WHERE bucket_name = ? ORDER BY title",
            params![bucket_name],
            |row| {
                Ok(CatalogRow {
                    id: row.get(0).unwrap_or_default(),
                    bucket_name: row.get(1).unwrap_or_default(),
                    bucket_url: row.get(2).unwrap_or_default(),
                    identifier: row.get(3).unwrap_or_default(),
                    title: row.get(4).unwrap_or_default(),
                    updated_at: row
                        .get::<_, String>(5)
                        .ok()
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_default(),
                })
            },
        )
    }
}
impl Row for CatalogRow {
    fn table() -> Table {
        Table::Catalog
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        conn.execute(
            "INSERT INTO catalog (bucket_name, bucket_url, identifier, title, updated_at) VALUES (?, ?, ?, ?, ?)",
            params![
                self.bucket_name,
                self.bucket_url,
                self.identifier,
                self.title,
                self.updated_at.to_rfc3339()
            ],
        )
        .map_err(|why| eyre!("Failed to insert catalog: {}", why))
        .map(|_| conn.last_insert_rowid())
    }
}
impl LinkCacheRow {
    /// Select a non-expired link cache entry by URL
    pub fn select_by_url(url: &str) -> DatabaseResult<Option<LinkCacheRow>> {
        let now = Utc::now().to_rfc3339();
        select_optional(
            "link cache",
            "SELECT id, url, status_code, is_reachable, checked_at, expires_at FROM link_cache WHERE url = ? AND expires_at > ?",
            params![url, now],
            |row| {
                Ok(LinkCacheRow {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    status_code: row.get(2)?,
                    is_reachable: row.get::<_, i32>(3)? != 0,
                    checked_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_default(),
                    expires_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_default(),
                })
            },
        )
    }
}
impl Row for LinkCacheRow {
    fn table() -> Table {
        Table::LinkCache
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        conn.execute(
            "INSERT INTO link_cache (url, status_code, is_reachable, checked_at, expires_at) VALUES (?, ?, ?, ?, ?)",
            params![
                self.url,
                self.status_code,
                self.is_reachable as i32,
                self.checked_at.to_rfc3339(),
                self.expires_at.to_rfc3339()
            ],
        )
        .map_err(|why| eyre!("Failed to insert link cache: {}", why))
        .map(|_| conn.last_insert_rowid())
    }
}
impl ResearchActivityCacheRow {
    /// Select a research activity cache entry by identifier
    pub fn select_by_identifier(identifier: &str) -> DatabaseResult<Option<ResearchActivityCacheRow>> {
        select_optional(
            "research activity cache",
            "SELECT id, identifier, source_bucket, title, downloaded_at, file_path FROM research_activity_cache WHERE identifier = ?",
            params![identifier],
            |row| {
                Ok(ResearchActivityCacheRow {
                    id: row.get(0)?,
                    identifier: row.get(1)?,
                    source_bucket: row.get(2)?,
                    title: row.get(3)?,
                    downloaded_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_default(),
                    file_path: row.get(5)?,
                })
            },
        )
    }
}
impl Row for ResearchActivityCacheRow {
    fn table() -> Table {
        Table::ResearchActivityCache
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        conn.execute(
            "INSERT INTO research_activity_cache (identifier, source_bucket, title, downloaded_at, file_path) VALUES (?, ?, ?, ?, ?)",
            params![
                self.identifier,
                self.source_bucket,
                self.title,
                self.downloaded_at.to_rfc3339(),
                self.file_path
            ],
        )
        .map_err(|why| eyre!("Failed to insert research activity cache: {}", why))
        .map(|_| conn.last_insert_rowid())
    }
}
impl ValidationRow {
    /// Select recent validation results for a path
    pub fn select_by_path(path: &str, limit: usize) -> DatabaseResult<Vec<ValidationRow>> {
        select(
            "validations",
            "SELECT id, path, check_type, success, message, checked_at FROM validation_history WHERE path = ? ORDER BY checked_at DESC LIMIT ?",
            params![path, limit as i64],
            |row| {
                Ok(ValidationRow {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    check_type: row.get(2)?,
                    success: row.get::<_, i32>(3)? != 0,
                    message: row.get(4)?,
                    checked_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_default(),
                })
            },
        )
    }
}
impl Row for ValidationRow {
    fn table() -> Table {
        Table::ValidationHistory
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        conn.execute(
            "INSERT INTO validation_history (path, check_type, success, message, checked_at) VALUES (?, ?, ?, ?, ?)",
            params![
                self.path,
                self.check_type,
                self.success as i32,
                self.message,
                self.checked_at.to_rfc3339()
            ],
        )
        .map_err(|why| eyre!("Failed to insert validation: {}", why))
        .map(|_| conn.last_insert_rowid())
    }
}
