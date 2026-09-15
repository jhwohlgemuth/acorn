//! Application constants shared across runtime capability layers.

use crate::util::SemanticVersion;

/// Prefix for an executable Rhai extension ABI marker.
pub const EXEC_HEADER_PREFIX: &str = "// @acorn-exec-v";
/// Maximum array length produced by executable Rhai extensions.
pub const MAX_EXEC_ARRAY_SIZE: usize = 1024;
/// Maximum executable Rhai extension source size.
pub const MAX_EXEC_BYTES: usize = 512 * 1024;
/// Maximum nested expression depth accepted by executable Rhai extensions.
pub const MAX_EXEC_EXPR_DEPTH: usize = 64;
/// Maximum function-call depth accepted by executable Rhai extensions.
pub const MAX_EXEC_FUNCTION_DEPTH: usize = 32;
/// Maximum script-defined functions accepted by executable Rhai extensions.
pub const MAX_EXEC_FUNCTIONS: usize = 64;
/// Maximum interned strings accepted by executable Rhai extensions.
pub const MAX_EXEC_INTERNED_STRINGS: usize = 1024;
/// Maximum map length produced by executable Rhai extensions.
pub const MAX_EXEC_MAP_SIZE: usize = 1024;
/// Maximum imported modules accepted by executable Rhai extensions.
pub const MAX_EXEC_MODULES: usize = 0;
/// Maximum operations allowed during one executable Rhai extension evaluation.
pub const MAX_EXEC_OPERATIONS: u64 = 100_000;
/// Maximum successful extensionless Rhai probes retained in memory.
pub const MAX_EXEC_PROBE_CACHE_ENTRIES: usize = 64;
/// Maximum string length produced by executable Rhai extensions.
pub const MAX_EXEC_STRING_SIZE: usize = 64 * 1024;
/// Maximum variables accepted by executable Rhai extensions.
pub const MAX_EXEC_VARIABLES: usize = 256;
/// Executable Rhai extension ABI supported by this release.
pub const SUPPORTED_EXEC_VERSION: SemanticVersion = SemanticVersion {
    major: 0,
    minor: 2,
    patch: 0,
};
