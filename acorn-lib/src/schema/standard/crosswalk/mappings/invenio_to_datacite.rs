//! Field mapping rules for Invenio → DataCite conversion
//!
//! Defines field-level transformations from Invenio Record to DataCite Record.
use crate::schema::standard::crosswalk::{mapping::transforms::year_to_date, FieldMapping, FieldRule};

/// Create field mapping rules from Invenio to DataCite
pub fn invenio_to_datacite() -> FieldMapping {
    FieldMapping::new("invenio", "datacite")
        .add_rule(FieldRule::required("identifier", "doi"))
        .add_rule(FieldRule::map("title", "title"))
        .add_rule(FieldRule::map("description", "description"))
        .add_rule(FieldRule::map("language", "language"))
        .add_rule(FieldRule::map("creators", "creators"))
        .add_rule(FieldRule::map("publication-year", "publication-year").with_transform(year_to_date))
        .add_rule(FieldRule::map("resource-type", "resource-type"))
        .add_rule(FieldRule::map("license", "license"))
        .add_rule(FieldRule::map("alternate-identifiers", "alternate-identifiers"))
        .add_rule(FieldRule::map("subjects", "subjects"))
        .add_rule(FieldRule::map("contributors", "contributors"))
        .add_rule(FieldRule::map("created", "dates"))
        .add_rule(FieldRule::map("updated", "dates"))
}
