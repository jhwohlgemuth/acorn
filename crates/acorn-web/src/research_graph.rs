//! Accessible browser view over the portable RAD graph projection.
use crate::icon::IconName;
use crate::script;
use crate::search_ui::SearchField;
use crate::ui::{Icon, Route};
use acorn_schema::{
    pid::{Arxiv, Handle, Identifier, PersistentIdentifier, PersistentIdentifierParse, DOI, ISNI, PID, RRID, SWHID},
    research_activity::{
        graph::{GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind, GraphRecord, ResearchGraph},
        ResearchActivity,
    },
};
use acorn_web::{search::SearchPattern, session::Workspace};
use alloc::collections::{BTreeMap, BTreeSet};
use dioxus::prelude::*;
use rustworkx_core::{
    centrality::{betweenness_centrality, closeness_centrality, degree_centrality},
    petgraph::algo::{connected_components, dijkstra},
    petgraph::graph::{NodeIndex, UnGraph},
};

pub(crate) const DIAGRAM_EDGE_LIMIT: usize = 160;
pub(crate) const DIAGRAM_NODE_LIMIT: usize = 80;
const EDGE_KINDS: [(&str, &str); 6] = [
    ("", "All relationships"),
    ("contains", "Contains"),
    ("credits", "Credits"),
    ("hasIdentifier", "Has identifier"),
    ("involves", "Involves"),
    ("relatesTo", "Explicitly relates to"),
];
const NODE_KINDS: [(&str, &str); 7] = [
    ("", "All entities"),
    ("activity", "Activities"),
    ("contributor", "Contributors"),
    ("identifier", "Identifiers"),
    ("organization", "Organizations"),
    ("project", "Projects"),
    ("reference", "Unresolved references"),
];
#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct GraphMetrics {
    pub(crate) clustering: f64,
    pub(crate) degree: f64,
    pub(crate) closeness: f64,
    pub(crate) betweenness: f64,
}
#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct GraphParameters {
    pub(crate) average_clustering: f64,
    pub(crate) average_shortest_path: f64,
    pub(crate) components: usize,
    pub(crate) density: f64,
    pub(crate) diameter: usize,
}
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct MetricSpectrum {
    pub(crate) min: f64,
    pub(crate) max: f64,
    pub(crate) position: f64,
}
pub(crate) fn analyze_graph(graph: &ResearchGraph) -> BTreeMap<String, GraphMetrics> {
    let (network, indices) = graph_network(graph);
    let degree = degree_centrality(&network, None);
    let closeness = closeness_centrality(&network, true, usize::MAX);
    let betweenness = betweenness_centrality(&network, false, true, usize::MAX);
    graph
        .nodes
        .iter()
        .filter_map(|node| {
            indices.get(node.id.as_str()).map(|index| {
                let ordinal = index.index();
                (
                    node.id.clone(),
                    GraphMetrics {
                        clustering: clustering_coefficient(&network, *index),
                        degree: degree.get(ordinal).copied().unwrap_or_default(),
                        closeness: closeness.get(ordinal).copied().flatten().unwrap_or_default(),
                        betweenness: betweenness.get(ordinal).copied().flatten().unwrap_or_default(),
                    },
                )
            })
        })
        .collect()
}
fn clustering_coefficient(network: &UnGraph<(), ()>, node: NodeIndex) -> f64 {
    let neighbors: Vec<_> = network.neighbors(node).collect();
    let possible = neighbors.len().saturating_mul(neighbors.len().saturating_sub(1)) / 2;
    let connected = neighbors
        .iter()
        .enumerate()
        .map(|(index, neighbor)| {
            neighbors
                .iter()
                .skip(index.saturating_add(1))
                .filter(|other| network.contains_edge(*neighbor, **other))
                .count()
        })
        .sum::<usize>();
    if possible == 0 {
        0.0
    } else {
        connected as f64 / possible as f64
    }
}
fn edge_ends((x1, y1): (f64, f64), (x2, y2): (f64, f64)) -> (f64, f64, f64, f64) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let length = (dx * dx + dy * dy).sqrt().max(1.0);
    (
        x1 + dx * 16.0 / length,
        y1 + dy * 16.0 / length,
        x2 - dx * 19.0 / length,
        y2 - dy * 19.0 / length,
    )
}
pub(crate) fn entity_href(node: &GraphNode, graph: &ResearchGraph) -> Option<String> {
    if matches!(node.kind, GraphNodeKind::Identifier | GraphNodeKind::Reference) {
        pid_href(&node.label)
    } else {
        graph
            .edges
            .iter()
            .filter(|edge| edge.kind == GraphEdgeKind::HasIdentifier && edge.source == node.id)
            .filter_map(|edge| {
                graph
                    .nodes
                    .iter()
                    .find(|target| target.id == edge.target)
                    .and_then(|target| pid_href(&target.label))
            })
            .next()
    }
}
fn entity_title(graph: Option<&ResearchGraph>, id: &str) -> Element {
    let node = graph.and_then(|graph| graph.nodes.iter().find(|node| node.id == id));
    let label = node.map_or(id, |node| node.label.as_str());
    let href = graph.zip(node).and_then(|(graph, node)| entity_href(node, graph));
    if let Some(href) = href {
        rsx! { a { class: "graph-entity-link", href, target: "_blank", rel: "noopener noreferrer", title: "Open PID website for {label}", onclick: move |event| event.stop_propagation(), "{label}" Icon { name: "external" } } }
    } else {
        rsx! { "{label}" }
    }
}
fn focus_radius_limit(graph: Option<&ResearchGraph>, parameters: Option<GraphParameters>) -> usize {
    parameters.map_or_else(
        || graph.map_or(1, |graph| graph.nodes.len().saturating_sub(1).max(1)),
        |parameters| parameters.diameter.max(1),
    )
}
pub(crate) fn graph_analysis_within_budget(graph: &ResearchGraph) -> bool {
    graph.nodes.len() <= DIAGRAM_NODE_LIMIT && graph.edges.len() <= DIAGRAM_EDGE_LIMIT
}
fn graph_network(graph: &ResearchGraph) -> (UnGraph<(), ()>, BTreeMap<&str, NodeIndex>) {
    let mut network = UnGraph::<(), ()>::new_undirected();
    let indices: BTreeMap<_, _> = graph.nodes.iter().map(|node| (node.id.as_str(), network.add_node(()))).collect();
    let connections: BTreeSet<_> = graph
        .edges
        .iter()
        .filter_map(|edge| {
            indices
                .get(edge.source.as_str())
                .zip(indices.get(edge.target.as_str()))
                .and_then(|(&source, &target)| (source != target).then_some(if source < target { (source, target) } else { (target, source) }))
        })
        .collect();
    for (source, target) in connections {
        network.add_edge(source, target, ());
    }
    (network, indices)
}
pub(crate) fn graph_parameters(graph: &ResearchGraph) -> GraphParameters {
    let (network, _) = graph_network(graph);
    let node_count = network.node_count();
    let (path_count, distance_sum, diameter) = network.node_indices().fold((0_usize, 0.0_f64, 0_usize), |totals, source| {
        dijkstra(&network, source, None, |_| 1_usize)
            .into_iter()
            .filter_map(|(target, distance)| (target > source).then_some(distance))
            .fold(totals, |(count, sum, maximum), distance| {
                (count.saturating_add(1), sum + distance as f64, maximum.max(distance))
            })
    });
    let average_clustering = if node_count > 0 {
        network.node_indices().map(|node| clustering_coefficient(&network, node)).sum::<f64>() / node_count as f64
    } else {
        0.0
    };
    let average_shortest_path = if path_count == 0 { 0.0 } else { distance_sum / path_count as f64 };
    let density = if node_count > 1 {
        let count = node_count as f64;
        2.0 * network.edge_count() as f64 / (count * (count - 1.0))
    } else {
        0.0
    };
    GraphParameters {
        average_clustering,
        average_shortest_path,
        components: connected_components(&network),
        density,
        diameter,
    }
}

