//! Local JSONL batch admission and per-project proposal folding.
use super::super::revisions::RevisionHistory;
use super::super::{form::FormAdmission, Submission};
use super::general;
use super::manifest::{secure_directory, BatchLine, BatchManifest, BatchProject, BatchSummary, PriorRecordLookup};
use super::types::BatchLineStatus;
use super::{digest_text, ApplicationConfiguration, FolderRegistry, LogbookProposal, ProcessRepositoryResponse, MAX_INBOX_BYTES, MAX_INDEX_BYTES};
use crate::io::api::gitlab::jsonl::{split_snapshot, JsonlLine};
use crate::io::{read_file_snapshot, ApiResult};
use crate::util::constants::app::APPLICATION;
use acorn_schema::hardware::memory::Memory;
use alloc::collections::BTreeMap;
use bon::Builder;
use color_eyre::eyre::eyre;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Builder)]
struct StageContext<'a> {
    config: &'a ApplicationConfiguration,
    config_path: &'a Path,
    registry: &'a FolderRegistry,
    prior: &'a PriorRecordLookup,
    history: &'a RevisionHistory,
    #[builder(default)]
    seen: BTreeMap<String, String>,
    #[builder(default)]
    projects: BTreeMap<PathBuf, BatchProject>,
}
impl BatchSummary {
    /// Stage one exact local JSONL snapshot as a reviewable batch.
    pub fn stage(config_path: &Path, inbox_path: &Path) -> ApiResult<Self> {
        ApplicationConfiguration::load_local_intake(config_path).and_then(|(config, resolved_config)| {
            read_file_snapshot(&resolved_config, MAX_INBOX_BYTES).and_then(|(_, config_text, _)| {
                read_file_snapshot(inbox_path, MAX_INBOX_BYTES).and_then(|(source_path, source_text, _)| {
                    let config_digest = digest_text(&config_text);
                    let source_digest = digest_text(&source_text);
                    let batch_id = digest_text(&format!(
                        "{APPLICATION}-local-batch-v1\0{config_digest}\0{}\0{source_digest}",
                        source_path.display()
                    ))
                    .trim_start_matches("sha256:")
                    .to_string();
                    secure_directory(&resolved_config).and_then(|directory| {
                        let path = directory.join(format!("{batch_id}.json"));
                        match path.exists() {
                            | true => BatchManifest::read(&path).and_then(|manifest| {
                                let same_batch = manifest.batch_id == batch_id
                                    && manifest.config_digest == config_digest
                                    && manifest.source_digest == source_digest
                                    && manifest.source_path == source_path;
                                match same_batch {
                                    | true => Ok(manifest.summary()),
                                    | false => Err(eyre!("Existing batch identity conflicts with the source snapshot")),
                                }
                            }),
                            | false => split_snapshot(&source_text, &Memory::kb(64)).and_then(|lines| {
                                PriorRecordLookup::from(directory.as_path()).load().and_then(|prior| {
                                    prior.revision_history(lines.iter().map(|line| line.raw.as_str())).and_then(|history| {
                                        let registry = config.registry();
                                        let mut context = StageContext::builder()
                                            .config(&config)
                                            .config_path(&resolved_config)
                                            .registry(&registry)
                                            .prior(&prior)
                                            .history(&history)
                                            .build();
                                        let lines = lines.into_iter().map(|line| line.stage(&mut context)).collect::<Vec<_>>();
                                        let can_apply = lines.iter().all(|line| line.status != BatchLineStatus::Rejected);
                                        let manifest = BatchManifest {
                                            batch_id,
                                            config_digest,
                                            source_path,
                                            source_digest,
                                            can_apply,
                                            lines,
                                            projects: context.projects.into_values().collect(),
                                        };
                                        manifest.write(&directory).map(|()| manifest.summary())
                                    })
                                })
                            }),
                        }
                    })
                })
            })
        })
    }
}
impl JsonlLine {
    fn stage(self, context: &mut StageContext<'_>) -> BatchLine {
        let JsonlLine {
            line: line_number,
            raw,
            digest,
            ..
        } = self;
        let parsed = serde_json::from_str::<Submission>(&raw).map_err(|why| eyre!("Invalid submission JSON: {why}"));
        let submission_id = parsed.as_ref().ok().map(|submission| submission.submission_id.to_ascii_lowercase());
        let result = parsed.and_then(|submission| {
            let id = submission.submission_id.to_ascii_lowercase();
            match context.seen.get(&id) {
                | Some(previous) if previous != &digest => Err(eyre!("Submission UUID conflicts with another line in this batch")),
                | Some(_) => context
                    .history
                    .admit(&submission, context.registry, context.config.general_logbook(), &|name| {
                        env::var(name).ok()
                    })
                    .and_then(|admission| {
                        context
                            .config
                            .local_destination(context.config_path, context.registry, &admission.folder_id)
                            .map(|_| (Some(admission), BatchLineStatus::Replayed))
                    }),
                | None => {
                    context.seen.insert(id.clone(), digest.clone());
                    context
                        .history
                        .admit(&submission, context.registry, context.config.general_logbook(), &|name| {
                            env::var(name).ok()
                        })
                        .and_then(|admission| {
                            context
                                .config
                                .local_destination(context.config_path, context.registry, &admission.folder_id)
                                .and_then(|destination| {
                                    context.prior.prior_submission(context.config_path, &id, &digest, &destination).and_then(
                                        |replayed| match replayed {
                                            | true => Ok((Some(admission), BatchLineStatus::Replayed)),
                                            | false => fold_project(&mut context.projects, &destination, &admission)
                                                .map(|()| (Some(admission), BatchLineStatus::Accepted)),
                                        },
                                    )
                                })
                        })
                }
            }
        });
        match result {
            | Ok((admission, status)) => BatchLine::builder()
                .line(line_number)
                .raw(raw)
                .digest(digest)
                .maybe_submission_id(submission_id)
                .maybe_operation_key(admission.as_ref().map(|admission| admission.operation_key.clone()))
                .maybe_project_id(admission.as_ref().map(|admission| admission.folder_id.clone()))
                .maybe_replaces(admission.as_ref().and_then(|admission| admission.replaces.clone()))
                .maybe_entry(admission.map(|admission| admission.entry))
                .status(status)
                .build(),
            | Err(why) => BatchLine::builder()
                .line(line_number)
                .raw(raw)
                .digest(digest)
                .maybe_submission_id(submission_id)
                .status(BatchLineStatus::Rejected)
                .reason(why.to_string())
                .build(),
        }
    }
}
fn fold_project(projects: &mut BTreeMap<PathBuf, BatchProject>, destination: &Path, admission: &FormAdmission) -> ApiResult<()> {
    match projects.get(destination).is_some_and(|project| project.project_id != admission.folder_id) {
        | true => Err(eyre!("Two canonical folders resolve to the same local index")),
        | false => {
            let base = match projects.get(destination) {
                | Some(project) => Ok(project.proposed_content.clone()),
                | None if admission.folder_id == "general" => general::read_content(destination),
                | None => read_file_snapshot(destination, MAX_INDEX_BYTES).map(|(_, text, _)| text),
            };
            base.and_then(|base| {
                if admission.folder_id == "general" {
                    return general::propose(&base, &admission.entry, admission.replaces.as_ref()).map(|proposed| {
                        match projects.get_mut(destination) {
                            | Some(project) => {
                                project.proposed_content = proposed.clone();
                                project.final_digest = digest_text(&proposed);
                            }
                            | None => {
                                projects.insert(
                                    destination.to_path_buf(),
                                    BatchProject::builder()
                                        .project_id("general")
                                        .destination(destination.to_path_buf())
                                        .base_digest(digest_text(&base))
                                        .final_digest(digest_text(&proposed))
                                        .base_content(base)
                                        .proposed_content(proposed)
                                        .build(),
                                );
                            }
                        }
                    });
                }
                destination
                    .file_name()
                    .and_then(|name| name.to_str())
                    .ok_or_else(|| eyre!("Local index has no UTF-8 filename"))
                    .and_then(|path| {
                        ProcessRepositoryResponse::try_from(
                            LogbookProposal::builder()
                                .base_revision(digest_text(&base))
                                .content(&base)
                                .entry(admission.entry.clone())
                                .maybe_replaces(admission.replaces.clone())
                                .path(path)
                                .build(),
                        )
                    })
                    .map(|response| {
                        let proposed = response.files.first().map_or(base.clone(), |change| change.content.clone());
                        match projects.get_mut(destination) {
                            | Some(project) => {
                                project.proposed_content = proposed.clone();
                                project.final_digest = digest_text(&proposed);
                            }
                            | None => {
                                projects.insert(
                                    destination.to_path_buf(),
                                    BatchProject::builder()
                                        .project_id(admission.folder_id.as_str())
                                        .destination(destination.to_path_buf())
                                        .base_digest(digest_text(&base))
                                        .final_digest(digest_text(&proposed))
                                        .base_content(base)
                                        .proposed_content(proposed)
                                        .build(),
                                );
                            }
                        }
                    })
            })
        }
    }
}
