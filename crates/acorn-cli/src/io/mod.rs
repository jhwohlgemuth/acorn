//! # Input / Output Handling
//!
//! This module contains functions for handling input and output from the terminal such as handling input passed via `stdin` (e.g., `cat file.txt | acorn`).
use acorn::prelude::{io, BufRead, PathBuf, Write};
use acorn::util::constants::app::APPLICATION;
use directories::BaseDirs;
use is_terminal::IsTerminal;

/// Enum for representing input/output stream state (e.g., "piped" or "not piped")
/// ### Notes
/// - "piped"
///   - stdin is piped (e.g., `echo "hello world" | acorn`)
///   - stdout is piped (e.g., `acorn | process`)
/// - "not piped" = stdin/stdout is not piped (e.g., `acorn`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputOutputStreamState {
    Piped,
    NotPiped,
}
/// Resolve the directory used for downloaded Chromium artifacts
pub fn chromium_cache_dir() -> Option<PathBuf> {
    BaseDirs::new().map(|dirs| dirs.cache_dir().join(APPLICATION).join("chromiumoxide"))
}
/// Check if stdout is piped (e.g., `acorn | process`)
pub fn is_stdout_piped() -> bool {
    stdout_piped_status() == InputOutputStreamState::Piped
}
/// Read input from stdin (e.g., `echo "hello world" | acorn`)
/// ### Example
/// ```ignore
/// // "echo "hello world" | acorn"
/// let input = read_stdin();
/// assert_eq!(input, Some("hello world".to_string()));
/// ```
pub fn read_stdin() -> Option<String> {
    match stdin_piped_status() {
        | InputOutputStreamState::Piped => {
            let stdin = io::stdin();
            let reader = stdin.lock();
            reader
                .lines()
                .collect::<core::result::Result<Vec<_>, _>>()
                .ok()
                .map(|lines| lines.join("\n"))
        }
        | InputOutputStreamState::NotPiped => None,
    }
}
/// Write value to stdout (e.g., `acorn | process`)
/// ### Example
/// ```ignore
/// // "acorn | process"
/// write_stdout("hello world");
/// ```
pub fn write_stdout<S>(value: S)
where
    S: Into<String>,
{
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(value.into().as_bytes());
    let _ = stdout.flush();
    drop(stdout);
}
fn stdin_piped_status() -> InputOutputStreamState {
    if io::stdin().is_terminal() {
        InputOutputStreamState::NotPiped
    } else {
        InputOutputStreamState::Piped
    }
}
fn stdout_piped_status() -> InputOutputStreamState {
    if io::stdout().is_terminal() {
        InputOutputStreamState::NotPiped
    } else {
        InputOutputStreamState::Piped
    }
}
