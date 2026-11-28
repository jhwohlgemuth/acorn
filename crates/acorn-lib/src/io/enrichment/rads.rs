use super::{
    async_trait, format, merge_unique_by, openalex, osti, param, ror, stream, ActivityContributor, AgentIdentifier, ApiResult, Configuration,
    ContactPoint, Enrich, EnrichmentCandidate, EnrichmentCandidateKey, EnrichmentDecision, EnrichmentFields, EnrichmentMetadata, EnrichmentOrigin,
    EnrichmentProvenance, EnrichmentResult, Location, OneOrMany, OrcidBatchResult, OrcidEnrichment, OrcidIdentity, OrcidPersonRef, OutputContributor,
    PersistentIdentifierConvert, PersistentIdentifierParse, Provider, PublicationIdentifierType, ResearchActivity, ResearchActivityMetadata,
    ResearchOutput, ResearchOutputMetadata, StreamExt, String, StringConversion, ToCitations, ToString, Vec, Website, DEFAULT_ORCID_SCHEMA_URI, DOI,
    ORCID,
};
use crate::io::api::orcid;
use crate::util::constants::app::ENRICHMENT_FIELDS;
use itertools::Itertools;

#[async_trait]
pub(in crate::io) trait ResearchActivityEnrichment {
    async fn resolve_ror(&self) -> ApiResult<(ResearchActivity, Option<String>, Vec<ror::SingleRecord>)>;
    fn select_ror(&self, organization: &str, response: ror::SearchResponse) -> (ResearchActivity, Option<String>, Vec<ror::SingleRecord>);
    fn ror_decision(source: &Location, records: &[ror::SingleRecord]) -> ApiResult<Option<EnrichmentDecision>>;
}
impl OrcidEnrichment for ActivityContributor {
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
        let identifier = profile.and_then(OrcidEnrichment::orcid_identifier).map(|value| {
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
impl OrcidEnrichment for ContactPoint {
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
impl OrcidEnrichment for OutputContributor {
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
            orcid: self.orcid.or_else(|| profile.and_then(OrcidEnrichment::orcid_identifier)),
            ..self
        }
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
        self.resolve_ror().await.map(|(data, conflict, _)| (data, conflict))
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
    async fn enrich_provider_with_decisions(&self, provider: Provider, source: &Location) -> ApiResult<EnrichmentResult<Self>> {
        match provider {
            | Provider::Orcid => self.enrich_orcid_people_with_decisions(source).await,
            | Provider::Ror => self.resolve_ror().await.and_then(|(data, conflict, records)| {
                Self::ror_decision(source, &records).map(|decision| {
                    EnrichmentResult::builder()
                        .data(data)
                        .conflicts(conflict.into_iter().collect())
                        .decisions(decision.into_iter().collect())
                        .build()
                })
            }),
            | _ => self
                .enrich_provider(provider, source)
                .await
                .map(|(data, conflicts, failures)| EnrichmentResult::builder().data(data).conflicts(conflicts).failures(failures).build()),
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
impl OrcidEnrichment for ResearchActivity {
    fn orcid_field(&self, path: &str) -> String {
        match path {
            | "contact" => "contact.identifier".to_string(),
            | path if path.starts_with("meta.outputs[") => format!("{path}.orcid"),
            | _ => format!("{path}.identifiers"),
        }
    }
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
#[async_trait]
impl ResearchActivityEnrichment for ResearchActivity {
    async fn resolve_ror(&self) -> ApiResult<(ResearchActivity, Option<String>, Vec<ror::SingleRecord>)> {
        let organization = self
            .contact
            .affiliation
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(self.contact.organization.as_str())
            .trim()
            .to_string();
        match self.meta.ror.as_ref().is_some_and(|values| !values.is_empty()) || organization.is_empty() {
            | true => Ok((self.clone(), None, Vec::new())),
            | false => {
                let options = ror::Options::from_env().with_params(vec![param!(FieldList, "query", organization.as_str())]);
                ror::search(&options).await.map(|response| self.select_ror(&organization, response))
            }
        }
    }
    fn select_ror(&self, organization: &str, response: ror::SearchResponse) -> (ResearchActivity, Option<String>, Vec<ror::SingleRecord>) {
        let exact = response
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
        let narrowed = match exact.len() > 1 {
            | true => self.contact.email.rsplit_once('@').map_or_else(Vec::new, |(_, domain)| {
                exact
                    .iter()
                    .filter(|record| record.domains.iter().any(|value| value.eq_ignore_ascii_case(domain)))
                    .cloned()
                    .collect()
            }),
            | false => exact.clone(),
        };
        let records = match narrowed.is_empty() {
            | true if exact.len() > 1 => exact,
            | _ => narrowed.clone(),
        };
        match narrowed.as_slice() {
            | [record] => (
                ResearchActivity {
                    meta: ResearchActivityMetadata {
                        ror: Some(vec![record.identifier.clone()].into()),
                        ..self.meta.clone()
                    },
                    ..self.clone()
                },
                None,
                Vec::new(),
            ),
            | [] => (self.clone(), None, records),
            | _ => (
                self.clone(),
                Some("ambiguous exact organization match; ROR was not changed".to_string()),
                records,
            ),
        }
    }
    fn ror_decision(source: &Location, records: &[ror::SingleRecord]) -> ApiResult<Option<EnrichmentDecision>> {
        let field = "meta.ror".to_string();
        let candidates = records
            .iter()
            .map(|record| EnrichmentCandidate {
                key: EnrichmentCandidateKey {
                    field: field.clone(),
                    origin: EnrichmentOrigin::Provider(Provider::Ror),
                    source: source.clone(),
                    value: record.identifier.clone(),
                },
                label: record
                    .names
                    .iter()
                    .find(|name| name.types.iter().any(|kind| matches!(kind, ror::OrganizationNameType::RorDisplay)))
                    .map(|name| name.value.clone())
                    .unwrap_or_else(|| record.identifier.clone()),
                evidence_source: record.identifier.clone(),
            })
            .sorted_by(|left, right| left.key.value.cmp(&right.key.value).then(left.label.cmp(&right.label)))
            .unique_by(|candidate| candidate.key.value.clone())
            .take(crate::util::constants::app::MAX_ENRICHMENT_CANDIDATES)
            .collect::<Vec<_>>();
        match candidates.is_empty() {
            | true => Ok(None),
            | false => EnrichmentDecision::new(
                EnrichmentOrigin::Provider(Provider::Ror),
                source.clone(),
                field,
                String::new(),
                "ambiguous exact organization match".to_string(),
                candidates,
            )
            .map(Some),
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
