//! # Schema constants
//!
/// Base URL for CodeMeta schema terms
pub const CODEMETA_CONTEXT: &str = "https://codemeta.github.io/terms";
/// Base URL for DCAT v3
pub const DCAT_CONTEXT: &str = "http://www.w3.org/ns/dcat#";
/// Default schema URI for ORCiD values
pub const DEFAULT_ORCID_SCHEMA_URI: &str = "https://orcid.org";
/// Default schema URI for ROR values
pub const DEFAULT_ROR_SCHEMA_URI: &str = "https://ror.org/";
/// Base URL for schema.org
pub const SCHEMA_ORG_CONTEXT: &str = "https://schema.org";
/// See <https://datacite-metadata-schema.readthedocs.io/en/4.5/properties/identifier/#a-identifiertype>
pub const DATACITE_IDENTIFIER_TYPE_CONTROLLED_VOCABULARY: &[&str; 1] = &["DOI"];
