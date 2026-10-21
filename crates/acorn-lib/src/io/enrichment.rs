//! Provider-neutral research activity enrichment.
use crate::io::api::citeas::ToCitations;
use crate::io::api::{openalex, orcid, osti, ror, Configuration, Param};
use crate::io::document::{SourceDocument, SourceDocuments};
use crate::io::workflow::EnrichmentResult;
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

pub(super) const ENRICHMENT_FIELDS: [&str; 9] = [
    "title",
    "subtitle",
    "contact.familyName",
    "contact.givenName",
    "contact.identifier",
    "meta.contributors",
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
#[async_trait]
pub(super) trait OrcidExt: Clone + Send + Sync {
    async fn enrich_orcid_people(&self) -> ApiResult<(Self, Vec<String>, Vec<String>)> {
        let result = OrcidBatch::new(self.orcid_people(None)).enrich().await;
        Ok((
            self.clone().with_orcid_profiles(&result, None),
            result.conflicts.clone(),
            result.failures.clone(),
        ))
    }
    fn orcid_display_name(&self) -> Option<String> {
        None
    }
    fn orcid_fill(&self, current: &Option<String>, _candidate: &Option<String>) -> Option<String> {
        current.clone()
    }
    fn orcid_identifier(&self) -> Option<String> {
        None
    }
    fn orcid_identity(&self) -> Option<OrcidIdentity> {
        None
    }
    fn orcid_identity_with_affiliations(&self, _affiliations: &Option<Vec<invenio::Affiliation>>) -> Option<OrcidIdentity> {
        self.orcid_identity()
    }
    fn orcid_people(&self, _path: Option<&str>) -> Vec<OrcidPersonRef> {
        Vec::new()
    }
    fn orcid_source(&self) -> Option<String> {
        OrcidBatch::new(self.orcid_people(None)).source()
    }
    fn with_orcid_profile(self, _profile: Option<&orcid::SearchResult>) -> Self {
        self
    }
    fn with_orcid_profiles(self, _result: &OrcidBatchResult, _path: Option<&str>) -> Self {
        self
    }
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
pub(super) enum OrcidResolution {
    Ambiguous,
    Missing,
    Resolved(orcid::SearchResult),
}
struct EnrichmentFields;
#[derive(Clone, Debug, Default)]
pub(super) struct OrcidBatch {
    pub(super) requests: Vec<OrcidRequest>,
}
#[derive(Clone, Debug, Default)]
pub(super) struct OrcidBatchResult {
    pub(super) conflicts: Vec<String>,
    pub(super) failures: Vec<String>,
    pub(super) profiles: Vec<(String, orcid::SearchResult)>,
}
#[derive(Clone, Debug)]
pub(super) struct OrcidIdentity {
    affiliations: Vec<String>,
    display_name: String,
    email: Option<String>,
    family_name: Option<String>,
    given_name: Option<String>,
    identifier: Option<String>,
}
#[derive(Clone, Debug)]
pub(super) struct OrcidPersonRef {
    identity: OrcidIdentity,
    path: String,
}
#[derive(Clone, Debug)]
pub(super) struct OrcidRequest {
    identity: OrcidIdentity,
    pub(super) paths: Vec<String>,
}
/// Ordered enrichment pipeline.
#[derive(Clone, Debug)]
pub struct Pipeline {
    providers: Vec<Provider>,
}
impl OrcidExt for ActivityContributor {
    fn orcid_identity(&self) -> Option<OrcidIdentity> {
        let identifier = self
            .identifiers
            .iter()
            .find(|identifier| identifier.value.is_orcid() || identifier.scheme.as_deref().is_some_and(|scheme| scheme.eq_ignore_ascii_case("ORCID")))
            .map(|identifier| identifier.value.clone());
        OrcidIdentity::new(self.name.clone(), Vec::new(), false, None, None, identifier, None)
    }
    fn with_orcid_profile(self, profile: Option<&orcid::SearchResult>) -> Self {
        let has_orcid = self
            .identifiers
            .iter()
            .any(|identifier| identifier.value.is_orcid() || identifier.scheme.as_deref().is_some_and(|scheme| scheme.eq_ignore_ascii_case("ORCID")));
        let identifier = profile.and_then(OrcidExt::orcid_identifier).map(|value| {
            AgentIdentifier::init()
                .value(ORCID::format(&value))
                .scheme("ORCID")
                .scheme_uri(DEFAULT_ORCID_SCHEMA_URI)
                .build()
        });
        Self {
            identifiers: match (has_orcid, identifier) {
                | (false, Some(identifier)) => self.identifiers.into_iter().chain([identifier]).collect(),
                | _ => self.identifiers,
            },
            ..self
        }
    }
}
#[async_trait]
impl EnrichmentMetadata for Cff {
    fn enrichment_source(&self, provider: Provider) -> Option<String> {
        match provider {
            | Provider::Orcid => self.orcid_source(),
            | _ => None,
        }
    }
    async fn enrich_provider(&self, provider: Provider, _source: &Location) -> ApiResult<(Self, Vec<String>, Vec<String>)> {
        match provider {
            | Provider::Orcid => self.enrich_orcid_people().await,
            | _ => Ok((self.clone(), Vec::new(), Vec::new())),
        }
    }
    fn enrichment_fields(before: &serde_json::Value, after: &serde_json::Value) -> Vec<String> {
        EnrichmentFields::changed(before, after, ["authors", "contact", "preferred-citation", "references"])
    }
    fn with_enrichment_provenance(self, _provider: Provider, _source: Option<String>, _fields: Vec<String>) -> Self {
        self
    }
}
#[async_trait]
impl OrcidExt for Cff {
    fn orcid_people(&self, _path: Option<&str>) -> Vec<OrcidPersonRef> {
        self.authors
            .orcid_people(Some("authors"))
            .into_iter()
            .chain(
                self.contact
                    .as_ref()
                    .map(|people| people.orcid_people(Some("contact")))
                    .unwrap_or_default(),
            )
            .chain(
                self.preferred_citation
                    .as_ref()
                    .map(|reference| reference.orcid_people(Some("preferredCitation")))
                    .unwrap_or_default(),
            )
            .chain(
                self.references
                    .as_ref()
                    .into_iter()
                    .flatten()
                    .enumerate()
                    .flat_map(|(index, reference)| reference.orcid_people(Some(&format!("references[{index}]")))),
            )
            .collect()
    }
    fn with_orcid_profiles(self, result: &OrcidBatchResult, _path: Option<&str>) -> Self {
        Self {
            authors: self.authors.clone().with_orcid_profiles(result, Some("authors")),
            contact: self.contact.clone().map(|people| people.with_orcid_profiles(result, Some("contact"))),
            preferred_citation: self
                .preferred_citation
                .clone()
                .map(|reference| reference.with_orcid_profiles(result, Some("preferredCitation"))),
            references: self.references.clone().map(|references| {
                references
                    .into_iter()
                    .enumerate()
                    .map(|(index, reference)| reference.with_orcid_profiles(result, Some(&format!("references[{index}]"))))
                    .collect()
            }),
            ..self
        }
    }
}
impl OrcidExt for CffAgent {
    fn orcid_identity(&self) -> Option<OrcidIdentity> {
        match self {
            | Self::Entity(_) => None,
            | Self::Person(person) => person.orcid_identity(),
        }
    }
    fn with_orcid_profile(self, profile: Option<&orcid::SearchResult>) -> Self {
        match self {
            | Self::Entity(entity) => Self::Entity(entity),
            | Self::Person(person) => Self::Person(person.with_orcid_profile(profile)),
        }
    }
}
impl OrcidExt for CffPerson {
    fn orcid_identity(&self) -> Option<OrcidIdentity> {
        let display_name = [self.given_names.as_deref(), self.family_names.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        OrcidIdentity::new(
            display_name,
            self.affiliation.clone().into_iter().collect(),
            true,
            self.given_names.clone(),
            self.family_names.clone(),
            self.orcid.clone(),
            self.email.clone(),
        )
    }
    fn with_orcid_profile(self, profile: Option<&orcid::SearchResult>) -> Self {
        match profile {
            | Some(profile) => Self {
                family_names: profile.orcid_fill(&self.family_names, &profile.family_names),
                given_names: profile.orcid_fill(&self.given_names, &profile.given_names),
                orcid: self.orcid.or_else(|| profile.orcid_identifier().map(|value| ORCID::format(&value))),
                ..self
            },
            | None => self,
        }
    }
}
impl OrcidExt for CffReference {
    fn orcid_people(&self, path: Option<&str>) -> Vec<OrcidPersonRef> {
        let path = path.unwrap_or_default();
        self.authors
            .orcid_people(Some(&format!("{path}.authors")))
            .into_iter()
            .chain(
                self.contact
                    .as_ref()
                    .map(|people| people.orcid_people(Some(&format!("{path}.contact"))))
                    .unwrap_or_default(),
            )
            .chain(
                self.editors
                    .as_ref()
                    .map(|people| people.orcid_people(Some(&format!("{path}.editors"))))
                    .unwrap_or_default(),
            )
            .chain(
                self.editors_series
                    .as_ref()
                    .map(|people| people.orcid_people(Some(&format!("{path}.editorsSeries"))))
                    .unwrap_or_default(),
            )
            .chain(
                self.recipients
                    .as_ref()
                    .map(|people| people.orcid_people(Some(&format!("{path}.recipients"))))
                    .unwrap_or_default(),
            )
            .chain(
                self.senders
                    .as_ref()
                    .map(|people| people.orcid_people(Some(&format!("{path}.senders"))))
                    .unwrap_or_default(),
            )
            .chain(
                self.translators
                    .as_ref()
                    .map(|people| people.orcid_people(Some(&format!("{path}.translators"))))
                    .unwrap_or_default(),
            )
            .collect()
    }
    fn with_orcid_profiles(self, result: &OrcidBatchResult, path: Option<&str>) -> Self {
        let path = path.unwrap_or_default();
        Self {
            authors: self.authors.clone().with_orcid_profiles(result, Some(&format!("{path}.authors"))),
            contact: self
                .contact
                .clone()
                .map(|people| people.with_orcid_profiles(result, Some(&format!("{path}.contact")))),
            editors: self
                .editors
                .clone()
                .map(|people| people.with_orcid_profiles(result, Some(&format!("{path}.editors")))),
            editors_series: self
                .editors_series
                .clone()
                .map(|people| people.with_orcid_profiles(result, Some(&format!("{path}.editorsSeries")))),
            recipients: self
                .recipients
                .clone()
                .map(|people| people.with_orcid_profiles(result, Some(&format!("{path}.recipients")))),
            senders: self
                .senders
                .clone()
                .map(|people| people.with_orcid_profiles(result, Some(&format!("{path}.senders")))),
            translators: self
                .translators
                .clone()
                .map(|people| people.with_orcid_profiles(result, Some(&format!("{path}.translators")))),
            ..self
        }
    }
}
impl OrcidExt for ContactPoint {
    fn orcid_identity(&self) -> Option<OrcidIdentity> {
        let display_name = match (self.given_name.trim().is_empty(), self.family_name.trim().is_empty()) {
            | (false, false) => format!("{} {}", self.given_name, self.family_name),
            | _ => String::new(),
        };
        OrcidIdentity::new(
            display_name,
            [
                self.affiliation.clone(),
                (!self.organization.trim().is_empty()).then_some(self.organization.clone()),
            ]
            .into_iter()
            .flatten()
            .collect(),
            true,
            Some(self.given_name.clone()),
            Some(self.family_name.clone()),
            self.identifier.clone(),
            Some(self.email.clone()),
        )
    }
    fn with_orcid_profile(self, profile: Option<&orcid::SearchResult>) -> Self {
        match profile {
            | Some(profile) => Self {
                family_name: profile
                    .orcid_fill(&Some(self.family_name.clone()), &profile.family_names)
                    .unwrap_or(self.family_name),
                given_name: profile
                    .orcid_fill(&Some(self.given_name.clone()), &profile.given_names)
                    .unwrap_or(self.given_name),
                identifier: self.identifier.or_else(|| profile.orcid_identifier()),
                ..self
            },
            | None => self,
        }
    }
}
impl OrcidExt for datacite::Contributor {
    fn orcid_identity(&self) -> Option<OrcidIdentity> {
        let personal = !matches!(self.name_type, Some(datacite::NameType::Organizational));
        let identifier = self.name_identifiers.as_ref().and_then(|identifiers| {
            identifiers
                .iter()
                .find(|identifier| {
                    identifier.name_identifier.is_orcid()
                        || identifier
                            .name_identifier_scheme
                            .as_deref()
                            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("ORCID"))
                })
                .map(|identifier| identifier.name_identifier.clone())
        });
        personal
            .then(|| {
                OrcidIdentity::new(
                    self.name.clone(),
                    self.affiliation
                        .as_ref()
                        .into_iter()
                        .flatten()
                        .map(|affiliation| affiliation.name.clone())
                        .collect(),
                    true,
                    self.given_name.clone(),
                    self.family_name.clone(),
                    identifier,
                    None,
                )
            })
            .flatten()
    }
    fn with_orcid_profile(self, profile: Option<&orcid::SearchResult>) -> Self {
        match profile {
            | Some(profile) => {
                let has_orcid = self.name_identifiers.as_ref().is_some_and(|identifiers| {
                    identifiers.iter().any(|identifier| {
                        identifier.name_identifier.is_orcid()
                            || identifier
                                .name_identifier_scheme
                                .as_deref()
                                .is_some_and(|scheme| scheme.eq_ignore_ascii_case("ORCID"))
                    })
                });
                let identifier = profile.orcid_identifier().map(|value| datacite::NameIdentifier {
                    name_identifier: ORCID::format(&value),
                    name_identifier_scheme: Some("ORCID".to_string()),
                    scheme_uri: Some(DEFAULT_ORCID_SCHEMA_URI.to_string()),
                });
                Self {
                    family_name: profile.orcid_fill(&self.family_name, &profile.family_names),
                    given_name: profile.orcid_fill(&self.given_name, &profile.given_names),
                    name_identifiers: match (has_orcid, identifier) {
                        | (false, Some(identifier)) => Some(self.name_identifiers.unwrap_or_default().into_iter().chain([identifier]).collect()),
                        | _ => self.name_identifiers,
                    },
                    ..self
                }
            }
            | None => self,
        }
    }
}
impl OrcidExt for datacite::Creator {
    fn orcid_identity(&self) -> Option<OrcidIdentity> {
        let contributor = datacite::Contributor {
            affiliation: self.affiliation.clone(),
            contributor_type: datacite::ContributorType::Other,
            family_name: self.family_name.clone(),
            given_name: self.given_name.clone(),
            name: self.name.clone(),
            name_identifiers: self.name_identifiers.clone(),
            name_type: self.name_type.clone(),
        };
        contributor.orcid_identity()
    }
    fn with_orcid_profile(self, profile: Option<&orcid::SearchResult>) -> Self {
        let metadata = datacite::Contributor {
            affiliation: self.affiliation,
            contributor_type: datacite::ContributorType::Other,
            family_name: self.family_name,
            given_name: self.given_name,
            name: self.name,
            name_identifiers: self.name_identifiers,
            name_type: self.name_type,
        };
        let contributor = metadata.with_orcid_profile(profile);
        Self {
            affiliation: contributor.affiliation,
            family_name: contributor.family_name,
            given_name: contributor.given_name,
            name: contributor.name,
            name_identifiers: contributor.name_identifiers,
            name_type: contributor.name_type,
        }
    }
}
#[async_trait]
impl EnrichmentMetadata for datacite::Record {
    fn enrichment_source(&self, provider: Provider) -> Option<String> {
        match provider {
            | Provider::Orcid => self.orcid_source(),
            | _ => None,
        }
    }
    async fn enrich_provider(&self, provider: Provider, _source: &Location) -> ApiResult<(Self, Vec<String>, Vec<String>)> {
        match provider {
            | Provider::Orcid => self.enrich_orcid_people().await,
            | _ => Ok((self.clone(), Vec::new(), Vec::new())),
        }
    }
    fn enrichment_fields(before: &serde_json::Value, after: &serde_json::Value) -> Vec<String> {
        EnrichmentFields::changed(before, after, ["attributes.creators", "attributes.contributors"])
    }
    fn with_enrichment_provenance(self, _provider: Provider, _source: Option<String>, _fields: Vec<String>) -> Self {
        self
    }
}
#[async_trait]
impl OrcidExt for datacite::Record {
    fn orcid_people(&self, _path: Option<&str>) -> Vec<OrcidPersonRef> {
        self.attributes
            .creators
            .as_ref()
            .map(|people| people.orcid_people(Some("attributes.creators")))
            .unwrap_or_default()
            .into_iter()
            .chain(
                self.attributes
                    .contributors
                    .as_ref()
                    .map(|people| people.orcid_people(Some("attributes.contributors")))
                    .unwrap_or_default(),
            )
            .collect()
    }
    fn with_orcid_profiles(self, result: &OrcidBatchResult, _path: Option<&str>) -> Self {
        Self {
            attributes: datacite::Attributes {
                creators: self
                    .attributes
                    .creators
                    .clone()
                    .map(|people| people.with_orcid_profiles(result, Some("attributes.creators"))),
                contributors: self
                    .attributes
                    .contributors
                    .clone()
                    .map(|people| people.with_orcid_profiles(result, Some("attributes.contributors"))),
                ..self.attributes.clone()
            },
            ..self
        }
    }
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
impl OrcidExt for invenio::Contributor {
    fn orcid_identity(&self) -> Option<OrcidIdentity> {
        self.person_or_org.orcid_identity_with_affiliations(&self.affiliations)
    }
    fn with_orcid_profile(self, profile: Option<&orcid::SearchResult>) -> Self {
        Self {
            person_or_org: self.person_or_org.with_orcid_profile(profile),
            ..self
        }
    }
}
impl OrcidExt for invenio::Creator {
    fn orcid_identity(&self) -> Option<OrcidIdentity> {
        self.person_or_org.orcid_identity_with_affiliations(&self.affiliations)
    }
    fn with_orcid_profile(self, profile: Option<&orcid::SearchResult>) -> Self {
        Self {
            person_or_org: self.person_or_org.with_orcid_profile(profile),
            ..self
        }
    }
}
impl OrcidExt for invenio::PersonOrOrganization {
    fn orcid_identity(&self) -> Option<OrcidIdentity> {
        self.orcid_identity_with_affiliations(&None)
    }
    fn with_orcid_profile(self, profile: Option<&orcid::SearchResult>) -> Self {
        match profile {
            | Some(profile) => {
                let has_orcid = self.identifiers.as_ref().is_some_and(|identifiers| {
                    identifiers
                        .iter()
                        .any(|identifier| identifier.scheme == invenio::PersonOrOrgIdentifierScheme::Orcid)
                });
                let identifier = profile.orcid_identifier().map(|identifier| invenio::PersonOrOrgIdentifier {
                    scheme: invenio::PersonOrOrgIdentifierScheme::Orcid,
                    identifier: ORCID::format(&identifier),
                });
                Self {
                    family_name: profile.orcid_fill(&self.family_name, &profile.family_names),
                    given_name: profile.orcid_fill(&self.given_name, &profile.given_names),
                    identifiers: match (has_orcid, identifier) {
                        | (false, Some(identifier)) => Some(self.identifiers.unwrap_or_default().into_iter().chain([identifier]).collect()),
                        | _ => self.identifiers,
                    },
                    name: self.name.or_else(|| profile.orcid_display_name()),
                    ..self
                }
            }
            | None => self,
        }
    }
    fn orcid_identity_with_affiliations(&self, affiliations: &Option<Vec<invenio::Affiliation>>) -> Option<OrcidIdentity> {
        let identifier = self.identifiers.as_ref().and_then(|identifiers| {
            identifiers
                .iter()
                .find(|identifier| identifier.scheme == invenio::PersonOrOrgIdentifierScheme::Orcid)
                .map(|identifier| identifier.identifier.clone())
        });
        self.kind
            .eq_ignore_ascii_case("personal")
            .then(|| {
                OrcidIdentity::new(
                    self.name.clone().unwrap_or_default(),
                    affiliations
                        .as_ref()
                        .into_iter()
                        .flatten()
                        .filter_map(|affiliation| affiliation.name.clone())
                        .collect(),
                    true,
                    self.given_name.clone(),
                    self.family_name.clone(),
                    identifier,
                    None,
                )
            })
            .flatten()
    }
}
#[async_trait]
impl EnrichmentMetadata for invenio::Record {
    fn enrichment_source(&self, provider: Provider) -> Option<String> {
        match provider {
            | Provider::Orcid => self.orcid_source(),
            | _ => None,
        }
    }
    async fn enrich_provider(&self, provider: Provider, _source: &Location) -> ApiResult<(Self, Vec<String>, Vec<String>)> {
        match provider {
            | Provider::Orcid => self.enrich_orcid_people().await,
            | _ => Ok((self.clone(), Vec::new(), Vec::new())),
        }
    }
    fn enrichment_fields(before: &serde_json::Value, after: &serde_json::Value) -> Vec<String> {
        EnrichmentFields::changed(before, after, ["metadata.creators", "metadata.contributors"])
    }
    fn with_enrichment_provenance(self, _provider: Provider, _source: Option<String>, _fields: Vec<String>) -> Self {
        self
    }
}
#[async_trait]
impl OrcidExt for invenio::Record {
    fn orcid_people(&self, _path: Option<&str>) -> Vec<OrcidPersonRef> {
        self.metadata
            .as_ref()
            .map(|metadata| {
                metadata
                    .creators
                    .as_ref()
                    .map(|people| people.orcid_people(Some("metadata.creators")))
                    .unwrap_or_default()
                    .into_iter()
                    .chain(
                        metadata
                            .contributors
                            .as_ref()
                            .map(|people| people.orcid_people(Some("metadata.contributors")))
                            .unwrap_or_default(),
                    )
                    .collect()
            })
            .unwrap_or_default()
    }
    fn with_orcid_profiles(self, result: &OrcidBatchResult, _path: Option<&str>) -> Self {
        Self {
            metadata: self.metadata.clone().map(|metadata| invenio::Metadata {
                creators: metadata
                    .creators
                    .clone()
                    .map(|people| people.with_orcid_profiles(result, Some("metadata.creators"))),
                contributors: metadata
                    .contributors
                    .clone()
                    .map(|people| people.with_orcid_profiles(result, Some("metadata.contributors"))),
                ..metadata
            }),
            ..self
        }
    }
}
impl OrcidExt for orcid::SearchResult {
    fn orcid_display_name(&self) -> Option<String> {
        self.credit_name.clone().filter(|value| !value.trim().is_empty()).or_else(|| {
            let value = [self.given_names.as_deref(), self.family_names.as_deref()]
                .into_iter()
                .flatten()
                .filter(|part| !part.trim().is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            (!value.is_empty()).then_some(value)
        })
    }
    fn orcid_fill(&self, current: &Option<String>, candidate: &Option<String>) -> Option<String> {
        current
            .clone()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| candidate.clone().filter(|value| !value.trim().is_empty()))
    }
    fn orcid_identifier(&self) -> Option<String> {
        self.orcid_id
            .as_ref()
            .map(|value| ORCID::from_string(value).identifier())
            .filter(|value| !value.is_empty())
    }
}
impl OrcidBatch {
    pub(super) fn new(people: Vec<OrcidPersonRef>) -> Self {
        let requests = people.iter().fold(Vec::<OrcidRequest>::new(), |requests, person| {
            let key = person.identity.key();
            match requests.iter().any(|request| request.identity.key() == key) {
                | true => requests
                    .into_iter()
                    .map(|request| match request.identity.key() == key {
                        | true => OrcidRequest {
                            paths: request.paths.into_iter().chain([person.path.clone()]).collect(),
                            ..request
                        },
                        | false => request,
                    })
                    .collect(),
                | false => requests
                    .into_iter()
                    .chain([OrcidRequest {
                        identity: person.identity.clone(),
                        paths: vec![person.path.clone()],
                    }])
                    .collect(),
            }
        });
        Self { requests }
    }
    fn source(&self) -> Option<String> {
        match self.requests.as_slice() {
            | [request] => Some(request.identity.source()),
            | _ => None,
        }
    }
    async fn enrich(self) -> OrcidBatchResult {
        stream::iter(self.requests)
            .fold(OrcidBatchResult::default(), |result, request| async move {
                match request.identity.fetch().await {
                    | Ok(OrcidResolution::Ambiguous) => OrcidBatchResult {
                        conflicts: result
                            .conflicts
                            .into_iter()
                            .chain(
                                request
                                    .paths
                                    .into_iter()
                                    .map(|path| format!("{path}: ambiguous compatible person match; ORCID was not changed")),
                            )
                            .collect(),
                        ..result
                    },
                    | Ok(OrcidResolution::Missing) => OrcidBatchResult {
                        conflicts: result
                            .conflicts
                            .into_iter()
                            .chain(
                                request
                                    .paths
                                    .into_iter()
                                    .map(|path| format!("{path}: no compatible person match; ORCID was not changed")),
                            )
                            .collect(),
                        ..result
                    },
                    | Ok(OrcidResolution::Resolved(profile)) => OrcidBatchResult {
                        profiles: result
                            .profiles
                            .into_iter()
                            .chain(request.paths.into_iter().map(|path| (path, profile.clone())))
                            .collect(),
                        ..result
                    },
                    | Err(why) => OrcidBatchResult {
                        failures: result
                            .failures
                            .into_iter()
                            .chain(request.paths.into_iter().map(|path| format!("{path}: {why}")))
                            .collect(),
                        ..result
                    },
                }
            })
            .await
    }
}
impl OrcidBatchResult {
    fn profile(&self, path: &str) -> Option<&orcid::SearchResult> {
        self.profiles.iter().find(|(candidate, _)| candidate == path).map(|(_, profile)| profile)
    }
}
impl OrcidIdentity {
    fn new(
        display_name: impl Into<String>,
        affiliations: Vec<String>,
        fill_names: bool,
        given_name: Option<String>,
        family_name: Option<String>,
        identifier: Option<String>,
        email: Option<String>,
    ) -> Option<Self> {
        let display_name = display_name.into().trim().to_string();
        let given_name = given_name.filter(|value| !value.trim().is_empty());
        let family_name = family_name.filter(|value| !value.trim().is_empty());
        let identifier = identifier
            .map(|value| ORCID::from_string(value).identifier())
            .filter(|value| !value.is_empty());
        let is_missing_name = given_name.is_none() || family_name.is_none();
        let is_searchable_name = (given_name.is_some() && family_name.is_some()) || !display_name.is_empty();
        match (identifier.is_some(), fill_names && is_missing_name, is_searchable_name) {
            | (true, true, _) | (false, _, true) => Some(Self {
                affiliations,
                display_name,
                email: email.filter(|value| !value.trim().is_empty()),
                family_name,
                given_name,
                identifier,
            }),
            | _ => None,
        }
    }
    fn affiliation_names(&self) -> Vec<String> {
        self.affiliations
            .iter()
            .cloned()
            .chain(
                self.affiliation_root()
                    .into_iter()
                    .flat_map(|organization| [Some(organization.name), organization.alternative_name].into_iter().flatten()),
            )
            .collect()
    }
    fn affiliation_root(&self) -> Option<Organization> {
        Organization::load().into_iter().find(|root| {
            [root.clone()].into_iter().chain(root.clone().members()).any(|member| {
                self.affiliations.iter().any(|affiliation| {
                    affiliation.normalized() == member.name.normalized()
                        || member
                            .alternative_name
                            .as_ref()
                            .is_some_and(|alternative_name| affiliation.normalized() == alternative_name.normalized())
                })
            })
        })
    }
    fn affiliation_ror(&self) -> Option<String> {
        self.affiliation_root().and_then(|organization| organization.ror).map(ROR::format)
    }
    fn email_domain(&self) -> Option<String> {
        self.email
            .as_deref()
            .map(str::trim)
            .and_then(|email| email.split_once('@'))
            .filter(|(local, domain)| !local.is_empty() && !domain.is_empty() && !domain.contains('@'))
            .map(|(_, domain)| domain.to_ascii_lowercase())
    }
    fn email_matches(&self, result: &orcid::SearchResult) -> bool {
        self.email.as_ref().is_some_and(|email| {
            result
                .emails
                .as_ref()
                .is_some_and(|values| values.iter().any(|value| value.trim().eq_ignore_ascii_case(email.trim())))
        })
    }
    pub(super) fn key(&self) -> String {
        let email = self
            .email
            .as_deref()
            .map(|email| email.normalized())
            .map(|email| format!("|email:{email}"))
            .unwrap_or_default();
        match &self.identifier {
            | Some(identifier) => format!("orcid:{identifier}"),
            | None => match (&self.given_name, &self.family_name) {
                | (Some(given_name), Some(family_name)) => {
                    format!("name:{}|{}{email}", given_name.normalized(), family_name.normalized())
                }
                | _ => format!("name:{}{email}", self.display_name.normalized()),
            },
        }
    }
    fn source(&self) -> String {
        self.identifier.clone().unwrap_or_else(|| match (&self.given_name, &self.family_name) {
            | (Some(given_name), Some(family_name)) => format!("{given_name} {family_name}"),
            | _ => self.display_name.clone(),
        })
    }
    pub(super) fn email_query(&self) -> Option<Param> {
        let email = self.email.as_deref().map(str::trim).map(str::to_ascii_lowercase);
        match (&self.given_name, &self.family_name, email.as_deref()) {
            | (Some(given_name), Some(family_name), Some(email)) => Some(Param::from_query_pair(
                "q",
                vec![
                    ("given-names", given_name.as_str()),
                    ("family-name", family_name.as_str()),
                    ("email", email),
                ],
            )),
            | (_, _, Some(email)) => Some(Param::from_query_pair(
                "q",
                vec![("credit-name", self.display_name.as_str()), ("email", email)],
            )),
            | _ => None,
        }
    }
    pub(super) fn email_domain_query(&self) -> Option<Param> {
        let email = self.email_domain().map(|domain| format!("*@{domain}"));
        match (&self.given_name, &self.family_name, email.as_deref()) {
            | (Some(given_name), Some(family_name), Some(email)) => Some(Param::from_query_pair(
                "q",
                vec![
                    ("given-names", given_name.as_str()),
                    ("family-name", family_name.as_str()),
                    ("email", email),
                ],
            )),
            | (_, _, Some(email)) => Some(Param::from_query_pair(
                "q",
                vec![("credit-name", self.display_name.as_str()), ("email", email)],
            )),
            | _ => None,
        }
    }
    async fn fetch_query(&self, query: Param) -> ApiResult<orcid::SearchResponse> {
        let fields = param!(
            FieldList,
            "fl",
            vec![
                "orcid",
                "email",
                "credit-name",
                "given-names",
                "family-name",
                "other-name",
                "current-institution-affiliation-name",
                "past-institution-affiliation-name",
            ]
        );
        orcid::search(&orcid::Options::from_env().with_params(vec![query, fields])).await
    }
    async fn fetch_refinement(&self, response: orcid::SearchResponse, query: Option<Param>, description: &str) -> ApiResult<orcid::SearchResponse> {
        match query {
            | Some(query) => match self.fetch_query(query).await {
                | Ok(refined) => Ok(self.prefer_refined_response(response, refined)),
                | Err(why) => {
                    warn!("ORCID {description} refinement failed — {why}");
                    Ok(response)
                }
            },
            | None => Ok(response),
        }
    }
    async fn fetch_name_or_email(&self) -> ApiResult<orcid::SearchResponse> {
        match self.email_query() {
            | Some(query) => match self.fetch_query(query).await {
                | Ok(response) if matches!(self.select(&response), Ok(Some(_))) => Ok(response),
                | Ok(_) => self.fetch_query(self.search_query()).await,
                | Err(why) => {
                    warn!("ORCID email-and-name lookup failed — {why}");
                    self.fetch_query(self.search_query()).await
                }
            },
            | None => self.fetch_query(self.search_query()).await,
        }
    }
    pub(super) fn search_query(&self) -> Param {
        match (&self.given_name, &self.family_name) {
            | (Some(given_name), Some(family_name)) => {
                Param::from_query_pair("q", vec![("given-names", given_name.as_str()), ("family-name", family_name.as_str())])
            }
            | _ => Param::from_query_pair("q", vec![("credit-name", self.display_name.as_str())]),
        }
    }
    async fn fetch(&self) -> ApiResult<OrcidResolution> {
        match &self.identifier {
            | Some(identifier) => orcid::record(&orcid::Options::from_env().with_identifier(identifier))
                .await
                .map(OrcidResolution::Resolved),
            | None => self.fetch_search().await.map(|response| self.resolve(&response)),
        }
    }
    async fn fetch_search(&self) -> ApiResult<orcid::SearchResponse> {
        match self.fetch_name_or_email().await {
            | Ok(response) => match matches!(self.select(&response), Ok(Some(_))) {
                | false => match self.fetch_refinement(response, self.email_domain_query(), "email-domain").await {
                    | Ok(refined) => match matches!(self.select(&refined), Ok(Some(_))) {
                        | false => self.fetch_refinement(refined, self.ror_query(), "affiliation-ROR").await,
                        | true => Ok(refined),
                    },
                    | Err(why) => Err(why),
                },
                | true => Ok(response),
            },
            | Err(why) => Err(why),
        }
    }
    fn exact_matches(&self, response: &orcid::SearchResponse) -> Vec<orcid::SearchResult> {
        response
            .results
            .iter()
            .filter(|result| {
                let email_matches = self.email_matches(result);
                let assembled = [result.given_names.as_deref(), result.family_names.as_deref()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" ");
                let display_name_matches = result
                    .credit_name
                    .as_deref()
                    .into_iter()
                    .chain(result.other_name.as_deref().unwrap_or_default().iter().map(String::as_str))
                    .chain([assembled.as_str()])
                    .any(|value| value.normalized() == self.display_name.normalized());
                let name_matches = match (&self.given_name, &self.family_name) {
                    | (Some(given_name), Some(family_name)) => {
                        let given_name_matches = result.given_names.as_deref().is_some_and(|value| {
                            value.normalized() == given_name.normalized()
                                || value
                                    .split_whitespace()
                                    .any(|component| component.normalized() == given_name.normalized())
                        });
                        let family_name_matches = result
                            .family_names
                            .as_deref()
                            .is_some_and(|value| value.normalized() == family_name.normalized());
                        (given_name_matches && family_name_matches) || display_name_matches
                    }
                    | _ => display_name_matches,
                };
                email_matches || name_matches
            })
            .cloned()
            .collect()
    }
    pub(super) fn prefer_refined_response(&self, response: orcid::SearchResponse, refined: orcid::SearchResponse) -> orcid::SearchResponse {
        match self.select(&refined) {
            | Ok(Some(_)) => refined,
            | Ok(None) | Err(()) => response,
        }
    }
    pub(super) fn resolve(&self, response: &orcid::SearchResponse) -> OrcidResolution {
        match self.select(response) {
            | Ok(Some(profile)) => OrcidResolution::Resolved(profile),
            | Ok(None) => OrcidResolution::Missing,
            | Err(()) => OrcidResolution::Ambiguous,
        }
    }
    pub(super) fn ror_query(&self) -> Option<Param> {
        let ror = self.affiliation_ror().map(|value| format!("\"{value}\""));
        match (&self.given_name, &self.family_name, ror.as_deref()) {
            | (Some(given_name), Some(family_name), Some(ror)) => Some(Param::from_query_pair(
                "q",
                vec![
                    ("given-names", given_name.as_str()),
                    ("family-name", family_name.as_str()),
                    ("ror-org-id", ror),
                ],
            )),
            | (_, _, Some(ror)) => Some(Param::from_query_pair(
                "q",
                vec![("credit-name", self.display_name.as_str()), ("ror-org-id", ror)],
            )),
            | _ => None,
        }
    }
    pub(super) fn select(&self, response: &orcid::SearchResponse) -> Result<Option<orcid::SearchResult>, ()> {
        let matches = self.exact_matches(response);
        let affiliation_names = self.affiliation_names();
        let has_discriminator = self.email.is_some() || !affiliation_names.is_empty();
        let narrowed = match matches.len() > 1 && has_discriminator {
            | true => matches
                .iter()
                .filter(|result| {
                    self.email_matches(result)
                        || affiliation_names.iter().any(|affiliation| {
                            result
                                .institution_names
                                .as_ref()
                                .is_some_and(|values| values.iter().any(|value| value.normalized() == affiliation.normalized()))
                        })
                })
                .cloned()
                .collect::<Vec<_>>(),
            | false => Vec::new(),
        };
        let selected = match narrowed.is_empty() {
            | true => matches,
            | false => narrowed,
        };
        match selected.as_slice() {
            | [profile] => Ok(Some(profile.clone())),
            | [] => Ok(None),
            | _ => Err(()),
        }
    }
}
impl OrcidExt for OutputContributor {
    fn orcid_identity(&self) -> Option<OrcidIdentity> {
        OrcidIdentity::new(
            self.name.clone(),
            self.affiliations.iter().map(|affiliation| affiliation.name.clone()).collect(),
            false,
            None,
            None,
            self.orcid.clone(),
            None,
        )
    }
    fn with_orcid_profile(self, profile: Option<&orcid::SearchResult>) -> Self {
        Self {
            orcid: self.orcid.or_else(|| profile.and_then(OrcidExt::orcid_identifier)),
            ..self
        }
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
                            EnrichmentResult::builder()
                                .data(data)
                                .conflicts(
                                    result
                                        .conflicts
                                        .into_iter()
                                        .chain(conflicts.into_iter().map(|value| format!("{provider}: {value}")))
                                        .collect(),
                                )
                                .decisions(result.decisions.into_iter().chain(decisions).collect())
                                .failures(
                                    result
                                        .failures
                                        .into_iter()
                                        .chain(failures.into_iter().map(|value| format!("{provider}: {value}")))
                                        .collect(),
                                )
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
    async fn enrich_orcid(&self) -> ApiResult<(Self, Vec<String>, Vec<String>)> {
        self.enrich_orcid_people().await
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
        match provider {
            | Provider::Orcid => self.orcid_source(),
            | _ => self.source_from(provider),
        }
    }
    async fn enrich_provider(&self, provider: Provider, source: &Location) -> ApiResult<(Self, Vec<String>, Vec<String>)> {
        match provider {
            | Provider::CiteAs => self
                .enrich_citeas()
                .await
                .map(|(data, conflict, failures)| (data, conflict.into_iter().collect(), failures)),
            | Provider::OpenAlex => self
                .enrich_openalex(source)
                .await
                .map(|(data, conflict, failures)| (data, conflict.into_iter().collect(), failures)),
            | Provider::Orcid => self.enrich_orcid().await,
            | Provider::Osti => self
                .enrich_osti()
                .await
                .map(|(data, conflict)| (data, conflict.into_iter().collect(), Vec::new())),
            | Provider::Ror => self
                .enrich_ror()
                .await
                .map(|(data, conflict)| (data, conflict.into_iter().collect(), Vec::new())),
        }
    }
    fn enrichment_fields(before: &serde_json::Value, after: &serde_json::Value) -> Vec<String> {
        EnrichmentFields::changed(before, after, ENRICHMENT_FIELDS)
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
#[async_trait]
impl OrcidExt for ResearchActivity {
    fn orcid_people(&self, _path: Option<&str>) -> Vec<OrcidPersonRef> {
        self.contact
            .orcid_identity()
            .map(|identity| OrcidPersonRef {
                identity,
                path: "contact".to_string(),
            })
            .into_iter()
            .chain(
                self.meta
                    .contributors
                    .as_ref()
                    .map(|people| people.iter().cloned().collect::<Vec<_>>().orcid_people(Some("meta.contributors")))
                    .unwrap_or_default(),
            )
            .chain(
                self.meta
                    .outputs
                    .as_ref()
                    .into_iter()
                    .flat_map(|outputs| outputs.iter())
                    .enumerate()
                    .flat_map(|(index, output)| {
                        output
                            .creators
                            .as_ref()
                            .map(|people| people.orcid_people(Some(&format!("meta.outputs[{index}].creators"))))
                            .unwrap_or_default()
                            .into_iter()
                            .chain(
                                output
                                    .contributors
                                    .as_ref()
                                    .map(|people| people.orcid_people(Some(&format!("meta.outputs[{index}].contributors"))))
                                    .unwrap_or_default(),
                            )
                    }),
            )
            .collect()
    }
    fn with_orcid_profiles(self, result: &OrcidBatchResult, _path: Option<&str>) -> Self {
        let contributors = self
            .meta
            .contributors
            .clone()
            .map(|people| people.into_vec().with_orcid_profiles(result, Some("meta.contributors")).into());
        let outputs = self.meta.outputs.clone().map(|outputs| {
            outputs
                .into_vec()
                .into_iter()
                .enumerate()
                .map(|(index, output)| ResearchOutput {
                    creators: output
                        .creators
                        .clone()
                        .map(|people| people.with_orcid_profiles(result, Some(&format!("meta.outputs[{index}].creators")))),
                    contributors: output
                        .contributors
                        .clone()
                        .map(|people| people.with_orcid_profiles(result, Some(&format!("meta.outputs[{index}].contributors")))),
                    ..output
                })
                .collect::<Vec<_>>()
                .into()
        });
        Self {
            contact: self.contact.clone().with_orcid_profile(result.profile("contact")),
            meta: ResearchActivityMetadata {
                contributors,
                outputs,
                ..self.meta.clone()
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
impl<T> OrcidExt for Vec<T>
where
    T: OrcidExt,
{
    fn orcid_people(&self, path: Option<&str>) -> Vec<OrcidPersonRef> {
        match path {
            | Some(path) => self
                .iter()
                .enumerate()
                .filter_map(|(index, person)| {
                    person.orcid_identity().map(|identity| OrcidPersonRef {
                        identity,
                        path: format!("{path}[{index}]"),
                    })
                })
                .collect(),
            | None => Vec::new(),
        }
    }
    fn with_orcid_profiles(self, result: &OrcidBatchResult, path: Option<&str>) -> Self {
        match path {
            | Some(path) => self
                .into_iter()
                .enumerate()
                .map(|(index, person)| person.with_orcid_profile(result.profile(&format!("{path}[{index}]"))))
                .collect(),
            | None => self,
        }
    }
}
