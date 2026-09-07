//! Authorized GitLab work-item citation intake workflow.

use super::webhook;
use super::{current_user, project_member, repository_file, upsert_work_item_note, work_item, work_item_notes, HookActor, Note, Options, WorkItem};
use crate::analyzer::host::discovery::{discover_identifiers, group_artifacts, ArtifactCandidate, Candidate};
use crate::io::api::citeas::{self, ToCitations};
use crate::io::api::Configuration;
use crate::io::ApiResult;
use crate::util::constants::app::{APPLICATION, WORK_ITEM_REPORT_MARKER};
use acorn_core::util::MarkdownSupport;
use acorn_core::Repository;
use acorn_schema::pid::{Identifier, PersistentIdentifierParse, DOI, PID};
use acorn_schema::standard::cff::{self, Agent, Cff, IdentifierType, Person};
use acorn_schema::ControlledVocabulary;
use alloc::collections::BTreeSet;
use color_eyre::eyre::eyre;
use core::iter::once;
use futures::future::join_all;
use itertools::Itertools;
use serde::Serialize;

const MAINTAINER_ACCESS_LEVEL: u64 = 40;
/// Authorization result for one work-item command.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum Authorization {
    /// The command author created the work item.
    Creator,
    /// The command author is an effective project Maintainer or Owner.
    Maintainer,
    /// The command author is not allowed to start intake.
    Denied {
        /// Safe denial reason shown in the report.
        reason: String,
    },
}
/// One candidate's CFF readiness result.
#[derive(Clone, Debug, Serialize)]
pub struct ArtifactReport {
    /// Grouped artifact candidate.
    pub artifact: ArtifactCandidate,
    /// Complete CFF 1.2.0 metadata when available.
    pub cff: Option<Cff>,
    /// Required metadata fields that remain missing.
    pub missing: Vec<String>,
    /// Non-fatal enrichment failure, if any.
    pub enrichment_error: Option<String>,
}
/// Structured result of one authorized or denied work-item intake operation.
#[derive(Clone, Debug, Serialize)]
pub struct Report {
    /// Project-scoped work-item identifier.
    pub iid: u64,
    /// Stable creator identifier.
    pub creator_id: u64,
    /// Creator username observed for this operation.
    pub creator_username: String,
    /// Authorization decision.
    pub authorization: Authorization,
    /// Independently classified artifact candidates.
    pub candidates: Vec<ArtifactReport>,
}
impl ArtifactCandidate {
    async fn enrich(self, options: &Options) -> (Self, Option<String>) {
        let (candidate, repository_error) = match self.repository_cff(options).await {
            | Some(Ok(cff)) => (self.merge_cff(cff), None),
            | Some(Err(why)) => (self, Some(why)),
            | None => (self, None),
        };
        let (candidate, enrichment_error) = match candidate.identifiers.iter().find(|identifier| identifier.kind == PID::DOI) {
            | Some(identifier) if candidate.title.is_none() || candidate.authors.is_empty() => {
                match DOI::from_string(&identifier.value).to_citations().await {
                    | Ok(citations) => (candidate.apply_citeas(citations), repository_error),
                    | Err(why) => (candidate, Some(repository_error.unwrap_or_else(|| why.to_string()))),
                }
            }
            | _ => (candidate, repository_error),
        };
        (candidate.enrich_repository(Some(options.clone())).await, enrichment_error)
    }
    async fn repository_cff(&self, options: &Options) -> Option<Result<Cff, String>> {
        match self.gitlab_repository(options.domain()).and_then(|repository| repository.project_path()) {
            | Some(project_path) => {
                let file_options = options.clone().with_identifier(project_path).with_path("CITATION.cff").with_sha("HEAD");
                match repository_file(&file_options).await {
                    | Ok(file) => Some(
                        file.decoded_content()
                            .map_err(|why| why.to_string())
                            .and_then(|content| String::from_utf8(content).map_err(|why| format!("Repository CITATION.cff is not UTF-8 — {why}")))
                            .and_then(|content| {
                                serde_norway::from_str::<Cff>(&content).map_err(|why| format!("Repository CITATION.cff is invalid — {why}"))
                            }),
                    ),
                    | Err(_) => None,
                }
            }
            | None => None,
        }
    }
    fn gitlab_repository(&self, domain: &str) -> Option<Repository> {
        self.canonical_url
            .iter()
            .map(String::as_str)
            .chain(self.websites.iter().filter_map(Candidate::url))
            .chain(
                self.identifiers
                    .iter()
                    .filter(|identifier| identifier.kind == PID::URL)
                    .map(|identifier| identifier.value.as_str()),
            )
            .filter_map(|value| Repository::from_remote(value, domain))
            .find(|repository| matches!(repository, Repository::GitLab { .. }))
    }
    fn merge_cff(self, cff: Cff) -> Self {
        let metadata = Self::from(cff);
        let identifiers = self
            .identifiers
            .into_iter()
            .chain(metadata.identifiers)
            .fold(Vec::new(), |mut identifiers, identifier| {
                if !identifiers.contains(&identifier) {
                    identifiers.push(identifier);
                }
                identifiers
            });
        Self {
            identifiers,
            canonical_url: metadata.canonical_url.or(self.canonical_url),
            title: metadata.title.filter(|title| !title.trim().is_empty()).or(self.title),
            description: metadata.description.or(self.description),
            authors: if metadata.authors.is_empty() { self.authors } else { metadata.authors },
            provider_ids: self.provider_ids,
            provenance: self.provenance,
            websites: self
                .websites
                .into_iter()
                .chain(metadata.websites)
                .unique_by(|candidate| candidate.url().unwrap_or_default().to_string())
                .collect(),
            keywords: ControlledVocabulary::normalize("keywords", self.keywords.into_iter().chain(metadata.keywords)).into_values(),
            sponsors: self.sponsors,
            partners: self.partners,
            related: self.related,
            technology: self.technology,
            contact: metadata.contact.or(self.contact),
        }
    }
    pub(crate) fn apply_citeas(self, citations: citeas::Citations) -> Self {
        let metadata = citations.metadata;
        let title = (!metadata.title.trim().is_empty()).then_some(metadata.title);
        let keywords = ControlledVocabulary::normalize("keywords", self.keywords.into_iter().chain(metadata.categories)).into_values();
        let authors = metadata
            .author
            .into_iter()
            .map(|author| format!("{} {}", author.given, author.family).trim().to_string())
            .filter(|author| !author.is_empty())
            .collect::<Vec<_>>();
        Self {
            canonical_url: (!metadata.url.trim().is_empty()).then_some(metadata.url),
            title: title.or(self.title),
            authors: if authors.is_empty() { self.authors } else { authors },
            keywords,
            ..self
        }
    }
    pub(crate) fn classify(self, enrichment_error: Option<String>) -> ArtifactReport {
        let missing = [
            self.identifiers.is_empty().then_some("Identifier"),
            self.title.as_deref().is_none_or(str::is_empty).then_some("Title"),
            self.authors.is_empty().then_some("Authors"),
        ]
        .into_iter()
        .flatten()
        .map(str::to_string)
        .collect::<Vec<_>>();
        let cff = missing.is_empty().then(|| Cff::from(self.clone()));
        ArtifactReport {
            artifact: self,
            cff,
            missing,
            enrichment_error,
        }
    }
    fn label(&self) -> String {
        self.title
            .clone()
            .or_else(|| self.identifiers.first().map(|identifier| identifier.value.clone()))
            .unwrap_or_else(|| "Unidentified candidate".to_string())
    }
}
impl From<Cff> for ArtifactCandidate {
    fn from(cff: Cff) -> Self {
        let identifiers = cff
            .doi
            .as_deref()
            .map(|doi| Identifier::new(doi).normalized())
            .into_iter()
            .flatten()
            .chain(cff.url.as_deref().map(|url| Identifier::new(url).normalized()).into_iter().flatten())
            .chain(
                cff.identifiers
                    .iter()
                    .flatten()
                    .filter_map(|identifier| Identifier::new(&identifier.value).normalized()),
            )
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let canonical_url = cff
            .url
            .clone()
            .or(cff.repository_code.clone())
            .or(cff.repository.clone())
            .or(cff.repository_artifact.clone());
        let website_pairs = [
            ("Landing page", cff.url.clone()),
            ("Source repository", cff.repository_code.clone()),
            ("Repository", cff.repository.clone()),
            ("Artifact repository", cff.repository_artifact.clone()),
        ];
        let websites = website_pairs
            .into_iter()
            .filter_map(|(description, url)| {
                url.map(|url| Candidate::Website {
                    description: description.to_string(),
                    url,
                })
            })
            .unique_by(|candidate| candidate.url().unwrap_or_default().to_string())
            .collect();
        Self {
            identifiers,
            canonical_url,
            title: Some(cff.title),
            description: cff.abstract_text,
            authors: cff
                .authors
                .into_iter()
                .map(|author| match author {
                    | Agent::Entity(entity) => entity.name,
                    | Agent::Person(person) => [person.given_names, person.family_names]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(" "),
                })
                .filter(|author| !author.trim().is_empty())
                .collect(),
            provider_ids: Vec::new(),
            provenance: Vec::new(),
            websites,
            keywords: ControlledVocabulary::normalize("keywords", cff.keywords.unwrap_or_default()).into_values(),
            sponsors: Vec::new(),
            partners: Vec::new(),
            related: Vec::new(),
            technology: Vec::new(),
            contact: cff.contact.as_deref().and_then(cff_contact),
        }
    }
}
impl From<ArtifactCandidate> for Cff {
    fn from(candidate: ArtifactCandidate) -> Self {
        let doi = candidate
            .identifiers
            .iter()
            .find(|identifier| identifier.kind == PID::DOI)
            .map(|identifier| identifier.value.clone());
        let url = candidate.canonical_url.clone().or_else(|| {
            candidate
                .identifiers
                .iter()
                .find(|identifier| identifier.kind == PID::URL)
                .map(|identifier| identifier.value.clone())
        });
        let identifiers = candidate
            .identifiers
            .iter()
            .filter(|identifier| identifier.kind != PID::DOI)
            .map(|identifier| cff::Identifier {
                description: None,
                kind: cff_identifier_type(identifier),
                value: identifier.value.clone(),
            })
            .collect::<Vec<_>>();
        Cff {
            authors: candidate.authors.iter().map(|author| Agent::Person(person_from_name(author))).collect(),
            doi,
            identifiers: (!identifiers.is_empty()).then_some(identifiers),
            title: candidate.title.clone().unwrap_or_default(),
            url,
            ..Cff::default()
        }
    }
}
impl Report {
    /// Render the idempotent work-item report note.
    pub fn render(&self) -> String {
        let application = APPLICATION.to_ascii_uppercase();
        match &self.authorization {
            | Authorization::Denied { reason } => format!(
                "{WORK_ITEM_REPORT_MARKER}\n## {application} citation intake\n\n**Authorization:** Denied\n\n{reason}\n\nNo repository changes were made."
            ),
            | Authorization::Creator | Authorization::Maintainer => {
                let authorization = match self.authorization {
                    | Authorization::Creator => "work-item creator",
                    | Authorization::Maintainer => "project Maintainer",
                    | Authorization::Denied { .. } => "denied",
                };
                let included = self
                    .candidates
                    .iter()
                    .filter(|candidate| candidate.cff.is_some())
                    .map(|candidate| candidate.artifact.label())
                    .collect::<Vec<_>>();
                let excluded = self
                    .candidates
                    .iter()
                    .filter(|candidate| candidate.cff.is_none())
                    .map(|candidate| {
                        let missing = candidate.missing.join(", ");
                        let enrichment = candidate
                            .enrichment_error
                            .as_deref()
                            .map(|error| format!("; enrichment failed: {error}"))
                            .unwrap_or_default();
                        format!("{} — missing {missing}{enrichment}", candidate.artifact.label())
                    })
                    .collect::<Vec<_>>();
                format!(
                    "{WORK_ITEM_REPORT_MARKER}\n## {application} citation intake\n\n**Authorization:** Accepted ({authorization})  \n**Creator:** `{}` (user {})\n\n### Included candidates{}\n\n### Excluded candidates{}",
                    self.creator_username,
                    self.creator_id,
                    included.to_markdown(),
                    excluded.to_markdown()
                )
            }
        }
    }
}
impl WorkItem {
    async fn authorize(&self, author_id: u64, options: &Options) -> Authorization {
        if author_id == self.author.identifier {
            Authorization::Creator
        } else {
            match project_member(options, author_id).await {
                | Ok(member) if member.identifier == author_id && member.access_level >= MAINTAINER_ACCESS_LEVEL => Authorization::Maintainer,
                | Ok(_) => Authorization::Denied {
                    reason: "Only the work-item creator or a project Maintainer can run `/acorn check`.".to_string(),
                },
                | Err(_) => Authorization::Denied {
                    reason: "Authorization could not be established; access is denied safely.".to_string(),
                },
            }
        }
    }
    pub(crate) fn content(&self, notes: &[Note], command_note_id: u64, bot_user_id: u64) -> String {
        once(self.title.as_str())
            .chain(once(self.description.as_str()))
            .chain(notes.iter().filter_map(|note| eligible_note(note, command_note_id, bot_user_id)))
            .filter(|content| !content.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}
/// Analyze an authorized `/acorn check` command and upsert its work-item report.
pub async fn analyze_work_item(options: &Options, actor: &HookActor, command_note_id: u64) -> ApiResult<Report> {
    match options.internal_identifier.as_deref().and_then(|iid| iid.parse::<u64>().ok()) {
        | Some(iid) => match work_item(options).await {
            | Ok(item) => match work_item_notes(options).await {
                | Ok(notes) => {
                    let authorization = match verified_command_author(&notes, command_note_id, actor) {
                        | Some(author_id) => item.authorize(author_id, options).await,
                        | None => Authorization::Denied {
                            reason: "The command author could not be verified; access is denied safely.".to_string(),
                        },
                    };
                    match authorization {
                        | Authorization::Denied { .. } => {
                            let report = Report {
                                iid,
                                creator_id: item.author.identifier,
                                creator_username: item.author.username,
                                authorization,
                                candidates: Vec::new(),
                            };
                            upsert_work_item_note(options, WORK_ITEM_REPORT_MARKER, &report.render())
                                .await
                                .map(|_| report)
                        }
                        | Authorization::Creator | Authorization::Maintainer => match current_user(options).await {
                            | Ok(bot) => {
                                let content = item.content(&notes, command_note_id, bot.identifier);
                                let candidates = classify_content(&content, options).await;
                                let report = Report {
                                    iid,
                                    creator_id: item.author.identifier,
                                    creator_username: item.author.username,
                                    authorization,
                                    candidates,
                                };
                                upsert_work_item_note(options, WORK_ITEM_REPORT_MARKER, &report.render())
                                    .await
                                    .map(|_| report)
                            }
                            | Err(why) => Err(why),
                        },
                    }
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        },
        | None => Err(eyre!("GitLab work-item IID is required")),
    }
}
fn cff_contact(values: &[Agent]) -> Option<Candidate> {
    values
        .iter()
        .map(|agent| match agent {
            | Agent::Entity(entity) => Candidate::Contact {
                identifier: entity
                    .orcid
                    .as_deref()
                    .and_then(|value| Identifier::new(value).normalized())
                    .filter(|identifier| identifier.kind == PID::ORCID)
                    .map(|identifier| identifier.value),
                email: entity.email.clone(),
            },
            | Agent::Person(person) => Candidate::Contact {
                identifier: person
                    .orcid
                    .as_deref()
                    .and_then(|value| Identifier::new(value).normalized())
                    .filter(|identifier| identifier.kind == PID::ORCID)
                    .map(|identifier| identifier.value),
                email: person.email.clone(),
            },
        })
        .filter(|candidate| matches!(candidate, Candidate::Contact { identifier, email } if identifier.is_some() || email.is_some()))
        .unique()
        .exactly_one()
        .ok()
}
fn cff_identifier_type(identifier: &Identifier) -> IdentifierType {
    match identifier.kind {
        | PID::URL => IdentifierType::Url,
        | PID::SWHID => IdentifierType::Swh,
        | _ => IdentifierType::Other,
    }
}
pub(crate) async fn classify_content(content: &str, options: &Options) -> Vec<ArtifactReport> {
    let candidates = discover_identifiers(content)
        .into_iter()
        .map(|identifier| ArtifactCandidate {
            identifiers: vec![identifier],
            ..ArtifactCandidate::default()
        })
        .chain(Cff::embedded(content).into_iter().map(ArtifactCandidate::from))
        .collect::<Vec<_>>();
    let enriched = join_all(group_artifacts(candidates).into_iter().map(|candidate| candidate.enrich(options))).await;
    let enrichment_errors = enriched
        .iter()
        .filter_map(|(candidate, error)| error.clone().map(|error| (candidate.identifiers.clone(), error)))
        .collect::<Vec<_>>();
    group_artifacts(enriched.into_iter().map(|(candidate, _)| candidate).collect())
        .into_iter()
        .map(|candidate| {
            let enrichment_error = enrichment_errors
                .iter()
                .find(|(identifiers, _)| identifiers.iter().any(|identifier| candidate.identifiers.contains(identifier)))
                .map(|(_, error)| error.clone());
            candidate.classify(enrichment_error)
        })
        .collect()
}
fn eligible_note(note: &Note, command_note_id: u64, bot_user_id: u64) -> Option<&str> {
    match note {
        | Note::WorkItem {
            identifier,
            body,
            author,
            system,
            confidential,
            internal,
        } => {
            let invalid = author.identifier == bot_user_id || author.bot || *system || *confidential || *internal;
            (*identifier < command_note_id && !invalid && !webhook::check_requested(body)).then_some(body)
        }
        | Note::MergeRequest { .. } => None,
    }
}
fn person_from_name(name: &str) -> Person {
    let (given_names, family_names) = name.rsplit_once(' ').map_or((None, Some(name.to_string())), |(given, family)| {
        (Some(given.to_string()), Some(family.to_string()))
    });
    Person {
        address: None,
        affiliation: None,
        alias: None,
        city: None,
        country: None,
        email: None,
        family_names,
        fax: None,
        given_names,
        name_particle: None,
        name_suffix: None,
        orcid: None,
        postal_code: None,
        region: None,
        tel: None,
        website: None,
    }
}
pub(crate) fn verified_command_author(notes: &[Note], command_note_id: u64, actor: &HookActor) -> Option<u64> {
    notes.iter().find_map(|note| match note {
        | Note::WorkItem {
            identifier,
            body,
            author,
            system,
            confidential,
            internal,
        } => {
            let verified = *identifier == command_note_id && author.identifier == actor.user_id;
            let invalid = actor.is_bot || author.bot || *system || *confidential || *internal;
            (verified && !invalid && webhook::check_requested(body)).then_some(author.identifier)
        }
        | Note::MergeRequest { .. } => None,
    })
}
