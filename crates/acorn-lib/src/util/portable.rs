//! Portable utility surface intended for the future `acorn-core` crate
pub use super::{
    base32_crockford_decode, base32_crockford_encode, constant_time_eq, contains_any, contains_any_with_prefix, detect_json, detect_xml,
    file_extension, find_first, format_bytes, frontmatter_and_body, glob_matches, merge, regex_capture_lookup, regex_inverse, regex_join,
    regex_to_glob, snake_case, strip_suffixes, suffix, to_ascii_alphanumeric, to_rfc3339, to_string, Checksum, ChecksumAlgorithm, License,
    LinkedData, MimeType, Searchable, SemanticVersion, StringInterpolation, ToMarkdown, ToProse, ToStringChunks, Unstructured,
};
