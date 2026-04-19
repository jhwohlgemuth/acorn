#[cfg(feature = "std")]
use crate::io::InputOutput;
use crate::schema::standard::text::Text;
#[cfg(feature = "std")]
use crate::test_utils::unique_path;
use crate::util::{ToMarkdown, ToProse};
use validator::Validate;

#[test]
fn test_text_to_prose_and_markdown_passthrough() {
    let data = Text {
        content: "Line one\n\nLine two".to_string(),
    };
    assert_eq!(data.to_prose(), "Line one\n\nLine two");
    assert_eq!(data.to_markdown(), "Line one\n\nLine two");
}
#[test]
fn test_text_validate_is_noop() {
    let data = Text {
        content: "any content should validate".to_string(),
    };
    assert!(data.validate().is_ok());
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
