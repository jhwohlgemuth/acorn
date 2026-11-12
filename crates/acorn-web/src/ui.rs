use acorn_web::operations::{self, Request, Tool};
use dioxus::html::{FileData, HasFileData};
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
    let route = use_route::<Route>();
    let current = match route {
        | Route::Home {} => "home".to_owned(),
        | Route::Page { page } => page,
    };
    use_effect(use_reactive!(|current| {
        let _ = current;
        document::eval("window.acornChrome?.tour.cancel(); window.acornChrome?.theme.sync(); requestAnimationFrame(() => { const heading = document.querySelector('#content-main h1'); if (heading) { heading.tabIndex = -1; heading.focus(); } window.scrollTo(0, 0); })");
    }));
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/vendor/driver.css") }
        div { class: format!("workspace {} {}", if collapsed() { "compact" } else { "" }, if drawer() { "drawer-open" } else { "" }), onkeydown: move |event| { if event.key() == Key::Escape && drawer() { drawer.set(false); document::eval("document.getElementById('nav-toggle')?.focus()"); } }, "data-theme": "dark",
            a { class: "skip", href: "#content-main", "Skip to content" }
            header { class: "mobile-bar", span { "ACORN" } button { id: "nav-toggle", onclick: move |_| { drawer.toggle(); if drawer() { document::eval("requestAnimationFrame(() => document.querySelector('#workspace-navigation a')?.focus())"); } }, "aria-expanded": drawer(), "aria-controls": "workspace-navigation", "aria-label": "Toggle navigation", Icon { name: "panel" } } }
            aside { id: "workspace-navigation", class: "sidebar", "aria-label": "Workspace navigation",
                div { class: "brand", img { src: asset!("/assets/generated/acorn.svg"), alt: "", width: "29" } span { "ACORN" } button { class: "collapse", onclick: move |_| collapsed.toggle(), title: "Toggle sidebar", "aria-label": "Toggle sidebar", "aria-expanded": !collapsed(), "aria-controls": "workspace-navigation", Icon { name: "panel" } } }
                nav { "aria-label": "Primary", onclick: move |_| { if drawer() { drawer.set(false); document::eval("requestAnimationFrame(() => document.querySelector('#content-main h1')?.focus())"); } },
                    p { class: "nav-heading", "WORKSPACE" }
                    Link { to: Route::Home {}, class: if current == "home" { "nav-item active" } else { "nav-item" }, title: "Home", "aria-label": "Home", "aria-current": if current == "home" { "page" } else { "false" }, Icon { name: "home" } span { class: "nav-label", "Home" } }
                    Link { to: Route::Page { page: "portfolio".into() }, class: if current == "portfolio" { "nav-item active" } else { "nav-item" }, title: "Portfolio", "aria-label": "Portfolio", "aria-current": if current == "portfolio" { "page" } else { "false" }, Icon { name: "portfolio" } span { class: "nav-label", "Portfolio" } }
                    for (heading, tools) in [
                        ("VALIDATE", &[Tool::Activity, Tool::Standard, Tool::Crosswalk][..]),
                        ("ANALYZE", &[Tool::Fair, Tool::Readability, Tool::Diff, Tool::BigO][..]),
                        ("IDENTIFIERS", &[Tool::Discover, Tool::Inspect][..]),
                    ] {
                        p { class: "nav-heading", "{heading}" }
                        for tool in tools.iter().copied() {
                            Link { to: Route::Page { page: tool.slug().into() }, class: if current == tool.slug() { "nav-item active" } else { "nav-item" }, title: tool.to_string(), "aria-label": tool.to_string(), "aria-current": if current == tool.slug() { "page" } else { "false" }, Icon { name: tool.icon() } span { class: "nav-label", "{tool.to_string()}" } }
                        }
                        if heading == "ANALYZE" {
                            Link { to: Route::Page { page: "geospatial".into() }, class: if current == "geospatial" { "nav-item active" } else { "nav-item" }, title: "Geospatial", "aria-label": "Geospatial", "aria-current": if current == "geospatial" { "page" } else { "false" }, Icon { name: "globe" } span { class: "nav-label", "Geospatial" } }
                        }
                    }
                    p { class: "nav-heading", "APPLICATION" }
                    for (slug, name, icon) in [("install", "Install", "download"), ("skill", "ACORN Skill", "book"), ("settings", "Settings", "settings"), ("about", "About", "info")] {
                        Link { to: Route::Page { page: slug.into() }, class: if current == slug { "nav-item active" } else { "nav-item" }, title: name, "aria-label": name, "aria-current": if current == slug { "page" } else { "false" }, Icon { name: icon } span { class: "nav-label", "{name}" } }
                    }
                }
                div { class: "sidebar-bottom",
                    button { id: "theme-toggle", class: "theme-toggle", "aria-label": "Dark theme", title: "Toggle dark or light theme", "aria-pressed": "true", onclick: move |_| { document::eval("window.acornChrome?.theme.toggle()"); }, span { class: "switch-track", span { class: "switch-thumb" } } span { id: "theme-label", class: "nav-label", "Dark" } }
                    small { "v{VERSION} · Browser" }
                    small { id: "webmcp-status", "data-adapter": asset!("/assets/generated/webmcp.js"), "WebMCP: initializing" }
                }
            }
            div { id: "notification", class: "notification", hidden: true, span { id: "notification-text", role: "status", "aria-live": "polite" } button { "aria-label": "Dismiss notification", onclick: move |_| { document::eval("window.acornChrome?.notifications.dismiss()"); }, "Dismiss" } }
            main { id: "content-main", inert: drawer().then_some(""), tabindex: "-1", Outlet::<Route> {} }
        }
    }
}
#[component]
fn Home() -> Element {
    let mut session = use_context::<Signal<String>>();
    let nav = use_navigator();
    let mut example = use_signal(|| 1_usize);
    let mut drop_error = use_signal(String::new);
    let mut dragging = use_signal(|| false);
    rsx! {
        document::Title { "ACORN · Research workspace" }
        section { class: "home",
            div { class: "home-banner", img { class: "banner-dark", src: asset!("/assets/generated/banner_dark_mode.png"), alt: "ACORN", width: "250" } img { class: "banner-light", src: asset!("/assets/generated/banner_light_mode.png"), alt: "ACORN", width: "250" } }
            h1 { "ACORN Browser" }
            div { id: "home-actions", class: "actions centered",
                button { id: "tour-start", class: "primary", "data-driver": asset!("/assets/vendor/driver.js"), onclick: move |_| { document::eval("window.acornChrome?.tour.start()"); }, "Take a tour" }
                button { id: "sample-data", onclick: move |_| { if let Some((_, content)) = RAD_EXAMPLES.get(example()) { session.set((*content).into()); nav.push(Route::Page { page: "research-activity".into() }); document::eval("window.acornChrome?.notifications.notify('Sample loaded. Run validation when ready.')"); } }, "Try sample data" }
            }
            label { class: "field sample-select", span { "Sample data" }
                select { value: "{example()}", onchange: move |event| { if let Ok(index) = event.value().parse::<usize>() { if index < RAD_EXAMPLES.len() { example.set(index); } } },
                    for (index, (label, _)) in RAD_EXAMPLES.iter().enumerate() { option { value: "{index}", selected: index == example(), "{label}" } }
                }
            }
            Link { id: "skill-entry", to: Route::Page { page: "skill".into() }, class: "skill-card", span { "The home of the ACORN Skill" } p { "Validate metadata, convert formats and inspect identifiers, then use ACORN in your agent environment." } span { class: "muted", "Explore the skill guide →" } }
            div { id: "local-entry", class: if dragging() { "card open-card dragging" } else { "card open-card" },
                ondragover: move |event| { event.prevent_default(); dragging.set(true); },
                ondragleave: move |_| dragging.set(false),
                ondrop: move |event: DragEvent| async move {
                    event.prevent_default(); dragging.set(false);
                    let files = event.files();
                    if files.len() != 1 { drop_error.set("Drop one UTF-8 file at a time. Folder loading is not available yet.".into()); }
                    else if let Some(file) = files.into_iter().next() {
                        match read_local_file(file, 262144).await {
                            Ok(text) => { drop_error.set(String::new()); session.set(text); nav.push(Route::Page { page: "research-activity".into() }); document::eval("window.acornChrome?.notifications.notify('File loaded. Run validation when ready.')"); },
                            Err(message) => drop_error.set(message),
                        }
                    }
                },
                h2 { "Open research activity data" } p { "Drop one UTF-8 file here, or choose a local file." }
                FileInput { onload: move |text| { session.set(text); nav.push(Route::Page { page: "research-activity".into() }); document::eval("window.acornChrome?.notifications.notify('File loaded. Run validation when ready.')"); }, limit: 262144 }
                if !drop_error().is_empty() { p { role: "alert", class: "error", "{drop_error}" } }
                p { class: "small muted", "Up to 256 KiB. Folder loading is not available yet." }
            }
            p { id: "privacy-note", class: "privacy", "Local by default. No uploads or telemetry. Your content stays in this tab; tools run only when you ask." }
            Link { to: Route::Page { page: "install".into() }, "Prefer a desktop app? View install options →" }
            Footer {}
        }
    }
}
#[component]
fn Page(page: String) -> Element {
    if let Some(tool) = Tool::iter().find(|tool| tool.slug() == page) {
        rsx! { ToolWorkspace { key: "{page}", tool } }
    } else {
        rsx! { InfoPage { page } }
    }
}
async fn read_local_file(file: FileData, limit: usize) -> Result<String, String> {
    if file.size() > limit as u64 {
        Err(format!("File exceeds the {limit}-byte limit. Current content is unchanged."))
    } else {
        file.read_bytes()
            .await
            .map_err(|_| "Could not read this file. Current content is unchanged.".into())
            .and_then(|bytes| {
                String::from_utf8(bytes.to_vec())
                    .map_err(|_| "Choose a UTF-8 text file. Current content is unchanged.".into())
                    .and_then(|text| {
                        if text.len() <= limit {
                            Ok(text)
                        } else {
                            Err(format!("File exceeds the {limit}-byte limit. Current content is unchanged."))
                        }
                    })
            })
    }
}

