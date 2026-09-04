use crate::io::api::gitlab::database::{EnqueueStatus, OperationQueue, OperationState};
use crate::io::database::backend::params;
use crate::io::ApiResult;
use crate::util::constants::app::MAX_GITLAB_OPERATION_ATTEMPTS;
use acorn_core::util::to_rfc3339;
use color_eyre::eyre::eyre;
use jiff::{SignedDuration, Timestamp};
use nanoid::nanoid;
use std::env::temp_dir;

fn queue() -> OperationQueue {
    OperationQueue::from_path(temp_dir().join(format!("acorn-bot-{}.db", nanoid!())))
}

fn settlement_failure() -> ApiResult<()> {
    Err(eyre!("injected settlement failure"))
}

#[test]
fn test_claims_retries_and_retains_terminal_failure() {
    let queue = queue();
    queue.enqueue("delivery-1", "work-item-check:1:2:3", "{}").unwrap();
    let mut operation = Some(queue.claim_next(SignedDuration::from_mins(5)).unwrap().unwrap());
    for expected_attempt in 1..=MAX_GITLAB_OPERATION_ATTEMPTS {
        let claim = operation.take().unwrap();
        assert_eq!(claim.attempts(), expected_attempt);
        let operation_key = claim.operation_key().to_string();
        let state = queue.fail(claim, "temporary").unwrap();
        if expected_attempt < MAX_GITLAB_OPERATION_ATTEMPTS {
            assert_eq!(state, OperationState::RetryableFailure);
            queue
                .with_connection(|connection| {
                    connection
                        .execute(
                            "UPDATE bot_operations SET available_at = ? WHERE operation_key = ?",
                            params![
                                to_rfc3339(Timestamp::now().checked_sub(SignedDuration::from_secs(1)).unwrap()),
                                operation_key
                            ],
                        )
                        .map(|_| ())
                        .map_err(|why| eyre!("{why}"))
                })
                .unwrap();
            operation = queue.claim_next(SignedDuration::from_mins(5)).unwrap();
        } else {
            assert_eq!(state, OperationState::TerminalFailure);
            assert_eq!(queue.state(&operation_key).unwrap(), Some(OperationState::TerminalFailure));
        }
    }
    assert_eq!(queue.counts().unwrap().failed, 1);
}

#[test]
fn test_duplicate_delivery_and_operation_are_idempotent() {
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
fn test_older_attempt_cannot_settle_recovered_operation() {
    let queue = queue();
    queue.enqueue("delivery-1", "mr-check:1:2:abc", "{}").unwrap();
    let stale = queue.claim_next(SignedDuration::from_mins(5)).unwrap().unwrap();
    queue
        .with_connection(|connection| {
            connection
                .execute(
                    "UPDATE bot_operations SET claimed_at = ? WHERE operation_key = ?",
                    params![
                        to_rfc3339(Timestamp::now().checked_sub(SignedDuration::from_mins(10)).unwrap()),
                        stale.operation_key()
                    ],
                )
                .map(|_| ())
                .map_err(|why| eyre!("{why}"))
        })
        .unwrap();
    let current = queue.claim_next(SignedDuration::from_mins(5)).unwrap().unwrap();
    let operation_key = current.operation_key().to_string();
    assert_eq!(current.attempts(), 2);
    assert!(queue.succeed(stale).unwrap_err().to_string().contains("stale or already settled"));
    assert_eq!(queue.state(&operation_key).unwrap(), Some(OperationState::Running));
    queue.succeed(current).unwrap();
    assert_eq!(queue.state(&operation_key).unwrap(), Some(OperationState::Succeeded));
}

#[test]
#[cfg(not(feature = "duckdb"))]
fn test_schema_does_not_store_raw_webhook_bodies() {
    let queue = queue();
    queue.migrate().unwrap();
    queue
        .with_connection(|connection| {
            connection
                .query_row("SELECT sql FROM sqlite_master WHERE name = 'bot_webhook_deliveries'", params![], |row| {
                    row.get::<_, String>(0)
                })
                .map_err(|why| eyre!("{why}"))
                .map(|sql| assert!(!sql.contains("raw_body")))
        })
        .unwrap();
}

#[test]
fn test_settlement_failure_leaves_claim_running_for_recovery() {
    let path = temp_dir().join(format!("acorn-bot-{}.db", nanoid!()));
    let queue = OperationQueue::from_path(path.clone()).with_settlement_hook(settlement_failure);
    queue.enqueue("delivery-1", "mr-check:1:2:abc", "{}").unwrap();
    let claim = queue.claim_next(SignedDuration::from_mins(5)).unwrap().unwrap();
    let operation_key = claim.operation_key().to_string();
    assert_eq!(queue.succeed(claim).unwrap_err().to_string(), "injected settlement failure");
    assert_eq!(queue.state(&operation_key).unwrap(), Some(OperationState::Running));
    let recovered = OperationQueue::from_path(path)
        .claim_next(SignedDuration::from_secs(-1))
        .unwrap()
        .unwrap();
    assert_eq!(recovered.attempts(), 2);
}

#[test]
fn test_stale_running_operation_is_recovered_after_restart() {
    let path = temp_dir().join(format!("acorn-bot-{}.db", nanoid!()));
    let queue = OperationQueue::from_path(path.clone());
    queue.enqueue("delivery-1", "mr-check:1:2:abc", "{}").unwrap();
    let claimed = queue.claim_next(SignedDuration::from_mins(5)).unwrap().unwrap();
    queue
        .with_connection(|connection| {
            connection
                .execute(
                    "UPDATE bot_operations SET claimed_at = ? WHERE operation_key = ?",
                    params![
                        to_rfc3339(Timestamp::now().checked_sub(SignedDuration::from_mins(10)).unwrap()),
                        claimed.operation_key()
                    ],
                )
                .map(|_| ())
                .map_err(|why| eyre!("{why}"))
        })
        .unwrap();
    let restarted = OperationQueue::from_path(path);
    assert_eq!(restarted.claim_next(SignedDuration::from_mins(5)).unwrap().unwrap().attempts(), 2);
}
