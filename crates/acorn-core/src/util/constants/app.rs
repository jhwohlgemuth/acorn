//! Application constants shared across runtime capability layers.

use crate::util::SemanticVersion;

/// Prefix for an executable Rhai extension ABI marker.
pub const EXEC_HEADER_PREFIX: &str = "// @acorn-exec-v";
/// Maximum executable Rhai extension source size.
pub const MAX_EXEC_BYTES: usize = 512 * 1024;
/// Executable Rhai extension ABI supported by this release.
pub const SUPPORTED_EXEC_VERSION: SemanticVersion = SemanticVersion {
    major: 0,
    minor: 2,
    patch: 0,
};
