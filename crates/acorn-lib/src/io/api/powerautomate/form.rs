//! Strict version-one PowerAutomate form envelope.
use super::PowerAutomateConfig;
use crate::io::api::webhooks::store::EnqueueStatus;
use crate::io::{ApiResult, EmailAddress};
use acorn_core::prelude::{String, Vec};
use acorn_core::validation::ValidationError;
use acorn_schema::research_activity::{LogbookCategory, LogbookEntry, LogbookEventType, LogbookState};
use acorn_schema::validation::{rules, Validate, ValidationReport};
use acorn_schema::ClassificationLevel;
use color_eyre::eyre::eyre;
use convert_case::{Case, Casing};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

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
/// Form-controlled fields for one manual logbook entry.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FormEntry {
    /// Subject of the development.
    pub category: LogbookCategory,
    /// Optional national security classification.
    pub classification: Option<ClassificationLevel>,
    /// Optional event format.
    pub event_type: Option<LogbookEventType>,
    /// Supporting statements.
    #[serde(default, deserialize_with = "optional_trimmed_items")]
    pub items: Option<Vec<String>>,
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
#[validate(schema(function = "FormSubmission::is_valid", skip_on_field_errors = false))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FormSubmission {
    /// Form-controlled logbook fields.
    pub entry: FormEntry,
    /// Stable deployment-owned project identifier.
    pub project_id: String,
    /// Wire schema version. Version `1` is supported.
    pub schema_version: u8,
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
impl FormSubmission {
    /// Convert authorized form fields into a manual, active logbook entry.
    pub fn entry(&self, config: &PowerAutomateConfig) -> ApiResult<LogbookEntry> {
        self.entry_after(config, None)
    }
    /// Convert a form while rejecting entries older than the latest successful report cutoff.
    pub fn entry_after(&self, config: &PowerAutomateConfig, cutoff: Option<&str>) -> ApiResult<LogbookEntry> {
        self.validate()
            .map_err(|why| eyre!("Invalid PowerAutomate form submission — {why}"))
            .and_then(|()| config.validate().map_err(|why| eyre!("Invalid PowerAutomate configuration — {why}")))
            .and_then(|()| config.project(&self.project_id))
            .and_then(|project| match project.members.contains(&self.submitter.email) {
                | true => Ok(()),
                | false => Err(eyre!("Submitter is not authorized for PowerAutomate project '{}'", self.project_id)),
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
                let entry = LogbookEntry::init()
                    .category(self.entry.category)
                    .maybe_classification(self.entry.classification.clone())
                    .maybe_event_type(self.entry.event_type)
                    .identifier(format!("form-{}", self.submission_id.to_ascii_lowercase()).to_case(Case::Kebab))
                    .maybe_items(self.entry.items.clone())
                    .state(self.entry.state)
                    .summary(self.entry.summary.trim())
                    .timestamp(self.entry.timestamp.trim())
                    .build();
                entry
                    .validate()
                    .map_err(|why| eyre!("Invalid normalized PowerAutomate logbook entry — {why}"))
                    .map(|()| entry)
            })
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
        if self.schema_version != 1 {
            report.add(
                "schemaVersion",
                ValidationError::new("version").with_message("Only schema version 1 is supported"),
            );
        }
        let entry = LogbookEntry::init()
            .category(self.entry.category)
            .maybe_classification(self.entry.classification.clone())
            .maybe_event_type(self.entry.event_type)
            .identifier("form-validation")
            .maybe_items(self.entry.items.clone())
            .state(self.entry.state)
            .summary(self.entry.summary.trim())
            .timestamp(self.entry.timestamp.trim())
            .build();
        if let Err(errors) = entry.validate() {
            report.append_prefixed("entry", errors);
        }
        report.finish()
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
fn optional_trimmed_items<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<Vec<String>>::deserialize(deserializer).map(|items| items.map(|items| items.into_iter().map(|item| item.trim().to_string()).collect()))
}
