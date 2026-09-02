//! Rich error contract for ACORN host integrations
//!
/// Result returned by host operations with rich diagnostic context.
#[cfg(feature = "std")]
pub type ApiResult<T> = core::result::Result<T, color_eyre::eyre::Report>;
