//! Runtime registration and execution for ACORN agent MCP tools
#[cfg(feature = "analysis")]
use crate::analyzer::Standard;
use crate::error::ApiResult;
use crate::io::database::schema::Table;
use crate::io::database::Database;
use crate::io::enrichment::{Enrich, Pipeline};
use acorn_core::prelude::{Box, String, Vec};
use acorn_core::util::{MarkdownSupport, MimeType};
use acorn_core::Location;
use acorn_schema::agent::tools::{NeedleToolDefinition, ToolDefinition, ToolExposure, ToolResult};
use acorn_schema::research_activity::{GraduationCandidates, ResearchActivity};
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
    Zon,
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
#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
enum UpdatesExportFormat {
    #[default]
    Markdown,
    Docx,
    Pdf,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ApplyLogbookGraduationInput {
    activity: ResearchActivity,
    accepted_ids: Vec<String>,
}
#[derive(Debug, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ApplyLogbookGraduationOutput {
    activity: ResearchActivity,
    archived_count: usize,
    research_activity_iid: String,
    revision_iid: String,
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
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProposeLogbookGraduationInput {
    activity: ResearchActivity,
}
#[derive(Debug, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct ProposeLogbookGraduationOutput {
    candidates: GraduationCandidates,
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
    mutation_available: bool,
}
#[derive(Clone)]
struct Unregistered;
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct UpdatesExportInput {
    activity: ResearchActivity,
    since: String,
    #[serde(default)]
    format: UpdatesExportFormat,
}
#[derive(Debug, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct UpdatesExportOutput {
    content: String,
    since: String,
    format: UpdatesExportFormat,
    entry_count: usize,
    archived_count: usize,
}
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
impl ApplyLogbookGraduationInput {
    async fn apply(self, database: Database<Table>) -> ApiResult<ApplyLogbookGraduationOutput> {
        database
            .graduate_logbook(self.activity, &self.accepted_ids)
            .map(|persisted| ApplyLogbookGraduationOutput {
                activity: persisted.activity,
                archived_count: persisted.archived_count,
                research_activity_iid: persisted.research_activity_iid,
                revision_iid: persisted.revision_iid,
            })
    }
}
#[cfg(feature = "analysis")]
impl CrosswalkFormat {
    fn mime(self) -> MimeType {
        match self {
            | Self::Json => MimeType::Json,
            | Self::Yaml => MimeType::Yaml,
            | Self::Zon => MimeType::Zon,
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
    /// MCP context with explicit mutation and structured-output policies.
    pub fn mcp_with_policy(offline: bool, allow_mutation: bool, max_output_bytes: usize) -> Self {
        Self {
            exposure: ToolExposure { needle: false, mcp: true },
            offline,
            allow_mutation,
            max_output_bytes,
        }
    }
}
impl ToolRegistry {
    /// Build the conservative built-in tool registry.
    pub fn acorn() -> ApiResult<Self> {
        Self::with_database(None)
    }
    /// Build the built-in registry with an optional persistent research-activity store.
    pub fn with_database(database: Option<Database<Table>>) -> ApiResult<Self> {
        let mut format_definition = ToolDefinition::new::<FormatAndEnrichInput, FormatAndEnrichOutput>(
            "acorn.format_and_enrich_research_activity_data",
            "Format and enrich research activity",
            "Normalize research activity data and enrich it using configured metadata providers.",
        );
        format_definition.effects.open_world = true;
        let mut apply_graduation_definition = ToolDefinition::new::<ApplyLogbookGraduationInput, ApplyLogbookGraduationOutput>(
            "acorn.apply_logbook_graduation",
            "Apply logbook graduation",
            "Transactionally back up the research activity, copy accepted logbook values into canonical metadata, and archive the retained entries.",
        );
        apply_graduation_definition.effects.read_only = false;
        apply_graduation_definition.effects.destructive = true;
        let mutation_available = database.is_some();
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
            })
            .and_then(|registry| {
                registry.register(
                    ToolDefinition::new::<UpdatesExportInput, UpdatesExportOutput>(
                        "acorn.export_research_activity_updates",
                        "Export research activity updates",
                        "Render logbook entries at or after an inclusive RFC 3339 timestamp.",
                    ),
                    UpdatesExportInput::export,
                )
            })
            .and_then(|registry| {
                registry.register(
                    ToolDefinition::new::<ProposeLogbookGraduationInput, ProposeLogbookGraduationOutput>(
                        "acorn.propose_logbook_graduation",
                        "Propose logbook graduation",
                        "Return reviewable logbook entries that can be copied into canonical activity metadata.",
                    ),
                    propose_logbook_graduation,
                )
            })
            .and_then(|registry| {
                let database = database.clone();
                registry.register(apply_graduation_definition, move |input: ApplyLogbookGraduationInput| {
                    let database = database.clone();
                    async move {
                        match database {
                            | Some(database) => input.apply(database).await,
                            | None => Err(eyre!("Tool 'acorn.apply_logbook_graduation' requires configured persistence")),
                        }
                    }
                })
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
        registry
            .and_then(|registry| registry.register(format_definition, format_and_enrich))
            .map(|registry| Self {
                mutation_available,
                ..registry
            })
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
                    let mutation_available = self.mutation_available;
                    let mut tools = self.tools;
                    tools.push(Tool::new(definition).register(Arc::new(typed_handler)));
                    tools.sort_by(|left, right| left.definition.name.cmp(&right.definition.name));
                    Ok(Self { tools, mutation_available })
                }
            })
    }
    /// Return definitions in deterministic name order.
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.filtered_definitions(|_| true)
    }
    /// Return definitions callable under the supplied runtime policy.
    pub fn available_definitions(&self, context: ToolCallContext) -> Vec<ToolDefinition> {
        let predicate = |definition: &ToolDefinition| {
            let exposed = (context.exposure.needle && definition.exposure.needle) || (context.exposure.mcp && definition.exposure.mcp);
            let mutation_allowed = definition.effects.read_only || (context.allow_mutation && self.mutation_available);
            exposed && mutation_allowed && !(context.offline && definition.effects.open_world)
        };
        self.filtered_definitions(predicate)
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
                let mutation_allowed = tool.definition.effects.read_only || (context.allow_mutation && self.mutation_available);
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
    fn filtered_definitions(&self, predicate: impl Fn(&ToolDefinition) -> bool) -> Vec<ToolDefinition> {
        self.tools
            .iter()
            .map(|tool| &tool.definition)
            .filter(|definition| predicate(definition))
            .cloned()
            .collect()
    }
}
impl fmt::Debug for ToolRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("ToolRegistry").field("definitions", &self.definitions()).finish()
    }
}
impl UpdatesExportInput {
    async fn export(self) -> ApiResult<UpdatesExportOutput> {
        self.activity
            .updates_since(&self.since)
            .map(|updates| UpdatesExportOutput {
                content: updates.to_markdown(),
                since: updates.since.clone(),
                format: self.format,
                entry_count: updates.entries.len(),
                archived_count: updates.archived_count(),
            })
            .map_err(|why| eyre!(why))
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
        .map(|conversion| {
            let (content, warnings) = conversion.into_parts();
            CrosswalkOutput {
                content,
                warnings: warnings.into_iter().map(|warning| warning.to_string()).collect(),
            }
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
async fn propose_logbook_graduation(input: ProposeLogbookGraduationInput) -> ApiResult<ProposeLogbookGraduationOutput> {
    Ok(ProposeLogbookGraduationOutput {
        candidates: input.activity.into(),
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
