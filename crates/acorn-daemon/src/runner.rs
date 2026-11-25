//! Fixed, bounded invocation of the co-installed ACORN CLI for daemon jobs.
use crate::config::JobConfig;
use acorn::io::standard_project_folder;
use acorn_service::RunState;
use acorn_store::daemon::{DaemonStore, StoreError};
use command_group::{CommandGroup, GroupChild};
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::Instant;
use thiserror::Error;

const DEFAULT_MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10 * 60);
#[derive(Clone, Copy)]
enum ReportSource {
    Stdout,
    File,
}
/// Failure to launch, capture, or persist a fixed daemon job.
#[derive(Debug, Error)]
pub enum RunnerError {
    /// A host process or report operation failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// The daemon history adapter rejected an update.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// A report path would be observed as a watched project input.
    #[error("daemon report path is inside the watched project")]
    ReportInsideProject,
    /// The configured action has no fixed daemon runner.
    #[error("unsupported daemon job kind")]
    UnsupportedKind,
}

struct Captured {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    timed_out: bool,
    cancelled: bool,
    truncated: bool,
    report_ready: bool,
}
/// Limits applied to one fixed ACORN child process.
#[derive(Clone, Copy, Debug)]
pub struct RunLimits {
    /// Maximum time allowed for the child process.
    pub timeout: Duration,
    /// Maximum bytes retained from each output stream or report file.
    pub output_bytes: usize,
}
/// Outcome retained by a daemon runner after a child exits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunOutcome {
    /// Child exit code, if the operating system supplied one.
    pub exit_code: Option<i32>,
    /// Path containing the bounded JSON report when captured.
    pub report_path: Option<PathBuf>,
    /// Whether the process exceeded its configured deadline.
    pub timed_out: bool,
    /// Whether shutdown cancelled the process.
    pub cancelled: bool,
    /// Whether a captured stream or report file exceeded its byte ceiling.
    pub truncated: bool,
}
impl Captured {
    fn succeeded(&self) -> bool {
        self.status.success() && !self.timed_out && !self.cancelled && !self.truncated && self.report_ready
    }
}
impl Default for RunLimits {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
        }
    }
}
impl From<Captured> for RunState {
    fn from(captured: Captured) -> Self {
        if captured.cancelled {
            Self::Interrupted
        } else if captured.succeeded() {
            Self::Succeeded
        } else {
            Self::Failed
        }
    }
}
fn capture(mut child: GroupChild, limits: RunLimits, stop: &AtomicBool, report: Option<&Path>) -> io::Result<Captured> {
    let stdout = child.inner().stdout.take().ok_or_else(|| io::Error::other("child stdout unavailable"));
    let stderr = child.inner().stderr.take().ok_or_else(|| io::Error::other("child stderr unavailable"));
    stdout.and_then(|stdout| {
        stderr.and_then(|stderr| {
            thread::scope(|scope| {
                let stdout_reader = scope.spawn(|| read_bounded(stdout, limits.output_bytes));
                let stderr_reader = scope.spawn(|| read_bounded(stderr, limits.output_bytes));
                let deadline = Instant::now().checked_add(limits.timeout).unwrap_or_else(Instant::now);
                let (status, timed_out, cancelled, oversized) = loop {
                    match child.try_wait() {
                        | Ok(Some(status)) => break (Ok(status), false, false, false),
                        | Ok(None) if stop.load(Ordering::Relaxed) => {
                            let stopped = child.kill().and_then(|()| child.wait());
                            break (stopped, false, true, false);
                        }
                        | Ok(None) if Instant::now() >= deadline => {
                            let stopped = child.kill().and_then(|()| child.wait());
                            break (stopped, true, false, false);
                        }
                        | Ok(None) if report_exceeds_limit(report, limits.output_bytes) => {
                            let stopped = child.kill().and_then(|()| child.wait());
                            break (stopped, false, false, true);
                        }
                        | Ok(None) => thread::sleep(Duration::from_millis(20)),
                        | Err(error) => break (Err(error), false, false, false),
                    }
                };
                let stdout = stdout_reader.join().map_err(|_| io::Error::other("stdout reader panicked"));
                let stderr = stderr_reader.join().map_err(|_| io::Error::other("stderr reader panicked"));
                status.and_then(|status| {
                    stdout.and_then(|stdout| {
                        stderr.and_then(|stderr| {
                            stdout.and_then(|(stdout, stdout_truncated)| {
                                stderr.map(|(stderr, stderr_truncated)| Captured {
                                    status,
                                    stdout,
                                    stderr,
                                    timed_out,
                                    cancelled,
                                    truncated: stdout_truncated || stderr_truncated || oversized,
                                    report_ready: false,
                                })
                            })
                        })
                    })
                })
            })
        })
    })
}
fn read_bounded(mut stream: impl Read, maximum: usize) -> io::Result<(Vec<u8>, bool)> {
    let mut retained = Vec::new();
    let mut truncated = false;
    let mut buffer = [0_u8; 8192];
    let result = loop {
        match stream.read(&mut buffer) {
            | Ok(0) => break Ok(()),
            | Ok(count) => {
                let available = maximum.saturating_sub(retained.len());
                retained.extend_from_slice(buffer.get(..count.min(available)).unwrap_or(&[]));
                truncated |= count > available;
            }
            | Err(error) => break Err(error),
        }
    };
    result.map(|()| (retained, truncated))
}

