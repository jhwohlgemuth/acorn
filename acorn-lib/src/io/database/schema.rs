//! Database schema model
//!
use crate::io::database::{
    self,
    macros::{build_query, define_required_fn},
    DatabaseResult,
};
use crate::prelude::PathBuf;
use acorn_macros::DatabaseRow;
use bon::Builder;
use chrono::{DateTime, Utc};
use color_eyre::eyre::eyre;
use core::fmt;
use rusqlite::{params, Connection, Params, Row as SqliteRow};
use serde::{Deserialize, Serialize};

define_required_fn!(1, required1, A, a);
define_required_fn!(3, required3, A, a, B, b, C, c);
define_required_fn!(4, required4, A, a, B, b, C, c, D, d);
/// Row metadata for database table mapping.
pub trait Row {
    /// Returns the table associated with this row type.
    fn table(&self) -> Table;
    /// Inserts this row into its corresponding table.
    fn insert(self, _conn: &Connection) -> DatabaseResult<i64>
    where
        Self: Sized;
    /// Selects rows from the database matching this row's field values.
    fn select(&self) -> DatabaseResult<Vec<Self>>
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
#[derive(Builder, Debug, Clone, Serialize, Deserialize, Default, DatabaseRow)]
#[builder(start_fn = init)]
#[row(table = activity, order_by = "executed_at DESC")]
pub struct ActivityRow {
    /// Unique identifier for the activity entry
    pub id: Option<i64>,
    /// Command that was executed
    pub command: Option<String>,
    /// When the command was executed
    pub executed_at: Option<DateTime<Utc>>,
    /// Path provided by user (if any)
    pub user_path: Option<String>,
    /// Whether the command succeeded
    pub success: Option<bool>,
}
/// Catalog/index entry from bucket
#[derive(Builder, Debug, Clone, Serialize, Deserialize, Default, DatabaseRow)]
#[builder(start_fn = init)]
#[row(table = catalog, order_by = "title")]
pub struct CatalogRow {
    /// Unique identifier for the catalog entry
    pub id: Option<i64>,
    /// Name of the bucket
    pub bucket_name: Option<String>,
    /// URL of the bucket repository
    pub bucket_url: Option<String>,
    /// Research activity identifier
    pub identifier: Option<String>,
    /// Research activity title
    pub title: Option<String>,
    /// When the entry was last updated
    pub updated_at: Option<DateTime<Utc>>,
}
/// Link check cache entry
#[derive(Builder, Debug, Clone, Serialize, Deserialize, Default, DatabaseRow)]
#[builder(start_fn = init)]
#[row(table = link_cache)]
pub struct LinkCacheRow {
    /// Unique identifier for the cache entry
    pub id: Option<i64>,
    /// The URL that was checked
    pub url: Option<String>,
    /// HTTP status code returned
    pub status_code: Option<i32>,
    /// Whether the URL was reachable
    pub is_reachable: Option<bool>,
    /// When the URL was checked
    pub checked_at: Option<DateTime<Utc>>,
    /// When the cache entry expires
    pub expires_at: Option<DateTime<Utc>>,
}
/// Research activity cache entry
#[derive(Builder, Debug, Clone, Serialize, Deserialize, Default, DatabaseRow)]
#[builder(start_fn = init)]
#[row(table = research_activity_cache)]
pub struct ResearchActivityCacheRow {
    /// Unique identifier for the cache entry
    pub id: Option<i64>,
    /// Research activity identifier
    pub identifier: Option<String>,
    /// Source bucket name
    pub source_bucket: Option<String>,
    /// Research activity title
    pub title: Option<String>,
    /// When the activity was downloaded
    pub downloaded_at: Option<DateTime<Utc>>,
    /// Local file path where the activity is stored
    pub file_path: Option<String>,
}
/// Validation/check result history
#[derive(Builder, Debug, Clone, Serialize, Deserialize, Default, DatabaseRow)]
#[builder(start_fn = init)]
#[row(table = validation_history, order_by = "checked_at DESC")]
pub struct ValidationRow {
    /// Unique identifier for the validation entry
    pub id: Option<i64>,
    /// Path to the file that was validated
    pub path: Option<String>,
    /// Type of check performed (schema, link, prose, readability)
    pub check_type: Option<String>,
    /// Whether the validation passed
    pub success: Option<bool>,
    /// Validation message or error details
    pub message: Option<String>,
    /// When the validation was performed
    pub checked_at: Option<DateTime<Utc>>,
}
impl From<&SqliteRow<'_>> for ActivityRow {
    fn from(row: &SqliteRow<'_>) -> Self {
        ActivityRow {
            id: row.get(0).ok(),
            command: row.get(1).ok(),
            executed_at: parse_rfc3339_column(row, 2),
            user_path: row.get(3).ok(),
            success: row.get::<_, i32>(4).ok().map(|value| value != 0),
        }
    }
}
impl From<SqliteRow<'_>> for ActivityRow {
    fn from(row: SqliteRow<'_>) -> Self {
        ActivityRow::from(&row)
    }
}
impl From<&SqliteRow<'_>> for CatalogRow {
    fn from(row: &SqliteRow<'_>) -> Self {
        CatalogRow {
            id: row.get(0).ok(),
            bucket_name: row.get(1).ok(),
            bucket_url: row.get(2).ok(),
            identifier: row.get(3).ok(),
            title: row.get(4).ok(),
            updated_at: parse_rfc3339_column(row, 5),
        }
    }
}
impl From<SqliteRow<'_>> for CatalogRow {
    fn from(row: SqliteRow<'_>) -> Self {
        CatalogRow::from(&row)
    }
}
impl From<&SqliteRow<'_>> for LinkCacheRow {
    fn from(row: &SqliteRow<'_>) -> Self {
        LinkCacheRow {
            id: row.get(0).ok(),
            url: row.get(1).ok(),
            status_code: row.get(2).ok(),
            is_reachable: row.get::<_, i32>(3).ok().map(|value| value != 0),
            checked_at: parse_rfc3339_column(row, 4),
            expires_at: parse_rfc3339_column(row, 5),
        }
    }
}
impl From<SqliteRow<'_>> for LinkCacheRow {
    fn from(row: SqliteRow<'_>) -> Self {
        LinkCacheRow::from(&row)
    }
}
impl From<&SqliteRow<'_>> for ResearchActivityCacheRow {
    fn from(row: &SqliteRow<'_>) -> Self {
        ResearchActivityCacheRow {
            id: row.get(0).ok(),
            identifier: row.get(1).ok(),
            source_bucket: row.get(2).ok(),
            title: row.get(3).ok(),
            downloaded_at: parse_rfc3339_column(row, 4),
            file_path: row.get(5).ok(),
        }
    }
}
impl From<SqliteRow<'_>> for ResearchActivityCacheRow {
    fn from(row: SqliteRow<'_>) -> Self {
        ResearchActivityCacheRow::from(&row)
    }
}
impl From<&SqliteRow<'_>> for ValidationRow {
    fn from(row: &SqliteRow<'_>) -> Self {
        ValidationRow {
            id: row.get(0).ok(),
            path: row.get(1).ok(),
            check_type: row.get(2).ok(),
            success: row.get::<_, i32>(3).ok().map(|value| value != 0),
            message: row.get(4).ok(),
            checked_at: parse_rfc3339_column(row, 5),
        }
    }
}
impl From<SqliteRow<'_>> for ValidationRow {
    fn from(row: SqliteRow<'_>) -> Self {
        ValidationRow::from(&row)
    }
}
impl fmt::Display for ActivityRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ActivityRow(id: {}, command: {}, success: {})",
            display_or_none(self.id.as_ref()),
            display_or_none(self.command.as_ref()),
            self.success.unwrap_or(false)
        )
        .and_then(|_| write_optional_field(f, "path", self.user_path.as_ref()))
        .and_then(|_| write_optional_timestamp(f, "executed_at", self.executed_at.as_ref()))
    }
}
impl fmt::Display for CatalogRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CatalogRow(id: {}, bucket: {}, identifier: {}, title: {})",
            display_or_none(self.id.as_ref()),
            display_or_none(self.bucket_name.as_ref()),
            display_or_none(self.identifier.as_ref()),
            display_or_none(self.title.as_ref())
        )
        .and_then(|_| write_optional_timestamp(f, "updated_at", self.updated_at.as_ref()))
    }
}
impl fmt::Display for LinkCacheRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LinkCacheRow(id: {}, url: {}, status: {}, reachable: {})",
            display_or_none(self.id.as_ref()),
            display_or_none(self.url.as_ref()),
            self.status_code.unwrap_or(0),
            self.is_reachable.unwrap_or(false)
        )
        .and_then(|_| write_optional_timestamp(f, "checked_at", self.checked_at.as_ref()))
        .and_then(|_| write_optional_timestamp(f, "expires_at", self.expires_at.as_ref()))
    }
}
impl fmt::Display for ResearchActivityCacheRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ResearchActivityCacheRow(id: {}, identifier: {}, bucket: {}, title: {})",
            display_or_none(self.id.as_ref()),
            display_or_none(self.identifier.as_ref()),
            display_or_none(self.source_bucket.as_ref()),
            display_or_none(self.title.as_ref())
        )
        .and_then(|_| write_optional_field(f, "file_path", self.file_path.as_ref()))
        .and_then(|_| write_optional_timestamp(f, "downloaded_at", self.downloaded_at.as_ref()))
    }
}
impl fmt::Display for ValidationRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ValidationRow(id: {}, path: {}, type: {}, success: {}, message: {})",
            display_or_none(self.id.as_ref()),
            display_or_none(self.path.as_ref()),
            display_or_none(self.check_type.as_ref()),
            self.success.unwrap_or(false),
            display_or_none(self.message.as_ref())
        )
        .and_then(|_| write_optional_timestamp(f, "checked_at", self.checked_at.as_ref()))
    }
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
    /// Executes a query and returns mapped rows for this table.
    pub fn rows<R, P>(&self, query: &str, params: P, path: Option<&PathBuf>) -> DatabaseResult<Vec<R>>
    where
        P: Params,
        for<'row> R: Row + From<&'row SqliteRow<'row>>,
    {
        database::Database::<Table>::from_path(path.cloned()).with_connection(|conn| {
            conn.prepare(query)
                .map_err(|why| eyre!("Failed to prepare {} query: {why}", self.name()))
                .and_then(|mut stmt| {
                    stmt.query_map(params, |row| Ok(R::from(row)))
                        .map_err(|why| eyre!("Failed to query {}: {why}", self.name()))
                        .and_then(|rows| {
                            rows.collect::<Result<Vec<_>, _>>()
                                .map_err(|why| eyre!("Failed to read {} rows: {why}", self.name()))
                        })
                })
        })
    }
    /// Prints mapped rows for this table.
    pub fn print<R>(&self, rows: Vec<R>)
    where
        R: fmt::Display,
    {
        rows.iter().enumerate().for_each(|(idx, row)| {
            println!("  {}. {}", idx + 1, row);
        });
    }
}
impl LinkCacheRow {
    /// Check if the cache entry is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|e| Utc::now() > e).unwrap_or(true)
    }
}
impl Row for ActivityRow {
    fn table(&self) -> Table {
        Table::Activity
    }
    fn select(&self) -> DatabaseResult<Vec<ActivityRow>> {
        let query = format!("SELECT id, command, executed_at, user_path, success FROM {}", Self::TABLE_NAME);
        let (query, params) = self.build_select_query(&query);
        self.table().rows(&query, params, None)
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        let ActivityRow {
            command,
            executed_at,
            user_path,
            success,
            ..
        } = self;
        let executed_at = executed_at.unwrap_or_else(Utc::now).to_rfc3339();
        let success = success.unwrap_or(true) as i32;
        let required = required1(command);
        match required {
            | Ok((command,)) => {
                let query = format!(
                    "INSERT INTO {} (command, executed_at, user_path, success) VALUES (?, ?, ?, ?)",
                    Self::TABLE_NAME
                );
                conn.execute(&query, params![command, executed_at, user_path, success,])
                    .map_err(|why| eyre!("Failed to insert activity: {}", why))
                    .map(|_| conn.last_insert_rowid())
            }
            | Err(why) => Err(why),
        }
    }
}
impl Row for CatalogRow {
    fn table(&self) -> Table {
        Table::Catalog
    }
    fn select(&self) -> DatabaseResult<Vec<CatalogRow>> {
        let query = format!(
            "SELECT id, bucket_name, bucket_url, identifier, title, updated_at FROM {}",
            Self::TABLE_NAME
        );
        let (query, params) = self.build_select_query(&query);
        self.table().rows(&query, params, None)
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        let CatalogRow {
            bucket_name,
            bucket_url,
            identifier,
            title,
            updated_at,
            ..
        } = self;
        let updated_at = updated_at.unwrap_or_else(Utc::now).to_rfc3339();
        let required = required4(bucket_name, bucket_url, identifier, title);
        match required {
            | Ok((bucket_name, bucket_url, identifier, title)) => {
                let query = format!(
                    "INSERT INTO {} (bucket_name, bucket_url, identifier, title, updated_at) VALUES (?, ?, ?, ?, ?)",
                    Self::TABLE_NAME
                );
                conn.execute(&query, params![bucket_name, bucket_url, identifier, title, updated_at])
                    .map_err(|why| eyre!("Failed to insert catalog: {}", why))
                    .map(|_| conn.last_insert_rowid())
            }
            | Err(why) => Err(why),
        }
    }
}
impl Row for LinkCacheRow {
    fn table(&self) -> Table {
        Table::LinkCache
    }
    fn select(&self) -> DatabaseResult<Vec<LinkCacheRow>> {
        let query = format!(
            "SELECT id, url, status_code, is_reachable, checked_at, expires_at FROM {}",
            Self::TABLE_NAME
        );
        let (query, params) = self.build_select_query(&query);
        self.table().rows(&query, params, None)
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        let LinkCacheRow {
            url,
            status_code,
            is_reachable,
            checked_at,
            expires_at,
            ..
        } = self;
        let checked_at = checked_at.unwrap_or_else(Utc::now).to_rfc3339();
        let required = required4(url, status_code, is_reachable, expires_at);
        match required {
            | Ok((url, status_code, is_reachable, expires_at)) => {
                let query = format!(
                    "INSERT INTO {} (url, status_code, is_reachable, checked_at, expires_at) VALUES (?, ?, ?, ?, ?)",
                    Self::TABLE_NAME
                );
                conn.execute(
                    &query,
                    params![url, status_code, is_reachable as i32, checked_at, expires_at.to_rfc3339()],
                )
                .map_err(|why| eyre!("Failed to insert link cache: {}", why))
                .map(|_| conn.last_insert_rowid())
            }
            | Err(why) => Err(why),
        }
    }
}
impl Row for ResearchActivityCacheRow {
    fn table(&self) -> Table {
        Table::ResearchActivityCache
    }
    fn select(&self) -> DatabaseResult<Vec<ResearchActivityCacheRow>> {
        let query = format!(
            "SELECT id, identifier, source_bucket, title, downloaded_at, file_path FROM {}",
            Self::TABLE_NAME
        );
        let (query, params) = self.build_select_query(&query);
        self.table().rows(&query, params, None)
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        let ResearchActivityCacheRow {
            identifier,
            source_bucket,
            title,
            downloaded_at,
            file_path,
            ..
        } = self;
        let downloaded_at = downloaded_at.unwrap_or_else(Utc::now).to_rfc3339();
        let required = required4(identifier, source_bucket, title, file_path);
        match required {
            | Ok((identifier, source_bucket, title, file_path)) => {
                let query = format!(
                    "INSERT INTO {} (identifier, source_bucket, title, downloaded_at, file_path) VALUES (?, ?, ?, ?, ?)",
                    Self::TABLE_NAME
                );
                conn.execute(&query, params![identifier, source_bucket, title, downloaded_at, file_path])
                    .map_err(|why| eyre!("Failed to insert research activity cache: {}", why))
                    .map(|_| conn.last_insert_rowid())
            }
            | Err(why) => Err(why),
        }
    }
}
impl Row for ValidationRow {
    fn table(&self) -> Table {
        Table::ValidationHistory
    }
    fn select(&self) -> DatabaseResult<Vec<ValidationRow>> {
        let query = format!("SELECT id, path, check_type, success, message, checked_at FROM {}", Self::TABLE_NAME);
        let (query, params) = self.build_select_query(&query);
        self.table().rows(&query, params, None)
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        let ValidationRow {
            path,
            check_type,
            success,
            message,
            checked_at,
            ..
        } = self;
        let success = success.unwrap_or(true) as i32;
        let checked_at = checked_at.unwrap_or_else(Utc::now).to_rfc3339();
        let required = required3(path, check_type, message);
        match required {
            | Ok((path, check_type, message)) => {
                let query = format!(
                    "INSERT INTO {} (path, check_type, success, message, checked_at) VALUES (?, ?, ?, ?, ?)",
                    Self::TABLE_NAME
                );
                conn.execute(&query, params![path, check_type, success, message, checked_at])
                    .map_err(|why| eyre!("Failed to insert validation: {}", why))
                    .map(|_| conn.last_insert_rowid())
            }
            | Err(why) => Err(why),
        }
    }
}
fn display_or_none<T>(value: Option<&T>) -> String
where
    T: fmt::Display,
{
    value.map(ToString::to_string).unwrap_or_else(|| "None".to_string())
}
fn parse_rfc3339_column(row: &SqliteRow<'_>, index: usize) -> Option<DateTime<Utc>> {
    row.get::<_, String>(index)
        .ok()
        .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
        .map(|value| value.with_timezone(&Utc))
}
fn write_optional_field<T>(f: &mut fmt::Formatter<'_>, label: &str, value: Option<&T>) -> fmt::Result
where
    T: fmt::Display,
{
    match value {
        | Some(value) => write!(f, ", {label}: {value}"),
        | None => Ok(()),
    }
}
fn write_optional_timestamp(f: &mut fmt::Formatter<'_>, label: &str, value: Option<&DateTime<Utc>>) -> fmt::Result {
    match value {
        | Some(value) => write!(f, ", {label}: {}", value.format("%Y-%m-%d %H:%M:%S")),
        | None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn activity_row_display_includes_optional_values() {
        let row = ActivityRow {
            id: Some(1),
            command: Some("download".to_string()),
            executed_at: Utc.with_ymd_and_hms(2026, 3, 10, 16, 54, 34).single(),
            user_path: Some("content".to_string()),
            success: Some(true),
        };
        assert_eq!(
            row.to_string(),
            "ActivityRow(id: 1, command: download, success: true), path: content, executed_at: 2026-03-10 16:54:34"
        );
    }

    #[test]
    fn validation_row_display_uses_none_for_missing_values() {
        let row = ValidationRow::default();
        assert_eq!(
            row.to_string(),
            "ValidationRow(id: None, path: None, type: None, success: false, message: None)"
        );
    }
}
