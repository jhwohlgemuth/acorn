//! Versioned SQLite storage for configured daemon jobs and bounded run history.
use acorn_service::registry::DurableOperations;
use acorn_service::{JobId, JobTrigger, RunState, ServiceError, ServiceErrorKind, ServiceResult};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const MAX_HISTORY_AGE_SECONDS: i64 = 30 * 24 * 60 * 60;
const MAX_HISTORY_PER_JOB: i64 = 100;
const SCHEMA_VERSION: i64 = 1;
/// Result of a daemon history operation.
pub type StoreResult<T> = Result<T, StoreError>;
/// Failure to open or update the daemon history database.
#[derive(Debug, Error)]
pub enum StoreError {
    /// The database cannot be used by this version of ACORN.
    #[error("unsupported daemon history schema version {0}")]
    Schema(i64),
    /// SQLite rejected an operation.
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    /// A previous database operation panicked while holding its lock.
    #[error("daemon history database lock is poisoned")]
    Lock,
    /// A requested run was not in the required lifecycle state.
    #[error("daemon run {0} is not active")]
    Inactive(i64),
}
/// Synchronized SQLite adapter shared by the worker and control API.
#[derive(Debug)]
pub struct DaemonStore(Mutex<Connection>);
/// Persisted metadata for one daemon job run.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RunRecord {
    /// Monotonic database identifier used by status and replay.
    pub id: i64,
    /// Stable configured job identifier.
    pub job_id: String,
    /// Cause of the queued run.
    pub trigger: String,
    /// Lifecycle state.
    pub state: String,
    /// Unix timestamp at queue time.
    pub queued_at: i64,
    /// Unix timestamp when claimed by the worker.
    pub started_at: Option<i64>,
    /// Unix timestamp when finished or recovered.
    pub finished_at: Option<i64>,
    /// Child exit status when available.
    pub exit_code: Option<i32>,
    /// Explicit cached report path when available.
    pub report_path: Option<PathBuf>,
    /// Bounded diagnostic retained for failed runs.
    pub detail: Option<String>,
}
impl DaemonStore {
    /// Open a database, apply its initial migration, and recover interrupted runs.
    pub fn open(path: impl AsRef<Path>) -> StoreResult<Self> {
        Connection::open(path).map_err(StoreError::from).and_then(Self::initialize)
    }
    fn initialize(mut connection: Connection) -> StoreResult<Self> {
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(StoreError::from)
            .and_then(|()| {
                connection
                    .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
                    .map_err(StoreError::from)
            })
            .and_then(|version| match version {
                | 0 => connection.transaction().map_err(StoreError::from).and_then(|tx| {
                    tx.execute_batch(include_str!("../assets/migrations/0001_schema.sql"))
                        .map_err(StoreError::from)
                        .and_then(|()| tx.commit().map_err(StoreError::from))
                }),
                | SCHEMA_VERSION => Ok(()),
                | other => Err(StoreError::Schema(other)),
            })
            .and_then(|()| {
                connection
                    .execute_batch(include_str!("../assets/migrations/0002_active_jobs.sql"))
                    .map_err(StoreError::from)
            })
            .and_then(|()| {
                connection
                    .execute(include_str!("../assets/migrations/0003_recover_interrupted_runs.sql"), params![now()])
                    .map(|_| ())
                    .map_err(StoreError::from)
            })
            .and_then(|()| {
                connection
                    .execute(
                        include_str!("../assets/migrations/0004_prune_expired_runs.sql"),
                        params![now().saturating_sub(MAX_HISTORY_AGE_SECONDS)],
                    )
                    .map(|_| Self(Mutex::new(connection)))
                    .map_err(StoreError::from)
            })
    }
    fn with_connection<T>(&self, operation: impl FnOnce(&mut Connection) -> StoreResult<T>) -> StoreResult<T> {
        self.0
            .lock()
            .map_err(|_| StoreError::Lock)
            .and_then(|mut connection| operation(&mut connection))
    }
    /// Register the active job set while retaining persisted pause settings and history.
    pub fn register_jobs(&self, jobs: &[JobId]) -> StoreResult<()> {
        self.activate_jobs(jobs, &[], &[])
    }
    /// Atomically activate jobs, retire superseded queued work, and queue startup runs.
    ///
    /// Running work and completed history retain their original run IDs and metadata.
    /// Persisted pause state follows each job ID across activations.
    pub fn activate_jobs(&self, jobs: &[JobId], superseded: &[JobId], startup: &[JobId]) -> StoreResult<()> {
        self.with_connection(|connection| {
            connection.transaction().map_err(StoreError::from).and_then(|tx| {
                tx.execute(include_str!("../assets/migrations/0005_clear_active_jobs.sql"), [])
                    .map_err(StoreError::from)
                    .and_then(|_| {
                        jobs.iter().try_for_each(|id| {
                            tx.execute(include_str!("../assets/migrations/0006_insert_job.sql"), params![id.as_ref()])
                                .map_err(StoreError::from)
                                .and_then(|_| {
                                    tx.execute(include_str!("../assets/migrations/0007_activate_job.sql"), params![id.as_ref()])
                                        .map(|_| ())
                                        .map_err(StoreError::from)
                                })
                        })
                    })
                    .and_then(|()| {
                        tx.execute(include_str!("../assets/migrations/0008_interrupt_removed_jobs.sql"), params![now()])
                            .map(|_| ())
                            .map_err(StoreError::from)
                    })
                    .and_then(|()| {
                        superseded.iter().try_for_each(|id| {
                            tx.execute(
                                include_str!("../assets/migrations/0009_interrupt_superseded_jobs.sql"),
                                params![id.as_ref(), now()],
                            )
                            .map(|_| ())
                            .map_err(StoreError::from)
                        })
                    })
                    .and_then(|()| {
                        startup.iter().try_for_each(|id| {
                            tx.execute(include_str!("../assets/migrations/0010_enqueue_startup.sql"), params![id.as_ref(), now()])
                                .map(|_| ())
                                .map_err(StoreError::from)
                        })
                    })
                    .and_then(|()| tx.commit().map_err(StoreError::from))
            })
        })
    }
    /// Return a registered job's pause state, or `None` for an unknown job.
    pub fn paused(&self, job: &JobId) -> StoreResult<Option<bool>> {
        self.with_connection(|connection| {
            connection
                .query_row(
                    include_str!("../assets/migrations/0011_select_paused.sql"),
                    params![job.as_ref()],
                    |row| row.get::<_, bool>(0),
                )
                .optional()
                .map_err(StoreError::from)
        })
    }
    /// Persist a registered job's pause state, returning whether it was found.
    pub fn set_paused(&self, job: &JobId, paused: bool) -> StoreResult<bool> {
        self.with_connection(|connection| {
            connection
                .execute(include_str!("../assets/migrations/0012_set_paused.sql"), params![job.as_ref(), paused])
                .map(|changed| changed == 1)
                .map_err(StoreError::from)
        })
    }
    /// Queue one run for an enabled registered job; return its new run ID.
    pub fn enqueue(&self, job: &JobId, trigger: JobTrigger) -> StoreResult<Option<i64>> {
        self.with_connection(|connection| {
            connection
                .execute(
                    include_str!("../assets/migrations/0013_enqueue_run.sql"),
                    params![job.as_ref(), trigger_name(trigger), now()],
                )
                .map(|changed| (changed == 1).then(|| connection.last_insert_rowid()))
                .map_err(StoreError::from)
        })
    }
    /// Atomically claim the oldest queued run when no other run is active.
    pub fn claim_next(&self) -> StoreResult<Option<RunRecord>> {
        self.with_connection(|connection| {
            connection.transaction().map_err(StoreError::from).and_then(|tx| {
                tx.query_row(include_str!("../assets/migrations/0014_select_next_run.sql"), [], |row| {
                    row.get::<_, i64>(0)
                })
                .optional()
                .map_err(StoreError::from)
                .and_then(|id| match id {
                    | None => tx.commit().map(|()| None).map_err(StoreError::from),
                    | Some(id) => tx
                        .execute(include_str!("../assets/migrations/0015_start_run.sql"), params![id, now()])
                        .map_err(StoreError::from)
                        .and_then(|_| read_run(&tx, id))
                        .and_then(|record| tx.commit().map(|()| record).map_err(StoreError::from)),
                })
            })
        })
    }
    /// Finish one active run and retain its exit and report metadata.
    pub fn finish(&self, id: i64, state: RunState, exit_code: Option<i32>, report_path: Option<&Path>, detail: Option<&str>) -> StoreResult<()> {
        let terminal = match state {
            | RunState::Succeeded => Ok("succeeded"),
            | RunState::Failed => Ok("failed"),
            | RunState::Interrupted => Ok("interrupted"),
            | RunState::Queued | RunState::Running => Err(StoreError::Inactive(id)),
        };
        terminal.and_then(|terminal| {
            self.with_connection(|connection| {
                connection.transaction().map_err(StoreError::from).and_then(|tx| {
                    tx.execute(
                        include_str!("../assets/migrations/0016_finish_run.sql"),
                        params![
                            id,
                            terminal,
                            now(),
                            exit_code,
                            report_path.map(|path| path.to_string_lossy().to_string()),
                            detail.map(|text| text.chars().take(4096).collect::<String>())
                        ],
                    )
                    .map_err(StoreError::from)
                    .and_then(|changed| match changed {
                        | 1 => trim_history(&tx, id).and_then(|()| tx.commit().map_err(StoreError::from)),
                        | _ => Err(StoreError::Inactive(id)),
                    })
                })
            })
        })
    }

