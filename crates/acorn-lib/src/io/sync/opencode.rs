//! OpenCode synchronization configuration types
//!
//! When syncing ACORN models to OpenCode, the sync command creates or updates
//! a custom provider entry that points to the local llama-swap instance.
//! Each model is registered as a keyed entry in the provider's model map.
use super::{McpSync, Options, RenderedOutput, SyncTarget};
use crate::{
    io::{home_directory, parse_jsonc_cst, read_file, ApiResult, CstRootNode, CstValue, PathConversion},
    util::constants::app::{APPLICATION, DEFAULT_OPENCODE_CONFIG_PATH},
};
use acorn_core::options::{SyncInner, TargetConfig};
use acorn_schema::{
    agent::opencode::{self, McpConfig, McpLocalConfig},
    validation::Validate,
};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
};
use color_eyre::eyre::eyre;
use core::{fmt, iter::once};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

/// Configuration for synchronizing models into OpenCode.
pub type Config = TargetConfig<Inner>;

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
/// OpenCode-specific synchronization configuration.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Inner {
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
impl Inner {
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
    pub(crate) fn discover_path(calling_directory: &Path, user_config: PathBuf) -> PathBuf {
        [
            calling_directory.join("opencode.jsonc"),
            calling_directory.join("opencode.json"),
            user_config.clone(),
            user_config.with_extension("json"),
        ]
        .into_iter()
        .find(|path| path.is_file())
        .unwrap_or(user_config)
    }
    /// Render an updated OpenCode configuration while preserving JSONC comments outside the managed provider
    pub fn render(&self, config: &opencode::Config) -> ApiResult<String> {
        self.render_with_cst(config, None)
    }
    pub(crate) fn render_with_cst(&self, config: &opencode::Config, cst: Option<&CstRootNode>) -> ApiResult<String> {
        match cst {
            | Some(cst) => Ok(config)
                .map(|config| {
                    let root = cst.object_value_or_set();
                    if let Some(provider) = config.provider.as_ref().and_then(|providers| providers.get(&self.provider_id)) {
                        let providers = root.object_value_or_set("provider");
                        match providers.get(&self.provider_id) {
                            | Some(property) => property.set_value(CstValue(provider).into()),
                            | None => {
                                providers.append(&self.provider_id, CstValue(provider).into());
                            }
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
                    let managed_mcp = config
                        .mcp
                        .as_ref()
                        .and_then(|servers| servers.get(APPLICATION))
                        .and_then(|server| serde_json::to_value(server).ok());
                    match (root.object_value("mcp"), managed_mcp.as_ref()) {
                        | (Some(servers), Some(server)) => match servers.get(APPLICATION) {
                            | Some(property) => property.set_value(CstValue(server).into()),
                            | None => {
                                servers.append(APPLICATION, CstValue(server).into());
                            }
                        },
                        | (Some(servers), None) => {
                            if let Some(property) = servers.get(APPLICATION) {
                                property.remove();
                            }
                        }
                        | (None, Some(server)) => {
                            root.object_value_or_set("mcp").append(APPLICATION, CstValue(server).into());
                        }
                        | (None, None) => {}
                    }
                    cst.to_string()
                })
                .and_then(|content| {
                    parse_jsonc_cst::<opencode::Config>(&content)
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
impl Default for Inner {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            provider_id: default_provider_id(),
            provider_name: default_provider_name(),
            default_model: None,
        }
    }
}
impl fmt::Display for Inner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.provider_id.fmt(formatter)
    }
}
impl McpSync for Inner {
    type Document = opencode::Config;
    const TARGET: &'static str = "OpenCode";
    fn update_mcp(&self, existing: opencode::Config, executable: &Path, prune: bool) -> ApiResult<opencode::Config> {
        let mut servers = existing.mcp.clone().unwrap_or_default();
        if prune {
            servers.remove(APPLICATION);
        } else {
            servers.insert(
                APPLICATION.to_string(),
                McpConfig::Local(McpLocalConfig {
                    command: vec![executable.display().to_string(), "serve".to_string(), "mcp".to_string()],
                    cwd: None,
                    environment: None,
                    enabled: Some(true),
                    timeout: None,
                }),
            );
        }
        Ok(opencode::Config {
            mcp: (!servers.is_empty()).then_some(servers),
            ..existing
        })
    }
}
impl SyncInner for Inner {
    fn merge(self, overrides: Self) -> Self {
        Self {
            base_url: overrides.base_url,
            provider_id: overrides.provider_id,
            provider_name: overrides.provider_name,
            default_model: overrides.default_model.or(self.default_model),
        }
    }
    fn merge_cli_overrides(self, _overrides: Self) -> Self {
        self
    }
}
impl SyncTarget for Config {
    const COMMAND: &'static str = "opencode";
    fn resolve_path(explicit: Option<&str>) -> ApiResult<PathBuf> {
        match explicit {
            | Some(path) => Ok(PathBuf::from(Path::new(path).cross_platform_display())),
            | None => current_dir()
                .map_err(|why| eyre!("Failed to get ACORN calling directory — {why}"))
                .and_then(|directory| home_directory(DEFAULT_OPENCODE_CONFIG_PATH).map(|user_config| Inner::discover_path(&directory, user_config))),
        }
    }
    fn render(&self, options: Options<'_>) -> ApiResult<RenderedOutput> {
        Self::resolve_path(self.path.as_deref()).and_then(|path| {
            let models = options
                .models
                .iter()
                .filter_map(|model| model.name.as_ref().or(model.id.as_ref()))
                .map(|name| (name.clone(), name.clone()))
                .collect::<Vec<_>>();
            path.is_file()
                .then(|| read_file(path.clone()))
                .transpose()
                .map(|content| content.unwrap_or_default())
                .and_then(|before| {
                    match path.is_file() {
                        | true => parse_jsonc_cst::<opencode::Config>(&before).map(|(config, cst)| (config, Some(cst))),
                        | false => Ok((opencode::Config::default(), None)),
                    }
                    .and_then(|(existing, cst)| {
                        match models.is_empty() {
                            | true => Ok(existing),
                            | false => self.upsert(&existing, &models, options.prune),
                        }
                        .and_then(|updated| match (options.mcp, options.executable) {
                            | (true, Some(executable)) => self.upsert_mcp(updated, executable, options.prune).map(|updated| (updated, cst)),
                            | (true, None) => Err(eyre!("MCP synchronization requires an absolute ACORN executable")),
                            | (false, _) => Ok((updated, cst)),
                        })
                    })
                    .and_then(|(opencode, cst)| self.render_with_cst(&opencode, cst.as_ref()))
                    .map(|content| RenderedOutput {
                        target: "OpenCode",
                        path,
                        before,
                        content,
                    })
                })
        })
    }
}
fn default_base_url() -> String {
    "http://localhost:8080/v1".to_string()
}
fn default_provider_id() -> String {
    "llama-swap".to_string()
}
fn default_provider_name() -> String {
    "Local (llama-swap)".to_string()
}
