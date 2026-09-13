//! Portable analyzer result models and supplied-content calculations
pub mod check;
pub mod error;
pub mod fair;
#[cfg(feature = "analysis")]
pub mod host;
pub mod model;
pub mod readability;
#[cfg(test)]
mod tests;

#[cfg(feature = "std")]
pub use check::IntoChecks;
pub use check::{
    check_readability_for_text, Check, CheckBatch, CheckCategory, CheckSeverity, CheckSummary, Finding, Findings, OutputFormat, Standard,
};
pub use fair::{
    FairCounts, FairDetails, FairEvidence, FairEvidenceItem, FairIndicator, FairIndicatorId, FairPrinciple, FairPriority, FairProfile, FairState,
};
