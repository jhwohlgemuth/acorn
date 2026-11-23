use crate::ui::Icon;
use dioxus::prelude::*;

const EDITOR_ASSETS: Asset = asset!("/assets/generated/editor");
#[component]
pub(crate) fn Code(text: String, language: String, #[props(default)] copyable: bool) -> Element {
    let rendered = use_resource(use_reactive((&text, &language), |(text, language)| async move {
        let args = serde_json::to_string(&(text, language)).unwrap_or_default();
        document::eval(&format!("if (!window.acornSyntax) await new Promise(resolve => window.addEventListener('acorn-syntax-ready', resolve, {{once:true}})); return window.acornSyntax.highlight(...{args});"))
            .join::<String>()
            .await
            .ok()
    }));
    rsx! {
        span { class: if copyable { "inline-code" } else { "code-text" },
            code { class: "syntax language-{language}",
                if let Some(Some(html)) = rendered.read().as_ref() { span { dangerous_inner_html: html.clone() } }
                else { "{text}" }
            }
            if copyable { CopyButton { text } }
        }
    }
}
#[component]
pub(crate) fn CodeBlock(text: String, language: String, #[props(default = "Copy code")] label: &'static str) -> Element {
    rsx! {
        div { class: "code-block",
            div { class: "code-toolbar", CopyButton { text: text.clone(), label } }
            div { class: "code-viewport",
                pre { Code { text, language } }
                div { class: "line-numbers", "aria-hidden": "true" }
            }
        }
    }
}
#[component]
pub(crate) fn CodeEditor(
    id: &'static str,
    label: String,
    value: String,
    language: String,
    rows: u32,
    #[props(default)] readonly: bool,
    #[props(default)] diagnostics: Vec<serde_json::Value>,
    #[props(default)] fullscreen: bool,
    #[props(default)] run_action: bool,
    #[props(default)] run_disabled: bool,
    #[props(default)] run_error: Option<String>,
    oninput: EventHandler<String>,
) -> Element {
    let mut edited = use_signal(|| None::<String>);
    use_effect(use_reactive(
        (&value, &language, &label, &diagnostics),
        move |(value, language, label, diagnostics)| {
            // Echoed keystrokes must not replace newer edits or reset the editor's selection.
            let replacement = if edited.peek().as_ref() == Some(&value) {
                None
            } else {
                edited.set(None);
                Some(value.clone())
            };
            let args = serde_json::to_string(&(id, replacement, language, label, diagnostics, value)).unwrap_or_default();
            document::eval(&format!("const [id, replacement, language, label, diagnostics, source] = {args}; const versions = window.acornEditorSyncVersions ||= {{}}; const version = versions[id] = (versions[id] || 0) + 1; if (!window.acornEditors) await new Promise(resolve => window.addEventListener('acorn-editors-ready', resolve, {{once:true}})); if (versions[id] === version && document.getElementById(id)?.value === source) {{ window.acornEditors.sync(id, replacement, language, label); window.acornEditors.setDiagnostics(id, diagnostics, source); }}"));
        },
    ));
    rsx! {
        document::Link { rel: "license", title: "CodeMirror dependency licenses", href: "{EDITOR_ASSETS}/LICENSES.txt" }
        document::Script { src: "{EDITOR_ASSETS}/editor.js" }
        div { class: "code-editor", "data-language": language, style: "--editor-height:calc({rows} * 1.6em + 20px)",
            div { class: "editor-toolbar", role: "group", "aria-label": "{label} editor controls",
                if fullscreen { span { class: "editor-fullscreen-name", "{label}" } }
                button { r#type: "button", class: "copy-code editor-icon-action", "aria-label": "Fold all", title: "Fold all regions (Ctrl+Alt+[)", onclick: move |_| { document::eval(&format!("window.acornEditors?.fold('{id}')")); }, Icon { name: "fold" } }
                button { r#type: "button", class: "copy-code editor-icon-action", "aria-label": "Expand all", title: "Expand all regions (Ctrl+Alt+])", onclick: move |_| { document::eval(&format!("window.acornEditors?.expand('{id}')")); }, Icon { name: "expand" } }
                button { r#type: "button", class: "copy-code editor-icon-action editor-wrap-toggle", "aria-label": "Wrap lines", "aria-pressed": "true", title: "Disable word wrap", onclick: move |_| { document::eval(&format!("window.acornEditors?.toggleWrap('{id}')")); }, Icon { name: "wrap" } }
                button { r#type: "button", class: "copy-code editor-icon-action", "aria-label": "Find", title: "Find in editor (Ctrl+F)", onclick: move |_| { document::eval(&format!("window.acornEditors?.openFind('{id}')")); }, Icon { name: "search" } }
                div { class: "editor-font-size", role: "group", "aria-label": "Editor font size",
                    button { r#type: "button", class: "copy-code", "data-size": "small", "aria-label": "Small", title: "Small font size", "aria-pressed": "true", onclick: move |_| { document::eval(&format!("window.acornEditors?.setFontSize('{id}', 'small')")); }, Icon { name: "font-small" } }
                    button { r#type: "button", class: "copy-code", "data-size": "big", "aria-label": "Big", title: "Big font size", "aria-pressed": "false", onclick: move |_| { document::eval(&format!("window.acornEditors?.setFontSize('{id}', 'big')")); }, Icon { name: "font-big" } }
                }
                CopyButton { text: value.clone(), icon_only: true }
                span { class: "editor-toolbar-spacer", "aria-hidden": "true" }
                if let Some(message) = run_error { span { class: "editor-run-error", title: "{message}", "{message}" } }
                div { class: "editor-issue-nav", role: "group", "aria-label": "Issue navigation", hidden: true,
                    span { class: "editor-issue-label", "Issues" }
                    button { r#type: "button", class: "copy-code", title: "Previous issue (←)", "aria-label": "Previous issue", "aria-keyshortcuts": "ArrowLeft", onclick: move |_| { document::eval(&format!("window.acornEditors?.navigateIssue('{id}', -1)")); }, "←" }
                    span { class: "editor-issue-position", "aria-live": "polite", "0 of 0" }
                    button { r#type: "button", class: "copy-code", title: "Next issue (→)", "aria-label": "Next issue", "aria-keyshortcuts": "ArrowRight", onclick: move |_| { document::eval(&format!("window.acornEditors?.navigateIssue('{id}', 1)")); }, "→" }
                    span { class: "editor-issue-message", role: "status", "aria-live": "polite" }
                }
                if run_action && run_disabled { span { class: "editor-run-status", role: "status", "aria-live": "polite", "Running…" } }
                if run_action { span { class: "editor-run-group",
                    button { r#type: "button", class: "primary editor-run-action", disabled: run_disabled, "aria-keyshortcuts": "Control+R", title: "Run (Ctrl+R)", onclick: move |_| { document::eval("document.getElementById('tool-run')?.click()"); }, Icon { name: "play" } "Run" }
                } }
                if fullscreen { span { class: "editor-fullscreen-group",
                    button { r#type: "button", class: "copy-code editor-fullscreen-toggle editor-icon-action", "aria-label": "Full screen", title: "Full screen (Esc to exit)", onclick: move |_| { document::eval(&format!("window.acornEditors?.toggleFullscreen('{id}')")); }, span { class: "enter-fullscreen-icon", Icon { name: "maximize" } } span { class: "exit-fullscreen-icon", Icon { name: "minimize" } } span { class: "enter-fullscreen-label", "Full screen" } span { class: "exit-fullscreen-label", "Exit full screen" } }
                } }
            }
            div { class: "editor-surface" }
            textarea { id, "aria-label": label, rows: "{rows}", readonly, spellcheck: "false", autocapitalize: "off", autocomplete: "off", value, oninput: move |event| {
                let value = event.value();
                edited.set(Some(value.clone()));
                oninput.call(value);
            } }
        }
    }
}
#[component]
pub(crate) fn CopyButton(text: String, #[props(default = "Copy code")] label: &'static str, #[props(default)] icon_only: bool) -> Element {
    rsx! {
        button { class: if icon_only { "copy-code editor-icon-action" } else { "copy-code" }, r#type: "button", "aria-label": label, title: "Copy to clipboard", onclick: move |_| {
            let encoded = serde_json::to_string(&text).unwrap_or_default();
            document::eval(&format!("window.acornCopy({encoded})"));
        }, Icon { name: "copy" } if !icon_only { "{label}" } }
    }
}
