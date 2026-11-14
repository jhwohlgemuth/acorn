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
    #[default]
    Original,
    /// Rule code, conversion loss type or FAIR state.
    Rule,
    /// Error, warning, suggestion, then information.
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
/// Partition the first 100 diagnostics by analyzer category, retaining order within each group.
pub fn diagnostic_groups(value: &Value) -> Vec<(CheckCategory, Vec<&Value>)> {
    CheckCategory::iter()
        .filter_map(|category| {
            let diagnostics = items(value, "diagnostics")
                .iter()
                .take(100)
                .filter(|diagnostic| CheckCategory::from(text(diagnostic, "category")) == category)
                .collect::<Vec<_>>();
            (!diagnostics.is_empty()).then_some((category, diagnostics))
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
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_diagnostic_groups_follow_enum_order_and_preserve_details() {
        let value = json!({"diagnostics": [
            {"category": "prose", "severity": "info", "path": "meta.keywords[0]"},
            {"category": "schema", "severity": "error", "path": "title"},
            {"category": "prose", "severity": "warning", "path": "meta.keywords[1]"},
            {"category": "schema", "severity": "error", "path": "contact.email"}
        ]});
        let groups = diagnostic_groups(&value);
        assert_eq!(groups.len(), 2);
        let schema = groups.first().unwrap();
        let prose = groups.get(1).unwrap();
        assert_eq!(schema.0, CheckCategory::Schema);
        assert_eq!(prose.0, CheckCategory::Prose);
        assert_eq!(
            schema.1.iter().map(|value| text(value, "path")).collect::<Vec<_>>(),
            ["title", "contact.email"]
        );
        assert_eq!(diagnostic_severity(prose.1.first().unwrap()), CheckSeverity::Info);
        assert_eq!(diagnostic_severity(prose.1.get(1).unwrap()), CheckSeverity::Warning);
        assert!(diagnostic_groups(&json!({"diagnostics": []})).is_empty());
        let many = json!({"diagnostics": vec![json!({"category": "schema", "severity": "error"}); 101]});
        assert_eq!(diagnostic_groups(&many).first().unwrap().1.len(), 100);
    }
    #[test]
    fn test_diff_counts_distinguish_headers_from_changed_separators() {
        assert_eq!(diff_counts(""), (0, 0));
        assert_eq!(diff_counts("--- before\n+++ after\n"), (0, 0));
        assert_eq!(diff_counts("--- before\n+++ after\n@@ -1 +1 @@\n----\n++++\n"), (1, 1));
        assert_eq!(
            diff_counts("--- before\n+++ after\n@@ -1 +1 @@\n-old\n+new\n@@ -9 +9,2 @@\n context\n+extra\n\\ No newline at end of file\n"),
            (2, 1)
        );
    }
    #[test]
    fn test_exports_preserve_artifact_bytes_and_nested_patch_values() {
        let diff = export(Tool::Diff, &json!({"mode":"unified","diff":"--- before\n+++ after\n+<script>\n"}));
        assert_eq!(diff.content, "--- before\n+++ after\n+<script>\n");
        assert_eq!(diff.filename, "acorn-changes.diff");
        let value = json!({"operations":[{"op":"add","path":"/x","value":{"nested":[1,2]}}],"mode":"json-patch"});
        assert_eq!(
            serde_json::from_str::<Value>(&export(Tool::Diff, &value).content).unwrap(),
            value.get("operations").unwrap().clone()
        );
        assert_eq!(
            export(Tool::Crosswalk, &json!({"content":"{\n  \"title\": \"Converted\"\n}"})).content,
            "{\n  \"title\": \"Converted\"\n}"
        );
    }
    #[test]
    fn test_fair_counts_exclude_inapplicable_and_keep_unknown_evidence() {
        let value = json!({"findings":[
            {"indicator":{"principle":"findable"},"state":"supported"},
            {"indicator":{"principle":"findable"},"state":"indeterminate"},
            {"indicator":{"principle":"findable"},"state":"not-applicable"},
            {"indicator":{"principle":"accessible"},"state":"unsupported"}
        ]});
        assert_eq!(fair_counts(&value, "findable"), (1, 2));
        assert_eq!(fair_counts(&value, "reusable"), (0, 0));
    }
    #[test]
    fn test_gauge_handles_absence_nonfinite_and_off_scale_scores() {
        assert_eq!(gauge(None, 100.0), None);
        assert_eq!(gauge(Some(f64::NAN), 100.0), None);
        assert_eq!(gauge(Some(50.0), 0.0), None);
        assert_eq!(gauge(Some(-20.0), 100.0), Some(0.0));
        assert_eq!(gauge(Some(120.0), 100.0), Some(100.0));
        assert_eq!(gauge(Some(25.0), 100.0), Some(25.0));
    }
    #[test]
    fn test_severity_colors_are_independent_of_category() {
        for category in CheckCategory::iter() {
            assert!(!category_style(&category).1.is_empty());
            for (severity, tone) in [
                (CheckSeverity::Error, "danger"),
                (CheckSeverity::Info, "info"),
                (CheckSeverity::Suggestion, "suggestion"),
                (CheckSeverity::Warning, "warning"),
            ] {
                let diagnostic = json!({"category": category, "severity": severity});
                assert_eq!(severity_style(&diagnostic_severity(&diagnostic)).1, tone);
            }
        }
    }
    #[test]
    fn test_sort_diagnostics_is_stable_reversible_and_keeps_source_order() {
        let values = vec![
            json!({"path":"z", "code":"beta", "severity":"info"}),
            json!({"path":"b", "code":"alpha", "severity":"error"}),
            json!({"path":"A", "code":"gamma", "severity":"warning"}),
            json!({"path":"c", "code":"delta", "severity":"suggestion"}),
            json!({"path":"d", "code":"alpha", "severity":"error"}),
        ];
        let original = values.clone();
        let paths = |order, descending| {
            sort_diagnostics(values.iter().collect(), DiagnosticList::Validation, order, descending)
                .into_iter()
                .map(|value| text(value, "path"))
                .collect::<Vec<_>>()
        };
        assert_eq!(paths(DiagnosticSort::Severity, false), ["b", "d", "A", "c", "z"]);
        assert_eq!(paths(DiagnosticSort::Severity, true), ["z", "c", "A", "b", "d"]);
        assert_eq!(paths(DiagnosticSort::Field, false), ["A", "b", "c", "d", "z"]);
        assert_eq!(paths(DiagnosticSort::Field, true), ["z", "d", "c", "b", "A"]);
        assert_eq!(paths(DiagnosticSort::Rule, false), ["b", "d", "z", "c", "A"]);
        assert_eq!(paths(DiagnosticSort::Original, true), ["z", "b", "A", "c", "d"]);
        assert_eq!(values, original);
        assert!(sort_diagnostics(Vec::new(), DiagnosticList::Validation, DiagnosticSort::Severity, false).is_empty());
    }
    #[test]
    fn test_sort_diagnostics_uses_crosswalk_and_fair_fields() {
        let warnings = [json!({"field":"title", "lossType":"z"}), json!({"field":"authors", "lossType":"a"})];
        for sort in [DiagnosticSort::Field, DiagnosticSort::Rule] {
            let sorted = sort_diagnostics(warnings.iter().collect(), DiagnosticList::Crosswalk, sort, false);
            assert_eq!(text(sorted.first().unwrap(), "field"), "authors");
        }
        let findings = [
            json!({"indicator":{"id":"RDA-F2"}, "state":"supported"}),
            json!({"indicator":{"id":"RDA-A1"}, "state":"indeterminate"}),
        ];
        for sort in [DiagnosticSort::Field, DiagnosticSort::Rule] {
            let sorted = sort_diagnostics(findings.iter().collect(), DiagnosticList::Fair, sort, false);
            assert_eq!(text(sorted.first().unwrap(), "state"), "indeterminate");
        }
    }
}
