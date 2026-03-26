//! Shared test utilities for `acorn-lib` unit tests.
use crate::io::database::{schema::Table, Database, Operations};
use crate::io::ApiResult;
use crate::prelude::{create_dir_all, PathBuf};
use core::time::Duration;
use std::time::SystemTime;

/// Create a temporary database, run migrations, and populate the licenses table.
///
/// Returns the database path so tests can set `ACORN_DATABASE_PATH` as needed.
pub async fn temp_database_with_licenses() -> ApiResult<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_nanos(0))
        .as_nanos();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target")
        .join("test_artifacts")
        .join(format!("licenses-{timestamp}.db"));
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let database = Database::<Table>::from_path(Some(path.clone()));
    database.migrate()?;
    let _ = database.populate(Table::Licenses).await?;
    Ok(path)
}
