//! Needle-native projections for the canonical ACORN tool registry.

use super::capability::{CapabilityError, CapabilitySelectionContext, ScopedCatalog, MAX_CAPABILITY_CATALOG_BYTES};
#[cfg(feature = "analysis")]
use super::CrosswalkInput as CanonicalCrosswalkInput;
use super::{
    CapabilityDefinition, ExportInput as CanonicalExportInput, FormatAndEnrichInput as CanonicalFormatAndEnrichInput, ProposeLogbookGraduationInput,
    Registered, ToolCallContext, ToolHandler, ToolRegistry, UpdatesExportInput as CanonicalUpdatesExportInput, ValidationInput,
};
use crate::error::ApiResult;
use crate::io::Parse;
use acorn_core::prelude::{Box, String, Vec};
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::{MimeType, StringConversion};
use acorn_schema::agent::tools::{NeedleToolDefinition, ToolDefinition, ToolExposure};
use acorn_schema::hardware::memory::Memory;
use acorn_schema::research_activity::ResearchActivity;
#[cfg(feature = "analysis")]
use acorn_schema::standard::crosswalk::CrosswalkStandard;
use acorn_schema::validation::Validate;
use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use color_eyre::eyre::{eyre, Report};
use core::future::Future;
use schemars::{schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MAX_PROMPT_CATALOG_BYTES: usize = 6 * 1024;

#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub(super) struct ActivityInput {
    /// Complete research activity JSON object copied verbatim from the request; never synthesize it from prose.
    #[validate(json)]
    activity: String,
}
#[cfg(feature = "analysis")]
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub(super) struct CrosswalkInput {
    #[validate(nonempty)]
    content: String,
    #[validate(nonempty)]
    source: String,
    #[validate(nonempty)]
    target: String,
    #[serde(default)]
    #[validate(nonempty)]
    input_format: Option<String>,
    #[serde(default)]
    #[validate(nonempty)]
    output_format: Option<String>,
}
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub(super) struct ExportInput {
    /// Complete research activity JSON object copied verbatim from the request; never synthesize it from prose.
    #[validate(json)]
    activity: String,
    #[serde(default)]
    #[validate(nonempty)]
    format: Option<String>,
}
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub(super) struct FormatAndEnrichInput {
    /// Complete research activity JSON object copied verbatim from the request; never synthesize it from prose.
    #[validate(json)]
    activity: String,
    #[validate(nonempty)]
    source: Option<String>,
}
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub(super) struct UpdatesExportInput {
    /// Complete research activity JSON object copied verbatim from the request; never synthesize it from prose.
    #[validate(json)]
    activity: String,
    #[validate(timestamp)]
    since: String,
    #[serde(default)]
    #[validate(nonempty)]
    format: Option<String>,
}
impl ActivityInput {
    pub(super) fn propose(self) -> ApiResult<ProposeLogbookGraduationInput> {
        ResearchActivity::parse(&self.activity).map(|activity| ProposeLogbookGraduationInput { activity })
    }
}
#[cfg(feature = "analysis")]
impl TryFrom<CrosswalkInput> for CanonicalCrosswalkInput {
    type Error = Report;
    fn try_from(input: CrosswalkInput) -> Result<Self, Self::Error> {
        CrosswalkStandard::parse(&input.source).and_then(|source| {
            CrosswalkStandard::parse(&input.target).and_then(|target| {
                input
                    .input_format
                    .as_deref()
                    .map(|value| value.parse::<MimeType>().map_err(Report::new))
                    .transpose()
                    .and_then(|input_format| {
                        input
                            .output_format
                            .as_deref()
                            .map(|value| value.parse::<MimeType>().map_err(Report::new))
                            .transpose()
                            .map(|output_format| Self {
                                content: input.content,
                                source,
                                target,
                                input_format,
                                output_format,
                            })
                    })
            })
        })
    }
}
impl TryFrom<ExportInput> for CanonicalExportInput {
    type Error = Report;
    fn try_from(input: ExportInput) -> Result<Self, Self::Error> {
        ResearchActivity::parse(&input.activity).and_then(|activity| {
            input
                .format
                .as_deref()
                .map(|value| value.parse::<MimeType>().map_err(Report::new))
                .transpose()
                .map(|format| Self { activity, format })
        })
    }
}
impl TryFrom<FormatAndEnrichInput> for CanonicalFormatAndEnrichInput {
    type Error = Report;
    fn try_from(input: FormatAndEnrichInput) -> Result<Self, Self::Error> {
        ResearchActivity::parse(&input.activity).map(|activity| Self {
            activity,
            source: input.source,
        })
    }
}
impl TryFrom<UpdatesExportInput> for CanonicalUpdatesExportInput {
    type Error = Report;
    fn try_from(input: UpdatesExportInput) -> Result<Self, Self::Error> {
        ResearchActivity::parse(&input.activity).and_then(|activity| {
            input
                .format
                .as_deref()
                .map(|value| value.parse::<MimeType>().map_err(Report::new))
                .transpose()
                .map(|format| Self {
                    activity,
                    since: input.since,
                    format,
                })
        })
    }
}
#[cfg(feature = "analysis")]
impl Parse<&str> for CrosswalkStandard {
    fn parse(value: &str) -> ApiResult<Self> {
        value
            .parse()
            .map_err(|_| eyre!("Unsupported crosswalk standard '{value}'; expected datacite, dcat, huwise, or invenio"))
    }
}
impl Registered {
    pub(super) fn handler(&self, context: ToolCallContext) -> &ToolHandler {
        match context.exposure {
            | ToolExposure { needle: true, mcp: false } => self.needle_handler.as_ref().unwrap_or(&self.handler),
            | _ => &self.handler,
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
}
impl ToolHandler {
    fn projected<A, C, O, R, N, Fut>(name: String, handler: Arc<R>, adapter: Arc<N>) -> Self
    where
        A: DeserializeOwned + Send + 'static,
        C: DeserializeOwned + Send + Validate + 'static,
        O: Serialize + 'static,
        R: Fn(A) -> Fut + Send + Sync + 'static,
        N: Fn(C) -> ApiResult<A> + Send + Sync + 'static,
        Fut: Future<Output = ApiResult<O>> + Send + 'static,
    {
        Self(Arc::new(move |arguments: Value| {
            let input = serde_json::from_value::<C>(arguments)
                .map_err(|why| eyre!("Invalid arguments for tool '{name}' — {why}"))
                .and_then(|input| {
                    input
                        .validate()
                        .map_err(|why| eyre!("Invalid arguments for tool '{name}' — {why}"))
                        .and_then(|()| adapter(input))
                });
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
}
impl ToolRegistry {
    #[cfg(feature = "needle")]
    pub(crate) fn has_capabilities(&self) -> bool {
        !self.capabilities.is_empty()
    }
    /// Return the deterministic Needle-native catalog.
    pub fn needle_tools(&self) -> Vec<NeedleToolDefinition> {
        self.needle_tools_with_context(ToolCallContext::needle(false))
    }
    /// Return tools available to Needle under the caller's execution policy.
    pub fn needle_tools_with_context(&self, context: ToolCallContext) -> Vec<NeedleToolDefinition> {
        self.tools
            .iter()
            .filter(|tool| context.exposure.needle && tool.definition.exposure.needle && self.is_available(&tool.definition, context))
            .map(|tool| {
                let mut parameters = tool.needle_parameters.clone().unwrap_or_else(|| tool.definition.input_schema.clone());
                if let Value::Object(parameters) = &mut parameters {
                    parameters.remove("$defs");
                    parameters.remove("$schema");
                    parameters.remove("additionalProperties");
                    parameters.remove("title");
                    parameters.entry("properties").or_insert_with(|| Value::Object(Default::default()));
                }
                NeedleToolDefinition {
                    name: tool.definition.name.clone(),
                    description: tool.definition.to_string(),
                    parameters,
                    triggers: tool.definition.needle_triggers(),
                }
            })
            .collect()
    }
    pub(super) fn register_projected<A, C, O, R, N, Fut>(self, definition: ToolDefinition, adapter: N, handler: R) -> ApiResult<Self>
    where
        A: DeserializeOwned + Send + 'static,
        C: DeserializeOwned + JsonSchema + Send + Validate + 'static,
        O: Serialize + 'static,
        R: Fn(A) -> Fut + Send + Sync + 'static,
        N: Fn(C) -> ApiResult<A> + Send + Sync + 'static,
        Fut: Future<Output = ApiResult<O>> + Send + 'static,
    {
        let name = definition.name.clone();
        let handler = Arc::new(handler);
        let canonical = ToolHandler::new::<A, O, R, Fut>(name.clone(), handler.clone());
        let projected = ToolHandler::projected::<A, C, O, R, N, Fut>(name, handler, Arc::new(adapter));
        serde_json::to_value(schema_for!(C))
            .map_err(|why| eyre!("Failed to serialize Needle tool schema — {why}"))
            .and_then(|parameters| self.insert(definition, canonical, Some(projected), Some(parameters)))
    }
}
impl TryFrom<ActivityInput> for ValidationInput<ResearchActivity> {
    type Error = Report;
    fn try_from(input: ActivityInput) -> Result<Self, Self::Error> {
        ResearchActivity::parse(&input.activity).map(|activity| Self { activity })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Family {
    Check,
    Diff,
    Doctor,
    Download,
    Exec,
    Export,
    Format,
    Gather,
    Import,
    Link,
    Report,
    Swhid,
    Sync,
    Unknown,
}
impl Family {
    fn for_prompt(words: &BTreeSet<String>) -> Option<Self> {
        let has = |terms: &[&str]| terms.iter().any(|term| words.contains(*term));
        match () {
            | _ if has(&["evaluate", "rhai", "hook", "script"]) || (has(&["static"]) && has(&["check"])) => Some(Self::Exec),
            | _ if has(&["diagnostic", "diagnostics", "doctor", "gpu", "readiness"]) => Some(Self::Doctor),
            | _ if has(&["export"]) => Some(Self::Export),
            | _ if has(&["normalize", "normalization", "enrich", "crosswalk"]) => Some(Self::Format),
            | _ if has(&["compare", "diff", "difference", "jaccard", "levenshtein", "similarity", "dice", "patch"]) => Some(Self::Diff),
            | _ if has(&["gather", "extract", "pid", "pids", "doi", "orcid", "rrid"]) && !has(&["calculate", "verify"]) => Some(Self::Gather),
            | _ if has(&["swhid", "hash", "verify"]) => Some(Self::Swhid),
            | _ if has(&["download", "fetch", "acquire", "bucket", "needle", "weights"]) => Some(Self::Download),
            | _ if has(&["publish", "report", "reports", "highlights"]) => Some(Self::Report),
            | _ if has(&["sync", "opencode", "vscode", "goose", "editor"]) => Some(Self::Sync),
            | _ if has(&["import", "endpoint", "openapi", "hugging"]) => Some(Self::Import),
            | _ if has(&["linked", "context", "jsonld"]) => Some(Self::Link),
            | _ if has(&[
                "export",
                "render",
                "rendering",
                "powerpoint",
                "archive",
                "bagit",
                "jsonl",
                "package",
                "zip",
            ]) =>
            {
                Some(Self::Export)
            }
            | _ if has(&["format", "normalize", "normalization", "enrich", "crosswalk"]) => Some(Self::Format),
            | _ if has(&["check", "checker", "validate", "analysis", "analyze", "metadata", "findings", "schema"]) => Some(Self::Check),
            | _ => None,
        }
    }
    fn group(self) -> usize {
        match self {
            | Self::Check | Self::Diff | Self::Doctor => 0,
            | Self::Exec | Self::Gather | Self::Swhid => 1,
            | Self::Export => 2,
            | Self::Format | Self::Link => 3,
            | Self::Download | Self::Import => 4,
            | Self::Report | Self::Sync => 5,
            | Self::Unknown => 6,
        }
    }
    fn keyword(self) -> &'static str {
        match self {
            | Self::Check => "check",
            | Self::Diff => "diff",
            | Self::Doctor => "doctor",
            | Self::Download => "download",
            | Self::Exec => "exec",
            | Self::Export => "export",
            | Self::Format => "format",
            | Self::Gather => "gather",
            | Self::Import => "import",
            | Self::Link => "link",
            | Self::Report => "report",
            | Self::Swhid => "swhid",
            | Self::Sync => "sync",
            | Self::Unknown => "",
        }
    }
    fn prompt_score(self, definition: &CapabilityDefinition, words: &BTreeSet<String>) -> usize {
        let name = definition.id.split('.').nth(1).unwrap_or_default();
        let family = self.keyword();
        let aliases: &[&str] = match self {
            | Self::Check => &["analyze", "inspect", "validate", "validation", "quality", "findings"],
            | Self::Diff => &[
                "compare",
                "difference",
                "jaccard",
                "jaro",
                "levenshtein",
                "similarity",
                "patch",
                "cosine",
                "dice",
            ],
            | Self::Doctor => &["diagnostic", "diagnostics", "readiness", "memory", "gpu"],
            | Self::Download => &["acquire", "fetch", "weights", "bucket"],
            | Self::Exec => &["script", "rhai", "hook", "manifest"],
            | Self::Export => &["render", "pdf", "powerpoint", "archive", "bagit", "jsonl"],
            | Self::Format => &["normalize", "normalization", "enrich", "formatter", "crosswalk"],
            | Self::Gather => &["extract", "identifier", "identifiers", "pid", "doi", "orcid", "rrid"],
            | Self::Import => &["persist", "endpoint", "hugging", "metadata"],
            | Self::Link => &["linked", "context", "jsonld"],
            | Self::Report => &["publish", "publication", "highlights", "logbook"],
            | Self::Swhid => &["hash", "revision", "identifier"],
            | Self::Sync => &["opencode", "vscode", "goose", "configuration", "editor"],
            | Self::Unknown => &[],
        };
        let family_score: usize = match (words.contains(family), aliases.iter().any(|alias| words.contains(*alias))) {
            | (true, _) => 200,
            | (false, true) if self == Self::Exec => 260,
            | (false, true) => 120,
            | (false, false) => 0,
        };
        let name_score = name
            .split(|character: char| !character.is_ascii_alphanumeric())
            .filter(|part| words.contains(*part))
            .count()
            .saturating_mul(35);
        let description_score = definition
            .description
            .to_ascii_lowercase()
            .split(|character: char| !character.is_ascii_alphanumeric())
            .filter(|word| word.len() >= 4 && words.contains(*word))
            .count()
            .saturating_mul(3);
        family_score.saturating_add(name_score).saturating_add(description_score)
    }
}
impl From<&str> for Family {
    fn from(id: &str) -> Self {
        let scoped = id
            .split_once(':')
            .filter(|(prefix, _)| !prefix.contains('.'))
            .map_or(id, |(_, scoped)| scoped);
        match scoped.split('.').nth(1).unwrap_or_default().split(['_', ':']).next().unwrap_or_default() {
            | "check" => Self::Check,
            | "diff" => Self::Diff,
            | "doctor" => Self::Doctor,
            | "download" => Self::Download,
            | "exec" => Self::Exec,
            | "export" => Self::Export,
            | "format" => Self::Format,
            | "gather" => Self::Gather,
            | "import" => Self::Import,
            | "link" => Self::Link,
            | "report" => Self::Report,
            | "swhid" => Self::Swhid,
            | "sync" => Self::Sync,
            | _ => Self::Unknown,
        }
    }
}
impl From<&String> for Family {
    fn from(id: &String) -> Self {
        Self::from(id.as_str())
    }
}
impl From<String> for Family {
    fn from(id: String) -> Self {
        Self::from(id.as_str())
    }
}
impl ToolRegistry {
    /// Serialize an explicitly scoped capability catalog within Needle's byte cap.
    /// Binding alternatives are deduplicated behind their capability contract.
    pub fn capability_catalog(&self, scope: Option<&[String]>, context: CapabilitySelectionContext) -> ApiResult<String> {
        self.serialize_capability_catalog(scope, context)
            .and_then(|serialized| match serialized.len() <= MAX_CAPABILITY_CATALOG_BYTES {
                | true => Ok(serialized),
                | false => Err(CapabilityError::CatalogTooLarge.into()),
            })
    }
    /// Serialize route choices without asking Needle to invent native arguments.
    pub fn selection_catalog(&self, scope: Option<&[String]>, context: CapabilitySelectionContext) -> ApiResult<String> {
        self.serialize_capability_catalog(scope, context).and_then(|catalog| {
            serde_json::from_str::<Vec<NeedleToolDefinition>>(&catalog)
                .map_err(|why| CapabilityError::CatalogSerialization(why.to_string()).into())
                .and_then(|definitions| {
                    let choices = definitions
                        .into_iter()
                        .map(|definition| NeedleToolDefinition {
                            parameters: serde_json::json!({
                                "type":"object",
                                "properties":{"source":{"type":"string"}},
                                "additionalProperties":false
                            }),
                            ..definition
                        })
                        .collect::<Vec<_>>();
                    serde_json::to_string(&choices).map_err(|why| CapabilityError::CatalogSerialization(why.to_string()).into())
                })
        })
    }
    /// Choose a deterministic prompt shortlist when the eligible catalog exceeds Needle's limit.
    pub fn scope_for_prompt(&self, prompt: &str, context: CapabilitySelectionContext) -> ApiResult<ScopedCatalog> {
        self.selection_catalog(None, context).and_then(|unscoped| {
            let unscoped_bytes = unscoped.len();
            match self.capabilities.len() <= 1 && unscoped_bytes <= MAX_PROMPT_CATALOG_BYTES {
                | true => Ok(ScopedCatalog {
                    ids: None,
                    bytes: Memory::from(unscoped_bytes as u64),
                    omitted: 0,
                    unscoped_bytes: Memory::from(unscoped_bytes as u64),
                }),
                | false => {
                    let intent_words = prompt.words().into_iter().collect::<BTreeSet<_>>();
                    let words = prompt
                        .to_ascii_lowercase()
                        .split(|character: char| !character.is_ascii_alphanumeric())
                        .filter(|word| !word.is_empty())
                        .map(str::to_string)
                        .collect::<BTreeSet<_>>();
                    let mut ranked = self
                        .capabilities
                        .iter()
                        .filter(|entry| {
                            entry
                                .bindings
                                .iter()
                                .any(|binding| binding.definition.availability.reason(&binding.definition, context).is_none())
                        })
                        .map(|entry| {
                            (
                                Family::from(&entry.definition.id).prompt_score(&entry.definition, &words),
                                entry.definition.id.clone(),
                            )
                        })
                        .collect::<Vec<_>>();
                    ranked.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
                    let hinted = Family::for_prompt(&intent_words).filter(|family| ranked.iter().any(|(_, id)| Family::from(id) == *family));
                    let groups = hinted.map(|family| vec![family.group()]).unwrap_or_else(|| {
                        ranked
                            .iter()
                            .map(|(_, id)| Family::from(id).group())
                            .fold(Vec::<usize>::new(), |mut groups, group| {
                                if !groups.contains(&group) {
                                    groups.push(group);
                                }
                                groups
                            })
                            .into_iter()
                            .take(2)
                            .collect::<Vec<_>>()
                    });
                    let within_family = |id: &String| match hinted {
                        | Some(family) => Family::from(id) == family,
                        | None => groups.contains(&Family::from(id).group()),
                    };
                    let mut grouped = ranked
                        .iter()
                        .filter(|(_, id)| within_family(id))
                        .map(|(_, id)| id.clone())
                        .collect::<Vec<_>>();
                    if hinted.is_none() {
                        grouped.extend(
                            [format!("{APPLICATION}.check"), format!("{APPLICATION}.export::bag")]
                                .into_iter()
                                .filter_map(|id| {
                                    ranked
                                        .iter()
                                        .find(|(_, available)| available == &id)
                                        .map(|(_, available)| available.clone())
                                }),
                        );
                    }
                    grouped.sort();
                    grouped.dedup();
                    let outside_group = ranked.len().saturating_sub(grouped.len());
                    self.selection_catalog(Some(&grouped), context)
                        .and_then(|catalog| match catalog.len() <= MAX_PROMPT_CATALOG_BYTES {
                            | true => Ok(ScopedCatalog {
                                ids: Some(grouped.clone()),
                                bytes: Memory::from(catalog.len() as u64),
                                omitted: outside_group,
                                unscoped_bytes: Memory::from(unscoped_bytes as u64),
                            }),
                            | false => ranked
                                .into_iter()
                                .filter(|(_, id)| within_family(id))
                                .try_fold((Vec::<String>::new(), 0_usize, 2_usize), |(ids, omitted, bytes), (_, id)| {
                                    let mut candidate = ids.clone();
                                    candidate.push(id);
                                    self.selection_catalog(Some(&candidate), context).map(|candidate_catalog| {
                                        match candidate_catalog.len() <= MAX_PROMPT_CATALOG_BYTES {
                                            | true => (candidate, omitted, candidate_catalog.len()),
                                            | false => (ids, omitted.saturating_add(1), bytes),
                                        }
                                    })
                                })
                                .and_then(|(ids, omitted, bytes)| match ids.is_empty() {
                                    | true => Err(CapabilityError::CatalogTooLarge.into()),
                                    | false => Ok(ScopedCatalog {
                                        ids: Some({
                                            let mut ids = ids;
                                            ids.sort();
                                            ids
                                        }),
                                        bytes: Memory::from(bytes as u64),
                                        omitted: omitted.saturating_add(outside_group),
                                        unscoped_bytes: Memory::from(unscoped_bytes as u64),
                                    }),
                                }),
                        })
                }
            }
        })
    }
    fn serialize_capability_catalog(&self, scope: Option<&[String]>, context: CapabilitySelectionContext) -> ApiResult<String> {
        let valid_scope = match scope {
            | Some([]) => Err(CapabilityError::EmptyScope.into()),
            | Some(ids) => ids
                .iter()
                .try_for_each(|id| match self.capabilities.iter().find(|entry| &entry.definition.id == id) {
                    | None => Err(CapabilityError::UnknownScope(id.clone()).into()),
                    | Some(entry)
                        if !entry
                            .bindings
                            .iter()
                            .any(|binding| binding.definition.availability.reason(&binding.definition, context).is_none()) =>
                    {
                        Err(CapabilityError::UnavailableScope(id.clone()).into())
                    }
                    | Some(_) => Ok(()),
                }),
            | None => Ok(()),
        };
        valid_scope.and_then(|()| {
            let definitions = self
                .capabilities
                .iter()
                .filter(|entry| {
                    scope.is_none_or(|ids| ids.contains(&entry.definition.id))
                        && entry
                            .bindings
                            .iter()
                            .any(|binding| binding.definition.availability.reason(&binding.definition, context).is_none())
                })
                .map(|entry| NeedleToolDefinition {
                    name: entry.definition.id.clone(),
                    description: entry.definition.description.clone(),
                    parameters: if entry.definition.id == format!("{APPLICATION}.check") {
                        serde_json::json!({
                            "type": "object",
                            "properties": {
                                "input": {
                                    "type": "object",
                                    "properties": {
                                        "kind": {"enum": ["directory", "file", "git-branch", "git-commit", "oci", "text", "url"]},
                                        "value": {"type": "string"}
                                    },
                                    "required": ["kind", "value"],
                                    "additionalProperties": false
                                }
                            },
                            "required": ["input"],
                            "additionalProperties": false
                        })
                    } else {
                        model_catalog_schema(&entry.definition.input_schema)
                    },
                    triggers: Vec::new(),
                })
                .collect::<Vec<_>>();
            serde_json::to_string(&definitions).map_err(|why| CapabilityError::CatalogSerialization(why.to_string()).into())
        })
    }
}
pub(super) fn model_catalog_schema(schema: &Value) -> Value {
    fn project(value: &Value, named_schemas: bool) -> Value {
        match value {
            | Value::Object(fields) => Value::Object(
                fields
                    .iter()
                    .filter(|(name, _)| named_schemas || !matches!(name.as_str(), "description" | "title"))
                    .map(|(name, value)| {
                        let projected = if !named_schemas && matches!(name.as_str(), "const" | "default" | "enum" | "examples") {
                            value.clone()
                        } else {
                            project(
                                value,
                                !named_schemas
                                    && matches!(
                                        name.as_str(),
                                        "$defs" | "definitions" | "dependentSchemas" | "patternProperties" | "properties"
                                    ),
                            )
                        };
                        (name.clone(), projected)
                    })
                    .collect(),
            ),
            | Value::Array(values) => Value::Array(values.iter().map(|value| project(value, false)).collect()),
            | _ => value.clone(),
        }
    }
    project(schema, false)
}
