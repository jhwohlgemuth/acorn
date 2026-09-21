//! Command runtime and process specifications
use core::{iter::once, time::Duration};
#[cfg(feature = "shell-lint")]
use shuck_linter::{lint, AnalysisRequest, Diagnostic, LinterSettings, Severity as LintSeverity, ShellDialect};
#[cfg(feature = "shell-lint")]
use shuck_parser::parser::Parser;
use std::{
    env,
    ffi::{OsStr, OsString},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Output, Stdio},
    thread,
    time::Instant,
};
use tracing::warn;

#[cfg(feature = "shell-lint")]
const SHELL_LINT_MIN_SEVERITY: &str = "ACORN_SHELL_LINT_MIN_SEVERITY";
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
/// Immutable specification for one native command invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandSpec {
    args: Vec<OsString>,
    current_dir: Option<PathBuf>,
    envs: Vec<(OsString, OsString)>,
    program: OsString,
}
impl CommandSpec {
    /// Create a command specification for `program`.
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            args: Vec::new(),
            current_dir: None,
            envs: Vec::new(),
            program: program.into(),
        }
    }
    /// Append one command argument.
    pub fn arg(mut self, argument: impl Into<OsString>) -> Self {
        self.args.push(argument.into());
        self
    }
    /// Append command arguments in iterator order.
    pub fn args<I, S>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(arguments.into_iter().map(Into::into));
        self
    }
    /// Return whether the configured program can be discovered on this host.
    pub fn available(&self) -> bool {
        find_executable(&self.program).is_some()
    }
    /// Run the command and return an error for a non-success exit status.
    pub fn checked_output(&self) -> io::Result<Output> {
        checked(self.output())
    }
    /// Run the command within `timeout` and return an error for a non-success exit status.
    pub fn checked_output_with_timeout(&self, timeout: Duration) -> io::Result<Output> {
        checked(self.output_with_timeout(timeout))
    }
    /// Set the child process working directory.
    pub fn current_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(path.into());
        self
    }
    /// Set one child environment override.
    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.envs.push((key.into(), value.into()));
        self
    }
    /// Run the command and capture standard output and error.
    pub fn output(&self) -> io::Result<Output> {
        self.command().output()
    }
    /// Run the command within `timeout` and capture standard output and error.
    pub fn output_with_timeout(&self, timeout: Duration) -> io::Result<Output> {
        let mut command = self.command();
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        command.spawn().and_then(|child| wait_with_output(child, timeout))
    }
    /// Spawn the configured command.
    pub fn spawn(&self) -> io::Result<Child> {
        self.command().spawn()
    }
    fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args).envs(self.envs.iter().map(|(key, value)| (key, value)));
        if let Some(path) = &self.current_dir {
            command.current_dir(path);
        }
        command
    }
}
/// Parse a command string using shell-aware word splitting (handles quoted arguments).
#[doc(hidden)]
pub fn parse_sh(cmd: &str) -> io::Result<(String, Vec<String>)> {
    shell_words::split(cmd)
        .map_err(|e| io::Error::other(format!("cmd!(sh ...): failed to parse command string: {e}")))
        .and_then(|parts| {
            let mut parts = parts.into_iter();
            match parts.next() {
                | Some(binary) => {
                    let args: Vec<String> = parts.collect();
                    Ok((binary, args))
                }
                | None => Err(io::Error::other("cmd!(sh ...): empty command string")),
            }
        })
}
/// Run a command, capture output, with optional working directory.
#[doc(hidden)]
pub fn run_output<I, S>(binary: impl AsRef<str>, args: I, dir: Option<&Path>) -> io::Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_output_with_env(binary, args, dir, core::iter::empty::<(OsString, OsString)>())
}
/// Run a command, capture output, with an optional working directory and child-process environment overrides.
#[doc(hidden)]
pub fn run_output_with_env<I, S, E, K, V>(binary: impl AsRef<str>, args: I, dir: Option<&Path>, envs: E) -> io::Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let binary = binary.as_ref();
    let args = args.into_iter().map(|value| value.as_ref().to_os_string()).collect::<Vec<OsString>>();
    log_command(binary, &args, dir);
    let mut cmd = command_with_args(binary, &args, dir);
    cmd.envs(envs);
    cmd.output()
}
/// Run a command with standard-input bytes and capture its output.
#[doc(hidden)]
pub fn run_output_with_stdin<I, S>(binary: impl AsRef<str>, args: I, dir: Option<&Path>, input: &[u8]) -> io::Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let binary = binary.as_ref();
    let args = args.into_iter().map(|value| value.as_ref().to_os_string()).collect::<Vec<OsString>>();
    log_command(binary, &args, dir);
    let mut cmd = command_with_args(binary, &args, dir);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.spawn().and_then(|mut child| {
        child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("failed to open command standard input"))
            .and_then(|mut stdin| stdin.write_all(input))
            .and_then(|()| child.wait_with_output())
    })
}
/// Run a command string using a shell, capture output, with optional working directory.
#[doc(hidden)]
pub fn run_shell_output(shell: &str, shell_args: &[&str], command: &str, dir: Option<&Path>) -> io::Result<Output> {
    log_shell_command(shell, shell_args, command, dir);
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
    log_shell_command(shell, shell_args, command, dir);
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
    let args = args.into_iter().map(|value| value.as_ref().to_os_string()).collect::<Vec<OsString>>();
    log_command(binary, &args, dir);
    let mut cmd = command_with_args(binary, &args, dir);
    cmd.status()
}
/// Spawn a detached-input command with inherited standard error.
#[doc(hidden)]
pub fn spawn<I, S>(binary: impl AsRef<OsStr>, args: I, dir: Option<&Path>) -> io::Result<Child>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let binary = binary.as_ref();
    let args = args.into_iter().map(|value| value.as_ref().to_os_string()).collect::<Vec<OsString>>();
    log_command(&binary.to_string_lossy(), &args, dir);
    let mut command = Command::new(binary);
    command.args(&args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::inherit());
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    command.spawn()
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
/// Render a binary and its arguments as a shell-safe command string.
#[doc(hidden)]
pub fn command_string(binary: &str, args: &[OsString]) -> String {
    once(escape(binary))
        .chain(args.iter().map(|argument| escape(&argument.to_string_lossy())))
        .collect::<Vec<_>>()
        .join(" ")
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
fn escape(value: &str) -> String {
    if value.is_empty() {
        "\"\"".to_string()
    } else if value.chars().any(|ch| ch.is_whitespace() || matches!(ch, '"' | '\'' | '\\')) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}
fn log_command(binary: &str, args: &[OsString], dir: Option<&Path>) {
    let rendered = once(escape(binary)).chain(redact_args(args)).collect::<Vec<_>>().join(" ");
    let cwd = dir.map(|path| path.display().to_string()).unwrap_or_else(|| "<current>".to_string());
    warn!(cwd, "=>  RUN ▶ {rendered}");
}
fn log_shell_command(shell: &str, shell_args: &[&str], command: &str, dir: Option<&Path>) {
    let args = shell_args
        .iter()
        .map(|value| OsString::from(*value))
        .chain(once(OsString::from(command)))
        .collect::<Vec<OsString>>();
    log_command(shell, &args, dir);
}
/// Remove registry authorization details from diagnostic text.
pub fn redact(value: &str) -> String {
    value
        .lines()
        .map(|line| match line.to_ascii_lowercase().contains("authorization") {
            | true => "auth=[REDACTED]",
            | false => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}
/// Render command arguments while replacing values of common secret flags.
pub fn redact_args(args: &[OsString]) -> Vec<String> {
    let sensitive_flags = ["--api-key", "--api-token", "--password", "--registration-token", "--secret", "--token"];
    args.iter()
        .scan(false, |redact_next, arg| {
            let raw = arg.to_string_lossy();
            let rendered = if *redact_next {
                *redact_next = false;
                "[REDACTED]".to_string()
            } else {
                match raw.split_once('=') {
                    | Some((flag, _)) if sensitive_flags.contains(&flag) => format!("{flag}=[REDACTED]"),
                    | _ => {
                        *redact_next = sensitive_flags.contains(&raw.as_ref());
                        escape(&raw)
                    }
                }
            };
            Some(rendered)
        })
        .collect::<Vec<_>>()
}
/// Find an executable by explicit path or by searching the process `PATH`.
pub fn find_executable(program: impl AsRef<OsStr>) -> Option<PathBuf> {
    let program = PathBuf::from(program.as_ref());
    match program.components().count() > 1 {
        | true => is_executable(&program).then_some(program),
        | false => env::var_os("PATH")
            .into_iter()
            .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
            .flat_map(|directory| executable_candidates(&directory, &program))
            .find(|candidate| is_executable(candidate)),
    }
}
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}
#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}
fn checked(result: io::Result<Output>) -> io::Result<Output> {
    result.and_then(|output| match output.status.success() {
        | true => Ok(output),
        | false => {
            let detail = redact(String::from_utf8_lossy(&output.stderr).trim());
            let message = match detail.is_empty() {
                | true => format!("command exited with status {}", output.status),
                | false => format!("command exited with status {} — {detail}", output.status),
            };
            Err(io::Error::other(message))
        }
    })
}
#[cfg(not(windows))]
fn executable_candidates(directory: &Path, program: &Path) -> Vec<PathBuf> {
    vec![directory.join(program)]
}
#[cfg(windows)]
fn executable_candidates(directory: &Path, program: &Path) -> Vec<PathBuf> {
    match program.extension().is_some() {
        | true => vec![directory.join(program)],
        | false => env::var_os("PATHEXT")
            .and_then(|value| value.into_string().ok())
            .unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".to_string())
            .split(';')
            .filter(|extension| !extension.is_empty())
            .map(|extension| directory.join(program).with_extension(extension.trim_start_matches('.')))
            .collect(),
    }
}
fn read_pipe(pipe: Option<impl Read>) -> io::Result<Vec<u8>> {
    pipe.map_or_else(
        || Ok(Vec::new()),
        |mut pipe| {
            let mut bytes = Vec::new();
            pipe.read_to_end(&mut bytes).map(|_| bytes)
        },
    )
}
fn shell_command(shell: &str, shell_args: &[&str], command: &str, dir: Option<&Path>) -> Command {
    let mut cmd = command_with_args(shell, shell_args, dir);
    cmd.arg(command);
    cmd
}
fn wait_with_output(mut child: Child, timeout: Duration) -> io::Result<Output> {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_reader = thread::spawn(move || read_pipe(stdout));
    let stderr_reader = thread::spawn(move || read_pipe(stderr));
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            | Ok(Some(status)) => break Ok(status),
            | Ok(None) if started.elapsed() < timeout => thread::sleep(Duration::from_millis(10)),
            | Ok(None) => {
                let timeout_error = io::Error::new(io::ErrorKind::TimedOut, format!("command timed out after {timeout:?}"));
                break child.kill().and_then(|()| child.wait()).and(Err(timeout_error));
            }
            | Err(why) => break Err(why),
        }
    };
    status.and_then(|status| {
        let stdout = join_pipe(stdout_reader);
        let stderr = join_pipe(stderr_reader);
        stdout.and_then(|stdout| stderr.map(|stderr| Output { status, stdout, stderr }))
    })
}
fn join_pipe(reader: thread::JoinHandle<io::Result<Vec<u8>>>) -> io::Result<Vec<u8>> {
    reader
        .join()
        .map_err(|_| io::Error::other("command output reader panicked"))
        .and_then(|result| result)
}
