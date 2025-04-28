use petgraph::visit::EdgeRef;
use petgraph::{graph::Graph, Direction};

pub fn get_node_parent_name(graph: &Graph<String, u8>, label: &str) -> String {
    let parent = match graph.node_indices().find(|n| graph[*n] == label) {
        | Some(node) => match graph.edges_directed(node, Direction::Incoming).clone().next() {
            | Some(value) => value.source(),
            | None => unreachable!(),
        },
        | None => unreachable!(),
    };
    match graph.node_weight(parent) {
        | Some(value) => value.to_string(),
        | None => unreachable!(),
    }
}
