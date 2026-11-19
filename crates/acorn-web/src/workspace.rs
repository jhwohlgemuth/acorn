use crate::details::{self, Detail};
use crate::search_ui::SearchField;
use crate::ui::{validate_files, FileInput, Icon, Route, UrlFileInput};
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

const TAG_COLOR_COUNT: usize = 8;
#[derive(Clone, Default, serde::Deserialize)]
struct BucketHistory {
    repositories: Vec<String>,
    refs: Vec<String>,
}
#[component]
fn TopicBars(counts: Vec<(String, usize)>, total: usize) -> Element {
    rsx! { div { class: "card topic-bars", h2 { "Top topics" }
        if counts.is_empty() { p { class: "muted", "No keywords in matching activities" } }
        else {
            p { class: "small muted", "Activities can appear under more than one topic." }
            for (label, count) in counts { div { class: "proportion", p { span { "{label}" } span { "{count}" } } progress { value: "{count}", max: "{total}", "aria-label": "{label}: {count} of {total} matching activities" } } }
        }
    } }
}
#[component]
fn StatusDonut(counts: BTreeMap<String, usize>) -> Element {
    let total = counts.values().sum::<usize>();
    let denominator = total.max(1) as f64;
    let gradient = counts
        .iter()
        .scan(0_usize, |offset, (status, count)| {
            let start = *offset as f64 * 100.0 / denominator;
            *offset += *count;
            let end = *offset as f64 * 100.0 / denominator;
            Some(format!("{} {start:.2}% {end:.2}%", status_color(status)))
        })
        .collect::<Vec<_>>()
        .join(", ");
    rsx! { div { class: "card", h2 { "Statuses" }
        if total == 0 { p { class: "muted", "No matching activities" } }
        else { div { class: "status-donut-content",
            div { class: "status-donut", style: format!("background:conic-gradient({gradient})"), "aria-hidden": "true",
                div { class: "status-donut-center", strong { "{total}" } small { "activities" } }
            }
            div { class: "status-donut-legend", for (status, count) in counts {
                div { class: "status-donut-legend-row",
                    span { class: "status-donut-legend-label", span { class: "status-donut-swatch", style: format!("background:{}", status_color(&status)), "aria-hidden": "true" } "{status}" }
                    strong { "{count}" }
                }
            } }
        } }
    } }
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
fn file_loadable(file: &SuppliedFile, tool: Tool) -> bool {
    let mime = MimeType::from(file.path.as_str());
    !mime.is_image()
        && file.content.len() <= tool.limit()
        && match tool {
            | Tool::Activity | Tool::Fair => matches!(mime, MimeType::Json),
            | Tool::Standard | Tool::Crosswalk => file_format(&file.path).is_some(),
            | _ => true,
        }
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
    match MimeType::from(file.path.as_str()) {
        | MimeType::Json => "json",
        | MimeType::Yaml => "yaml",
        | MimeType::Markdown => "markdown",
        | MimeType::Text => "text",
        | mime if mime.is_image() => "image",
        | _ => "other",
    }
}
fn route_tool(route: &Route) -> Option<Tool> {
    match route {
        | Route::Page { page } => Tool::iter().find(|tool| tool.slug() == page),
        | Route::Home {} => None,
    }
}
fn tree_children(paths: &[String], prefix: &str, descending: bool) -> Vec<(bool, String)> {
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
fn visible_file_paths(files: &[SuppliedFile], pattern: &SearchPattern, kind: &str, tool: Option<Tool>) -> Vec<String> {
    files
        .iter()
        .filter(|file| pattern.matches(&file.path))
        .filter(|file| match kind {
            | "all" => true,
            | "loadable" => tool.is_none_or(|tool| file_loadable(file, tool)),
            | kind => file_kind(file) == kind,
        })
        .map(|file| file.path.clone())
        .collect()
}
#[component]
pub(crate) fn LoadedFiles(
    summary: String,
    #[props(default)] onload: Option<EventHandler<String>>,
    #[props(default)] onselect: Option<EventHandler<SuppliedFile>>,
    #[props(default)] selected_tool: Option<Tool>,
) -> Element {
    let workspace = use_context::<Signal<Workspace>>();
    let route = use_route::<Route>();
    let tool = selected_tool.or_else(|| route_tool(&route));
    let query = use_signal(String::new);
    let regex = use_signal(|| false);
    let case_sensitive = use_signal(|| false);
    let mut kind = use_signal(|| "all".to_owned());
    let mut descending = use_signal(|| false);
    let snapshot = workspace();
    let pattern = SearchPattern::new(&query(), regex(), case_sensitive());
    let paths = pattern
        .as_ref()
        .map(|pattern| visible_file_paths(&snapshot.files, pattern, &kind(), tool))
        .unwrap_or_default();
    rsx! { details { class: "loaded-tree", summary {
        span { class: "loaded-tree-icon", Icon { name: "folder-open" } }
        span { class: "loaded-tree-label", "{summary}" }
    }
        if let (Some(tool), Some(onload)) = (tool, onload) { FileInput { onload, limit: tool.limit(), tool } }
        if !snapshot.files.is_empty() {
            div { class: "loaded-files-controls",
                SearchField { label: "Find a file", placeholder: "Search paths…", query, regex, case_sensitive, error: pattern.as_ref().err().cloned() }
                label { class: "field", "Show" select { value: kind(), onchange: move |event| kind.set(event.value()),
                    option { value: "all", "All files" }
                    if tool.is_some() { option { value: "loadable", "Available here" } }
                    for (value, label) in [("json", "JSON"), ("yaml", "YAML"), ("markdown", "Markdown"), ("text", "Text"), ("image", "Images")] { option { value, "{label}" } }
                } }
                label { class: "field", "Sort" select { value: if descending() { "descending" } else { "ascending" }, onchange: move |event| descending.set(event.value() == "descending"),
                    option { value: "ascending", "Name A–Z" }
                    option { value: "descending", "Name Z–A" }
                } }
            }
            p { class: "loaded-files-count small muted", role: "status", "{paths.len()} of {snapshot.files.len()} files shown" }
            if paths.is_empty() { p { class: "loaded-files-empty small muted", "No matching files." } }
            else { FileTree { paths, prefix: String::new(), descending: descending(), tool, onselect } }
        }
    } }
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
            if is_file { button { class: "file-leaf", disabled: !loadable, title: if loadable { "" } else { "This file cannot be loaded into this page" }, "aria-pressed": workspace.read().selected.as_deref() == Some(path.as_str()), onclick: move |_| { if loadable { let snapshot = workspace(); if let Some(file) = snapshot.files.iter().find(|file| file.path == path) { if tool.is_some() { if let Some(onselect) = onselect { onselect.call(file.clone()); } else { input.set(file.content.clone()); } } else { details::open(drawer, Detail::file(file, &snapshot)); } if !MimeType::from(file.path.as_str()).is_image() { workspace.write().selected = Some(path.clone()); } } } }, if let Some(thumbnail) = thumbnail { img { class: "file-thumb", src: thumbnail, alt: "" } } else { Icon { name: "file" } } "{name}" } }
            else { details { summary { Icon { name: "folder-open" } "{name}" } FileTree { paths: paths.clone(), prefix: format!("{path}/"), descending, tool, onselect } } }
        }}}
    } } }
}
#[component]
pub(crate) fn Loader() -> Element {
    let mut workspace = use_context::<Signal<Workspace>>();
    let mut input = use_context::<Signal<String>>();
    let mut message = use_signal(String::new);
    let mut choose = use_signal(|| false);
    let mut url_open = use_signal(|| false);
    let mut close_url = move || {
        url_open.set(false);
        document::eval("document.getElementById('home-source-trigger')?.focus()");
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
        if let Ok(saved) = document::eval("if (!window.acornRemote) await new Promise(resolve => window.addEventListener('acorn-remote-ready', resolve, {once:true})); return window.acornRemote.history.read();").join::<BucketHistory>().await {
            history.set(saved);
        }
    });
    use_future(move || async move {
        if let Ok(host) = document::eval("if (!window.acornRemote) await new Promise(resolve => window.addEventListener('acorn-remote-ready', resolve, {once:true})); return window.acornRemote.credentialStatus();").join::<String>().await {
            credential_status.set(host);
        }
    });
    use_future(move || async move {
        let mut channel = document::eval("if (!window.acornWorkspace) await new Promise(resolve => window.addEventListener('acorn-effects-ready', resolve, {once:true})); window.acornWorkspace.connect(batch => dioxus.send(batch)); await dioxus.recv();");
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
        document::eval("window.acornWorkspace?.disconnect()");
    });
    rsx! {
        p { class: "load-caption muted", "Open research activity data" }
        div { id: "local-entry", class: "load-tiles",
            div { class: "load-tile",
                button { id: "home-source-trigger", class: "tile-action", "aria-expanded": choose(), "aria-controls": "loader-choices", onclick: move |_| { url_open.set(false); choose.toggle(); }, span { class: "tile-icon folder-icon", "aria-hidden": "true", Icon { name: "folder-open" } } strong { "Open file or folder" } small { "Files, folders or URL" } }
                if choose() { div { id: "loader-choices", class: "actions", "aria-label": "Choose content source",
                    button { onclick: move |_| { choose.set(false); document::eval("window.acornWorkspace.pick('file')"); }, Icon { name: "file" } "Files" }
                    button { onclick: move |_| { choose.set(false); document::eval("window.acornWorkspace.pick('folder')"); }, Icon { name: "folder-open" } "Folder" }
                    button { onclick: move |_| { choose.set(false); url_open.set(true); document::eval("requestAnimationFrame(() => document.querySelector('.url-file-loader input')?.focus())"); }, Icon { name: "link" } "URL" }
                } }
            }
            div { class: "load-tile", "data-workspace-drop": "true", button { class: "tile-action", onclick: move |_| { document::eval("window.acornWorkspace.pick('file')"); }, span { class: "tile-icon upload-icon", "aria-hidden": "true", Icon { name: "upload" } } strong { "Drop content" } small { "Drop files or folders" } } }
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
        p { class: "load-limits small muted", "JSON, YAML, Markdown, text, SVG, PNG, JPEG or WebP · 2 MiB per file · 500 files · 32 MiB stored total" }
        details { class: "remote-entry", summary {
            "Open a " span { class: "repository-name repository-github", img { src: asset!("/assets/generated/github.svg"), alt: "", width: "16", height: "16" } "GitHub" }
            " or " span { class: "repository-name", img { src: asset!("/assets/generated/gitlab.svg"), alt: "", width: "16", height: "16" } "GitLab" } " bucket"
        }
        form { class: "remote-loader", onsubmit: move |event| async move {
            event.prevent_default();
            if remote_loading() { return; }
            remote_loading.set(true); remote_error.set(String::new()); remote_success.set(String::new()); message.set(String::new());
            let args = serde_json::to_string(&(remote_url(), remote_ref())).unwrap_or_default();
            let result = document::eval(&format!("if (!window.acornRemote) await new Promise(resolve => window.addEventListener('acorn-remote-ready', resolve, {{once:true}})); let result; try {{ result = {{ok: await window.acornRemote.load(...{args})}}; }} catch(error) {{ result = {{error: error.message}}; }} return {{...result, history: window.acornRemote.history.read()}};")).join::<Value>().await;
            match result {
                Ok(value) => {
                    if let Ok(saved) = serde_json::from_value::<BucketHistory>(value.get("history").cloned().unwrap_or(Value::Null)) { history.set(saved); }
                    if let Some(error) = value.get("error").and_then(Value::as_str) { remote_error.set(error.to_owned()); }
                    else {
                        let mut loaded = value.get("ok").cloned().unwrap_or(Value::Null);
                        let source = loaded.get("source").cloned().and_then(|value| serde_json::from_value::<WorkspaceSource>(value).ok());
                        let unavailable = loaded.get("unavailable").and_then(Value::as_u64).unwrap_or(0);
                        if let Some(object) = loaded.as_object_mut() { object.remove("source"); object.remove("unavailable"); }
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
                                remote_success.set(format!("Bucket loaded: {} {noun}{origin}; {} entries skipped{failed}.", next.files.len(), next.skipped));
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
                    let result = document::eval("try { const field = document.getElementById('repository-token'); const status = window.acornRemote.setCredential(field.value); field.value = ''; return {status}; } catch(error) { return {error: error.message}; }").join::<Value>().await;
                    match result {
                        Ok(value) => if let Some(status) = value.get("status").and_then(Value::as_str) { credential_status.set(status.to_owned()); credential_message.set("Token saved for repository loads in this tab.".into()); credential_error.set(String::new()); } else { credential_error.set(value.get("error").and_then(Value::as_str).unwrap_or("Enter a valid access token.").to_owned()); credential_message.set(String::new()); },
                        Err(_) => { credential_error.set("Could not save the repository token. Try again.".into()); credential_message.set(String::new()); },
                    }
                }, "Save token" }
                button { r#type: "button", class: "danger", disabled: credential_status().is_empty(), onclick: move |_| async move { if matches!(document::eval("const removed = window.acornRemote?.clearCredential() ?? false; const field = document.getElementById('repository-token'); if (field) field.value = ''; return removed;").join::<bool>().await, Ok(true)) { credential_status.set(String::new()); credential_message.set("Token removed from this tab.".into()); credential_error.set(String::new()); } else { credential_error.set("Could not remove the repository token from this tab.".into()); credential_message.set(String::new()); } }, "Remove token" } }
                if !credential_message().is_empty() { p { role: "status", class: "status", "{credential_message}" } }
                if !credential_error().is_empty() { p { role: "alert", class: "error", "{credential_error}" } }
            }
            label { class: "field", "Repository or URL" input { r#type: "text", required: true, disabled: remote_loading(), value: remote_url(), oninput: move |event| remote_url.set(event.value()), list: "bucket-repositories", autocomplete: "off", placeholder: "owner/repo or https://gitlab.example.org/group/project", spellcheck: "false" } }
            datalist { id: "bucket-repositories", for repository in &history.read().repositories { option { value: repository.clone() } } }
            datalist { id: "bucket-refs", for reference in &history.read().refs { option { value: reference.clone() } } }
            div { class: "remote-actions", label { class: "field", "Branch / tag / commit (optional)" input { disabled: remote_loading(), value: remote_ref(), oninput: move |event| remote_ref.set(event.value()), list: "bucket-refs", autocomplete: "off", placeholder: "Repository default", spellcheck: "false" } }
                button { r#type: "submit", disabled: remote_loading() || remote_url().trim().is_empty(), "aria-busy": remote_loading(), Icon { name: "download" } if remote_loading() { "Loading…" } else { "Load bucket" } }
                if remote_loading() { button { r#type: "button", onclick: move |_| { document::eval("window.acornRemote?.cancel()"); }, "Cancel" } }
            }
            p { class: "small muted", "Public repositories, folder or file URLs. For branches containing /, enter the full branch above. Loading replaces this session after a successful download." }
            if !remote_success().is_empty() { div { class: "status bucket-success", role: "status", span { "{remote_success}" } Link { to: Route::Page { page: "portfolio".into() }, "Explore Portfolio →" } } }
            div { class: "actions small muted", span { "Recent inputs are remembered in this browser." }
                button { r#type: "button", class: "small danger", disabled: remote_loading() || (history.read().repositories.is_empty() && history.read().refs.is_empty()), onclick: move |_| {
                    document::eval("window.acornRemote?.history.clear()"); history.set(BucketHistory::default());
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
#[component]
pub(crate) fn Portfolio() -> Element {
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
    let snapshot = workspace();
    let activities = snapshot.activities();
    let index = ResearchIndex::from(&snapshot);
    let tag_colors = tag_colors(&index.projects);
    let types = session::distribution(&activities, session::activity_type);
    let statuses = session::distribution(&activities, session::activity_status);
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
    rsx! {
        p { class: "lede", "Research activities in this session · {activities.len()} validated {record_noun}" }
        if snapshot.files.is_empty() {
            div { class: "card empty", Icon { name: "portfolio" } h2 { "No research activities loaded" } p { "Open local content or try the bundled sample from Home." } Link { to: Route::Home {}, "Open Home →" } }
        } else {
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
                            {let path = file.path.clone(); let detail = Detail::file(file, &snapshot); let slide = snapshot.slides(&file.path, value).into_iter().next(); rsx! {
                                button { r#type: "button", class: "card catalog-activity-row", onclick: move |_| { workspace.write().selected = Some(path.clone()); details::open(drawer, detail.clone()); },
                                    if let Some(slide) = slide { img { class: "catalog-activity-thumb", src: slide.src().to_owned(), alt: "" } }
                                    span { class: "catalog-activity-body",
                                        strong { class: "catalog-activity-title", "{text(value, \"title\")}" }
                                        span { class: "small muted", "{file.path}" }
                                    }
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
            div { class: "metrics",
                for (label, count) in [("Activities", filtered.len()), ("Organizations", session::entities(&filtered, "organizations").len()), ("Contributors", session::entities(&filtered, "contributors").len()), ("Identifiers", session::identifiers(&filtered).len())] {
                    div { class: "card metric", strong { "{count}" } small { "{label}" } }
                }
            }
            div { class: "distribution-grid", TopicBars { counts: top_topics(&filtered), total: filtered.len() } StatusDonut { counts: session::distribution(&filtered, session::activity_status) } }
            table { class: "activity-table", caption { "{filtered.len()} matching {activity_noun}" }
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
fn top_topics(records: &[(&SuppliedFile, &Value)]) -> Vec<(String, usize)> {
    let counts = records.iter().fold(BTreeMap::new(), |mut counts, (_, value)| {
        session::many(&value["meta"]["keywords"])
            .into_iter()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .for_each(|keyword| *counts.entry(keyword.to_owned()).or_insert(0_usize) += 1);
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
fn tag_colors(projects: &[Project]) -> BTreeMap<&str, usize> {
    projects
        .iter()
        .flat_map(|project| project.keywords.iter().map(String::as_str))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .enumerate()
        .map(|(index, tag)| (tag, index % TAG_COLOR_COUNT))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_format_for_loaded_content() {
        assert_eq!(file_format("record.json"), Some("json"));
        assert_eq!(file_format("record.yaml"), Some("yaml"));
        assert_eq!(file_format("record.md"), Some("markdown"));
        assert_eq!(file_format("record.jsonld"), Some("json-ld"));
        assert_eq!(file_format("cover.svg"), None);
    }
    #[test]
    fn test_loaded_file_matches_page_input() {
        let json = SuppliedFile {
            path: "project/index.json".into(),
            content: "{}".into(),
        };
        let yaml = SuppliedFile {
            path: "project/index.yaml".into(),
            content: "title: Test".into(),
        };
        let svg = SuppliedFile {
            path: "project/banner.svg".into(),
            content: "<svg/>".into(),
        };
        let long_text = SuppliedFile {
            path: "project/notes.txt".into(),
            content: "x".repeat(Tool::Inspect.limit() + 1),
        };
        assert!(file_loadable(&json, Tool::Activity));
        assert!(!file_loadable(&yaml, Tool::Activity));
        assert!(file_loadable(&yaml, Tool::Standard));
        assert!(!file_loadable(&svg, Tool::Standard));
        assert!(!file_loadable(&long_text, Tool::Inspect));
    }
    #[test]
    fn test_tag_colors_repeat_after_palette_and_ignore_project_order() {
        let tags = (0..=TAG_COLOR_COUNT).map(|index| format!("tag-{index:02}")).collect::<Vec<_>>();
        let first = tags.first().expect("first tag");
        let wrapped = tags.get(TAG_COLOR_COUNT).expect("tag after the palette");
        let project = |keywords: Vec<String>| Project {
            id: String::new(),
            title: String::new(),
            summary: String::new(),
            paths: Vec::new(),
            organizations: Vec::new(),
            keywords,
        };
        let projects = [project(tags.clone()), project(vec![first.clone()])];
        let colors = tag_colors(&projects);
        assert_eq!(colors.len(), TAG_COLOR_COUNT + 1);
        assert_eq!(colors.values().copied().collect::<BTreeSet<_>>().len(), TAG_COLOR_COUNT);
        assert_eq!(colors.get(first.as_str()), colors.get(wrapped.as_str()));
        assert_eq!(colors, tag_colors(&[projects[1].clone(), projects[0].clone()]));
    }
    #[test]
    fn test_top_topics_counts_activities_once_and_sorts_by_frequency() {
        let files = (0..4)
            .map(|index| SuppliedFile {
                path: format!("{index}.json"),
                content: String::new(),
            })
            .collect::<Vec<_>>();
        let values = [
            json!({"meta":{"keywords":["mapping","mapping","visualization"]}}),
            json!({"meta":{"keywords":["mapping","geospatial"]}}),
            json!({"meta":{"keywords":["geospatial","visualization"]}}),
            json!({"meta":{"keywords":["mapping"]}}),
        ];
        let records = files.iter().zip(values.iter()).collect::<Vec<_>>();
        assert_eq!(
            top_topics(&records),
            [("mapping".into(), 3), ("geospatial".into(), 2), ("visualization".into(), 2)]
        );
        assert!(top_topics(&[]).is_empty());
    }
    #[test]
    fn test_tree_children_keep_folders_first_in_both_directions() {
        let paths = vec!["z.txt".into(), "b/index.json".into(), "a.txt".into(), "a/index.json".into()];
        assert_eq!(
            tree_children(&paths, "", false),
            vec![(false, "a".into()), (false, "b".into()), (true, "a.txt".into()), (true, "z.txt".into())]
        );
        assert_eq!(
            tree_children(&paths, "", true),
            vec![(false, "b".into()), (false, "a".into()), (true, "z.txt".into()), (true, "a.txt".into())]
        );
    }
    #[test]
    fn test_visible_file_paths_preserve_matching_hierarchy() {
        let files = vec![
            SuppliedFile {
                path: "Gold/index.json".into(),
                content: "{}".into(),
            },
            SuppliedFile {
                path: "Gold/banner.svg".into(),
                content: "<svg/>".into(),
            },
            SuppliedFile {
                path: "Other/notes.txt".into(),
                content: "notes".into(),
            },
        ];
        let gold = SearchPattern::new("gold", false, false).expect("literal query");
        let all = SearchPattern::new("", false, false).expect("empty query");
        assert_eq!(visible_file_paths(&files, &gold, "all", None), vec!["Gold/index.json", "Gold/banner.svg"]);
        assert_eq!(visible_file_paths(&files, &all, "image", None), vec!["Gold/banner.svg"]);
        assert_eq!(
            visible_file_paths(&files, &gold, "loadable", Some(Tool::Activity)),
            vec!["Gold/index.json"]
        );
        let case_sensitive = SearchPattern::new("gold", false, true).expect("literal query");
        assert!(visible_file_paths(&files, &case_sensitive, "all", None).is_empty());
        let regex = SearchPattern::new(r"Gold/.*\.svg$", true, true).expect("regex query");
        assert_eq!(visible_file_paths(&files, &regex, "all", None), vec!["Gold/banner.svg"]);
    }
}
