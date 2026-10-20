//! Reviewable, resumable batches for standalone local JSONL intake.
use super::types::{BatchApplyStatus, BatchInspectionStatus, BatchLineStatus, BatchProjectApplyStatus, BatchProjectState};
use super::{digest_text, ApplicationConfiguration, Destination, FolderRegistry, Proposal, MAX_INDEX_BYTES};
use crate::io::{read_file_snapshot, sync_parent_directory, ApiResult};
use acorn_host::fs::SafePath;
use acorn_schema::research_activity::LogbookEntry;
use acorn_schema::validation::Validate;
use alloc::collections::BTreeMap;
use bon::Builder;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all, read_dir, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const MAX_MANIFEST_BYTES: usize = 128 * 1024 * 1024;
pub(super) enum PriorRecordLookup {
    Pending(PathBuf),
    Loaded(BTreeMap<String, PriorRecord>),
}
/// Machine-readable result of applying a reviewed batch.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchApplySummary {
    /// Stable batch ID.
    pub batch_id: String,
    /// Complete or partial.
    pub status: BatchApplyStatus,
    /// Per-project outcomes.
    pub projects: Vec<BatchProjectOutcome>,
    /// Safe failure reason for a partial apply.
    pub error: Option<String>,
}
/// Read-only view of a staged batch.
#[derive(Builder, Clone, Debug, Serialize, Validate)]
#[builder(on(String, into))]
#[serde(rename_all = "camelCase")]
pub struct BatchInspection {
    /// Stable batch ID.
    pub batch_id: String,
    /// Canonical source path.
    pub source_path: PathBuf,
    /// Digest of the exact LF-terminated snapshot.
    #[validate(digest)]
    pub source_digest: String,
    /// Whether the batch has no rejected lines.
    pub can_apply: bool,
    /// Rejected, staged, partial, complete, or stale.
    pub status: BatchInspectionStatus,
    /// Per-line admission outcomes.
    pub lines: Vec<BatchLineView>,
    /// Per-project reviewed changes.
    pub projects: Vec<BatchProjectView>,
}
#[derive(Builder, Clone, Debug, Deserialize, Serialize, Validate)]
#[builder(on(String, into))]
#[serde(deny_unknown_fields)]
pub(super) struct BatchLine {
    pub(super) line: u64,
    pub(super) raw: String,
    #[validate(digest)]
    pub(super) digest: String,
    pub(super) submission_id: Option<String>,
    pub(super) operation_key: Option<String>,
    pub(super) project_id: Option<String>,
    pub(super) entry: Option<LogbookEntry>,
    pub(super) status: BatchLineStatus,
    pub(super) reason: Option<String>,
}
/// A line shown during read-only batch inspection.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchLineView {
    /// One-based physical source line number.
    pub line: u64,
    /// Submission UUID, when the envelope parsed.
    pub submission_id: Option<String>,
    /// Canonical project ID, when admission succeeded.
    pub project_id: Option<String>,
    /// Status: accepted, replayed, or rejected.
    pub status: BatchLineStatus,
    /// Safe rejection reason, when applicable.
    pub reason: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub(super) struct BatchManifest {
    pub(super) batch_id: String,
    #[validate(digest)]
    pub(super) config_digest: String,
    pub(super) source_path: PathBuf,
    #[validate(digest)]
    pub(super) source_digest: String,
    pub(super) can_apply: bool,
    #[validate(nested)]
    pub(super) lines: Vec<BatchLine>,
    #[validate(nested)]
    pub(super) projects: Vec<BatchProject>,
}
#[derive(Builder, Clone, Debug, Deserialize, Serialize, Validate)]
#[builder(on(String, into))]
#[serde(deny_unknown_fields)]
pub(super) struct BatchProject {
    pub(super) project_id: String,
    pub(super) destination: PathBuf,
    pub(super) base_content: String,
    pub(super) proposed_content: String,
    #[validate(digest)]
    pub(super) base_digest: String,
    #[validate(digest)]
    pub(super) final_digest: String,
    #[builder(default)]
    pub(super) applied: bool,
}
/// Outcome for one project after an apply attempt.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchProjectOutcome {
    /// Canonical project ID.
    pub project_id: String,
    /// Canonical local index path.
    pub destination: PathBuf,
    /// Applied, already-applied, or pending.
    pub status: BatchProjectApplyStatus,
}
/// A project shown during read-only batch inspection.
#[derive(Builder, Clone, Debug, Serialize, Validate)]
#[builder(on(String, into))]
#[serde(rename_all = "camelCase")]
pub struct BatchProjectView {
    /// Canonical project ID.
    pub project_id: String,
    /// Canonical local index path.
    pub destination: PathBuf,
    /// Exact index text read while staging.
    pub base_content: String,
    /// Proposed index text after all accepted entries.
    pub proposed_content: String,
    /// Digest of the staged base content.
    #[validate(digest)]
    pub base_digest: String,
    /// Digest of the staged final content.
    #[validate(digest)]
    pub final_digest: String,
    /// Whether this project has been applied.
    pub applied: bool,
    /// Current index state: base, final, or changed.
    pub state: BatchProjectState,
}
/// Counts and identity reported after staging one batch.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSummary {
    /// Stable identifier for this exact source and configuration snapshot.
    pub batch_id: String,
    /// Whether every line can be applied.
    pub can_apply: bool,
    /// Number of accepted new submissions.
    pub accepted: usize,
    /// Number of previously applied submissions.
    pub replayed: usize,
    /// Number of rejected lines.
    pub rejected: usize,
}
#[derive(Validate)]
pub(super) struct PriorRecord {
    #[validate(digest)]
    digest: String,
    destination: Option<PathBuf>,
    applied: bool,
}
impl BatchInspection {
    /// Inspect one staged batch without modifying its manifest or canonical indexes.
    pub fn inspect(config_path: &Path, batch_id: &str) -> ApiResult<Self> {
        ApplicationConfiguration::load_local_intake(config_path).and_then(|(_, resolved_config)| {
            manifest_path(&resolved_config, batch_id)
                .and_then(|path| BatchManifest::read(&path))
                .and_then(|manifest| match manifest.batch_id == batch_id {
                    | true => {
                        let mut inspection = manifest.inspection();
                        inspection.projects.iter_mut().for_each(|project| {
                            project.state = read_file_snapshot(&project.destination, MAX_INDEX_BYTES)
                                .map(|(_, text, _)| digest_text(&text))
                                .map_or_else(
                                    |_| BatchProjectState::Changed,
                                    |digest| match (digest == project.final_digest, digest == project.base_digest) {
                                        | (true, _) => BatchProjectState::Final,
                                        | (false, true) => BatchProjectState::Base,
                                        | _ => BatchProjectState::Changed,
                                    },
                                );
                            project.applied = project.state == BatchProjectState::Final;
                        });
                        inspection.status = match () {
                            | _ if !inspection.can_apply => BatchInspectionStatus::Rejected,
                            | _ if inspection.projects.iter().any(|project| project.state == BatchProjectState::Changed) => {
                                BatchInspectionStatus::Stale
                            }
                            | _ if inspection.projects.iter().all(|project| project.state == BatchProjectState::Final) => {
                                BatchInspectionStatus::Complete
                            }
                            | _ if inspection.projects.iter().any(|project| project.state == BatchProjectState::Final) => {
                                BatchInspectionStatus::Partial
                            }
                            | _ => BatchInspectionStatus::Staged,
                        };
                        Ok(inspection)
                    }
                    | false => Err(eyre!("Batch identity does not match its manifest")),
                })
        })
    }
}
impl BatchManifest {
    fn inspection(&self) -> BatchInspection {
        BatchInspection::builder()
            .batch_id(self.batch_id.as_str())
            .source_path(self.source_path.clone())
            .source_digest(self.source_digest.as_str())
            .can_apply(self.can_apply)
            .status(BatchInspectionStatus::Staged)
            .lines(
                self.lines
                    .iter()
                    .map(|line| BatchLineView {
                        line: line.line,
                        submission_id: line.submission_id.clone(),
                        project_id: line.project_id.clone(),
                        status: line.status,
                        reason: line.reason.clone(),
                    })
                    .collect(),
            )
            .projects(
                self.projects
                    .iter()
                    .map(|project| {
                        BatchProjectView::builder()
                            .project_id(project.project_id.as_str())
                            .destination(project.destination.clone())
                            .base_content(project.base_content.as_str())
                            .proposed_content(project.proposed_content.as_str())
                            .base_digest(project.base_digest.as_str())
                            .final_digest(project.final_digest.as_str())
                            .applied(project.applied)
                            .state(if project.applied {
                                BatchProjectState::Final
                            } else {
                                BatchProjectState::Base
                            })
                            .build()
                    })
                    .collect(),
            )
            .build()
    }
    pub(super) fn summary(&self) -> BatchSummary {
        let (accepted, replayed, rejected) = self
            .lines
            .iter()
            .fold((0_usize, 0_usize, 0_usize), |(accepted, replayed, rejected), line| match line.status {
                | BatchLineStatus::Accepted => (accepted.saturating_add(1), replayed, rejected),
                | BatchLineStatus::Rejected => (accepted, replayed, rejected.saturating_add(1)),
                | BatchLineStatus::Replayed => (accepted, replayed.saturating_add(1), rejected),
            });
        BatchSummary {
            batch_id: self.batch_id.clone(),
            can_apply: self.can_apply,
            accepted,
            replayed,
            rejected,
        }
    }
    pub(super) fn read(path: &Path) -> ApiResult<Self> {
        read_file_snapshot(path, MAX_MANIFEST_BYTES)
            .and_then(|(_, text, _)| serde_json::from_str::<Self>(&text).map_err(|why| eyre!("Invalid local batch manifest: {why}")))
            .and_then(|manifest| {
                manifest
                    .validate()
                    .map_err(|why| eyre!("Invalid local batch manifest: {why}"))
                    .map(|()| manifest)
            })
    }
    pub(super) fn write(&self, directory: &Path) -> ApiResult<()> {
        self.validate()
            .map_err(|why| eyre!("Invalid local batch manifest: {why}"))
            .and_then(|()| {
                SafePath::new(format!("{}.json", self.batch_id))
                    .and_then(|relative| relative.materialize_under(directory))
                    .map_err(|why| eyre!("Unsafe local batch manifest path: {why}"))
            })
            .and_then(|path| {
                serde_json::to_vec_pretty(self)
                    .map_err(|why| eyre!("Cannot encode local batch manifest: {why}"))
                    .and_then(|content| match content.len() > MAX_MANIFEST_BYTES {
                        | true => Err(eyre!("Local batch manifest exceeds the supported size")),
                        | false => {
                            let temporary = path.with_extension(format!("{}.tmp", nanoid::nanoid!()));
                            let mut options = OpenOptions::new();
                            options.write(true).create_new(true);
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::OpenOptionsExt;
                                options.mode(0o600);
                            }
                            let result = options
                                .open(&temporary)
                                .and_then(|mut file| file.write_all(&content).and_then(|()| file.sync_all()))
                                .and_then(|()| fs::rename(&temporary, &path));
                            match result {
                                | Ok(()) => sync_parent_directory(&path),
                                | Err(why) => {
                                    let _ = fs::remove_file(&temporary);
                                    Err(eyre!("Cannot write local batch manifest: {why}"))
                                }
                            }
                        }
                    })
            })
    }
}
impl BatchProject {
    pub(super) fn lock_destinations(projects: &[Self]) -> ApiResult<Vec<File>> {
        projects
            .iter()
            .map(|project| {
                let lock_path = project.destination.with_extension("json.acorn-intake.lock");
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(&lock_path)
                    .map_err(|why| eyre!("Cannot open local intake lock '{}': {why}", lock_path.display()))
                    .and_then(|file| {
                        file.try_lock()
                            .map_err(|why| eyre!("Local index is owned by another intake writer: {why}"))
                            .map(|()| file)
                    })
            })
            .collect()
    }
}
impl FolderRegistry {
    pub(super) fn local_destination(&self, selector: &str) -> ApiResult<PathBuf> {
        self.resolve_folder(selector)
            .and_then(|resolved| match (resolved.folder.bucket.is_some(), resolved.destination) {
                | (true, _) => Err(eyre!("Local intake does not support bucket-associated folders")),
                | (false, Destination::Local(path)) => path
                    .symlink_metadata()
                    .map_err(|why| eyre!("Cannot inspect local index: {why}"))
                    .and_then(|metadata| match metadata.is_symlink() || !metadata.is_file() {
                        | true => Err(eyre!("Local index must be a regular non-symlink file")),
                        | false => fs::canonicalize(&path).map_err(|why| eyre!("Cannot resolve local index: {why}")),
                    }),
                | (false, Destination::GitLab { .. }) => Err(eyre!("GitLab destination requires the GitLab review adapter")),
                | (false, Destination::Remote(_)) => Err(eyre!("Remote destination has no local review adapter")),
            })
    }
}
impl From<&Path> for PriorRecordLookup {
    fn from(directory: &Path) -> Self {
        Self::Pending(directory.to_path_buf())
    }
}
impl PriorRecordLookup {
    pub(super) fn load(self) -> ApiResult<Self> {
        match self {
            | Self::Pending(directory) => read_dir(&directory)
                .map_err(|why| eyre!("Cannot read batch directory: {why}"))
                .and_then(|entries| {
                    entries.into_iter().try_fold(Self::Loaded(BTreeMap::new()), |mut lookup, entry| {
                        entry
                            .map_err(|why| eyre!("Cannot read batch entry: {why}"))
                            .map(|entry| entry.path())
                            .and_then(|path| match path.extension().is_some_and(|extension| extension == "json") {
                                | true => BatchManifest::read(&path)
                                    .and_then(|manifest| manifest.lines.iter().try_for_each(|line| lookup.record_line(&manifest, line))),
                                | false => Ok(()),
                            })
                            .map(|()| lookup)
                    })
                }),
            | Self::Loaded(_) => Ok(self),
        }
    }
    pub(super) fn prior_submission(&self, config_path: &Path, id: &str, digest: &str, destination: &Path) -> ApiResult<bool> {
        Proposal::directory(config_path)
            .and_then(|path| Proposal::path(&path, id))
            .and_then(|proposal| match proposal.exists() {
                | true => Proposal::read(&proposal).and_then(|previous| match () {
                    | _ if previous.source_digest != digest => Err(eyre!("Submission UUID was already staged with different bytes")),
                    | _ if previous.destination != destination => Err(eyre!("Submission UUID was previously routed to a different local index")),
                    | _ => Ok(previous.applied
                        || read_file_snapshot(&previous.destination, MAX_INDEX_BYTES).is_ok_and(|(_, text, _)| text == previous.proposed_content)),
                }),
                | false => Ok(false),
            })
            .and_then(|applied| match self {
                | Self::Loaded(records) => match (applied, records.get(id)) {
                    | (true, _) => Ok(true),
                    | (_, Some(previous)) if previous.digest != digest => Err(eyre!("Submission UUID was already staged with different bytes")),
                    | (_, Some(previous)) if previous.destination.as_deref().is_some_and(|path| path != destination) => {
                        Err(eyre!("Submission UUID was previously routed to a different local index"))
                    }
                    | (_, Some(previous)) => Ok(previous.applied),
                    | (false, None) => Ok(false),
                },
                | Self::Pending(_) => Err(eyre!("Prior local batch records were not loaded")),
            })
    }
    fn record_line(&mut self, manifest: &BatchManifest, line: &BatchLine) -> ApiResult<()> {
        match self {
            | Self::Loaded(records) => match (line.status, line.submission_id.as_ref()) {
                | (BatchLineStatus::Rejected, _) | (_, None) => Ok(()),
                | (_, Some(id)) => {
                    let project = manifest
                        .projects
                        .iter()
                        .find(|project| line.project_id.as_deref() == Some(project.project_id.as_str()));
                    let applied = project.is_some_and(|project| {
                        project.applied
                            || read_file_snapshot(&project.destination, MAX_INDEX_BYTES)
                                .is_ok_and(|(_, text, _)| digest_text(&text) == project.final_digest)
                    });
                    match records.get_mut(id) {
                        | Some(record)
                            if record.digest != line.digest
                                || record
                                    .destination
                                    .as_ref()
                                    .zip(project)
                                    .is_some_and(|(path, project)| path != &project.destination) =>
                        {
                            Err(eyre!("Prior local batches contain a conflicting submission UUID"))
                        }
                        | Some(record) => {
                            record.applied |= applied;
                            if record.destination.is_none() {
                                record.destination = project.map(|project| project.destination.clone());
                            }
                            Ok(())
                        }
                        | None => {
                            records.insert(
                                id.clone(),
                                PriorRecord {
                                    digest: line.digest.clone(),
                                    destination: project.map(|project| project.destination.clone()),
                                    applied,
                                },
                            );
                            Ok(())
                        }
                    }
                }
            },
            | Self::Pending(_) => Err(eyre!("Prior local batch records were not loaded")),
        }
    }
}
pub(super) fn batch_directory(config_path: &Path) -> ApiResult<PathBuf> {
    config_path
        .parent()
        .ok_or_else(|| eyre!("Configuration path has no parent"))
        .and_then(|parent| {
            SafePath::new(".acorn/intake/batches")
                .and_then(|relative| relative.materialize_under(parent))
                .map_err(|why| eyre!("Unsafe local batch directory: {why}"))
        })
}
pub(super) fn manifest_path(config_path: &Path, batch_id: &str) -> ApiResult<PathBuf> {
    match batch_id.len() == 64 && batch_id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        | true => batch_directory(config_path).and_then(|directory| {
            SafePath::new(format!("{}.json", batch_id.to_ascii_lowercase()))
                .and_then(|relative| relative.materialize_under(&directory))
                .map_err(|why| eyre!("Unsafe local batch manifest path: {why}"))
        }),
        | false => Err(eyre!("Batch ID must be a SHA-256 hex digest")),
    }
}
pub(super) fn secure_directory(config_path: &Path) -> ApiResult<PathBuf> {
    batch_directory(config_path).and_then(|path| match path.symlink_metadata().is_ok_and(|metadata| metadata.is_symlink()) {
        | true => Err(eyre!("Local batch directory must not be a symlink")),
        | false => create_dir_all(&path)
            .map_err(|why| eyre!("Cannot create local batch directory: {why}"))
            .and_then(|()| {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).map_err(|why| eyre!("Cannot protect local batch directory: {why}"))
                }
                #[cfg(not(unix))]
                {
                    Ok(())
                }
            })
            .map(|()| path),
    })
}
