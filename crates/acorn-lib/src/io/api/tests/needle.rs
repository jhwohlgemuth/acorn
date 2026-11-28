use super::super::needle::CompleteResponse;
use acorn_core::Noul;
use serde_json::json;

#[test]
fn test_needle_confidence_drops_out_of_range_numbers_without_weakening_noul() {
    for (input, expected) in [
        (
            json!({"type":"respond","confidence":0.5}),
            Some(Noul::new(0.5).expect("valid confidence")),
        ),
        (json!({"type":"respond","confidence":1.1}), None),
        (json!({"type":"respond","confidence":-0.1}), None),
        (json!({"type":"respond","confidence":null}), None),
        (json!({"type":"respond"}), None),
    ] {
        let response = serde_json::from_value::<CompleteResponse>(input.clone()).expect("Needle response");
        assert_eq!(response.confidence, expected, "{input}");
    }
    assert!(serde_json::from_value::<CompleteResponse>(json!({"type":"respond","confidence":"invalid"})).is_err());
    assert!(serde_json::from_value::<Noul>(json!(1.1)).is_err());
}
#[test]
fn test_parses_documented_native_response() {
    let response: CompleteResponse =
        serde_json::from_str(include_str!("../../../../../../tests/fixtures/needle/complete-call-v3.json")).expect("native response");
    assert!(response.response_type.is_call());
    assert_eq!(response.function_calls.first().map(|call| call.name.as_str()), Some("acorn.version"));
    assert!(response.suppressed_calls.is_empty());
    assert!(response
        .validation
        .is_some_and(|validation| validation.ungrounded.is_empty() && !validation.negation));
}
