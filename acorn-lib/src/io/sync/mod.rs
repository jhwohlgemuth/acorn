//! Configuration synchronization schemas
//!
//! Provides types for the `acorn sync` command, which resolves ACORN model
//! entries to downloaded GGUF files and merges the corresponding models into
//! llama-swap and OpenCode project configuration.
use crate::io::{config::FilterSet, files_all, home_directory, write_file, ApiResult};
use crate::prelude::{create_dir_all, remove_file, rename, PathBuf};
use crate::schema::agent::{self, ModelDetails};
use crate::util::constants::app::DEFAULT_MODELS_DIRECTORY;
use crate::util::Label;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use color_eyre::eyre::eyre;
use core::{fmt, iter::once};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::info;

pub mod llama_swap;
pub mod opencode;

#[derive(Debug)]
enum SyncError {
    LlamaSwap(color_eyre::Report),
    OpenCode(color_eyre::Report),
}
/// Top-level synchronization configuration section
///
/// Added to `ApplicationConfiguration` as the optional `config` field.
///
/// ### Example
///
/// ```yaml
/// config:
///   llamaSwap:
///     path: ./llama-swap.yaml
///     modelsDirectory: ~/.models
///     executable: llama-server
///     contextSize: 8192
///   opencode:
///     path: ./opencode.jsonc
///     baseUrl: http://localhost:8080/v1
///     providerId: llama-swap
/// ```
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// llama-swap configuration for local inference
    pub llama_swap: Option<llama_swap::Config>,
    /// OpenCode configuration for local inference
    pub opencode: Option<opencode::Config>,
}
/// Inputs controlling a configuration synchronization operation
#[derive(Clone, Copy, Debug, Default)]
pub struct Options<'a> {
    /// Resolved models to synchronize.
    pub models: &'a [ModelDetails],
    /// Synchronize OpenCode configuration.
    pub opencode: bool,
    /// Synchronize llama-swap configuration.
    pub llama_swap: bool,
    /// Preview configuration without writing files.
    pub dry_run: bool,
    /// Remove stale managed models.
    pub prune: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Shard {
    key: String,
    path: PathBuf,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Shards(Vec<Shard>);
impl Config {
    /// Merge two sync configs, preferring `other` values for overlapping fields
    pub fn merge(self, other: Self) -> Self {
        Self {
            llama_swap: other.llama_swap.or(self.llama_swap),
            opencode: other.opencode.or(self.opencode),
        }
    }
    /// Resolve the model directory from a CLI override, configuration, or the user default
    pub fn resolve_models_dir(&self, override_directory: Option<PathBuf>) -> ApiResult<PathBuf> {
        override_directory
            .or_else(|| {
                self.llama_swap
                    .as_ref()
                    .and_then(|config| config.models_directory.as_ref())
                    .map(PathBuf::from)
            })
            .map_or_else(|| home_directory(DEFAULT_MODELS_DIRECTORY), Ok)
    }
    /// Synchronize all selected configuration targets
    pub fn sync(&self, options: Options<'_>) -> ApiResult<()> {
        match options.models.is_empty() {
            | true => {
                info!("{} No models resolved — nothing to synchronize", Label::CAUTION);
                Ok(())
            }
            | false => {
                let sync_llama_swap = options.llama_swap || !options.opencode;
                let sync_opencode = options.opencode || !options.llama_swap;
                let llama_swap = sync_llama_swap
                    .then(|| self.sync_llama_swap(options).map_err(|why| eyre!(SyncError::LlamaSwap(why))))
                    .transpose();
                llama_swap.and_then(|llama_swap| {
                    sync_opencode
                        .then(|| self.sync_opencode(options).map_err(|why| eyre!(SyncError::OpenCode(why))))
                        .transpose()
                        .map(|opencode| {
                            let paths = [llama_swap, opencode].into_iter().flatten().collect::<Vec<_>>();
                            match options.dry_run {
                                | true => info!("{} Dry run complete — no files were modified", Label::pass()),
                                | false => paths.iter().for_each(|path| info!("{} Updated {}", Label::pass(), path.cyan())),
                            }
                        })
                })
            }
        }
    }
    /// Synchronize resolved models into llama-swap configuration
    pub fn sync_llama_swap(&self, options: Options<'_>) -> ApiResult<String> {
        self.llama_swap
            .as_ref()
            .ok_or_else(|| eyre!("llama-swap configuration is not set"))
            .and_then(|config| {
                config.resolve_path().and_then(|config_path| {
                    let models = options
                        .models
                        .iter()
                        .map(|model| {
                            let model_name = model.name.as_deref().or(model.id.as_deref()).unwrap_or("unknown");
                            (serde_norway::Value::String(model_name.to_string()), config.model_entry(model))
                        })
                        .collect();
                    let root = config
                        .models_directory
                        .as_ref()
                        .map(|directory| {
                            (
                                serde_norway::Value::String("modelsDir".to_string()),
                                serde_norway::Value::String(directory.clone()),
                            )
                        })
                        .into_iter()
                        .chain(once((
                            serde_norway::Value::String("models".to_string()),
                            serde_norway::Value::Mapping(models),
                        )))
                        .collect();
                    serde_norway::to_string(&serde_norway::Value::Mapping(root))
                        .map_err(|why| eyre!("Failed to serialize llama-swap config: {why}"))
                        .and_then(|content| Self::write_output("llama-swap", config_path, content, options.dry_run))
                })
            })
    }
    /// Synchronize resolved models into OpenCode configuration
    pub fn sync_opencode(&self, options: Options<'_>) -> ApiResult<String> {
        self.opencode
            .as_ref()
            .ok_or_else(|| eyre!("OpenCode configuration is not set"))
            .and_then(|config| {
                opencode::Config::resolve_path(config.path.as_deref()).and_then(|config_path| {
                    let models = options
                        .models
                        .iter()
                        .filter_map(|model| model.name.as_ref().or(model.id.as_ref()))
                        .map(|name| (name.clone(), name.clone()))
                        .collect::<Vec<_>>();
                    match options.prune {
                        | true => config.prune(agent::opencode::Config::load(PathBuf::from(&config_path)), &models),
                        | false => Ok(agent::opencode::Config::load(PathBuf::from(&config_path))),
                    }
                    .map(|existing| config.upsert(&existing, &models))
                    .map(|opencode| match config.default_model.as_ref() {
                        | Some(default_model) => opencode.with_model(format!("{}/{default_model}", config.provider_id)),
                        | None => opencode,
                    })
                    .and_then(|opencode| serde_json::to_string_pretty(&opencode).map_err(|why| eyre!("Failed to serialize OpenCode config: {why}")))
                    .and_then(|content| Self::write_output("OpenCode", config_path, content, options.dry_run))
                })
            })
    }
    /// Write synchronized configuration atomically or print it during a dry run
    pub fn write_output(target: &str, path: String, content: String, dry_run: bool) -> ApiResult<String> {
        match dry_run {
            | true => {
                info!("{} {target} configuration that would be written:", Label::run());
                println!("{content}");
                Ok(path)
            }
            | false => {
                let temp_path = PathBuf::from(format!("{path}.acorn-sync-tmp"));
                temp_path
                    .parent()
                    .map_or(Ok(()), create_dir_all)
                    .map_err(|why| eyre!("Failed to create parent directory for {target} config: {why}"))
                    .and_then(|()| write_file(&temp_path, content))
                    .map_err(|why| eyre!("Failed to write temporary {target} config: {why}"))
                    .and_then(|()| {
                        rename(&temp_path, &path).map_err(|why| {
                            let _ = remove_file(&temp_path);
                            eyre!("Failed to rename temporary {target} config: {why}")
                        })
                    })
                    .map(|()| path)
            }
        }
    }
}
impl From<PathBuf> for Shard {
    fn from(path: PathBuf) -> Self {
        let filename = path.file_name().and_then(|value| value.to_str()).unwrap_or("").to_string();
        Self {
            key: Self::key(&filename).unwrap_or(filename),
            path,
        }
    }
}
impl fmt::Display for SyncError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::LlamaSwap(why) => write!(formatter, "Synchronize llama-swap — {why}"),
            | Self::OpenCode(why) => write!(formatter, "Synchronize OpenCode — {why}"),
        }
    }
}
impl core::error::Error for SyncError {}
impl From<Vec<PathBuf>> for Shards {
    fn from(paths: Vec<PathBuf>) -> Self {
        Self(paths.into_iter().map(Shard::from).collect())
    }
}
impl Shard {
    fn key(filename: &str) -> Option<String> {
        filename
            .strip_suffix(".gguf")
            .and_then(|stem| stem.rsplit_once("-of-"))
            .filter(|(_, count)| !count.is_empty() && count.chars().all(|character| character.is_ascii_digit()))
            .and_then(|(before_count, count)| {
                before_count
                    .rsplit_once('-')
                    .filter(|(base, index)| !base.is_empty() && !index.is_empty() && index.chars().all(|character| character.is_ascii_digit()))
                    .map(|(base, _)| format!("{base}-of-{count}"))
            })
    }
}
impl Shards {
    fn into_paths(self) -> Vec<PathBuf> {
        self.0.into_iter().map(|shard| shard.path).collect()
    }
    fn unique(self) -> Self {
        Self(
            self.0
                .into_iter()
                .fold(BTreeMap::new(), |mut groups, shard| {
                    groups.entry(shard.key.clone()).or_insert(shard);
                    groups
                })
                .into_values()
                .collect(),
        )
    }
}
/// Discover GGUF files beneath a model's expected directory
///
/// Given a `models_directory` and a `model_name`, searches for
/// `<models_directory>/<model_name>/**/*.gguf` and returns the first match.
/// For sharded models, returns the first shard of a consistently named set.
pub fn resolve_gguf(models_directory: &str, model_name: &str, filter: Option<&[String]>, ignore: Option<&[String]>) -> ApiResult<Vec<ModelDetails>> {
    let model_dir = PathBuf::from(models_directory).join(model_name);
    match model_dir.is_dir() {
        | false => Err(eyre!("Model directory does not exist — {}", model_dir.display())),
        | true => {
            let gguf_files = files_all(model_dir.clone(), Some(vec!["gguf"]));
            match gguf_files.is_empty() {
                | true => Err(eyre!("No GGUF files found in model directory — {}", model_dir.display())),
                | false => {
                    let filter = FilterSet::filter(
                        gguf_files,
                        filter.unwrap_or(&[]),
                        ignore.unwrap_or(&[]),
                        |path| path.file_name().and_then(|value| value.to_str()).unwrap_or("").to_string(),
                        |_| true,
                    );
                    filter.and_then(|filtered| match filtered.is_empty() {
                        | true => Err(eyre!("All GGUF files for '{}' were excluded by filter/ignore patterns", model_name)),
                        | false => Ok(Shards::from(filtered)
                            .unique()
                            .into_paths()
                            .into_iter()
                            .map(|path| ModelDetails {
                                id: Some(model_name.to_string()),
                                name: Some(model_name.to_string()),
                                path: Some(path.display().to_string()),
                                ..Default::default()
                            })
                            .collect()),
                    })
                }
            }
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
    use crate::prelude::{create_dir_all, remove_dir_all, write};
    use crate::test::utils::temp_dir;

    #[test]
    fn test_shard_key_matches_pattern() {
        assert_eq!(Shard::key("qwen-00001-of-00003.gguf"), Some("qwen-of-00003".to_string()));
        assert_eq!(Shard::key("model-00003-of-00010.gguf"), Some("model-of-00010".to_string()));
    }
    #[test]
    fn test_shard_key_returns_none_for_non_shard() {
        assert_eq!(Shard::key("qwen-Q4_K_M.gguf"), None);
        assert_eq!(Shard::key("model.gguf"), None);
    }
    #[test]
    fn test_deduplicate_shards_keeps_first() {
        let dir = temp_dir("dedup-shards");
        create_dir_all(&dir).unwrap();
        let files = vec![
            dir.join("model-00001-of-00003.gguf"),
            dir.join("model-00002-of-00003.gguf"),
            dir.join("model-00003-of-00003.gguf"),
            dir.join("standalone.gguf"),
        ];
        let deduped = Shards::from(files).unique();
        assert_eq!(deduped.0.len(), 2);
        let _ = remove_dir_all(dir);
    }
    #[test]
    fn test_resolve_gguf_finds_files() {
        let dir = temp_dir("resolve-gguf");
        let model_dir = dir.join("test-model");
        create_dir_all(&model_dir).unwrap();
        write(model_dir.join("model.gguf"), b"fake").unwrap();
        let resolved = resolve_gguf(&dir.display().to_string(), "test-model", None, None);
        assert!(resolved.is_ok());
        let models = resolved.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name.as_deref(), Some("test-model"));
        let _ = remove_dir_all(dir);
    }
    #[test]
    fn test_resolve_gguf_fails_on_missing_dir() {
        let resolved = resolve_gguf("/nonexistent", "model", None, None);
        assert!(resolved.is_err());
    }
    #[test]
    fn test_resolve_gguf_fails_on_no_gguf_files() {
        let dir = temp_dir("resolve-gguf-empty");
        let model_dir = dir.join("empty-model");
        create_dir_all(&model_dir).unwrap();
        let resolved = resolve_gguf(&dir.display().to_string(), "empty-model", None, None);
        assert!(resolved.is_err());
        let _ = remove_dir_all(dir);
    }
    #[test]
    fn test_resolve_gguf_applies_filter_and_ignore_patterns() {
        let dir = temp_dir("resolve-gguf-filter");
        let model_dir = dir.join("test-model").join("nested");
        create_dir_all(&model_dir).unwrap();
        write(model_dir.join("alpha.gguf"), b"alpha").unwrap();
        write(model_dir.join("beta.gguf"), b"beta").unwrap();
        let filtered = resolve_gguf(&dir.display().to_string(), "test-model", Some(&["alpha".to_string()]), None).unwrap();
        let ignored = resolve_gguf(&dir.display().to_string(), "test-model", None, Some(&["alpha".to_string()])).unwrap();
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].path.as_deref().is_some_and(|path| path.ends_with("alpha.gguf")));
        assert_eq!(ignored.len(), 1);
        assert!(ignored[0].path.as_deref().is_some_and(|path| path.ends_with("beta.gguf")));
        let _ = remove_dir_all(dir);
    }
    #[test]
    fn test_write_output_creates_parent_directories() {
        let dir = temp_dir("sync-output");
        let path = dir.join("nested").join("config.yaml");
        let result = Config::write_output("test", path.display().to_string(), "models: {}".to_string(), false);
        assert!(result.is_ok());
        assert!(path.is_file());
        let _ = remove_dir_all(dir);
    }
}
