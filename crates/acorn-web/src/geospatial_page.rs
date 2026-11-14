use crate::details::{self, Detail};
use crate::ui::{Icon, Route};
use acorn_web::{
    geospatial::{self, LocationIndex, Marker, MARKER_LIMIT},
    session::Workspace,
};
use alloc::collections::BTreeSet;
use dioxus::prelude::*;
use serde_json::{json, Value};

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
    let mut index = use_signal(|| None::<Result<LocationIndex, String>>);
    let mut search = use_signal(String::new);
    let mut country = use_signal(String::new);
    let mut selected = use_signal(|| pins().selected);
    let mut error = use_signal(String::new);
    let mut descending = use_signal(|| false);
    use_future(move || async move {
        let url = serde_json::to_string(&asset!("/assets/geospatial/ror-locations.json").to_string()).unwrap_or_default();
        let script = format!("const response = await fetch({url}, {{credentials:'omit', redirect:'error'}}); if(!response.ok) throw new Error('Local registry unavailable'); const bytes = await response.arrayBuffer(); if(bytes.byteLength > 33554432) throw new Error('Local registry too large'); return JSON.parse(new TextDecoder().decode(bytes));");
        index.set(Some(document::eval(&script).join::<LocationIndex>().await.map_err(|_| {
            "The bundled ROR location index could not load. Reload the app to retry; no online registry was contacted.".into()
        })));
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
                    if let Some(marker) = geospatial::markers_with_identifiers(&workspace(), index, &pins().identifiers)
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
    let data = index();
    let snapshot = workspace();
    let all = data
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .map(|index| geospatial::markers_with_identifiers(&snapshot, index, &pins().identifiers))
        .unwrap_or_default();
    let countries = all
        .iter()
        .filter_map(|marker| marker.location.as_ref().map(|location| location.country.clone()))
        .collect::<BTreeSet<_>>();
    let query = search().to_lowercase();
    let mut filtered = all
        .iter()
        .filter(|marker| {
            (query.is_empty() || format!("{} {}", marker.name, marker.id).to_lowercase().contains(&query))
                && (country().is_empty() || marker.location.as_ref().is_some_and(|location| location.country == country()))
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
    let focused = all.iter().find(|marker| Some(&marker.id) == selected().as_ref());
    let payload = json!({"markers":filtered,"selected":selection});
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
                        if let Some(location) = &marker.location { small { "{location.country} · {location.latitude:.4}, {location.longitude:.4}" } }
                        else { small { if marker.resolved { "This ROR record has no usable coordinates in the local index." } else { "This ROR is not in the local index. No map location is available." } } }
                    }
                    button { r#type: "button", onclick: move |_| details::open(drawer, detail.clone()), "Inspect" }
                }
            }}
        }
        div { class: "options",
            label { class: "field", "Search organizations" input { r#type: "search", placeholder: "Search name or ROR identifier…", value: search(), oninput: move |event| search.set(event.value()) } }
            label { class: "field", "Country" select { value: country(), onchange: move |event| country.set(event.value()), option { value: "", "All countries" } for name in countries { option { value: "{name}", "{name}" } } } }
        }
        h2 { class: "section-heading", Icon { name: "globe" } "Organization locations" }
        div { id: "geospatial-map", class: "geospatial-map", hidden: !error().is_empty(), "aria-label": "Organization globe. The table below provides equivalent organization data." }
        if !error().is_empty() { p { role: "status", class: "status invalid", "{error}" } }
        match data {
            None => rsx! { p { role: "status", "Loading bundled organization locations…" } },
            Some(Err(message)) => rsx! { p { role: "alert", class: "error", "{message}" } },
            Some(Ok(index)) => rsx! {
                p { class: "small muted", "ROR {index.version} · {index.date} · {index.license}. Local NASA Black Marble imagery; no online map, terrain or registry services." }
                details { summary { "Dataset provenance" } p { a { href: index.source, target: "_blank", rel: "noopener noreferrer", "Pinned ROR data release ↗" } } p { class: "small digest", "Source JSON SHA-256: {index.source_sha256}" } a { href: asset!("/assets/geospatial/ROR_PROVENANCE.md"), "Generation and license details" } }
            },
        }
        p { "{all.len()} organizations · {all.iter().filter(|marker| !marker.resolved).count()} unresolved · {all.iter().filter(|marker| marker.resolved && marker.location.is_none()).count()} without coordinates" }
        if total > MARKER_LIMIT { p { class: "status invalid", "Showing {MARKER_LIMIT} of {total} matching organizations. Narrow the filters to see the remainder." } }
        if filtered.is_empty() && !all.is_empty() { div { class: "card empty", h2 { "No matching organizations" } p { "Try another search or reset the filters." } button { onclick: move |_| { search.set(String::new()); country.set(String::new()); }, "Reset filters" } } }
        table { class: "organization-table", caption { "{filtered.len()} matching organizations" }
            thead { tr { th { scope: "col", "aria-sort": if descending() { "descending" } else { "ascending" }, button { onclick: move |_| descending.toggle(), "Organization" } } th { scope: "col", "Country" } th { scope: "col", "Coordinates" } th { scope: "col", "References" } } }
            tbody { for marker in &filtered { {let id = marker.id.clone(); let detail = Detail::organization(marker); rsx! {
                tr { td { button { "aria-pressed": selection.as_ref() == Some(&id), onclick: move |_| { selected.set(Some(id.clone())); details::open(drawer, detail.clone()); }, "{marker.name}" } small { a { href: marker.id.clone(), target: "_blank", rel: "noopener noreferrer", "{marker.id}" } } }
                    if let Some(location) = &marker.location { td { "{location.country}" } td { "{location.latitude:.4}, {location.longitude:.4}" } }
                    else { td { "Not available" } td { if marker.resolved { "No coordinates" } else { "Not in local registry" } } }
                    td { "{marker.references}" }
                }
            }} } }
        }
        if filtered.is_empty() && !snapshot.activities().is_empty() { p { "No referenced ROR organizations match this view." } }
    }
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
