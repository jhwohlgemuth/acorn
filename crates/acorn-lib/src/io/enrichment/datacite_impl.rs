use super::{
    async_trait, datacite, format, ApiResult, EnrichmentFields, EnrichmentMetadata, EnrichmentResult, Location, OrcidBatchResult, OrcidEnrichment,
    OrcidIdentity, OrcidPersonRef, PersistentIdentifierConvert, PersistentIdentifierParse, Provider, String, ToString, Vec, DEFAULT_ORCID_SCHEMA_URI,
    ORCID,
};
use crate::io::api::orcid;

impl OrcidEnrichment for datacite::Contributor {
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
impl OrcidEnrichment for datacite::Creator {
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
    async fn enrich_provider_with_decisions(&self, provider: Provider, source: &Location) -> ApiResult<EnrichmentResult<Self>> {
        match provider {
            | Provider::Orcid => self.enrich_orcid_people_with_decisions(source).await,
            | _ => Ok(EnrichmentResult::builder().data(self.clone()).build()),
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
impl OrcidEnrichment for datacite::Record {
    fn orcid_field(&self, path: &str) -> String {
        format!("{path}.nameIdentifiers")
    }
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
