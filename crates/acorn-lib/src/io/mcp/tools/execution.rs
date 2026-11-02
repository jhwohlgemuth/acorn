//! One terminal capability invocation with caller-bound authority.
use super::{CapabilitySelectionContext, HandlerCall, SelectedBinding, ToolRegistry};
use crate::util::constants::app::DEFAULT_JSON_RPC_MAX_BODY_BYTES;
use acorn_core::prelude::{String, Vec};
use acorn_core::util::SemanticVersion;
use acorn_schema::hardware::Memory;
use core::future::{self, Future};
use core::pin::Pin;
use core::time::Duration;
use serde::Serialize;
use serde_json::Value;
use std::time::Instant;

const COOPERATIVE_CLEANUP_TIMEOUT: Duration = Duration::from_secs(4);

/// Terminal outcome of one routed operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionOutcome {
    /// A validated route is ready for caller review without execution.
    Proposed,
    /// One handler completed and its typed output passed validation.
    Completed,
    /// Required caller input is absent.
    NeedsInput,
    /// Grounding or route policy withheld execution.
    Withheld,
    /// The selected binding is stale or ineligible.
    Unavailable,
    /// An exact mutation grant is absent or does not match.
    PermissionRequired,
    /// Structured input or output limits were violated.
    InvalidInput,
    /// The caller cancelled an in-flight invocation.
    Cancelled,
    /// The caller deadline expired.
    TimedOut,
    /// A transport failed before a tool result was available.
    TransportFailed,
    /// A handler failed or returned an invalid result.
    ToolFailed,
}
/// Bounded reference to a generated artifact.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ArtifactReference {
    /// Reference URI or local path emitted by the handler.
    pub uri: String,
}
/// Policy and resource bounds for one routed invocation.
pub struct ExecutionContext {
    /// Executor eligibility policy.
    pub selection: CapabilitySelectionContext,
    /// Select and report the route without invoking its handler.
    pub preview: bool,
    /// Explicit caller binding choice; never inferred from model output.
    pub binding_override: Option<String>,
    /// Configured equivalent binding preference.
    pub binding_preference: Option<String>,
    /// Whether arguments came from explicit caller data or passed the route grounding gate.
    pub grounded: bool,
    /// Exact structured arguments supplied by the caller, when present.
    pub caller_arguments: Option<Value>,
    /// Optional exact mutation grant.
    pub mutation: Option<MutationAuthority>,
    /// Maximum serialized input size.
    pub max_input: Memory,
    /// Maximum serialized output size.
    pub max_output: Memory,
    /// Optional caller deadline.
    pub deadline: Option<Instant>,
    /// Optional cancellation future; cooperative handlers receive the signal and finish bounded cleanup.
    pub cancellation: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,
}
/// Secret-free identity of the selected operation and implementation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExecutionProvenance {
    /// Canonical operation ID.
    pub capability_id: String,
    /// Contract version used for this invocation.
    pub capability_version: SemanticVersion,
    /// Selected configured executor ID.
    pub binding_id: String,
}
/// Exact caller grant for one mutating invocation.
#[derive(Clone, Debug, PartialEq)]
pub struct MutationAuthority {
    /// Approved capability ID.
    pub capability_id: String,
    /// Approved executor binding ID.
    pub binding_id: String,
    /// Approved structured arguments, including the destination field.
    pub arguments: Value,
    /// Approved destination, matching `arguments.destination` exactly.
    pub destination: String,
}
/// Final structured result from one terminal handler call.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RoutedExecution {
    /// Terminal status.
    pub outcome: ExecutionOutcome,
    /// Validated handler output when complete.
    pub content: Option<Value>,
    /// Bounded output artifact references.
    pub artifacts: Vec<ArtifactReference>,
    /// Selected contract identity; never contains credentials.
    pub provenance: ExecutionProvenance,
    /// Whether an in-flight effect may have started before cancellation or timeout.
    pub effects_may_have_occurred: bool,
}
impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            selection: CapabilitySelectionContext::default(),
            preview: false,
            binding_override: None,
            binding_preference: None,
            grounded: false,
            caller_arguments: None,
            mutation: None,
            max_input: Memory::from(DEFAULT_JSON_RPC_MAX_BODY_BYTES as u64),
            max_output: Memory::from(DEFAULT_JSON_RPC_MAX_BODY_BYTES as u64),
            deadline: None,
            cancellation: None,
        }
    }
}
impl RoutedExecution {
    fn new(selected: &SelectedBinding, outcome: ExecutionOutcome, effects_may_have_occurred: bool) -> Self {
        Self {
            outcome,
            content: None,
            artifacts: Vec::new(),
            provenance: ExecutionProvenance {
                capability_id: selected.capability.id.clone(),
                capability_version: selected.capability.version,
                binding_id: selected.binding.id.clone(),
            },
            effects_may_have_occurred,
        }
    }
}
impl ToolRegistry {
    /// Revalidate a frozen route and execute its handler at most once.
    pub async fn execute_selected(&self, selected: &SelectedBinding, arguments: Value, context: ExecutionContext) -> RoutedExecution {
        let handler = self.current_handler(selected, context.selection);
        let input_size = serde_json::to_vec(&arguments).map_or(usize::MAX, |encoded| encoded.len());
        let authorized = selected.capability.effects.read_only
            || context.mutation.as_ref().is_some_and(|grant| {
                grant.capability_id == selected.capability.id
                    && grant.binding_id == selected.binding.id
                    && grant.arguments == arguments
                    && arguments.get("destination").and_then(Value::as_str) == Some(grant.destination.as_str())
            });
        let preflight =
            if handler.is_none() || (selected.capability.effects.open_world && context.selection.offline && !selected.binding.offline_safe) {
                Some(ExecutionOutcome::Unavailable)
            } else if !context.grounded {
                Some(ExecutionOutcome::Withheld)
            } else if !arguments.is_object()
                || !valid_schema(&selected.capability.input_schema, &arguments)
                || context.max_input.can_contain(u64::try_from(input_size).unwrap_or(u64::MAX)) != Some(true)
            {
                Some(ExecutionOutcome::InvalidInput)
            } else if context.preview {
                Some(ExecutionOutcome::Proposed)
            } else if !authorized {
                Some(ExecutionOutcome::PermissionRequired)
            } else if context.deadline.is_some_and(|deadline| deadline <= Instant::now()) {
                Some(ExecutionOutcome::TimedOut)
            } else {
                None
            };
        match (preflight, handler) {
            | (Some(outcome), _) => RoutedExecution::new(selected, outcome, false),
            | (None, None) => RoutedExecution::new(selected, ExecutionOutcome::Unavailable, false),
            | (None, Some(handler)) => {
                let deadline = async move {
                    match context.deadline {
                        | Some(deadline) => tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await,
                        | None => future::pending::<()>().await,
                    }
                };
                let cancellation = async move {
                    match context.cancellation {
                        | Some(cancel) => cancel.await,
                        | None => future::pending::<()>().await,
                    }
                };
                let mut call = handler.start(arguments);
                tokio::select! {
                    biased;
                    _ = cancellation => {
                        cancel_call(&mut call).await;
                        RoutedExecution::new(selected, ExecutionOutcome::Cancelled, true)
                    },
                    _ = deadline => {
                        cancel_call(&mut call).await;
                        RoutedExecution::new(selected, ExecutionOutcome::TimedOut, true)
                    },
                    result = call.future.as_mut() => match result {
                        Err(_) => RoutedExecution::new(selected, ExecutionOutcome::ToolFailed, true),
                        Ok(value) => {
                            let output_size = serde_json::to_vec(&value).map_or(usize::MAX, |encoded| encoded.len());
                            if context.max_output.can_contain(u64::try_from(output_size).unwrap_or(u64::MAX)) != Some(true)
                                || !valid_schema(&selected.capability.output_schema, &value)
                            {
                                RoutedExecution::new(selected, ExecutionOutcome::ToolFailed, true)
                            } else {
                                let mut completed = RoutedExecution::new(selected, ExecutionOutcome::Completed, true);
                                completed.artifacts = artifacts(&value);
                                completed.content = Some(value);
                                completed
                            }
                        }
                    },
                }
            }
        }
    }
}
fn artifacts(value: &Value) -> Vec<ArtifactReference> {
    value
        .get("artifacts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(128)
        .filter_map(|artifact| artifact.as_str().or_else(|| artifact.get("uri").and_then(Value::as_str)))
        .filter(|uri| !uri.is_empty() && uri.len() <= 1024)
        .map(|uri| ArtifactReference { uri: uri.to_string() })
        .collect()
}
async fn cancel_call(call: &mut HandlerCall) {
    if let Some(cancellation) = call.cancellation.take() {
        let _ = cancellation.send(());
        let _ = tokio::time::timeout(COOPERATIVE_CLEANUP_TIMEOUT, call.future.as_mut()).await;
    }
}
fn valid_schema(schema: &Value, value: &Value) -> bool {
    jsonschema::validator_for(schema).is_ok_and(|validator| validator.is_valid(value))
}
