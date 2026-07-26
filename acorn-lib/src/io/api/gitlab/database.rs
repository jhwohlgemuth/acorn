//! Durable GitLab bot delivery and operation queue.
use crate::io::database::backend::{params, Connection};
use crate::io::database::resolve_database_path;
use crate::io::ApiResult;
use crate::prelude::{Mutex, OnceLock, PathBuf};
use crate::util::constants::app::{MAX_GITLAB_OPERATION_ATTEMPTS, MAX_GITLAB_OPERATION_BACKOFF_SECONDS};
use chrono::{Duration, Utc};
use color_eyre::eyre::eyre;
use core::fmt;
use serde::{Deserialize, Serialize};

static CONNECTION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Durable state of one canonical bot operation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationState {
    /// Waiting for a worker.
    Queued,
    /// Claimed by a worker.
    Running,
    /// Completed successfully.
    Succeeded,
    /// Failed but eligible for another attempt.
    RetryableFailure,
    /// Exhausted its retry budget.
    TerminalFailure,
}
impl fmt::Display for OperationState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::Queued => "queued",
            | Self::Running => "running",
            | Self::Succeeded => "succeeded",
            | Self::RetryableFailure => "retryable-failure",
            | Self::TerminalFailure => "terminal-failure",
        })
    }
}
impl OperationState {
    fn parse(value: &str) -> ApiResult<Self> {
        match value {
            | "queued" => Ok(Self::Queued),
            | "running" => Ok(Self::Running),
            | "succeeded" => Ok(Self::Succeeded),
            | "retryable-failure" => Ok(Self::RetryableFailure),
            | "terminal-failure" => Ok(Self::TerminalFailure),
            | _ => Err(eyre!("Unknown bot operation state `{value}`")),
        }
    }
}

/// Result of transactionally recording a delivery and its canonical operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnqueueStatus {
    /// A new delivery and operation were recorded.
    Inserted,
    /// This delivery identifier was already recorded.
    DuplicateDelivery,
    /// The delivery was new but its canonical operation already existed.
    DuplicateOperation,
}

/// One operation claimed by a worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimedOperation {
    /// Canonical idempotency key.
    pub operation_key: String,
    /// Delivery that first created the operation.
    pub delivery_id: String,
    /// Normalized event JSON.
    pub event_json: String,
    /// One-based attempt number.
    pub attempts: u32,
}

/// Aggregate durable queue counters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct QueueCounts {
    /// Operations ready or waiting for a retry.
    pub queued: u64,
    /// Operations currently claimed.
    pub running: u64,
    /// Operations that completed successfully.
    pub succeeded: u64,
    /// Operations that exhausted retries.
    pub failed: u64,
    /// Deliveries whose canonical operation was already present.
    pub deduplicated: u64,
}

