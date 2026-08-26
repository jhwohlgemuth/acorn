#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
#[cfg(feature = "std")]
use crate::io::InputOutput;
use crate::schema::standard::text::{Document, Text};
#[cfg(feature = "std")]
use crate::test::utils::{fixture_path, unique_path};
use crate::util::{MarkdownSupport, ToProse, Unstructured};
use validator::Validate;

#[cfg(feature = "std")]
#[test]
fn test_document_input_output_reads_docx_fixture() {
    let source = fixture_path("acorn.docx");
    let result = Document::read(source).expect("failed to read docx file");
    assert!(result.content.contains("ACORN"));
}
#[cfg(feature = "std")]
#[test]
fn test_document_input_output_rejects_docx_write() {
    let output = unique_path("docx-io", "docx");
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).expect("failed to create test_artifacts directory");
    }
    let source = Document {
        content: "plain text content".to_string(),
    };
    let result = source.write(output.clone());
    assert!(result.is_err());
    let _cleanup = std::fs::remove_file(output);
}
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
#[cfg(feature = "std")]
#[test]
fn test_text_input_output_round_trip_txt() {
    let output = unique_path("text-io", "txt");
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).expect("failed to create test_artifacts directory");
    }
    let source = Text {
        content: "plain text content\nwith two lines".to_string(),
    };
    source.write(output.clone()).expect("failed to write text file");
    let result = Text::read(output.clone()).expect("failed to read text file");
    assert_eq!(result, source);
    let _cleanup = std::fs::remove_file(output);
}
#[cfg(feature = "std")]
#[test]
fn test_text_input_output_rejects_unsupported_extension() {
    let output = unique_path("text-io", "pptx");
    let result = Text::read(output);
    assert!(result.is_err());
}
#[cfg(feature = "std")]
#[test]
fn test_text_try_from_path_uses_supported_extensions() {
    let markdown = fixture_path("gather/depth/direct.md");
    let result = Text::try_from(markdown.as_path()).expect("supported Markdown path should load");
    assert!(!result.content.is_empty());

    let unsupported = unique_path("text-io", "pptx");
    let result = Text::try_from(unsupported.as_path());
    assert!(result.is_err());
}
