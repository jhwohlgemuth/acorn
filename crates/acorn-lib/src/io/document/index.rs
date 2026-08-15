//! Immutable indexing of positions within loaded documents.
use super::matching::{aligned_score, unique_max, UniqueMatch};
use super::SourceDocument;
use crate::prelude::{String, ToString, Vec};
use crate::util::MimeType;
use core::fmt;
use core::ops::{Add, Range};
use jsonc_parser::ast::Value as JsonValue;
use jsonc_parser::{parse_to_ast, CollectOptions, ParseOptions};
use serde_json::Value;

/// Result of resolving a [`DocumentQuery`]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentMatch {
    /// Exactly one source span matched.
    Unique(DocumentSpan),
    /// No source span matched safely.
    Missing,
    /// More than one source span matched.
    Ambiguous,
}
/// One component of a structured path within a document
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DocumentPathSegment {
    /// Object or mapping key.
    Key(String),
    /// Array or sequence index.
    Index(usize),
}
/// Structured path to a value within a document
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct DocumentPath(Vec<DocumentPathSegment>);
/// Byte position and corresponding display coordinates within a document
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentPosition {
    /// Zero-based UTF-8 byte offset.
    pub byte: usize,
    /// One-based physical line number.
    pub line: usize,
    /// One-based character column.
    pub column: usize,
}
/// Half-open UTF-8 byte range within a document
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentSpan(pub Range<usize>);
/// Display-ready excerpt retaining its original physical line numbering
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentExcerpt {
    /// Source content with blank placeholders for omitted physical lines
    pub content: String,
    /// Highlight span relative to [`Self::content`]
    pub span: DocumentSpan,
}
/// Format-neutral criteria for finding a value in a document.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DocumentQuery {
    paths: Vec<DocumentPath>,
    value: Option<String>,
    needle: Option<String>,
}
#[derive(Clone, Debug)]
struct DocumentEntry {
    path: DocumentPath,
    value: Option<String>,
    span: DocumentSpan,
}
#[derive(Clone, Debug)]
struct SemanticEntry {
    path: DocumentPath,
    value: Option<String>,
}
/// Lexicographic confidence, ordered from hard index agreement through ancestral key evidence.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
struct Confidence {
    exact_indices: usize,
    exact_keys: usize,
    normalized_keys: usize,
    matching_ancestry: usize,
}
/// Immutable lookup index over a loaded [`SourceDocument`].
#[derive(Clone, Debug)]
pub struct DocumentIndex {
    document: SourceDocument,
    entries: Vec<DocumentEntry>,
    line_starts: Vec<usize>,
    physical: bool,
    semantic: Vec<SemanticEntry>,
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
    fn confidence(&self, actual: &Self) -> Option<(Confidence, bool, bool)> {
        let depth = self.0.len();
        let confidence = aligned_score(&self.0, &actual.0, |index, expected, actual| match (expected, actual) {
            | (DocumentPathSegment::Index(expected), DocumentPathSegment::Index(actual)) if expected == actual => {
                let confidence = Confidence {
                    exact_indices: 1,
                    ..Confidence::default()
                };
                Some(confidence)
            }
            | (DocumentPathSegment::Key(expected), DocumentPathSegment::Key(actual)) if expected == actual => {
                let confidence = Confidence {
                    exact_keys: 1,
                    matching_ancestry: usize::from(index.saturating_add(1) < depth),
                    ..Confidence::default()
                };
                Some(confidence)
            }
            | (DocumentPathSegment::Key(expected), DocumentPathSegment::Key(actual)) if field_names_match(expected, actual) => {
                let confidence = Confidence {
                    normalized_keys: 1,
                    matching_ancestry: usize::from(index.saturating_add(1) < depth),
                    ..Confidence::default()
                };
                Some(confidence)
            }
            | (DocumentPathSegment::Key(_), DocumentPathSegment::Key(_)) => Some(Confidence::default()),
            | _ => None,
        })?;
        let mechanical = self.0.iter().zip(&actual.0).all(|(expected, actual)| match (expected, actual) {
            | (DocumentPathSegment::Key(expected), DocumentPathSegment::Key(actual)) => field_names_match(expected, actual),
            | (DocumentPathSegment::Index(expected), DocumentPathSegment::Index(actual)) => expected == actual,
            | _ => false,
        });
        let anchored = confidence.exact_keys.saturating_add(confidence.normalized_keys) > 0;
        Some((confidence, mechanical, anchored))
    }
    fn semantic_entries(&self, value: &Value) -> Vec<SemanticEntry> {
        match value {
            | Value::Object(object) => object
                .iter()
                .flat_map(|(key, value)| self.with(DocumentPathSegment::Key(key.clone())).semantic_entries(value))
                .collect(),
            | Value::Array(array) => array
                .iter()
                .enumerate()
                .flat_map(|(index, value)| self.with(DocumentPathSegment::Index(index)).semantic_entries(value))
                .collect(),
            | value => vec![SemanticEntry {
                path: self.clone(),
                value: Some(scalar(value)),
            }],
        }
    }
    fn collect_entries(&self, value: &JsonValue<'_>) -> Vec<DocumentEntry> {
        match value {
            | JsonValue::Object(object) => object
                .properties
                .iter()
                .flat_map(|property| {
                    self.with(DocumentPathSegment::Key(property.name.as_str().to_string()))
                        .collect_entries(&property.value)
                })
                .collect(),
            | JsonValue::Array(array) => array
                .elements
                .iter()
                .enumerate()
                .flat_map(|(index, value)| self.with(DocumentPathSegment::Index(index)).collect_entries(value))
                .collect(),
            | JsonValue::StringLit(value) => vec![DocumentEntry {
                path: self.clone(),
                value: Some(value.value.to_string()),
                span: DocumentSpan(value.range.start..value.range.end),
            }],
            | JsonValue::NumberLit(value) => vec![DocumentEntry {
                path: self.clone(),
                value: Some(value.value.to_string()),
                span: DocumentSpan(value.range.start..value.range.end),
            }],
            | JsonValue::BooleanLit(value) => vec![DocumentEntry {
                path: self.clone(),
                value: Some(value.value.to_string()),
                span: DocumentSpan(value.range.start..value.range.end),
            }],
            | JsonValue::NullKeyword(value) => vec![DocumentEntry {
                path: self.clone(),
                value: None,
                span: DocumentSpan(value.range.start..value.range.end),
            }],
        }
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
    fn matches(&self, value: Option<&str>) -> bool {
        let value_matches = self.value.as_deref().is_none_or(|expected| value == Some(expected));
        let needle_matches = self
            .needle
            .as_deref()
            .is_none_or(|needle| value.is_some_and(|value| value.contains(needle)));
        value_matches && needle_matches
    }
}
impl DocumentIndex {
    /// Index a loaded document without changing its content.
    pub fn new(document: SourceDocument) -> Self {
        let entries = document.json_entries();
        let line_starts = core::iter::once(0)
            .chain(document.content.match_indices('\n').map(|(index, _)| index.saturating_add(1)))
            .collect();
        let content = &document.content;
        let parsed_yaml = document.is_yaml().then(|| serde_norway::from_str::<Value>(content).ok()).flatten();
        let semantic = parsed_yaml
            .map(|value| DocumentPath::default().semantic_entries(&value))
            .unwrap_or_default();
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
    /// Resolve a query to its one-based physical line and character column
    pub fn locate(&self, query: &DocumentQuery) -> Option<DocumentPosition> {
        match self.resolve(query) {
            | DocumentMatch::Unique(span) => self.position(span.0.start),
            | DocumentMatch::Missing | DocumentMatch::Ambiguous => None,
        }
    }
    /// Create an excerpt whose highlighted span has at most `max_prefix` bytes of leading context.
    pub fn excerpt(&self, span: &DocumentSpan, max_prefix: usize) -> Option<DocumentExcerpt> {
        let DocumentSpan(range) = span;
        let content = &self.document.content;
        let ordered = range.start <= range.end;
        let in_bounds = range.end <= content.len();
        let boundaries = content.is_char_boundary(range.start) && content.is_char_boundary(range.end);
        let valid = ordered && in_bounds && boundaries;
        valid.then(|| self.position(range.start)).flatten().and_then(|position| {
            let line_offset = position.line.saturating_sub(1);
            self.line_starts.get(line_offset).copied().and_then(|line_start| {
                let should_truncate = range.start.saturating_sub(line_start) > max_prefix;
                let (excerpt_start, ellipsis) = if should_truncate {
                    (
                        prefix_boundary(&self.document.content, range.start.saturating_sub(max_prefix), range.start),
                        "...",
                    )
                } else {
                    (line_start, "")
                };
                self.document.content.get(excerpt_start..).map(|suffix| {
                    let content = format!("{}{ellipsis}{suffix}", "\n".repeat(line_offset));
                    let adjustment = line_offset.saturating_add(ellipsis.len());
                    let adjusted_start = range.start.saturating_sub(excerpt_start).saturating_add(adjustment);
                    let adjusted_end = range.end.saturating_sub(excerpt_start).saturating_add(adjustment);
                    let span = DocumentSpan(adjusted_start..adjusted_end);
                    DocumentExcerpt { content, span }
                })
            })
        })
    }
    /// Resolve a query only when its physical span can be determined safely.
    pub fn resolve(&self, query: &DocumentQuery) -> DocumentMatch {
        match (self.physical, query.paths.is_empty()) {
            | (false, _) => DocumentMatch::Missing,
            | (true, true) => self.resolve_text(query),
            | (true, false) => match resolve_candidate(
                &self.entries,
                query,
                |entry| &entry.path,
                |entry| entry.value.as_deref(),
                |entry| {
                    query.needle.as_deref().is_none_or(|needle| {
                        self.document
                            .content
                            .get(entry.span.0.clone())
                            .is_some_and(|value| value.contains(needle))
                    })
                },
            ) {
                | UniqueMatch::Unique(entry) => DocumentMatch::Unique(narrow_span(&self.document.content, &entry.span, query.needle.as_deref())),
                | UniqueMatch::Ambiguous => DocumentMatch::Ambiguous,
                | UniqueMatch::Missing => {
                    match resolve_candidate(&self.semantic, query, |entry| &entry.path, |entry| entry.value.as_deref(), |_| true) {
                        | UniqueMatch::Unique(_) => self.resolve_text(query),
                        | UniqueMatch::Ambiguous => DocumentMatch::Ambiguous,
                        | UniqueMatch::Missing => DocumentMatch::Missing,
                    }
                }
            },
        }
    }
    fn resolve_text(&self, query: &DocumentQuery) -> DocumentMatch {
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
impl Add for Confidence {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            exact_indices: self.exact_indices.saturating_add(other.exact_indices),
            exact_keys: self.exact_keys.saturating_add(other.exact_keys),
            normalized_keys: self.normalized_keys.saturating_add(other.normalized_keys),
            matching_ancestry: self.matching_ancestry.saturating_add(other.matching_ancestry),
        }
    }
}
impl fmt::Display for DocumentPosition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.line, self.column)
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
                    .map(|value| DocumentPath::default().collect_entries(&value))
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
fn field_names_match(left: &str, right: &str) -> bool {
    normalized_bytes(left).eq(normalized_bytes(right))
}
fn normalized_bytes(value: &str) -> impl Iterator<Item = u8> + '_ {
    value
        .bytes()
        .filter(|byte| !matches!(byte, b'_' | b'-'))
        .map(|byte| byte.to_ascii_lowercase())
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
fn resolve_candidate<'a, T>(
    candidates: &'a [T],
    query: &DocumentQuery,
    path: impl Copy + Fn(&T) -> &DocumentPath,
    value: impl Copy + Fn(&T) -> Option<&str>,
    additional_constraint: impl Copy + Fn(&T) -> bool,
) -> UniqueMatch<&'a T> {
    let matches = |candidate: &T| query.matches(value(candidate)) && additional_constraint(candidate);
    let exact = unique_candidate(query.paths.iter().flat_map(|expected| {
        candidates
            .iter()
            .filter(move |candidate| path(candidate) == expected && matches(candidate))
    }));
    match exact {
        | UniqueMatch::Missing => {
            let matching_values = candidates.iter().filter(|candidate| matches(candidate)).count();
            let scored = |mechanical_only: bool| {
                query.paths.iter().flat_map(move |expected| {
                    candidates.iter().filter_map(move |candidate| {
                        matches(candidate)
                            .then(|| expected.confidence(path(candidate)))
                            .flatten()
                            .filter(|(_, mechanical, anchored)| {
                                let convention_matches = !mechanical_only || *mechanical;
                                let anchor_matches = *anchored || matching_values == 1;
                                convention_matches && anchor_matches
                            })
                            .map(|(confidence, _, _)| (confidence, candidate))
                    })
                })
            };
            match unique_max(scored(true)) {
                | UniqueMatch::Missing => unique_max(scored(false)),
                | result => result,
            }
        }
        | result => result,
    }
}
fn unique_candidate<T>(mut candidates: impl Iterator<Item = T>) -> UniqueMatch<T> {
    match (candidates.next(), candidates.next()) {
        | (None, _) => UniqueMatch::Missing,
        | (Some(candidate), None) => UniqueMatch::Unique(candidate),
        | (Some(_), Some(_)) => UniqueMatch::Ambiguous,
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
