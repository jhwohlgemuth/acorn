//! Immutable indexing of positions within loaded documents.
use super::SourceDocument;
use crate::prelude::{String, ToString, Vec};
use crate::util::MimeType;
use core::ops::Range;
use jsonc_parser::ast::Value as JsonValue;
use jsonc_parser::{parse_to_ast, CollectOptions, ParseOptions};
use serde_json::Value;

/// One component of a structured path within a document.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DocumentPathSegment {
    /// Object or mapping key.
    Key(String),
    /// Array or sequence index.
    Index(usize),
}
/// Structured path to a value within a document.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct DocumentPath(Vec<DocumentPathSegment>);
/// Byte position and corresponding display coordinates within a document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentPosition {
    /// Zero-based UTF-8 byte offset.
    pub byte: usize,
    /// One-based physical line number.
    pub line: usize,
    /// One-based character column.
    pub column: usize,
}
/// Half-open UTF-8 byte range within a document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentSpan(pub Range<usize>);
/// Display-ready excerpt retaining its original physical line numbering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentExcerpt {
    /// Source content with blank placeholders for omitted physical lines.
    pub content: String,
    /// Highlight span relative to [`Self::content`].
    pub span: DocumentSpan,
}
/// Format-neutral criteria for finding a value in a document.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DocumentQuery {
    paths: Vec<DocumentPath>,
    value: Option<String>,
    needle: Option<String>,
}
/// Result of resolving a [`DocumentQuery`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentMatch {
    /// Exactly one source span matched.
    Unique(DocumentSpan),
    /// No source span matched safely.
    Missing,
    /// More than one source span matched.
    Ambiguous,
}
#[derive(Clone, Debug)]
struct DocumentEntry {
    path: DocumentPath,
    value: Option<String>,
    span: DocumentSpan,
}
/// Immutable lookup index over a loaded [`SourceDocument`].
#[derive(Clone, Debug)]
pub struct DocumentIndex {
    document: SourceDocument,
    entries: Vec<DocumentEntry>,
    line_starts: Vec<usize>,
    physical: bool,
    semantic: Option<Value>,
}
impl DocumentPath {
    /// Parse dotted keys and bracketed array indexes into a structured path.
    pub fn parse(value: &str) -> Self {
        let (mut segments, key) = value
            .replace("r#", "")
            .chars()
            .fold((Vec::new(), String::new()), |(mut segments, mut key), character| {
                match character {
                    | '.' => {
                        if !key.is_empty() {
                            segments.push(DocumentPathSegment::Key(core::mem::take(&mut key)));
                        }
                    }
                    | '[' => {
                        if !key.is_empty() {
                            segments.push(DocumentPathSegment::Key(core::mem::take(&mut key)));
                        }
                        key.push(character);
                    }
                    | ']' if key.starts_with('[') => {
                        let index = key.trim_start_matches('[').parse::<usize>().ok();
                        if let Some(index) = index {
                            segments.push(DocumentPathSegment::Index(index));
                        }
                        key.clear();
                    }
                    | _ => key.push(character),
                }
                (segments, key)
            });
        if !key.is_empty() {
            segments.push(DocumentPathSegment::Key(key));
        }
        Self(segments)
    }
    fn with(&self, segment: DocumentPathSegment) -> Self {
        Self(self.0.iter().cloned().chain(core::iter::once(segment)).collect())
    }
    fn value<'a>(&self, root: &'a Value) -> Option<&'a Value> {
        self.0.iter().try_fold(root, |value, segment| match segment {
            | DocumentPathSegment::Key(key) => value.get(key),
            | DocumentPathSegment::Index(index) => value.get(index),
        })
    }
}
impl DocumentQuery {
    /// Create an empty document query.
    pub fn new() -> Self {
        Self::default()
    }
    /// Add a candidate structured path.
    pub fn with_path(mut self, path: DocumentPath) -> Self {
        self.paths.push(path);
        self
    }
    /// Add an exact decoded value.
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
    /// Add a substring to highlight within a matched value or document.
    pub fn with_needle(mut self, needle: impl Into<String>) -> Self {
        self.needle = Some(needle.into());
        self
    }
}
impl DocumentIndex {
    /// Index a loaded document without changing its content.
    pub fn new(document: SourceDocument) -> Self {
        let entries = document.json_entries();
        let line_starts = core::iter::once(0)
            .chain(document.content.match_indices('\n').map(|(index, _)| index.saturating_add(1)))
            .collect();
        let semantic = document
            .is_yaml()
            .then(|| serde_norway::from_str::<Value>(&document.content).ok())
            .flatten();
        Self {
            physical: document.is_physical_text(),
            document,
            entries,
            line_starts,
            semantic,
        }
    }
    /// Return the indexed document.
    pub fn document(&self) -> &SourceDocument {
        &self.document
    }
    /// Convert a byte offset to one-based physical line and character column.
    pub fn position(&self, byte: usize) -> Option<DocumentPosition> {
        self.document.content.is_char_boundary(byte).then(|| {
            let line_index = self.line_starts.partition_point(|start| *start <= byte).saturating_sub(1);
            let line_start = self.line_starts.get(line_index).copied().unwrap_or_default();
            let column = self.document.content[line_start..byte].chars().count().saturating_add(1);
            DocumentPosition {
                byte,
                line: line_index.saturating_add(1),
                column,
            }
        })
    }
    /// Create an excerpt whose highlighted span has at most `max_prefix` bytes of leading context.
    pub fn excerpt(&self, span: &DocumentSpan, max_prefix: usize) -> Option<DocumentExcerpt> {
        let DocumentSpan(range) = span;
        if range.start > range.end
            || range.end > self.document.content.len()
            || !self.document.content.is_char_boundary(range.start)
            || !self.document.content.is_char_boundary(range.end)
        {
            return None;
        }
        let position = self.position(range.start)?;
        let line_offset = position.line.saturating_sub(1);
        let line_start = self.line_starts.get(line_offset).copied()?;
        let should_truncate = range.start.saturating_sub(line_start) > max_prefix;
        let (excerpt_start, ellipsis) = if should_truncate {
            (
                prefix_boundary(&self.document.content, range.start.saturating_sub(max_prefix), range.start),
                "...",
            )
        } else {
            (line_start, "")
        };
        let suffix = self.document.content.get(excerpt_start..)?;
        let line_padding = "\n".repeat(line_offset);
        let mut content = line_padding;
        content.push_str(ellipsis);
        content.push_str(suffix);
        let adjustment = line_offset.saturating_add(ellipsis.len());
        let adjusted_start = range.start.saturating_sub(excerpt_start).saturating_add(adjustment);
        let adjusted_end = range.end.saturating_sub(excerpt_start).saturating_add(adjustment);
        Some(DocumentExcerpt {
            content,
            span: DocumentSpan(adjusted_start..adjusted_end),
        })
    }
    /// Resolve a query only when its physical span can be determined safely.
    pub fn resolve(&self, query: &DocumentQuery) -> DocumentMatch {
        if !self.physical {
            return DocumentMatch::Missing;
        }
        let path_matches = query
            .paths
            .iter()
            .flat_map(|path| self.entries.iter().filter(move |entry| &entry.path == path))
            .filter(|entry| query.value.as_ref().is_none_or(|value| entry.value.as_ref() == Some(value)))
            .map(|entry| narrow_span(&self.document.content, &entry.span, query.needle.as_deref()))
            .collect::<Vec<_>>();
        match unique_span(path_matches) {
            | DocumentMatch::Missing => self.resolve_text(query),
            | result => result,
        }
    }
    fn resolve_text(&self, query: &DocumentQuery) -> DocumentMatch {
        let semantic_matches = query.paths.is_empty()
            || self.semantic.as_ref().is_some_and(|root| {
                query
                    .paths
                    .iter()
                    .filter_map(|path| path.value(root))
                    .any(|value| query.value.as_ref().is_none_or(|expected| scalar(value) == *expected))
            });
        if !semantic_matches {
            return DocumentMatch::Missing;
        }
        let sought = query.needle.as_ref().or(query.value.as_ref());
        sought.map_or(DocumentMatch::Missing, |sought| {
            unique_span(
                self.document
                    .content
                    .match_indices(sought)
                    .map(|(start, value)| DocumentSpan(start..start.saturating_add(value.len())))
                    .collect(),
            )
        })
    }
}
impl SourceDocument {
    fn json_entries(&self) -> Vec<DocumentEntry> {
        let mime = MimeType::from(self.source.as_str());
        if mime.is_json() || mime.is_jsonc() {
            {
                parse_to_ast(&self.content, &CollectOptions::default(), &ParseOptions::default())
                    .ok()
                    .and_then(|result| result.value)
                    .map(|value| collect_json_entries(&value, &DocumentPath::default()))
                    .unwrap_or_default()
            }
        } else {
            Default::default()
        }
    }
    fn is_physical_text(&self) -> bool {
        let mime = MimeType::from(self.format.as_str());
        !(mime.is_doc()
            || mime.is_docx()
            || mime.is_epub()
            || mime.is_odp()
            || mime.is_ods()
            || mime.is_odt()
            || mime.is_pdf()
            || mime.is_ppt()
            || mime.is_powerpoint()
            || mime.is_rtf())
    }
}
fn prefix_boundary(content: &str, candidate: usize, span_start: usize) -> usize {
    let candidate = (0..=candidate.min(content.len()))
        .rev()
        .find(|index| content.is_char_boundary(*index))
        .unwrap_or_default();
    content
        .get(candidate..span_start)
        .and_then(|prefix| {
            prefix
                .char_indices()
                .find(|(_, character)| character.is_whitespace())
                .map(|(index, character)| candidate.saturating_add(index).saturating_add(character.len_utf8()))
        })
        .unwrap_or(candidate)
}
fn collect_json_entries(value: &JsonValue<'_>, path: &DocumentPath) -> Vec<DocumentEntry> {
    match value {
        | JsonValue::Object(object) => object
            .properties
            .iter()
            .flat_map(|property| collect_json_entries(&property.value, &path.with(DocumentPathSegment::Key(property.name.as_str().to_string()))))
            .collect(),
        | JsonValue::Array(array) => array
            .elements
            .iter()
            .enumerate()
            .flat_map(|(index, value)| collect_json_entries(value, &path.with(DocumentPathSegment::Index(index))))
            .collect(),
        | JsonValue::StringLit(value) => vec![DocumentEntry {
            path: path.clone(),
            value: Some(value.value.to_string()),
            span: DocumentSpan(value.range.start..value.range.end),
        }],
        | JsonValue::NumberLit(value) => vec![DocumentEntry {
            path: path.clone(),
            value: Some(value.value.to_string()),
            span: DocumentSpan(value.range.start..value.range.end),
        }],
        | JsonValue::BooleanLit(value) => vec![DocumentEntry {
            path: path.clone(),
            value: Some(value.value.to_string()),
            span: DocumentSpan(value.range.start..value.range.end),
        }],
        | JsonValue::NullKeyword(value) => vec![DocumentEntry {
            path: path.clone(),
            value: None,
            span: DocumentSpan(value.range.start..value.range.end),
        }],
    }
}
fn scalar(value: &Value) -> String {
    match value {
        | Value::String(value) => value.clone(),
        | Value::Null => "null".to_string(),
        | _ => value.to_string(),
    }
}
fn narrow_span(content: &str, span: &DocumentSpan, needle: Option<&str>) -> DocumentSpan {
    needle
        .and_then(|needle| {
            content
                .get(span.0.clone())
                .and_then(|value| value.find(needle))
                .map(|offset| (offset, needle.len()))
        })
        .map_or_else(
            || span.clone(),
            |(offset, length)| DocumentSpan(span.0.start.saturating_add(offset)..span.0.start.saturating_add(offset).saturating_add(length)),
        )
}
fn unique_span(spans: Vec<DocumentSpan>) -> DocumentMatch {
    match spans.as_slice() {
        | [] => DocumentMatch::Missing,
        | [span] => DocumentMatch::Unique(span.clone()),
        | _ => DocumentMatch::Ambiguous,
    }
}
