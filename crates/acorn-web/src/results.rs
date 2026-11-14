use crate::details::DiagnosticButton;
use crate::geospatial_page::RorMapButton;
use crate::syntax::{Code, CodeBlock, CopyButton};
use crate::ui::{Icon, ResultTree, Route};
use acorn_analysis::{CheckCategory, CheckSeverity};
use acorn_web::{
    operations::Tool,
    presentation::{self, items, text, DiagnosticList, DiagnosticSort},
};
use alloc::collections::BTreeSet;
use case::CaseExt;
use dioxus::prelude::*;
use serde_json::Value;

#[component]
fn Complexity(value: Value) -> Element {
    let explanation = match text(&value, "dominance") {
        | "left" => "The left expression dominates the right asymptotically.",
        | "right" => "The right expression dominates the left asymptotically.",
        | "equivalent" => "Both expressions have equivalent asymptotic growth.",
        | "incomparable" => "Neither expression dominates under the supplied assumptions.",
        | _ => "Dominance could not be determined under the supplied assumptions.",
    };
    rsx! {
        p { class: "status", "{text(&value, \"dominance\")}" } p { "{explanation}" }
        div { class: "comparison-flow", Code { text: text(&value, "left").to_owned(), language: "expression", copyable: true } span { "↔" } Code { text: text(&value, "right").to_owned(), language: "expression", copyable: true } }
        details { summary { "Applied assumptions" } ResultTree { value: value.get("assumptions").unwrap_or(&Value::Null).clone(), depth: 0 } }
        h3 { "Growth illustration: n and n²" }
        p { class: "small muted", "Bundled example, independent of your expressions. Finite samples illustrate growth; they do not determine symbolic dominance." }
        svg { class: "growth-chart", view_box: "0 0 440 180", role: "img", "aria-label": "n and n squared growth illustration. Sample values are listed below.",
            path { d: "M30 10V150H420", stroke: "currentColor", fill: "none" }
            path { d: "M30 150L126 147.8L222 145.6L318 143.4L414 141.2", stroke: "var(--focus)", stroke_width: "3", fill: "none" }
            path { d: "M30 150L126 141.2L222 115L318 71.2L414 10", stroke: "currentColor", stroke_width: "3", stroke_dasharray: "6 4", fill: "none" }
            text { x: "405", y: "175", fill: "currentColor", "n" }
        }
        p { class: "small", "Solid: n · dashed: n²" }
        table { caption { "Illustrative samples" } thead { tr { th { scope: "col", "n" } th { scope: "col", "n²" } } } tbody { for n in [0, 4, 8, 12, 16] { tr { td { "{n}" } td { "{n * n}" } } } } }
    }
}
#[component]
fn DiagnosticSortControls(mut sort: Signal<DiagnosticSort>, mut descending: Signal<bool>, list: DiagnosticList) -> Element {
    let field_label = if list == DiagnosticList::Fair { "Indicator" } else { "Field" };
    let rule_label = match list {
        | DiagnosticList::Crosswalk => "Loss type",
        | DiagnosticList::Fair => "State",
        | DiagnosticList::Validation => "Rule",
    };
    let direction = match (sort(), descending()) {
        | (DiagnosticSort::Severity, false) => "Highest first",
        | (DiagnosticSort::Severity, true) => "Lowest first",
        | (_, false) => "A–Z",
        | (_, true) => "Z–A",
    };
    rsx! {
        div { class: "diagnostic-sort", role: "group", "aria-label": "Sort diagnostics",
            span { class: "muted", "Sort" }
            for (order, label, icon) in [(DiagnosticSort::Original, "Original", "filecheck"), (DiagnosticSort::Severity, "Severity", "alert"), (DiagnosticSort::Field, field_label, "code"), (DiagnosticSort::Rule, rule_label, "book")] {
                if order != DiagnosticSort::Severity || list != DiagnosticList::Crosswalk {
                    button { r#type: "button", "aria-pressed": sort() == order,
                        onclick: move |_| { sort.set(order); descending.set(false); },
                        Icon { name: icon } "{label}"
                    }
                }
            }
            button { r#type: "button", class: "sort-direction", disabled: sort() == DiagnosticSort::Original,
                "aria-label": "Sort direction: {direction}", title: "Reverse sort direction",
                onclick: move |_| { let next = !descending(); descending.set(next); },
                span { "aria-hidden": "true", if descending() { "↓" } else { "↑" } } "{direction}"
            }
        }
    }
}
#[component]
fn Difference(value: Value) -> Element {
    let operations = items(&value, "operations");
    let diff = text(&value, "diff");
    let (added, removed) = presentation::diff_counts(diff);
    rsx! {
        p { class: "status", if value.get("changed").unwrap_or(&Value::Null) == true { "Changes found" } else { "Documents are equal" } }
        div { class: "comparison-flow", span { "Before" } span { "→" } span { "After" } }
        if text(&value, "mode") == "unified" {
            p { "{added} added lines · {removed} removed lines" }
            CodeBlock { text: diff.to_owned(), language: "diff", label: "Copy diff" }
        } else {
            p { "{operations.len()} JSON Patch operations" }
            for operation in operations { div { class: "result-item", strong { "{text(operation, \"op\")} · {text(operation, \"path\")}" } ResultTree { value: operation.clone(), depth: 0 } } }
        }
    }
}
#[component]
fn Discovery(value: Value) -> Element {
    let mut session = use_context::<Signal<String>>();
    let mut hidden = use_signal(BTreeSet::<String>::new);
    let nav = use_navigator();
    let identifiers = items(&value, "identifiers").iter().take(100).collect::<Vec<_>>();
    let kinds = identifiers
        .iter()
        .map(|identifier| text(identifier, "kind").to_owned())
        .collect::<BTreeSet<_>>();
    let visible = identifiers
        .iter()
        .copied()
        .filter(|identifier| !hidden.read().contains(text(identifier, "kind")))
        .collect::<Vec<_>>();
    let all_selected = visible.len() == identifiers.len();
    let all_state = if all_selected {
        "true"
    } else if visible.is_empty() {
        "false"
    } else {
        "mixed"
    };
    let all_kinds = kinds.clone();
    rsx! {
        p { class: "status", "{value[\"count\"]} unique identifiers · normalized and deduplicated" }
        if items(&value, "identifiers").is_empty() { p { "No supported persistent identifiers found." } }
        if !identifiers.is_empty() {
            div { class: "pid-filters", role: "group", "aria-label": "Filter identifiers by type",
                button { r#type: "button", class: "pid-filter-all", "aria-pressed": all_state,
                    title: if all_selected { "Hide all PID types" } else { "Show all PID types" },
                    onclick: move |_| hidden.set(if all_selected { all_kinds.clone() } else { BTreeSet::new() }),
                    "All" span { class: "pid-filter-count", "{identifiers.len()}" }
                }
                for kind in kinds {
                    {let count = identifiers.iter().filter(|identifier| text(identifier, "kind") == kind).count(); rsx! {
                        button { r#type: "button", class: "pid-badge", "data-pid": kind.clone(), "aria-pressed": !hidden.read().contains(&kind),
                            onclick: move |_| {
                                let next = hidden().symmetric_difference(&BTreeSet::from([kind.clone()])).cloned().collect();
                                hidden.set(next);
                            },
                            "{kind}" span { class: "pid-filter-count", "{count}" }
                        }
                    }}
                }
            }
            p { class: "small muted pid-filter-summary", role: "status", "Showing {visible.len()} of {identifiers.len()} identifiers" }
            if visible.is_empty() { p { class: "muted small", "No PID types selected. Choose a type or All to show results." } }
        }
        for identifier in visible {
            {let normalized = text(identifier, "normalized").to_owned(); rsx! {
                div { class: "identifier-row", PidBadge { kind: text(identifier, "kind").to_owned() } Code { text: normalized.clone(), language: "identifier", copyable: true }
                    if text(identifier, "kind").eq_ignore_ascii_case("ror") { RorMapButton { identifier: normalized.clone() } }
                    button { onclick: move |_| { session.set(normalized.clone()); nav.push(Route::Page { page: "inspect".into() }); }, "Inspect →" }
                }
            }}
        }
        if value.get("truncated").unwrap_or(&Value::Null) == true { p { class: "status invalid", "Results were truncated at the 100-identifier limit. Narrow your input to discover additional identifiers." } }
    }
}
#[component]
fn Fair(value: Value, source: Option<String>) -> Element {
    let sort = use_signal(DiagnosticSort::default);
    let descending = use_signal(|| false);
    let (_, category_icon) = presentation::category_style(&CheckCategory::Fair);
    rsx! {
        h3 { class: "diagnostic-group-title", Icon { name: category_icon } "Local FAIR evidence" }
        for (principle, label) in [("findable", "Findable"), ("accessible", "Accessible"), ("interoperable", "Interoperable"), ("reusable", "Reusable")] {
            {let (supported, total) = presentation::fair_counts(&value, principle); rsx! {
                div { class: "proportion", p { strong { "{label}" } span { "{supported} / {total} supported" } }
                    if total > 0 { progress { value: "{supported}", max: "{total}", "aria-label": "{label}: {supported} of {total} applicable findings supported" } }
                    else { small { "No applicable findings" } }
                }
            }}
        }
        p { class: "small muted", "Based on supplied metadata only. Unknown evidence remains in the denominator; not-applicable findings are excluded. No identifiers are resolved." }
        details { class: "diagnostic-group", "data-category": "fair", open: true,
            summary { class: "diagnostic-group-summary", h3 { class: "diagnostic-group-title", Icon { name: category_icon } "FAIR findings" span { class: "badge", "{items(&value, \"findings\").len()}" } } }
            if items(&value, "findings").len() > 1 { DiagnosticSortControls { sort, descending, list: DiagnosticList::Fair } }
            ol { class: "diagnostics", role: "list", for finding in presentation::sort_diagnostics(items(&value, "findings").iter().collect(), DiagnosticList::Fair, sort(), descending()) {
                {let (severity, tone) = presentation::severity_style(&presentation::finding_severity(finding)); rsx! {
                li { class: "diagnostic", "data-tone": tone,
                    div { class: "diagnostic-heading", Icon { name: category_icon } strong { "{text(&finding[\"indicator\"], \"id\")}" } span { class: "badge diagnostic-badge", "{severity}" } }
                    p { class: "finding-state", {text(finding, "state").replace('-', " ").to_capitalized()} }
                    p { "{text(&finding[\"indicator\"], \"description\")}" }
                    if finding["evidence"].is_string() { p { "{text(finding, \"evidence\")}" } }
                    if text(finding, "state") != "supported" { p { class: "muted", "{text(finding, \"remediation\")}" } }
                    if let Some(source) = &source { DiagnosticButton { finding: finding.clone(), source: source.clone(), checks: value.clone(), tool: Tool::Fair } }
                }}}
            } }
        }
    }
}
#[component]
fn Inspection(value: Value) -> Element {
    rsx! {
        p { class: if text(&value, "status") == "valid" { "status" } else { "status invalid" }, "{text(&value, \"status\")}" if value.get("kind").is_some_and(Value::is_string) { " · " PidBadge { kind: text(&value, "kind").to_owned() } } }
        if text(&value, "status") == "ambiguous" { p { "Select an explicit identifier kind and run again." } ul { for candidate in items(&value, "candidates") { li { PidBadge { kind: candidate.as_str().unwrap_or("Unknown kind").to_owned() } } } } }
        if text(&value, "status") == "invalid" { p { "This value does not match the selected or recognized identifier syntax. Check the input and try again." } }
        if value.get("normalized").unwrap_or(&Value::Null).is_string() { dl { class: "result-fields", dt { "Normalized identifier" } dd { Code { text: text(&value, "normalized").to_owned(), language: "identifier", copyable: true } } } }
        if text(&value, "kind").eq_ignore_ascii_case("ror") && text(&value, "status") == "valid" { RorMapButton { identifier: text(&value, "normalized").to_owned() } }
        p { class: "small muted", "Local syntax inspection only. This does not establish that the identifier exists or resolves." }
    }
}
#[component]
fn PidBadge(kind: String) -> Element {
    rsx! { span { class: "badge pid-badge", "data-pid": kind.to_ascii_lowercase(), "{kind}" } }
}
#[component]
fn Readability(value: Value) -> Element {
    rsx! {
        for check in items(&value, "checks") {
            {let score = check["score"].as_f64().filter(|score| score.is_finite());
            let threshold = check["threshold"].as_f64().filter(|score| score.is_finite());
            let maximum = if text(check, "metric") == "fres" || text(check, "metric") == "lix" { 100.0 } else { 30.0 };
            rsx! { section { class: "readability-result", "data-status": if score.is_none() || threshold.is_none() { "unavailable" } else if check["success"] == true { "pass" } else { "review" },
                h3 { "{presentation::label(text(check, \"metric\"))}" }
                if let (Some(score), Some(threshold)) = (score, threshold) {
                    p { class: "score", "{score:.1}" }
                    p { class: if check["success"] == true { "status" } else { "status invalid" }, if check["success"] == true { "Within the configured threshold" } else { "Review the readability of this text" } }
                    p { class: "small muted", if text(check, "metric") == "fres" { "Target: at least {threshold} · higher is easier." } else { "Target: at most {threshold} · lower is easier." } " Actual score: {score:.2}." }
                    div { class: "score-gauge", "aria-hidden": "true",
                        span { class: "score-marker", style: "left:{presentation::gauge(Some(score), maximum).unwrap_or_default()}%" }
                        span { class: "threshold-marker", style: "left:{presentation::gauge(Some(threshold), maximum).unwrap_or_default()}%" }
                    }
                    p { class: "gauge-labels small muted", span { "0" } span { "Solid: score · dashed: threshold" } span { "{maximum}" } }
                    if !(0.0..=maximum).contains(&score) { p { class: "small", "Score is outside the illustrated scale; the exact value above is retained." } }
                } else { p { class: "status invalid", "A finite score and threshold are unavailable for this text." } }
            }}}
        }
    }
}
#[component]
pub(crate) fn ResultView(tool: Tool, value: Value, #[props(default)] source: Option<String>) -> Element {
    let sort = use_signal(DiagnosticSort::default);
    let descending = use_signal(|| false);
    let json = serde_json::to_string_pretty(&value).unwrap_or_default();
    let artifact = presentation::export(tool, &value);
    let copy = artifact.content.clone();
    let conversion_icon = if items(&value, "warnings").is_empty() { "check" } else { "alert" };
    let (crosswalk_label, crosswalk_icon) = presentation::category_style(&CheckCategory::Crosswalk);
    let (warning_label, warning_tone) = presentation::severity_style(&CheckSeverity::Warning);
    rsx! {
        match tool {
            Tool::Activity | Tool::Standard => rsx! { Validation { value: value.clone(), source: source.clone(), tool } },
            Tool::Crosswalk => rsx! {
                p { class: if items(&value, "warnings").is_empty() { "status diagnostic-status" } else { "status invalid diagnostic-status" },
                    Icon { name: conversion_icon }
                    "Converted · {items(&value, \"warnings\").len()} loss warnings"
                }
                if !items(&value, "warnings").is_empty() {
                details { class: "diagnostic-group", "data-category": "crosswalk", open: true,
                summary { class: "diagnostic-group-summary", h3 { class: "diagnostic-group-title", Icon { name: crosswalk_icon } "{crosswalk_label}" span { class: "badge", "{items(&value, \"warnings\").len()}" } } }
                if items(&value, "warnings").len() > 1 { DiagnosticSortControls { sort, descending, list: DiagnosticList::Crosswalk } }
                ol { class: "diagnostics", role: "list", for warning in presentation::sort_diagnostics(items(&value, "warnings").iter().take(100).collect(), DiagnosticList::Crosswalk, sort(), descending()) {
                    li { class: "diagnostic", "data-tone": warning_tone,
                        div { class: "diagnostic-heading", Icon { name: crosswalk_icon } strong { "{text(warning, \"field\")}" } span { class: "badge diagnostic-badge", "{warning_label}" } }
                        p { "{text(warning, \"lossType\")}" }
                        if warning["details"].is_string() { p { "{text(warning, \"details\")}" } }
                        if let Some(source) = &source { DiagnosticButton { finding: warning.clone(), source: source.clone(), checks: value.clone(), tool } }
                    }
                } }
                } }
                h3 { "Converted output" } CodeBlock { text: text(&value, "content").to_owned(), language: "json" }
            },
            Tool::Fair => rsx! { Fair { value: value.clone(), source: source.clone() } },
            Tool::Readability => rsx! { Readability { value: value.clone() } },
            Tool::Diff => rsx! { Difference { value: value.clone() } },
            Tool::Discover => rsx! { Discovery { value: value.clone() } },
            Tool::Inspect => rsx! { Inspection { value: value.clone() } },
            Tool::BigO => rsx! { Complexity { value: value.clone() } },
        }
        div { class: "actions result-actions",
            button { onclick: move |_| {
                let args = serde_json::to_string(&(artifact.content.clone(), artifact.filename.clone(), artifact.mime)).unwrap_or_default();
                document::eval(&format!("window.acornDownload(...{args})"));
            }, Icon { name: "download" } "Download result" }
            CopyButton { text: copy, label: "Copy result" }
        }
        p { id: "output-status", role: "status", class: "muted small" }
        details { summary { "Raw result JSON" } CodeBlock { text: json, language: "json", label: "Copy JSON" } }
    }
}
#[component]
fn Validation(value: Value, source: Option<String>, tool: Tool) -> Element {
    let sort = use_signal(DiagnosticSort::default);
    let descending = use_signal(|| false);
    let diagnostics = items(&value, "diagnostics");
    let valid = value.get("valid").unwrap_or(&Value::Null) == true;
    let status_icon = if valid { "check" } else { "alert" };
    rsx! {
        p { class: if valid { "status diagnostic-status" } else { "status invalid diagnostic-status" },
            Icon { name: status_icon }
            span { if valid { "Valid" } else { "Review needed" } " · {diagnostics.len()} diagnostics" }
        }
        if diagnostics.iter().any(|diagnostic| text(diagnostic, "category") == "fair") {
            p { class: "small muted", "Includes local FAIR evidence checks. Suggestions and information do not affect schema validity; identifiers are not resolved." }
        }
        if diagnostics.len() > 1 { DiagnosticSortControls { sort, descending, list: DiagnosticList::Validation } }
        for (category, group) in presentation::diagnostic_groups(&value) {
            {let (label, icon) = presentation::category_style(&category); rsx! {
            details { key: "{category}", class: "diagnostic-group", "data-category": category.to_string(), open: true,
                summary { class: "diagnostic-group-summary",
                    h3 { class: "diagnostic-group-title", Icon { name: icon } "{label}" span { class: "badge", "{group.len()}" } }
                }
                ol { class: "diagnostics", role: "list", for diagnostic in presentation::sort_diagnostics(group, DiagnosticList::Validation, sort(), descending()) {
                    {let (severity, tone) = presentation::severity_style(&presentation::diagnostic_severity(diagnostic)); rsx! {
                    li { class: "diagnostic", "data-tone": tone,
                        div { class: "diagnostic-heading", Icon { name: icon }
                            strong { "{diagnostic[\"path\"].as_str().filter(|path| !path.is_empty()).unwrap_or(\"root\")}" }
                            span { class: "badge diagnostic-badge", "{severity}" }
                        }
                        p { "{text(diagnostic, \"message\")}" }
                        small { class: "diagnostic-meta", "{text(diagnostic, \"kind\")}" if diagnostic["code"].is_string() { " · Code: {text(diagnostic, \"code\")}" } }
                        if let Some(source) = &source { DiagnosticButton { finding: diagnostic.clone(), source: source.clone(), checks: value.clone(), tool } }
                    }}}
                } }
            }}}
        }
        if diagnostics.len() > 100 { p { "Showing the first 100 diagnostics, grouped by category. Download the result for all diagnostics." } }
    }
}
