//! VS Code custom endpoint synchronization configuration types
use super::{McpSync, Options, RenderedOutput, SyncTarget};
use crate::io::api;
use crate::io::{read_file, ApiResult};
use crate::util::constants::app::{APPLICATION, DEFAULT_VSCODE_CONFIG_PATH};
use crate::util::StringConversion;
use acorn_schema::agent::ModelDetails;
use acorn_schema::validation::Validate;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use color_eyre::eyre::eyre;
use directories::BaseDirs;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;
use std::env::current_dir;
use std::path::Path;
use std::path::PathBuf;
use veil::Redact;

/// Configuration for synchronizing models into VS Code.
#[skip_serializing_none]
#[derive(Clone, Deserialize, Serialize, Validate, Redact)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Path to `chatLanguageModels.json` on disk.
    #[serde(skip_serializing)]
    #[validate(length(min = 1))]
    pub path: Option<String>,
    /// Path to workspace `mcp.json` on disk.
    #[serde(skip_serializing)]
    #[validate(length(min = 1))]
    pub mcp_path: Option<String>,
    /// Full OpenAI-compatible chat completions endpoint URL.
    #[serde(default = "default_url")]
    #[validate(url)]
    pub url: String,
    /// Human-readable provider group name.
    #[serde(default = "default_provider_name")]
    #[validate(length(min = 1))]
    pub provider_name: String,
    /// API key or VS Code input variable used by the endpoint.
    #[serde(default, skip_serializing)]
    #[redact]
    pub api_key: Option<api::Secret>,
    /// Default maximum input tokens advertised for each model.
    #[serde(default = "default_max_input_tokens")]
    #[validate(range(min = 1))]
    pub max_input_tokens: u64,
    /// Default maximum output tokens advertised for each model.
    #[serde(default = "default_max_output_tokens")]
    #[validate(range(min = 1))]
    pub max_output_tokens: u64,
}
impl Config {
    pub(crate) fn render_mcp(&self, options: Options<'_>) -> ApiResult<RenderedOutput> {
        options
            .executable
            .ok_or_else(|| eyre!("MCP synchronization requires an absolute ACORN executable"))
            .and_then(|executable| {
                self.mcp_path
                    .as_ref()
                    .map(PathBuf::from)
                    .map_or_else(
                        || {
                            current_dir()
                                .map(|directory| directory.join(".vscode/mcp.json"))
                                .map_err(|why| eyre!(why))
                        },
                        Ok,
                    )
                    .map(|path| (executable, path))
            })
            .and_then(|(executable, path)| {
                path.is_file()
                    .then(|| read_file(path.clone()))
                    .transpose()
                    .map(|content| (executable, path, content.unwrap_or_default()))
            })
            .and_then(|(executable, path, before)| {
                match before.is_empty() {
                    | true => Ok(Value::Object(Map::new())),
                    | false => serde_json::from_str(&before).map_err(|why| eyre!("Failed to parse existing VS Code MCP config: {why}")),
                }
                .and_then(|existing| self.upsert_mcp(existing, executable, options.prune))
                .and_then(|updated| {
                    serde_json::to_string_pretty(&updated)
                        .map(|content| format!("{content}\n"))
                        .map_err(|why| eyre!("Failed to serialize VS Code MCP config: {why}"))
                })
                .map(|content| RenderedOutput {
                    target: "VS Code MCP",
                    path,
                    before,
                    content,
                })
            })
    }
    /// Upsert the managed custom endpoint provider while preserving unrelated providers and properties.
    pub fn upsert(&self, existing: Value, models: &[ModelDetails], prune: bool) -> ApiResult<Value> {
        match existing {
            | Value::Array(providers) => Ok(providers),
            | _ => Err(eyre!("VS Code language-model configuration must be a JSON array")),
        }
        .map(|mut providers| {
            let managed_index = providers.iter().position(|provider| {
                provider.get("vendor").and_then(Value::as_str) == Some("customendpoint")
                    && provider.get("name").and_then(Value::as_str) == Some(self.provider_name.as_str())
            });
            let mut provider = managed_index
                .and_then(|index| providers.get(index))
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            let current_ids = models
                .iter()
                .filter_map(|model| model.id.as_ref())
                .map(String::as_str)
                .collect::<Vec<_>>();
            let retained = provider
                .get("models")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|model| !prune && model.get("id").and_then(Value::as_str).is_some_and(|id| !current_ids.contains(&id)))
                .cloned();
            let entries = retained
                .chain(models.iter().filter_map(|model| self.model_entry(model)))
                .collect::<Vec<_>>();
            provider.insert("name".to_string(), Value::String(self.provider_name.clone()));
            provider.insert("vendor".to_string(), Value::String("customendpoint".to_string()));
            provider.insert("apiType".to_string(), Value::String("chat-completions".to_string()));
            provider.insert("models".to_string(), Value::Array(entries));
            if let Some(api_key) = self.api_key.as_ref() {
                provider.insert("apiKey".to_string(), Value::String(ExposeSecret::expose_secret(api_key).to_string()));
            } else if !provider.contains_key("apiKey") {
                provider.insert("apiKey".to_string(), Value::String("none".to_string()));
            }
            let value = Value::Object(provider);
            match managed_index.and_then(|index| providers.get_mut(index)) {
                | Some(existing) => *existing = value,
                | None => providers.push(value),
            }
            Value::Array(providers)
        })
    }
    fn model_entry(&self, model: &ModelDetails) -> Option<Value> {
        model.id.as_ref().map(|id| {
            let output = model.limit.as_ref().and_then(|limit| limit.output).unwrap_or(self.max_output_tokens);
            let input = model.limit.as_ref().map_or(self.max_input_tokens, |limit| {
                limit.input.unwrap_or_else(|| limit.context.saturating_sub(output).max(1))
            });
            Value::Object(
                [
                    ("id".to_string(), Value::String(id.clone())),
                    ("name".to_string(), Value::String(model.name.as_ref().unwrap_or(id).clone())),
                    ("url".to_string(), Value::String(self.url.clone())),
                    ("toolCalling".to_string(), Value::Bool(model.tool_call.unwrap_or(true))),
                    ("vision".to_string(), Value::Bool(false)),
                    ("maxInputTokens".to_string(), Value::Number(input.into())),
                    ("maxOutputTokens".to_string(), Value::Number(output.into())),
                ]
                .into_iter()
                .collect::<Map<_, _>>(),
            )
        })
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            path: None,
            mcp_path: None,
            url: default_url(),
            provider_name: default_provider_name(),
            api_key: None,
            max_input_tokens: default_max_input_tokens(),
            max_output_tokens: default_max_output_tokens(),
        }
    }
}
impl McpSync for Config {
    type Document = Value;
    const TARGET: &'static str = "VS Code";
    fn update_mcp(&self, existing: Value, executable: &Path, prune: bool) -> ApiResult<Value> {
        match existing {
            | Value::Object(mut root) => {
                let mut servers = root.remove("servers").and_then(|value| value.as_object().cloned()).unwrap_or_default();
                if prune {
                    servers.remove(APPLICATION);
                } else {
                    servers.insert(
                        APPLICATION.to_string(),
                        Value::Object(
                            [
                                (
                                    "args".to_string(),
                                    Value::Array(vec![Value::String("serve".to_string()), Value::String("mcp".to_string())]),
                                ),
                                ("command".to_string(), Value::String(executable.display().to_string())),
                                ("type".to_string(), Value::String("stdio".to_string())),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    );
                }
                if !servers.is_empty() {
                    root.insert("servers".to_string(), Value::Object(servers));
                }
                Ok(Value::Object(root))
            }
            | _ => Err(eyre!("VS Code MCP configuration must be a JSON object")),
        }
    }
}
impl SyncTarget for Config {
    const COMMAND: &'static str = "code";
    fn merge(self, overrides: Self) -> Self {
        Self {
            path: overrides.path.or(self.path),
            mcp_path: overrides.mcp_path.or(self.mcp_path),
            url: overrides.url,
            provider_name: overrides.provider_name,
            api_key: overrides.api_key.or(self.api_key),
            max_input_tokens: overrides.max_input_tokens,
            max_output_tokens: overrides.max_output_tokens,
        }
    }
    fn merge_cli_overrides(self, overrides: Self) -> Self {
        Self {
            path: overrides.path.or(self.path),
            mcp_path: overrides.mcp_path.or(self.mcp_path),
            ..self
        }
    }
    fn resolve_path(explicit: Option<&str>) -> ApiResult<PathBuf> {
        explicit.map(|path| PathBuf::from(path.to_string().to_cross_platform_path())).map_or_else(
            || {
                BaseDirs::new()
                    .map(|directories| directories.config_dir().join(DEFAULT_VSCODE_CONFIG_PATH))
                    .ok_or_else(|| eyre!("Failed to resolve platform configuration directory"))
            },
            Ok,
        )
    }
    fn render(&self, options: Options<'_>) -> ApiResult<RenderedOutput> {
        Self::resolve_path(self.path.as_deref()).and_then(|path| {
            path.is_file()
                .then(|| read_file(path.clone()))
                .transpose()
                .map(|content| content.unwrap_or_default())
                .and_then(|before| {
                    match before.is_empty() {
                        | true => Ok(Value::Array(Vec::new())),
                        | false => {
                            serde_json::from_str(&before).map_err(|why| eyre!("Failed to parse existing VS Code language-model config: {why}"))
                        }
                    }
                    .and_then(|existing| self.upsert(existing, options.models, options.prune))
                    .and_then(|updated| {
                        serde_json::to_string_pretty(&updated)
                            .map(|content| format!("{content}\n"))
                            .map_err(|why| eyre!("Failed to serialize VS Code language-model config: {why}"))
                    })
                    .map(|content| RenderedOutput {
                        target: "VS Code",
                        path,
                        before,
                        content,
                    })
                })
        })
    }
}
const fn default_max_input_tokens() -> u64 {
    28_672
}
const fn default_max_output_tokens() -> u64 {
    4_096
}
fn default_provider_name() -> String {
    "Local (llama-swap)".to_string()
}
fn default_url() -> String {
    "http://localhost:8080/v1/chat/completions".to_string()
}
