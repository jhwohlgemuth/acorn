//! Turtle serialization of the source-backed RAD graph projection.
use super::graph::ResearchGraph;
use acorn_core::prelude::{format, String, ToString, Vec};

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";

impl ResearchGraph {
    /// Serialize supported graph entities and edges as deterministic Turtle.
    ///
    /// The output is the N-Triples subset of Turtle. UI layout and source paths
    /// are omitted; this is not a lossless RAD serialization.
    pub fn to_turtle(&self) -> String {
        let node_lines = self.nodes.iter().flat_map(|node| {
            [
                format!("<{}> <{RDF_TYPE}> <urn:acorn:graph:class:{}> .", node.id, node.kind.as_ref()),
                format!("<{}> <{RDFS_LABEL}> \"{}\" .", node.id, escape_literal(&node.label)),
            ]
        });
        let edge_lines = self
            .edges
            .iter()
            .map(|edge| format!("<{}> <urn:acorn:graph:predicate:{}> <{}> .", edge.source, edge.kind.as_ref(), edge.target));
        let mut lines = node_lines.chain(edge_lines).collect::<Vec<_>>();
        lines.sort();
        lines.dedup();
        format!("{}\n", lines.join("\n"))
    }
}
fn escape_literal(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            | '\\' => "\\\\".into(),
            | '"' => "\\\"".into(),
            | '\n' => "\\n".into(),
            | '\r' => "\\r".into(),
            | '\t' => "\\t".into(),
            | character if character.is_control() => format!("\\u{:04X}", character as u32),
            | character => character.to_string(),
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/graph_rdf.rs"]
mod tests;
