//! Portable data contracts for ACORN agent tools
use acorn_core::prelude::alloc::{format, vec, String, ToString, Vec};
use acorn_core::{AcornError, AcornResult};
use core::fmt;
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
    /// Case-insensitive routing expressions that require this tool when matched.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggers: Vec<String>,
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
            triggers: Vec::new(),
        }
    }
}
impl fmt::Display for ToolDefinition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self.name.as_str() {
            | "acorn.check_research_activity_data" | "acorn.validate_research_activity_data" =>
                "Validate an explicitly supplied, complete ACORN research activity JSON object. Never synthesize activity JSON from prose and never use this tool to modify or persist data.",
            | "acorn.export_research_activity_data" =>
                "Serialize an explicitly supplied, complete ACORN research activity JSON object. Do not use for unrelated information or synthesize activity JSON from prose.",
            | "acorn.export_research_activity_updates" =>
                "Render updates from an explicitly supplied, complete ACORN research activity JSON object at or after a supplied RFC 3339 timestamp.",
            | "acorn.format_and_enrich_research_activity_data" =>
                "Normalize an explicitly supplied, complete ACORN research activity JSON object and enrich it using configured metadata providers.",
            | "acorn.propose_logbook_graduation" =>
                "Propose reviewable changes from an explicitly supplied, complete ACORN research activity JSON object. This tool cannot apply, persist, or directly modify data.",
            | _ => &self.description,
        };
        formatter.write_str(description)
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
    /// Return Needle routing expressions for this tool.
    pub fn needle_triggers(&self) -> Vec<String> {
        match self.name.as_str() {
            | "acorn.version" => vec!["acorn.*version".to_string(), "version.*acorn".to_string()],
            | _ => Vec::new(),
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