fn report_exceeds_limit(report: Option<&Path>, maximum: usize) -> bool {
    report
        .and_then(|path| fs::metadata(path).ok())
        .is_some_and(|metadata| metadata.len() > u64::try_from(maximum).unwrap_or(u64::MAX))
}
/// Run a claimed check job through the paired CLI and persist its result.
pub fn run_check(job: &JobConfig, run_id: i64, store: &DaemonStore, binary: &Path, limits: RunLimits) -> Result<RunOutcome, RunnerError> {
    let report = standard_project_folder("check", None).with_extension("json");
    run_check_at(job, run_id, store, binary, &report, limits)
}
pub(crate) fn run_check_at(
    job: &JobConfig,
    run_id: i64,
    store: &DaemonStore,
    binary: &Path,
    report: &Path,
    limits: RunLimits,
) -> Result<RunOutcome, RunnerError> {
    run_check_at_until(job, run_id, store, binary, report, limits, &AtomicBool::new(false))
}
pub(crate) fn run_check_at_until(
    job: &JobConfig,
    run_id: i64,
    store: &DaemonStore,
    binary: &Path,
    report: &Path,
    limits: RunLimits,
    stop: &AtomicBool,
) -> Result<RunOutcome, RunnerError> {
    let input = job.root().join(job.input());
    let mut command = Command::new(binary);
    command
        .arg("check")
        .arg(&input)
        .args(["--output", "json"])
        .current_dir(job.root())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    run_fixed(command, run_id, store, report, ReportSource::Stdout, limits, stop)
}
/// Run a check job with a shared shutdown signal.
pub fn run_check_until(
    job: &JobConfig,
    run_id: i64,
    store: &DaemonStore,
    binary: &Path,
    limits: RunLimits,
    stop: &AtomicBool,
) -> Result<RunOutcome, RunnerError> {
    let report = standard_project_folder("check", None).with_extension("json");
    run_check_at_until(job, run_id, store, binary, &report, limits, stop)
}
fn run_fixed(
    mut command: Command,
    run_id: i64,
    store: &DaemonStore,
    report: &Path,
    report_source: ReportSource,
    limits: RunLimits,
    stop: &AtomicBool,
) -> Result<RunOutcome, RunnerError> {
    let result = command.group_spawn().map_err(RunnerError::from).and_then(|child| {
        let file_report = matches!(report_source, ReportSource::File).then_some(report);
        capture(child, limits, stop, file_report).map_err(RunnerError::from)
    });
    let report_written = result.as_ref().ok().and_then(|captured| match (report_source, captured.truncated) {
        | (ReportSource::Stdout, false) if !captured.stdout.is_empty() => fs::write(report, &captured.stdout).ok().map(|()| report.to_path_buf()),
        | (ReportSource::File, false) => fs::metadata(report)
            .ok()
            .filter(|metadata| metadata.is_file() && metadata.len() <= limits.output_bytes as u64)
            .map(|_| report.to_path_buf()),
        | _ => None,
    });
    let file_oversized = matches!(report_source, ReportSource::File) && report_exceeds_limit(Some(report), limits.output_bytes);
    let cleanup = if matches!(report_source, ReportSource::File) && report_written.is_none() {
        fs::remove_file(report).or_else(|error| match error.kind() {
            | io::ErrorKind::NotFound => Ok(()),
            | _ => Err(error),
        })
    } else {
        Ok(())
    };
    let (state, exit_code, detail, outcome) = match result {
        | Ok(captured) => {
            let captured = Captured {
                report_ready: report_written.is_some(),
                truncated: captured.truncated || file_oversized,
                ..captured
            };
            let exit_code = captured.status.code();
            let timed_out = captured.timed_out;
            let cancelled = captured.cancelled;
            let truncated = captured.truncated;
            let detail = match captured.succeeded() {
                | true => None,
                | false => {
                    let stderr = String::from_utf8_lossy(&captured.stderr).chars().take(4096).collect::<String>();
                    Some(match (stderr.is_empty(), report_written.is_none()) {
                        | (true, true) => "report absent or exceeds limit".to_string(),
                        | _ => stderr,
                    })
                }
            };
            let state = RunState::from(captured);
            (
                state,
                exit_code,
                detail,
                Ok(RunOutcome {
                    exit_code,
                    report_path: report_written.clone(),
                    timed_out,
                    cancelled,
                    truncated,
                }),
            )
        }
        | Err(error) => (RunState::Failed, None, Some(error.to_string()), Err(error)),
    };
    store
        .finish(run_id, state, exit_code, report_written.as_deref(), detail.as_deref())
        .map_err(RunnerError::from)
        .and_then(|()| cleanup.map_err(RunnerError::from))
        .and(outcome)
}
/// Run a claimed gather job through the paired CLI and persist its result.
pub fn run_gather(job: &JobConfig, run_id: i64, store: &DaemonStore, binary: &Path, limits: RunLimits) -> Result<RunOutcome, RunnerError> {
    let report = standard_project_folder("gather", None).with_extension("json");
    run_gather_at(job, run_id, store, binary, &report, limits)
}
pub(crate) fn run_gather_at(
    job: &JobConfig,
    run_id: i64,
    store: &DaemonStore,
    binary: &Path,
    report: &Path,
    limits: RunLimits,
) -> Result<RunOutcome, RunnerError> {
    run_gather_at_until(job, run_id, store, binary, report, limits, &AtomicBool::new(false))
}
fn run_gather_at_until(
    job: &JobConfig,
    run_id: i64,
    store: &DaemonStore,
    binary: &Path,
    report: &Path,
    limits: RunLimits,
    stop: &AtomicBool,
) -> Result<RunOutcome, RunnerError> {
    match report.starts_with(job.root()) {
        | true => store
            .finish(run_id, RunState::Failed, None, None, Some("report path is inside the watched project"))
            .map_err(RunnerError::from)
            .and(Err(RunnerError::ReportInsideProject)),
        | false => {
            let input = job.root().join(job.input());
            let mut command = Command::new(binary);
            command
                .arg("gather")
                .arg(&input)
                .arg("--output")
                .arg(report)
                .args(["--format", "json"])
                .current_dir(job.root())
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            run_fixed(command, run_id, store, report, ReportSource::File, limits, stop)
        }
    }
}
/// Run a gather job with a shared shutdown signal.
pub fn run_gather_until(
    job: &JobConfig,
    run_id: i64,
    store: &DaemonStore,
    binary: &Path,
    limits: RunLimits,
    stop: &AtomicBool,
) -> Result<RunOutcome, RunnerError> {
    let report = standard_project_folder("gather", None).with_extension("json");
    run_gather_at_until(job, run_id, store, binary, &report, limits, stop)
}
