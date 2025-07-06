//! ## Graph Utilities
//!
//! This module provides functions for working with graph data structures.
//!
//! <div class="warning">Primary use is to model organizational data as a graph.</div>
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{graph::Graph, Direction};

type _Graph = Graph<String, u8>;
/// Find a node by label.
///
/// # Arguments
///
/// * `graph` - The graph to search.
/// * `label` - The label to search for.
///
/// # Return
///
/// The node index with the given `label`, or `None` if not found.
pub fn node_from_label(graph: &_Graph, label: &str) -> Option<NodeIndex> {
    match graph.node_indices().find(|n| graph[*n] == label) {
        | Some(node) => Some(node),
        | None => None,
    }
}
/// Get the name of a node in the graph.
///
/// # Arguments
///
/// * `graph` - The graph containing the node.
/// * `node` - The node whose name to retrieve.
///
/// # Return
///
/// The name associated with the node, or `None` if not found.
pub fn node_name(graph: &_Graph, node: NodeIndex) -> Option<String> {
    match graph.node_weight(node) {
        | Some(value) => Some(value.to_string()),
        | None => None,
    }
}
/// Find the parent of a node in the graph.
///
/// # Arguments
///
/// * `graph` - The graph containing the node.
/// * `node` - The node whose parent to retrieve.
///
/// # Return
///
/// The parent of the node, or `None` if not found.
pub fn node_parent(graph: &_Graph, node: NodeIndex) -> Option<NodeIndex> {
    match graph.edges_directed(node, Direction::Incoming).clone().next() {
        | Some(edge) => Some(edge.source()),
        | None => None,
    }
}
