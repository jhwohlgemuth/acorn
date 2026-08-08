//! Authorized GitLab work-item citation intake workflow.

use super::webhook;
use super::{current_user, project_member, repository_file, upsert_work_item_note, work_item, work_item_notes, HookActor, Note, Options, WorkItem};
use crate::analyzer::discovery::{discover_identifiers, group_artifacts, ArtifactCandidate};
use crate::io::api::citeas;
use crate::io::api::Configuration;
use crate::io::ApiResult;
use crate::param;
use crate::schema::pid::{Identifier, PID};
use crate::schema::standard::cff::{self, Agent, Cff, IdentifierType, Person};
use crate::util::constants::app::{APPLICATION, WORK_ITEM_REPORT_MARKER};
use crate::util::ToMarkdown;
use crate::Location;
use color_eyre::eyre::eyre;
use core::iter::once;
use futures::future::join_all;
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
pub struct Candidate {
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
    pub candidates: Vec<Candidate>,
}
impl ArtifactCandidate {
    async fn enrich(self, options: &Options) -> (Self, Option<String>) {
        let (candidate, repository_error) = match self.repository_cff(options).await {
            | Some(Ok(cff)) => (self.merge_cff(cff), None),
            | Some(Err(why)) => (self, Some(why)),
            | None => (self, None),
        };
        match candidate.identifiers.iter().find(|identifier| identifier.kind == PID::DOI) {
            | Some(identifier) if candidate.title.is_none() || candidate.authors.is_empty() => {
                let options = citeas::Options::from_env().with_params(vec![param!(TemplateValue, "doi", &identifier.value)]);
                match citeas::search(&options).await {
                    | Ok(citations) => (candidate.apply_citeas(citations), repository_error),
                    | Err(why) => (candidate, Some(repository_error.unwrap_or_else(|| why.to_string()))),
                }
            }
            | _ => (candidate, repository_error),
        }
    }
    async fn repository_cff(&self, options: &Options) -> Option<Result<Cff, String>> {
        match self.gitlab_project_path(options.domain()) {
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
    fn gitlab_project_path(&self, domain: &str) -> Option<String> {
        let configured = Location::Simple(if domain.contains("://") {
            domain.to_string()
        } else {
            format!("https://{domain}")
        });
        self.identifiers
            .iter()
            .filter(|identifier| identifier.kind == PID::URL)
            .map(|identifier| Location::Simple(identifier.value.clone()))
            .find(|candidate| configured.host() == candidate.host() && configured.port() == candidate.port())
            .and_then(|candidate| candidate.path())
            .and_then(|path| {
                let project_path = path.trim_matches('/').split("/-/").next().unwrap_or_default().trim_end_matches(".git");
                (project_path.split('/').count() >= 2).then(|| project_path.to_string())
            })
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
            authors: if metadata.authors.is_empty() { self.authors } else { metadata.authors },
        }
    }
    pub(crate) fn apply_citeas(self, citations: citeas::Citations) -> Self {
        let metadata = citations.metadata;
        let title = (!metadata.title.trim().is_empty()).then_some(metadata.title);
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
            ..self
        }
    }
    pub(crate) fn classify(self, enrichment_error: Option<String>) -> Candidate {
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
        Candidate {
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
            .map(|doi| Identifier::new(doi).normalize())
            .into_iter()
            .flatten()
            .chain(cff.url.as_deref().map(|url| Identifier::new(url).normalize()).into_iter().flatten())
            .collect::<Vec<_>>();
        Self {
            identifiers,
            canonical_url: cff.url,
            title: Some(cff.title),
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
                kind: if identifier.kind == PID::URL {
                    IdentifierType::Url
                } else {
                    IdentifierType::Other
                },
                value: identifier.value.clone(),
            })
            .collect::<Vec<_>>();
        Cff {
            authors: candidate
                .authors
                .iter()
                .map(|author| Agent::Person(Person::from(author.as_str())))
                .collect(),
            doi,
            identifiers: (!identifiers.is_empty()).then_some(identifiers),
            title: candidate.title.clone().unwrap_or_default(),
            url,
            ..Cff::default()
        }
    }
}
impl From<&str> for Person {
    fn from(name: &str) -> Self {
        let (given_names, family_names) = name.rsplit_once(' ').map_or((None, Some(name.to_string())), |(given, family)| {
            (Some(given.to_string()), Some(family.to_string()))
        });
        Self {
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
pub(crate) async fn classify_content(content: &str, options: &Options) -> Vec<Candidate> {
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
