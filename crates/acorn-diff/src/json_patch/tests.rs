use super::{DiffValue, Operation, OperationKind, PointerError, Rfc6902};
use alloc::string::{String, ToString};
use proptest::collection::{btree_map, vec};
use proptest::prelude::*;
use serde_json::{json, Value};

fn assert_patch_applies(before: Value, after: &Value) {
    let encoded = serde_json::to_value(before.diff(after)).expect("serializable patch");
    let oracle: json_patch::Patch = serde_json::from_value(encoded).expect("valid RFC 6902 patch");
    let mut actual = before;
    json_patch::patch(&mut actual, &oracle).expect("patch applies");
    assert_eq!(&actual, after);
}
fn json_key() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z]{1,8}",
        Just(String::new()),
        Just("/".to_string()),
        Just("~".to_string()),
        Just("a/b~c".to_string()),
        vec(any::<char>(), 0..8).prop_map(|characters| characters.into_iter().collect()),
    ]
}
fn json_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|value| Value::Number(value.into())),
        ".{0,12}".prop_map(Value::String),
    ];
    leaf.prop_recursive(4, 64, 8, |inner| {
        prop_oneof![
            vec(inner.clone(), 0..8).prop_map(Value::Array),
            btree_map(json_key(), inner, 0..8).prop_map(|values| Value::Object(values.into_iter().collect())),
        ]
    })
}

#[test]
fn test_escapes_json_pointer_tokens_and_applies() {
    let before = json!({"": 1, "a/b": 2, "m~n": 3, "snowman-☃": 4});
    let after = json!({"": 0, "a/b": 2, "m~n": 30, "snowman-☃": 4, "x/y~z": 5});
    assert_eq!(
        serde_json::to_value(before.diff(&after)).expect("serializable patch"),
        json!([
            {"op":"replace","path":"/","value":0},
            {"op":"replace","path":"/m~0n","value":30},
            {"op":"add","path":"/x~1y~0z","value":5}
        ])
    );
    assert_patch_applies(before, &after);
}
#[test]
fn test_generated_patch_applies_and_sorts_object_keys() {
    let before = json!({"z": 0, "remove": true, "array": [1, 2, 3]});
    let after = json!({"a": 1, "z": 2, "array": [1, 4, 5, 6]});
    let patch = before.diff(&after);
    let encoded = serde_json::to_value(&patch).expect("serializable patch");
    let oracle: json_patch::Patch = serde_json::from_value(encoded.clone()).expect("valid patch");
    let mut actual = before;
    json_patch::patch(&mut actual, &oracle).expect("patch applies");
    assert_eq!(actual, after);
    assert_eq!(encoded.pointer("/0/path"), Some(&json!("/remove")));
}
proptest! {
    #[test]
    fn test_generated_patch_conforms_to_oracle(before in json_value(), after in json_value()) {
        let patch = before.diff(&after);
        let encoded = serde_json::to_value(&patch).expect("serializable patch");
        let oracle: json_patch::Patch = serde_json::from_value(encoded).expect("valid RFC 6902 patch");
        let mut actual = before;
        json_patch::patch(&mut actual, &oracle).expect("generated patch applies");
        prop_assert_eq!(actual, after);
    }
}
#[test]
fn test_handles_empty_collections_and_root_replacement() {
    let empty = json!({});
    assert!(empty.diff(&json!({})).is_empty());
    let before = json!({"value": 1});
    let after = json!([]);
    assert_eq!(
        serde_json::to_value(before.diff(&after)).expect("serializable patch"),
        json!([{"op":"replace","path":"","value":[]}])
    );
    assert_patch_applies(before, &after);
    assert_eq!(
        serde_json::to_value(json!([]).diff(&json!([1]))).expect("serializable patch"),
        json!([{"op":"add","path":"/-","value":1}])
    );
}
#[test]
fn test_rejects_invalid_pointers_and_move_descendants() {
    assert!(Operation::remove("").is_ok());
    assert!(Operation::remove("/~0/~1").is_ok());
    assert!(matches!(Operation::remove("a"), Err(PointerError::MissingLeadingSlash(_))));
    assert!(matches!(Operation::remove("/~"), Err(PointerError::InvalidEscape(_))));
    assert!(matches!(Operation::remove("/~2"), Err(PointerError::InvalidEscape(_))));
    assert!(matches!(Operation::move_("", "/a"), Err(PointerError::MoveIntoChild { .. })));
    assert!(matches!(Operation::move_("/a", "/a/b"), Err(PointerError::MoveIntoChild { .. })));
    assert!(Operation::move_("/a", "/a").is_ok());
    assert!(Operation::move_("/a", "/ab").is_ok());
    assert!(Operation::move_("/a", "/a~1b").is_ok());
}
#[test]
fn test_serializes_every_operation_shape() {
    let operations = [
        Operation::add("/a", json!(1)).expect("valid pointer"),
        Operation::copy("/a", "/b").expect("valid pointers"),
        Operation::move_("/a", "/b").expect("valid pointers"),
        Operation::remove("/a").expect("valid pointer"),
        Operation::replace("/a", json!(2)).expect("valid pointer"),
        Operation::test("/a", json!(1)).expect("valid pointer"),
    ];
    assert_eq!(operations[0].kind(), OperationKind::Add);
    assert_eq!(
        serde_json::to_value(operations).expect("serializable"),
        json!([
            {"op":"add","path":"/a","value":1},
            {"op":"copy","from":"/a","path":"/b"},
            {"op":"move","from":"/a","path":"/b"},
            {"op":"remove","path":"/a"},
            {"op":"replace","path":"/a","value":2},
            {"op":"test","path":"/a","value":1}
        ])
    );
}
