//! llama-swap configuration types
//!
//! llama-swap is a lightweight HTTP proxy that manages multiple local LLM
//! models behind a single OpenAI-compatible endpoint. Each model entry is
//! a command that launches `llama-server` with the appropriate flags.
//!
//! See <https://github.com/mostlygeek/llama-swap/blob/main/docs/configuration.md>
use crate::args;
use crate::io::{home_directory, ApiResult};
use crate::prelude::OsString;
use crate::schema::agent::ModelDetails;
use crate::util::cmd::command_string;
use crate::util::constants::app::DEFAULT_LLAMA_SWAP_CONFIG_PATH;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use core::{fmt, iter::once};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use validator::{Validate, ValidationError, ValidationErrors};

/// Human-readable alias for a synchronized model
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(transparent)]
pub struct Alias {
    #[validate(length(min = 1))]
    value: String,
}
/// Additional llama-server command-line argument
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(from = "String", into = "String")]
pub enum Argument {
    /// Argument reserved for ACORN's llama-swap integration
    Reserved(String),
    /// User-provided llama-server argument
    Value(String),
}
/// Root llama-swap configuration
///
/// The `models` map is keyed by ACORN model name. Each entry's `command`
/// is generated from the resolved GGUF path and configured defaults.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Path to the llama-swap configuration file on disk
    #[serde(skip_serializing)]
    #[validate(length(min = 1))]
    pub path: Option<String>,
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
/// Validated llama-swap environment entry serialized as `KEY=VALUE`
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(transparent)]
pub struct EnvironmentVariable {
    #[validate(custom(function = "is_keyvalue_string"))]
    value: String,
}
/// Per-model override entry keyed by ACORN model name
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
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
impl Config {
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
        let context_args = context_size
            .map(|size| vec!["--ctx-size".to_string(), size.to_string()])
            .unwrap_or_default();
        let model_args = gguf_path.map(|path| vec!["--model".to_string(), path.to_string()]).unwrap_or_default();
        let environment_args = environment.into_iter().flatten().map(|entry| format!("env:{entry}"));
        let arguments: Vec<OsString> = args![
            "--port",
            "${PORT}",
            ..model_args,
            ..context_args,
            ..extra_args.into_iter().flatten().map(ToString::to_string),
            ..environment_args
        ];
        command_string(executable, &arguments)
    }
    /// Merge populated runtime overrides into this configuration
    pub fn merge(self, overrides: Self) -> Self {
        Self {
            path: overrides.path.or(self.path),
            models_directory: overrides.models_directory.or(self.models_directory),
            executable: overrides.executable.or(self.executable),
            context_size: overrides.context_size.or(self.context_size),
            ttl: overrides.ttl.or(self.ttl),
            extra_args: overrides.extra_args.or(self.extra_args),
            environment: overrides.environment.or(self.environment),
            models: overrides.models.or(self.models),
        }
    }
    /// Merge CLI-supported overrides while preserving configured model defaults
    pub fn merge_cli_overrides(self, overrides: Self) -> Self {
        Self {
            path: overrides.path.or(self.path),
            models_directory: overrides.models_directory.or(self.models_directory),
            ..self
        }
    }
    /// Build a llama-swap model entry from resolved model details
    pub fn model_entry(&self, model: &ModelDetails) -> serde_norway::Value {
        let model_name = model.name.as_deref().or(model.id.as_deref()).unwrap_or("unknown");
        let overrides = self.models.as_ref().and_then(|models| models.get(model_name));
        let aliases = overrides.and_then(|config| config.aliases.as_ref()).map(|aliases| {
            serde_norway::Value::Sequence(
                aliases
                    .iter()
                    .map(|alias| serde_norway::Value::String(alias.as_str().to_string()))
                    .collect(),
            )
        });
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
        let mapping = once((serde_norway::Value::String("command".to_string()), serde_norway::Value::String(command)))
            .chain(overrides.and_then(|config| config.ttl).or(self.ttl).map(|ttl| {
                (
                    serde_norway::Value::String("ttl".to_string()),
                    serde_norway::Value::Number(serde_norway::Number::from(ttl)),
                )
            }))
            .chain(aliases.map(|value| (serde_norway::Value::String("aliases".to_string()), value)))
            .collect();
        serde_norway::Value::Mapping(mapping)
    }
    /// Resolve the configured llama-swap path or the user default.
    pub fn resolve_path(&self) -> ApiResult<String> {
        self.path.clone().map_or_else(
            || home_directory(DEFAULT_LLAMA_SWAP_CONFIG_PATH).map(|path| path.display().to_string()),
            Ok,
        )
    }
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
impl From<String> for Argument {
    fn from(value: String) -> Self {
        match value.split_once('=').map_or(value.as_str(), |(option, _)| option) {
            | "--port" | "--model" | "-m" => Self::Reserved(value),
            | _ => Self::Value(value),
        }
    }
}
impl From<&str> for Argument {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}
impl From<Argument> for String {
    fn from(value: Argument) -> Self {
        value.to_string()
    }
}
impl<'a> From<(&'a Config, &'a [String])> for ModelValidation<'a> {
    fn from((config, model_ids): (&'a Config, &'a [String])) -> Self {
        Self { config, model_ids }
    }
}
impl Validate for Argument {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            | Self::Reserved(_) => {
                let mut errors = ValidationErrors::new();
                errors.add(
                    "argument",
                    ValidationError::new("reserved").with_message("Cannot contain --port, --model, or -m".into()),
                );
                Err(errors)
            }
            | Self::Value(_) => Ok(()),
        }
    }
}
impl Validate for ModelValidation<'_> {
    fn validate(&self) -> Result<(), ValidationErrors> {
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
fn is_keyvalue_string(value: &str) -> Result<(), ValidationError> {
    let is_valid = value
        .split_once('=')
        .is_some_and(|(key, _)| !key.trim().is_empty() && !key.chars().any(char::is_whitespace));
    match is_valid {
        | true => Ok(()),
        | false => Err(ValidationError::new("keyvalue").with_message("Provide a valid KEY=VALUE entry".into())),
    }
}
fn validation_errors(code: &'static str, message: String) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    errors.add("models", ValidationError::new(code).with_message(message.into()));
    errors
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_argument_validation_rejects_reserved_options() {
        ["--port", "--port=9000", "--model", "--model=/tmp/model.gguf", "-m=/tmp/model.gguf"]
            .into_iter()
            .for_each(|argument| assert!(Argument::from(argument).validate().is_err()));
        assert!(Argument::from("--flash-attn").validate().is_ok());
    }
    #[test]
    fn test_build_command_simple() {
        let cmd = Config::build_command("llama-server", Some("/models/qwen.gguf"), None, None, Some(8192));
        assert_eq!(cmd, "llama-server --port ${PORT} --model /models/qwen.gguf --ctx-size 8192");
    }
    #[test]
    fn test_build_command_with_environment() {
        let env = vec![EnvironmentVariable::from("CUDA_VISIBLE_DEVICES=0")];
        let cmd = Config::build_command("llama-server", Some("/models/qwen.gguf"), None, Some(&env), None);
        assert!(cmd.contains("env:CUDA_VISIBLE_DEVICES=0"));
    }
    #[test]
    fn test_build_command_with_extra_args() {
        let extras = vec![Argument::from("--flash-attn"), Argument::from("on")];
        let cmd = Config::build_command("llama-server", Some("/models/qwen.gguf"), Some(&extras), None, None);
        assert!(cmd.contains("--flash-attn"));
        assert!(cmd.contains("on"));
    }
    #[test]
    fn test_build_command_with_optional_values() {
        let cmd = Config::build_command("llama-server", None, None, None, None);
        assert_eq!(cmd, "llama-server --port ${PORT}");
    }
    #[test]
    fn test_build_command_with_spaces_in_path() {
        let cmd = Config::build_command("llama-server", Some("/path with spaces/model.gguf"), None, None, None);
        assert!(cmd.contains('"'));
    }
    #[test]
    fn test_resolve_path_uses_llama_swap_user_config() {
        assert_eq!(
            Config::default().resolve_path().unwrap(),
            home_directory(DEFAULT_LLAMA_SWAP_CONFIG_PATH).unwrap().display().to_string()
        );
    }
    #[test]
    fn test_validate_rejects_invalid_context_ttl_environment_and_aliases() {
        let model_ids = vec!["qwen".to_string(), "gemma".to_string()];
        let invalid = [
            Config {
                context_size: Some(0),
                ..Default::default()
            },
            Config {
                ttl: Some(-1),
                ..Default::default()
            },
            Config {
                environment: Some(vec![EnvironmentVariable::from("INVALID")]),
                ..Default::default()
            },
            Config {
                models: Some(
                    [
                        (
                            "qwen".to_string(),
                            ModelOverride {
                                aliases: Some(vec![Alias::from("shared")]),
                                ..Default::default()
                            },
                        ),
                        (
                            "gemma".to_string(),
                            ModelOverride {
                                aliases: Some(vec![Alias::from("shared")]),
                                ..Default::default()
                            },
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
                ..Default::default()
            },
        ];
        invalid.iter().for_each(|config| {
            assert!(Validate::validate(config).is_err() || ModelValidation::from((config, model_ids.as_slice())).validate().is_err());
        });
    }
}
