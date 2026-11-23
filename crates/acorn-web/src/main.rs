//! ACORN's Dioxus browser composition root
extern crate alloc;
pub(crate) use acorn_web::media;
use dioxus::prelude::*;

mod dashboard;
mod details;
mod geospatial_page;
mod location;
mod results;
mod sample;
mod search_ui;
mod skill;
mod syntax;
mod ui;
mod workspace;

#[derive(Clone, Copy)]
struct RecoveryTimedOut(Signal<bool>);

fn app() -> Element {
    let mut session = use_context_provider(|| Signal::new(String::new()));
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
        let saved = document::eval("if (!window.acornWorkspaceStorage) await new Promise(resolve => window.addEventListener('acorn-workspace-storage-ready', resolve, {once:true})); const storage = window.acornWorkspaceStorage; const workspace = await storage.restore(); return {workspace, recoveryTimedOut: storage.recoveryTimedOut}").join::<serde_json::Value>().await;
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
                let evaluator = document::eval(&format!("const storage = window.acornWorkspaceStorage; const saved = await storage.save({value}); if (saved) try {{ localStorage.setItem('acorn.workspace.changed.v1', crypto.randomUUID()); }} catch {{}} return {{...await storage.status(), newFailure: !saved}}"));
                spawn(async move {
                    let result = evaluator.join::<serde_json::Value>().await;
                    if let Ok(status) = result {
                        storage_status.set(ui::parse_storage_status(&status));
                        if status.get("newFailure") == Some(&serde_json::Value::Bool(true)) {
                            document::eval("window.acornChrome?.notifications.notify('Workspace saving stopped. Browser storage may be full, unavailable, or changed in another tab. Your current session remains open.', 'warning')");
                        }
                    }
                });
            }
        }
    });
    use_future(move || async move {
        loop {
            let changed = document::eval("return await new Promise(resolve => window.addEventListener('storage', function changed(event) { if (event.key !== 'acorn.workspace.changed.v1') return; window.removeEventListener('storage', changed); resolve(true); }))")
                .join::<bool>()
                .await;
            if changed.is_err() {
                break;
            }
            let saved = document::eval("return await window.acornWorkspaceStorage.restore()")
                .join::<serde_json::Value>()
                .await;
            let next = saved
                .as_ref()
                .ok()
                .and_then(|value| serde_json::from_value::<acorn_web::session::Workspace>(value.clone()).ok())
                .and_then(acorn_web::session::Workspace::restored);
            if let Some(next) = next {
                session.set(next.content());
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
    use_context_provider(|| Signal::new(None::<Result<acorn_web::geospatial::LocationIndex, String>>));
    rsx! {
        document::Link {
            id: "app-favicon",
            rel: "icon",
            r#type: "image/svg+xml",
            href: asset!("/assets/generated/acorn-app-icon.svg"),
            "data-dark-icon": asset!("/assets/generated/acorn-app-icon.svg"),
            "data-light-icon": asset!("/assets/generated/acorn-app-icon-light.svg"),
        }
        document::Link { rel: "license", title: "Lucide icon license", href: asset!("/assets/vendor/lucide.LICENSE") }
        document::Link { rel: "license", title: "Devicon icon license", href: asset!("/assets/vendor/devicon.LICENSE") }
        document::Link { rel: "license", title: "Simple Icons license", href: asset!("/assets/vendor/simple-icons.LICENSE") }
        document::Link { id: "app-stylesheet", rel: "stylesheet", href: asset!("/assets/app.css") }
        document::Script { src: asset!("/assets/url-state.js") }
        document::Script { src: asset!("/assets/effects.js"), "data-worker": asset!("/assets/worker.js") }
        document::Script { src: asset!("/assets/details.js") }
        document::Script { src: asset!("/assets/carousel.js") }
        document::Script { src: asset!("/assets/remote.js") }
        document::Script { src: asset!("/assets/workspace-storage.js") }
        document::Script { src: asset!("/assets/dashboard-layout.js") }
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
