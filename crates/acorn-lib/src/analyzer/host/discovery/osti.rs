//! OSTI-backed remote discovery search support.
use super::{
    candidate_repositories, candidate_urls, link_check, Candidate, RemoteEntity, RemoteMatch, RemoteProvider, RemoteResolution, RemoteSearchRequest,
    RemoteSearchResponse,
};
use crate::io::api::citeas::ToCitations;
use crate::io::api::osti::SearchResults;
use crate::io::api::{gitlab, orcid, osti, Configuration};
use crate::io::ApiResult;
use crate::RepositoryExt;
use acorn_schema::discovery::{Person, ProjectOrganization};
use acorn_schema::pid::{PersistentIdentifierParse, DOI, PID};
use color_eyre::eyre::Report as EyreReport;
use futures::future::join_all;
use itertools::Itertools;

impl From<RemoteSearchRequest> for osti::Options {
    fn from(request: RemoteSearchRequest) -> Self {
        Self {
            query: String::new(),
            view: request.entity.into(),
            organization: request.organization,
            organization_role: request.organization_role,
            limit: request.limit,
            start: request.offset,
            all: request.all,
        }
    }
}
impl RemoteMatch {
    async fn resolve(self) -> Self {
        self.resolve_person().await.resolve_project().await.resolve_repository().await
    }
    async fn resolve_person(self) -> Self {
        match (self.entity, self.pid.as_deref()) {
            | (RemoteEntity::Person, Some(identifier)) => {
                let options = orcid::Options::from_env().with_identifier(identifier);
                match orcid::record(&options).await {
                    | Ok(profile) => {
                        let resolution = Some(RemoteResolution::from_result("orcid", Ok(orcid::SearchResponse::from(profile.clone()))));
                        let enriched = self.with_orcid(profile);
                        Self { resolution, ..enriched }
                    }
                    | Err(why) => Self {
                        resolution: Some(RemoteResolution::from_error("orcid", why)),
                        ..self
                    },
                }
            }
            | _ => self,
        }
    }
    async fn resolve_project(self) -> Self {
        match self.normalized_pid() {
            | Some(identifier) if identifier.kind == PID::DOI => {
                let result = DOI::from_string(identifier.value).to_citations().await;
                Self {
                    resolution: Some(RemoteResolution::from_result("citeas", result)),
                    ..self
                }
            }
            | _ => self,
        }
    }
    async fn resolve_repository(self) -> Self {
        match self.entity {
            | RemoteEntity::Project => {
                let options = gitlab::Options::from_env();
                let repositories = candidate_repositories(self.url.as_deref(), &self.websites, options.domain());
                match acorn_core::Repository::technology(&repositories, Some(options)).await {
                    | Some(technology) => Self { technology, ..self },
                    | None => self,
                }
            }
            | _ => self,
        }
    }
    pub(super) fn with_orcid(self, profile: orcid::SearchResult) -> Self {
        match serde_json::from_value::<Person>(self.metadata.clone()) {
            | Ok(person) => {
                let profile_name = profile.credit_name.or_else(|| {
                    let name = [profile.given_names, profile.family_names]
                        .into_iter()
                        .flatten()
                        .filter(|value| !value.trim().is_empty())
                        .join(" ")
                        .trim()
                        .to_string();
                    (!name.is_empty()).then_some(name)
                });
                let name = profile_name.filter(|value| !value.is_empty()).unwrap_or(person.name);
                let orcid = profile.orcid_id.or(person.orcid);
                let email = person
                    .email
                    .or_else(|| profile.emails.into_iter().flatten().find(|value| !value.trim().is_empty()));
                let affiliations = person
                    .affiliations
                    .into_iter()
                    .chain(profile.institution_names.into_iter().flatten())
                    .filter(|value| !value.trim().is_empty())
                    .unique()
                    .collect();
                let enriched = Person {
                    name: name.clone(),
                    orcid: orcid.clone(),
                    email,
                    affiliations,
                    ..person
                };
                Self {
                    identifier: orcid.clone().unwrap_or(self.identifier),
                    title: name,
                    pid: orcid,
                    metadata: serde_json::to_value(enriched).unwrap_or(self.metadata),
                    ..self
                }
            }
            | Err(_) => self,
        }
    }
}
impl RemoteSearchResponse {
    pub(super) fn from_osti(response: osti::SearchResponse) -> ApiResult<Self> {
        let matches = match response.results {
            | SearchResults::Projects(values) => values
                .into_iter()
                .map(|value| {
                    let websites = project_websites(&value);
                    let sponsors = organization_names(&value.sponsoring_organizations);
                    let partners = organization_names(
                        &value
                            .research_organizations
                            .iter()
                            .chain(&value.contributing_organizations)
                            .chain(&value.developing_organizations)
                            .cloned()
                            .collect::<Vec<_>>(),
                    );
                    serde_json::to_value(&value).map_err(EyreReport::from).map(|metadata| {
                        RemoteMatch::init()
                            .entity(RemoteEntity::Project)
                            .identifier(value.code_id.to_string())
                            .title(value.software_title)
                            .maybe_pid(value.doi)
                            .maybe_url(
                                value
                                    .repository_link
                                    .clone()
                                    .or(value.landing_page.clone())
                                    .or_else(|| value.links.iter().find(|link| link.rel == "citation").map(|link| link.href.clone())),
                            )
                            .metadata(metadata)
                            .maybe_description(value.description)
                            .websites(websites)
                            .sponsors(sponsors)
                            .partners(partners)
                            .build()
                    })
                })
                .collect::<ApiResult<Vec<_>>>(),
            | SearchResults::People(values) => values
                .into_iter()
                .map(|value| {
                    serde_json::to_value(&value).map_err(EyreReport::from).map(|metadata| {
                        RemoteMatch::init()
                            .entity(RemoteEntity::Person)
                            .identifier(value.orcid.clone().or(value.email.clone()).unwrap_or_else(|| value.name.clone()))
                            .title(value.name)
                            .maybe_pid(value.orcid)
                            .metadata(metadata)
                            .build()
                    })
                })
                .collect::<ApiResult<Vec<_>>>(),
            | SearchResults::Organizations(values) => values
                .into_iter()
                .map(|value| {
                    serde_json::to_value(&value).map_err(EyreReport::from).map(|metadata| {
                        RemoteMatch::init()
                            .entity(RemoteEntity::Organization)
                            .identifier(value.aliases.first().cloned().unwrap_or_else(|| value.name.clone()))
                            .title(value.name)
                            .metadata(metadata)
                            .build()
                    })
                })
                .collect::<ApiResult<Vec<_>>>(),
        };
        matches.map(|matches| Self {
            provider: RemoteProvider::Osti,
            total: response.project_total,
            offset: response.offset,
            has_more: response.has_more,
            matches,
            resolution_checks: Default::default(),
        })
    }
    pub(super) async fn resolve(self) -> Self {
        let matches = join_all(self.matches.into_iter().map(RemoteMatch::resolve)).await;
        let links = matches
            .iter()
            .flat_map(|value| candidate_urls(value.url.as_deref(), &value.websites))
            .collect::<Vec<_>>();
        let resolution_checks = join_all(links.into_iter().map(|url| link_check(Some(url), Some("remote.url".to_string()))))
            .await
            .into();
        Self {
            matches,
            resolution_checks,
            ..self
        }
    }
}
fn organization_names(values: &[ProjectOrganization]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.organization_name.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unique()
        .collect()
}
fn project_websites(value: &osti::Project) -> Vec<Candidate> {
    value
        .repository_link
        .as_ref()
        .map(|url| Candidate::Website {
            description: "Source repository".to_string(),
            url: url.clone(),
        })
        .into_iter()
        .chain(value.landing_page.as_ref().map(|url| Candidate::Website {
            description: "Project landing page".to_string(),
            url: url.clone(),
        }))
        .chain(
            value
                .links
                .iter()
                .filter(|link| !link.href.trim().is_empty())
                .map(|link| Candidate::Website {
                    description: match link.rel.trim() {
                        | "" => "Related project link".to_string(),
                        | relation => relation.to_string(),
                    },
                    url: link.href.clone(),
                }),
        )
        .unique_by(|candidate| candidate.url().unwrap_or_default().to_string())
        .collect()
}
