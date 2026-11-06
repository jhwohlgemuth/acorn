//! Strict PowerAutomate form envelope.
use crate::io::api::webhooks::store::EnqueueStatus;
use crate::io::config::application::{FolderRegistry, GeneralLogbookConfig};
use crate::io::{ApiResult, EmailAddress};
use acorn_core::prelude::{String, Vec};
use acorn_core::validation::ValidationError;
use acorn_schema::research_activity::{
    ActivityEntry, LogbookCategory, LogbookDiscovery, LogbookEntry, LogbookEntryOrigin, LogbookEventType, LogbookState, ProjectHealth, ProjectRisk,
    ReportingWindow, StatusUpdateEntry,
};
use acorn_schema::validation::{rules, Validate, ValidationReport};
use acorn_schema::ClassificationLevel;
use color_eyre::eyre::eyre;
use convert_case::{Case, Casing};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::env;
use strum::EnumIs;

/// Form-controlled fields for an activity or project status update.
#[derive(Clone, Debug, Deserialize, EnumIs, JsonSchema, Serialize)]
#[serde(untagged)]
pub enum FormEntry {
    /// Existing activity intake fields.
    Activity(ActivityFormEntry),
    /// Project health and leadership significance fields.
    StatusUpdate(StatusFormEntry),
}

/// Receipt disposition backed by the durable webhook queue.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReceiptDisposition {
    /// The exact delivery identifier was already recorded.
    DuplicateDelivery,
    /// A new delivery mapped to an existing canonical operation.
    DuplicateOperation,
    /// A new operation was queued.
    Inserted,
}

/// Form-controlled fields for one manual activity entry.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ActivityFormEntry {
    /// Whether the entry has already been graduated into canonical metadata.
    #[serde(default)]
    pub archived: bool,
    /// Subject of the development.
    pub category: LogbookCategory,
    /// Optional national security classification.
    pub classification: Option<ClassificationLevel>,
    /// Provenance for a discovered entry.
    pub discovery: Option<LogbookDiscovery>,
    /// Optional event format.
    pub event_type: Option<LogbookEventType>,
    /// Whether the entry appears among report title-page updates.
    #[serde(default)]
    pub global: bool,
    /// Optional stable identifier; defaults to the submission UUID identity.
    pub identifier: Option<String>,
    /// Supporting statements.
    #[serde(default, deserialize_with = "optional_trimmed_items")]
    pub items: Option<Vec<String>>,
    /// Manual or discovered source of the entry.
    #[serde(default)]
    pub origin: LogbookEntryOrigin,
    /// Temporal state of the development.
    pub state: LogbookState,
    /// Concise description of the development.
    pub summary: String,
    /// RFC 3339 temporal anchor.
    pub timestamp: String,
}

/// Transport-independent result of admitting one form submission.
#[derive(Clone, Debug)]
pub(crate) struct FormAdmission {
    pub(crate) folder_id: String,
    pub(crate) entry: LogbookEntry,
    pub(crate) operation_key: String,
    pub(crate) replaces: Option<LogbookEntry>,
}

/// Immediate durable-intake response returned to PowerAutomate.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FormReceipt {
    /// Whether the delivery or operation was newly inserted or already known.
    pub disposition: ReceiptDisposition,
    /// Canonical durable operation identifier.
    pub operation_id: String,
}

/// Form-controlled fields for one project status update.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StatusFormEntry {
    /// Identify this update when correcting a prior submission.
    pub identifier: Option<String>,
    /// Record when this update was submitted.
    pub timestamp: String,
    /// Use the submission range unless this update covers a different period.
    pub range: Option<ReportingWindow>,
    /// Describe the most important project change or result this month.
    pub summary: String,
    /// Explain why this change matters to leadership.
    pub significance: String,
    /// Assess the health of the project overall.
    pub health: ProjectHealth,
    /// Cite a concrete delivery, demonstration, test, approval, or other proof.
    pub evidence: Option<String>,
    /// Explain a risk or dependency leadership should know about, when relevant.
    pub risk: Option<ProjectRisk>,
}

/// Strict version-one form submission.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[validate(schema(function = "Submission::is_valid", skip_on_field_errors = false))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Submission {
    /// Form-controlled logbook fields.
    pub entry: FormEntry,
    /// Stable deployment-owned folder identifier or alias.
    pub project_id: String,
    /// Default reporting range for a status update without its own range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub range: Option<ReportingWindow>,
    /// Immutable correction linkage; absent on an original receipt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub revision: Option<SubmissionRevision>,
    /// Stable UUID assigned by the submitting flow.
    #[validate(uuid)]
    pub submission_id: String,
    /// Submission instant with an explicit RFC 3339 offset.
    #[validate(timestamp)]
    pub submitted_at: String,
    /// Authenticated human submitter metadata.
    #[validate(nested)]
    pub submitter: Submitter,
}

