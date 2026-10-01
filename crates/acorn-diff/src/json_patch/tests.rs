use super::{DiffValue, Operation, OperationKind, PointerError, Rfc6902};
use proptest::collection::{btree_map, vec};
use proptest::prelude::*;
use serde_json::json;

fn json_value() -> impl Strategy<Value = serde_json::Value> {
    let leaf = prop_oneof![
        Just(serde_json::Value::Null),
        any::<bool>().prop_map(serde_json::Value::Bool),
        any::<i64>().prop_map(|value| serde_json::Value::Number(value.into())),
        ".{0,12}".prop_map(serde_json::Value::String),
    ];
    leaf.prop_recursive(4, 64, 8, |inner| {
        prop_oneof![
            vec(inner.clone(), 0..8).prop_map(serde_json::Value::Array),
            btree_map("[a-z]{1,8}", inner, 0..8).prop_map(|values| serde_json::Value::Object(values.into_iter().collect())),
        ]
    })
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
fn test_rejects_invalid_pointers_and_child_moves() {
    assert!(matches!(Operation::remove("a"), Err(PointerError::MissingLeadingSlash(_))));
    assert!(matches!(Operation::remove("/~2"), Err(PointerError::InvalidEscape(_))));
    assert!(matches!(Operation::move_("/a", "/a/b"), Err(PointerError::MoveIntoChild { .. })));
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
