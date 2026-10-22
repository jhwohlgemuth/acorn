use super::{
    async_trait, format, invenio, ApiResult, EnrichmentCandidate, EnrichmentCandidateKey, EnrichmentDecision, EnrichmentOrigin, EnrichmentResult,
    Location, PersistentIdentifier, PersistentIdentifierParse, Provider, String, ToString, Vec, ORCID,
};
use crate::io::api::orcid;
use futures::stream::{self, StreamExt};
use itertools::Itertools;

mod identity;
#[async_trait]
pub(in crate::io) trait OrcidEnrichment: Clone + Send + Sync {
    async fn enrich_orcid_people(&self) -> ApiResult<(Self, Vec<String>, Vec<String>)> {
        let result = OrcidBatch::new(self.orcid_people(None)).enrich().await;
        Ok((
            self.clone().with_orcid_profiles(&result, None),
            result.conflicts.clone(),
            result.failures.clone(),
        ))
    }
    async fn enrich_orcid_people_with_decisions(&self, source: &Location) -> ApiResult<EnrichmentResult<Self>> {
        let result = OrcidBatch::new(self.orcid_people(None)).enrich().await;
        result
            .ambiguous
            .iter()
            .try_fold(Vec::new(), |decisions, (path, profiles)| {
                orcid_decision(source, self.orcid_field(path), profiles).map(|decision| decisions.into_iter().chain(decision).collect())
            })
            .map(|decisions| {
                EnrichmentResult::builder()
                    .data(self.clone().with_orcid_profiles(&result, None))
                    .conflicts(result.conflicts)
                    .decisions(decisions)
                    .failures(result.failures)
                    .build()
            })
    }
    fn orcid_field(&self, path: &str) -> String {
        format!("{path}.orcid")
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
pub(in crate::io) enum OrcidResolution {
    Ambiguous(Vec<orcid::SearchResult>),
    Missing,
    Resolved(orcid::SearchResult),
}
#[derive(Clone, Debug, Default)]
pub(in crate::io) struct OrcidBatch {
    pub(in crate::io) requests: Vec<OrcidRequest>,
}
#[derive(Clone, Debug, Default)]
pub(in crate::io) struct OrcidBatchResult {
    pub(in crate::io) ambiguous: Vec<(String, Vec<orcid::SearchResult>)>,
    pub(in crate::io) conflicts: Vec<String>,
    pub(in crate::io) failures: Vec<String>,
    pub(in crate::io) profiles: Vec<(String, orcid::SearchResult)>,
}
#[derive(Clone, Debug)]
pub(in crate::io) struct OrcidIdentity {
    pub(super) affiliations: Vec<String>,
    pub(super) display_name: String,
    pub(super) email: Option<String>,
    pub(super) family_name: Option<String>,
    pub(super) given_name: Option<String>,
    pub(super) identifier: Option<String>,
}
#[derive(Clone, Debug)]
pub(in crate::io) struct OrcidPersonRef {
    pub(super) identity: OrcidIdentity,
    pub(super) path: String,
}
#[derive(Clone, Debug)]
pub(in crate::io) struct OrcidRequest {
    pub(super) identity: OrcidIdentity,
    pub(in crate::io) paths: Vec<String>,
}
impl OrcidBatch {
    pub(in crate::io) fn new(people: Vec<OrcidPersonRef>) -> Self {
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
    pub(super) fn source(&self) -> Option<String> {
        match self.requests.as_slice() {
            | [request] => Some(request.identity.source()),
            | _ => None,
        }
    }
    pub(super) async fn enrich(self) -> OrcidBatchResult {
        stream::iter(self.requests)
            .fold(OrcidBatchResult::default(), |result, request| async move {
                match request.identity.fetch().await {
                    | Ok(OrcidResolution::Ambiguous(profiles)) => OrcidBatchResult {
                        ambiguous: result
                            .ambiguous
                            .into_iter()
                            .chain(request.paths.iter().cloned().map(|path| (path, profiles.clone())))
                            .collect(),
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
    pub(in crate::io) fn profile(&self, path: &str) -> Option<&orcid::SearchResult> {
        self.profiles.iter().find(|(candidate, _)| candidate == path).map(|(_, profile)| profile)
    }
}
impl OrcidEnrichment for orcid::SearchResult {
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
impl<T> OrcidEnrichment for Vec<T>
where
    T: OrcidEnrichment,
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
pub(in crate::io) fn orcid_decision(source: &Location, field: String, profiles: &[orcid::SearchResult]) -> ApiResult<Option<EnrichmentDecision>> {
    let candidates = profiles
        .iter()
        .filter_map(|profile| {
            profile.orcid_identifier().map(|identifier| {
                let evidence_source = ORCID::format(&identifier);
                let value = match field.starts_with("meta.outputs[") {
                    | true => identifier,
                    | false => evidence_source.clone(),
                };
                let key = EnrichmentCandidateKey {
                    field: field.clone(),
                    origin: EnrichmentOrigin::Provider(Provider::Orcid),
                    source: source.clone(),
                    value: value.clone(),
                };
                let label = profile
                    .credit_name
                    .clone()
                    .or_else(|| {
                        let name = [profile.given_names.as_deref(), profile.family_names.as_deref()]
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>()
                            .join(" ");
                        (!name.is_empty()).then_some(name)
                    })
                    .unwrap_or_else(|| value.clone());
                EnrichmentCandidate { key, label, evidence_source }
            })
        })
        .sorted_by(|left, right| left.key.value.cmp(&right.key.value).then(left.label.cmp(&right.label)))
        .unique_by(|candidate| candidate.key.value.clone())
        .take(crate::util::constants::app::MAX_ENRICHMENT_CANDIDATES)
        .collect::<Vec<_>>();
    match candidates.is_empty() {
        | true => Ok(None),
        | false => EnrichmentDecision::new(
            EnrichmentOrigin::Provider(Provider::Orcid),
            source.clone(),
            field,
            String::new(),
            "ambiguous compatible person match".to_string(),
            candidates,
        )
        .map(Some),
    }
}
