//! # Input / Output Handling
//!
//! This module contains functions for handling input and output from the terminal such as handling input passed via `stdin` (e.g., `cat file.txt | acorn`).
use acorn::prelude::{io, BufRead, Write};
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
            let lines = reader.lines();
            // TODO: Handle errors and get rid of unwrap
            let value = lines.last().expect("Failed to read stdin line").unwrap();
            Some(value)
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
    stdout.write_all(value.into().as_bytes()).unwrap();
    stdout.flush().unwrap();
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
