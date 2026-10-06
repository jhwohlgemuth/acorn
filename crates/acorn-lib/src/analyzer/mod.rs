//! Portable analyzer result models and supplied-content calculations
pub mod check;
#[cfg(feature = "analysis")]
pub mod diff;
pub mod error;
pub mod fair;
#[cfg(feature = "analysis")]
pub mod fix;
#[cfg(feature = "analysis")]
pub(crate) mod gpu;
#[cfg(feature = "analysis")]
pub mod host;
pub mod model;
pub mod readability;
#[cfg(test)]
mod tests;

#[cfg(feature = "analysis")]
pub use check::IntoChecks;
pub use check::{
    check_readability_for_text, Check, CheckBatch, CheckCategory, CheckSeverity, CheckSummary, Finding, Findings, OutputFormat, Standard,
};
pub use fair::{
    FairCounts, FairDetails, FairEvidence, FairEvidenceItem, FairIndicator, FairIndicatorId, FairPrinciple, FairPriority, FairProfile, FairState,
};
