//! Managed [Needle 3](https://cactuscompute.com/needle) inference sidecar.
//!
//! Needle 3 is a compact local model specialized for selecting and calling tools.
//! ACORN runs the pinned standalone engine from the [upstream project](https://github.com/cactus-compute/needle);
//! see the [ACORN Needle guide](https://acorn.ornl.gov/inference/needle.html) for usage details.
use crate::io::api::needle::{Client, CompleteResponse};
use crate::io::mcp::{
    CapabilityError, CapabilitySelectionContext, ExecutionContext, ExecutionOutcome, ExecutionProvenance, RoutedExecution, ToolCallContext,
    ToolRegistry,
};
use crate::io::sidecar::{ResolvedAssets, Sidecar, SidecarAssets};
use crate::io::{standard_project_folder, write_file, ApiResult};
use crate::util::constants::app::{DEFAULT_NEEDLE_CONFIDENCE_THRESHOLD, DEFAULT_NEEDLE_MAX_ROUNDS};
use acorn_core::prelude::{create_dir, PathBuf, String, Vec};
use acorn_core::Noul;
use acorn_schema::agent::tools::{NeedleToolDefinition, ToolResult};
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use futures::stream::{self, StreamExt};
use process::Runner;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub mod assets;
mod process;

const MAX_CATALOG_BYTES: usize = 32 * 1024;
/// Reason to withhold automatic execution of a proposed Needle call.
///
/// Withholding is an abstention, not a technical failure.
/// The response stays reportable and the calls are never executed.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "reason")]
pub enum WithheldExecution {
    /// Calibrated confidence fell below the configured execution threshold.
    Confidence {
        /// Calibrated confidence Needle reported for the call.
        confidence: Noul,
        /// Minimum confidence ACORN required before executing a call.
        threshold: Noul,
    },
    /// Needle reported no usable calibrated confidence for the call.
    Uncalibrated,
    /// Needle reported that the request negated the proposed call.
    Negation,
    /// Needle reported argument fields that were not grounded in the request.
    Ungrounded {
        /// Argument fields Needle could not ground.
        fields: Vec<String>,
    },
}
/// Result of a bounded Needle inference loop.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceResult {
    /// Final native Needle response.
    pub response: CompleteResponse,
    /// Tool results executed during the loop.
    pub results: Vec<ToolResult>,
    /// Why the execution gate withheld the final response's calls, when it did.
    pub withheld: Option<WithheldExecution>,
}
/// Running Needle 3 sidecar bound to one immutable tool catalog.
#[derive(Debug)]
pub struct Needle {
    runner: Runner,
    client: Client,
    registry: ToolRegistry,
    confidence_threshold: Noul,
    max_rounds: usize,
    offline: bool,
}
/// One Needle decision and at most one terminal capability execution.
#[skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, Serialize)]
#[builder(start_fn = init)]
pub struct RoutedInference {
    /// Original Needle decision for audit and route preview.
    pub response: Option<CompleteResponse>,
    /// Terminal route outcome.
    pub outcome: ExecutionOutcome,
    /// Selected route when one was grounded and eligible.
    pub selected: Option<ExecutionProvenance>,
    /// Validated execution envelope when a binding was selected.
    pub execution: Option<RoutedExecution>,
    /// Reason the Needle execution gate withheld a proposed call.
    pub withheld: Option<WithheldExecution>,
}
impl CompleteResponse {
    fn execution_gate(&self, threshold: Noul) -> ApiResult<Result<(), WithheldExecution>> {
        let valid_type = self.response_type.is_call();
        let confidence = self
            .confidence
            .and_then(|confidence| Noul::new(confidence).ok())
            .ok_or(WithheldExecution::Uncalibrated);
        match (valid_type, self.validation.as_ref(), confidence) {
            | (false, _, _) => Err(eyre!("Needle response type '{}' cannot contain function calls", self.response_type)),
            | (_, Some(validation), _) if validation.negation => Ok(Err(WithheldExecution::Negation)),
            | (_, Some(validation), _) if !validation.ungrounded.is_empty() => Ok(Err(WithheldExecution::Ungrounded {
                fields: validation.ungrounded.clone(),
            })),
            | (_, _, Ok(confidence)) if confidence < threshold => Ok(Err(WithheldExecution::Confidence { confidence, threshold })),
            | (_, _, Ok(_)) => Ok(Ok(())),
            | (_, _, Err(denied)) => Ok(Err(denied)),
        }
    }
    fn response_error(&self) -> color_eyre::eyre::Report {
        let detail = self
            .error
            .as_deref()
            .or(self.reason.as_deref())
            .or(self.error_code.as_deref())
            .unwrap_or("unknown error");
        eyre!("Needle inference failed — {detail}")
    }
}
impl Needle {
    /// Resolve verified assets and start Needle with a caller-owned tool registry.
    pub async fn start_with_registry(custom_location: Option<&str>, offline: bool, registry: ToolRegistry) -> ApiResult<Self> {
        Self::start_with_scoped_registry(custom_location, offline, registry, None).await
    }
    /// Start Needle with a caller-owned registry and an optional explicit capability scope.
    pub async fn start_with_scoped_registry(
        custom_location: Option<&str>,
        offline: bool,
        registry: ToolRegistry,
        scope: Option<Vec<String>>,
    ) -> ApiResult<Self> {
        match Self::resolve_assets(custom_location, offline, true).await {
            | Ok(assets) => Self::start_with_assets_and_registry(assets, offline, registry, scope).await,
            | Err(why) => Err(why),
        }
    }
    async fn start_with_assets_and_registry(
        resolved: ResolvedAssets,
        offline: bool,
        registry: ToolRegistry,
        scope: Option<Vec<String>>,
    ) -> ApiResult<Self> {
        let context = ToolCallContext::needle(offline);
        let catalog = registry.active_catalog(context, scope.as_deref());
        let start = match catalog {
            | Ok(catalog) => serde_json::from_str::<Vec<NeedleToolDefinition>>(&catalog)
                .map_err(|why| eyre!("Invalid active Needle catalog — {why}"))
                .and_then(|definitions| assets::validate_tools::<Self, _>(resolved, &definitions))
                .and_then(|resolved| {
                    let session_root = standard_project_folder(&format!("{}/sessions", Self::KIND), None);
                    let tools = session_root.join("tools.json");
                    assets::tool_index::<Self>(&resolved, &catalog, &session_root)
                        .and_then(|tool_index| {
                            create_dir(&session_root)
                                .map_err(|why| eyre!("Failed to create Needle session directory — {why}"))
                                .and_then(|()| write_file(&tools, catalog.clone()))
                                .and_then(|()| tool_index.prepare())
                                .map(|()| tool_index)
                        })
                        .map(|tool_index| (resolved, tool_index, session_root, tools))
                })
                .map(|(resolved, tool_index, session_root, tools)| async move {
                    match Runner::start(&resolved, &tools, tool_index, session_root).await {
                        | Ok(runner) => {
                            let endpoint = runner.endpoint().to_string();
                            Client::new(endpoint).map(|client| Self {
                                runner,
                                client,
                                registry,
                                confidence_threshold: DEFAULT_NEEDLE_CONFIDENCE_THRESHOLD,
                                max_rounds: DEFAULT_NEEDLE_MAX_ROUNDS,
                                offline,
                            })
                        }
                        | Err(why) => Err(why),
                    }
                }),
            | Err(why) => Err(why),
        };
        match start {
            | Ok(future) => future.await,
            | Err(why) => Err(why),
        }
    }
    /// Override the confidence threshold, constrained to the inclusive unit interval
    pub fn with_confidence_threshold(self, threshold: Noul) -> Self {
        Self {
            confidence_threshold: threshold,
            ..self
        }
    }
    /// Override the positive maximum inference rounds
    pub fn with_max_rounds(self, max_rounds: usize) -> ApiResult<Self> {
        match max_rounds {
            | 0 => Err(eyre!("Needle maximum rounds must be greater than zero")),
            | _ => Ok(Self { max_rounds, ..self }),
        }
    }
    /// Return the private loopback endpoint of the managed Needle process.
    pub fn endpoint(&self) -> &str {
        self.runner.endpoint()
    }
    /// Ask Needle once, then execute at most one grounded capability call.
    pub async fn route_once(self, prompt: String, mut context: ExecutionContext) -> ApiResult<RoutedInference> {
        context.selection.offline |= self.offline;
        match self.client.complete(prompt).await {
            | Ok(response) => Self::execute_route_response(&self.registry, response, self.confidence_threshold, context).await,
            | Err(_) => Ok(RoutedInference::init().outcome(ExecutionOutcome::TransportFailed).build()),
        }
    }
    async fn execute_route_response(
        registry: &ToolRegistry,
        response: CompleteResponse,
        threshold: Noul,
        mut context: ExecutionContext,
    ) -> ApiResult<RoutedInference> {
        let base = |outcome, withheld| {
            RoutedInference::init()
                .response(response.clone())
                .outcome(outcome)
                .maybe_withheld(withheld)
                .build()
        };
        if !response.success || response.error.is_some() {
            Ok(base(ExecutionOutcome::ToolFailed, None))
        } else if let [call] = response.function_calls.as_slice() {
            if context.caller_arguments.as_ref().is_some_and(|arguments| arguments != &call.arguments) {
                Ok(base(ExecutionOutcome::Withheld, None))
            } else {
                match response.execution_gate(threshold) {
                    | Err(_) => Ok(base(ExecutionOutcome::InvalidInput, None)),
                    | Ok(Err(withheld)) => Ok(base(ExecutionOutcome::Withheld, Some(withheld))),
                    | Ok(Ok(())) => {
                        let selection = registry.select_capability_binding(
                            &call.name,
                            context.binding_override.as_deref(),
                            context.binding_preference.as_deref(),
                            context.selection,
                        );
                        match selection {
                            | Err(why) => Ok(base(
                                if matches!(why.downcast_ref::<CapabilityError>(), Some(CapabilityError::AmbiguousBindings(_))) {
                                    ExecutionOutcome::NeedsInput
                                } else {
                                    ExecutionOutcome::Unavailable
                                },
                                None,
                            )),
                            | Ok(selected) => {
                                context.grounded = true;
                                let execution = registry.execute_selected(&selected, call.arguments.clone(), context).await;
                                Ok(RoutedInference::init()
                                    .response(response)
                                    .outcome(execution.outcome)
                                    .selected(execution.provenance.clone())
                                    .execution(execution)
                                    .build())
                            }
                        }
                    }
                }
            }
        } else {
            Ok(base(
                if response.response_type.is_refuse() {
                    ExecutionOutcome::Withheld
                } else {
                    ExecutionOutcome::NeedsInput
                },
                None,
            ))
        }
    }
}
#[async_trait]
impl Sidecar for Needle {
    type Input = String;
    type Output = InferenceResult;
    type ToolIndex = assets::NeedleToolIndex;