#[component]
fn GraphMetricCard(label: &'static str, metric: &'static str, score: f64, spectrum: MetricSpectrum) -> Element {
    let value = format!("{score:.2}");
    let minimum = format!("{:.2}", spectrum.min);
    let maximum = format!("{:.2}", spectrum.max);
    let position = format!("--graph-spectrum-position: {:.2}%;", spectrum.position);
    let description = format!("{label}: {value}. Smallest {minimum}; largest {maximum}.");
    rsx! {
        div { "data-metric": metric,
            dt { "{label}" }
            dd {
                span { class: "graph-score", "{value}" }
                div { class: "graph-spectrum-track", role: "img", "aria-label": "{description}", style: "{position}",
                    span { class: "graph-spectrum-marker", "aria-hidden": "true" }
                }
                div { class: "graph-spectrum-scale", "aria-hidden": "true",
                    span { "Smallest {minimum}" }
                    span { "Largest {maximum}" }
                }
            }
        }
    }
}
pub(crate) fn keywords(workspace: &Workspace) -> BTreeMap<String, Vec<String>> {
    workspace
        .activities()
        .into_iter()
        .map(|(file, value)| {
            let values = match value.get("meta").and_then(|meta| meta.get("keywords")) {
                | Some(serde_json::Value::String(value)) => vec![value.clone()],
                | Some(serde_json::Value::Array(values)) => values.iter().filter_map(serde_json::Value::as_str).map(ToString::to_string).collect(),
                | _ => Vec::new(),
            };
            (file.path.clone(), values)
        })
        .collect()
}
pub(crate) fn metric_spectrum(analyses: &BTreeMap<String, GraphMetrics>, score: f64, metric: fn(&GraphMetrics) -> f64) -> MetricSpectrum {
    let (min, max) = analyses
        .values()
        .map(metric)
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), value| (min.min(value), max.max(value)));
    let (min, max) = if min.is_finite() && max.is_finite() { (min, max) } else { (0.0, 0.0) };
    let position = if max > min {
        ((score - min) / (max - min) * 100.0).clamp(0.0, 100.0)
    } else {
        50.0
    };
    MetricSpectrum { min, max, position }
}
fn node_icon_name(kind: GraphNodeKind) -> &'static str {
    match kind {
        | GraphNodeKind::Activity => "file",
        | GraphNodeKind::Contributor => "graph-contributor",
        | GraphNodeKind::Identifier => "",
        | GraphNodeKind::Organization => "graph-organization",
        | GraphNodeKind::Project => "folder-open",
        | GraphNodeKind::Reference => "link",
    }
}
pub(crate) fn pid_href(value: &str) -> Option<String> {
    let identifier = Identifier::new(value).normalized();
    identifier.and_then(|identifier| {
        let href = match identifier.kind {
            | PID::Arxiv => Arxiv::from_string(&identifier.value).url(),
            | PID::DOI | PID::RAID => DOI::from_string(&identifier.value).url(),
            | PID::Handle => Handle::from_string(&identifier.value).url(),
            | PID::ISNI => ISNI::from_string(&identifier.value).url(),
            | PID::ORCID | PID::ROR | PID::URL => identifier.value,
            | PID::RRID => RRID::from_string(&identifier.value).url(),
            | PID::SWHID => SWHID::from_string(&identifier.value).url(),
            | _ => String::new(),
        };
        (href.starts_with("https://") || href.starts_with("http://")).then_some(href)
    })
}
fn point(node: &GraphNode, nodes: &[&GraphNode]) -> (f64, f64) {
    let peers: Vec<_> = nodes.iter().filter(|peer| peer.kind == node.kind).collect();
    let index = peers.iter().position(|peer| peer.id == node.id).unwrap_or(0);
    let angle = (index as f64 / peers.len().max(1) as f64) * core::f64::consts::TAU;
    let radius = match node.kind {
        | GraphNodeKind::Activity => 145.0,
        | GraphNodeKind::Contributor if peers.len() == 1 => 0.0,
        | GraphNodeKind::Contributor => 210.0,
        | GraphNodeKind::Identifier => 345.0,
        | GraphNodeKind::Organization => 285.0,
        | GraphNodeKind::Project => 225.0,
        | GraphNodeKind::Reference => 310.0,
    };
    (400.0 + radius * angle.cos(), 400.0 + radius * angle.sin())
}
pub(crate) fn project(workspace: &Workspace) -> Result<ResearchGraph, String> {
    let source = workspace.source.as_ref().map_or_else(
        || "local".to_string(),
        |origin| format!("{}:{}:{}:{}:{}", origin.kind, origin.host, origin.project, origin.path, origin.revision),
    );
    let activities: Result<Vec<_>, _> = workspace
        .activities()
        .into_iter()
        .map(|(file, value)| {
            serde_json::from_value::<ResearchActivity>(value.clone())
                .map(|activity| (file.path.as_str(), activity))
                .map_err(|error| format!("{}: {error}", file.path))
        })
        .collect();
    activities.and_then(|activities| {
        let records: Vec<_> = activities
            .iter()
            .map(|(path, activity)| GraphRecord {
                source: &source,
                path,
                activity,
            })
            .collect();
        ResearchGraph::from_records(&records)
    })
}
/// Interactive visual and semantic view of the loaded validated RAD connections.
#[component]
pub(crate) fn ResearchGraphPage() -> Element {
    let mut session = use_context::<Signal<String>>();
    let mut workspace = use_context::<Signal<Workspace>>();
    let query = use_signal(String::new);
    let regex = use_signal(|| false);
    let case_sensitive = use_signal(|| false);
    let mut node_kind = use_signal(String::new);
    let mut edge_kind = use_signal(String::new);
    let mut sort = use_signal(|| "label".to_owned());
    let mut entity_query = use_signal(String::new);
    let entity_regex = use_signal(|| false);
    let entity_case_sensitive = use_signal(|| false);
    let mut entity_filter = use_signal(String::new);
    let mut entity_descending = use_signal(|| false);
    let mut relationship_query = use_signal(String::new);
    let relationship_regex = use_signal(|| false);
    let relationship_case_sensitive = use_signal(|| false);
    let mut relationship_filter = use_signal(String::new);
    let mut relationship_sort = use_signal(|| "original".to_owned());
    let mut relationship_descending = use_signal(|| false);
    let mut tab = use_signal(|| "selected".to_owned());
    let mut panel_expanded = use_signal(|| true);
    let mut selected = use_signal(String::new);
    let mut focused = use_signal(|| false);
    let mut focus_radius = use_signal(|| 1_usize);
    let mut restored_url = use_signal(|| None::<[String; 4]>);
    let mut applied_url = use_signal(|| false);
    let mut active_signature = use_signal(String::new);
    use_future(move || async move {
        restored_url.set(script::graph_url_state().await.ok());
    });
    let graph_memo = use_memo(move || project(&workspace()));
    let analysis_memo = use_memo(move || {
        graph_memo()
            .as_ref()
            .ok()
            .filter(|graph| graph_analysis_within_budget(graph))
            .map(analyze_graph)
    });
    let parameters_memo = use_memo(move || {
        graph_memo()
            .as_ref()
            .ok()
            .filter(|graph| graph_analysis_within_budget(graph))
            .map(graph_parameters)
    });
    let keyword_memo = use_memo(move || keywords(&workspace()));
    let graph_result = graph_memo();
    let graph = graph_result.as_ref().ok();
    let analyses = analysis_memo().unwrap_or_default();
    let parameters = parameters_memo();
    let analysis_available = graph.is_some_and(graph_analysis_within_budget);
    let radius_limit = focus_radius_limit(graph, parameters);
    use_effect(move || {
        let current = graph_memo();
        let maximum = focus_radius_limit(current.as_ref().ok(), parameters_memo());
        if focus_radius() > maximum {
            focus_radius.set(maximum);
        }
    });
    let keyword_map = keyword_memo();
    let export_graph = graph.cloned();
    let selected_node = graph.and_then(|graph| graph.nodes.iter().find(|node| node.id == selected()));
    let selected_id = selected_node.map(|node| node.id.as_str());
    let selected_metrics = selected_id.and_then(|id| analyses.get(id)).copied().unwrap_or_default();
    let clustering_spectrum = metric_spectrum(&analyses, selected_metrics.clustering, |metrics| metrics.clustering);
    let degree_spectrum = metric_spectrum(&analyses, selected_metrics.degree, |metrics| metrics.degree);
    let closeness_spectrum = metric_spectrum(&analyses, selected_metrics.closeness, |metrics| metrics.closeness);
    let betweenness_spectrum = metric_spectrum(&analyses, selected_metrics.betweenness, |metrics| metrics.betweenness);
    let graph_signature = graph.map(signature).unwrap_or_default();
    use_effect(move || {
        let current = graph_memo();
        let apply_signature = current.as_ref().ok().map(signature).unwrap_or_default();
        let graph_ids: Vec<String> = current
            .as_ref()
            .ok()
            .map_or_else(Vec::new, |graph| graph.nodes.iter().map(|node| node.id.clone()).collect());
        if !applied_url() && !graph_ids.is_empty() {
            if let Some([kind, edge, ordinal, saved_signature]) = restored_url() {
                if saved_signature == apply_signature {
                    let kind = if NODE_KINDS.iter().any(|(value, _)| *value == kind) {
                        kind
                    } else {
                        String::new()
                    };
                    let edge = if EDGE_KINDS.iter().any(|(value, _)| *value == edge) {
                        edge
                    } else {
                        String::new()
                    };
                    let ordinal = if let Some(id) = ordinal.parse::<usize>().ok().and_then(|index| graph_ids.get(index)) {
                        selected.set(id.clone());
                        ordinal
                    } else {
                        String::new()
                    };
                    node_kind.set(kind.clone());
                    edge_kind.set(edge.clone());
                    script::set_graph_url(&kind, &edge, &ordinal, &apply_signature);
                } else if !saved_signature.is_empty() {
                    script::set_graph_url("", "", "", "");
                }
                active_signature.set(apply_signature.clone());
                applied_url.set(true);
            }
        } else if applied_url() && active_signature() != apply_signature {
            selected.set(String::new());
            focused.set(false);
            node_kind.set(String::new());
            edge_kind.set(String::new());
            entity_filter.set(String::new());
            entity_query.set(String::new());
            relationship_filter.set(String::new());
            relationship_query.set(String::new());
            script::set_graph_url("", "", "", "");
            active_signature.set(apply_signature.clone());
        }
    });
    let selected_ordinal = graph
        .and_then(|graph| graph.nodes.iter().position(|node| Some(node.id.as_str()) == selected_id))
        .map(|index| index.to_string())
        .unwrap_or_default();
    let pattern = SearchPattern::new(&query(), regex(), case_sensitive());
    let (nodes, edges) = graph.map_or_else(
        || (Vec::new(), Vec::new()),
        |graph| {
            visible(
                graph,
                &keyword_map,
                &pattern,
                &node_kind(),
                &edge_kind(),
                focused().then_some(selected_id).flatten(),
                focus_radius().min(radius_limit),
            )
        },
    );
    let sort_by_kind = sort() == "kind";
    let entity_pattern = SearchPattern::new(&entity_query(), entity_regex(), entity_case_sensitive());
    let mut listed_nodes: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| entity_filter().is_empty() || node.kind.as_ref() == entity_filter())
        .filter(|node| {
            entity_pattern.as_ref().is_ok_and(|pattern| {
                pattern.matches(&node.label)
                    || pattern.matches(&node.id)
                    || pattern.matches(&node.path)
                    || pattern.matches(&node.source)
                    || (node.kind == GraphNodeKind::Activity
                        && keyword_map
                            .get(&node.path)
                            .is_some_and(|values| values.iter().any(|value| pattern.matches(value))))
            })
        })
        .collect();
    listed_nodes.sort_by(|left, right| {
        let left_label = left.label.to_lowercase();
        let right_label = right.label.to_lowercase();
        if sort_by_kind {
            (left.kind.as_ref(), &left_label, &left.id).cmp(&(right.kind.as_ref(), &right_label, &right.id))
        } else {
            (&left_label, left.kind.as_ref(), &left.id).cmp(&(&right_label, right.kind.as_ref(), &right.id))
        }
    });
    if entity_descending() {
        listed_nodes.reverse();
    }
    let relationship_pattern = SearchPattern::new(&relationship_query(), relationship_regex(), relationship_case_sensitive());
    let labels: BTreeMap<&str, &str> = graph.map_or_else(BTreeMap::new, |graph| {
        graph.nodes.iter().map(|node| (node.id.as_str(), node.label.as_str())).collect()
    });
    let mut listed_edges: Vec<_> = edges
        .iter()
        .copied()
        .filter(|edge| relationship_filter().is_empty() || edge.kind.as_ref() == relationship_filter())
        .filter(|edge| {
            relationship_pattern.as_ref().is_ok_and(|pattern| {
                pattern.matches(edge.kind.as_ref())
                    || edge.relation_type.as_ref().is_some_and(|value| pattern.matches(value))
                    || pattern.matches(&edge.field)
                    || pattern.matches(&edge.path)
                    || pattern.matches(labels.get(edge.source.as_str()).copied().unwrap_or(&edge.source))
                    || pattern.matches(labels.get(edge.target.as_str()).copied().unwrap_or(&edge.target))
            })
        })
        .collect();
    match relationship_sort().as_str() {
        | "kind" => listed_edges.sort_by_key(|edge| (edge.kind.as_ref(), edge.id.as_str())),
        | "source" => listed_edges.sort_by_key(|edge| (labels.get(edge.source.as_str()).copied().unwrap_or("").to_lowercase(), edge.id.as_str())),
        | "target" => listed_edges.sort_by_key(|edge| (labels.get(edge.target.as_str()).copied().unwrap_or("").to_lowercase(), edge.id.as_str())),
        | _ => {}
    }
    if relationship_descending() {
        listed_edges.reverse();
    }
    let over_budget = nodes.len() > DIAGRAM_NODE_LIMIT || edges.len() > DIAGRAM_EDGE_LIMIT;
    let diagram_nodes: Vec<_> = if over_budget { Vec::new() } else { nodes.clone() };
    let diagram_edges: Vec<_> = if over_budget { Vec::new() } else { edges.clone() };
    let show_diagram_labels = diagram_nodes.len() <= 16;
    let selected_edges: Vec<_> = graph.map_or_else(Vec::new, |graph| {
        graph
            .edges
            .iter()
            .filter(|edge| selected_id.is_some_and(|id| edge.source == id || edge.target == id))
            .collect()
    });
    rsx! {
    div { class: "research-graph",
        p { class: "lede", "Explore connections authored in the validated research activities in this workspace." }
        if let Err(error) = &graph_result {
            p { class: "graph-message error", role: "alert", "The graph could not be built: {error}" }
        } else if graph.is_some_and(|graph| graph.nodes.is_empty()) {
            p { class: "graph-message card", "No validated research activities are loaded. Add RAD files from Portfolio to explore their connections." }
            Link { to: Route::Page { page: "portfolio".into() }, "Open Portfolio" }
        } else {
            div { class: if panel_expanded() { "graph-layout" } else { "graph-layout panel-collapsed" },
            div { class: "graph-canvas-column",
            details { class: "tool-settings graph-settings",
                summary { Icon { name: "settings" } "Settings" }
                div { class: "options",
                    SearchField { label: "Search entities", visible_label: Some("Search"), placeholder: "Search names, identifiers, and keywords", query, regex, case_sensitive, error: pattern.as_ref().err().cloned(), compact: true }
                    label { class: "field", span { "Entity type" } select { "aria-label": "Entity type", value: "{node_kind}", onchange: {
                        let graph_signature = graph_signature.clone();
                        let selected_ordinal = selected_ordinal.clone();
                        move |event| { let kind = event.value(); node_kind.set(kind.clone()); script::set_graph_url(&kind, &edge_kind(), &selected_ordinal, &graph_signature); }
                    },
                        for (value, label) in NODE_KINDS { option { value, "{label}" } }
                    } }
                    label { class: "field", span { "Relationship type" } select { "aria-label": "Relationship type", value: "{edge_kind}", onchange: {
                        let graph_signature = graph_signature.clone();
                        let selected_ordinal = selected_ordinal.clone();
                        move |event| { let edge = event.value(); edge_kind.set(edge.clone()); script::set_graph_url(&node_kind(), &edge, &selected_ordinal, &graph_signature); }
                    },
                        for (value, label) in EDGE_KINDS { option { value, "{label}" } }
                    } }
                    div { class: "graph-display-controls",
                        div { class: "field graph-sort-control", span { "Sort entities" }
                            div { class: "skip-check-choices", role: "group", "aria-label": "Sort entities",
                                button { r#type: "button", "aria-label": "Sort by name", title: "Name A–Z", "aria-pressed": sort() == "label", onclick: move |_| sort.set("label".to_owned()), Icon { name: "sort-title" } }
                                button { r#type: "button", "aria-label": "Sort by entity type", title: "Entity type", "aria-pressed": sort() == "kind", onclick: move |_| sort.set("kind".to_owned()), Icon { name: "portfolio" } }
                            }
                        }
                        div { class: "field graph-focus-control", span { "Neighborhood" }
                            div { class: "vocabulary-choices graph-neighborhood-choices", role: "group", "aria-label": "Neighborhood", "data-selected": if focused() { "on" } else { "off" },
                                button { r#type: "button", "aria-pressed": !focused(), onclick: move |_| focused.set(false), "OFF" }
                                button { r#type: "button", "aria-pressed": focused(), onclick: move |_| focused.set(true), "ON" }
                            }
                        }
                        label { class: "field graph-radius-control", span { "Focus radius" } input { r#type: "number", min: "1", max: "{radius_limit}", step: "1", "aria-label": "Focus radius", value: "{focus_radius().min(radius_limit)}", onchange: move |event| focus_radius.set(event.value().parse::<usize>().unwrap_or(1).clamp(1, radius_limit)) } }
                    }
                }
            }
            p { class: "graph-count", "Showing {nodes.len()} entities and {edges.len()} relationships from {graph.map_or(0, |graph| graph.nodes.len())} entities and {graph.map_or(0, |graph| graph.edges.len())} relationships." }
                if nodes.is_empty() { p { class: "graph-message card", "No entities match these filters. Clear the search or change an entity type." } }
                if over_budget {
                    div { class: "graph-over-budget card",
                        p { class: "graph-message", role: "status", "The diagram limit is {DIAGRAM_NODE_LIMIT} entities and {DIAGRAM_EDGE_LIMIT} relationships. Narrow the filters or focus on a selected entity. The tabs on the right still contain all matching results." }
                        button { class: "graph-export-control", "aria-label": "Download Turtle", title: "Download Turtle", disabled: export_graph.is_none(), onclick: move |_| {
                            if let Some(graph) = &export_graph { script::download_result(&graph.to_turtle(), "acorn-research-graph.ttl", "text/turtle;charset=utf-8"); }
                        }, Icon { name: "download" } }
                    }
                } else if !diagram_nodes.is_empty() {
                div { class: "graph-visual card",
                    svg { class: "graph-svg", view_box: "0 0 800 800", role: "img", "data-focused": focused(), "data-focus-radius": "{focus_radius().min(radius_limit)}", "data-graph-signature": "{graph_signature}", "aria-label": "Research activity connection diagram. Drag entities to move them, drag empty space with one finger to pan, or use two fingers to pan and pinch to zoom. Tap an entity to select it, or empty space to clear the selection. Scroll to zoom with a mouse. Use the entity list for keyboard navigation.", onclick: {
                        let graph_signature = graph_signature.clone();
                        move |_| { selected.set(String::new()); script::set_graph_url(&node_kind(), &edge_kind(), "", &graph_signature); }
                    },
                        defs {
                            marker { id: "graph-arrow", view_box: "0 0 10 10", ref_x: "9", ref_y: "5", marker_width: "5", marker_height: "5", orient: "auto", path { d: "M 0 0 L 10 5 L 0 10 z" } }
                            marker { id: "graph-arrow-hover", view_box: "0 0 10 10", ref_x: "9", ref_y: "5", marker_width: "5", marker_height: "5", orient: "auto", path { d: "M 0 0 L 10 5 L 0 10 z" } }
                        }
                        for edge in diagram_edges {
                            if let (Some(source), Some(target)) = (
                                diagram_nodes.iter().find(|node| node.id == edge.source),
                                diagram_nodes.iter().find(|node| node.id == edge.target),
                            ) {
                                {
                                    let (x1, y1, x2, y2) = edge_ends(point(source, &diagram_nodes), point(target, &diagram_nodes));
                                    rsx! { line { class: "graph-edge", "data-edge-id": edge.id.as_str(), "data-kind": edge.kind.as_ref(), "data-source": edge.source.as_str(), "data-target": edge.target.as_str(), x1: "{x1}", y1: "{y1}", x2: "{x2}", y2: "{y2}", title { "{edge.kind.as_ref()}" } } }
                                }
                            }
                        }
                        for node in &diagram_nodes {
                            {
                                let (x, y) = point(node, &diagram_nodes);
                                let id = node.id.clone();
                                let ordinal = graph.and_then(|graph| graph.nodes.iter().position(|item| item.id == id)).map(|index| index.to_string()).unwrap_or_default();
                                let graph_signature = graph_signature.clone();
                                let icon = node_icon_name(node.kind).parse::<IconName>().unwrap_or(IconName::Unknown);
                                let artwork = icon.legacy_artwork();
                                let uswds_artwork = icon.uswds_symbol().map(|id| format!(r#"<use href="{}#{id}" width="24" height="24"></use>"#, asset!("/assets/vendor/uswds-sprite.svg")));
                                rsx! { g { class: if selected_id == Some(node.id.as_str()) { "graph-node selected" } else { "graph-node" }, "data-kind": node.kind.as_ref(), "data-node-id": node.id.as_str(), "data-origin-x": "{x}", "data-origin-y": "{y}", onclick: move |event| { event.stop_propagation(); selected.set(id.clone()); tab.set("selected".to_owned()); panel_expanded.set(true); script::set_graph_url(&node_kind(), &edge_kind(), &ordinal, &graph_signature); },
                                    title { "{node.kind.as_ref()}: {node.label}" }
                                    circle { cx: "{x}", cy: "{y}", r: "14" }
                                    g { class: "graph-node-mark", transform: "translate({x - 8.5} {y - 8.5}) scale(0.7)", "aria-hidden": "true",
                                        if node.kind == GraphNodeKind::Identifier { text { x: "12", y: "12", "ID" } }
                                        else {
                                            g { class: "legacy-icon", dangerous_inner_html: artwork }
                                            if let Some(uswds_artwork) = uswds_artwork { g { class: "usa-icon", dangerous_inner_html: uswds_artwork } }
                                        }
                                    }
                                    if show_diagram_labels || selected_id == Some(node.id.as_str()) { text { class: "graph-node-label", x: "{x}", y: "{y + 30.0}", "{node.label}" } }
                                } }
                            }
                        }
                    }
                    div { class: "graph-view-controls skip-check-choices", role: "group", "aria-label": "Graph view controls",
                        button { "aria-label": "Zoom in", title: "Zoom in", "data-graph-action": "zoom-in", Icon { name: "zoom-in" } }
                        button { "aria-label": "Zoom out", title: "Zoom out", "data-graph-action": "zoom-out", Icon { name: "zoom-out" } }
                        button { "aria-label": "Fit graph", title: "Fit graph", "data-graph-action": "fit", Icon { name: "fit" } }
                        button { "aria-label": "Reset graph layout and view", title: "Reset graph layout and view", "data-graph-action": "reset", Icon { name: "reset" } }
                        button { "aria-label": "Fullscreen graph", title: "Fullscreen graph", "data-graph-action": "fullscreen",
                            span { class: "graph-fullscreen-enter", Icon { name: "maximize" } }
                            span { class: "graph-fullscreen-exit", Icon { name: "minimize" } }
                        }
                    }
                    button { class: "graph-export-control", "aria-label": "Download Turtle", title: "Download Turtle", disabled: export_graph.is_none(), onclick: move |_| {
                        if let Some(graph) = &export_graph { script::download_result(&graph.to_turtle(), "acorn-research-graph.ttl", "text/turtle;charset=utf-8"); }
                    }, Icon { name: "download" } }
                    if let Some(parameters) = parameters {
                        div { class: "graph-parameters", "aria-label": "Graph parameters",
                            strong { "Graph parameters" }
                            dl {
                                div { dt { "Longest path" } dd { "{parameters.diameter} hops" } }
                                div { dt { "Average shortest path" } dd { "{parameters.average_shortest_path:.2}" } }
                                div { dt { "Graph clustering" } dd { "{parameters.average_clustering:.2}" } }
                                div { dt { "Components" } dd { "{parameters.components}" } }
                                div { dt { "Density" } dd { "{parameters.density:.2}" } }
                            }
                        }
                    } else {
                        p { class: "graph-parameters graph-analysis-note", "Graph parameters are unavailable because the loaded graph exceeds {DIAGRAM_NODE_LIMIT} entities or {DIAGRAM_EDGE_LIMIT} relationships." }
                    }
                    button { class: "graph-minimap", "aria-label": "Fit graph from minimap", "data-graph-action": "fit", title: "Graph overview. Select to fit the graph.",
                        svg { class: "graph-minimap-svg", view_box: "0 0 144 96", "aria-hidden": "true" }
                    }
                }
            }
            div { class: "graph-legend", role: "group", "aria-label": "Graph legend",
                strong { "Legend" }
                for (kind, label) in [(GraphNodeKind::Activity, "Activity"), (GraphNodeKind::Contributor, "Contributor"), (GraphNodeKind::Identifier, "Identifier"), (GraphNodeKind::Organization, "Organization"), (GraphNodeKind::Project, "Project"), (GraphNodeKind::Reference, "Unresolved reference")] {
                    span { class: "graph-legend-item", "data-kind": kind.as_ref(), span { class: "graph-legend-dot", "aria-hidden": "true",
                        if kind == GraphNodeKind::Identifier { "ID" } else { Icon { name: node_icon_name(kind) } }
                    } "{label}" }
                }
                span { class: "graph-legend-item", span { class: "graph-legend-relation", "aria-hidden": "true" } "Explicit relation" }
            }
            }
            section { class: "card graph-side-panel", "aria-label": "Graph details",
                div { class: "graph-panel-header",
                    div { class: "graph-tabs", role: "tablist", "aria-label": "Graph details",
                        for (name, label) in [("selected", "Selected"), ("entities", "Entities"), ("relationships", "Relationships")] {
                            button { id: "graph-tab-{name}", r#type: "button", role: "tab", "aria-selected": tab() == name, "aria-controls": "graph-tab-panel", onclick: move |_| tab.set(name.to_owned()), "{label}" }
                        }
                    }
                    button { class: "collapse graph-panel-toggle", r#type: "button", "aria-label": "Toggle graph details", "aria-controls": "graph-tab-panel", "aria-expanded": panel_expanded(), onclick: move |_| panel_expanded.toggle(), Icon { name: match panel_expanded() { true => "panel-open", false => "panel" } } }
                }
                div { id: "graph-tab-panel", class: "graph-panel-body", role: "tabpanel", "aria-labelledby": match tab().as_str() { "entities" => "graph-tab-entities", "relationships" => "graph-tab-relationships", _ => "graph-tab-selected" },
                if tab() == "selected" {
                section { key: "{selected_id.unwrap_or_default()}", class: "graph-inspector", "aria-label": "Selected entity",
                    div { class: "graph-panel-intro", p { class: "graph-eyebrow", "INSPECT" } h2 { "Selected entity" } p { "Source details and direct connections" } }
                    if let Some(node) = selected_node {
                        div { class: "graph-selected-card",
                            span { class: "graph-type-icon", "data-kind": node.kind.as_ref(), "aria-hidden": "true",
                                if node.kind == GraphNodeKind::Identifier { "ID" } else { Icon { name: node_icon_name(node.kind) } }
                            }
                            div { span { class: "graph-kind", "{node.kind.as_ref()}" } h3 { {entity_title(graph, &node.id)} } }
                        }
                        dl { class: "graph-provenance",
                            div { dt { "Source" } dd { "{node.source}" } }
                            div { dt { "RAD file" } dd { "{node.path}" } }
                        }
                        if node.kind == GraphNodeKind::Activity {
                            if let Some(values) = keyword_map.get(&node.path).filter(|values| !values.is_empty()) {
                                div { class: "graph-detail-section", h4 { "Keywords" }
                                    ul { class: "graph-keywords", for keyword in values { li { "{keyword}" } } }
                                }
                            }
                        }
                        if node.kind == GraphNodeKind::Reference { p { class: "graph-note", "This authored reference has no unambiguous loaded activity target." } }
                        Link { class: "graph-source-link", to: Route::Page { page: "review".into() }, onclick: {
                            let path = node.path.clone();
                            move |_| {
                                let content = workspace.peek().files.iter().find(|file| file.path == path).map(|file| file.content.clone());
                                if let Some(content) = content { workspace.write().selected = Some(path.clone()); session.set(content); }
                            }
                        }, "Open source file in Review" }
                        details { class: "graph-detail-section graph-algorithms", "aria-label": "Graph algorithm results", open: true,
                            summary { "Graph analysis" }
                            if analysis_available {
                                p { class: "graph-analysis-note", "Each bar places this entity between the smallest and largest score across all loaded entities. Scores use undirected connections; repeated links count once and self-links are ignored." }
                                dl { class: "graph-algorithm-grid",
                                    GraphMetricCard { label: "Local clustering", metric: "clustering", score: selected_metrics.clustering, spectrum: clustering_spectrum }
                                    GraphMetricCard { label: "Degree centrality", metric: "degree", score: selected_metrics.degree, spectrum: degree_spectrum }
                                    GraphMetricCard { label: "Closeness centrality", metric: "closeness", score: selected_metrics.closeness, spectrum: closeness_spectrum }
                                    GraphMetricCard { label: "Betweenness centrality", metric: "betweenness", score: selected_metrics.betweenness, spectrum: betweenness_spectrum }
                                }
                            } else {
                                p { class: "graph-analysis-note", "Graph analysis is unavailable because the loaded graph exceeds {DIAGRAM_NODE_LIMIT} entities or {DIAGRAM_EDGE_LIMIT} relationships." }
                            }
                        }
                        details { class: "graph-detail-section graph-direct-relationships",
                            summary { "Direct relationships" span { class: "graph-count-badge", "{selected_edges.len()}" } }
                            if selected_edges.is_empty() { p { class: "graph-empty", "No direct relationships." } }
                            ul { class: "graph-edge-list", for edge in selected_edges {
                                li {
                                    div { class: "graph-edge-topline", span { class: "graph-kind", "{edge.kind.as_ref()}" }
                                        if let Some(relation_type) = &edge.relation_type { span { class: "graph-relation-type", "relation type: {relation_type}" } }
                                    }
                                    p { class: "graph-edge-entities",
                                        {entity_title(graph, &edge.source)}
                                        span { class: "graph-edge-arrow", "aria-hidden": "true", "→" }
                                        {entity_title(graph, &edge.target)}
                                    }
                                    p { class: "graph-edge-source", "{edge.field} · {edge.path}" }
                                }
                            } }
                        }
                    } else { p { class: "graph-empty", "Select an entity in the diagram or list to inspect its source and direct relationships." } }
                }
                } else if tab() == "entities" {
                section { class: "graph-list", "aria-label": "Graph entities",
                    div { class: "graph-panel-intro", p { class: "graph-eyebrow", "BROWSE" } h2 { "Entities" span { class: "graph-count-badge", "{listed_nodes.len()}" } } p { "Select an entity to inspect its connections" } }
                    details { class: "result-settings graph-tab-settings",
                        summary { Icon { name: "settings" } "Settings" }
                        div { class: "result-controls",
                            SearchField { label: "Search listed entities", visible_label: Some("Search"), placeholder: "Search entities and keywords", query: entity_query, regex: entity_regex, case_sensitive: entity_case_sensitive, error: entity_pattern.as_ref().err().cloned(), compact: true }
                            div { class: "result-filters", role: "group", "aria-label": "Filter listed entities",
                                for (value, label) in NODE_KINDS {
                                    button { r#type: "button", "aria-pressed": entity_filter() == value,
                                        onclick: move |_| entity_filter.set(value.to_owned()), "{label}"
                                        span { class: "filter-count", "{nodes.iter().filter(|node| value.is_empty() || node.kind.as_ref() == value).count()}" }
                                    }
                                }
                            }
                            div { class: "result-sort", role: "group", "aria-label": "Sort listed entities",
                                span { class: "muted", "Sort" }
                                for (value, label) in [("label", "Name"), ("kind", "Type")] {
                                    button { r#type: "button", "aria-pressed": sort() == value, onclick: move |_| sort.set(value.to_owned()), "{label}" }
                                }
                                button { r#type: "button", class: "sort-direction", "aria-label": if entity_descending() { "Sort entities ascending" } else { "Sort entities descending" },
                                    title: "Reverse sort direction", onclick: move |_| entity_descending.toggle(), span { "aria-hidden": "true", if entity_descending() { "↓" } else { "↑" } }
                                }
                            }
                        }
                    }
                    p { class: "small muted result-summary", role: "status", "Showing {listed_nodes.len()} of {nodes.len()} entities" }
                    if listed_nodes.is_empty() { p { class: "graph-empty", "No entities match these settings." } }
                    ul { for node in &listed_nodes { li { class: if selected_id == Some(node.id.as_str()) { "graph-item selected" } else { "graph-item" }, "data-graph-node-id": node.id.as_str(), onclick: {
                        let id = node.id.clone();
                        let ordinal = graph.and_then(|graph| graph.nodes.iter().position(|item| item.id == id)).map(|index| index.to_string()).unwrap_or_default();
                        let graph_signature = graph_signature.clone();
                        move |_| { selected.set(id.clone()); tab.set("selected".to_owned()); panel_expanded.set(true); script::set_graph_url(&node_kind(), &edge_kind(), &ordinal, &graph_signature); }
                    }, span { class: "graph-type-icon", "data-kind": node.kind.as_ref(), "aria-hidden": "true",
                        if node.kind == GraphNodeKind::Identifier { "ID" } else { Icon { name: node_icon_name(node.kind) } }
                    }
                        span { class: "graph-item-copy", span { class: "graph-kind", "{node.kind.as_ref()}" } strong { {entity_title(graph, &node.id)} } small { "{node.path}" } }
                        button { class: "graph-item-select", r#type: "button", "aria-label": "Select {node.label}", title: "Select entity", "aria-pressed": selected_id == Some(node.id.as_str()), Icon { name: "eye" } }
                    } } }
                }
                } else {
                section { class: "graph-relationships", "aria-label": "Graph relationships",
                div { class: "graph-panel-intro", p { class: "graph-eyebrow", "TRACE" } h2 { "Relationships" span { class: "graph-count-badge", "{listed_edges.len()}" } } p { "Directed links authored in the RAD files" } }
                details { class: "result-settings graph-tab-settings",
                    summary { Icon { name: "settings" } "Settings" }
                    div { class: "result-controls",
                        SearchField { label: "Search listed relationships", visible_label: Some("Search"), placeholder: "Search links, entities, and sources", query: relationship_query, regex: relationship_regex, case_sensitive: relationship_case_sensitive, error: relationship_pattern.as_ref().err().cloned(), compact: true }
                        div { class: "result-filters", role: "group", "aria-label": "Filter listed relationships",
                            for (value, label) in EDGE_KINDS {
                                button { r#type: "button", "aria-pressed": relationship_filter() == value,
                                    onclick: move |_| relationship_filter.set(value.to_owned()), "{label}"
                                    span { class: "filter-count", "{edges.iter().filter(|edge| value.is_empty() || edge.kind.as_ref() == value).count()}" }
                                }
                            }
                        }
                        div { class: "result-sort", role: "group", "aria-label": "Sort listed relationships",
                            span { class: "muted", "Sort" }
                            for (value, label) in [("original", "Original"), ("kind", "Type"), ("source", "Source"), ("target", "Target")] {
                                button { r#type: "button", "aria-pressed": relationship_sort() == value,
                                    onclick: move |_| { relationship_sort.set(value.to_owned()); relationship_descending.set(false); }, "{label}" }
                            }
                            button { r#type: "button", class: "sort-direction", "aria-label": if relationship_descending() { "Sort relationships ascending" } else { "Sort relationships descending" },
                                title: "Reverse sort direction", onclick: move |_| relationship_descending.toggle(), span { "aria-hidden": "true", if relationship_descending() { "↓" } else { "↑" } }
                            }
                        }
                    }
                }
                p { class: "small muted result-summary", role: "status", "Showing {listed_edges.len()} of {edges.len()} relationships" }
                if listed_edges.is_empty() { p { class: "graph-empty", "No relationships match these settings." } }
                ul { class: "graph-edge-list", for edge in listed_edges { li { "data-graph-edge-id": edge.id.as_str(), tabindex: "0",
                    div { class: "graph-edge-topline", span { class: "graph-kind", "{edge.kind.as_ref()}" }
                        if let Some(relation_type) = &edge.relation_type { span { class: "graph-relation-type", "relation type: {relation_type}" } }
                    }
                    p { class: "graph-edge-entities",
                        {entity_title(graph, &edge.source)}
                        span { class: "graph-edge-arrow", "aria-hidden": "true", "→" }
                        {entity_title(graph, &edge.target)}
                    }
                    p { class: "graph-edge-source", "{edge.field} · {edge.path}" }
                } } }
                }
                }
                }
                }
            }
            }
        }
    }
}
fn signature(graph: &ResearchGraph) -> String {
    let value = graph
        .nodes
        .iter()
        .flat_map(|node| node.id.bytes().chain([0]))
        .fold(0xcbf29ce484222325_u64, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3));
    format!("{value:016x}")
}
pub(crate) fn visible<'a>(
    graph: &'a ResearchGraph,
    keywords: &BTreeMap<String, Vec<String>>,
    pattern: &Result<SearchPattern, String>,
    node_kind: &str,
    edge_kind: &str,
    focus: Option<&str>,
    radius: usize,
) -> (Vec<&'a GraphNode>, Vec<&'a GraphEdge>) {
    let matches_edge = |edge: &GraphEdge| edge_kind.is_empty() || edge.kind.as_ref() == edge_kind;
    let neighbors: BTreeSet<&str> = focus.map_or_else(BTreeSet::new, |id| {
        let mut reachable = BTreeSet::from([id]);
        for _ in 0..radius {
            let adjacent: BTreeSet<_> = graph
                .edges
                .iter()
                .filter(|edge| matches_edge(edge) && (reachable.contains(edge.source.as_str()) || reachable.contains(edge.target.as_str())))
                .flat_map(|edge| [edge.source.as_str(), edge.target.as_str()])
                .collect();
            reachable.extend(adjacent);
        }
        reachable
    });
    let nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|node| node_kind.is_empty() || node.kind.as_ref() == node_kind)
        .filter(|node| {
            pattern.as_ref().is_ok_and(|pattern| {
                pattern.matches(&node.label)
                    || (node.kind == GraphNodeKind::Activity
                        && keywords
                            .get(&node.path)
                            .is_some_and(|values| values.iter().any(|value| pattern.matches(value))))
            })
        })
        .filter(|node| focus.is_none() || neighbors.contains(node.id.as_str()) || focus == Some(node.id.as_str()))
        .collect();
    let ids: BTreeSet<&str> = nodes.iter().map(|node| node.id.as_str()).collect();
    let edges = graph
        .edges
        .iter()
        .filter(|edge| matches_edge(edge))
        .filter(|edge| ids.contains(edge.source.as_str()) && ids.contains(edge.target.as_str()))
        .collect();
    (nodes, edges)
}
