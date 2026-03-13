//! # Schema constants
//!
use lazy_static::lazy_static;
use sophia::api::ns::Namespace;
/// Default schema URI for ORCiD values
pub const DEFAULT_ORCID_SCHEMA_URI: &str = "https://orcid.org";
/// Default schema URI for ROR values
pub const DEFAULT_ROR_SCHEMA_URI: &str = "https://ror.org/";
/// See <https://datacite-metadata-schema.readthedocs.io/en/4.5/properties/identifier/#a-identifiertype>
pub const DATACITE_IDENTIFIER_TYPE_CONTROLLED_VOCABULARY: &[&str; 1] = &["DOI"];
lazy_static! {
    /// RDF namespace for [CodeMeta](https://codemeta.github.io/) schema terms
    pub static ref CODEMETA: Namespace<&'static str> = Namespace::new_unchecked("https://codemeta.github.io/terms/");
    /// RDF namespace for [DCAT](https://www.w3.org/TR/vocab-dcat-3/) v3
    pub static ref DCAT: Namespace<&'static str> = Namespace::new_unchecked("http://www.w3.org/ns/dcat#");
    /// RDF namespace for [schema.org](https://schema.org/)
    pub static ref SCHEMA_ORG: Namespace<&'static str> = Namespace::new_unchecked("https://schema.org/");
    /// RDF namespace for [ORCiD](https://orcid.org/)
    pub static ref ORCID: Namespace<&'static str> = Namespace::new_unchecked("https://orcid.org/");
    /// RDF namespace for [ROR](https://ror.org/)
    pub static ref ROR: Namespace<&'static str> = Namespace::new_unchecked("https://ror.org/");
}
/// Resolve a schema.org term to its full IRI string
pub fn schema_org(suffix: &str) -> String {
    SCHEMA_ORG.get_unchecked(suffix).to_string()
}
/// Resolve a CodeMeta term to its full IRI string
pub fn codemeta(suffix: &str) -> String {
    CODEMETA.get_unchecked(suffix).to_string()
}
/// Resolve a DCAT term to its full IRI string
pub fn dcat(suffix: &str) -> String {
    DCAT.get_unchecked(suffix).to_string()
}
