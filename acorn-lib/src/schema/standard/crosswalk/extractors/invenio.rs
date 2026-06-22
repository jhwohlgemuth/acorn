//! Invenio Record schema extractor
//!
//! Converts invenio::Record into normalized FieldMap representation.
use crate::schema::standard::crosswalk::{FieldMap, FieldValue, SchemaExtractor};
use crate::schema::standard::invenio;

impl SchemaExtractor for invenio::Record {
    fn extract_fields(&self) -> FieldMap {
        let mut fields = FieldMap::new();
        if let Some(pid) = &self.pid {
            if let Some(pk) = pid.pk {
                fields.insert("identifier", FieldValue::IRI(pk.to_string()));
            }
        }
        if fields.is_empty() && self.id.is_some() {
            fields.insert("identifier", FieldValue::IRI(self.id.clone().unwrap_or_default()));
        }
        if let Some(metadata) = &self.metadata {
            // Title (required in Invenio)
            if let Some(title) = &metadata.title {
                fields.insert("title", FieldValue::String(title.clone()));
            }
            if let Some(creators) = &metadata.creators {
                if !creators.is_empty() {
                    let creator_names: Vec<String> = creators.iter().filter_map(|c| c.person_or_org.name.clone()).collect();
                    if !creator_names.is_empty() {
                        fields.insert("creators", FieldValue::StringVec(creator_names));
                    }
                }
            }
            if let Some(pub_date) = &metadata.publication_date {
                if let Some(year_str) = pub_date.split('-').next() {
                    if let Ok(year) = year_str.parse::<f64>() {
                        fields.insert("publication-year", FieldValue::Number(year));
                    }
                }
            }
            if let Some(resource_type) = &metadata.resource_type {
                if let Some(id) = &resource_type.id {
                    fields.insert("resource-type", FieldValue::String(id.clone()));
                }
            }
            if let Some(description) = &metadata.description {
                fields.insert("description", FieldValue::String(description.clone()));
            }
            if let Some(languages) = &metadata.languages {
                if !languages.is_empty() {
                    fields.insert("language", FieldValue::String(languages[0].id.clone()));
                }
            }
            if let Some(identifiers) = &metadata.identifiers {
                if !identifiers.is_empty() {
                    let id_strings: Vec<String> = identifiers.iter().map(|id| id.identifier.clone()).collect();
                    fields.insert("alternate-identifiers", FieldValue::StringVec(id_strings));
                }
            }
            if let Some(subjects) = &metadata.subjects {
                if !subjects.is_empty() {
                    let subject_strings: Vec<String> = subjects.iter().filter_map(|s| s.subject.clone()).collect();
                    if !subject_strings.is_empty() {
                        fields.insert("subjects", FieldValue::StringVec(subject_strings));
                    }
                }
            }
            if let Some(rights) = &metadata.rights {
                if !rights.is_empty() {
                    if let Some(link) = &rights[0].link {
                        fields.insert("license", FieldValue::IRI(link.clone()));
                    }
                }
            }
            if let Some(contributors) = &metadata.contributors {
                if !contributors.is_empty() {
                    let contributor_names: Vec<String> = contributors.iter().filter_map(|c| c.person_or_org.name.clone()).collect();
                    if !contributor_names.is_empty() {
                        fields.insert("contributors", FieldValue::StringVec(contributor_names));
                    }
                }
            }
        }
        if let Some(created) = &self.created {
            fields.insert("created", FieldValue::Date(created.clone()));
        }
        if let Some(updated) = &self.updated {
            fields.insert("updated", FieldValue::Date(updated.clone()));
        }
        fields
    }
}
