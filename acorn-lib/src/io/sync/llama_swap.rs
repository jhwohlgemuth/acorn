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
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::iter::once;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Root llama-swap configuration
///
/// The `models` map is keyed by ACORN model name. Each entry's `command`
/// is generated from the resolved GGUF path and configured defaults.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Path to the llama-swap configuration file on disk
    #[serde(skip)]
    pub path: Option<String>,
    /// Default directory where downloaded model weights live
    pub models_directory: Option<String>,
    /// Executable name or path for `llama-server`
    pub executable: Option<String>,
    /// Default context window size for all models
    pub context_size: Option<u64>,
    /// Default time-to-live in seconds; 0 means no expiry
    pub ttl: Option<i64>,
    /// Extra command-line arguments applied to every model command
    pub extra_args: Option<Vec<String>>,
    /// Extra environment variables as `KEY=VALUE` entries applied to every model
    pub environment: Option<Vec<String>>,
    /// Per-model overrides keyed by ACORN model name
    pub models: Option<BTreeMap<String, ModelOverride>>,
}
/// Per-model override entry keyed by ACORN model name
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelOverride {
    /// Command to launch the inference server for this model
    pub command: Option<String>,
    /// Model-specific context window size (overrides the global default)
    pub context_size: Option<u64>,
    /// Time-to-live in seconds; 0 means no expiry
    pub ttl: Option<i64>,
    /// Human-readable aliases for selecting this model
    pub aliases: Option<Vec<String>>,
    /// Additional command-line arguments for `llama-server`
    pub extra_args: Option<Vec<String>>,
    /// Extra environment variables as `KEY=VALUE` entries
    pub environment: Option<Vec<String>>,
    /// Executable name or path for this model (overrides global)
    pub executable: Option<String>,
}
impl Config {
    /// Build a shell-safe `llama-server` command string for a given GGUF model path
    /// ### Note
    /// The command uses `${PORT}` as a placeholder that llama-swap replaces at runtime
    pub fn build_command(executable: &str, gguf_path: &str, context_size: Option<u64>, extra_args: &[String], environment: &[String]) -> String {
        let context_args = context_size
            .map(|size| vec!["--ctx-size".to_string(), size.to_string()])
            .unwrap_or_default();
        let environment_args = environment.iter().map(|entry| format!("env:{entry}"));
        let arguments: Vec<OsString> = args![
            "--port",
            "${PORT}",
            "--model",
            gguf_path,
            ..context_args,
            ..extra_args.iter().cloned(),
            ..environment_args
        ];
        command_string(executable, &arguments)
    }
    /// Build a llama-swap model entry from resolved model details
    pub fn model_entry(&self, model: &ModelDetails) -> serde_norway::Value {
        let model_name = model.name.as_deref().or(model.id.as_deref()).unwrap_or("unknown");
        let overrides = self.models.as_ref().and_then(|models| models.get(model_name));
        let aliases = overrides
            .and_then(|config| config.aliases.as_ref())
            .map(|aliases| serde_norway::Value::Sequence(aliases.iter().cloned().map(serde_norway::Value::String).collect()));
        let context_size = overrides.and_then(|config| config.context_size).or(self.context_size);
        let command = Self::build_command(
            overrides
                .and_then(|config| config.executable.as_deref())
                .or(self.executable.as_deref())
                .unwrap_or("llama-server"),
            model.path.as_deref().unwrap_or_default(),
            context_size,
            overrides
                .and_then(|config| config.extra_args.as_deref())
                .or(self.extra_args.as_deref())
                .unwrap_or(&[]),
            overrides
                .and_then(|config| config.environment.as_deref())
                .or(self.environment.as_deref())
                .unwrap_or(&[]),
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
    /// Return `true` if `extra_args` contains reserved llama-swap options.
    ///
    /// `--port`, `--model`, and `-m` are owned by ACORN and must not appear
    /// in user-provided extra arguments.
    pub fn has_reserved_extra_args(extra_args: &[String]) -> bool {
        extra_args.iter().any(|arg| arg == "--port" || arg == "--model" || arg == "-m")
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_build_command_simple() {
        let cmd = Config::build_command("llama-server", "/models/qwen.gguf", Some(8192), &[], &[]);
        assert_eq!(cmd, "llama-server --port ${PORT} --model /models/qwen.gguf --ctx-size 8192");
    }
    #[test]
    fn test_build_command_with_extra_args() {
        let extras = vec!["--flash-attn".to_string(), "on".to_string()];
        let cmd = Config::build_command("llama-server", "/models/qwen.gguf", None, &extras, &[]);
        assert!(cmd.contains("--flash-attn"));
        assert!(cmd.contains("on"));
    }
    #[test]
    fn test_build_command_with_spaces_in_path() {
        let cmd = Config::build_command("llama-server", "/path with spaces/model.gguf", None, &[], &[]);
        assert!(cmd.contains('"'));
    }
    #[test]
    fn test_build_command_with_environment() {
        let env = vec!["CUDA_VISIBLE_DEVICES=0".to_string()];
        let cmd = Config::build_command("llama-server", "/models/qwen.gguf", None, &[], &env);
        assert!(cmd.contains("env:CUDA_VISIBLE_DEVICES=0"));
    }
    #[test]
    fn test_has_reserved_extra_args_detects_port() {
        assert!(Config::has_reserved_extra_args(&["--port".to_string()]));
        assert!(Config::has_reserved_extra_args(&["--model".to_string()]));
        assert!(!Config::has_reserved_extra_args(&["--flash-attn".to_string()]));
    }
    #[test]
    fn test_resolve_path_uses_llama_swap_user_config() {
        assert_eq!(
            Config::default().resolve_path().unwrap(),
            home_directory(DEFAULT_LLAMA_SWAP_CONFIG_PATH).unwrap().display().to_string()
        );
    }
}