    /// Read one run by ID.
    pub fn run(&self, id: i64) -> StoreResult<Option<RunRecord>> {
        self.with_connection(|connection| read_run(connection, id))
    }
    /// Return newest runs for one job, bounded by the caller's limit and retention.
    pub fn history(&self, job: &JobId, limit: usize) -> StoreResult<Vec<RunRecord>> {
        self.with_connection(|connection| {
            connection
                .prepare(include_str!("../assets/migrations/0017_select_history.sql"))
                .map_err(StoreError::from)
                .and_then(|mut statement| {
                    statement
                        .query_map(params![job.as_ref(), limit.min(MAX_HISTORY_PER_JOB as usize) as i64], row_to_run)
                        .map_err(StoreError::from)
                        .and_then(|rows| rows.collect::<rusqlite::Result<Vec<_>>>().map_err(StoreError::from))
                })
        })
    }
}
impl DurableOperations for DaemonStore {
    fn state(&self, operation_key: &str) -> ServiceResult<Option<String>> {
        operation_key
            .parse::<i64>()
            .map_err(|_| ServiceError::new(ServiceErrorKind::InvalidInput, "invalid daemon run ID"))
            .and_then(|id| self.run(id).map_err(service_storage_error))
            .map(|run| run.map(|record| record.state))
    }
    fn replay(&self, operation_key: &str) -> ServiceResult<bool> {
        operation_key
            .parse::<i64>()
            .map_err(|_| ServiceError::new(ServiceErrorKind::InvalidInput, "invalid daemon run ID"))
            .and_then(|id| self.run(id).map_err(service_storage_error))
            .and_then(|run| match run {
                | None => Ok(false),
                | Some(record) => JobId::try_from(record.job_id).and_then(|job| {
                    self.enqueue(&job, JobTrigger::Replay)
                        .map_err(service_storage_error)
                        .map(|queued| queued.is_some())
                }),
            })
    }
}
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().min(i64::MAX as u64) as i64)
        .unwrap_or(0)
}
fn read_run(connection: &Connection, id: i64) -> StoreResult<Option<RunRecord>> {
    connection
        .query_row(include_str!("../assets/migrations/0018_select_run.sql"), params![id], row_to_run)
        .optional()
        .map_err(StoreError::from)
}
fn row_to_run(row: &rusqlite::Row<'_>) -> rusqlite::Result<RunRecord> {
    let mut record = RunRecord::default();
    [
        row.get(0).map(|value| record.id = value),
        row.get(1).map(|value| record.job_id = value),
        row.get(2).map(|value| record.trigger = value),
        row.get(3).map(|value| record.state = value),
        row.get(4).map(|value| record.queued_at = value),
        row.get(5).map(|value| record.started_at = value),
        row.get(6).map(|value| record.finished_at = value),
        row.get(7).map(|value| record.exit_code = value),
        row.get::<_, Option<String>>(8).map(|value| record.report_path = value.map(PathBuf::from)),
        row.get(9).map(|value| record.detail = value),
    ]
    .into_iter()
    .collect::<rusqlite::Result<()>>()
    .map(|()| record)
}
fn service_storage_error(_error: StoreError) -> ServiceError {
    ServiceError::new(ServiceErrorKind::Internal, "daemon history is unavailable")
}
fn trigger_name(trigger: JobTrigger) -> &'static str {
    match trigger {
        | JobTrigger::FileChange => "file-change",
        | JobTrigger::Interval => "interval",
        | JobTrigger::Manual => "manual",
        | JobTrigger::Replay => "replay",
        | JobTrigger::Startup => "startup",
    }
}
fn trim_history(transaction: &Transaction<'_>, finished_id: i64) -> StoreResult<()> {
    transaction
        .execute(
            include_str!("../assets/migrations/0019_trim_history.sql"),
            params![finished_id, now().saturating_sub(MAX_HISTORY_AGE_SECONDS), MAX_HISTORY_PER_JOB],
        )
        .map(|_| ())
        .map_err(StoreError::from)
}
