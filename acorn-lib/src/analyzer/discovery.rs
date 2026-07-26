//! Pure artifact discovery and grouping helpers.
use crate::schema::pid::{Identifier, PID};
use serde::{Deserialize, Serialize};
/// Candidate artifact record before standard-specific mapping
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArtifactCandidate {
    /// Identifiers associated with this artifact
    pub identifiers: Vec<Identifier>,
    /// Resolver-proven canonical URL
    pub canonical_url: Option<String>,
    /// Enriched title
    pub title: Option<String>,
    /// Enriched author display names
    pub authors: Vec<String>,
}
/// Discover and normalize supported identifiers from prose or metadata text
pub fn discover_identifiers(content: &str) -> Vec<Identifier> {
    content
        .split_whitespace()
        .filter_map(|value| Identifier::new(value).normalize())
        .fold(Vec::new(), |mut identifiers, identifier| {
            if !identifiers.contains(&identifier) {
                identifiers.push(identifier);
            }
            identifiers
        })
}
/// Group artifacts only when canonical identifiers or enriched metadata prove equivalence
pub fn group_artifacts(candidates: Vec<ArtifactCandidate>) -> Vec<ArtifactCandidate> {
    candidates.into_iter().fold(Vec::<ArtifactCandidate>::new(), |mut grouped, candidate| {
        match grouped.iter_mut().find(|existing| same_artifact(existing, &candidate)) {
            | Some(existing) => {
                candidate.identifiers.into_iter().for_each(|identifier| {
                    if !existing.identifiers.contains(&identifier) {
                        existing.identifiers.push(identifier);
                    }
                });
                existing.identifiers.sort();
                if existing.canonical_url.is_none() {
                    existing.canonical_url = candidate.canonical_url;
                }
                if existing.title.is_none() {
                    existing.title = candidate.title;
                }
                if existing.authors.is_empty() {
                    existing.authors = candidate.authors;
                }
            }
            | None => grouped.push(candidate),
        }
        grouped
    })
}

fn same_artifact(left: &ArtifactCandidate, right: &ArtifactCandidate) -> bool {
    let canonical_match = left
        .identifiers
        .iter()
        .filter(|identifier| matches!(identifier.kind, PID::DOI | PID::URL))
        .any(|identifier| right.identifiers.contains(identifier))
        || left
            .canonical_url
            .as_ref()
            .zip(right.canonical_url.as_ref())
            .is_some_and(|(left, right)| left == right);
    let metadata_match = left
        .title
        .as_ref()
        .zip(right.title.as_ref())
        .filter(|(left, right)| normalized_text(left) == normalized_text(right))
        .is_some()
        && !left.authors.is_empty()
        && normalized_authors(&left.authors) == normalized_authors(&right.authors);
    canonical_match || metadata_match
}

fn normalized_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ").to_ascii_lowercase()
}

fn normalized_authors(values: &[String]) -> Vec<String> {
    let mut values = values.iter().map(|value| normalized_text(value)).collect::<Vec<_>>();
    values.sort();
    values
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, clippy::unwrap_used)]
    use super::*;
    #[test]
    fn identifier_hash_is_stable_and_short() {
        let identifier = Identifier::new("doi:10.1234/abc").normalize().unwrap();
        assert_eq!(identifier.identifier_hash().len(), 12);
        assert_eq!(identifier.identifier_hash(), identifier.identifier_hash());
    }
    #[test]
    fn discovers_supported_identifiers_without_duplicates() {
        let identifiers = discover_identifiers("Results: doi:10.1234/example and https://doi.org/10.1234/example plus https://example.org/artifact");
        assert_eq!(identifiers.len(), 2);
        assert_eq!(identifiers[0].kind, PID::DOI);
        assert_eq!(identifiers[1].kind, PID::URL);
    }

    #[test]
    fn groups_only_proven_equivalent_candidates() {
        let doi = Identifier::new("doi:10.1234/abc").normalize().unwrap();
        let url = Identifier::new("https://example.org/artifact").normalize().unwrap();
        let candidates = vec![
            ArtifactCandidate {
                identifiers: vec![doi.clone()],
                title: Some("A Result".to_string()),
                authors: vec!["Alice Example".to_string()],
                ..ArtifactCandidate::default()
            },
            ArtifactCandidate {
                identifiers: vec![doi, url],
                ..ArtifactCandidate::default()
            },
            ArtifactCandidate {
                identifiers: vec![Identifier::new("doi:10.9999/other").normalize().unwrap()],
                title: Some("A Different Result".to_string()),
                authors: vec!["Alice Example".to_string()],
                ..ArtifactCandidate::default()
            },
        ];
        let grouped = group_artifacts(candidates);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].identifiers.len(), 2);
    }
}
