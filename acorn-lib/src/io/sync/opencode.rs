//! OpenCode synchronization configuration types
//!
//! When syncing ACORN models to OpenCode, the sync command creates or updates
//! a custom provider entry that points to the local llama-swap instance.
//! Each model is registered as a keyed entry in the provider's model map.
use crate::io::{ApiResult, CstValue};
use crate::prelude::current_dir;
use crate::schema::agent::opencode;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::string::ToString;
use color_eyre::eyre::eyre;
use core::{fmt, iter::once};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use validator::Validate;

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
enum Entry {
    Provider {
        npm: String,
        name: String,
        options: BTreeMap<String, String>,
    },
    Model {
        name: String,
    },
}
/// Configuration for synchronizing models into OpenCode
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Path to the OpenCode configuration file on disk
    #[serde(skip_serializing)]
    #[validate(length(min = 1))]
    pub path: Option<String>,
    /// Base URL for the OpenAI-compatible endpoint (e.g., `http://localhost:8080/v1`)
    #[serde(default = "default_base_url")]
    #[validate(length(min = 1))]
    pub base_url: String,
    /// Provider identifier in the OpenCode configuration
    #[serde(default = "default_provider_id")]
    #[validate(length(min = 1))]
    pub provider_id: String,
    /// Human-readable provider display name
    #[serde(default = "default_provider_name")]
    #[validate(length(min = 1))]
    pub provider_name: String,
    /// Default model to use when none is specified
    #[validate(length(min = 1))]
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
impl fmt::Display for Config {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.provider_id.fmt(formatter)
    }
}
impl Config {
    /// Merge populated runtime overrides into this configuration
    pub fn merge(self, overrides: Self) -> Self {
        Self {
            path: overrides.path.or(self.path),
            base_url: overrides.base_url,
            provider_id: overrides.provider_id,
            provider_name: overrides.provider_name,
            default_model: overrides.default_model.or(self.default_model),
        }
    }
    /// Merge CLI-supported overrides while preserving configured provider values
    pub fn merge_cli_overrides(self, overrides: Self) -> Self {
        Self {
            path: overrides.path.or(self.path),
            ..self
        }
    }
    /// Build the JSON structure for a model entry.
    pub fn model_entry(&self, display_name: &str) -> Value {
        serde_json::to_value(Entry::Model {
            name: display_name.to_string(),
        })
        .unwrap_or(Value::Null)
    }
    /// Build the JSON structure for an OpenCode provider entry.
    pub fn provider_entry(&self) -> Value {
        serde_json::to_value(Entry::Provider {
            npm: "@ai-sdk/openai-compatible".to_string(),
            name: self.provider_name.clone(),
            options: once(("baseURL".to_string(), self.base_url.clone())).collect(),
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
    /// Render an updated OpenCode configuration while preserving JSONC comments outside the managed provider
    pub fn render(&self, config: &opencode::Config) -> ApiResult<String> {
        match &config.cst {
            | Some(cst) => config
                .provider
                .as_ref()
                .and_then(|providers| providers.get(&self.provider_id))
                .ok_or_else(|| eyre!("Managed OpenCode provider '{self}' is missing"))
                .map(|provider| {
                    let root = cst.object_value_or_set();
                    let providers = root.object_value_or_set("provider");
                    match providers.get(&self.provider_id) {
                        | Some(property) => property.set_value(CstValue(provider).into()),
                        | None => {
                            providers.append(&self.provider_id, CstValue(provider).into());
                        }
                    }
                    if let Some(model) = config.model.as_ref() {
                        match root.get("model") {
                            | Some(property) => property.set_value(model.clone().into()),
                            | None => {
                                root.append("model", model.clone().into());
                            }
                        }
                    }
                    cst.to_string()
                })
                .and_then(|content| {
                    opencode::Config::parse_jsonc(&content)
                        .map(|_| content)
                        .map_err(|why| eyre!("Generated OpenCode JSONC is invalid — {why}"))
                }),
            | None => serde_json::to_string_pretty(config)
                .map_err(|why| eyre!("Failed to serialize OpenCode config — {why}"))
                .and_then(|content| {
                    serde_json::from_str::<opencode::Config>(&content)
                        .map(|_| content)
                        .map_err(|why| eyre!("Generated OpenCode JSON is invalid — {why}"))
                }),
        }
    }
    /// Upsert the managed provider into an existing OpenCode `Config`
    ///
    /// Returns the modified config with the llama-swap provider and its models
    /// upserted, preserving all other existing configuration.
    pub fn upsert(&self, existing: &opencode::Config, model_ids: &[(String, String)], prune: bool) -> ApiResult<opencode::Config> {
        let current_ids = model_ids.iter().map(|(identifier, _)| identifier.as_str()).collect::<BTreeSet<_>>();
        let existing_provider = existing
            .provider
            .as_ref()
            .and_then(|providers| providers.get(&self.provider_id))
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let existing_models = existing_provider.get("models").and_then(Value::as_object).cloned().unwrap_or_default();
        let models = existing_models
            .into_iter()
            .filter(|(identifier, _)| !prune || current_ids.contains(identifier.as_str()))
            .chain(model_ids.iter().map(|(identifier, name)| (identifier.clone(), self.model_entry(name))))
            .collect::<serde_json::Map<_, _>>();
        let options = existing_provider
            .get("options")
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|options| options.iter())
            .map(|(key, value)| (key.clone(), value.clone()))
            .chain(once(("baseURL".to_string(), Value::String(self.base_url.clone()))))
            .collect::<serde_json::Map<_, _>>();
        let provider = existing_provider
            .into_iter()
            .chain([
                ("npm".to_string(), Value::String("@ai-sdk/openai-compatible".to_string())),
                ("name".to_string(), Value::String(self.provider_name.clone())),
                ("options".to_string(), Value::Object(options)),
                ("models".to_string(), Value::Object(models)),
            ])
            .collect::<serde_json::Map<_, _>>();
        let providers = existing
            .provider
            .as_ref()
            .into_iter()
            .flat_map(|providers| providers.iter())
            .map(|(identifier, value)| (identifier.clone(), value.clone()))
            .chain(once((self.provider_id.clone(), Value::Object(provider))))
            .collect::<BTreeMap<_, _>>();
        let root_model = existing
            .model
            .as_ref()
            .and_then(|model| model.split_once('/'))
            .filter(|(provider, _)| *provider == self.provider_id);
        match (prune, root_model, self.default_model.as_ref()) {
            | (true, Some((_, model)), None) if !current_ids.contains(model) => Err(eyre!(
                "Root model '{self}/{model}' points to a removed entry without a configured defaultModel"
            )),
            | _ => Ok(opencode::Config {
                model: self
                    .default_model
                    .as_ref()
                    .map(|model| format!("{}/{model}", self.provider_id))
                    .or_else(|| existing.model.clone()),
                provider: Some(providers),
                ..existing.clone()
            }),
        }
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
        assert_eq!(config.to_string(), "llama-swap");
    }
    #[test]
    fn test_model_json_structure() {
        let json = Config::default().model_entry("Qwen GGUF");
        assert_eq!(json.get("name").unwrap().as_str().unwrap(), "Qwen GGUF");
    }
    #[test]
    fn test_provider_json_structure() {
        let config = Config::default();
        let json = config.provider_entry();
        assert_eq!(json.get("npm").unwrap().as_str().unwrap(), "@ai-sdk/openai-compatible");
        assert_eq!(json.get("name").unwrap().as_str().unwrap(), "Llama Swap");
        let options = json.get("options").unwrap();
        assert_eq!(options.get("baseURL").unwrap().as_str().unwrap(), "http://localhost:8080/v1");
    }
    #[test]
    fn test_render_preserves_jsonc_comments_outside_managed_provider() {
        let existing = opencode::Config::parse_jsonc(
            r#"{
  // root comment
  "username": "acorn",
  "provider": {
    // unrelated provider comment
    "other": {"options": {"baseURL": "https://example.test"}},
    "llama-swap": {"models": {"stale": {"name": "Stale"}}}
  }
}"#,
        )
        .unwrap();
        let config = Config::default();
        let updated = config.upsert(&existing, &[("qwen".to_string(), "Qwen".to_string())], false).unwrap();
        let rendered = config.render(&updated).unwrap();
        assert!(rendered.contains("// root comment"));
        assert!(rendered.contains("// unrelated provider comment"));
        assert!(rendered.contains("\"stale\""));
        assert!(rendered.contains("\"qwen\""));
        assert_eq!(rendered, config.render(&updated).unwrap());
    }
    #[test]
    fn test_upsert_is_additive_unless_pruned() {
        let existing = opencode::Config {
            username: Some("acorn".to_string()),
            provider: Some(
                [
                    ("other".to_string(), serde_json::json!({"options": {"baseURL": "https://example.test"}})),
                    (
                        "llama-swap".to_string(),
                        serde_json::json!({
                            "custom": true,
                            "options": {"apiKey": "secret"},
                            "models": {"stale": {"name": "Stale"}}
                        }),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
            ..Default::default()
        };
        let config = Config::default();
        let models = [("qwen".to_string(), "Qwen".to_string())];
        let additive = config.upsert(&existing, &models, false).unwrap();
        let provider = additive.provider.as_ref().unwrap().get("llama-swap").unwrap();
        assert!(provider.pointer("/models/stale").is_some());
        assert!(provider.pointer("/models/qwen").is_some());
        assert_eq!(provider.get("custom"), Some(&Value::Bool(true)));
        assert_eq!(provider.pointer("/options/apiKey").and_then(Value::as_str), Some("secret"));
        assert!(additive.provider.as_ref().unwrap().contains_key("other"));
        assert_eq!(additive.username.as_deref(), Some("acorn"));
        let pruned = config.upsert(&existing, &models, true).unwrap();
        let provider = pruned.provider.as_ref().unwrap().get("llama-swap").unwrap();
        assert!(provider.pointer("/models/stale").is_none());
        assert!(provider.pointer("/models/qwen").is_some());
    }
    #[test]
    fn test_upsert_protects_pruned_root_model() {
        let existing = opencode::Config {
            model: Some("llama-swap/stale".to_string()),
            provider: Some(
                [("llama-swap".to_string(), serde_json::json!({"models": {"stale": {"name": "Stale"}}}))]
                    .into_iter()
                    .collect(),
            ),
            ..Default::default()
        };
        let models = [("qwen".to_string(), "Qwen".to_string())];
        assert!(Config::default().upsert(&existing, &models, true).is_err());
        let replacement = Config {
            default_model: Some("qwen".to_string()),
            ..Default::default()
        }
        .upsert(&existing, &models, true)
        .unwrap();
        assert_eq!(replacement.model.as_deref(), Some("llama-swap/qwen"));
    }
}
