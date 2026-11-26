//! Deterministic, source-backed connections among validated research activities.
use super::{ResearchActivity, TypedIdentifier};
use crate::pid::{Identifier, PublicationIdentifierType, PID};
use acorn_core::prelude::{format, String, ToString, Vec};
use alloc::collections::{BTreeMap, BTreeSet};

/// The authored relationship represented by a directed edge.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GraphEdgeKind {
    /// Project contains a RAD file.
    Contains,
    /// RAD credits a structured contributor.
    Credits,
    /// Entity has an explicit identifier.
    HasIdentifier,
    /// RAD involves a structured organization.
    Involves,
    /// RAD explicitly relates to another activity or reference.
    RelatesTo,
}
/// The entity represented by a graph node.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GraphNodeKind {
    /// One RAD file.
    Activity,
    /// A structured contributor.
    Contributor,
    /// An explicit persistent identifier.
    Identifier,
    /// A structured organization.
    Organization,
    /// A source-local RAD project grouping.
    Project,
    /// An unresolved or ambiguous authored relation target.
    Reference,
}
/// One directed, source-backed graph relationship.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphEdge {
    /// Opaque, deterministic edge identifier.
    pub id: String,
    /// Relationship type.
    pub kind: GraphEdgeKind,
    /// Source node IRI.
    pub source: String,
    /// Target node IRI.
    pub target: String,
    /// RAD path that authored this edge.
    pub path: String,
    /// RAD field path that authored this edge.
    pub field: String,
    /// Original related-resource relation type, if applicable.
    pub relation_type: Option<String>,
}
/// One graph entity, with source provenance for inspection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphNode {
    /// Opaque, deterministic node IRI.
    pub id: String,
    /// Entity type.
    pub kind: GraphNodeKind,
    /// Display label from the source RAD.
    pub label: String,
    /// Workspace source identity.
    pub source: String,
    /// Relative RAD path that supplied the node.
    pub path: String,
}
impl GraphNode {
    fn new(kind: GraphNodeKind, identity: &str, label: &str, record: &GraphRecord<'_>) -> Self {
        Self {
            id: urn(kind.as_ref(), identity),
            kind,
            label: label.to_string(),
            source: record.source.to_string(),
            path: record.path.to_string(),
        }
    }
}
/// A validated RAD with its workspace source and relative file path.
#[derive(Clone, Copy, Debug)]
pub struct GraphRecord<'a> {
    /// Source identity, including a repository revision when applicable.
    pub source: &'a str,
    /// Normalized relative path within the source.
    pub path: &'a str,
    /// Validated research activity.
    pub activity: &'a ResearchActivity,
}
/// Immutable, sorted projection of a validated RAD snapshot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResearchGraph {
    /// Nodes sorted by stable IRI.
    pub nodes: Vec<GraphNode>,
    /// Edges sorted by stable identifier.
    pub edges: Vec<GraphEdge>,
}
impl AsRef<str> for GraphEdgeKind {
    fn as_ref(&self) -> &str {
        match self {
            | Self::Contains => "contains",
            | Self::Credits => "credits",
            | Self::HasIdentifier => "hasIdentifier",
            | Self::Involves => "involves",
            | Self::RelatesTo => "relatesTo",
        }
    }
}
impl AsRef<str> for GraphNodeKind {
    fn as_ref(&self) -> &str {
        match self {
            | Self::Activity => "activity",
            | Self::Contributor => "contributor",
            | Self::Identifier => "identifier",
            | Self::Organization => "organization",
            | Self::Project => "project",
            | Self::Reference => "reference",
        }
    }
}
impl ResearchGraph {
    /// Project validated records without mutating their RAD
    pub fn from_records(records: &[GraphRecord<'_>]) -> Result<Self, String> {
        let mut ordered = records.to_vec();
        ordered.sort_by(|left, right| (left.source, left.path).cmp(&(right.source, right.path)));
        let duplicate = ordered.windows(2).find_map(|pair| match pair {
            | [first, second] if first.source == second.source && first.path == second.path => Some((first.source, first.path)),
            | _ => None,
        });
        duplicate.map_or_else(
            || {
                let aliases = aliases(&ordered);
                let (nodes, edges) = ordered.iter().fold((BTreeMap::new(), BTreeMap::new()), |(mut nodes, mut edges), record| {
                    add_record(record, &aliases, &mut nodes, &mut edges);
                    (nodes, edges)
                });
                Ok(Self {
                    nodes: nodes.into_values().collect(),
                    edges: edges.into_values().collect(),
                })
            },
            |(source, path)| Err(format!("duplicate RAD source path: {source}:{path}")),
        )
    }
}
fn add_edge(
    edges: &mut BTreeMap<String, GraphEdge>,
    kind: GraphEdgeKind,
    source: &str,
    target: &str,
    record: &GraphRecord<'_>,
    field: &str,
    relation_type: Option<&str>,
) {
    let id = key(&[source, kind.as_ref(), target, record.source, record.path, field]);
    edges.entry(id.clone()).or_insert_with(|| GraphEdge {
        id,
        kind,
        source: source.to_string(),
        target: target.to_string(),
        path: record.path.to_string(),
        field: field.to_string(),
        relation_type: relation_type.map(ToString::to_string),
    });
}
fn add_identifier(
    nodes: &mut BTreeMap<String, GraphNode>,
    edges: &mut BTreeMap<String, GraphEdge>,
    owner: &str,
    value: &str,
    normalized: &str,
    record: &GraphRecord<'_>,
    field: &str,
) {
    let identifier = add_node(nodes, GraphNode::new(GraphNodeKind::Identifier, normalized, value, record));
    add_edge(edges, GraphEdgeKind::HasIdentifier, owner, &identifier, record, field, None);
}
fn add_node(nodes: &mut BTreeMap<String, GraphNode>, value: GraphNode) -> String {
    let id = value.id.clone();
    nodes.entry(id.clone()).or_insert(value);
    id
}
fn add_record(
    record: &GraphRecord<'_>,
    aliases: &BTreeMap<(String, String), BTreeSet<String>>,
    nodes: &mut BTreeMap<String, GraphNode>,
    edges: &mut BTreeMap<String, GraphEdge>,
) {
    let meta = &record.activity.meta;
    let activity = add_node(
        nodes,
        GraphNode::new(
            GraphNodeKind::Activity,
            &key(&[record.source, record.path]),
            &record.activity.title,
            record,
        ),
    );
    let project = add_node(
        nodes,
        GraphNode::new(GraphNodeKind::Project, &key(&[record.source, &meta.identifier]), &meta.identifier, record),
    );
    add_edge(edges, GraphEdgeKind::Contains, &project, &activity, record, "meta.identifier", None);
    for (field, scheme, values) in [
        ("meta.doi", "DOI", meta.doi.as_ref()),
        ("meta.handle", "Handle", meta.handle.as_ref()),
        ("meta.raid", "RAiD", meta.raid.as_ref()),
    ] {
        values
            .into_iter()
            .flat_map(|values| values.iter())
            .enumerate()
            .for_each(|(index, value)| {
                let normalized = if field == "meta.doi" {
                    publication_identifier(value)
                } else {
                    normalized_identifier(value, Some(scheme))
                };
                if let Some(normalized) = normalized {
                    add_identifier(nodes, edges, &activity, value, &normalized, record, &format!("{field}[{index}]"));
                }
            });
    }
    meta.organizations
        .as_ref()
        .into_iter()
        .flat_map(|values| values.iter())
        .enumerate()
        .for_each(|(index, organization)| {
            let primary = organization.identifiers.iter().find_map(typed_identifier);
            let identity = primary
                .clone()
                .unwrap_or_else(|| key(&[record.source, record.path, "organization", &index.to_string()]));
            let owner = add_node(nodes, GraphNode::new(GraphNodeKind::Organization, &identity, &organization.name, record));
            add_edge(
                edges,
                GraphEdgeKind::Involves,
                &activity,
                &owner,
                record,
                &format!("meta.organizations[{index}]"),
                None,
            );
            organization.identifiers.iter().enumerate().for_each(|(id_index, identifier)| {
                if let Some(normalized) = typed_identifier(identifier) {
                    add_identifier(
                        nodes,
                        edges,
                        &owner,
                        &identifier.value,
                        &normalized,
                        record,
                        &format!("meta.organizations[{index}].identifiers[{id_index}]"),
                    );
                }
            });
        });
    meta.contributors
        .as_ref()
        .into_iter()
        .flat_map(|values| values.iter())
        .enumerate()
        .for_each(|(index, contributor)| {
            let primary = contributor
                .identifiers
                .iter()
                .find_map(|identifier| normalized_identifier(&identifier.value, identifier.scheme.as_deref().or(identifier.scheme_uri.as_deref())));
            let identity = primary
                .clone()
                .unwrap_or_else(|| key(&[record.source, record.path, "contributor", &index.to_string()]));
            let owner = add_node(nodes, GraphNode::new(GraphNodeKind::Contributor, &identity, &contributor.name, record));
            add_edge(
                edges,
                GraphEdgeKind::Credits,
                &activity,
                &owner,
                record,
                &format!("meta.contributors[{index}]"),
                None,
            );
            contributor.identifiers.iter().enumerate().for_each(|(id_index, identifier)| {
                if let Some(normalized) = normalized_identifier(&identifier.value, identifier.scheme.as_deref().or(identifier.scheme_uri.as_deref()))
                {
                    add_identifier(
                        nodes,
                        edges,
                        &owner,
                        &identifier.value,
                        &normalized,
                        record,
                        &format!("meta.contributors[{index}].identifiers[{id_index}]"),
                    );
                }
            });
        });
    meta.related_resources
        .as_ref()
        .into_iter()
        .flat_map(|values| values.iter())
        .enumerate()
        .for_each(|(index, related)| {
            let reference = &related.identifier;
            let lookup = typed_identifier(reference).unwrap_or_else(|| reference.value.trim().to_ascii_lowercase());
            let matches = aliases.get(&(record.source.to_string(), lookup.clone()));
            let target = matches
                .filter(|matches| matches.len() == 1)
                .and_then(|matches| matches.iter().next())
                .cloned()
                .unwrap_or_else(|| {
                    add_node(
                        nodes,
                        GraphNode::new(GraphNodeKind::Reference, &key(&[record.source, &lookup]), &reference.value, record),
                    )
                });
            add_edge(
                edges,
                GraphEdgeKind::RelatesTo,
                &activity,
                &target,
                record,
                &format!("meta.relatedResources[{index}]"),
                Some(&related.relation_type),
            );
        });
}
fn aliases(records: &[GraphRecord<'_>]) -> BTreeMap<(String, String), BTreeSet<String>> {
    records.iter().fold(BTreeMap::new(), |mut aliases, record| {
        let activity_id = urn(GraphNodeKind::Activity.as_ref(), &key(&[record.source, record.path]));
        let values = [record.path, record.activity.meta.identifier.as_str()]
            .into_iter()
            .map(|value| value.trim().to_ascii_lowercase())
            .chain(
                record
                    .activity
                    .meta
                    .doi
                    .iter()
                    .flat_map(|values| values.iter())
                    .filter_map(|value| publication_identifier(value)),
            )
            .chain(
                record
                    .activity
                    .meta
                    .handle
                    .iter()
                    .flat_map(|values| values.iter())
                    .filter_map(|value| normalized_identifier(value, Some("Handle"))),
            )
            .chain(
                record
                    .activity
                    .meta
                    .raid
                    .iter()
                    .flat_map(|values| values.iter())
                    .filter_map(|value| normalized_identifier(value, Some("RAiD"))),
            );
        values.for_each(|value| {
            aliases
                .entry((record.source.to_string(), value))
                .or_insert_with(BTreeSet::new)
                .insert(activity_id.clone());
        });
        aliases
    })
}
fn key(parts: &[&str]) -> String {
    parts.iter().map(|part| format!("{}:{part}", part.len())).collect()
}
fn normalized_identifier(value: &str, scheme: Option<&str>) -> Option<String> {
    scheme
        .filter(|scheme| !scheme.trim().is_empty())
        .and_then(|scheme| {
            Identifier {
                kind: PID::infer(scheme),
                value: value.to_string(),
            }
            .normalized()
        })
        .map(|identifier| identifier.identity_key())
}
fn publication_identifier(value: &str) -> Option<String> {
    let scheme = match PublicationIdentifierType::from(value) {
        | PublicationIdentifierType::Doi(_) => Some("DOI"),
        | PublicationIdentifierType::Arxiv(_) => Some("arXiv"),
        | PublicationIdentifierType::Unknown => None,
    };
    normalized_identifier(value, scheme)
}
fn typed_identifier(identifier: &TypedIdentifier) -> Option<String> {
    normalized_identifier(&identifier.value, identifier.scheme.as_deref().or(identifier.scheme_uri.as_deref()))
}
fn urn(kind: &str, key: &str) -> String {
    let hex = key.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    format!("urn:acorn:graph:node:{kind}:{hex}")
}

#[cfg(test)]
#[path = "tests/graph.rs"]
mod tests;
