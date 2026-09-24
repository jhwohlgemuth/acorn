//! Runtime registration and execution for ACORN agent MCP tools
mod needle;

#[cfg(feature = "analysis")]
use crate::analyzer::Standard;
use crate::error::ApiResult;
use crate::io::database::schema::Table;
use crate::io::database::{Database, LogbookGraduationPersistence};
use crate::io::enrichment::{Enrich, Pipeline};
use acorn_core::prelude::{Box, String, Vec};
use acorn_core::util::{MarkdownSupport, MimeType};
use acorn_core::Location;
use acorn_schema::agent::tools::{ToolDefinition, ToolExposure, ToolResult};
use acorn_schema::research_activity::{GraduationCandidates, ResearchActivity};
#[cfg(feature = "analysis")]
use acorn_schema::standard::crosswalk::CrosswalkStandard;
use acorn_schema::validation::Validate;
use alloc::sync::Arc;
use color_eyre::eyre::{eyre, Report};
use core::fmt;
use core::future::Future;
use core::pin::Pin;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

type ToolFuture = Pin<Box<dyn Future<Output = ApiResult<Value>> + Send>>;
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
    input_format: Option<MimeType>,
    output_format: Option<MimeType>,
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
    format: Option<MimeType>,
}
#[derive(Debug, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct ExportOutput {
    content: String,
    format: MimeType,
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
    needle_handler: Option<ToolHandler>,
}
#[derive(Clone)]
struct Tool<State> {
    definition: ToolDefinition,
    needle_parameters: Option<Value>,
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
#[derive(Clone)]
struct ToolHandler(Arc<dyn Fn(Value) -> ToolFuture + Send + Sync>);
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
    format: Option<MimeType>,
}
#[derive(Debug, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct UpdatesExportOutput {
    content: String,
    since: String,
    format: MimeType,
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
        database.graduate_logbook(self.activity, &self.accepted_ids).map(Into::into)
    }
}
impl From<LogbookGraduationPersistence> for ApplyLogbookGraduationOutput {
    fn from(persisted: LogbookGraduationPersistence) -> Self {
        Self {
            activity: persisted.activity,
            archived_count: persisted.archived_count,
            research_activity_iid: persisted.research_activity_iid,
            revision_iid: persisted.revision_iid,
        }
    }
}
#[cfg(feature = "analysis")]
impl TryFrom<CrosswalkInput> for CrosswalkOutput {
    type Error = Report;
    fn try_from(input: CrosswalkInput) -> Result<Self, Self::Error> {
        let input_format = input.input_format.unwrap_or(MimeType::Json);
        let output_format = input.output_format.unwrap_or(MimeType::Json);
        match (&input_format, &output_format) {
            | (MimeType::Json | MimeType::Yaml | MimeType::Zon, MimeType::Json | MimeType::Yaml | MimeType::Zon) => Standard::from(input.source)
                .crosswalk_with_warnings(&input.content, input_format, Standard::from(input.target), output_format)
                .map(|conversion| {
                    let (content, warnings) = conversion.into_parts();
                    Self {
                        content,
                        warnings: warnings.into_iter().map(|warning| warning.to_string()).collect(),
                    }
                })
                .map_err(|why| eyre!("Crosswalk failed — {why}")),
            | _ => Err(eyre!("Crosswalk formats must be JSON, YAML, or ZON")),
        }
    }
}
impl ExportInput {
    async fn apply(self) -> ApiResult<ExportOutput> {
        let format = self.format.unwrap_or(MimeType::Json);
        match format {
            | MimeType::Json | MimeType::Markdown | MimeType::Pdf | MimeType::Powerpoint | MimeType::Yaml => self
                .activity
                .serialize_as(&format)
                .map(|content| ExportOutput { content, format })
                .map_err(Into::into),
            | _ => Err(eyre!("Export format must be JSON, Markdown, PDF, PowerPoint, or YAML")),
        }
    }
}
impl FormatAndEnrichInput {
    async fn apply(self) -> ApiResult<FormatAndEnrichOutput> {
        let source = Location::from(self.source.as_deref().unwrap_or("agent-tool"));
        Pipeline::default()
            .enrich((source, self.activity.format()))
            .await
            .map(|result| FormatAndEnrichOutput {
                activity: result.data.format(),
                conflicts: result.conflicts,
                failures: result.failures,
            })
    }
}
#[cfg(feature = "analysis")]
impl From<CrosswalkStandard> for Standard {
    fn from(standard: CrosswalkStandard) -> Self {
        match standard {
            | CrosswalkStandard::Datacite => Self::Datacite,
            | CrosswalkStandard::Dcat => Self::Dcat,
            | CrosswalkStandard::Huwise => Self::Huwise,
            | CrosswalkStandard::Invenio => Self::Invenio,
        }
    }
}
impl Tool<Unregistered> {
    fn new(definition: ToolDefinition) -> Self {
        Self {
            definition,
            needle_parameters: None,
            state: Unregistered,
        }
    }
    fn register(self, handler: ToolHandler, needle_handler: Option<ToolHandler>, needle_parameters: Option<Value>) -> Tool<Registered> {
        Tool {
            definition: self.definition,
            needle_parameters,
            state: Registered { handler, needle_handler },
        }
    }
}
impl ToolCallContext {
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
impl ToolHandler {
    fn new<I, O, F, Fut>(name: String, handler: Arc<F>) -> Self
    where
        I: DeserializeOwned + Send + 'static,
        O: Serialize + 'static,
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ApiResult<O>> + Send + 'static,
    {
        Self(Arc::new(move |arguments: Value| {
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
        }))
    }
    fn call(&self, arguments: Value) -> ToolFuture {
        (self.0)(arguments)
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
        apply_graduation_definition.exposure.needle = false;
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
                registry.register_projected(
                    ToolDefinition::new::<ValidationInput<ResearchActivity>, ValidationOutput>(
                        "acorn.validate_research_activity_data",
                        "Validate research activity",
                        "Validate ACORN research activity data and return every schema validation issue.",
                    ),
                    |input: needle::ActivityInput| ValidationInput::<ResearchActivity>::try_from(input),
                    |input: ValidationInput<ResearchActivity>| async move { ValidationOutput::try_from(input) },
                )
            })
            .and_then(|registry| {
                registry.register_projected(
                    ToolDefinition::new::<ValidationInput<ResearchActivity>, ValidationOutput>(
                        "acorn.check_research_activity_data",
                        "Check research activity",
                        "Check ACORN research activity data and return every schema validation issue.",
                    ),
                    |input: needle::ActivityInput| ValidationInput::<ResearchActivity>::try_from(input),
                    |input: ValidationInput<ResearchActivity>| async move { ValidationOutput::try_from(input) },
                )
            })
            .and_then(|registry| {
                registry.register_projected(
                    ToolDefinition::new::<ExportInput, ExportOutput>(
                        "acorn.export_research_activity_data",
                        "Export research activity",
                        "Serialize ACORN research activity data as JSON, Markdown, PDF, PowerPoint, or YAML.",
                    ),
                    |input: needle::ExportInput| ExportInput::try_from(input),
                    ExportInput::apply,
                )
            })
            .and_then(|registry| {
                registry.register_projected(
                    ToolDefinition::new::<UpdatesExportInput, UpdatesExportOutput>(
                        "acorn.export_research_activity_updates",
                        "Export research activity updates",
                        "Render logbook entries at or after an inclusive RFC 3339 timestamp.",
                    ),
                    |input: needle::UpdatesExportInput| UpdatesExportInput::try_from(input),
                    UpdatesExportInput::export,
                )
            })
            .and_then(|registry| {
                registry.register_projected(
                    ToolDefinition::new::<ProposeLogbookGraduationInput, ProposeLogbookGraduationOutput>(
                        "acorn.propose_logbook_graduation",
                        "Propose logbook graduation",
                        "Return reviewable logbook entries that can be copied into canonical activity metadata.",
                    ),
                    needle::ActivityInput::propose,
                    |input: ProposeLogbookGraduationInput| async move {
                        Ok(ProposeLogbookGraduationOutput {
                            candidates: input.activity.into(),
                        })
                    },
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
            registry.register_projected(
                ToolDefinition::new::<CrosswalkInput, CrosswalkOutput>(
                    "acorn.crosswalk_metadata_standard",
                    "Crosswalk metadata standards",
                    "Convert JSON or YAML metadata among DataCite, DCAT, HuWise, and Invenio while reporting field-loss warnings.",
                ),
                |input: needle::CrosswalkInput| CrosswalkInput::try_from(input),
                |input: CrosswalkInput| async move { CrosswalkOutput::try_from(input) },
            )
        });
        registry
            .and_then(|registry| {
                registry.register_projected(
                    format_definition,
                    |input: needle::FormatAndEnrichInput| FormatAndEnrichInput::try_from(input),
                    FormatAndEnrichInput::apply,
                )
            })
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
        let typed_handler = ToolHandler::new::<I, O, F, Fut>(definition.name.clone(), Arc::new(handler));
        self.insert(definition, typed_handler, None, None)
    }
    fn insert(
        self,
        definition: ToolDefinition,
        handler: ToolHandler,
        needle_handler: Option<ToolHandler>,
        needle_parameters: Option<Value>,
    ) -> ApiResult<Self> {
        definition
            .validate()
            .map_err(Into::into)
            .and_then(|()| match self.tools.iter().any(|tool| tool.definition.name == definition.name) {
                | true => Err(eyre!("Duplicate tool name '{}'", definition.name)),
                | false => {
                    let mutation_available = self.mutation_available;
                    let mut tools = self.tools;
                    tools.push(Tool::new(definition).register(handler, needle_handler, needle_parameters));
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
                    | (_, _, _, arguments @ Value::Object(_)) => match tool.state.handler(context).call(arguments).await {
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
impl UpdatesExportInput {
    async fn export(self) -> ApiResult<UpdatesExportOutput> {
        let format = self.format.unwrap_or(MimeType::Markdown);
        match format {
            | MimeType::Docx | MimeType::Markdown | MimeType::Pdf => self
                .activity
                .updates_since(&self.since)
                .map(|updates| UpdatesExportOutput {
                    content: updates.to_markdown(),
                    since: updates.since.clone(),
                    format,
                    entry_count: updates.entries.len(),
                    archived_count: updates.archived_count(),
                })
                .map_err(|why| eyre!(why)),
            | _ => Err(eyre!("Updates export format must be DOCX, Markdown, or PDF")),
        }
    }
}
impl<T: Validate> TryFrom<ValidationInput<T>> for ValidationOutput {
    type Error = Report;
    fn try_from(input: ValidationInput<T>) -> Result<Self, Self::Error> {
        Ok(match input.activity.validate() {
            | Ok(()) => Self {
                valid: true,
                errors: Vec::new(),
            },
            | Err(report) => Self {
                valid: false,
                errors: report.iter().map(ToString::to_string).collect(),
            },
        })
    }
}
