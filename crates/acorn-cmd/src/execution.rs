//! Managed native process execution.
use alloc::{collections::VecDeque, sync::Arc};
use command_group::{CommandGroup, GroupChild};
use core::{
    fmt,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::{
    io::{self, Read},
    process::{ChildStderr, ChildStdout, Command, ExitStatus, Stdio},
    sync::{Condvar, Mutex},
    thread::{self, JoinHandle},
    time::Instant,
};
use thiserror::Error;

const DEFAULT_MAX_BYTES: usize = 1024 * 1024;
const DEFAULT_MAX_LINES: usize = 10_000;
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const READ_BUFFER_SIZE: usize = 8192;

/// Receives attributed process-output chunks while a command is running.
pub trait OutputSink: Send + Sync {
    /// Handle one stdout or stderr byte chunk.
    fn write(&self, stream: OutputStream, bytes: &[u8]) -> io::Result<()>;
}

/// Failure returned by managed command execution.
#[derive(Debug, Error)]
pub enum CommandError {
    /// Execution was cancelled when checked outcome semantics were requested.
    #[error("command execution was cancelled")]
    Cancelled(Box<ExecutionResult>),
    /// Reading a process-output stream failed.
    #[error("failed to capture {stream:?}: {source}")]
    Capture {
        /// Execution state available after cleanup.
        context: Box<ExecutionContext>,
        /// Underlying read or reader-thread failure.
        #[source]
        source: io::Error,
        /// Stream whose reader failed.
        stream: OutputStream,
    },
    /// Process-group cleanup failed.
    #[error("failed to clean up managed process group: {source}")]
    Cleanup {
        /// Execution state available after cleanup was attempted.
        context: Box<ExecutionContext>,
        /// Underlying cleanup failure.
        #[source]
        source: io::Error,
    },
    /// An execution option could not be represented by the host clock.
    #[error("invalid execution option `{option}`: {message}")]
    InvalidOption {
        /// Invalid option name.
        option: &'static str,
        /// Validation detail.
        message: String,
    },
    /// The process leader exited unsuccessfully when checked outcome semantics were requested.
    #[error("command exited unsuccessfully")]
    NonZeroExit(Box<ExecutionResult>),
    /// The process group could not be created.
    #[error("failed to spawn managed command: {source}")]
    Spawn {
        /// Underlying spawn failure.
        #[source]
        source: io::Error,
    },
    /// An output sink rejected a process-output chunk.
    #[error("output sink rejected {stream:?}: {source}")]
    Sink {
        /// Execution state available after cleanup.
        context: Box<ExecutionContext>,
        /// Underlying sink failure.
        #[source]
        source: io::Error,
        /// Stream whose sink call failed.
        stream: OutputStream,
    },
    /// Execution timed out when checked outcome semantics were requested.
    #[error("command execution timed out")]
    TimedOut(Box<ExecutionResult>),
    /// Waiting for the process leader failed.
    #[error("failed to wait for managed command: {source}")]
    Wait {
        /// Execution state available after cleanup.
        context: Box<ExecutionContext>,
        /// Underlying wait failure.
        #[source]
        source: io::Error,
    },
}
/// Identifies a captured process-output stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputStream {
    /// Standard error.
    Stderr,
    /// Standard output.
    Stdout,
}
/// Describes why managed process execution ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessTermination {
    /// A caller cancelled execution.
    Cancelled,
    /// The process leader exited before cancellation or timeout was observed.
    Exited,
    /// The configured timeout elapsed.
    TimedOut,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReaderFailureKind {
    Capture,
    Sink,
}
enum Trigger {
    Cancelled,
    Exited(ExitStatus),
    ReaderFailure,
    TimedOut,
    Wait(io::Error),
}
#[derive(Debug, Default)]
struct CancellationState {
    cancelled: AtomicBool,
    changed: Condvar,
    lock: Mutex<()>,
}
/// Cheap, cooperative cancellation shared between execution threads.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    state: Arc<CancellationState>,
}
/// Per-stream memory limits for managed output capture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureLimits {
    max_bytes: Option<usize>,
    max_lines: Option<usize>,
}
/// Raw retained process output and exact truncation metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CapturedOutput {
    bytes: Vec<u8>,
    truncated_bytes: u64,
    truncated_lines: u64,
}
/// Partial execution state retained when an operational failure occurs.
#[derive(Debug)]
pub struct ExecutionContext {
    duration: Duration,
    status: Option<ProcessStatus>,
    stderr: CapturedOutput,
    stdout: CapturedOutput,
}
/// Options controlling one managed command execution.
#[derive(Clone, Default)]
pub struct ExecutionOptions {
    cancellation: CancellationToken,
    capture_limits: CaptureLimits,
    sink: Option<Arc<dyn OutputSink>>,
    timeout: Option<Duration>,
}
/// Complete outcome of managed command execution.
#[derive(Debug)]
pub struct ExecutionResult {
    duration: Duration,
    status: Option<ProcessStatus>,
    stderr: CapturedOutput,
    stdout: CapturedOutput,
    termination: ProcessTermination,
}
/// Platform process-leader status preserved after reaping.
#[derive(Clone, Copy, Debug)]
pub struct ProcessStatus {
    inner: ExitStatus,
}
#[derive(Debug)]
struct ReaderFailure {
    kind: ReaderFailureKind,
    source: io::Error,
}
#[derive(Debug)]
struct ReaderOutcome {
    captured: CapturedOutput,
    failure: Option<ReaderFailure>,
}
struct ReaderTask {
    handle: Option<JoinHandle<ReaderOutcome>>,
    outcome: Option<ReaderOutcome>,
}
struct Supervisor {
    active: bool,
    child: GroupChild,
    started: Instant,
    stderr: ReaderTask,
    stdout: ReaderTask,
}
struct TailCapture {
    active_line_starts: VecDeque<u64>,
    bytes: VecDeque<u8>,
    limits: CaptureLimits,
    line_limit_starts: VecDeque<u64>,
    previous_newline: bool,
    retained_start: u64,
    seen_bytes: u64,
    total_lines: u64,
    truncated_lines: u64,
}
impl CancellationToken {
    /// Create an uncancelled token.
    pub fn new() -> Self {
        Self::default()
    }
    /// Cancel this token and wake executions waiting on it.
    pub fn cancel(&self) {
        self.state.cancelled.store(true, Ordering::Release);
        self.state.changed.notify_all();
    }
    /// Return whether cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.state.cancelled.load(Ordering::Acquire)
    }
    fn wait(&self, duration: Duration) {
        let guard = match self.state.lock.lock() {
            | Ok(guard) => guard,
            | Err(poisoned) => poisoned.into_inner(),
        };
        if !self.is_cancelled() {
            let _wait = self.state.changed.wait_timeout(guard, duration);
        }
    }
}
impl CaptureLimits {
    /// Create capture limits applied independently to stdout and stderr.
    pub const fn new(max_bytes: Option<usize>, max_lines: Option<usize>) -> Self {
        Self { max_bytes, max_lines }
    }
    /// Disable both capture ceilings.
    pub const fn unlimited() -> Self {
        Self::new(None, None)
    }
    /// Return the maximum retained bytes per stream.
    pub const fn max_bytes(&self) -> Option<usize> {
        self.max_bytes
    }
    /// Return the maximum retained logical lines per stream.
    pub const fn max_lines(&self) -> Option<usize> {
        self.max_lines
    }
}
impl Default for CaptureLimits {
    fn default() -> Self {
        Self::new(Some(DEFAULT_MAX_BYTES), Some(DEFAULT_MAX_LINES))
    }
}
impl CapturedOutput {
    /// Borrow the retained raw bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Consume the capture and return its retained raw bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
    /// Return the number of leading bytes omitted from the retained tail.
    pub const fn truncated_bytes(&self) -> u64 {
        self.truncated_bytes
    }
    /// Return the number of logical lines wholly or partially omitted from the retained tail.
    pub const fn truncated_lines(&self) -> u64 {
        self.truncated_lines
    }
}
impl CommandError {
    /// Borrow complete execution output carried by a checked outcome error.
    pub fn result(&self) -> Option<&ExecutionResult> {
        match self {
            | Self::Cancelled(result) | Self::NonZeroExit(result) | Self::TimedOut(result) => Some(result),
            | _ => None,
        }
    }
    /// Borrow partial execution state carried by an operational error.
    pub fn context(&self) -> Option<&ExecutionContext> {
        match self {
            | Self::Capture { context, .. } | Self::Cleanup { context, .. } | Self::Sink { context, .. } | Self::Wait { context, .. } => {
                Some(context)
            }
            | _ => None,
        }
    }
}
impl ExecutionContext {
    /// Return elapsed spawn-to-cleanup duration.
    pub const fn duration(&self) -> Duration {
        self.duration
    }
    /// Borrow the process-leader status when one was recovered.
    pub const fn status(&self) -> Option<&ProcessStatus> {
        self.status.as_ref()
    }
    /// Borrow captured stderr.
    pub const fn stderr(&self) -> &CapturedOutput {
        &self.stderr
    }
    /// Borrow captured stdout.
    pub const fn stdout(&self) -> &CapturedOutput {
        &self.stdout
    }
}
impl fmt::Debug for ExecutionOptions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ExecutionOptions")
            .field("cancellation", &self.cancellation)
            .field("capture_limits", &self.capture_limits)
            .field("sink", &self.sink.is_some())
            .field("timeout", &self.timeout)
            .finish()
    }
}
impl ExecutionOptions {
    /// Create default managed-execution options.
    pub fn new() -> Self {
        Self::default()
    }
    /// Borrow the cancellation token.
    pub const fn cancellation(&self) -> &CancellationToken {
        &self.cancellation
    }
    /// Return the capture limits.
    pub const fn capture_limits(&self) -> CaptureLimits {
        self.capture_limits
    }
    /// Return the optional timeout.
    pub const fn timeout(&self) -> Option<Duration> {
        self.timeout
    }
    /// Replace the cancellation token.
    pub fn with_cancellation(mut self, cancellation: CancellationToken) -> Self {
        self.cancellation = cancellation;
        self
    }
    /// Replace the per-stream capture limits.
    pub const fn with_capture_limits(mut self, capture_limits: CaptureLimits) -> Self {
        self.capture_limits = capture_limits;
        self
    }
    /// Stream output chunks to a shared callback.
    pub fn with_sink(mut self, sink: impl OutputSink + 'static) -> Self {
        self.sink = Some(Arc::new(sink));
        self
    }
    /// Set the spawn-to-reap timeout.
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}
impl ExecutionResult {
    /// Return elapsed spawn-to-reap duration.
    pub const fn duration(&self) -> Duration {
        self.duration
    }
    /// Borrow the process-leader status, absent only for pre-spawn cancellation.
    pub const fn status(&self) -> Option<&ProcessStatus> {
        self.status.as_ref()
    }
    /// Borrow captured stderr.
    pub const fn stderr(&self) -> &CapturedOutput {
        &self.stderr
    }
    /// Borrow captured stdout.
    pub const fn stdout(&self) -> &CapturedOutput {
        &self.stdout
    }
    /// Return why execution ended.
    pub const fn termination(&self) -> ProcessTermination {
        self.termination
    }
    pub(crate) fn into_output(self) -> Option<std::process::Output> {
        self.status.map(|status| std::process::Output {
            status: status.inner,
            stderr: self.stderr.bytes,
            stdout: self.stdout.bytes,
        })
    }
}
impl<F> OutputSink for F
where
    F: Fn(OutputStream, &[u8]) -> io::Result<()> + Send + Sync,
{
    fn write(&self, stream: OutputStream, bytes: &[u8]) -> io::Result<()> {
        self(stream, bytes)
    }
}
impl ProcessStatus {
    /// Return the platform exit code, or `None` when terminated by a Unix signal.
    pub fn code(&self) -> Option<i32> {
        self.inner.code()
    }
    /// Return the terminating Unix signal when available.
    pub fn signal(&self) -> Option<i32> {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;

            self.inner.signal()
        }
        #[cfg(not(unix))]
        {
            None
        }
    }
    /// Return whether the process leader exited successfully.
    pub fn success(&self) -> bool {
        self.inner.success()
    }
}
impl From<ExitStatus> for ProcessStatus {
    fn from(inner: ExitStatus) -> Self {
        Self { inner }
    }
}
impl ReaderOutcome {
    fn capture(captured: CapturedOutput, source: io::Error) -> Self {
        Self {
            captured,
            failure: Some(ReaderFailure {
                kind: ReaderFailureKind::Capture,
                source,
            }),
        }
    }
    fn success(captured: CapturedOutput) -> Self {
        Self { captured, failure: None }
    }
}
impl ReaderTask {
    fn new<R>(reader: R, stream: OutputStream, limits: CaptureLimits, sink: Option<Arc<dyn OutputSink>>) -> Self
    where
        R: Read + Send + 'static,
    {
        Self {
            handle: Some(thread::spawn(move || drain(reader, stream, limits, sink.as_deref()))),
            outcome: None,
        }
    }
    fn finish(&mut self) {
        if self.outcome.is_none() {
            self.outcome = self.handle.take().map(|handle| match handle.join() {
                | Ok(outcome) => outcome,
                | Err(_) => ReaderOutcome::capture(CapturedOutput::default(), io::Error::other("process output reader panicked")),
            });
        }
    }
    fn has_failure(&self) -> bool {
        self.outcome.as_ref().and_then(|outcome| outcome.failure.as_ref()).is_some()
    }
    fn poll(&mut self) {
        if self.handle.as_ref().is_some_and(JoinHandle::is_finished) {
            self.finish();
        }
    }
    fn take_outcome(&mut self) -> ReaderOutcome {
        self.finish();
        self.outcome
            .take()
            .unwrap_or_else(|| ReaderOutcome::capture(CapturedOutput::default(), io::Error::other("process output reader was unavailable")))
    }
}
impl Supervisor {
    fn complete(mut self, trigger: Trigger) -> Result<ExecutionResult, CommandError> {
        let natural_status = match trigger {
            | Trigger::Exited(status) => Some(status),
            | _ => None,
        };
        let kill_error = self.child.kill().err().filter(|error| !process_is_gone(error));
        let waited = self.child.wait();
        self.stderr.finish();
        self.stdout.finish();
        self.active = false;
        let stderr = self.stderr.take_outcome();
        let stdout = self.stdout.take_outcome();
        let status = natural_status.or_else(|| waited.as_ref().ok().copied()).map(ProcessStatus::from);
        let duration = self.started.elapsed();
        let context = || {
            Box::new(ExecutionContext {
                duration,
                status,
                stderr: stderr.captured.clone(),
                stdout: stdout.captured.clone(),
            })
        };
        if let Some(source) = kill_error {
            Err(CommandError::Cleanup { context: context(), source })
        } else if let Err(source) = waited {
            Err(CommandError::Wait { context: context(), source })
        } else if let Some(failure) = stdout.failure {
            match failure.kind {
                | ReaderFailureKind::Capture => Err(CommandError::Capture {
                    context: context(),
                    source: failure.source,
                    stream: OutputStream::Stdout,
                }),
                | ReaderFailureKind::Sink => Err(CommandError::Sink {
                    context: context(),
                    source: failure.source,
                    stream: OutputStream::Stdout,
                }),
            }
        } else if let Some(failure) = stderr.failure {
            match failure.kind {
                | ReaderFailureKind::Capture => Err(CommandError::Capture {
                    context: context(),
                    source: failure.source,
                    stream: OutputStream::Stderr,
                }),
                | ReaderFailureKind::Sink => Err(CommandError::Sink {
                    context: context(),
                    source: failure.source,
                    stream: OutputStream::Stderr,
                }),
            }
        } else if let Trigger::Wait(source) = trigger {
            Err(CommandError::Wait { context: context(), source })
        } else {
            Ok(ExecutionResult {
                duration,
                status,
                stderr: stderr.captured,
                stdout: stdout.captured,
                termination: match trigger {
                    | Trigger::Cancelled => ProcessTermination::Cancelled,
                    | Trigger::Exited(_) | Trigger::ReaderFailure | Trigger::Wait(_) => ProcessTermination::Exited,
                    | Trigger::TimedOut => ProcessTermination::TimedOut,
                },
            })
        }
    }
    fn readers_failed(&self) -> bool {
        self.stderr.has_failure() || self.stdout.has_failure()
    }
    fn poll_readers(&mut self) {
        self.stderr.poll();
        self.stdout.poll();
    }
}
impl Drop for Supervisor {
    fn drop(&mut self) {
        if self.active {
            let _kill = self.child.kill();
            let _wait = self.child.wait();
            self.stderr.finish();
            self.stdout.finish();
        }
    }
}
impl TailCapture {
    fn new(limits: CaptureLimits) -> Self {
        Self {
            active_line_starts: VecDeque::new(),
            bytes: VecDeque::new(),
            limits,
            line_limit_starts: VecDeque::new(),
            previous_newline: false,
            retained_start: 0,
            seen_bytes: 0,
            total_lines: 0,
            truncated_lines: 0,
        }
    }
    fn finish(self) -> CapturedOutput {
        CapturedOutput {
            bytes: self.bytes.into_iter().collect(),
            truncated_bytes: self.retained_start,
            truncated_lines: self.truncated_lines,
        }
    }
    fn push(&mut self, chunk: &[u8]) -> io::Result<()> {
        chunk.iter().try_for_each(|byte| self.push_byte(*byte))
    }
    fn push_byte(&mut self, byte: u8) -> io::Result<()> {
        let line_start = self.seen_bytes == 0 || self.previous_newline;
        if line_start {
            self.total_lines = match self.total_lines.checked_add(1) {
                | Some(total) => total,
                | None => return Err(io::Error::other("process output line count exceeded u64")),
            };
            if self.limits.max_bytes.is_some() || self.limits.max_lines.is_some() {
                self.active_line_starts.push_back(self.seen_bytes);
            }
            if let Some(max_lines) = self.limits.max_lines {
                self.line_limit_starts.push_back(self.seen_bytes);
                while self.line_limit_starts.len() > max_lines {
                    self.line_limit_starts.pop_front();
                }
            }
        }
        self.bytes.push_back(byte);
        self.seen_bytes = match self.seen_bytes.checked_add(1) {
            | Some(total) => total,
            | None => return Err(io::Error::other("process output byte count exceeded u64")),
        };
        self.previous_newline = byte == b'\n';
        let byte_start = self
            .limits
            .max_bytes
            .map(|limit| self.seen_bytes.saturating_sub(u64::try_from(limit).unwrap_or(u64::MAX)))
            .unwrap_or_default();
        let line_start = match self.limits.max_lines {
            | Some(0) => self.seen_bytes,
            | Some(limit) if self.total_lines > u64::try_from(limit).unwrap_or(u64::MAX) => {
                self.line_limit_starts.front().copied().unwrap_or(self.seen_bytes)
            }
            | _ => 0,
        };
        let retained_start = self.retained_start.max(byte_start).max(line_start);
        let dropped = retained_start.saturating_sub(self.retained_start);
        let dropped = usize::try_from(dropped).unwrap_or(usize::MAX).min(self.bytes.len());
        self.bytes.drain(..dropped);
        self.retained_start = retained_start;
        while self.active_line_starts.front().is_some_and(|start| *start < retained_start) {
            self.active_line_starts.pop_front();
            self.truncated_lines = match self.truncated_lines.checked_add(1) {
                | Some(total) => total,
                | None => return Err(io::Error::other("truncated process output line count exceeded u64")),
            };
        }
        Ok(())
    }
}
fn drain<R>(mut reader: R, stream: OutputStream, limits: CaptureLimits, sink: Option<&dyn OutputSink>) -> ReaderOutcome
where
    R: Read,
{
    let mut capture = TailCapture::new(limits);
    let mut buffer = [0_u8; READ_BUFFER_SIZE];
    loop {
        match reader.read(&mut buffer) {
            | Ok(0) => break ReaderOutcome::success(capture.finish()),
            | Ok(read) => {
                let Some(chunk) = buffer.get(..read) else {
                    let source = io::Error::other("process-output reader returned an invalid byte count");
                    break ReaderOutcome::capture(capture.finish(), source);
                };
                if let Err(source) = capture.push(chunk) {
                    break ReaderOutcome::capture(capture.finish(), source);
                }
                if let Some(sink) = sink {
                    if let Err(source) = sink.write(stream, chunk) {
                        break ReaderOutcome {
                            captured: capture.finish(),
                            failure: Some(ReaderFailure {
                                kind: ReaderFailureKind::Sink,
                                source,
                            }),
                        };
                    }
                }
            }
            | Err(source) => break ReaderOutcome::capture(capture.finish(), source),
        }
    }
}
pub(crate) fn execute(command: &mut Command, options: &ExecutionOptions) -> Result<ExecutionResult, CommandError> {
    if options.cancellation.is_cancelled() {
        return Ok(ExecutionResult {
            duration: Duration::ZERO,
            status: None,
            stderr: CapturedOutput::default(),
            stdout: CapturedOutput::default(),
            termination: ProcessTermination::Cancelled,
        });
    }
    let started = Instant::now();
    let deadline = match options.timeout {
        | Some(timeout) => match started.checked_add(timeout) {
            | Some(deadline) => Some(deadline),
            | None => {
                return Err(CommandError::InvalidOption {
                    option: "timeout",
                    message: "duration exceeds the host monotonic clock range".to_string(),
                });
            }
        },
        | None => None,
    };
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    command
        .group_spawn()
        .map_err(|source| CommandError::Spawn { source })
        .and_then(|mut child| {
            let stdout = child.inner().stdout.take();
            let stderr = child.inner().stderr.take();
            match (stdout, stderr) {
                | (Some(stdout), Some(stderr)) => supervise(child, stdout, stderr, options, started, deadline),
                | _ => {
                    let supervisor = Supervisor {
                        active: true,
                        child,
                        started,
                        stderr: ReaderTask::new(io::empty(), OutputStream::Stderr, options.capture_limits, options.sink.clone()),
                        stdout: ReaderTask::new(io::empty(), OutputStream::Stdout, options.capture_limits, options.sink.clone()),
                    };
                    let source = io::Error::other("managed command did not expose both output pipes");
                    supervisor.complete(Trigger::Wait(source))
                }
            }
        })
}
fn process_is_gone(error: &io::Error) -> bool {
    matches!(error.kind(), io::ErrorKind::InvalidInput | io::ErrorKind::NotFound) || cfg!(unix) && error.raw_os_error() == Some(3)
}
fn supervise(
    child: GroupChild,
    stdout: ChildStdout,
    stderr: ChildStderr,
    options: &ExecutionOptions,
    started: Instant,
    deadline: Option<Instant>,
) -> Result<ExecutionResult, CommandError> {
    let mut supervisor = Supervisor {
        active: true,
        child,
        started,
        stderr: ReaderTask::new(stderr, OutputStream::Stderr, options.capture_limits, options.sink.clone()),
        stdout: ReaderTask::new(stdout, OutputStream::Stdout, options.capture_limits, options.sink.clone()),
    };
    let trigger = loop {
        supervisor.poll_readers();
        if supervisor.readers_failed() {
            break Trigger::ReaderFailure;
        }
        match supervisor.child.try_wait() {
            | Ok(Some(status)) => break Trigger::Exited(status),
            | Err(source) => break Trigger::Wait(source),
            | Ok(None) if options.cancellation.is_cancelled() => break Trigger::Cancelled,
            | Ok(None) if deadline.is_some_and(|deadline| Instant::now() >= deadline) => break Trigger::TimedOut,
            | Ok(None) => {
                let wait = deadline
                    .map(|deadline| deadline.saturating_duration_since(Instant::now()).min(POLL_INTERVAL))
                    .unwrap_or(POLL_INTERVAL);
                options.cancellation.wait(wait);
            }
        }
    };
    supervisor.complete(trigger)
}

#[cfg(test)]
mod tests;
