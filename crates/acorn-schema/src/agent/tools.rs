//! Portable data contracts for ACORN agent tools
use acorn_core::prelude::alloc::{format, String, ToString};
use acorn_core::{AcornError, AcornResult};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
/// Interfaces through which a tool may be exposed.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolExposure {
    /// Expose the tool to Needle.
    pub needle: bool,
    /// Expose the tool through MCP.
    pub mcp: bool,
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
impl From<ToolDefinition> for NeedleToolDefinition {
    fn from(definition: ToolDefinition) -> Self {
        Self {
            name: definition.name,
            description: definition.description,
            parameters: definition.input_schema,
        }
    }
}
impl ToolDefinition {
    /// Construct a read-only tool definition from typed input and output schemas
    pub fn new<I: JsonSchema, O: JsonSchema>(name: &str, title: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            input_schema: serde_json::to_value(schema_for!(I)).unwrap_or_else(|_| serde_json::json!({ "type": "object" })),
            output_schema: serde_json::to_value(schema_for!(O)).ok(),
            effects: ToolEffects {
                read_only: true,
                destructive: false,
                idempotent: true,
                open_world: false,
            },
            exposure: ToolExposure { needle: true, mcp: true },
        }
    }
    /// Validate the portable name, description, schemas, and effect metadata
    pub fn validate(&self) -> AcornResult<()> {
        let valid_characters = self
            .name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.'));
        let valid_name = (1..=128).contains(&self.name.len()) && valid_characters;
        [
            (!valid_name).then(|| {
                AcornError::new(format!(
                    "Invalid tool name '{}'; use 1-128 ASCII letters, digits, '.', '_' or '-'",
                    self.name
                ))
            }),
            self.title
                .trim()
                .is_empty()
                .then(|| AcornError::new(format!("Tool '{}' requires a title", self.name))),
            self.description
                .trim()
                .is_empty()
                .then(|| AcornError::new(format!("Tool '{}' requires a description", self.name))),
            (self.input_schema.get("type").and_then(Value::as_str) != Some("object"))
                .then(|| AcornError::new(format!("Tool '{}' input schema must describe an object", self.name))),
            self.output_schema
                .as_ref()
                .and_then(|schema| schema.get("type"))
                .and_then(Value::as_str)
                .is_some_and(|kind| kind != "object")
                .then(|| AcornError::new(format!("Tool '{}' output schema must describe an object", self.name))),
            (self.effects.read_only && self.effects.destructive)
                .then(|| AcornError::new(format!("Read-only tool '{}' cannot be destructive", self.name))),
        ]
        .into_iter()
        .flatten()
        .next()
        .map_or(Ok(()), Err)
    }
}
impl ToolResult {
    /// Construct a successful canonical result
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
