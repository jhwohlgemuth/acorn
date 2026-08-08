//! Database backend abstraction layer.
//!
//! Conditionally re-exports types from either `rusqlite` (default) or `duckdb`
//! based on enabled feature flags.
use crate::io::ApiResult;
use crate::prelude::{env, String};
use crate::util::constants::env::DATABASE_BACKEND;
use color_eyre::eyre::eyre;
#[cfg(feature = "duckdb")]
pub use duckdb::{params, params_from_iter, Connection, Error, Params, ParamsFromIter, Row as BackendRow, ToSql};
use jiff::Timestamp;
#[cfg(not(feature = "duckdb"))]
pub use rusqlite::{params, params_from_iter, Connection, Error, Params, ParamsFromIter, Row as BackendRow, ToSql};

/// SQLite backend value.
pub const BACKEND_SQLITE: &str = "sqlite";
/// DuckDB backend value.
pub const BACKEND_DUCKDB: &str = "duckdb";
/// Additional parsing operations for database backend rows.
pub trait BackendRowExt {
    /// Parse an RFC 3339 timestamp from the column at `index`.
    fn parse_rfc3339(&self, index: usize) -> Option<Timestamp>;
}
impl BackendRowExt for BackendRow<'_> {
    fn parse_rfc3339(&self, index: usize) -> Option<Timestamp> {
        self.get::<_, String>(index).ok().and_then(|value| value.parse().ok())
    }
}
/// Returns the backend compiled into this binary.
pub fn backend() -> &'static str {
    #[cfg(feature = "duckdb")]
    {
        BACKEND_DUCKDB
    }
    #[cfg(not(feature = "duckdb"))]
    {
        BACKEND_SQLITE
    }
}
/// Returns the backend selected from environment, defaulting to compiled backend.
pub fn selected_backend() -> String {
    env::var(DATABASE_BACKEND)
        .map(|value| value.trim().to_ascii_lowercase())
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| backend().to_string())
}
/// Validates runtime backend selection for the current build.
pub fn validate_backend_selection() -> ApiResult<String> {
    let selected = selected_backend();
    if selected != BACKEND_SQLITE && selected != BACKEND_DUCKDB {
        return Err(eyre!(
            "Invalid database backend '{}'. Supported values are '{}' or '{}'",
            selected,
            BACKEND_SQLITE,
            BACKEND_DUCKDB
        ));
    }
    let bundled = backend();
    if selected != bundled {
        return Err(eyre!(
            "Database backend '{}' is not available in this build (bundled backend: '{}'). Rebuild ACORN with matching feature flags.",
            selected,
            bundled,
        ));
    }
    Ok(selected)
}