#[component]
fn FileInput(onload: EventHandler<String>, limit: usize) -> Element {
    let mut error = use_signal(String::new);
    rsx! {
        label { class: "file-picker", "Open a local file"
            input { r#type: "file", onchange: move |event: FormEvent| async move {
                if let Some(file) = event.files().into_iter().next() {
                    match read_local_file(file, limit).await {
                        Ok(text) => { error.set(String::new()); onload.call(text); },
                        Err(message) => error.set(message),
                    }
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
        section { class: "page info-page", "data-page": "{page}", h1 { "{title}" }
            match page.as_str() {
                "portfolio" => rsx! { p { class: "lede", "Research Activity Data in this session." } div { class: "card", if session().is_empty() { h2 { "No content loaded" } p { "Open a file or try the sample from Home to get started." } } else { h2 { "Current document" } p { "{session().len()} UTF-8 bytes loaded in this tab." } p { "Validate the document to review its Research Activity fields." } Link { to: Route::Page { page: "research-activity".into() }, "Review Research Activity →" } } } p { class: "muted", "Multi-activity portfolios are not available yet." } },
                "geospatial" => rsx! { p { class: "lede", "Research organizations, in context." } div { class: "card", h2 { "Map workspace is planned" } p { "The offline organization map and equivalent location table are not available yet. No map or registry requests are made by this browser." } } },
                "install" => rsx! { p { class: "lede", "Run the browser workspace from your checkout." } div { class: "card", h2 { "Browser development" } pre { "make web-dev\nmake web-build\nmake web-preview\nmake web-check\nmake web-test" } } div { class: "card", h2 { "Desktop packages · pending" } p { "Windows, macOS and Linux GUI packages are planned. WinGet, Scoop, Debian, AppImage and Homebrew cask publication is not yet available." } } },
                "skill" => rsx! { p { class: "lede", "ACORN workflows for your coding agent." } div { class: "card", h2 { "Canonical repository skill" } p { "The skill explains ACORN workflows to an agent. It does not run inside this application or grant access to this tab." } a { href: "https://code.ornl.gov/research-enablement/acorn/-/tree/main/crates/acorn-cli/assets/skills/acorn", target: "_blank", rel: "noopener noreferrer", "View canonical skill source ↗" } p { class: "muted", "Release-matched offline skill downloads are not yet exposed in this interface. Exported agent copies remain user-managed." } } },
                "settings" => rsx! { p { class: "lede", "A local workspace, with no account or persistent storage." } div { class: "card", h2 { "Appearance" } p { id: "theme-mode", "Following your system preference." } p { "Use the theme switch to choose dark or light for this session." } button { onclick: move |_| { document::eval("window.acornChrome?.theme.reset()"); }, "Use system theme" } h2 { "Session" } p { "Content is held in this tab only. Reloading clears it." } h2 { "Release" } p { "ACORN {VERSION} · Browser" } } },
                "about" => rsx! { p { class: "lede", "Accessible Content Optimization for Research Needs" } a { class: "card source-card", href: "https://code.ornl.gov/research-enablement/acorn", target: "_blank", rel: "noopener noreferrer", h2 { "Built in the open ↗" } p { "Explore the source, report an issue, or contribute." } } h2 { "ACORN Skill" } p { "The app provides local research tools; the matching skill explains their workflows in your agent environment." } div { class: "actions", Link { to: Route::Page { page: "skill".into() }, "Explore the ACORN Skill →" } Link { to: Route::Page { page: "install".into() }, "Install options →" } } h2 { "Research tools, close to your data" } p { "ACORN helps maintain and validate research activity data. This browser workspace runs the existing portable ACORN operations on content you supply." } h2 { "Local by design" } p { "Rust and WebAssembly power the tools. Dioxus renders the interface. A shared Tauri desktop delivery is planned." } Technology {} div { class: "card ai-disclosure", a { href: "https://www.aihonestybadge.com/ai-assisted-badge", target: "_blank", rel: "noopener noreferrer", img { src: asset!("/assets/ai-assisted.svg"), alt: "AI Assisted", width: "113", height: "20" } } h2 { "AI-use disclosure" } p { "AI tools assisted with code, documentation and design under human direction. The maintainer reviews and tests the work. Review generated code before use." } div { class: "agent-links",
                    for (id, name, url) in [("codex", "Codex", "https://openai.com/codex/"), ("copilot", "GitHub Copilot", "https://github.com/features/copilot"), ("microsoft", "Microsoft 365 Copilot", "https://www.microsoft.com/microsoft-365/copilot")] {
                        a { href: url, target: "_blank", rel: "noopener noreferrer", img { src: format!("{}#{id}", asset!("/assets/agent-marks.svg")), alt: "", width: "20", height: "20" } "{name}" }
                    }
                } } p { class: "muted", "The green accent is provisional and ORNL-inspired; no institutional endorsement is implied." } },
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

#[component]
fn Icon(name: &'static str) -> Element {
    let artwork = match name {
        | "book" => {
            r#"<path class="s" d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20V4H6.5A2.5 2.5 0 0 0 4 6.5Z" />
      <path class="s" d="M4 6.5v13M9 10l2 2 4-4" />"#
        }
        | "chart" => r#"<path d="M4 3v17h17M7 16l4-3 4-5 5-4" fill="none" stroke="currentColor" stroke-width="1.6" />"#,
        | "compass" => {
            r#"<circle class="s" cx="12" cy="12" r="9" />
      <path class="s" d="m16 8-2.5 5.5L8 16l2.5-5.5Z" />"#
        }
        | "diff" => r#"<path class="s" d="M18 3v18M6 8V3M3 6l3-3 3 3M3 18l3 3 3-3M6 21v-5M15 7h6M15 17h6" />"#,
        | "download" => {
            r#"<path d="M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />"#
        }
        | "filecheck" => {
            r#"<path class="s" d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z" />
      <path class="s" d="M14 2v6h6M9 15l2 2 4-4" />"#
        }
        | "globe" => {
            r#"<circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" stroke-width="1.5" />
      <ellipse cx="12" cy="12" rx="4" ry="9" fill="none" stroke="currentColor" stroke-width="1.5" />
      <path d="M3 12h18" stroke="currentColor" />"#
        }
        | "home" => r#"<path class="s" d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1Z" />"#,
        | "info" => {
            r#"<circle class="s" cx="12" cy="12" r="9" />
      <path class="s" d="M12 11v5M12 8h.01" />"#
        }
        | "panel" => {
            r#"<rect class="s" x="3" y="4" width="18" height="16" rx="2" />
      <path class="s" d="M9 4v16M14 8l-3 4 3 4" />"#
        }
        | "portfolio" => {
            r#"<rect class="s" x="3" y="3" width="7" height="7" rx="1" />
      <rect class="s" x="14" y="3" width="7" height="7" rx="1" />
      <rect class="s" x="3" y="14" width="7" height="7" rx="1" />
      <rect class="s" x="14" y="14" width="7" height="7" rx="1" />"#
        }
        | "search" => {
            r#"<circle class="stroke" cx="11" cy="11" r="7" />
      <path class="stroke" d="m20 20-4-4" />"#
        }
        | "settings" => {
            r#"<circle class="s" cx="12" cy="12" r="3" />
      <path class="s" d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-2.8 2.8-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.6v.2h-4V21a1.7 1.7 0 0 0-1-1.6 1.7 1.7 0 0 0-1.9.3l-.1.1L4.2 17l.1-.1a1.7 1.7 0 0 0 .3-1.9A1.7 1.7 0 0 0 3 14H2.8v-4H3a1.7 1.7 0 0 0 1.6-1 1.7 1.7 0 0 0-.3-1.9L4.2 7 7 4.2l.1.1A1.7 1.7 0 0 0 9 4.6a1.7 1.7 0 0 0 1-1.6v-.2h4V3a1.7 1.7 0 0 0 1 1.6 1.7 1.7 0 0 0 1.9-.3l.1-.1L19.8 7l-.1.1a1.7 1.7 0 0 0-.3 1.9 1.7 1.7 0 0 0 1.6 1h.2v4H21a1.7 1.7 0 0 0-1.6 1Z" />"#
        }
        | "shield" => {
            r#"<path class="s" d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10Z" />
      <path class="s" d="m9 12 2 2 4-4" />"#
        }
        | "shuffle" => r#"<path class="s" d="M16 3h5v5M4 20 21 3M21 16v5h-5M15 15l6 6M4 4l5 5" />"#,
        | _ => "",
    };
    rsx! { svg { class: "nav-icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", "focusable": "false", dangerous_inner_html: artwork } }
}
#[component]
fn Technology() -> Element {
    rsx! {
        h2 { "Built with" }
        div { class: "technology-grid",
            for (id, name, purpose, url) in [
                ("rust", "Rust", "Portable computation", "https://www.rust-lang.org/"),
                ("webassembly", "WebAssembly", "Local browser runtime", "https://webassembly.org/"),
                ("dioxus", "Dioxus", "Shared interface", "https://dioxuslabs.com/"),
                ("tauri", "Tauri", "Desktop delivery · planned", "https://tauri.app/"),
            ] {
                a { class: "technology-card", href: url, target: "_blank", rel: "noopener noreferrer",
                    img { src: format!("{}#{id}", asset!("/assets/technology.svg")), alt: "", width: "32", height: "32" }
                    span { strong { "{name}" } small { "{purpose}" } }
                }
            }
            for (icon, name, purpose, url) in [("compass", "Driver.js", "Guided tour", "https://driverjs.com/"), ("shield", "WebMCP", "Optional agent tools", "https://webmachinelearning.github.io/webmcp/")] {
                a { class: "technology-card", href: url, target: "_blank", rel: "noopener noreferrer", Icon { name: icon } span { strong { "{name}" } small { "{purpose}" } } }
            }
        }
        p { class: "small muted", "Technology names and marks identify their respective projects; no endorsement is implied. " a { href: asset!("/assets/NOTICE.md"), "Artwork sources and notices" } }
    }
}
