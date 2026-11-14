use crate::results::ResultView;
use crate::syntax::{CodeBlock, CodeEditor};
use crate::ui::{Icon, Route};
use acorn_web::{
    geospatial::Marker,
    operations::Tool,
    presentation::text,
    session::{self, SuppliedFile, Workspace},
};
use dioxus::prelude::*;
use serde_json::Value;

#[derive(Clone, PartialEq)]
pub(crate) struct Detail {
    title: String,
    subtitle: String,
    source: String,
    filename: String,
    language: String,
    path: Option<String>,
    value: Option<Value>,
    checks: Option<Value>,
    tool: Tool,
    finding: Option<Value>,
    marker: Option<Marker>,
}
impl Detail {
    pub(crate) fn file(file: &SuppliedFile, workspace: &Workspace) -> Self {
        let value = serde_json::from_str::<Value>(&file.content).ok();
        let title = value
            .as_ref()
            .and_then(|value| value.get("title").and_then(Value::as_str))
            .unwrap_or(&file.path)
            .to_owned();
        Self {
            title,
            subtitle: file.path.clone(),
            source: file.content.clone(),
            filename: file.path.rsplit('/').next().unwrap_or("source.txt").into(),
            language: match file.path.rsplit('.').next() {
                | Some("json") => "json",
                | Some("yaml" | "yml") => "yaml",
                | Some("md") => "markdown",
                | _ => "auto",
            }
            .into(),
            path: Some(file.path.clone()),
            value,
            checks: workspace.validations.get(&file.path).cloned(),
            tool: Tool::Activity,
            finding: None,
            marker: None,
        }
    }
    pub(crate) fn organization(marker: &Marker) -> Self {
        Self {
            title: marker.name.clone(),
            subtitle: "Organization · local ROR registry".into(),
            source: serde_json::to_string_pretty(marker).unwrap_or_default(),
            filename: "organization.json".into(),
            language: "json".into(),
            path: None,
            value: None,
            checks: None,
            tool: Tool::Activity,
            finding: None,
            marker: Some(marker.clone()),
        }
    }
}
#[component]
fn ActivityOverview(value: Value) -> Element {
    let identifiers = ["doi", "handle", "raid", "swhid", "ror"]
        .into_iter()
        .flat_map(|field| session::many(metadata(&value, field)))
        .filter_map(Value::as_str)
        .chain(
            ["contributors", "organizations"]
                .into_iter()
                .flat_map(|field| session::many(metadata(&value, field)))
                .flat_map(|entity| session::many(entity.get("identifiers").unwrap_or(&Value::Null)))
                .filter_map(|id| id.get("value").and_then(Value::as_str)),
        )
        .collect::<alloc::collections::BTreeSet<_>>();
    rsx! {
        if let Some(subtitle) = value.get("subtitle").and_then(Value::as_str) { p { class: "details-description", "{subtitle}" } }
        dl { class: "details-facts",
            div { dt { "Type" } dd { "{session::activity_type(&value)}" } }
            div { dt { "Status" } dd { "{session::activity_status(&value)}" } }
        }
        for (field, label, icon) in [("contributors", "Contributors", "users"), ("organizations", "Organizations", "globe")] {
            section { h3 { class: "details-section-title", Icon { name: icon } "{label}" span { class: "badge", "{session::many(&value[\"meta\"][field]).len()}" } }
                ul { class: "details-entities", for entity in session::many(metadata(&value, field)) {
                    li { "{entity.as_str().or_else(|| entity[\"name\"].as_str()).unwrap_or(\"Unnamed\")}" }
                } }
                if session::many(metadata(&value, field)).is_empty() { p { class: "small muted", "Not provided" } }
            }
        }
        h3 { class: "details-section-title", Icon { name: "link" } "Identifiers" span { class: "badge", "{identifiers.len()}" } }
        ul { class: "details-entities", for identifier in identifiers {
            li { if let Some(href) = identifier_href(identifier) { a { href, target: "_blank", rel: "noopener noreferrer", "{identifier}" } } else { "{identifier}" } }
        } }
        if let Some(sections) = value.get("sections").filter(|sections| sections.is_object()) {
            for (name, content) in sections.as_object().into_iter().flatten() {
                details { summary { "{acorn_web::presentation::label(name)}" }
                    for paragraph in session::many(content) { if let Some(paragraph) = paragraph.as_str() { p { class: "details-description", "{paragraph}" } } }
                }
            }
        }
    }
}
#[component]
pub(crate) fn DetailsDrawer() -> Element {
    let mut drawer = use_context::<Signal<Option<Detail>>>();
    let mut tab = use_signal(|| "overview");
    let workspace = use_context::<Signal<Workspace>>();
    let mut input = use_context::<Signal<String>>();
    let nav = use_navigator();
    let route = use_route::<Route>();
    use_future(move || async move {
        let mut channel = document::eval("if (!window.acornDetails) await new Promise(resolve => window.addEventListener('acorn-details-ready', resolve, {once:true})); window.acornDetails.connect(() => dioxus.send(null)); await dioxus.recv();");
        while channel.recv::<Value>().await.is_ok() {
            drawer.set(None);
        }
    });
    use_effect(use_reactive((&route,), move |(_route,)| {
        if drawer.peek().is_some() {
            document::eval("window.acornDetails?.close(false)");
            drawer.set(None);
        }
    }));
    let detail = drawer();
    let visible = detail.is_some();
    use_effect(use_reactive((&detail,), move |(detail,)| {
        if detail.is_some() {
            tab.set("overview");
            document::eval("requestAnimationFrame(() => { const panel = document.getElementById('details-drawer'); if (panel?.open) { panel.querySelector('.details-body')?.scrollTo(0, 0); panel.querySelector('.details-close')?.focus({preventScroll:true}); } })");
        }
    }));
    use_effect(use_reactive((&visible,), move |(visible,)| {
        if visible {
            document::eval("if (!window.acornDetails) await new Promise(resolve => window.addEventListener('acorn-details-ready', resolve, {once:true})); requestAnimationFrame(() => window.acornDetails.open())");
        } else {
            document::eval("window.acornDetails?.close()");
        }
    }));
    let snapshot = workspace();
    rsx! {
        dialog { id: "details-drawer", class: "details-drawer", "aria-labelledby": "details-title",
            div { class: "details-resize", role: "separator", tabindex: "0", "aria-label": "Resize details drawer", "aria-orientation": "vertical", "aria-valuemin": "360", "aria-valuemax": "900", "aria-valuenow": "480", title: "Drag to resize · Left/Right arrows to adjust width" }
            if let Some(detail) = detail {
                header { class: "details-header",
                    div { p { class: "details-eyebrow", "{detail.subtitle}" } h2 { id: "details-title", tabindex: "-1", "{detail.title}" } }
                    button { class: "details-close", "aria-label": "Close details", title: "Close details (Esc)", onclick: move |_| { document::eval("window.acornDetails?.close()"); }, "×" }
                }
                div { class: "details-tabs", role: "tablist", "aria-label": "Details view",
                    for (name, label, icon) in [("overview", "Overview", "filecheck"), ("source", "Source", "code"), ("checks", "Checks", "check")] {
                        button { id: "details-tab-{name}", role: "tab", "aria-selected": tab() == name, "aria-controls": "details-panel", tabindex: if tab() == name { "0" } else { "-1" }, onclick: move |_| tab.set(name), Icon { name: icon } "{label}" }
                    }
                }
                div { id: "details-panel", class: "details-body", role: "tabpanel", "aria-labelledby": "details-tab-{tab}", tabindex: "0",
                    match tab() {
                        "source" => rsx! {
                            p { class: "small muted", "Read-only preview · {detail.source.len()} UTF-8 bytes" }
                            CodeEditor { id: "drawer-source", label: "Source preview", value: detail.source.clone(), language: detail.language.clone(), rows: 24, readonly: true, oninput: |_| {} }
                            button { class: "copy-code details-download", onclick: { let content = detail.source.clone(); let filename = detail.filename.clone(); move |_| {
                                let args = serde_json::to_string(&(content.clone(), filename.clone(), "text/plain;charset=utf-8")).unwrap_or_default();
                                document::eval(&format!("window.acornDownload(...{args})"));
                            } }, Icon { name: "download" } "Download source" }
                        },
                        "checks" => rsx! {
                            if let Some(checks) = &detail.checks {
                                ResultView { tool: detail.tool, value: checks.clone() }
                            } else if let Some(marker) = &detail.marker {
                                p { class: "small muted", "Existing checks for related activities. No new validation is run." }
                                for path in &marker.activities {
                                    details { summary { "{path}" }
                                        if let Some(checks) = snapshot.validations.get(path) { ResultView { tool: Tool::Activity, value: checks.clone() } }
                                        else { p { class: "muted", "No checks available for this activity." } }
                                    }
                                }
                            } else { div { class: "details-empty", Icon { name: "check" } h3 { "No checks yet" } p { "Open this file in Research Activity to validate it." } } }
                        },
                        _ => rsx! {
                            if let Some(finding) = &detail.finding { FindingOverview { finding: finding.clone(), value: detail.value.clone() } }
                            else if let Some(marker) = &detail.marker {
                                dl { class: "details-facts",
                                    div { dt { "ROR" } dd { a { href: marker.id.clone(), target: "_blank", rel: "noopener noreferrer", "{marker.id}" } } }
                                    div { dt { "Registry" } dd { if marker.resolved { "Found in local index" } else { "Not in local index" } } }
                                    div { dt { "References" } dd { "{marker.references}" } }
                                    if let Some(location) = &marker.location {
                                        div { dt { "Country" } dd { "{location.country}" } }
                                        div { dt { "Coordinates" } dd { "{location.latitude:.4}, {location.longitude:.4}" } }
                                    }
                                }
                                h3 { class: "details-section-title", Icon { name: "portfolio" } "Related activities" }
                                if marker.activities.is_empty() { p { class: "small muted", "No validated activities in this session reference this organization." } }
                                for path in &marker.activities {
                                    if let Some(file) = snapshot.files.iter().find(|file| &file.path == path) {
                                        {let next = Detail::file(file, &snapshot); rsx! {
                                            button { class: "details-related", onclick: move |_| open(drawer, next.clone()), strong { "{next.title}" } small { "{path}" } }
                                        }}
                                    }
                                }
                            } else if let Some(value) = &detail.value {
                                ActivityOverview { value: value.clone() }
                            } else { div { class: "details-empty", Icon { name: "code" } h3 { "File preview" } p { "Open Source to read this file. Its original content is preserved." } } }
                        },
                    }
                }
                if detail.marker.is_none() {
                    footer { class: "details-footer", button { class: "primary", onclick: move |_| {
                        input.set(detail.source.clone());
                        if let Some(path) = &detail.path { let mut workspace = workspace; let mut next = workspace(); next.selected = Some(path.clone()); workspace.set(next); }
                        document::eval("window.acornDetails?.close(false)"); drawer.set(None);
                        nav.push(Route::Page { page: "research-activity".into() });
                        document::eval("requestAnimationFrame(() => (document.getElementById('content-editor') || document.getElementById('content'))?.focus())");
                    }, Icon { name: "code" } "Open in Research Activity" } }
                }
            }
        }
    }
}
#[component]
pub(crate) fn DiagnosticButton(finding: Value, source: String, checks: Value, tool: Tool) -> Element {
    let drawer = use_context::<Signal<Option<Detail>>>();
    rsx! { button { class: "inspect-diagnostic", r#type: "button", title: "Inspect this finding and its source", onclick: move |_| {
        let path = finding.get("path").and_then(Value::as_str).or_else(|| finding.get("field").and_then(Value::as_str)).unwrap_or("No field path");
        open(drawer, Detail {
            title: "Diagnostic details".into(), subtitle: path.into(), source: source.clone(), filename: "input.txt".into(), language: "auto".into(),
            path: None, value: serde_json::from_str(&source).ok(), checks: Some(checks.clone()), tool, finding: Some(finding.clone()), marker: None,
        });
    }, Icon { name: "search" } "Inspect" } }
}
fn field_context<'a>(value: &'a Value, path: &str) -> Option<(Option<&'a Value>, &'a Value)> {
    let normalized = path.trim_start_matches('$').trim_start_matches('.').replace('[', ".").replace(']', "");
    let parts = if path.starts_with('/') {
        path.split('/')
            .skip(1)
            .map(|part| part.replace("~1", "/").replace("~0", "~"))
            .collect::<Vec<_>>()
    } else {
        normalized.split('.').filter(|part| !part.is_empty()).map(str::to_owned).collect()
    };
    if parts.is_empty() || path == "root" {
        return Some((Some(value), value));
    }
    parts.split_last().and_then(|(last, ancestors)| {
        let parent = ancestors.iter().try_fold(value, |current, part| {
            if current.is_array() {
                part.parse::<usize>().ok().and_then(|index| current.get(index))
            } else {
                current.get(part)
            }
        });
        parent.map(|parent| {
            (
                if parent.is_array() {
                    last.parse::<usize>().ok().and_then(|index| parent.get(index))
                } else {
                    parent.get(last)
                },
                parent,
            )
        })
    })
}
#[component]
fn FindingOverview(finding: Value, value: Option<Value>) -> Element {
    let path = finding
        .get("path")
        .and_then(Value::as_str)
        .or_else(|| finding.get("field").and_then(Value::as_str));
    let severity = if finding.get("indicator").is_some() {
        acorn_web::presentation::finding_severity(&finding)
    } else if finding.get("lossType").is_some() {
        acorn_analysis::CheckSeverity::Warning
    } else {
        acorn_web::presentation::diagnostic_severity(&finding)
    };
    let (label, tone) = acorn_web::presentation::severity_style(&severity);
    let located = path.and_then(|path| value.as_ref().and_then(|value| field_context(value, path)));
    rsx! {
        div { class: "diagnostic", "data-tone": tone,
            div { class: "diagnostic-heading", Icon { name: "alert" } strong { "{path.unwrap_or(\"Document\")}" } span { class: "badge diagnostic-badge", "{label}" } }
            p { "{finding[\"message\"].as_str().or_else(|| finding[\"details\"].as_str()).or_else(|| finding[\"indicator\"][\"description\"].as_str()).unwrap_or(\"Review the source and checks for this finding.\")}" }
            if finding.get("code").is_some_and(Value::is_string) { small { "Rule: {text(&finding, \"code\")}" } }
        }
        if let Some((field, parent)) = located {
            h3 { class: "details-section-title", "Affected value" }
            if let Some(field) = field { CodeBlock { text: serde_json::to_string_pretty(field).unwrap_or_default(), language: "json" } }
            else { p { class: "small muted", "This field is missing from the supplied data." } }
            h3 { class: "details-section-title", "Surrounding data" }
            CodeBlock { text: serde_json::to_string_pretty(parent).unwrap_or_default(), language: "json" }
        } else { p { class: "small muted", "No concrete JSON field could be located. The complete original input is available in Source." } }
    }
}
fn identifier_href(identifier: &str) -> Option<String> {
    if identifier.starts_with("https://") || identifier.starts_with("http://") {
        Some(identifier.into())
    } else if identifier.starts_with("10.") && identifier.contains('/') {
        Some(format!("https://doi.org/{identifier}"))
    } else {
        None
    }
}
fn metadata<'a>(value: &'a Value, field: &str) -> &'a Value {
    value.get("meta").and_then(|meta| meta.get(field)).unwrap_or(&Value::Null)
}
pub(crate) fn open(mut drawer: Signal<Option<Detail>>, detail: Detail) {
    document::eval("window.acornDetails?.rememberFocus()");
    drawer.set(Some(detail));
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_field_context_preserves_missing_and_nested_values() {
        let value = json!({"meta":{"contributors":[{"name":"Ada"}]}, "a/b":{"x.y":null}});
        assert_eq!(field_context(&value, "$.meta.contributors[0].name").unwrap().0, Some(&json!("Ada")));
        assert_eq!(field_context(&value, "meta.contributors[0].email").unwrap().0, None);
        assert_eq!(field_context(&value, "/a~1b/x.y").unwrap().0, Some(&Value::Null));
        assert!(field_context(&value, "meta.unknown[0].name").is_none());
    }
    #[test]
    fn test_identifier_links_reject_active_content() {
        assert_eq!(identifier_href("javascript:alert(1)"), None);
        assert_eq!(identifier_href("10.1234/example"), Some("https://doi.org/10.1234/example".into()));
    }
}
