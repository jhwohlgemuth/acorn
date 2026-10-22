use super::{
    async_trait, format, ApiResult, Cff, CffAgent, CffPerson, CffReference, EnrichmentFields, EnrichmentMetadata, EnrichmentResult, Location,
    OrcidBatchResult, OrcidEnrichment, OrcidIdentity, OrcidPersonRef, PersistentIdentifierParse, Provider, String, Vec, ORCID,
};
use crate::io::api::orcid;

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
    async fn enrich_provider_with_decisions(&self, provider: Provider, source: &Location) -> ApiResult<EnrichmentResult<Self>> {
        match provider {
            | Provider::Orcid => self.enrich_orcid_people_with_decisions(source).await,
            | _ => Ok(EnrichmentResult::builder().data(self.clone()).build()),
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
impl OrcidEnrichment for Cff {
    fn orcid_field(&self, path: &str) -> String {
        format!("{}.orcid", path.replace("preferredCitation", "preferred-citation"))
    }
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
impl OrcidEnrichment for CffAgent {
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
impl OrcidEnrichment for CffPerson {
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
impl OrcidEnrichment for CffReference {
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
