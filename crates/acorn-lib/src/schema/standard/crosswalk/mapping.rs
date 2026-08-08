//! Field transformation rules for schema conversions
//!
//! Defines how fields are mapped from one schema to another,
//! including transformation logic and rule composition.
use super::CrosswalkError;
use crate::prelude::HashMap;
use crate::prelude::*;
use core::fmt;
use serde_json::Value;

/// A transformation function that maps one FieldValue to another
///
/// Used to normalize field values during conversion (e.g., convert year to ISO 8601 date).
pub type FieldTransform = fn(FieldValue) -> Result<FieldValue, String>;
/// Normalized representation of a metadata field
///
/// Covers the most common metadata types: strings, collections, dates, IRIs, and objects.
#[derive(Clone, Debug)]
pub enum FieldValue {
    /// Single text value
    String(String),
    /// Multiple text values
    StringVec(Vec<String>),
    /// ISO 8601 date or datetime
    Date(String),
    /// URI or IRI (distinguished for semantic clarity)
    IRI(String),
    /// Numeric value (for byte sizes, years, etc.)
    Number(f64),
    /// Nested object (recursive field map)
    Object(Box<Fields>),
    /// List of objects
    ObjectVec(Vec<Fields>),
    /// Fallback for untyped values (JSON)
    Json(Value),
}
/// A complete mapping specification from one schema to another
///
/// Contains all field rules and metadata about the conversion.
#[derive(Clone)]
pub struct FieldMapping {
    /// Source schema identifier (e.g., "datacite", "dcat")
    pub source: &'static str,
    /// Target schema identifier
    pub target: &'static str,
    /// All mapping rules for this conversion
    pub rules: Vec<FieldRule>,
}
/// A single field mapping rule from source to target schema
///
/// Supports one-to-many mappings (one source field → multiple target fields)
/// via multiple rules with the same source.
#[derive(Clone)]
pub struct FieldRule {
    /// Source field name (kebab-case)
    pub source: &'static str,
    /// Target field name(s); if "field1|field2", apply to both
    pub target: &'static str,
    /// Optional transformation function
    pub transform: Option<FieldTransform>,
    /// Whether this field is required in target schema
    pub required: bool,
}
impl From<HashMap<String, FieldValue>> for Fields {
    fn from(fields: HashMap<String, FieldValue>) -> Self {
        Self { fields }
    }
}
impl Fields {
    /// Create a new empty field map
    pub fn new() -> Self {
        Self { fields: HashMap::new() }
    }
    /// Insert a field value
    pub fn insert(&mut self, key: impl Into<String>, value: FieldValue) {
        self.fields.insert(key.into(), value);
    }
    /// Get a field value by key
    pub fn get(&self, key: &str) -> Option<&FieldValue> {
        self.fields.get(key)
    }
    /// Remove a field by key, returning the value if present
    pub fn remove(&mut self, key: &str) -> Option<FieldValue> {
        self.fields.remove(key)
    }
    /// Check if a field exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }
    /// Get all keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.fields.keys()
    }
    /// Get all key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&String, &FieldValue)> {
        self.fields.iter()
    }
    /// Check if field map is empty
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
    /// Number of fields
    pub fn len(&self) -> usize {
        self.fields.len()
    }
    /// Get string field, returning error with context if missing or wrong type
    pub fn get_string(&self, key: &str) -> Result<String, String> {
        self.get(key)
            .and_then(|v| v.as_string().map(|s| s.to_string()))
            .ok_or_else(|| format!("expected string field '{key}'"))
    }
    /// Get optional string field
    pub fn get_string_opt(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.as_string().map(|s| s.to_string()))
    }
    /// Get string vector field
    pub fn get_string_vec(&self, key: &str) -> Result<Vec<String>, String> {
        self.get(key)
            .and_then(|v| v.as_string_vec().map(|sv| sv.to_vec()))
            .ok_or_else(|| format!("expected string vector field '{key}'"))
    }
    /// Get optional string vector field
    pub fn get_string_vec_opt(&self, key: &str) -> Option<Vec<String>> {
        self.get(key).and_then(|v| v.as_string_vec().map(|sv| sv.to_vec()))
    }
    /// Get date field
    pub fn get_date(&self, key: &str) -> Result<String, String> {
        self.get(key)
            .and_then(|v| v.as_date().map(|d| d.to_string()))
            .ok_or_else(|| format!("expected date field '{key}'"))
    }
    /// Get optional date field
    pub fn get_date_opt(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.as_date().map(|d| d.to_string()))
    }

    /// Get IRI field
    pub fn get_iri(&self, key: &str) -> Result<String, String> {
        self.get(key)
            .and_then(|v| v.as_iri().map(|iri| iri.to_string()))
            .ok_or_else(|| format!("expected IRI field '{key}'"))
    }
    /// Get optional IRI field
    pub fn get_iri_opt(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.as_iri().map(|iri| iri.to_string()))
    }
    /// Get number field
    pub fn get_number(&self, key: &str) -> Result<f64, String> {
        self.get(key)
            .and_then(|v| v.as_number())
            .ok_or_else(|| format!("expected number field '{key}'"))
    }
    /// Get optional number field
    pub fn get_number_opt(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.as_number())
    }
    /// Get object field
    pub fn get_object(&self, key: &str) -> Result<&Fields, String> {
        self.get(key)
            .and_then(|v| v.as_object())
            .ok_or_else(|| format!("expected object field '{key}'"))
    }
    /// Get object vector field
    pub fn get_object_vec(&self, key: &str) -> Result<&[Fields], String> {
        self.get(key)
            .and_then(|v| v.as_object_vec())
            .ok_or_else(|| format!("expected object vector field '{key}'"))
    }
}
/// Canonical field map — all metadata represented as a normalized dictionary
///
/// Keys follow kebab-case convention for consistency.
/// Values use FieldValue enum for type safety.
#[derive(Clone, Debug, Default)]
pub struct Fields {
    fields: HashMap<String, FieldValue>,
}
impl FieldMapping {
    /// Create a new empty field mapping
    pub fn new(source: &'static str, target: &'static str) -> Self {
        Self {
            source,
            target,
            rules: Vec::new(),
        }
    }
    /// Add a rule to this mapping
    pub fn with_rule(mut self, rule: FieldRule) -> Self {
        self.rules.push(rule);
        self
    }
    /// Apply all rules to transform source to target
    ///
    /// Returns missing non-required fields that could not be mapped.
    pub fn apply(&self, source: &Fields, target: &mut Fields) -> Result<Vec<String>, CrosswalkError> {
        let mut missing = Vec::new();
        for rule in &self.rules {
            if let Ok(Some(field)) = rule.apply(source, target) {
                missing.push(field);
            }
        }
        Ok(missing)
    }
}
impl FieldRule {
    /// Create a new mapping rule with identity transformation
    pub fn new(source: &'static str, target: &'static str) -> Self {
        Self {
            source,
            target,
            transform: None,
            required: false,
        }
    }
    /// Create a required field rule (shorthand for `new(f, t).required()`)
    pub fn required_field(source: &'static str, target: &'static str) -> Self {
        Self {
            source,
            target,
            transform: None,
            required: true,
        }
    }
    /// Create an optional field rule (equivalent to `new(f, t)`)
    pub fn map(source: &'static str, target: &'static str) -> Self {
        Self {
            source,
            target,
            transform: None,
            required: false,
        }
    }
    /// Make this field required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    /// Add a transformation function
    pub fn with_transform(mut self, transform: FieldTransform) -> Self {
        self.transform = Some(transform);
        self
    }
    /// Apply this rule to extract value from source and insert into target
    pub fn apply(&self, source: &Fields, target: &mut Fields) -> Result<Option<String>, CrosswalkError> {
        match source.get(self.source) {
            | Some(value) => {
                let transformed = match self.transform {
                    | Some(f) => f(value.clone()).map_err(|reason| CrosswalkError::TransformationFailed {
                        field: self.source.to_string(),
                        reason,
                    })?,
                    | None => value.clone(),
                };
                for target_name in self.target.split('|') {
                    target.insert(target_name.trim(), transformed.clone());
                }
                Ok(None)
            }
            | None => {
                if self.required {
                    Err(CrosswalkError::MissingRequiredField(self.source.to_string()))
                } else {
                    Ok(Some(self.source.to_string()))
                }
            }
        }
    }
}
impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::String(s) => write!(f, "{s}"),
            | Self::StringVec(v) => write!(f, "[{}]", v.join(", ")),
            | Self::Date(d) => write!(f, "{d}"),
            | Self::IRI(iri) => write!(f, "{iri}"),
            | Self::Number(n) => write!(f, "{n}"),
            | Self::Object(_) => write!(f, "{{...}}"),
            | Self::ObjectVec(v) => write!(f, "[...{}]", v.len()),
            | Self::Json(val) => write!(f, "{val}"),
        }
    }
}
impl From<&Fields> for HashMap<String, FieldValue> {
    fn from(map: &Fields) -> Self {
        map.fields.clone()
    }
}
impl FieldValue {
    /// Get string value, returning None if not a string variant
    pub fn as_string(&self) -> Option<&str> {
        match self {
            | Self::String(s) => Some(s),
            | _ => None,
        }
    }
    /// Get string vector, returning None if not a StringVec variant
    pub fn as_string_vec(&self) -> Option<&[String]> {
        match self {
            | Self::StringVec(v) => Some(v),
            | _ => None,
        }
    }
    /// Get date value, returning None if not a Date variant
    pub fn as_date(&self) -> Option<&str> {
        match self {
            | Self::Date(d) => Some(d),
            | _ => None,
        }
    }
    /// Get IRI value, returning None if not an IRI variant
    pub fn as_iri(&self) -> Option<&str> {
        match self {
            | Self::IRI(iri) => Some(iri),
            | _ => None,
        }
    }
    /// Get numeric value, returning None if not a Number variant
    pub fn as_number(&self) -> Option<f64> {
        match self {
            | Self::Number(n) => Some(*n),
            | _ => None,
        }
    }
    /// Get object, returning None if not an Object variant
    pub fn as_object(&self) -> Option<&Fields> {
        match self {
            | Self::Object(map) => Some(map),
            | _ => None,
        }
    }
    /// Get object vector, returning None if not an ObjectVec variant
    pub fn as_object_vec(&self) -> Option<&[Fields]> {
        match self {
            | Self::ObjectVec(v) => Some(v),
            | _ => None,
        }
    }
}
/// Create field mapping rules from DataCite to DCAT
pub fn datacite_to_dcat() -> FieldMapping {
    FieldMapping::new("datacite", "dcat")
        .with_rule(FieldRule::new("doi", "identifier").required())
        .with_rule(FieldRule::new("title", "title"))
        .with_rule(FieldRule::new("description", "description"))
        .with_rule(FieldRule::new("language", "language").with_transform(as_vec))
        .with_rule(FieldRule::new("creators", "creators"))
        .with_rule(FieldRule::new("publisher", "publisher"))
        .with_rule(FieldRule::new("publication-year", "issued").with_transform(year_to_date))
        .with_rule(FieldRule::new("license", "license"))
        .with_rule(FieldRule::new("url", "landing_page"))
        .with_rule(FieldRule::new("subjects", "keywords"))
        .with_rule(FieldRule::new("resource-type", "type_").with_transform(resource_type_to_iri))
}
/// Returns field mapping rules for DataCite → HuWise conversion
pub fn datacite_to_huwise() -> FieldMapping {
    FieldMapping::new("datacite", "huwise")
        .with_rule(FieldRule::map("doi", "identifier").required())
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
/// Create field mapping rules from DataCite to Invenio
pub fn datacite_to_invenio() -> FieldMapping {
    FieldMapping::new("datacite", "invenio")
        .with_rule(FieldRule::map("doi", "identifier").required())
        .with_rule(FieldRule::map("title", "title"))
        .with_rule(FieldRule::map("description", "description"))
        .with_rule(FieldRule::map("language", "language"))
        .with_rule(FieldRule::map("creators", "creators"))
        .with_rule(FieldRule::map("publication-year", "publication-year").with_transform(extract_year))
        .with_rule(FieldRule::map("resource-type", "resource-type"))
        .with_rule(FieldRule::map("license", "license"))
        .with_rule(FieldRule::map("alternate-identifiers", "alternate-identifiers"))
        .with_rule(FieldRule::map("subjects", "subjects"))
        .with_rule(FieldRule::map("contributors", "contributors"))
        .with_rule(FieldRule::map("version", "version"))
}
/// Create field mapping rules from DCAT to DataCite
pub fn dcat_to_datacite() -> FieldMapping {
    FieldMapping::new("dcat", "datacite")
        .with_rule(FieldRule::new("identifier", "doi").required().with_transform(first_of_vec))
        .with_rule(FieldRule::new("title", "title"))
        .with_rule(FieldRule::new("description", "description"))
        .with_rule(FieldRule::new("language", "language").with_transform(first_of_vec))
        .with_rule(FieldRule::new("creators", "creators"))
        .with_rule(FieldRule::new("publisher", "publisher"))
        .with_rule(FieldRule::new("issued", "publication-year").with_transform(extract_year))
        .with_rule(FieldRule::new("license", "license"))
        .with_rule(FieldRule::new("type_", "resource-type").with_transform(iri_to_resource_type))
        .with_rule(FieldRule::new("keywords", "subjects"))
        .with_rule(FieldRule::new("themes", "subjects"))
}
/// Returns field mapping rules for HuWise → DataCite conversion
pub fn huwise_to_datacite() -> FieldMapping {
    FieldMapping::new("huwise", "datacite")
        .with_rule(FieldRule::map("doi", "doi").required())
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
/// Create field mapping rules from Invenio to DataCite
pub fn invenio_to_datacite() -> FieldMapping {
    FieldMapping::new("invenio", "datacite")
        .with_rule(FieldRule::map("identifier", "doi").required())
        .with_rule(FieldRule::map("title", "title"))
        .with_rule(FieldRule::map("description", "description"))
        .with_rule(FieldRule::map("language", "language"))
        .with_rule(FieldRule::map("creators", "creators"))
        .with_rule(FieldRule::map("publication-year", "publication-year"))
        .with_rule(FieldRule::map("resource-type", "resource-type"))
        .with_rule(FieldRule::map("license", "license"))
        .with_rule(FieldRule::map("alternate-identifiers", "alternate-identifiers"))
        .with_rule(FieldRule::map("subjects", "subjects"))
        .with_rule(FieldRule::map("contributors", "contributors"))
}
/// Common field transformations across schemas
/// Extract first element from string vector
pub fn first_of_vec(value: FieldValue) -> Result<FieldValue, String> {
    match value {
        | FieldValue::StringVec(mut v) if !v.is_empty() => Ok(FieldValue::String(v.remove(0))),
        | FieldValue::StringVec(_) => Err("empty string vector".to_string()),
        | _ => Err("expected string vector".to_string()),
    }
}
/// Extract all elements from string vector (no-op)
pub fn as_vec(value: FieldValue) -> Result<FieldValue, String> {
    match value {
        | FieldValue::StringVec(_) => Ok(value),
        | FieldValue::String(s) => Ok(FieldValue::StringVec(vec![s])),
        | _ => Err("expected string or string vector".to_string()),
    }
}
/// Parse year from ISO 8601 date or accept as-is
pub fn extract_year(value: FieldValue) -> Result<FieldValue, String> {
    match value {
        | FieldValue::Date(d) => {
            let year = d
                .split('-')
                .next()
                .and_then(|y| y.parse::<i32>().ok())
                .ok_or_else(|| format!("cannot extract year from date: {d}"))?;
            Ok(FieldValue::Number(year as f64))
        }
        | FieldValue::String(s) => s
            .parse::<i32>()
            .map(|y| FieldValue::Number(y as f64))
            .map_err(|_| format!("cannot parse year from string: {s}")),
        | FieldValue::Number(n) => Ok(FieldValue::Number(n)),
        | _ => Err("expected date, string, or number".to_string()),
    }
}
/// Convert year to ISO 8601 date (Jan 1)
pub fn year_to_date(value: FieldValue) -> Result<FieldValue, String> {
    match value {
        | FieldValue::Number(n) => Ok(FieldValue::Date(format!("{:04}-01-01", n as i32))),
        | FieldValue::String(s) => {
            let _: i32 = s.parse().map_err(|_| format!("cannot parse year: {s}"))?;
            Ok(FieldValue::Date(format!("{s}-01-01")))
        }
        | _ => Err("expected number or string year".to_string()),
    }
}
/// ResourceTypeGeneral enum to RDF/schema.org IRI
pub fn resource_type_to_iri(value: FieldValue) -> Result<FieldValue, String> {
    match value {
        | FieldValue::String(s) => {
            let iri = match s.as_str() {
                | "Dataset" => "http://www.w3.org/ns/dcat#Dataset",
                | "Software" => "https://schema.org/SoftwareApplication",
                | "Audiovisual" => "https://schema.org/VideoObject",
                | "Book" => "https://schema.org/Book",
                | "BookChapter" => "https://schema.org/Chapter",
                | "ConferencePaper" | "JournalArticle" => "https://schema.org/ScholarlyArticle",
                | "Dissertation" => "https://schema.org/Thesis",
                | "Image" => "https://schema.org/ImageObject",
                | "Journal" => "https://schema.org/Periodical",
                | "Report" => "https://schema.org/Report",
                | "Text" => "https://schema.org/CreativeWork",
                | _ => return Err(format!("unknown resource type: {s}")),
            };
            Ok(FieldValue::IRI(iri.to_string()))
        }
        | _ => Err("expected string resource type".to_string()),
    }
}
/// RDF/schema.org IRI to ResourceTypeGeneral enum value
pub fn iri_to_resource_type(value: FieldValue) -> Result<FieldValue, String> {
    match value {
        | FieldValue::IRI(iri) => {
            let type_str = match iri.as_str() {
                | "http://www.w3.org/ns/dcat#Dataset" => "Dataset",
                | "https://schema.org/SoftwareApplication" => "Software",
                | "https://schema.org/VideoObject" => "Audiovisual",
                | "https://schema.org/Book" => "Book",
                | "https://schema.org/Chapter" => "BookChapter",
                | "https://schema.org/ScholarlyArticle" => "JournalArticle",
                | "https://schema.org/Thesis" => "Dissertation",
                | "https://schema.org/ImageObject" => "Image",
                | "https://schema.org/Periodical" => "Journal",
                | "https://schema.org/Report" => "Report",
                | "https://schema.org/CreativeWork" => "Text",
                | _ => return Err(format!("unknown IRI: {iri}")),
            };
            Ok(FieldValue::String(type_str.to_string()))
        }
        | _ => Err("expected IRI resource type".to_string()),
    }
}
