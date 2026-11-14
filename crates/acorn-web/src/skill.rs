use crate::syntax::{CodeBlock, CopyButton};
use crate::ui::{Icon, Route};
use dioxus::prelude::*;
use serde_json::Value;

const SOURCE: &str = "https://code.ornl.gov/research-enablement/acorn/-/tree/main/crates/acorn-cli/assets/skills/acorn";
const VERSION: &str = env!("CARGO_PKG_VERSION");
#[component]
pub(crate) fn Install() -> Element {
    rsx! {
        p { class: "lede", "Choose the desktop app, the skill, or both." }
        div { class: "workflow-grid",
            div { class: "card", h2 { "🖥️ RAD Workspace desktop" } p { "The shared research workspace for Windows, macOS and Linux." } p { class: "status invalid", "Desktop packages are not published yet." } }
            div { class: "card", h2 { "📖 ACORN Skill" } p { "Local research workflows for a compatible coding agent, matched to this app release." } Link { to: Route::Page { page: "skill".into() }, "Open the skill guide and download →" } }
        }
        h2 { "Desktop channels" }
        for (name, description) in [("WinGet", "Windows · signed installer"), ("Scoop", "Windows · desktop archive"), ("Debian package", "Debian Linux · .deb"), ("AppImage", "Linux · portable application"), ("Homebrew cask", "macOS · notarized application")] {
            div { class: "card channel-row", div { h3 { "{name}" } p { class: "small muted", "{description}" } } span { class: "badge", "Pending" } }
        }
        p { class: "muted small", "No desktop version, architecture, checksum or installation command is advertised until its release is verified. These GUI channels are separate from the ACORN CLI." }
        div { class: "card", h2 { "🔒 Your files remain yours" } p { "App updates replace the bundled skill. Exported skills and research files remain user-managed. Installing or removing the app will not silently change agent directories or the separately installed CLI." } }
    }
}
#[component]
pub(crate) fn Skill(compact: bool) -> Element {
    let base = asset!("/assets/generated/skill").to_string();
    let mut bundle = use_signal(|| None::<Result<Value, String>>);
    let load_base = base.clone();
    use_future(move || {
        let base = load_base.clone();
        async move {
            let args = serde_json::to_string(&(base, VERSION)).unwrap_or_default();
            let script =
                format!("try {{ if (!window.acornSkill) await new Promise(resolve => window.addEventListener('acorn-effects-ready', resolve, {{once:true}})); return {{ok: await window.acornSkill.load(...{args})}}; }} catch(error) {{ return {{error: error.message}}; }}");
            let result = document::eval(&script)
                .join::<Value>()
                .await
                .map_err(|_| "Could not read the bundled skill. Reload the app and try again.".into())
                .and_then(|value| {
                    if let Some(message) = value.get("error").unwrap_or(&Value::Null).as_str() {
                        Err(message.into())
                    } else {
                        Ok(value.get("ok").unwrap_or(&Value::Null).clone())
                    }
                });
            bundle.set(Some(result));
        }
    });
    rsx! {
        if !compact { p { class: "lede", "Research metadata workflows for your agent. Bundled with app v{VERSION}." } }
        match bundle() {
            None => rsx! { p { role: "status", "Checking the bundled skill…" } },
            Some(Err(message)) => rsx! { div { class: "error", role: "alert", h2 { "Skill unavailable" } p { "{message}" } p { "Downloads are disabled until the app and its bundle match. Your exported copies are unchanged." } } },
            Some(Ok(value)) => {
                let definition = value.get("definition").unwrap_or(&Value::Null).as_str().unwrap_or_default().to_owned();
                let raw = definition.clone();
                let copied = definition.clone();
                let download_base = base.clone();
                rsx! {
                    p { class: "status", "App and bundled skill match · v{VERSION}" }
                    if !compact {
                        div { class: "card", h2 { "🚀 Install and start" }
                            ol { li { "Read the bundled definition and review its workflows." } li { "Download and extract the complete skill folder into your repository’s agent skill directory." } li { "Start a new compatible agent session and invoke the ACORN skill." } }
                            button { class: "primary", onclick: move |_| { let args = serde_json::to_string(&(download_base.clone(), VERSION)).unwrap_or_default(); document::eval(&format!("window.acornSkill.download(...{args})")); }, "Download skill folder" }
                            div { class: "actions", button { onclick: move |_| { let text = serde_json::to_string(&raw).unwrap_or_default(); document::eval(&format!("window.acornDownload({text}, 'SKILL.md', 'text/plain;charset=utf-8')")); }, "Download SKILL.md" }
                                CopyButton { text: copied, label: "Copy definition" }
                            }
                            p { id: "output-status", role: "status", class: "small muted" }
                            details { summary { "Read bundled SKILL.md" } CodeBlock { text: definition, language: "markdown" } }
                            a { href: SOURCE, target: "_blank", rel: "noopener noreferrer", "Canonical source ↗" }
                        }
                        WorkflowExamples {}
                        div { class: "card", h2 { "🔄 Keep versions together" } p { "Updating the app replaces its bundled skill. Exported and installed copies remain under your control: compare their version and digest, then explicitly replace them when upgrading." } p { "To roll back, use a retained matching app and skill release. To remove an exported skill, remove that copy from your agent’s skill directory. Clearing this browser session does not remove exported files or your CLI installation." } }
                    }
                    details { summary { "Bundled skill identity" } dl { class: "result-fields",
                        dt { "App version" } dd { "{VERSION}" }
                        dt { "Skill revision (SHA-256)" } dd { "{value[\"manifest\"][\"skillRevision\"].as_str().unwrap_or_default()}" }
                        dt { "Definition SHA-256" } dd { "{value[\"manifest\"][\"definition\"][\"sha256\"].as_str().unwrap_or_default()}" }
                        dt { "Archive SHA-256" } dd { "{value[\"manifest\"][\"archive\"][\"sha256\"].as_str().unwrap_or_default()}" }
                    } }
                }
            }
        }
    }
}
#[component]
fn WorkflowExamples() -> Element {
    let mut category = use_signal(|| "All");
    let examples = [
        (
            "Validate",
            "filecheck",
            "Validate metadata",
            "Check a research activity and understand what needs attention.",
            "Use $acorn to validate this research activity and explain each diagnostic before suggesting changes.",
            "research-activity",
        ),
        (
            "Convert",
            "diff",
            "Convert metadata",
            "Map metadata to DataCite and review any information lost in conversion.",
            "Use $acorn to convert this metadata to DataCite and explain any conversion losses.",
            "crosswalk",
        ),
        (
            "Inspect",
            "search",
            "Inspect identifiers",
            "Check identifier syntax locally, without resolving identifiers online.",
            "Use $acorn to inspect the identifiers in this document without resolving them on the network.",
            "inspect",
        ),
        (
            "Review",
            "book",
            "Review research text",
            "Assess readability and find ways to make research text clearer.",
            "Use $acorn to assess this research summary’s readability and suggest clearer wording.",
            "readability",
        ),
    ];
    rsx! {
        section { class: "workflow-examples", "aria-labelledby": "workflow-heading",
            h2 { id: "workflow-heading", "Workflow examples" }
            p { class: "small muted", "Choose a workflow and copy its prompt into your agent session." }
            div { class: "workflow-filters", role: "group", "aria-label": "Filter workflow examples",
                for filter in ["All", "Validate", "Convert", "Inspect", "Review"] {
                    button { r#type: "button", "aria-pressed": category() == filter, onclick: move |_| category.set(filter),
                        "{filter} " span { class: "filter-count", {examples.iter().filter(|example| filter == "All" || example.0 == filter).count().to_string()} }
                    }
                }
            }
            div { class: "workflow-grid",
                for (kind, icon, title, description, prompt, tool) in examples.into_iter().filter(|example| category() == "All" || example.0 == category()) {
                    article { key: "{title}", class: "card workflow-example", "data-category": kind, "aria-label": title,
                        div { class: "workflow-header",
                            span { class: "workflow-icon", Icon { name: icon } }
                            div { class: "workflow-intro", span { class: "workflow-category", "{kind}" } h3 { "{title}" } }
                            details { class: "workflow-menu", "name": "workflow-actions",
                                summary { "aria-label": "Actions for {title}", title: "Workflow actions", "⋯" }
                                div { class: "workflow-menu-items",
                                    Link { to: Route::Page { page: tool.into() }, Icon { name: "play" } "Open related tool" }
                                    button { r#type: "button", onclick: move |_| {
                                        let args = serde_json::to_string(&(prompt, format!("acorn-{tool}-prompt.txt"), "text/plain;charset=utf-8")).unwrap_or_default();
                                        document::eval(&format!("window.acornDownload(...{args}); document.querySelectorAll('.workflow-menu[open]').forEach(menu => menu.removeAttribute('open'))"));
                                    }, Icon { name: "download" } "Download prompt" }
                                }
                            }
                        }
                        p { class: "workflow-description muted", "{description}" }
                        CodeBlock { text: prompt, language: "prompt", label: "Copy workflow" }
                    }
                }
            }
        }
    }
}
