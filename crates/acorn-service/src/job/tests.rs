use super::*;
use acorn_core::prelude::{vec, ToString, Vec};

#[test]
fn test_coalescing_finish_transitions() {
    assert_eq!(CoalescingState::Running.finish(), Ok(CoalescingState::Idle));
    assert_eq!(CoalescingState::RunningPending.finish(), Ok(CoalescingState::Queued));
}
#[test]
fn test_coalescing_rejects_invalid_start_and_finish() {
    assert_eq!(
        CoalescingState::Idle.start(),
        Err(CoalescingTransitionError::Start {
            state: CoalescingState::Idle
        })
    );
    assert_eq!(
        CoalescingState::Queued.finish(),
        Err(CoalescingTransitionError::Finish {
            state: CoalescingState::Queued
        })
    );
}
#[test]
fn test_coalescing_start_transition() {
    assert_eq!(CoalescingState::Queued.start(), Ok(CoalescingState::Running));
}
#[test]
fn test_coalescing_trigger_transitions() {
    assert_eq!(CoalescingState::Idle.trigger(), CoalescingState::Queued);
    assert_eq!(CoalescingState::Queued.trigger(), CoalescingState::Queued);
    assert_eq!(CoalescingState::Running.trigger(), CoalescingState::RunningPending);
    assert_eq!(CoalescingState::RunningPending.trigger(), CoalescingState::RunningPending);
}
#[test]
fn test_job_id_accepts_boundaries_and_round_trips() {
    for value in ["a".to_string(), "check-project-2".to_string(), "a".repeat(MAX_JOB_ID_BYTES)] {
        let identifier = JobId::new(value.as_str()).expect("valid job identifier");
        let encoded = serde_json::to_string(&identifier).expect("serialize job identifier");
        let decoded = serde_json::from_str::<JobId>(&encoded).expect("deserialize job identifier");
        assert_eq!(decoded, identifier, "identifier {value}");
    }
}
#[test]
fn test_job_id_rejects_invalid_values() {
    let values: Vec<String> = vec![
        "".to_string(),
        "-job".to_string(),
        "job-".to_string(),
        "Job".to_string(),
        "job_name".to_string(),
        "a".repeat(MAX_JOB_ID_BYTES + 1),
    ];
    for value in values {
        assert!(JobId::new(value.as_str()).is_err(), "identifier {value}");
        let encoded = serde_json::to_string(&value).expect("serialize invalid identifier fixture");
        assert!(serde_json::from_str::<JobId>(&encoded).is_err(), "deserialized identifier {value}");
    }
}
#[test]
fn test_job_schedule_accepts_each_trigger_shape() {
    let changes: JobSchedule = serde_json::from_str(r#"{"onChange":true}"#).expect("change schedule");
    let interval: JobSchedule = serde_json::from_str(r#"{"everySeconds":60}"#).expect("interval schedule");
    let both: JobSchedule = serde_json::from_str(r#"{"onChange":true,"everySeconds":60}"#).expect("combined schedule");
    assert_eq!(changes, JobSchedule::for_changes());
    assert_eq!(interval, JobSchedule::for_interval(Seconds::new(60)).expect("positive interval"));
    assert_eq!(
        both,
        JobSchedule::for_change_and_interval(Seconds::new(60)).expect("positive combined interval")
    );
    assert_eq!(
        serde_json::to_value(both).expect("serialize combined schedule"),
        serde_json::json!({"onChange": true, "everySeconds": 60})
    );
}
#[test]
fn test_job_schedule_rejects_invalid_shapes() {
    for value in [r#"{}"#, r#"{"everySeconds":0}"#, r#"{"onChange":true,"unknown":1}"#] {
        assert!(serde_json::from_str::<JobSchedule>(value).is_err(), "schedule {value}");
    }
    assert!(JobSchedule::for_interval(Seconds::new(0)).is_err());
}
#[test]
fn test_job_values_use_stable_serialized_spellings() {
    assert_eq!(
        serde_json::to_value([
            JobTrigger::FileChange,
            JobTrigger::Interval,
            JobTrigger::Manual,
            JobTrigger::Replay,
            JobTrigger::Startup,
        ])
        .expect("job triggers"),
        serde_json::json!(["file-change", "interval", "manual", "replay", "startup"])
    );
    assert_eq!(
        serde_json::to_value([
            RunState::Failed,
            RunState::Interrupted,
            RunState::Queued,
            RunState::Running,
            RunState::Succeeded,
        ])
        .expect("run states"),
        serde_json::json!(["failed", "interrupted", "queued", "running", "succeeded"])
    );
    assert_eq!(
        serde_json::to_value([
            CoalescingState::Idle,
            CoalescingState::Queued,
            CoalescingState::Running,
            CoalescingState::RunningPending,
        ])
        .expect("coalescing states"),
        serde_json::json!(["idle", "queued", "running", "running-pending"])
    );
}
#[test]
fn test_run_state_classifies_terminal_states() {
    assert!(!RunState::Queued.is_terminal());
    assert!(!RunState::Running.is_terminal());
    assert!(RunState::Failed.is_terminal());
    assert!(RunState::Interrupted.is_terminal());
    assert!(RunState::Succeeded.is_terminal());
}
