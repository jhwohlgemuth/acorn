//! Managed Needle 2 inference sidecar.

pub mod assets;
mod process;

use crate::io::api::needle::{Client, CompleteResponse};
use crate::io::{standard_project_folder, write_file, ApiResult};
use crate::tools::{ToolCallContext, ToolRegistry};
use acorn_core::prelude::{String, Vec};
use acorn_schema::schema::agent::tools::ToolResult;
use color_eyre::eyre::eyre;
use process::Runner;
use serde::{Deserialize, Serialize};
use std::fs::create_dir;
use std::path::PathBuf;

/// Default minimum confidence required before ACORN executes a Needle call.
pub const DEFAULT_CONFIDENCE_THRESHOLD: f64 = 0.8;
/// Default maximum tool-selection rounds in one inference request.
pub const DEFAULT_MAX_ROUNDS: usize = 8;

/// Result of a bounded Needle inference loop.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceResult {
    /// Final native Needle response.
    pub response: CompleteResponse,
    /// Tool results executed during the loop.
    pub results: Vec<ToolResult>,
}

/// Running Needle sidecar bound to one immutable tool catalog.
#[derive(Debug)]
pub struct Sidecar {
    _runner: Runner,
    client: Client,
    registry: ToolRegistry,
    confidence_threshold: f64,
    max_rounds: usize,
    offline: bool,
}

impl Sidecar {
    /// Resolve assets, write the canonical catalog, and start Needle.
    pub async fn start(custom_location: Option<&str>, offline: bool) -> ApiResult<Self> {
        let catalog = ToolRegistry::acorn().and_then(|registry| {
            serde_json::to_string_pretty(&registry.needle_tools())
                .map(|catalog| (registry, catalog))
                .map_err(|why| eyre!("Failed to serialize Needle tools — {why}"))
        });
        let start = match catalog {
            | Ok((registry, catalog)) => match assets::resolve(custom_location, offline).await {
                | Ok(resolved) => assets::validate_tools(&resolved, &registry.needle_tools())
                    .and_then(|()| assets::tool_index(&catalog))
                    .and_then(|tool_index| {
                        let session_root = standard_project_folder("needle/sessions", None);
                        let tools = session_root.join("tools.json");
                        create_dir(&session_root)
                            .map_err(|why| eyre!("Failed to create Needle session directory — {why}"))
                            .and_then(|()| write_file(&tools, catalog.clone()))
                            .map(|()| (tool_index, session_root, tools))
                    })
                    .map(|(tool_index, session_root, tools)| async move {
                        match Runner::start(&resolved, &tools, &tool_index, session_root).await {
                            | Ok(runner) => {
                                let endpoint = runner.endpoint().to_string();
                                Client::new(endpoint).map(|client| Self {
                                    _runner: runner,
                                    client,
                                    registry,
                                    confidence_threshold: DEFAULT_CONFIDENCE_THRESHOLD,
                                    max_rounds: DEFAULT_MAX_ROUNDS,
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

    /// Override the confidence threshold, constrained to the inclusive unit interval.
    pub fn with_confidence_threshold(self, confidence_threshold: f64) -> ApiResult<Self> {
        match (0.0..=1.0).contains(&confidence_threshold) {
            | true => Ok(Self {
                confidence_threshold,
                ..self
            }),
            | false => Err(eyre!("Needle confidence threshold must be between 0 and 1")),
        }
    }

    /// Override the positive maximum inference rounds.
    pub fn with_max_rounds(self, max_rounds: usize) -> ApiResult<Self> {
        match max_rounds {
            | 0 => Err(eyre!("Needle maximum rounds must be greater than zero")),
            | _ => Ok(Self { max_rounds, ..self }),
        }
    }

    /// Execute a bounded, confidence-gated tool loop.
    pub async fn infer(&mut self, prompt: impl Into<String>) -> ApiResult<InferenceResult> {
        let mut input = prompt.into();
        let mut results = Vec::new();
        let mut round = 0_usize;
        loop {
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
                            let called = response
                                .function_calls
                                .iter()
                                .map(|call| {
                                    self.registry
                                        .call_with_context(&call.name, call.arguments.clone(), ToolCallContext::needle(self.offline))
                                })
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
        }
    }
}

/// Serialize the canonical Needle catalog for CLI and bundle use.
pub fn tools_json() -> ApiResult<String> {
    ToolRegistry::acorn()
        .and_then(|registry| serde_json::to_string_pretty(&registry.needle_tools()).map_err(|why| eyre!("Failed to serialize Needle tools — {why}")))
}

/// Write the canonical Needle catalog to a file.
pub fn write_tools(path: PathBuf) -> ApiResult<()> {
    tools_json().and_then(|content| write_file(path, content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_json_is_deterministic_and_native() {
        let first = tools_json().expect("first export");
        let second = tools_json().expect("second export");
        assert_eq!(first, second);
        let value: serde_json::Value = serde_json::from_str(&first).expect("tools JSON");
        assert!(value.as_array().is_some_and(|tools| {
            tools
                .iter()
                .all(|tool| tool.get("parameters").is_some() && tool.get("input_schema").is_none())
        }));
    }
}
