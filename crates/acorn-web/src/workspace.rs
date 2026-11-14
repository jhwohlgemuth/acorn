use crate::details::{self, Detail};
use crate::ui::{Icon, Route, UrlFileInput};
use acorn_web::{
    operations::{self, Request, Tool},
    presentation::text,
    session::{self, Selection, SuppliedFile, Workspace},
};
use alloc::collections::BTreeSet;
use dioxus::prelude::*;
use serde_json::{json, Value};

#[derive(Clone, Default, serde::Deserialize)]
struct BucketHistory {
    repositories: Vec<String>,
    refs: Vec<String>,
}
#[component]
fn Distribution(title: &'static str, counts: alloc::collections::BTreeMap<String, usize>) -> Element {
    let total = counts.values().sum::<usize>();
    rsx! { div { class: "card", h2 { "{title}" } if total == 0 { p { class: "muted", "No matching activities" } } for (label, count) in counts { div { class: "proportion", p { span { "{label}" } span { "{count}" } } progress { value: "{count}", max: "{total}", "aria-label": "{label}: {count} of {total} activities" } } } } }
}
#[component]
pub(crate) fn FileTree(paths: Vec<String>, prefix: String) -> Element {
    let workspace = use_context::<Signal<Workspace>>();
    let drawer = use_context::<Signal<Option<Detail>>>();
    let children = paths
        .iter()
        .filter_map(|path| path.strip_prefix(&prefix))
        .filter_map(|relative| relative.split('/').next())
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    rsx! { ul { class: "file-tree", for name in children {
        {let path = format!("{prefix}{name}");
        let is_file = paths.contains(&path);
        rsx! { li {
            if is_file { button { class: "file-leaf", "aria-pressed": workspace.read().selected.as_deref() == Some(path.as_str()), onclick: move |_| { let snapshot = workspace(); if let Some(file) = snapshot.files.iter().find(|file| file.path == path) { details::open(drawer, Detail::file(file, &snapshot)); } }, span { "aria-hidden": "true", "📄" } "{name}" } }
            else { details { summary { Icon { name: "folder-open" } "{name}" } FileTree { paths: paths.clone(), prefix: format!("{path}/") } } }
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
    let mut remote_loading = use_signal(|| false);
    let mut history = use_signal(BucketHistory::default);
    use_future(move || async move {
        if let Ok(saved) = document::eval("if (!window.acornRemote) await new Promise(resolve => window.addEventListener('acorn-remote-ready', resolve, {once:true})); return window.acornRemote.history.read();").join::<BucketHistory>().await {
            history.set(saved);
        }
    });
    use_future(move || async move {
        let mut channel = document::eval("if (!window.acornWorkspace) await new Promise(resolve => window.addEventListener('acorn-effects-ready', resolve, {once:true})); window.acornWorkspace.connect(batch => dioxus.send(batch)); await dioxus.recv();");
        while let Ok(batch) = channel.recv::<Selection>().await {
            if batch.files.is_empty() {
                message.set(format!(
                    "No files loaded; {} entries skipped or unreadable. Your previous session is unchanged.",
                    batch.skipped
                ));
            } else {
                let next = workspace().replace(batch);
                message.set(format!(
                    "{} files loaded; {} entries skipped. Choose Portfolio to validate and explore activities.",
                    next.files.len(),
                    next.skipped
                ));
                input.set(next.content());
                workspace.set(next);
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
                let next = workspace().replace(Selection { files: vec![file], skipped: 0 });
                input.set(next.content()); workspace.set(next); message.set("File loaded. Choose Portfolio to validate and explore activities.".into()); close_url();
            } }
        } }
        p { class: "load-limits small muted", "JSON, YAML, Markdown or text · 256 KiB per file · 500 files · 16 MiB total" }
        details { class: "remote-entry", summary {
            "Open a " span { class: "repository-name repository-github", img { src: asset!("/assets/generated/github.svg"), alt: "", width: "16", height: "16" } "GitHub" }
            " or " span { class: "repository-name", img { src: asset!("/assets/generated/gitlab.svg"), alt: "", width: "16", height: "16" } "GitLab" } " bucket"
        }
        form { class: "remote-loader", onsubmit: move |event| async move {
            event.prevent_default();
            if remote_loading() { return; }
            remote_loading.set(true); remote_error.set(String::new());
            let args = serde_json::to_string(&(remote_url(), remote_ref())).unwrap_or_default();
            let result = document::eval(&format!("if (!window.acornRemote) await new Promise(resolve => window.addEventListener('acorn-remote-ready', resolve, {{once:true}})); let result; try {{ result = {{ok: await window.acornRemote.load(...{args})}}; }} catch(error) {{ result = {{error: error.message}}; }} return {{...result, history: window.acornRemote.history.read()}};")).join::<Value>().await;
            match result {
                Ok(value) => {
                    if let Ok(saved) = serde_json::from_value::<BucketHistory>(value.get("history").cloned().unwrap_or(Value::Null)) { history.set(saved); }
                    if let Some(error) = value.get("error").and_then(Value::as_str) { remote_error.set(error.to_owned()); }
                    else {
                        match serde_json::from_value::<Selection>(value.get("ok").cloned().unwrap_or(Value::Null)) {
                            Ok(selection) => { let next = workspace().replace(selection); message.set(format!("{} remote files loaded; {} entries skipped. Open Portfolio to validate activities.", next.files.len(), next.skipped)); input.set(next.content()); workspace.set(next); },
                            Err(_) => remote_error.set("The repository returned an invalid file selection. Your previous session is unchanged.".into()),
                        }
                    }
                },
                Err(_) => remote_error.set("Could not load this repository. Your previous session is unchanged.".into()),
            }
            remote_loading.set(false);
        },
            label { class: "field", "Repository or URL" input { r#type: "text", required: true, disabled: remote_loading(), value: remote_url(), oninput: move |event| remote_url.set(event.value()), list: "bucket-repositories", autocomplete: "off", placeholder: "owner/repo or https://gitlab.com/group/project", spellcheck: "false" } }
            datalist { id: "bucket-repositories", for repository in &history.read().repositories { option { value: repository.clone() } } }
            datalist { id: "bucket-refs", for reference in &history.read().refs { option { value: reference.clone() } } }
            div { class: "remote-actions", label { class: "field", "Branch / tag / commit (optional)" input { disabled: remote_loading(), value: remote_ref(), oninput: move |event| remote_ref.set(event.value()), list: "bucket-refs", autocomplete: "off", placeholder: "Repository default", spellcheck: "false" } }
                button { r#type: "submit", disabled: remote_loading() || remote_url().trim().is_empty(), Icon { name: "download" } if remote_loading() { "Loading…" } else { "Load bucket" } }
                if remote_loading() { button { r#type: "button", onclick: move |_| { document::eval("window.acornRemote?.cancel()"); }, "Cancel" } }
            }
            p { class: "small muted", "Public repositories, folder or file URLs. For branches containing /, enter the full branch above. Loading replaces this session after a successful download." }
            div { class: "actions small muted", span { "Recent inputs are remembered in this browser." }
                button { r#type: "button", class: "small", disabled: remote_loading() || (history.read().repositories.is_empty() && history.read().refs.is_empty()), onclick: move |_| {
                    document::eval("window.acornRemote?.history.clear()"); history.set(BucketHistory::default());
                }, "Clear suggestions" }
            }
            if remote_loading() { p { role: "status", "Downloading research files…" } }
            if !remote_error().is_empty() { p { class: "error", role: "alert", "{remote_error}" } }
        }
        }
        if !message().is_empty() { p { role: "status", class: "small", "{message}" } }
        if !workspace().files.is_empty() { details { class: "loaded-tree", summary { "{workspace().files.len()} loaded files" } FileTree { paths: workspace.read().files.iter().map(|file| file.path.clone()).collect(), prefix: String::new() } } Link { to: Route::Page { page: "portfolio".into() }, "Explore Portfolio →" } }
    }
}
#[component]
pub(crate) fn Portfolio() -> Element {
    let mut workspace = use_context::<Signal<Workspace>>();
    let drawer = use_context::<Signal<Option<Detail>>>();
    let mut search = use_signal(String::new);
    let mut kind = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut sort = use_signal(|| "title");
    let mut descending = use_signal(|| false);
    let mut selected = use_signal(|| None::<String>);
    let snapshot = workspace();
    let activities = snapshot.activities();
    let types = session::distribution(&activities, session::activity_type);
    let statuses = session::distribution(&activities, session::activity_status);
    let query = search().to_lowercase();
    let mut filtered = activities
        .iter()
        .copied()
        .filter(|(file, value)| {
            (query.is_empty() || format!("{} {}", text(value, "title"), file.path).to_lowercase().contains(&query))
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
        let order = key(left).cmp(&key(right)).then(left_file.path.cmp(&right_file.path));
        if descending() {
            order.reverse()
        } else {
            order
        }
    });
    let selection = filtered
        .iter()
        .find(|(file, _)| Some(&file.path) == selected().as_ref())
        .or_else(|| filtered.first())
        .copied();
    let invalid = snapshot.validations.values().filter(|value| value["valid"] != true).count();
    rsx! {
        p { class: "lede", "Research activities in this session · {activities.len()} validated records" }
        if snapshot.files.is_empty() {
            div { class: "card empty", Icon { name: "portfolio" } h2 { "No research activities loaded" } p { "Open local content or try the bundled sample from Home." } Link { to: Route::Home {}, "Open Home →" } }
        } else {
            details { summary { "{snapshot.files.len()} loaded files · {snapshot.bytes()} UTF-8 bytes" } FileTree { paths: snapshot.files.iter().map(|file| file.path.clone()).collect(), prefix: String::new() } }
            div { class: "actions", button { class: "primary", onclick: move |_| {
                let mut next = workspace();
                next.validations = next.files.iter().filter(|file| file.path.to_lowercase().ends_with(".json")).map(|file| {
                    let result = operations::run(Tool::Activity, &Request { input: file.content.clone(), ..Request::default() }).unwrap_or_else(|message| json!({"valid":false,"error":message}));
                    (file.path.clone(), result)
                }).collect();
                workspace.set(next);
            }, "Validate loaded activities" } }
            p { class: "small muted", "Validation reads JSON files only and runs when requested. Other files remain available in the loaded tree. {invalid} invalid or unreadable activities." }
            if activities.is_empty() { p { class: "status invalid", "No validated activities yet. Validate the loaded JSON files, or inspect a file in Research Activity." } }
            label { class: "field", "Search activities" input { r#type: "search", value: search(), oninput: move |event| search.set(event.value()), placeholder: "Search title or source file…" } }
            div { class: "options",
                label { class: "field", "Activity type" select { value: kind(), onchange: move |event| kind.set(event.value()), option { value: "", "All activity types" } for name in types.keys() { option { value: "{name}", "{name}" } } } }
                label { class: "field", "Status" select { value: status(), onchange: move |event| status.set(event.value()), option { value: "", "All statuses" } for name in statuses.keys() { option { value: "{name}", "{name}" } } } }
            }
            div { class: "metrics",
                for (label, count) in [("Activities", filtered.len()), ("Organizations", session::entities(&filtered, "organizations").len()), ("Contributors", session::entities(&filtered, "contributors").len()), ("Identifiers", session::identifiers(&filtered).len())] {
                    div { class: "card metric", strong { "{count}" } small { "{label}" } }
                }
            }
            div { class: "distribution-grid", Distribution { title: "Activity types", counts: session::distribution(&filtered, session::activity_type) } Distribution { title: "Statuses", counts: session::distribution(&filtered, session::activity_status) } }
            table { class: "activity-table", caption { "{filtered.len()} matching activities" }
                thead { tr { for (field, label) in [("title", "Activity"), ("type", "Type"), ("status", "Status")] {
                    th { scope: "col", "aria-sort": if sort() == field { if descending() { "descending" } else { "ascending" } } else { "none" }, button { onclick: move |_| { if sort() == field { descending.toggle(); } else { sort.set(field); descending.set(false); } }, "{label}" } }
                } } }
                tbody { for (file, value) in &filtered { {let path = file.path.clone(); let detail = Detail::file(file, &snapshot); rsx! {
                    tr { td { button { class: "activity-select", "aria-pressed": selection.is_some_and(|(chosen, _)| chosen.path == path), onclick: move |_| { selected.set(Some(path.clone())); details::open(drawer, detail.clone()); }, "{text(value, \"title\")}" } small { "{file.path}" } } td { "{session::activity_type(value)}" } td { "{session::activity_status(value)}" } }
                }} } }
            }
            if filtered.is_empty() && !activities.is_empty() { div { class: "card empty", h2 { "No matching activities" } p { "Try another search or reset the filters." } button { onclick: move |_| { search.set(String::new()); kind.set(String::new()); status.set(String::new()); }, "Reset filters" } } }

        }
    }
}
