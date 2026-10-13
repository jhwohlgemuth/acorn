use crate::{
    research_activity::LogbookEntry,
    types::{OneOrMany, OneOrManyCodec},
    validation::Validate,
};
use acorn_core::util::MimeType;
use proptest::{
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence},
};
use serde_json::{json, Value};

fn one_or_many_strings() -> impl Strategy<Value = OneOrMany<String>> {
    prop_oneof![
        "[A-Za-z0-9 _-]{0,32}".prop_map(OneOrMany::One),
        prop::collection::vec("[A-Za-z0-9 _-]{0,32}", 0..6).prop_map(OneOrMany::Many),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/proptest-regressions/types/one-or-many.txt"))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn one_or_many_preserves_shape_through_structured_codecs(value in one_or_many_strings()) {
        for mime in [MimeType::Json, MimeType::Yaml, MimeType::Zon] {
            let content = value.encode(mime.clone()).expect("generated one-or-many value should serialize");
            let parsed = OneOrMany::<String>::parse(&content, mime).expect("serialized one-or-many value should parse");
            prop_assert_eq!(parsed, value.clone());
        }
    }
}
#[test]
fn json_lines_logbook_entries_use_the_generic_ordered_codec() {
    let content = include_str!("../../../../tests/fixtures/jsonl/logbook-entries.jsonl");
    let canonical = content.replace("\r\n", "\n");
    let entries = OneOrMany::<LogbookEntry>::parse(content, MimeType::JsonLines).expect("parse logbook entry records");
    let identifiers = entries.iter().map(|entry| entry.identifier.as_str()).collect::<Vec<_>>();
    assert_eq!(identifiers, vec!["experiment-completed", "software-released"]);
    assert!(entries.iter().all(|entry| entry.validate().is_ok()));
    assert_eq!(entries.encode(MimeType::JsonLines).expect("serialize logbook entry records"), canonical);
}
#[test]
fn json_lines_parse_accepts_lf_crlf_and_optional_final_newline() {
    let content = "{\"name\":\"α\"}\r\n{\"name\":\"β\"}\n";
    let parsed = OneOrMany::<Value>::parse(content, MimeType::JsonLines).expect("parse JSON Lines");
    assert_eq!(parsed, OneOrMany::Many(vec![json!({ "name": "α" }), json!({ "name": "β" })]));
    let without_final_newline = OneOrMany::<Value>::parse("{\"name\":\"α\"}", MimeType::JsonLines).expect("parse one record");
    assert_eq!(without_final_newline, OneOrMany::Many(vec![json!({ "name": "α" })]));
}
#[test]
fn json_lines_parse_rejects_blank_and_malformed_records_with_line_numbers() {
    let blank = OneOrMany::<Value>::parse("{\"id\":1}\n \n{\"id\":2}", MimeType::JsonLines).expect_err("blank record should fail");
    assert!(blank.to_string().contains("line 2"));
    let malformed = OneOrMany::<Value>::parse(include_str!("../../../../tests/fixtures/jsonl/malformed.jsonl"), MimeType::JsonLines)
        .expect_err("invalid record should fail");
    assert!(malformed.to_string().contains("line 2"));
    assert!(malformed.to_string().contains("expected value"));
}
#[test]
fn json_lines_parse_rejects_empty_input_and_bare_carriage_returns() {
    let empty = OneOrMany::<Value>::parse("", MimeType::JsonLines).expect_err("empty input should fail");
    assert!(empty.to_string().contains("cannot be empty"));
    let carriage_return = OneOrMany::<Value>::parse("{\"id\":1}\r{\"id\":2}", MimeType::JsonLines).expect_err("bare carriage return should fail");
    assert!(carriage_return.to_string().contains("line 1"));
    assert!(carriage_return.to_string().contains("carriage return"));
}
#[test]
fn json_lines_serialize_is_compact_ordered_and_final_newline_terminated() {
    let values = OneOrMany::Many(vec![json!({ "id": 1, "name": "α" }), json!({ "id": 2, "name": "β" })]);
    let content = values.encode(MimeType::JsonLines).expect("serialize JSON Lines");
    assert_eq!(content, "{\"id\":1,\"name\":\"α\"}\n{\"id\":2,\"name\":\"β\"}\n");
    let parsed = OneOrMany::<Value>::parse(&content, MimeType::JsonLines).expect("round-trip JSON Lines");
    assert_eq!(parsed, values);
}
#[test]
fn json_lines_serialize_rejects_empty_batches() {
    let values = OneOrMany::<Value>::Many(Vec::new());
    let error = values.encode(MimeType::JsonLines).expect_err("empty batch should fail");
    assert!(error.to_string().contains("cannot be empty"));
}
#[test]
fn json_lines_serialize_writes_one_record_for_single_values() {
    let value = OneOrMany::One(json!({ "id": 1 }));
    assert_eq!(value.encode(MimeType::JsonLines).expect("serialize one record"), "{\"id\":1}\n");
}
