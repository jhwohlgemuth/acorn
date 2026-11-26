//! ACORN's Dioxus browser composition root
extern crate alloc;
pub(crate) use acorn_web::media;
pub(crate) use acorn_web::url_intent;
use dioxus::prelude::*;

mod dashboard;
mod details;
mod geospatial_page;
mod icon;
mod location;
mod research_graph;
mod results;
mod sample;
mod script;
mod search_ui;
mod skill;
mod syntax;
mod ui;
mod window_sync;
mod workspace;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
struct RecoveryTimedOut(Signal<bool>);

fn app() -> Element {
    let mut session = use_context_provider(|| Signal::new(String::new()));
    let mut url_entry = use_context_provider(|| Signal::new(None::<url_intent::UrlEntry>));
    use_context_provider(ui::ToolSessions::new);
    let initially_collapsed = js_sys::Reflect::get(&js_sys::global(), &"acornInitialSidebarCollapsed".into())
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    use_context_provider(|| Signal::new(initially_collapsed));
    let mut workspace = use_context_provider(|| Signal::new(acorn_web::session::Workspace::default()));
    let mut storage_status = use_context_provider(|| Signal::new((0_usize, false, 0_usize, 0_usize)));
    let mut restored = use_signal(|| false);
    let mut skip_save = use_signal(|| false);
    let mut recovery_timed_out = use_context_provider(|| RecoveryTimedOut(Signal::new(false))).0;
    use_future(move || async move {
        let mut channel = script::url_entry_channel();
        while let Ok(entry) = channel.recv::<url_intent::UrlEntry>().await {
            url_entry.set(Some(entry));
        }
    });
    use_future(move || async move {
        let saved = script::restore_workspace_snapshot().await;
        recovery_timed_out.set(
            saved
                .as_ref()
                .ok()
                .and_then(|value| value.get("recoveryTimedOut"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        );
        let restorable = saved
            .ok()
            .and_then(|value| value.get("workspace").cloned())
            .and_then(|value| serde_json::from_value::<acorn_web::session::Workspace>(value).ok())
            .and_then(acorn_web::session::Workspace::restored);
        if let Some(mut checked) = restorable {
            checked.validations = ui::validate_files(
                checked
                    .files
                    .iter()
                    .filter(|file| checked.validations.contains_key(&file.path))
                    .cloned()
                    .collect(),
            )
            .await
            .into_iter()
            .map(|(path, result)| {
                (
                    path,
                    result.unwrap_or_else(|message| serde_json::json!({"valid": false, "error": message})),
                )
            })
            .collect();
            session.set(checked.content());
            workspace.set(checked);
        }
        restored.set(true);
    });
    use_effect(move || {
        if restored() {
            let current = workspace();
            if *skip_save.peek() {
                skip_save.set(false);
            } else if let Ok(value) = serde_json::to_string(&current) {
                window_sync::publish_json("acorn.workspace.live.v1", &value);
                let evaluator = script::save_workspace(&value);
                spawn(async move {
                    let result = evaluator.await;
                    if let Ok(status) = result {
                        storage_status.set(ui::parse_storage_status(&status));
                        if status.get("newFailure") == Some(&serde_json::Value::Bool(true)) {
                            script::notify_workspace_save_failure("Workspace saving stopped. Browser storage may be full, unavailable, or changed in another tab. Your current session remains open.");
                        } else {
                            window_sync::notify_storage("acorn.workspace.changed.v1");
                        }
                    }
                });
            }
        }
    });
    use_future(move || async move {
        let mut channel = window_sync::subscribe("acorn.workspace.live.v1");
        while let Ok(value) = channel.recv::<serde_json::Value>().await {
            if !restored() {
                continue;
            }
            if let Some(next) = serde_json::from_value::<acorn_web::session::Workspace>(value)
                .ok()
                .and_then(acorn_web::session::Workspace::restored)
            {
                if next.selected.is_some() || next.files.is_empty() {
                    let content = next.content();
                    if *session.peek() != content {
                        session.set(content);
                    }
                }
                if *workspace.peek() != next {
                    skip_save.set(true);
                    workspace.set(next);
                }
            }
        }
    });
    use_future(move || async move {
        let mut changes = window_sync::subscribe_storage("acorn.workspace.changed.v1");
        while changes.recv::<bool>().await.is_ok() {
            let saved = script::restore_saved_workspace().await;
            let next = saved
                .as_ref()
                .ok()
                .and_then(|value| serde_json::from_value::<acorn_web::session::Workspace>(value.clone()).ok())
                .and_then(acorn_web::session::Workspace::restored);
            if let Some(next) = next {
                if next.selected.is_some() || next.files.is_empty() {
                    let content = next.content();
                    if *session.peek() != content {
                        session.set(content);
                    }
                }
                if *workspace.peek() != next {
                    skip_save.set(true);
                    workspace.set(next);
                }
            } else if matches!(saved, Ok(serde_json::Value::Null)) {
                session.set(String::new());
                let empty = acorn_web::session::Workspace::default();
                if *workspace.peek() != empty {
                    skip_save.set(true);
                    workspace.set(empty);
                }
            }
        }
    });
    use_context_provider(|| Signal::new(None::<details::Detail>));
    use_context_provider(|| Signal::new(geospatial_page::MapPins::default()));
    use_context_provider(|| Signal::new(geospatial_page::LocationIndexCache::default()));
    rsx! {
        document::Link {
            id: "app-favicon",
            rel: "icon",
            r#type: "image/svg+xml",
            href: asset!("/assets/generated/dark/app-icon.svg"),
            "data-dark-icon": asset!("/assets/generated/dark/app-icon.svg"),
            "data-light-icon": asset!("/assets/generated/light/app-icon.svg"),
            "data-uswds-icon": asset!("/assets/generated/uswds/app-icon.svg"),
        }
        document::Link { rel: "license", title: "USWDS icon license", href: asset!("/assets/vendor/uswds.LICENSE") }
        document::Link { rel: "license", title: "Lucide icon license", href: asset!("/assets/vendor/lucide.LICENSE") }
        document::Link { rel: "license", title: "Devicon icon license", href: asset!("/assets/vendor/devicon.LICENSE") }
        document::Link { rel: "license", title: "Simple Icons license", href: asset!("/assets/vendor/simple-icons.LICENSE") }
        document::Link { id: "app-stylesheet", rel: "stylesheet", href: asset!("/assets/app.css") }
        document::Script { src: asset!("/assets/url-state.js") }
        document::Script { r#type: "module", src: asset!("/assets/window-sync.js") }
        document::Script { src: asset!("/assets/effects.js"), "data-worker": asset!("/assets/worker.js") }
        document::Script { src: asset!("/assets/details.js") }
        document::Script { src: asset!("/assets/carousel.js") }
        document::Script { src: asset!("/assets/remote.js") }
        document::Script { src: asset!("/assets/workspace-storage.js") }
        document::Script { src: asset!("/assets/dashboard-layout.js") }
        document::Script { src: asset!("/assets/graph.js") }
        document::Script { src: asset!("/assets/syntax.js"), "data-prism": asset!("/assets/vendor/prism.js") }
        if restored() {
            if recovery_timed_out() {
                div { class: "startup-recovery-warning error", role: "alert",
                    p { "Browser storage did not finish loading. Your saved data was left untouched, and this session cannot save changes. See Browser storage in Settings." }
                }
            }
            Router::<ui::Route> {}
        } else {
            main { class: "page", role: "status", "Restoring saved workspace…" }
        }
    }
}
fn main() {
    // The operation worker loads this same payload; only a document hosts the interface.
    if js_sys::Reflect::has(&js_sys::global(), &"document".into()).unwrap_or(false) {
        dioxus::launch(app);
    }
}
