use crate::ui::Route;
use acorn_web::{
    operations::{self, Request, Tool},
    presentation::text,
    session::{self, Selection, Workspace},
};
use alloc::collections::BTreeSet;
use dioxus::prelude::*;
use serde_json::{json, Value};

#[component]
pub(crate) fn Loader() -> Element {
    let mut workspace = use_context::<Signal<Workspace>>();
    let mut input = use_context::<Signal<String>>();
    let mut message = use_signal(String::new);
    let mut choose = use_signal(|| false);
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
        p { class: "muted", "Open research activity data" }
        div { id: "local-entry", class: "load-tiles",
            div { class: "load-tile",
                button { class: "tile-action", "aria-expanded": choose(), "aria-controls": "loader-choices", onclick: move |_| choose.toggle(), strong { "Open file or folder" } small { "Choose local research content" } }
                if choose() { div { id: "loader-choices", class: "actions", "aria-label": "Choose content source",
                    button { onclick: move |_| { choose.set(false); document::eval("window.acornWorkspace.pick('file')"); }, "Files" }
                    button { onclick: move |_| { choose.set(false); document::eval("window.acornWorkspace.pick('folder')"); }, "Folder" }
                } }
            }
            div { class: "load-tile", "data-workspace-drop": "true", strong { "Drop content" } small { "Files stay on this device" } p { class: "small muted", "Drop files or a folder here" } }
        }
        p { class: "small muted", "JSON, YAML, Markdown or text · 256 KiB per file · 500 files · 16 MiB total" }
        if !message().is_empty() { p { role: "status", class: "small", "{message}" } }
        if !workspace().files.is_empty() { details { class: "loaded-tree", summary { "{workspace().files.len()} loaded files" } FileTree { paths: workspace.read().files.iter().map(|file| file.path.clone()).collect(), prefix: String::new() } } Link { to: Route::Page { page: "portfolio".into() }, "Explore Portfolio →" } }
    }
}
#[component]
pub(crate) fn FileTree(paths: Vec<String>, prefix: String) -> Element {
    let mut workspace = use_context::<Signal<Workspace>>();
    let mut input = use_context::<Signal<String>>();
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
            if is_file { button { class: "file-leaf", "aria-pressed": workspace.read().selected.as_deref() == Some(path.as_str()), onclick: move |_| { let mut next = workspace(); next.selected = Some(path.clone()); input.set(next.content()); workspace.set(next); }, "{name}" } }
            else { details { summary { "{name}" } FileTree { paths: paths.clone(), prefix: format!("{path}/") } } }
        }}}
    } } }
}
#[component]
pub(crate) fn Portfolio() -> Element {
    let mut workspace = use_context::<Signal<Workspace>>();
    let mut input = use_context::<Signal<String>>();
    let mut search = use_signal(String::new);
    let mut kind = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut sort = use_signal(|| "title");
    let mut descending = use_signal(|| false);
    let mut selected = use_signal(|| None::<String>);
    let nav = use_navigator();
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
            div { class: "card empty", h2 { "No research activities loaded" } p { "Open local content or try the bundled sample from Home." } Link { to: Route::Home {}, "Open Home →" } }
        } else {
            details { summary { "{snapshot.files.len()} loaded files · {snapshot.bytes()} UTF-8 bytes" } FileTree { paths: snapshot.files.iter().map(|file| file.path.clone()).collect(), prefix: String::new() } }
            div { class: "actions", button { onclick: move |_| {
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
                    div { class: "card metric", small { "{label}" } strong { "{count}" } }
                }
            }
            div { class: "distribution-grid", Distribution { title: "Activity types", counts: session::distribution(&filtered, session::activity_type) } Distribution { title: "Statuses", counts: session::distribution(&filtered, session::activity_status) } }
            table { class: "activity-table", caption { "{filtered.len()} matching activities" }
                thead { tr { for (field, label) in [("title", "Activity"), ("type", "Type"), ("status", "Status")] {
                    th { scope: "col", "aria-sort": if sort() == field { if descending() { "descending" } else { "ascending" } } else { "none" }, button { onclick: move |_| { if sort() == field { descending.toggle(); } else { sort.set(field); descending.set(false); } }, "{label}" } }
                } } }
                tbody { for (file, value) in &filtered { {let path = file.path.clone(); rsx! {
                    tr { td { button { class: "activity-select", "aria-pressed": selection.is_some_and(|(chosen, _)| chosen.path == path), onclick: move |_| selected.set(Some(path.clone())), "{text(value, \"title\")}" } small { "{file.path}" } } td { "{session::activity_type(value)}" } td { "{session::activity_status(value)}" } }
                }} } }
            }
            if filtered.is_empty() && !activities.is_empty() { p { "No activities match these filters." } }
            if let Some((file, value)) = selection {
                {let path = file.path.clone(); let content = file.content.clone(); rsx! {
                    section { class: "card selected-activity", h2 { "{text(value, \"title\")}" }
                        if value["subtitle"].is_string() { p { "{text(value, \"subtitle\")}" } }
                        div { class: "comparison-flow", span { "{session::many(&value[\"meta\"][\"contributors\"]).len()} contributors" } span { "↔ Activity ↔" } span { "{session::many(&value[\"meta\"][\"organizations\"]).len()} organizations" } }
                        button { class: "primary", onclick: move |_| { input.set(content.clone()); let mut next = workspace(); next.selected = Some(path.clone()); workspace.set(next); nav.push(Route::Page { page: "research-activity".into() }); }, "Open selected activity" }
                    }
                }}
            }
        }
    }
}
#[component]
fn Distribution(title: &'static str, counts: alloc::collections::BTreeMap<String, usize>) -> Element {
    let total = counts.values().sum::<usize>();
    rsx! { div { class: "card", h2 { "{title}" } if total == 0 { p { class: "muted", "No matching activities" } } for (label, count) in counts { div { class: "proportion", p { span { "{label}" } span { "{count}" } } progress { value: "{count}", max: "{total}", "aria-label": "{label}: {count} of {total} activities" } } } } }
}
