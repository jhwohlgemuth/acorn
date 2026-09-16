//! Provider-neutral research activity enrichment.
use crate::io::api::citeas::ToCitations;
use crate::io::api::{openalex, orcid, osti, ror, Configuration};
use crate::io::document::{SourceDocument, SourceDocuments};
use crate::io::workflow::EnrichmentResult;
use crate::io::{ApiResult, InputOutput};
use crate::param;
use crate::util::StringConversion;
use acorn_core::prelude::{format, String, ToString, Vec};
use acorn_core::util::merge_unique_by;
use acorn_core::Location;
use acorn_schema::enrichment::Provider;
use acorn_schema::pid::{PersistentIdentifierParse, PublicationIdentifierType, DOI};
use acorn_schema::research_activity::output::ResearchOutput;
use acorn_schema::research_activity::{EnrichmentProvenance, ResearchActivity, ResearchActivityMetadata};
use acorn_schema::{ContactPoint, OneOrMany, Website};
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use serde::Serialize;
use strum::IntoEnumIterator;
use tracing::warn;

pub(super) const ENRICHMENT_FIELDS: [&str; 8] = [
    "title",
    "subtitle",
    "contact.familyName",
    "contact.givenName",
    "contact.identifier",
    "meta.doi",
    "meta.outputs",
    "meta.ror",
];
/// Provider-specific enrichment operations for a supplied input type.
#[async_trait]
pub trait Enrich<Input: Send = ()>: Send + Sync {
    /// Result produced by enrichment.
    type Output: Send;
    /// Enrich this value using its configured providers.
    async fn enrich(&self, input: Input) -> Self::Output;
    /// Enrich DOI outputs with CiteAs metadata when applicable.
    async fn enrich_citeas(&self) -> ApiResult<(Self, Option<String>, Vec<String>)>
    where
        Self: Clone + Sized,
    {
        Ok((self.clone(), None, Vec::new()))
    }
    /// Enrich scholarly outputs with OpenAlex metadata when applicable.
    async fn enrich_openalex(&self, _source: &Location) -> ApiResult<(Self, Option<String>, Vec<String>)>
    where
        Self: Clone + Sized,
    {
        Ok((self.clone(), None, Vec::new()))
    }
    /// Resolve a missing contact ORCID or fill missing contact names from an ORCID profile.
    async fn enrich_orcid(&self) -> ApiResult<(Self, Option<String>)>
    where
        Self: Clone + Sized,
    {
        Ok((self.clone(), None))
    }
    /// Enrich DOE CODE metadata using an OSTI identifier or DOI when applicable.
    async fn enrich_osti(&self) -> ApiResult<(Self, Option<String>)>
    where
        Self: Clone + Sized,
    {
        Ok((self.clone(), None))
    }
    /// Resolve a missing organization ROR when an unambiguous exact match exists.
    async fn enrich_ror(&self) -> ApiResult<(Self, Option<String>)>
    where
        Self: Clone + Sized,
    {
        Ok((self.clone(), None))
    }
}
/// Metadata-standard hooks used by the provider pipeline.
///
/// Implementing this trait lets a standard use [`Pipeline`] without coupling
/// the pipeline's planning and change types to [`ResearchActivity`].
#[async_trait]
pub trait EnrichmentMetadata: InputOutput + Clone + Serialize + Send + Sync {
    /// Return the lookup source associated with a provider.
    fn enrichment_source(&self, provider: Provider) -> Option<String>;
    /// Apply one provider and return updated data, an optional conflict, and non-fatal failures.
    async fn enrich_provider(&self, provider: Provider, source: &Location) -> ApiResult<(Self, Option<String>, Vec<String>)>;
    /// Return the fields changed by a provider.
    fn enrichment_fields(before: &serde_json::Value, after: &serde_json::Value) -> Vec<String>;
    /// Record provenance for fields changed by a provider.
    fn with_enrichment_provenance(self, provider: Provider, source: Option<String>, fields: Vec<String>) -> Self;
}
/// Standard-neutral access to publication identifiers and research outputs.
///
/// Metadata standards implement this alongside [`InputOutput`] to opt into
/// providers such as OpenAlex without making provider code depend on one schema.
pub trait ResearchOutputMetadata: InputOutput + Clone + Send + Sync {
    /// Return publication identifiers that providers can resolve.
    fn publication_identifiers(&self) -> Vec<String>;
    /// Return the standard's outputs in ACORN's shared output representation.
    fn research_outputs(&self) -> Vec<ResearchOutput>;
    /// Replace the standard's outputs after enrichment.
    fn with_research_outputs(self, outputs: Vec<ResearchOutput>) -> Self;
}
/// Ordered enrichment pipeline.
#[derive(Clone, Debug)]
pub struct Pipeline {
    providers: Vec<Provider>,
}
impl Default for Pipeline {
    fn default() -> Self {
        Self::new(Provider::iter().collect())
    }
}
impl Pipeline {
    /// Construct a pipeline in the supplied deterministic provider order.
    pub fn new(providers: Vec<Provider>) -> Self {
        Self { providers }
    }
}
#[async_trait]
impl<T> Enrich<(Location, T)> for Pipeline
where
    T: EnrichmentMetadata + 'static,
{
    type Output = ApiResult<EnrichmentResult<T>>;
    async fn enrich(&self, (source, data): (Location, T)) -> Self::Output {
        Ok(stream::iter(self.providers.iter().copied())
            .fold(
                EnrichmentResult {
                    data,
                    conflicts: Vec::new(),
                    failures: Vec::new(),
                },
                |result, provider| {
                    let source = source.clone();
                    async move {
                        let lookup = result.data.enrichment_source(provider);
                        let original = result.data.clone();
                        let before = serde_json::to_value(&result.data).unwrap_or_default();
                        let enriched = result.data.enrich_provider(provider, &source).await;
                        match enriched {
                            | Ok((data, conflict, failures)) => {
                                let after = serde_json::to_value(&data).unwrap_or_default();
                                let fields = T::enrichment_fields(&before, &after);
                                let data = match fields.is_empty() {
                                    | true => data,
                                    | false => data.with_enrichment_provenance(provider, lookup, fields),
                                };
                                EnrichmentResult {
                                    data,
                                    conflicts: result
                                        .conflicts
                                        .into_iter()
                                        .chain(conflict.map(|value| format!("{provider}: {value}")))
                                        .collect(),
                                    failures: result
                                        .failures
                                        .into_iter()
                                        .chain(failures.into_iter().map(|value| format!("{provider}: {value}")))
                                        .collect(),
                                }
                            }
                            | Err(why) => EnrichmentResult {
                                data: original,
                                conflicts: result.conflicts,
                                failures: result.failures.into_iter().chain([format!("{provider}: {why}")]).collect(),
                            },
                        }
                    }
                },
            )
            .await)
    }
}
#[async_trait]
impl Enrich<()> for ResearchActivity {
    type Output = ApiResult<(Self, Option<String>)>;
    async fn enrich(&self, (): ()) -> Self::Output {
        Ok((self.clone(), None))
    }
    async fn enrich_citeas(&self) -> ApiResult<(Self, Option<String>, Vec<String>)> {
        let (data, failures) = stream::iter(self.meta.doi.clone().unwrap_or_default())
            .fold((self.clone(), Vec::new()), |(data, failures), identifier| async move {
                match data.meta.outputs.as_ref().is_some_and(|outputs| {
                    outputs
                        .iter()
                        .any(|output| output.doi.as_ref().is_some_and(|doi| DOI::format(doi) == DOI::format(&identifier)))
                }) {
                    | true => (data, failures),
                    | false => match PublicationIdentifierType::from(identifier.as_str()).to_citations().await {
                        | Ok(citations) => {
                            let metadata = citations.metadata;
                            let output = ResearchOutput::init()
                                .identifier(metadata.id)
                                .maybe_doi((!metadata.doi.is_empty()).then_some(DOI::format(&metadata.doi)))
                                .title(metadata.title)
                                .kind(metadata.resource_type)
                                .maybe_publication_year((metadata.year > 0).then_some(metadata.year as i32))
                                .maybe_websites(
                                    (!metadata.url.is_empty()).then(|| vec![Website::at(metadata.url).description("CiteAs resource").build()]),
                                )
                                .build();
                            (
                                Self {
                                    meta: ResearchActivityMetadata {
                                        outputs: Some(data.meta.outputs.clone().unwrap_or_default().into_iter().chain([output]).collect()),
                                        ..data.meta
                                    },
                                    ..data
                                },
                                failures,
                            )
                        }
                        | Err(why) => (data, failures.into_iter().chain([format!("{identifier}: {why}")]).collect()),
                    },
                }
            })
            .await;
        Ok((data, None, failures))
    }
    async fn enrich_openalex(&self, source: &Location) -> ApiResult<(Self, Option<String>, Vec<String>)> {
        openalex::Provider::from_env()
            .enrich((source.clone(), self.clone()))
            .await
            .map(|result| (result.data, None, result.failures))
    }
    async fn enrich_orcid(&self) -> ApiResult<(Self, Option<String>)> {
        let identifier = self.contact.identifier.as_deref().filter(|value| !value.trim().is_empty());
        let given_name_missing = self.contact.given_name.trim().is_empty();
        let family_name_missing = self.contact.family_name.trim().is_empty();
        match (identifier, given_name_missing || family_name_missing) {
            | (Some(identifier), true) => {
                let options = orcid::Options::from_env().with_identifier(identifier);
                orcid::record(&options).await.map(|profile| {
                    (
                        Self {
                            contact: with_orcid_names(&self.contact, &profile),
                            ..self.clone()
                        },
                        None,
                    )
                })
            }
            | (None, false) => {
                let options = orcid::Options::from_env().with_params(vec![param!(
                    QueryPair,
                    "q",
                    (
                        ("given-names", self.contact.given_name.as_str()),
                        ("family-name", self.contact.family_name.as_str())
                    )
                )]);
                orcid::search(&options).await.map(|response| {
                    let (contact, conflict) = with_orcid_identifier(&self.contact, response);
                    (Self { contact, ..self.clone() }, conflict)
                })
            }
            | _ => Ok((self.clone(), None)),
        }
    }
    async fn enrich_osti(&self) -> ApiResult<(Self, Option<String>)> {
        let project = match self.meta.identifier.strip_prefix("osti-").and_then(|value| value.parse::<u64>().ok()) {
            | Some(code) => osti::record(code).await.map(Some),
            | None => match self.meta.doi.as_ref().and_then(|values| values.first()) {
                | Some(doi) => osti::search(&osti::Options::default().with_query(doi))
                    .await
                    .map(|response| match response.results {
                        | osti::SearchResults::Projects(projects) => projects.into_iter().next(),
                        | _ => None,
                    }),
                | None => Ok(None),
            },
        };
        project.map(|project| match project {
            | Some(project) => {
                let candidate = Self::from(project);
                let doi = merge_unique_by(
                    self.meta.doi.clone().unwrap_or_default(),
                    candidate.meta.doi.unwrap_or_default(),
                    |left, right| left.eq_ignore_ascii_case(right),
                );
                (
                    Self {
                        subtitle: self.subtitle.clone().or(candidate.subtitle),
                        meta: ResearchActivityMetadata {
                            doi: (!doi.is_empty()).then(|| doi.into()),
                            ..self.meta.clone()
                        },
                        ..self.clone()
                    },
                    None,
                )
            }
            | None => (self.clone(), None),
        })
    }
    async fn enrich_ror(&self) -> ApiResult<(Self, Option<String>)> {
        let organization = self
            .contact
            .affiliation
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(self.contact.organization.as_str())
            .trim()
            .to_string();
        match self.meta.ror.as_ref().is_some_and(|values| !values.is_empty()) || organization.is_empty() {
            | true => Ok((self.clone(), None)),
            | false => {
                let options = ror::Options::from_env().with_params(vec![param!(FieldList, "query", organization.as_str())]);
                ror::search(&options).await.map(|response| {
                    let matches = response
                        .items
                        .into_iter()
                        .filter(|record| {
                            matches!(record.status, ror::Status::Active)
                                && record.names.iter().any(|name| {
                                    name.types
                                        .iter()
                                        .any(|kind| matches!(kind, ror::OrganizationNameType::RorDisplay | ror::OrganizationNameType::Alias))
                                        && name.value.normalized() == organization.normalized()
                                })
                        })
                        .collect::<Vec<_>>();
                    let matches = match matches.len() > 1 {
                        | true => self.contact.email.rsplit_once('@').map_or_else(Vec::new, |(_, domain)| {
                            matches
                                .into_iter()
                                .filter(|record| record.domains.iter().any(|value| value.eq_ignore_ascii_case(domain)))
                                .collect()
                        }),
                        | false => matches,
                    };
                    match matches.as_slice() {
                        | [record] => (
                            Self {
                                meta: ResearchActivityMetadata {
                                    ror: Some(vec![record.identifier.clone()].into()),
                                    ..self.meta.clone()
                                },
                                ..self.clone()
                            },
                            None,
                        ),
                        | [] => (self.clone(), None),
                        | _ => (self.clone(), Some("ambiguous exact organization match; ROR was not changed".to_string())),
                    }
                })
            }
        }
    }
}
#[async_trait]
impl EnrichmentMetadata for ResearchActivity {
    fn enrichment_source(&self, provider: Provider) -> Option<String> {
        self.source_from(provider)
    }
    async fn enrich_provider(&self, provider: Provider, source: &Location) -> ApiResult<(Self, Option<String>, Vec<String>)> {
        match provider {
            | Provider::CiteAs => self.enrich_citeas().await,
            | Provider::OpenAlex => self.enrich_openalex(source).await,
            | Provider::Orcid => self.enrich_orcid().await.map(|(data, conflict)| (data, conflict, Vec::new())),
            | Provider::Osti => self.enrich_osti().await.map(|(data, conflict)| (data, conflict, Vec::new())),
            | Provider::Ror => self.enrich_ror().await.map(|(data, conflict)| (data, conflict, Vec::new())),
        }
    }
    fn enrichment_fields(before: &serde_json::Value, after: &serde_json::Value) -> Vec<String> {
        ENRICHMENT_FIELDS
            .into_iter()
            .filter(|path| {
                path.split('.').try_fold(before, |value, component| value.get(component))
                    != path.split('.').try_fold(after, |value, component| value.get(component))
            })
            .map(ToString::to_string)
            .collect()
    }
    fn with_enrichment_provenance(self, provider: Provider, source: Option<String>, fields: Vec<String>) -> Self {
        Self {
            meta: ResearchActivityMetadata {
                enrichment: Some(
                    self.meta
                        .enrichment
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .chain([EnrichmentProvenance::init()
                            .provider(provider)
                            .maybe_source(source)
                            .fields(fields)
                            .build()])
                        .collect(),
                ),
                ..self.meta
            },
            ..self
        }
    }
}
impl ResearchOutputMetadata for ResearchActivity {
    fn publication_identifiers(&self) -> Vec<String> {
        self.meta.doi.clone().map(OneOrMany::into_vec).unwrap_or_default()
    }
    fn research_outputs(&self) -> Vec<ResearchOutput> {
        self.meta.outputs.clone().map(OneOrMany::into_vec).unwrap_or_default()
    }
    fn with_research_outputs(self, outputs: Vec<ResearchOutput>) -> Self {
        Self {
            meta: ResearchActivityMetadata {
                outputs: (!outputs.is_empty()).then(|| outputs.into()),
                ..self.meta
            },
            ..self
        }
    }
}
#[async_trait]
impl Enrich<()> for SourceDocument {
    type Output = (Self, Option<String>, Vec<String>);
    async fn enrich(&self, (): ()) -> Self::Output {
        match ResearchActivity::parse(&self.content) {
            | Some(data) => match Pipeline::default().enrich((Location::from(self.source.as_str()), data)).await {
                | Ok(result) => {
                    result.conflicts.iter().for_each(|conflict| warn!("Enrichment conflict — {conflict}"));
                    match serde_json::to_string(&result.data) {
                        | Ok(content) => (
                            Self::init()
                                .content(content.clone())
                                .format(self.format.clone())
                                .source(self.source.clone())
                                .build(),
                            Some(content),
                            result.failures,
                        ),
                        | Err(why) => (self.clone(), None, result.failures.into_iter().chain([why.to_string()]).collect()),
                    }
                }
                | Err(why) => (self.clone(), None, vec![why.to_string()]),
            },
            | None => (self.clone(), None, Vec::new()),
        }
    }
}
#[async_trait]
impl Enrich<bool> for SourceDocuments {
    type Output = (Self, Vec<String>, Vec<String>);
    async fn enrich(&self, enabled: bool) -> Self::Output {
        match enabled {
            | false => (self.clone(), Vec::new(), Vec::new()),
            | true => stream::iter(self.0.iter())
                .then(|document| document.enrich(()))
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .fold(
                    (Self::default(), Vec::new(), Vec::new()),
                    |(documents, enriched, failures), (document, content, errors)| {
                        (
                            Self(documents.0.into_iter().chain([document]).collect()),
                            enriched.into_iter().chain(content).collect(),
                            failures.into_iter().chain(errors).collect(),
                        )
                    },
                ),
        }
    }
}
fn with_orcid_identifier(contact: &ContactPoint, response: orcid::SearchResponse) -> (ContactPoint, Option<String>) {
    let matches = response
        .results
        .into_iter()
        .filter(|result| {
            result
                .given_names
                .as_deref()
                .is_some_and(|value| value.normalized() == contact.given_name.normalized())
                && result
                    .family_names
                    .as_deref()
                    .is_some_and(|value| value.normalized() == contact.family_name.normalized())
        })
        .collect::<Vec<_>>();
    let matches = match matches.len() > 1 {
        | true => matches
            .into_iter()
            .filter(|result| {
                (!contact.email.trim().is_empty()
                    && result
                        .emails
                        .as_ref()
                        .is_some_and(|values| values.iter().any(|value| value.eq_ignore_ascii_case(contact.email.trim()))))
                    || (!contact.organization.trim().is_empty()
                        && result
                            .institution_names
                            .as_ref()
                            .is_some_and(|values| values.iter().any(|value| value.normalized() == contact.organization.normalized())))
            })
            .collect::<Vec<_>>(),
        | false => matches,
    };
    match matches.as_slice() {
        | [result] => (
            ContactPoint {
                identifier: result.orcid_id.clone(),
                ..contact.clone()
            },
            None,
        ),
        | [] => (contact.clone(), None),
        | _ => (
            contact.clone(),
            Some("ambiguous exact contact match; identifier was not changed".to_string()),
        ),
    }
}
fn with_orcid_names(contact: &ContactPoint, profile: &orcid::SearchResult) -> ContactPoint {
    let fill = |current: &str, candidate: &Option<String>| match current.trim().is_empty() {
        | true => candidate
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| current.to_string()),
        | false => current.to_string(),
    };
    ContactPoint {
        family_name: fill(&contact.family_name, &profile.family_names),
        given_name: fill(&contact.given_name, &profile.given_names),
        ..contact.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contact(given_name: &str, family_name: &str, identifier: Option<&str>) -> ContactPoint {
        ContactPoint {
            given_name: given_name.to_string(),
            family_name: family_name.to_string(),
            identifier: identifier.map(ToString::to_string),
            ..ContactPoint::default()
        }
    }
    fn profile(identifier: &str, given_name: Option<&str>, family_name: Option<&str>) -> orcid::SearchResult {
        orcid::SearchResult {
            orcid_id: Some(identifier.to_string()),
            given_names: given_name.map(ToString::to_string),
            family_names: family_name.map(ToString::to_string),
            credit_name: None,
            emails: None,
            institution_names: None,
            other_name: None,
        }
    }
    fn response(results: impl IntoIterator<Item = orcid::SearchResult>) -> orcid::SearchResponse {
        let results = results.into_iter().collect::<Vec<_>>();
        orcid::SearchResponse {
            num_found: results.len(),
            namespace: String::new(),
            results,
        }
    }
    #[tokio::test]
    async fn test_orcid_enrichment_skips_complete_or_incomplete_contacts() {
        let incomplete = ResearchActivity {
            contact: contact("Jane", "", None),
            ..ResearchActivity::default()
        };
        let (actual, conflict) = incomplete.enrich_orcid().await.expect("incomplete contact should be skipped");
        assert!(actual.contact.identifier.is_none());
        assert_eq!(actual.contact.given_name, "Jane");
        assert!(actual.contact.family_name.is_empty());
        assert!(conflict.is_none());
        let complete = ResearchActivity {
            contact: contact("Jason", "Wohlgemuth", Some("0000-0002-2057-9115")),
            ..ResearchActivity::default()
        };
        let (actual, conflict) = complete.enrich_orcid().await.expect("complete contact should be skipped");
        assert_eq!(actual.contact.given_name, "Jason");
        assert_eq!(actual.contact.family_name, "Wohlgemuth");
        assert!(conflict.is_none());
    }
    #[test]
    fn test_orcid_identifier_uses_exact_matches_and_reports_ambiguity() {
        let exact = contact(" Jason ", "Wohlgemuth", None);
        let (actual, conflict) = with_orcid_identifier(
            &exact,
            response([
                profile("0000-0002-1825-0097", Some("Other"), Some("Person")),
                profile("0000-0002-2057-9115", Some("jason"), Some("Wohlgemuth")),
            ]),
        );
        assert_eq!(actual.identifier.as_deref(), Some("0000-0002-2057-9115"));
        assert!(conflict.is_none());
        let ambiguous = ContactPoint {
            email: "jane@example.com".to_string(),
            ..contact("Jane", "Doe", None)
        };
        let matches = ["0000-0002-1825-0097", "0000-0002-2057-9115"].map(|identifier| orcid::SearchResult {
            emails: Some(vec![ambiguous.email.clone()]),
            ..profile(identifier, Some("Jane"), Some("Doe"))
        });
        let (actual, conflict) = with_orcid_identifier(&ambiguous, response(matches));
        assert!(actual.identifier.is_none());
        assert_eq!(conflict.as_deref(), Some("ambiguous exact contact match; identifier was not changed"));
    }
    #[test]
    fn test_orcid_names_fill_only_missing_values() {
        let blank = contact("", "", Some("0000-0002-2057-9115"));
        let actual = with_orcid_names(&blank, &profile("0000-0002-2057-9115", Some("Jane"), Some("Doe")));
        assert_eq!(actual.given_name, "Jane");
        assert_eq!(actual.family_name, "Doe");
        let partial = contact("Jane", "", Some("0000-0002-2057-9115"));
        let actual = with_orcid_names(&partial, &profile("0000-0002-2057-9115", Some("Janet"), Some("Doe")));
        assert_eq!(actual.given_name, "Jane");
        assert_eq!(actual.family_name, "Doe");
        let unavailable = contact("", " ", Some("0000-0002-2057-9115"));
        let actual = with_orcid_names(&unavailable, &profile("0000-0002-2057-9115", None, Some(" ")));
        assert!(actual.given_name.is_empty());
        assert_eq!(actual.family_name, " ");
    }
    #[test]
    fn test_orcid_source_prefers_identifier_then_names() {
        let identified = ResearchActivity {
            contact: contact("", "", Some("0000-0002-2057-9115")),
            ..ResearchActivity::default()
        };
        assert_eq!(identified.source_from(Provider::Orcid).as_deref(), Some("0000-0002-2057-9115"));
        let named = ResearchActivity {
            contact: contact("Jane", "Doe", None),
            ..ResearchActivity::default()
        };
        assert_eq!(named.source_from(Provider::Orcid).as_deref(), Some("Jane Doe"));
    }
}
