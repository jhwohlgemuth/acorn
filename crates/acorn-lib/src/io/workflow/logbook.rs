//! Pure repository proposals for normalized logbook form entries.
use super::{FileChange, FileChangeKind, RepositoryChangeSet, WorkflowFileReport};
use crate::io::ApiResult;
use acorn_core::{
    prelude::{String, Vec},
    util::MimeType,
};
use acorn_host::fs::SafePath;
use acorn_schema::{
    research_activity::{Logbook, LogbookEntry, ResearchActivity},
    validation::Validate,
};
use color_eyre::eyre::eyre;

/// Immutable input for one logbook repository proposal.
#[derive(Clone, Debug)]
pub struct LogbookProposal {
    /// Exact source revision read by the caller.
    pub base_revision: String,
    /// Normalized research activity JSON read at `base_revision`.
    pub content: String,
    /// Validated entry proposed for the activity logbook.
    pub entry: LogbookEntry,
    /// Repository-relative research activity path.
    pub path: String,
}
impl TryFrom<LogbookProposal> for RepositoryChangeSet {
    type Error = color_eyre::eyre::Report;

    /// Append a missing entry, accept an identical replay, or reject an identifier conflict.
    fn try_from(proposal: LogbookProposal) -> ApiResult<Self> {
        let LogbookProposal {
            base_revision,
            content,
            entry,
            path,
        } = proposal;
        match (base_revision.trim().is_empty(), SafePath::new(&path)) {
            | (true, _) => Err(eyre!("Repository revision is required")),
            | (_, Err(why)) => Err(eyre!(why)),
            | (false, Ok(safe_path)) => entry
                .validate()
                .map_err(|why| eyre!("Invalid logbook entry — {why}"))
                .and_then(|()| {
                    eserde::json::from_str::<ResearchActivity>(&content).map_err(|why| eyre!("Failed to parse research activity JSON — {why}"))
                })
                .and_then(|activity| {
                    activity
                        .validate()
                        .map_err(|why| eyre!("Invalid research activity data — {why}"))
                        .map(|()| activity)
                })
                .and_then(|activity| {
                    let existing = activity
                        .logbook
                        .as_ref()
                        .into_iter()
                        .flat_map(|logbook| &logbook.entries)
                        .find(|candidate| candidate.identifier == entry.identifier);
                    match existing {
                        | Some(existing) => serde_json::to_value(existing)
                            .and_then(|existing| serde_json::to_value(&entry).map(|entry| existing == entry))
                            .map_err(|why| eyre!("Failed to compare logbook replay — {why}"))
                            .and_then(|identical| match identical {
                                | true => Ok((activity, false)),
                                | false => Err(eyre!("Logbook entry identifier '{}' already has different content", entry.identifier)),
                            }),
                        | None => {
                            let logbook = match activity.logbook {
                                | Some(logbook) => Logbook {
                                    entries: logbook.entries.into_iter().chain(core::iter::once(entry)).collect(),
                                    ..logbook
                                },
                                | None => Logbook::init().entries(vec![entry]).build(),
                            };
                            Ok((
                                ResearchActivity {
                                    logbook: Some(logbook),
                                    ..activity
                                },
                                true,
                            ))
                        }
                    }
                })
                .and_then(|(activity, changed)| {
                    let path = safe_path.as_path().to_string_lossy().to_string();
                    match changed {
                        | false => Ok(RepositoryChangeSet {
                            base_revision,
                            files: Vec::new(),
                            reports: vec![WorkflowFileReport {
                                changed: false,
                                format: MimeType::Json,
                                path,
                            }],
                        }),
                        | true => serde_json::to_string_pretty(&activity)
                            .map_err(|why| eyre!("Failed to serialize research activity JSON — {why}"))
                            .map(|content| RepositoryChangeSet {
                                base_revision,
                                files: vec![FileChange {
                                    content: format!("{content}\n"),
                                    kind: FileChangeKind::Update,
                                    path: path.clone(),
                                }],
                                reports: vec![WorkflowFileReport {
                                    changed: true,
                                    format: MimeType::Json,
                                    path,
                                }],
                            }),
                    }
                }),
        }
    }
}
