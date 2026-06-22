//! DataCite → DCAT field mapping
//!
//! Maps DataCite Record fields to DCAT Dataset fields.
//!
//! # Mapping Strategy
//!
//! - DOI → identifier (primary identifier)
//! - titles → title (first title only; alternatives preserved in warning)
//! - descriptions → description
//! - creators → creator (as Agent)
//! - publisher → publisher (as Agent)
//! - publication_year → issued (converted to ISO 8601 date)
//! - resource_types.general → type_ (mapped to RDF IRI)
//! - subjects → keywords (preserved without vocab info)
//! - license → license
//! - url → landing_page
//!
//! # Data Loss
//!
//! Fields with no DCAT equivalent:
//! - sizes, formats (DCAT Distribution-level, not Dataset)
//! - contributors (DCAT lacks contributor types)
//! - funding_references (no DCAT equivalent)
//! - dates (complex date types don't map to single issued date)
use crate::schema::standard::crosswalk::{mapping::transforms::{resource_type_to_iri, year_to_date}, FieldMapping, FieldRule};

pub fn datacite_to_dcat() -> FieldMapping {
    FieldMapping::new("datacite", "dcat")
        .with_rule(FieldRule::new("doi", "identifier").required())
        .with_rule(FieldRule::new("title", "title"))
        .with_rule(FieldRule::new("description", "description"))
        .with_rule(FieldRule::new("language", "language"))
        .with_rule(FieldRule::new("creators", "creators"))
        .with_rule(FieldRule::new("publisher", "publisher"))
        .with_rule(FieldRule::new("publication-year", "issued").with_transform(year_to_date))
        .with_rule(FieldRule::new("license", "license"))
        .with_rule(FieldRule::new("url", "landing_page"))
        .with_rule(FieldRule::new("subjects", "keywords"))
        .with_rule(FieldRule::new("resource-type", "type_").with_transform(resource_type_to_iri))
}
