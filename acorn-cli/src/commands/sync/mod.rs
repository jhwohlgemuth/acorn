use crate::cli::{arguments, Void};
use acorn::io::config::{ApplicationConfiguration, ModelEntry};
use acorn::io::sync::{self, Config};
use acorn::util::Label;
use tracing::info;

/// Synchronize local inference configuration from ACORN model entries
pub async fn run(args: &arguments::Sync) -> Void {
    let &arguments::Sync {
        ref config,
        opencode,
        llama_swap,
        ref models_dir,
        dry_run,
        prune,
        ..
    } = args;
    ApplicationConfiguration::load(config).and_then(|configuration| {
        let llama_swap_config = sync::llama_swap::Config::from(args);
        let opencode_config = sync::opencode::Config::from(args);
        let overrides = Config {
            llama_swap: (llama_swap_config.path.is_some() || llama_swap_config.models_directory.is_some()).then_some(llama_swap_config),
            opencode: opencode_config.path.is_some().then_some(opencode_config),
        };
        let sync_config = configuration.config.clone().unwrap_or_default().merge_cli_overrides(overrides);
        sync_config.resolve_models_dir(models_dir.as_deref()).and_then(|models_dir| {
            info!("{} Resolving models from configuration", Label::run());
            ModelEntry::requests(configuration.models.as_deref().unwrap_or_default()).and_then(|requests| {
                requests
                    .into_iter()
                    .map(|request| request.resolve(&models_dir))
                    .collect::<Result<Vec<_>, _>>()
                    .and_then(|resolved| {
                        sync_config.sync(sync::Options {
                            models: &resolved,
                            opencode,
                            llama_swap,
                            dry_run,
                            prune,
                        })
                    })
            })
        })
    })
}
impl From<&arguments::Sync> for sync::llama_swap::Config {
    fn from(args: &arguments::Sync) -> Self {
        Self {
            path: args.llama_swap_config.as_ref().map(|path| path.display().to_string()),
            models_directory: args.models_dir.as_ref().map(|path| path.display().to_string()),
            ..Default::default()
        }
    }
}
impl From<&arguments::Sync> for sync::opencode::Config {
    fn from(args: &arguments::Sync) -> Self {
        Self {
            path: args.opencode_config.as_ref().map(|path| path.display().to_string()),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )]
    use super::*;
    use acorn::prelude::PathBuf;

    #[test]
    fn test_resolve_sync_config_preserves_llama_swap_values_with_path_overrides() {
        let configuration = ApplicationConfiguration {
            config: Some(Config {
                llama_swap: Some(sync::llama_swap::Config {
                    path: Some("configured.yaml".to_string()),
                    executable: Some("my-server".to_string()),
                    context_size: Some(8192),
                    ..Default::default()
                }),
                opencode: None,
            }),
            ..Default::default()
        };
        let args = arguments::Sync {
            config: None,
            opencode: false,
            llama_swap: false,
            models_dir: Some(PathBuf::from("cli-models")),
            opencode_config: None,
            llama_swap_config: Some(PathBuf::from("cli.yaml")),
            dry_run: false,
            prune: false,
            verbose: clap_verbosity_flag::Verbosity::new(0, 0),
        };
        let llama_swap = sync::llama_swap::Config::from(&args);
        let opencode = sync::opencode::Config::from(&args);
        let overrides = Config {
            llama_swap: (llama_swap.path.is_some() || llama_swap.models_directory.is_some()).then_some(llama_swap),
            opencode: opencode.path.is_some().then_some(opencode),
        };
        let resolved = configuration.config.clone().unwrap_or_default().merge_cli_overrides(overrides);
        let llama = resolved.llama_swap.unwrap();
        assert_eq!(llama.path.as_deref(), Some("cli.yaml"));
        assert_eq!(llama.models_directory.as_deref(), Some("cli-models"));
        assert_eq!(llama.executable.as_deref(), Some("my-server"));
        assert_eq!(llama.context_size, Some(8192));
    }
    #[test]
    fn test_resolve_sync_config_preserves_opencode_values_with_path_override() {
        let configuration = ApplicationConfiguration {
            config: Some(Config {
                llama_swap: None,
                opencode: Some(sync::opencode::Config {
                    path: Some("configured.jsonc".to_string()),
                    base_url: "http://configured.test/v1".to_string(),
                    provider_id: "configured-provider".to_string(),
                    provider_name: "Configured Provider".to_string(),
                    default_model: Some("configured-model".to_string()),
                }),
            }),
            ..Default::default()
        };
        let args = arguments::Sync {
            config: None,
            opencode: false,
            llama_swap: false,
            models_dir: None,
            opencode_config: Some(PathBuf::from("cli.jsonc")),
            llama_swap_config: None,
            dry_run: false,
            prune: false,
            verbose: clap_verbosity_flag::Verbosity::new(0, 0),
        };
        let opencode = sync::opencode::Config::from(&args);
        let resolved = configuration.config.clone().unwrap_or_default().merge_cli_overrides(Config {
            llama_swap: None,
            opencode: Some(opencode),
        });
        let opencode = resolved.opencode.unwrap();
        assert_eq!(opencode.path.as_deref(), Some("cli.jsonc"));
        assert_eq!(opencode.base_url, "http://configured.test/v1");
        assert_eq!(opencode.provider_id, "configured-provider");
        assert_eq!(opencode.provider_name, "Configured Provider");
        assert_eq!(opencode.default_model.as_deref(), Some("configured-model"));
    }
    #[test]
    fn test_sync_config_cli_merge_adds_unconfigured_target() {
        let merged = Config::default().merge_cli_overrides(Config {
            llama_swap: Some(sync::llama_swap::Config {
                path: Some("cli.yaml".to_string()),
                ..Default::default()
            }),
            opencode: None,
        });
        assert_eq!(merged.llama_swap.unwrap().path.as_deref(), Some("cli.yaml"));
    }
    #[test]
    fn test_sync_config_merge_prefers_non_path_overrides() {
        let base = Config {
            llama_swap: Some(sync::llama_swap::Config {
                executable: Some("base-server".to_string()),
                ..Default::default()
            }),
            opencode: Some(sync::opencode::Config {
                base_url: "http://base.test/v1".to_string(),
                provider_id: "base-provider".to_string(),
                ..Default::default()
            }),
        };
        let overrides = Config {
            llama_swap: Some(sync::llama_swap::Config {
                executable: Some("override-server".to_string()),
                ..Default::default()
            }),
            opencode: Some(sync::opencode::Config {
                base_url: "http://override.test/v1".to_string(),
                provider_id: "override-provider".to_string(),
                ..Default::default()
            }),
        };
        let merged = base.merge(overrides);
        let llama_swap = merged.llama_swap.unwrap();
        let opencode = merged.opencode.unwrap();
        assert_eq!(llama_swap.executable.as_deref(), Some("override-server"));
        assert_eq!(opencode.base_url, "http://override.test/v1");
        assert_eq!(opencode.provider_id, "override-provider");
    }
}
