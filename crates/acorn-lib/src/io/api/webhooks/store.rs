//! Durable provider-neutral webhook delivery and operation queue.
use crate::io::api::json_rpc::{InvocationContext, JsonRpcOperationRegistry, Notification};
use crate::io::api::webhooks::{self, CallbackResolver, OperationSpec, PreparedDelivery, WebhookRuntime};
#[cfg(not(feature = "duckdb"))]
use crate::io::database::backend::Error::QueryReturnedNoRows;
use crate::io::database::backend::{params, Connection};
use crate::io::database::resolve_database_path;
use crate::io::http::HttpService;
use crate::io::ApiResult;
use crate::util::constants::app::{MAX_WEBHOOK_OPERATION_ATTEMPTS, MAX_WEBHOOK_OPERATION_BACKOFF_SECONDS};
use acorn_core::util::to_rfc3339;
use acorn_schema::validation::Validate;
use color_eyre::eyre::eyre;
use core::fmt;
use jiff::{SignedDuration, Timestamp};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static CONNECTION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
const INTAKE_MIGRATION: &str = include_str!("../../../../assets/migrations/0014_create_powerautomate_jsonl.sql");
const GITLAB_EVENT_CURSOR_MIGRATION: &str = include_str!("../../../../assets/migrations/0015_create_gitlab_event_cursors.sql");
const WEBHOOK_EFFECT_MIGRATION: &str = include_str!("../../../../assets/migrations/0013_create_webhook_effects.sql");
const WEBHOOK_MIGRATION: &str = include_str!("../../../../assets/migrations/0012_create_webhook_runtime.sql");
/// Provider-neutral name for the durable webhook operation queue.
pub type WebhookStore = OperationQueue;
trait ConnectionExt {
    fn begin_effect(&self, effect_key: &str, effect_kind: &str) -> ApiResult<()>;
    fn effect_result<T: DeserializeOwned>(&self, effect_key: &str, effect_kind: &str) -> ApiResult<Option<T>>;
    fn enqueue_notification(&self, delivery_id: &str, operation_key: &str, event_json: &str) -> ApiResult<EnqueueStatus>;
    fn read_intake_admission(&self, operation_key: &str) -> ApiResult<Option<IntakeAdmission>>;
    fn read_runtime_timestamps(&self) -> ApiResult<RuntimeTimestamps>;
    fn record_outcome(&self, state: OperationState, now: &str) -> ApiResult<()>;
    fn select_available(&self, now: &str) -> ApiResult<Option<ClaimedOperation>>;
    fn select_available_callback(&self, now: &str) -> ApiResult<Option<ClaimedCallback>>;
    fn succeed_effect(&self, effect_key: &str, effect_kind: &str, result_json: &str) -> ApiResult<()>;
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
/// Durable state of one canonical webhook operation.
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
    /// Cancelled before a worker started it.
    Cancelled,
}
/// Aggregate durable callback states for manual delivery summaries.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct CallbackCounts {
    /// Ready callback intents that have not been attempted.
    pub queued: u64,
    /// Callback intents waiting for another attempt.
    pub retryable: u64,
    /// Callback intents currently claimed by a worker.
    pub running: u64,
    /// Callback intents accepted by their destination.
    pub succeeded: u64,
    /// Callback intents that exhausted attempts.
    pub failed: u64,
}
/// Worker that resolves callback credentials at attempt time and dispatches one durable intent.
pub struct CallbackWorker<S, R> {
    queue: OperationQueue,
    resolver: R,
    runtime: WebhookRuntime,
    service: S,
    stale_after: SignedDuration,
}
/// One durable outbound callback claim.
#[derive(Debug, Eq, PartialEq)]
pub struct ClaimedCallback {
    attempts: u32,
    callback_key: String,
    destination: String,
    payload_json: String,
}
/// One operation claimed by a worker.
///
/// This opaque typestate token represents the only queue state that callers may settle.
/// It cannot be cloned and is consumed by [`OperationQueue::succeed`] or
/// [`OperationQueue::fail`]. Other states remain authoritative in the database.
///
/// ```compile_fail
/// # use acorn::io::api::webhooks::store::ClaimedOperation;
///
/// # fn duplicate(claim: ClaimedOperation) {
///     let duplicate = claim.clone();
/// # }
/// ```
///
/// ```compile_fail
/// # use acorn::io::api::webhooks::store::{ClaimedOperation, OperationQueue};
///
/// # fn settle_twice(queue: &OperationQueue, claim: ClaimedOperation) {
///     queue.succeed(claim).unwrap();
///     queue.fail(claim, "failed").unwrap();
/// # }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct ClaimedOperation {
    /// Canonical idempotency key.
    operation_key: String,
    /// Delivery that first created the operation.
    delivery_id: String,
    /// Normalized event JSON.
    event_json: String,
    /// One-based attempt number.
    attempts: u32,
}
/// Immutable accepted intake payload retained after inbox cleanup.
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct IntakeAdmission {
    /// Exact original line, excluding its LF delimiter.
    pub raw: String,
    /// Digest of the original bytes.
    #[validate(digest)]
    pub digest: String,
    /// Serialized provider-owned entry and resolved route.
    pub payload: String,
}
/// Inspectable outcome and source evidence for one inbox observation.
#[derive(bon::Builder, Clone, Debug, Deserialize, Serialize, Validate)]
pub struct IntakeRecord {
    /// Credential-free source identity including project, branch, and path.
    pub source: String,
    /// Immutable source commit.
    #[validate(commit)]
    pub commit: String,
    /// One-based physical line.
    #[validate(range(min = 1))]
    pub line: u64,
    /// Original byte offset.
    pub byte_offset: u64,
    /// Exact line bytes, excluding LF.
    pub raw: String,
    /// Digest of the original bytes.
    #[validate(digest)]
    pub digest: String,
    /// Original routing selector when parsing succeeded.
    pub selector: Option<String>,
    /// Canonical operation identity when parsing succeeded.
    pub operation_key: Option<String>,
    /// Immutable accepted payload, absent on rejection.
    pub admission: Option<String>,
    /// Inspectable rejection reason.
    pub rejection: Option<String>,
}
/// Last fully queued GitLab project event for one source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GitLabEventCursor {
    pub(crate) created_at: String,
    pub(crate) event_id: u64,
}
/// Database-backed queue for authenticated, normalized webhook events.
#[derive(Clone, Debug)]
pub struct OperationQueue {
    path: Option<PathBuf>,
    #[cfg(test)]
    settlement_hook: Option<fn() -> ApiResult<()>>,
}
/// Worker that dispatches claimed webhook notifications through the shared operation registry.
#[derive(Clone, Debug)]
pub struct OperationWorker {
    context: InvocationContext,
    queue: OperationQueue,
    registry: JsonRpcOperationRegistry,
    runtime: WebhookRuntime,
    stale_after: SignedDuration,
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
    /// Operations explicitly cancelled before execution.
    pub cancelled: u64,
    /// Deliveries whose canonical operation was already present.
    pub deduplicated: u64,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RuntimeTimestamps {
    pub(crate) last_error_at: Option<Timestamp>,
    pub(crate) last_success_at: Option<Timestamp>,
}
impl<S: HttpService, R: CallbackResolver> CallbackWorker<S, R> {
    /// Construct a durable outbound callback worker.
    pub fn new(queue: OperationQueue, service: S, resolver: R, stale_after: SignedDuration) -> Self {
        Self {
            queue,
            resolver,
            runtime: WebhookRuntime::default(),
            service,
            stale_after,
        }
    }
    /// Claim and attempt one callback, returning whether work was available.
    pub async fn run_once(&self) -> ApiResult<bool> {
        match self.runtime.initialize(&self.queue, self.stale_after) {
            | Err(why) => Err(why),
            | Ok(()) => match self.runtime.claim(|| self.queue.claim_callback(self.stale_after)) {
                | Ok(Some((callback, _claim))) => {
                    let result = match self.resolver.resolve(&callback.destination) {
                        | Ok(destination) => {
                            let (target, headers, signer) = destination.parts();
                            match signer {
                                | Some(signer) => {
                                    let raw_body = callback.payload_json.as_bytes();
                                    let dispatch = webhooks::dispatch_signed(
                                        &self.service,
                                        target,
                                        raw_body,
                                        headers,
                                        signer,
                                        &callback.callback_key,
                                        Timestamp::now(),
                                    );
                                    dispatch.await.map(|result| result.status_code).map_err(|why| why.to_string())
                                }
                                | None => match serde_json::from_str::<serde_json::Value>(&callback.payload_json) {
                                    | Ok(payload) => webhooks::dispatch(&self.service, target, &payload, headers)
                                        .await
                                        .map(|result| result.status_code)
                                        .map_err(|why| why.to_string()),
                                    | Err(why) => Err(format!("Failed to decode durable callback payload — {why}")),
                                },
                            }
                        }
                        | Err(why) => Err(why.to_string()),
                    };
                    match result {
                        | Ok(status) => match self.queue.succeed_callback(callback, status) {
                            | Ok(()) => self.runtime.refresh(&self.queue).map(|()| true),
                            | Err(why) => {
                                self.runtime.mark_unavailable();
                                Err(why)
                            }
                        },
                        | Err(why) => match self.queue.fail_callback(callback, &why) {
                            | Ok(_) => self.runtime.refresh(&self.queue).and_then(|()| Err(eyre!(why))),
                            | Err(settlement_error) => {
                                self.runtime.mark_unavailable();
                                Err(settlement_error)
                            }
                        },
                    }
                }
                | Ok(None) => Ok(false),
                | Err(why) => Err(why),
            },
        }
    }
    /// Use a shared runtime for readiness and drain coordination.
    pub fn with_runtime(self, runtime: WebhookRuntime) -> Self {
        Self { runtime, ..self }
    }
}
impl ClaimedCallback {
    /// One-based delivery attempt number.
    pub fn attempts(&self) -> u32 {
        self.attempts
    }
    /// Stable callback idempotency key.
    pub fn callback_key(&self) -> &str {
        &self.callback_key
    }
}
impl ClaimedOperation {
    /// Return the one-based attempt number.
    pub fn attempts(&self) -> u32 {
        self.attempts
    }
    /// Return the delivery that first created the operation.
    pub fn delivery_id(&self) -> &str {
        &self.delivery_id
    }
    /// Return the normalized event JSON.
    pub fn event_json(&self) -> &str {
        &self.event_json
    }
    /// Decode the persisted JSON-RPC notification.
    pub fn notification(&self) -> ApiResult<Notification> {
        serde_json::from_str(&self.event_json).map_err(|why| eyre!("Failed to decode webhook operation notification — {why}"))
    }
    /// Return the canonical idempotency key.
    pub fn operation_key(&self) -> &str {
        &self.operation_key
    }
}
impl ConnectionExt for Connection {
    fn begin_effect(&self, key: &str, kind: &str) -> ApiResult<()> {
        let now = to_rfc3339(Timestamp::now());
        self.execute(
            "INSERT INTO webhook_effects (effect_key, effect_kind, state, result_json, created_at, updated_at)
             VALUES (?, ?, 'pending', NULL, ?, ?) ON CONFLICT(effect_key) DO NOTHING",
            params![key, kind, now, now],
        )
        .map_err(|why| eyre!("Failed to begin webhook effect — {why}"))
        .and_then(|_| {
            self.query_row("SELECT effect_kind FROM webhook_effects WHERE effect_key = ?", params![key], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|why| eyre!("Failed to read webhook effect kind — {why}"))
        })
        .and_then(|stored_kind| match stored_kind == kind {
            | true => Ok(()),
            | false => Err(eyre!("Webhook effect key `{key}` is already assigned to `{stored_kind}`, not `{kind}`")),
        })
    }
    fn effect_result<T: DeserializeOwned>(&self, effect_key: &str, effect_kind: &str) -> ApiResult<Option<T>> {
        let result = self.query_row(
            "SELECT effect_kind, state, result_json FROM webhook_effects WHERE effect_key = ?",
            params![effect_key],
            |row| {
                row.get::<_, String>(0).and_then(|kind| {
                    row.get::<_, String>(1)
                        .and_then(|state| row.get::<_, Option<String>>(2).map(|json| (kind, state, json)))
                })
            },
        );
        match result {
            | Err(why) if is_no_rows(&why) => Ok(None),
            | Err(why) => Err(eyre!("Failed to read webhook effect — {why}")),
            | Ok((stored_kind, _, _)) if stored_kind != effect_kind => Err(eyre!(
                "Webhook effect key `{effect_key}` is assigned to `{stored_kind}`, not `{effect_kind}`"
            )),
            | Ok((_, state, Some(json))) if state == "succeeded" => serde_json::from_str(&json)
                .map(Some)
                .map_err(|why| eyre!("Failed to decode webhook effect result — {why}")),
            | Ok(_) => Ok(None),
        }
    }
    fn enqueue_notification(&self, delivery_id: &str, operation_key: &str, event_json: &str) -> ApiResult<EnqueueStatus> {
        let now = to_rfc3339(Timestamp::now());
        self.execute(
            "INSERT INTO webhook_deliveries (delivery_id, operation_key, event_json, received_at, deduplicated)
                         VALUES (?, ?, ?, ?, 0) ON CONFLICT(delivery_id) DO NOTHING",
            params![delivery_id, operation_key, event_json, now],
        )
        .map_err(|why| eyre!("Failed to record webhook delivery — {why}"))
        .and_then(|deliveries| {
            if deliveries == 0 {
                Ok(EnqueueStatus::DuplicateDelivery)
            } else {
                self.execute(
                    "INSERT INTO webhook_operations
                                     (operation_key, delivery_id, event_json, state, attempts, available_at, created_at, updated_at)
                                     VALUES (?, ?, ?, 'queued', 0, ?, ?, ?)
                                     ON CONFLICT(operation_key) DO NOTHING",
                    params![operation_key, delivery_id, event_json, now, now, now],
                )
                .map_err(|why| eyre!("Failed to record webhook operation — {why}"))
                .and_then(|operations| {
                    if operations == 0 {
                        self.execute(
                            "UPDATE webhook_deliveries SET deduplicated = 1 WHERE delivery_id = ?",
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
    }
    fn read_intake_admission(&self, operation_key: &str) -> ApiResult<Option<IntakeAdmission>> {
        match self.query_row(
            "SELECT raw, digest, admission_json FROM webhook_intake_admissions WHERE operation_key = ?",
            params![operation_key],
            |row| {
                row.get(0).and_then(|raw| {
                    row.get(1)
                        .and_then(|digest| row.get(2).map(|payload| IntakeAdmission { raw, digest, payload }))
                })
            },
        ) {
            | Ok(admission) => Ok(Some(admission)),
            | Err(why) if is_no_rows(&why) => Ok(None),
            | Err(why) => Err(eyre!("Failed to read intake admission — {why}")),
        }
    }
    fn read_runtime_timestamps(&self) -> ApiResult<RuntimeTimestamps> {
        self.query_row(
            "SELECT last_error_at, last_success_at FROM webhook_runtime_state WHERE singleton = 1",
            params![],
            |row| {
                row.get::<_, Option<String>>(0)
                    .and_then(|error| row.get::<_, Option<String>>(1).map(|success| (error, success)))
            },
        )
        .map_err(|why| eyre!("Failed to read webhook runtime state — {why}"))
        .and_then(|(last_error_at, last_success_at)| {
            parse_timestamp(last_error_at, "last_error_at").and_then(|last_error_at| {
                parse_timestamp(last_success_at, "last_success_at").map(|last_success_at| RuntimeTimestamps {
                    last_error_at,
                    last_success_at,
                })
            })
        })
    }
    fn record_outcome(&self, state: OperationState, now: &str) -> ApiResult<()> {
        let column = match state {
            | OperationState::Succeeded => Some("last_success_at"),
            | OperationState::RetryableFailure | OperationState::TerminalFailure => Some("last_error_at"),
            | OperationState::Cancelled | OperationState::Queued | OperationState::Running => None,
        };
        column.map_or_else(
            || Ok(()),
            |column| {
                self.execute(
                    &format!("UPDATE webhook_runtime_state SET {column} = ? WHERE singleton = 1"),
                    params![now],
                )
                .map(|_| ())
                .map_err(|why| eyre!("Failed to record webhook runtime outcome — {why}"))
            },
        )
    }
    fn select_available(&self, now: &str) -> ApiResult<Option<ClaimedOperation>> {
        let result = self.query_row(
            "SELECT operation_key, delivery_id, event_json, attempts
             FROM webhook_operations
             WHERE state IN ('queued', 'retryable-failure') AND available_at <= ?
             ORDER BY created_at, operation_key LIMIT 1",
            params![now],
            |row| {
                row.get::<_, i64>(3).and_then(|attempts| {
                    row.get(0).and_then(|operation_key| {
                        row.get(1).and_then(|delivery_id| {
                            row.get(2).map(|event_json| ClaimedOperation {
                                operation_key,
                                delivery_id,
                                event_json,
                                attempts: u32::try_from(attempts).unwrap_or_default(),
                            })
                        })
                    })
                })
            },
        );
        match result {
            | Ok(operation) => Ok(Some(operation)),
            | Err(why) if is_no_rows(&why) => Ok(None),
            | Err(why) => Err(eyre!("Failed to select an available webhook operation — {why}")),
        }
    }
    fn select_available_callback(&self, now: &str) -> ApiResult<Option<ClaimedCallback>> {
        let result = self.query_row(
            "SELECT callback_key, destination, payload_json, attempts
             FROM webhook_callbacks
             WHERE state IN ('queued', 'retryable-failure') AND available_at <= ?
             ORDER BY created_at, callback_key LIMIT 1",
            params![now],
            |row| {
                row.get::<_, i64>(3).and_then(|attempts| {
                    row.get(0).and_then(|callback_key| {
                        row.get(1).and_then(|destination| {
                            row.get(2).map(|payload_json| ClaimedCallback {
                                attempts: u32::try_from(attempts).unwrap_or_default(),
                                callback_key,
                                destination,
                                payload_json,
                            })
                        })
                    })
                })
            },
        );
        match result {
            | Ok(callback) => Ok(Some(callback)),
            | Err(why) if is_no_rows(&why) => Ok(None),
            | Err(why) => Err(eyre!("Failed to select an available webhook callback — {why}")),
        }
    }
    fn succeed_effect(&self, effect_key: &str, effect_kind: &str, result_json: &str) -> ApiResult<()> {
        let now = to_rfc3339(Timestamp::now());
        self.execute(
            "UPDATE webhook_effects SET state = 'succeeded', result_json = ?, updated_at = ?
             WHERE effect_key = ? AND effect_kind = ?",
            params![result_json, now, effect_key, effect_kind],
        )
        .map_err(|why| eyre!("Failed to complete webhook effect — {why}"))
        .and_then(|updated| match updated == 1 {
            | true => Ok(()),
            | false => Err(eyre!("Webhook effect `{effect_key}` could not be completed")),
        })
    }
}
impl IntakeRecord {
    fn insert_admission(&self, connection: &Connection, key: &str, payload: &str) -> ApiResult<Self> {
        connection
            .query_row("SELECT COUNT(*) FROM webhook_operations WHERE operation_key = ?", params![key], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|why| eyre!("Failed to check intake identity — {why}"))
            .and_then(|count| match count {
                | 0 => connection
                    .execute(
                        "INSERT INTO webhook_intake_admissions (operation_key, raw, digest, admission_json) VALUES (?, ?, ?, ?)",
                        params![key, self.raw, self.digest, payload],
                    )
                    .map_err(|why| eyre!("Failed to retain intake admission — {why}"))
                    .map(|_| self.clone()),
                | _ => Ok(Self {
                    admission: None,
                    rejection: Some("Existing operation has no verifiable JSONL byte evidence".to_string()),
                    ..self.clone()
                }),
            })
    }
    fn persist(&self, connection: &Connection) -> ApiResult<()> {
        i64::try_from(self.line)
            .map_err(|why| eyre!("Intake line exceeds database range — {why}"))
            .and_then(|line| serde_json::to_string(self).map_err(Into::into).map(|json| (line, json)))
            .and_then(|(line, json)| {
                connection
                    .execute(
                        "INSERT INTO webhook_intake_records (source, commit_sha, line, digest, record_json) VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT(source, commit_sha, line, digest) DO UPDATE SET record_json = excluded.record_json",
                        params![self.source, self.commit, line, self.digest, json],
                    )
                    .map_err(|why| eyre!("Failed to retain intake observation — {why}"))
                    .map(|_| ())
            })
    }
    fn prepare(&self, connection: &Connection, operation: Option<&OperationSpec>) -> ApiResult<Self> {
        match (operation, self.operation_key.as_deref(), self.admission.as_deref()) {
            | (Some(operation), Some(key), Some(payload)) if format!("{}:{}", operation.provider, operation.idempotency_key) == key => connection
                .read_intake_admission(key)
                .and_then(|existing| match existing {
                    | Some(existing) if existing.raw != self.raw || existing.digest != self.digest => Ok(Self {
                        admission: None,
                        rejection: Some("Submission UUID conflicts with previously admitted bytes".to_string()),
                        ..self.clone()
                    }),
                    | Some(existing) => Ok(Self {
                        admission: Some(existing.payload),
                        rejection: None,
                        ..self.clone()
                    }),
                    | None => self.insert_admission(connection, key, payload),
                })
                .and_then(|outcome| match &outcome.admission {
                    | Some(_) => serde_json::to_string(&operation.notification)
                        .map_err(Into::into)
                        .and_then(|notification| {
                            let delivery = format!("{}:powerautomate-jsonl:{}:{}:{}", operation.provider, self.source, self.commit, self.line);
                            connection.enqueue_notification(&delivery, key, &notification).map(|_| outcome)
                        }),
                    | None => Ok(outcome),
                }),
            | (None, _, None) if self.rejection.is_some() => Ok(self.clone()),
            | _ => Err(eyre!("Incomplete intake admission")),
        }
    }
}
impl From<PathBuf> for OperationQueue {
    fn from(path: PathBuf) -> Self {
        Self {
            path: Some(path),
            #[cfg(test)]
            settlement_hook: None,
        }
    }
}
impl OperationQueue {
    /// Read the last fully queued event for one GitLab project.
    pub(crate) fn gitlab_event_cursor(&self, source: &str) -> ApiResult<Option<GitLabEventCursor>> {
        self.migrate().and_then(|_| {
            self.with_connection(|connection| {
                let result = connection.query_row(
                    "SELECT created_at, event_id FROM gitlab_event_cursors WHERE source = ?",
                    params![source],
                    |row| {
                        row.get::<_, String>(0)
                            .and_then(|created_at| row.get::<_, String>(1).map(|event_id| (created_at, event_id)))
                    },
                );
                match result {
                    | Ok((created_at, event_id)) => event_id
                        .parse::<u64>()
                        .map(|event_id| Some(GitLabEventCursor { created_at, event_id }))
                        .map_err(|why| eyre!("Invalid GitLab event cursor ID — {why}")),
                    | Err(why) if is_no_rows(&why) => Ok(None),
                    | Err(why) => Err(eyre!("Failed to read GitLab event cursor — {why}")),
                }
            })
        })
    }
    /// Persist the last event only after the complete project event scan was queued.
    pub(crate) fn set_gitlab_event_cursor(&self, source: &str, cursor: &GitLabEventCursor) -> ApiResult<()> {
        self.migrate().and_then(|_| {
            self.with_connection(|connection| {
                connection
                    .execute(
                        "INSERT INTO gitlab_event_cursors (source, created_at, event_id) VALUES (?, ?, ?)
                         ON CONFLICT(source) DO UPDATE SET created_at = excluded.created_at, event_id = excluded.event_id",
                        params![source, cursor.created_at, cursor.event_id.to_string()],
                    )
                    .map(|_| ())
                    .map_err(|why| eyre!("Failed to persist GitLab event cursor — {why}"))
            })
        })
    }
    /// Count callback intents by durable delivery state.
    pub fn callback_counts(&self) -> ApiResult<CallbackCounts> {
        self.migrate().and_then(|_| {
            self.with_connection(|connection| {
                let count = |state: &str| {
                    connection
                        .query_row("SELECT COUNT(*) FROM webhook_callbacks WHERE state = ?", params![state], |row| {
                            row.get::<_, i64>(0)
                        })
                        .map(|value| u64::try_from(value).unwrap_or_default())
                };
                count("queued")
                    .and_then(|queued| count("retryable-failure").map(|retryable| (queued, retryable)))
                    .and_then(|(queued, retryable)| count("running").map(|running| (queued, retryable, running)))
                    .and_then(|(queued, retryable, running)| count("succeeded").map(|succeeded| (queued, retryable, running, succeeded)))
                    .and_then(|(queued, retryable, running, succeeded)| {
                        count("terminal-failure").map(|failed| CallbackCounts {
                            queued,
                            retryable,
                            running,
                            succeeded,
                            failed,
                        })
                    })
                    .map_err(|why| eyre!("Failed to count callback intents — {why}"))
            })
        })
    }
    #[cfg(not(test))]
    fn before_settlement(&self) -> ApiResult<()> {
        Ok(())
    }
    #[cfg(test)]
    fn before_settlement(&self) -> ApiResult<()> {
        self.settlement_hook.map_or(Ok(()), |hook| hook())
    }
    /// Begin one idempotent external effect, preserving an existing attempt or result.
    pub fn begin_effect(&self, key: &str, kind: &str) -> ApiResult<()> {
        match (key.trim().is_empty(), kind.trim().is_empty()) {
            | (true, _) => Err(eyre!("Webhook effect key is required")),
            | (_, true) => Err(eyre!("Webhook effect kind is required")),
            | _ => self
                .migrate()
                .and_then(|_| self.with_connection(|connection| connection.begin_effect(key, kind))),
        }
    }
    /// Count persisted attempts for one outbound callback.
    pub fn callback_attempts(&self, callback_key: &str) -> ApiResult<u64> {
        self.migrate().and_then(|_| {
            self.with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM webhook_callback_attempts WHERE callback_key = ?",
                        params![callback_key],
                        |row| row.get::<_, i64>(0),
                    )
                    .map(|value| u64::try_from(value).unwrap_or_default())
                    .map_err(|why| eyre!("Failed to count webhook callback attempts — {why}"))
            })
        })
    }
    /// Read the current state for one outbound callback.
    pub fn callback_state(&self, callback_key: &str) -> ApiResult<Option<OperationState>> {
        self.migrate().and_then(|_| {
            self.with_connection(|connection| {
                let result = connection.query_row(
                    "SELECT state FROM webhook_callbacks WHERE callback_key = ?",
                    params![callback_key],
                    |row| row.get::<_, String>(0),
                );
                match result {
                    | Ok(value) => OperationState::try_from(value.as_str()).map(Some),
                    | Err(why) if is_no_rows(&why) => Ok(None),
                    | Err(why) => Err(eyre!("Failed to read webhook callback state — {why}")),
                }
            })
        })
    }
    /// Cancel queued work. Running or completed work is left unchanged.
    pub fn cancel(&self, operation_key: &str) -> ApiResult<bool> {
        self.migrate().and_then(|_| {
            self.with_connection(|connection| {
                let now = to_rfc3339(Timestamp::now());
                connection
                    .execute(
                        "UPDATE webhook_operations SET state = 'cancelled', updated_at = ?
                         WHERE operation_key = ? AND state IN ('queued', 'retryable-failure')",
                        params![now, operation_key],
                    )
                    .map(|updated| updated == 1)
                    .map_err(|why| eyre!("Failed to cancel webhook operation — {why}"))
            })
        })
    }
    /// Recover stale callback claims and claim the oldest available intent.
    pub fn claim_callback(&self, stale_after: SignedDuration) -> ApiResult<Option<ClaimedCallback>> {
        self.migrate().and_then(|_| {
            self.with_transaction(|connection| {
                let now = Timestamp::now();
                let stale_before = to_rfc3339(now.checked_sub(stale_after).unwrap_or(now));
                let now = to_rfc3339(now);
                connection
                    .execute(
                        "UPDATE webhook_callbacks SET state = 'queued', claimed_at = NULL, available_at = ?, updated_at = ?
                         WHERE state = 'running' AND claimed_at < ?",
                        params![now, now, stale_before],
                    )
                    .map_err(|why| eyre!("Failed to recover stale webhook callbacks — {why}"))
                    .and_then(|_| connection.select_available_callback(&now))
                    .and_then(|callback| match callback {
                        | Some(callback) => connection
                            .execute(
                                "UPDATE webhook_callbacks SET state = 'running', attempts = attempts + 1, claimed_at = ?, updated_at = ?
                                 WHERE callback_key = ? AND state IN ('queued', 'retryable-failure')",
                                params![now, now, callback.callback_key],
                            )
                            .map_err(|why| eyre!("Failed to claim webhook callback — {why}"))
                            .map(|updated| {
                                (updated == 1).then(|| ClaimedCallback {
                                    attempts: callback.attempts.saturating_add(1),
                                    ..callback
                                })
                            }),
                        | None => Ok(None),
                    })
            })
        })
    }
    /// Recover stale work and claim the oldest available operation.
    pub fn claim_next(&self, stale_after: SignedDuration) -> ApiResult<Option<ClaimedOperation>> {
        self.migrate().and_then(|_| {
            self.with_transaction(|connection| {
                let now = Timestamp::now();
                let stale_before = to_rfc3339(now.checked_sub(stale_after).unwrap_or(now));
                let now = to_rfc3339(now);
                connection
                    .execute(
                        "UPDATE webhook_operations
                         SET state = 'queued', claimed_at = NULL, available_at = ?, updated_at = ?
                         WHERE state = 'running' AND claimed_at < ?",
                        params![now, now, stale_before],
                    )
                    .map_err(|why| eyre!("Failed to recover stale webhook operations — {why}"))
                    .and_then(|_| connection.select_available(&now))
                    .and_then(|operation| match operation {
                        | Some(operation) => connection
                            .execute(
                                "UPDATE webhook_operations
                                 SET state = 'running', attempts = attempts + 1, claimed_at = ?, updated_at = ?
                                 WHERE operation_key = ? AND state IN ('queued', 'retryable-failure')",
                                params![now, now, operation.operation_key],
                            )
                            .map_err(|why| eyre!("Failed to claim webhook operation — {why}"))
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
    /// Use the configured ACORN database path.
    pub fn configured() -> Self {
        Self {
            path: None,
            #[cfg(test)]
            settlement_hook: None,
        }
    }
    /// Count operations and deduplicated deliveries.
    pub fn counts(&self) -> ApiResult<QueueCounts> {
        self.migrate().and_then(|_| {
            self.with_connection(|connection| {
                let count = |state: &str| {
                    connection
                        .query_row("SELECT COUNT(*) FROM webhook_operations WHERE state = ?", params![state], |row| {
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
                        count("cancelled").map(|cancelled| (queued, running, succeeded, failed, cancelled))
                    })
                    .and_then(|(queued, running, succeeded, failed, cancelled)| {
                        connection
                            .query_row("SELECT COUNT(*) FROM webhook_deliveries WHERE deduplicated = 1", params![], |row| {
                                row.get::<_, i64>(0)
                            })
                            .map(|deduplicated| QueueCounts {
                                queued,
                                running,
                                succeeded,
                                failed,
                                cancelled,
                                deduplicated: u64::try_from(deduplicated).unwrap_or_default(),
                            })
                    })
                    .map_err(|why| eyre!("Failed to count webhook operations — {why}"))
            })
        })
    }
    /// Return a completed external effect result, or `None` while absent or pending.
    pub fn effect_result<T: DeserializeOwned>(&self, key: &str, kind: &str) -> ApiResult<Option<T>> {
        self.migrate()
            .and_then(|_| self.with_connection(|connection| connection.effect_result(key, kind)))
    }
    /// Atomically persist a delivery and its canonical operation.
    pub fn enqueue(&self, delivery_id: &str, operation_key: &str, event_json: &str) -> ApiResult<EnqueueStatus> {
        self.migrate()
            .and_then(|_| self.with_transaction(|connection| connection.enqueue_notification(delivery_id, operation_key, event_json)))
    }
    /// Persist one outbound callback intent without storing its URL or credentials.
    pub fn enqueue_callback<T: Serialize>(&self, callback_key: &str, destination: &str, payload: &T) -> ApiResult<bool> {
        match (callback_key.trim().is_empty(), destination.trim().is_empty()) {
            | (true, _) => Err(eyre!("Webhook callback key is required")),
            | (_, true) => Err(eyre!("Webhook callback destination name is required")),
            | _ => serde_json::to_string(payload)
                .map_err(|why| eyre!("Failed to encode webhook callback payload — {why}"))
                .and_then(|payload_json| {
                    self.migrate().and_then(|_| {
                        self.with_connection(|connection| {
                            let now = to_rfc3339(Timestamp::now());
                            connection
                                .execute(
                                    "INSERT INTO webhook_callbacks
                                     (callback_key, destination, payload_json, state, attempts, available_at, created_at, updated_at)
                                     VALUES (?, ?, ?, 'queued', 0, ?, ?, ?) ON CONFLICT(callback_key) DO NOTHING",
                                    params![callback_key, destination, payload_json, now, now, now],
                                )
                                .map(|inserted| inserted == 1)
                                .map_err(|why| eyre!("Failed to record webhook callback intent — {why}"))
                        })
                    })
                }),
        }
    }
    /// Persist every operation selected from one normalized delivery.
    pub fn enqueue_prepared<T: Serialize>(&self, prepared: &PreparedDelivery<T>) -> ApiResult<Vec<EnqueueStatus>> {
        match prepared.operations.first() {
            | None => Ok(Vec::new()),
            | Some(first) => {
                let delivery_id = format!("{}:{}", first.provider, prepared.delivery.delivery_id);
                let first_operation_key = format!("{}:{}", first.provider, first.idempotency_key);
                serde_json::to_string(&prepared.delivery)
                    .map_err(|why| eyre!("Failed to encode normalized webhook delivery — {why}"))
                    .and_then(|delivery_json| {
                        self.migrate().and_then(|_| {
                            self.with_transaction(|connection| {
                                let now = to_rfc3339(Timestamp::now());
                                connection
                                    .execute(
                                        "INSERT INTO webhook_deliveries (delivery_id, operation_key, event_json, received_at, deduplicated)
                                         VALUES (?, ?, ?, ?, 0) ON CONFLICT(delivery_id) DO NOTHING",
                                        params![delivery_id, first_operation_key, delivery_json, now],
                                    )
                                    .map_err(|why| eyre!("Failed to record webhook delivery — {why}"))
                                    .and_then(|inserted| match inserted {
                                        | 0 => Ok(vec![EnqueueStatus::DuplicateDelivery; prepared.operations.len()]),
                                        | _ => prepared
                                            .operations
                                            .iter()
                                            .map(|operation| {
                                                let operation_key = format!("{}:{}", operation.provider, operation.idempotency_key);
                                                serde_json::to_string(&operation.notification)
                                                    .map_err(|why| eyre!("Failed to encode webhook operation notification — {why}"))
                                                    .and_then(|notification| {
                                                        connection
                                                            .execute(
                                                                "INSERT INTO webhook_operations
                                                                 (operation_key, delivery_id, event_json, state, attempts, available_at, created_at, updated_at)
                                                                 VALUES (?, ?, ?, 'queued', 0, ?, ?, ?) ON CONFLICT(operation_key) DO NOTHING",
                                                                params![operation_key, delivery_id, notification, now, now, now],
                                                            )
                                                            .map(|inserted| match inserted {
                                                                | 0 => EnqueueStatus::DuplicateOperation,
                                                                | _ => EnqueueStatus::Inserted,
                                                            })
                                                            .map_err(|why| eyre!("Failed to record webhook operation — {why}"))
                                                    })
                                            })
                                            .collect::<ApiResult<Vec<_>>>()
                                            .and_then(|statuses| match statuses.contains(&EnqueueStatus::DuplicateOperation) {
                                                | true => connection
                                                    .execute(
                                                        "UPDATE webhook_deliveries SET deduplicated = 1 WHERE delivery_id = ?",
                                                        params![delivery_id],
                                                    )
                                                    .map_err(|why| eyre!("Failed to mark duplicate webhook operation — {why}"))
                                                    .map(|_| statuses),
                                                | false => Ok(statuses),
                                            }),
                                    })
                            })
                        })
                    })
            }
        }
    }
    /// Persist a provider-namespaced JSON-RPC notification.
    pub fn enqueue_spec(&self, delivery_id: &str, operation: &OperationSpec) -> ApiResult<EnqueueStatus> {
        let delivery_id = format!("{}:{delivery_id}", operation.provider);
        let operation_key = format!("{}:{}", operation.provider, operation.idempotency_key);
        serde_json::to_string(&operation.notification)
            .map_err(|why| eyre!("Failed to encode webhook operation notification — {why}"))
            .and_then(|notification| self.enqueue(&delivery_id, &operation_key, &notification))
    }
    /// Consume a running claim and schedule a bounded retry, or make it terminal after five attempts.
    pub fn fail(&self, operation: ClaimedOperation, error: &str) -> ApiResult<OperationState> {
        let operation_key = operation.operation_key.clone();
        let attempts = operation.attempts;
        let state = OperationState::from(operation);
        let exponent = attempts.saturating_sub(1).min(8);
        let backoff = 1_i64
            .checked_shl(exponent)
            .unwrap_or(MAX_WEBHOOK_OPERATION_BACKOFF_SECONDS)
            .min(MAX_WEBHOOK_OPERATION_BACKOFF_SECONDS);
        let now = Timestamp::now();
        let available_at = now.checked_add(SignedDuration::from_secs(backoff)).unwrap_or(now);
        self.update_state(operation_key, attempts, state, Some(error), available_at)
            .map(|_| state)
    }
    /// Record a callback failure and schedule a bounded retry.
    pub fn fail_callback(&self, callback: ClaimedCallback, error: &str) -> ApiResult<OperationState> {
        let callback_key = callback.callback_key.clone();
        let attempts = callback.attempts;
        let state = OperationState::from(callback);
        let exponent = attempts.saturating_sub(1).min(8);
        let backoff = 1_i64
            .checked_shl(exponent)
            .unwrap_or(MAX_WEBHOOK_OPERATION_BACKOFF_SECONDS)
            .min(MAX_WEBHOOK_OPERATION_BACKOFF_SECONDS);
        let now = Timestamp::now();
        let available_at = now.checked_add(SignedDuration::from_secs(backoff)).unwrap_or(now);
        self.settle_callback(callback_key, attempts, state, None, Some(error), available_at)
            .map(|_| state)
    }
    pub(crate) fn initialize(&self, stale_after: SignedDuration) -> ApiResult<RuntimeTimestamps> {
        self.migrate().and_then(|_| {
            self.with_transaction(|connection| {
                let now = Timestamp::now();
                let stale_before = to_rfc3339(now.checked_sub(stale_after).unwrap_or(now));
                let now = to_rfc3339(now);
                connection
                    .execute(
                        "UPDATE webhook_callbacks SET state = 'queued', claimed_at = NULL, available_at = ?, updated_at = ?
                         WHERE state = 'running' AND claimed_at < ?",
                        params![now, now, stale_before],
                    )
                    .map_err(|why| eyre!("Failed to recover stale webhook callbacks — {why}"))
                    .and_then(|_| {
                        connection
                            .execute(
                                "UPDATE webhook_operations
                                 SET state = 'queued', claimed_at = NULL, available_at = ?, updated_at = ?
                                 WHERE state = 'running' AND claimed_at < ?",
                                params![now, now, stale_before],
                            )
                            .map_err(|why| eyre!("Failed to recover stale webhook operations — {why}"))
                    })
                    .and_then(|_| connection.read_runtime_timestamps())
            })
        })
    }
    /// Read a previously admitted payload without resolving current configuration.
    pub fn intake_admission(&self, operation_key: &str) -> ApiResult<Option<IntakeAdmission>> {
        self.migrate()
            .and_then(|()| self.with_connection(|connection| connection.read_intake_admission(operation_key)))
    }
    /// Return durable observations for one credential-free source identity.
    pub fn intake_records(&self, source: &str) -> ApiResult<Vec<IntakeRecord>> {
        self.migrate().and_then(|()| {
            self.with_connection(|connection| {
                connection
                    .prepare("SELECT record_json FROM webhook_intake_records WHERE source = ? ORDER BY commit_sha, line")
                    .map_err(|why| eyre!("Failed to prepare intake observations — {why}"))
                    .and_then(|mut statement| {
                        statement
                            .query_map(params![source], |row| row.get::<_, String>(0))
                            .map_err(|why| eyre!("Failed to read intake observations — {why}"))
                            .and_then(|rows| {
                                rows.map(|row| {
                                    row.map_err(|why| eyre!("Failed to read intake observation — {why}"))
                                        .and_then(|json| serde_json::from_str(&json).map_err(Into::into))
                                })
                                .collect()
                            })
                    })
            })
        })
    }
    /// Create the durable queue tables when absent.
    pub fn migrate(&self) -> ApiResult<()> {
        self.with_connection(|connection| {
            connection
                .execute_batch(&format!(
                    "{WEBHOOK_MIGRATION}\n{WEBHOOK_EFFECT_MIGRATION}\n{INTAKE_MIGRATION}\n{GITLAB_EVENT_CURSOR_MIGRATION}"
                ))
                .map_err(|why| eyre!("Failed to migrate webhook operation queue — {why}"))
        })
    }
    /// Atomically retain an observation, its immutable admission, and queued operation.
    ///
    /// Existing identities require exact bytes. Legacy operations without intake evidence remain conflicts.
    pub fn record_intake(&self, record: &IntakeRecord, operation: Option<&OperationSpec>) -> ApiResult<IntakeRecord> {
        self.migrate().and_then(|()| {
            self.with_transaction(|connection| {
                record
                    .prepare(connection, operation)
                    .and_then(|outcome| outcome.persist(connection).map(|()| outcome))
            })
        })
    }
    /// Requeue completed, cancelled, or terminal work for an explicit replay.
    pub fn replay(&self, operation_key: &str) -> ApiResult<bool> {
        self.migrate().and_then(|_| {
            self.with_connection(|connection| {
                let now = to_rfc3339(Timestamp::now());
                connection
                    .execute(
                        "UPDATE webhook_operations
                         SET state = 'queued', attempts = 0, available_at = ?, claimed_at = NULL, last_error = NULL, updated_at = ?
                         WHERE operation_key = ? AND state IN ('succeeded', 'terminal-failure', 'cancelled')",
                        params![now, now, operation_key],
                    )
                    .map(|updated| updated == 1)
                    .map_err(|why| eyre!("Failed to replay webhook operation — {why}"))
            })
        })
    }
    pub(crate) fn runtime_timestamps(&self) -> ApiResult<RuntimeTimestamps> {
        self.migrate()
            .and_then(|_| self.with_connection(|connection| connection.read_runtime_timestamps()))
    }
    fn settle_callback(
        &self,
        callback_key: String,
        attempts: u32,
        state: OperationState,
        status_code: Option<u16>,
        error: Option<&str>,
        available_at: Timestamp,
    ) -> ApiResult<()> {
        self.migrate().and_then(|_| {
            self.with_transaction(|connection| {
                let now = to_rfc3339(Timestamp::now());
                connection
                    .execute(
                        "UPDATE webhook_callbacks
                         SET state = ?, available_at = ?, claimed_at = NULL, last_error = ?, updated_at = ?
                         WHERE callback_key = ? AND state = 'running' AND attempts = ?",
                        params![state.to_string(), to_rfc3339(available_at), error, now, callback_key, attempts],
                    )
                    .map_err(|why| eyre!("Failed to settle webhook callback — {why}"))
                    .and_then(|updated| {
                        (updated == 1)
                            .then_some(())
                            .ok_or_else(|| eyre!("Webhook callback claim is stale or already settled"))
                    })
                    .and_then(|()| {
                        connection
                            .execute(
                                "INSERT INTO webhook_callback_attempts
                                 (callback_key, attempt, status_code, error, attempted_at) VALUES (?, ?, ?, ?, ?)",
                                params![callback_key, attempts, status_code, error, now],
                            )
                            .map(|_| ())
                            .map_err(|why| eyre!("Failed to record webhook callback attempt — {why}"))
                    })
                    .and_then(|()| connection.record_outcome(state, &now))
            })
        })
    }
    /// Read the current state for one operation.
    pub fn state(&self, operation_key: &str) -> ApiResult<Option<OperationState>> {
        self.migrate().and_then(|_| {
            self.with_connection(|connection| {
                let result = connection.query_row(
                    "SELECT state FROM webhook_operations WHERE operation_key = ?",
                    params![operation_key],
                    |row| row.get::<_, String>(0),
                );
                match result {
                    | Ok(value) => OperationState::try_from(value.as_str()).map(Some),
                    | Err(why) if is_no_rows(&why) => Ok(None),
                    | Err(why) => Err(eyre!("Failed to read webhook operation state — {why}")),
                }
            })
        })
    }
    /// Consume a running claim and mark its matching attempt successful.
    pub fn succeed(&self, operation: ClaimedOperation) -> ApiResult<()> {
        self.update_state(
            operation.operation_key,
            operation.attempts,
            OperationState::Succeeded,
            None,
            Timestamp::now(),
        )
    }
    /// Mark one claimed callback successful and record the HTTP status.
    pub fn succeed_callback(&self, callback: ClaimedCallback, status_code: u16) -> ApiResult<()> {
        self.settle_callback(
            callback.callback_key,
            callback.attempts,
            OperationState::Succeeded,
            Some(status_code),
            None,
            Timestamp::now(),
        )
    }
    /// Mark one external effect successful and persist its typed reconciliation result.
    pub fn succeed_effect<T: Serialize>(&self, key: &str, kind: &str, result: &T) -> ApiResult<()> {
        serde_json::to_string(result)
            .map_err(|why| eyre!("Failed to encode webhook effect result — {why}"))
            .and_then(|result_json| {
                self.begin_effect(key, kind)
                    .and_then(|()| self.with_connection(|connection| connection.succeed_effect(key, kind, &result_json)))
            })
    }
    fn update_state(
        &self,
        operation_key: String,
        attempts: u32,
        state: OperationState,
        error: Option<&str>,
        available_at: Timestamp,
    ) -> ApiResult<()> {
        self.migrate().and_then(|_| self.before_settlement()).and_then(|_| {
            self.with_transaction(|connection| {
                let now = to_rfc3339(Timestamp::now());
                connection
                    .execute(
                        "UPDATE webhook_operations
                         SET state = ?, available_at = ?, claimed_at = NULL, last_error = ?, updated_at = ?
                         WHERE operation_key = ? AND state = 'running' AND attempts = ?",
                        params![state.to_string(), to_rfc3339(available_at), error, now, operation_key, attempts],
                    )
                    .map_err(|why| eyre!("Failed to update webhook operation — {why}"))
                    .and_then(|updated| {
                        (updated == 1)
                            .then_some(())
                            .ok_or_else(|| eyre!("Bot operation `{operation_key}` attempt {attempts} is stale or already settled"))
                    })
                    .and_then(|()| connection.record_outcome(state, &now))
            })
        })
    }
    pub(crate) fn with_connection<T>(&self, operation: impl FnOnce(&Connection) -> ApiResult<T>) -> ApiResult<T> {
        CONNECTION_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .map_err(|why| eyre!("Failed to acquire webhook database lock — {why}"))
            .and_then(|_guard| {
                resolve_database_path(self.path.as_ref())
                    .and_then(|path| Connection::open(path).map_err(|why| eyre!("Failed to open webhook database — {why}")))
                    .and_then(|connection| operation(&connection))
            })
    }
    #[cfg(test)]
    pub(crate) fn with_settlement_hook(self, settlement_hook: fn() -> ApiResult<()>) -> Self {
        Self {
            settlement_hook: Some(settlement_hook),
            ..self
        }
    }
    fn with_transaction<T>(&self, operation: impl FnOnce(&Connection) -> ApiResult<T>) -> ApiResult<T> {
        self.with_connection(|connection| {
            connection
                .execute_batch("BEGIN TRANSACTION")
                .map_err(|why| eyre!("Failed to begin webhook transaction — {why}"))
                .and_then(|_| match operation(connection) {
                    | Ok(value) => connection
                        .execute_batch("COMMIT")
                        .map_err(|why| eyre!("Failed to commit webhook transaction — {why}"))
                        .map(|_| value),
                    | Err(why) => {
                        connection.execute_batch("ROLLBACK").ok();
                        Err(why)
                    }
                })
        })
    }
}
impl fmt::Display for OperationState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::Queued => "queued",
            | Self::Running => "running",
            | Self::Succeeded => "succeeded",
            | Self::RetryableFailure => "retryable-failure",
            | Self::TerminalFailure => "terminal-failure",
            | Self::Cancelled => "cancelled",
        })
    }
}
impl From<ClaimedCallback> for OperationState {
    fn from(callback: ClaimedCallback) -> Self {
        match callback.attempts >= MAX_WEBHOOK_OPERATION_ATTEMPTS {
            | true => Self::TerminalFailure,
            | false => Self::RetryableFailure,
        }
    }
}
impl From<ClaimedOperation> for OperationState {
    fn from(operation: ClaimedOperation) -> Self {
        match operation.attempts >= MAX_WEBHOOK_OPERATION_ATTEMPTS {
            | true => Self::TerminalFailure,
            | false => Self::RetryableFailure,
        }
    }
}
impl TryFrom<&str> for OperationState {
    type Error = color_eyre::Report;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            | "queued" => Ok(Self::Queued),
            | "running" => Ok(Self::Running),
            | "succeeded" => Ok(Self::Succeeded),
            | "retryable-failure" => Ok(Self::RetryableFailure),
            | "terminal-failure" => Ok(Self::TerminalFailure),
            | "cancelled" => Ok(Self::Cancelled),
            | _ => Err(eyre!("Unknown webhook operation state `{value}`")),
        }
    }
}
impl OperationWorker {
    /// Construct a worker over one durable store and operation registry.
    pub fn new(queue: OperationQueue, registry: JsonRpcOperationRegistry, context: InvocationContext, stale_after: SignedDuration) -> Self {
        Self {
            context,
            queue,
            registry,
            runtime: WebhookRuntime::default(),
            stale_after,
        }
    }
    /// Claim and dispatch one operation, returning whether work was available.
    pub async fn run_once(&self) -> ApiResult<bool> {
        match self.runtime.initialize(&self.queue, self.stale_after) {
            | Err(why) => Err(why),
            | Ok(()) => match self.runtime.claim(|| self.queue.claim_next(self.stale_after)) {
                | Ok(Some((operation, _claim))) => {
                    let operation_key = operation.operation_key().to_string();
                    let result = match operation.notification() {
                        | Ok(notification) => self
                            .registry
                            .invoke(&notification.method, notification.params, self.context.clone())
                            .await
                            .map(|_| ())
                            .map_err(|why| eyre!(why)),
                        | Err(why) => Err(why),
                    };
                    match result {
                        | Ok(()) => match self.queue.succeed(operation) {
                            | Ok(()) => self.runtime.refresh(&self.queue).map(|()| true),
                            | Err(why) => {
                                self.runtime.mark_unavailable();
                                Err(eyre!("Failed to settle successful webhook operation `{operation_key}` — {why}"))
                            }
                        },
                        | Err(why) => {
                            let message = why.to_string();
                            match self.queue.fail(operation, &message) {
                                | Ok(_) => self
                                    .runtime
                                    .refresh(&self.queue)
                                    .and_then(|()| Err(eyre!("Webhook operation `{operation_key}` failed — {message}"))),
                                | Err(why) => {
                                    self.runtime.mark_unavailable();
                                    Err(why)
                                }
                            }
                        }
                    }
                }
                | Ok(None) => Ok(false),
                | Err(why) => Err(why),
            },
        }
    }
    /// Use a shared runtime for readiness and drain coordination.
    pub fn with_runtime(self, runtime: WebhookRuntime) -> Self {
        Self { runtime, ..self }
    }
}
#[cfg(not(feature = "duckdb"))]
fn is_no_rows(error: &crate::io::database::backend::Error) -> bool {
    matches!(error, QueryReturnedNoRows)
}
#[cfg(feature = "duckdb")]
fn is_no_rows(error: &crate::io::database::backend::Error) -> bool {
    error.to_string().contains("Query returned no rows")
}
fn parse_timestamp(value: Option<String>, field: &str) -> ApiResult<Option<Timestamp>> {
    value
        .map(|value| {
            value
                .parse::<Timestamp>()
                .map_err(|why| eyre!("Failed to parse webhook runtime {field} — {why}"))
        })
        .transpose()
}
