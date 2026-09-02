//! Native client and data contract for the Needle 2 localhost runner.

use super::localhost;
use crate::io::ApiResult;
use acorn_core::prelude::{String, Vec};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One completion request accepted by the standalone Needle server.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompleteRequest {
    /// User query or serialized tool result for the next inference turn.
    pub input: String,
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

/// Native response returned by Needle's `/complete` endpoint.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct CompleteResponse {
    /// Response kind such as `call`, `respond`, or `refuse`.
    #[serde(rename = "type")]
    pub response_type: String,
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
    /// Short unconstrained derivation emitted by Needle.
    pub reasoning: Option<String>,
    /// Calibrated confidence for base-model responses.
    pub confidence: Option<f64>,
    /// Prompt-processing throughput.
    pub prefill_tps: Option<f64>,
    /// Decode throughput.
    pub decode_tps: Option<f64>,
    /// Peak session memory reported by the runner.
    pub peak_ram_mb: Option<f64>,
}

/// Typed client for a locally bound Needle server.
#[derive(Clone, Debug)]
pub struct Client {
    localhost: localhost::Client,
}

impl Client {
    /// Connect to a validated loopback Needle endpoint.
    pub fn new(base_url: impl Into<String>) -> ApiResult<Self> {
        localhost::Client::new(base_url).map(|localhost| Self { localhost })
    }

    /// Complete one Needle inference turn.
    pub async fn complete(&self, input: impl Into<String>) -> ApiResult<CompleteResponse> {
        self.localhost.post_json("/complete", &CompleteRequest { input: input.into() }).await
    }

    /// Return the validated endpoint URL.
    pub fn endpoint(&self) -> &str {
        self.localhost.base_url()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_documented_native_response() {
        let response: CompleteResponse = serde_json::from_value(serde_json::json!({
            "type": "call",
            "success": true,
            "error": null,
            "error_code": null,
            "function_calls": [{ "name": "acorn.version", "arguments": {} }],
            "reasoning": "version requested",
            "confidence": 0.94,
            "prefill_tps": 4300.0,
            "decode_tps": 850.0,
            "peak_ram_mb": 28.0
        }))
        .expect("native response");
        assert_eq!(response.response_type, "call");
        assert_eq!(response.function_calls.first().map(|call| call.name.as_str()), Some("acorn.version"));
    }
}
