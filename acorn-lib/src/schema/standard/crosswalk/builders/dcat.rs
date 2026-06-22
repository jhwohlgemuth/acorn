//! DCAT Dataset schema builder
//!
//! Reconstructs dcat::Dataset from normalized FieldMap representation.
use crate::schema::standard::crosswalk::{CrosswalkError, FieldMap, SchemaBuilder};
use crate::schema::standard::dcat;

impl SchemaBuilder for dcat::Dataset {
    fn build_from_fields(fields: &FieldMap) -> Result<Self, CrosswalkError> {
        let identifier = fields.get_string_vec_opt("identifier");
        let mut title = None;
        if let Some(title_str) = fields.get_string_opt("title") {
            title = Some(vec![title_str]);
        }
        let mut description = None;
        if let Some(desc_str) = fields.get_string_opt("description") {
            description = Some(vec![desc_str]);
        }
        let issued = fields.get_date_opt("issued");
        let language = fields.get_string_vec_opt("language");
        let keywords = fields.get_string_vec_opt("keywords");
        let themes = fields.get_string_vec_opt("themes");
        let version = fields.get_string_opt("version");
        let mut publisher = None;
        if let Some(pub_name) = fields.get_string_opt("publisher") {
            publisher = Some(dcat::Agent {
                name: Some(pub_name),
                homepage: None,
                email: None,
                identifier: None,
            });
        }
        let mut creator = None;
        if let Some(creator_names) = fields.get_string_vec_opt("creators") {
            creator = Some(
                creator_names
                    .into_iter()
                    .map(|name| dcat::Agent {
                        name: Some(name),
                        homepage: None,
                        email: None,
                        identifier: None,
                    })
                    .collect(),
            );
        }
        let license = fields.get_iri_opt("license");
        let mut spatial = None;
        if let Some(spatial_str) = fields.get_string_opt("spatial") {
            spatial = Some(vec![dcat::Location {
                geometry: Some(spatial_str),
                bbox: None,
                centroid: None,
            }]);
        }
        Ok(dcat::Dataset {
            title,
            description,
            identifier,
            issued,
            modified: None,
            language,
            publisher,
            creator,
            contact_point: None,
            keywords,
            themes,
            license,
            rights: None,
            access_rights: None,
            has_policy: None,
            conforms_to: None,
            landing_page: None,
            relation: None,
            type_: None,
            version,
            version_notes: None,
            previous_version: None,
            has_version: None,
            has_current_version: None,
            replaces: None,
            status: None,
            is_referenced_by: None,
            has_part: None,
            qualified_relation: None,
            first: None,
            last: None,
            previous: None,
            distribution: None,
            frequency: None,
            in_series: None,
            spatial,
            spatial_resolution_in_meters: None,
            temporal: None,
            temporal_resolution: None,
            was_generated_by: None,
        })
    }
}
