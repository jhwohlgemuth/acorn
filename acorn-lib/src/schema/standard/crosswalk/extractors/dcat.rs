//! DCAT Dataset schema extractor
//!
//! Converts dcat::Dataset into normalized FieldMap representation.
use crate::schema::standard::crosswalk::{FieldMap, FieldValue, SchemaExtractor};
use crate::schema::standard::dcat;

impl SchemaExtractor for dcat::Dataset {
    fn extract_fields(&self) -> FieldMap {
        let mut fields = FieldMap::new();
        if let Some(identifiers) = &self.identifier {
            if !identifiers.is_empty() {
                fields.insert("identifier", FieldValue::StringVec(identifiers.clone()));
            }
        }
        if let Some(titles) = &self.title {
            if !titles.is_empty() {
                fields.insert("title", FieldValue::String(titles[0].clone()));
                if titles.len() > 1 {
                    let alt_titles = titles[1..].to_vec();
                    fields.insert("alternative-titles", FieldValue::StringVec(alt_titles));
                }
            }
        }
        if let Some(descriptions) = &self.description {
            if !descriptions.is_empty() {
                fields.insert("description", FieldValue::String(descriptions[0].clone()));
            }
        }
        if let Some(issued) = &self.issued {
            fields.insert("issued", FieldValue::Date(issued.clone()));
        }
        if let Some(keywords) = &self.keywords {
            fields.insert("keywords", FieldValue::StringVec(keywords.clone()));
        }
        if let Some(themes) = &self.themes {
            fields.insert("themes", FieldValue::StringVec(themes.clone()));
        }
        if let Some(language) = &self.language {
            fields.insert("language", FieldValue::StringVec(language.clone()));
        }
        if let Some(version) = &self.version {
            fields.insert("version", FieldValue::String(version.clone()));
        }
        if let Some(publisher) = &self.publisher {
            if let Some(name) = &publisher.name {
                fields.insert("publisher", FieldValue::String(name.clone()));
            }
        }
        if let Some(creators) = &self.creator {
            let creator_names: Vec<String> = creators.iter().filter_map(|c| c.name.clone()).collect();
            if !creator_names.is_empty() {
                fields.insert("creators", FieldValue::StringVec(creator_names));
            }
        }
        if let Some(license) = &self.license {
            fields.insert("license", FieldValue::IRI(license.clone()));
        }
        if let Some(spatial) = &self.spatial {
            if !spatial.is_empty() {
                if let Some(geometry) = &spatial[0].geometry {
                    fields.insert("spatial", FieldValue::String(geometry.clone()));
                }
            }
        }
        fields
    }
}
