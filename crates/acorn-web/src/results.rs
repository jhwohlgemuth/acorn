use crate::ui::{ResultTree, Route};
use acorn_web::{
    operations::Tool,
    presentation::{self, items, text},
};
use dioxus::prelude::*;
use serde_json::Value;

#[component]
pub(crate) fn ResultView(tool: Tool, value: Value) -> Element {
    let json = serde_json::to_string_pretty(&value).unwrap_or_default();
    let artifact = presentation::export(tool, &value);
    let copy = artifact.content.clone();
    rsx! {
        match tool {
            Tool::Activity | Tool::Standard => rsx! { Validation { value: value.clone() } },
            Tool::Crosswalk => rsx! {
                p { class: if items(&value, "warnings").is_empty() { "status" } else { "status invalid" }, "Converted · {items(&value, \"warnings\").len()} loss warnings" }
                ol { class: "diagnostics", for warning in items(&value, "warnings").iter().take(100) {
                    li { strong { "{text(warning, \"field\")}" } p { "{text(warning, \"lossType\")}" } if warning["details"].is_string() { p { "{text(warning, \"details\")}" } } }
                } }
                h3 { "Converted output" } pre { "{text(&value, \"content\")}" }
            },
            Tool::Fair => rsx! { Fair { value: value.clone() } },
            Tool::Readability => rsx! { Readability { value: value.clone() } },
            Tool::Diff => rsx! { Difference { value: value.clone() } },
            Tool::Discover => rsx! { Discovery { value: value.clone() } },
            Tool::Inspect => rsx! { Inspection { value: value.clone() } },
            Tool::BigO => rsx! { Complexity { value: value.clone() } },
        }
        div { class: "actions",
            button { onclick: move |_| {
                let args = serde_json::to_string(&(artifact.content.clone(), artifact.filename.clone(), artifact.mime)).unwrap_or_default();
                document::eval(&format!("window.acornDownload(...{args})"));
            }, "Download result" }
            button { onclick: move |_| { let encoded = serde_json::to_string(&copy).unwrap_or_default(); document::eval(&format!("window.acornCopy({encoded})")); }, "Copy result" }
        }
        p { id: "output-status", role: "status", class: "muted small" }
        details { summary { "Raw result JSON" } pre { "{json}" } }
    }
}
#[component]
fn Validation(value: Value) -> Element {
    let diagnostics = items(&value, "diagnostics");
    let valid = value.get("valid").unwrap_or(&Value::Null) == true;
    rsx! {
        p { class: if valid { "status" } else { "status invalid" }, if valid { "Valid" } else { "Review needed" } " · {diagnostics.len()} diagnostics" }
        ol { class: "diagnostics", for diagnostic in diagnostics.iter().take(100) {
            li {
                strong { "{diagnostic[\"path\"].as_str().filter(|path| !path.is_empty()).unwrap_or(\"root\")}" }
                span { class: "badge", "{text(diagnostic, \"kind\")}" }
                p { "{text(diagnostic, \"message\")}" }
                if diagnostic["code"].is_string() { small { class: "muted", "Code: {text(diagnostic, \"code\")}" } }
            }
        } }
        if diagnostics.len() > 100 { p { "Showing the first 100 diagnostics in returned order. Download the result for all diagnostics." } }
    }
}
#[component]
fn Fair(value: Value) -> Element {
    rsx! {
        h3 { "Local FAIR evidence" }
        for (principle, label) in [("findable", "Findable"), ("accessible", "Accessible"), ("interoperable", "Interoperable"), ("reusable", "Reusable")] {
            {let (supported, total) = presentation::fair_counts(&value, principle); rsx! {
                div { class: "proportion", p { strong { "{label}" } span { "{supported} / {total} supported" } }
                    if total > 0 { progress { value: "{supported}", max: "{total}", "aria-label": "{label}: {supported} of {total} applicable findings supported" } }
                    else { small { "No applicable findings" } }
                }
            }}
        }
        p { class: "small muted", "Based on supplied metadata only. Unknown evidence remains in the denominator; not-applicable findings are excluded. No identifiers are resolved." }
        details { open: true, summary { "{items(&value, \"findings\").len()} findings" }
            ol { class: "diagnostics", for finding in items(&value, "findings") {
                li { strong { "{text(&finding[\"indicator\"], \"id\")}" } span { class: "badge", "{text(finding, \"state\")}" }
                    p { "{text(&finding[\"indicator\"], \"description\")}" }
                    if finding["evidence"].is_string() { p { "{text(finding, \"evidence\")}" } }
                    if text(finding, "state") != "supported" { p { class: "muted", "{text(finding, \"remediation\")}" } }
                }
            } }
        }
    }
}
#[component]
fn Readability(value: Value) -> Element {
    rsx! {
        for check in items(&value, "checks") {
            {let score = check["score"].as_f64().filter(|score| score.is_finite());
            let threshold = check["threshold"].as_f64().filter(|score| score.is_finite());
            let maximum = if text(check, "metric") == "fres" || text(check, "metric") == "lix" { 100.0 } else { 30.0 };
            rsx! {
                h3 { "{presentation::label(text(check, \"metric\"))}" }
                if let (Some(score), Some(threshold)) = (score, threshold) {
                    p { class: "score", "{score:.1}" }
                    p { class: if check["success"] == true { "status" } else { "status invalid" }, if check["success"] == true { "Within the configured threshold" } else { "Review the readability of this text" } }
                    p { "Configured threshold: {threshold}. Actual score: {score:.2}." }
                    div { class: "score-gauge", "aria-hidden": "true",
                        span { class: "score-marker", style: "left:{presentation::gauge(Some(score), maximum).unwrap_or_default()}%" }
                        span { class: "threshold-marker", style: "left:{presentation::gauge(Some(threshold), maximum).unwrap_or_default()}%" }
                    }
                    p { class: "gauge-labels small muted", span { "0" } span { "Solid: score · dashed: threshold" } span { "{maximum}" } }
                    if !(0.0..=maximum).contains(&score) { p { class: "small", "Score is outside the illustrated scale; the exact value above is retained." } }
                } else { p { class: "status invalid", "A finite score and threshold are unavailable for this text." } }
            }}
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
            pre { class: "diff-output", for line in diff.lines() { span { class: if line.starts_with('+') { "diff-added" } else if line.starts_with('-') { "diff-removed" } else { "" }, "{line}\n" } } }
        } else {
            p { "{operations.len()} JSON Patch operations" }
            for operation in operations { div { class: "result-item", strong { "{text(operation, \"op\")} · {text(operation, \"path\")}" } ResultTree { value: operation.clone(), depth: 0 } } }
        }
    }
}
#[component]
fn Discovery(value: Value) -> Element {
    let mut session = use_context::<Signal<String>>();
    let nav = use_navigator();
    rsx! {
        p { class: "status", "{value[\"count\"]} unique identifiers · normalized and deduplicated" }
        if items(&value, "identifiers").is_empty() { p { "No supported persistent identifiers found." } }
        for identifier in items(&value, "identifiers").iter().take(100) {
            {let normalized = text(identifier, "normalized").to_owned(); rsx! {
                div { class: "identifier-row", span { class: "badge", "{text(identifier, \"kind\")}" } code { "{normalized}" }
                    button { onclick: move |_| { session.set(normalized.clone()); nav.push(Route::Page { page: "inspect".into() }); }, "Inspect →" }
                }
            }}
        }
        if value.get("truncated").unwrap_or(&Value::Null) == true { p { class: "status invalid", "Results were truncated at the 100-identifier limit. Narrow your input to discover additional identifiers." } }
    }
}
#[component]
fn Inspection(value: Value) -> Element {
    rsx! {
        p { class: if text(&value, "status") == "valid" { "status" } else { "status invalid" }, "{text(&value, \"status\")} · {text(&value, \"kind\")}" }
        if text(&value, "status") == "ambiguous" { p { "Select an explicit identifier kind and run again." } ul { for candidate in items(&value, "candidates") { li { "{candidate.as_str().unwrap_or(\"Unknown kind\")}" } } } }
        if text(&value, "status") == "invalid" { p { "This value does not match the selected or recognized identifier syntax. Check the input and try again." } }
        if value.get("normalized").unwrap_or(&Value::Null).is_string() { dl { class: "result-fields", dt { "Normalized identifier" } dd { code { "{text(&value, \"normalized\")}" } } } }
        p { class: "small muted", "Local syntax inspection only. This does not establish that the identifier exists or resolves." }
    }
}
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
        div { class: "comparison-flow", code { "{text(&value, \"left\")}" } span { "↔" } code { "{text(&value, \"right\")}" } }
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
