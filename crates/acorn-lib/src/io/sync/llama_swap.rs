//! llama-swap configuration types
//!
//! llama-swap is a lightweight HTTP proxy that manages multiple local LLM
//! models behind a single OpenAI-compatible endpoint. Each model entry is
//! a command that launches `llama-server` with the appropriate flags.
//!
//! See <https://github.com/mostlygeek/llama-swap/blob/main/docs/configuration.md>
use super::{Options, RenderedOutput, SyncTarget};
use crate::{
    io::{home_directory, read_file, ApiResult},
    util::{constants::app::DEFAULT_LLAMA_SWAP_CONFIG_PATH, StringConversion, ToStrings},
};
use acorn_cmd::{args, runtime::command_string};
use acorn_core::{
    options::{SyncInner, TargetConfig},
    validation::ValidationError,
};
use acorn_schema::{
    agent::ModelDetails,
    validation::{Validate, ValidationReport},
};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec::Vec,
};
use color_eyre::eyre::eyre;
use core::{fmt, iter::once};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_norway::{Mapping, Number, Value};
use serde_with::skip_serializing_none;
use std::{ffi::OsString, path::PathBuf};

/// Root llama-swap synchronization configuration.
pub type Config = TargetConfig<Inner>;

/// Additional llama-server command-line argument
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(from = "String", into = "String")]
pub enum Argument {
    /// Argument reserved for ACORN's llama-swap integration
    Reserved(String),
    /// User-provided llama-server argument
    Value(String),
}
/// Human-readable alias for a synchronized model
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(transparent)]
pub struct Alias {
    #[validate(length(min = 1))]
    value: String,
}
/// Validated llama-swap environment entry serialized as `KEY=VALUE`
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(transparent)]
pub struct EnvironmentVariable {
    #[validate(custom(function = "is_keyvalue_string"))]
    value: String,
}
/// llama-swap-specific synchronization configuration.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Inner {
    /// Default directory where downloaded model weights live
    pub models_directory: Option<String>,
    /// Executable name or path for `llama-server`
    #[validate(length(min = 1))]
    pub executable: Option<String>,
    /// Default context window size for all models
    #[validate(range(min = 1))]
    pub context_size: Option<u64>,
    /// Default time-to-live in seconds; 0 means no expiry
    #[validate(range(min = 0))]
    pub ttl: Option<i64>,
    /// Extra command-line arguments applied to every model command
    #[validate(nested)]
    pub extra_args: Option<Vec<Argument>>,
    /// Extra environment variables as `KEY=VALUE` entries applied to every model
    #[validate(nested)]
    pub environment: Option<Vec<EnvironmentVariable>>,
    /// Per-model overrides keyed by ACORN model name
    #[validate(nested)]
    pub models: Option<BTreeMap<String, ModelOverride>>,
}
/// Per-model override entry keyed by ACORN model name
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ModelOverride {
    /// Model-specific context window size (overrides the global default)
    #[validate(range(min = 1))]
    pub context_size: Option<u64>,
    /// Time-to-live in seconds; 0 means no expiry
    #[validate(range(min = 0))]
    pub ttl: Option<i64>,
    /// Human-readable aliases for selecting this model
    #[validate(nested)]
    pub aliases: Option<Vec<Alias>>,
    /// Additional command-line arguments for `llama-server`
    #[validate(nested)]
    pub extra_args: Option<Vec<Argument>>,
    /// Extra environment variables as `KEY=VALUE` entries
    #[validate(nested)]
    pub environment: Option<Vec<EnvironmentVariable>>,
    /// Executable name or path for this model (overrides global)
    #[validate(length(min = 1))]
    pub executable: Option<String>,
}
pub(super) struct ModelValidation<'a> {
    config: &'a Config,
    model_ids: &'a [String],
}
impl Alias {
    fn as_str(&self) -> &str {
        &self.value
    }
}
impl From<&str> for Alias {
    fn from(value: &str) -> Self {
        Self { value: value.to_string() }
    }
}
impl fmt::Display for Argument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::Reserved(value) | Self::Value(value) => value.fmt(formatter),
        }
    }
}
impl From<&str> for Argument {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}
impl From<String> for Argument {
    fn from(value: String) -> Self {
        match value.split_once('=').map_or(value.as_str(), |(option, _)| option) {
            | "--port" | "--model" | "-m" => Self::Reserved(value),
            | _ => Self::Value(value),
        }
    }
}
impl Validate for Argument {
    fn validate(&self) -> Result<(), ValidationReport> {
        match self {
            | Self::Reserved(_) => {
                let mut errors = ValidationReport::new();
                errors.add(
                    "argument",
                    ValidationError::new("reserved").with_message("Cannot contain --port, --model, or -m"),
                );
                Err(errors)
            }
            | Self::Value(_) => Ok(()),
        }
    }
}
impl SyncTarget for Config {
    const COMMAND: &'static str = "llama-swap";
    fn resolve_path(explicit: Option<&str>) -> ApiResult<PathBuf> {
        explicit
            .map(|path| PathBuf::from(path.to_string().to_cross_platform_path()))
            .map_or_else(|| home_directory(DEFAULT_LLAMA_SWAP_CONFIG_PATH), Ok)
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
                        | false => serde_norway::from_str(&before).map_err(|why| eyre!("Failed to parse existing llama-swap config: {why}")),
                    }
                    .and_then(|existing| self.upsert(existing, options.models, options.prune))
                    .and_then(|merged| {
                        serde_norway::to_string(&merged)
                            .map_err(|why| eyre!("Failed to serialize llama-swap config: {why}"))
                            .and_then(|content| {
                                serde_norway::from_str::<Value>(&content)
                                    .map(|_| RenderedOutput {
                                        target: "llama-swap",
                                        path,
                                        before,
                                        content,
                                    })
                                    .map_err(|why| eyre!("Generated llama-swap YAML is invalid: {why}"))
                            })
                    })
                })
        })
    }
}
impl fmt::Display for EnvironmentVariable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(formatter)
    }
}
impl From<&str> for EnvironmentVariable {
    fn from(value: &str) -> Self {
        Self { value: value.to_string() }
    }
}
impl Inner {
    /// Build a shell-safe `llama-server` command string for a given GGUF model path
    /// ### Note
    /// The command uses `${PORT}` as a placeholder that llama-swap replaces at runtime
    pub fn build_command(
        executable: &str,
        gguf_path: Option<&str>,
        extra_args: Option<&[Argument]>,
        environment: Option<&[EnvironmentVariable]>,
        context_size: Option<u64>,
    ) -> String {
        let line = |command: &str, arguments: Vec<OsString>| command_string(command, &arguments);
        let extra = extra_args.into_iter().flatten().map(ToString::to_string).collect::<Vec<_>>();
        let arguments = args![
            "--offline",
            "--jinja",
            ("--batch-size", "2048"),
            ("--host", "0.0.0.0"),
            ("--sleep-idle-seconds", "600"),
            ("--tools", "all"),
            ("--ubatch-size", "2048")
        ];
        let defaults = arguments.iter().enumerate().filter_map(|(index, option)| {
            let option = option.to_string_lossy();
            option.starts_with("--").then(|| {
                let values = index
                    .checked_add(1)
                    .and_then(|index| arguments.get(index))
                    .filter(|value| !value.to_string_lossy().starts_with("--"))
                    .cloned()
                    .into_iter()
                    .collect();
                line(&option, values)
            })
        });
        let model = gguf_path.map(|path| line("--model", args![path]));
        let context = context_size.map(|size| line("--ctx-size", args![size.to_string()]));
        let extra = extra.into_iter().map(OsString::from).collect::<Vec<_>>();
        let extra = extra.split_first().map(|(command, arguments)| command_string(command, arguments));
        let environment = environment.into_iter().flatten().map(|entry| command_string(format!("env:{entry}"), &[]));
        once(command_string(executable, &[]))
            .chain(model)
            .chain(once(line("--port", args!["${PORT}"])))
            .chain(context)
            .chain(defaults)
            .chain(extra)
            .chain(environment)
            .collect::<Vec<_>>()
            .join("\n  ")
    }
    /// Build a llama-swap model entry from resolved model details
    pub fn model_entry(&self, model: &ModelDetails) -> Value {
        let model_name = model.name.as_deref().or(model.id.as_deref()).unwrap_or("unknown");
        let overrides = self.models.as_ref().and_then(|models| models.get(model_name));
        let aliases = overrides
            .and_then(|config| config.aliases.as_ref())
            .map(|aliases| Value::Sequence(aliases.iter().map(|alias| Value::String(alias.as_str().to_string())).collect()));
        let context_size = overrides.and_then(|config| config.context_size).or(self.context_size);
        let command = Self::build_command(
            overrides
                .and_then(|config| config.executable.as_deref())
                .or(self.executable.as_deref())
                .unwrap_or("llama-server"),
            model.path.as_deref(),
            overrides.and_then(|config| config.extra_args.as_deref()).or(self.extra_args.as_deref()),
            overrides.and_then(|config| config.environment.as_deref()).or(self.environment.as_deref()),
            context_size,
        );
        let metadata = Value::Mapping(once((Value::String("acorn".to_string()), Value::Bool(true))).collect());
        let mapping = once((Value::String("proxy".to_string()), Value::String("http://127.0.0.1:${PORT}".to_string())))
            .chain(once((Value::String("cmd".to_string()), Value::String(command))))
            .chain(
                overrides
                    .and_then(|config| config.ttl)
                    .or(self.ttl)
                    .map(|ttl| (Value::String("ttl".to_string()), Value::Number(Number::from(ttl)))),
            )
            .chain(aliases.map(|value| (Value::String("aliases".to_string()), value)))
            .chain(once((Value::String("metadata".to_string()), metadata)))
            .collect();
        Value::Mapping(mapping)
    }
    /// Merge synchronized models into an existing llama-swap document
    pub fn upsert(&self, existing: Value, models: &[ModelDetails], prune: bool) -> ApiResult<Value> {
        match existing {
            | Value::Null => self.upsert(Value::Mapping(Default::default()), models, prune),
            | Value::Mapping(root) => {
                let mut root = merge_mappings(Self::base_config(), root);
                let models_key = Value::String("models".to_string());
                root.remove(Value::String("modelsDir".to_string()));
                let existing_models = root
                    .remove(&models_key)
                    .and_then(|value| match value {
                        | Value::Mapping(models) => Some(models),
                        | _ => None,
                    })
                    .unwrap_or_default();
                let current_ids = models
                    .iter()
                    .filter_map(|model| model.name.as_ref().or(model.id.as_ref()))
                    .cloned()
                    .collect::<BTreeSet<_>>();
                let retained = existing_models
                    .into_iter()
                    .filter(|(identifier, model)| {
                        !prune || identifier.as_str().is_some_and(|identifier| current_ids.contains(identifier)) || !is_managed(model)
                    })
                    .collect::<Mapping>();
                let merged = models.iter().fold(retained, |mut entries, model| {
                    let identifier = model
                        .name
                        .as_ref()
                        .or(model.id.as_ref())
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    let key = Value::String(identifier);
                    let existing = entries.remove(&key).unwrap_or_else(|| Value::Mapping(Default::default()));
                    entries.insert(key, merge_model(existing, self.model_entry(model)));
                    entries
                });
                root.insert(models_key, Value::Mapping(merged));
                Ok(Value::Mapping(root))
            }
            | _ => Err(eyre!("Existing llama-swap configuration root must be a mapping")),
        }
    }
    fn base_config() -> Mapping {
        let proxy = once((Value::String("listen".to_string()), Value::String("http://0.0.0.0:10732".to_string()))).collect();
        let macros = [
            ("context_size", "${env.LLAMA_ARG_CTX_SIZE}"),
            ("parallel", "${env.LLAMA_ARG_N_PARALLEL}"),
            ("models_dir", "${env.HOME}/.models"),
            ("ssl_key", "${env.HOME}/certs/my.key"),
            ("ssl_cert", "${env.HOME}/certs/my.pem"),
        ]
        .into_iter()
        .map(|(key, value)| (Value::String(key.to_string()), Value::String(value.to_string())))
        .collect();
        [
            (Value::String("healthCheckTimeout".to_string()), Value::Number(Number::from(500))),
            (Value::String("proxy".to_string()), Value::Mapping(proxy)),
            (Value::String("macros".to_string()), Value::Mapping(macros)),
        ]
        .into_iter()
        .collect()
    }
}
impl SyncInner for Inner {
    fn merge(self, overrides: Self) -> Self {
        Self {
            models_directory: overrides.models_directory.or(self.models_directory),
            executable: overrides.executable.or(self.executable),
            context_size: overrides.context_size.or(self.context_size),
            ttl: overrides.ttl.or(self.ttl),
            extra_args: overrides.extra_args.or(self.extra_args),
            environment: overrides.environment.or(self.environment),
            models: overrides.models.or(self.models),
        }
    }
    fn merge_cli_overrides(self, overrides: Self) -> Self {
        Self {
            models_directory: overrides.models_directory.or(self.models_directory),
            ..self
        }
    }
}
impl Validate for ModelValidation<'_> {
    fn validate(&self) -> Result<(), ValidationReport> {
        self.config
            .models
            .as_ref()
            .into_iter()
            .flat_map(|models| models.iter())
            .try_fold(BTreeSet::new(), |mut aliases, (model_id, config)| {
                match self.model_ids.iter().any(|candidate| candidate == model_id) {
                    | false => Err(validation_errors(
                        "unknown_model",
                        format!("llamaSwap.models contains unknown model override '{model_id}'"),
                    )),
                    | true => config
                        .aliases
                        .as_ref()
                        .into_iter()
                        .flatten()
                        .try_for_each(|alias| {
                            let alias = alias.as_str();
                            match (
                                self.model_ids.iter().any(|candidate| candidate == alias),
                                aliases.insert(alias.to_string()),
                            ) {
                                | (true, _) => Err(validation_errors(
                                    "model_alias",
                                    format!("llamaSwap alias '{alias}' conflicts with a configured model ID"),
                                )),
                                | (_, false) => Err(validation_errors("duplicate_alias", format!("Duplicate llamaSwap alias '{alias}'"))),
                                | _ => Ok(()),
                            }
                        })
                        .map(|()| aliases),
                }
            })
            .map(|_| ())
    }
}
impl<'a> From<(&'a Config, &'a [String])> for ModelValidation<'a> {
    fn from((config, model_ids): (&'a Config, &'a [String])) -> Self {
        Self { config, model_ids }
    }
}
impl From<Argument> for String {
    fn from(value: Argument) -> Self {
        value.to_string()
    }
}
fn is_keyvalue_string(value: &str) -> Result<(), ValidationError> {
    let is_valid = value
        .split_once('=')
        .is_some_and(|(key, _)| !key.trim().is_empty() && !key.chars().any(char::is_whitespace));
    match is_valid {
        | true => Ok(()),
        | false => Err(ValidationError::new("keyvalue").with_message("Provide a valid KEY=VALUE entry")),
    }
}
fn is_managed(model: &Value) -> bool {
    model
        .as_mapping()
        .and_then(|model| model.get(Value::String("metadata".to_string())))
        .and_then(Value::as_mapping)
        .and_then(|metadata| metadata.get(Value::String("acorn".to_string())))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}
