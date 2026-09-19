//! Durable outbound events for automated artifact workflows.
use super::store::OperationQueue;
use crate::io::ApiResult;
use acorn_core::prelude::String;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Signed metadata event delivered to a named webhook callback destination.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", tag = "event")]
pub enum ArtifactEvent {
    /// An immutable generated artifact package is ready.
    #[serde(rename = "artifact-ready")]
    ArtifactReady {
        /// SHA-256 checksum of the PDF.
        checksum: String,
        /// Authenticated GitLab package URL.
        package_url: String,
        /// Stable project identifier.
        project_id: String,
        /// Exclusive reporting upper bound.
        through: String,
    },
    /// Intake or artifact creation failed after durable retries began.
    #[serde(rename = "failed")]
    Failed {
        /// Secret-free failure summary.
        message: String,
        /// Stable project identifier.
        project_id: String,
    },
    /// A rolling draft merge request is ready for review.
    #[serde(rename = "mr-created")]
    MergeRequestCreated {
        /// Browser URL for the draft merge request.
        merge_request_url: String,
        /// Stable project identifier.
        project_id: String,
    },
    /// A successful reporting period contained no entries.
    #[serde(rename = "report-no-changes")]
    ReportNoChanges {
        /// Stable project identifier.
        project_id: String,
        /// Exclusive reporting upper bound.
        through: String,
    },
    /// One or more graduation candidates require a human decision.
    #[serde(rename = "review-needed")]
    ReviewNeeded {
        /// Browser URL for the draft merge request.
        merge_request_url: String,
        /// Stable project identifier.
        project_id: String,
    },
}
impl ArtifactEvent {
    /// Persist this event for signed, retryable callback delivery.
    pub fn enqueue(&self, queue: &OperationQueue, destination: &str, idempotency_key: &str) -> ApiResult<bool> {
        queue.enqueue_callback(&format!("webhook:{}:{idempotency_key}", self.kind()), destination, self)
    }
    fn kind(&self) -> &'static str {
        self.into()
    }
}
impl From<&ArtifactEvent> for &'static str {
    fn from(value: &ArtifactEvent) -> Self {
        match value {
            | ArtifactEvent::ArtifactReady { .. } => "artifact-ready",
            | ArtifactEvent::Failed { .. } => "failed",
            | ArtifactEvent::MergeRequestCreated { .. } => "mr-created",
            | ArtifactEvent::ReportNoChanges { .. } => "report-no-changes",
            | ArtifactEvent::ReviewNeeded { .. } => "review-needed",
        }
    }
}
