//! Application constants shared across runtime capability layers.

use crate::util::SemanticVersion;

#[cfg(feature = "alloc")]
pub(crate) const ABSOLUTE_DIFFERENCE_VARIANT: &str = "linear-absolute-difference-cutoff";
/// Canonical application name shared across ACORN crates.
pub const APPLICATION: &str = "acorn";
#[cfg(feature = "alloc")]
pub(crate) const COLOGNE_PHONETIC_VARIANT: &str = "cologne-phonetics-1969";
#[cfg(feature = "alloc")]
pub(crate) const COSINE_VARIANT: &str = "finite-l2-normalized-vector";
#[cfg(feature = "alloc")]
pub(crate) const DAMERAU_LEVENSHTEIN_VARIANT: &str = "unicode-scalar-unrestricted-unit-cost";
#[cfg(feature = "alloc")]
pub(crate) const EXACT_VARIANT: &str = "unicode-scalar-equality";
/// Prefix for an executable Rhai extension ABI marker.
pub const EXEC_HEADER_PREFIX: &str = "// @acorn-exec-v";
#[cfg(feature = "alloc")]
pub(crate) const EXTENDED_JACCARD_VARIANT: &str = "unique-thresholded-best-token-pairs";
#[cfg(feature = "alloc")]
pub(crate) const HAMMING_VARIANT: &str = "unicode-scalar-unit-cost";
#[cfg(feature = "alloc")]
pub(crate) const JARO_VARIANT: &str = "conventional-jaro";
#[cfg(feature = "alloc")]
pub(crate) const JARO_WINKLER_VARIANT: &str = "conventional-four-scalar-prefix";
#[cfg(feature = "alloc")]
pub(crate) const LEVENSHTEIN_VARIANT: &str = "unicode-scalar-unit-cost";
/// Maximum array length produced by executable Rhai extensions.
pub const MAX_EXEC_ARRAY_SIZE: usize = 1024;
/// Maximum executable Rhai extension source size.
pub const MAX_EXEC_BYTES: usize = 512 * 1024;
/// Maximum host capability calls allowed during one extension evaluation.
pub const MAX_EXEC_CAPABILITY_CALLS: usize = 64;
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
/// Maximum number of network requests allowed during one extension evaluation.
pub const MAX_EXEC_NETWORK_REQUESTS: usize = 8;
/// Maximum operations allowed during one executable Rhai extension evaluation.
pub const MAX_EXEC_OPERATIONS: u64 = 100_000;
/// Maximum string length produced by executable Rhai extensions.
pub const MAX_EXEC_STRING_SIZE: usize = 64 * 1024;
/// Maximum total bytes read or proposed by one extension evaluation.
pub const MAX_EXEC_TOTAL_BYTES: usize = 4 * 1024 * 1024;
/// Maximum variables accepted by executable Rhai extensions.
pub const MAX_EXEC_VARIABLES: usize = 256;
/// Maximum wall-clock duration of one extension evaluation.
pub const MAX_EXEC_WALL_TIME_MILLISECONDS: u64 = 30_000;
#[cfg(feature = "alloc")]
pub(crate) const METAPHONE_VARIANT: &str = "original-metaphone-1990";
#[cfg(feature = "alloc")]
pub(crate) const MONGE_ELKAN_VARIANT: &str = "directional-best-right-token-mean";
#[cfg(feature = "alloc")]
pub(crate) const OPTIMAL_STRING_ALIGNMENT_VARIANT: &str = "unicode-scalar-restricted-unit-cost";
#[cfg(feature = "alloc")]
pub(crate) const PERCENTAGE_DIFFERENCE_VARIANT: &str = "symmetric-percentage-difference-cutoff";
#[cfg(feature = "alloc")]
pub(crate) const PREFIX_VARIANT: &str = "unicode-scalar-prefix-equality";
#[cfg(feature = "alloc")]
pub(crate) const SET_VARIANT: &str = "deduplicated-set";
#[cfg(feature = "alloc")]
pub(crate) const SOFT_TFIDF_VARIANT: &str = "two-document-ratio-idf-unique-best-match";
#[cfg(feature = "alloc")]
pub(crate) const SOUNDEX_VARIANT: &str = "american-soundex-four-character";
#[cfg(feature = "alloc")]
pub(crate) const SUFFIX_VARIANT: &str = "unicode-scalar-suffix-equality";
/// Executable Rhai extension ABI supported by this release.
pub const SUPPORTED_EXEC_VERSION: SemanticVersion = SemanticVersion {
    major: 0,
    minor: 2,
    patch: 0,
};
#[cfg(feature = "alloc")]
pub(crate) const TFIDF_COSINE_VARIANT: &str = "two-document-ratio-idf-cosine";
#[cfg(feature = "alloc")]
pub(crate) const WEIGHTED_LEVENSHTEIN_VARIANT: &str = "unicode-scalar-configurable-cost";
