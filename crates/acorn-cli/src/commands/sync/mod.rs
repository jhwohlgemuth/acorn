use crate::cli::{arguments, Void};
use acorn::io::config::{ApplicationConfiguration, ModelEntry};
use acorn::io::sync::{self, Config};
use acorn::prelude::PathBuf;
use acorn::schema::agent::ModelSelectors;
use acorn::util::Label;
use tracing::info;
/// Synchronize local inference configuration from ACORN model entries
pub async fn run(args: &arguments::Sync, database_path: &Option<PathBuf>, no_local_database: bool, offline: bool) -> Void {
    let &arguments::Sync {
        ref config,
        ref model_file,
        force,
        assume_models,
        no_fallback,
        opencode,
        llama_swap,
        vscode,
        goose,
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
                let vscode_config = sync::vscode::Config::from(args);
                let goose_config = sync::goose::Config::from(args);
                let overrides = Config {
                    goose: goose_config.path.is_some().then_some(goose_config),
                    llama_swap: (llama_swap_config.path.is_some() || llama_swap_config.models_directory.is_some()).then_some(llama_swap_config),
                    opencode: opencode_config.path.is_some().then_some(opencode_config),
                    vscode: vscode_config.path.is_some().then_some(vscode_config),
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
                    let options = sync::ModelRequestOptions {
                        models_dir: &models_dir,
                        assume_models,
                        fallbacks: Vec::new(),
                    };
                    let resolved = match no_local_database || no_fallback {
                        | true => ModelEntry::resolve(&entries, &options),
                        | false => ModelEntry::resolve_with_fallbacks(&entries, &options, database_path.clone()),
                    };
                    resolved.and_then(|resolved| {
                        let options = sync::Options::init()
                            .models(&resolved)
                            .entries(&entries)
                            .opencode(opencode)
                            .llama_swap(llama_swap)
                            .vscode(vscode)
                            .goose(goose)
                            .dry_run(dry_run)
                            .no_color(no_color)
                            .prune(prune)
                            .force(force)
                            .assume_models(assume_models)
                            .models_dir(&models_dir)
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
impl From<&arguments::Sync> for sync::vscode::Config {
    fn from(args: &arguments::Sync) -> Self {
        Self {
            path: args.vscode_config.as_ref().map(|path| path.display().to_string()),
            ..Default::default()
        }
    }
}
impl From<&arguments::Sync> for sync::goose::Config {
    fn from(args: &arguments::Sync) -> Self {
        Self {
            path: args.goose_config.as_ref().map(|path| path.display().to_string()),
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
    use acorn::io::database::schema::{ModelRow, Table};
    use acorn::io::database::{Database, Operations};
    use acorn::prelude::{create_dir_all, read_to_string, write};

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
        let options = sync::ModelRequestOptions {
            models_dir: &models_dir,
            assume_models: false,
            fallbacks: Vec::new(),
        };
        let resolved = ModelEntry::resolve(&entries, &options).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].id.as_deref(), Some("mozilla/test-llama"));
    }
    #[test]
    fn test_resolve_models_uses_downloaded_fallback_repository() {
        let root = temp_test_dir("sync_fallback_repository");
        let _cleanup = TestCleanup::new(root.clone());
        let models_dir = root.join("models");
        let fallback_dir = models_dir.join("unsloth").join("gpt-oss-20b-GGUF");
        let database_path = root.join("acorn.db");
        create_dir_all(&fallback_dir).unwrap();
        write(fallback_dir.join("gpt-oss-20b-Q4_K_M.gguf"), b"model").unwrap();
        let database = Database::<Table>::from_path(Some(database_path.clone()));
        database.migrate().unwrap();
        database
            .insert(
                ModelRow::init()
                    .model_id("openai/gpt-oss-20b")
                    .weights(
                        r#"[{"label":"Q4_K_M","url":"https://huggingface.co/unsloth/gpt-oss-20b-GGUF/resolve/main/gpt-oss-20b-Q4_K_M.gguf","quantization":"Q4_K_M"}]"#,
                    )
                    .build(),
            )
            .unwrap();
        let entries = [ModelEntry::Selector("openai/gpt-oss-20b".to_string())];
        let options = sync::ModelRequestOptions {
            models_dir: &models_dir,
            assume_models: false,
            fallbacks: Vec::new(),
        };
        let resolved = ModelEntry::resolve_with_fallbacks(&entries, &options, Some(database_path)).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].id.as_deref(), Some("openai/gpt-oss-20b"));
        assert!(resolved[0].path.as_deref().is_some_and(|path| path.ends_with("gpt-oss-20b-Q4_K_M.gguf")));
    }
    #[tokio::test]
    async fn test_sync_no_fallback_skips_downloaded_fallback_repository() {
        let root = temp_test_dir("sync_no_fallback_repository");
        let _cleanup = TestCleanup::new(root.clone());
        let models_dir = root.join("models");
        let fallback_dir = models_dir.join("unsloth").join("gpt-oss-20b-GGUF");
        let database_path = root.join("acorn.db");
        let config_path = root.join("acorn.json");
        let opencode_path = root.join("opencode.jsonc");
        create_dir_all(&fallback_dir).unwrap();
        write(fallback_dir.join("gpt-oss-20b-Q4_K_M.gguf"), b"model").unwrap();
        write(&config_path, br#"{"models":["openai/gpt-oss-20b"]}"#).unwrap();
        let database = Database::<Table>::from_path(Some(database_path.clone()));
        database.migrate().unwrap();
        database
            .insert(
                ModelRow::init()
                    .model_id("openai/gpt-oss-20b")
                    .weights(
                        r#"[{"label":"Q4_K_M","url":"https://huggingface.co/unsloth/gpt-oss-20b-GGUF/resolve/main/gpt-oss-20b-Q4_K_M.gguf","quantization":"Q4_K_M"}]"#,
                    )
                    .build(),
            )
            .unwrap();
        let args = arguments::Sync {
            config: Some(config_path),
            model_file: None,
            force: true,
            assume_models: false,
            no_fallback: true,
            opencode: true,
            llama_swap: false,
            vscode: false,
            goose: false,
            models_dir: Some(models_dir),
            opencode_config: Some(opencode_path.clone()),
            llama_swap_config: None,
            vscode_config: None,
            goose_config: None,
            dry_run: false,
            no_color: false,
            prune: false,
            verbose: clap_verbosity_flag::Verbosity::new(0, 0),
        };
        run(&args, &Some(database_path.clone()), false, true).await.unwrap();
        assert!(!opencode_path.exists());
        run(&arguments::Sync { no_fallback: false, ..args }, &Some(database_path), false, true)
            .await
            .unwrap();
        assert!(read_to_string(opencode_path).unwrap().contains("openai/gpt-oss-20b"));
    }
    #[test]
    fn test_assume_models_uses_model_path_from_identifier() {
        let models_dir = PathBuf::from("models");
        let entries = [ModelEntry::Selector("acme/missing".to_string())];
        let options = sync::ModelRequestOptions {
            models_dir: &models_dir,
            assume_models: true,
            fallbacks: Vec::new(),
        };
        let resolved = ModelEntry::resolve(&entries, &options).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].id.as_deref(), Some("acme/missing"));
        assert_eq!(
            resolved[0].path.as_deref(),
            Some(models_dir.join("acme/missing").display().to_string().as_str())
        );
    }
    #[tokio::test]
    async fn test_force_and_assume_models_are_independent() {
        let root = temp_test_dir("sync_force_assume_models");
        let _cleanup = TestCleanup::new(root.clone());
        let config_path = root.join("acorn.json");
        let opencode_path = root.join("opencode.json");
        let models_dir = root.join("models");
        write(&config_path, br#"{"models":["acme/missing"]}"#).unwrap();
        let args = arguments::Sync {
            config: Some(config_path),
            model_file: None,
            force: true,
            assume_models: false,
            no_fallback: true,
            opencode: true,
            llama_swap: false,
            vscode: false,
            goose: false,
            models_dir: Some(models_dir),
            opencode_config: Some(opencode_path.clone()),
            llama_swap_config: None,
            vscode_config: None,
            goose_config: None,
            dry_run: false,
            no_color: false,
            prune: false,
            verbose: clap_verbosity_flag::Verbosity::new(0, 0),
        };
        run(&args, &None, true, true).await.unwrap();
        assert!(!opencode_path.exists());
        run(&arguments::Sync { assume_models: true, ..args }, &None, true, true).await.unwrap();
        assert!(read_to_string(opencode_path).unwrap().contains("acme/missing"));
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
                ..Default::default()
            }),
            ..Default::default()
        };
        configuration
            .sync(sync::Options {
                entries: &[ModelEntry::Selector("acme/model".to_string())],
                force: true,
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
                force: true,
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
                force: true,
                assume_models: true,
                no_fallback: false,
                opencode: true,
                llama_swap: false,
                vscode: false,
                goose: false,
                models_dir: Some(models_dir),
                opencode_config: Some(opencode_path.clone()),
                llama_swap_config: None,
                vscode_config: None,
                goose_config: None,
                dry_run: false,
                no_color: false,
                prune: false,
                verbose: clap_verbosity_flag::Verbosity::new(0, 0),
            },
            &None,
            true,
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let args = arguments::Sync {
            config: None,
            model_file: None,
            force: false,
            assume_models: false,
            no_fallback: false,
            opencode: false,
            llama_swap: false,
            vscode: false,
            goose: false,
            models_dir: Some(PathBuf::from("cli-models")),
            opencode_config: None,
            llama_swap_config: Some(PathBuf::from("cli.yaml")),
            vscode_config: None,
            goose_config: None,
            dry_run: false,
            no_color: false,
            prune: false,
            verbose: clap_verbosity_flag::Verbosity::new(0, 0),
        };
        let llama_swap = sync::llama_swap::Config::from(&args);
        let opencode = sync::opencode::Config::from(&args);
        let overrides = Config {
            goose: None,
            llama_swap: (llama_swap.path.is_some() || llama_swap.models_directory.is_some()).then_some(llama_swap),
            opencode: opencode.path.is_some().then_some(opencode),
            vscode: None,
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let args = arguments::Sync {
            config: None,
            model_file: None,
            force: false,
            assume_models: false,
            no_fallback: false,
            opencode: false,
            llama_swap: false,
            vscode: false,
            goose: false,
            models_dir: None,
            opencode_config: Some(PathBuf::from("cli.jsonc")),
            llama_swap_config: None,
            vscode_config: None,
            goose_config: None,
            dry_run: false,
            no_color: false,
            prune: false,
            verbose: clap_verbosity_flag::Verbosity::new(0, 0),
        };
        let opencode = sync::opencode::Config::from(&args);
        let resolved = configuration.config.clone().unwrap_or_default().merge_cli_overrides(Config {
            goose: None,
            llama_swap: None,
            opencode: Some(opencode),
            vscode: None,
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
            goose: None,
            llama_swap: Some(sync::llama_swap::Config {
                path: Some("cli.yaml".to_string()),
                ..Default::default()
            }),
            opencode: None,
            vscode: None,
        });
        assert_eq!(merged.llama_swap.unwrap().path.as_deref(), Some("cli.yaml"));
    }
    #[test]
    fn test_sync_config_merge_prefers_non_path_overrides() {
        let base = Config {
            goose: None,
            llama_swap: Some(sync::llama_swap::Config {
                executable: Some("base-server".to_string()),
                ..Default::default()
            }),
            opencode: Some(sync::opencode::Config {
                base_url: "http://base.test/v1".to_string(),
                provider_id: "base-provider".to_string(),
                ..Default::default()
            }),
            vscode: None,
        };
        let overrides = Config {
            goose: None,
            llama_swap: Some(sync::llama_swap::Config {
                executable: Some("override-server".to_string()),
                ..Default::default()
            }),
            opencode: Some(sync::opencode::Config {
                base_url: "http://override.test/v1".to_string(),
                provider_id: "override-provider".to_string(),
                ..Default::default()
            }),
            vscode: None,
        };
        let merged = base.merge(overrides);
        let llama_swap = merged.llama_swap.unwrap();
        let opencode = merged.opencode.unwrap();
        assert_eq!(llama_swap.executable.as_deref(), Some("override-server"));
        assert_eq!(opencode.base_url, "http://override.test/v1");
        assert_eq!(opencode.provider_id, "override-provider");
    }
}
