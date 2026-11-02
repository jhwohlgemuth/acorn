//! Direct format output compared with the shared native normalization operation.
use acorn::io::mcp::{format_acre, normalize_metadata_file, FormatPreviewInput, FormatStandard, InputReference};
use acorn_schema::standard::cff::Cff;
use acorn_schema::vocabulary::VocabularyFuzzyMode;
use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_direct_acre_format_matches_native_formatter() {
    let root = tempdir().expect("temporary root");
    let path = root.path().join(".acorn.jsonc");
    let original = include_str!("../../../tests/fixtures/config/basic.jsonc");
    fs::write(&path, original).expect("fixture");
    let expected = format_acre(&path, original).expect("native ACRE formatting");
    let output = Command::cargo_bin("acorn")
        .expect("binary")
        .current_dir(root.path())
        .args(["--no-local-database", "format", ".acorn.jsonc", "--standard", "acre"])
        .output()
        .expect("direct format");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(fs::read_to_string(path).expect("formatted ACRE"), expected);
}
#[test]
fn test_direct_cff_format_matches_native_formatter() {
    let root = tempdir().expect("temporary root");
    let path = root.path().join("CITATION.cff");
    let original = include_str!("../../../tests/fixtures/CITATION.cff");
    fs::write(&path, original).expect("fixture");
    let normalized = normalize_metadata_file::<Cff>(&path, VocabularyFuzzyMode::Strict).expect("native formatting");
    let output = Command::cargo_bin("acorn")
        .expect("binary")
        .current_dir(root.path())
        .args(["--no-local-database", "format", "CITATION.cff", "--standard", "cff"])
        .output()
        .expect("direct format");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let expected = if normalized.before == normalized.content {
        original
    } else {
        &normalized.content
    };
    assert_eq!(fs::read_to_string(path).expect("formatted CFF"), expected);
}
#[tokio::test]
async fn test_direct_enriched_format_matches_native_preview() {
    let root = tempdir().expect("temporary root");
    let path = root.path().join("CITATION.cff");
    let original = include_str!("../../../tests/fixtures/CITATION.cff")
        .lines()
        .filter(|line| !line.trim_start().starts_with("orcid:"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&path, &original).expect("fixture");
    let native = FormatPreviewInput {
        input: InputReference::File(PathBuf::from("CITATION.cff")),
        standard: FormatStandard::CitationFileFormat,
        enrich: true,
        providers: Vec::new(),
        fuzzy: VocabularyFuzzyMode::Strict,
    }
    .preview(root.path(), false)
    .await
    .expect("native enrichment preview");
    let output = Command::cargo_bin("acorn")
        .expect("binary")
        .current_dir(root.path())
        .args(["--no-local-database", "format", "CITATION.cff", "--standard", "cff", "--enrich"])
        .output()
        .expect("direct enrichment");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let change = native.changes.first().expect("change");
    let expected = if change.diff.is_empty() { &original } else { &change.content };
    assert_eq!(fs::read_to_string(path).expect("enriched CFF"), expected.as_str());
}
#[tokio::test]
async fn test_direct_format_preview_matches_native_diff_without_writing() {
    let root = tempdir().expect("temporary root");
    let path = root.path().join(".acorn.jsonc");
    let original = format!("\n{}", include_str!("../../../tests/fixtures/config/basic.jsonc"));
    fs::write(&path, &original).expect("fixture");
    let native = FormatPreviewInput {
        input: InputReference::File(PathBuf::from(".acorn.jsonc")),
        standard: FormatStandard::ApplicationConfigurationResearchEnablement,
        enrich: false,
        providers: Vec::new(),
        fuzzy: VocabularyFuzzyMode::Strict,
    }
    .preview(root.path(), true)
    .await
    .expect("native preview");
    let change = native.changes.first().expect("first change");
    assert!(!change.diff.is_empty());
    let source = path.display().to_string();
    let output = Command::cargo_bin("acorn")
        .expect("binary")
        .current_dir(root.path())
        .args(["--no-local-database", "format", &source, "--standard", "acre", "--dry-run", "--no-color"])
        .output()
        .expect("direct preview");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let diff_body = change.diff.splitn(3, '\n').nth(2).expect("diff body");
    assert!(String::from_utf8_lossy(&output.stdout).contains(diff_body));
    assert_eq!(fs::read_to_string(path).expect("unchanged ACRE"), original);
}
