#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::schema::standard::text::{Document, Text};
use acorn_core::util::{MarkdownSupport, ToProse, Unstructured};
use acorn_schema::validation::Validate;
#[test]
fn test_document_to_prose_and_markdown_passthrough() {
    let data = Document {
        content: "Line one\n\nLine two".to_string(),
    };
    assert_eq!(data.to_prose(), "Line one\n\nLine two");
    assert_eq!(data.to_markdown(), "Line one\n\nLine two");
    assert_eq!(data.content(), "Line one\n\nLine two");
}
#[test]
fn test_document_validate_is_noop() {
    let data = Document {
        content: "any content should validate".to_string(),
    };
    assert!(data.validate().is_ok());
}
#[test]
fn test_text_to_prose_and_markdown_passthrough() {
    let data = Text {
        content: "Line one\n\nLine two".to_string(),
    };
    assert_eq!(data.to_prose(), "Line one\n\nLine two");
    assert_eq!(data.to_markdown(), "Line one\n\nLine two");
    assert_eq!(data.content(), "Line one\n\nLine two");
}
#[test]
fn test_text_validate_is_noop() {
    let data = Text {
        content: "any content should validate".to_string(),
    };
    assert!(data.validate().is_ok());
}
#[test]
fn test_text_converts_from_string_values() {
    assert_eq!(
        Text::from("borrowed"),
        Text {
            content: "borrowed".to_string()
        }
    );
    assert_eq!(
        Text::from("owned".to_string()),
        Text {
            content: "owned".to_string()
        }
    );
}
