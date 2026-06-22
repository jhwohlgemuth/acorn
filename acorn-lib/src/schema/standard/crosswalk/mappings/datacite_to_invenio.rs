//! Field mapping rules for DataCite → Invenio conversion
//!
//! Defines field-level transformations from DataCite Record to Invenio Record.
use crate::schema::standard::crosswalk::{mapping::transforms::extract_year, FieldMapping, FieldRule};

/// Create field mapping rules from DataCite to Invenio
pub fn datacite_to_invenio() -> FieldMapping {
    FieldMapping::new("datacite", "invenio")
        .add_rule(FieldRule::required("doi", "identifier"))
        .add_rule(FieldRule::map("title", "title"))
        .add_rule(FieldRule::map("description", "description"))
        .add_rule(FieldRule::map("language", "language"))
        .add_rule(FieldRule::map("creators", "creators"))
        .add_rule(FieldRule::map("publication-year", "publication-year").with_transform(extract_year))
        .add_rule(FieldRule::map("resource-type", "resource-type"))
        .add_rule(FieldRule::map("license", "license"))
        .add_rule(FieldRule::map("alternate-identifiers", "alternate-identifiers"))
        .add_rule(FieldRule::map("subjects", "subjects"))
        .add_rule(FieldRule::map("contributors", "contributors"))
        .add_rule(FieldRule::map("version", "version"))
}
