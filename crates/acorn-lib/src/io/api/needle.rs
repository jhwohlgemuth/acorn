//! Native client and data contract for the Needle 3 localhost runner
use super::localhost;
use crate::io::ApiResult;
use acorn_core::prelude::{String, Vec};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::{Display, EnumIs};

/// Kind of response returned by Needle's `/complete` endpoint.
#[derive(Clone, Copy, Debug, Deserialize, Display, EnumIs, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ResponseType {
    /// One or more tool calls were selected.
    Call,
    /// The request was refused.
    Refuse,
    /// A final response was generated.
    Respond,
}
/// Typed client for a locally bound Needle server
#[derive(Clone, Debug)]
pub struct Client {
    localhost: localhost::Client,
}
/// One completion request accepted by the standalone Needle server.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompleteRequest {
    /// User query or serialized tool result for the next inference turn.
    pub input: String,
}
/// Native response returned by Needle's `/complete` endpoint.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct CompleteResponse {
    /// Response kind such as `call`, `respond`, or `refuse`.
    #[serde(rename = "type")]
    pub response_type: ResponseType,
    /// Whether inference completed successfully.
    #[serde(default)]
    pub success: bool,
    /// Human-readable inference failure.
    pub error: Option<String>,
    /// Stable upstream failure code when present.
    pub error_code: Option<String>,
    /// Selected function calls.
    #[serde(default)]
    pub function_calls: Vec<FunctionCall>,
    /// Stable machine-readable reason for a refusal or validation decision.
    pub reason: Option<String>,
    /// Short unconstrained derivation emitted by Needle.
    pub reasoning: Option<String>,
    /// Calibrated confidence for base-model responses.
    pub confidence: Option<f64>,
    /// Calls withheld by Needle's confidence or grounding policy.
    #[serde(default)]
    pub suppressed_calls: Vec<FunctionCall>,
    /// Prompt-processing throughput.
    pub prefill_tps: Option<f64>,
    /// Decode throughput.
    pub decode_tps: Option<f64>,
    /// Peak session memory reported by the runner.
    pub peak_ram_mb: Option<f64>,
    /// Grounding and negation checks reported by Needle.
    pub validation: Option<ResponseValidation>,
}
/// Function selected by Needle with grammar-constrained arguments.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionCall {
    /// Registered function name.
    pub name: String,
    /// Function arguments.
    #[serde(default)]
    pub arguments: Value,
}
/// Validation metadata attached to a Needle 3 completion.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ResponseValidation {
    /// Argument fields that are not grounded in the conversation.
    #[serde(default)]
    pub ungrounded: Vec<String>,
    /// Whether the selected call conflicts with a negated request.
    #[serde(default)]
    pub negation: bool,
}
impl Client {
    /// Connect to a validated loopback Needle endpoint
    pub fn new(base_url: impl Into<String>) -> ApiResult<Self> {
        localhost::Client::new(base_url).map(|localhost| Self { localhost })
    }
    /// Complete one Needle inference turn
    pub async fn complete(&self, input: impl Into<String>) -> ApiResult<CompleteResponse> {
        self.localhost.post_json("/complete", &CompleteRequest { input: input.into() }).await
    }
    /// Return the validated endpoint URL
    pub fn endpoint(&self) -> &str {
        self.localhost.base_url()
    }
}
