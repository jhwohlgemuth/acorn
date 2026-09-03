//! Runtime registration and execution for ACORN agent MCP tools
#[cfg(feature = "analysis")]
use crate::analyzer::Standard;
use crate::error::ApiResult;
use crate::io::enrichment::{Enrich, Pipeline};
use acorn_core::prelude::{Box, String, Vec};
use acorn_core::util::MimeType;
use acorn_core::Location;
use acorn_schema::schema::agent::tools::{NeedleToolDefinition, ToolDefinition, ToolExposure, ToolResult};
use acorn_schema::schema::research_activity::ResearchActivity;
use acorn_schema::validation::Validate;
use alloc::sync::Arc;
use color_eyre::eyre::eyre;
use core::fmt;
use core::future::Future;
use core::pin::Pin;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

type ToolFuture = Pin<Box<dyn Future<Output = ApiResult<Value>> + Send>>;
type ToolHandler = Arc<dyn Fn(Value) -> ToolFuture + Send + Sync>;
#[cfg(feature = "analysis")]
#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
enum CrosswalkFormat {
    #[default]
    Json,
    Yaml,
}
#[cfg(feature = "analysis")]
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
enum CrosswalkStandard {
    Datacite,
    Dcat,
    Huwise,
    Invenio,
}
#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
enum ExportFormat {
    #[default]
    Json,
    Markdown,
    Pdf,
    Powerpoint,
    Yaml,
}
#[cfg(feature = "analysis")]
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CrosswalkInput {
    content: String,
    source: CrosswalkStandard,
    target: CrosswalkStandard,
    #[serde(default)]
    input_format: CrosswalkFormat,
    #[serde(default)]
    output_format: CrosswalkFormat,
}
#[cfg(feature = "analysis")]
#[derive(Debug, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct CrosswalkOutput {
    content: String,
    warnings: Vec<String>,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ExportInput {
    activity: ResearchActivity,
    #[serde(default)]
    format: ExportFormat,
}
#[derive(Debug, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct ExportOutput {
    content: String,
    format: ExportFormat,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct FormatAndEnrichInput {
    activity: ResearchActivity,
    source: Option<String>,
}
#[derive(Debug, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct FormatAndEnrichOutput {
    activity: ResearchActivity,
    conflicts: Vec<String>,
    failures: Vec<String>,
}
#[derive(Clone)]
struct Registered {
    handler: ToolHandler,
}
#[derive(Clone)]
struct Tool<State> {
    definition: ToolDefinition,
    state: State,
}
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
/// Immutable registry of explicitly allowed ACORN tools.
#[derive(Clone, Default)]
pub struct ToolRegistry {
    tools: Vec<Tool<Registered>>,
}
#[derive(Clone)]
struct Unregistered;
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ValidationInput<T> {
    /// Value to validate.
    activity: T,
}
#[derive(Debug, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct ValidationOutput {
    /// Whether the supplied value is valid.
    valid: bool,
    /// Complete ordered list of validation issues.
    errors: Vec<String>,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct VersionInput {}
