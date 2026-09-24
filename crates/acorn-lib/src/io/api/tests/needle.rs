use super::super::needle::CompleteResponse;

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
