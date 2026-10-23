//! Needle-native projections for the canonical ACORN tool registry.

#[cfg(feature = "analysis")]
use super::CrosswalkInput as CanonicalCrosswalkInput;
use super::{
    ExportInput as CanonicalExportInput, FormatAndEnrichInput as CanonicalFormatAndEnrichInput, ProposeLogbookGraduationInput, Registered,
    ToolCallContext, ToolHandler, ToolRegistry, UpdatesExportInput as CanonicalUpdatesExportInput, ValidationInput,
};
use crate::error::ApiResult;
use crate::io::Parse;
use acorn_core::prelude::{Box, String, Vec};
use acorn_core::util::MimeType;
use acorn_schema::agent::tools::{NeedleToolDefinition, ToolDefinition, ToolExposure};
use acorn_schema::research_activity::ResearchActivity;
#[cfg(feature = "analysis")]
use acorn_schema::standard::crosswalk::CrosswalkStandard;
use acorn_schema::validation::Validate;
use alloc::sync::Arc;
use color_eyre::eyre::{eyre, Report};
use core::future::Future;
use schemars::{schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
