use super::{
    async_trait, format, invenio, ApiResult, EnrichmentFields, EnrichmentMetadata, EnrichmentResult, Location, OrcidBatchResult, OrcidEnrichment,
    OrcidIdentity, OrcidPersonRef, PersistentIdentifierParse, Provider, String, Vec, ORCID,
};
use crate::io::api::orcid;

impl OrcidEnrichment for invenio::Contributor {
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
impl OrcidEnrichment for invenio::Creator {
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
impl OrcidEnrichment for invenio::PersonOrOrganization {
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
    async fn enrich_provider_with_decisions(&self, provider: Provider, source: &Location) -> ApiResult<EnrichmentResult<Self>> {
        match provider {
            | Provider::Orcid => self.enrich_orcid_people_with_decisions(source).await,
            | _ => Ok(EnrichmentResult::builder().data(self.clone()).build()),
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
impl OrcidEnrichment for invenio::Record {
    fn orcid_field(&self, path: &str) -> String {
        format!("{path}.person_or_org.identifiers")
    }
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
