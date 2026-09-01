//! Error contracts for portable domain code and host integrations.
pub use acorn_core::error::{AcornError, AcornResult};
/// Result returned by host operations with rich diagnostic context.
#[cfg(feature = "std")]
pub type ApiResult<T> = core::result::Result<T, color_eyre::eyre::Report>;
