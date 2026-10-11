#[cfg(feature = "std")]
use super::MimeType;
use super::{
    detect_json, stable_deduplicate, Checksum, ChecksumAlgorithm, ParseOptions, ParseOptionsExt, SemanticVersion, StringConversion, StringExt,
};
use crate::prelude::alloc::{String, ToString};
use crate::util::constants::app::EXEC_HEADER_PREFIX;
use jsonc_parser::parse_to_serde_value;
use serde_json::Value;

#[test]
fn test_checksum_from_parses_borrowed_and_owned_values() {
    let checksum = Checksum::from("SHA512:abc123");
    assert_eq!(checksum.algorithm, ChecksumAlgorithm::Sha512);
    assert_eq!(checksum.checksum_value, "abc123");
    let owned_checksum = Checksum::from("md5:def456".to_string());
    assert_eq!(owned_checksum.algorithm, ChecksumAlgorithm::Md5);
    assert_eq!(owned_checksum.checksum_value, "def456");
}
#[test]
fn test_checksum_from_uses_default_algorithm_for_malformed_values() {
    ["abc123", "sha256:"].into_iter().for_each(|value| {
        let checksum = Checksum::from(value);
        assert_eq!(checksum.algorithm, ChecksumAlgorithm::Sha256);
        assert_eq!(checksum.checksum_value, value);
    });
}
#[test]
fn test_mime_type_from_resolves_file_type_names_and_aliases() {
    assert_eq!(MimeType::from("json"), MimeType::Json);
    assert_eq!(MimeType::from("activity.JSONL"), MimeType::JsonLines);
    assert_eq!(MimeType::from("markdown"), MimeType::Markdown);
    assert_eq!(MimeType::from("activity.ndjson"), MimeType::JsonLines);
    assert_eq!(MimeType::from("powerpoint"), MimeType::Powerpoint);
    assert_eq!(MimeType::from("zon"), MimeType::Zon);
    assert_eq!(MimeType::JsonLines.file_type(), "jsonl");
    assert_eq!(MimeType::JsonLines.to_string(), "application/x-ndjson");
    assert_eq!(
        serde_json::to_string(&MimeType::JsonLines).expect("serialize MIME type"),
        r#""json-lines""#
    );
    assert!(!MimeType::JsonLines.is_repository_file());
}
#[test]
fn test_mime_type_infers_rhai_from_abi_header() {
    let source = b"// @acorn-exec-v0.2.0\nthis is not valid Rhai";
    assert_eq!(MimeType::infer(source), Some(MimeType::Rhai));
}
#[test]
fn test_mime_type_parse_rejects_unsupported_file_types() {
    assert_eq!("json".parse::<MimeType>(), Ok(MimeType::Json));
    assert_eq!(
        "activity.unknown"
            .parse::<MimeType>()
            .expect_err("unknown extension should fail")
            .to_string(),
        "Unsupported file format 'activity.unknown'"
    );
    assert_eq!(
        "unknown".parse::<MimeType>().expect_err("unknown type should fail").to_string(),
        "Unsupported file format 'unknown'"
    );
}
#[test]
fn test_mime_type_recognizes_only_canonical_zon_extension() {
    assert_eq!(MimeType::from("activity.zonf"), MimeType::Zon);
    assert_eq!(MimeType::Zon.file_type(), "zonf");
    assert_eq!(MimeType::Zon.to_string(), "text/zonf");
    assert_eq!(MimeType::from("activity.zon"), MimeType::Vendor("zon".to_string()));
}
#[test]
fn test_parse_options_strict_jsonc_enforces_shared_syntax() {
    let options = ParseOptions::strict_jsonc();
    [
        r#"{"name":"value"}"#,
        "{// line comment\n\"name\":\"value\"}",
        r#"{/* block comment */"name":"value"}"#,
        r#"{"name":"value",}"#,
    ]
    .into_iter()
    .for_each(|content| {
        assert!(
            parse_to_serde_value::<Value>(content, &options).is_ok(),
            "expected valid JSONC: {content}"
        );
        assert!(detect_json(content), "expected JSON detection: {content}");
    });
    [
        r#"{name:"value"}"#,
        r#"{"a":1 "b":2}"#,
        r#"{'name':'value'}"#,
        r#"{"value":0x10}"#,
        r#"{"value":+1}"#,
    ]
    .into_iter()
    .for_each(|content| {
        assert!(
            parse_to_serde_value::<Value>(content, &options).is_err(),
            "expected invalid JSONC: {content}"
        );
        assert!(!detect_json(content), "expected JSON rejection: {content}");
    });
}
#[test]
fn test_semantic_version_extracts_prefixed_version() {
    assert_eq!(
        SemanticVersion::extract("// @acorn-exec-v0.2.0", EXEC_HEADER_PREFIX),
        Some(SemanticVersion::init().major(0).minor(2).patch(0).build())
    );
}
#[test]
fn test_stable_deduplicate_retains_first_occurrence_order() {
    assert_eq!(stable_deduplicate(["beta", "alpha", "beta", "gamma"]), vec!["beta", "alpha", "gamma"]);
}
#[test]
fn test_string_conversion_complete_date() {
    assert_eq!("2024-02-29".to_complete_date(), Some("2024-02-29"));
    assert_eq!("2023-02-29".to_complete_date(), None);
    assert_eq!("2024-02".to_complete_date(), None);
    assert_eq!("2024-02-29T00:00:00Z".to_complete_date(), None);
}
#[test]
fn test_string_conversion_truncates_at_utf8_boundary() {
    assert_eq!("aéz".truncate_utf8(3), "aé");
    assert_eq!("aéz".truncate_utf8(2), "a");
    assert_eq!("aéz".truncate_utf8(4), "aéz");
}
#[test]
fn test_string_ext_default_compaction_is_identity() {
    let value = " value ".to_string();
    assert_eq!(value.clone().trim_and_compact(), value);
}
#[test]
fn test_string_ext_vectors_trim_compact_and_detect_numeric_values() {
    let borrowed = vec![" alpha ", "", " 42 "].trim_and_compact();
    assert_eq!(borrowed, vec!["alpha", "42"]);
    assert!(borrowed.is_numeric());
    let owned = vec![" alpha ".to_string(), String::new(), " 42 ".to_string()].trim_and_compact();
    assert_eq!(owned, vec!["alpha".to_string(), "42".to_string()]);
    assert!(owned.is_numeric());
}
