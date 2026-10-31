//! Revalidate every staged line and project before writing a local batch.
use super::super::revisions::RevisionHistory;
use super::general;
use super::manifest::{batch_directory, BatchLine, BatchManifest, BatchProject, PriorRecordLookup};
use super::types::BatchLineStatus;
use super::{
    digest_text, ApplicationConfiguration, FolderRegistry, LogbookProposal, ProcessRepositoryResponse, Submission, MAX_INBOX_BYTES, MAX_INDEX_BYTES,
};
use crate::io::api::gitlab::jsonl::{split_snapshot, JsonlLine};
use crate::io::{read_file_snapshot, ApiResult};
use acorn_schema::hardware::memory::Memory;
use alloc::collections::BTreeMap;
use color_eyre::eyre::eyre;
use std::env;
use std::path::Path;

type BatchContext<'a> = (&'a ApplicationConfiguration, &'a Path, &'a str);
type LineContext<'a> = (
    &'a JsonlLine,
    &'a BatchManifest,
    &'a Path,
    &'a ApplicationConfiguration,
    &'a FolderRegistry,
    &'a PriorRecordLookup,
    &'a RevisionHistory,
    &'a mut BTreeMap<String, String>,
);
pub(super) trait Preflight<Context> {
    fn preflight(&self, context: Context) -> ApiResult<()>;
}
impl<'a> Preflight<LineContext<'a>> for BatchLine {
    fn preflight(&self, (source, manifest, config_path, config, registry, prior, history, seen): LineContext<'a>) -> ApiResult<()> {
        let stored = self;
        match () {
            | _ if source.line != stored.line || source.raw != stored.raw || source.digest != stored.digest => {
                Err(eyre!("Batch line evidence changed; restage"))
            }
            | _ if stored.status != BatchLineStatus::Accepted && stored.status != BatchLineStatus::Replayed => {
                Err(eyre!("Batch contains an invalid line status"))
            }
            | _ => serde_json::from_str::<Submission>(&stored.raw)
                .map_err(|_| eyre!("Invalid staged submission"))
                .and_then(|submission| {
                    history
                        .admit(&submission, registry, config.general_logbook(), &|name| env::var(name).ok())
                        .and_then(|admission| {
                            config
                                .local_destination(config_path, registry, &admission.folder_id)
                                .and_then(|destination| {
                                    let entry_matches = stored.entry.as_ref().and_then(|entry| serde_json::to_value(entry).ok())
                                        == serde_json::to_value(&admission.entry).ok();
                                    let predecessor_matches =
                                        serde_json::to_value(&stored.replaces).ok() == serde_json::to_value(&admission.replaces).ok();
                                    let id = submission.submission_id.to_ascii_lowercase();
                                    let earlier = seen.insert(id.clone(), stored.digest.clone());
                                    let prior_matches = match (stored.status, earlier.is_none()) {
                                        | (BatchLineStatus::Replayed, true) => prior.prior_submission(config_path, &id, &stored.digest, &destination),
                                        | _ => Ok(true),
                                    };
                                    prior_matches.and_then(|prior_matches| {
                                        let duplicate_changed = earlier.as_ref().is_some_and(|digest| digest != &stored.digest)
                                            || (stored.status == BatchLineStatus::Accepted && earlier.is_some());
                                        let identity_changed = stored.submission_id.as_deref() != Some(id.as_str())
                                            || stored.operation_key.as_deref() != Some(admission.operation_key.as_str())
                                            || stored.project_id.as_deref() != Some(admission.folder_id.as_str());
                                        let project_missing = stored.status == BatchLineStatus::Accepted
                                            && !manifest
                                                .projects
                                                .iter()
                                                .any(|project| project.project_id == admission.folder_id && project.destination == destination);
                                        let status_changed = duplicate_changed || identity_changed || project_missing;
                                        let admission_matches = prior_matches && entry_matches && predecessor_matches && !status_changed;
                                        match admission_matches {
                                            | true => Ok(()),
                                            | false => Err(eyre!("Batch admission or route changed; restage")),
                                        }
                                    })
                                })
                        })
                }),
        }
    }
}
impl<'a> Preflight<BatchContext<'a>> for BatchManifest {
    fn preflight(&self, (config, config_path, batch_id): BatchContext<'a>) -> ApiResult<()> {
        let manifest = self;
        match () {
            | _ if !manifest.can_apply || manifest.lines.iter().any(|line| line.status == BatchLineStatus::Rejected) => {
                Err(eyre!("Batch contains rejected lines and cannot be applied"))
            }
            | _ if manifest.batch_id != batch_id => Err(eyre!("Batch identity does not match its manifest")),
            | _ => read_file_snapshot(config_path, MAX_INBOX_BYTES).and_then(|(_, config_text, _)| {
                read_file_snapshot(&manifest.source_path, MAX_INBOX_BYTES).and_then(|(source_path, source_text, _)| {
                    let snapshot_matches = digest_text(&config_text) == manifest.config_digest
                        && source_path == manifest.source_path
                        && digest_text(&source_text) == manifest.source_digest;
                    match snapshot_matches {
                        | true => {
                            split_snapshot(&source_text, &Memory::kb(64)).and_then(|source_lines| match source_lines.len() == manifest.lines.len() {
                                | true => batch_directory(config_path).and_then(|directory| {
                                    PriorRecordLookup::from(directory.as_path()).load().and_then(|prior| {
                                        let history = prior.revision_history(source_lines.iter().map(|line| line.raw.as_str()))?;
                                        let registry = config.registry();
                                        let mut seen = BTreeMap::<String, String>::new();
                                        source_lines
                                            .iter()
                                            .zip(&manifest.lines)
                                            .try_for_each(|(source, stored)| {
                                                stored.preflight((source, manifest, config_path, config, &registry, &prior, &history, &mut seen))
                                            })
                                            .and_then(|()| manifest.projects.iter().try_for_each(|project| project.preflight(manifest)))
                                    })
                                }),
                                | false => Err(eyre!("Batch line count changed; restage")),
                            })
                        }
                        | false => Err(eyre!("Batch source or configuration changed; restage")),
                    }
                })
            }),
        }
    }
}
impl Preflight<&BatchManifest> for BatchProject {
    fn preflight(&self, manifest: &BatchManifest) -> ApiResult<()> {
        let project = self;
        match digest_text(&project.base_content) == project.base_digest && digest_text(&project.proposed_content) == project.final_digest {
            | true => manifest
                .lines
                .iter()
                .filter(|line| line.status == BatchLineStatus::Accepted && line.project_id.as_deref() == Some(&project.project_id))
                .try_fold(project.base_content.clone(), |content, line| {
                    project
                        .destination
                        .file_name()
                        .and_then(|name| name.to_str())
                        .ok_or_else(|| eyre!("Local index has no UTF-8 filename"))
                        .and_then(|path| {
                            line.entry.clone().ok_or_else(|| eyre!("Batch entry is missing")).map(|entry| {
                                LogbookProposal::builder()
                                    .base_revision(digest_text(&content))
                                    .content(&content)
                                    .entry(entry)
                                    .maybe_replaces(line.replaces.clone())
                                    .path(path)
                                    .build()
                            })
                        })
                        .and_then(|proposal| match project.project_id.as_str() {
                            | "general" => general::propose(&content, &proposal.entry, proposal.replaces.as_ref()),
                            | _ => ProcessRepositoryResponse::try_from(proposal)
                                .map(|response| response.files.first().map_or(content, |change| change.content.clone())),
                        })
                })
                .and_then(|rebuilt| match rebuilt == project.proposed_content {
                    | true => match project.project_id.as_str() {
                        | "general" => general::read_content(&project.destination),
                        | _ => read_file_snapshot(&project.destination, MAX_INDEX_BYTES).map(|(_, current, _)| current),
                    }
                    .and_then(|current| {
                        let current_digest = digest_text(&current);
                        match current_digest == project.base_digest || current_digest == project.final_digest {
                            | true => Ok(()),
                            | false => Err(eyre!("Local index changed since staging; restage")),
                        }
                    }),
                    | false => Err(eyre!("Batch proposed content does not match its submissions")),
                }),
            | false => Err(eyre!("Batch project content digest does not match its receipt")),
        }
    }
}
