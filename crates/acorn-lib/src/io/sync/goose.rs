//! Goose CLI YAML synchronization configuration types
use super::{McpSync, Options, RenderedOutput, SyncTarget};
#[cfg(not(target_os = "windows"))]
use crate::io::home_directory;
#[cfg(not(target_os = "windows"))]
use crate::util::constants::app::DEFAULT_GOOSE_CONFIG_PATH;
#[cfg(target_os = "windows")]
use crate::util::constants::app::DEFAULT_GOOSE_WINDOWS_CONFIG_PATH;
use crate::{
    io::{read_file, ApiResult},
    util::constants::app::APPLICATION,
};
use acorn_cmd::args;
use acorn_core::options::{SyncInner, TargetConfig};
use acorn_schema::{agent::ModelDetails, validation::Validate};
use alloc::string::{String, ToString};
use color_eyre::eyre::eyre;
#[cfg(target_os = "windows")]
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use serde_norway::Value;
use serde_with::skip_serializing_none;
use std::path::{Path, PathBuf};

/// Configuration for synchronizing a local OpenAI-compatible model into Goose CLI.
pub type Config = TargetConfig<Inner>;
/// Goose-specific synchronization configuration.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Inner {
    /// OpenAI-compatible endpoint origin.
    #[serde(default = "default_host")]
    #[validate(url)]
    pub host: String,
    /// Chat completions path appended to the endpoint origin.
    #[serde(default = "default_base_path")]
    #[validate(length(min = 1))]
    pub base_path: String,
    /// Model to activate; defaults to the first synchronized model.
    #[validate(length(min = 1))]
    pub default_model: Option<String>,
}
impl Inner {
    /// Upsert the active OpenAI provider while preserving unrelated Goose settings.
    pub fn upsert(&self, existing: Value, models: &[ModelDetails]) -> ApiResult<Value> {
        self.default_model
            .as_ref()
            .or_else(|| models.iter().find_map(|model| model.id.as_ref()))
            .ok_or_else(|| eyre!("Goose synchronization requires at least one model"))
            .and_then(|model| {
                match existing {
                    | Value::Mapping(root) => Ok(root),
                    | _ => Err(eyre!("Goose configuration must be a YAML mapping")),
                }
                .map(|mut root| {
                    let providers_key = Value::String("providers".to_string());
                    let mut providers = root
                        .remove(&providers_key)
                        .and_then(|value| value.as_mapping().cloned())
                        .unwrap_or_default();
                    let openai_key = Value::String("openai".to_string());
                    let mut openai = providers
                        .remove(&openai_key)
                        .and_then(|value| value.as_mapping().cloned())
                        .unwrap_or_default();
                    openai.insert(Value::String("enabled".to_string()), Value::Bool(true));
                    openai.insert(Value::String("model".to_string()), Value::String(model.clone()));
                    openai.insert(Value::String("configured".to_string()), Value::Bool(true));
                    providers.insert(openai_key, Value::Mapping(openai));
                    root.insert(Value::String("active_provider".to_string()), Value::String("openai".to_string()));
                    root.insert(providers_key, Value::Mapping(providers));
                    root.insert(Value::String("OPENAI_HOST".to_string()), Value::String(self.host.clone()));
                    root.insert(Value::String("OPENAI_BASE_PATH".to_string()), Value::String(self.base_path.clone()));
                    Value::Mapping(root)
                })
            })
    }
}
impl Default for Inner {
    fn default() -> Self {
        Self {
            host: default_host(),
            base_path: default_base_path(),
            default_model: None,
        }
    }
}
impl SyncInner for Inner {
    fn merge(self, overrides: Self) -> Self {
        Self {
            host: overrides.host,
            base_path: overrides.base_path,
            default_model: overrides.default_model.or(self.default_model),
        }
    }
    fn merge_cli_overrides(self, _overrides: Self) -> Self {
        self
    }
}
impl McpSync for Inner {
    type Document = Value;
    const TARGET: &'static str = "Goose";
    fn update_mcp(&self, existing: Value, executable: &Path, prune: bool) -> ApiResult<Value> {
        match existing {
            | Value::Mapping(mut root) => {
                let extensions_key = Value::String("extensions".to_string());
                let mut extensions = root
                    .remove(&extensions_key)
                    .and_then(|value| value.as_mapping().cloned())
                    .unwrap_or_default();
                let acorn = Value::String(APPLICATION.to_string());
                if prune {
                    extensions.remove(&acorn);
                } else {
                    extensions.insert(
                        acorn,
                        Value::Mapping(
                            [
                                (
                                    Value::String("args".to_string()),
                                    Value::Sequence(
                                        args!["serve", "mcp"]
                                            .into_iter()
                                            .map(|argument| Value::String(argument.to_string_lossy().into_owned()))
                                            .collect(),
                                    ),
                                ),
                                (Value::String("cmd".to_string()), Value::String(executable.display().to_string())),
                                (Value::String("enabled".to_string()), Value::Bool(true)),
                                (Value::String("type".to_string()), Value::String("stdio".to_string())),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    );
                }
                if !extensions.is_empty() {
                    root.insert(extensions_key, Value::Mapping(extensions));
                }
                Ok(Value::Mapping(root))
            }
            | _ => Err(eyre!("Goose configuration must be a YAML mapping")),
        }
    }
}
impl SyncTarget for Config {
    const COMMAND: &'static str = "goose";
    fn resolve_path(explicit: Option<&str>) -> ApiResult<PathBuf> {
        explicit.map(PathBuf::from).map_or_else(
            || {
                #[cfg(target_os = "windows")]
                let path = BaseDirs::new()
                    .map(|directories| directories.config_dir().join(DEFAULT_GOOSE_WINDOWS_CONFIG_PATH))
                    .ok_or_else(|| eyre!("Failed to resolve platform configuration directory"));
                #[cfg(not(target_os = "windows"))]
                let path = home_directory(DEFAULT_GOOSE_CONFIG_PATH);
                path
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
                        | true => Ok(Value::Mapping(Default::default())),
                        | false => serde_norway::from_str(&before).map_err(|why| eyre!("Failed to parse existing Goose config: {why}")),
                    }
                    .and_then(|existing| match options.models.is_empty() {
                        | true => Ok(existing),
                        | false => self.upsert(existing, options.models),
                    })
                    .and_then(|existing| match (options.mcp, options.executable) {
                        | (true, Some(executable)) => self.upsert_mcp(existing, executable, options.prune),
                        | (true, None) => Err(eyre!("MCP synchronization requires an absolute ACORN executable")),
                        | (false, _) => Ok(existing),
                    })
                    .and_then(|updated| serde_norway::to_string(&updated).map_err(|why| eyre!("Failed to serialize Goose config: {why}")))
                    .map(|content| RenderedOutput {
                        target: "Goose",
                        path,
                        before,
                        content,
                    })
                })
        })
    }
}
fn default_base_path() -> String {
    "v1/chat/completions".to_string()
}
fn default_host() -> String {
    "http://localhost:8080".to_string()
}
