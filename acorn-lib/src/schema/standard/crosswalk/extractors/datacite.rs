//! DataCite Record schema extractor
//!
//! Converts datacite::Record into normalized FieldMap representation.
use crate::schema::standard::crosswalk::{FieldMap, FieldValue, SchemaExtractor};
use crate::schema::standard::datacite;

impl SchemaExtractor for datacite::Record {
    fn extract_fields(&self) -> FieldMap {
        let mut fields = FieldMap::new();
        fields.insert("doi", FieldValue::String(self.attributes.doi.clone()));
        if let Some(titles) = &self.attributes.titles {
            if !titles.is_empty() {
                fields.insert("title", FieldValue::String(titles[0].title.clone()));
                if titles.len() > 1 {
                    let alt_titles: Vec<String> = titles[1..].iter().map(|t| t.title.clone()).collect();
                    fields.insert("alternative-titles", FieldValue::StringVec(alt_titles));
                }
            }
        }
        if let Some(descriptions) = &self.attributes.descriptions {
            if !descriptions.is_empty() {
                fields.insert("description", FieldValue::String(descriptions[0].description.clone()));
            }
        }
        if let Some(language) = &self.attributes.language {
            fields.insert("language", FieldValue::String(language.clone()));
        }
        if let Some(creators) = &self.attributes.creators {
            if !creators.is_empty() {
                let creator_names: Vec<String> = creators.iter().map(|c| c.name.clone()).collect();
                fields.insert("creators", FieldValue::StringVec(creator_names));
            }
        }
        if let Some(publisher) = &self.attributes.publisher {
            fields.insert("publisher", FieldValue::String(publisher.name.clone()));
        }

        if let Some(pub_year) = self.attributes.publication_year {
            fields.insert("publication-year", FieldValue::Number(pub_year as f64));
        }
        if let Some(url) = &self.attributes.url {
            fields.insert("url", FieldValue::IRI(url.clone()));
        }
        if let Some(rights) = &self.attributes.rights_list {
            if !rights.is_empty() {
                if let Some(uri) = &rights[0].rights_uri {
                    fields.insert("license", FieldValue::IRI(uri.clone()));
                }
            }
        }
        if let Some(identifiers) = &self.attributes.alternate_identifiers {
            if !identifiers.is_empty() {
                let alt_ids: Vec<String> = identifiers.iter().map(|id| id.alternate_identifier.clone()).collect();
                fields.insert("alternate-identifiers", FieldValue::StringVec(alt_ids));
            }
        }
        if let Some(subjects) = &self.attributes.subjects {
            if !subjects.is_empty() {
                let subject_strings: Vec<String> = subjects.iter().map(|s| s.subject.clone()).collect();
                fields.insert("subjects", FieldValue::StringVec(subject_strings));
            }
        }
        fields
    }
}
