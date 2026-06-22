//! DataCite to HuWise field mapping rules
use crate::schema::standard::crosswalk::mapping::{FieldMapping, FieldRule};

/// Returns field mapping rules for DataCite → HuWise conversion
pub fn datacite_to_huwise() -> FieldMapping {
    FieldMapping {
        from_schema: "datacite".to_string(),
        to_schema: "huwise".to_string(),
        rules: vec![
            // Simple 1:1 field mappings (inverse of huwise_to_datacite)
            FieldRule {
                source_field: "identifier".to_string(),
                target_field: "identifier".to_string(),
                transform: None,
                required: true,
            },
            FieldRule {
                source_field: "title".to_string(),
                target_field: "title".to_string(),
                transform: None,
                required: true,
            },
            FieldRule {
                source_field: "description".to_string(),
                target_field: "description".to_string(),
                transform: None,
                required: false,
            },
            FieldRule {
                source_field: "creators".to_string(),
                target_field: "creators".to_string(),
                transform: None,
                required: false,
            },
            FieldRule {
                source_field: "contributors".to_string(),
                target_field: "contributors".to_string(),
                transform: None,
                required: false,
            },
            FieldRule {
                source_field: "publication-year".to_string(),
                target_field: "publication-year".to_string(),
                transform: None,
                required: false,
            },
            FieldRule {
                source_field: "resource-type".to_string(),
                target_field: "resource-type".to_string(),
                transform: None,
                required: false,
            },
            FieldRule {
                source_field: "subjects".to_string(),
                target_field: "subjects".to_string(),
                transform: None,
                required: false,
            },
            FieldRule {
                source_field: "language".to_string(),
                target_field: "language".to_string(),
                transform: None,
                required: false,
            },
            FieldRule {
                source_field: "publisher".to_string(),
                target_field: "publisher".to_string(),
                transform: None,
                required: false,
            },
            FieldRule {
                source_field: "license".to_string(),
                target_field: "license".to_string(),
                transform: None,
                required: false,
            },
            FieldRule {
                source_field: "version".to_string(),
                target_field: "version".to_string(),
                transform: None,
                required: false,
            },
        ],
    }
}
