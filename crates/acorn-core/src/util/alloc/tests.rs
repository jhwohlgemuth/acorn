#[cfg(feature = "std")]
use super::MimeType;
use super::{
    detect_json, escape_ntriples_literal, expand_brace_glob, merge_unique, merge_unique_by, redact_sensitive_text, split_list, stable_deduplicate,
    to_ascii_alphanumeric, Checksum, ChecksumAlgorithm, ParseOptions, ParseOptionsExt, SemanticVersion, StringConversion, StringExt,
    StringInterpolation,
};
use crate::prelude::alloc::{String, ToString};
use crate::util::constants::app::EXEC_HEADER_PREFIX;
use jsonc_parser::parse_to_serde_value;
use proptest::prelude::*;
use serde_json::Value;

proptest! {
    #[test]
    fn test_ascii_alphanumeric_is_idempotent(input in any::<String>()) {
        let normalized = to_ascii_alphanumeric(&input);
        prop_assert!(normalized.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit()));
        prop_assert_eq!(&normalized, &to_ascii_alphanumeric(&normalized));
    }
    #[test]
    fn test_brace_expansion_only_substitutes_choices(
        prefix in "[a-z]{0,8}", first in "[a-z]{1,8}", second in "[a-z]{1,8}", suffix in "[a-z]{0,8}",
    ) {
        let source = format!("{prefix}{{{first},{second}}}{suffix}");
        let expanded = expand_brace_glob(&source).expect("generated brace expression");
        prop_assert_eq!(expanded, vec![format!("{prefix}{first}{suffix}"), format!("{prefix}{second}{suffix}")]);
        let plain = format!("{prefix}{suffix}");
        prop_assert_eq!(expand_brace_glob(&plain).expect("brace-free expression"), vec![plain]);
    }
    #[test]
    fn test_merge_unique_by_has_no_equal_pair(
        left in proptest::collection::vec(any::<u8>(), 0..40),
        right in proptest::collection::vec(any::<u8>(), 0..40),
    ) {
        let merged = merge_unique_by(left, right, u8::eq);
        prop_assert!(merged.iter().enumerate().all(|(index, value)| merged.iter().skip(index.saturating_add(1)).all(|later| value != later)));
    }
    #[test]
    fn test_merge_unique_obeys_set_laws(
        left in proptest::collection::vec(any::<i16>(), 0..40),
        right in proptest::collection::vec(any::<i16>(), 0..40),
        third in proptest::collection::vec(any::<i16>(), 0..40),
    ) {
        let merged = merge_unique(left.clone(), right.clone());
        prop_assert!(merged.windows(2).all(|pair| pair.first().zip(pair.get(1)).is_some_and(|(left, right)| left < right)));
        prop_assert_eq!(&merged, &merge_unique(right.clone(), left.clone()));
        prop_assert_eq!(&merged, &merge_unique(merged.clone(), merged.clone()));
        let left_associative = merge_unique(merge_unique(left.clone(), right.clone()), third.clone());
        let right_associative = merge_unique(left, merge_unique(right, third));
        prop_assert_eq!(left_associative, right_associative);
    }
    #[test]
    fn test_ntriples_literal_has_no_raw_delimiters(input in any::<String>(), control in 0_u8..=31) {
        let input = format!("{}{input}", char::from(control));
        let escaped = escape_ntriples_literal(&input);
        prop_assert!(!escaped.chars().any(char::is_control));
        let mut characters = escaped.chars();
        while let Some(character) = characters.next() {
            if character == '\\' {
                match characters.next() {
                    | Some('u') => {
                        for _ in 0..4 {
                            prop_assert!(characters.next().is_some_and(|digit| digit.is_ascii_hexdigit()));
                        }
                    }
                    | Some('\\' | '"' | 'n' | 'r' | 't') => {}
                    | _ => prop_assert!(false, "invalid N-Triples escape in {escaped:?}"),
                }
            } else {
                prop_assert_ne!(character, '"');
            }
        }
    }
    #[test]
    fn test_redaction_is_idempotent_and_removes_secrets(secret in "[a-z0-9]{6,20}") {
        let source = format!("Bearer {secret} token={secret} https://user:{secret}@example.org/path?token={secret}#fragment");
        let redacted = redact_sensitive_text(&source);
        prop_assert!(!redacted.contains(&secret));
        prop_assert_eq!(&redacted, &redact_sensitive_text(&redacted));
    }
    #[test]
    fn test_split_list_has_only_trimmed_nonempty_items(input in any::<String>()) {
        let items = split_list(&input);
        prop_assert!(items.iter().all(|item| !item.is_empty() && item.trim() == item));
    }
    #[test]
    fn test_stable_deduplicate_keeps_first_occurrences(input in proptest::collection::vec(any::<u8>(), 0..80)) {
        let unique = stable_deduplicate(input.clone());
        prop_assert!(unique.iter().enumerate().all(|(index, value)| unique.iter().skip(index.saturating_add(1)).all(|later| value != later)));
        prop_assert!(unique.iter().all(|value| input.contains(value)));
        let positions: Vec<_> = unique.iter().map(|value| input.iter().position(|original| original == value)).collect();
        prop_assert!(positions.windows(2).all(|pair| pair.first().zip(pair.get(1)).is_some_and(|(left, right)| left < right)));
        prop_assert_eq!(&unique, &stable_deduplicate(unique.clone()));
    }
}
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
fn test_escape_ntriples_literal_escapes_string_delimiters_and_controls() {
    assert_eq!(escape_ntriples_literal("plain"), "plain");
    assert_eq!(escape_ntriples_literal("a\\b\"c\n\r\t"), "a\\\\b\\\"c\\n\\r\\t");
}
#[test]
fn test_mime_type_ensure_supported_accepts_only_declared_formats() {
    let supported = [MimeType::Json, MimeType::Yaml];
    assert!(MimeType::Json.ensure_supported(&supported, "Schema export").is_ok());
    assert_eq!(
        MimeType::Pdf
            .ensure_supported(&supported, "Schema export")
            .expect_err("PDF should be rejected")
            .to_string(),
        "Schema export does not support pdf"
    );
}
#[test]
fn test_mime_type_from_resolves_file_type_names_and_aliases() {
    assert_eq!(MimeType::from("json"), MimeType::Json);
    assert_eq!(MimeType::from("activity.JSONL"), MimeType::JsonLines);
    assert_eq!(MimeType::from("markdown"), MimeType::Markdown);
    assert_eq!(MimeType::from("activity.ndjson"), MimeType::JsonLines);
    assert_eq!(MimeType::from("powerpoint"), MimeType::Powerpoint);
    assert_eq!(MimeType::from("settings.RON"), MimeType::Ron);
    assert_eq!(MimeType::from("zon"), MimeType::Zon);
    assert_eq!(MimeType::JsonLines.file_type(), "jsonl");
    assert_eq!(MimeType::JsonLines.to_string(), "application/x-ndjson");
    assert_eq!(
        serde_json::to_string(&MimeType::JsonLines).expect("serialize MIME type"),
        r#""json-lines""#
    );
    assert!(!MimeType::JsonLines.is_repository_file());
    assert_eq!(MimeType::Ron.file_type(), "ron");
    assert_eq!(MimeType::Ron.to_string(), "application/ron");
    assert!(!MimeType::Ron.is_repository_file());
}
#[test]
fn test_mime_type_identifies_archive_formats() {
    [MimeType::Gzip, MimeType::SevenZip, MimeType::Tar, MimeType::Zip]
        .iter()
        .for_each(|format| assert!(format.is_archive()));
    assert!(!MimeType::Json.is_archive());
}
#[test]
fn test_mime_type_infers_rhai_from_abi_header() {
    let source = b"// @acorn-exec-v0.2.0\nthis is not valid Rhai";
    assert_eq!(MimeType::infer(source), Some(MimeType::Rhai));
}
#[test]
fn test_mime_type_metadata_format_key_matches_web_parser() {
    let choices = [
        (MimeType::Json, "json"),
        (MimeType::Jsonc, "jsonc"),
        (MimeType::LdJson, "json-ld"),
        (MimeType::Yaml, "yaml"),
        (MimeType::Zon, "zon"),
        (MimeType::Cff, "cff"),
        (MimeType::Markdown, "markdown"),
        (MimeType::RdfXml, "rdf-xml"),
        (MimeType::Turtle, "turtle"),
        (MimeType::Text, "text"),
    ];
    choices.into_iter().for_each(|(mime, key)| assert_eq!(mime.metadata_format_key(), key));
}
#[test]
fn test_mime_type_parse_rejects_unsupported_file_types() {
    assert_eq!("json".parse::<MimeType>(), Ok(MimeType::Json));
    assert_eq!(MimeType::from("rdf-xml"), MimeType::RdfXml);
    assert_eq!(MimeType::from("turtle"), MimeType::Turtle);
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
fn test_mime_type_resolves_rdf_formats_without_claiming_generic_xml() {
    assert_eq!(MimeType::from("record.jsonld"), MimeType::LdJson);
    assert_eq!(MimeType::from("record.rdf"), MimeType::RdfXml);
    assert_eq!(MimeType::from("record.rdfxml"), MimeType::RdfXml);
    assert_eq!(MimeType::from("record.ttl"), MimeType::Turtle);
    assert_eq!(MimeType::from("record.xml"), MimeType::Vendor("xml".to_string()));
    assert_eq!(MimeType::RdfXml.to_string(), "application/rdf+xml");
    assert_eq!(MimeType::Turtle.to_string(), "text/turtle");
}
#[test]
fn test_ntriples_escapes_every_ascii_control() {
    for code_point in (0_u8..=31).chain(127..=159) {
        let input = char::from(code_point).to_string();
        let escaped = escape_ntriples_literal(&input);
        assert!(!escaped.chars().any(char::is_control), "unescaped U+{code_point:04X}");
    }
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
fn test_string_conversion_sluggifies_strings_and_paths() {
    assert_eq!("  Alice.Example  ".sluggify(), "alice-example");
    assert_eq!("---".sluggify(), "");
    assert_eq!(String::from("Hello,   World!").sluggify(), "hello-world");
    #[cfg(feature = "std")]
    assert_eq!(std::path::Path::new("Hello World").sluggify(), "hello-world");
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
#[test]
fn test_string_interpolation_replaces_contiguous_scalar_placeholders() {
    assert_eq!(
        "{{title}} {{ title }} {{  title  }}".replace_placeholder_with_string("title", "Value"),
        "Value Value Value"
    );
    assert_eq!("before {{ title }} after".replace_placeholder_with_string("title", ""), "before  after");
    assert_eq!("{{ other }}".replace_placeholder_with_string("title", "Value"), "{{ other }}");
}
#[test]
fn test_words_excludes_paths_and_dotted_tokens() {
    assert_eq!("Check the metadata in export/index.json".words(), vec!["check", "the", "metadata", "in"]);
    assert_eq!("Export, then review!".words(), vec!["export", "then", "review"]);
}
