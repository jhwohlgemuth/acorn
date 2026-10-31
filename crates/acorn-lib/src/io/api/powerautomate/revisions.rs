//! Pure correction-chain validation for reviewed local batches.
use super::{form::FormAdmission, Submission};
use crate::io::config::application::{FolderRegistry, GeneralLogbookConfig};
use crate::io::ApiResult;
use acorn_schema::validation::{rules, Validate};
use alloc::collections::{BTreeMap, BTreeSet};
use color_eyre::eyre::eyre;

pub(super) struct RevisionHistory {
    records: BTreeMap<String, Submission>,
    conflicts: BTreeSet<String>,
}
impl RevisionHistory {
    pub(super) fn read<'a>(rows: impl Iterator<Item = &'a str>) -> ApiResult<Self> {
        let mut records = BTreeMap::<String, (&str, Submission)>::new();
        let mut conflicts = BTreeSet::new();
        for (raw, submission) in rows.filter_map(|raw| serde_json::from_str::<Submission>(raw).ok().map(|submission| (raw, submission))) {
            let id = submission.submission_id.to_ascii_lowercase();
            if records.get(&id).is_some_and(|(previous, _)| *previous != raw) {
                conflicts.insert(id);
            } else {
                records.insert(id, (raw, submission));
            }
        }
        Ok(Self {
            records: records.into_iter().map(|(id, (_, submission))| (id, submission)).collect(),
            conflicts,
        })
    }
    pub(super) fn admit(
        &self,
        submission: &Submission,
        registry: &FolderRegistry,
        general: Option<&GeneralLogbookConfig>,
        lookup: &dyn Fn(&str) -> Option<String>,
    ) -> ApiResult<FormAdmission> {
        match &submission.revision {
            | None => submission.admit_local(registry, general, lookup),
            | Some(_) if self.conflicts.contains(&submission.submission_id.to_ascii_lowercase()) => {
                Err(eyre!("Conflicting receipt bytes for submission UUID"))
            }
            | Some(revision) => submission.validate().map_err(|why| eyre!("Invalid correction — {why}")).and_then(|()| {
                (0..128)
                    .try_fold((submission, BTreeSet::new()), |(current, mut visited), _| match &current.revision {
                        | None => Ok((current, visited)),
                        | Some(link) => {
                            let id = current.submission_id.to_ascii_lowercase();
                            match (visited.insert(id.clone()), link.root_submission_id == revision.root_submission_id) {
                                | (false, _) => Err(eyre!("Correction chain is cyclic or exceeds 128 revisions")),
                                | (_, false) => Err(eyre!("Correction chain changes its original receipt")),
                                | (true, true) => self
                                    .records
                                    .get(&link.supersedes_submission_id.to_ascii_lowercase())
                                    .ok_or_else(|| eyre!("Correction predecessor is missing from this source and applied batch history"))
                                    .and_then(|parent| {
                                        let forked = self.records.values().any(|other| {
                                            other.submission_id.to_ascii_lowercase() != id
                                                && other.revision.as_ref().is_some_and(|other_link| {
                                                    other_link.supersedes_submission_id.eq_ignore_ascii_case(&parent.submission_id)
                                                })
                                        });
                                        let invalid_state = !current.submitter.email.as_str().eq_ignore_ascii_case(parent.submitter.email.as_str())
                                            || current.project_id != parent.project_id
                                            || current.entry.global != parent.entry.global
                                            || current.entry.identifier != parent.entry.identifier
                                            || current.entry.archived
                                            || parent.entry.archived
                                            || rules::parse_timestamp(&current.submitted_at).ok() < rules::parse_timestamp(&parent.submitted_at).ok();
                                        match () {
                                            | _ if forked => {
                                                Err(eyre!("Competing corrections reference the same predecessor; operator review is required"))
                                            }
                                            | _ if self.conflicts.contains(&parent.submission_id.to_ascii_lowercase()) => {
                                                Err(eyre!("Conflicting predecessor receipt bytes"))
                                            }
                                            | _ => parent.validate().map_err(|why| eyre!("Invalid predecessor — {why}")).and_then(|()| {
                                                match invalid_state {
                                                    | true => Err(eyre!(
                                                        "Correction changes ownership, placement or identity, or has an invalid predecessor state"
                                                    )),
                                                    | false => Ok((parent, visited)),
                                                }
                                            }),
                                        }
                                    }),
                            }
                        }
                    })
                    .and_then(|(root, _)| match root.revision.is_some() {
                        | true => Err(eyre!("Correction chain is cyclic or exceeds 128 revisions")),
                        | false => Ok(root),
                    })
                    .and_then(|root| match root.submission_id.eq_ignore_ascii_case(&revision.root_submission_id) {
                        | false => Err(eyre!("Correction does not resolve to its declared original receipt")),
                        | true => {
                            let normalize = |receipt: &Submission| {
                                Submission {
                                    revision: None,
                                    submission_id: root.submission_id.clone(),
                                    ..receipt.clone()
                                }
                                .admit_local(registry, general, lookup)
                            };
                            self.records
                                .get(&revision.supersedes_submission_id.to_ascii_lowercase())
                                .ok_or_else(|| eyre!("Correction predecessor is missing"))
                                .and_then(|parent| {
                                    normalize(parent).and_then(|previous| {
                                        normalize(submission).map(|admission| FormAdmission {
                                            operation_key: submission.operation_key(),
                                            replaces: Some(previous.entry),
                                            ..admission
                                        })
                                    })
                                })
                        }
                    })
            }),
        }
    }
}
