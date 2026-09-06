//! Host/runtime constants for the ACORN application and adapters.

/// Application, service, UI, filesystem, and networking constants.
pub mod app;
/// Runtime environment variable names.
pub mod env;
/// OCI artifact, ModelKit, ModelPack, and ORAS constants.
pub mod oci;
/// Vale runtime configuration and package constants.
pub mod vale;
/// Maximum number of source characters preserved before a highlighted analyzer span.
pub const MAX_LENGTH_REPORT_SPAN_PREFIX: usize = 50;
/// URL-encoded caret character.
pub const URL_ENCODED_CARAT: &str = "%5E";
/// URL-encoded space character.
pub const URL_ENCODED_SPACE: &str = "%20";
