//! Pure repository proposals for normalized logbook form entries.
use super::{FileChange, FileChangeKind, ProcessRepositoryResponse, WorkflowFileReport};
use crate::io::{ApiResult, FromPath};
use acorn_core::{
    prelude::{String, Vec},
    util::MimeType,
};
use acorn_host::fs::SafePath;
use acorn_schema::{
    codec::DocumentCodec,
    research_activity::{Logbook, LogbookEntry, ResearchActivity},
    validation::Validate,
};
use bon::Builder;
use color_eyre::eyre::eyre;

/// Immutable input for one logbook repository proposal.
#[derive(Builder, Clone, Debug)]
#[builder(on(String, into))]
pub struct LogbookProposal {
    /// Exact source revision read by the caller.
    pub base_revision: String,
    /// Research activity content read at `base_revision`.
    pub content: String,
    /// Validated entry proposed for the activity logbook.
    pub entry: LogbookEntry,
    /// Repository-relative research activity path.
    pub path: String,
}
impl TryFrom<LogbookProposal> for ProcessRepositoryResponse {
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
                    let format = MimeType::from_path(safe_path.as_path());
                    ResearchActivity::decode(&content, &format).map_err(|diagnostics| {
                        eyre!(
                            "Failed to parse research activity {format} — {}",
                            diagnostics.into_iter().map(|item| item.message).collect::<Vec<_>>().join("; ")
                        )
                    })
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
                    let format = MimeType::from_path(safe_path.as_path());
                    match changed {
                        | false => Ok(ProcessRepositoryResponse {
                            base_revision,
                            files: Vec::new(),
                            reports: vec![WorkflowFileReport {
                                changed: false,
                                format,
                                path,
                            }],
                        }),
                        | true => activity
                            .encode(&format)
                            .map_err(|why| eyre!("Failed to serialize research activity {format} — {}", why.message))
                            .map(|content| ProcessRepositoryResponse {
                                base_revision,
                                files: vec![FileChange {
                                    content: format!("{}\n", content.trim_end_matches('\n')),
                                    kind: FileChangeKind::Update,
                                    path: path.clone(),
                                }],
                                reports: vec![WorkflowFileReport { changed: true, format, path }],
                            }),
                    }
                }),
        }
    }
}
