//! HuWise Dataset schema extractor
//!
//! Converts huwise::Dataset into normalized FieldMap representation.
//! Uses block-aware extraction: prioritizes datacite → default → dublin_core → dcat → dcat_ap
use crate::schema::standard::crosswalk::{FieldMap, FieldValue, SchemaExtractor};
use crate::schema::standard::huwise;
use serde_json::Value;

impl SchemaExtractor for huwise::Dataset {
    fn extract_fields(&self) -> FieldMap {
        let mut fields = FieldMap::new();
        fields.insert("identifier", FieldValue::String(self.dataset_id.clone()));
        if let Some(datacite_meta) = &self.metas.datacite {
            extract_from_datacite_block(datacite_meta, &mut fields);
        } else if let Some(default_meta) = &self.metas.r#default {
            extract_from_default_block(default_meta, &mut fields);
        } else if let Some(dc_meta) = &self.metas.dublin_core {
            extract_from_dublin_core_block(dc_meta, &mut fields);
        } else if let Some(dcat_meta) = &self.metas.dcat {
            extract_from_dcat_block(dcat_meta, &mut fields);
        } else if let Some(dcat_ap_meta) = &self.metas.dcat_ap {
            extract_from_dcat_ap_block(dcat_ap_meta, &mut fields);
        }
        fields
    }
}
fn extract_from_datacite_block(meta: &huwise::Datacite, fields: &mut FieldMap) {
    if let Some(title) = &meta.title {
        fields.insert("title", FieldValue::String(title.clone()));
    }
    if let Some(doi) = &meta.identifier {
        fields.insert("doi", FieldValue::String(doi.clone()));
    }
    if let Some(creator) = &meta.creator {
        if let Some(creators) = extract_json_string_array(creator) {
            if !creators.is_empty() {
                fields.insert("creators", FieldValue::StringVec(creators));
            }
        }
    }
    if let Some(pub_year_str) = &meta.publication_year {
        if let Ok(year) = pub_year_str.parse::<f64>() {
            fields.insert("publication-year", FieldValue::Number(year));
        }
    }
    if let Some(description) = &meta.description {
        fields.insert("description", FieldValue::String(description.clone()));
    }
    if let Some(subjects) = &meta.subject {
        if !subjects.is_empty() {
            fields.insert("subjects", FieldValue::StringVec(subjects.clone()));
        }
    }
    if let Some(language) = &meta.language {
        fields.insert("language", FieldValue::String(language.clone()));
    }
    if let Some(publisher) = &meta.publisher {
        fields.insert("publisher", FieldValue::String(publisher.clone()));
    }
    if let Some(resource_type) = &meta.resource_type {
        if let Some(s) = resource_type.as_str() {
            fields.insert("resource-type", FieldValue::String(s.to_string()));
        }
    }
    if let Some(rights) = &meta.rights {
        if let Some(s) = rights.as_str() {
            fields.insert("license", FieldValue::String(s.to_string()));
        }
    }
    if let Some(version) = &meta.version {
        fields.insert("version", FieldValue::String(version.clone()));
    }
}
/// Extract fields from DefaultMeta block (comprehensive fallback)
fn extract_from_default_block(meta: &huwise::DefaultMeta, fields: &mut FieldMap) {
    if let Some(title) = &meta.title {
        fields.insert("title", FieldValue::String(title.clone()));
    }
    if let Some(description) = &meta.description {
        // Strip HTML tags if present
        let clean_desc = strip_html(description);
        fields.insert("description", FieldValue::String(clean_desc));
    }
    if let Some(keywords) = &meta.keyword {
        if !keywords.is_empty() {
            fields.insert("subjects", FieldValue::StringVec(keywords.clone()));
        }
    }
    if let Some(language) = &meta.language {
        fields.insert("language", FieldValue::String(language.clone()));
    }
    if let Some(publisher) = &meta.publisher {
        fields.insert("publisher", FieldValue::String(publisher.clone()));
    }
    if let Some(license) = &meta.license {
        fields.insert("license", FieldValue::String(license.clone()));
    }
    if let Some(modified) = &meta.modified {
        fields.insert("updated", FieldValue::Date(modified.clone()));
    }
}
fn extract_from_dublin_core_block(meta: &huwise::DublinCore, fields: &mut FieldMap) {
    if let Some(title) = &meta.title {
        fields.insert("title", FieldValue::String(title.clone()));
    }
    if let Some(abstract_text) = &meta.abstract_text {
        let clean_desc = strip_html(abstract_text);
        fields.insert("description", FieldValue::String(clean_desc));
    }
    if let Some(creator) = &meta.creator {
        fields.insert("creators", FieldValue::String(creator.clone()));
    }
    if let Some(subject) = &meta.subject {
        if !subject.is_empty() {
            fields.insert("subjects", FieldValue::StringVec(subject.clone()));
        }
    }
    if let Some(language) = &meta.language {
        fields.insert("language", FieldValue::String(language.clone()));
    }
    if let Some(publisher) = &meta.publisher {
        fields.insert("publisher", FieldValue::String(publisher.clone()));
    }
    if let Some(issued) = &meta.issued {
        fields.insert("publication-year", FieldValue::Date(issued.clone()));
    }
    if let Some(license) = &meta.license {
        fields.insert("license", FieldValue::String(license.clone()));
    }
}
fn extract_from_dcat_block(meta: &huwise::Dcat, fields: &mut FieldMap) {
    if let Some(creator) = &meta.creator {
        fields.insert("creators", FieldValue::String(creator.clone()));
    }
    if let Some(issued) = &meta.issued {
        fields.insert("publication-year", FieldValue::Date(issued.clone()));
    }
    if let Some(spatial) = &meta.spatial {
        fields.insert("spatial", FieldValue::String(spatial.clone()));
    }
}
fn extract_from_dcat_ap_block(meta: &huwise::DcatAp, fields: &mut FieldMap) {
    if let Some(title) = &meta.title {
        fields.insert("title", FieldValue::String(title.clone()));
    }
    if let Some(description) = &meta.description {
        let clean_desc = strip_html(description);
        fields.insert("description", FieldValue::String(clean_desc));
    }
    if let Some(keywords) = &meta.keyword {
        if !keywords.is_empty() {
            fields.insert("subjects", FieldValue::StringVec(keywords.clone()));
        }
    }
    if let Some(publisher) = &meta.publisher_name {
        fields.insert("publisher", FieldValue::String(publisher.clone()));
    }
}
fn extract_json_string_array(value: &Value) -> Option<Vec<String>> {
    match value {
        | Value::Array(arr) => {
            let strings: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
            if !strings.is_empty() {
                Some(strings)
            } else {
                None
            }
        }
        | Value::String(s) => Some(vec![s.clone()]),
        | _ => None,
    }
}
// TODO: Probably better way to sanitize HTML
fn strip_html(html: &str) -> String {
    // Remove common HTML tags
    let s = html
        .replace("<p>", "")
        .replace("</p>", " ")
        .replace("<br>", " ")
        .replace("<br/>", " ")
        .replace("<strong>", "")
        .replace("</strong>", "")
        .replace("<em>", "")
        .replace("</em>", "")
        .replace("<b>", "")
        .replace("</b>", "")
        .replace("<i>", "")
        .replace("</i>", "");
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
