//! Database schema model
//!
use super::{Database, DatabaseResult, Row, SelectQuery, TableSchemaProvider};
use crate::io::database::macros::{build_query, define_required_fn};
use crate::prelude::PathBuf;
use crate::util::{print_values_as_table, Label};
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
/// Database tables
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
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
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
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
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
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
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
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
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
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
impl From<ActivityRow> for Vec<String> {
    fn from(row: ActivityRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.command.unwrap_or_default(),
            row.executed_at.map_or_else(String::new, |v| v.to_rfc3339()),
            row.user_path.unwrap_or_default(),
            row.success.map_or_else(String::new, |v| v.to_string()),
        ]
    }
}
impl From<CatalogRow> for Vec<String> {
    fn from(row: CatalogRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.bucket_name.unwrap_or_default(),
            row.bucket_url.unwrap_or_default(),
            row.identifier.unwrap_or_default(),
            row.title.unwrap_or_default(),
            row.updated_at.map_or_else(String::new, |v| v.to_rfc3339()),
        ]
    }
}
impl From<LinkCacheRow> for Vec<String> {
    fn from(row: LinkCacheRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.url.unwrap_or_default(),
            row.status_code.map_or_else(String::new, |v| v.to_string()),
            row.is_reachable.map_or_else(String::new, |v| v.to_string()),
            row.checked_at.map_or_else(String::new, |v| v.to_rfc3339()),
            row.expires_at.map_or_else(String::new, |v| v.to_rfc3339()),
        ]
    }
}
impl From<ResearchActivityCacheRow> for Vec<String> {
    fn from(row: ResearchActivityCacheRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.identifier.unwrap_or_default(),
            row.source_bucket.unwrap_or_default(),
            row.title.unwrap_or_default(),
            row.downloaded_at.map_or_else(String::new, |v| v.to_rfc3339()),
            row.file_path.unwrap_or_default(),
        ]
    }
}
impl From<ValidationRow> for Vec<String> {
    fn from(row: ValidationRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.path.unwrap_or_default(),
            row.check_type.unwrap_or_default(),
            row.success.map_or_else(String::new, |v| v.to_string()),
            row.message.unwrap_or_default(),
            row.checked_at.map_or_else(String::new, |v| v.to_rfc3339()),
        ]
    }
}
impl fmt::Display for ActivityRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ActivityRow = id: {}, command: {}, success: {}",
            display_or_none(self.id.as_ref()),
            display_or_none(self.command.as_ref()),
            self.success.unwrap_or(false),
        )
        .and_then(|_| write_optional_field(f, "path", self.user_path.as_ref()))
        .and_then(|_| write_optional_timestamp(f, "executed_at", self.executed_at.as_ref()))
    }
}
impl fmt::Display for CatalogRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CatalogRow = id: {}, bucket: {}, identifier: {}, title: {}",
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
            "LinkCacheRow = id: {}, url: {}, status: {}, reachable: {}",
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
            "ResearchActivityCacheRow = id: {}, identifier: {}, bucket: {}, title: {}",
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
            "ValidationRow = id: {}, path: {}, type: {}, success: {}, message: {}",
            display_or_none(self.id.as_ref()),
            display_or_none(self.path.as_ref()),
            display_or_none(self.check_type.as_ref()),
            self.success.unwrap_or(false),
            display_or_none(self.message.as_ref())
        )
        .and_then(|_| write_optional_timestamp(f, "checked_at", self.checked_at.as_ref()))
    }
}
impl TableSchemaProvider for Table {
    fn all() -> &'static [Self] {
        &[
            Table::Activity,
            Table::Catalog,
            Table::LinkCache,
            Table::ValidationHistory,
            Table::ResearchActivityCache,
        ]
    }
    /// Returns CREATE TABLE SQL statement
    fn create_statement(&self) -> &'static str {
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
    /// Returns the table name as a string
    fn name(&self) -> &'static str {
        match self {
            | Table::Activity => "activity",
            | Table::Catalog => "catalog",
            | Table::LinkCache => "link_cache",
            | Table::ValidationHistory => "validation_history",
            | Table::ResearchActivityCache => "research_activity_cache",
        }
    }
    /// Print all table rows
    fn print(self, path: Option<PathBuf>) {
        fn print_rows<R>(table: Table, path: Option<&PathBuf>)
        where
            R: Into<Vec<String>> + Row + Default + fmt::Display,
            for<'row> R: From<&'row SqliteRow<'row>>,
        {
            let row = R::default();
            let fields = <R as Row>::fields();
            let query = format!("SELECT {} FROM {}", fields.join(", "), table.name());
            let (query, params) = row.build_select_query(&query);
            let rows = table
                .rows::<R, _>(&query, params, path)
                .map(|rows| rows.into_iter().map(Into::into).collect::<Vec<Vec<String>>>())
                .unwrap_or_default();
            print_values_as_table::<String>(fields.to_vec(), rows, Some(table.name().to_string()));
        }
        match self {
            | Table::Activity => print_rows::<ActivityRow>(self, path.as_ref()),
            | Table::Catalog => print_rows::<CatalogRow>(self, path.as_ref()),
            | Table::LinkCache => print_rows::<LinkCacheRow>(self, path.as_ref()),
            | Table::ValidationHistory => print_rows::<ValidationRow>(self, path.as_ref()),
            | Table::ResearchActivityCache => print_rows::<ResearchActivityCacheRow>(self, path.as_ref()),
        }
    }
    /// Get table rows
    fn rows<R, P>(&self, query: &str, params: P, path: Option<&PathBuf>) -> DatabaseResult<Vec<R>>
    where
        P: Params,
        for<'row> R: Row + From<&'row SqliteRow<'row>>,
    {
        Database::<Table>::from_path(path.cloned()).with_connection(|conn| {
            conn.prepare(query)
                .map_err(|why| eyre!("=> {} Failed to prepare {} query: {why}", Label::fail(), self.name()))
                .and_then(|mut stmt| {
                    stmt.query_map(params, |row| Ok(R::from(row)))
                        .map_err(|why| eyre!("=> {} Failed to query {}: {why}", Label::fail(), self.name()))
                        .and_then(|rows| {
                            rows.collect::<Result<Vec<_>, _>>()
                                .map_err(|why| eyre!("=> {} Failed to read {} rows: {why}", Label::fail(), self.name()))
                        })
                })
        })
    }
}
impl Row for ActivityRow {
    fn table(&self) -> Table {
        Table::Activity
    }
    fn select(&self) -> DatabaseResult<Vec<ActivityRow>> {
        let table = self.table();
        let fields = Self::FIELDS.join(", ");
        let query = format!("SELECT {fields} FROM {}", table.name());
        let (query, params) = self.build_select_query(&query);
        table.rows(&query, params, None)
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        let ActivityRow {
            command,
            executed_at,
            user_path,
            success,
            ..
        } = self.clone();
        let executed_at = executed_at.unwrap_or_else(Utc::now).to_rfc3339();
        let success = success.unwrap_or(true) as i32;
        let required = required1(command);
        match required {
            | Ok((command,)) => {
                let query = format!(
                    "INSERT INTO {} (command, executed_at, user_path, success) VALUES (?, ?, ?, ?)",
                    self.table().name()
                );
                conn.execute(&query, params![command, executed_at, user_path, success,])
                    .map_err(|why| eyre!("=> {} Failed to insert activity: {why}", Label::fail()))
                    .map(|_| conn.last_insert_rowid())
            }
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        ActivityRow::build_select_query(self, base)
    }
}
impl Row for CatalogRow {
    fn table(&self) -> Table {
        Table::Catalog
    }
    fn select(&self) -> DatabaseResult<Vec<CatalogRow>> {
        let table = self.table();
        let fields = Self::FIELDS.join(", ");
        let query = format!("SELECT {fields} FROM {}", table.name());
        let (query, params) = self.build_select_query(&query);
        table.rows(&query, params, None)
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        let CatalogRow {
            bucket_name,
            bucket_url,
            identifier,
            title,
            updated_at,
            ..
        } = self.clone();
        let updated_at = updated_at.unwrap_or_else(Utc::now).to_rfc3339();
        let required = required4(bucket_name, bucket_url, identifier, title);
        match required {
            | Ok((bucket_name, bucket_url, identifier, title)) => {
                let query = format!(
                    "INSERT INTO {} (bucket_name, bucket_url, identifier, title, updated_at) VALUES (?, ?, ?, ?, ?)",
                    self.table().name()
                );
                conn.execute(&query, params![bucket_name, bucket_url, identifier, title, updated_at])
                    .map_err(|why| eyre!("=> {} Failed to insert catalog: {why}", Label::fail()))
                    .map(|_| conn.last_insert_rowid())
            }
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        CatalogRow::build_select_query(self, base)
    }
}
impl Row for LinkCacheRow {
    fn table(&self) -> Table {
        Table::LinkCache
    }
    fn select(&self) -> DatabaseResult<Vec<LinkCacheRow>> {
        let table = self.table();
        let fields = Self::FIELDS.join(", ");
        let query = format!("SELECT {fields} FROM {}", table.name());
        let (query, params) = self.build_select_query(&query);
        table.rows(&query, params, None)
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        let LinkCacheRow {
            url,
            status_code,
            is_reachable,
            checked_at,
            expires_at,
            ..
        } = self.clone();
        let checked_at = checked_at.unwrap_or_else(Utc::now).to_rfc3339();
        let required = required4(url, status_code, is_reachable, expires_at);
        match required {
            | Ok((url, status_code, is_reachable, expires_at)) => {
                let query = format!(
                    "INSERT INTO {} (url, status_code, is_reachable, checked_at, expires_at) VALUES (?, ?, ?, ?, ?)",
                    self.table().name()
                );
                conn.execute(
                    &query,
                    params![url, status_code, is_reachable as i32, checked_at, expires_at.to_rfc3339()],
                )
                .map_err(|why| eyre!("=> {} Failed to insert link cache: {why}", Label::fail()))
                .map(|_| conn.last_insert_rowid())
            }
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        LinkCacheRow::build_select_query(self, base)
    }
}
impl Row for ResearchActivityCacheRow {
    fn table(&self) -> Table {
        Table::ResearchActivityCache
    }
    fn select(&self) -> DatabaseResult<Vec<ResearchActivityCacheRow>> {
        let table = self.table();
        let fields = Self::FIELDS.join(", ");
        let query = format!("SELECT {fields} FROM {}", table.name());
        let (query, params) = self.build_select_query(&query);
        table.rows(&query, params, None)
    }
    fn insert(self, conn: &Connection) -> DatabaseResult<i64> {
        let ResearchActivityCacheRow {
            identifier,
            source_bucket,
            title,
            downloaded_at,
            file_path,
            ..
        } = self.clone();
        let downloaded_at = downloaded_at.unwrap_or_else(Utc::now).to_rfc3339();
        let required = required4(identifier, source_bucket, title, file_path);
        match required {
            | Ok((identifier, source_bucket, title, file_path)) => {
                let query = format!(
                    "INSERT INTO {} (identifier, source_bucket, title, downloaded_at, file_path) VALUES (?, ?, ?, ?, ?)",
                    self.table().name()
                );
                conn.execute(&query, params![identifier, source_bucket, title, downloaded_at, file_path])
                    .map_err(|why| eyre!("=> {} Failed to insert research activity cache: {why}", Label::fail()))
                    .map(|_| conn.last_insert_rowid())
            }
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        ResearchActivityCacheRow::build_select_query(self, base)
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
        } = self.clone();
        let success = success.unwrap_or(true) as i32;
        let checked_at = checked_at.unwrap_or_else(Utc::now).to_rfc3339();
        let required = required3(path, check_type, message);
        match required {
            | Ok((path, check_type, message)) => {
                let query = format!(
                    "INSERT INTO {} (path, check_type, success, message, checked_at) VALUES (?, ?, ?, ?, ?)",
                    self.table().name()
                );
                conn.execute(&query, params![path, check_type, success, message, checked_at])
                    .map_err(|why| eyre!("=> {} Failed to insert validation: {why}", Label::fail()))
                    .map(|_| conn.last_insert_rowid())
            }
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        ValidationRow::build_select_query(self, base)
    }
}
impl LinkCacheRow {
    /// Check if the cache entry is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|e| Utc::now() > e).unwrap_or(true)
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
