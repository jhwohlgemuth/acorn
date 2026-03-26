//! Database schema model
//!
use super::backend::{self, BackendRow, Connection, Params};
use super::{Database, Row, SelectQuery, TableSchemaProvider};
use crate::io::api::{self, DatabasePersistence};
use crate::io::database::macros::{build_query, define_required_fn};
use crate::io::{create_progress_bar, finish_progress_bar, ApiResult, ProgressType};
use crate::prelude::PathBuf;
use crate::util::{print_values_as_table, Label};
use acorn_macros::DatabaseRow;
use async_trait::async_trait;
use bon::Builder;
use chrono::{DateTime, Utc};
use color_eyre::eyre::eyre;
use core::fmt;
use serde::{Deserialize, Serialize};

define_required_fn!(1, required1, A, a);
define_required_fn!(3, required3, A, a, B, b, C, c);
define_required_fn!(4, required4, A, a, B, b, C, c, D, d);
define_required_fn!(5, required5, A, a, B, b, C, c, D, d, E, e);
/// Database tables
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[repr(i32)]
pub enum Table {
    /// CLI activity log
    Activity = 0,
    /// Research activity index entries from buckets
    Catalog = 1,
    /// SPDX license data
    Licenses = 2,
    /// Cached link check results
    LinkCache = 3,
    /// Validation/check results history
    ValidationHistory = 4,
    /// Downloaded research activity snapshots
    ResearchActivityCache = 5,
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
/// SPDX license entry
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[row(table = licenses)]
pub struct LicenseRow {
    /// Unique identifier for the license entry
    pub id: Option<i64>,
    /// Unique license identifier (ex., "MIT")
    pub identifier: Option<String>,
    /// License name (ex., "MIT License")
    pub name: Option<String>,
    /// Specifies whether the entire license is deprecated
    pub is_deprecated: Option<bool>,
    /// Specifies whether the License is listed as free by the Free Software Foundation (FSF)
    pub is_free_software: Option<bool>,
    /// Specifies whether the license is approved by the Open Source Initiative (OSI)
    pub is_open_source: Option<bool>,
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
impl From<&str> for Table {
    fn from(value: &str) -> Self {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            | "activity" => Self::Activity,
            | "catalog" => Self::Catalog,
            | "licenses" | "license" => Self::Licenses,
            | "link_cache" | "linkcache" => Self::LinkCache,
            | "validation_history" | "validationhistory" => Self::ValidationHistory,
            | "research_activity_cache" | "researchactivitycache" => Self::ResearchActivityCache,
            | _ => Self::Activity,
        }
    }
}
impl From<&BackendRow<'_>> for ActivityRow {
    fn from(row: &BackendRow<'_>) -> Self {
        ActivityRow {
            id: row.get(0).ok(),
            command: row.get(1).ok(),
            executed_at: parse_rfc3339_column(row, 2),
            user_path: row.get(3).ok(),
            success: row.get::<_, i32>(4).ok().map(|value| value != 0),
        }
    }
}
impl From<BackendRow<'_>> for ActivityRow {
    fn from(row: BackendRow<'_>) -> Self {
        ActivityRow::from(&row)
    }
}
impl From<&BackendRow<'_>> for LicenseRow {
    fn from(row: &BackendRow<'_>) -> Self {
        LicenseRow {
            id: row.get(0).ok(),
            identifier: row.get(1).ok(),
            name: row.get(2).ok(),
            is_deprecated: row.get::<_, i32>(3).ok().map(|value| value != 0),
            is_free_software: row.get::<_, i32>(4).ok().map(|value| value != 0),
            is_open_source: row.get::<_, i32>(5).ok().map(|value| value != 0),
        }
    }
}
impl From<BackendRow<'_>> for LicenseRow {
    fn from(row: BackendRow<'_>) -> Self {
        LicenseRow::from(&row)
    }
}
impl From<&BackendRow<'_>> for CatalogRow {
    fn from(row: &BackendRow<'_>) -> Self {
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
impl From<BackendRow<'_>> for CatalogRow {
    fn from(row: BackendRow<'_>) -> Self {
        CatalogRow::from(&row)
    }
}
impl From<&BackendRow<'_>> for LinkCacheRow {
    fn from(row: &BackendRow<'_>) -> Self {
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
impl From<BackendRow<'_>> for LinkCacheRow {
    fn from(row: BackendRow<'_>) -> Self {
        LinkCacheRow::from(&row)
    }
}
impl From<&BackendRow<'_>> for ResearchActivityCacheRow {
    fn from(row: &BackendRow<'_>) -> Self {
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
impl From<BackendRow<'_>> for ResearchActivityCacheRow {
    fn from(row: BackendRow<'_>) -> Self {
        ResearchActivityCacheRow::from(&row)
    }
}
impl From<&BackendRow<'_>> for ValidationRow {
    fn from(row: &BackendRow<'_>) -> Self {
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
impl From<BackendRow<'_>> for ValidationRow {
    fn from(row: BackendRow<'_>) -> Self {
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
impl From<LicenseRow> for Vec<String> {
    fn from(row: LicenseRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.identifier.unwrap_or_default(),
            row.name.unwrap_or_default(),
            row.is_deprecated.map_or_else(String::new, |v| v.to_string()),
            row.is_free_software.map_or_else(String::new, |v| v.to_string()),
            row.is_open_source.map_or_else(String::new, |v| v.to_string()),
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
impl fmt::Display for LicenseRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LicenseRow = id: {}, identifier: {}, name: {}, deprecated: {}, free: {}, osi: {}",
            display_or_none(self.id.as_ref()),
            display_or_none(self.identifier.as_ref()),
            display_or_none(self.name.as_ref()),
            self.is_deprecated.unwrap_or(false),
            self.is_free_software.unwrap_or(false),
            self.is_open_source.unwrap_or(false),
        )
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
#[async_trait]
impl TableSchemaProvider for Table {
    fn all() -> &'static [Self] {
        &[
            Table::Activity,
            Table::Catalog,
            Table::Licenses,
            Table::LinkCache,
            Table::ValidationHistory,
            Table::ResearchActivityCache,
        ]
    }
    /// Returns CREATE TABLE SQL statement
    fn create_statement(&self) -> String {
        let columns = match self {
            | Table::Activity => {
                r#"
                    command TEXT NOT NULL,
                    executed_at TEXT NOT NULL,
                    user_path TEXT,
                    success INTEGER NOT NULL DEFAULT 1
                "#
            }
            | Table::Catalog => {
                r#"
                    bucket_name TEXT NOT NULL,
                    bucket_url TEXT NOT NULL,
                    identifier TEXT NOT NULL,
                    title TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    UNIQUE(bucket_name, identifier)
                "#
            }
            | Table::Licenses => {
                r#"
                    identifier TEXT NOT NULL UNIQUE,
                    name TEXT NOT NULL,
                    is_deprecated INTEGER NOT NULL,
                    is_free_software INTEGER NOT NULL,
                    is_open_source INTEGER NOT NULL
                "#
            }
            | Table::LinkCache => {
                r#"
                    url TEXT NOT NULL UNIQUE,
                    status_code INTEGER NOT NULL,
                    is_reachable INTEGER NOT NULL,
                    checked_at TEXT NOT NULL,
                    expires_at TEXT NOT NULL
                "#
            }
            | Table::ValidationHistory => {
                r#"
                    path TEXT NOT NULL,
                    check_type TEXT NOT NULL,
                    success INTEGER NOT NULL,
                    message TEXT NOT NULL,
                    checked_at TEXT NOT NULL
                "#
            }
            | Table::ResearchActivityCache => {
                r#"
                    identifier TEXT NOT NULL UNIQUE,
                    source_bucket TEXT NOT NULL,
                    title TEXT NOT NULL,
                    downloaded_at TEXT NOT NULL,
                    file_path TEXT NOT NULL
                "#
            }
        };

        format!(
            r#"
                CREATE TABLE IF NOT EXISTS {} (
                    {},
                    {}
                )
                "#,
            self.name(),
            id_column_definition(),
            columns.trim()
        )
    }
    /// Returns the table name as a string
    fn name(&self) -> &'static str {
        match self {
            | Table::Activity => "activity",
            | Table::Catalog => "catalog",
            | Table::Licenses => "licenses",
            | Table::LinkCache => "link_cache",
            | Table::ValidationHistory => "validation_history",
            | Table::ResearchActivityCache => "research_activity_cache",
        }
    }
    async fn populate(&self) -> ApiResult<usize> {
        match &self {
            | Table::Licenses => {
                let progress = create_progress_bar(0, ProgressType::Spinner);
                progress.set_message("Downloading SPDX license data...");
                let response = api::spdx::download().await;
                let message = format!("{}SPDX metadata download complete", Label::CHECKMARK);
                finish_progress_bar(&progress, message);
                match response {
                    | Ok(data) => data.persist(Database::<Self>::from_path(None)).await,
                    | Err(why) => Err(eyre!("Failed to download SPDX license data — {why}")),
                }
            }
            | _ => unimplemented!("populate() not implemented for {} table", self.name()),
        }
    }
    /// Print all table rows
    fn print(self, path: Option<PathBuf>) {
        fn print_rows<R>(table: Table, path: Option<&PathBuf>)
        where
            R: Into<Vec<String>> + Row + Default + fmt::Display,
            for<'row> R: From<&'row BackendRow<'row>>,
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
            | Table::Licenses => print_rows::<LicenseRow>(self, path.as_ref()),
            | Table::LinkCache => print_rows::<LinkCacheRow>(self, path.as_ref()),
            | Table::ValidationHistory => print_rows::<ValidationRow>(self, path.as_ref()),
            | Table::ResearchActivityCache => print_rows::<ResearchActivityCacheRow>(self, path.as_ref()),
        }
    }
    /// Get table rows
    fn rows<R, P>(&self, query: &str, params: P, path: Option<&PathBuf>) -> ApiResult<Vec<R>>
    where
        P: Params,
        for<'row> R: Row + From<&'row BackendRow<'row>>,
    {
        Database::<Table>::from_path(path.cloned()).with_connection(|conn| {
            conn.prepare(query)
                .map_err(|why| eyre!("=> {} Failed to prepare {} query: {why}", Label::fail(), self.name()))
                .and_then(|mut stmt| {
                    stmt.query_map(params, |row| Ok(R::from(row)))
                        .map_err(|why| eyre!("=> {} Failed to query {}: {why}", Label::fail(), self.name()))
                        .and_then(|rows| {
                            rows.collect::<core::result::Result<Vec<_>, _>>()
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
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
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
            | Ok((command,)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![id, command, executed_at, user_path, success];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl Row for CatalogRow {
    fn table(&self) -> Table {
        Table::Catalog
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
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
            | Ok((bucket_name, bucket_url, identifier, title)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![id, bucket_name, bucket_url, identifier, title, updated_at];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl Row for LicenseRow {
    fn table(&self) -> Table {
        Table::Licenses
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
            identifier,
            name,
            is_deprecated,
            is_free_software,
            is_open_source,
            ..
        } = self.clone();
        let required = required5(identifier, name, is_deprecated, is_free_software, is_open_source);
        match required {
            | Ok((identifier, name, is_deprecated, is_free_software, is_open_source)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![id, identifier, name, is_deprecated as i32, is_free_software as i32, is_open_source as i32];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl Row for LinkCacheRow {
    fn table(&self) -> Table {
        Table::LinkCache
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
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
            | Ok((url, status_code, is_reachable, expires_at)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![id, url, status_code, is_reachable as i32, checked_at, expires_at.to_rfc3339()];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl Row for ResearchActivityCacheRow {
    fn table(&self) -> Table {
        Table::ResearchActivityCache
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
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
            | Ok((identifier, source_bucket, title, file_path)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![id, identifier, source_bucket, title, downloaded_at, file_path];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl Row for ValidationRow {
    fn table(&self) -> Table {
        Table::ValidationHistory
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
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
            | Ok((path, check_type, message)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![id, path, check_type, success, message, checked_at];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
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
fn execute_insert<R, P>(row: &R, conn: &Connection, params: P) -> ApiResult<usize>
where
    R: Row,
    P: Params,
{
    let table_name = row.table().name();
    let table_columns = <R as Row>::fields();
    let fields = table_columns.join(", ");
    let placeholders = table_columns.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let query = format!("INSERT INTO {table_name} ({fields}) VALUES ({placeholders})");
    conn.execute(&query, params)
        .map_err(|why| eyre!("=> {} Failed to insert row: {why}", Label::fail()))
}
fn parse_rfc3339_column(row: &BackendRow<'_>, index: usize) -> Option<DateTime<Utc>> {
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

#[cfg(feature = "duckdb")]
fn id_column_definition() -> &'static str {
    "id BIGINT PRIMARY KEY"
}

#[cfg(not(feature = "duckdb"))]
fn id_column_definition() -> &'static str {
    "id INTEGER PRIMARY KEY AUTOINCREMENT"
}