fn merge_mappings(defaults: Mapping, overrides: Mapping) -> Mapping {
    defaults.into_iter().fold(overrides, |mut merged, (key, default_value)| {
        let value = match (merged.get(&key), default_value) {
            | (Some(Value::Mapping(overrides)), Value::Mapping(defaults)) => Some(Value::Mapping(merge_mappings(defaults, overrides.clone()))),
            | (None, value) => Some(value),
            | _ => None,
        };
        if let Some(value) = value {
            merged.insert(key, value);
        }
        merged
    })
}
fn merge_model(existing: Value, generated: Value) -> Value {
    let owned = vec!["cmd", "command", "ttl", "aliases"]
        .to_strings()
        .into_iter()
        .map(Value::String)
        .collect::<Vec<_>>();
    let mut existing = match existing {
        | Value::Mapping(mapping) => mapping,
        | _ => Default::default(),
    };
    let mut generated = match generated {
        | Value::Mapping(mapping) => mapping,
        | _ => Default::default(),
    };
    let proxy_key = Value::String("proxy".to_string());
    if existing.contains_key(&proxy_key) {
        generated.remove(&proxy_key);
    }
    let metadata_key = Value::String("metadata".to_string());
    let metadata = existing
        .remove(&metadata_key)
        .and_then(|value| match value {
            | Value::Mapping(mapping) => Some(mapping),
            | _ => None,
        })
        .unwrap_or_default()
        .into_iter()
        .chain(
            generated
                .remove(&metadata_key)
                .and_then(|value| match value {
                    | Value::Mapping(mapping) => Some(mapping),
                    | _ => None,
                })
                .unwrap_or_default(),
        )
        .collect();
    Value::Mapping(
        existing
            .into_iter()
            .filter(|(key, _)| !owned.contains(key))
            .chain(generated)
            .chain(once((metadata_key, Value::Mapping(metadata))))
            .collect(),
    )
}
fn validation_errors(code: &'static str, message: String) -> ValidationReport {
    let mut errors = ValidationReport::new();
    errors.add("models", ValidationError::new(code).with_message(message));
    errors
}
