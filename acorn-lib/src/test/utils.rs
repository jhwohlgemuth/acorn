#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
//! Shared test utilities for `acorn-lib` unit tests.
use crate::cmd;
use crate::io::command_exists;
use crate::io::database::{schema::Table, Database, Operations};
use crate::io::ApiResult;
use crate::prelude::{create_dir_all, write, CommandOutput, PathBuf};
use chrono::Utc;
use core::time::Duration;
use std::time::SystemTime;

pub(crate) fn cleanup(path: &PathBuf) {
    let _ = write(path, []);
}
/// Get path to test fixtures relative to workspace root
pub fn fixture_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join(path)
}
/// Create a temporary database, run migrations, and populate the licenses table.
///
/// Returns the database path so tests can set `ACORN_DATABASE_PATH` as needed.
pub(crate) async fn temp_database_with_licenses() -> ApiResult<PathBuf> {
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
pub(crate) fn temp_file() -> PathBuf {
    let dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let filename = format!("test_docx_{}.docx", timestamp);
    dir.join(filename)
}
/// Build a unique path under `target/test_artifacts`.
pub(crate) fn unique_path(name: &str, extension: &str) -> PathBuf {
    let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or_default();
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target")
        .join("test_artifacts")
        .join(format!("{name}-{timestamp}.{extension}"))
}
/// Command string for `sh` variant that echoes `msg`.
pub(crate) fn sh_echo(msg: &str) -> String {
    if cfg!(unix) {
        format!("echo {msg}")
    } else {
        format!("cmd /c echo {msg}")
    }
}
/// Binary for echo in array/variable forms.
pub(crate) fn echo_bin() -> &'static str {
    if cfg!(unix) {
        "echo"
    } else {
        "cmd"
    }
}
/// Args to pass after `echo_bin()` to echo `msg`.
pub(crate) fn echo_args(msg: &str) -> Vec<String> {
    if cfg!(unix) {
        vec![msg.to_string()]
    } else {
        vec!["/c".into(), "echo".into(), msg.into()]
    }
}
/// Binary for stderr capture.
pub(crate) fn stderr_bin() -> &'static str {
    if cfg!(unix) {
        "sh"
    } else {
        "cmd"
    }
}
/// Args to pass after `stderr_bin()` to write `msg` to stderr.
pub(crate) fn stderr_args(msg: &str) -> Vec<String> {
    if cfg!(unix) {
        vec!["-c".into(), format!("echo {msg} >&2")]
    } else {
        vec!["/c".into(), format!("echo {msg} 1>&2")]
    }
}
/// Binary for exit code testing.
pub(crate) fn exit_bin() -> &'static str {
    if cfg!(unix) {
        "sh"
    } else {
        "cmd"
    }
}
/// Args to pass after `exit_bin()` to exit with `code`.
pub(crate) fn exit_args(code: i32) -> Vec<String> {
    if cfg!(unix) {
        vec!["-c".into(), format!("exit {code}")]
    } else {
        vec!["/c".into(), format!("exit /b {code}")]
    }
}
pub(crate) fn pwsh_is_healthy() -> bool {
    if !command_exists("pwsh") {
        return false;
    }
    match cmd!(pwsh "Write-Output 'shell-pwsh-health'") {
        | Ok(output) => {
            let stdout = output.stdout();
            let stderr = output.stderr();
            output.status.success()
                && stdout.contains("shell-pwsh-health")
                && !stdout.contains("Unhandled exception")
                && !stderr.contains("Unhandled exception")
        }
        | Err(_) => false,
    }
}
