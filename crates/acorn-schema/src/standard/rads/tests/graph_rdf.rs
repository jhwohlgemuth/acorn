use super::*;
use crate::standard::rads::graph::{GraphRecord, ResearchGraph};
use crate::standard::rads::{ResearchActivity, ResearchActivityMetadata, TypedIdentifier, TypedRelatedResource};
use crate::OneOrMany;
use sophia::api::prelude::*;
use sophia::inmem::graph::FastGraph;
use sophia::turtle::parser::turtle::TurtleParser;

fn graph(title: &str) -> ResearchGraph {
    let activity = ResearchActivity::init()
        .title(title.to_string())
        .meta(ResearchActivityMetadata::init().identifier("project-one".to_string()).build())
        .build();
    ResearchGraph::from_records(&[GraphRecord {
        source: "local",
        path: "a.json",
        activity: &activity,
    }])
    .expect("graph")
}
#[test]
fn test_turtle_coalesces_duplicate_relation_triples_and_keeps_reference() {
    let mut activity = ResearchActivity::init()
        .title("Research activity".to_string())
        .meta(ResearchActivityMetadata::init().identifier("project-one".to_string()).build())
        .build();
    activity.meta.related_resources = Some(OneOrMany::Many(
        ["References", "IsRelatedTo"]
            .into_iter()
            .map(|relation| {
                TypedRelatedResource::init()
                    .identifier(TypedIdentifier::init().value("missing.json").build())
                    .relation_type(relation)
                    .build()
            })
            .collect(),
    ));
    let graph = ResearchGraph::from_records(&[GraphRecord {
        source: "local",
        path: "a.json",
        activity: &activity,
    }])
    .expect("graph");
    assert_eq!(graph.edges.len(), 3);
    let turtle = graph.to_turtle();
    let parsed = TurtleParser { base: None }
        .parse_str(&turtle)
        .collect_triples::<FastGraph>()
        .expect("parsed Turtle");
    assert_eq!(parsed.triples().count(), graph.nodes.len() * 2 + 2);
    assert!(turtle.contains("urn:acorn:graph:class:reference"));
}
#[test]
fn test_turtle_escapes_literals() {
    let graph = graph("Title \"quoted\" \\ path\nnext");
    let turtle = graph.to_turtle();
    assert!(turtle.contains("Title \\\"quoted\\\" \\\\ path\\nnext"));
    let parsed = TurtleParser { base: None }.parse_str(&turtle).collect_triples::<FastGraph>();
    assert!(parsed.is_ok(), "escaped Turtle should parse: {parsed:?}");
}
#[test]
fn test_turtle_is_deterministic_and_parseable() {
    let graph = graph("Research activity");
    let turtle = graph.to_turtle();
    let parsed = TurtleParser { base: None }.parse_str(&turtle).collect_triples::<FastGraph>();
    assert!(parsed.is_ok(), "Turtle should parse: {parsed:?}");
    assert_eq!(parsed.expect("parsed graph").triples().count(), graph.nodes.len() * 2 + graph.edges.len());
    assert_eq!(turtle, graph.to_turtle());
    assert!(turtle.contains("urn:acorn:graph:predicate:contains"));
}
