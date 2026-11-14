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
    oninput: EventHandler<String>,
) -> Element {
    let mut edited = use_signal(|| None::<String>);
    use_effect(use_reactive((&value, &language, &label), move |(value, language, label)| {
        // Echoed keystrokes must not replace newer edits or reset the editor's selection.
        let replacement = if edited.peek().as_ref() == Some(&value) {
            None
        } else {
            edited.set(None);
            Some(value)
        };
        let args = serde_json::to_string(&(id, replacement, language, label)).unwrap_or_default();
        document::eval(&format!("window.acornEditors?.sync(...{args});"));
    }));
    rsx! {
        document::Link { rel: "license", title: "CodeMirror dependency licenses", href: "{EDITOR_ASSETS}/LICENSES.txt" }
        document::Script { src: "{EDITOR_ASSETS}/editor.js" }
        div { class: "code-editor", "data-language": language, style: "--editor-height:calc({rows} * 1.6em + 20px)",
            div { class: "editor-toolbar", role: "group", "aria-label": "{label} editor controls",
                button { r#type: "button", class: "copy-code", title: "Fold all regions (Ctrl+Alt+[)", onclick: move |_| { document::eval(&format!("window.acornEditors?.fold('{id}')")); }, Icon { name: "fold" } "Fold all" }
                button { r#type: "button", class: "copy-code", title: "Expand all regions (Ctrl+Alt+])", onclick: move |_| { document::eval(&format!("window.acornEditors?.expand('{id}')")); }, Icon { name: "expand" } "Expand all" }
                CopyButton { text: value.clone() }
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
pub(crate) fn CopyButton(text: String, #[props(default = "Copy code")] label: &'static str) -> Element {
    rsx! {
        button { class: "copy-code", r#type: "button", title: "Copy to clipboard", onclick: move |_| {
            let encoded = serde_json::to_string(&text).unwrap_or_default();
            document::eval(&format!("window.acornCopy({encoded})"));
        }, Icon { name: "copy" } "{label}" }
    }
}
