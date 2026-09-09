use crate::cli::{arguments, Void};
use acorn::io::config::{ApplicationConfiguration, ModelEntry};
use acorn::io::sync::{self, Config};
use acorn::io::ModelSelectorsExt;
use acorn::util::Label;
use acorn_schema::agent::ModelSelectors;
use color_eyre::eyre::eyre;
use std::env::current_exe;
use std::path::PathBuf;
use tracing::info;

impl From<&arguments::Sync> for sync::goose::Config {
    fn from(args: &arguments::Sync) -> Self {
        Self {
            path: args.goose_config.as_ref().map(|path| path.display().to_string()),
            ..Default::default()
        }
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
            mcp_path: args.vscode_mcp_config.as_ref().map(|path| path.display().to_string()),
            ..Default::default()
        }
    }
}
fn resolve_executable(enabled: bool) -> Result<Option<PathBuf>, color_eyre::Report> {
    enabled
        .then(current_exe)
        .transpose()
        .map_err(|why| eyre!("Failed to resolve ACORN executable — {why}"))
        .and_then(|executable| {
            executable
                .map(|path| path.canonicalize())
                .transpose()
                .map_err(|why| eyre!("Failed to canonicalize ACORN executable — {why}"))
        })
}
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
        mcp,
        ref models_dir,
        dry_run,
        no_color,
        prune,
        ..
    } = args;
    match mcp && llama_swap {
        | true => Err(eyre!("--mcp cannot be combined with --llama-swap")),
        | false => match ModelSelectors::default().resolve(model_file, offline).await {
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
                            resolve_executable(mcp).and_then(|executable| {
                                let options = sync::Options::init()
                                    .models(&resolved)
                                    .entries(&entries)
                                    .opencode(opencode)
                                    .llama_swap(llama_swap)
                                    .vscode(vscode)
                                    .goose(goose)
                                    .mcp(mcp)
                                    .maybe_executable(executable.as_deref())
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
                })
            }
            | Err(why) => Err(why),
        },
    }
}

#[cfg(test)]
mod tests;
