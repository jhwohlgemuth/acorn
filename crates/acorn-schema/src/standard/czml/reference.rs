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
        let mut escaped = false;
        let mut has_separator = false;
        let mut has_id = false;
        let mut has_segment = false;
        let mut valid = true;
        for character in self.0.chars() {
            if escaped {
                if has_separator {
                    has_segment = true;
                } else {
                    has_id = true;
                }
                escaped = false;
            } else {
                match character {
                    | '\\' => escaped = true,
                    | '#' if !has_separator => {
                        valid &= has_id;
                        has_separator = true;
                    }
                    | '#' => valid = false,
                    | '.' if has_separator => {
                        valid &= has_segment;
                        has_segment = false;
                    }
                    | _ if has_separator => has_segment = true,
                    | _ => has_id = true,
                }
            }
        }
        valid &= has_separator && has_id && has_segment && !escaped;
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
