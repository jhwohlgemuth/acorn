//! Pure display and export projections of portable operation results.
use crate::operations::Tool;
use serde_json::Value;

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
/// Read a string field without displaying JSON quotation marks.
pub fn text<'a>(value: &'a Value, field: &str) -> &'a str {
    value.get(field).and_then(Value::as_str).unwrap_or("Not provided")
}
/// Read a result array, preserving adapter ordering.
pub fn items<'a>(value: &'a Value, field: &str) -> &'a [Value] {
    value.get(field).and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default()
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
/// Bounded gauge position; absent or nonfinite values have no visual position.
pub fn gauge(value: Option<f64>, maximum: f64) -> Option<f64> {
    value
        .filter(|score| score.is_finite() && maximum.is_finite() && maximum > 0.0)
        .map(|score| (score / maximum * 100.0).clamp(0.0, 100.0))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
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
}
