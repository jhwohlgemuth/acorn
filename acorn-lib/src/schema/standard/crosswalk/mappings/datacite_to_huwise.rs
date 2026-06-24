//! DataCite to HuWise field mapping rules
use crate::schema::standard::crosswalk::{FieldMapping, FieldRule};

/// Returns field mapping rules for DataCite → HuWise conversion
pub fn datacite_to_huwise() -> FieldMapping {
    FieldMapping::new("datacite", "huwise")
        .with_rule(FieldRule::required_field("doi", "identifier"))
        .with_rule(FieldRule::map("title", "title"))
        .with_rule(FieldRule::map("description", "description"))
        .with_rule(FieldRule::map("creators", "creators"))
        .with_rule(FieldRule::map("publication-year", "publication-year"))
        .with_rule(FieldRule::map("resource-type", "resource-type"))
        .with_rule(FieldRule::map("subjects", "subjects"))
        .with_rule(FieldRule::map("language", "language"))
        .with_rule(FieldRule::map("publisher", "publisher"))
        .with_rule(FieldRule::map("license", "license"))
        .with_rule(FieldRule::map("version", "version"))
}
