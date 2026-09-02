//! Runtime registration and execution for ACORN agent tools.

use crate::error::ApiResult;
use acorn_core::prelude::{String, Vec};
use acorn_schema::schema::agent::tools::{NeedleToolDefinition, ToolDefinition, ToolEffects, ToolExposure, ToolResult};
use acorn_schema::schema::research_activity::ResearchActivity;
use acorn_schema::validation::Validate;
use alloc::sync::Arc;
use color_eyre::eyre::eyre;
use core::fmt;
use schemars::{schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Runtime policy applied before a registered tool handler is called.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolCallContext {
    /// Interface requesting the call.
    pub exposure: ToolExposure,
    /// Whether calls that interact with external systems must be rejected.
    pub offline: bool,
    /// Whether a non-read-only handler may run.
    pub allow_mutation: bool,
    /// Maximum serialized structured-result size.
    pub max_output_bytes: usize,
}

type ToolHandler = Arc<dyn Fn(Value) -> ApiResult<Value> + Send + Sync>;

#[derive(Clone)]
struct RegisteredTool {
    definition: ToolDefinition,
    handler: ToolHandler,
}

/// Immutable registry of explicitly allowed ACORN tools.
#[derive(Clone, Default)]
pub struct ToolRegistry {
    tools: Vec<RegisteredTool>,
}

impl fmt::Debug for ToolRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("ToolRegistry").field("definitions", &self.definitions()).finish()
    }
}

impl ToolCallContext {
    /// Restrictive context for a Needle-selected call.
    pub fn needle(offline: bool) -> Self {
        Self {
            exposure: ToolExposure { needle: true, mcp: false },
            offline,
            allow_mutation: false,
            max_output_bytes: 1024 * 1024,
        }
    }

    /// Restrictive context for an MCP call.
    pub fn mcp(offline: bool) -> Self {
        Self {
            exposure: ToolExposure { needle: false, mcp: true },
            offline,
            allow_mutation: false,
            max_output_bytes: 1024 * 1024,
        }
    }
}

impl ToolRegistry {
    /// Build the conservative built-in ACORN tool registry.
    pub fn acorn() -> ApiResult<Self> {
        Self::default()
            .register(
                definition::<VersionInput, VersionOutput>("acorn.version", "ACORN version", "Return the version of the running ACORN library."),
                |_: VersionInput| {
                    Ok(VersionOutput {
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    })
                },
            )
            .and_then(|registry| {
                registry.register(
                    definition::<ValidateResearchActivityInput, ValidateResearchActivityOutput>(
                        "acorn.validate_research_activity",
                        "Validate research activity",
                        "Validate an ACORN research activity JSON object and return every schema validation issue.",
                    ),
                    validate_research_activity,
                )
            })
    }

    /// Register one definition and handler, rejecting invalid or duplicate names.
    pub fn register<I, O, F>(self, definition: ToolDefinition, handler: F) -> ApiResult<Self>
    where
        I: DeserializeOwned + Send + 'static,
        O: Serialize + 'static,
        F: Fn(I) -> ApiResult<O> + Send + Sync + 'static,
    {
        let name = definition.name.clone();
        let typed_handler = move |arguments: Value| {
            serde_json::from_value::<I>(arguments)
                .map_err(|why| eyre!("Invalid arguments for tool '{name}' — {why}"))
                .and_then(&handler)
                .and_then(|output| serde_json::to_value(output).map_err(|why| eyre!("Invalid output from tool '{name}' — {why}")))
        };
        definition
            .validate()
            .map_err(Into::into)
            .and_then(|()| match self.tools.iter().any(|tool| tool.definition.name == definition.name) {
                | true => Err(eyre!("Duplicate tool name '{}'", definition.name)),
                | false => {
                    let mut tools = self.tools;
                    tools.push(RegisteredTool {
                        definition,
                        handler: Arc::new(typed_handler),
                    });
                    tools.sort_by(|left, right| left.definition.name.cmp(&right.definition.name));
                    Ok(Self { tools })
                }
            })
    }

