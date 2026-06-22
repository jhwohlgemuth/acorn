//! HuWise Dataset schema builder
//!
//! Reconstructs huwise::Dataset from normalized FieldMap representation.
//! Uses Option B strategy: populates only the datacite block as primary target.
use crate::schema::standard::crosswalk::{CrosswalkError, FieldMap, SchemaBuilder};
use crate::schema::standard::huwise;
use serde_json::Value;

impl SchemaBuilder for huwise::Dataset {
    fn build_from_fields(fields: &FieldMap) -> Result<Self, CrosswalkError> {
        let dataset_id = fields.get_string("identifier")?;
        let datacite_meta = Some(build_datacite_block(fields));
        let meta = huwise::Meta {
            datacite: datacite_meta,
            r#default: None,
            dublin_core: None,
            dcat: None,
            dcat_ap: None,
            custom_template: None,
        };
        Ok(huwise::Dataset {
            dataset_id,
            has_attachments: false,
            attachments_count: 0,
            has_records: false,
            fields: Value::Array(vec![]),
            metas: meta,
            features: vec![],
        })
    }
}
fn build_datacite_block(fields: &FieldMap) -> huwise::Datacite {
    let title = fields.get_string_opt("title");
    let identifier = fields.get_string_opt("identifier");
    let creator = fields
        .get_string_vec_opt("creators")
        .map(|names| Value::Array(names.into_iter().map(Value::String).collect()));
    let publication_year = fields.get_number_opt("publication-year").map(|y| (y as i32).to_string());
    let description = fields.get_string_opt("description");
    let subject = fields.get_string_vec_opt("subjects");
    let language = fields.get_string_opt("language");
    let publisher = fields.get_string_opt("publisher");
    let resource_type = fields.get_string_opt("resource-type").map(Value::String);
    let rights = fields.get_string_opt("license").map(Value::String);
    let version = fields.get_string_opt("version");
    huwise::Datacite {
        identifier,
        title,
        alternative_title: None,
        publisher,
        creator,
        publication_year,
        date: None,
        subject,
        contributor: None,
        description,
        language,
        resource_type,
        alternate_identifier: None,
        related_identifier: None,
        size: None,
        format: None,
        version,
        rights,
        geolocation: None,
    }
}
