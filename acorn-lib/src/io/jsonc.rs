//! JSONC (JSON with Comments) parsing utilities
//!
//! Provides conservative JSONC deserialization that accepts:
//! - `//` line comments
//! - `/* block */` comments
//! - trailing commas
//!
//! Rejects: unquoted keys, single-quoted strings, hexadecimal numbers,
//! missing commas, and other non-JSON extensions.
use color_eyre::eyre::{eyre, Result};
use serde::de::DeserializeOwned;

/// Conservative JSONC parse options — JSON-compatible plus comments and trailing commas.
fn jsonc_options() -> jsonc_parser::ParseOptions {
    jsonc_parser::ParseOptions {
        allow_comments: true,
        allow_trailing_commas: true,
        allow_loose_object_property_names: false,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    }
}

/// Parse a JSONC string into a concrete type.
///
/// Uses `jsonc_parser::parse_to_serde_value` for direct typed deserialization.
/// Works with any type that implements `serde::DeserializeOwned`.
///
/// # Errors
///
/// Returns an error if the content is not valid JSONC or cannot be deserialized
/// into the target type.
pub fn parse<T: DeserializeOwned>(content: &str) -> Result<T> {
    jsonc_parser::parse_to_serde_value(content, &jsonc_options()).map_err(|why| eyre!("JSONC parse error — {why}"))
}

/// Detect if content is JSON-like (starts with `{` or `[` after trimming).
///
/// This works for both strict JSON and JSONC since both start with the same
/// delimiters. Use this for content-based routing when the file extension
/// is not available.
pub fn is_json_like(content: &str) -> bool {
    let trimmed = content.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        #[serde(default)]
        items: Vec<String>,
    }

    #[test]
    fn parse_jsonc_with_line_comment() {
        let content = r#"{
            // This is a comment
            "name": "test",
            "items": ["a", "b"]
        }"#;
        let config: TestConfig = parse(content).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.items, vec!["a", "b"]);
    }

    #[test]
    fn parse_jsonc_with_block_comment() {
        let content = r#"{
            /* Block comment */
            "name": "test"
        }"#;
        let config: TestConfig = parse(content).unwrap();
        assert_eq!(config.name, "test");
    }

    #[test]
    fn parse_jsonc_with_trailing_commas() {
        let content = r#"{
            "name": "test",
            "items": ["a", "b",],
        }"#;
        let config: TestConfig = parse(content).unwrap();
        assert_eq!(config.items, vec!["a", "b"]);
    }

    #[test]
    fn parse_jsonc_rejects_single_quotes() {
        let content = r#"{
            'name': 'test'
        }"#;
        let result: Result<TestConfig> = parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn parse_jsonc_rejects_unquoted_keys() {
        let content = r#"{
            name: "test"
        }"#;
        let result: Result<TestConfig> = parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn parse_jsonc_rejects_hex_numbers() {
        let content = r#"{
            "name": "test",
            "count": 0xFF
        }"#;
        let result: Result<TestConfig> = parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn parse_plain_json_works() {
        let content = r#"{"name": "test", "items": []}"#;
        let config: TestConfig = parse(content).unwrap();
        assert_eq!(config.name, "test");
    }

    #[test]
    fn is_json_like_detects_object() {
        assert!(is_json_like(r#"{ "key": "value" }"#));
        assert!(is_json_like(r#"  { "key": "value" }"#));
        assert!(is_json_like(r#"[1, 2, 3]"#));
    }

    #[test]
    fn is_json_like_rejects_non_json() {
        assert!(!is_json_like("hello world"));
        assert!(!is_json_like("<html>"));
        assert!(!is_json_like(""));
    }
}
