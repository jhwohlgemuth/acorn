use acorn_web::operations::{self, Request, Tool};
use dioxus::prelude::*;
use serde_json::Value;
use strum::IntoEnumIterator;

const SAMPLE: &str = include_str!("../assets/examples/publications.json");
const RAD_EXAMPLES: [(&str, &str); 3] = [
    ("Research summary", include_str!("../assets/examples/summary.json")),
    ("Publications and affiliations", SAMPLE),
    ("Compute resources and software", include_str!("../assets/examples/computing.json")),
];
const VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Clone, Debug, PartialEq, Routable)]
pub(crate) enum Route {
    #[layout(Shell)]
    #[route("/")]
    Home {},
    #[route("/:page")]
    Page { page: String },
}
#[component]
fn Shell() -> Element {
    let mut collapsed = use_signal(|| false);
    let mut drawer = use_signal(|| false);
    let mut dark = use_signal(|| true);
    let route = use_route::<Route>();
    let current = match route {
        | Route::Home {} => "home".to_owned(),
        | Route::Page { page } => page,
    };
    rsx! {
        div { class: format!("workspace {} {}", if collapsed() { "compact" } else { "" }, if drawer() { "drawer-open" } else { "" }), onkeydown: move |event| { if event.key() == Key::Escape { drawer.set(false); document::eval("document.getElementById('nav-toggle')?.focus()"); } }, "data-theme": if dark() { "dark" } else { "light" },
            a { class: "skip", href: "#content-main", "Skip to content" }
            header { class: "mobile-bar", span { "ACORN" } button { id: "nav-toggle", onclick: move |_| drawer.toggle(), "aria-expanded": drawer(), "aria-controls": "workspace-navigation", "aria-label": "Toggle navigation", "☰" } }
            aside { id: "workspace-navigation", class: "sidebar", "aria-label": "Workspace navigation",
                div { class: "brand", img { src: asset!("/assets/generated/acorn.svg"), alt: "", width: "29" } span { "ACORN" } button { class: "collapse", onclick: move |_| collapsed.toggle(), title: "Toggle sidebar", "aria-label": "Toggle sidebar", "☰" } }
                nav { onclick: move |_| drawer.set(false),
                    Link { to: Route::Home {}, class: if current == "home" { "nav-item active" } else { "nav-item" }, title: "Home", span { class: "nav-icon", "⌂" } span { class: "nav-label", "Home" } }
                    p { class: "nav-heading", "WORKSPACE" }
                    for (slug, name, icon) in [("portfolio", "Portfolio", "▤"), ("geospatial", "Geospatial", "◎")] {
                        Link { to: Route::Page { page: slug.into() }, class: if current == slug { "nav-item active" } else { "nav-item" }, title: name, span { class: "nav-icon", "{icon}" } span { class: "nav-label", "{name}" } }
                    }
                    p { class: "nav-heading", "RESEARCH TOOLS" }
                    for tool in Tool::iter() {
                        Link { to: Route::Page { page: tool.slug().into() }, class: if current == tool.slug() { "nav-item active" } else { "nav-item" }, title: tool.to_string(), span { class: "nav-icon", "◇" } span { class: "nav-label", "{tool.to_string()}" } }
                    }
                    p { class: "nav-heading", "RESOURCES" }
                    for (slug, name, icon) in [("install", "Install", "↓"), ("skill", "ACORN Skill", "✧"), ("settings", "Settings", "⚙"), ("about", "About", "ⓘ")] {
                        Link { to: Route::Page { page: slug.into() }, class: if current == slug { "nav-item active" } else { "nav-item" }, title: name, span { class: "nav-icon", "{icon}" } span { class: "nav-label", "{name}" } }
                    }
                }
                div { class: "sidebar-bottom",
                    button { class: "theme-toggle", "aria-pressed": dark(), onclick: move |_| dark.toggle(), span { class: "switch-track", span { class: "switch-thumb" } } span { class: "nav-label", if dark() { "Dark" } else { "Light" } } }
                    small { "v{VERSION} · Browser" }
                    small { id: "webmcp-status", "data-adapter": asset!("/assets/generated/webmcp.js"), "WebMCP: initializing" }
                }
            }
            main { id: "content-main", tabindex: "-1", Outlet::<Route> {} }
        }
    }
}
#[component]
fn Home() -> Element {
    let mut session = use_context::<Signal<String>>();
    let nav = use_navigator();
    let mut example = use_signal(|| 1_usize);
    rsx! {
        document::Title { "ACORN · Research workspace" }
        section { class: "home",
            img { class: "home-mark", src: asset!("/assets/generated/acorn.svg"), alt: "ACORN", width: "72" }
            h1 { "Your research, connected." }
            p { class: "lede", "Explore, validate and improve your research metadata. All in your browser." }
            label { class: "field", span { "Example research activity" }
                select { value: "{example()}", onchange: move |event| { if let Ok(index) = event.value().parse::<usize>() { if index < RAD_EXAMPLES.len() { example.set(index); } } },
                    for (index, (label, _)) in RAD_EXAMPLES.iter().enumerate() { option { value: "{index}", "{label}" } }
                }
            }
            div { class: "actions centered",
                button { onclick: move |_| { document::eval("document.getElementById('quick-start').showModal()"); }, "Take a tour" }
                button { class: "primary", onclick: move |_| { if let Some((_, content)) = RAD_EXAMPLES.get(example()) { session.set((*content).into()); nav.push(Route::Page { page: "research-activity".into() }); } }, "Try sample data" }
            }
            p { class: "muted", "Examples use illustrative data. " a { href: asset!("/assets/examples/.acorn.json"), download: ".acorn.json", "Download ACRE configuration" } " for the CLI; configuration validation is not available in this browser." }
            Link { to: Route::Page { page: "skill".into() }, class: "skill-card", span { "✧  ACORN Skill" } p { "Bring ACORN workflows to your coding agent." } span { class: "muted", "View the skill guide →" } }
            div { class: "card open-card", h2 { "Start with your data" } p { "Open a local text or JSON file, or paste content into any tool." } FileInput { onload: move |text| { session.set(text); nav.push(Route::Page { page: "research-activity".into() }); }, limit: 262144 } }
            p { class: "privacy", "◈ Local by design. Your content stays in this tab. No account, upload, or automatic tool execution." }
            Link { to: Route::Page { page: "install".into() }, "Use ACORN on your desktop →" }
            Footer {}
        }
        dialog { id: "quick-start", h2 { "A quick look around" } ol { li { "Load sample data or open a local file." } li { "Choose one of nine research tools in the sidebar." } li { "Select options, then run the operation explicitly." } li { "Review structured results and download them if needed." } } button { class: "primary", onclick: move |_| { document::eval("document.getElementById('quick-start').close()"); }, "Got it" } }
    }
}
#[component]
fn Page(page: String) -> Element {
    use_effect(move || {
        document::eval("document.getElementById('content-main')?.focus(); window.scrollTo(0,0)");
    });
    if let Some(tool) = Tool::iter().find(|tool| tool.slug() == page) {
        rsx! { ToolWorkspace { key: "{page}", tool } }
    } else {
        rsx! { InfoPage { page } }
    }
}
#[component]
fn FileInput(onload: EventHandler<String>, limit: usize) -> Element {
    let mut error = use_signal(String::new);
    rsx! {
        label { class: "file-picker", "Open a local file"
            input { r#type: "file", onchange: move |event: FormEvent| async move {
                if let Some(file) = event.files().first() {
                    if file.size() > limit as u64 { error.set(format!("File exceeds the {limit}-byte limit. Current content is unchanged.")); }
                    else { match file.read_bytes().await {
                        Ok(bytes) => match String::from_utf8(bytes.to_vec()) { Ok(text) if text.len() <= limit => { error.set(String::new()); onload.call(text); }, _ => error.set("Choose a UTF-8 text file within the limit.".into()) },
                        Err(_) => error.set("Could not read this file. Current content is unchanged.".into()),
                    }}
                }
            } }
        }
        if !error().is_empty() { p { role: "alert", class: "error", "{error}" } }
    }
}
#[component]
fn SelectField(label: &'static str, value: String, options: Vec<&'static str>, onchange: EventHandler<String>) -> Element {
    rsx! { label { class: "field", span { "{label}" } select { value, onchange: move |event| onchange.call(event.value()), for option in options { option { value: option, if option.is_empty() { "Auto-detect" } else { "{option}" } } } } } }
}
#[component]
fn ToolWorkspace(tool: Tool) -> Element {
    let mut session = use_context::<Signal<String>>();
    let mut request = use_signal(|| Request {
        input: session(),
        ..Request::default()
    });
    let mut result = use_signal(|| None::<Result<Value, String>>);
    use_effect(use_reactive((&tool,), move |(_tool,)| {
        request.set(Request {
            input: session.peek().clone(),
            ..Request::default()
        });
        result.set(None);
    }));
    use_effect(move || {
        let _ = request();
        result.set(None);
    });
    let title = tool.to_string();
    let limit = tool.limit();
    let input = request().input;
    rsx! {
        document::Title { "{title} · ACORN" }
        section { class: "page tool-page",
            div { class: "eyebrow", "RESEARCH TOOLS / LOCAL COMPUTATION" }
            h1 { "{title}" } p { class: "lede", "{tool.description()}" }
            div { class: "tool-grid",
                section { class: "card editor", h2 { "Input" }
                    div { class: "actions", FileInput { limit, onload: move |text: String| { request.set(Request { input: text, ..request() }); result.set(None); } }
                        button { onclick: move |_| { let (input, after) = sample(tool); request.set(Request { input, after, ..Request::default() }); result.set(None); }, "Use example" }
                    }
                    div { class: "options",
                        if matches!(tool, Tool::Activity) { SelectField { label: "Vocabulary matching", value: request().fuzzy, options: vec!["off", "relaxed", "strict"], onchange: move |v| request.set(Request { fuzzy: v, ..request() }) } }
                        if matches!(tool, Tool::Standard | Tool::Crosswalk | Tool::Fair) { SelectField { label: "Standard / source", value: request().standard, options: if tool == Tool::Crosswalk { vec!["research-activity-data", "raid", "datacite", "dublin-core"] } else { vec!["research-activity-data", "raid", "datacite", "dublin-core", "citation-file-format", "ckan", "document", "invenio", "kitfile", "modelkit", "text"] }, onchange: move |v| request.set(Request { standard: v, ..request() }) } }
                        if matches!(tool, Tool::Standard | Tool::Crosswalk) { SelectField { label: "Format", value: request().format, options: vec!["json", "jsonc", "json-ld", "yaml", "zon", "cff", "markdown", "rdf-xml", "turtle", "text"], onchange: move |v| request.set(Request { format: v, ..request() }) } }
                        if tool == Tool::Crosswalk { SelectField { label: "Destination", value: request().target, options: vec!["datacite", "dublin-core", "raid", "research-activity-data"], onchange: move |v| request.set(Request { target: v, ..request() }) } }
                        if tool == Tool::Readability { SelectField { label: "Metric", value: request().metric, options: vec!["fres", "ari", "cli", "fkgl", "gfi", "lix", "smog"], onchange: move |v| request.set(Request { metric: v, ..request() }) } }
                        if tool == Tool::Diff { SelectField { label: "Diff format", value: request().mode, options: vec!["unified", "json-patch"], onchange: move |v| request.set(Request { mode: v, ..request() }) } }
                        if tool == Tool::Inspect { SelectField { label: "Identifier kind", value: request().kind, options: vec!["", "ark", "arxiv", "doi", "handle", "isbn", "isni", "orcid", "patent", "raid", "ror", "rrid", "swhid"], onchange: move |v| request.set(Request { kind: v, ..request() }) } }
                    }
                    label { class: "field", span { if tool == Tool::BigO { "Left expression" } else if tool == Tool::Diff { "Before" } else { "Content" } }
                        textarea { id: "content", spellcheck: "false", rows: if tool == Tool::BigO { "3" } else { "14" }, value: input.clone(), oninput: move |e| { request.set(Request { input: e.value(), ..request() }); result.set(None); } }
                    }
                    small { class: if input.len() > limit { "error" } else { "muted" }, "{input.len()} / {limit} UTF-8 bytes" }
                    if matches!(tool, Tool::Diff | Tool::BigO) {
                        label { class: "field", span { if tool == Tool::BigO { "Right expression" } else { "After" } } textarea { id: "after", rows: "4", spellcheck: "false", value: request().after, oninput: move |e| { request.set(Request { after: e.value(), ..request() }); result.set(None); } } }
                    }
                    if tool == Tool::BigO { label { class: "field", "Symbol assumptions (JSON)" textarea { rows: "2", value: request().assumptions, oninput: move |e| request.set(Request { assumptions: e.value(), ..request() }) } } }
                    if tool == Tool::Crosswalk { details { summary { "Import seeds (when required)" } label { class: "field", "Contact JSON" textarea { rows: "3", value: request().contact, oninput: move |e| request.set(Request { contact: e.value(), ..request() }) } } label { class: "field", "Local identifier" input { value: request().local_identifier, oninput: move |e| request.set(Request { local_identifier: e.value(), ..request() }) } } } }
                    div { class: "actions run-actions", button { class: "primary", onclick: move |_| { session.set(request().input); result.set(Some(operations::run(tool, &request()))); }, "Run {title}" } button { onclick: move |_| { request.set(Request::default()); result.set(None); }, "Clear" } }
                    p { class: "muted small", "Runs locally. No identifiers are resolved and no content is uploaded." }
                }
                section { class: "card results", "aria-live": "polite", h2 { "Results" }
                    match result() {
                        None => rsx! { div { class: "empty", span { "◇" } h3 { "Ready when you are" } p { "Add your content, choose options, and run the tool." } } },
                        Some(Err(message)) => rsx! { div { class: "error", role: "alert", h3 { "Could not complete this operation" } p { "{message}" } } },
                        Some(Ok(value)) => rsx! { ResultView { value } },
                    }
                }
            }
            Footer {}
        }
    }
}
#[component]
fn ResultView(value: Value) -> Element {
    let json = serde_json::to_string_pretty(&value).unwrap_or_default();
    let download = json.clone();
    let copy = json.clone();
    rsx! {
        p { class: if value.get("valid") == Some(&Value::Bool(false)) { "status invalid" } else { "status" },
            if value.get("valid") == Some(&Value::Bool(false)) { "Review needed · validation diagnostics below" } else { "Operation complete" }
        }
        ResultTree { value: value.clone(), depth: 0 }
        div { class: "actions", button { onclick: move |_| { let encoded = serde_json::to_string(&download).unwrap_or_default(); document::eval(&format!("window.acornDownload({encoded})")); }, "Download JSON" } button { onclick: move |_| { let encoded = serde_json::to_string(&copy).unwrap_or_default(); document::eval(&format!("window.acornCopy({encoded})")); }, "Copy JSON" } }
        p { id: "output-status", role: "status", class: "muted small" }
        details { summary { "Raw JSON" } pre { "{json}" } }
    }
}
#[component]
fn ResultTree(value: Value, depth: usize) -> Element {
    match value {
        | Value::Object(fields) => {
            rsx! { dl { class: "result-fields", for (name, value) in fields { div { class: "result-field", dt { "{name}" } dd { ResultTree { value, depth: depth.saturating_add(1) } } } } } }
        }
        | Value::Array(items) => {
            let count = items.len();
            rsx! { div { if count == 0 { span { class: "muted", "None" } } for (index, item) in items.into_iter().take(100).enumerate() { div { class: "result-item", key: "{index}", ResultTree { value: item, depth: depth.saturating_add(1) } } } if count > 100 { p { "Showing 100 of {count} entries. Download JSON for the full result." } } } }
        }
        | Value::String(text) => rsx! { span { class: "result-text", "{text}" } },
        | Value::Null => rsx! { span { class: "muted", "Not provided" } },
        | other => rsx! { span { "{other}" } },
    }
}
#[component]
fn InfoPage(page: String) -> Element {
    let session = use_context::<Signal<String>>();
    let title = match page.as_str() {
        | "portfolio" => "Portfolio",
        | "geospatial" => "Geospatial",
        | "install" => "Install ACORN",
        | "skill" => "ACORN Skill",
        | "settings" => "Settings",
        | "about" => "About ACORN",
        | _ => "Page not found",
    };
    rsx! {
        document::Title { "{title} · ACORN" }
        section { class: "page info-page", h1 { "{title}" }
            match page.as_str() {
                "portfolio" => rsx! { p { class: "lede", "Research Activity Data in this session." } div { class: "card", if session().is_empty() { h2 { "No content loaded" } p { "Open a file or try the sample from Home to get started." } } else { h2 { "Current document" } p { "{session().len()} UTF-8 bytes loaded in this tab." } p { "Validate the document to review its Research Activity fields." } Link { to: Route::Page { page: "research-activity".into() }, "Review Research Activity →" } } } p { class: "muted", "The full multi-activity Portfolio is planned in ABW-19." } },
                "geospatial" => rsx! { p { class: "lede", "Research organizations, in context." } div { class: "card", h2 { "Map workspace is planned" } p { "The offline organization map and equivalent location table arrive in ABW-18. No map or registry requests are made by this browser." } } },
                "install" => rsx! { p { class: "lede", "Run the browser workspace from your checkout." } div { class: "card", h2 { "Browser development" } pre { "make web-dev\nmake web-build\nmake web-preview\nmake web-check\nmake web-test" } } div { class: "card", h2 { "Desktop packages · pending" } p { "Windows, macOS and Linux GUI packages are planned. WinGet, Scoop, Debian, AppImage and Homebrew cask publication is not yet available." } } },
                "skill" => rsx! { p { class: "lede", "ACORN workflows for your coding agent." } div { class: "card", h2 { "Canonical repository skill" } p { "The skill explains ACORN workflows to an agent. It does not run inside this application or grant access to this tab." } a { href: "https://code.ornl.gov/research-enablement/acorn/-/tree/main/crates/acorn-cli/assets/skills/acorn", target: "_blank", rel: "noopener noreferrer", "View canonical skill source ↗" } p { class: "muted", "Release-matched offline skill downloads are planned in ABW-21/22. Exported agent copies remain user-managed." } } },
                "settings" => rsx! { p { class: "lede", "A local workspace, with no account or persistent storage." } div { class: "card", h2 { "Appearance" } p { "Use the theme switch in the sidebar to choose dark or light for this session." } h2 { "Session" } p { "Content is held in this tab only. Reloading clears it." } h2 { "Release" } p { "ACORN {VERSION} · Browser" } } },
                "about" => rsx! { p { class: "lede", "Accessible Content Optimization for Research Needs" } a { class: "card source-card", href: "https://code.ornl.gov/research-enablement/acorn", target: "_blank", rel: "noopener noreferrer", h2 { "Built in the open ↗" } p { "Explore the source, report an issue, or contribute." } } h2 { "Research tools, close to your data" } p { "ACORN helps maintain and validate research activity data. This browser workspace runs the existing portable ACORN operations on content you supply." } h2 { "Local by design" } p { "Rust and WebAssembly power the tools. Dioxus renders the interface. A shared Tauri desktop delivery is planned." } div { class: "card", h2 { "AI-assisted development" } p { "The maintainer directs, reviews and tests this project, with assistance from Codex, GitHub Copilot and Microsoft 365 Copilot. Review generated results and code before relying on them." } } p { class: "muted", "The green accent is provisional and ORNL-inspired; no institutional endorsement is implied." } },
                _ => rsx! { p { "This route is not available." } Link { to: Route::Home {}, "Return Home" } },
            }
            if page != "about" { Footer {} }
        }
    }
}
#[component]
fn Footer() -> Element {
    rsx! { footer { "Made with ❤️ by " a { href: "https://github.com/jhwohlgemuth", target: "_blank", rel: "noopener noreferrer", strong { "jhwohlgemuth" } } } }
}
fn sample(tool: Tool) -> (String, String) {
    match tool {
        Tool::BigO => ("O(n log n)".into(), "O(n^2)".into()),
        Tool::Diff => ("Research data before review.\n".into(), "Research data after review.\n".into()),
        Tool::Discover => ("Read DOI 10.1000/182 and ORCID 0000-0002-1825-0097.".into(), String::new()),
        Tool::Inspect => ("10.1000/182".into(), String::new()),
        Tool::Readability => ("Research data should be easy to find and use. Clear descriptions help people understand the work. Share your methods so others can build on your results.".into(), String::new()),
        Tool::Activity | Tool::Crosswalk | Tool::Fair | Tool::Standard => (SAMPLE.into(), String::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::Mutation;
    use serde_json::json;
    #[test]
    fn test_result_tree_preserves_nested_patch_values() {
        let value = json!({"patch": [
            {"op": "add", "path": "/title", "value": "Added title"},
            {"op": "replace", "path": "/x", "value": 2}
        ]});
        let mut dom = VirtualDom::new_with_props(ResultTree, ResultTreeProps { value, depth: 0 });
        let mutations = dom.rebuild_to_vec();
        let text = mutations
            .edits
            .iter()
            .filter_map(|edit| match edit {
                | Mutation::CreateTextNode { value, .. } | Mutation::SetText { value, .. } => Some(value.as_str()),
                | _ => None,
            })
            .collect::<Vec<_>>();
        assert!(text.contains(&"Added title"), "add value must be visible: {text:?}");
        assert!(text.contains(&"2"), "replacement value must be visible: {text:?}");
        assert!(text.contains(&"/title"));
        assert!(text.contains(&"/x"));
    }
}
