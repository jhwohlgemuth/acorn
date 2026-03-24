//! Database backend abstraction layer.
//!
//! Conditionally re-exports types from either `rusqlite` (default) or `duckdb`
//! based on enabled feature flags.
use crate::io::ApiResult;
use crate::prelude::env;
use color_eyre::eyre::eyre;

/// Environment variable used to select a database backend at runtime.
pub const ENV_DATABASE_BACKEND: &str = "ACORN_DATABASE_BACKEND";
/// Environment variable used to override database path at runtime.
pub const ENV_DATABASE_PATH: &str = "ACORN_DATABASE_PATH";
/// SQLite backend value.
pub const BACKEND_SQLITE: &str = "sqlite";
/// DuckDB backend value.
pub const BACKEND_DUCKDB: &str = "duckdb";

#[cfg(feature = "duckdb")]
pub use duckdb::{params, params_from_iter, Connection, Params, ParamsFromIter, Row as BackendRow, ToSql};
#[cfg(not(feature = "duckdb"))]
pub use rusqlite::{params, params_from_iter, Connection, Params, ParamsFromIter, Row as BackendRow, ToSql};

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
    env::var(ENV_DATABASE_BACKEND)
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
