//! Provider-neutral research activity enrichment.
use crate::io::api::citeas::ToCitations;
use crate::io::api::{openalex, osti, ror, Configuration, Param};
use crate::io::document::{SourceDocument, SourceDocuments};
use crate::io::workflow::{EnrichmentCandidate, EnrichmentCandidateKey, EnrichmentDecision, EnrichmentOrigin, EnrichmentResult};
use crate::io::{ApiResult, InputOutput};
use crate::param;
use crate::util::StringConversion;
use acorn_core::prelude::{format, String, ToString, Vec};
use acorn_core::util::merge_unique_by;
use acorn_core::Location;
use acorn_schema::enrichment::Provider;
use acorn_schema::namespaces::DEFAULT_ORCID_SCHEMA_URI;
use acorn_schema::pid::{PersistentIdentifier, PersistentIdentifierConvert, PersistentIdentifierParse, PublicationIdentifierType, DOI, ORCID, ROR};
use acorn_schema::research_activity::output::{Contributor as OutputContributor, ResearchOutput};
use acorn_schema::research_activity::{ActivityContributor, AgentIdentifier, EnrichmentProvenance, ResearchActivity, ResearchActivityMetadata};
use acorn_schema::standard::cff::{Agent as CffAgent, Cff, Person as CffPerson, Reference as CffReference};
use acorn_schema::standard::{datacite, invenio};
use acorn_schema::{ContactPoint, OneOrMany, Organization, Website};
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use serde::Serialize;
use strum::IntoEnumIterator;
use tracing::warn;

mod cff;
mod datacite_impl;
mod invenio_impl;
mod orcid;
mod research_activity;
mod source;
mod vocabulary;
#[cfg(test)]
pub(super) use orcid::{orcid_decision, OrcidBatch, OrcidResolution};
pub(super) use orcid::{OrcidBatchResult, OrcidEnrichment, OrcidIdentity, OrcidPersonRef};
#[cfg(test)]
pub(super) use research_activity::ResearchActivityEnrichment;
pub(crate) use vocabulary::vocabulary_decisions;

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
    /// Resolve missing person ORCIDs or fill missing person names from ORCID profiles.
    async fn enrich_orcid(&self) -> ApiResult<(Self, Vec<String>, Vec<String>)>
    where
        Self: Clone + Sized,
    {
        Ok((self.clone(), Vec::new(), Vec::new()))
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
    /// Return the fields changed by a provider.
    fn enrichment_fields(before: &serde_json::Value, after: &serde_json::Value) -> Vec<String>;
    /// Return the lookup source associated with a provider.
    fn enrichment_source(&self, provider: Provider) -> Option<String>;
    /// Apply one provider and return updated data, conflicts, and non-fatal failures.
    async fn enrich_provider(&self, provider: Provider, source: &Location) -> ApiResult<(Self, Vec<String>, Vec<String>)>;
    /// Apply one provider while retaining any typed choices it reports.
    async fn enrich_provider_with_decisions(&self, provider: Provider, source: &Location) -> ApiResult<EnrichmentResult<Self>> {
        self.enrich_provider(provider, source)
            .await
            .map(|(data, conflicts, failures)| EnrichmentResult::builder().data(data).conflicts(conflicts).failures(failures).build())
    }
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
struct EnrichmentFields;
/// Ordered enrichment pipeline.
#[derive(Clone, Debug)]
pub struct Pipeline {
    providers: Vec<Provider>,
}
impl EnrichmentFields {
    fn changed<const N: usize>(before: &serde_json::Value, after: &serde_json::Value, paths: [&str; N]) -> Vec<String> {
        paths
            .into_iter()
            .filter(|path| {
                path.split('.').try_fold(before, |value, component| value.get(component))
                    != path.split('.').try_fold(after, |value, component| value.get(component))
            })
            .map(ToString::to_string)
            .collect()
    }
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
            .fold(EnrichmentResult::builder().data(data).build(), |result, provider| {
                let source = source.clone();
                async move {
                    let lookup = result.data.enrichment_source(provider);
                    let original = result.data.clone();
                    let before = serde_json::to_value(&result.data).unwrap_or_default();
                    let enriched = result.data.enrich_provider_with_decisions(provider, &source).await;
                    match enriched {
                        | Ok(EnrichmentResult {
                            data,
                            conflicts,
                            decisions,
                            failures,
                        }) => {
                            let after = serde_json::to_value(&data).unwrap_or_default();
                            let fields = T::enrichment_fields(&before, &after);
                            let data = match fields.is_empty() {
                                | true => data,
                                | false => data.with_enrichment_provenance(provider, lookup, fields),
                            };
                            let conflicts = result
                                .conflicts
                                .into_iter()
                                .chain(conflicts.into_iter().map(|value| format!("{provider}: {value}")))
                                .collect();
                            let failures = result
                                .failures
                                .into_iter()
                                .chain(failures.into_iter().map(|value| format!("{provider}: {value}")))
                                .collect();
                            EnrichmentResult::builder()
                                .data(data)
                                .conflicts(conflicts)
                                .decisions(result.decisions.into_iter().chain(decisions).collect())
                                .failures(failures)
                                .build()
                        }
                        | Err(why) => EnrichmentResult::builder()
                            .data(original)
                            .conflicts(result.conflicts)
                            .decisions(result.decisions)
                            .failures(result.failures.into_iter().chain([format!("{provider}: {why}")]).collect())
                            .build(),
                    }
                }
            })
            .await)
    }
}
