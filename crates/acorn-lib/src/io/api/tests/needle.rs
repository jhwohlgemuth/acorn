use super::super::needle::CompleteResponse;

#[test]
fn test_parses_documented_native_response() {
    let response: CompleteResponse = serde_json::from_value(serde_json::json!({
        "type": "call",
        "success": true,
        "error": null,
        "error_code": null,
        "function_calls": [{ "name": "acorn.version", "arguments": {} }],
        "reasoning": "version requested",
        "confidence": 0.94,
        "prefill_tps": 4300.0,
        "decode_tps": 850.0,
        "peak_ram_mb": 28.0
    }))
    .expect("native response");
    assert_eq!(response.response_type, "call");
    assert_eq!(response.function_calls.first().map(|call| call.name.as_str()), Some("acorn.version"));
}
