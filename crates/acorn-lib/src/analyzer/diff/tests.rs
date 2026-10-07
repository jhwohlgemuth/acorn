use super::{DiffFormat, DiffInput, SemanticDiff, SemanticDiffError};
use acorn_core::util::{MarkdownSupport, MimeType};
use acorn_schema::{
    research_activity::ResearchActivity,
    standard::{zon, Standard},
};

fn input<'a>(content: &'a str, label: &'a str, media_type: MimeType) -> DiffInput<'a> {
    DiffInput { content, label, media_type }
}

#[test]
fn test_automatic_comparison_does_not_crosswalk_mismatched_standards() {
    let before = input(r#"{"attributes":{},"id":"10.1/example"}"#, "a/datacite.json", MimeType::Json);
    let after = input(r#"{"@type":"dcat:Dataset"}"#, "b/dcat.json", MimeType::Json);
    let patch = SemanticDiff::compare(&before, &after, DiffFormat::JsonPatch, None);
    let unified = SemanticDiff::compare(&before, &after, DiffFormat::Unified, None);
    assert!(matches!(patch, Err(SemanticDiffError::StandardMismatch { .. })));
    assert!(matches!(unified, Ok(SemanticDiff::Unified(diff)) if !diff.is_empty()));
}
#[test]
fn test_automatic_unified_falls_back_for_unknown_content() {
    let before = input("old\n", "a/file.txt", MimeType::Text);
    let after = input("new\n", "b/file.txt", MimeType::Text);
    let result = SemanticDiff::compare(&before, &after, DiffFormat::Unified, None);
    assert!(matches!(result, Ok(SemanticDiff::Unified(diff)) if diff.contains("-old") && diff.contains("+new")));
}
#[test]
fn test_canonical_json_ignores_object_key_order() {
    let source = include_str!("../../../../../tests/fixtures/analyzer/fair/metadata-minimal-rad.json");
    let reordered = serde_json::from_str::<serde_json::Value>(source)
        .and_then(|value| serde_json::to_string_pretty(&value))
        .expect("fixture is valid JSON");
    let before = input(source, "a/file.json", MimeType::Json);
    let after = input(&reordered, "b/file.json", MimeType::Json);
    let result = SemanticDiff::compare(&before, &after, DiffFormat::JsonPatch, Some(Standard::ResearchActivityData));
    assert!(matches!(result, Ok(SemanticDiff::JsonPatch(patch)) if patch.is_empty()));
}
#[test]
fn test_cff_yaml_is_compared_semantically() {
    let source = include_str!("../../../../../tests/fixtures/CITATION.cff");
    let canonical = serde_norway::from_str::<serde_json::Value>(source)
        .and_then(|value| serde_norway::to_string(&value))
        .expect("fixture is valid CFF");
    let before = input(source, "a/CITATION.cff", MimeType::Cff);
    let after = input(&canonical, "b/CITATION.cff", MimeType::Cff);
    let result = SemanticDiff::compare(&before, &after, DiffFormat::JsonPatch, None);
    assert!(matches!(result, Ok(SemanticDiff::JsonPatch(patch)) if patch.is_empty()));
}
#[test]
fn test_json_patch_never_falls_back() {
    let before = input("old\n", "a/file.txt", MimeType::Text);
    let after = input("new\n", "b/file.txt", MimeType::Text);
    let result = SemanticDiff::compare(&before, &after, DiffFormat::JsonPatch, None);
    assert!(matches!(result, Err(SemanticDiffError::UnknownStandard { .. })));
}
#[test]
fn test_rad_jsonc_yaml_zon_and_markdown_are_compared_semantically() {
    let source = include_str!("../../../../../tests/fixtures/analyzer/fair/metadata-minimal-rad.json");
    let value = serde_json::from_str::<serde_json::Value>(source).expect("fixture is valid JSON");
    let activity = serde_json::from_value::<ResearchActivity>(value.clone()).expect("fixture is valid RAD");
    let cases = [
        (format!("// comment\n{source}"), MimeType::Jsonc),
        (serde_norway::to_string(&value).expect("serialize YAML"), MimeType::Yaml),
        (zon::encode(&value).expect("serialize ZON"), MimeType::Zon),
        (activity.to_markdown(), MimeType::Markdown),
    ];
    cases.into_iter().for_each(|(content, media_type)| {
        let before = input(&content, "a/activity", media_type.clone());
        let after = input(&content, "b/activity", media_type);
        let result = SemanticDiff::compare(&before, &after, DiffFormat::JsonPatch, Some(Standard::ResearchActivityData));
        assert!(matches!(result, Ok(SemanticDiff::JsonPatch(patch)) if patch.is_empty()));
    });
}
#[test]
fn test_strict_detection_rejects_arbitrary_json_objects() {
    let before = input(r#"{"name":"before"}"#, "a/file.json", MimeType::Json);
    let after = input(r#"{"name":"after"}"#, "b/file.json", MimeType::Json);
    let result = SemanticDiff::compare(&before, &after, DiffFormat::JsonPatch, None);
    assert!(matches!(result, Err(SemanticDiffError::UnknownStandard { .. })));
}
