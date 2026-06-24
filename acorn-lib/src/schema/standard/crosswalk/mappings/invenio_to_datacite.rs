//! Field mapping rules for Invenio → DataCite conversion
//!
//! Defines field-level transformations from Invenio Record to DataCite Record.
use crate::schema::standard::crosswalk::{FieldMapping, FieldRule};

/// Create field mapping rules from Invenio to DataCite
pub fn invenio_to_datacite() -> FieldMapping {
    FieldMapping::new("invenio", "datacite")
        .with_rule(FieldRule::required_field("identifier", "doi"))
        .with_rule(FieldRule::map("title", "title"))
        .with_rule(FieldRule::map("description", "description"))
        .with_rule(FieldRule::map("language", "language"))
        .with_rule(FieldRule::map("creators", "creators"))
        .with_rule(FieldRule::map("publication-year", "publication-year"))
        .with_rule(FieldRule::map("resource-type", "resource-type"))
        .with_rule(FieldRule::map("license", "license"))
        .with_rule(FieldRule::map("alternate-identifiers", "alternate-identifiers"))
        .with_rule(FieldRule::map("subjects", "subjects"))
        .with_rule(FieldRule::map("contributors", "contributors"))
}
