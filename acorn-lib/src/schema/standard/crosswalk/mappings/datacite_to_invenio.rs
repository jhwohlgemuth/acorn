//! Field mapping rules for DataCite → Invenio conversion
//!
//! Defines field-level transformations from DataCite Record to Invenio Record.
use crate::schema::standard::crosswalk::{mapping::transforms::extract_year, FieldMapping, FieldRule};

/// Create field mapping rules from DataCite to Invenio
pub fn datacite_to_invenio() -> FieldMapping {
    FieldMapping::new("datacite", "invenio")
        .with_rule(FieldRule::required_field("doi", "identifier"))
        .with_rule(FieldRule::map("title", "title"))
        .with_rule(FieldRule::map("description", "description"))
        .with_rule(FieldRule::map("language", "language"))
        .with_rule(FieldRule::map("creators", "creators"))
        .with_rule(FieldRule::map("publication-year", "publication-year").with_transform(extract_year))
        .with_rule(FieldRule::map("resource-type", "resource-type"))
        .with_rule(FieldRule::map("license", "license"))
        .with_rule(FieldRule::map("alternate-identifiers", "alternate-identifiers"))
        .with_rule(FieldRule::map("subjects", "subjects"))
        .with_rule(FieldRule::map("contributors", "contributors"))
        .with_rule(FieldRule::map("version", "version"))
}
