//! Generated browser operation wire-format laws.
#![cfg(not(target_arch = "wasm32"))]
use acorn_web::operations::{Reply, Request, Tool};
use proptest::{
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence},
};
use serde_json::Value;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROPTEST_CASES").ok().and_then(|value| value.parse().ok()).unwrap_or(256),
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/proptest-regressions/operations.txt"))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn test_operation_request_round_trips_generated_messages(
        tool in prop::sample::select(vec![Tool::Activity, Tool::BigO, Tool::Crosswalk, Tool::Diff, Tool::Discover, Tool::Fair, Tool::Inspect, Tool::Readability, Tool::Standard]),
        input in any::<String>(),
        after in any::<String>(),
        contact in any::<String>(),
        skip in prop::collection::vec(any::<String>(), 0..8),
    ) {
        let request = Request { input, after, contact, skip, ..Request::default() };
        let encoded = serde_json::to_string(&(tool, &request)).expect("serialize operation request");
        let decoded: (Tool, Request) = serde_json::from_str(&encoded).expect("deserialize operation request");
        prop_assert_eq!(decoded, (tool, request));
    }
    #[test]
    fn test_worker_reply_round_trips_generated_values(
        text in any::<String>(),
        number in any::<i64>(),
        success in any::<bool>(),
    ) {
        let reply = if success {
            Reply::Ok(serde_json::json!({"text": text, "number": number}))
        } else {
            Reply::Error(text)
        };
        let encoded = serde_json::to_string(&reply).expect("serialize worker reply");
        let decoded: Reply = serde_json::from_str(&encoded).expect("deserialize worker reply");
        prop_assert_eq!(decoded, reply);
        prop_assert!(matches!(serde_json::from_str::<Value>(&encoded), Ok(Value::Object(_))));
    }
}
