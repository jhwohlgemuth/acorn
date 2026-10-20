//! Explicit staging and application of standalone local JSONL submissions.
use super::Submission;
use crate::io::api::gitlab::jsonl::split_snapshot;
use crate::io::config::application::{Destination, FolderRegistry};
use crate::io::config::ApplicationConfiguration;
use crate::io::workflow::{LogbookProposal, ProcessRepositoryResponse};
use crate::io::{durable_replace_file, read_file_snapshot, sync_parent_directory, ApiResult};
use acorn_core::prelude::{String, Vec};
use acorn_schema::hardware::memory::Memory;
use acorn_schema::research_activity::LogbookEntry;
use acorn_schema::validation::Validate;
use color_eyre::eyre::eyre;
use core::fmt;
use data_encoding::HEXLOWER;
use manifest::{manifest_path, BatchManifest, BatchProject, BatchProjectOutcome};
pub use manifest::{BatchApplySummary, BatchInspection, BatchSummary};
use preflight::Preflight;
use ring::digest::{digest, SHA256};
use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
pub use types::{BatchApplyStatus, BatchInspectionStatus, BatchLineStatus, BatchProjectApplyStatus, BatchProjectState};

mod manifest;
mod preflight;
mod stage;
mod types;

const MAX_INBOX_BYTES: usize = 10 * 1024 * 1024;
const MAX_INDEX_BYTES: usize = 10 * 1024 * 1024;
const MAX_PROPOSAL_BYTES: usize = 64 * 1024 * 1024;
trait LocalIndexProposal {
    fn proposed_content(self, index: &Path) -> ApiResult<(String, String)>;
}
/// Result of explicitly applying a local proposal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplyStatus {
    /// The canonical index was changed by this invocation.
    Applied,
    /// This proposal had already been applied or its entry was already present.
    AlreadyApplied,
}
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
struct Proposal {
    #[validate(uuid)]
    submission_id: String,
    source_path: PathBuf,
    #[validate(range(min = 1))]
    source_line: u64,
    #[validate(nonempty)]
    raw: String,
    #[validate(digest)]
    source_digest: String,
    #[validate(nonempty)]
    folder_id: String,
    destination: PathBuf,
    #[validate(nested)]
    entry: LogbookEntry,
    #[validate(digest)]
    base_digest: String,
    #[validate(nonempty)]
    proposed_content: String,
    applied: bool,
}
/// A line rejected while other local inbox lines were staged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageRejection {
    /// One-based physical inbox line number.
    pub line: u64,
    /// Reason the line could not be staged.
    pub reason: String,
}
/// Result of inspecting and staging one local JSONL inbox.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StageSummary {
    /// Newly created or refreshed proposal identifiers.
    pub staged: Vec<String>,
    /// Already staged or applied identical submissions.
    pub existing: Vec<String>,
    /// Rejected lines with inspectable reasons.
    pub rejected: Vec<StageRejection>,
}
impl ApplicationConfiguration {
    /// Load a local intake configuration and its canonical file path.
    pub fn load_local_intake(path: &Path) -> ApiResult<(Self, PathBuf)> {
        Self::load_with_path(&Some(path.to_path_buf())).and_then(|(config, resolved)| {
            resolved
                .ok_or_else(|| eyre!("An explicit configuration file is required"))
                .and_then(|resolved| {
                    fs::canonicalize(&resolved)
                        .map_err(|why| eyre!("Cannot resolve configuration '{}': {why}", resolved.display()))
                        .map(|resolved| (config, resolved))
                })
        })
    }
    /// Apply one staged proposal if the local index still has its staged base digest.
    pub fn apply(&self, config_path: &Path, submission_id: &str) -> ApiResult<ApplyStatus> {
        Proposal::directory(config_path)
            .and_then(|directory| Proposal::path(&directory, submission_id))
            .and_then(|path| Proposal::read(&path).and_then(|proposal| proposal.apply(&path, self, submission_id)))
    }
    /// Apply one reviewed batch, returning per-project outcomes even after a later write fails.
    pub fn apply_batch(&self, config_path: &Path, batch_id: &str) -> ApiResult<BatchApplySummary> {
        manifest_path(config_path, batch_id).and_then(|path| {
            BatchManifest::read(&path).and_then(|mut manifest| {
                manifest
                    .preflight((self, config_path, batch_id))
                    .and_then(|()| BatchProject::lock_destinations(&manifest.projects))
                    .and_then(|locks| manifest.preflight((self, config_path, batch_id)).map(|()| locks))
                    .map(|locks| {
                        let mut outcomes = Vec::new();
                        let mut failure = None;
                        for project in manifest.projects.clone() {
                            let current = read_file_snapshot(&project.destination, MAX_INDEX_BYTES).map(|(_, text, _)| text);
                            let result = current.and_then(|text| {
                                let digest = digest_text(&text);
                                match (digest == project.final_digest, digest == project.base_digest) {
                                    | (true, _) => Ok(BatchProjectApplyStatus::AlreadyApplied),
                                    | (false, true) => durable_replace_file(&project.destination, project.proposed_content.as_bytes())
                                        .map(|()| BatchProjectApplyStatus::Applied),
                                    | _ => Err(eyre!("Local index changed since staging")),
                                }
                            });
                            match result {
                                | Ok(status) => {
                                    if let Some(project) = manifest.projects.iter_mut().find(|item| item.destination == project.destination) {
                                        project.applied = true;
                                    }
                                    outcomes.push(BatchProjectOutcome {
                                        project_id: project.project_id.clone(),
                                        destination: project.destination.clone(),
                                        status,
                                    });
                                    let receipt = path
                                        .parent()
                                        .ok_or_else(|| eyre!("Batch manifest has no parent"))
                                        .and_then(|directory| manifest.write(directory));
                                    if let Err(why) = receipt {
                                        failure = Some(format!("Cannot persist batch receipt: {why}"));
                                        break;
                                    }
                                }
                                | Err(why) => {
                                    failure = Some(why.to_string());
                                    break;
                                }
                            }
                        }
                        drop(locks);
                        let complete = failure.is_none();
                        let reported = manifest
                            .projects
                            .iter()
                            .map(|project| {
                                outcomes
                                    .iter()
                                    .find(|outcome| outcome.destination == project.destination)
                                    .cloned()
                                    .unwrap_or_else(|| BatchProjectOutcome {
                                        project_id: project.project_id.clone(),
                                        destination: project.destination.clone(),
                                        status: BatchProjectApplyStatus::Pending,
                                    })
                            })
                            .collect();
                        BatchApplySummary {
                            batch_id: batch_id.to_string(),
                            status: if complete {
                                BatchApplyStatus::Complete
                            } else {
                                BatchApplyStatus::Partial
                            },
                            projects: reported,
                            error: failure,
                        }
                    })
            })
        })
    }
    /// Stage valid local records without changing the inbox or any canonical index.
    pub fn stage(&self, config_path: &Path, inbox_path: &Path) -> ApiResult<StageSummary> {
        read_file_snapshot(inbox_path, MAX_INBOX_BYTES).and_then(|(source, content, _)| {
            split_snapshot(&content, &Memory::kb(64)).and_then(|lines| {
                Proposal::directory(config_path).and_then(|directory| {
                    create_dir_all(&directory)
                        .map_err(|why| eyre!("Cannot create local proposal directory '{}': {why}", directory.display()))
                        .map(|()| {
                            let registry = self.registry();
                            lines.into_iter().fold(StageSummary::default(), |mut summary, line| {
                                match stage_line(&directory, &source, line.line, &line.raw, &line.digest, &registry) {
                                    | Ok((id, true)) => summary.staged.push(id),
                                    | Ok((id, false)) => summary.existing.push(id),
                                    | Err(why) => summary.rejected.push(StageRejection {
                                        line: line.line,
                                        reason: why.to_string(),
                                    }),
                                }
                                summary
                            })
                        })
                })
            })
        })
    }
}
impl LocalIndexProposal for LogbookEntry {
    fn proposed_content(self, index: &Path) -> ApiResult<(String, String)> {
        read_file_snapshot(index, MAX_INDEX_BYTES).and_then(|(_, base, _)| {
            index
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| eyre!("Local index has no UTF-8 filename"))
                .and_then(|path| {
                    let base_digest = digest_text(&base);
                    ProcessRepositoryResponse::try_from(
                        LogbookProposal::builder()
                            .base_revision(&base_digest)
                            .content(&base)
                            .entry(self)
                            .path(path)
                            .build(),
                    )
                    .map(|response| {
                        let content = response.files.first().map_or(base, |change| change.content.clone());
                        (base_digest, content)
                    })
                })
        })
    }
}
impl Proposal {
    fn directory(config_path: &Path) -> ApiResult<PathBuf> {
        config_path
            .parent()
            .ok_or_else(|| eyre!("Configuration path has no parent"))
            .map(|root| root.join(".acorn/intake/proposals"))
    }
    fn path(directory: &Path, id: &str) -> ApiResult<PathBuf> {
        let bytes = id.as_bytes();
        let valid = bytes.len() == 36
            && bytes.iter().enumerate().all(|(index, byte)| match [8, 13, 18, 23].contains(&index) {
                | true => *byte == b'-',
                | false => byte.is_ascii_hexdigit(),
            });
        match valid {
            | true => Ok(directory.join(format!("{}.json", id.to_ascii_lowercase()))),
            | false => Err(eyre!("Submission ID must be a UUID")),
        }
    }
    fn read(path: &Path) -> ApiResult<Self> {
        read_file_snapshot(path, MAX_PROPOSAL_BYTES)
            .and_then(|(_, content, _)| serde_json::from_str(&content).map_err(|why| eyre!("Invalid local proposal '{}': {why}", path.display())))
            .and_then(|proposal: Self| {
                proposal
                    .validate()
                    .map_err(|why| eyre!("Invalid local proposal '{}': {why}", path.display()))
                    .map(|()| proposal)
            })
    }
    fn write(&self, path: &Path, replace: bool) -> ApiResult<()> {
        self.validate()
            .map_err(|why| eyre!("Invalid local proposal: {why}"))
            .and_then(|()| serde_json::to_vec_pretty(self).map_err(|why| eyre!("Cannot serialize local proposal: {why}")))
            .and_then(|content| match replace {
                | true => durable_replace_file(path, &content),
                | false => OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)
                    .and_then(|mut file| file.write_all(&content).and_then(|()| file.sync_all()))
                    .map_err(|why| eyre!("Cannot create local proposal '{}': {why}", path.display()))
                    .and_then(|()| sync_parent_directory(path)),
            })
    }
    fn apply(mut self, path: &Path, config: &ApplicationConfiguration, requested_id: &str) -> ApiResult<ApplyStatus> {
        match self.submission_id == requested_id.to_ascii_lowercase() && digest_text(&self.raw) == self.source_digest {
            | true => serde_json::from_str::<Submission>(&self.raw)
                .map_err(|why| eyre!("Invalid staged submission: {why}"))
                .and_then(|submission| submission.local_route(&config.registry()))
                .and_then(|(folder_id, destination, entry)| {
                    serde_json::to_value(&entry)
                        .and_then(|left| serde_json::to_value(&self.entry).map(|right| left == right))
                        .map_err(|why| eyre!("Cannot compare normalized entry: {why}"))
                        .and_then(
                            |same_entry| match folder_id == self.folder_id && destination == self.destination && same_entry {
                                | true => self.apply_at(path, &destination, entry),
                                | false => Err(eyre!("Local proposal route or normalized entry changed; restage")),
                            },
                        )
                }),
            | false => Err(eyre!("Local proposal identity or source digest does not match")),
        }
    }
    fn apply_at(&mut self, path: &Path, destination: &Path, entry: LogbookEntry) -> ApiResult<ApplyStatus> {
        match self.applied {
            | true => Ok(ApplyStatus::AlreadyApplied),
            | false => read_file_snapshot(destination, MAX_INDEX_BYTES).and_then(|(_, current, _)| match () {
                | _ if current == self.proposed_content => {
                    self.applied = true;
                    self.write(path, true).map(|()| ApplyStatus::AlreadyApplied)
                }
                | _ if digest_text(&current) != self.base_digest => Err(eyre!("Local index changed since staging; restage this submission")),
                | _ => entry
                    .proposed_content(destination)
                    .and_then(|(_, expected)| match expected == self.proposed_content {
                        | true => durable_replace_file(destination, self.proposed_content.as_bytes()).and_then(|()| {
                            self.applied = true;
                            self.write(path, true).map(|()| ApplyStatus::Applied)
                        }),
                        | false => Err(eyre!("Local proposal content does not match its source; restage")),
                    }),
            }),
        }
    }
}
impl fmt::Display for StageSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} staged, {} existing, {} rejected",
            self.staged.len(),
            self.existing.len(),
            self.rejected.len()
        )
    }
}
impl Submission {
    fn local_route(&self, registry: &FolderRegistry) -> ApiResult<(String, PathBuf, LogbookEntry)> {
        self.entry(registry).and_then(|entry| {
            registry
                .resolve_folder(&self.project_id)
                .and_then(|resolved| match (resolved.folder.bucket.is_some(), resolved.destination) {
                    | (true, _) => Err(eyre!("Local intake does not support bucket-associated folders")),
                    | (false, Destination::Local(path)) => fs::canonicalize(&path)
                        .map_err(|why| eyre!("Cannot resolve local index '{}': {why}", path.display()))
                        .map(|canonical| (resolved.folder.id, canonical, entry)),
                    | (false, Destination::GitLab { .. }) => Err(eyre!("GitLab destination requires the GitLab review adapter")),
                    | (false, Destination::Remote(_)) => Err(eyre!("Remote destination has no local review adapter")),
                })
        })
    }
}
fn digest_text(text: &str) -> String {
    format!("sha256:{}", HEXLOWER.encode(digest(&SHA256, text.as_bytes()).as_ref()))
}
fn stage_line(directory: &Path, source: &Path, line: u64, raw: &str, source_digest: &str, registry: &FolderRegistry) -> ApiResult<(String, bool)> {
    serde_json::from_str::<Submission>(raw)
        .map_err(|why| eyre!("Invalid submission JSON: {why}"))
        .and_then(|submission| {
            submission.local_route(registry).and_then(|(folder_id, destination, entry)| {
                Proposal::path(directory, &submission.submission_id).and_then(|path| {
                    let existing = match path.exists() {
                        | true => Proposal::read(&path).map(Some),
                        | false => Ok(None),
                    };
                    existing.and_then(|prior| match prior.as_ref() {
                        | Some(prior) if prior.raw != raw || prior.source_digest != source_digest => {
                            Err(eyre!("Submission UUID was already staged with different bytes"))
                        }
                        | Some(prior) if prior.applied => Ok((submission.submission_id.to_ascii_lowercase(), false)),
                        | _ => entry.clone().proposed_content(&destination).and_then(|(base_digest, content)| {
                            let proposal = Proposal {
                                submission_id: submission.submission_id.to_ascii_lowercase(),
                                source_path: source.to_path_buf(),
                                source_line: line,
                                raw: raw.to_string(),
                                source_digest: source_digest.to_string(),
                                folder_id,
                                destination,
                                entry,
                                base_digest,
                                proposed_content: content,
                                applied: false,
                            };
                            let unchanged = prior.as_ref().is_some_and(|prior| {
                                prior.base_digest == proposal.base_digest && prior.proposed_content == proposal.proposed_content
                            });
                            match unchanged {
                                | true => Ok((proposal.submission_id, false)),
                                | false => proposal.write(&path, prior.is_some()).map(|()| (proposal.submission_id, true)),
                            }
                        }),
                    })
                })
            })
        })
}
