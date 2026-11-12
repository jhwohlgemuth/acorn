use crate::ui::Route;
use dioxus::prelude::*;
use serde_json::Value;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SOURCE: &str = "https://code.ornl.gov/research-enablement/acorn/-/tree/main/crates/acorn-cli/assets/skills/acorn";
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
                        div { class: "card", h2 { "Install and start" }
                            ol { li { "Read the bundled definition and review its workflows." } li { "Download and extract the complete skill folder into your repository’s agent skill directory." } li { "Start a new compatible agent session and invoke the ACORN skill." } }
                            button { class: "primary", onclick: move |_| { let args = serde_json::to_string(&(download_base.clone(), VERSION)).unwrap_or_default(); document::eval(&format!("window.acornSkill.download(...{args})")); }, "Download skill folder" }
                            div { class: "actions", button { onclick: move |_| { let text = serde_json::to_string(&raw).unwrap_or_default(); document::eval(&format!("window.acornDownload({text}, 'SKILL.md', 'text/plain;charset=utf-8')")); }, "Download SKILL.md" }
                                button { onclick: move |_| { let text = serde_json::to_string(&copied).unwrap_or_default(); document::eval(&format!("window.acornCopy({text})")); }, "Copy definition" }
                            }
                            p { id: "output-status", role: "status", class: "small muted" }
                            details { summary { "Read bundled SKILL.md" } pre { "{definition}" } }
                            a { href: SOURCE, target: "_blank", rel: "noopener noreferrer", "Canonical source ↗" }
                        }
                        h2 { "Use the skill" }
                        div { class: "workflow-grid", for (title, prompt) in [
                            ("Validate metadata", "Use $acorn to validate this research activity and explain each diagnostic before suggesting changes."),
                            ("Convert metadata", "Use $acorn to convert this metadata to DataCite and explain any conversion losses."),
                            ("Inspect identifiers", "Use $acorn to inspect the identifiers in this document without resolving them on the network."),
                            ("Review research text", "Use $acorn to assess this research summary’s readability and suggest clearer wording."),
                        ] { div { class: "card", h3 { "{title}" } p { "{prompt}" } button { onclick: move |_| { let text = serde_json::to_string(prompt).unwrap_or_default(); document::eval(&format!("window.acornCopy({text})")); }, "Copy workflow" } } } }
                        div { class: "card", h2 { "Keep versions together" } p { "Updating the app replaces its bundled skill. Exported and installed copies remain under your control: compare their version and digest, then explicitly replace them when upgrading." } p { "To roll back, use a retained matching app and skill release. To remove an exported skill, remove that copy from your agent’s skill directory. Clearing this browser session does not remove exported files or your CLI installation." } }
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
pub(crate) fn Install() -> Element {
    rsx! {
        p { class: "lede", "Choose the desktop app, the skill, or both." }
        div { class: "workflow-grid",
            div { class: "card", h2 { "ACORN desktop" } p { "The shared research workspace for Windows, macOS and Linux." } p { class: "status invalid", "Desktop packages are not published yet." } }
            div { class: "card", h2 { "ACORN Skill" } p { "Local research workflows for a compatible coding agent, matched to this app release." } Link { to: Route::Page { page: "skill".into() }, "Open the skill guide and download →" } }
        }
        h2 { "Desktop channels" }
        for (name, description) in [("WinGet", "Windows · signed installer"), ("Scoop", "Windows · desktop archive"), ("Debian package", "Debian Linux · .deb"), ("AppImage", "Linux · portable application"), ("Homebrew cask", "macOS · notarized application")] {
            div { class: "card channel-row", div { h3 { "{name}" } p { class: "small muted", "{description}" } } span { class: "badge", "Pending" } }
        }
        p { class: "muted small", "No desktop version, architecture, checksum or installation command is advertised until its release is verified. These GUI channels are separate from the ACORN CLI." }
        div { class: "card", h2 { "Your files remain yours" } p { "App updates replace the bundled skill. Exported skills and research files remain user-managed. Installing or removing the app will not silently change agent directories or the separately installed CLI." } }
    }
}
