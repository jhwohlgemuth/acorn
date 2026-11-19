use dioxus::prelude::*;

#[component]
pub(crate) fn SearchField(
    label: &'static str,
    placeholder: &'static str,
    mut query: Signal<String>,
    mut regex: Signal<bool>,
    mut case_sensitive: Signal<bool>,
    error: Option<String>,
    #[props(default)] compact: bool,
) -> Element {
    rsx! { div { class: if compact { "search-control compact" } else { "search-control" },
        div { class: "field",
            span { "{label}" }
            div { class: "search-input",
                input { r#type: "search", value: query(), placeholder, "aria-label": label, oninput: move |event| query.set(event.value()) }
                div { class: "search-modes", role: "group", "aria-label": "{label} options",
                    button { r#type: "button", "aria-label": "Regular expression", title: "Use a regular expression", "aria-pressed": regex(), onclick: move |_| regex.toggle(), "Regex" }
                    button { r#type: "button", "aria-label": "Case sensitive", title: "Match uppercase and lowercase exactly", "aria-pressed": case_sensitive(), onclick: move |_| case_sensitive.toggle(), "Aa" }
                }
            }
        }
        if let Some(error) = error { p { class: "search-error", role: "alert", "Invalid regex: {error}" } }
    } }
}
