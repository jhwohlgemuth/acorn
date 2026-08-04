//! Goose CLI YAML synchronization configuration types
use super::{Options, RenderedOutput, SyncTarget};
use crate::io::{home_directory, read_file, ApiResult};
use crate::prelude::PathBuf;
use crate::schema::agent::ModelDetails;
use crate::util::constants::app::DEFAULT_GOOSE_CONFIG_PATH;
#[cfg(target_os = "windows")]
use crate::util::constants::app::DEFAULT_GOOSE_WINDOWS_CONFIG_PATH;
use alloc::string::{String, ToString};
use color_eyre::eyre::eyre;
#[cfg(target_os = "windows")]
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use serde_norway::Value;
use serde_with::skip_serializing_none;
use validator::Validate;

/// Configuration for synchronizing a local OpenAI-compatible model into Goose CLI.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Path to Goose's `config.yaml` on disk.
    #[serde(skip_serializing)]
    #[validate(length(min = 1))]
    pub path: Option<String>,
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
impl Default for Config {
    fn default() -> Self {
        Self {
            path: None,
            host: default_host(),
            base_path: default_base_path(),
            default_model: None,
        }
    }
}
impl SyncTarget for Config {
    const COMMAND: &'static str = "goose";
    fn merge(self, overrides: Self) -> Self {
        Self {
            path: overrides.path.or(self.path),
            host: overrides.host,
            base_path: overrides.base_path,
            default_model: overrides.default_model.or(self.default_model),
        }
    }
    fn merge_cli_overrides(self, overrides: Self) -> Self {
        Self {
            path: overrides.path.or(self.path),
            ..self
        }
    }
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
                    .and_then(|existing| self.upsert(existing, options.models))
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
impl Config {
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
fn default_host() -> String {
    "http://localhost:8080".to_string()
}
fn default_base_path() -> String {
    "v1/chat/completions".to_string()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_upsert_preserves_unrelated_settings_and_provider_fields() {
        let existing: Value =
            serde_norway::from_str("GOOSE_MODE: approve\nproviders:\n  anthropic:\n    enabled: true\n  openai:\n    custom: retained\n").unwrap();
        let models = [ModelDetails::init().id("qwen").build()];
        let updated = Config::default().upsert(existing, &models).unwrap();
        let rendered = serde_norway::to_string(&updated).unwrap();
        assert!(rendered.contains("GOOSE_MODE: approve"));
        assert!(rendered.contains("anthropic:"));
        assert!(rendered.contains("custom: retained"));
        assert!(rendered.contains("model: qwen"));
        assert!(rendered.contains("OPENAI_BASE_PATH: v1/chat/completions"));
    }
}
