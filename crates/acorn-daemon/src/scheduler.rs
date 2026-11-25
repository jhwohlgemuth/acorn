//! Single-worker daemon scheduling with debounced file events and fixed intervals.
use crate::config::JobConfig;
use crate::runner::{run_check_until, run_gather_until, RunLimits, RunnerError};
use acorn_core::time::UnitOfTime;
use acorn_service::{CoalescingState, JobId, JobSchedule, JobTrigger, RunState, WorkflowAction};
use acorn_store::daemon::{DaemonStore, StoreError};
use alloc::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::time::Duration;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::RwLock;
use std::thread::{self, JoinHandle};
use std::time::Instant;
use thiserror::Error;
use tokio::sync::oneshot;

const DEBOUNCE_MS: u64 = 250;
const TICK: Duration = Duration::from_millis(50);
pub(crate) enum Event {
    File(JobId, u64),
    Manual(JobId, oneshot::Sender<Result<bool, SchedulerError>>),
    Pause(JobId, bool, oneshot::Sender<Result<bool, SchedulerError>>),
    Reload(Vec<JobConfig>, oneshot::Sender<Result<u64, SchedulerError>>),
    Finished(i64, JobId, Result<(), RunnerError>),
    Stop,
}
/// Failure to initialize or operate the daemon's one global worker.
#[derive(Debug, Error)]
pub enum SchedulerError {
    /// A configured input could not be watched.
    #[error(transparent)]
    Watch(#[from] notify::Error),
    /// Durable job or run state could not be updated.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// A fixed child process could not be executed or recorded.
    #[error(transparent)]
    Runner(#[from] RunnerError),
    /// A run referenced a job absent from the validated configuration.
    #[error("queued daemon job '{0}' is not configured")]
    UnknownJob(String),
    /// A configured action has no fixed daemon runner.
    #[error("daemon job '{0}' has an unsupported action")]
    UnsupportedKind(String),
    /// The daemon worker is no longer accepting control requests.
    #[error("daemon worker is unavailable")]
    Stopped,
    /// A job execution thread failed to start.
    #[error("failed to start daemon job worker: {0}")]
    WorkerSpawn(#[from] std::io::Error),
    /// A job execution thread panicked.
    #[error("daemon job worker panicked")]
    WorkerPanicked,
    /// The shared job snapshot cannot be read or published.
    #[error("daemon job snapshot lock is poisoned")]
    SnapshotLock,
    /// Job generation cannot be incremented.
    #[error("daemon job generation exhausted")]
    GenerationExhausted,
}
pub(crate) struct ActiveRun {
    pub(crate) id: i64,
    pub(crate) job: JobConfig,
    pub(crate) thread: JoinHandle<()>,
}
struct PreparedReload {
    generation: u64,
    jobs: Vec<JobConfig>,
    slots: BTreeMap<JobId, Slot>,
    startup: Vec<JobId>,
    superseded: Vec<JobId>,
    watch_epoch: u64,
    watchers: Vec<RecommendedWatcher>,
}
pub(crate) struct Runtime {
    pub(crate) active: Option<ActiveRun>,
    pub(crate) generation: u64,
    pub(crate) jobs: Vec<JobConfig>,
    pub(crate) slots: BTreeMap<JobId, Slot>,
    pub(crate) started: Instant,
    pub(crate) watch_epoch: u64,
    pub(crate) watchers: Vec<RecommendedWatcher>,
}
/// Owns the daemon's one global worker and input watchers.
pub struct Scheduler {
    binary: PathBuf,
    jobs: Vec<JobConfig>,
    limits: RunLimits,
    pub(crate) receiver: Receiver<Event>,
    pub(crate) snapshot: Arc<RwLock<SchedulerSnapshot>>,
    stop: Arc<AtomicBool>,
    pub(crate) store: Arc<DaemonStore>,
    sender: Sender<Event>,
    watch_nonce: AtomicU64,
}
/// Control channel for stopping or changing pause state while the worker runs.
#[derive(Clone)]
pub struct SchedulerHandle {
    sender: Sender<Event>,
    snapshot: Arc<RwLock<SchedulerSnapshot>>,
    stop: Arc<AtomicBool>,
}
/// Published job generation used by daemon control reads.
#[derive(Clone, Debug)]
pub struct SchedulerSnapshot {
    /// Monotonic generation of the active job set.
    pub generation: u64,
    /// Validated jobs in configuration order.
    pub jobs: Vec<JobConfig>,
}
#[derive(Clone, Debug)]
pub(crate) struct Slot {
    pub(crate) schedule: JobSchedule,
    pub(crate) state: CoalescingState,
    pub(crate) paused: bool,
    pub(crate) next_interval_ms: Option<u64>,
    pub(crate) file_due_ms: Option<u64>,
    pub(crate) pending_trigger: Option<JobTrigger>,
}
impl Scheduler {
    /// Build a scheduler and a separate control handle for validated jobs.
    #[must_use]
    pub fn new(jobs: Vec<JobConfig>, store: Arc<DaemonStore>, binary: PathBuf, limits: RunLimits) -> (Self, SchedulerHandle) {
        let (sender, receiver) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let snapshot = Arc::new(RwLock::new(SchedulerSnapshot {
            generation: 0,
            jobs: Vec::new(),
        }));
        let handle = SchedulerHandle {
            sender: sender.clone(),
            snapshot: Arc::clone(&snapshot),
            stop: Arc::clone(&stop),
        };
        (
            Self {
                binary,
                jobs,
                limits,
                receiver,
                snapshot,
                stop,
                store,
                sender,
                watch_nonce: AtomicU64::new(0),
            },
            handle,
        )
    }

    /// Run startup, file-change, and interval jobs until the control handle stops the worker.
    pub fn run(self) -> Result<(), SchedulerError> {
        self.run_with_ready(|| {})
    }
    /// Run the worker and notify the caller only after watches and startup runs are ready.
    pub fn run_with_ready(self, ready: impl FnOnce()) -> Result<(), SchedulerError> {
        let unsupported = self
            .jobs
            .iter()
            .find(|job| !matches!(job.kind(), WorkflowAction::Check | WorkflowAction::Gather));
        match unsupported {
            | Some(job) => Err(SchedulerError::UnsupportedKind(job.id().to_string())),
            | None => self
                .store
                .register_jobs(&self.jobs.iter().map(|job| job.id().clone()).collect::<Vec<_>>())
                .map_err(SchedulerError::from)
                .and_then(|()| self.run_registered(ready)),
        }
    }
    fn run_registered(self, ready: impl FnOnce()) -> Result<(), SchedulerError> {
        let started = Instant::now();
        let slots = self
            .jobs
            .iter()
            .map(|job| {
                self.store
                    .paused(job.id())
                    .map(|paused| (job.id().clone(), Slot::new(job.schedule(), paused.unwrap_or(false), 0)))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map_err(SchedulerError::from);
        slots.and_then(|mut slots| {
            self.allocate_watch_epoch()
                .and_then(|watch_epoch| self.watch_inputs(&self.jobs, watch_epoch).map(|watchers| (watch_epoch, watchers)))
                .and_then(|(watch_epoch, watchers)| {
                    self.jobs
                        .iter()
                        .try_for_each(|job| {
                            slots
                                .get_mut(job.id())
                                .map(|slot| slot.trigger(JobTrigger::Startup))
                                .filter(|queued| *queued)
                                .map(|_| self.store.enqueue(job.id(), JobTrigger::Startup).map(|_| ()))
                                .unwrap_or(Ok(()))
                                .map_err(SchedulerError::from)
                        })
                        .and_then(|()| {
                            self.snapshot.write().map_err(|_| SchedulerError::SnapshotLock).map(|mut snapshot| {
                                *snapshot = SchedulerSnapshot {
                                    generation: 1,
                                    jobs: self.jobs.clone(),
                                };
                                Runtime {
                                    active: None,
                                    generation: 1,
                                    jobs: self.jobs.clone(),
                                    slots,
                                    started,
                                    watch_epoch,
                                    watchers,
                                }
                            })
                        })
                        .and_then(|mut runtime| {
                            ready();
                            loop {
                                match self.poll(&mut runtime) {
                                    | Ok(true) => {}
                                    | Ok(false) => break Ok(()),
                                    | Err(error) => break Err(error),
                                }
                            }
                        })
                })
        })
    }
    pub(crate) fn watch_inputs(&self, jobs: &[JobConfig], generation: u64) -> Result<Vec<RecommendedWatcher>, SchedulerError> {
        jobs.iter()
            .filter(|job| job.schedule().on_change())
            .map(|job| {
                let input = job.root().join(job.input());
                let watch_path = if input.is_dir() {
                    input.clone()
                } else {
                    input.parent().unwrap_or(job.root()).to_path_buf()
                };
                let recursive = if input.is_dir() {
                    RecursiveMode::Recursive
                } else {
                    RecursiveMode::NonRecursive
                };
                let sender = self.sender.clone();
                let id = job.id().clone();
                notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
                    if let Ok(event) = result {
                        if event
                            .paths
                            .iter()
                            .any(|path| path == &input || (recursive == RecursiveMode::Recursive && path.starts_with(&input)))
                        {
                            let _ = sender.send(Event::File(id.clone(), generation));
                        }
                    }
                })
                .map_err(SchedulerError::from)
                .and_then(|mut watcher| watcher.watch(&watch_path, recursive).map(|()| watcher).map_err(SchedulerError::from))
            })
            .collect()
    }
    fn allocate_watch_epoch(&self) -> Result<u64, SchedulerError> {
        self.watch_nonce
            .try_update(Ordering::AcqRel, Ordering::Acquire, |value| value.checked_add(1))
            .map(|previous| previous.saturating_add(1))
            .map_err(|_| SchedulerError::GenerationExhausted)
    }
    fn drain_events(&self, runtime: &mut Runtime, now_ms: u64) -> Result<(), SchedulerError> {
        loop {
            match self.receiver.try_recv() {
                | Ok(event) => match self.apply_event(runtime, event, now_ms) {
                    | Ok(()) => {}
                    | Err(error) => break Err(error),
                },
                | Err(_) => break Ok(()),
            }
        }
    }
    pub(crate) fn apply_event(&self, runtime: &mut Runtime, event: Event, now_ms: u64) -> Result<(), SchedulerError> {
        match event {
            | Event::File(job, generation) => {
                if generation == runtime.watch_epoch {
                    if let Some(slot) = runtime.slots.get_mut(&job) {
                        slot.file_event(now_ms);
                    }
                }
                Ok(())
            }
            | Event::Manual(job, reply) => {
                let _ = reply.send(self.manual(runtime, &job));
                Ok(())
            }
            | Event::Pause(job, paused, reply) => {
                let _ = reply.send(self.pause(runtime, &job, paused));
                Ok(())
            }
            | Event::Reload(jobs, reply) => {
                let _ = reply.send(self.apply_reload(runtime, jobs, now_ms));
                Ok(())
            }
            | Event::Finished(id, job, result) => self.finish_one(runtime, id, &job, result),
            | Event::Stop => Ok(()),
        }
    }
    fn tick(&self, runtime: &mut Runtime, now_ms: u64) -> Result<(), SchedulerError> {
        runtime.slots.iter_mut().try_for_each(|(id, slot)| {
            slot.due(now_ms).into_iter().try_for_each(|cause| match slot.trigger(cause) {
                | true => self.store.enqueue(id, cause).map(|_| ()).map_err(SchedulerError::from),
                | false => Ok(()),
            })
        })
    }
    fn poll(&self, runtime: &mut Runtime) -> Result<bool, SchedulerError> {
        let now_ms = elapsed_ms(runtime.started);
        self.drain_events(runtime, now_ms)
            .and_then(|()| match self.stop.load(Ordering::Relaxed) {
                | true => Ok(()),
                | false => self.tick(runtime, now_ms).and_then(|()| self.start_one(runtime)),
            })
            .and_then(|()| match (self.stop.load(Ordering::Relaxed), runtime.active.is_some()) {
                | (true, false) => Ok(false),
                | _ => match self.receiver.recv_timeout(TICK) {
                    | Ok(event) => self.apply_event(runtime, event, elapsed_ms(runtime.started)).map(|()| true),
                    | Err(mpsc::RecvTimeoutError::Timeout) => match runtime.active.as_ref().is_some_and(|active| active.thread.is_finished()) {
                        | true => runtime
                            .active
                            .take()
                            .map(|active| active.thread.join())
                            .map(|_| Err(SchedulerError::WorkerPanicked))
                            .unwrap_or(Ok(true)),
                        | false => Ok(true),
                    },
                    | Err(mpsc::RecvTimeoutError::Disconnected) => Err(SchedulerError::Stopped),
                },
            })
    }
    fn start_one(&self, runtime: &mut Runtime) -> Result<(), SchedulerError> {
        match runtime.active.is_some() {
            | true => Ok(()),
            | false => self.store.claim_next().map_err(SchedulerError::from).and_then(|claimed| match claimed {
                | None => Ok(()),
                | Some(run) => match runtime.jobs.iter().find(|job| job.id().as_str() == run.job_id).cloned() {
                    | None => self
                        .store
                        .finish(run.id, RunState::Failed, None, None, Some("configured job is unavailable"))
                        .map_err(SchedulerError::from)
                        .and(Err(SchedulerError::UnknownJob(run.job_id))),
                    | Some(job) => {
                        let id = job.id().clone();
                        let worker_job = job.clone();
                        let binary = self.binary.clone();
                        let limits = self.limits;
                        let sender = self.sender.clone();
                        let stop = Arc::clone(&self.stop);
                        let store = Arc::clone(&self.store);
                        if let Some(slot) = runtime.slots.get_mut(&id) {
                            slot.start();
                        }
                        thread::Builder::new()
                            .name(format!("acorn-job-{id}"))
                            .spawn(move || {
                                let result = match worker_job.kind() {
                                    | WorkflowAction::Check => run_check_until(&worker_job, run.id, &store, &binary, limits, &stop),
                                    | WorkflowAction::Gather => run_gather_until(&worker_job, run.id, &store, &binary, limits, &stop),
                                    | _ => store
                                        .finish(run.id, RunState::Failed, None, None, Some("unsupported daemon job kind"))
                                        .map_err(RunnerError::from)
                                        .and(Err(RunnerError::UnsupportedKind)),
                                };
                                let _ = sender.send(Event::Finished(run.id, id, result.map(|_| ())));
                            })
                            .map(|thread| {
                                runtime.active = Some(ActiveRun { id: run.id, job, thread });
                            })
                            .map_err(SchedulerError::WorkerSpawn)
                            .or_else(|error| {
                                self.store
                                    .finish(run.id, RunState::Failed, None, None, Some("job worker could not start"))
                                    .map_err(SchedulerError::from)
                                    .and(Err(error))
                            })
                    }
                },
            }),
        }
    }
    pub(crate) fn finish_one(&self, runtime: &mut Runtime, id: i64, job: &JobId, result: Result<(), RunnerError>) -> Result<(), SchedulerError> {
        match runtime.active.take() {
            | Some(active) if active.id == id => active
                .thread
                .join()
                .map_err(|_| SchedulerError::WorkerPanicked)
                .and_then(|()| {
                    runtime
                        .slots
                        .get_mut(job)
                        .and_then(Slot::finish)
                        .map(|cause| self.store.enqueue(job, cause).map(|_| ()))
                        .unwrap_or(Ok(()))
                        .map_err(SchedulerError::from)
                })
                .and_then(|()| match result {
                    | Ok(()) | Err(RunnerError::Io(_) | RunnerError::ReportInsideProject | RunnerError::UnsupportedKind) => Ok(()),
                    | Err(error) => Err(SchedulerError::Runner(error)),
                }),
            | Some(active) => {
                runtime.active = Some(active);
                Err(SchedulerError::WorkerPanicked)
            }
            | None => Err(SchedulerError::WorkerPanicked),
        }
    }
    fn manual(&self, runtime: &mut Runtime, job: &JobId) -> Result<bool, SchedulerError> {
        match runtime.slots.get_mut(job) {
            | None => Ok(false),
            | Some(slot) if slot.paused => Ok(false),
            | Some(slot) => {
                let previous = slot.clone();
                let queued = slot.trigger(JobTrigger::Manual);
                let result = match queued {
                    | true => self
                        .store
                        .enqueue(job, JobTrigger::Manual)
                        .map(|id| id.is_some())
                        .map_err(SchedulerError::from),
                    | false => Ok(true),
                };
                match result {
                    | Ok(true) => Ok(true),
                    | Ok(false) => {
                        *slot = previous;
                        Ok(false)
                    }
                    | Err(error) => {
                        *slot = previous;
                        Err(error)
                    }
                }
            }
        }
    }
    fn pause(&self, runtime: &mut Runtime, job: &JobId, paused: bool) -> Result<bool, SchedulerError> {
        match runtime.slots.get_mut(job) {
            | None => Ok(false),
            | Some(slot) => self.store.set_paused(job, paused).map_err(SchedulerError::from).inspect(|found| {
                if *found {
                    slot.paused = paused;
                    if paused {
                        slot.file_due_ms = None;
                    }
                }
            }),
        }
    }
    fn apply_reload(&self, runtime: &mut Runtime, jobs: Vec<JobConfig>, now_ms: u64) -> Result<u64, SchedulerError> {
        self.apply_reload_with(runtime, jobs, now_ms, |jobs, watch_epoch| self.watch_inputs(jobs, watch_epoch))
    }
    pub(crate) fn apply_reload_with(
        &self,
        runtime: &mut Runtime,
        jobs: Vec<JobConfig>,
        now_ms: u64,
        prepare_watchers: impl FnOnce(&[JobConfig], u64) -> Result<Vec<RecommendedWatcher>, SchedulerError>,
    ) -> Result<u64, SchedulerError> {
        match jobs == runtime.jobs {
            | true => Ok(runtime.generation),
            | false => self.prepare_reload(runtime, jobs, now_ms, prepare_watchers).and_then(|prepared| {
                self.snapshot.write().map_err(|_| SchedulerError::SnapshotLock).and_then(|mut snapshot| {
                    let ids = prepared.jobs.iter().map(|job| job.id().clone()).collect::<Vec<_>>();
                    self.store
                        .activate_jobs(&ids, &prepared.superseded, &prepared.startup)
                        .map_err(SchedulerError::from)
                        .map(|()| {
                            runtime.generation = prepared.generation;
                            runtime.jobs = prepared.jobs;
                            runtime.slots = prepared.slots;
                            runtime.watch_epoch = prepared.watch_epoch;
                            runtime.watchers = prepared.watchers;
                            *snapshot = SchedulerSnapshot {
                                generation: runtime.generation,
                                jobs: runtime.jobs.clone(),
                            };
                            runtime.generation
                        })
                })
            }),
        }
    }
    fn prepare_reload(
        &self,
        runtime: &Runtime,
        jobs: Vec<JobConfig>,
        now_ms: u64,
        prepare_watchers: impl FnOnce(&[JobConfig], u64) -> Result<Vec<RecommendedWatcher>, SchedulerError>,
    ) -> Result<PreparedReload, SchedulerError> {
        let supported = jobs
            .iter()
            .find(|job| !matches!(job.kind(), WorkflowAction::Check | WorkflowAction::Gather))
            .map(|job| Err(SchedulerError::UnsupportedKind(job.id().to_string())))
            .unwrap_or(Ok(()));
        supported
            .and_then(|()| runtime.generation.checked_add(1).ok_or(SchedulerError::GenerationExhausted))
            .and_then(|generation| {
                self.allocate_watch_epoch()
                    .and_then(|watch_epoch| prepare_watchers(&jobs, watch_epoch).map(|watchers| (generation, watch_epoch, watchers)))
            })
            .and_then(|(generation, watch_epoch, watchers)| {
                jobs.iter()
                    .try_fold((BTreeMap::new(), Vec::new()), |(mut slots, mut startup), job| {
                        self.store.paused(job.id()).map_err(SchedulerError::from).map(|paused| {
                            let paused = paused.unwrap_or(false);
                            let unchanged = runtime.jobs.iter().any(|old| old == job);
                            let active = runtime.active.as_ref().is_some_and(|run| run.job.id() == job.id());
                            let slot = match unchanged {
                                | true => runtime
                                    .slots
                                    .get(job.id())
                                    .cloned()
                                    .unwrap_or_else(|| Slot::new(job.schedule(), paused, now_ms)),
                                | false => {
                                    let mut slot = Slot::new(job.schedule(), paused, now_ms);
                                    match (active, paused) {
                                        | (true, false) => {
                                            slot.state = CoalescingState::RunningPending;
                                            slot.pending_trigger = Some(JobTrigger::Startup);
                                        }
                                        | (true, true) => slot.state = CoalescingState::Running,
                                        | (false, false) => {
                                            slot.trigger(JobTrigger::Startup);
                                            startup.push(job.id().clone());
                                        }
                                        | (false, true) => {}
                                    }
                                    slot
                                }
                            };
                            slots.insert(job.id().clone(), slot);
                            (slots, startup)
                        })
                    })
                    .map(|(slots, startup)| {
                        let superseded = runtime
                            .jobs
                            .iter()
                            .filter(|old| !jobs.iter().any(|job| job == *old))
                            .map(|job| job.id().clone())
                            .collect();
                        PreparedReload {
                            generation,
                            jobs,
                            slots,
                            startup,
                            superseded,
                            watch_epoch,
                            watchers,
                        }
                    })
            })
    }
}
impl SchedulerHandle {
    /// Return the active immutable job generation for control reads.
    pub fn snapshot(&self) -> Result<SchedulerSnapshot, SchedulerError> {
        self.snapshot
            .read()
            .map(|snapshot| snapshot.clone())
            .map_err(|_| SchedulerError::SnapshotLock)
    }
    /// Submit one coalesced manual trigger for an enabled job.
    pub async fn request_run(&self, job: &JobId) -> Result<bool, SchedulerError> {
        let (reply, received) = oneshot::channel();
        match self.sender.send(Event::Manual(job.clone(), reply)) {
            | Ok(()) => received.await.map_err(|_| SchedulerError::Stopped).and_then(core::convert::identity),
            | Err(_) => Err(SchedulerError::Stopped),
        }
    }
    /// Persist and notify the worker of a job pause or resume request.
    pub async fn set_paused(&self, job: &JobId, paused: bool) -> Result<bool, SchedulerError> {
        let (reply, received) = oneshot::channel();
        match self.sender.send(Event::Pause(job.clone(), paused, reply)) {
            | Ok(()) => received.await.map_err(|_| SchedulerError::Stopped).and_then(core::convert::identity),
            | Err(_) => Err(SchedulerError::Stopped),
        }
    }
    /// Publish a fully validated replacement job set and return its generation.
    pub async fn reload(&self, jobs: Vec<JobConfig>) -> Result<u64, SchedulerError> {
        let (reply, received) = oneshot::channel();
        match self.sender.send(Event::Reload(jobs, reply)) {
            | Ok(()) => received.await.map_err(|_| SchedulerError::Stopped).and_then(core::convert::identity),
            | Err(_) => Err(SchedulerError::Stopped),
        }
    }
    /// Request an orderly worker shutdown, cancelling an active child.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.sender.send(Event::Stop);
    }
}
impl Slot {
    pub(crate) fn new(schedule: JobSchedule, paused: bool, now_ms: u64) -> Self {
        Self {
            schedule,
            state: CoalescingState::Idle,
            paused,
            next_interval_ms: schedule
                .every_seconds()
                .map(|seconds| now_ms.saturating_add(seconds.get().saturating_mul(1000))),
            file_due_ms: None,
            pending_trigger: None,
        }
    }

    pub(crate) fn file_event(&mut self, now_ms: u64) {
        if self.schedule.on_change() && !self.paused {
            self.file_due_ms = Some(now_ms.saturating_add(DEBOUNCE_MS));
        }
    }

    pub(crate) fn due(&mut self, now_ms: u64) -> Vec<JobTrigger> {
        let interval = self.next_interval_ms.filter(|deadline| now_ms >= *deadline).map(|_| {
            self.next_interval_ms = self
                .schedule
                .every_seconds()
                .map(|seconds| now_ms.saturating_add(seconds.get().saturating_mul(1000)));
            JobTrigger::Interval
        });
        let file = self.file_due_ms.filter(|deadline| now_ms >= *deadline).map(|_| {
            self.file_due_ms = None;
            JobTrigger::FileChange
        });
        if self.paused {
            Vec::new()
        } else {
            interval.into_iter().chain(file).collect()
        }
    }
    pub(crate) fn trigger(&mut self, cause: JobTrigger) -> bool {
        match (self.paused, self.state) {
            | (true, _) | (false, CoalescingState::Queued) => false,
            | (false, CoalescingState::Idle) => {
                self.state = self.state.trigger();
                true
            }
            | (false, CoalescingState::Running | CoalescingState::RunningPending) => {
                self.state = self.state.trigger();
                self.pending_trigger.get_or_insert(cause);
                false
            }
        }
    }
    pub(crate) fn start(&mut self) {
        if let Ok(state) = self.state.start() {
            self.state = state;
        }
    }
    pub(crate) fn finish(&mut self) -> Option<JobTrigger> {
        match self.state.finish() {
            | Ok(state) => {
                self.state = state;
                if state == CoalescingState::Queued && !self.paused {
                    self.pending_trigger.take()
                } else {
                    if self.paused {
                        self.state = CoalescingState::Idle;
                    }
                    self.pending_trigger = None;
                    None
                }
            }
            | Err(_) => None,
        }
    }
}
fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}
