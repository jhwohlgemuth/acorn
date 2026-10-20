//! Typed outcomes for local batch staging, inspection, and application.
use serde::{Deserialize, Serialize};

/// Result of applying a reviewed batch.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum BatchApplyStatus {
    /// Every project reached its final content.
    Complete,
    /// At least one project remains pending after an apply failure.
    Partial,
}
/// Current status of a staged batch.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum BatchInspectionStatus {
    /// Every project has its proposed content.
    Complete,
    /// Some projects have their proposed content.
    Partial,
    /// At least one source line was rejected.
    Rejected,
    /// No project has been applied yet.
    Staged,
    /// An index changed independently of the batch.
    Stale,
}
/// Admission outcome for one JSONL source line.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum BatchLineStatus {
    /// New submission included in the proposal.
    Accepted,
    /// Invalid or conflicting submission.
    Rejected,
    /// Previously staged or applied submission.
    Replayed,
}
/// Apply outcome for one project in a batch.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum BatchProjectApplyStatus {
    /// Proposed content was already present.
    AlreadyApplied,
    /// Proposed content was written now.
    Applied,
    /// No write was attempted or completed.
    Pending,
}
/// Current content state of one canonical project index.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum BatchProjectState {
    /// Index still has its staged base content.
    Base,
    /// Index changed independently of the batch.
    Changed,
    /// Index has the proposed final content.
    Final,
}
impl From<&str> for BatchApplyStatus {
    fn from(value: &str) -> Self {
        match value {
            | "complete" => Self::Complete,
            | _ => Self::Partial,
        }
    }
}
impl From<&str> for BatchInspectionStatus {
    fn from(value: &str) -> Self {
        match value {
            | "complete" => Self::Complete,
            | "partial" => Self::Partial,
            | "rejected" => Self::Rejected,
            | "staged" => Self::Staged,
            | _ => Self::Stale,
        }
    }
}
impl From<&str> for BatchLineStatus {
    fn from(value: &str) -> Self {
        match value {
            | "accepted" => Self::Accepted,
            | "replayed" => Self::Replayed,
            | _ => Self::Rejected,
        }
    }
}
impl From<&str> for BatchProjectApplyStatus {
    fn from(value: &str) -> Self {
        match value {
            | "already-applied" => Self::AlreadyApplied,
            | "applied" => Self::Applied,
            | _ => Self::Pending,
        }
    }
}
impl From<&str> for BatchProjectState {
    fn from(value: &str) -> Self {
        match value {
            | "base" => Self::Base,
            | "final" => Self::Final,
            | _ => Self::Changed,
        }
    }
}
