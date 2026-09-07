//! Managed [Needle 2](https://cactuscompute.com/needle) inference sidecar.
//!
//! Needle 2 is a compact local model specialized for selecting and calling tools. ACORN runs the
//! pinned standalone engine from the [upstream project](https://github.com/cactus-compute/needle);
//! see the [ACORN Needle guide](https://acorn.ornl.gov/inference/needle.html) for usage details.
pub mod assets;
mod process;
#[cfg(test)]
mod tests;

use crate::io::api::needle::{Client, CompleteResponse};
use crate::io::mcp::{ToolCallContext, ToolRegistry};
use crate::io::sidecar::Sidecar;
use crate::io::{standard_project_folder, write_file, ApiResult};
use crate::util::constants::app::{DEFAULT_NEEDLE_CONFIDENCE_THRESHOLD, DEFAULT_NEEDLE_MAX_ROUNDS};
use acorn_core::prelude::{create_dir, PathBuf, String, Vec};
use acorn_schema::agent::tools::ToolResult;
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use futures::stream::{self, StreamExt};
use process::Runner;
use serde::{Deserialize, Serialize};

/// Result of a bounded Needle inference loop.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceResult {
    /// Final native Needle response.
    pub response: CompleteResponse,
    /// Tool results executed during the loop.
    pub results: Vec<ToolResult>,
}
/// Running Needle 2 sidecar bound to one immutable tool catalog.
#[derive(Debug)]
pub struct Needle {
    runner: Runner,
    client: Client,
    registry: ToolRegistry,
    confidence_threshold: f64,
    max_rounds: usize,
    offline: bool,
}
impl Needle {
    /// Override the confidence threshold, constrained to the inclusive unit interval
    pub fn with_confidence_threshold(self, threshold: f64) -> ApiResult<Self> {
        match (0.0..=1.0).contains(&threshold) {
            | true => Ok(Self {
                confidence_threshold: threshold,
                ..self
            }),
            | false => Err(eyre!("Needle confidence threshold must be between 0 and 1")),
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
    type Output = InferenceResult;

    async fn start(custom_location: Option<&str>, offline: bool) -> ApiResult<Self> {
        let catalog = ToolRegistry::acorn().and_then(|registry| {
            serde_json::to_string_pretty(&registry.needle_tools())
                .map(|catalog| (registry, catalog))
                .map_err(|why| eyre!("Failed to serialize Needle tools — {why}"))
        });
        let start = match catalog {
            | Ok((registry, catalog)) => match assets::resolve(custom_location, offline, true).await {
                | Ok(resolved) => resolved
                    .validate_tools(&registry.needle_tools())
                    .and_then(|resolved| assets::tool_index(&catalog).map(|tool_index| (resolved, tool_index)))
                    .and_then(|(resolved, tool_index)| {
                        let session_root = standard_project_folder("needle/sessions", None);
                        let tools = session_root.join("tools.json");
                        create_dir(&session_root)
                            .map_err(|why| eyre!("Failed to create Needle session directory — {why}"))
                            .and_then(|()| write_file(&tools, catalog.clone()))
                            .map(|()| (resolved, tool_index, session_root, tools))
                    })
                    .map(|(resolved, tool_index, session_root, tools)| async move {
                        match Runner::start(&resolved, &tools, &tool_index, session_root).await {
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
            },
            | Err(why) => Err(why),
        };
        match start {
            | Ok(future) => future.await,
            | Err(why) => Err(why),
        }
    }
    async fn infer(self, prompt: String) -> Option<ApiResult<InferenceResult>> {
        let mut input = prompt;
        let mut results = Vec::new();
        let mut round = 0_usize;
        Some(loop {
            round = round.saturating_add(1);
            if round > self.max_rounds {
                break Err(eyre!("Needle inference exceeded the {}-round limit", self.max_rounds));
            }
            match self.client.complete(input).await {
                | Ok(response) if response.error.is_some() => {
                    break Err(eyre!("Needle inference failed — {}", response.error.unwrap_or_default()));
                }
                | Ok(response) if response.function_calls.is_empty() => break Ok(InferenceResult { response, results }),
                | Ok(response) => {
                    let confidence = response
                        .confidence
                        .ok_or_else(|| eyre!("Needle response has no calibrated confidence; automatic execution is disabled"));
                    match confidence {
                        | Ok(confidence) if confidence >= self.confidence_threshold => {
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
                        | Ok(confidence) => {
                            break Err(eyre!(
                                "Needle confidence {confidence:.3} is below the {:.3} execution threshold",
                                self.confidence_threshold
                            ));
                        }
                        | Err(why) => break Err(why),
                    }
                }
                | Err(why) => break Err(why),
            }
        })
    }
}
/// Write the canonical Needle catalog to a file.
pub fn write_tools(path: PathBuf) -> ApiResult<()> {
    ToolRegistry::acorn()
        .and_then(|registry| serde_json::to_string_pretty(&registry.needle_tools()).map_err(|why| eyre!("Failed to serialize Needle tools — {why}")))
        .and_then(|content| write_file(path, content))
}
