use crate::cli::{arguments, Void};
use acorn::io::config::{ApplicationConfiguration, ModelEntry};
use acorn::io::sync::{self, Config};
use acorn::schema::agent::ModelSelectors;
use acorn::util::Label;
use tracing::info;
/// Synchronize local inference configuration from ACORN model entries
pub async fn run(args: &arguments::Sync, offline: bool) -> Void {
    let &arguments::Sync {
        ref config,
        ref model_file,
        force,
        opencode,
        llama_swap,
        ref models_dir,
        dry_run,
        no_color,
        prune,
        ..
    } = args;
    match ModelSelectors::default().resolve(model_file, offline).await {
        | Ok(selectors) => {
            let file_entries = selectors.iter().map(|selector| ModelEntry::Selector(selector.to_string()));
            ApplicationConfiguration::load(config).and_then(|configuration| {
                let llama_swap_config = sync::llama_swap::Config::from(args);
                let opencode_config = sync::opencode::Config::from(args);
                let overrides = Config {
                    llama_swap: (llama_swap_config.path.is_some() || llama_swap_config.models_directory.is_some()).then_some(llama_swap_config),
                    opencode: opencode_config.path.is_some().then_some(opencode_config),
                };
                let sync_config = configuration.config.clone().unwrap_or_default().merge_cli_overrides(overrides);
                let entries = configuration
                    .models
                    .unwrap_or_default()
                    .into_iter()
                    .chain(file_entries)
                    .collect::<Vec<_>>();
                sync_config.resolve_models_dir(models_dir.as_deref()).and_then(|models_dir| {
                    info!("{} Resolving models for synchronization", Label::run());
                    ModelEntry::resolve(&entries, &models_dir, force).and_then(|resolved| {
                        let options = sync::Options::init()
                            .models(&resolved)
                            .opencode(opencode)
                            .llama_swap(llama_swap)
                            .dry_run(dry_run)
                            .no_color(no_color)
                            .prune(prune)
                            .force(force)
                            .build();
                        sync_config.sync(options)
                    })
                })
            })
        }
        | Err(why) => Err(why),
    }
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
    use crate::test::util::{temp_test_dir, TestCleanup};
    use acorn::prelude::{create_dir_all, read_to_string, write, PathBuf};

    #[test]
    fn test_resolve_models_skips_unresolved_entries() {
        let models_dir = temp_test_dir("sync_skip_unresolved");
        let _cleanup = TestCleanup::new(models_dir.clone());
        let model_dir = models_dir.join("mozilla").join("test-llama");
        create_dir_all(&model_dir).unwrap();
        write(model_dir.join("tiny-llama.gguf"), b"model").unwrap();
        let entries = vec![
            ModelEntry::Selector("mozilla/test-llama".to_string()),
            ModelEntry::Selector("openai/gpt-oss-2b".to_string()),
        ];
        let resolved = ModelEntry::resolve(&entries, &models_dir, false).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].id.as_deref(), Some("mozilla/test-llama"));
    }
    #[test]
    fn test_force_assumes_model_path_from_identifier() {
        let models_dir = PathBuf::from("models");
        let entries = [ModelEntry::Selector("acme/missing".to_string())];
        let resolved = ModelEntry::resolve(&entries, &models_dir, true).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].id.as_deref(), Some("acme/missing"));
        assert_eq!(
            resolved[0].path.as_deref(),
            Some(models_dir.join("acme/missing").display().to_string().as_str())
        );
    }
    #[test]
    fn test_application_configuration_sync_writes_selected_target() {
        let root = temp_test_dir("sync_selected_models");
        let _cleanup = TestCleanup::new(root.clone());
        let models_dir = root.join("models");
        let model_dir = models_dir.join("acme").join("model");
        let opencode_path = root.join("opencode.json");
        let llama_swap_path = root.join("llama-swap.yaml");
        create_dir_all(&model_dir).unwrap();
        write(model_dir.join("model.gguf"), b"model").unwrap();
        let configuration = ApplicationConfiguration {
            config: Some(Config {
                llama_swap: Some(sync::llama_swap::Config {
                    path: Some(llama_swap_path.display().to_string()),
                    ..Default::default()
                }),
                opencode: Some(sync::opencode::Config {
                    path: Some(opencode_path.display().to_string()),
                    ..Default::default()
                }),
            }),
            ..Default::default()
        };
        configuration
            .sync(sync::Options {
                entries: &[ModelEntry::Selector("acme/model".to_string())],
                opencode: true,
                dry_run: true,
                models_dir: Some(&models_dir),
                ..Default::default()
            })
            .unwrap();
        assert!(!opencode_path.exists());
        configuration
            .sync(sync::Options {
                entries: &[ModelEntry::Selector("acme/model".to_string())],
                opencode: true,
                models_dir: Some(&models_dir),
                ..Default::default()
            })
            .unwrap();
        assert!(opencode_path.is_file());
        assert!(!llama_swap_path.exists());
    }
    #[tokio::test]
    async fn test_sync_combines_configured_models_with_model_file() {
        let root = temp_test_dir("sync_model_file");
        let _cleanup = TestCleanup::new(root.clone());
        let models_dir = root.join("models");
        let config_path = root.join("acorn.json");
        let model_file = root.join("models.txt");
        let opencode_path = root.join("opencode.json");
        ["configured", "listed"].iter().for_each(|name| {
            let model_dir = models_dir.join("acme").join(name);
            create_dir_all(&model_dir).unwrap();
            write(model_dir.join("model.gguf"), b"model").unwrap();
        });
        write(&config_path, br#"{"models":["acme/configured"]}"#).unwrap();
        write(&model_file, b"acme/listed\n").unwrap();
        run(
            &arguments::Sync {
                config: Some(config_path),
                model_file: Some(model_file.display().to_string()),
                force: false,
                opencode: true,
                llama_swap: false,
                models_dir: Some(models_dir),
                opencode_config: Some(opencode_path.clone()),
                llama_swap_config: None,
                dry_run: false,
                no_color: false,
                prune: false,
                verbose: clap_verbosity_flag::Verbosity::new(0, 0),
            },
            true,
        )
        .await
        .unwrap();
        let output = read_to_string(opencode_path).unwrap();
        assert!(output.contains("acme/configured"));
        assert!(output.contains("acme/listed"));
    }
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
            model_file: None,
            force: false,
            opencode: false,
            llama_swap: false,
            models_dir: Some(PathBuf::from("cli-models")),
            opencode_config: None,
            llama_swap_config: Some(PathBuf::from("cli.yaml")),
            dry_run: false,
            no_color: false,
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
            model_file: None,
            force: false,
            opencode: false,
            llama_swap: false,
            models_dir: None,
            opencode_config: Some(PathBuf::from("cli.jsonc")),
            llama_swap_config: None,
            dry_run: false,
            no_color: false,
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
