use crate::details::{self, Detail};
use crate::search_ui::SearchField;
use crate::ui::{Icon, Route};
use acorn_core::util::suffix;
use acorn_web::{
    geospatial::{self, LocationIndex, Marker, MARKER_LIMIT},
    research_index::ResearchIndex,
    search::SearchPattern,
    session::Workspace,
};
use alloc::collections::BTreeSet;
use dioxus::prelude::*;
use serde_json::Value;

#[derive(Clone, Default)]
pub(crate) struct MapPins {
    pub(crate) identifiers: BTreeSet<String>,
    selected: Option<String>,
}
#[component]
pub(crate) fn Geospatial() -> Element {
    let workspace = use_context::<Signal<Workspace>>();
    let mut pins = use_context::<Signal<MapPins>>();
    let drawer = use_context::<Signal<Option<Detail>>>();
    let mut index = use_context::<Signal<Option<Result<LocationIndex, String>>>>();
    let mut search = use_signal(String::new);
    let regex = use_signal(|| false);
    let case_sensitive = use_signal(|| false);
    let mut country = use_signal(String::new);
    let mut project = use_signal(String::new);
    let mut selected = use_signal(|| pins().selected);
    let mut expanded_projects = use_signal(|| None::<String>);
    let mut error = use_signal(String::new);
    let mut descending = use_signal(|| false);
    use_future(move || async move {
        // The session keeps one parsed registry; only a missing or failed load fetches again.
        if !index.peek().as_ref().is_some_and(Result::is_ok) {
            load_index(index).await;
        }
    });
    use_future(move || async move {
        let base = asset!("/assets/generated/geospatial").to_string();
        #[cfg(debug_assertions)]
        let module = option_env!("ACORN_MAP_DEV_URL")
            .map(|url| format!("{url}/src/main.tsx"))
            .unwrap_or_else(|| format!("{base}/map.js"));
        #[cfg(not(debug_assertions))]
        let module = format!("{base}/map.js");
        let css = if module.ends_with(".tsx") {
            String::new()
        } else {
            format!("{base}/map.css")
        };
        let args = serde_json::to_string(&(module, css)).unwrap_or_default();
        let mut channel = document::eval(&format!(
            "if (!window.acornMap) await new Promise(resolve => window.addEventListener('acorn-effects-ready', resolve, {{once:true}})); await window.acornMap.connect(...{args}, event => dioxus.send(event)); await dioxus.recv();"
        ));
        while let Ok(event) = channel.recv::<Value>().await {
            if event.get("type").unwrap_or(&Value::Null) == "select" {
                let id = event.get("id").and_then(Value::as_str).map(str::to_owned);
                selected.set(id.clone());
                if let Some(Ok(index)) = index.peek().as_ref() {
                    if let Some(marker) = geospatial::markers_with_identifiers(&workspace.read(), index, &pins.read().identifiers)
                        .iter()
                        .find(|marker| Some(&marker.id) == id.as_ref())
                    {
                        details::open(drawer, Detail::organization(marker));
                    }
                }
            } else if event.get("type").unwrap_or(&Value::Null) == "error" {
                error.set(
                    event
                        .get("message")
                        .unwrap_or(&Value::Null)
                        .as_str()
                        .unwrap_or("Map unavailable. Use the table below.")
                        .into(),
                );
            }
        }
    });
    use_drop(move || {
        document::eval("window.acornMap?.disconnect()");
    });
    let data = index.read();
    let snapshot = workspace.read();
    let catalog = ResearchIndex::from(&*snapshot);
    let active_project = project();
    let active_project = if catalog.projects.iter().any(|item| item.id == active_project) {
        active_project
    } else {
        String::new()
    };
    let all = data
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .map(|index| geospatial::markers_with_identifiers(&snapshot, index, &pins.read().identifiers))
        .unwrap_or_default();
    let countries = all
        .iter()
        .filter_map(|marker| marker.location.as_ref().map(|location| location.country.clone()))
        .collect::<BTreeSet<_>>();
    let pattern = SearchPattern::new(&search(), regex(), case_sensitive());
    let mut filtered = all
        .iter()
        .filter(|marker| {
            pattern
                .as_ref()
                .is_ok_and(|pattern| pattern.matches(&format!("{} {}", marker.name, marker.id)))
                && (country().is_empty() || marker.location.as_ref().is_some_and(|location| location.country == country()))
                && (active_project.is_empty() || marker.project_ids.contains(&active_project))
        })
        .cloned()
        .collect::<Vec<_>>();
    let sort = |left: &Marker, right: &Marker| {
        let order = left.name.cmp(&right.name).then(left.id.cmp(&right.id));
        if descending() {
            order.reverse()
        } else {
            order
        }
    };
    filtered.sort_by(sort);
    let total = filtered.len();
    let selected_position = filtered.iter().position(|marker| Some(&marker.id) == selected().as_ref());
    if let Some(position) = selected_position.filter(|position| *position >= MARKER_LIMIT) {
        let chosen = filtered.remove(position);
        filtered.truncate(MARKER_LIMIT - 1);
        filtered.push(chosen);
        filtered.sort_by(sort);
    } else {
        filtered.truncate(MARKER_LIMIT);
    }
    let selection = selected().filter(|id| filtered.iter().any(|marker| &marker.id == id));
    let focused = filtered.iter().find(|marker| Some(&marker.id) == selection.as_ref());
    let organization_noun = format!("organization{}", suffix(filtered.len()));
    let payload = geospatial::map_payload(&filtered, &catalog.projects, selection.as_deref());
    use_effect(use_reactive((&payload,), move |(payload,)| {
        document::eval(&format!("window.acornMap?.update({payload})"));
    }));
    rsx! {
        p { class: "lede", "Organizations from validated activities and selected ROR identifiers · local ROR lookup" }
        if snapshot.activities().is_empty() && pins().identifiers.is_empty() { div { class: "card empty", Icon { name: "globe" } h2 { "No validated activities" } p { "Load and validate research activities to explore their organization locations." } Link { to: Route::Page { page: "portfolio".into() }, "Open Portfolio →" } } }
        if !pins().identifiers.is_empty() {
            div { class: "map-pins actions",
                span { class: "small muted", "{pins().identifiers.len()} added from identifiers" }
                button { r#type: "button", onclick: move |_| { pins.set(MapPins::default()); selected.set(None); }, "Clear added locations" }
            }
        }
        if let Some(marker) = focused {
            {let detail = Detail::organization(marker); rsx! {
                div { class: "map-focus", role: "status",
                    Icon { name: "globe" }
                    div { strong { "{marker.name}" } small { "{marker.id}" }
                        if !marker.project_ids.is_empty() { small { "Associated projects: {marker.project_ids.len()}" } }
                        if let Some(location) = &marker.location { small { "{location.country} · {location.latitude:.4}, {location.longitude:.4}" } }
                        else { small { if marker.resolved { "This ROR record has no usable coordinates in the local index." } else { "This ROR is not in the local index. No map location is available." } } }
                    }
                    button { r#type: "button", onclick: move |_| details::open(drawer, detail.clone()), "Inspect" }
                }
            }}
        }
        div { class: "options",
            label { class: "field", "Project" select { value: active_project, onchange: move |event| project.set(event.value()), option { value: "", "All projects" } for item in &catalog.projects { option { value: "{item.id}", "{item.title}" } } } }
            SearchField { label: "Search organizations", placeholder: "Search name or ROR identifier…", query: search, regex, case_sensitive, error: pattern.as_ref().err().cloned() }
            label { class: "field", "Country" select { value: country(), onchange: move |event| country.set(event.value()), option { value: "", "All countries" } for name in countries { option { value: "{name}", "{name}" } } } }
        }
        h2 { class: "section-heading", Icon { name: "globe" } "Organization locations" }
        div { id: "geospatial-map", class: "geospatial-map", hidden: !error().is_empty(), "aria-label": "Organization globe. The table below provides equivalent organization data." }
        if !error().is_empty() { p { role: "status", class: "status invalid", "{error}" } }
        match &*data {
            None => rsx! { p { role: "status", "Loading bundled organization locations…" } },
            Some(Err(message)) => rsx! { div { p { role: "alert", class: "error", "{message}" } button { r#type: "button", onclick: move |_| { index.set(None); spawn(load_index(index)); }, "Retry local index" } } },
            Some(Ok(index)) => rsx! {
                p { class: "small muted", "ROR {index.version} · {index.date} · {index.license}. Local NASA Black Marble imagery; no online map, terrain or registry services." }
                details { summary { "Dataset provenance" } p { a { href: index.source.clone(), target: "_blank", rel: "noopener noreferrer", "Pinned ROR data release ↗" } } p { class: "small digest", "Source JSON SHA-256: {index.source_sha256}" } a { href: asset!("/assets/geospatial/ROR_PROVENANCE.md"), "Generation and license details" } }
            },
        }
        p { "{catalog.projects.len()} projects · {all.len()} organizations · {all.iter().filter(|marker| !marker.resolved).count()} unresolved · {all.iter().filter(|marker| marker.resolved && marker.location.is_none()).count()} without coordinates" }
        if total > MARKER_LIMIT { p { class: "status invalid", "Showing {MARKER_LIMIT} of {total} matching organizations. Narrow the filters to see the remainder." } }
        if filtered.is_empty() && !all.is_empty() { div { class: "card empty", h2 { "No matching organizations" } p { "Try another search or reset the filters." } button { onclick: move |_| { search.set(String::new()); country.set(String::new()); project.set(String::new()); }, "Reset filters" } } }
        table { class: "organization-table", caption { "{filtered.len()} matching {organization_noun}" }
            thead { tr { th { scope: "col", "aria-sort": if descending() { "descending" } else { "ascending" }, button { onclick: move |_| descending.toggle(), "Organization and projects" } } th { scope: "col", "Country" } th { scope: "col", "Coordinates" } th { scope: "col", "References" } } }
            tbody { for marker in &filtered { {let id = marker.id.clone(); let detail = Detail::organization(marker); let expanded = expanded_projects().as_ref() == Some(&marker.id); let expand_id = marker.id.clone(); rsx! {
                tr { td { button { "aria-pressed": selection.as_ref() == Some(&id), onclick: move |_| { selected.set(Some(id.clone())); details::open(drawer, detail.clone()); }, "{marker.name}" } small { a { href: marker.id.clone(), target: "_blank", rel: "noopener noreferrer", "{marker.id}" } }
                    if !marker.project_ids.is_empty() { small { "Projects ({marker.project_ids.len()}):" } for project_id in marker.project_ids.iter().take(if expanded { marker.project_ids.len() } else { 5 }) { if let Some(item) = catalog.projects.iter().find(|item| &item.id == project_id) { {let project_detail = Detail::from(item); rsx! { button { r#type: "button", onclick: move |_| details::open(drawer, project_detail.clone()), "{item.title}" } } } } }
                        if marker.project_ids.len() > 5 { button { r#type: "button", onclick: move |_| expanded_projects.set(if expanded { None } else { Some(expand_id.clone()) }), if expanded { "Show fewer projects" } else { "Show all projects" } } }
                    }
                }
                    if let Some(location) = &marker.location { td { "{location.country}" } td { "{location.latitude:.4}, {location.longitude:.4}" } }
                    else { td { "Not available" } td { if marker.resolved { "No coordinates" } else { "Not in local registry" } } }
                    td { "{marker.references}" }
                }
            }} } }
        }
        if filtered.is_empty() && !snapshot.activities().is_empty() { p { "No referenced ROR organizations match this view." } }
    }
}
async fn load_index(mut index: Signal<Option<Result<LocationIndex, String>>>) {
    let url = serde_json::to_string(&asset!("/assets/geospatial/ror-locations.json").to_string()).unwrap_or_default();
    let script = format!("const response = await fetch({url}, {{credentials:'omit', redirect:'error'}}); if(!response.ok) throw new Error('Local registry unavailable'); const bytes = await response.arrayBuffer(); if(bytes.byteLength > 33554432) throw new Error('Local registry too large'); return JSON.parse(new TextDecoder().decode(bytes));");
    index.set(Some(document::eval(&script).join::<LocationIndex>().await.map_err(|_| {
        "The bundled ROR location index could not load. Retry to read the local asset again; no online registry was contacted.".into()
    })));
}
#[component]
pub(crate) fn RorMapButton(identifier: String) -> Element {
    let mut pins = use_context::<Signal<MapPins>>();
    let mut drawer = use_context::<Signal<Option<Detail>>>();
    let nav = use_navigator();
    let Some(id) = geospatial::canonical_ror(&identifier) else {
        return rsx! {};
    };
    rsx! {
        button { r#type: "button", class: "ror-map-button", title: "View this organization using the local ROR location index",
            onclick: move |_| {
                let mut next = pins();
                next.identifiers.insert(id.clone());
                next.selected = Some(id.clone());
                pins.set(next);
                drawer.set(None);
                nav.push(Route::Page { page: "geospatial".into() });
            },
            Icon { name: "globe" } "View on map"
        }
    }
}
