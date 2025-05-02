use petgraph::stable_graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{graph::Graph, Direction};

type _Graph = Graph<String, u8>;
pub fn get_node_from_label(graph: &_Graph, label: &str) -> Option<NodeIndex> {
    match graph.node_indices().find(|n| graph[*n] == label) {
        | Some(node) => Some(node),
        | None => None,
    }
}
pub fn get_node_name(graph: &_Graph, node: NodeIndex) -> Option<String> {
    match graph.node_weight(node) {
        | Some(value) => Some(value.to_string()),
        | None => None,
    }
}
pub fn get_node_parent(graph: &_Graph, node: NodeIndex) -> Option<NodeIndex> {
    match graph.edges_directed(node, Direction::Incoming).clone().next() {
        | Some(edge) => Some(edge.source()),
        | None => None,
    }
}