/// Database-backed queue for authenticated, normalized webhook events.
#[derive(Clone, Debug)]
pub struct OperationQueue {
    path: Option<PathBuf>,
}
impl OperationQueue {
    /// Use the configured ACORN database path.
    pub fn configured() -> Self {
        Self { path: None }
    }
    /// Use an explicit database path.
    pub fn from_path(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }
    /// Create the durable queue tables when absent.
    pub fn migrate(&self) -> ApiResult<()> {
        self.with_connection(|connection| {
            connection
                .execute_batch(
                    r#"
                    CREATE TABLE IF NOT EXISTS bot_webhook_deliveries (
                        delivery_id TEXT PRIMARY KEY,
                        operation_key TEXT NOT NULL,
                        event_json TEXT NOT NULL,
                        received_at TEXT NOT NULL,
                        deduplicated INTEGER NOT NULL DEFAULT 0
                    );
                    CREATE TABLE IF NOT EXISTS bot_operations (
                        operation_key TEXT PRIMARY KEY,
                        delivery_id TEXT NOT NULL,
                        event_json TEXT NOT NULL,
                        state TEXT NOT NULL,
                        attempts INTEGER NOT NULL DEFAULT 0,
                        available_at TEXT NOT NULL,
                        claimed_at TEXT,
                        last_error TEXT,
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL
                    );
                    "#,
                )
                .map_err(|why| eyre!("Failed to migrate GitLab bot queue — {why}"))
        })
    }
    /// Atomically persist a delivery and its canonical operation.
    pub fn enqueue(&self, delivery_id: &str, operation_key: &str, event_json: &str) -> ApiResult<EnqueueStatus> {
        self.migrate().and_then(|_| {
            self.with_transaction(|connection| {
                let now = Utc::now().to_rfc3339();
                connection
                    .execute(
                        "INSERT INTO bot_webhook_deliveries (delivery_id, operation_key, event_json, received_at, deduplicated)
                         VALUES (?, ?, ?, ?, 0) ON CONFLICT(delivery_id) DO NOTHING",
                        params![delivery_id, operation_key, event_json, now],
                    )
                    .map_err(|why| eyre!("Failed to record webhook delivery — {why}"))
                    .and_then(|deliveries| {
                        if deliveries == 0 {
                            Ok(EnqueueStatus::DuplicateDelivery)
                        } else {
                            connection
                                .execute(
                                    "INSERT INTO bot_operations
                                     (operation_key, delivery_id, event_json, state, attempts, available_at, created_at, updated_at)
                                     VALUES (?, ?, ?, 'queued', 0, ?, ?, ?)
                                     ON CONFLICT(operation_key) DO NOTHING",
                                    params![operation_key, delivery_id, event_json, now, now, now],
                                )
                                .map_err(|why| eyre!("Failed to record webhook operation — {why}"))
                                .and_then(|operations| {
                                    if operations == 0 {
                                        connection
                                            .execute(
                                                "UPDATE bot_webhook_deliveries SET deduplicated = 1 WHERE delivery_id = ?",
                                                params![delivery_id],
                                            )
                                            .map_err(|why| eyre!("Failed to mark duplicate webhook operation — {why}"))
                                            .map(|_| EnqueueStatus::DuplicateOperation)
                                    } else {
                                        Ok(EnqueueStatus::Inserted)
                                    }
                                })
                        }
                    })
            })
        })
    }
    /// Recover stale work and claim the oldest available operation.
    pub fn claim_next(&self, stale_after: Duration) -> ApiResult<Option<ClaimedOperation>> {
        self.migrate().and_then(|_| {
            self.with_transaction(|connection| {
                let now = Utc::now();
                let stale_before = now.checked_sub_signed(stale_after).unwrap_or(now).to_rfc3339();
                let now = now.to_rfc3339();
                connection
                    .execute(
                        "UPDATE bot_operations
                         SET state = 'queued', claimed_at = NULL, available_at = ?, updated_at = ?
                         WHERE state = 'running' AND claimed_at < ?",
                        params![now, now, stale_before],
                    )
                    .map_err(|why| eyre!("Failed to recover stale bot operations — {why}"))
                    .and_then(|_| select_available(connection, &now))
                    .and_then(|operation| match operation {
                        | Some(operation) => connection
                            .execute(
                                "UPDATE bot_operations
                                 SET state = 'running', attempts = attempts + 1, claimed_at = ?, updated_at = ?
                                 WHERE operation_key = ? AND state IN ('queued', 'retryable-failure')",
                                params![now, now, operation.operation_key],
                            )
                            .map_err(|why| eyre!("Failed to claim bot operation — {why}"))
                            .map(|updated| {
                                (updated == 1).then(|| ClaimedOperation {
                                    attempts: operation.attempts.saturating_add(1),
                                    ..operation
                                })
                            }),
                        | None => Ok(None),
                    })
            })
        })
    }
    /// Mark an operation successful.
    pub fn succeed(&self, operation_key: &str) -> ApiResult<()> {
        self.update_state(operation_key, OperationState::Succeeded, None, Utc::now())
    }
    /// Retain a failure and schedule a bounded retry, or make it terminal after five attempts.
    pub fn fail(&self, operation: &ClaimedOperation, error: &str) -> ApiResult<OperationState> {
        let state = if operation.attempts >= MAX_GITLAB_OPERATION_ATTEMPTS {
            OperationState::TerminalFailure
        } else {
            OperationState::RetryableFailure
        };
        let exponent = operation.attempts.saturating_sub(1).min(8);
        let backoff = 1_i64
            .checked_shl(exponent)
            .unwrap_or(MAX_GITLAB_OPERATION_BACKOFF_SECONDS)
            .min(MAX_GITLAB_OPERATION_BACKOFF_SECONDS);
        let available_at = Utc::now().checked_add_signed(Duration::seconds(backoff)).unwrap_or_else(Utc::now);
        self.update_state(&operation.operation_key, state, Some(error), available_at)
            .map(|_| state)
    }
    /// Read the current state for one operation.
    pub fn state(&self, operation_key: &str) -> ApiResult<Option<OperationState>> {
        self.migrate().and_then(|_| {
            self.with_connection(|connection| {
                let result = connection.query_row(
                    "SELECT state FROM bot_operations WHERE operation_key = ?",
                    params![operation_key],
                    |row| row.get::<_, String>(0),
                );
                match result {
                    | Ok(value) => OperationState::parse(&value).map(Some),
                    | Err(why) if is_no_rows(&why) => Ok(None),
                    | Err(why) => Err(eyre!("Failed to read bot operation state — {why}")),
                }
            })
        })
    }
    /// Count operations and deduplicated deliveries.
    pub fn counts(&self) -> ApiResult<QueueCounts> {
        self.migrate().and_then(|_| {
            self.with_connection(|connection| {
                let count = |state: &str| {
                    connection
                        .query_row("SELECT COUNT(*) FROM bot_operations WHERE state = ?", params![state], |row| {
                            row.get::<_, i64>(0)
                        })
                        .map(|value| u64::try_from(value).unwrap_or_default())
                };
                count("queued")
                    .and_then(|queued| count("retryable-failure").map(|retryable| queued.saturating_add(retryable)))
                    .and_then(|queued| count("running").map(|running| (queued, running)))
                    .and_then(|(queued, running)| count("succeeded").map(|succeeded| (queued, running, succeeded)))
                    .and_then(|(queued, running, succeeded)| count("terminal-failure").map(|failed| (queued, running, succeeded, failed)))
                    .and_then(|(queued, running, succeeded, failed)| {
                        connection
                            .query_row("SELECT COUNT(*) FROM bot_webhook_deliveries WHERE deduplicated = 1", params![], |row| {
                                row.get::<_, i64>(0)
                            })
                            .map(|deduplicated| QueueCounts {
                                queued,
                                running,
                                succeeded,
                                failed,
                                deduplicated: u64::try_from(deduplicated).unwrap_or_default(),
                            })
                    })
                    .map_err(|why| eyre!("Failed to count bot operations — {why}"))
            })
        })
    }
    fn update_state(&self, operation_key: &str, state: OperationState, error: Option<&str>, available_at: chrono::DateTime<Utc>) -> ApiResult<()> {
        self.migrate().and_then(|_| {
            self.with_connection(|connection| {
                let now = Utc::now().to_rfc3339();
                connection
                    .execute(
                        "UPDATE bot_operations
                         SET state = ?, available_at = ?, claimed_at = NULL, last_error = ?, updated_at = ?
                         WHERE operation_key = ?",
                        params![state.to_string(), available_at.to_rfc3339(), error, now, operation_key],
                    )
                    .map_err(|why| eyre!("Failed to update bot operation — {why}"))
                    .and_then(|updated| {
                        (updated == 1)
                            .then_some(())
                            .ok_or_else(|| eyre!("Bot operation `{operation_key}` was not found"))
                    })
            })
        })
    }
    fn with_connection<T>(&self, operation: impl FnOnce(&Connection) -> ApiResult<T>) -> ApiResult<T> {
        CONNECTION_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .map_err(|why| eyre!("Failed to acquire GitLab bot database lock — {why}"))
            .and_then(|_guard| {
                resolve_database_path(self.path.as_ref())
                    .and_then(|path| Connection::open(path).map_err(|why| eyre!("Failed to open GitLab bot database — {why}")))
                    .and_then(|connection| operation(&connection))
            })
    }
    fn with_transaction<T>(&self, operation: impl FnOnce(&Connection) -> ApiResult<T>) -> ApiResult<T> {
        self.with_connection(|connection| {
            connection
                .execute_batch("BEGIN TRANSACTION")
                .map_err(|why| eyre!("Failed to begin GitLab bot transaction — {why}"))
                .and_then(|_| match operation(connection) {
                    | Ok(value) => connection
                        .execute_batch("COMMIT")
                        .map_err(|why| eyre!("Failed to commit GitLab bot transaction — {why}"))
                        .map(|_| value),
                    | Err(why) => {
                        connection.execute_batch("ROLLBACK").ok();
                        Err(why)
                    }
                })
        })
    }
}

