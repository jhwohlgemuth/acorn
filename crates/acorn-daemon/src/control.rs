//! Authenticated JSON-RPC job control over the foreground scheduler and history store.
use crate::config::{DaemonConfig, JobConfig};
use crate::reload;
use crate::scheduler::{SchedulerError, SchedulerHandle};
use acorn_service::registry::DurableOperations;
use acorn_service::{AcornService, JobId, OperationEffects, OperationRegistry, ServiceError, ServiceErrorKind, ServiceResult, WorkflowAction};
use acorn_store::daemon::{DaemonStore, RunRecord, StoreError};
use alloc::sync::Arc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_HISTORY_LIMIT: usize = 20;
const MAX_HISTORY_LIMIT: usize = 100;

#[derive(JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct AcceptedOutput {
    accepted: bool,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Empty {}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct HistoryInput {
    job_id: JobId,
    limit: Option<usize>,
}
#[derive(JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct HistoryOutput {
    runs: Vec<RunView>,
}
/// Daemon-owned job operations exposed only through the injected server registry.
pub struct JobControl {
    config_path: Arc<PathBuf>,
    initial_config: Arc<DaemonConfig>,
    scheduler: SchedulerHandle,
    store: Arc<DaemonStore>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct JobInput {
    job_id: JobId,
}
#[derive(JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct JobView {
    id: JobId,
    kind: WorkflowAction,
    paused: bool,
}
#[derive(JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct ListOutput {
    jobs: Vec<JobView>,
}
#[derive(JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct PausedOutput {
    paused: bool,
}
#[derive(JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct ReloadOutput {
    generation: u64,
}
#[derive(JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RunView {
    id: i64,
    job_id: String,
    trigger: String,
    state: String,
    queued_at: i64,
    started_at: Option<i64>,
    finished_at: Option<i64>,
    exit_code: Option<i32>,
    report_path: Option<String>,
    detail: Option<String>,
}
#[derive(JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct StatusOutput {
    job: JobView,
    latest: Option<RunView>,
}
impl JobControl {
    /// Bind validated jobs, their durable history, and the running scheduler.
    #[must_use]
    pub fn new(config_path: PathBuf, initial_config: DaemonConfig, scheduler: SchedulerHandle, store: Arc<DaemonStore>) -> Self {
        Self {
            config_path: Arc::new(config_path),
            initial_config: Arc::new(initial_config),
            scheduler,
            store,
        }
    }
    /// Build the portable catalog plus daemon job methods.
    pub fn registry(self) -> ServiceResult<OperationRegistry> {
        let durable: Arc<dyn DurableOperations> = self.store.clone();
        let list_scheduler = self.scheduler.clone();
        let list_store = Arc::clone(&self.store);
        let status_scheduler = self.scheduler.clone();
        let status_store = Arc::clone(&self.store);
        let history_store = Arc::clone(&self.store);
        let run_snapshot = self.scheduler.clone();
        let run_scheduler = self.scheduler.clone();
        let pause_snapshot = self.scheduler.clone();
        let pause_scheduler = self.scheduler.clone();
        let resume_snapshot = self.scheduler.clone();
        let resume_scheduler = self.scheduler.clone();
        let reload_scheduler = self.scheduler;
        let reload_path = Arc::clone(&self.config_path);
        let initial_config = Arc::clone(&self.initial_config);
        let mutation = OperationEffects {
            mutation: true,
            ..OperationEffects::default()
        };
        OperationRegistry::acorn(AcornService, env!("CARGO_PKG_VERSION"), Some(durable))
            .and_then(|registry| {
                registry.register("acorn.jobs.list", OperationEffects::default(), move |_: Empty, _| {
                    let scheduler = list_scheduler.clone();
                    let store = Arc::clone(&list_store);
                    async move {
                        scheduler.snapshot().map_err(scheduler_error).and_then(|snapshot| {
                            snapshot
                                .jobs
                                .iter()
                                .map(|job| job_view(job, &store))
                                .collect::<ServiceResult<Vec<_>>>()
                                .map(|jobs| ListOutput { jobs })
                        })
                    }
                })
            })
            .and_then(|registry| {
                registry.register("acorn.jobs.status", OperationEffects::default(), move |input: JobInput, _| {
                    let scheduler = status_scheduler.clone();
                    let store = Arc::clone(&status_store);
                    async move {
                        scheduler
                            .snapshot()
                            .map_err(scheduler_error)
                            .and_then(|snapshot| configured_job(&snapshot.jobs, &input.job_id).and_then(|job| job_view(job, &store)))
                            .and_then(|job| {
                                store.history(&input.job_id, 1).map_err(storage_error).map(|runs| StatusOutput {
                                    job,
                                    latest: runs.into_iter().next().map(RunView::from),
                                })
                            })
                    }
                })
            })
            .and_then(|registry| {
                registry.register("acorn.jobs.history", OperationEffects::default(), move |input: HistoryInput, _| {
                    let store = Arc::clone(&history_store);
                    async move {
                        history_limit(input.limit)
                            .and_then(|limit| known_job(&store, &input.job_id).map(|()| limit))
                            .and_then(|limit| store.history(&input.job_id, limit).map_err(storage_error))
                            .map(|runs| HistoryOutput {
                                runs: runs.into_iter().map(RunView::from).collect(),
                            })
                    }
                })
            })
            .and_then(|registry| {
                registry.register("acorn.jobs.run", mutation, move |input: JobInput, _| {
                    let snapshot = run_snapshot.clone();
                    let scheduler = run_scheduler.clone();
                    async move {
                        let configured = snapshot
                            .snapshot()
                            .map_err(scheduler_error)
                            .and_then(|snapshot| configured_job(&snapshot.jobs, &input.job_id).map(|_| ()));
                        match configured {
                            | Ok(()) => scheduler
                                .request_run(&input.job_id)
                                .await
                                .map_err(scheduler_error)
                                .map(|accepted| AcceptedOutput { accepted }),
                            | Err(error) => Err(error),
                        }
                    }
                })
            })
            .and_then(|registry| {
                registry.register("acorn.jobs.pause", mutation, move |input: JobInput, _| {
                    let snapshot = pause_snapshot.clone();
                    let scheduler = pause_scheduler.clone();
                    async move {
                        let configured = snapshot
                            .snapshot()
                            .map_err(scheduler_error)
                            .and_then(|snapshot| configured_job(&snapshot.jobs, &input.job_id).map(|_| ()));
                        match configured {
                            | Ok(()) => scheduler
                                .set_paused(&input.job_id, true)
                                .await
                                .map_err(scheduler_error)
                                .and_then(|found| found.then_some(PausedOutput { paused: true }).ok_or_else(|| unknown_job(&input.job_id))),
                            | Err(error) => Err(error),
                        }
                    }
                })
            })
            .and_then(|registry| {
                registry.register("acorn.jobs.resume", mutation, move |input: JobInput, _| {
                    let snapshot = resume_snapshot.clone();
                    let scheduler = resume_scheduler.clone();
                    async move {
                        let configured = snapshot
                            .snapshot()
                            .map_err(scheduler_error)
                            .and_then(|snapshot| configured_job(&snapshot.jobs, &input.job_id).map(|_| ()));
                        match configured {
                            | Ok(()) => scheduler
                                .set_paused(&input.job_id, false)
                                .await
                                .map_err(scheduler_error)
                                .and_then(|found| found.then_some(PausedOutput { paused: false }).ok_or_else(|| unknown_job(&input.job_id))),
                            | Err(error) => Err(error),
                        }
                    }
                })
            })
            .and_then(|registry| {
                registry.register("acorn.jobs.reload", mutation, move |_: Empty, _| {
                    let path = Arc::clone(&reload_path);
                    let baseline = Arc::clone(&initial_config);
                    let scheduler = reload_scheduler.clone();
                    async move {
                        let loaded = tokio::task::spawn_blocking(move || reload::load(&path, &baseline))
                            .await
                            .map_err(|_| ServiceError::new(ServiceErrorKind::Internal, "daemon configuration loader failed"))
                            .and_then(core::convert::identity);
                        match loaded {
                            | Ok(jobs) => scheduler
                                .reload(jobs)
                                .await
                                .map_err(reload_scheduler_error)
                                .map(|generation| ReloadOutput { generation }),
                            | Err(error) => Err(error),
                        }
                    }
                })
            })
    }
}
impl From<RunRecord> for RunView {
    fn from(record: RunRecord) -> Self {
        Self {
            id: record.id,
            job_id: record.job_id,
            trigger: record.trigger,
            state: record.state,
            queued_at: record.queued_at,
            started_at: record.started_at,
            finished_at: record.finished_at,
            exit_code: record.exit_code,
            report_path: record.report_path.map(|path| path.to_string_lossy().into_owned()),
            detail: record.detail,
        }
    }
}
fn configured_job<'a>(jobs: &'a [JobConfig], id: &JobId) -> ServiceResult<&'a JobConfig> {
    jobs.iter().find(|job| job.id() == id).ok_or_else(|| unknown_job(id))
}
fn history_limit(limit: Option<usize>) -> ServiceResult<usize> {
    let limit = limit.unwrap_or(DEFAULT_HISTORY_LIMIT);
    (1..=MAX_HISTORY_LIMIT)
        .contains(&limit)
        .then_some(limit)
        .ok_or_else(|| ServiceError::new(ServiceErrorKind::InvalidInput, "history limit must be between 1 and 100"))
}
fn job_view(job: &JobConfig, store: &DaemonStore) -> ServiceResult<JobView> {
    store.paused(job.id()).map_err(storage_error).and_then(|paused| {
        paused
            .map(|paused| JobView {
                id: job.id().clone(),
                kind: job.kind(),
                paused,
            })
            .ok_or_else(|| unknown_job(job.id()))
    })
}
fn known_job(store: &DaemonStore, id: &JobId) -> ServiceResult<()> {
    store
        .paused(id)
        .map_err(storage_error)
        .and_then(|paused| paused.map(|_| ()).ok_or_else(|| unknown_job(id)))
}
fn reload_scheduler_error(error: SchedulerError) -> ServiceError {
    match error {
        | SchedulerError::Watch(_) => ServiceError::new(ServiceErrorKind::InvalidInput, "daemon input watch could not be prepared"),
        | SchedulerError::UnsupportedKind(_) => ServiceError::new(ServiceErrorKind::InvalidInput, "daemon job action has no fixed runner"),
        | other => scheduler_error(other),
    }
}
fn scheduler_error(_error: SchedulerError) -> ServiceError {
    ServiceError::new(ServiceErrorKind::Internal, "daemon job scheduler is unavailable")
}
fn storage_error(_error: StoreError) -> ServiceError {
    ServiceError::new(ServiceErrorKind::Internal, "daemon history is unavailable")
}
fn unknown_job(id: &JobId) -> ServiceError {
    ServiceError::new(ServiceErrorKind::InvalidInput, format!("unknown daemon job '{id}'"))
}
