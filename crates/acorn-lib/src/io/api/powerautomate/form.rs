//! Strict PowerAutomate form envelope.
use crate::io::api::webhooks::store::EnqueueStatus;
use crate::io::config::application::FolderRegistry;
use crate::io::{ApiResult, EmailAddress};
use acorn_core::prelude::{String, Vec};
use acorn_schema::research_activity::{LogbookCategory, LogbookDiscovery, LogbookEntry, LogbookEntryOrigin, LogbookEventType, LogbookState};
use acorn_schema::validation::{rules, Validate, ValidationReport};
use acorn_schema::ClassificationLevel;
use color_eyre::eyre::eyre;
use convert_case::{Case, Casing};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::env;

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
/// Transport-independent result of admitting one form submission.
#[derive(Clone, Debug)]
pub(crate) struct FormAdmission {
    pub(crate) folder_id: String,
    pub(crate) entry: LogbookEntry,
    pub(crate) operation_key: String,
}
/// Form-controlled fields for one manual logbook entry.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FormEntry {
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
/// Immediate durable-intake response returned to PowerAutomate.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FormReceipt {
    /// Whether the delivery or operation was newly inserted or already known.
    pub disposition: ReceiptDisposition,
    /// Canonical durable operation identifier.
    pub operation_id: String,
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
/// Human submitter identity supplied by PowerAutomate.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Submitter {
    /// Submitter email matched against the deployment allowlist.
    #[validate(nested)]
    pub email: EmailAddress,
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
impl Submission {
    /// Validate, authorize, and normalize a submission against one routing snapshot.
    pub(crate) fn admit(&self, routing: &FolderRegistry, lookup: &dyn Fn(&str) -> Option<String>, cutoff: Option<&str>) -> ApiResult<FormAdmission> {
        routing.folder(&self.project_id).map(|folder| folder.id.clone()).and_then(|folder_id| {
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
                                    rules::parse_timestamp(&self.entry.timestamp)
                                        .map_err(|_| eyre!("Invalid logbook entry timestamp"))
                                        .and_then(|timestamp| match timestamp < cutoff {
                                            | true => Err(eyre!("Logbook entry predates the latest successful report cutoff")),
                                            | false => Ok(()),
                                        })
                                })
                        })
                        .transpose()
                        .map(|_| ())
                })
                .and_then(|()| {
                    let identifier = self
                        .entry
                        .identifier
                        .clone()
                        .unwrap_or_else(|| format!("form-{}", self.submission_id.to_ascii_lowercase()).to_case(Case::Kebab));
                    let entry = LogbookEntry::init()
                        .archived(self.entry.archived)
                        .category(self.entry.category)
                        .maybe_classification(self.entry.classification.clone())
                        .maybe_discovery(self.entry.discovery.clone())
                        .maybe_event_type(self.entry.event_type)
                        .global(self.entry.global)
                        .identifier(identifier)
                        .maybe_items(self.entry.items.clone())
                        .origin(self.entry.origin)
                        .project_id(&folder_id)
                        .state(self.entry.state)
                        .summary(self.entry.summary.trim())
                        .timestamp(self.entry.timestamp.trim())
                        .build();
                    entry
                        .validate()
                        .map_err(|why| eyre!("Invalid normalized PowerAutomate logbook entry — {why}"))
                        .map(|()| FormAdmission {
                            folder_id,
                            entry,
                            operation_key: self.operation_key(),
                        })
                })
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
        rules::parse_timestamp(&self.entry.timestamp)
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
        let entry = LogbookEntry::init()
            .archived(self.entry.archived)
            .category(self.entry.category)
            .maybe_classification(self.entry.classification.clone())
            .maybe_discovery(self.entry.discovery.clone())
            .maybe_event_type(self.entry.event_type)
            .global(self.entry.global)
            .identifier(self.entry.identifier.as_deref().unwrap_or("form-validation"))
            .maybe_items(self.entry.items.clone())
            .origin(self.entry.origin)
            .project_id(&self.project_id)
            .state(self.entry.state)
            .summary(self.entry.summary.trim())
            .timestamp(self.entry.timestamp.trim())
            .build();
        if let Err(errors) = entry.validate() {
            report.append_prefixed("entry", errors);
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
