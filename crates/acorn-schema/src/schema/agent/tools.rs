//! Portable data contracts for ACORN agent tools.

use acorn_core::prelude::alloc::{format, String};
use acorn_core::{AcornError, AcornResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Interfaces through which a tool may be exposed.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolExposure {
    /// Expose the tool to Needle.
    pub needle: bool,
    /// Expose the tool through MCP.
    pub mcp: bool,
}

/// Effect metadata used for policy enforcement and protocol hints.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolEffects {
    /// The tool does not modify its environment.
    pub read_only: bool,
    /// The tool may perform a destructive operation.
    pub destructive: bool,
    /// Repeating the same call has no additional effect.
    pub idempotent: bool,
    /// The tool may interact with external entities.
    pub open_world: bool,
}

/// Serializable definition of a callable ACORN tool.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolDefinition {
    /// Stable programmatic tool name.
    pub name: String,
    /// Human-readable tool title.
    pub title: String,
    /// Description used by a model when selecting the tool.
    pub description: String,
    /// JSON Schema for tool arguments.
    pub input_schema: Value,
    /// Optional JSON Schema for structured output.
    pub output_schema: Option<Value>,
    /// Effect metadata.
    pub effects: ToolEffects,
    /// Protocol exposure allowlist.
    pub exposure: ToolExposure,
}

/// Needle-native tool schema written to `tools.json`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NeedleToolDefinition {
    /// Stable programmatic tool name.
    pub name: String,
    /// Description used by Needle for selection and retrieval.
    pub description: String,
    /// JSON Schema for tool arguments.
    pub parameters: Value,
}

/// Canonical result returned by a tool handler.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolResult {
    /// Human-readable or backwards-compatible text representation.
    pub content: String,
    /// Machine-readable result.
    pub structured_content: Value,
    /// Whether the handler completed with a tool-level error.
    pub is_error: bool,
}

impl ToolDefinition {
    /// Validate the portable name, description, schemas, and effect metadata.
    pub fn validate(&self) -> AcornResult<()> {
        let valid_name = (1..=128).contains(&self.name.len())
            && self
                .name
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.'));
        match (
            valid_name,
            self.title.trim().is_empty(),
            self.description.trim().is_empty(),
            self.input_schema.get("type").and_then(Value::as_str),
            self.output_schema.as_ref().and_then(|schema| schema.get("type")).and_then(Value::as_str),
            self.effects.read_only && self.effects.destructive,
        ) {
            | (false, _, _, _, _, _) => Err(AcornError::new(format!(
                "Invalid tool name '{}'; use 1-128 ASCII letters, digits, '.', '_' or '-'",
                self.name
            ))),
            | (_, true, _, _, _, _) => Err(AcornError::new(format!("Tool '{}' requires a title", self.name))),
            | (_, _, true, _, _, _) => Err(AcornError::new(format!("Tool '{}' requires a description", self.name))),
            | (_, _, _, Some("object"), Some(kind), _) if kind != "object" => {
                Err(AcornError::new(format!("Tool '{}' output schema must describe an object", self.name)))
            }
            | (_, _, _, Some("object"), _, true) => Err(AcornError::new(format!("Read-only tool '{}' cannot be destructive", self.name))),
            | (_, _, _, Some("object"), _, false) => Ok(()),
            | _ => Err(AcornError::new(format!("Tool '{}' input schema must describe an object", self.name))),
        }
    }

    /// Convert this definition into Needle's native tool schema.
    pub fn to_needle(&self) -> Option<NeedleToolDefinition> {
        self.exposure.needle.then(|| NeedleToolDefinition {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: self.input_schema.clone(),
        })
    }
}

impl ToolResult {
    /// Construct a successful canonical result.
    pub fn success(value: Value) -> AcornResult<Self> {
        serde_json::to_string(&value)
            .map(|content| Self {
                content,
                structured_content: value,
                is_error: false,
            })
            .map_err(|why| AcornError::new(format!("Failed to serialize tool result — {why}")))
    }
}
