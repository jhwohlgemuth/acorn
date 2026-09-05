//! Portable analyzer result models and supplied-content calculations
pub mod check;
pub mod error;
#[cfg(feature = "analysis")]
pub mod host;
pub mod model;
pub mod readability;

#[cfg(feature = "std")]
pub use check::IntoChecks;
pub use check::{check_readability_for_text, summary, Check, CheckCategory, CheckSeverity, Checks, OutputFormat, Standard};
