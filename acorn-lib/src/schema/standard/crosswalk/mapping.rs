//! Field transformation rules for schema conversions
//!
//! Defines how fields are mapped from one schema to another,
//! including transformation logic and rule composition.
use super::{CrosswalkError, FieldMap, FieldValue};

/// A transformation function that maps one FieldValue to another
///
/// Used to normalize field values during conversion (e.g., convert year to ISO 8601 date).
pub type FieldTransform = fn(FieldValue) -> Result<FieldValue, String>;
/// A single field mapping rule from source to target schema
///
/// Supports one-to-many mappings (one source field → multiple target fields)
/// via multiple rules with the same source.
#[derive(Clone)]
pub struct FieldRule {
    /// Source field name (kebab-case)
    pub source_field: &'static str,
    /// Target field name(s); if "field1|field2", apply to both
    pub target_field: &'static str,
    /// Optional transformation function
    pub transform: Option<FieldTransform>,
    /// Whether this field is required in target schema
    pub required: bool,
}
impl FieldRule {
    /// Create a new mapping rule with identity transformation
    pub fn new(source_field: &'static str, target_field: &'static str) -> Self {
        Self {
            source_field,
            target_field,
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
    pub fn apply(&self, source: &FieldMap, target: &mut FieldMap) -> Result<Option<String>, CrosswalkError> {
        match source.get(self.source_field) {
            | Some(value) => {
                let transformed = match self.transform {
                    | Some(f) => f(value.clone()).map_err(|reason| CrosswalkError::TransformationFailed {
                        field: self.source_field.to_string(),
                        reason,
                    })?,
                    | None => value.clone(),
                };
                for target_name in self.target_field.split('|') {
                    target.insert(target_name.trim(), transformed.clone());
                }
                Ok(None)
            }
            | None => {
                if self.required {
                    Err(CrosswalkError::MissingRequiredField(self.source_field.to_string()))
                } else {
                    Ok(Some(self.source_field.to_string()))
                }
            }
        }
    }
}
/// A complete mapping specification from one schema to another
///
/// Contains all field rules and metadata about the conversion.
#[derive(Clone)]
pub struct FieldMapping {
    /// Source schema identifier (e.g., "datacite", "dcat")
    pub from_schema: &'static str,
    /// Target schema identifier
    pub to_schema: &'static str,
    /// All mapping rules for this conversion
    pub rules: Vec<FieldRule>,
}
impl FieldMapping {
    /// Create a new empty field mapping
    pub fn new(from_schema: &'static str, to_schema: &'static str) -> Self {
        Self {
            from_schema,
            to_schema,
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
    pub fn apply(&self, source: &FieldMap, target: &mut FieldMap) -> Result<Vec<String>, CrosswalkError> {
        let mut missing = Vec::new();
        for rule in &self.rules {
            if let Ok(Some(field)) = rule.apply(source, target) {
                missing.push(field);
            }
        }
        Ok(missing)
    }
}
/// Common field transformations across schemas
pub mod transforms {
    use super::*;

    /// Identity transformation (no change)
    pub fn identity(value: FieldValue) -> Result<FieldValue, String> {
        Ok(value)
    }
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
                    | "ConferencePaper" => "https://schema.org/ScholarlyArticle",
                    | "Dissertation" => "https://schema.org/Thesis",
                    | "Image" => "https://schema.org/ImageObject",
                    | "Journal" => "https://schema.org/Periodical",
                    | "JournalArticle" => "https://schema.org/ScholarlyArticle",
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
    /// Remove "DOI: " prefix if present, return raw DOI
    pub fn extract_doi(value: FieldValue) -> Result<FieldValue, String> {
        match value {
            | FieldValue::String(mut s) => {
                if s.starts_with("DOI: ") || s.starts_with("doi: ") {
                    s = s.split_off(5);
                }
                Ok(FieldValue::String(s))
            }
            | _ => Err("expected string DOI".to_string()),
        }
    }
}
