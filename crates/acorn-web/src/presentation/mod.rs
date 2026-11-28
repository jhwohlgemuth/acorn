//! Pure display and export projections of portable operation results.
use crate::operations::Tool;
use acorn_analysis::{CheckCategory, CheckSeverity, FairDetails};
use core::cmp::Reverse;
use serde_json::Value;
use strum::IntoEnumIterator;

/// Diagnostic payload shape supplied by an operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DiagnosticList {
    /// Metadata conversion losses.
    Crosswalk,
    /// Local FAIR evidence findings.
    Fair,
    /// Schema and advisory diagnostics.
    Validation,
}
/// User-selected ordering within diagnostic categories.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DiagnosticSort {
    /// Field path or FAIR indicator.
    Field,
    /// Preserve the operation's original order.
    Original,
    /// Rule code, conversion loss type or FAIR state.
    Rule,
    /// Error, warning, suggestion, then information.
    #[default]
    Severity,
}
/// A content-only export; it carries no filesystem authority.
#[derive(Clone, Debug, PartialEq)]
pub struct Export {
    /// Exact exported text.
    pub content: String,
    /// Suggested basename.
    pub filename: String,
    /// Export media type.
    pub mime: &'static str,
}
/// Label and decorative icon for each canonical analyzer category.
pub fn category_style(category: &CheckCategory) -> (&'static str, &'static str) {
    match category {
        | CheckCategory::Schema => ("Schema", "code"),
        | CheckCategory::Link => ("Links", "link"),
        | CheckCategory::Prose => ("Prose", "book"),
        | CheckCategory::Quality => ("Quality", "filecheck"),
        | CheckCategory::Compute => ("Compute", "settings"),
        | CheckCategory::Fair => ("FAIR", "shield"),
        | CheckCategory::Readability => ("Readability", "book"),
        | CheckCategory::Crosswalk => ("Crosswalk", "shuffle"),
    }
}
/// Partition supplied diagnostics by analyzer category, retaining order within each group.
pub fn diagnostic_groups<'a>(diagnostics: &[&'a Value]) -> Vec<(CheckCategory, Vec<&'a Value>)> {
    CheckCategory::iter()
        .filter_map(|category| {
            let group = diagnostics
                .iter()
                .copied()
                .filter(|diagnostic| CheckCategory::from(text(diagnostic, "category")) == category)
                .collect::<Vec<_>>();
            (!group.is_empty()).then_some((category, group))
        })
        .collect()
}
/// Read severity emitted by the adapter; legacy validation errors retain error severity.
pub fn diagnostic_severity(value: &Value) -> CheckSeverity {
    serde_json::from_value(value["severity"].clone()).unwrap_or_default()
}
/// Count added and removed lines in a single-file unified diff, excluding its header.
pub fn diff_counts(diff: &str) -> (usize, usize) {
    diff.lines()
        .skip_while(|line| !line.starts_with("@@ "))
        .fold((0_usize, 0_usize), |(added, removed), line| {
            (
                added.saturating_add(usize::from(line.starts_with('+'))),
                removed.saturating_add(usize::from(line.starts_with('-'))),
            )
        })
}
/// Produce the user-facing artifact rather than the result envelope when supported.
pub fn export(tool: Tool, value: &Value) -> Export {
    let json = |value: &Value| serde_json::to_string_pretty(value).unwrap_or_default();
    match tool {
        | Tool::Crosswalk if value["content"].is_string() => Export {
            content: text(value, "content").into(),
            filename: "acorn-converted.json".into(),
            mime: "application/json;charset=utf-8",
        },
        | Tool::Diff if text(value, "mode") == "unified" => Export {
            content: text(value, "diff").into(),
            filename: "acorn-changes.diff".into(),
            mime: "text/plain;charset=utf-8",
        },
        | Tool::Diff if text(value, "mode") == "json-patch" => Export {
            content: json(&value["operations"]),
            filename: "acorn-changes.json".into(),
            mime: "application/json-patch+json;charset=utf-8",
        },
        | Tool::Format if value["content"].is_string() => Export {
            content: text(value, "content").into(),
            filename: format!(
                "acorn-formatted.{}",
                match text(value, "format") {
                    | "markdown" => "md",
                    | "json-ld" => "jsonld",
                    | "rdf-xml" => "rdf",
                    | "turtle" => "ttl",
                    | "zon" => "zonf",
                    | format => format,
                }
            ),
            mime: match text(value, "format") {
                | "markdown" => "text/markdown;charset=utf-8",
                | "cff" | "yaml" => "application/yaml;charset=utf-8",
                | "json-ld" => "application/ld+json;charset=utf-8",
                | "rdf-xml" => "application/rdf+xml;charset=utf-8",
                | "turtle" => "text/turtle;charset=utf-8",
                | "zon" => "text/plain;charset=utf-8",
                | _ => "application/json;charset=utf-8",
            },
        },
        | _ => Export {
            content: json(value),
            filename: format!("acorn-{}.json", tool.slug()),
            mime: "application/json;charset=utf-8",
        },
    }
}
/// Supported and applicable finding counts; not-applicable findings are excluded.
pub fn fair_counts(value: &Value, principle: &str) -> (usize, usize) {
    items(value, "findings")
        .iter()
        .filter(|finding| text(&finding["indicator"], "principle") == principle)
        .fold((0_usize, 0_usize), |(supported, total), finding| {
            (
                supported.saturating_add(usize::from(text(finding, "state") == "supported")),
                total.saturating_add(usize::from(text(finding, "state") != "not-applicable")),
            )
        })
}
/// Use the analyzer's FAIR severity conversion rather than treating evidence states as severities.
pub fn finding_severity(value: &Value) -> CheckSeverity {
    serde_json::from_value::<FairDetails>(value.clone())
        .map(CheckSeverity::from)
        .unwrap_or_default()
}
/// Bounded gauge position; absent or nonfinite values have no visual position.
pub fn gauge(value: Option<f64>, maximum: f64) -> Option<f64> {
    value
        .filter(|score| score.is_finite() && maximum.is_finite() && maximum > 0.0)
        .map(|score| (score / maximum * 100.0).clamp(0.0, 100.0))
}
/// Read a result array, preserving adapter ordering.
pub fn items<'a>(value: &'a Value, field: &str) -> &'a [Value] {
    value.get(field).and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default()
}
/// Human-readable select labels without changing the values sent to the adapter.
pub fn label(value: &str) -> &str {
    match value {
        | "" => "Auto-detect",
        | "research-activity-data" => "Research Activity Data",
        | "datacite" => "DataCite",
        | "raid" => "RAiD",
        | "dublin-core" => "Dublin Core",
        | "fres" => "Flesch Reading Ease",
        | "ari" => "Automated Readability Index",
        | "cli" => "Coleman–Liau Index",
        | "fkgl" => "Flesch–Kincaid Grade Level",
        | "gfi" => "Gunning Fog Index",
        | "lix" => "LIX",
        | "smog" => "SMOG",
        | "json-patch" => "JSON Patch",
        | "unified" => "Unified diff",
        | "off" => "Off · exact vocabulary",
        | "relaxed" => "Relaxed matching",
        | "strict" => "Strict matching",
        | other => other,
    }
}
/// Label and theme-aware color token driven exclusively by analyzer severity.
pub fn severity_style(severity: &CheckSeverity) -> (&'static str, &'static str) {
    match severity {
        | CheckSeverity::Error => ("Error", "danger"),
        | CheckSeverity::Info => ("Info", "info"),
        | CheckSeverity::Suggestion => ("Suggestion", "suggestion"),
        | CheckSeverity::Warning => ("Warning", "warning"),
    }
}
/// Stably sort a displayed diagnostic subset without modifying the result or its exports.
pub fn sort_diagnostics(mut values: Vec<&Value>, list: DiagnosticList, order: DiagnosticSort, descending: bool) -> Vec<&Value> {
    let key = |value: &&Value| match order {
        | DiagnosticSort::Original => (0, String::new()),
        | DiagnosticSort::Severity => {
            let severity = match list {
                | DiagnosticList::Crosswalk => CheckSeverity::Warning,
                | DiagnosticList::Fair => finding_severity(value),
                | DiagnosticList::Validation => diagnostic_severity(value),
            };
            (u8::from(&severity), String::new())
        }
        | DiagnosticSort::Field => (
            0,
            match list {
                | DiagnosticList::Crosswalk => text(value, "field"),
                | DiagnosticList::Fair => text(&value["indicator"], "id"),
                | DiagnosticList::Validation => value["path"].as_str().filter(|path| !path.is_empty()).unwrap_or("root"),
            }
            .to_lowercase(),
        ),
        | DiagnosticSort::Rule => (
            0,
            match list {
                | DiagnosticList::Crosswalk => text(value, "lossType"),
                | DiagnosticList::Fair => text(value, "state"),
                | DiagnosticList::Validation => text(value, "code"),
            }
            .to_lowercase(),
        ),
    };
    if order != DiagnosticSort::Original {
        if descending {
            values.sort_by_cached_key(|value| Reverse(key(value)));
        } else {
            values.sort_by_cached_key(key);
        }
    }
    values
}
/// Read a string field without displaying JSON quotation marks.
pub fn text<'a>(value: &'a Value, field: &str) -> &'a str {
    value.get(field).and_then(Value::as_str).unwrap_or("Not provided")
}

#[cfg(test)]
mod tests;
