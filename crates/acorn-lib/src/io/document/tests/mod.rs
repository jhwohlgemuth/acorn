use super::*;

fn document(content: &str, source: &str) -> SourceDocument {
    SourceDocument::init().content(content).format("text").source(source).build()
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
fn test_rejects_text_match_when_yaml_path_has_another_value() {
    let content = "title: Alpha\nmessage: Beta mentions Alpha\n";
    let index = DocumentIndex::new(document(content, "input.yaml"));
    let query = DocumentQuery::new().with_path(DocumentPath::parse("message")).with_value("Alpha");
    assert_eq!(index.resolve(&query), DocumentMatch::Missing);
}
#[test]
fn test_reports_ambiguous_text_without_guessing() {
    let index = DocumentIndex::new(document("value\nvalue\n", "input.yaml"));
    assert_eq!(index.resolve(&DocumentQuery::new().with_value("value")), DocumentMatch::Ambiguous);
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
