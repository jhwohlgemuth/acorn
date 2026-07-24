use crate::cli::{arguments, Void};
use acorn::io::config::ApplicationConfiguration;
use acorn::io::sync::{self, Config};
use acorn::util::Label;
use tracing::info;

/// Synchronize local inference configuration from ACORN model entries
pub async fn run(args: &arguments::Sync) -> Void {
    ApplicationConfiguration::load(&args.config).and_then(|configuration| {
        let llama_swap = sync::llama_swap::Config::from(args);
        let opencode = sync::opencode::Config::from(args);
        let overrides = Config {
            llama_swap: (llama_swap.path.is_some() || llama_swap.models_directory.is_some()).then_some(llama_swap),
            opencode: opencode.path.is_some().then_some(opencode),
        };
        let sync_config = configuration.resolve_sync_config(overrides);
        sync_config.resolve_models_dir(args.models_dir.clone()).and_then(|models_dir| {
            info!("{} Resolving models from configuration", Label::run());
            configuration.resolve_models(&models_dir).and_then(|resolved| {
                sync_config.sync(sync::Options {
                    models: &resolved,
                    opencode: args.opencode,
                    llama_swap: args.llama_swap,
                    dry_run: args.dry_run,
                    prune: args.prune,
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

    #[test]
    fn test_resolve_sync_config_merges_overrides() {
        let configuration = ApplicationConfiguration {
            config: Some(Config {
                llama_swap: Some(sync::llama_swap::Config {
                    executable: Some("my-server".to_string()),
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
            models_dir: None,
            opencode_config: None,
            llama_swap_config: None,
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
        let resolved = configuration.resolve_sync_config(overrides);
        let llama = resolved.llama_swap.unwrap();
        assert_eq!(llama.executable.as_deref(), Some("my-server"));
    }
    #[test]
    fn test_sync_config_merge_prefers_other() {
        let base = Config {
            llama_swap: Some(sync::llama_swap::Config {
                executable: Some("base-server".to_string()),
                ..Default::default()
            }),
            opencode: None,
        };
        let other = Config {
            llama_swap: Some(sync::llama_swap::Config {
                executable: Some("override-server".to_string()),
                ..Default::default()
            }),
            opencode: None,
        };
        let merged = base.merge(other);
        assert_eq!(merged.llama_swap.unwrap().executable.as_deref(), Some("override-server"));
    }
}
