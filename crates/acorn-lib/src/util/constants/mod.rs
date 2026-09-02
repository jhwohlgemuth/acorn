//! Host/runtime constants for the ACORN application and adapters.

/// Application, service, UI, filesystem, and networking constants.
pub mod app;
/// Runtime environment variable names.
pub mod env;
/// Vale runtime configuration and package constants.
pub mod vale;

/// URL-encoded space character.
pub const URL_ENCODED_SPACE: &str = "%20";
/// URL-encoded caret character.
pub const URL_ENCODED_CARAT: &str = "%5E";
/// Maximum number of source characters preserved before a highlighted analyzer span.
pub const MAX_LENGTH_REPORT_SPAN_PREFIX: usize = 50;
