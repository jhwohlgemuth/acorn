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
#[test]
fn test_identifier_location_finds_discovery_example_identifiers() {
    let source = include_str!("../../assets/examples/discovery.txt");
    for identifier in Identifier::find_all(source)
        .into_iter()
        .filter(|identifier| identifier.kind.supports_discovery())
        .filter_map(|identifier| identifier.normalized())
    {
        assert!(
            identifier_location(source, identifier.kind.as_ref(), &identifier.value).is_some(),
            "missing source location for {:?}",
            identifier
        );
    }
}
#[test]
fn test_identifier_location_handles_json_escaped_slashes() {
    let source = "é\nhttps:\\/\\/doi.org\\/10.15760\\/etd.6294";
    assert!(identifier_location(source, "doi", "10.15760/etd.6294").is_some());
}
#[test]
fn test_identifier_location_skips_invalid_prefix_and_counts_unicode() {
    let source = "é\nhttps://ror.org/01qz5mb56xxx\nROR 01qz5mb56";
    assert_eq!(identifier_location(source, "ror", "https://ror.org/01qz5mb56"), Some((3, 5, 3, 14)));
}
#[test]
fn test_identifier_locations_mark_repeated_aliases_once_each() {
    let source = "https://ror.org/01qz5mb56 and ROR 01qz5mb56";
    assert_eq!(
        identifier_locations(source, "ror", "https://ror.org/01qz5mb56"),
        vec![(1, 1, 1, 26), (1, 35, 1, 44)]
    );
}
