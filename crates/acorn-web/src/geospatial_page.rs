use crate::details::{self, Detail};
use crate::script;
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

/// Retained ROR subset and the identifiers used to load it.
#[derive(Default)]
pub(crate) struct LocationIndexCache {
    identifiers: BTreeSet<String>,
    result: Option<Result<LocationIndex, String>>,
    request: u64,
}
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
    let index = use_context::<Signal<LocationIndexCache>>();
    let mut search = use_signal(String::new);
    let regex = use_signal(|| false);
    let case_sensitive = use_signal(|| false);
    let mut country = use_signal(String::new);
    let mut project = use_signal(String::new);
    let mut selected = use_signal(|| pins().selected);
    let mut expanded_projects = use_signal(|| None::<String>);
    let mut error = use_signal(String::new);
    let mut descending = use_signal(|| false);
    let requested_identifiers = geospatial::referenced_identifiers(&workspace.read(), &pins.read().identifiers);
    use_effect(use_reactive((&requested_identifiers,), move |(identifiers,)| {
        reload_index(index, identifiers, false);
    }));
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
        let mut channel = script::map_channel(&module, &css);
        while let Ok(event) = channel.recv::<Value>().await {
            if event.get("type").unwrap_or(&Value::Null) == "select" {
                let id = event.get("id").and_then(Value::as_str).map(str::to_owned);
                selected.set(id.clone());
                if let Some(Ok(index)) = index.peek().result.as_ref() {
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
        script::disconnect_map();
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
        .result
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
        script::update_map(&payload);
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
        match data.result.as_ref() {
            None => rsx! { p { role: "status", "Loading bundled organization locations…" } },
            Some(Err(message)) => rsx! { div { p { role: "alert", class: "error", "{message}" } button { r#type: "button", onclick: move |_| reload_index(index, geospatial::referenced_identifiers(&workspace.read(), &pins.read().identifiers), true), "Retry local index" } } },
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
fn needs_index_load(cache: &LocationIndexCache, identifiers: &BTreeSet<String>, retry: bool) -> bool {
    retry || cache.identifiers != *identifiers || cache.result.is_none()
}
fn reload_index(mut cache: Signal<LocationIndexCache>, identifiers: BTreeSet<String>, retry: bool) {
    let (reload, serial) = {
        let state = cache.peek();
        (needs_index_load(&state, &identifiers, retry), state.request.saturating_add(1))
    };
    if reload {
        {
            let mut state = cache.write();
            state.identifiers = identifiers.clone();
            state.result = None;
            state.request = serial;
        }
        spawn(async move {
            let url = asset!("/assets/geospatial/ror-locations.json").to_string();
            let worker = asset!("/assets/scripts/ror-index-worker.js").to_string();
            let result = script::load_ror_index(&url, &worker, &identifiers).await.map_err(|_| {
                "The bundled ROR location index could not load. Retry to read the local asset again; no online registry was contacted.".into()
            });
            if cache.peek().request == serial {
                let mut state = cache.write();
                state.result = Some(result);
            }
        });
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
        button { r#type: "button", class: "ror-map-button", "aria-label": "View on map", title: "Open in Geospatial using the local ROR location index",
            onclick: move |_| {
                let mut next = pins();
                next.identifiers.insert(id.clone());
                next.selected = Some(id.clone());
                pins.set(next);
                drawer.set(None);
                nav.push(Route::Page { page: "geospatial".into() });
            },
            Icon { name: "globe" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;

    #[test]
    fn test_location_index_cache_reuses_matching_subset_and_retries_on_request() {
        let identifiers = BTreeSet::from(["https://ror.org/01qz5mb56".to_string()]);
        let index = LocationIndex {
            version: "test".into(),
            date: "test".into(),
            source: "test".into(),
            source_sha256: "test".into(),
            license: "test".into(),
            organizations: BTreeMap::new(),
        };
        let mut cache = LocationIndexCache {
            identifiers: identifiers.clone(),
            result: Some(Ok(index)),
            ..LocationIndexCache::default()
        };
        assert!(!needs_index_load(&cache, &identifiers, false));
        assert!(needs_index_load(&cache, &BTreeSet::new(), false));
        assert!(needs_index_load(&cache, &identifiers, true));
        cache.result = Some(Err("failed".into()));
        assert!(!needs_index_load(&cache, &identifiers, false));
        assert!(needs_index_load(&cache, &identifiers, true));
        cache.result = None;
        assert!(needs_index_load(&cache, &identifiers, false));
    }
}
