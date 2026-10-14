#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::super::{
    llama_swap::{Alias, Argument, Config, EnvironmentVariable, Inner, ModelOverride, ModelValidation},
    SyncTarget,
};
use crate::io::home_directory;
use crate::util::constants::app::DEFAULT_LLAMA_SWAP_CONFIG_PATH;
use acorn_core::OneOrMany;
use acorn_schema::agent::ModelDetails;
use acorn_schema::validation::Validate;
use serde_norway::Value;

#[test]
fn test_argument_validation_rejects_reserved_options() {
    ["--port", "--port=9000", "--model", "--model=/tmp/model.gguf", "-m=/tmp/model.gguf"]
        .into_iter()
        .for_each(|argument| assert!(Argument::from(argument).validate().is_err()));
    assert!(Argument::from("--flash-attn").validate().is_ok());
}
#[test]
fn test_build_command_simple() {
    let cmd = Inner::build_command("llama-server", Some("/models/qwen.gguf"), None, None, Some(8192));
    assert_eq!(
        cmd,
        "llama-server\n  --model /models/qwen.gguf\n  --port ${PORT}\n  --ctx-size 8192\n  --offline\n  --jinja\n  --batch-size 2048\n  --host 0.0.0.0\n  --sleep-idle-seconds 600\n  --tools all\n  --ubatch-size 2048"
    );
}
#[test]
fn test_build_command_with_environment() {
    let env = vec![EnvironmentVariable::from("CUDA_VISIBLE_DEVICES=0")];
    let cmd = Inner::build_command("llama-server", Some("/models/qwen.gguf"), None, Some(&env), None);
    assert!(cmd.contains("env:CUDA_VISIBLE_DEVICES=0"));
}
#[test]
fn test_build_command_with_extra_args() {
    let extras = vec![
        Argument::from("--batch-size"),
        Argument::from("1024"),
        Argument::from("--flash-attn"),
        Argument::from("on"),
    ];
    let cmd = Inner::build_command("llama-server", Some("/models/qwen.gguf"), Some(&extras), None, None);
    assert!(cmd.contains("--batch-size 1024"));
    assert!(cmd
        .find("--batch-size 2048")
        .zip(cmd.rfind("--batch-size 1024"))
        .is_some_and(|(default, override_)| default < override_));
    assert!(cmd.contains("--flash-attn"));
    assert!(cmd.contains("on"));
}
#[test]
fn test_build_command_with_optional_values() {
    let cmd = Inner::build_command("llama-server", None, None, None, None);
    assert!(cmd.starts_with("llama-server\n  --port ${PORT}\n  --offline"));
    assert!(cmd.ends_with("--ubatch-size 2048"));
}
#[test]
fn test_build_command_with_spaces_in_path() {
    let cmd = Inner::build_command("llama-server", Some("/path with spaces/model.gguf"), None, None, None);
    assert!(cmd.contains('"'));
}
#[test]
fn test_resolve_path_uses_llama_swap_user_config() {
    assert_eq!(
        Config::resolve_path(None).unwrap(),
        home_directory(DEFAULT_LLAMA_SWAP_CONFIG_PATH).unwrap()
    );
}
#[test]
fn test_upsert_adds_gold_base_defaults() {
    let models = [ModelDetails::init().id("qwen").name("qwen").path("/models/qwen.gguf").build()];
    let output = Config::default()
        .upsert(Value::Mapping(Default::default()), &models, false)
        .and_then(|value| serde_norway::to_string(&value).map_err(Into::into))
        .unwrap();
    assert!(output.contains("healthCheckTimeout: 500"));
    assert!(output.contains("listen: http://0.0.0.0:10732"));
    assert!(output.contains("context_size: ${env.LLAMA_ARG_CTX_SIZE}"));
    assert!(output.contains("parallel: ${env.LLAMA_ARG_N_PARALLEL}"));
    assert!(output.contains("models_dir: ${env.HOME}/.models"));
    assert!(output.contains("ssl_key: ${env.HOME}/certs/my.key"));
    assert!(output.contains("ssl_cert: ${env.HOME}/certs/my.pem"));
    assert!(output.contains("proxy: http://127.0.0.1:${PORT}"));
    assert!(output.contains("--offline\n"));
    assert!(output.contains("--tools all\n"));
}
#[test]
fn test_upsert_preserves_unrelated_values_and_prunes_only_managed_models() {
    let existing: Value = serde_norway::from_str(
        r#"modelsDir: /legacy/models
healthCheckTimeout: 120
models:
  qwen:
    cmd: old
    proxy: http://127.0.0.1:9000
    metadata:
      owner: user
  stale:
    cmd: stale
    metadata:
      acorn: true
  custom:
    cmd: custom
"#,
    )
    .unwrap();
    let models = [ModelDetails::init().id("qwen").name("qwen").path("/models/qwen.gguf").build()];
    let config = Config::default();
    let additive = config.upsert(existing.clone(), &models, false).unwrap();
    let additive_text = serde_norway::to_string(&additive).unwrap();
    assert!(additive_text.contains("healthCheckTimeout: 120"));
    assert!(additive_text.contains("proxy: http://127.0.0.1:9000"));
    assert!(additive_text.contains("owner: user"));
    assert!(additive_text.contains("acorn: true"));
    assert!(additive_text.contains("stale:"));
    assert!(additive_text.contains("custom:"));
    assert!(additive_text.contains("cmd: |-"));
    assert!(additive_text.contains("llama-server\n"));
    assert!(additive_text.contains("--port ${PORT}\n"));
    assert!(additive_text.contains("--model /models/qwen.gguf\n"));
    assert!(!additive_text.contains("modelsDir"));
    let pruned = config.upsert(existing, &models, true).unwrap();
    let pruned_text = serde_norway::to_string(&pruned).unwrap();
    assert!(!pruned_text.contains("stale:"));
    assert!(pruned_text.contains("custom:"));
    let repeated = config.upsert(pruned, &models, true).unwrap();
    assert_eq!(pruned_text, serde_norway::to_string(&repeated).unwrap());
}
#[test]
fn test_validate_rejects_invalid_context_ttl_environment_and_aliases() {
    let model_ids = vec!["qwen".to_string(), "gemma".to_string()];
    let invalid = [
        Config {
            inner: Inner {
                context_size: Some(0),
                ..Default::default()
            },
            ..Default::default()
        },
        Config {
            inner: Inner {
                ttl: Some(-1),
                ..Default::default()
            },
            ..Default::default()
        },
        Config {
            inner: Inner {
                environment: Some(OneOrMany::One(EnvironmentVariable::from("INVALID"))),
                ..Default::default()
            },
            ..Default::default()
        },
        Config {
            inner: Inner {
                models: Some(
                    [
                        (
                            "qwen".to_string(),
                            ModelOverride {
                                aliases: Some(OneOrMany::One(Alias::from("shared"))),
                                ..Default::default()
                            },
                        ),
                        (
                            "gemma".to_string(),
                            ModelOverride {
                                aliases: Some(OneOrMany::One(Alias::from("shared"))),
                                ..Default::default()
                            },
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
                ..Default::default()
            },
            ..Default::default()
        },
    ];
    invalid.iter().for_each(|config| {
        assert!(Validate::validate(config).is_err() || ModelValidation::from((config, model_ids.as_slice())).validate().is_err());
    });
}
