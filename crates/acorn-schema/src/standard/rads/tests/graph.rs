use super::*;
use crate::standard::rads::{ActivityOrganization, ResearchActivityMetadata, TypedIdentifier, TypedRelatedResource};
use crate::OneOrMany;

fn activity(title: &str, project: &str) -> ResearchActivity {
    ResearchActivity::init()
        .title(title.to_string())
        .meta(ResearchActivityMetadata::init().identifier(project.to_string()).build())
        .build()
}
fn record<'a>(path: &'a str, activity: &'a ResearchActivity) -> GraphRecord<'a> {
    GraphRecord {
        source: "local",
        path,
        activity,
    }
}
#[test]
fn test_graph_detects_duplicate_source_paths() {
    let first = activity("First activity", "project-one");
    let second = activity("Second activity", "project-one");
    let result = ResearchGraph::from_records(&[record("a.json", &first), record("a.json", &second)]);
    assert_eq!(result.err().as_deref(), Some("duplicate RAD source path: local:a.json"));
}
#[test]
fn test_graph_keeps_unnamed_identity_source_local_and_is_order_independent() {
    let organization = ActivityOrganization::init().name("Example Lab").build();
    let mut first = activity("First activity", "project-one");
    let mut second = activity("Second activity", "project-one");
    first.meta.organizations = Some(OneOrMany::One(organization.clone()));
    second.meta.organizations = Some(OneOrMany::One(organization));
    let forward = ResearchGraph::from_records(&[record("a.json", &first), record("b.json", &second)]).expect("graph");
    let reverse = ResearchGraph::from_records(&[record("b.json", &second), record("a.json", &first)]).expect("graph");
    assert_eq!(forward, reverse);
    assert_eq!(forward.nodes.iter().filter(|node| node.kind == GraphNodeKind::Project).count(), 1);
    assert_eq!(forward.nodes.iter().filter(|node| node.kind == GraphNodeKind::Organization).count(), 2);
    assert_eq!(forward.edges.iter().filter(|edge| edge.kind == GraphEdgeKind::Contains).count(), 2);
}
#[test]
fn test_graph_links_arxiv_publication_identifier_and_reference() {
    let mut first = activity("First activity", "project-one");
    let mut second = activity("Second activity", "project-two");
    first.meta.related_resources = Some(OneOrMany::One(
        TypedRelatedResource::init()
            .identifier(TypedIdentifier::init().value("arXiv:2106.09685").scheme("arXiv").build())
            .relation_type("References")
            .build(),
    ));
    second.meta.doi = Some(OneOrMany::One("arXiv:2106.09685".to_string()));
    let graph = ResearchGraph::from_records(&[record("a.json", &first), record("b.json", &second)]).expect("graph");
    let target = graph
        .nodes
        .iter()
        .find(|node| node.kind == GraphNodeKind::Activity && node.label == "Second activity")
        .expect("target activity");
    assert!(graph
        .nodes
        .iter()
        .any(|node| node.kind == GraphNodeKind::Identifier && node.label == "arXiv:2106.09685"));
    assert!(graph
        .edges
        .iter()
        .any(|edge| edge.kind == GraphEdgeKind::RelatesTo && edge.target == target.id));
    assert_eq!(graph.nodes.iter().filter(|node| node.kind == GraphNodeKind::Reference).count(), 0);
}
#[test]
fn test_graph_resolves_explicit_relation_and_preserves_missing_target() {
    let mut first = activity("First activity", "project-one");
    let second = activity("Second activity", "project-two");
    first.meta.related_resources = Some(OneOrMany::Many(vec![
        TypedRelatedResource::init()
            .identifier(crate::standard::rads::TypedIdentifier::init().value("b.json").build())
            .relation_type("References")
            .build(),
        TypedRelatedResource::init()
            .identifier(crate::standard::rads::TypedIdentifier::init().value("missing.json").build())
            .relation_type("References")
            .build(),
    ]));
    let graph = ResearchGraph::from_records(&[record("a.json", &first), record("b.json", &second)]).expect("graph");
    let related = graph
        .edges
        .iter()
        .filter(|edge| edge.kind == GraphEdgeKind::RelatesTo)
        .collect::<Vec<_>>();
    assert_eq!(related.len(), 2);
    assert_eq!(graph.nodes.iter().filter(|node| node.kind == GraphNodeKind::Reference).count(), 1);
    assert!(related.iter().all(|edge| edge.relation_type.as_deref() == Some("References")));
}
#[test]
fn test_graph_shares_organizations_only_with_valid_explicit_identity() {
    let organization = ActivityOrganization::init()
        .name("Example Lab")
        .identifiers(vec![TypedIdentifier::init().value("https://ror.org/01qz5mb56").scheme("ROR").build()])
        .build();
    let mut first = activity("First activity", "project-one");
    let mut second = activity("Second activity", "project-two");
    first.meta.organizations = Some(OneOrMany::One(organization.clone()));
    second.meta.organizations = Some(OneOrMany::One(organization));
    let graph = ResearchGraph::from_records(&[record("a.json", &first), record("b.json", &second)]).expect("graph");
    assert_eq!(graph.nodes.iter().filter(|node| node.kind == GraphNodeKind::Organization).count(), 1);
    assert_eq!(graph.nodes.iter().filter(|node| node.kind == GraphNodeKind::Identifier).count(), 1);
    assert_eq!(graph.edges.iter().filter(|edge| edge.kind == GraphEdgeKind::Involves).count(), 2);
}
#[test]
fn test_graph_source_qualifies_equal_paths() {
    let first = activity("First activity", "project-one");
    let second = activity("Second activity", "project-one");
    let graph = ResearchGraph::from_records(&[
        record("a.json", &first),
        GraphRecord {
            source: "remote",
            path: "a.json",
            activity: &second,
        },
    ])
    .expect("graph");
    assert_eq!(graph.nodes.iter().filter(|node| node.kind == GraphNodeKind::Project).count(), 2);
    assert_eq!(graph.nodes.iter().filter(|node| node.kind == GraphNodeKind::Activity).count(), 2);
}
