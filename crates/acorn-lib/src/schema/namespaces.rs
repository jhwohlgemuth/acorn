//! # Schema constants
//!
use crate::prelude::*;
use lazy_static::lazy_static;
use sophia::api::ns::Namespace;
/// Default schema URI for ORCiD values
pub const DEFAULT_ORCID_SCHEMA_URI: &str = "https://orcid.org";
/// Default schema URI for RAiD values
pub const DEFAULT_RAID_SCHEMA_URI: &str = "https://raid.org";
/// Default schema URI for ROR values
pub const DEFAULT_ROR_SCHEMA_URI: &str = "https://ror.org/";
/// See <https://datacite-metadata-schema.readthedocs.io/en/4.5/properties/identifier/#a-identifiertype>
pub const DATACITE_IDENTIFIER_TYPE_CONTROLLED_VOCABULARY: &[&str; 1] = &["DOI"];
lazy_static! {
    /// RDF namespace for bibliographic ontology ([BIBO](http://purl.org/ontology/bibo/))
    ///
    /// Describes bibliographic things on the semantic Web in RDF.
    /// This ontology can be used as a citation ontology, as a document classification ontology, or simply as a way to describe any kind of document in RDF.
    pub static ref BIBO: Namespace<&'static str> = Namespace::new_unchecked("http://purl.org/ontology/bibo/");
    /// RDF namespace for [CodeMeta](https://codemeta.github.io/) schema terms
    pub static ref CODEMETA: Namespace<&'static str> = Namespace::new_unchecked("https://codemeta.github.io/terms/");
    /// RDF namespace for [DCAT](https://www.w3.org/TR/vocab-dcat-3/) v3
    pub static ref DCAT: Namespace<&'static str> = Namespace::new_unchecked("http://www.w3.org/ns/dcat#");
    /// RDF namespace for [Dublin Core Terms](https://www.dublincore.org/specifications/dublin-core/dcmi-terms/)
    pub static ref DCTERMS: Namespace<&'static str> = Namespace::new_unchecked("http://purl.org/dc/terms/");
    /// RDF namespace for Friend of a Friend ([FOAF](http://xmlns.com/foaf/0.1/))
    ///
    /// A project devoted to linking people and information using the Web.
    /// It provides a way to describe people, their activities and their relations to other people and objects.
    pub static ref FOAF: Namespace<&'static str> = Namespace::new_unchecked("http://xmlns.com/foaf/0.1/");
    /// RDF namespace for [ORCiD](https://orcid.org/)
    pub static ref ORCID: Namespace<&'static str> = Namespace::new_unchecked("https://orcid.org/");
    /// RDF namespace for [ROR](https://ror.org/)
    pub static ref ROR: Namespace<&'static str> = Namespace::new_unchecked("https://ror.org/");
    /// RDF namespace for [schema.org](https://schema.org/)
    pub static ref SCHEMA_ORG: Namespace<&'static str> = Namespace::new_unchecked("https://schema.org/");
}
/// Resolve a BIBO term to its full IRI string
pub fn bibo(suffix: &str) -> String {
    BIBO.get_unchecked(suffix).to_string()
}
/// Resolve a CodeMeta term to its full IRI string
pub fn codemeta(suffix: &str) -> String {
    CODEMETA.get_unchecked(suffix).to_string()
}
/// Resolve a DCAT term to its full IRI string
pub fn dcat(suffix: &str) -> String {
    DCAT.get_unchecked(suffix).to_string()
}
/// Resolve a Dublin Core term to its full IRI string
pub fn dcterms(suffix: &str) -> String {
    DCTERMS.get_unchecked(suffix).to_string()
}
/// Resolve a FOAF term to its full IRI string
pub fn foaf(suffix: &str) -> String {
    FOAF.get_unchecked(suffix).to_string()
}
/// Resolve a schema.org term to its full IRI string
pub fn schema_org(suffix: &str) -> String {
    SCHEMA_ORG.get_unchecked(suffix).to_string()
}
