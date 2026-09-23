use super::{decode::decode_value_with_limits, decode_value, encode, Limits, ZonError, ZonErrorCode};
use alloc::string::{String, ToString};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{json, Value};

fn supported_value() -> impl Strategy<Value = Value> {
    let string = any::<String>()
        .prop_filter("ZON excludes most control characters", |value| {
            value
                .chars()
                .all(|character| !character.is_control() || matches!(character, '\n' | '\r' | '\t'))
        })
        .prop_map(Value::String);
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i32>().prop_map(|value| Value::Number(value.into())),
        string,
    ];
    leaf.prop_recursive(4, 64, 8, |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 0..6).prop_map(Value::Array),
            proptest::collection::btree_map("[A-Za-z0-9 _:'\\[\\]{}]{1,12}", inner, 0..6)
                .prop_map(|fields| Value::Object(fields.into_iter().collect())),
        ]
    })
}

#[test]
fn test_canonicalizes_numbers_and_normalizes_accepted_layout() {
    let value = json!({
        "large": serde_json::from_str::<Value>("1e20").unwrap(),
        "small": serde_json::from_str::<Value>("1.25e-3").unwrap(),
        "zero": serde_json::from_str::<Value>("-0.0").unwrap()
    });
    assert_eq!(encode(&value).unwrap(), "large:100000000000000000000\nsmall:0.00125\nzero:0");
    assert_eq!(decode_value("large:1e20  \r\n\r\nsmall:0.00125\r\nzero:-0.0\r\n").unwrap(), value);
}
#[test]
fn test_decodes_pinned_typescript_reference_fixture() {
    let zon = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/zon/reference-aliases.zonf"));
    let expected = serde_json::from_str::<Value>(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/zon/reference-aliases.json"
    )))
    .unwrap();
    let decoded = decode_value(zon).unwrap();
    assert_eq!(decoded, expected);
    assert_eq!(decode_value(&encode(&decoded).unwrap()).unwrap(), expected);
}
#[test]
fn test_decodes_reference_aliases() {
    assert_eq!(
        decode_value("@2:id,name\n1,'Alice'\n2,Bob\n").unwrap(),
        json!([{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}])
    );
    assert_eq!(
        decode_value("a:true\nb:FALSE\nc:nil\nd:1e3").unwrap(),
        json!({"a": true, "b": false, "c": null, "d": 1000.0})
    );
}
#[test]
fn test_encodes_canonical_object_and_table() {
    let value = json!({
        "context": {"task": "Our favorite hikes together", "season": "spring_2025"},
        "users": [
            {"active": true, "id": 1, "name": "Alice"},
            {"active": false, "id": 2, "name": "Bob"}
        ]
    });
    assert_eq!(
        encode(&value).unwrap(),
        "context{season:spring_2025,task:Our favorite hikes together}\nusers:@(2):active,id,name\nT,1,Alice\nF,2,Bob"
    );
    assert_eq!(decode_value(&encode(&value).unwrap()).unwrap(), value);
}
#[test]
fn test_encodes_irregular_arrays_as_quoted_compounds() {
    let value = json!({
        "items": [1, "T", {"id": 2}],
        "literal": "[not-an-array]",
        "literal_array": ["[]"]
    });
    let encoded = encode(&value).unwrap();
    assert!(encoded.contains("items:\"[1,\\\"T\\\",{id:2}]\""));
    assert_eq!(decode_value(&encoded).unwrap(), value);
}
#[test]
fn test_error_from_code_defaults_optional_context() {
    let error = ZonError::from(ZonErrorCode::E104);
    assert_eq!(error.code(), "E104");
    assert_eq!(error.column(), None);
    assert_eq!(error.context(), None);
    assert_eq!(error.line(), None);
    assert_eq!(error.message(), "An object key is empty.");
    assert_eq!(ZonErrorCode::E104.message(), "An object key is empty.");
    assert_eq!(ZonErrorCode::E104.to_string(), "E104");
}
#[test]
fn test_rejects_malformed_strings_numbers_and_document_prefixes() {
    assert_eq!(decode_value("value:\"unterminated").unwrap_err().code(), "E102");
    assert_eq!(decode_value("value:01").unwrap_err().code(), "E103");
    assert_eq!(decode_value("value:1e").unwrap_err().code(), "E103");
    assert_eq!(decode_value("\u{feff}value:1").unwrap_err().code(), "E103");
    assert_eq!(decode_value("value:1\rnext:2").unwrap_err().code(), "E103");
}
#[test]
fn test_reports_duplicate_escape_and_bounded_limit_errors() {
    assert_eq!(decode_value("a:1\na:2").unwrap_err().code(), "E103");
    assert_eq!(decode_value("a:\"bad\\q\"").unwrap_err().code(), "E101");
    let limits = Limits {
        array_items: 1,
        depth: 2,
        document_bytes: 8,
        line_bytes: 4,
        object_keys: 1,
    };
    assert_eq!(decode_value_with_limits("12345", limits).unwrap_err().code(), "E302");
    assert_eq!(decode_value_with_limits("123456789", limits).unwrap_err().code(), "E301");
    assert_eq!(
        decode_value_with_limits(
            "\"[1,2]\"",
            Limits {
                document_bytes: 20,
                line_bytes: 20,
                ..limits
            }
        )
        .unwrap_err()
        .code(),
        "E303"
    );
    assert_eq!(
        decode_value_with_limits(
            "a:1\nb:2",
            Limits {
                document_bytes: 20,
                line_bytes: 20,
                ..limits
            }
        )
        .unwrap_err()
        .code(),
        "E304"
    );
}
#[test]
fn test_reports_strict_table_errors() {
    assert_eq!(decode_value("@(2):id\n1").unwrap_err().code(), "E001");
    assert_eq!(decode_value("@(1):id,name\n1").unwrap_err().code(), "E002");
    assert_eq!(decode_value("@(x):id").unwrap_err().code(), "E003");
    assert_eq!(decode_value("@(0):").unwrap_err().code(), "E004");
}
#[test]
fn test_round_trips_csv_quoting_unicode_and_literal_aliases() {
    let value = json!({
        "rows": [
            {"note": "a,b", "text": "say \"hi\""},
            {"note": "line\nbreak", "text": "'alias' Ω"}
        ]
    });
    let encoded = encode(&value).unwrap();
    assert!(encoded.contains("\"a,b\""));
    assert!(encoded.contains("\"say \"\"hi\"\"\""));
    assert_eq!(decode_value(&encoded).unwrap(), value);
}
proptest! {
    #![proptest_config(ProptestConfig::with_failure_persistence(
        FileFailurePersistence::Direct("tests/proptest-regressions/zon/tests.txt")
    ))]
    #[test]
    fn round_trips_supported_values(value in supported_value()) {
        let encoded = encode(&value).unwrap();
        prop_assert_eq!(decode_value(&encoded).unwrap(), value);
    }
}
