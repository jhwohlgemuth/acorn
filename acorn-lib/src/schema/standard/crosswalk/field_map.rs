//! Canonical intermediate representation for schema conversions
//!
//! FieldMap enables type-safe field mapping between schemas without serde_json overhead.
//! It provides a normalized view of metadata that all schemas can be extracted to/from.
use crate::prelude::HashMap;
use core::fmt;
use serde_json::Value;

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
    Object(Box<FieldMap>),
    /// List of objects
    ObjectVec(Vec<FieldMap>),
    /// Fallback for untyped values (JSON)
    Json(Value),
}
/// Canonical field map: all metadata represented as a normalized dictionary
///
/// Keys follow kebab-case convention for consistency.
/// Values use FieldValue enum for type safety.
#[derive(Clone, Debug, Default)]
pub struct FieldMap {
    fields: HashMap<String, FieldValue>,
}
impl FieldMap {
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
    pub fn get_object(&self, key: &str) -> Result<&FieldMap, String> {
        self.get(key)
            .and_then(|v| v.as_object())
            .ok_or_else(|| format!("expected object field '{key}'"))
    }
    /// Get object vector field
    pub fn get_object_vec(&self, key: &str) -> Result<&[FieldMap], String> {
        self.get(key)
            .and_then(|v| v.as_object_vec())
            .ok_or_else(|| format!("expected object vector field '{key}'"))
    }
}
impl From<HashMap<String, FieldValue>> for FieldMap {
    fn from(fields: HashMap<String, FieldValue>) -> Self {
        Self { fields }
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
impl From<&FieldMap> for HashMap<String, FieldValue> {
    fn from(map: &FieldMap) -> Self {
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
    pub fn as_object(&self) -> Option<&FieldMap> {
        match self {
            | Self::Object(map) => Some(map),
            | _ => None,
        }
    }
    /// Get object vector, returning None if not an ObjectVec variant
    pub fn as_object_vec(&self) -> Option<&[FieldMap]> {
        match self {
            | Self::ObjectVec(v) => Some(v),
            | _ => None,
        }
    }
}
