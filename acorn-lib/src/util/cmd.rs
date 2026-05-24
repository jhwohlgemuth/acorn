//! Internal helpers for the `cmd!` macro.
//!
//! These are `#[doc(hidden)]` and not part of the public API.
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus, Output};

/// Parse a command string using shell-aware word splitting (handles quoted arguments).
#[doc(hidden)]
pub fn parse_sh(cmd: &str) -> io::Result<(String, Vec<String>)> {
    let parts = shell_words::split(cmd).map_err(|e| io::Error::other(format!("cmd!(sh ...): failed to parse command string: {e}")))?;
    let mut parts = parts.into_iter();
    match parts.next() {
        | Some(binary) => {
            let args: Vec<String> = parts.collect();
            Ok((binary, args))
        }
        | None => Err(io::Error::other("cmd!(sh ...): empty command string")),
    }
}
/// Run a command, capture output, with optional working directory.
#[doc(hidden)]
pub fn run_output<I, S>(binary: &str, args: I, dir: Option<&Path>) -> io::Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = Command::new(binary);
    cmd.args(args);
    if let Some(dir) = dir {
        cmd.current_dir(dir);
    }
    cmd.output()
}
/// Run a command, check status only, with optional working directory.
#[doc(hidden)]
pub fn run_status<I, S>(binary: &str, args: I, dir: Option<&Path>) -> io::Result<ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = Command::new(binary);
    cmd.args(args);
    if let Some(dir) = dir {
        cmd.current_dir(dir);
    }
    cmd.status()
}
/// Convert a command result to `Result<String, String>`.
///
/// Returns stdout (trimmed) on success, or a descriptive error message on failure.
#[doc(hidden)]
pub fn try_from_output(result: io::Result<Output>) -> Result<String, String> {
    match result {
        | Ok(output) if output.status.success() => Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()),
        | Ok(output) => {
            let msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if msg.is_empty() {
                Err(format!("process exited with status {}", output.status))
            } else {
                Err(msg)
            }
        }
        | Err(e) => Err(format!("command execution failed: {e}")),
    }
}