#[derive(Debug, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct VersionOutput {
    version: String,
}
#[cfg(feature = "analysis")]
impl CrosswalkFormat {
    fn mime(self) -> MimeType {
        match self {
            | Self::Json => MimeType::Json,
            | Self::Yaml => MimeType::Yaml,
        }
    }
}
#[cfg(feature = "analysis")]
impl CrosswalkStandard {
    fn standard(self) -> Standard {
        match self {
            | Self::Datacite => Standard::Datacite,
            | Self::Dcat => Standard::Dcat,
            | Self::Huwise => Standard::Huwise,
            | Self::Invenio => Standard::Invenio,
        }
    }
}
impl Tool<Unregistered> {
    fn new(definition: ToolDefinition) -> Self {
        Self {
            definition,
            state: Unregistered,
        }
    }
    fn register(self, handler: ToolHandler) -> Tool<Registered> {
        Tool {
            definition: self.definition,
            state: Registered { handler },
        }
    }
}
impl ToolCallContext {
    /// Restrictive context for a Needle-selected call
    pub fn needle(offline: bool) -> Self {
        Self {
            exposure: ToolExposure { needle: true, mcp: false },
            offline,
            allow_mutation: false,
            max_output_bytes: 1024 * 1024,
        }
    }
    /// Restrictive context for an MCP call
    pub fn mcp(offline: bool) -> Self {
        Self {
            exposure: ToolExposure { needle: false, mcp: true },
            offline,
            allow_mutation: false,
            max_output_bytes: 1024 * 1024,
        }
    }
}
impl fmt::Debug for ToolRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("ToolRegistry").field("definitions", &self.definitions()).finish()
    }
}
impl ToolRegistry {
    /// Build the conservative built-in tool registry.
    pub fn acorn() -> ApiResult<Self> {
        let mut format_definition = ToolDefinition::new::<FormatAndEnrichInput, FormatAndEnrichOutput>(
            "acorn.format_and_enrich_research_activity_data",
            "Format and enrich research activity",
            "Normalize research activity data and enrich it using configured metadata providers.",
        );
        format_definition.effects.open_world = true;
        let registry = Self::default()
            .register(
                ToolDefinition::new::<VersionInput, VersionOutput>(
                    "acorn.version",
                    "ACORN version",
                    "Return the version of the running ACORN library.",
                ),
                |_: VersionInput| async {
                    Ok(VersionOutput {
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    })
                },
            )
            .and_then(|registry| {
                registry.register(
                    ToolDefinition::new::<ValidationInput<ResearchActivity>, ValidationOutput>(
                        "acorn.validate_research_activity_data",
                        "Validate research activity",
                        "Validate ACORN research activity data and return every schema validation issue.",
                    ),
                    validate::<ResearchActivity>,
                )
            })
            .and_then(|registry| {
                registry.register(
                    ToolDefinition::new::<ValidationInput<ResearchActivity>, ValidationOutput>(
                        "acorn.check_research_activity_data",
                        "Check research activity",
                        "Check ACORN research activity data and return every schema validation issue.",
                    ),
                    validate::<ResearchActivity>,
                )
            })
            .and_then(|registry| {
                registry.register(
                    ToolDefinition::new::<ExportInput, ExportOutput>(
                        "acorn.export_research_activity_data",
                        "Export research activity",
                        "Serialize ACORN research activity data as JSON, Markdown, PDF, PowerPoint, or YAML.",
                    ),
                    export,
                )
            });
        #[cfg(feature = "analysis")]
        let registry = registry.and_then(|registry| {
            registry.register(
                ToolDefinition::new::<CrosswalkInput, CrosswalkOutput>(
                    "acorn.crosswalk_metadata_standard",
                    "Crosswalk metadata standards",
                    "Convert JSON or YAML metadata among DataCite, DCAT, HuWise, and Invenio while reporting field-loss warnings.",
                ),
                crosswalk,
            )
        });
        registry.and_then(|registry| registry.register(format_definition, format_and_enrich))
    }
    /// Register one definition and handler, rejecting invalid or duplicate names.
    pub fn register<I, O, F, Fut>(self, definition: ToolDefinition, handler: F) -> ApiResult<Self>
    where
        I: DeserializeOwned + Send + 'static,
        O: Serialize + 'static,
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ApiResult<O>> + Send + 'static,
    {
        let name = definition.name.clone();
        let handler = Arc::new(handler);
        let typed_handler = move |arguments: Value| -> ToolFuture {
            let input = serde_json::from_value::<I>(arguments).map_err(|why| eyre!("Invalid arguments for tool '{name}' — {why}"));
            let handler = handler.clone();
            let name = name.clone();
            Box::pin(async move {
                match input {
                    | Ok(input) => match handler(input).await {
                        | Ok(output) => serde_json::to_value(output).map_err(|why| eyre!("Invalid output from tool '{name}' — {why}")),
                        | Err(why) => Err(why),
                    },
                    | Err(why) => Err(why),
                }
            })
        };
        definition
            .validate()
            .map_err(Into::into)
            .and_then(|()| match self.tools.iter().any(|tool| tool.definition.name == definition.name) {
                | true => Err(eyre!("Duplicate tool name '{}'", definition.name)),
                | false => {
                    let mut tools = self.tools;
                    tools.push(Tool::new(definition).register(Arc::new(typed_handler)));
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
        self.tools
            .iter()
            .filter(|tool| tool.definition.exposure.needle)
            .map(|tool| tool.definition.clone().into())
            .collect()
    }
    /// Invoke one registered tool through the requested exposure boundary.
    pub async fn call(&self, name: &str, arguments: Value, exposure: ToolExposure) -> ApiResult<ToolResult> {
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
        .await
    }
    /// Invoke one registered tool after enforcing exposure, effect, and output policies.
    pub async fn call_with_context(&self, name: &str, arguments: Value, context: ToolCallContext) -> ApiResult<ToolResult> {
        match self.tools.iter().find(|tool| tool.definition.name == name) {
            | Some(tool) => {
                let exposed = (context.exposure.needle && tool.definition.exposure.needle) || (context.exposure.mcp && tool.definition.exposure.mcp);
                let mutation_allowed = tool.definition.effects.read_only || context.allow_mutation;
                let offline_violation = tool.definition.effects.open_world && context.offline;
                match (exposed, mutation_allowed, offline_violation, arguments) {
                    | (false, _, _, _) => Err(eyre!("Tool '{name}' is not exposed through this interface")),
                    | (_, false, _, _) => Err(eyre!("Tool '{name}' requires mutation approval")),
                    | (_, _, true, _) => Err(eyre!("Tool '{name}' cannot interact with external systems while offline")),
                    | (_, _, _, arguments @ Value::Object(_)) => match (tool.state.handler)(arguments).await {
                        | Ok(value) => ToolResult::success(value).map_err(|why| eyre!(why)).and_then(|result| {
                            match result.content.len() <= context.max_output_bytes.max(1) {
                                | true => Ok(result),
                                | false => Err(eyre!("Tool '{name}' result exceeded the configured output limit")),
                            }
                        }),
                        | Err(why) => Err(why),
                    },
                    | _ => Err(eyre!("Tool '{name}' arguments must be a JSON object")),
                }
            }
            | None => Err(eyre!("Unknown tool '{name}'")),
        }
    }
}

#[cfg(feature = "analysis")]
async fn crosswalk(input: CrosswalkInput) -> ApiResult<CrosswalkOutput> {
    input
        .source
        .standard()
        .crosswalk_with_warnings(
            &input.content,
            input.input_format.mime(),
            input.target.standard(),
            input.output_format.mime(),
        )
        .map(|(content, warnings)| CrosswalkOutput {
            content,
            warnings: warnings.into_iter().map(|warning| warning.to_string()).collect(),
        })
        .map_err(|why| eyre!("Crosswalk failed — {why}"))
}
async fn export(input: ExportInput) -> ApiResult<ExportOutput> {
    let mime = match input.format {
        | ExportFormat::Json => MimeType::Json,
        | ExportFormat::Markdown => MimeType::Markdown,
        | ExportFormat::Pdf => MimeType::Pdf,
        | ExportFormat::Powerpoint => MimeType::Powerpoint,
        | ExportFormat::Yaml => MimeType::Yaml,
    };
    input
        .activity
        .serialize_as(&mime)
        .map(|content| ExportOutput {
            content,
            format: input.format,
        })
        .map_err(Into::into)
}
async fn format_and_enrich(input: FormatAndEnrichInput) -> ApiResult<FormatAndEnrichOutput> {
    let source = Location::from(input.source.as_deref().unwrap_or("agent-tool"));
    Pipeline::default()
        .enrich((source, input.activity.format()))
        .await
        .map(|result| FormatAndEnrichOutput {
            activity: result.data.format(),
            conflicts: result.conflicts,
            failures: result.failures,
        })
}
async fn validate<T: Validate>(input: ValidationInput<T>) -> ApiResult<ValidationOutput> {
    Ok(match input.activity.validate() {
        | Ok(()) => ValidationOutput {
            valid: true,
            errors: Vec::new(),
        },
        | Err(report) => ValidationOutput {
            valid: false,
            errors: report.iter().map(ToString::to_string).collect(),
        },
    })
}