fn select_available(connection: &Connection, now: &str) -> ApiResult<Option<ClaimedOperation>> {
    let result = connection.query_row(
        "SELECT operation_key, delivery_id, event_json, attempts
         FROM bot_operations
         WHERE state IN ('queued', 'retryable-failure') AND available_at <= ?
         ORDER BY created_at, operation_key LIMIT 1",
        params![now],
        |row| {
            let attempts = row.get::<_, i64>(3)?;
            Ok(ClaimedOperation {
                operation_key: row.get(0)?,
                delivery_id: row.get(1)?,
                event_json: row.get(2)?,
                attempts: u32::try_from(attempts).unwrap_or_default(),
            })
        },
    );
    match result {
        | Ok(operation) => Ok(Some(operation)),
        | Err(why) if is_no_rows(&why) => Ok(None),
        | Err(why) => Err(eyre!("Failed to select an available bot operation — {why}")),
    }
}

#[cfg(not(feature = "duckdb"))]
fn is_no_rows(error: &crate::io::database::backend::Error) -> bool {
    matches!(error, crate::io::database::backend::Error::QueryReturnedNoRows)
}
#[cfg(feature = "duckdb")]
fn is_no_rows(error: &crate::io::database::backend::Error) -> bool {
    error.to_string().contains("Query returned no rows")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use nanoid::nanoid;
    use std::env::temp_dir;

    fn queue() -> OperationQueue {
        OperationQueue::from_path(temp_dir().join(format!("acorn-bot-{}.db", nanoid!())))
    }

    #[test]
    fn duplicate_delivery_and_operation_are_idempotent() {
        let queue = queue();
        assert_eq!(
            queue.enqueue("delivery-1", "mr-check:1:2:abc", r#"{"event":"normalized"}"#).unwrap(),
            EnqueueStatus::Inserted
        );
        assert_eq!(
            queue.enqueue("delivery-1", "mr-check:1:2:abc", r#"{"event":"changed"}"#).unwrap(),
            EnqueueStatus::DuplicateDelivery
        );
        assert_eq!(
            queue.enqueue("delivery-2", "mr-check:1:2:abc", r#"{"event":"normalized"}"#).unwrap(),
            EnqueueStatus::DuplicateOperation
        );
        assert_eq!(queue.counts().unwrap().deduplicated, 1);
    }

    #[test]
    fn claims_retries_and_retains_terminal_failure() {
        let queue = queue();
        queue.enqueue("delivery-1", "work-item-check:1:2:3", "{}").unwrap();
        let mut operation = queue.claim_next(Duration::minutes(5)).unwrap().unwrap();
        assert_eq!(operation.attempts, 1);
        for expected_attempt in 1..=MAX_GITLAB_OPERATION_ATTEMPTS {
            operation.attempts = expected_attempt;
            let state = queue.fail(&operation, "temporary").unwrap();
            if expected_attempt < MAX_GITLAB_OPERATION_ATTEMPTS {
                assert_eq!(state, OperationState::RetryableFailure);
                queue
                    .with_connection(|connection| {
                        connection
                            .execute(
                                "UPDATE bot_operations SET available_at = ? WHERE operation_key = ?",
                                params![
                                    Utc::now().checked_sub_signed(Duration::seconds(1)).unwrap().to_rfc3339(),
                                    operation.operation_key
                                ],
                            )
                            .map(|_| ())
                            .map_err(|why| eyre!("{why}"))
                    })
                    .unwrap();
                operation = queue.claim_next(Duration::minutes(5)).unwrap().unwrap();
            } else {
                assert_eq!(state, OperationState::TerminalFailure);
            }
        }
        assert_eq!(queue.state(&operation.operation_key).unwrap(), Some(OperationState::TerminalFailure));
        assert_eq!(queue.counts().unwrap().failed, 1);
    }

    #[test]
    fn stale_running_operation_is_recovered_after_restart() {
        let queue = queue();
        queue.enqueue("delivery-1", "mr-check:1:2:abc", "{}").unwrap();
        let claimed = queue.claim_next(Duration::minutes(5)).unwrap().unwrap();
        queue
            .with_connection(|connection| {
                connection
                    .execute(
                        "UPDATE bot_operations SET claimed_at = ? WHERE operation_key = ?",
                        params![
                            Utc::now().checked_sub_signed(Duration::minutes(10)).unwrap().to_rfc3339(),
                            claimed.operation_key
                        ],
                    )
                    .map(|_| ())
                    .map_err(|why| eyre!("{why}"))
            })
            .unwrap();
        let restarted = OperationQueue::from_path(queue.path.clone().unwrap());
        assert_eq!(restarted.claim_next(Duration::minutes(5)).unwrap().unwrap().attempts, 2);
    }

    #[test]
    #[cfg(not(feature = "duckdb"))]
    fn schema_does_not_store_raw_webhook_bodies() {
        let queue = queue();
        queue.migrate().unwrap();
        queue
            .with_connection(|connection| {
                let sql = connection
                    .query_row("SELECT sql FROM sqlite_master WHERE name = 'bot_webhook_deliveries'", params![], |row| {
                        row.get::<_, String>(0)
                    })
                    .map_err(|why| eyre!("{why}"))?;
                assert!(!sql.contains("raw_body"));
                Ok(())
            })
            .unwrap();
    }
}