/// The original receipt and immediate predecessor of a correction.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SubmissionRevision {
    /// UUID of the original submission, shared by every correction.
    #[validate(uuid)]
    pub root_submission_id: String,
    /// UUID of the exact receipt the contributor reviewed before editing.
    #[validate(uuid)]
    pub supersedes_submission_id: String,
}

/// Human submitter identity supplied by PowerAutomate.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Submitter {
    /// Submitter email matched against the deployment allowlist.
    #[validate(nested)]
    pub email: EmailAddress,
}

impl FormEntry {
    fn activity(&self) -> Option<&ActivityFormEntry> {
        match self {
            | Self::Activity(entry) => Some(entry),
            | Self::StatusUpdate(_) => None,
        }
    }
    pub(crate) fn identifier(&self) -> Option<&str> {
        match self {
            | Self::Activity(entry) => entry.identifier.as_deref(),
            | Self::StatusUpdate(entry) => entry.identifier.as_deref(),
        }
    }
    pub(crate) fn global(&self) -> bool {
        self.activity().is_some_and(|entry| entry.global)
    }
    pub(crate) fn archived(&self) -> bool {
        self.activity().is_some_and(|entry| entry.archived)
    }
    fn status_update(&self) -> Option<&StatusFormEntry> {
        match self {
            | Self::Activity(_) => None,
            | Self::StatusUpdate(entry) => Some(entry),
        }
    }
    fn timestamp(&self) -> &str {
        match self {
            | Self::Activity(entry) => &entry.timestamp,
            | Self::StatusUpdate(entry) => &entry.timestamp,
        }
    }
}

impl FormReceipt {
    pub(crate) fn new(operation_id: String, status: EnqueueStatus) -> Self {
        Self {
            disposition: status.into(),
            operation_id,
        }
    }
}

impl From<EnqueueStatus> for ReceiptDisposition {
    fn from(value: EnqueueStatus) -> Self {
        match value {
            | EnqueueStatus::Inserted => Self::Inserted,
            | EnqueueStatus::DuplicateDelivery => Self::DuplicateDelivery,
            | EnqueueStatus::DuplicateOperation => Self::DuplicateOperation,
        }
    }
}

impl StatusFormEntry {
    fn effective_range<'a>(&'a self, fallback: Option<&'a ReportingWindow>) -> Option<&'a ReportingWindow> {
        self.range.as_ref().or(fallback)
    }
    fn has_bounded_range(&self, fallback: Option<&ReportingWindow>) -> bool {
        self.range_start(fallback).is_some()
    }
    fn range_start<'a>(&'a self, fallback: Option<&'a ReportingWindow>) -> Option<&'a str> {
        self.effective_range(fallback).and_then(|range| range.left.as_deref())
    }
}

