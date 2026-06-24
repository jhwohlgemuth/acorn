//! HuWise to DataCite field mapping rules
use crate::schema::standard::crosswalk::{FieldMapping, FieldRule};

/// Returns field mapping rules for HuWise → DataCite conversion
pub fn huwise_to_datacite() -> FieldMapping {
    FieldMapping::new("huwise", "datacite")
        .with_rule(FieldRule::required_field("doi", "doi"))
        .with_rule(FieldRule::map("title", "title"))
        .with_rule(FieldRule::map("description", "description"))
        .with_rule(FieldRule::map("creators", "creators"))
        .with_rule(FieldRule::map("contributors", "contributors"))
        .with_rule(FieldRule::map("publication-year", "publication-year"))
        .with_rule(FieldRule::map("resource-type", "resource-type"))
        .with_rule(FieldRule::map("subjects", "subjects"))
        .with_rule(FieldRule::map("language", "language"))
        .with_rule(FieldRule::map("publisher", "publisher"))
        .with_rule(FieldRule::map("license", "license"))
        .with_rule(FieldRule::map("version", "version"))
}
