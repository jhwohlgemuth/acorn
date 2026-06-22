//! DataCite Record schema builder
//!
//! Reconstructs datacite::Record from normalized FieldMap representation.
use crate::schema::standard::crosswalk::{CrosswalkError, FieldMap, SchemaBuilder};
use crate::schema::standard::datacite;

impl SchemaBuilder for datacite::Record {
    fn build_from_fields(fields: &FieldMap) -> Result<Self, CrosswalkError> {
        let doi = fields.get_string("doi")?;
        let mut titles = None;
        if let Some(title_str) = fields.get_string_opt("title") {
            titles = Some(vec![datacite::Title {
                title: title_str,
                title_type: None,
            }]);
        }
        let mut descriptions = None;
        if let Some(desc_str) = fields.get_string_opt("description") {
            descriptions = Some(vec![datacite::Description {
                description: desc_str,
                description_type: None,
                language: None,
            }]);
        }
        let language = fields.get_string_opt("language");
        let mut creators = None;
        if let Some(creator_names) = fields.get_string_vec_opt("creators") {
            creators = Some(
                creator_names
                    .into_iter()
                    .map(|name| datacite::Creator {
                        name,
                        name_type: None,
                        given_name: None,
                        family_name: None,
                        name_identifiers: None,
                        affiliation: None,
                    })
                    .collect(),
            );
        }
        let mut publisher = None;
        if let Some(pub_name) = fields.get_string_opt("publisher") {
            publisher = Some(datacite::Publisher {
                name: pub_name,
                publisher_identifier: None,
                publisher_identifier_scheme: None,
                scheme_uri: None,
            });
        }
        let publication_year = fields.get_number_opt("publication-year").map(|n| n as i32);
        let mut attributes = datacite::Attributes {
            doi,
            event: None,
            titles,
            creators,
            publisher,
            publication_year,
            resource_types: None,
            url: fields.get_iri_opt("url"),
            subjects: None,
            contributors: None,
            dates: None,
            language,
            alternate_identifiers: None,
            related_identifiers: None,
            sizes: None,
            formats: None,
            version: fields.get_string_opt("version"),
            rights_list: None,
            descriptions,
            geo_locations: None,
            funding_references: None,
            schema_version: None,
        };
        if let Some(license_iri) = fields.get_iri_opt("license") {
            attributes.rights_list = Some(vec![datacite::Rights {
                rights: None,
                rights_uri: Some(license_iri),
                rights_identifier: None,
                rights_identifier_scheme: None,
                scheme_uri: None,
            }]);
        }
        Ok(datacite::Record {
            id: attributes.doi.clone(),
            kind: "dois".to_string(),
            attributes,
        })
    }
}
