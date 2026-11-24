use crate::geospatial_page::{Geospatial, MapPins};
use crate::icon::IconName;
use crate::results::ResultView;
use crate::sample;
use crate::skill::{Install, Skill};
use crate::syntax::CodeEditor;
use crate::window_sync::{self, ToolSignals};
use crate::workspace::{file_format, LoadedFiles, Loader, Portfolio};
use acorn_analysis::readability::ReadabilityType;
use acorn_core::util::MimeType;
use acorn_schema::{pid::PID, standard::Standard};
use acorn_web::operations::{self, Request, Tool};
use acorn_web::presentation;
use acorn_web::session::{Selection, SuppliedFile, Workspace, FILE_LIMIT, TOTAL_LIMIT};
use alloc::collections::BTreeMap;
use core::iter::once;
use dioxus::html::FileData;
use dioxus::prelude::*;
use serde_json::Value;
use strum::{EnumIs, IntoEnumIterator};

const APP_NAME: &str = "ACORN Research Intelligence";
const FORMAT_CHOICES: [(MimeType, &str, &str); 10] = [
    (MimeType::Json, "JSON", "JavaScript Object Notation"),
    (MimeType::Jsonc, "JSONC", "JavaScript Object Notation with Comments"),
    (MimeType::LdJson, "JSON-LD", "JavaScript Object Notation for Linked Data"),
    (MimeType::Yaml, "YAML", "YAML Ain't Markup Language"),
    (MimeType::Zon, "ZON", "Zig Object Notation"),
    (MimeType::Cff, "CFF", "Citation File Format"),
    (MimeType::Markdown, "MD", "Markdown"),
    (MimeType::RdfXml, "RDF", "Resource Description Framework XML"),
    (MimeType::Turtle, "TTL", "Turtle"),
    (MimeType::Text, "TXT", "Plain text"),
];
const FORMAT_WIDTHS: [usize; 10] = [42, 46, 60, 44, 37, 35, 32, 37, 34, 35];
const RAD_TITLE: &str = "Research Activity Data";
const STANDARD_CHOICES: [(Standard, &str, &str); 11] = [
    (Standard::CitationFileFormat, "CFF", "Citation File Format"),
    (Standard::Ckan, "CKAN", "Comprehensive Knowledge Archive Network"),
    (Standard::Datacite, "DCITE", "DataCite Metadata Schema"),
    (Standard::Document, "DOC", "Document"),
    (Standard::DublinCore, "DC", "Dublin Core"),
    (Standard::Invenio, "INV", "Invenio"),
    (Standard::Kitfile, "KIT", "Kitfile"),
    (Standard::ModelKit, "MK", "ModelKit"),
    (Standard::Raid, "RAiD", "Research Activity Identifier"),
    (Standard::ResearchActivityData, "RADS", "Research Activity Data"),
    (Standard::Text, "TXT", "Plain text"),
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
#[derive(Clone, Copy, EnumIs)]
enum WindowType {
    CodeEditor,
    Dashboard,
    Workspace,
}
impl WindowType {
    fn for_route(route: &Route, window_name: &str) -> Self {
        if window_name.starts_with("acorn-editor:") {
            return Self::CodeEditor;
        }
        match route {
            | Route::Page { page } if page == "dashboard" => Self::Dashboard,
            | Route::Home {} | Route::Page { .. } => Self::Workspace,
        }
    }
}
/// In-memory tool forms and results retained while navigating between tools.
#[derive(Clone, Copy)]
pub(crate) struct ToolSessions {
    activity: ToolSignals,
    big_o: ToolSignals,
    crosswalk: ToolSignals,
    diff: ToolSignals,
    discover: ToolSignals,
    fair: ToolSignals,
    inspect: ToolSignals,
    readability: ToolSignals,
    standard: ToolSignals,
}
impl ToolSessions {
    pub(crate) fn new() -> Self {
        let review_request = Signal::new(Request::default());
        let review_last_request = Signal::new(Request::default());
        let review_seen_session = Signal::new(String::new());
        let review_drafts = Signal::new(BTreeMap::new());
        let review_editor_source = Signal::new(None);
        Self {
            activity: ToolSignals::isolated(),
            big_o: ToolSignals::isolated(),
            crosswalk: ToolSignals::isolated(),
            diff: ToolSignals::isolated(),
            discover: ToolSignals::isolated(),
            fair: ToolSignals::new(
                review_request,
                review_last_request,
                review_seen_session,
                review_drafts,
                review_editor_source,
            ),
            inspect: ToolSignals::isolated(),
            readability: ToolSignals::new(
                review_request,
                review_last_request,
                review_seen_session,
                review_drafts,
                review_editor_source,
            ),
            standard: ToolSignals::new(
                review_request,
                review_last_request,
                review_seen_session,
                review_drafts,
                review_editor_source,
            ),
        }
    }
    fn for_tool(self, tool: Tool) -> ToolSignals {
        match tool {
            | Tool::Activity => self.activity,
            | Tool::BigO => self.big_o,
            | Tool::Crosswalk => self.crosswalk,
            | Tool::Diff => self.diff,
            | Tool::Discover => self.discover,
            | Tool::Fair => self.fair,
            | Tool::Inspect => self.inspect,
            | Tool::Readability => self.readability,
            | Tool::Standard => self.standard,
        }
    }
    fn clear_review_results(mut self) {
        self.fair.result.set(None);
        self.readability.result.set(None);
        self.standard.result.set(None);
    }
    fn clear_running(self) {
        for mut signals in [
            self.activity,
            self.big_o,
            self.crosswalk,
            self.diff,
            self.discover,
            self.fair,
            self.inspect,
            self.readability,
            self.standard,
        ] {
            signals.running.set(false);
        }
    }
}
fn editor_source_file(workspace: &Workspace, source: Option<&SuppliedFile>) -> Option<SuppliedFile> {
    source.and_then(|source| {
        workspace
            .files
            .iter()
            .find(|file| file.path == source.path && file.content == source.content)
            .cloned()
    })
}
fn editor_save_allowed(workspace: &Workspace, file: &SuppliedFile, input: &str, limit: usize) -> bool {
    input.len() <= limit.min(FILE_LIMIT) && workspace.bytes().saturating_sub(file.content.len()).saturating_add(input.len()) <= TOTAL_LIMIT
}
#[cfg(test)]
mod editor_source_tests {
    use super::*;

    #[test]
    fn save_source_follows_editor_input_across_workspace_selection_changes() {
        let source = SuppliedFile {
            path: "a.json".into(),
            content: "original A".into(),
        };
        let other = SuppliedFile {
            path: "b.json".into(),
            content: "original B".into(),
        };
        let mut workspace = Workspace {
            files: vec![source.clone(), other],
            selected: Some("b.json".into()),
            ..Workspace::default()
        };
        assert_eq!(
            editor_source_file(&workspace, Some(&source)).as_ref().map(|file| file.path.as_str()),
            Some("a.json")
        );
        workspace.files.get_mut(0).expect("source file in fixture").content = "newer A".into();
        assert!(editor_source_file(&workspace, Some(&source)).is_none());
    }
}
#[component]
fn AppName() -> Element {
    rsx! { "{APP_NAME}" }
}
/// The single dispatch seam for browser operations: the operation worker when available, otherwise in-thread.
pub(crate) async fn execute(tool: Tool, request: Request) -> Result<Value, String> {
    let message = serde_json::to_string(&(tool, &request)).unwrap_or_default();
    let channel = document::eval("const message = await dioxus.recv(); return await (window.acornOperations?.run(message) ?? { unavailable: 'The operation client is unavailable.' });");
    let reply = match channel.send(message) {
        | Ok(()) => channel.join::<Value>().await.unwrap_or(Value::Null),
        | Err(_) => Value::Null,
    };
    match (reply.get("reply").and_then(Value::as_str), reply.get("error").and_then(Value::as_str)) {
        | (Some(text), _) => operations::decode(text),
        | (None, Some(message)) => Err(message.to_owned()),
        | (None, None) => {
            // Let the busy state paint before the synchronous computation holds the main thread.
            let _ = document::eval("await new Promise(resolve => requestAnimationFrame(() => setTimeout(resolve)))").await;
            operations::run(tool, &request)
        }
    }
}
#[component]
pub(crate) fn FileInput(onload: EventHandler<String>, limit: usize, tool: Tool) -> Element {
    let mut error = use_signal(String::new);
    let mut via_url = use_signal(|| false);
    let mut generation = use_signal(|| 0_usize);
    use_effect(use_reactive((&tool,), move |_| {
        let next = generation.peek().wrapping_add(1);
        generation.set(next);
        via_url.set(false);
        error.set(String::new());
    }));
    let mut finish = move |file: SuppliedFile| {
        error.set(String::new());
        via_url.set(false);
        onload.call(file.content);
        document::eval("requestAnimationFrame(() => document.getElementById('content-editor')?.focus())");
    };
    rsx! {
        div { class: "file-picker",
            div { class: "source-choices",
                button { r#type: "button", class: "source-option", onclick: move |_| {
                    via_url.set(false); error.set(String::new());
                    document::eval(&format!("const picker = document.getElementById('source-file-{}'); if (picker) {{ picker.value = ''; picker.click(); }}", tool.slug()));
                }, span { class: "source-option-icon", Icon { name: "folder-open" } } span { strong { "Choose file" } small { "From your device" } } }
                button { r#type: "button", class: "source-option source-option-link", "aria-expanded": via_url(), onclick: move |_| {
                    let next = !via_url(); via_url.set(next); error.set(String::new());
                    if next { document::eval("requestAnimationFrame(() => { const input = document.querySelector('.file-picker .url-file-loader input'); if (input) input.focus(); })"); }
                }, span { class: "source-option-icon", Icon { name: "link" } } span { strong { "From URL" } small { "A link from the web" } } }
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
            if via_url() { UrlFileInput { limit, reset_key: tool.slug().to_owned(), onload: finish } }
            if !error().is_empty() { p { role: "alert", class: "source-feedback", "data-tone": "error", Icon { name: "alert" } "{error}" } }
        }
    }
}
#[component]
fn Footer() -> Element {
    rsx! { footer { div { class: "credit-line", "Made with ❤️ by " a { href: "https://github.com/jhwohlgemuth", target: "_blank", rel: "noopener noreferrer", strong { "jhwohlgemuth" } } } } }
}
#[component]
fn Home() -> Element {
    let mut session = use_context::<Signal<String>>();
    let nav = use_navigator();
    let mut workspace = use_context::<Signal<Workspace>>();
    let mut loading = use_signal(|| false);
    let sample_loaded = sample::is_loaded(&workspace());
    rsx! {
        document::Title { "{APP_NAME}" }
        section { class: "home",
            div { class: "home-banner", img { class: "banner-dark", src: asset!("/assets/generated/dark/banner.png"), alt: "ACORN", width: "320" } img { class: "banner-light", src: asset!("/assets/generated/light/banner.png"), alt: "ACORN", width: "320" } img { class: "banner-uswds", src: asset!("/assets/generated/uswds/banner.png"), alt: "ACORN", width: "320" } }
            div { class: "home-brand",
                h1 { class: "home-subtitle", "Research Intelligence" }
            }
            div { class: "home-cta",
                p { class: "home-tagline", "Plant an acorn, grow your science 🌱" }
                div { id: "home-actions", class: "actions centered",
                    button { id: "tour-start", class: "primary", "data-driver": asset!("/assets/vendor/driver.js"), onclick: move |_| { document::eval("window.acornChrome?.tour.start()"); }, Icon { name: "compass" } "Take a tour" }
                    if !sample_loaded { button { id: "sample-data", class: "sample-action", disabled: loading(), "aria-busy": loading(), onclick: move |_| async move {
                        loading.set(true);
                        let mut loaded = Workspace::default().replace(Selection { files: sample::files(), skipped: 0 });
                        loaded.validations = validate_files(loaded.files.iter().filter(|file| matches!(MimeType::from(file.path.as_str()), MimeType::Json)).cloned().collect()).await.into_iter().filter_map(|(path, result)| result.ok().map(|value| (path, value))).collect();
                        loaded.selected = sample_activity_path(&loaded);
                        loading.set(false);
                        session.set(loaded.content()); workspace.set(loaded); nav.push(Route::Page { page: "portfolio".into() });
                        document::eval("window.acornChrome?.notifications.notify('Synthetic research bucket loaded.', 'success')");
                    }, span { "aria-hidden": "true", "✨" } if loading() { "Loading samples…" } else { "Try Sample Data" } } }
                }
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
    let icon = name.parse::<IconName>().unwrap_or(IconName::Unknown);
    let artwork = icon.legacy_artwork();
    let uswds_artwork = icon
        .uswds_symbol()
        .map(|id| format!(r#"<use href="{}#{id}"></use>"#, asset!("/assets/vendor/uswds-sprite.svg")));
    rsx! {
        svg { class: "nav-icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", "focusable": "false", role: "img",
            g { class: "legacy-icon", dangerous_inner_html: artwork }
            if let Some(uswds_artwork) = uswds_artwork {
                g { class: "usa-icon", dangerous_inner_html: uswds_artwork }
            }
        }
    }
}
#[component]
fn InfoPage(page: String) -> Element {
    let mut collapsed = use_context::<Signal<bool>>();
    let mut map_pins = use_context::<Signal<MapPins>>();
    let mut session = use_context::<Signal<String>>();
    let mut workspace = use_context::<Signal<Workspace>>();
    let mut storage_status = use_context::<Signal<(usize, bool, usize, usize)>>();
    let recovery_timed_out = use_context::<crate::RecoveryTimedOut>().0;
    let mut storage_error = use_signal(String::new);
    let mut storage_message = use_signal(String::new);
    let mut settings_message = use_signal(String::new);
    let mut credential_status = use_signal(String::new);
    let mut credential_error = use_signal(String::new);
    let mut credential_message = use_signal(String::new);
    let mut recent_inputs = use_signal(|| (0_usize, 0_usize));
    use_future(move || async move {
        if let Ok(value) = document::eval("if (!window.acornWorkspaceStorage) await new Promise(resolve => window.addEventListener('acorn-workspace-storage-ready', resolve, {once:true})); return await window.acornWorkspaceStorage.status()")
            .join::<Value>()
            .await
        {
            storage_status.set(parse_storage_status(&value));
        }
    });
    use_future(move || async move {
        if let Ok(host) = document::eval("if (!window.acornRemote) await new Promise(resolve => window.addEventListener('acorn-remote-ready', resolve, {once:true})); return window.acornRemote.credentialStatus();").join::<String>().await {
            credential_status.set(host);
        }
    });
    use_future(move || async move {
        if let Ok(counts) = document::eval("if (!window.acornRemote) await new Promise(resolve => window.addEventListener('acorn-remote-ready', resolve, {once:true})); const history = window.acornRemote.history.read(); return [history.repositories.length, history.refs.length];").join::<(usize, usize)>().await {
            recent_inputs.set(counts);
        }
    });
    let title = match page.as_str() {
        | "aspect" => "ASPECT",
        | "portfolio" => "Portfolio",
        | "dashboard" => "Dashboard",
        | "geospatial" => "Geospatial",
        | "s3a" => "S3A",
        | "install" => "Install ACORN Research Intelligence",
        | "skill" => "ACORN Skill",
        | "settings" => "Settings",
        | "about" => "About ACORN Research Intelligence",
        | _ => "Page not found",
    };
    let document_title = if title.ends_with(APP_NAME) {
        title.to_owned()
    } else {
        format!("{title} · {APP_NAME}")
    };
    let source_summary = workspace
        .read()
        .source
        .as_ref()
        .map(|source| format!("{} · {}/{}", source.kind, source.host, source.project));
    let (saved_bytes, _, site_usage, site_quota) = storage_status();
    let site_percent = if site_quota > 0 {
        (site_usage as f64 / site_quota as f64 * 100.0).min(100.0)
    } else {
        0.0
    };
    let saved_label = if recovery_timed_out() {
        "Unavailable while recovery is stalled".to_string()
    } else if saved_bytes > 0 {
        storage_size(saved_bytes)
    } else {
        "No saved files".to_string()
    };
    rsx! {
        document::Title { "{document_title}" }
        section { class: "page info-page", "data-page": "{page}", h1 { class: "page-title", title: RAD_TITLE, Icon { name: match page.as_str() { "aspect" => "chart", "portfolio" | "dashboard" => "portfolio", "geospatial" => "globe", "s3a" => "database", "install" => "download", "skill" => "book", "settings" => "settings", _ => "info" } }
            "{title}"
        }
            match page.as_str() {
                "aspect" | "s3a" => rsx! { div { class: "card construction-card", div { class: "construction-copy", h2 { "Under construction" } p { "This page will visualize {title} data." }
                        if page == "aspect" { p { "Learn more about " a { href: "https://acorn.ornl.gov/concepts/aspect/index.html", "ASPECT" } "." } }
                    }
                    img { class: "construction-gif", src: asset!("/assets/my_acorn_is_missing.gif"), alt: "My acorn is missing", width: "400", height: "237" }
                } },
                "portfolio" => rsx! { Portfolio { dashboard_only: false } },
                "dashboard" => rsx! { Portfolio { dashboard_only: true } },
                "geospatial" => rsx! { Geospatial {} },
                "install" => rsx! { Install {} },
                "skill" => rsx! { Skill { compact: false } },
                "settings" => rsx! { p { class: "lede", "Appearance and your local session." }
                    div { class: "card", h2 { "🎨 Appearance" }
                        div { class: "theme-selector-grid", role: "group", "aria-label": "Themes",
                            div { class: "theme-selector theme-selector-acorn", role: "group", "aria-label": "ACORN theme",
                                span { class: "theme-thumbnail", "aria-hidden": "true", span { class: "theme-thumbnail-sidebar" } span { class: "theme-thumbnail-content" } }
                                span { class: "theme-selector-name", "ACORN" }
                                div { class: "theme-choices", role: "group", "aria-label": "ACORN color mode",
                                    for mode in ["Light", "Dark", "System"] {
                                        button { "aria-label": mode, "data-theme-choice": mode.to_lowercase(), "aria-pressed": "false", onclick: move |_| { let mode = mode.to_lowercase(); document::eval(&format!("window.acornChrome?.theme.choose('{mode}')")); }, "{mode}" }
                                    }
                                }
                            }
                            button { class: "theme-selector theme-selector-uswds", "data-theme-choice": "uswds", "aria-pressed": "false", onclick: move |_| { document::eval("window.acornChrome?.theme.choose('uswds')"); },
                                span { class: "theme-thumbnail", "aria-hidden": "true", span { class: "theme-thumbnail-sidebar" } span { class: "theme-thumbnail-content" } }
                                span { class: "theme-selector-name", "USWDS" }
                                span { class: "theme-selector-note", "Light only" }
                            }
                        }
                        p { id: "theme-mode", class: "small muted", "Following your system preference." }
                        div { class: "actions",
                            button { onclick: move |_| async move {
                                collapsed.set(false);
                                let _ = js_sys::Reflect::set(&js_sys::global(), &"acornInitialSidebarCollapsed".into(), &false.into());
                                let cleared = document::eval("if (window.acornChrome) window.acornChrome.theme.reset(); try { sessionStorage.removeItem('acorn-sidebar-collapsed'); } catch {} const tourCleared = window.acornChrome?.tour.reset() ?? false; const layoutCleared = window.acornDashboardLayout?.clear() ?? false; return tourCleared && layoutCleared")
                                    .join::<bool>()
                                    .await
                                    .unwrap_or(false);
                                settings_message.set(if cleared {
                                    "Saved appearance, sidebar, tour, and dashboard settings cleared."
                                } else {
                                    "Some saved settings could not be cleared. Check browser storage and retry."
                                }.into());
                            }, "Clear saved settings" }
                        }
                        if !settings_message().is_empty() { p { role: "status", class: "status", "{settings_message}" } }
                    }
                    div { class: "card", id: "app-data", h2 { class: "section-heading", Icon { name: "folder-open" } "Your data" }
                        dl { class: "result-fields", dt { "Loaded now" } dd { "{workspace.read().files.len()} files · {workspace.read().activities().len()} validated activities" } if let Some(source_summary) = source_summary { dt { "Bucket source" } dd { "{source_summary}" } } }
                        p { class: "small muted", "Download includes the workspace currently open in this tab. It cannot include saved files that browser storage could not recover." }
                        div { class: "actions",
                            button { disabled: workspace.read().files.is_empty(), onclick: move |_| {
                                if let Ok(content) = serde_json::to_string(&workspace()) {
                                    if let Ok(argument) = serde_json::to_string(&content) {
                                        document::eval(&format!("window.acornDownload({argument}, 'acorn-workspace.json')"));
                                    }
                                }
                            }, "Download open workspace" }
                        }
                    }
                    div { class: "card browser-storage", id: "browser-storage", h2 { class: "section-heading", Icon { name: "database" } "Browser storage" }
                        if site_quota > 0 {
                            div { class: "storage-usage-label", span { "Site usage" } strong { "{storage_size(site_usage)} of about {storage_size(site_quota)}" } }
                            progress { value: "{site_usage}", max: "{site_quota}", "aria-label": "Site storage used" }
                            p { class: "small muted", "{site_percent:.1}% of the browser's estimated quota is in use. The browser sets this limit; ACORN cannot increase it." }
                        } else {
                            p { class: "small muted", "This browser did not provide a storage quota estimate." }
                        }
                        dl { class: "result-fields", dt { "ACORN saved workspace" } dd { "{saved_label}" } dt { "Latest save" } dd { if storage_status().1 { "Failed" } else { "No failure reported" } } }
                        p { class: "small muted", "Site usage includes this origin's other browser data and cached assets. It is an estimate, so it may be larger than the ACORN workspace shown above." }
                        if recovery_timed_out() { p { role: "alert", class: "error", "The saved workspace database is not responding. Reloading repeats the same read. Close and reopen the browser to try recovery. ACORN has not erased that database. Deletion is disabled until storage responds." } }
                        div { class: "actions",
                            button { onclick: move |_| async move {
                                storage_message.set("Refreshing browser estimate…".into());
                                let estimate = document::eval("return await window.acornWorkspaceStorage.status()").join::<Value>().await;
                                if let Ok(value) = estimate {
                                    storage_status.set(parse_storage_status(&value));
                                    storage_message.set("Browser estimate refreshed.".into());
                                } else {
                                    storage_message.set("Could not refresh the browser estimate.".into());
                                }
                            }, "Refresh estimate" }
                            button { class: "danger", disabled: recovery_timed_out(), onclick: move |_| async move {
                                let deleted = document::eval("window.acornWorkspace?.cancel(); window.acornRemote?.cancel(); let workspaceDeleted = false; let remoteDeleted = false; let layoutDeleted = false; let tourDeleted = false; try { workspaceDeleted = await window.acornWorkspaceStorage?.delete() ?? false; } catch {} try { remoteDeleted = window.acornRemote?.clearData() ?? false; } catch {} try { layoutDeleted = window.acornDashboardLayout?.clear() ?? false; } catch {} try { tourDeleted = window.acornChrome?.tour.reset() ?? false; } catch {} return [workspaceDeleted, remoteDeleted, layoutDeleted, tourDeleted]")
                                    .join::<(bool, bool, bool, bool)>()
                                    .await;
                                workspace.set(Workspace::default());
                                session.set(String::new());
                                map_pins.set(MapPins::default());
                                credential_error.set(String::new());
                                credential_message.set(String::new());
                                if let Ok((workspace_deleted, remote_deleted, layout_deleted, tour_deleted)) = deleted {
                                    if workspace_deleted { storage_status.set((0, false, 0, 0)); window_sync::notify_storage("acorn.workspace.changed.v1"); }
                                    if remote_deleted { recent_inputs.set((0, 0)); credential_status.set(String::new()); }
                                    if workspace_deleted && remote_deleted && layout_deleted && tour_deleted {
                                        storage_error.set(String::new());
                                        document::eval("window.acornChrome?.theme.reset(); window.acornChrome?.notifications.notify('All app data deleted.', 'success')");
                                    } else {
                                        storage_error.set("Open session cleared, but some saved browser data could not be deleted. Clear this site's data in your browser and retry.".into());
                                    }
                                } else {
                                    storage_error.set("Open session cleared, but saved browser data could not be verified. Clear this site's data in your browser and retry.".into());
                                }
                            }, "Delete all data" }
                        }
                        if !storage_message().is_empty() { p { role: "status", class: "status", "{storage_message}" } }
                        if !storage_error().is_empty() { p { role: "alert", class: "error", "{storage_error}" } }
                        if storage_status().1 && !recovery_timed_out() { p { role: "alert", class: "error", "Workspace saving failed. Your current session remains open, but the saved copy was not updated. Free browser storage or reload if another tab changed this workspace." } }
                        p { class: "small muted", "Deletes loaded and saved files, tool input, map pins, dashboard layout, tour completion, recent bucket inputs, and the repository token in this tab." } }
                    div { class: "card", h2 { "Repository access" }
                        p { class: "small muted", "One token is available in this tab for repositories on its provider and HTTPS host. Add or replace it beside the repository loader on Home." }
                        p { class: "small", if credential_status().is_empty() { "No repository token in this tab." } else { "Token •••••••• configured for {credential_status}." } }
                        div { class: "actions credential-actions", Link { to: Route::Home {}, "Add or replace token on Home →" }
                            button { class: "danger", disabled: credential_status().is_empty(), onclick: move |_| async move { if matches!(document::eval("return window.acornRemote?.clearCredential() ?? false").join::<bool>().await, Ok(true)) { credential_status.set(String::new()); credential_error.set(String::new()); credential_message.set("Token removed from this tab.".into()); } else { credential_error.set("Could not remove the repository token from this tab.".into()); credential_message.set(String::new()); } }, "Forget repository token" } }
                        if !credential_message().is_empty() { p { role: "status", class: "status", "{credential_message}" } }
                        if !credential_error().is_empty() { p { role: "alert", class: "error", "{credential_error}" } }
                        p { class: "small muted", "Tokens use session storage only in a secure browser context. They survive reloads in this tab, remain separate from saved workspace data, and expire when this tab session ends. Revoke a token at its provider if needed." }
                    }
                    div { class: "card", h2 { "Recent bucket inputs" }
                        p { class: "small muted", "This browser remembers repository URLs and branch, tag, or commit entries for suggestions. It does not store a token in this history." }
                        p { class: "small", "{recent_inputs().0} repository URLs · {recent_inputs().1} refs remembered" }
                        button { class: "danger", disabled: recent_inputs() == (0, 0), onclick: move |_| async move {
                            if let Ok(counts) = document::eval("window.acornRemote?.history.clear(); const history = window.acornRemote?.history.read(); return [history?.repositories.length ?? 0, history?.refs.length ?? 0];").join::<(usize, usize)>().await { recent_inputs.set(counts); }
                        }, "Clear recent bucket inputs" }
                    }
                    div { class: "card", h2 { "📦 Release" } p { title: RAD_TITLE, "{APP_NAME} · v{VERSION}" } Skill { compact: true }
                        div { class: "actions", Link { to: Route::Page { page: "skill".into() }, "ACORN Skill →" } Link { to: Route::Page { page: "install".into() }, "Install options →" } }
                    }
                },
                "about" => rsx! { p { class: "lede", "Accessible Content Optimization for Research Needs" } a { class: "card source-card", href: "https://code.ornl.gov/research-enablement/acorn", target: "_blank", rel: "noopener noreferrer", div { img { src: asset!("/assets/generated/acorn.svg"), alt: "", width: "32" } h2 { AppName {} } } span { "View source ↗" } } h2 { "ACORN Skill" } p { "ACORN Research Intelligence provides local research tools. The matching skill explains their workflows in your agent environment." } div { class: "actions", Link { to: Route::Page { page: "skill".into() }, "Explore the ACORN Skill →" } Link { to: Route::Page { page: "install".into() }, "Install options →" } } div { class: "about-principles", div { class: "card", h2 { "Local by default" } p { "Local files stay on this device. Remote files and buckets are fetched only when you choose Load. No uploads or telemetry." } } div { class: "card", h2 { "One workspace" } p { "ACORN Research Intelligence combines portable research tools with a matching agent skill. Desktop delivery is planned." } } } Technology {} div { class: "card ai-disclosure", a { href: "https://www.aihonestybadge.com/ai-assisted-badge", target: "_blank", rel: "noopener noreferrer", img { src: asset!("/assets/ai-assisted.svg"), alt: "AI Assisted", width: "113", height: "20" } } h2 { "AI-use disclosure" } p { "AI tools assisted with code, documentation and design under human direction. The maintainer reviews and tests the work. Review generated code before use." } div { class: "agent-links",
                    for (id, name, url) in [("codex", "Codex", "https://openai.com/codex/"), ("copilot", "GitHub Copilot", "https://github.com/features/copilot"), ("microsoft", "Microsoft 365 Copilot", "https://www.microsoft.com/microsoft-365/copilot")] {
                        a { href: url, target: "_blank", rel: "noopener noreferrer", img { src: format!("{}#{id}", asset!("/assets/agent-marks.svg")), alt: "", width: "20", height: "20" } "{name}" }
                    }
                } } p { class: "muted", "Developed at Oak Ridge National Laboratory · Open-source research tools" } },
                _ => rsx! { p { "This route is not available." } Link { to: Route::Home {}, "Return Home" } },
            }
        }
    }
}
fn initial_review_tab() -> Tool {
    let current = js_sys::Reflect::get(&js_sys::global(), &"location".into())
        .ok()
        .and_then(|location| js_sys::Reflect::get(&location, &"hash".into()).ok())
        .and_then(|hash| hash.as_string())
        .unwrap_or_default();
    let initial = js_sys::Reflect::get(&js_sys::global(), &"acornInitialReviewFragment".into())
        .ok()
        .and_then(|hash| hash.as_string())
        .unwrap_or_default();
    review_tab(if current.is_empty() { &initial } else { current.trim_start_matches('#') })
}
#[component]
fn Page(page: String) -> Element {
    if page == "home" {
        rsx! { Home {} }
    } else if matches!(page.as_str(), "metadata" | "review" | "research-activity" | "metadata-standard") {
        rsx! { ToolWorkspace { key: "{page}", initial_tool: Tool::Standard, review: true } }
    } else if let Some(tool) = Tool::iter().find(|tool| tool.slug() == page) {
        rsx! { ToolWorkspace { key: "{page}", initial_tool: tool, review: tool.is_review() } }
    } else {
        rsx! { InfoPage { page } }
    }
}
pub(crate) fn parse_storage_status(value: &Value) -> (usize, bool, usize, usize) {
    (
        value["bytes"].as_u64().unwrap_or(0) as usize,
        value["failed"] == true,
        value["usage"].as_u64().unwrap_or(0) as usize,
        value["quota"].as_u64().unwrap_or(0) as usize,
    )
}
fn storage_size(bytes: usize) -> String {
    const KIB: usize = 1024;
    const MIB: usize = KIB * KIB;
    const GIB: usize = MIB * KIB;
    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} B")
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
fn review_tab(fragment: &str) -> Tool {
    match fragment {
        | "fair" => Tool::Fair,
        | "readability" => Tool::Readability,
        | _ => Tool::Standard,
    }
}
fn sample_activity_path(workspace: &Workspace) -> Option<String> {
    workspace
        .activities()
        .into_iter()
        .find(|(file, _)| file.path.ends_with("/index.json"))
        .map(|(file, _)| file.path.clone())
}
fn identifier_choices() -> Vec<(String, &'static str, &'static str)> {
    once((String::new(), "AUTO", "Auto-detect identifier kind"))
        .chain(PID::iter().filter_map(|kind| {
            let label = match &kind {
                | PID::ARK => Some(("ARK", "Archival Resource Key")),
                | PID::Arxiv => Some(("ARXIV", "arXiv identifier")),
                | PID::DOI => Some(("DOI", "Digital Object Identifier")),
                | PID::Handle => Some(("HDL", "Handle identifier")),
                | PID::ISBN => Some(("ISBN", "International Standard Book Number")),
                | PID::ISNI => Some(("ISNI", "International Standard Name Identifier")),
                | PID::ORCID => Some(("ORCID", "Open Researcher and Contributor ID")),
                | PID::Patent => Some(("PAT", "Patent number")),
                | PID::RAID => Some(("RAiD", "Research Activity Identifier")),
                | PID::ROR => Some(("ROR", "Research Organization Registry")),
                | PID::RRID => Some(("RRID", "Research Resource Identifier")),
                | PID::SWHID => Some(("SWHID", "Software Heritage Identifier")),
                | _ => None,
            };
            label.map(|(abbreviation, title)| (kind.to_string(), abbreviation, title))
        }))
        .collect()
}
#[component]
fn ChoiceField(
    label: &'static str,
    value: String,
    choices: Vec<(String, &'static str, &'static str)>,
    width: usize,
    widths: Vec<usize>,
    onchange: EventHandler<String>,
) -> Element {
    let widths: Vec<_> = (0..choices.len()).map(|index| widths.get(index).copied().unwrap_or(width)).collect();
    let selected = choices.iter().position(|(option, _, _)| option == &value).unwrap_or_default();
    let offset = widths.iter().take(selected).sum::<usize>().saturating_add(selected);
    let selected_width = widths.get(selected).copied().unwrap_or(width);
    let columns = widths.iter().map(|width| format!("{width}px")).collect::<Vec<_>>().join(" ");
    use_effect(move || {
        document::eval(&format!(
            "requestAnimationFrame(() => {{ const bar = document.querySelector('.choice-scroll[data-choice-group=\"{label}\"]'); const selected = bar?.querySelector('[aria-pressed=\"true\"]'); if (selected) bar.scrollLeft = selected.offsetLeft - (bar.clientWidth - selected.clientWidth) / 2; }})"
        ));
    });
    rsx! {
        fieldset { class: "field vocabulary-field choice-field", legend { "{label}" }
            div { class: "choice-scroll", "data-choice-group": label,
                div { class: if label == "Format" { "choice-bar format-choice-bar" } else { "choice-bar" }, style: "grid-template-columns:{columns};--choice-offset:{offset}px;--choice-width:{selected_width}px", role: "group", "aria-label": "{label}",
                    for (option, abbreviation, title) in choices {
                        button { r#type: "button", title, "aria-pressed": value.as_str() == option.as_str(),
                            onclick: move |_| onchange.call(option.clone()), "{abbreviation}" }
                    }
                }
            }
        }
    }
}
#[component]
fn StandardField(label: &'static str, value: String, crosswalk: bool, onchange: EventHandler<String>) -> Element {
    let choices = STANDARD_CHOICES
        .iter()
        .copied()
        .filter(|(standard, _, _)| {
            !crosswalk
                || matches!(
                    standard,
                    Standard::Datacite | Standard::DublinCore | Standard::Raid | Standard::ResearchActivityData
                )
        })
        .map(|(standard, abbreviation, title)| (standard.to_string(), abbreviation, title))
        .collect();
    rsx! { ChoiceField { label, value, choices, width: 37, widths: Vec::new(), onchange } }
}
#[component]
fn ReadabilityControls(
    label: &'static str,
    metric: String,
    threshold: String,
    onmetric: EventHandler<String>,
    onthreshold: EventHandler<String>,
) -> Element {
    let selected_metric = ReadabilityType::from(metric.as_str());
    rsx! {
        div { class: "readability-controls",
            fieldset { class: "field vocabulary-field readability-metric-field", legend { "{label}" }
                div { class: "metric-choices", "data-selected": metric.as_str(), role: "group", "aria-label": "Readability metric",
                    for choice in ReadabilityType::iter() {
                        {let value = choice.to_string(); rsx! {
                            button { r#type: "button", title: choice.full_name(), "aria-pressed": metric == value,
                                onclick: move |_| onmetric.call(choice.to_string()), "{value.to_uppercase()}" }
                        }}
                    }
                }
            }
            label { if selected_metric == ReadabilityType::FRES { "Min" } else { "Max" }
                input { r#type: "number", step: "any", value: threshold, placeholder: "{selected_metric.maximum_allowed()}",
                    "aria-label": if selected_metric == ReadabilityType::FRES { "Minimum readability score" } else { "Maximum readability score" },
                    oninput: move |event| onthreshold.call(event.value()) }
            }
        }
    }
}
#[component]
fn Shell() -> Element {
    let mut collapsed = use_context::<Signal<bool>>();
    let mut drawer = use_signal(|| false);
    let initial_theme = js_sys::Reflect::get(&js_sys::global(), &"acornInitialTheme".into())
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| "light".into());
    let initially_dark = initial_theme == "dark";
    let initially_uswds = initial_theme == "uswds";
    let route = use_route::<Route>();
    let window_name = js_sys::Reflect::get(&js_sys::global(), &"name".into())
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_default();
    let window_type = WindowType::for_route(&route, &window_name);
    let current = match route {
        | Route::Home {} => "home".to_owned(),
        | Route::Page { page } => page,
    };
    let review_active = matches!(current.as_str(), "review" | "metadata" | "research-activity" | "metadata-standard")
        || Tool::iter().any(|tool| tool.is_review() && current == tool.slug());
    use_effect(use_reactive!(|current| {
        let _ = current;
        document::eval("window.acornChrome?.tour.cancel(); window.acornChrome?.theme.sync(); requestAnimationFrame(() => { const heading = document.querySelector('#content-main h1'); if (heading) { heading.tabIndex = -1; heading.focus(); } window.scrollTo(0, 0); })");
    }));
    if window_type.is_dashboard() || window_type.is_code_editor() {
        return rsx! {
            div { class: "workspace standalone-window", "data-theme": initial_theme,
                div { id: "notifications", class: "notifications", "aria-live": "polite", "aria-relevant": "additions" }
                div { id: "notification-template", class: "notification", "data-kind": "info", hidden: true,
                    span { class: "notification-icon notification-icon-success", Icon { name: "check" } }
                    span { class: "notification-icon notification-icon-warning", Icon { name: "alert" } }
                    span { class: "notification-icon notification-icon-info", Icon { name: "info" } }
                    span { class: "notification-text" }
                    button { "aria-label": "Dismiss notification", "Dismiss" } }
                main { id: "content-main", tabindex: "-1", Outlet::<Route> {} }
            }
        };
    }
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/vendor/driver.css") }
        div { class: format!("workspace {} {}", if collapsed() { "compact" } else { "" }, if drawer() { "drawer-open" } else { "" }), onkeydown: move |event| { if event.key() == Key::Escape && drawer() { drawer.set(false); document::eval("document.getElementById('nav-toggle')?.focus()"); } }, "data-theme": initial_theme,
            a { class: "skip", href: "#content-main", "Skip to content" }
            header { class: "mobile-bar", span { AppName {} } button { id: "nav-toggle", onclick: move |_| { drawer.toggle(); if drawer() { document::eval("requestAnimationFrame(() => document.querySelector('#workspace-navigation a')?.focus())"); } }, "aria-expanded": drawer(), "aria-controls": "workspace-navigation", "aria-label": "Toggle navigation", Icon { name: match drawer() { true => "panel", false => "panel-open" } } } }
            aside { id: "workspace-navigation", class: "sidebar", "aria-label": "Workspace navigation",
                div { class: "nav-controls",
                    Link { to: Route::Home {}, class: "sidebar-brand", title: "ACORN home", "aria-label": "ACORN home", onclick: move |_| { if drawer() { drawer.set(false); } },
                        img { class: "brand-icon-dark", src: asset!("/assets/generated/dark/app-icon.svg"), alt: "", width: "28", height: "28" }
                        img { class: "brand-icon-light", src: asset!("/assets/generated/light/app-icon.svg"), alt: "", width: "28", height: "28" }
                        img { class: "brand-icon-uswds", src: asset!("/assets/generated/uswds/app-icon.svg"), alt: "", width: "28", height: "28" }
                        span { class: "sidebar-brand-name", "aria-hidden": "true", "ACORN" }
                    }
                    button { class: "collapse", onclick: move |_| {
                        let next = !collapsed();
                        collapsed.set(next);
                        let _ = js_sys::Reflect::set(&js_sys::global(), &"acornInitialSidebarCollapsed".into(), &next.into());
                        document::eval(&format!("try {{ sessionStorage.setItem('acorn-sidebar-collapsed', '{next}'); }} catch {{}}"));
                    }, title: if collapsed() { "Expand sidebar" } else { "Collapse sidebar" }, "aria-label": if collapsed() { "Expand sidebar" } else { "Collapse sidebar" }, "aria-expanded": !collapsed(), "aria-controls": "workspace-navigation", Icon { name: match collapsed() { true => "panel-open", false => "panel" } } }
                }
                SidebarSession { onopen: move |_| { if drawer() { drawer.set(false); document::eval("requestAnimationFrame(() => document.querySelector('#content-main h1')?.focus())"); } } }
                nav { "aria-label": "Primary", onclick: move |_| { if drawer() { drawer.set(false); document::eval("requestAnimationFrame(() => document.querySelector('#content-main h1')?.focus())"); } },
                    p { class: "nav-heading", "WORKSPACE" }
                    Link { to: Route::Home {}, class: if current == "home" { "nav-item active" } else { "nav-item" }, title: "Home", "aria-label": "Home", "aria-current": if current == "home" { "page" } else { "false" }, Icon { name: "home" } span { class: "nav-label", "Home" } }
                    Link { to: Route::Page { page: "portfolio".into() }, class: if current == "portfolio" { "nav-item active" } else { "nav-item" }, title: "Portfolio", "aria-label": "Portfolio", "aria-current": if current == "portfolio" { "page" } else { "false" }, Icon { name: "portfolio" } span { class: "nav-label", "Portfolio" } }
                    Link { to: Route::Page { page: "review".into() }, class: if review_active { "nav-item active" } else { "nav-item" }, title: "Review", "aria-label": "Review", "aria-current": if review_active { "page" } else { "false" }, Icon { name: "filecheck" } span { class: "nav-label", "Review" } }
                    for (tool, label) in [(Tool::Discover, "Discover"), (Tool::Inspect, "Inspect"), (Tool::Crosswalk, "Convert")] {
                        Link { to: Route::Page { page: tool.slug().into() }, class: if current == tool.slug() { "nav-item active" } else { "nav-item" }, title: tool.to_string(), "aria-label": tool.to_string(), "aria-current": if current == tool.slug() { "page" } else { "false" }, Icon { name: tool.icon() } span { class: "nav-label", "{label}" } }
                    }
                    p { class: "nav-heading", "ANALYZE" }
                    for (slug, name, icon) in [("geospatial", "Geospatial", "globe"), ("aspect", "ASPECT", "chart"), ("s3a", "S3A", "database")] {
                        Link { to: Route::Page { page: slug.into() }, class: if current == slug { "nav-item active" } else { "nav-item" }, title: name, "aria-label": name, "aria-current": if current == slug { "page" } else { "false" }, Icon { name: icon } span { class: "nav-label", "{name}" } }
                    }
                    for tool in [Tool::Diff, Tool::BigO] {
                        Link { to: Route::Page { page: tool.slug().into() }, class: if current == tool.slug() { "nav-item active" } else { "nav-item" }, title: tool.to_string(), "aria-label": tool.to_string(), "aria-current": if current == tool.slug() { "page" } else { "false" }, Icon { name: tool.icon() } span { class: "nav-label", "{tool.to_string()}" } }
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
            div { id: "notifications", class: "notifications", "aria-live": "polite", "aria-relevant": "additions" }
            div { id: "notification-template", class: "notification", "data-kind": "info", hidden: true,
                span { class: "notification-icon notification-icon-success", Icon { name: "check" } }
                span { class: "notification-icon notification-icon-warning", Icon { name: "alert" } }
                span { class: "notification-icon notification-icon-info", Icon { name: "info" } }
                span { class: "notification-text" }
                button { "aria-label": "Dismiss notification", "Dismiss" } }
            main { id: "content-main", inert: drawer().then_some(""), tabindex: "-1", div { class: "page-toolbar", button { id: "theme-toggle", class: "theme-toggle", "aria-label": if initially_dark { "Switch to light theme (current: Dark)" } else if initially_uswds { "Switch to dark theme (current: USWDS)" } else { "Switch to dark theme (current: Light)" }, title: "Switch between dark and light themes", "aria-pressed": if initially_uswds { None } else { Some(initially_dark) }, onclick: move |_| { document::eval("window.acornChrome?.theme.toggle()"); }, span { class: "switch-track", span { class: "switch-thumb" } } span { class: "theme-icon", "aria-hidden": "true", span { class: "theme-moon", "☾" } span { class: "theme-sun", "☀" } } span { id: "theme-label", class: "nav-label", if initially_uswds { "USWDS" } else if initially_dark { "Dark" } else { "Light" } } } span { class: "theme-static", span { class: "theme-flag", "aria-hidden": "true", "🇺🇸" } a { href: "https://designsystem.digital.gov/", target: "_blank", rel: "noopener noreferrer", "USWDS" } } } Outlet::<Route> {} }
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
            a { class: "technology-card", href: "https://driverjs.com/", target: "_blank", rel: "noopener noreferrer",
                img { src: asset!("/assets/javascript-logo.svg"), alt: "", width: "32", height: "32" }
                span { strong { "Driver.js" } small { "Guided tour" } }
            }
            a { class: "technology-card", href: "https://webmachinelearning.github.io/webmcp/", target: "_blank", rel: "noopener noreferrer",
                Icon { name: "shield" }
                span { strong { "WebMCP" } small { "Optional agent tools" } }
            }
        }
        p { class: "small muted", "Technology names and marks identify their respective projects; no endorsement is implied. " a { href: asset!("/assets/NOTICE.md"), "Artwork sources and notices" } }
    }
}
#[component]
fn ToolWorkspace(initial_tool: Tool, review: bool) -> Element {
    const ACTIVITY_CHECKS: [(&str, &str, &str, &str); 4] = [
        ("schema", "Schema", "filecheck", "Schema"),
        ("prose", "Prose / vocabulary", "book", "Prose"),
        ("fair", "FAIR", "shield", "FAIR"),
        ("readability", "Readability", "chart", "Readability"),
    ];
    const FAIR_CHECKS: [(&str, &str, &str, &str); 4] = [
        ("findable", "Findable", "search", "F"),
        ("accessible", "Accessible", "link", "A"),
        ("interoperable", "Interoperable", "code", "I"),
        ("reusable", "Reusable", "shuffle", "R"),
    ];
    let session = use_context::<Signal<String>>();
    let mut workspace = use_context::<Signal<Workspace>>();
    let sync_review_url = review && initial_tool == Tool::Standard;
    let mut selected_tool = use_signal(|| if sync_review_url { initial_review_tab() } else { initial_tool });
    use_future(move || async move {
        if sync_review_url {
            let mut channel = window_sync::fragments();
            while let Ok(fragment) = channel.recv::<String>().await {
                selected_tool.set(review_tab(&fragment));
            }
        }
    });
    let tool = if review { selected_tool() } else { initial_tool };
    let sessions = use_context::<ToolSessions>();
    use_drop(move || sessions.clear_running());
    let signals = sessions.for_tool(tool);
    let mut request = signals.request;
    let mut result = signals.result;
    let mut running = signals.running;
    let mut last_request = signals.last_request;
    let mut seen_session = signals.seen_session;
    let mut drafts = signals.drafts;
    let mut editor_source = signals.editor_source;
    window_sync::use_tool_sync(tool, signals, session);
    use_hook(|| {
        document::eval("window.acornOperations?.warm()");
    });
    use_effect(move || {
        let current = request();
        if *last_request.peek() != current {
            last_request.set(current);
            if review {
                sessions.clear_review_results();
            } else {
                result.set(None);
            }
            document::eval("window.acornRemote?.file.cancel()");
        }
    });
    use_effect(move || {
        let content = session();
        if *seen_session.peek() != content {
            seen_session.set(content.clone());
            let snapshot = workspace();
            let source_is_current = editor_source
                .peek()
                .as_ref()
                .is_some_and(|source| Some(&source.path) == snapshot.selected.as_ref() && source.content == content);
            if source_is_current {
                return;
            }
            drafts.set(BTreeMap::new());
            let source = snapshot
                .files
                .iter()
                .find(|file| Some(&file.path) == snapshot.selected.as_ref() && file.content == content)
                .cloned();
            editor_source.set(source);
            let previous = request.peek().clone();
            request.set(Request { input: content, ..previous });
        }
    });
    let title = if review { "Review".to_owned() } else { tool.to_string() };
    let lede = if review {
        "Validate metadata, assess FAIR principles, or analyze readability."
    } else {
        tool.description()
    };
    let limit = tool.limit();
    let current = request.read();
    let skip_choices: &[(&str, &str, &str, &str)] = match tool {
        | Tool::Activity => &ACTIVITY_CHECKS,
        | Tool::Standard if current.standard == "research-activity-data" => &ACTIVITY_CHECKS,
        | Tool::Fair => &FAIR_CHECKS,
        | _ => &[],
    };
    let skipped_checks = skip_choices
        .iter()
        .filter(|(value, _, _, _)| current.skip.iter().any(|item| item == value))
        .count();
    let all_skipped = !skip_choices.is_empty() && skipped_checks == skip_choices.len();
    let input = current.input.clone();
    let snapshot = workspace();
    let selected_file = editor_source_file(&snapshot, editor_source().as_ref());
    let edited_file = selected_file.as_ref().filter(|file| file.content != input).cloned();
    let save_allowed = edited_file
        .as_ref()
        .is_some_and(|file| editor_save_allowed(&snapshot, file, &input, limit));
    let discard_file = edited_file.clone();
    let needs_content = input.trim().is_empty() && !(tool == Tool::Diff && current.mode == "unified");
    let language = match tool {
        | Tool::BigO => "expression".to_owned(),
        | Tool::Inspect => "identifier".to_owned(),
        | Tool::Standard | Tool::Crosswalk => current.format.clone(),
        | Tool::Activity | Tool::Fair => "json".to_owned(),
        | _ => "auto".to_owned(),
    };
    let diagnostics = result()
        .and_then(Result::ok)
        .and_then(|value| value.get("diagnostics").and_then(Value::as_array).cloned())
        .unwrap_or_default();
    rsx! {
        document::Title { "{title} · {APP_NAME}" }
        section { class: if review { "page tool-page review-page" } else { "page tool-page" }, "data-tool": tool.slug(),

            h1 { class: "page-title", Icon { name: if review { "filecheck" } else { tool.icon() } } "{title}" }
            p { class: "lede", "{lede}" }
            if review { div { class: "review-modes", role: "group", "aria-label": "Review type",
                for mode in [Tool::Standard, Tool::Fair, Tool::Readability] {
                    button { r#type: "button", "aria-pressed": tool == mode, onclick: move |_| {
                        selected_tool.set(mode);
                        if sync_review_url {
                            let fragment = match mode { Tool::Fair => "fair", Tool::Readability => "readability", Tool::Activity | Tool::Standard | Tool::Crosswalk | Tool::Diff | Tool::Discover | Tool::Inspect | Tool::BigO => "" };
                            document::eval(&format!("if (!window.acornUrl) await new Promise(resolve => window.addEventListener('acorn-url-ready', resolve, {{once:true}})); window.acornInitialReviewFragment = ''; window.acornUrl.setFragment('{fragment}')"));
                        }
                    }, Icon { name: mode.icon() } "{mode}" }
                }
            } }
            div { class: "tool-grid",
                div { class: "tool-input-column",
                LoadedFiles {
                    summary: "Select content".to_owned(),
                    selected_tool: Some(tool),
                    onload: move |text: String| { workspace.write().selected = None; editor_source.set(None); request.set(Request { input: text, ..request() }); result.set(None); },
                    onselect: move |file: SuppliedFile| {
                        let previous = request();
                        let content = drafts.read().get(&file.path).filter(|(source, _)| source == &file.content).map(|(_, draft)| draft.clone()).unwrap_or_else(|| file.content.clone());
                        let format = if matches!(tool, Tool::Standard | Tool::Crosswalk) {
                            file_format(&file.path).unwrap_or(&previous.format).to_owned()
                        } else {
                            previous.format.clone()
                        };
                        editor_source.set(Some(file));
                        request.set(Request { input: content, format, ..previous });
                        result.set(None);
                    },
                }
                section { class: "card editor",
                    div { class: "input-heading",
                        h2 { class: "section-heading", "Input" }
                        button { class: "sample-action", title: match tool { Tool::Activity | Tool::Standard => "Load intentionally flawed RAD with multiple validation issues", Tool::Diff => "Load a nested JSON example for semantic diff", Tool::BigO | Tool::Crosswalk | Tool::Discover | Tool::Fair | Tool::Inspect | Tool::Readability => "Load a detailed example" }, onclick: move |_| { workspace.write().selected = None; editor_source.set(None); request.set(operations::example_request(tool)); result.set(None); }, span { "aria-hidden": "true", "✨" } "Use example" }
                    }
                    if tool != Tool::Discover { details { class: "tool-settings",
                        summary { Icon { name: "settings" } "Settings"
                            if skipped_checks > 0 { span { class: "tool-settings-count", "{skipped_checks} skipped" } }
                        }
                        div { class: "options",
                        if matches!(tool, Tool::Standard | Tool::Crosswalk | Tool::Fair) { StandardField { label: "Standard", value: current.standard.clone(), crosswalk: tool == Tool::Crosswalk, onchange: move |v| request.set(Request { standard: v, ..request() }) } }
                        if matches!(tool, Tool::Standard | Tool::Crosswalk) {
                            ChoiceField { label: "Format", value: current.format.clone(), choices: FORMAT_CHOICES.into_iter().map(|(format, abbreviation, title)| (format.metadata_format_key().into(), abbreviation, title)).collect(), width: 42, widths: FORMAT_WIDTHS.to_vec(), onchange: move |v| request.set(Request { format: v, ..request() }) }
                        }
                        if !skip_choices.is_empty() {
                            div { class: "check-settings-row",
                            fieldset { class: "skip-checks", legend { "Skip checks" }
                                div { class: "skip-check-choices", role: "group", "aria-label": "Skip check types",
                                    button { r#type: "button", "aria-pressed": all_skipped, title: if all_skipped { "Run all checks" } else { "Skip all checks" },
                                        onclick: move |_| {
                                            let mut next = request().skip;
                                            next.retain(|item| !skip_choices.iter().any(|(value, _, _, _)| item == value));
                                            if !all_skipped { next.extend(skip_choices.iter().map(|(value, _, _, _)| (*value).to_owned())); }
                                            request.set(Request { skip: next, ..request() });
                                        }, Icon { name: "x" } span { "All" } }
                                    for (value, label, icon, compact) in skip_choices.iter().copied() {
                                        button { r#type: "button", "aria-label": label, title: if current.skip.iter().any(|item| item == value) { format!("Run {label} checks") } else { format!("Skip {label} checks") }, "aria-pressed": current.skip.iter().any(|item| item == value),
                                            onclick: move |_| {
                                                let mut next = request().skip;
                                                if next.iter().any(|item| item == value) { next.retain(|item| item != value); }
                                                else { next.push(value.to_owned()); }
                                                request.set(Request { skip: next, ..request() });
                                            }, Icon { name: icon } span { class: "skip-check-label", "{compact}" } }
                                    }
                                }
                            }
                            if matches!(tool, Tool::Activity | Tool::Standard) && current.standard == "research-activity-data" && !current.skip.iter().any(|item| item == "readability") {
                                ReadabilityControls { label: "Readability metric", metric: current.activity_metric.clone(), threshold: current.activity_max.clone(),
                                    onmetric: move |v| request.set(Request { activity_metric: v, ..request() }),
                                    onthreshold: move |v| request.set(Request { activity_max: v, ..request() }) }
                            }
                        }
                        }
                        if tool == Tool::Activity || (tool == Tool::Standard && current.standard == "research-activity-data") {
                            fieldset { class: "field vocabulary-field", legend { "Vocabulary matching" }
                                div { class: "vocabulary-choices", "data-selected": current.fuzzy.as_str(),
                                    for (value, label) in [("off", "Off"), ("relaxed", "Relaxed"), ("strict", "Strict")] {
                                        button { r#type: "button", "aria-pressed": current.fuzzy == value, title: presentation::label(value), onclick: move |_| request.set(Request { fuzzy: value.into(), ..request() }), "{label}" }
                                    }
                                }
                            }
                        }
                        if tool == Tool::Crosswalk { StandardField { label: "Destination", value: current.target.clone(), crosswalk: true, onchange: move |v| request.set(Request { target: v, ..request() }) } }
                        if tool == Tool::Readability {
                            ReadabilityControls { label: "Metric", metric: current.metric.clone(), threshold: current.threshold.clone(),
                                onmetric: move |v| request.set(Request { metric: v, ..request() }),
                                onthreshold: move |v| request.set(Request { threshold: v, ..request() }) }
                        }
                        if tool == Tool::Diff {
                            ChoiceField { label: "Diff format", value: current.mode.clone(), choices: vec![("unified".into(), "Unified", "Unified diff"), ("json-patch".into(), "JSON Patch", "JSON Patch")], width: 76, widths: Vec::new(), onchange: move |v| request.set(Request { mode: v, ..request() }) }
                        }
                        if tool == Tool::Inspect {
                            ChoiceField { label: "Identifier", value: current.kind.clone(), choices: identifier_choices(), width: 40, widths: Vec::new(), onchange: move |v| request.set(Request { kind: v, ..request() }) }
                        }
                        if tool == Tool::BigO { div { class: "field", span { "Symbol assumptions (JSON)" } CodeEditor { id: "assumptions", label: "Symbol assumptions (JSON)", language: "json", rows: 2, value: current.assumptions.clone(), oninput: move |value| request.set(Request { assumptions: value, ..request() }) } } }
                        if tool == Tool::Crosswalk {
                            section { class: "import-seeds", "aria-labelledby": "import-seeds-title",
                                header { class: "import-seeds-heading",
                                    h3 { id: "import-seeds-title", "Import seeds" }
                                    span { "When required" }
                                }
                                div { class: "field", span { "Contact JSON" } CodeEditor { id: "contact", label: "Contact JSON", language: "json", rows: 3, value: current.contact.clone(), oninput: move |value| request.set(Request { contact: value, ..request() }) } }
                                label { class: "field", "Local identifier" input { value: current.local_identifier.clone(), oninput: move |e| request.set(Request { local_identifier: e.value(), ..request() }) } }
                            }
                        }
                        }
                    }
                    }
                    if let Some(file) = edited_file { div { class: "editor-draft", role: "status",
                        span { "Edited copy of {file.path}. Changes are kept here until you save or discard them." }
                        button { r#type: "button", disabled: !save_allowed, onclick: move |_| {
                            let mut next = workspace();
                            let content = request().input;
                            if let Some(file) = editor_source_file(&next, editor_source().as_ref())
                                .filter(|file| file.content != content && editor_save_allowed(&next, file, &content, limit))
                            {
                                if let Some(source) = next.files.iter_mut().find(|source| source.path == file.path) {
                                    source.content = content.clone();
                                    next.validations.remove(&file.path);
                                    workspace.set(next);
                                    editor_source.set(Some(SuppliedFile { path: file.path.clone(), content }));
                                    drafts.write().remove(&file.path);
                                }
                            }
                        }, "Save to workspace" }
                        if !save_allowed { small { "Reduce this input to fit the editor or workspace limit before saving." } }
                        button { r#type: "button", onclick: move |_| {
                            if let Some(file) = &discard_file {
                                drafts.write().remove(&file.path);
                                request.set(Request { input: file.content.clone(), ..request() });
                                result.set(None);
                            }
                        }, "Discard edits" }
                    } }
                    div { class: if matches!(tool, Tool::Diff | Tool::BigO) { "editor-pair" } else { "" },
                        div { class: "field", span { if tool == Tool::BigO { "Left expression" } else if tool == Tool::Diff { "Before" } else { "Content" } }
                            CodeEditor { id: "content", label: if tool == Tool::BigO { "Left expression" } else if tool == Tool::Diff { "Before" } else { "Content" }, language: language.clone(), rows: if tool == Tool::BigO { 3 } else { 7 }, value: input.clone(), diagnostics, fullscreen: true, run_action: true, run_disabled: running(), run_error: if running() { None } else { result().and_then(Result::err) }, oninput: move |value: String| {
                                if let Some(file) = &selected_file {
                                    if value == file.content { drafts.write().remove(&file.path); }
                                    else { drafts.write().insert(file.path.clone(), (file.content.clone(), value.clone())); }
                                }
                                request.set(Request { input: value, ..request() }); result.set(None);
                            } }
                            small { class: if input.len() > limit { "error" } else { "muted" }, "{input.len()} / {limit} UTF-8 bytes" }
                        }
                        if matches!(tool, Tool::Diff | Tool::BigO) {
                            div { class: "field", span { if tool == Tool::BigO { "Right expression" } else { "After" } }
                                CodeEditor { id: "after", label: if tool == Tool::BigO { "Right expression" } else { "After" }, language: language.clone(), rows: if tool == Tool::BigO { 3 } else { 7 }, value: current.after.clone(), oninput: move |value| { request.set(Request { after: value, ..request() }); result.set(None); } }
                                small { class: if current.after.len() > limit { "error" } else { "muted" }, "{current.after.len()} / {limit} UTF-8 bytes" }
                            }
                        }
                    }
                    if tool == Tool::Readability { p { class: "small muted", "{input.split_whitespace().count()} words · {input.len()} UTF-8 bytes" } }
                    div { class: "actions run-actions", button { id: "tool-run", class: "primary", disabled: running(), "aria-busy": running(), onclick: move |_| async move {
                        running.set(true);
                        let submitted = request.peek().clone();
                        let output = execute(tool, submitted.clone()).await.map(|mut value| {
                            crate::location::annotate(&mut value, &submitted.input, &submitted.format);
                            value
                        });
                        running.set(false);
                        if tool == Tool::Activity || (tool == Tool::Standard && submitted.standard == "research-activity-data" && submitted.format == "json") {
                            let mut next = workspace();
                            if let Some(file) = next.files.iter().find(|file| Some(&file.path) == next.selected.as_ref() && file.content == submitted.input) {
                                if let Ok(value) = &output { next.validations.insert(file.path.clone(), value.clone()); }
                            }
                            workspace.set(next);
                        }
                        let celebrate = matches!(tool, Tool::Activity | Tool::Standard) && output.as_ref().is_ok_and(|value| value["valid"] == true);
                        // Edits made while the operation ran supersede its result.
                        if *request.peek() == submitted {
                            result.set(Some(output)); document::eval(&format!("requestAnimationFrame(() => {{ const fullscreen = document.querySelector('.code-editor.is-fullscreen'); const heading = document.querySelector('[data-tool=\"{}\"] #result-heading'); if (fullscreen) fullscreen.querySelector('.editor-run-action')?.focus({{preventScroll:true}}); else heading?.focus(); if (heading && {celebrate}) window.acornCelebrate?.(); }})", tool.slug()));
                        }
                    }, Icon { name: tool.icon() } if running() { "Running…" } else { "{tool.action_label()}" } } button { onclick: move |_| { workspace.write().selected = None; editor_source.set(None); request.set(Request::default()); result.set(None); }, "Clear" } }
                    p { class: "muted small", "Runs locally. No identifiers are resolved and no content is uploaded." }
                }
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
                button { r#type: "submit", class: "source-submit", disabled: loading() || url().trim().is_empty(), "aria-busy": loading(), if loading() { span { class: "source-spinner", "aria-hidden": "true" } "Loading…" } else { Icon { name: "download" } "Load URL" } }
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
/// Validate Research Activity files one at a time through [`execute`].
pub(crate) async fn validate_files(files: Vec<SuppliedFile>) -> Vec<(String, Result<Value, String>)> {
    let mut results = Vec::with_capacity(files.len());
    for file in files {
        let result = execute(
            Tool::Activity,
            Request {
                input: file.content,
                ..Request::default()
            },
        )
        .await;
        results.push((file.path, result));
    }
    results
}