impl Submission {
    /// Validate, authorize, and normalize a submission against one routing snapshot.
    pub(crate) fn admit(&self, routing: &FolderRegistry, lookup: &dyn Fn(&str) -> Option<String>, cutoff: Option<&str>) -> ApiResult<FormAdmission> {
        match &self.revision {
            | Some(_) => Err(eyre!(
                "Corrections require batch intake with their receipt history; use intake stage --batch"
            )),
            | None => routing.folder(&self.project_id).map(|folder| folder.id.clone()).and_then(|folder_id| {
                let normalized = Self {
                    project_id: folder_id.clone(),
                    ..self.clone()
                };
                normalized
                    .validate()
                    .map_err(|why| eyre!("Invalid PowerAutomate form submission — {why}"))
                    .and_then(|()| routing.authorizes(&folder_id, self.submitter.email.as_str(), lookup))
                    .and_then(|authorized| match authorized {
                        | true => Ok(()),
                        | false => Err(eyre!("Submitter is not authorized for PowerAutomate folder '{}'", folder_id)),
                    })
                    .and_then(|()| {
                        cutoff
                            .map(|cutoff| {
                                rules::parse_timestamp(cutoff)
                                    .map_err(|_| eyre!("Invalid report cutoff timestamp"))
                                    .and_then(|cutoff| {
                                        rules::parse_timestamp(self.entry.timestamp())
                                            .map_err(|_| eyre!("Invalid logbook entry timestamp"))
                                            .and_then(|timestamp| match timestamp < cutoff {
                                                | true => Err(eyre!("Logbook entry predates the latest successful report cutoff")),
                                                | false => self.entry.status_update().map_or(Ok(()), |update| {
                                                    update
                                                        .range_start(self.range.as_ref())
                                                        .ok_or_else(|| eyre!("Status update requires a bounded reporting range"))
                                                        .and_then(|left| {
                                                            rules::parse_timestamp(left)
                                                                .map_err(|_| eyre!("Invalid status reporting range"))
                                                                .and_then(|left| match left < cutoff {
                                                                    | true => Err(eyre!(
                                                                        "Status reporting range predates the latest successful report cutoff"
                                                                    )),
                                                                    | false => Ok(()),
                                                                })
                                                        })
                                                }),
                                            })
                                    })
                            })
                            .transpose()
                            .map(|_| ())
                    })
                    .and_then(|()| self.normalized_admission(folder_id))
            }),
        }
    }
    pub(crate) fn admit_local(
        &self,
        routing: &FolderRegistry,
        general: Option<&GeneralLogbookConfig>,
        lookup: &dyn Fn(&str) -> Option<String>,
    ) -> ApiResult<FormAdmission> {
        match self.project_id.as_str() {
            | "general" => general.ok_or_else(|| eyre!("General logbook is not configured")).and_then(|general| {
                self.validate()
                    .map_err(|why| eyre!("Invalid PowerAutomate form submission — {why}"))
                    .and_then(|()| match self.entry.is_activity() && self.entry.global() && !self.entry.archived() {
                        | true => Ok(()),
                        | false => Err(eyre!("General submissions require global: true and archived: false")),
                    })
                    .and_then(|()| {
                        let email = self.submitter.email.as_str();
                        match general.members.iter().any(|member| member.as_str().eq_ignore_ascii_case(email)) {
                            | true => self.normalized_admission("general".to_string()),
                            | false => Err(eyre!("Submitter is not authorized for the general logbook")),
                        }
                    })
            }),
            | _ => self.admit(routing, lookup, None),
        }
    }
    fn normalized_entry(&self, folder_id: &str) -> LogbookEntry {
        let fallback = || format!("form-{}", self.submission_id.to_ascii_lowercase()).to_case(Case::Kebab);
        match &self.entry {
            | FormEntry::Activity(form) => ActivityEntry::init()
                .archived(form.archived)
                .category(form.category)
                .maybe_classification(form.classification.clone())
                .maybe_discovery(form.discovery.clone())
                .maybe_event_type(form.event_type)
                .global(form.global)
                .identifier(form.identifier.clone().unwrap_or_else(fallback))
                .maybe_items(form.items.clone())
                .origin(form.origin)
                .project_id(folder_id)
                .state(form.state)
                .summary(form.summary.trim())
                .timestamp(form.timestamp.trim())
                .build()
                .into(),
            | FormEntry::StatusUpdate(form) => LogbookEntry::StatusUpdate(
                StatusUpdateEntry::init()
                    .identifier(form.identifier.clone().unwrap_or_else(fallback))
                    .project_id(folder_id)
                    .timestamp(form.timestamp.trim())
                    .maybe_range(form.effective_range(self.range.as_ref()).cloned())
                    .summary(form.summary.trim())
                    .significance(form.significance.trim())
                    .health(form.health.clone())
                    .maybe_evidence(form.evidence.as_deref().map(str::trim))
                    .maybe_risk(form.risk.clone())
                    .build(),
            ),
        }
    }
    fn normalized_admission(&self, folder_id: String) -> ApiResult<FormAdmission> {
        let entry = self.normalized_entry(&folder_id);
        entry
            .validate()
            .map_err(|why| eyre!("Invalid normalized PowerAutomate logbook entry — {why}"))
            .map(|()| FormAdmission {
                folder_id,
                entry,
                operation_key: self.operation_key(),
                replaces: None,
            })
    }
    /// Convert authorized form fields into a manual, active logbook entry.
    pub fn entry(&self, routing: &FolderRegistry) -> ApiResult<LogbookEntry> {
        self.entry_after(routing, None)
    }
    /// Convert a form while rejecting entries older than the latest successful report cutoff.
    pub fn entry_after(&self, routing: &FolderRegistry, cutoff: Option<&str>) -> ApiResult<LogbookEntry> {
        self.admit(routing, &|name| env::var(name).ok(), cutoff).map(|admission| admission.entry)
    }
    pub(crate) fn intake_branch(&self) -> ApiResult<String> {
        rules::parse_timestamp(self.entry.timestamp())
            .map_err(|_| eyre!("Invalid logbook entry timestamp"))
            .map(|timestamp| {
                format!(
                    "acorn/powerautomate/{}/{}",
                    self.project_id.to_ascii_lowercase(),
                    timestamp.strftime("%Y-%m")
                )
            })
    }
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        let entry = self.normalized_entry(&self.project_id);
        if let Err(errors) = entry.validate() {
            report.append_prefixed("entry", errors);
        }
        let status_range_is_unbounded = self
            .entry
            .status_update()
            .is_some_and(|update| !update.has_bounded_range(self.range.as_ref()));
        if status_range_is_unbounded {
            report.add(
                "range",
                ValidationError::new("reporting_window").with_message("Status updates require a bounded submission or entry reporting range"),
            );
        }
        report.finish()
    }
    /// Canonical durable operation identity shared by HTTP and JSONL intake.
    pub fn operation_key(&self) -> String {
        format!("powerautomate:form:v1:{}", self.submission_id.to_ascii_lowercase())
    }
}

fn optional_trimmed_items<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<Vec<String>>::deserialize(deserializer).map(|items| items.map(|items| items.into_iter().map(|item| item.trim().to_string()).collect()))
}
