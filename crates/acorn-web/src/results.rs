use crate::details::DiagnosticButton;
use crate::geospatial_page::RorMapButton;
use crate::search_ui::SearchField;
use crate::syntax::{Code, CodeBlock, CopyButton};
use crate::ui::{Icon, ResultTree, Route};
use acorn_analysis::{CheckCategory, CheckSeverity};
use acorn_core::big_o::{Exponent, Expression, Term};
use acorn_web::{
    operations::Tool,
    presentation::{self, items, text, DiagnosticList, DiagnosticSort},
    search::SearchPattern,
};
use alloc::collections::BTreeSet;
use alloc::rc::Rc;
use case::CaseExt;
use dioxus::prelude::*;
use serde_json::Value;

#[component]
fn Complexity(value: Value) -> Element {
    let dominance = text(&value, "dominance");
    let explanation = match dominance {
        | "left" => "The left expression grows faster asymptotically.",
        | "right" => "The right expression grows faster asymptotically.",
        | "equivalent" => "Both expressions have equivalent asymptotic growth.",
        | "incomparable" => "Independent growth directions prevent a single ordering.",
        | _ => "The available assumptions do not prove an ordering.",
    };
    let left = text(&value, "left").to_owned();
    let right = text(&value, "right").to_owned();
    let assumptions = value.get("assumptions").unwrap_or(&Value::Null);
    let samples = complexity_samples(&left, &right, assumptions);
    rsx! {
        section { class: "complexity-verdict", "data-dominance": dominance,
            span { class: "complexity-verdict-label", "{presentation::label(dominance)}" }
            h3 { "{explanation}" }
            p { class: "small muted", "A symbolic comparison under the applied assumptions." }
        }
        div { class: "complexity-expressions",
            article { class: "complexity-expression", "data-side": "left", span { class: "complexity-expression-label", "Left" } Code { text: left.clone(), language: "expression", copyable: true } }
            span { class: "complexity-versus", "aria-hidden": "true", "vs" }
            article { class: "complexity-expression", "data-side": "right", span { class: "complexity-expression-label", "Right" } Code { text: right.clone(), language: "expression", copyable: true } }
        }
        if let Some(samples) = samples {
            {let maximum = samples.iter().flat_map(|sample| [sample.left, sample.right]).map(f64::ln_1p).fold(0.0_f64, f64::max);
            let maximum_n = samples.last().map_or(1, |sample| sample.n);
            let left_path = complexity_path(&samples, maximum, |sample| sample.left);
            let right_path = complexity_path(&samples, maximum, |sample| sample.right); rsx! {
                section { class: "complexity-graph",
                    div { class: "complexity-graph-heading", h3 { "Projected growth curves" } div { class: "complexity-legend", span { "data-side": "left", "Left" } span { "data-side": "right", "Right" } } }
                    svg { class: "growth-chart", view_box: "0 0 520 220", role: "img", "aria-label": "Projected growth curves for the submitted left and right expressions. Sample values are listed below.",
                        for y in [30, 70, 110, 150, 190] { line { x1: "46", y1: "{y}", x2: "500", y2: "{y}", class: "chart-grid" } }
                        line { x1: "46", y1: "20", x2: "46", y2: "190", class: "chart-axis" }
                        line { x1: "46", y1: "190", x2: "500", y2: "190", class: "chart-axis" }
                        path { d: left_path, class: "complexity-line", "data-side": "left" }
                        path { d: right_path, class: "complexity-line", "data-side": "right" }
                        text { x: "12", y: "24", class: "chart-label", "value" }
                        text { x: "490", y: "212", class: "chart-label", "n" }
                        for (index, sample) in samples.iter().enumerate() { if index % 2 == 0 || index + 1 == samples.len() { text { x: "{chart_x(index, samples.len())}", y: "208", class: "chart-tick", "{sample.n}" } } }
                    }
                    p { class: "small muted", "Zoomed-out view from n = 1 to {maximum_n} with logarithmic x and y axes · growing symbols use n · fixed symbols use 2 · shrinking symbols use 1/n. Curves illustrate finite samples; the symbolic result above remains authoritative." }
                    details { summary { "View sampled values" }
                        table { caption { "Projected values for submitted expressions" } thead { tr { th { scope: "col", "n" } th { scope: "col", "data-side": "left", "Left" } th { scope: "col", "data-side": "right", "Right" } } } tbody { for sample in samples { tr { td { "{sample.n}" } td { "{format_sample(sample.left)}" } td { "{format_sample(sample.right)}" } } } } }
                    }
                }
            }}
        } else {
            p { class: "complexity-graph-unavailable small muted", "A numeric curve is unavailable for one or more symbolic terms. The asymptotic comparison above is still valid." }
        }
        details { summary { "Applied assumptions" } ResultTree { value: assumptions.clone(), depth: 0 } }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct ComplexitySample {
    n: u32,
    left: f64,
    right: f64,
}
fn complexity_samples(left: &str, right: &str, assumptions: &Value) -> Option<Vec<ComplexitySample>> {
    Expression::parse(left).ok().zip(Expression::parse(right).ok()).and_then(|(left, right)| {
        let samples = [1_u32, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096]
            .into_iter()
            .map_while(|n| {
                evaluate_complexity(left.root(), f64::from(n), assumptions)
                    .zip(evaluate_complexity(right.root(), f64::from(n), assumptions))
                    .and_then(|(left, right)| (left.is_finite() && right.is_finite()).then_some(ComplexitySample { n, left, right }))
            })
            .collect::<Vec<_>>();
        (samples.len() >= 2).then_some(samples)
    })
}
fn evaluate_complexity(term: &Term, n: f64, assumptions: &Value) -> Option<f64> {
    match term {
        | Term::Add(terms) => terms
            .iter()
            .try_fold(0.0, |sum, term| evaluate_complexity(term, n, assumptions).map(|value| sum + value)),
        | Term::Constant(value) => Some(*value as f64),
        | Term::Divide(numerator, denominator) => evaluate_complexity(numerator, n, assumptions)
            .zip(evaluate_complexity(denominator, n, assumptions))
            .and_then(|(numerator, denominator)| (denominator > 0.0).then_some(numerator / denominator)),
        | Term::Log(argument) => evaluate_complexity(argument, n, assumptions).map(|value| value.max(1.0).ln()),
        | Term::LogBase { argument, .. } => evaluate_complexity(argument, n, assumptions).map(|value| value.max(1.0).log2()),
        | Term::Multiply(terms) => terms.iter().try_fold(1.0, |product, term| {
            evaluate_complexity(term, n, assumptions).map(|value| product * value)
        }),
        | Term::OutputWork(argument) => evaluate_complexity(argument, n, assumptions),
        | Term::Polylog(_) => None,
        | Term::Power { base, exponent } => evaluate_complexity(base, n, assumptions)
            .zip(evaluate_exponent(exponent, n, assumptions))
            .map(|(base, exponent)| base.powf(exponent)),
        | Term::Variable(name) => Some(symbol_value(name, n, assumptions)),
    }
}
fn evaluate_exponent(exponent: &Exponent, n: f64, assumptions: &Value) -> Option<f64> {
    match exponent {
        | Exponent::Integer(value) => Some(*value as f64),
        | Exponent::Rational(numerator, denominator) => Some(*numerator as f64 / *denominator as f64),
        | Exponent::Variable(name) => Some(symbol_value(name, n, assumptions)),
        | Exponent::Difference(left, right) => Some(symbol_value(left, n, assumptions) - symbol_value(right, n, assumptions)),
    }
}
fn symbol_value(name: &str, n: f64, assumptions: &Value) -> f64 {
    match assumptions.get(name).and_then(Value::as_str) {
        | Some("fixed-positive" | "fixed-log-base") => 2.0,
        | Some("shrinking-positive") => n.recip(),
        | _ => n,
    }
}
fn chart_x(index: usize, count: usize) -> f64 {
    46.0 + 454.0 * index as f64 / count.saturating_sub(1).max(1) as f64
}
fn complexity_path(samples: &[ComplexitySample], maximum: f64, value: impl Fn(&ComplexitySample) -> f64) -> String {
    samples
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let x = chart_x(index, samples.len());
            let y = 190.0 - 160.0 * value(sample).ln_1p() / maximum.max(f64::EPSILON);
            format!("{}{x:.1},{y:.1}", if index == 0 { "M" } else { "L" })
        })
        .collect()
}
fn format_sample(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("{value:.2e}")
    } else if value.fract().abs() < 0.001 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
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
fn ResultSearch(query: Signal<String>, regex: Signal<bool>, case_sensitive: Signal<bool>, label: &'static str, error: Option<String>) -> Element {
    rsx! { SearchField { label, placeholder: "Search results", query, regex, case_sensitive, error, compact: true } }
}
fn matches_query(value: &Value, pattern: &SearchPattern) -> bool {
    pattern.matches(&value.to_string())
}
#[component]
fn Difference(value: Value) -> Element {
    let mut hidden = use_signal(BTreeSet::<String>::new);
    let mut sort = use_signal(|| PatchSort::Original);
    let mut descending = use_signal(|| false);
    let operations = items(&value, "operations");
    let diff = text(&value, "diff");
    let (added, removed) = presentation::diff_counts(diff);
    let kinds = operations
        .iter()
        .map(|operation| text(operation, "op").to_owned())
        .collect::<BTreeSet<_>>();
    let mut visible = operations
        .iter()
        .enumerate()
        .filter(|(_, operation)| !hidden.read().contains(text(operation, "op")))
        .collect::<Vec<_>>();
    visible.sort_by(|(left_index, left), (right_index, right)| {
        let order = match sort() {
            | PatchSort::Original => left_index.cmp(right_index),
            | PatchSort::Operation => text(left, "op").cmp(text(right, "op")),
            | PatchSort::Path => text(left, "path").cmp(text(right, "path")),
        };
        order.then_with(|| left_index.cmp(right_index))
    });
    if descending() && sort() != PatchSort::Original {
        visible.reverse();
    }
    let all_selected = visible.len() == operations.len();
    let all_kinds = kinds.clone();
    rsx! {
        p { class: "status", if value.get("changed").unwrap_or(&Value::Null) == true { "Changes found" } else { "Documents are equal" } }
        div { class: "comparison-flow", span { "Before" } span { "→" } span { "After" } }
        if text(&value, "mode") == "unified" {
            p { "{added} added lines · {removed} removed lines" }
            CodeBlock { text: diff.to_owned(), language: "diff", label: "Copy diff" }
        } else {
            p { "{operations.len()} JSON Patch operations" }
            if operations.len() > 1 {
                div { class: "result-controls",
                    div { class: "result-filters", role: "group", "aria-label": "Filter JSON Patch operations",
                        button { r#type: "button", "aria-pressed": all_selected,
                            title: if all_selected { "Hide all operation types" } else { "Show all operation types" },
                            onclick: move |_| hidden.set(if all_selected { all_kinds.clone() } else { BTreeSet::new() }),
                            "All" span { class: "filter-count", "{operations.len()}" }
                        }
                        for kind in kinds {
                            {let count = operations.iter().filter(|operation| text(operation, "op") == kind).count(); rsx! {
                                button { r#type: "button", class: "patch-filter", "data-operation": kind.clone(), "aria-pressed": !hidden.read().contains(&kind),
                                    onclick: move |_| {
                                        let next = hidden().symmetric_difference(&BTreeSet::from([kind.clone()])).cloned().collect();
                                        hidden.set(next);
                                    },
                                    "{kind}" span { class: "filter-count", "{count}" }
                                }
                            }}
                        }
                    }
                    div { class: "result-sort", role: "group", "aria-label": "Sort JSON Patch operations",
                        span { class: "muted", "Sort" }
                        for (order, label) in [(PatchSort::Original, "Original"), (PatchSort::Operation, "Operation"), (PatchSort::Path, "Path")] {
                            button { r#type: "button", "aria-pressed": sort() == order, onclick: move |_| { sort.set(order); descending.set(false); }, "{label}" }
                        }
                        button { r#type: "button", class: "sort-direction", disabled: sort() == PatchSort::Original,
                            "aria-label": if descending() { "Sort ascending" } else { "Sort descending" }, title: "Reverse sort direction",
                            onclick: move |_| descending.toggle(), span { "aria-hidden": "true", if descending() { "↓" } else { "↑" } }
                        }
                    }
                }
                p { class: "small muted result-summary", role: "status", "Showing {visible.len()} of {operations.len()} operations" }
            }
            if visible.is_empty() { p { class: "muted small", "No operation types selected. Choose a type or All to show results." } }
            ol { class: "patch-operations",
                for (index, operation) in visible { PatchOperation { index, operation: (*operation).clone() } }
            }
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum PatchSort {
    Original,
    Operation,
    Path,
}
#[component]
fn PatchOperation(index: usize, operation: Value) -> Element {
    let position = index.saturating_add(1);
    let kind = text(&operation, "op").to_owned();
    let path = text(&operation, "path").to_owned();
    let from = operation.get("from").and_then(Value::as_str).map(str::to_owned);
    let value = operation.get("value").cloned();
    rsx! {
        li { class: "patch-operation", "data-operation": kind.clone(),
            div { class: "patch-operation-heading",
                span { class: "patch-operation-index", title: "Operation {position}", "{position}" }
                strong { class: "patch-operation-kind", "{kind}" }
                code { class: "patch-operation-path", "{path}" }
            }
            if let Some(from) = from {
                div { class: "patch-operation-detail",
                    span { class: "patch-operation-label", "From" }
                    code { class: "patch-operation-path", "{from}" }
                }
            }
            if let Some(value) = value {
                div { class: "patch-operation-detail",
                    span { class: "patch-operation-label", "Value" }
                    div { class: "patch-operation-value", ResultTree { value, depth: 0 } }
                }
            }
        }
    }
}
#[component]
fn Discovery(value: Value) -> Element {
    let mut session = use_context::<Signal<String>>();
    let mut hidden = use_signal(BTreeSet::<String>::new);
    let mut sort = use_signal(|| IdentifierSort::Original);
    let mut descending = use_signal(|| false);
    let nav = use_navigator();
    let identifiers = items(&value, "identifiers").iter().take(100).collect::<Vec<_>>();
    let kinds = identifiers
        .iter()
        .map(|identifier| text(identifier, "kind").to_owned())
        .collect::<BTreeSet<_>>();
    let mut visible = identifiers
        .iter()
        .enumerate()
        .filter(|(_, identifier)| !hidden.read().contains(text(identifier, "kind")))
        .map(|(index, identifier)| (index, *identifier))
        .collect::<Vec<_>>();
    visible.sort_by(|(left_index, left), (right_index, right)| {
        let order = match sort() {
            | IdentifierSort::Original => left_index.cmp(right_index),
            | IdentifierSort::Type => text(left, "kind").cmp(text(right, "kind")),
            | IdentifierSort::Value => text(left, "normalized").cmp(text(right, "normalized")),
        };
        order.then_with(|| left_index.cmp(right_index))
    });
    if descending() && sort() != IdentifierSort::Original {
        visible.reverse();
    }
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
            div { class: "result-controls",
                div { class: "pid-filters result-filters", role: "group", "aria-label": "Filter identifiers by type",
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
                if identifiers.len() > 1 {
                    div { class: "result-sort", role: "group", "aria-label": "Sort identifiers",
                        span { class: "muted", "Sort" }
                        for (order, label) in [(IdentifierSort::Original, "Original"), (IdentifierSort::Type, "Type"), (IdentifierSort::Value, "Value")] {
                            button { r#type: "button", "aria-pressed": sort() == order, onclick: move |_| { sort.set(order); descending.set(false); }, "{label}" }
                        }
                        button { r#type: "button", class: "sort-direction", disabled: sort() == IdentifierSort::Original,
                            "aria-label": if descending() { "Sort ascending" } else { "Sort descending" }, title: "Reverse sort direction",
                            onclick: move |_| descending.toggle(), span { "aria-hidden": "true", if descending() { "↓" } else { "↑" } }
                        }
                    }
                }
            }
            p { class: "small muted pid-filter-summary", role: "status", "Showing {visible.len()} of {identifiers.len()} identifiers" }
            if visible.is_empty() { p { class: "muted small", "No PID types selected. Choose a type or All to show results." } }
        }
        for (_, identifier) in visible {
            {let normalized = text(identifier, "normalized").to_owned(); rsx! {
                div { class: "identifier-row", PidBadge { kind: text(identifier, "kind").to_owned() } span { class: "inline-code", code { class: "syntax language-identifier", "{normalized}" } CopyButton { text: normalized.clone() } }
                    if text(identifier, "kind").eq_ignore_ascii_case("ror") { RorMapButton { identifier: normalized.clone() } }
                    button { onclick: move |_| { session.set(normalized.clone()); nav.push(Route::Page { page: "inspect".into() }); }, "Inspect →" }
                }
            }}
        }
        if value.get("truncated").unwrap_or(&Value::Null) == true { p { class: "status invalid", "Results were truncated at the 100-identifier limit. Narrow your input to discover additional identifiers." } }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum IdentifierSort {
    Original,
    Type,
    Value,
}
#[component]
fn Fair(value: Rc<Value>, source: Option<Rc<String>>) -> Element {
    let sort = use_signal(DiagnosticSort::default);
    let descending = use_signal(|| false);
    let query = use_signal(String::new);
    let regex = use_signal(|| false);
    let case_sensitive = use_signal(|| false);
    let pattern = SearchPattern::new(&query(), regex(), case_sensitive());
    let findings = items(&value, "findings");
    let skipped_label = value
        .get("skippedChecks")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "));
    let visible = findings
        .iter()
        .filter(|finding| pattern.as_ref().is_ok_and(|pattern| matches_query(finding, pattern)))
        .collect::<Vec<_>>();
    let (_, category_icon) = presentation::category_style(&CheckCategory::Fair);
    rsx! {
        h3 { class: "diagnostic-group-title", Icon { name: category_icon } "Local FAIR evidence" }
        for (principle, label) in [("findable", "Findable"), ("accessible", "Accessible"), ("interoperable", "Interoperable"), ("reusable", "Reusable")] {
            {let (supported, total) = presentation::fair_counts(&value, principle);
             let tone = if supported * 4 < total { "red" } else if supported * 4 < total * 3 { "yellow" } else { "green" };
             rsx! {
                div { class: "proportion", p { strong { "{label}" } span { "{supported} / {total} supported" } }
                    if total > 0 { progress { value: "{supported}", max: "{total}", "data-tone": tone, "aria-label": "{label}: {supported} of {total} applicable findings supported" } }
                    else { small { "No applicable findings" } }
                }
            }}
        }
        p { class: "small muted", "Based on supplied metadata only. Unknown evidence remains in the denominator; not-applicable findings are excluded. No identifiers are resolved." }
        if let Some(skipped) = skipped_label.filter(|label| !label.is_empty()) {
            p { class: "small muted", "Skipped: {skipped}" }
        }
        details { class: "diagnostic-group", "data-category": "fair", open: true,
            summary { class: "diagnostic-group-summary", h3 { class: "diagnostic-group-title", Icon { name: category_icon } "FAIR findings" span { class: "badge", "{findings.len()}" } } }
            if findings.len() > 1 { div { class: "result-controls", ResultSearch { query, regex, case_sensitive, label: "Filter FAIR findings", error: pattern.as_ref().err().cloned() } } DiagnosticSortControls { sort, descending, list: DiagnosticList::Fair } }
            if visible.len() != findings.len() { p { class: "small muted result-summary", role: "status", "Showing {visible.len()} of {findings.len()} findings" } }
            if visible.is_empty() { p { class: "muted small", if findings.is_empty() { "No FAIR checks were run." } else { "No findings match this filter." } } }
            ol { class: "diagnostics", role: "list", for finding in presentation::sort_diagnostics(visible, DiagnosticList::Fair, sort(), descending()) {
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
    let query = use_signal(String::new);
    let regex = use_signal(|| false);
    let case_sensitive = use_signal(|| false);
    let pattern = SearchPattern::new(&query(), regex(), case_sensitive());
    let mut raw = use_signal(|| false);
    // Shared once per result so diagnostic rows hold pointers instead of copies of the source and result.
    let shared = use_memo(use_reactive((&value, &source), |(value, source)| (Rc::new(value), source.map(Rc::new))));
    let artifact = use_memo(use_reactive((&tool, &value), |(tool, value)| presentation::export(tool, &value)));
    let (checks, source) = shared();
    let warnings = items(&value, "warnings");
    let visible_warnings = warnings
        .iter()
        .take(100)
        .filter(|warning| pattern.as_ref().is_ok_and(|pattern| matches_query(warning, pattern)))
        .collect::<Vec<_>>();
    let conversion_icon = if warnings.is_empty() { "check" } else { "alert" };
    let (crosswalk_label, crosswalk_icon) = presentation::category_style(&CheckCategory::Crosswalk);
    let (warning_label, warning_tone) = presentation::severity_style(&CheckSeverity::Warning);
    rsx! {
        match tool {
            Tool::Activity | Tool::Standard => rsx! { Validation { value: checks.clone(), source: source.clone(), tool } },
            Tool::Crosswalk => rsx! {
                p { class: if warnings.is_empty() { "status diagnostic-status" } else { "status invalid diagnostic-status" },
                    Icon { name: conversion_icon }
                    "Converted · {warnings.len()} loss warnings"
                }
                if !warnings.is_empty() {
                details { class: "diagnostic-group", "data-category": "crosswalk", open: true,
                summary { class: "diagnostic-group-summary", h3 { class: "diagnostic-group-title", Icon { name: crosswalk_icon } "{crosswalk_label}" span { class: "badge", "{warnings.len()}" } } }
                if warnings.len() > 1 { div { class: "result-controls", ResultSearch { query, regex, case_sensitive, label: "Filter crosswalk warnings", error: pattern.as_ref().err().cloned() } } DiagnosticSortControls { sort, descending, list: DiagnosticList::Crosswalk } }
                if !query().trim().is_empty() { p { class: "small muted result-summary", role: "status", "Showing {visible_warnings.len()} of {warnings.len()} warnings" } }
                if visible_warnings.is_empty() { p { class: "muted small", "No warnings match this filter." } }
                ol { class: "diagnostics", role: "list", for warning in presentation::sort_diagnostics(visible_warnings, DiagnosticList::Crosswalk, sort(), descending()) {
                    li { class: "diagnostic", "data-tone": warning_tone,
                        div { class: "diagnostic-heading", Icon { name: crosswalk_icon } strong { "{text(warning, \"field\")}" } span { class: "badge diagnostic-badge", "{warning_label}" } }
                        p { "{text(warning, \"lossType\")}" }
                        if warning["details"].is_string() { p { "{text(warning, \"details\")}" } }
                        if let Some(source) = &source { DiagnosticButton { finding: warning.clone(), source: source.clone(), checks: checks.clone(), tool } }
                    }
                } }
                } }
                h3 { "Converted output" } CodeBlock { text: text(&value, "content").to_owned(), language: "json" }
            },
            Tool::Fair => rsx! { Fair { value: checks.clone(), source: source.clone() } },
            Tool::Readability => rsx! { Readability { value: value.clone() } },
            Tool::Diff => rsx! { Difference { value: value.clone() } },
            Tool::Discover => rsx! { Discovery { value: value.clone() } },
            Tool::Inspect => rsx! { Inspection { value: value.clone() } },
            Tool::BigO => rsx! { Complexity { value: value.clone() } },
        }
        div { class: "actions result-actions",
            button { onclick: move |_| {
                let artifact = artifact.read();
                let args = serde_json::to_string(&(&artifact.content, &artifact.filename, artifact.mime)).unwrap_or_default();
                document::eval(&format!("window.acornDownload(...{args})"));
            }, Icon { name: "download" } "Download result" }
            CopyButton { text: artifact.read().content.clone(), label: "Copy result" }
        }
        p { id: "output-status", role: "status", class: "muted small" }
        details { summary { onclick: move |_| raw.toggle(), "Raw result JSON" }
            if raw() { CodeBlock { text: serde_json::to_string_pretty(&value).unwrap_or_default(), language: "json", label: "Copy JSON" } }
        }
    }
}
#[component]
fn Validation(value: Rc<Value>, source: Option<Rc<String>>, tool: Tool) -> Element {
    let sort = use_signal(DiagnosticSort::default);
    let descending = use_signal(|| false);
    let query = use_signal(String::new);
    let regex = use_signal(|| false);
    let case_sensitive = use_signal(|| false);
    let pattern = SearchPattern::new(&query(), regex(), case_sensitive());
    let diagnostics = items(&value, "diagnostics");
    let skipped_label = value
        .get("skippedChecks")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "));
    let visible_count = diagnostics
        .iter()
        .take(100)
        .filter(|diagnostic| pattern.as_ref().is_ok_and(|pattern| matches_query(diagnostic, pattern)))
        .count();
    let valid = value.get("valid").unwrap_or(&Value::Null) == true;
    let schema_skipped = value
        .get("skippedChecks")
        .and_then(Value::as_array)
        .is_some_and(|items| items.iter().any(|item| item == "schema"));
    let status_icon = if valid { "check" } else { "alert" };
    rsx! {
        p { class: if valid { "status diagnostic-status" } else { "status invalid diagnostic-status" },
            Icon { name: status_icon }
            span { if schema_skipped { "Schema not checked" } else if valid { "Valid" } else { "Review needed" } " · {diagnostics.len()} diagnostics" }
        }
        if diagnostics.iter().any(|diagnostic| text(diagnostic, "category") == "fair") {
            p { class: "small muted", "Includes local FAIR evidence checks. Suggestions and information do not affect schema validity; identifiers are not resolved." }
        }
        if let Some(skipped) = skipped_label.filter(|label| !label.is_empty()) {
            p { class: "small muted", "Skipped: {skipped}" }
        }
        if diagnostics.len() > 1 { div { class: "result-controls", ResultSearch { query, regex, case_sensitive, label: "Filter diagnostics", error: pattern.as_ref().err().cloned() } } DiagnosticSortControls { sort, descending, list: DiagnosticList::Validation } }
        if visible_count != diagnostics.len() { p { class: "small muted result-summary", role: "status", "Showing {visible_count} of {diagnostics.len()} diagnostics" } }
        if visible_count == 0 { p { class: "muted small", if diagnostics.is_empty() { "No diagnostics from the selected checks." } else { "No diagnostics match this filter." } } }
        for (category, group) in presentation::diagnostic_groups(&value) {
            {let group = group.into_iter().filter(|diagnostic| pattern.as_ref().is_ok_and(|pattern| matches_query(diagnostic, pattern))).collect::<Vec<_>>();
            let (label, icon) = presentation::category_style(&category); rsx! {
            if !group.is_empty() { details { key: "{category}", class: "diagnostic-group", "data-category": category.to_string(), open: true,
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
            } }}}
        }
        if diagnostics.len() > 100 { p { "Showing the first 100 diagnostics, grouped by category. Download the result for all diagnostics." } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::Mutation;
    use serde_json::json;

    #[test]
    fn test_json_patch_hierarchy_preserves_operation_details() {
        let value = json!({
            "changed": true,
            "mode": "json-patch",
            "operations": [
                {"op": "add", "path": "/authors/0", "value": {"name": "Ada"}},
                {"op": "move", "from": "/draft", "path": "/published"}
            ]
        });
        let mut dom = VirtualDom::new_with_props(Difference, DifferenceProps { value });
        let mutations = dom.rebuild_to_vec();
        let text = mutations
            .edits
            .iter()
            .filter_map(|edit| match edit {
                | Mutation::CreateTextNode { value, .. } | Mutation::SetText { value, .. } => Some(value.as_str()),
                | _ => None,
            })
            .collect::<Vec<_>>();
        for expected in ["1", "add", "/authors/0", "Ada", "2", "move", "/draft", "/published"] {
            assert!(text.contains(&expected), "missing {expected:?}: {text:?}");
        }
    }

    #[test]
    fn test_complexity_samples_plot_submitted_expressions_and_assumptions() {
        let assumptions = json!({"n": "growing", "k": "fixed-positive"});
        let samples = complexity_samples("O(n log n + k)", "O(n^2)", &assumptions).expect("supported expressions should produce samples");
        assert_eq!(samples.len(), 13);
        assert_eq!(samples.first().copied(), Some(ComplexitySample { n: 1, left: 2.0, right: 1.0 }));
        assert_eq!(
            samples.last().copied(),
            Some(ComplexitySample {
                n: 4096,
                left: 4096.0 * 4096.0_f64.ln() + 2.0,
                right: 16_777_216.0
            })
        );
        let path = complexity_path(
            &samples,
            samples.iter().map(|sample| sample.right.ln_1p()).fold(0.0_f64, f64::max),
            |sample| sample.left,
        );
        assert!(path.starts_with('M'), "path must begin with a move: {path}");
        assert_eq!(path.matches('L').count(), 12, "path must connect every sample: {path}");
        let fast = complexity_samples("O(n^n)", "O(n^2)", &assumptions).expect("finite prefix should remain graphable");
        assert_eq!(fast.last().map(|sample| sample.n), Some(128));
        assert!(complexity_samples("O(polylog(n))", "O(n)", &assumptions).is_none());
    }
}