    async fn start_with_assets(resolved: ResolvedAssets, offline: bool) -> ApiResult<Self> {
        match ToolRegistry::acorn() {
            | Ok(registry) => Self::start_with_assets_and_registry(resolved, offline, registry, None).await,
            | Err(why) => Err(why),
        }
    }
    async fn infer(self, prompt: String) -> ApiResult<InferenceResult> {
        let mut input = prompt;
        let mut results = Vec::new();
        let mut round = 0_usize;
        loop {
            round = round.saturating_add(1);
            if round > self.max_rounds {
                break Err(eyre!("Needle inference exceeded the {}-round limit", self.max_rounds));
            }
            match self.client.complete(input).await {
                | Ok(response) if !response.success || response.error.is_some() => break Err(response.response_error()),
                | Ok(response) if !(response.response_type.is_call() || response.response_type.is_refuse() || response.response_type.is_respond()) => {
                    break Err(eyre!("Needle returned unknown response type '{}'", response.response_type));
                }
                | Ok(response) if response.function_calls.is_empty() => {
                    break Ok(InferenceResult {
                        response,
                        results,
                        withheld: None,
                    })
                }
                | Ok(response) => match response.execution_gate(self.confidence_threshold) {
                    | Ok(Ok(())) => {
                        let called = stream::iter(response.function_calls.iter())
                            .then(|call| {
                                let context = ToolCallContext::needle(self.offline);
                                self.registry.call_with_context(&call.name, call.arguments.clone(), context)
                            })
                            .collect::<Vec<_>>()
                            .await
                            .into_iter()
                            .collect::<ApiResult<Vec<_>>>();
                        match called {
                            | Ok(called) => {
                                let payload = match called.as_slice() {
                                    | [result] => result.structured_content.clone(),
                                    | results => serde_json::Value::Array(results.iter().map(|result| result.structured_content.clone()).collect()),
                                };
                                match serde_json::to_string(&payload) {
                                    | Ok(serialized) => {
                                        input = serialized;
                                        results.extend(called);
                                    }
                                    | Err(why) => break Err(eyre!("Failed to serialize Needle tool results — {why}")),
                                }
                            }
                            | Err(why) => break Err(why),
                        }
                    }
                    | Ok(Err(withheld)) => {
                        break Ok(InferenceResult {
                            response,
                            results,
                            withheld: Some(withheld),
                        })
                    }
                    | Err(why) => break Err(why),
                },
                | Err(why) => break Err(why),
            }
        }
    }
}
impl ToolRegistry {
    fn active_catalog(&self, context: ToolCallContext, scope: Option<&[String]>) -> ApiResult<String> {
        match self.has_capabilities() {
            | true => self.capability_catalog(
                scope,
                CapabilitySelectionContext {
                    offline: context.offline,
                    allow_remote: false,
                },
            ),
            | false => self.serialize_catalog(context),
        }
    }
    fn serialize_catalog(&self, context: ToolCallContext) -> ApiResult<String> {
        serde_json::to_string_pretty(&self.needle_tools_with_context(context))
            .map_err(|why| eyre!("Failed to serialize Needle tools — {why}"))
            .and_then(|catalog| match catalog.len() <= MAX_CATALOG_BYTES {
                | true => Ok(catalog),
                | false => Err(eyre!("Needle tool catalog exceeds the {MAX_CATALOG_BYTES}-byte limit")),
            })
    }
}
/// Write the canonical Needle catalog to a file.
pub fn write_tools(path: PathBuf) -> ApiResult<()> {
    ToolRegistry::acorn()
        .and_then(|registry| registry.serialize_catalog(ToolCallContext::needle(false)))
        .and_then(|content| write_file(path, content))
}
#[cfg(test)]
mod tests;
