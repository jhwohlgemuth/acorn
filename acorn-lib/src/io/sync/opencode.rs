//! OpenCode synchronization configuration types
//!
//! When syncing ACORN models to OpenCode, the sync command creates or updates
//! a custom provider entry that points to the local llama-swap instance.
//! Each model is registered as a keyed entry in the provider's model map.
use crate::io::ApiResult;
use crate::prelude::current_dir;
use crate::schema::agent::{opencode, ProviderDetails};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
enum Entry {
    Provider {
        #[serde(rename = "@ai-sdk/openai-compatible")]
        sdk: BTreeMap<String, Value>,
        options: BTreeMap<String, String>,
    },
    Model {
        #[serde(rename = "providerID")]
        provider_id: String,
        model: String,
        name: String,
    },
}
/// Configuration for synchronizing models into OpenCode
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Path to the OpenCode configuration file on disk
    #[serde(skip)]
    pub path: Option<String>,
    /// Base URL for the OpenAI-compatible endpoint (e.g., `http://localhost:8080/v1`)
    #[serde(default = "default_base_url")]
    pub base_url: String,
    /// Provider identifier in the OpenCode configuration
    #[serde(default = "default_provider_id")]
    pub provider_id: String,
    /// Human-readable provider display name
    #[serde(default = "default_provider_name")]
    pub provider_name: String,
    /// Default model to use when none is specified
    pub default_model: Option<String>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            path: None,
            base_url: default_base_url(),
            provider_id: default_provider_id(),
            provider_name: default_provider_name(),
            default_model: None,
        }
    }
}
impl Config {
    /// Build the JSON structure for an OpenCode provider entry.
    pub fn provider_entry(&self) -> Value {
        serde_json::to_value(Entry::Provider {
            sdk: BTreeMap::new(),
            options: core::iter::once(("baseURL".to_string(), self.base_url.clone())).collect(),
        })
        .unwrap_or(Value::Null)
    }
    /// Build the JSON structure for a model entry.
    pub fn model_entry(&self, model_id: &str, display_name: &str) -> Value {
        serde_json::to_value(Entry::Model {
            provider_id: self.provider_id.clone(),
            model: model_id.to_string(),
            name: display_name.to_string(),
        })
        .unwrap_or(Value::Null)
    }
    /// Determine the path to the OpenCode config file.
    ///
    /// Search order:
    /// 1. Explicit path from `--opencode-config`
    /// 2. `opencode.jsonc` in the working directory
    /// 3. `opencode.json` in the working directory
    ///
    /// If neither exists, defaults to `opencode.jsonc`.
    pub fn resolve_path(explicit: Option<&str>) -> ApiResult<String> {
        match explicit {
            | Some(path) => Ok(path.to_string()),
            | None => opencode::Config::resolve()
                .and_then(|config| config.path)
                .map(|path| path.display().to_string())
                .map_or_else(
                    || {
                        current_dir()
                            .map(|directory| directory.join("opencode.jsonc").display().to_string())
                            .map_err(|why| eyre!("Failed to get working directory: {why}"))
                    },
                    Ok,
                ),
        }
    }
    /// Upsert the managed provider into an existing OpenCode `Config`
    ///
    /// Returns the modified config with the llama-swap provider and its models
    /// upserted, preserving all other existing configuration.
    pub fn upsert(&self, existing: &opencode::Config, model_ids: &[(String, String)]) -> opencode::Config {
        let models = model_ids
            .iter()
            .map(|(identifier, name)| (identifier.clone(), self.model_entry(identifier, name)))
            .collect::<BTreeMap<_, _>>();
        let provider_entry = self
            .provider_entry()
            .as_object()
            .map(|provider| {
                Value::Object(
                    provider
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .chain(core::iter::once((
                            "models".to_string(),
                            serde_json::to_value(models).unwrap_or(Value::Null),
                        )))
                        .collect(),
                )
            })
            .unwrap_or_else(|| self.provider_entry());
        let providers = existing
            .provider
            .as_ref()
            .into_iter()
            .flat_map(|providers| providers.iter())
            .map(|(identifier, details)| (identifier.clone(), serde_json::to_value(details).unwrap_or(Value::Null)))
            .chain(core::iter::once((self.provider_id.clone(), provider_entry)))
            .filter_map(|(identifier, value)| serde_json::from_value::<ProviderDetails>(value).ok().map(|details| (identifier, details)))
            .collect::<BTreeMap<_, _>>();
        existing.clone().with_provider(providers)
    }
    /// Remove stale managed models from the OpenCode provider
    ///
    /// Models that are inside the managed provider but not in `current_models` are removed.
    /// If the root `model` field points to a removed entry and no `default_model` is configured, returns an error.
    pub fn prune(&self, mut config: opencode::Config, current_models: &[(String, String)]) -> ApiResult<opencode::Config> {
        let managed_provider = &self.provider_id;
        let current_ids: BTreeMap<&str, ()> = current_models.iter().map(|(id, _)| (id.as_str(), ())).collect();
        // Check root model
        if let Some(ref root_model) = config.model {
            let parts: Vec<&str> = root_model.splitn(2, '/').collect();
            let model_id = parts.get(1).copied().unwrap_or_default();
            if parts.first().copied() == Some(managed_provider.as_str()) && !current_ids.contains_key(model_id) {
                if self.default_model.is_some() {
                    config.model = self.default_model.clone();
                } else {
                    return Err(eyre!(
                        "Root model '{}' points to a removed entry without a configured defaultModel",
                        root_model
                    ));
                }
            }
        }
        // Remove stale models from provider
        if let Some(ref mut providers) = config.provider {
            if let Some(provider_val) = providers.get_mut(managed_provider) {
                let raw = serde_json::to_value(&*provider_val).unwrap_or(Value::Null);
                if let Some(obj) = raw.as_object() {
                    let cleaned: serde_json::Map<String, Value> = obj
                        .iter()
                        .filter(|(k, _)| {
                            matches!(k.as_str(), "models" | "@ai-sdk/openai-compatible" | "options") || current_ids.contains_key(k.as_str())
                        })
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    if let Ok(rebuilt) = serde_json::from_value(Value::Object(cleaned)) {
                        *provider_val = rebuilt;
                    }
                }
            }
        }
        Ok(config)
    }
}
fn default_base_url() -> String {
    "http://localhost:8080/v1".to_string()
}
fn default_provider_id() -> String {
    "llama-swap".to_string()
}
fn default_provider_name() -> String {
    "Llama Swap".to_string()
}
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_default_values() {
        let config = Config::default();
        assert_eq!(config.base_url, "http://localhost:8080/v1");
        assert_eq!(config.provider_id, "llama-swap");
        assert_eq!(config.provider_name, "Llama Swap");
    }
    #[test]
    fn test_provider_json_structure() {
        let config = Config::default();
        let json = config.provider_entry();
        assert!(json.get("@ai-sdk/openai-compatible").is_some());
        let options = json.get("options").unwrap();
        assert_eq!(options.get("baseURL").unwrap().as_str().unwrap(), "http://localhost:8080/v1");
    }
    #[test]
    fn test_model_json_structure() {
        let json = Config::default().model_entry("qwen-gguf", "Qwen GGUF");
        assert_eq!(json.get("providerID").unwrap().as_str().unwrap(), "llama-swap");
        assert_eq!(json.get("model").unwrap().as_str().unwrap(), "qwen-gguf");
        assert_eq!(json.get("name").unwrap().as_str().unwrap(), "Qwen GGUF");
    }
}
