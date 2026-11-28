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
    let diagnostics = items(&value, "diagnostics").iter().collect::<Vec<_>>();
    let groups = diagnostic_groups(&diagnostics);
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
    assert!(diagnostic_groups(&[]).is_empty());
    let many = json!({"diagnostics": vec![json!({"category": "schema", "severity": "error"}); 101]});
    let many_diagnostics = items(&many, "diagnostics").iter().collect::<Vec<_>>();
    assert_eq!(diagnostic_groups(&many_diagnostics).first().unwrap().1.len(), 101);
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
    let formatted = export(
        Tool::Format,
        &json!({"changed":true,"content":"{\n  \"title\": \"Formatted\"\n}","format":"json"}),
    );
    assert_eq!(formatted.content, "{\n  \"title\": \"Formatted\"\n}");
    assert_eq!(formatted.filename, "acorn-formatted.json");
    let yaml = export(Tool::Format, &json!({"content":"title: Formatted\n","format":"yaml"}));
    assert_eq!(yaml.content, "title: Formatted\n");
    assert_eq!(yaml.filename, "acorn-formatted.yaml");
    assert_eq!(yaml.mime, "application/yaml;charset=utf-8");
    let markdown = export(Tool::Format, &json!({"content":"# Project\n","format":"markdown"}));
    assert_eq!(markdown.filename, "acorn-formatted.md");
    assert_eq!(markdown.mime, "text/markdown;charset=utf-8");
    let zon = export(Tool::Format, &json!({"content":"title: Project\n","format":"zon"}));
    assert_eq!(zon.filename, "acorn-formatted.zonf");
    assert_eq!(zon.mime, "text/plain;charset=utf-8");
    for (format, extension, mime) in [
        ("json-ld", "jsonld", "application/ld+json;charset=utf-8"),
        ("rdf-xml", "rdf", "application/rdf+xml;charset=utf-8"),
        ("turtle", "ttl", "text/turtle;charset=utf-8"),
    ] {
        let formatted = export(Tool::Format, &json!({"content":"source","format":format}));
        assert_eq!(formatted.content, "source");
        assert_eq!(formatted.filename, format!("acorn-formatted.{extension}"));
        assert_eq!(formatted.mime, mime);
    }
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
