//! Lexical references to packet properties.
use crate::validation::{Validate, ValidationReport};
use acorn_core::prelude::alloc::String;
use acorn_core::validation::ValidationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Authored `id#property.path` reference, retained without rewriting escapes.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Reference(pub String);
/// Reference whose target position supplies an orientation from velocity.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct VelocityReference(pub Reference);

impl Validate for Reference {
    fn validate(&self) -> Result<(), ValidationReport> {
        let valid = self.0.split_once('#').is_some_and(|(id, path)| {
            !id.is_empty() && !path.is_empty() && !path.contains('#') && path.split('.').all(|segment| !segment.is_empty())
        });
        valid.then_some(()).ok_or_else(|| {
            ValidationReport::from_error(
                "",
                ValidationError::new("czml_reference").with_message("Expected id#property.path with nonempty segments"),
            )
        })
    }
}
impl Validate for VelocityReference {
    fn validate(&self) -> Result<(), ValidationReport> {
        self.0.validate()
    }
}
