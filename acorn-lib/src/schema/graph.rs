//! ## Graph Utilities
//!
//! This module provides functions for working with graph data structures.
//!
//! <div class="warning">Primary use is to model organizational data as a graph.</div>
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{graph::Graph, Direction};

type _Graph = Graph<String, u8>;
pub fn node_from_label(graph: &_Graph, label: &str) -> Option<NodeIndex> {
    match graph.node_indices().find(|n| graph[*n] == label) {
        | Some(node) => Some(node),
        | None => None,
    }
}
pub fn node_name(graph: &_Graph, node: NodeIndex) -> Option<String> {
    match graph.node_weight(node) {
        | Some(value) => Some(value.to_string()),
        | None => None,
    }
}
pub fn node_parent(graph: &_Graph, node: NodeIndex) -> Option<NodeIndex> {
    match graph.edges_directed(node, Direction::Incoming).clone().next() {
        | Some(edge) => Some(edge.source()),
        | None => None,
    }
}
