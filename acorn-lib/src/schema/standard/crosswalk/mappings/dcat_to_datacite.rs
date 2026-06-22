//! DCAT → DataCite field mapping
//!
//! Maps DCAT Dataset fields to DataCite Record fields.
//!
//! # Mapping Strategy
//!
//! - identifier (first or first DOI-like) → doi
//! - identifier (rest) → alternate_identifiers
//! - title → titles
//! - description → descriptions
//! - creator → creators (as Creator)
//! - publisher → publisher (as Publisher)
//! - issued → publication_year (extract year from ISO 8601)
//! - type_ → resource_types.general (map IRI back to enum)
//! - keywords → subjects (as uncontrolled keywords)
//! - themes → subjects (as controlled keywords/themes)
//! - license → rights_list (as rights_uri)
//! - language → language
//!
//! # Data Loss
//!
//! Fields with no DataCite equivalent:
//! - frequency (DCAT accrual periodicity)
//! - access_rights (different from DataCite rights)
//! - distribution (DCAT feature; DataCite lacks this)
//! - spatial/temporal (limited support; no dedicated fields)
//! - was_generated_by (PROV provenance)
use crate::schema::standard::crosswalk::{mapping::transforms::{extract_year, iri_to_resource_type}, FieldMapping, FieldRule};

pub fn dcat_to_datacite() -> FieldMapping {
    FieldMapping::new("dcat", "datacite")
        .with_rule(FieldRule::new("identifier", "doi").required())
        .with_rule(FieldRule::new("title", "title"))
        .with_rule(FieldRule::new("description", "description"))
        .with_rule(FieldRule::new("language", "language"))
        .with_rule(FieldRule::new("creators", "creators"))
        .with_rule(FieldRule::new("publisher", "publisher"))
        .with_rule(FieldRule::new("issued", "publication-year").with_transform(extract_year))
        .with_rule(FieldRule::new("license", "license"))
        .with_rule(FieldRule::new("type_", "resource-type").with_transform(iri_to_resource_type))
        .with_rule(FieldRule::new("keywords", "subjects"))
        .with_rule(FieldRule::new("themes", "subjects"))
}
