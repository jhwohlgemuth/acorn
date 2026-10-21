//! Transport-neutral prompt preparation and inference contracts.

use crate::error::ApiResult;
use acorn_core::{
    prelude::{Box, String},
    time::{Milliseconds, UnitOfTime},
};
use acorn_macros::With;
use acorn_schema::agent::{FrontMatter, PromptFileAsset, PromptTemplateConfiguration};
use core::future::Future;
use core::pin::Pin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
mod chat;
pub use chat::{AttachedTarget, ChatAction, ChatContext, ChatEvent, ChatRole, ChatState, ChatTransition, ChatTurn};
#[cfg(feature = "template")]
mod prompt;
#[cfg(feature = "template")]
pub use prompt::PromptHydrator;
/// Future returned by an inference backend.
pub type InferenceFuture<'a> = Pin<Box<dyn Future<Output = ApiResult<InferenceResult>> + Send + 'a>>;
/// Consumer-owned interface implemented by inference transports.
pub trait InferenceBackend: Send + Sync {
    /// Report the stable capabilities available from this backend.
    fn capabilities(&self) -> InferenceCapabilities;
    /// Run one inference request.
    fn infer<'a>(&'a self, request: &'a InferenceRequest) -> InferenceFuture<'a>;
}
/// Normalized reason an inference request finished.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, strum::EnumIs)]
#[serde(rename_all = "kebab-case")]
pub enum InferenceFinishReason {
    /// The backend reached a configured output limit.
    Length,
    /// Backend-specific reason that has no stable mapping.
    Other(String),
    /// The backend refused to process the request.
    Refusal,
    /// The backend completed normally.
    Stop,
}
/// Permission policy applied to an inference request.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InferencePermissionPolicy {
    /// Deny filesystem writes, commands, and other mutating tools.
    DenyMutation,
}
/// Serialization format used for the complete prompt input-data block.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptDataFormat {
    /// Pretty-printed JSON, intended primarily for troubleshooting.
    Json,
    /// Canonical ZON, used by default for compact model-facing data.
    #[default]
    Zon,
}
/// Fully rendered prompt ready for an inference backend.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HydratedPrompt {
    /// Embedded asset that produced this prompt.
    pub asset: PromptFileAsset,
    /// Rendered prompt body without frontmatter.
    pub body: String,
    /// Validated metadata extracted from rendered frontmatter.
    pub metadata: FrontMatter,
}
/// Stable inference capabilities shared across transports.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceCapabilities {
    /// Whether binary or document attachments are supported.
    pub attachments: bool,
    /// Whether the backend can expose mutating tools.
    pub mutating_tools: bool,
    /// Whether incremental output events are supported.
    pub streaming: bool,
    /// Whether the backend can enforce a supplied output schema.
    pub structured_output: bool,
    /// Whether tool calls are supported.
    pub tools: bool,
}
/// Secret-free provenance attached to an inference result.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceProvenance {
    /// Stable backend kind such as `memory`, `acp`, or `openai`.
    pub backend: String,
    /// Agent identifier when an agent transport was used.
    pub agent: Option<String>,
    /// Model identifier reported or selected for the request.
    pub model: Option<String>,
    /// Secret-free request or session correlation identifier.
    pub request_id: Option<String>,
}
/// Secret-free result of a bounded inference readiness request.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceReadiness {
    /// Capabilities reported by the configured backend.
    pub capabilities: InferenceCapabilities,
    /// Normalized completion reason.
    pub finish_reason: InferenceFinishReason,
    /// Secret-free backend provenance.
    pub provenance: InferenceProvenance,
    /// Whether the backend completed the bounded request without refusal.
    pub ready: bool,
}
/// Transport-neutral one-shot inference request.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize, With)]
#[serde(deny_unknown_fields)]
pub struct InferenceRequest {
    /// Optional agent selection understood by the configured backend.
    pub agent: Option<String>,
    /// Optional model selection understood by the configured backend.
    pub model: Option<String>,
    /// Permission policy for tools requested during inference.
    pub permission_policy: InferencePermissionPolicy,
    /// Fully rendered prompt document.
    pub prompt: HydratedPrompt,
    /// Optional JSON Schema for structured output.
    pub response_schema: Option<Value>,
    /// Request timeout.
    pub timeout: Milliseconds,
}
/// Transport-neutral inference result.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceResult {
    /// Stable completion reason.
    pub finish_reason: InferenceFinishReason,
    /// Secret-free backend provenance.
    pub provenance: InferenceProvenance,
    /// Parsed structured output when available.
    pub structured: Option<Value>,
    /// Final text assembled by the backend.
    pub text: String,
    /// Usage reported by the backend when available.
    pub usage: Option<InferenceUsage>,
}
/// Normalized token usage reported by an inference backend.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceUsage {
    /// Number of input tokens processed.
    pub input_tokens: Option<u64>,
    /// Number of output tokens produced.
    pub output_tokens: Option<u64>,
}
/// Deterministic backend used to test inference orchestration without I/O.
#[derive(Clone, Debug)]
pub struct InMemoryBackend {
    capabilities: InferenceCapabilities,
    result: InferenceResult,
}
/// Namespaced values supplied to a prompt template.
#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PromptContext<T> {
    /// Inference and prompt behavior configuration.
    pub config: PromptTemplateConfiguration,
    /// Caller-owned prompt input data.
    pub data: T,
}
impl InferenceReadiness {
    /// Run one bounded request and return only a secret-free readiness summary.
    pub async fn run<B>(backend: &B, request: &InferenceRequest) -> ApiResult<Self>
    where
        B: InferenceBackend,
    {
        run_inference(backend, request).await.map(|result| {
            let ready = !result.finish_reason.is_refusal();
            Self {
                capabilities: backend.capabilities(),
                finish_reason: result.finish_reason,
                provenance: result.provenance,
                ready,
            }
        })
    }
}
impl InferenceBackend for InMemoryBackend {
    fn capabilities(&self) -> InferenceCapabilities {
        self.capabilities
    }
    fn infer<'a>(&'a self, _request: &'a InferenceRequest) -> InferenceFuture<'a> {
        Box::pin(core::future::ready(Ok(self.result.clone())))
    }
}
impl InMemoryBackend {
    /// Construct a backend that returns one deterministic result.
    pub fn new(result: InferenceResult) -> Self {
        Self {
            capabilities: InferenceCapabilities::default(),
            result,
        }
    }
    /// Replace the capabilities reported by this backend.
    pub fn with_capabilities(self, capabilities: InferenceCapabilities) -> Self {
        Self { capabilities, ..self }
    }
}
/// Validate and run one transport-neutral inference request.
pub fn run_inference<'a, B>(backend: &'a B, request: &'a InferenceRequest) -> InferenceFuture<'a>
where
    B: InferenceBackend,
{
    match (request.prompt.body.trim().is_empty(), request.timeout.is_zero()) {
        | (true, _) => Box::pin(core::future::ready(Err(color_eyre::eyre::eyre!("Inference prompt body cannot be empty")))),
        | (_, true) => Box::pin(core::future::ready(Err(color_eyre::eyre::eyre!(
            "Inference timeout must be greater than zero"
        )))),
        | (false, false) => backend.infer(request),
    }
}

#[cfg(test)]
mod tests;
