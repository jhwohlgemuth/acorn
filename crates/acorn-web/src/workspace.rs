use crate::dashboard::{ChartSegment, DashboardTile, DonutTile, GridCell, MetricTile, TileLayout, PORTFOLIO_TILES};
use crate::details::{self, Detail};
use crate::script;
use crate::search_ui::SearchField;
use crate::ui::{validate_files, FileInput, Icon, Route, TemplatePicker, UrlFileInput};
use crate::window_sync;
use acorn_core::util::MimeType;
use acorn_web::{
    operations::Tool,
    presentation::text,
    research_index::{CatalogSort, Project, ResearchIndex},
    search::SearchPattern,
    session::{self, Selection, SuppliedFile, Workspace, WorkspaceSource},
};
use alloc::collections::{BTreeMap, BTreeSet};
use dioxus::prelude::*;
use serde_json::{json, Value};
use strum::IntoEnumIterator;

const FILE_KIND_FILTERS: [(Option<MimeType>, &str, &str, &str); 6] = [
    (Some(MimeType::Json), "json", "JSON", "JSON files"),
    (Some(MimeType::Yaml), "yaml", "YAML", "YAML files"),
    (Some(MimeType::Markdown), "markdown", "Markdown", "Markdown files"),
    (Some(MimeType::Text), "text", "Text", "Text files"),
    (None, "image", "Images", "Image files"),
    (None, "other", "Other", "Other files"),
];
const IDENTIFIER_COLOR_COUNT: usize = 8;
pub(crate) const TAG_COLOR_COUNT: usize = 8;
#[derive(Clone, Default, serde::Deserialize)]
pub(crate) struct BucketHistory {
    repositories: Vec<String>,
    refs: Vec<String>,
}
#[component]
fn ActivityStatus(status: String) -> Element {
    let (icon, label) = match status.as_str() {
        | "active" => ("play", "Active"),
        | "canceled" => ("x", "Canceled"),
        | "completed" => ("check", "Completed"),
        | "paused" => ("pause", "Paused"),
        | _ => ("info", status.as_str()),
    };
    rsx! { span { class: "catalog-activity-status", "data-status": status.as_str(), Icon { name: icon } "{label}" } }
}
pub(crate) fn file_format(path: &str) -> Option<&'static str> {
    match MimeType::from(path) {
        | MimeType::Cff => Some("cff"),
        | MimeType::Json => Some("json"),
        | MimeType::Jsonc => Some("jsonc"),
        | MimeType::LdJson => Some("json-ld"),
        | MimeType::Markdown => Some("markdown"),
        | MimeType::RdfXml => Some("rdf-xml"),
        | MimeType::Text => Some("text"),
        | MimeType::Turtle => Some("turtle"),
        | MimeType::Yaml => Some("yaml"),
        | MimeType::Zon => Some("zon"),
        | _ => None,
    }
}
fn file_kind(file: &SuppliedFile) -> &'static str {
    let mime = MimeType::from(file.path.as_str());
    FILE_KIND_FILTERS
        .iter()
        .find(|(filter, _, _, _)| filter.as_ref() == Some(&mime))
        .map(|(_, value, _, _)| *value)
        .unwrap_or(if mime.is_image() { "image" } else { "other" })
}
pub(crate) fn file_loadable(file: &SuppliedFile, tool: Tool) -> bool {
    let mime = MimeType::from(file.path.as_str());
    !mime.is_image()
        && file.content.len() <= tool.limit()
        && match tool {
            | Tool::Activity | Tool::Fair => matches!(mime, MimeType::Json),
            | Tool::Format => matches!(
                mime,
                MimeType::Cff
                    | MimeType::Json
                    | MimeType::Jsonc
                    | MimeType::LdJson
                    | MimeType::Markdown
                    | MimeType::RdfXml
                    | MimeType::Turtle
                    | MimeType::Yaml
                    | MimeType::Zon
            ),
            | Tool::Standard | Tool::Crosswalk => file_format(&file.path).is_some(),
            | _ => true,
        }
}
#[component]
fn FileTree(paths: Vec<String>, prefix: String, descending: bool, tool: Option<Tool>, onselect: Option<EventHandler<SuppliedFile>>) -> Element {
    let mut workspace = use_context::<Signal<Workspace>>();
    let mut input = use_context::<Signal<String>>();
    let drawer = use_context::<Signal<Option<Detail>>>();
    let children = tree_children(&paths, &prefix, descending);
    rsx! { ul { class: "file-tree", for (is_file, name) in children {
        {let path = format!("{prefix}{name}");
        let thumbnail = workspace.read().image_file(&path);
        let loadable = workspace.read().files.iter().find(|file| file.path == path).is_some_and(|file| tool.is_none_or(|tool| file_loadable(file, tool)));
        rsx! { li {
            if is_file { button { class: "file-leaf", disabled: !loadable, title: if loadable { "" } else { "This file cannot be loaded into this page" }, "aria-pressed": workspace.read().selected.as_deref() == Some(path.as_str()), onclick: move |_| {
                if loadable {
                    let snapshot = workspace();
                    if let Some(file) = snapshot.files.iter().find(|file| file.path == path) {
                        if tool.is_some() {
                            if let Some(onselect) = onselect { onselect.call(file.clone()); } else { input.set(file.content.clone()); }
                        } else { details::open(drawer, Detail::file(file, &snapshot)); }
                        if !MimeType::from(file.path.as_str()).is_image() { workspace.write().selected = Some(path.clone()); }
                    }
                }
            }, if let Some(thumbnail) = thumbnail { img { class: "file-thumb", src: thumbnail, alt: "" } } else { Icon { name: "file" } } "{name}" } }
            else { details { summary { Icon { name: "folder-open" } "{name}" } FileTree { paths: paths.clone(), prefix: format!("{path}/"), descending, tool, onselect } } }
        }}}
    } } }
}
#[component]
fn IdentifierDonut(counts: BTreeMap<String, usize>) -> Element {
    let segments = counts
        .into_iter()
        .enumerate()
        .map(|(index, (label, count))| ChartSegment {
            color: format!("var(--identifier-{})", index % IDENTIFIER_COLOR_COUNT),
            label,
            count,
        })
        .collect();
    rsx! { DonutTile { tile_id: "identifiers", class: "identifier-donut", title: "Identifiers", center_label: "unique IDs", empty_message: "No identifiers in matching activities", description: Some("Unique values across matching activities."), segments } }
}
#[component]
pub(crate) fn LoadedFiles(
    summary: String,
    #[props(default)] initial_url: String,
    #[props(default)] onload: Option<EventHandler<String>>,
    #[props(default)] onselect: Option<EventHandler<SuppliedFile>>,
    #[props(default)] ontemplate: Option<EventHandler<(String, MimeType)>>,
    #[props(default)] selected_tool: Option<Tool>,
) -> Element {
    let workspace = use_context::<Signal<Workspace>>();
    let route = use_route::<Route>();
    let tool = selected_tool.or_else(|| route_tool(&route));
    let query = use_signal(String::new);
    let regex = use_signal(|| false);
    let case_sensitive = use_signal(|| false);
    let mut kind = use_signal(|| "all".to_owned());
    let mut selected_kinds = use_signal(|| {
        FILE_KIND_FILTERS
            .iter()
            .map(|(_, value, _, _)| (*value).to_owned())
            .collect::<BTreeSet<_>>()
    });
    let mut descending = use_signal(|| false);
    let snapshot = workspace();
    let pattern = SearchPattern::new(&query(), regex(), case_sensitive());
    let selected = selected_kinds();
    let all_selected = selected.len() == FILE_KIND_FILTERS.len();
    let paths = pattern
        .as_ref()
        .map(|pattern| visible_file_paths(&snapshot.files, pattern, &kind(), tool, (tool.is_none()).then_some(&selected)))
        .unwrap_or_default();
    rsx! { details { class: "loaded-tree", open: !initial_url.is_empty(), summary {
        span { class: "loaded-tree-icon", Icon { name: "folder-open" } }
        span { class: "loaded-tree-label", "{summary}" }
    }
        if let (Some(tool), Some(onload), Some(ontemplate)) = (tool, onload, ontemplate) { FileInput { onload, limit: tool.limit(), tool, ontemplate, initial_url: initial_url.clone() } }
        if !snapshot.files.is_empty() {
            div { class: "loaded-files-controls result-controls",
                SearchField { label: "Find a file", placeholder: "Search paths…", query, regex, case_sensitive, error: pattern.as_ref().err().cloned() }
                div { class: "result-filters", role: "group", "aria-label": "Show files",
                    span { class: "muted", "Show" }
                    if tool.is_some() {
                        for (value, label, title) in [("all", "All", "All files"), ("loadable", "Available", "Available here"), ("json", "JSON", "JSON files"), ("yaml", "YAML", "YAML files"), ("markdown", "Markdown", "Markdown files"), ("text", "Text", "Text files"), ("image", "Images", "Image files")] {
                            button { r#type: "button", title, "aria-pressed": kind() == value, onclick: move |_| kind.set(value.to_owned()), "{label}" }
                        }
                    } else {
                        button { r#type: "button", title: "Toggle all file types", "aria-pressed": all_selected,
                            onclick: move |_| selected_kinds.set(if all_selected { BTreeSet::new() } else { FILE_KIND_FILTERS.iter().map(|(_, value, _, _)| (*value).to_owned()).collect() }), "All"
                        }
                        for (_, value, label, title) in FILE_KIND_FILTERS {
                            button { r#type: "button", title, "aria-pressed": selected.contains(value),
                                onclick: move |_| {
                                    let mut next = selected_kinds();
                                    if !next.insert(value.to_owned()) { next.remove(value); }
                                    selected_kinds.set(next);
                                }, "{label}"
                            }
                        }
                    }
                }
                div { class: "result-filters loaded-files-sort", role: "group", "aria-label": "Sort files",
                    span { class: "muted", "Sort" }
                    for (reverse, label, title) in [(false, "A–Z", "Name A–Z"), (true, "Z–A", "Name Z–A")] {
                        button { r#type: "button", title, "aria-pressed": descending() == reverse, onclick: move |_| descending.set(reverse), "{label}" }
                    }
                }
            }
            p { class: "loaded-files-count small muted", role: "status", "{paths.len()} of {snapshot.files.len()} files shown" }
            if paths.is_empty() { p { class: "loaded-files-empty small muted", "No matching files." } }
            else { FileTree { paths, prefix: String::new(), descending: descending(), tool, onselect } }
        }
    } }
}
#[component]
pub(crate) fn Loader(ontemplate: EventHandler<(String, MimeType)>) -> Element {
    let mut workspace = use_context::<Signal<Workspace>>();
    let mut input = use_context::<Signal<String>>();
    let mut message = use_signal(String::new);
    let mut choose = use_signal(|| false);
    let mut templates_open = use_signal(|| false);
    let mut url_open = use_signal(|| false);
    let mut close_url = move || {
        url_open.set(false);
        script::focus_home_source_trigger();
    };
    let mut remote_url = use_signal(String::new);
    let mut remote_ref = use_signal(String::new);
    let mut remote_error = use_signal(String::new);
    let mut remote_success = use_signal(String::new);
    let mut remote_loading = use_signal(|| false);
    let mut credential_status = use_signal(String::new);
    let mut credential_message = use_signal(String::new);
    let mut credential_error = use_signal(String::new);
    let mut history = use_signal(BucketHistory::default);
    use_future(move || async move {
        if let Ok(saved) = script::read_remote_history().await {
            history.set(saved);
        }
    });
    use_future(move || async move {
        if let Ok(host) = script::credential_status().await {
            credential_status.set(host);
        }
    });
    use_future(move || async move {
        let mut channel = script::workspace_picker_channel();
        while let Ok(batch) = channel.recv::<Selection>().await {
            remote_success.set(String::new());
            if batch.files.is_empty() {
                message.set(format!(
                    "No files loaded; {} entries skipped or unreadable. Your previous session is unchanged.",
                    batch.skipped
                ));
            } else {
                message.set("Loading and validating research activities…".into());
                let next = workspace().replace(batch);
                let files = next.files.clone();
                input.set(next.content());
                workspace.set(next);
                let validations = validate_files(
                    files
                        .iter()
                        .filter(|file| matches!(MimeType::from(file.path.as_str()), MimeType::Json))
                        .cloned()
                        .collect(),
                )
                .await
                .into_iter()
                .map(|(path, result)| (path, result.unwrap_or_else(|message| json!({"valid":false,"error":message}))))
                .collect();
                if workspace.peek().files == files {
                    workspace.write().validations = validations;
                    message.set(format!(
                        "{} files loaded; {} entries skipped. Explore activities in Portfolio.",
                        files.len(),
                        workspace.peek().skipped
                    ));
                }
            }
        }
    });
    use_drop(move || {
        script::disconnect_workspace_picker();
    });
    rsx! {
        p { class: "load-caption muted", "Open research activity data" }
        div { id: "local-entry", class: "load-tiles",
            div { class: "load-tile",
                button { id: "home-source-trigger", class: "tile-action", "aria-expanded": choose(), "aria-controls": "loader-choices", onclick: move |_| { url_open.set(false); templates_open.set(false); choose.toggle(); }, span { class: "tile-icon folder-icon", "aria-hidden": "true", Icon { name: "folder-open" } } strong { "Open file or folder" } small { "Files, folders or URL" } }
                if choose() { div { id: "loader-choices", class: "actions", "aria-label": "Choose content source",
                    button { onclick: move |_| { choose.set(false); script::pick_workspace_file(); }, Icon { name: "file" } "Files" }
                    button { onclick: move |_| { choose.set(false); script::pick_workspace_folder(); }, Icon { name: "folder-open" } "Folder" }
                    button { onclick: move |_| { choose.set(false); url_open.set(true); script::focus_url_loader(); }, Icon { name: "link" } "URL" }
                } }
            }
            div { class: "load-tile", button { r#type: "button", class: "tile-action", "data-template-trigger": "true", "aria-expanded": templates_open(), onclick: move |_| { choose.set(false); url_open.set(false); templates_open.toggle(); }, span { class: "tile-icon", "aria-hidden": "true", Icon { name: "filecheck" } } strong { "Start from template" } small { "JSON or YAML" } } }
            div { class: "load-tile", "data-workspace-drop": "true", button { class: "tile-action", onclick: move |_| { templates_open.set(false); script::pick_workspace_file(); }, span { class: "tile-icon upload-icon", "aria-hidden": "true", Icon { name: "upload" } } strong { "Drop content" } small { "Drop files or folders" } } }
            if templates_open() { div { class: "home-template-picker", TemplatePicker { onselect: move |template| { templates_open.set(false); ontemplate.call(template); } } } }
        }
        if url_open() { div { class: "source-tray home-source-tray", role: "region", "aria-label": "Load content options",
            onkeydown: move |event| { if event.key() == Key::Escape { event.prevent_default(); event.stop_propagation(); close_url(); } },
            div { class: "source-tray-heading", span { Icon { name: "link" } "Bring a link" }
                button { r#type: "button", class: "source-close", "aria-label": "Close content loader", onclick: move |_| close_url(), "×" }
            }
            UrlFileInput { limit: session::FILE_LIMIT, workspace_file: true, onload: move |file: SuppliedFile| {
                remote_success.set(String::new());
                let next = workspace().replace(Selection { files: vec![file], skipped: 0 });
                input.set(next.content()); workspace.set(next); message.set("File loaded. Choose Portfolio to validate and explore activities.".into()); close_url();
            } }
        } }
        p { class: "load-limits small muted", "CFF, JSON, JSON-LD, RDF/XML, Turtle, YAML, Markdown, text, SVG, PNG, JPEG or WebP · 2 MiB per file · 500 files · 32 MiB stored total" }
        details { class: "remote-entry", summary {
            "Open a " span { class: "repository-name repository-github", img { src: asset!("/assets/generated/github.svg"), alt: "", width: "16", height: "16" } "GitHub" }
            " or " span { class: "repository-name", img { src: asset!("/assets/generated/gitlab.svg"), alt: "", width: "16", height: "16" } "GitLab" } " bucket"
        }
        form { class: "remote-loader", onsubmit: move |event| async move {
            event.prevent_default();
            if remote_loading() { return; }
            remote_loading.set(true); remote_error.set(String::new()); remote_success.set(String::new()); message.set(String::new());
            let result = script::load_remote_bucket(&remote_url(), &remote_ref()).await;
            match result {
                Ok(value) => {
                    if let Ok(saved) = serde_json::from_value::<BucketHistory>(value.get("history").cloned().unwrap_or(Value::Null)) { history.set(saved); }
                    if let Some(error) = value.get("error").and_then(Value::as_str) { remote_error.set(error.to_owned()); }
                    else {
                        let mut loaded = value.get("ok").cloned().unwrap_or(Value::Null);
                        let source = loaded.get("source").cloned().and_then(|value| serde_json::from_value::<WorkspaceSource>(value).ok());
                        let unavailable = loaded.get("unavailable").and_then(Value::as_u64).unwrap_or(0);
                        let reasons = loaded.get("reasons").cloned().unwrap_or(Value::Null);
                        if let Some(object) = loaded.as_object_mut() { object.remove("source"); object.remove("unavailable"); object.remove("reasons"); }
                        match serde_json::from_value::<Selection>(loaded) {
                            Ok(selection) if selection.files.is_empty() => remote_error.set("No supported files were found in this bucket. Your previous session is unchanged.".into()),
                            Ok(selection) => {
                                let mut next = workspace().replace(selection);
                                next.source = source;
                                next.validations = validate_files(next.files.iter().filter(|file| matches!(MimeType::from(file.path.as_str()), MimeType::Json)).cloned().collect())
                                    .await
                                    .into_iter()
                                    .map(|(path, result)| (path, result.unwrap_or_else(|message| json!({"valid":false,"error":message}))))
                                    .collect();
                                let origin = next.source.as_ref().map(|source| format!(" from {}/{}", source.host, source.project)).unwrap_or_default();
                                let noun = if next.files.len() == 1 { "file" } else { "files" };
                                let failed = if unavailable == 0 { String::new() } else { format!(" ({unavailable} could not be downloaded)") };
                                let details = [
                                    ("unsupported", "unsupported paths or formats"),
                                    ("fileLimit", "over the file limit"),
                                    ("invalidContent", "invalid content"),
                                    ("unavailable", "download failures"),
                                    ("totalLimit", "over the total limit"),
                                ]
                                .into_iter()
                                .filter_map(|(key, label)| reasons.get(key).and_then(Value::as_u64).filter(|count| *count > 0).map(|count| format!("{count} {label}")))
                                .collect::<Vec<_>>()
                                .join(", ");
                                let explanation = if details.is_empty() { failed } else { format!(" ({details})") };
                                remote_success.set(format!("Bucket loaded: {} {noun}{origin}; {} entries skipped{explanation}.", next.files.len(), next.skipped));
                                input.set(next.content());
                                workspace.set(next);
                            },
                            Err(_) => remote_error.set("The repository returned an invalid file selection. Your previous session is unchanged.".into()),
                        }
                    }
                },
                Err(_) => remote_error.set("Could not load this repository. Your previous session is unchanged.".into()),
            }
            remote_loading.set(false);
        },
            div { class: "repository-credential", h3 { "Private repository access" }
                p { class: "small muted", "Save one token for repository loads in this tab. It is sent to the HTTPS host of each GitHub or GitLab repository URL you load; redirects to another host are rejected. GitLab needs read_api; GitHub needs Contents read." }
                p { class: "small", if credential_status().is_empty() { "No token saved for this tab." } else { "Token •••••••• configured for {credential_status}." } }
                label { class: "field", "Access token" input { id: "repository-token", r#type: "password", autocomplete: "off", spellcheck: "false" } }
                p { class: "small muted", "The token survives reloads, stays separate from workspace data, and is cleared when the tab session ends." }
                div { class: "actions credential-actions", button { r#type: "button", class: "primary", onclick: move |_| async move {
                    let result = script::set_workspace_credential().await;
                    match result {
                        Ok(value) => if let Some(status) = value.get("status").and_then(Value::as_str) { credential_status.set(status.to_owned()); credential_message.set("Token saved for repository loads in this tab.".into()); credential_error.set(String::new()); } else { credential_error.set(value.get("error").and_then(Value::as_str).unwrap_or("Enter a valid access token.").to_owned()); credential_message.set(String::new()); },
                        Err(_) => { credential_error.set("Could not save the repository token. Try again.".into()); credential_message.set(String::new()); },
                    }
                }, "Save token" }
                button { r#type: "button", class: "danger", disabled: credential_status().is_empty(), onclick: move |_| async move { if matches!(script::clear_workspace_credential().await, Ok(true)) { credential_status.set(String::new()); credential_message.set("Token removed from this tab.".into()); credential_error.set(String::new()); } else { credential_error.set("Could not remove the repository token from this tab.".into()); credential_message.set(String::new()); } }, "Remove token" } }
                if !credential_message().is_empty() { p { role: "status", class: "status", "{credential_message}" } }
                if !credential_error().is_empty() { p { role: "alert", class: "error", "{credential_error}" } }
            }
            label { class: "field", "Repository or URL" input { r#type: "text", required: true, disabled: remote_loading(), value: remote_url(), oninput: move |event| remote_url.set(event.value()), list: "bucket-repositories", autocomplete: "off", placeholder: "owner/repo or https://gitlab.example.org/group/project", spellcheck: "false" } }
            datalist { id: "bucket-repositories", for repository in &history.read().repositories { option { value: repository.clone() } } }
            datalist { id: "bucket-refs", for reference in &history.read().refs { option { value: reference.clone() } } }
            div { class: "remote-actions", label { class: "field", "Branch / tag / commit (optional)" input { disabled: remote_loading(), value: remote_ref(), oninput: move |event| remote_ref.set(event.value()), list: "bucket-refs", autocomplete: "off", placeholder: "Repository default", spellcheck: "false" } }
                button { r#type: "submit", disabled: remote_loading() || remote_url().trim().is_empty(), "aria-busy": remote_loading(), Icon { name: "download" } if remote_loading() { "Loading…" } else { "Load bucket" } }
                if remote_loading() { button { r#type: "button", onclick: move |_| { script::cancel_remote_load(); }, "Cancel" } }
            }
            p { class: "small muted", "Public repositories, folder or file URLs. For branches containing /, enter the full branch above. Loading replaces this session after a successful download." }
            if !remote_success().is_empty() { div { class: "status bucket-success", role: "status", span { "{remote_success}" } Link { to: Route::Page { page: "portfolio".into() }, "Explore Portfolio →" } } }
            div { class: "actions small muted", span { "Recent inputs are remembered in this browser." }
                button { r#type: "button", class: "small danger", disabled: remote_loading() || (history.read().repositories.is_empty() && history.read().refs.is_empty()), onclick: move |_| {
                    script::clear_remote_history(); history.set(BucketHistory::default());
                }, "Clear suggestions" }
            }
            if remote_loading() { p { role: "status", "Downloading research files…" } }
            if !remote_error().is_empty() { p { class: "error", role: "alert", "{remote_error}" } }
        }
        }
        if !message().is_empty() { p { role: "status", class: "small", "{message}" } }
        if !workspace().files.is_empty() { LoadedFiles { summary: format!("{} loaded files", workspace().files.len()) } Link { to: Route::Page { page: "portfolio".into() }, "Explore Portfolio →" } }
    }
}
fn notify_dashboard_kind(message: &str, kind: &'static str) {
    script::notify_dashboard(message, kind);
}
#[component]
pub(crate) fn Portfolio(dashboard_only: bool) -> Element {
    let mut workspace = use_context::<Signal<Workspace>>();
    let drawer = use_context::<Signal<Option<Detail>>>();
    let mut search = use_signal(String::new);
    let regex = use_signal(|| false);
    let case_sensitive = use_signal(|| false);
    let mut kind = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut sort = use_signal(|| "title");
    let mut descending = use_signal(|| false);
    let mut layout = use_signal(|| "list");
    let mut validating = use_signal(|| false);
    let mut tile_layout = use_signal(TileLayout::default);
    let mut tile_ready = use_signal(|| false);
    let mut arrange_tiles = use_signal(|| false);
    let mut dragged_tile = use_signal(|| None::<String>);
    let mut drop_tile = use_signal(|| None::<String>);
    let mut drop_cell = use_signal(|| None::<GridCell>);
    use_future(move || async move {
        let restored = script::load_dashboard_layout().await;
        if let Ok(Some(saved)) = restored {
            tile_layout.set(saved.normalized());
        }
        tile_ready.set(true);
    });
    use_future(move || async move {
        let mut changes = window_sync::subscribe_storage("acorn.dashboard.layout.v1");
        while changes.recv::<bool>().await.is_ok() {
            let layout = script::reload_dashboard_layout().await;
            match layout {
                | Ok(Some(saved)) => tile_layout.set(saved.normalized()),
                | Ok(None) => tile_layout.set(TileLayout::default()),
                | Err(_) => break,
            }
        }
    });
    let snapshot = workspace();
    let activities = snapshot.activities();
    let index = ResearchIndex::from(&snapshot);
    let tag_colors = tag_colors(&index.projects);
    let types = session::distribution(activities.iter().map(|(_, value)| session::activity_type(value)));
    let statuses = session::distribution(activities.iter().map(|(_, value)| session::activity_status(value)));
    let pattern = SearchPattern::new(&search(), regex(), case_sensitive());
    let matches = pattern
        .as_ref()
        .map(|pattern| {
            index.search_with(
                pattern,
                if sort() == "relevance" {
                    CatalogSort::Relevance
                } else {
                    CatalogSort::Alphabetical
                },
            )
        })
        .unwrap_or_default();
    let ranks = matches
        .iter()
        .enumerate()
        .map(|(rank, activity)| (activity.path.as_str(), rank))
        .collect::<alloc::collections::BTreeMap<_, _>>();
    let mut filtered = activities
        .iter()
        .copied()
        .filter(|(file, value)| {
            ranks.contains_key(file.path.as_str())
                && (kind().is_empty() || session::activity_type(value) == kind())
                && (status().is_empty() || session::activity_status(value) == status())
        })
        .collect::<Vec<_>>();
    filtered.sort_by(|(left_file, left), (right_file, right)| {
        let key = |value: &Value| match sort() {
            | "type" => session::activity_type(value).to_owned(),
            | "status" => session::activity_status(value).to_owned(),
            | _ => text(value, "title").to_lowercase(),
        };
        let order = if sort() == "relevance" {
            ranks.get(left_file.path.as_str()).cmp(&ranks.get(right_file.path.as_str()))
        } else {
            key(left).cmp(&key(right)).then(left_file.path.cmp(&right_file.path))
        };
        if descending() {
            order.reverse()
        } else {
            order
        }
    });
    let project_count = index
        .projects
        .iter()
        .filter(|project| project.paths.iter().any(|path| filtered.iter().any(|(file, _)| &file.path == path)))
        .count();
    let activity_noun = if filtered.len() == 1 { "activity" } else { "activities" };
    let project_noun = if project_count == 1 { "project" } else { "projects" };
    let record_noun = if activities.len() == 1 { "record" } else { "records" };
    let selection = filtered
        .iter()
        .find(|(file, _)| Some(&file.path) == snapshot.selected.as_ref())
        .or_else(|| filtered.first())
        .copied();
    let invalid = snapshot.validations.values().filter(|value| value["valid"] != true).count();
    let current_tile_layout = tile_layout();
    let visible_tile_count = current_tile_layout.order.iter().filter(|id| current_tile_layout.visible(id)).count();
    let visible_metrics = current_tile_layout
        .order
        .iter()
        .filter(|id| current_tile_layout.visible(id) && matches!(id.as_str(), "activities" | "organizations" | "contributors"))
        .cloned()
        .collect::<Vec<_>>();
    let visible_metric_count = visible_metrics.len();
    let first_metric = visible_metrics.first().cloned();
    rsx! {
        p { class: "lede", if dashboard_only { "Dashboard for {activities.len()} validated {record_noun} in this session" } else { "Research activities in this session · {activities.len()} validated {record_noun}" } }
        if snapshot.files.is_empty() && !dashboard_only {
            div { class: "card empty", Icon { name: "portfolio" } h2 { "No research activities loaded" } p { "Open local content or try the bundled sample from Home." } Link { to: Route::Home {}, "Open Home →" } }
        } else {
            if !dashboard_only {
            LoadedFiles { summary: format!("{} loaded files · {} stored bytes", snapshot.files.len(), snapshot.bytes()) }
            div { class: "actions", button { class: "primary", disabled: validating(), "aria-busy": validating(), onclick: move |_| async move {
                validating.set(true);
                let files = workspace.peek().files.clone();
                let validations = validate_files(files.iter().filter(|file| matches!(MimeType::from(file.path.as_str()), MimeType::Json)).cloned().collect()).await;
                validating.set(false);
                // A newer selection replaces the session while validation runs; its files are not these results' source.
                if workspace.peek().files == files {
                    workspace.write().validations = validations.into_iter().map(|(path, result)| (path, result.unwrap_or_else(|message| json!({"valid":false,"error":message})))).collect();
                }
            }, if validating() { "Validating…" } else { "Validate loaded activities" } } }
            p { class: "small muted", "Validation reads JSON files only and runs when requested. Other files remain available in the loaded tree. {invalid} invalid or unreadable activities." }
            if activities.is_empty() { p { class: "status invalid", "No validated activities yet. Validate the loaded JSON files, or inspect a file in Review." } }
            SearchField { label: "Search activities", placeholder: "Search titles, people, organizations, identifiers or files…", query: search, regex, case_sensitive, error: pattern.as_ref().err().cloned() }
            div { class: "options",
                label { class: "field", "Activity type" select { value: kind(), onchange: move |event| kind.set(event.value()), option { value: "", "All activity types" } for name in types.keys() { option { value: "{name}", "{name}" } } } }
                label { class: "field", "Status" select { value: status(), onchange: move |event| status.set(event.value()), option { value: "", "All statuses" } for name in statuses.keys() { option { value: "{name}", "{name}" } } } }
            }
            div { class: "catalog-toolbar", p { role: "status", "{filtered.len()} {activity_noun} in {project_count} {project_noun}" }
                div { class: "actions catalog-view-controls",
                    div { class: "catalog-layout", role: "group", "aria-label": "Portfolio layout",
                        button { "aria-label": "List", title: "List view", "aria-pressed": layout() == "list", onclick: move |_| layout.set("list"), Icon { name: "list" } }
                        button { "aria-label": "Cards", title: "Cards view", "aria-pressed": layout() == "cards", onclick: move |_| layout.set("cards"), Icon { name: "portfolio" } }
                    }
                    div { class: "catalog-sort", role: "group", "aria-label": "Sort activities",
                        button { r#type: "button", "aria-label": "Sort by title", title: "Sort by title", "aria-pressed": sort() == "title", onclick: move |_| { sort.set("title"); descending.set(false); }, Icon { name: "sort-title" } }
                        button { r#type: "button", "aria-label": "Sort by relevance", title: "Sort by relevance", "aria-pressed": sort() == "relevance", onclick: move |_| { sort.set("relevance"); descending.set(false); }, Icon { name: "search" } }
                        button { r#type: "button", "aria-label": "Sort by type", title: "Sort by type", "aria-pressed": sort() == "type", onclick: move |_| { sort.set("type"); descending.set(false); }, Icon { name: "file" } }
                        button { r#type: "button", "aria-label": "Sort by status", title: "Sort by status", "aria-pressed": sort() == "status", onclick: move |_| { sort.set("status"); descending.set(false); }, Icon { name: "check" } }
                        button { r#type: "button", "aria-label": "Descending order", title: "Descending order", "aria-pressed": descending(), onclick: move |_| descending.toggle(),
                            if descending() { Icon { name: "sort-desc" } } else { Icon { name: "sort-asc" } }
                        }
                    }
                }
            }
            if !filtered.is_empty() { section { class: "catalog-activity-section", "aria-label": "Activities", h2 { "Activities" }
                div { class: "catalog-activity-grid", "data-layout": layout(),
                    if layout() == "list" {
                        for (file, value) in &filtered {
                            {let path = file.path.clone(); let detail = Detail::file(file, &snapshot); let slide = snapshot.slides(&file.path, value).into_iter().next(); let project = index.projects.iter().find(|project| project.paths.contains(&file.path)); rsx! {
                                button { r#type: "button", class: "card catalog-activity-row", onclick: move |_| { workspace.write().selected = Some(path.clone()); details::open(drawer, detail.clone()); },
                                    if let Some(slide) = slide { img { class: "catalog-activity-thumb", src: slide.src().to_owned(), alt: "" } }
                                    span { class: "catalog-activity-body",
                                        strong { class: "catalog-activity-title", "{text(value, \"title\")}" }
                                        span { class: "small muted", "{file.path}" }
                                    }
                                    if let Some(project) = project { if !project.keywords.is_empty() { span { class: "catalog-keywords", "aria-label": "Keywords",
                                        for keyword in &project.keywords { span { class: "catalog-keyword", "data-tag-color": tag_colors.get(keyword.as_str()).copied().unwrap_or_default().to_string(), "{keyword}" } }
                                    } } }
                                    span { class: "catalog-activity-meta", ActivityStatus { status: session::activity_status(value).to_owned() } }
                                }
                            }}
                        }
                    } else {
                        for (file, value) in &filtered {
                            {let path = file.path.clone(); let detail = Detail::file(file, &snapshot); let slide = snapshot.slides(&file.path, value).into_iter().next(); let project = index.projects.iter().find(|project| project.paths.contains(&file.path)); rsx! {
                                button { r#type: "button", class: "card catalog-activity-card", onclick: move |_| { workspace.write().selected = Some(path.clone()); details::open(drawer, detail.clone()); },
                                    if let Some(slide) = slide { img { class: "catalog-activity-cover", src: slide.src().to_owned(), alt: "" } }
                                    span { class: "catalog-activity-card-body",
                                        strong { class: "catalog-activity-card-title", "{text(value, \"title\")}" }
                                        ActivityStatus { status: session::activity_status(value).to_owned() }
                                        if let Some(project) = project {
                                            if !project.summary.is_empty() { span { class: "catalog-project-summary", "{project.summary}" } }
                                            if !project.keywords.is_empty() { span { class: "catalog-keywords", "aria-label": "Keywords",
                                                for keyword in &project.keywords { span { class: "catalog-keyword", "data-tag-color": tag_colors.get(keyword.as_str()).copied().unwrap_or_default().to_string(), "{keyword}" } }
                                            } }
                                        }
                                    }
                                }
                            }}
                        }
                    }
                } }
            }
            }
            div { class: "dashboard-actions",
                details { class: "dashboard-settings",
                    summary { Icon { name: "settings" } "Dashboard settings" }
                    div { class: "dashboard-settings-content",
                    div { class: "dashboard-sort", role: "group", "aria-label": "Sort dashboard tiles",
                    span { class: "small muted", "Sort" }
                    button { r#type: "button", disabled: !tile_ready(), "aria-pressed": tile_layout().positions.is_empty() && tile_layout().order == TileLayout::default().order, onclick: move |_| { let next = tile_layout().restore_order(); tile_layout.set(next.clone()); notify_dashboard_kind("Original tile order restored.", "success"); spawn(async move { if !save_tile_layout(&next).await { notify_dashboard_kind("Tile order changed, but could not be saved on this device.", "warning"); } }); }, "Original" }
                    button { r#type: "button", disabled: !tile_ready(), "aria-pressed": tile_layout().positions.is_empty() && tile_layout().order == tile_layout().sort_by_label(false).order, onclick: move |_| { let next = tile_layout().sort_by_label(false); tile_layout.set(next.clone()); notify_dashboard_kind("Dashboard sorted A–Z.", "success"); spawn(async move { if !save_tile_layout(&next).await { notify_dashboard_kind("Tile order changed, but could not be saved on this device.", "warning"); } }); }, "A–Z" }
                    button { r#type: "button", disabled: !tile_ready(), "aria-pressed": tile_layout().positions.is_empty() && tile_layout().order == tile_layout().sort_by_label(true).order, onclick: move |_| { let next = tile_layout().sort_by_label(true); tile_layout.set(next.clone()); notify_dashboard_kind("Dashboard sorted Z–A.", "success"); spawn(async move { if !save_tile_layout(&next).await { notify_dashboard_kind("Tile order changed, but could not be saved on this device.", "warning"); } }); }, "Z–A" }
                    }
                    div { class: "dashboard-other-controls",
                        button { class: "dashboard-arrange", r#type: "button", disabled: !tile_ready(), "aria-pressed": arrange_tiles(), title: "Show empty grid cells for placing tiles", onclick: move |_| { arrange_tiles.toggle(); dragged_tile.set(None); drop_tile.set(None); drop_cell.set(None); }, "Place freely" }
                        button { class: "dashboard-restore", r#type: "button", disabled: !tile_ready() || tile_layout().hidden.is_empty(), onclick: move |_| { let next = tile_layout().restore_hidden(); tile_layout.set(next.clone()); notify_dashboard_kind("Hidden dashboard tiles restored.", "success"); spawn(async move { if !save_tile_layout(&next).await { notify_dashboard_kind("Tile visibility changed, but could not be saved on this device.", "warning"); } }); }, "Restore hidden tiles" }
                        button { class: "dashboard-reset", r#type: "button", disabled: !tile_ready() || tile_layout() == TileLayout::default(), onclick: move |_| { tile_layout.set(TileLayout::default()); arrange_tiles.set(false); notify_dashboard_kind("Dashboard layout reset.", "success"); spawn(async move { if !reset_tile_layout().await { notify_dashboard_kind("Dashboard layout reset for this session, but saved settings could not be cleared.", "warning"); } }); }, "Reset dashboard" }
                        if !dashboard_only { a { class: "dashboard-open", href: "dashboard", target: "_blank", rel: "noopener noreferrer", "Open dashboard in new tab ↗" } }
                    }
                    }
                }
            }
            div { class: "portfolio-dashboard", "data-dashboard-page": "portfolio", "data-layout": if tile_layout() == TileLayout::default() && !arrange_tiles() { "default" } else { "custom" }, "data-positioned": arrange_tiles() || !tile_layout().positions.is_empty(), "data-arranging": arrange_tiles(), "data-metric-count": "{visible_metric_count}",
                for (position, tile_id) in tile_layout().order.iter().filter(|id| tile_layout().visible(id)).enumerate() {
                    { let tile_id = tile_id.clone(); let up_id = tile_id.clone(); let down_id = tile_id.clone(); let hide_id = tile_id.clone(); let drag_id = tile_id.clone(); let over_id = tile_id.clone(); let drop_id = tile_id.clone(); let label = tile_label(&tile_id); let cell = tile_layout().placement(&tile_id).unwrap_or(GridCell { row: 1, column: 1 }); let span = if matches!(tile_id.as_str(), "activities" | "organizations" | "contributors") { 1 } else { 2 }; let first_metric_tile = first_metric.as_deref() == Some(tile_id.as_str()); rsx! { div { key: "{tile_id}", class: if dragged_tile().as_deref() == Some(tile_id.as_str()) { "dashboard-slot is-dragging" } else if drop_tile().as_deref() == Some(tile_id.as_str()) { "dashboard-slot is-drop-target" } else { "dashboard-slot" }, "data-first-metric": if first_metric_tile { "true" } else { "false" }, style: "--tile-row:{cell.row};--tile-column:{cell.column};--tile-span:{span}",
                        ondragover: move |event| { if !arrange_tiles() && dragged_tile().as_deref().is_some_and(|source| source != over_id.as_str()) { event.prevent_default(); event.data_transfer().set_drop_effect("move"); if drop_tile().as_deref() != Some(over_id.as_str()) { drop_tile.set(Some(over_id.clone())); } } },
                        ondrop: move |event| { if !arrange_tiles() { event.prevent_default(); let source = dragged_tile(); dragged_tile.set(None); drop_tile.set(None); if let Some(source) = source { let next = tile_layout().move_to(&source, &drop_id); if next != tile_layout() { tile_layout.set(next.clone()); notify_dashboard_kind("Dashboard tiles reordered.", "success"); spawn(async move { if !save_tile_layout(&next).await { notify_dashboard_kind("Tile order changed, but could not be saved on this device.", "warning"); } }); } } } },
                        div { class: "dashboard-tile-controls", role: "group", "aria-label": "{label} tile controls",
                            button { class: "dashboard-drag-handle", r#type: "button", disabled: !tile_ready(), draggable: "true", "aria-label": if arrange_tiles() { "Drag {label} to place" } else { "Drag {label} to reorder" }, title: if arrange_tiles() { "Drag {label} to an empty grid cell" } else { "Drag {label} to reorder" }, ondragstart: move |event| { let transfer = event.data_transfer(); let _ = transfer.set_data("text/plain", &drag_id); transfer.set_effect_allowed("move"); dragged_tile.set(Some(drag_id.clone())); drop_tile.set(None); drop_cell.set(None); }, ondragend: move |_| { dragged_tile.set(None); drop_tile.set(None); drop_cell.set(None); }, "⋮⋮" }
                            button { r#type: "button", disabled: !tile_ready() || position == 0, "aria-label": "Move {label} up", title: "Move {label} up", onclick: move |_| { let next = tile_layout().move_by(&up_id, -1); tile_layout.set(next.clone()); notify_dashboard_kind(&format!("Moved {label} up."), "success"); spawn(async move { if !save_tile_layout(&next).await { notify_dashboard_kind("Tile order changed, but could not be saved on this device.", "warning"); } }); }, "↑" }
                            button { r#type: "button", disabled: !tile_ready() || position + 1 == visible_tile_count, "aria-label": "Move {label} down", title: "Move {label} down", onclick: move |_| { let next = tile_layout().move_by(&down_id, 1); tile_layout.set(next.clone()); notify_dashboard_kind(&format!("Moved {label} down."), "success"); spawn(async move { if !save_tile_layout(&next).await { notify_dashboard_kind("Tile order changed, but could not be saved on this device.", "warning"); } }); }, "↓" }
                            button { r#type: "button", disabled: !tile_ready(), "aria-label": "Hide {label}", title: "Hide {label}", onclick: move |_| { let next = tile_layout().set_visible(&hide_id, false); tile_layout.set(next.clone()); notify_dashboard_kind(&format!("{label} hidden."), "success"); script::focus_dashboard_settings(); spawn(async move { if !save_tile_layout(&next).await { notify_dashboard_kind("Tile visibility changed, but could not be saved on this device.", "warning"); } }); }, "×" }
                        }
                        match tile_id.as_str() {
                            "activities" => rsx! { MetricTile { tile_id: "activities", label: "Activities", count: filtered.len() } },
                            "organizations" => rsx! { MetricTile { tile_id: "organizations", label: "Organizations", count: session::entities(&filtered, "organizations").len() } },
                            "contributors" => rsx! { MetricTile { tile_id: "contributors", label: "Contributors", count: session::entities(&filtered, "contributors").len() } },
                            "top-topics" => rsx! { TopicBars { counts: top_topics(&filtered), total: filtered.len() } },
                            "statuses" => rsx! { StatusDonut { counts: session::distribution(filtered.iter().map(|(_, value)| session::activity_status(value))) } },
                            "identifiers" => rsx! { IdentifierDonut { counts: session::distribution(session::identifier_labels(&filtered)) } },
                            _ => rsx! {},
                        }
                    } } }
                }
                if arrange_tiles() && visible_tile_count > 0 {
                    for cell in tile_layout().empty_cells() {
                        { let row = cell.row; let column = cell.column; rsx! { div { key: "empty-{row}-{column}", class: if drop_cell() == Some(cell) { "dashboard-empty-cell is-drop-target" } else { "dashboard-empty-cell" }, "data-grid-cell": "{row}-{column}", style: "--tile-row:{row};--tile-column:{column}", "aria-label": "Empty dashboard position, row {row}, column {column}",
                            ondragover: move |event| { if let Some(source) = dragged_tile() { if tile_layout().can_place(&source, cell) { event.prevent_default(); event.data_transfer().set_drop_effect("move"); drop_cell.set(Some(cell)); } } },
                            ondragleave: move |_| { if drop_cell() == Some(cell) { drop_cell.set(None); } },
                            ondrop: move |event| { event.prevent_default(); let source = dragged_tile(); dragged_tile.set(None); drop_cell.set(None); if let Some(source) = source { let next = tile_layout().place_at(&source, cell); if next != tile_layout() { tile_layout.set(next.clone()); notify_dashboard_kind(&format!("Tile placed at row {row}, column {column}."), "success"); spawn(async move { if !save_tile_layout(&next).await { notify_dashboard_kind("Tile placement changed, but could not be saved on this device.", "warning"); } }); } } },
                        } } }
                    }
                }
                if tile_layout().hidden.len() == PORTFOLIO_TILES.len() { div { class: "card empty", p { "No dashboard tiles shown. Open Dashboard settings to restore hidden tiles or reset the layout." } } }
            }
            if !dashboard_only { table { class: "activity-table", caption { "{filtered.len()} matching {activity_noun}" }
                thead { tr { for (field, label) in [("title", "Activity"), ("type", "Type"), ("status", "Status")] {
                    th { scope: "col", "aria-sort": if sort() == field { if descending() { "descending" } else { "ascending" } } else { "none" }, button { onclick: move |_| { if sort() == field { descending.toggle(); } else { sort.set(field); descending.set(false); } }, "{label}" } }
                } } }
                tbody { for (file, value) in &filtered { {let path = file.path.clone(); let detail = Detail::file(file, &snapshot); rsx! {
                    tr { td { button { class: "activity-select", "aria-pressed": selection.is_some_and(|(chosen, _)| chosen.path == path), onclick: move |_| { workspace.write().selected = Some(path.clone()); details::open(drawer, detail.clone()); }, "{text(value, \"title\")}" } small { "{file.path}" } } td { "{session::activity_type(value)}" } td { "{session::activity_status(value)}" } }
                }} } }
            }
            if filtered.is_empty() && !activities.is_empty() { div { class: "card empty", h2 { "No matching activities" } p { "Try another search or reset the filters." } button { onclick: move |_| { search.set(String::new()); kind.set(String::new()); status.set(String::new()); }, "Reset filters" } } }
            }

        }
    }
}
async fn reset_tile_layout() -> bool {
    script::reset_dashboard_layout().await.unwrap_or(false)
}
fn route_tool(route: &Route) -> Option<Tool> {
    match route {
        | Route::Page { page } => Tool::iter().find(|tool| tool.slug() == page),
        | Route::Home {} | Route::Graph { .. } => None,
    }
}
async fn save_tile_layout(layout: &TileLayout) -> bool {
    script::save_dashboard_layout(layout).await.unwrap_or(false)
}
fn status_color(status: &str) -> &'static str {
    match status {
        | "active" => "var(--status-active)",
        | "canceled" => "var(--status-canceled)",
        | "completed" => "var(--status-completed)",
        | "paused" => "var(--status-paused)",
        | _ => "var(--muted)",
    }
}
#[component]
fn StatusDonut(counts: BTreeMap<String, usize>) -> Element {
    let segments = counts
        .into_iter()
        .map(|(label, count)| ChartSegment {
            color: status_color(&label).into(),
            label,
            count,
        })
        .collect();
    rsx! { DonutTile { tile_id: "statuses", class: "status-donut", title: "Statuses", center_label: "activities", empty_message: "No matching activities", segments } }
}
pub(crate) fn tag_colors(projects: &[Project]) -> BTreeMap<&str, usize> {
    projects
        .iter()
        .flat_map(|project| project.keywords.iter().map(String::as_str))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .enumerate()
        .map(|(index, tag)| (tag, index % TAG_COLOR_COUNT))
        .collect()
}
fn tile_label(id: &str) -> &'static str {
    PORTFOLIO_TILES.iter().find(|(tile, _)| *tile == id).map_or("Tile", |(_, label)| *label)
}
pub(crate) fn top_topics(records: &[(&SuppliedFile, &Value)]) -> Vec<(String, usize)> {
    let counts = records.iter().fold(BTreeMap::new(), |mut counts, (_, value)| {
        session::many(value.pointer("/meta/keywords").unwrap_or(&Value::Null))
            .into_iter()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .for_each(|keyword| {
                counts
                    .entry(keyword.to_owned())
                    .and_modify(|count: &mut usize| *count = count.saturating_add(1))
                    .or_insert(1);
            });
        counts
    });
    let mut topics = counts.into_iter().collect::<Vec<_>>();
    topics.sort_by(|(left, left_count), (right, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left.to_lowercase().cmp(&right.to_lowercase()))
            .then_with(|| left.cmp(right))
    });
    topics.truncate(6);
    topics
}
#[component]
fn TopicBars(counts: Vec<(String, usize)>, total: usize) -> Element {
    rsx! { DashboardTile { tile_id: "top-topics", class: "topic-bars", h2 { "Top topics" }
        if counts.is_empty() { p { class: "muted", "No keywords in matching activities" } }
        else {
            p { class: "small muted", "Activities can appear under more than one topic." }
            for (label, count) in counts { div { class: "proportion", p { span { "{label}" } span { "{count}" } } progress { value: "{count}", max: "{total}", "aria-label": "{label}: {count} of {total} matching activities" } } }
        }
    } }
}
pub(crate) fn tree_children(paths: &[String], prefix: &str, descending: bool) -> Vec<(bool, String)> {
    let mut children = paths
        .iter()
        .filter_map(|path| path.strip_prefix(prefix))
        .filter_map(|relative| relative.split('/').next())
        .map(|name| (paths.contains(&format!("{prefix}{name}")), name.to_owned()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if descending {
        children.sort_by(|(left_file, left), (right_file, right)| left_file.cmp(right_file).then_with(|| right.cmp(left)));
    }
    children
}
pub(crate) fn visible_file_paths(
    files: &[SuppliedFile],
    pattern: &SearchPattern,
    kind: &str,
    tool: Option<Tool>,
    selected_kinds: Option<&BTreeSet<String>>,
) -> Vec<String> {
    files
        .iter()
        .filter(|file| pattern.matches(&file.path))
        .filter(|file| match selected_kinds {
            | Some(selected) => selected.contains(file_kind(file)),
            | None => match kind {
                | "all" => true,
                | "loadable" => tool.is_none_or(|tool| file_loadable(file, tool)),
                | kind => file_kind(file) == kind,
            },
        })
        .map(|file| file.path.clone())
        .collect()
}
