use crate::analyzer::readability::flesch_kincaid_grade_level;
use crate::io::docx::extract_text_from_path;
use crate::prelude::{write, PathBuf};
use crate::test_utils::{cleanup, fixture_path, temp_file};

#[test]
fn test_extract_sample_fixture() {
    let result = extract_text_from_path(fixture_path("sample.docx"));
    assert!(result.is_err(), "Sample fixture should currently fail parsing");
}
#[test]
fn test_extract_acorn_fixture() {
    let result = extract_text_from_path(fixture_path("acorn.docx"));
    assert!(result.is_ok(), "Acorn fixture should parse");
    let text = result.unwrap();
    assert!(text.contains("ACORN"), "Acorn fixture text should contain ACORN");
    assert!(
        text.contains("Research Activity Data (RAD)"),
        "Acorn fixture text should contain Research Activity Data (RAD)"
    );
}
#[test]
fn test_extract_acorn_fixture_readability() {
    let text = extract_text_from_path(fixture_path("acorn.docx")).unwrap();
    assert_eq!(17.34, flesch_kincaid_grade_level(&text));
}
#[test]
fn test_extract_nonexistent_file() {
    // Test: Should return error when file doesn't exist
    let nonexistent_path = PathBuf::from("/nonexistent/path/to/file.docx");
    let result = extract_text_from_path(&nonexistent_path);
    assert!(result.is_err(), "Should fail for nonexistent file");
}
#[test]
fn test_extract_invalid_docx() {
    // Test: Should return error when file is not a valid DOCX
    let temp_path = temp_file();
    write(&temp_path, b"Not a valid DOCX file").unwrap();
    let result = extract_text_from_path(&temp_path);
    assert!(result.is_err(), "Should fail for invalid DOCX format");
    cleanup(&temp_path);
}
#[test]
fn test_extract_empty_zip() {
    // Test: Should handle DOCX-like structure gracefully
    let temp_path = temp_file();
    // Write minimal ZIP header (PK signature)
    write(&temp_path, [0x50, 0x4b, 0x03, 0x04]).unwrap();
    let result = extract_text_from_path(&temp_path);
    // Should either succeed with empty string or return error - both are acceptable
    // The important thing is it doesn't panic
    let _ = result;
    cleanup(&temp_path);
}
#[test]
fn test_extract_function_signature() {
    // Test: Verify the function can be called with different path types
    // This tests the impl Into<PathBuf> trait
    let nonexistent = "/fake/path.docx";
    let result = extract_text_from_path(nonexistent);
    assert!(result.is_err(), "Should accept &str and return error for missing file");
}
