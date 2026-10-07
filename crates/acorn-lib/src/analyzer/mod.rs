//! Native analyzer adapters and host-side analysis services.
pub(crate) use acorn_analysis::{check, error, fair, readability, vale};
pub(crate) use acorn_analysis::{Check, CheckBatch, CheckCategory, CheckSeverity};
#[cfg(feature = "analysis")]
pub mod diff;
#[cfg(feature = "analysis")]
pub mod fix;
#[cfg(feature = "analysis")]
pub(crate) mod gpu;
#[cfg(feature = "analysis")]
pub mod host;
