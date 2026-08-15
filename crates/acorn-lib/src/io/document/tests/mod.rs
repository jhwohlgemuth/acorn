use super::matching::{aligned_score, unique_max, UniqueMatch};
use super::*;

fn document(content: &str, source: &str) -> SourceDocument {
    SourceDocument::init().content(content).format("text").source(source).build()
}

#[test]
fn test_aligned_score_rejects_different_lengths() {
    assert_eq!(
        aligned_score(&[1], &[1, 2], |_, expected, actual| (*expected == *actual).then_some(1)),
        None
    );
}
#[test]
fn test_aligned_score_rejects_incompatible_segments() {
    assert_eq!(
        aligned_score(&[1, 2], &[1, 3], |_, expected, actual| (*expected == *actual).then_some(1)),
        None
    );
}
#[test]
fn test_aligned_score_sums_compatible_segments() {
    assert_eq!(
        aligned_score(&[1, 2], &[1, 2], |_, expected, actual| (*expected == *actual).then_some(1)),
        Some(2)
    );
}

#[test]
fn test_anydoc_format_round_trip() {
    let formats = [
        DocumentFormat::Csv,
        DocumentFormat::Doc,
        DocumentFormat::Docx,
        DocumentFormat::Epub,
        DocumentFormat::Excel,
        DocumentFormat::Odp,
        DocumentFormat::Ods,
        DocumentFormat::Odt,
        DocumentFormat::Pdf,
        DocumentFormat::Ppt,
        DocumentFormat::Pptx,
        DocumentFormat::Rtf,
    ];
    formats
        .into_iter()
        .for_each(|format| assert_eq!(DocumentFormat::from(Format::from(format)), format));
}
#[test]
fn test_counts_crlf_as_one_physical_line_break() {
    let index = DocumentIndex::new(document("first\r\nsecond", "input.txt"));
    assert_eq!(index.position(7), Some(DocumentPosition { byte: 7, line: 2, column: 1 }));
}
#[test]
fn test_counts_unicode_columns_from_byte_offsets() {
    let index = DocumentIndex::new(document("αβ\nvalue", "input.txt"));
    assert_eq!(index.position(6), Some(DocumentPosition { byte: 6, line: 2, column: 2 }));
}
#[test]
fn test_document_position_display_renders_line_and_column() {
    let position = DocumentPosition {
        byte: 42,
        line: 3,
        column: 7,
    };
    assert_eq!(position.to_string(), "3:7");
}
#[test]
fn test_locates_resolved_query() {
    let index = DocumentIndex::new(document("first\nvalue", "input.txt"));
    let query = DocumentQuery::new().with_value("value");
    assert_eq!(index.locate(&query), Some(DocumentPosition { byte: 6, line: 2, column: 1 }));
}
#[test]
fn test_docx_converts_to_markdown() {
    let converted = to_markdown("../../tests/fixtures/acorn.docx");
    assert!(converted.is_ok());
    assert!(!converted.unwrap_or_default().is_empty());
}
#[test]
fn test_document_format_mime_round_trip() {
    let formats = [
        DocumentFormat::Csv,
        DocumentFormat::Doc,
        DocumentFormat::Docx,
        DocumentFormat::Epub,
        DocumentFormat::Excel,
        DocumentFormat::Odp,
        DocumentFormat::Ods,
        DocumentFormat::Odt,
        DocumentFormat::Pdf,
        DocumentFormat::Ppt,
        DocumentFormat::Pptx,
        DocumentFormat::Rtf,
    ];
    formats.into_iter().for_each(|format| {
        let mime = MimeType::from(format);
        assert_eq!(DocumentFormat::try_from(&mime).ok(), Some(format));
    });
}
#[test]
fn test_excerpts_long_line_prefixes_and_retains_physical_line() {
    let content = "first\n      alpha beta gamma target tail\nlast";
    let index = DocumentIndex::new(document(content, "input.txt"));
    let start = content.find("target").expect("test content contains target");
    let excerpt = index
        .excerpt(&DocumentSpan(start..start + "target".len()), 12)
        .expect("valid span has an excerpt");
    assert_eq!(excerpt.content, "\n...beta gamma target tail\nlast");
    assert_eq!(&excerpt.content[excerpt.span.0], "target");
}
#[test]
fn test_excerpts_short_prefixes_from_the_physical_line_start() {
    let content = "first\n  target tail";
    let index = DocumentIndex::new(document(content, "input.txt"));
    let start = content.find("target").expect("test content contains target");
    let excerpt = index
        .excerpt(&DocumentSpan(start..start + "target".len()), 50)
        .expect("valid span has an excerpt");
    assert_eq!(excerpt.content, "\n  target tail");
    assert_eq!(&excerpt.content[excerpt.span.0], "target");
}
#[test]
fn test_narrows_json_value_to_requested_word() {
    let content = "{\"impact\":\"Use geo-spatial data\"}";
    let index = DocumentIndex::new(document(content, "index.json"));
    let query = DocumentQuery::new()
        .with_path(DocumentPath::parse("impact"))
        .with_value("Use geo-spatial data")
        .with_needle("geo-spatial");
    let span = match index.resolve(&query) {
        | DocumentMatch::Unique(span) => span,
        | result => {
            assert!(matches!(result, DocumentMatch::Unique(_)), "expected a unique source span");
            return;
        }
    };
    assert_eq!(&content[span.0], "geo-spatial");
}
#[test]
fn test_parses_dotted_and_indexed_paths() {
    assert_eq!(DocumentPath::parse("meta.doi[2]"), DocumentPath::parse("r#meta.doi[2]"));
}
#[test]
fn test_reports_ambiguous_text_without_guessing() {
    let index = DocumentIndex::new(document("value\nvalue\n", "input.yaml"));
    assert_eq!(index.resolve(&DocumentQuery::new().with_value("value")), DocumentMatch::Ambiguous);
}
#[test]
fn test_reports_ambiguous_yaml_text_when_only_unanchored_scalar_matches() {
    let content = "title: Alpha\nmessage: Beta mentions Alpha\n";
    let index = DocumentIndex::new(document(content, "input.yaml"));
    let query = DocumentQuery::new().with_path(DocumentPath::parse("message")).with_value("Alpha");
    assert_eq!(index.resolve(&query), DocumentMatch::Ambiguous);
}
#[test]
fn test_resolver_prefers_canonical_exact_path() {
    let content = r#"{"contentUrl":"canonical.png","content_url":"fallback.png"}"#;
    let index = DocumentIndex::new(document(content, "index.json"));
    let query = DocumentQuery::new()
        .with_path(DocumentPath::parse("contentUrl"))
        .with_value("canonical.png");
    let span = match index.resolve(&query) {
        | DocumentMatch::Unique(span) => span,
        | result => {
            assert!(matches!(result, DocumentMatch::Unique(_)), "canonical path should resolve");
            return;
        }
    };
    assert_eq!(&content[span.0], "\"canonical.png\"");
}
#[test]
fn test_resolver_rejects_candidate_with_different_array_index() {
    let content = r#"{"items":[{"name":"only"}]}"#;
    let index = DocumentIndex::new(document(content, "index.json"));
    let query = DocumentQuery::new().with_path(DocumentPath::parse("items[1].name")).with_value("only");
    assert_eq!(index.resolve(&query), DocumentMatch::Missing);
}
#[test]
fn test_resolver_rejects_candidate_without_matching_needle() {
    let content = r#"{"url":"https://example.com"}"#;
    let index = DocumentIndex::new(document(content, "index.json"));
    let query = DocumentQuery::new().with_path(DocumentPath::parse("url")).with_needle("ornl.gov");
    assert_eq!(index.resolve(&query), DocumentMatch::Missing);
}
#[test]
fn test_resolver_rejects_candidate_without_matching_value() {
    let content = r#"{"url":"https://example.com"}"#;
    let index = DocumentIndex::new(document(content, "index.json"));
    let query = DocumentQuery::new().with_path(DocumentPath::parse("url")).with_value("https://ornl.gov");
    assert_eq!(index.resolve(&query), DocumentMatch::Missing);
}
#[test]
fn test_resolver_resolves_camel_case_content_url() {
    let content = r#"{"contentUrl":"image.png"}"#;
    let index = DocumentIndex::new(document(content, "index.json"));
    let query = DocumentQuery::new().with_path(DocumentPath::parse("content_url")).with_value("image.png");
    assert!(matches!(index.resolve(&query), DocumentMatch::Unique(_)));
}
#[test]
fn test_resolver_resolves_datacite_attribute_alias_when_anchored() {
    let content = r#"{"nameIdentifiers":[{"@schemeURI":"https://example.com/scheme"}]}"#;
    let index = DocumentIndex::new(document(content, "datacite.json"));
    let query = DocumentQuery::new()
        .with_path(DocumentPath::parse("name_identifiers[0].scheme_uri"))
        .with_value("https://example.com/scheme");
    assert!(matches!(index.resolve(&query), DocumentMatch::Unique(_)));
}
#[test]
fn test_resolver_resolves_datacite_text_alias_when_anchored() {
    let content = r#"{"rights":[{"$text":"Open"}]}"#;
    let index = DocumentIndex::new(document(content, "datacite.json"));
    let query = DocumentQuery::new().with_path(DocumentPath::parse("rights[0].rights")).with_value("Open");
    assert!(matches!(index.resolve(&query), DocumentMatch::Unique(_)));
}
#[test]
fn test_resolver_resolves_duplicate_value_with_stronger_structure() {
    let content = r#"{"meta":{"alias":"same"},"other":{"alias":"same"}}"#;
    let index = DocumentIndex::new(document(content, "index.json"));
    let query = DocumentQuery::new().with_path(DocumentPath::parse("meta.value")).with_value("same");
    let span = match index.resolve(&query) {
        | DocumentMatch::Unique(span) => span,
        | result => {
            assert!(matches!(result, DocumentMatch::Unique(_)), "stronger ancestor should resolve");
            return;
        }
    };
    assert_eq!(index.position(span.0.start).map(|position| position.column), Some(18));
}
#[test]
fn test_resolver_resolves_jsonc_normalized_path() {
    let content = "{ // retained comment\n  \"contentUrl\": \"image.png\"\n}";
    let index = DocumentIndex::new(document(content, "index.jsonc"));
    let query = DocumentQuery::new().with_path(DocumentPath::parse("content_url")).with_value("image.png");
    assert!(matches!(index.resolve(&query), DocumentMatch::Unique(_)));
}
#[test]
fn test_resolver_resolves_mixed_retained_alias_path() {
    let content = r#"{"meta":{"graphics":[{"contentUrl":"image.png"}]}}"#;
    let index = DocumentIndex::new(document(content, "index.json"));
    let query = DocumentQuery::new()
        .with_path(DocumentPath::parse("meta.media[0].content_url"))
        .with_value("image.png");
    assert!(matches!(index.resolve(&query), DocumentMatch::Unique(_)));
}
#[test]
fn test_resolver_resolves_raid_organization_alias() {
    let content = r#"{"organisation":[{"schemaUri":"https://example.com/schema"}]}"#;
    let index = DocumentIndex::new(document(content, "raid.json"));
    let query = DocumentQuery::new()
        .with_path(DocumentPath::parse("organization[0].schema_uri"))
        .with_value("https://example.com/schema");
    assert!(matches!(index.resolve(&query), DocumentMatch::Unique(_)));
}
#[test]
fn test_resolver_resolves_retained_id_alias() {
    let content = r#"{"meta":{"id":"invalid identifier"}}"#;
    let index = DocumentIndex::new(document(content, "index.json"));
    let query = DocumentQuery::new()
        .with_path(DocumentPath::parse("meta.identifier"))
        .with_value("invalid identifier");
    assert!(matches!(index.resolve(&query), DocumentMatch::Unique(_)));
}
#[test]
fn test_resolver_resolves_retained_phone_alias() {
    let content = r#"{"contact":{"phone":"invalid phone"}}"#;
    let index = DocumentIndex::new(document(content, "index.json"));
    let query = DocumentQuery::new()
        .with_path(DocumentPath::parse("contact.telephone"))
        .with_value("invalid phone");
    assert!(matches!(index.resolve(&query), DocumentMatch::Unique(_)));
}
#[test]
fn test_resolver_resolves_yaml_alias_path_before_text_search() {
    let content = "meta:\n  id: invalid identifier\n";
    let index = DocumentIndex::new(document(content, "index.yaml"));
    let query = DocumentQuery::new()
        .with_path(DocumentPath::parse("meta.identifier"))
        .with_value("invalid identifier");
    assert!(matches!(index.resolve(&query), DocumentMatch::Unique(_)));
}
#[test]
fn test_resolver_returns_ambiguous_for_tied_candidates() {
    let content = r#"{"meta":{"first":"same","second":"same"}}"#;
    let index = DocumentIndex::new(document(content, "index.json"));
    let query = DocumentQuery::new().with_path(DocumentPath::parse("meta.value")).with_value("same");
    assert_eq!(index.resolve(&query), DocumentMatch::Ambiguous);
}
#[test]
fn test_resolver_returns_ambiguous_for_repeated_yaml_text() {
    let content = "meta:\n  id: repeated\nmessage: repeated\n";
    let index = DocumentIndex::new(document(content, "index.yaml"));
    let query = DocumentQuery::new()
        .with_path(DocumentPath::parse("meta.identifier"))
        .with_value("repeated");
    assert_eq!(index.resolve(&query), DocumentMatch::Ambiguous);
}
#[test]
fn test_resolves_json_path_to_original_value() {
    let content = "{\n  \"meta\": {\n    \"doi\": [\"one\", \"Not a DOI\"]\n  }\n}";
    let index = DocumentIndex::new(document(content, "index.json"));
    let query = DocumentQuery::new().with_path(DocumentPath::parse("meta.doi[1]")).with_value("Not a DOI");
    let span = match index.resolve(&query) {
        | DocumentMatch::Unique(span) => span,
        | result => {
            assert!(matches!(result, DocumentMatch::Unique(_)), "expected a unique source span");
            return;
        }
    };
    assert_eq!(&content[span.0.clone()], "\"Not a DOI\"");
    assert_eq!(
        index.position(span.0.start),
        Some(DocumentPosition {
            byte: 33,
            line: 3,
            column: 20
        })
    );
}
#[test]
fn test_resolves_unique_nested_yaml_scalar_conservatively() {
    let content = "authors:\n  - name: Alice\n    role: lead\n";
    let index = DocumentIndex::new(document(content, "CITATION.cff"));
    let query = DocumentQuery::new().with_path(DocumentPath::parse("authors[0].name")).with_value("Alice");
    let span = match index.resolve(&query) {
        | DocumentMatch::Unique(span) => span,
        | result => {
            assert!(matches!(result, DocumentMatch::Unique(_)), "expected a unique YAML scalar span");
            return;
        }
    };
    assert_eq!(&content[span.0.clone()], "Alice");
    assert_eq!(index.position(span.0.start).map(|position| position.line), Some(2));
}
#[test]
fn test_source_document_extracts_docx() {
    let converted = SourceDocument::at("../../tests/fixtures/acorn.docx").extract();
    assert!(converted.is_ok());
    assert!(!converted.unwrap_or_default().is_empty());
}
#[test]
fn test_unique_max_recovers_after_a_lower_tie() {
    assert_eq!(unique_max([(1, "first"), (1, "second"), (2, "third")]), UniqueMatch::Unique("third"));
}
#[test]
fn test_unique_max_rejects_tied_highest_scores() {
    assert_eq!(unique_max([(2, "first"), (2, "second"), (1, "third")]), UniqueMatch::Ambiguous);
}
#[test]
fn test_unique_max_selects_the_highest_score() {
    assert_eq!(unique_max([(1, "first"), (3, "second"), (2, "third")]), UniqueMatch::Unique("second"));
}
