//! Schema registry and metadata
//!
//! Provides discovery and introspection of all supported metadata schemas.
//! Enables runtime schema detection and conversion discovery.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Metadata about a schema and its conversion capabilities
#[derive(Clone, Debug)]
pub struct SchemaMetadata {
    /// Schema identifier (e.g., "datacite")
    pub name: &'static str,
    /// User-friendly display name
    pub display_name: &'static str,
    /// Whether schema has a Catalog type for batch conversions
    pub has_catalog: bool,
    /// Nesting depth (1=flat, 5+=deeply nested)
    pub nesting_level: u8,
    /// Short description
    pub description: &'static str,
    /// Canonical main resource type name
    pub main_type: &'static str,
}

impl SchemaMetadata {
    /// Create schema metadata
    pub fn new(
        name: &'static str,
        display_name: &'static str,
        has_catalog: bool,
        nesting_level: u8,
        description: &'static str,
        main_type: &'static str,
    ) -> Self {
        Self {
            name,
            display_name,
            has_catalog,
            nesting_level,
            description,
            main_type,
        }
    }
}

/// Central registry of all metadata schemas
fn build_registry() -> HashMap<&'static str, SchemaMetadata> {
    let mut registry = HashMap::new();

    registry.insert(
        "datacite",
        SchemaMetadata::new(
            "datacite",
            "DataCite",
            true,
            4,
            "DataCite Metadata Schema v4.6 for DOI registration",
            "Record",
        ),
    );

    registry.insert(
        "dcat",
        SchemaMetadata::new(
            "dcat",
            "DCAT v3",
            false,
            3,
            "Data Catalog Vocabulary (W3C) for dataset descriptions",
            "Dataset",
        ),
    );

    registry.insert(
        "cff",
        SchemaMetadata::new("cff", "CFF", true, 2, "Citation File Format for software metadata", "Cff"),
    );

    registry.insert(
        "invenio",
        SchemaMetadata::new(
            "invenio",
            "InvenioRDM",
            true,
            5,
            "InvenioRDM metadata schema for research data repositories",
            "Record",
        ),
    );

    registry.insert(
        "huwise",
        SchemaMetadata::new(
            "huwise",
            "HuWise",
            true,
            4,
            "HuWise dataset metadata with multi-block profiles",
            "Dataset",
        ),
    );

    registry
}

static SCHEMA_REGISTRY: OnceLock<HashMap<&'static str, SchemaMetadata>> = OnceLock::new();

/// Get the global schema registry
pub fn schema_registry() -> &'static HashMap<&'static str, SchemaMetadata> {
    SCHEMA_REGISTRY.get_or_init(build_registry)
}

/// Get metadata for a schema by name
pub fn get_schema(name: &str) -> Option<SchemaMetadata> {
    schema_registry().get(name).cloned()
}

/// Get all schema names
pub fn all_schemas() -> Vec<&'static str> {
    schema_registry().keys().copied().collect()
}

/// Check if a conversion is theoretically possible (both schemas exist)
pub fn can_convert(from: &str, to: &str) -> bool {
    schema_registry().contains_key(from) && schema_registry().contains_key(to)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_populated() {
        let registry = schema_registry();
        assert!(registry.len() >= 5, "registry should have at least 5 schemas");
        assert!(registry.contains_key("datacite"));
        assert!(registry.contains_key("dcat"));
        assert!(registry.contains_key("cff"));
        assert!(registry.contains_key("invenio"));
        assert!(registry.contains_key("huwise"));
    }

    #[test]
    fn can_convert_works() {
        assert!(can_convert("datacite", "dcat"));
        assert!(can_convert("dcat", "datacite"));
        assert!(!can_convert("unknown", "dcat"));
    }
}