    /// Return definitions in deterministic name order.
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.iter().map(|tool| tool.definition.clone()).collect()
    }

    /// Return the deterministic Needle-native catalog.
    pub fn needle_tools(&self) -> Vec<NeedleToolDefinition> {
        self.tools.iter().filter_map(|tool| tool.definition.to_needle()).collect()
    }

    /// Invoke one registered tool through the requested exposure boundary.
    pub fn call(&self, name: &str, arguments: Value, exposure: ToolExposure) -> ApiResult<ToolResult> {
        self.call_with_context(
            name,
            arguments,
            ToolCallContext {
                exposure,
                offline: false,
                allow_mutation: false,
                max_output_bytes: 1024 * 1024,
            },
        )
    }

    /// Invoke one registered tool after enforcing exposure, effect, and output policies.
    pub fn call_with_context(&self, name: &str, arguments: Value, context: ToolCallContext) -> ApiResult<ToolResult> {
        self.tools
            .iter()
            .find(|tool| tool.definition.name == name)
            .ok_or_else(|| eyre!("Unknown tool '{name}'"))
            .and_then(|tool| {
                match (
                    context.exposure.needle && tool.definition.exposure.needle,
                    context.exposure.mcp && tool.definition.exposure.mcp,
                ) {
                    | (true, _) | (_, true) => Ok(tool),
                    | _ => Err(eyre!("Tool '{name}' is not exposed through this interface")),
                }
            })
            .and_then(|tool| {
                match (
                    tool.definition.effects.read_only || context.allow_mutation,
                    tool.definition.effects.open_world && context.offline,
                ) {
                    | (false, _) => Err(eyre!("Tool '{name}' requires mutation approval")),
                    | (_, true) => Err(eyre!("Tool '{name}' cannot interact with external systems while offline")),
                    | _ => Ok(tool),
                }
            })
            .and_then(|tool| match arguments {
                | Value::Object(_) => (tool.handler)(arguments),
                | _ => Err(eyre!("Tool '{name}' arguments must be a JSON object")),
            })
            .and_then(|value| ToolResult::success(value).map_err(Into::into))
            .and_then(|result| match result.content.len() <= context.max_output_bytes.max(1) {
                | true => Ok(result),
                | false => Err(eyre!("Tool '{name}' result exceeded the configured output limit")),
            })
    }
}

fn definition<I, O>(name: &str, title: &str, description: &str) -> ToolDefinition
where
    I: JsonSchema,
    O: JsonSchema,
{
    ToolDefinition {
        name: name.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        input_schema: serde_json::to_value(schema_for!(I)).unwrap_or_else(|_| json!({ "type": "object" })),
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

fn validate_research_activity(input: ValidateResearchActivityInput) -> ApiResult<ValidateResearchActivityOutput> {
    Ok(match serde_json::from_value::<ResearchActivity>(input.activity) {
        | Ok(activity) => match activity.validate() {
            | Ok(()) => ValidateResearchActivityOutput {
                valid: true,
                errors: Vec::new(),
            },
            | Err(report) => ValidateResearchActivityOutput {
                valid: false,
                errors: report.iter().map(ToString::to_string).collect(),
            },
        },
        | Err(why) => ValidateResearchActivityOutput {
            valid: false,
            errors: vec![why.to_string()],
        },
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct VersionInput {}

#[derive(Debug, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct VersionOutput {
    version: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ValidateResearchActivityInput {
    /// Research activity JSON object to validate.
    activity: Value,
}

#[derive(Debug, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct ValidateResearchActivityOutput {
    /// Whether the research activity is valid.
    valid: bool,
    /// Complete ordered list of validation issues.
    errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_deterministic_and_shared() {
        let registry = ToolRegistry::acorn().expect("registry");
        let definitions = registry.definitions();
        let needle = registry.needle_tools();
        assert_eq!(definitions.len(), needle.len());
        assert!(definitions
            .windows(2)
            .all(|pair| pair.first().zip(pair.get(1)).is_some_and(|(left, right)| left.name < right.name)));
        definitions.iter().zip(needle).for_each(|(definition, projected)| {
            assert_eq!(definition.name, projected.name);
            assert_eq!(definition.description, projected.description);
            assert_eq!(definition.input_schema, projected.parameters);
        });
    }

    #[test]
    fn registry_calls_only_object_arguments() {
        let registry = ToolRegistry::acorn().expect("registry");
        let exposure = ToolExposure { needle: true, mcp: false };
        let result = registry.call("acorn.version", json!({}), exposure).expect("version result");
        assert_eq!(
            result.structured_content.get("version").and_then(Value::as_str),
            Some(env!("CARGO_PKG_VERSION"))
        );
        assert!(registry.call("acorn.version", json!({ "unexpected": true }), exposure).is_err());
        assert!(registry.call("acorn.version", json!([]), exposure).is_err());
        assert!(registry.call("missing", json!({}), exposure).is_err());
    }

    #[test]
    fn validation_tool_returns_all_validation_state() {
        let registry = ToolRegistry::acorn().expect("registry");
        let result = registry
            .call(
                "acorn.validate_research_activity",
                json!({ "activity": { "title": "x" } }),
                ToolExposure { needle: false, mcp: true },
            )
            .expect("validation result");
        assert_eq!(result.structured_content.get("valid").and_then(Value::as_bool), Some(false));
        assert!(result
            .structured_content
            .get("errors")
            .and_then(Value::as_array)
            .is_some_and(|errors| !errors.is_empty()));
    }
}
