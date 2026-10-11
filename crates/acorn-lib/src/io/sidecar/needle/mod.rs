//! Managed [Needle 3](https://cactuscompute.com/needle) inference sidecar.
//!
//! Needle 3 is a compact local model specialized for selecting and calling tools. ACORN runs the
//! pinned standalone engine from the [upstream project](https://github.com/cactus-compute/needle);
//! see the [ACORN Needle guide](https://acorn.ornl.gov/inference/needle.html) for usage details.
pub mod assets;
mod process;

use crate::io::api::needle::{Client, CompleteResponse};
use crate::io::mcp::{ToolCallContext, ToolRegistry};
use crate::io::sidecar::{ResolvedAssets, Sidecar, SidecarAssets};
use crate::io::{standard_project_folder, write_file, ApiResult};
use crate::util::constants::app::{DEFAULT_NEEDLE_CONFIDENCE_THRESHOLD, DEFAULT_NEEDLE_MAX_ROUNDS};
use acorn_core::prelude::{create_dir, PathBuf, String, Vec};
use acorn_core::Noul;
use acorn_schema::agent::tools::ToolResult;
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use futures::stream::{self, StreamExt};
use process::Runner;
use serde::{Deserialize, Serialize};

const MAX_CATALOG_BYTES: usize = 32 * 1024;
/// Result of a bounded Needle inference loop.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceResult {
    /// Final native Needle response.
    pub response: CompleteResponse,
    /// Tool results executed during the loop.
    pub results: Vec<ToolResult>,
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
impl CompleteResponse {
    fn execution_confidence(&self, threshold: Noul) -> ApiResult<()> {
        let valid_type = self.response_type.is_call();
        let confidence = self
            .confidence
            .and_then(|confidence| Noul::new(confidence).ok())
            .ok_or_else(|| eyre!("Needle response has no valid calibrated confidence; automatic execution is disabled"));
        match (valid_type, self.validation.as_ref(), confidence) {
            | (false, _, _) => Err(eyre!("Needle response type '{}' cannot contain function calls", self.response_type)),
            | (_, Some(validation), _) if validation.negation => Err(eyre!("Needle rejected automatic execution because the request was negated")),
            | (_, Some(validation), _) if !validation.ungrounded.is_empty() => Err(eyre!(
                "Needle rejected automatic execution because these fields were not grounded: {}",
                validation.ungrounded.join(", ")
            )),
            | (_, _, Ok(confidence)) if confidence < threshold => {
                Err(eyre!("Needle confidence {confidence:.3} is below the {threshold:.3} execution threshold"))
            }
            | (_, _, confidence) => confidence.map(|_| ()),
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
}
#[async_trait]
impl Sidecar for Needle {
    type Input = String;
    type Output = InferenceResult;
    type ToolIndex = assets::NeedleToolIndex;

    async fn start_with_assets(resolved: ResolvedAssets, offline: bool) -> ApiResult<Self> {
        let catalog = ToolRegistry::acorn().and_then(|registry| registry.serialize_catalog().map(|catalog| (registry, catalog)));
        let start = match catalog {
            | Ok((registry, catalog)) => assets::validate_tools::<Self, _>(resolved, &registry.needle_tools())
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
                | Ok(response) if response.function_calls.is_empty() => break Ok(InferenceResult { response, results }),
                | Ok(response) => {
                    let confidence = response.execution_confidence(self.confidence_threshold);
                    match confidence {
                        | Ok(_) => {
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
                                        | results => {
                                            serde_json::Value::Array(results.iter().map(|result| result.structured_content.clone()).collect())
                                        }
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
                        | Err(why) => break Err(why),
                    }
                }
                | Err(why) => break Err(why),
            }
        }
    }
}
impl ToolRegistry {
    fn serialize_catalog(&self) -> ApiResult<String> {
        serde_json::to_string_pretty(&self.needle_tools())
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
        .and_then(|registry| registry.serialize_catalog())
        .and_then(|content| write_file(path, content))
}

#[cfg(test)]
mod tests;
