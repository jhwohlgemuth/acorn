//! Durable JSONL admission with immutable routes and exact-byte replay protection.
use super::{config, form};
use crate::io::api::{
    gitlab::{self, jsonl::JsonlLine, PackageLifecycle},
    json_rpc,
    webhooks::{
        self,
        store::{IntakeRecord, OperationQueue},
    },
    Configuration,
};
use crate::io::config::{application, bucket};
use crate::io::ApiResult;
use acorn_core::Location;
use acorn_schema::{research_activity, validation::Validate};
use color_eyre::eyre::eyre;
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::env;

/// Entry and route fixed at the first successful admission.
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AdmittedSubmission {
    /// Canonicalized form; the original selector remains in source evidence.
    pub submission: form::Submission,
    /// Validated, authorized, cutoff-checked entry.
    pub entry: research_activity::LogbookEntry,
    /// Credential-free GitLab service URL fixed at admission.
    #[validate(url)]
    pub domain: String,
    /// Numeric destination GitLab project.
    pub project: u64,
    /// Repository-relative index path.
    pub path: String,
    /// Review branch derived from canonical folder and entry month.
    pub branch: String,
    /// Credential-free collection identity, when associated.
    pub bucket: Option<bucket::BucketSourceIdentity>,
}
impl AdmittedSubmission {
    fn operation(&self) -> ApiResult<webhooks::OperationSpec> {
        serde_json::to_value(&self.submission)
            .map_err(Into::into)
            .and_then(|params| {
                json_rpc::Notification::new(json_rpc::MethodName::from(["webhooks", "powerautomate", "form"]), params).map_err(Into::into)
            })
            .map(|notification| {
                webhooks::OperationSpec::new(
                    "powerautomate",
                    format!("form:v1:{}", self.submission.submission_id.to_ascii_lowercase()),
                    notification,
                )
            })
    }
}
impl IntakeRecord {
    async fn admit(
        self,
        config: &config::PowerAutomateConfig,
        routing: &application::FolderRegistry,
        queue: &OperationQueue,
        options: &gitlab::Options,
    ) -> ApiResult<Self> {
        match serde_json::from_str::<form::Submission>(&self.raw) {
            | Ok(submission) => {
                let record = Self {
                    selector: Some(submission.project_id.clone()),
                    operation_key: Some(submission.operation_key()),
                    ..self
                };
                match queue.intake_admission(&submission.operation_key()) {
                    | Ok(Some(previous)) if previous.raw != record.raw || previous.digest != record.digest => {
                        queue.reject(record, "Submission UUID conflicts with previously admitted bytes".to_string())
                    }
                    | Ok(Some(previous)) => serde_json::from_str::<AdmittedSubmission>(&previous.payload)
                        .map_err(Into::into)
                        .and_then(|admission| queue.persist(record, admission)),
                    | Ok(None) => match routing.resolve_folder(&submission.project_id) {
                        | Ok(folder) => match folder.destination {
                            | application::Destination::GitLab { project, path } if project.parse::<u64>().ok() == config.gitlab_project_id().ok() => {
                                let normalized = submission
                                    .admit(routing, &|name| env::var(name).ok(), None)
                                    .map(|admission| form::Submission {
                                        project_id: admission.folder_id,
                                        ..submission
                                    });
                                match normalized {
                                    | Ok(normalized) => match validate_bucket(routing, &normalized.project_id, config, options).await {
                                        | Ok(Err(reason)) => queue.reject(record, reason),
                                        | Ok(Ok(bucket)) => {
                                            let package = gitlab::Package::new(
                                                options.clone(),
                                                gitlab::PackageOptions::new(&config.package, &normalized.project_id),
                                            );
                                            let latest = match package {
                                                | Ok(value) => value.download().await,
                                                | Err(why) => Err(why),
                                            };
                                            match latest {
                                                | Ok(latest) => {
                                                    let cutoff = latest.as_ref().map(|manifest| manifest.through.as_str());
                                                    let admission = normalized.admit(routing, &|name| env::var(name).ok(), cutoff);
                                                    let branch = normalized.intake_branch();
                                                    let project_id = config.gitlab_project_id();
                                                    let url = gitlab::service_url(options);
                                                    match (admission, branch, project_id, url) {
                                                        | (Ok(admission), Ok(branch), Ok(project), Ok(domain)) => queue.persist(
                                                            Self {
                                                                operation_key: Some(admission.operation_key),
                                                                ..record
                                                            },
                                                            AdmittedSubmission {
                                                                submission: normalized,
                                                                entry: admission.entry,
                                                                project,
                                                                domain,
                                                                path,
                                                                branch,
                                                                bucket,
                                                            },
                                                        ),
                                                        | (Err(why), _, _, _) | (_, Err(why), _, _) | (_, _, Err(why), _) | (_, _, _, Err(why)) => {
                                                            queue.reject(record, why.to_string())
                                                        }
                                                    }
                                                }
                                                | Err(why) => Err(why),
                                            }
                                        }
                                        | Err(why) => Err(why),
                                    },
                                    | Err(why) => queue.reject(record, why.to_string()),
                                }
                            }
                            | _ => queue.reject(
                                record,
                                "Folder does not resolve into the configured GitLab project; local intake requires explicit proposal/apply"
                                    .to_string(),
                            ),
                        },
                        | Err(why) => queue.reject(record, why.to_string()),
                    },
                    | Err(why) => Err(why),
                }
            }
            | Err(why) => queue.reject(self, format!("Invalid PowerAutomate form — {why}")),
        }
    }
}
impl OperationQueue {
    fn persist(&self, record: IntakeRecord, admission: AdmittedSubmission) -> ApiResult<IntakeRecord> {
        admission.operation().and_then(|operation| {
            serde_json::to_string(&admission).map_err(Into::into).and_then(|payload| {
                self.record_intake(
                    &IntakeRecord {
                        admission: Some(payload),
                        ..record
                    },
                    Some(&operation),
                )
            })
        })
    }
    fn reject(&self, record: IntakeRecord, reason: String) -> ApiResult<IntakeRecord> {
        self.record_intake(
            &IntakeRecord {
                admission: None,
                rejection: Some(reason),
                ..record
            },
            None,
        )
    }
}
/// Admit every line independently, retaining rejections and retrying infrastructure failures.
///
/// Returned record errors are retryable and must never authorize cleanup.
/// The caller must consume every result so one unavailable project does not block another.
pub async fn admit_snapshot(
    snapshot: &gitlab::jsonl::Snapshot,
    config: &config::PowerAutomateConfig,
    routing: &application::FolderRegistry,
    queue: &OperationQueue,
    options: &gitlab::Options,
) -> ApiResult<Vec<ApiResult<IntakeRecord>>> {
    match (config.gitlab_project_id(), &config.intake) {
        | (Ok(project), Some(intake)) if options.identifier().and_then(|value| value.parse::<u64>().ok()) == Some(project) => {
            match source_identity(intake, options) {
                | Ok(source) => {
                    let admit_lines = stream::iter(&snapshot.lines)
                        .then(|line| {
                            IntakeRecord::builder()
                                .source(source.clone())
                                .commit(snapshot.commit.clone())
                                .line(line.line)
                                .byte_offset(line.byte_offset)
                                .raw(line.raw.clone())
                                .digest(line.digest.clone())
                                .build()
                                .admit(config, routing, queue, options)
                        })
                        .collect()
                        .await;
                    Ok(admit_lines)
                }
                | Err(why) => Err(why),
            }
        }
        | (_, None) => Err(eyre!("JSONL intake is not configured")),
        | _ => Err(eyre!("JSONL source options must name the configured numeric GitLab project")),
    }
}
/// Whether an exact line is already recoverable from durable storage.
pub fn is_admitted(line: &JsonlLine, queue: &OperationQueue) -> ApiResult<bool> {
    match serde_json::from_str::<form::Submission>(&line.raw) {
        | Err(_) => Ok(false),
        | Ok(submission) => queue
            .intake_admission(&submission.operation_key())
            .map(|admission| admission.is_some_and(|admission| admission.raw == line.raw && admission.digest == line.digest)),
    }
}
/// Credential-free identity distinguishing inboxes on the same GitLab service.
pub fn source_identity(intake: &application::PowerAutomateIntake, options: &gitlab::Options) -> ApiResult<String> {
    gitlab::service_url(options)
        .and_then(|service| serde_json::to_string(&(service, options.identifier(), &intake.branch, &intake.path)).map_err(Into::into))
}
async fn validate_bucket(
    routing: &application::FolderRegistry,
    selector: &str,
    config: &config::PowerAutomateConfig,
    options: &gitlab::Options,
) -> ApiResult<Result<Option<bucket::BucketSourceIdentity>, String>> {
    match routing.bucket_identity(selector) {
        | Ok(None) => Ok(Ok(None)),
        | Ok(Some(identity)) => {
            let location = Location::from(&identity);
            let origin = location.origin().zip(gitlab::service_url(options).ok()).is_some_and(|(bucket, service)| {
                bucket == service && !location.path().is_some_and(|path| path.split('/').any(|segment| segment == "-"))
            });
            match (identity.kind(), origin, routing.bucket_project(selector)) {
                | (bucket::BucketSourceKind::GitLab, true, Ok(Some(project))) => {
                    let identifier = urlencoding::encode(&project).into_owned();
                    gitlab::project_details(&options.clone().with_identifier(identifier))
                        .await
                        .and_then(|details| {
                            config.gitlab_project_id().map(|expected| match details.identifier == expected {
                                | true => Ok(Some(identity)),
                                | false => Err("Associated bucket belongs to a different GitLab project".to_string()),
                            })
                        })
                }
                | _ => Ok(Err(
                    "Associated collection cannot be verified as the destination GitLab repository".to_string()
                )),
            }
        }
        | Err(why) => Ok(Err(why.to_string())),
    }
}
