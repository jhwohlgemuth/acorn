//! Internal helpers for the `cmd!` macro.
//!
//! These are `#[doc(hidden)]` and not part of the public API.
#[cfg(feature = "shell-lint")]
use crate::prelude::env;
use crate::prelude::{io, Command, ExitStatus, Output, Path};
#[cfg(feature = "shell-lint")]
use crate::util::constants::env::SHELL_LINT_MIN_SEVERITY;
#[cfg(feature = "shell-lint")]
use shuck_linter::{lint, AnalysisRequest, Diagnostic, LinterSettings, Severity as LintSeverity, ShellDialect};
#[cfg(feature = "shell-lint")]
use shuck_parser::parser::Parser;

#[cfg(feature = "shell-lint")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum Severity {
    Hint,
    Warning,
    Error,
}
#[cfg(feature = "shell-lint")]
pub(crate) struct Linter;
#[cfg(feature = "shell-lint")]
impl Severity {
    pub(crate) fn minimum() -> Self {
        env::var(SHELL_LINT_MIN_SEVERITY)
            .ok()
            .map(|value| Self::from(value.as_str()))
            .unwrap_or(Self::Hint)
    }

    pub(crate) fn at_or_above(diagnostics: Vec<Diagnostic>, minimum_severity: Self) -> Vec<Diagnostic> {
        diagnostics
            .into_iter()
            .filter(|diagnostic| Self::from(diagnostic.severity) >= minimum_severity)
            .collect()
    }
}
#[cfg(feature = "shell-lint")]
impl From<&str> for Severity {
    fn from(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            | "hint" => Self::Hint,
            | "warning" | "warn" => Self::Warning,
            | "error" | "err" => Self::Error,
            | _ => Self::Hint,
        }
    }
}
#[cfg(feature = "shell-lint")]
impl From<Severity> for LintSeverity {
    fn from(value: Severity) -> Self {
        match value {
            | Severity::Hint => LintSeverity::Hint,
            | Severity::Warning => LintSeverity::Warning,
            | Severity::Error => LintSeverity::Error,
        }
    }
}
#[cfg(feature = "shell-lint")]
impl From<LintSeverity> for Severity {
    fn from(value: LintSeverity) -> Self {
        match value {
            | LintSeverity::Hint => Severity::Hint,
            | LintSeverity::Warning => Severity::Warning,
            | LintSeverity::Error => Severity::Error,
        }
    }
}
#[cfg(feature = "shell-lint")]
impl Linter {
    pub(crate) fn dialect(shell: &str) -> Option<ShellDialect> {
        match shell {
            | "bash" => Some(ShellDialect::Bash),
            | "sh" => Some(ShellDialect::Sh),
            | "dash" => Some(ShellDialect::Dash),
            | "ksh" => Some(ShellDialect::Ksh),
            | "mksh" => Some(ShellDialect::Mksh),
            | "zsh" => Some(ShellDialect::Zsh),
            | _ => None,
        }
    }
    fn error(diagnostics: &[Diagnostic]) -> io::Error {
        let extra = diagnostics.len().saturating_sub(5);
        let rendered = diagnostics
            .iter()
            .take(5)
            .map(|diagnostic| format!("{} {}", diagnostic.code(), diagnostic.message))
            .chain((extra > 0).then(|| format!("... and {extra} more")))
            .collect::<Vec<String>>()
            .join(" | ");
        io::Error::other(format!("shell lint failed: {rendered}"))
    }
    pub(crate) fn run(shell: &str, command: &str) -> io::Result<()> {
        match Self::dialect(shell) {
            | Some(dialect) => {
                let parse_result = Parser::with_dialect(command, dialect.parser_dialect()).parse();
                let settings = LinterSettings::default().with_shell(dialect);
                let diagnostics = lint(AnalysisRequest::from_parse_result(&parse_result, command, &settings));
                let filtered = Severity::at_or_above(diagnostics, Severity::minimum());
                if filtered.is_empty() {
                    Ok(())
                } else {
                    Err(Self::error(&filtered))
                }
            }
            | None => Ok(()),
        }
    }
}
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
    let mut cmd = command_with_args(binary, args, dir);
    cmd.output()
}
/// Run a command string using a shell, capture output, with optional working directory.
#[doc(hidden)]
pub fn run_shell_output(shell: &str, shell_args: &[&str], command: &str, dir: Option<&Path>) -> io::Result<Output> {
    #[cfg(feature = "shell-lint")]
    {
        Linter::run(shell, command).and_then(|()| {
            let mut cmd = shell_command(shell, shell_args, command, dir);
            cmd.output()
        })
    }
    #[cfg(not(feature = "shell-lint"))]
    {
        let mut cmd = shell_command(shell, shell_args, command, dir);
        cmd.output()
    }
}
/// Run a command string using a shell, check status only, with optional working directory.
#[doc(hidden)]
pub fn run_shell_status(shell: &str, shell_args: &[&str], command: &str, dir: Option<&Path>) -> io::Result<ExitStatus> {
    #[cfg(feature = "shell-lint")]
    {
        Linter::run(shell, command).and_then(|()| {
            let mut cmd = shell_command(shell, shell_args, command, dir);
            cmd.status()
        })
    }
    #[cfg(not(feature = "shell-lint"))]
    {
        let mut cmd = shell_command(shell, shell_args, command, dir);
        cmd.status()
    }
}
/// Run a command, check status only, with optional working directory.
#[doc(hidden)]
pub fn run_status<I, S>(binary: &str, args: I, dir: Option<&Path>) -> io::Result<ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = command_with_args(binary, args, dir);
    cmd.status()
}
/// Convert a command result to `Result<String, String>`.
///
/// Returns stdout (trimmed) on success, or a descriptive error message on failure.
#[doc(hidden)]
pub fn try_from_output(result: io::Result<Output>) -> Result<String, String> {
    match result {
        | Ok(output) if output.status.success() => match String::from_utf8(output.stdout) {
            | Ok(value) => Ok(value.trim().to_string()),
            | Err(why) => Err(format!("stdout is not valid UTF-8: {why}")),
        },
        | Ok(output) => {
            let msg = match String::from_utf8(output.stderr) {
                | Ok(value) => value.trim().to_string(),
                | Err(_) => String::new(),
            };
            if msg.is_empty() {
                Err(format!("process exited with status {}", output.status))
            } else {
                Err(msg)
            }
        }
        | Err(e) => Err(format!("command execution failed: {e}")),
    }
}
fn command_with_args<I, S>(binary: &str, args: I, dir: Option<&Path>) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = Command::new(binary);
    cmd.args(args);
    if let Some(dir) = dir {
        cmd.current_dir(dir);
    }
    cmd
}
fn shell_command(shell: &str, shell_args: &[&str], command: &str, dir: Option<&Path>) -> Command {
    let mut cmd = command_with_args(shell, shell_args, dir);
    cmd.arg(command);
    cmd
}
