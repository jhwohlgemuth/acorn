use crate::daemon::DaemonStore;
use acorn_service::{JobId, JobTrigger, RunState};
use proptest::{
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence},
};

#[derive(Clone, Debug)]
struct ModelRun {
    id: i64,
    state: &'static str,
    trigger: &'static str,
    exit_code: Option<i32>,
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROPTEST_CASES").ok().and_then(|value| value.parse().ok()).unwrap_or(64),
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/proptest-regressions/store-state.txt"))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn test_generated_store_operations_agree_with_state_model(
        actions in prop::collection::vec((0_u8..6, any::<bool>(), 0_usize..8), 1..60),
    ) {
        let store = DaemonStore::open(":memory:").expect("in-memory store");
        let job = JobId::new("model-job").expect("valid model job");
        store.register_jobs(core::slice::from_ref(&job)).expect("register model job");
        let mut paused = false;
        let mut running = None;
        let mut claimed = Vec::new();
        let mut model = Vec::<ModelRun>::new();
        for (action, flag, limit) in actions {
            match action {
                0 => store.register_jobs(core::slice::from_ref(&job)).expect("reregister job"),
                1 => store.activate_jobs(core::slice::from_ref(&job), &[], &[]).expect("activate job"),
                2 => {
                    let queued = store.enqueue(&job, JobTrigger::Manual).expect("enqueue job");
                    prop_assert_eq!(queued.is_some(), !paused);
                    if let Some(id) = queued {
                        model.push(ModelRun { id, state: "queued", trigger: "manual", exit_code: None });
                    }
                }
                3 => {
                    prop_assert!(store.set_paused(&job, flag).expect("set pause"));
                    paused = flag;
                }
                4 => {
                    let claimed_run = store.claim_next().expect("claim next run");
                    let expected = (!paused && running.is_none())
                        .then(|| model.iter().find(|run| run.state == "queued").map(|run| run.id))
                        .flatten();
                    prop_assert_eq!(claimed_run.as_ref().map(|run| run.id), expected);
                    if let Some(record) = claimed_run {
                        prop_assert!(!claimed.contains(&record.id), "run claimed twice");
                        claimed.push(record.id);
                        running = Some(record.id);
                        let run = model.iter_mut().find(|run| run.id == record.id).expect("claimed run in model");
                        run.state = "running";
                    }
                }
                _ => {
                    if let Some(id) = running {
                        let state = if flag { RunState::Succeeded } else { RunState::Failed };
                        let expected_state = if flag { "succeeded" } else { "failed" };
                        let exit_code = if flag { 0 } else { 2 };
                        store.finish(id, state, Some(exit_code), None, Some("generated run")).expect("finish running job");
                        let actual = store.run(id).expect("read finished run").expect("finished run exists");
                        prop_assert_eq!(actual.state.as_str(), expected_state);
                        prop_assert_eq!(actual.exit_code, Some(exit_code));
                        prop_assert_eq!(actual.detail.as_deref(), Some("generated run"));
                        let run = model.iter_mut().find(|run| run.id == id).expect("finished run in model");
                        run.state = expected_state;
                        run.exit_code = Some(exit_code);
                        running = None;
                    }
                }
            }
            prop_assert_eq!(store.paused(&job).expect("read pause"), Some(paused));
            let actual = store.history(&job, limit).expect("read bounded history");
            let expected = model.iter().rev().take(limit).collect::<Vec<_>>();
            prop_assert_eq!(actual.len(), expected.len());
            for (record, run) in actual.iter().zip(expected) {
                prop_assert_eq!(record.id, run.id);
                prop_assert_eq!(record.state.as_str(), run.state);
                prop_assert_eq!(record.trigger.as_str(), run.trigger);
                prop_assert_eq!(record.exit_code, run.exit_code);
            }
        }
    }
}
