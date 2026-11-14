use crate::geospatial_page::{Geospatial, MapPins};
use crate::results::ResultView;
use crate::skill::{Install, Skill};
use crate::syntax::CodeEditor;
use crate::workspace::{FileTree, Loader, Portfolio};
use acorn_web::operations::{self, Request, Tool};
use acorn_web::presentation;
use acorn_web::session::{Selection, SuppliedFile, Workspace};
use dioxus::html::FileData;
use dioxus::prelude::*;
use serde_json::Value;
use strum::IntoEnumIterator;

const APP_NAME: &str = "RAD Workspace";
const RAD_EXAMPLES: [(&str, &str); 3] = [
    ("Research summary", include_str!("../assets/examples/summary.json")),
    ("Publications and affiliations", SAMPLE),
    ("Compute resources and software", include_str!("../assets/examples/computing.json")),
];
const RAD_TITLE: &str = "Research Activity Data";
const SAMPLE: &str = include_str!("../assets/examples/publications.json");
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
fn AppName() -> Element {
    rsx! { abbr { title: RAD_TITLE, "RAD" } " Workspace" }
}
#[component]
fn FileInput(onload: EventHandler<String>, limit: usize, tool: Tool) -> Element {
    let mut error = use_signal(String::new);
    let mut opened = use_signal(|| false);
    let mut via_url = use_signal(|| false);
    let mut loaded_name = use_signal(String::new);
    let mut generation = use_signal(|| 0_usize);
    use_effect(use_reactive((&tool,), move |_| {
        let next = generation.peek().wrapping_add(1);
        generation.set(next);
        opened.set(false);
        via_url.set(false);
        error.set(String::new());
        loaded_name.set(String::new());
    }));
    let mut close = move || {
        let next = generation.peek().wrapping_add(1);
        generation.set(next);
        opened.set(false);
        via_url.set(false);
        error.set(String::new());
        document::eval(&format!("document.getElementById('source-trigger-{}')?.focus()", tool.slug()));
    };
    let mut finish = move |file: SuppliedFile| {
        loaded_name.set(file.path);
        error.set(String::new());
        opened.set(false);
        via_url.set(false);
        onload.call(file.content);
        document::eval("requestAnimationFrame(() => document.getElementById('content-editor')?.focus())");
    };
    rsx! {
        div { class: "file-picker",
            div { class: "source-launcher",
                button { id: "source-trigger-{tool.slug()}", r#type: "button", class: "source-trigger", "aria-expanded": opened(), "aria-controls": "source-tray-{tool.slug()}",
                    onclick: move |_| { if opened() { close(); } else { opened.set(true); } },
                    Icon { name: "folder-open" } "Load content" span { class: "source-chevron", "aria-hidden": "true", "⌄" }
                }
                if !loaded_name().is_empty() { span { role: "status", class: "source-loaded", title: loaded_name(), Icon { name: "check" } "Loaded {loaded_name}" } }
            }
            input { id: "source-file-{tool.slug()}", r#type: "file", hidden: true, "aria-label": "Choose file", onchange: move |event: FormEvent| async move {
                document::eval("window.acornRemote?.file.cancel()");
                let started = generation();
                if let Some(file) = event.files().into_iter().next() {
                    let path = file.name();
                    let result = read_local_file(file, limit).await;
                    if started == generation() {
                        match result {
                            Ok(content) => finish(SuppliedFile { path, content }),
                            Err(message) => error.set(message),
                        }
                    }
                }
            } }
            if opened() {
                div { id: "source-tray-{tool.slug()}", class: "source-tray", role: "region", "aria-label": "Load content options",
                    onkeydown: move |event| { if event.key() == Key::Escape { event.prevent_default(); event.stop_propagation(); close(); } },
                    div { class: "source-choices",
                        button { r#type: "button", class: "source-option", onclick: move |_| {
                            via_url.set(false); error.set(String::new());
                            document::eval(&format!("const picker = document.getElementById('source-file-{}'); if (picker) {{ picker.value = ''; picker.click(); }}", tool.slug()));
                        }, span { class: "source-option-icon", Icon { name: "folder-open" } } span { strong { "Choose file" } small { "From your device" } } }
                        button { r#type: "button", class: "source-option source-option-link", "aria-expanded": via_url(), onclick: move |_| {
                            let next = !via_url(); via_url.set(next); error.set(String::new());
                            if next { document::eval("requestAnimationFrame(() => document.querySelector('.url-file-loader input')?.focus())"); }
                        }, span { class: "source-option-icon", Icon { name: "link" } } span { strong { "From URL" } small { "A link from the web" } } }
                        button { r#type: "button", class: "source-close", "aria-label": "Close content loader", onclick: move |_| close(), "×" }
                    }
                    if via_url() { UrlFileInput { limit, reset_key: tool.slug().to_owned(), onload: finish } }
                    if !error().is_empty() { p { role: "alert", class: "source-feedback", "data-tone": "error", Icon { name: "alert" } "{error}" } }
                }
            }
        }
    }
}
#[component]
fn Footer() -> Element {
    rsx! { footer { p { class: "acronym-note", "RAD = Research Activity Data." } div { class: "credit-line", "Made with ❤️ by " a { href: "https://github.com/jhwohlgemuth", target: "_blank", rel: "noopener noreferrer", strong { "jhwohlgemuth" } } } } }
}
#[component]
fn Home() -> Element {
    let mut session = use_context::<Signal<String>>();
    let nav = use_navigator();
    let mut workspace = use_context::<Signal<Workspace>>();
    rsx! {
        document::Title { "{APP_NAME}" }
        section { class: "home",
            div { class: "home-banner", img { class: "banner-dark", src: asset!("/assets/generated/banner_dark_mode.png"), alt: "ACORN", width: "320" } img { class: "banner-light", src: asset!("/assets/generated/banner_light_mode.png"), alt: "ACORN", width: "320" } }
            div { class: "home-brand",
                h1 { class: "home-subtitle", AppName {} }
                p { class: "home-tagline", "Plant an acorn, grow your science" }
            }
            div { id: "home-actions", class: "actions centered",
                button { id: "tour-start", class: "primary", "data-driver": asset!("/assets/vendor/driver.js"), onclick: move |_| { document::eval("window.acornChrome?.tour.start()"); }, Icon { name: "compass" } "Take a tour" }
                button { id: "sample-data", class: "sample-action", onclick: move |_| {
                    let mut loaded = Workspace::default().replace(Selection { files: RAD_EXAMPLES.iter().enumerate().map(|(index, (_, content))| SuppliedFile { path: format!("sample-{index}.json"), content: (*content).into() }).collect(), skipped: 0 });
                    loaded.validations = loaded.files.iter().filter_map(|file| operations::run(Tool::Activity, &Request { input: file.content.clone(), ..Request::default() }).ok().map(|value| (file.path.clone(), value))).collect();
                    session.set(loaded.content()); workspace.set(loaded); nav.push(Route::Page { page: "portfolio".into() });
                    document::eval("window.acornChrome?.notifications.notify('Bundled sample activities loaded.')");
                }, span { "aria-hidden": "true", "✨" } "Try Sample Data" }
            }
            Link { id: "skill-entry", to: Route::Page { page: "skill".into() }, class: "skill-card", span { "The home of the ACORN Skill" } p { "Validate metadata, convert formats and inspect identifiers, then use ACORN in your agent environment." } span { class: "muted", "Explore the skill guide →" } }
            Loader {}
            Link { to: Route::Page { page: "install".into() }, "Prefer a desktop app? View install options →" }
            Footer {}
        }
    }
}
#[component]
pub(crate) fn Icon(name: &'static str) -> Element {
    let artwork = match name {
        | "alert" => r#"<path d="m10.3 4-8 14a2 2 0 0 0 1.7 3h16a2 2 0 0 0 1.7-3l-8-14a2 2 0 0 0-3.4 0ZM12 9v4m0 4h.01"/>"#,
        | "book" => {
            r#"<path class="s" d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20V4H6.5A2.5 2.5 0 0 0 4 6.5Z" />
      <path class="s" d="M4 6.5v13M9 10l2 2 4-4" />"#
        }
        | "chart" => r#"<path d="M4 3v17h17M7 16l4-3 4-5 5-4" fill="none" stroke="currentColor" stroke-width="1.6" />"#,
        | "check" => r#"<circle cx="12" cy="12" r="9"/><path d="m8 12 3 3 5-6"/>"#,
        | "code" => r#"<path d="m8 7-5 5 5 5m8-10 5 5-5 5m-3-13-2 16"/>"#,
        | "compass" => {
            r#"<circle class="s" cx="12" cy="12" r="9" />
      <path class="s" d="m16 8-2.5 5.5L8 16l2.5-5.5Z" />"#
        }
        | "copy" => {
            r#"<rect class="s" x="8" y="8" width="12" height="13" rx="2"/><path class="s" d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3"/>"#
        }
        | "diff" => r#"<path class="s" d="M18 3v18M6 8V3M3 6l3-3 3 3M3 18l3 3 3-3M6 21v-5M15 7h6M15 17h6" />"#,
        | "download" => {
            r#"<path d="M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />"#
        }
        | "expand" => r#"<path d="m8 6 4-4 4 4M12 2v7m-4 9 4 4 4-4m-4 4v-7M4 12h16"/>"#,
        | "file" => r#"<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6M8 13h8M8 17h5"/>"#,
        | "filecheck" => {
            r#"<path class="s" d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z" />
      <path class="s" d="M14 2v6h6M9 15l2 2 4-4" />"#
        }
        | "fold" => r#"<path d="m8 3 4 4 4-4m-4-1v5m-4 14 4-4 4 4m-4 1v-5M4 12h16"/>"#,
        | "folder-open" => {
            r#"<path d="m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"/>"#
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
        | "link" => r#"<path d="m10 13 4-4m-6 6-2 2a3.5 3.5 0 0 1-5-5l4-4a3.5 3.5 0 0 1 5 0m4 1 2-2a3.5 3.5 0 0 1 5 5l-4 4a3.5 3.5 0 0 1-5 0"/>"#,
        | "panel" => {
            r#"<rect class="s" x="3" y="4" width="18" height="16" rx="2" />
      <path class="s" d="M9 4v16M14 8l-3 4 3 4" />"#
        }
        | "panel-open" => r#"<rect class="s" x="3" y="4" width="18" height="16" rx="2" /><path class="s" d="M9 4v16m4-12 3 4-3 4" />"#,
        | "play" => r#"<path class="s" d="m8 4 12 8-12 8Z" />"#,
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
        | "upload" => r#"<path d="M12 3v12"/><path d="m17 8-5-5-5 5"/><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>"#,
        | "users" => {
            r#"<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2m20 0v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"/><circle cx="9" cy="7" r="4"/>"#
        }
        | _ => "",
    };
    rsx! { svg { class: "nav-icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", "focusable": "false", dangerous_inner_html: artwork } }
}
#[component]
fn InfoPage(page: String) -> Element {
    let mut map_pins = use_context::<Signal<MapPins>>();
    let mut session = use_context::<Signal<String>>();
    let mut workspace = use_context::<Signal<Workspace>>();
    let title = match page.as_str() {
        | "portfolio" => "Portfolio",
        | "geospatial" => "Geospatial",
        | "install" => "Install RAD Workspace",
        | "skill" => "ACORN Skill",
        | "settings" => "Settings",
        | "about" => "About RAD Workspace",
        | _ => "Page not found",
    };
    let document_title = if title.ends_with(APP_NAME) {
        title.to_owned()
    } else {
        format!("{title} · {APP_NAME}")
    };
    rsx! {
        document::Title { "{document_title}" }
        section { class: "page info-page", "data-page": "{page}", h1 { class: "page-title", title: RAD_TITLE, Icon { name: match page.as_str() { "portfolio" => "portfolio", "geospatial" => "globe", "install" => "download", "skill" => "book", "settings" => "settings", _ => "info" } } "{title}" }
            match page.as_str() {
                "portfolio" => rsx! { Portfolio {} },
                "geospatial" => rsx! { Geospatial {} },
                "install" => rsx! { Install {} },
                "skill" => rsx! { Skill { compact: false } },
                "settings" => rsx! { p { class: "lede", "Appearance and your local session." }
                    div { class: "card", h2 { "🎨 Appearance" }
                        div { class: "theme-choices", "aria-label": "Appearance",
                            for mode in ["Dark", "Light", "System"] {
                                button { "aria-label": mode, "data-theme-choice": mode.to_lowercase(), "aria-pressed": "false", onclick: move |_| { let mode = mode.to_lowercase(); document::eval(&format!("window.acornChrome?.theme.choose('{mode}')")); }, "{mode}" }
                            }
                        }
                        p { id: "theme-mode", class: "small muted", "Following your system preference." }
                    }
                    div { class: "card", h2 { class: "section-heading", Icon { name: "folder-open" } "Loaded session" }
                        dl { class: "result-fields", dt { "Activities" } dd { "{workspace().activities().len()} validated" } dt { "Files" } dd { "{workspace().files.len()} · {workspace().bytes()} UTF-8 bytes" } dt { "Persistence" } dd { "None · this tab only" } }
                        button { disabled: workspace().files.is_empty() && session().is_empty() && map_pins().identifiers.is_empty(), onclick: move |_| { workspace.set(Workspace::default()); session.set(String::new()); map_pins.set(MapPins::default()); document::eval("window.acornWorkspace?.cancel(); window.acornChrome?.notifications.notify('Loaded session cleared. Exported copies are unchanged.')"); }, "Clear loaded session" }
                        p { class: "small muted", "Clearing or reloading removes this session only. Exported files and installed skill copies are unchanged." } }
                    div { class: "card", h2 { "📦 Release" } p { title: RAD_TITLE, "{APP_NAME} · v{VERSION}" } Skill { compact: true }
                        div { class: "actions", Link { to: Route::Page { page: "skill".into() }, "ACORN Skill →" } Link { to: Route::Page { page: "install".into() }, "Install options →" } }
                    }
                },
                "about" => rsx! { p { class: "lede", "Accessible Content Optimization for Research Needs" } a { class: "card source-card", href: "https://code.ornl.gov/research-enablement/acorn", target: "_blank", rel: "noopener noreferrer", div { img { src: asset!("/assets/generated/acorn.svg"), alt: "", width: "32" } h2 { AppName {} } } span { "View source ↗" } } h2 { "ACORN Skill" } p { "The app provides local research tools; the matching skill explains their workflows in your agent environment." } div { class: "actions", Link { to: Route::Page { page: "skill".into() }, "Explore the ACORN Skill →" } Link { to: Route::Page { page: "install".into() }, "Install options →" } } div { class: "about-principles", div { class: "card", h2 { "Local by default" } p { "Local files stay on this device. Remote files and buckets are fetched only when you choose Load. No uploads or telemetry." } } div { class: "card", h2 { "One workspace" } p { "Portable research tools and a matching agent skill. Desktop delivery is planned." } } } Technology {} div { class: "card ai-disclosure", a { href: "https://www.aihonestybadge.com/ai-assisted-badge", target: "_blank", rel: "noopener noreferrer", img { src: asset!("/assets/ai-assisted.svg"), alt: "AI Assisted", width: "113", height: "20" } } h2 { "AI-use disclosure" } p { "AI tools assisted with code, documentation and design under human direction. The maintainer reviews and tests the work. Review generated code before use." } div { class: "agent-links",
                    for (id, name, url) in [("codex", "Codex", "https://openai.com/codex/"), ("copilot", "GitHub Copilot", "https://github.com/features/copilot"), ("microsoft", "Microsoft 365 Copilot", "https://www.microsoft.com/microsoft-365/copilot")] {
                        a { href: url, target: "_blank", rel: "noopener noreferrer", img { src: format!("{}#{id}", asset!("/assets/agent-marks.svg")), alt: "", width: "20", height: "20" } "{name}" }
                    }
                } } p { class: "muted", "Developed at Oak Ridge National Laboratory · Open-source research tools" } },
                _ => rsx! { p { "This route is not available." } Link { to: Route::Home {}, "Return Home" } },
            }
        }
    }
}
#[component]
fn Page(page: String) -> Element {
    if page == "home" {
        rsx! { Home {} }
    } else if let Some(tool) = Tool::iter().find(|tool| tool.slug() == page) {
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
pub(crate) fn ResultTree(value: Value, depth: usize) -> Element {
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
fn SelectField(label: &'static str, value: String, options: Vec<&'static str>, onchange: EventHandler<String>) -> Element {
    rsx! { label { class: "field", span { "{label}" } select { value, onchange: move |event| onchange.call(event.value()), for option in options { option { value: option, "{presentation::label(option)}" } } } } }
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
            header { class: "mobile-bar", span { AppName {} } button { id: "nav-toggle", onclick: move |_| { drawer.toggle(); if drawer() { document::eval("requestAnimationFrame(() => document.querySelector('#workspace-navigation a')?.focus())"); } }, "aria-expanded": drawer(), "aria-controls": "workspace-navigation", "aria-label": "Toggle navigation", Icon { name: match drawer() { true => "panel", false => "panel-open" } } } }
            aside { id: "workspace-navigation", class: "sidebar", "aria-label": "Workspace navigation",
                div { class: "nav-controls", button { class: "collapse", onclick: move |_| collapsed.toggle(), title: if collapsed() { "Expand sidebar" } else { "Collapse sidebar" }, "aria-label": if collapsed() { "Expand sidebar" } else { "Collapse sidebar" }, "aria-expanded": !collapsed(), "aria-controls": "workspace-navigation", Icon { name: match collapsed() { true => "panel-open", false => "panel" } } } }
                SidebarSession { onopen: move |_| { if drawer() { drawer.set(false); document::eval("requestAnimationFrame(() => document.querySelector('#content-main h1')?.focus())"); } } }
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
                            Link { to: Route::Page { page: tool.slug().into() }, class: if current == tool.slug() { "nav-item active" } else { "nav-item" }, title: tool.to_string(), "aria-label": tool.to_string(), "aria-current": if current == tool.slug() { "page" } else { "false" }, Icon { name: tool.icon() } span { class: "nav-label", if tool == Tool::Discover { "Discover" } else if tool == Tool::Inspect { "Inspect" } else { "{tool.to_string()}" } } }
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

                    small { title: RAD_TITLE, "{APP_NAME} · v{VERSION}" }
                    small { id: "webmcp-status", "data-adapter": asset!("/assets/generated/webmcp.js"), "WebMCP: initializing" }
                }
            }
            div { id: "notification", class: "notification", hidden: true, span { id: "notification-text", role: "status", "aria-live": "polite" } button { "aria-label": "Dismiss notification", onclick: move |_| { document::eval("window.acornChrome?.notifications.dismiss()"); }, "Dismiss" } }
            main { id: "content-main", inert: drawer().then_some(""), tabindex: "-1", div { class: "page-toolbar", button { id: "theme-toggle", class: "theme-toggle", "aria-label": "Dark theme", title: "Toggle dark or light theme", "aria-pressed": "true", onclick: move |_| { document::eval("window.acornChrome?.theme.toggle()"); }, span { class: "switch-track", span { class: "switch-thumb" } } span { class: "theme-icon", "aria-hidden": "true", span { class: "theme-moon", "☾" } span { class: "theme-sun", "☀" } } span { id: "theme-label", class: "nav-label", "Dark" } } } Outlet::<Route> {} }
            crate::details::DetailsDrawer {}
        }
    }
}
#[component]
fn SidebarSession(onopen: EventHandler<()>) -> Element {
    let workspace = use_context::<Signal<Workspace>>();
    let snapshot = workspace.read();
    let files = snapshot.files.len();
    let validated = snapshot.activities().len();
    let selected = snapshot.selected.as_deref().unwrap_or("No file selected");
    let activity_label = if validated == 1 { "activity" } else { "activities" };
    let file_label = if files == 1 { "file" } else { "files" };
    let summary = format!("{validated} validated {activity_label} · {files} {file_label}. Selected: {selected}");
    rsx! {
        Link { to: Route::Page { page: "portfolio".into() }, class: "sidebar-session", title: summary.clone(), "aria-label": summary, onclick: move |_| onopen.call(()),
            span { class: "session-stats session-detail", "{validated} {activity_label} · {files} {file_label}" }
            span { class: "session-count", "{validated} · {files}" }
        }
    }
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
#[component]
fn ToolWorkspace(tool: Tool) -> Element {
    let session = use_context::<Signal<String>>();
    let mut workspace = use_context::<Signal<Workspace>>();
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
        document::eval("window.acornRemote?.file.cancel()");
    });
    use_effect(move || {
        let content = session();
        let previous = request.peek().clone();
        request.set(Request { input: content, ..previous });
    });
    let title = tool.to_string();
    let limit = tool.limit();
    let input = request().input;
    let needs_content = input.trim().is_empty() && !(tool == Tool::Diff && request().mode == "unified");
    let language = match tool {
        | Tool::BigO => "expression".to_owned(),
        | Tool::Inspect => "identifier".to_owned(),
        | Tool::Standard | Tool::Crosswalk => request().format,
        | Tool::Activity | Tool::Fair => "json".to_owned(),
        | _ => "auto".to_owned(),
    };
    rsx! {
        document::Title { "{title} · {APP_NAME}" }
        section { class: "page tool-page",

            h1 { class: "page-title", Icon { name: tool.icon() } "{title}" } p { class: "lede", "{tool.description()}" }
            if !workspace().files.is_empty() { details { summary { "Loaded files" } FileTree { paths: workspace.read().files.iter().map(|file| file.path.clone()).collect(), prefix: String::new() } } }
            div { class: "tool-grid",
                section { class: "card editor",
                    div { class: "input-heading",
                        h2 { class: "section-heading", "Input" }
                        button { class: "sample-action", title: match tool { Tool::Activity | Tool::Standard => "Load intentionally flawed RAD with multiple validation issues", Tool::Diff => "Load a nested JSON example for semantic diff", Tool::BigO | Tool::Crosswalk | Tool::Discover | Tool::Fair | Tool::Inspect | Tool::Readability => "Load a detailed example" }, onclick: move |_| { request.set(operations::example_request(tool)); result.set(None); }, span { "aria-hidden": "true", "✨" } "Use example" }
                    }
                    FileInput { limit, tool, onload: move |text: String| { request.set(Request { input: text, ..request() }); result.set(None); } }
                    div { class: "options",
                        if matches!(tool, Tool::Activity) {
                            fieldset { class: "field vocabulary-field", legend { "Vocabulary matching" }
                                div { class: "vocabulary-choices",
                                    for (value, label) in [("off", "Off"), ("relaxed", "Relaxed"), ("strict", "Strict")] {
                                        button { r#type: "button", "aria-pressed": request().fuzzy == value, title: presentation::label(value), onclick: move |_| request.set(Request { fuzzy: value.into(), ..request() }), "{label}" }
                                    }
                                }
                            }
                        }
                        if matches!(tool, Tool::Standard | Tool::Crosswalk | Tool::Fair) { SelectField { label: "Standard / source", value: request().standard, options: if tool == Tool::Crosswalk { vec!["research-activity-data", "raid", "datacite", "dublin-core"] } else { vec!["research-activity-data", "raid", "datacite", "dublin-core", "citation-file-format", "ckan", "document", "invenio", "kitfile", "modelkit", "text"] }, onchange: move |v| request.set(Request { standard: v, ..request() }) } }
                        if matches!(tool, Tool::Standard | Tool::Crosswalk) { SelectField { label: "Format", value: request().format, options: vec!["json", "jsonc", "json-ld", "yaml", "zon", "cff", "markdown", "rdf-xml", "turtle", "text"], onchange: move |v| request.set(Request { format: v, ..request() }) } }
                        if tool == Tool::Crosswalk { SelectField { label: "Destination", value: request().target, options: vec!["datacite", "dublin-core", "raid", "research-activity-data"], onchange: move |v| request.set(Request { target: v, ..request() }) } }
                        if tool == Tool::Readability { SelectField { label: "Metric", value: request().metric, options: vec!["fres", "ari", "cli", "fkgl", "gfi", "lix", "smog"], onchange: move |v| request.set(Request { metric: v, ..request() }) } }
                        if tool == Tool::Diff {
                            fieldset { class: "field vocabulary-field", legend { "Diff format" }
                                div { class: "diff-choices",
                                    for (value, icon, label) in [("unified", "±", "Unified diff"), ("json-patch", "{}", "JSON Patch")] {
                                        button { r#type: "button", "data-format": value, "aria-pressed": request().mode == value,
                                            onclick: move |_| request.set(Request { mode: value.into(), ..request() }),
                                            span { class: "diff-format-icon", "aria-hidden": "true", "{icon}" } "{label}"
                                        }
                                    }
                                }
                            }
                        }
                        if tool == Tool::Inspect { SelectField { label: "Identifier kind", value: request().kind, options: vec!["", "ark", "arxiv", "doi", "handle", "isbn", "isni", "orcid", "patent", "raid", "ror", "rrid", "swhid"], onchange: move |v| request.set(Request { kind: v, ..request() }) } }
                    }
                    div { class: if matches!(tool, Tool::Diff | Tool::BigO) { "editor-pair" } else { "" },
                        div { class: "field", span { if tool == Tool::BigO { "Left expression" } else if tool == Tool::Diff { "Before" } else { "Content" } }
                            CodeEditor { id: "content", label: if tool == Tool::BigO { "Left expression" } else if tool == Tool::Diff { "Before" } else { "Content" }, language: language.clone(), rows: if tool == Tool::BigO { 3 } else { 7 }, value: input.clone(), oninput: move |value| { request.set(Request { input: value, ..request() }); result.set(None); } }
                            small { class: if input.len() > limit { "error" } else { "muted" }, "{input.len()} / {limit} UTF-8 bytes" }
                        }
                        if matches!(tool, Tool::Diff | Tool::BigO) {
                            div { class: "field", span { if tool == Tool::BigO { "Right expression" } else { "After" } }
                                CodeEditor { id: "after", label: if tool == Tool::BigO { "Right expression" } else { "After" }, language: language.clone(), rows: if tool == Tool::BigO { 3 } else { 7 }, value: request().after, oninput: move |value| { request.set(Request { after: value, ..request() }); result.set(None); } }
                                small { class: if request().after.len() > limit { "error" } else { "muted" }, "{request().after.len()} / {limit} UTF-8 bytes" }
                            }
                        }
                    }
                    if tool == Tool::Readability { p { class: "small muted", "{input.split_whitespace().count()} words · {input.len()} UTF-8 bytes" } }
                    if tool == Tool::BigO { div { class: "field", span { "Symbol assumptions (JSON)" } CodeEditor { id: "assumptions", label: "Symbol assumptions (JSON)", language: "json", rows: 2, value: request().assumptions, oninput: move |value| request.set(Request { assumptions: value, ..request() }) } } }
                    if tool == Tool::Crosswalk { details { summary { "Import seeds (when required)" } div { class: "field", span { "Contact JSON" } CodeEditor { id: "contact", label: "Contact JSON", language: "json", rows: 3, value: request().contact, oninput: move |value| request.set(Request { contact: value, ..request() }) } } label { class: "field", "Local identifier" input { value: request().local_identifier, oninput: move |e| request.set(Request { local_identifier: e.value(), ..request() }) } } } }
                    div { class: "actions run-actions", button { class: "primary", onclick: move |_| { let output = operations::run(tool, &request());
                        if tool == Tool::Activity {
                            let mut next = workspace();
                            if let Some(file) = next.files.iter().find(|file| Some(&file.path) == next.selected.as_ref() && file.content == request().input) {
                                if let Ok(value) = &output { next.validations.insert(file.path.clone(), value.clone()); }
                            }
                            workspace.set(next);
                        }
                        let celebrate = matches!(tool, Tool::Activity | Tool::Standard) && output.as_ref().is_ok_and(|value| value["valid"] == true);
                        result.set(Some(output)); document::eval(&format!("requestAnimationFrame(() => {{ document.getElementById('result-heading')?.focus(); if ({celebrate}) window.acornCelebrate?.(); }})")); }, Icon { name: tool.icon() } "{tool.action_label()}" } button { onclick: move |_| { request.set(Request::default()); result.set(None); }, "Clear" } }
                    p { class: "muted small", "Runs locally. No identifiers are resolved and no content is uploaded." }
                }
                section { class: "card results", "aria-live": "polite", h2 { id: "result-heading", class: "section-heading", tabindex: "-1", "Results" }
                    match result() {
                        None => rsx! { div { class: "empty", Icon { name: tool.icon() } h3 { "Ready when you are" } p { "Add your content, choose options, and select {tool.action_label()}." } } },
                        Some(Err(message)) => rsx! {
                            div { class: "operation-error", role: "alert", "data-tone": if needs_content { "warning" } else { "danger" },
                                div { class: "operation-error-heading",
                                    span { class: "operation-error-icon", Icon { name: "alert" } }
                                    h3 { if needs_content { "Add content to get started" } else { "Could not complete this operation" } }
                                }
                                p { class: "operation-error-message",
                                    if needs_content { "Paste content, open a file, or use an example before running {title}." }
                                    else { "{message}" }
                                }
                                div { class: "operation-error-actions",
                                    button { r#type: "button", onclick: move |_| {
                                        document::eval("(document.getElementById('content-editor') || document.getElementById('content'))?.focus()");
                                    }, Icon { name: "code" } "Back to input" }
                                }
                            }
                        },
                        Some(Ok(value)) => rsx! { ResultView { tool, value, source: input.clone() } },
                    }
                }
            }
        }
    }
}
#[component]
pub(crate) fn UrlFileInput(
    onload: EventHandler<SuppliedFile>,
    limit: usize,
    #[props(default)] workspace_file: bool,
    #[props(default)] reset_key: String,
) -> Element {
    let mut url = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut error = use_signal(String::new);
    let mut generation = use_signal(|| 0_usize);
    use_effect(use_reactive((&reset_key,), move |_| {
        let next = generation.peek().wrapping_add(1);
        generation.set(next);
        url.set(String::new());
        loading.set(false);
        error.set(String::new());
        document::eval("window.acornRemote?.file.cancel()");
    }));
    use_drop(move || {
        document::eval("window.acornRemote?.file.cancel()");
    });
    let cancelled = error().contains("cancelled");
    let feedback_icon = if cancelled { "info" } else { "alert" };
    rsx! {
        form { class: "url-file-loader", onsubmit: move |event| async move {
            event.prevent_default();
            if loading() { return; }
            loading.set(true); error.set(String::new());
            let started = generation();
            let args = serde_json::to_string(&(url(), limit, workspace_file)).unwrap_or_default();
            let result = document::eval(&format!("if (!window.acornRemote) await new Promise(resolve => window.addEventListener('acorn-remote-ready', resolve, {{once:true}})); try {{ return {{ok: await window.acornRemote.file.load(...{args})}}; }} catch(error) {{ return {{error: error.message}}; }}")).join::<Value>().await;
            if started != generation() { return; }
            loading.set(false);
            match result {
                Ok(value) => {
                    if let Some(message) = value.get("error").and_then(Value::as_str) { error.set(message.to_owned()); }
                    else {
                        match serde_json::from_value::<SuppliedFile>(value.get("ok").cloned().unwrap_or(Value::Null)) {
                            Ok(file) => onload.call(file),
                            Err(_) => error.set("The URL returned an invalid file. Current content is unchanged.".into()),
                        }
                    }
                },
                Err(_) => error.set("Could not load this file. Current content is unchanged.".into()),
            }
        },
            div { class: "url-file-controls",
                label { class: "field", "File URL" input { r#type: "url", required: true, disabled: loading(), value: url(), oninput: move |event| url.set(event.value()), placeholder: "https://example.org/file.json", spellcheck: "false" } }
                button { r#type: "submit", class: "source-submit", disabled: loading() || url().trim().is_empty(), if loading() { span { class: "source-spinner", "aria-hidden": "true" } "Loading…" } else { Icon { name: "download" } "Load URL" } }
                if loading() { button { r#type: "button", onclick: move |_| { document::eval("window.acornRemote?.file.cancel()"); }, "Cancel" } }
            }
            p { class: "small muted", "Direct file links · GitHub · GitLab" }
            details { class: "source-tips", summary { Icon { name: "info" } "Link tips" }
                p { "Use a public HTTPS link to a UTF-8 text file, up to {limit} bytes. The server must allow browser access (CORS). If a link is blocked, download the file and choose it from your device." }
            }
            if loading() { p { role: "status", class: "small muted", "Downloading file…" } }
            if !error().is_empty() { p { role: if cancelled { "status" } else { "alert" }, class: "source-feedback", "data-tone": if cancelled { "info" } else { "error" }, Icon { name: feedback_icon } "{error}" } }
        }
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
