use super::*;
use serde_json::json;

#[test]
fn test_annotate_nested_paths_without_guessing_missing_fields() {
    let source = "{\n  \"meta\": {\n    \"names\": [\"é\", \"Ada\"]\n  }\n}";
    let mut value = json!({"diagnostics": [
        {"path": "meta.names[1]"},
        {"path": "$.meta.names[0]"},
        {"path": "meta.absent"},
        {"path": "meta"}
    ]});
    annotate(&mut value, source, "json");
    let diagnostics = value.get("diagnostics").and_then(Value::as_array).expect("diagnostics array");
    let first = diagnostics.first().expect("first diagnostic");
    let second = diagnostics.get(1).expect("second diagnostic");
    let missing = diagnostics.get(2).expect("missing field diagnostic");
    let object = diagnostics.get(3).expect("object diagnostic");
    assert_eq!(first.get("line"), Some(&json!(3)));
    assert_eq!(first.get("character"), Some(&json!(20)));
    assert_eq!(first.get("endLine"), Some(&json!(3)));
    assert_eq!(first.get("endCharacter"), Some(&json!(25)));
    assert_eq!(second.get("line"), Some(&json!(3)));
    assert_eq!(second.get("endCharacter"), Some(&json!(18)));
    assert!(missing.get("line").is_none());
    assert_eq!(object.get("endLine"), Some(&json!(4)));
    assert_eq!(object.get("endCharacter"), Some(&json!(4)));
}
#[test]
fn test_annotate_syntax_error_location() {
    let mut value = json!({"diagnostics": [{"kind": "parse", "message": "Invalid JSON", "path": null}]});
    annotate(&mut value, "{\n  \"meta\": [\n}", "json");
    let diagnostic = value
        .get("diagnostics")
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .expect("parse diagnostic");
    assert_eq!(diagnostic.get("line"), Some(&json!(3)));
    assert_eq!(diagnostic.get("character"), Some(&json!(1)));
}
