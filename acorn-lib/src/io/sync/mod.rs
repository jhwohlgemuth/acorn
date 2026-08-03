//! Configuration synchronization schemas
//!
//! Provides types for the `acorn sync` command, which resolves ACORN model
//! entries to downloaded GGUF files and merges the corresponding models into
//! llama-swap and OpenCode project configuration.
use crate::io::{
    config::{FilterSet, ModelEntry},
    files_all, home_directory, read_file, write_file, ApiResult, Source,
};
use crate::prelude::{canonicalize, create_dir_all, remove_file, rename, Path, PathBuf};
use crate::schema::agent::{self, ModelDetails};
use crate::util::constants::app::DEFAULT_MODELS_DIRECTORY;
use crate::util::{text_diff_changes_with_color, Label};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use bon::Builder;
use color_eyre::eyre::eyre;
use core::fmt;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_norway::Value;
use serde_with::skip_serializing_none;
use tracing::info;
use validator::Validate;

pub mod llama_swap;
pub mod opencode;

#[derive(Debug)]
enum SyncError {
    LlamaSwap(color_eyre::Report),
    OpenCode(color_eyre::Report),
}
#[derive(Debug)]
enum ShardError {
    Ambiguous { model: String, paths: Vec<String> },
    Canonicalize { model: String, why: color_eyre::Report },
    Malformed(Shard),
    EmptyCandidate,
    EmptyGroup,
    Incomplete { key: String, count: u64 },
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
#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// llama-swap configuration for local inference
    #[validate(nested)]
    pub llama_swap: Option<llama_swap::Config>,
    /// OpenCode configuration for local inference
    #[validate(nested)]
    pub opencode: Option<opencode::Config>,
}
/// Inputs controlling a configuration synchronization operation
#[derive(Builder, Clone, Copy, Debug, Default)]
#[builder(start_fn = init)]
pub struct Options<'a> {
    /// Resolved models to synchronize.
    #[builder(default)]
    pub models: &'a [ModelDetails],
    /// Model configuration entries to resolve before synchronization.
    #[builder(default)]
    pub entries: &'a [ModelEntry],
    /// Synchronize OpenCode configuration.
    #[builder(default)]
    pub opencode: bool,
    /// Synchronize llama-swap configuration.
    #[builder(default)]
    pub llama_swap: bool,
    /// Preview configuration without writing files.
    #[builder(default)]
    pub dry_run: bool,
    /// Disable color in dry-run output.
    #[builder(default)]
    pub no_color: bool,
    /// Remove stale managed models.
    #[builder(default)]
    pub prune: bool,
    /// Assume model paths from their identifiers without checking the filesystem.
    #[builder(default)]
    pub force: bool,
    /// Models directory override used to resolve model entries.
    pub models_dir: Option<&'a Path>,
}
/// A normalized model entry ready for local GGUF resolution.
#[derive(Clone, Debug)]
pub struct ModelRequest {
    id: String,
    source: Source,
    filter: Vec<String>,
    ignore: Vec<String>,
}
#[derive(Clone, Debug)]
struct RenderedOutput {
    target: &'static str,
    path: PathBuf,
    before: String,
    content: String,
}
/// A parsed GGUF file candidate used to validate and order split model shards.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Shard {
    key: String,
    index: Option<u64>,
    count: Option<u64>,
    malformed: bool,
    path: PathBuf,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Shards(Vec<Shard>);
impl Config {
    /// Merge populated runtime overrides into configured synchronization targets
    pub fn merge(self, overrides: Self) -> Self {
        Self {
            llama_swap: match (self.llama_swap, overrides.llama_swap) {
                | (Some(config), Some(overrides)) => Some(config.merge(overrides)),
                | (config, overrides) => overrides.or(config),
            },
            opencode: match (self.opencode, overrides.opencode) {
                | (Some(config), Some(overrides)) => Some(config.merge(overrides)),
                | (config, overrides) => overrides.or(config),
            },
        }
    }
    /// Merge CLI-supported overrides into configured synchronization targets
    pub fn merge_cli_overrides(self, overrides: Self) -> Self {
        Self {
            llama_swap: match (self.llama_swap, overrides.llama_swap) {
                | (Some(config), Some(overrides)) => Some(config.merge_cli_overrides(overrides)),
                | (config, overrides) => overrides.or(config),
            },
            opencode: match (self.opencode, overrides.opencode) {
                | (Some(config), Some(overrides)) => Some(config.merge_cli_overrides(overrides)),
                | (config, overrides) => overrides.or(config),
            },
        }
    }
    /// Resolve the model directory from a CLI override, configuration, or the user default
    pub fn resolve_models_dir(&self, override_directory: Option<&Path>) -> ApiResult<PathBuf> {
        override_directory
            .map(PathBuf::from)
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
        let selected = self.selected(&options);
        let model_ids = options.models.iter().filter_map(|model| model.id.as_ref()).cloned().collect::<Vec<_>>();
        Validate::validate(&selected)
            .map_err(|why| eyre!("Invalid synchronization configuration: {why}"))
            .and_then(|()| {
                selected.llama_swap.as_ref().map_or(Ok(()), |config| {
                    Validate::validate(&llama_swap::ModelValidation::from((config, model_ids.as_slice())))
                        .map_err(|why| eyre!("Invalid llama-swap model configuration: {why}"))
                })
            })
            .and_then(|()| {
                selected
                    .opencode
                    .as_ref()
                    .and_then(|config| config.default_model.as_ref())
                    .map_or(Ok(()), |default_model| match model_ids.iter().any(|model_id| model_id == default_model) {
                        | true => Ok(()),
                        | false => Err(eyre!("opencode.defaultModel references unknown model '{default_model}'")),
                    })
            })
            .and_then(|()| match options.models.is_empty() {
                | true => {
                    info!("{} No models resolved — nothing to synchronize", Label::CAUTION);
                    Ok(())
                }
                | false => selected
                    .llama_swap
                    .is_some()
                    .then(|| selected.render_llama_swap(options).map_err(|why| eyre!(SyncError::LlamaSwap(why))))
                    .transpose()
                    .and_then(|llama_swap_output| {
                        selected
                            .opencode
                            .is_some()
                            .then(|| selected.render_opencode(options).map_err(|why| eyre!(SyncError::OpenCode(why))))
                            .transpose()
                            .and_then(|opencode_output| {
                                let outputs = [llama_swap_output, opencode_output].into_iter().flatten().collect::<Vec<_>>();
                                match options.dry_run {
                                    | true => {
                                        outputs.iter().for_each(|output| {
                                            let path = output.path.display().to_string();
                                            match (output.before == output.content, options.no_color) {
                                                | (true, true) => info!("=> {} No changes for {path}", Label::CAUTION),
                                                | (true, false) => info!("=> {} No changes for {}", Label::CAUTION, path.cyan()),
                                                | (false, _) => {
                                                    match options.no_color {
                                                        | true => println!("\n{path}"),
                                                        | false => println!("\n{}", path.cyan().bold()),
                                                    }
                                                    text_diff_changes_with_color(&output.before, &output.content, !options.no_color)
                                                        .iter()
                                                        .for_each(|(_, line)| print!("{line}"));
                                                }
                                            }
                                        });
                                        info!("=> {} Dry run complete — no files were modified", Label::pass());
                                        Ok(Vec::new())
                                    }
                                    | false => Self::commit(&outputs),
                                }
                            })
                            .map(|paths| {
                                paths
                                    .iter()
                                    .for_each(|path| info!("=> {} Updated {}", Label::pass(), path.display().cyan()));
                            })
                    }),
            })
    }
    fn selected(&self, options: &Options<'_>) -> Self {
        let sync_llama_swap = options.llama_swap || !options.opencode;
        let sync_opencode = options.opencode || !options.llama_swap;
        Self {
            llama_swap: sync_llama_swap.then(|| self.llama_swap.clone().unwrap_or_default()),
            opencode: sync_opencode.then(|| self.opencode.clone().unwrap_or_default()),
        }
    }
    fn commit(outputs: &[RenderedOutput]) -> ApiResult<Vec<PathBuf>> {
        let changed = outputs.iter().filter(|output| output.before != output.content).collect::<Vec<_>>();
        let paths = changed.iter().map(|output| output.path.clone()).collect::<Vec<_>>();
        let unique_paths = paths.iter().collect::<BTreeSet<_>>();
        match unique_paths.len() == paths.len() {
            | true => Ok(()),
            | false => Err(eyre!("Selected synchronization targets resolve to the same output path")),
        }
        .and_then(|()| {
            changed
                .iter()
                .try_for_each(|output| match (output.temp_path().exists(), output.backup_path().exists()) {
                    | (true, _) => Err(eyre!("Temporary sync path already exists — {}", output.temp_path().display())),
                    | (_, true) => Err(eyre!("Backup sync path already exists — {}", output.backup_path().display())),
                    | _ => Ok(()),
                })
        })
        .and_then(|()| {
            changed
                .iter()
                .try_for_each(|output| {
                    output
                        .path
                        .parent()
                        .map_or(Ok(()), create_dir_all)
                        .map_err(|why| eyre!("Failed to create parent directory for {} config — {why}", output.target))
                        .and_then(|()| write_file(output.temp_path(), output.content.clone()))
                        .map_err(|why| eyre!("Failed to stage {} config — {why}", output.target))
                })
                .inspect_err(|_why| {
                    changed.iter().for_each(|output| output.cleanup_temp());
                })
        })
        .and_then(|()| {
            changed
                .iter()
                .filter(|output| output.path.is_file())
                .try_fold(Vec::new(), |mut backed_up, output| match rename(&output.path, output.backup_path()) {
                    | Ok(()) => {
                        backed_up.push(*output);
                        Ok(backed_up)
                    }
                    | Err(why) => {
                        backed_up.iter().for_each(|output: &&RenderedOutput| output.restore_backup());
                        changed.iter().for_each(|output| output.cleanup_temp());
                        Err(eyre!("Failed to prepare coordinated configuration update: {why}"))
                    }
                })
                .and_then(|backed_up| {
                    changed
                        .iter()
                        .try_fold(Vec::new(), |mut committed, output| match rename(output.temp_path(), &output.path) {
                            | Ok(()) => {
                                committed.push(*output);
                                Ok(committed)
                            }
                            | Err(why) => {
                                committed.iter().for_each(|output: &&RenderedOutput| output.cleanup_target());
                                backed_up.iter().for_each(|output| output.restore_backup());
                                changed.iter().for_each(|output| output.cleanup_temp());
                                Err(eyre!("Failed to commit coordinated configuration update: {why}"))
                            }
                        })
                        .map(|_| {
                            backed_up.iter().for_each(|output| {
                                remove_file(output.backup_path()).ok();
                            });
                        })
                })
        })
        .map(|()| paths)
    }
    fn render_llama_swap(&self, options: Options<'_>) -> ApiResult<RenderedOutput> {
        self.llama_swap
            .as_ref()
            .ok_or_else(|| eyre!("llama-swap configuration is not set"))
            .and_then(|config| {
                config.resolve_path().and_then(|config_path| {
                    let path = PathBuf::from(config_path);
                    path.is_file()
                        .then(|| read_file(path.clone()))
                        .transpose()
                        .map(|content| content.unwrap_or_default())
                        .and_then(|before| {
                            let existing = match before.is_empty() {
                                | true => Ok(Value::Mapping(Default::default())),
                                | false => serde_norway::from_str(&before).map_err(|why| eyre!("Failed to parse existing llama-swap config: {why}")),
                            };
                            existing
                                .and_then(|existing| config.upsert(existing, options.models, options.prune))
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
            })
    }
    fn render_opencode(&self, options: Options<'_>) -> ApiResult<RenderedOutput> {
        self.opencode
            .as_ref()
            .ok_or_else(|| eyre!("OpenCode configuration is not set"))
            .and_then(|config| {
                opencode::Config::resolve_path(config.path.as_deref()).and_then(|config_path| {
                    let path = PathBuf::from(config_path);
                    let models = options
                        .models
                        .iter()
                        .filter_map(|model| model.name.as_ref().or(model.id.as_ref()))
                        .map(|name| (name.clone(), name.clone()))
                        .collect::<Vec<_>>();
                    path.is_file()
                        .then(|| read_file(path.clone()))
                        .transpose()
                        .map(|content| content.unwrap_or_default())
                        .and_then(|before| {
                            match path.is_file() {
                                | true => agent::opencode::Config::from_path(&path),
                                | false => Ok(agent::opencode::Config::default()),
                            }
                            .and_then(|existing| config.upsert(&existing, &models, options.prune))
                            .and_then(|opencode| config.render(&opencode))
                            .map(|content| RenderedOutput {
                                target: "OpenCode",
                                path,
                                before,
                                content,
                            })
                        })
                })
            })
    }
}
impl RenderedOutput {
    fn backup_path(&self) -> PathBuf {
        PathBuf::from(format!("{}.acorn-sync-backup", self.path.display()))
    }
    fn cleanup_target(&self) {
        remove_file(&self.path).ok();
    }
    fn cleanup_temp(&self) {
        remove_file(self.temp_path()).ok();
    }
    fn restore_backup(&self) {
        self.cleanup_target();
        rename(self.backup_path(), &self.path).ok();
    }
    fn temp_path(&self) -> PathBuf {
        PathBuf::from(format!("{}.acorn-sync-tmp", self.path.display()))
    }
}
impl ModelRequest {
    /// Build model details from the configured source or expected models-directory path without checking the filesystem.
    pub fn assume(self, models_dir: &Path) -> ModelDetails {
        let path = match self.source {
            | Source::Local { path, .. } => path,
            | _ => models_dir.join(&self.id),
        };
        ModelDetails::init()
            .id(self.id.clone())
            .name(self.id)
            .path(path.display().to_string())
            .build()
    }
    fn details(self, path: PathBuf) -> ApiResult<ModelDetails> {
        canonicalize(&path)
            .map_err(|why| {
                eyre!(ShardError::Canonicalize {
                    model: self.id.clone(),
                    why: why.into(),
                })
            })
            .map(|path| ModelDetails {
                id: Some(self.id.clone()),
                name: Some(self.id),
                path: Some(path.display().to_string()),
                ..Default::default()
            })
    }
    /// Return the configured model identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
    fn is_gguf(path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("gguf"))
    }
    fn new(id: String, source: Source, filter: Vec<String>, ignore: Vec<String>) -> ApiResult<Self> {
        match id.trim() {
            | "" => Err(eyre!("Configured model ID cannot be empty")),
            | id => Ok(Self {
                id: id.to_string(),
                source,
                filter,
                ignore,
            }),
        }
    }
    /// Resolve this request to one local GGUF model.
    pub fn resolve(self, models_dir: &Path) -> ApiResult<ModelDetails> {
        let direct_path = match &self.source {
            | Source::Local { path, .. } if path.is_file() || Self::is_gguf(path) => Some(path.clone()),
            | _ => None,
        };
        match direct_path {
            | Some(path) => self.resolve_direct(&path),
            | None => {
                resolve_gguf(&models_dir.display().to_string(), &self.id, Some(&self.filter), Some(&self.ignore)).and_then(|models| {
                    match models.as_slice() {
                        | [model] => model
                            .path
                            .as_ref()
                            .map(PathBuf::from)
                            .ok_or_else(|| eyre!("Resolved model '{}' has no GGUF path", self.id))
                            .and_then(|path| self.details(path)),
                        | [] => Err(eyre!("No GGUF candidates resolved for model '{}'", self.id)),
                        | _ => Err(eyre!(ShardError::Ambiguous {
                            model: self.id.clone(),
                            paths: models.iter().filter_map(|model| model.path.clone()).collect(),
                        })),
                    }
                })
            }
        }
    }
    fn resolve_direct(self, path: &Path) -> ApiResult<ModelDetails> {
        match (path.is_file(), Self::is_gguf(path)) {
            | (false, _) => Err(eyre!("Direct local GGUF source for '{}' does not exist — {}", self.id, path.display())),
            | (_, true) => FilterSet::filter(
                vec![path.to_path_buf()],
                &self.filter,
                &self.ignore,
                |path| path.file_name().and_then(|value| value.to_str()).unwrap_or("").to_string(),
                |_| true,
            )
            .and_then(|paths| match paths.as_slice() {
                | [path] => self.details(path.clone()),
                | _ => Err(eyre!("Direct GGUF source for '{}' was excluded by filter/ignore patterns", self.id)),
            }),
            | _ => Err(eyre!(
                "Direct local model source for '{}' is not a GGUF file — {}",
                self.id,
                path.display()
            )),
        }
    }
}
impl TryFrom<&ModelEntry> for ModelRequest {
    type Error = color_eyre::Report;
    fn try_from(entry: &ModelEntry) -> Result<Self, Self::Error> {
        match entry {
            | ModelEntry::Selector(selector) => {
                let selector = selector.trim();
                let source = Source::from(selector);
                let id = match &source {
                    | Source::Local { path, .. } if path.is_file() || Self::is_gguf(path) => source.name(),
                    | _ => selector.to_string(),
                };
                Self::new(id, source, Vec::new(), Vec::new())
            }
            | ModelEntry::Entry(options) => Self::new(
                options.name.clone(),
                Source::from(&options.source).with_name(options.name.as_str()),
                options.filter.clone().unwrap_or_default(),
                options.ignore.clone().unwrap_or_default(),
            ),
        }
    }
}
impl From<PathBuf> for Shard {
    fn from(path: PathBuf) -> Self {
        let filename = path.file_name().and_then(|value| value.to_str()).unwrap_or("").to_string();
        let parts = Self::parts(&filename);
        let malformed = Self::is_sharded(&filename) && parts.is_none();
        Self {
            key: parts.as_ref().map_or_else(|| filename.clone(), |(key, _, _)| key.clone()),
            index: parts.as_ref().map(|(_, index, _)| *index),
            count: parts.map(|(_, _, count)| count),
            malformed,
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
impl fmt::Display for Shard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.path.display().fmt(formatter)
    }
}
impl fmt::Display for ShardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::Ambiguous { model, paths } => {
                write!(
                    formatter,
                    "Multiple independent GGUF candidates resolved for model '{model}': {}",
                    paths.join(", ")
                )
            }
            | Self::Canonicalize { model, why } => {
                write!(formatter, "Failed to resolve absolute GGUF path for '{model}': {why}")
            }
            | Self::Malformed(shard) => write!(formatter, "Malformed GGUF shard filename '{shard}'"),
            | Self::EmptyCandidate => formatter.write_str("GGUF candidate group is empty"),
            | Self::EmptyGroup => formatter.write_str("GGUF shard group is empty"),
            | Self::Incomplete { key, count } => {
                write!(
                    formatter,
                    "Incomplete or inconsistent GGUF shard set '{key}' — expected shards 1 through {count}"
                )
            }
        }
    }
}
impl core::error::Error for ShardError {}
impl From<Vec<PathBuf>> for Shards {
    fn from(paths: Vec<PathBuf>) -> Self {
        Self(paths.into_iter().map(Shard::from).collect())
    }
}
impl Shard {
    fn is_sharded(filename: &str) -> bool {
        filename
            .strip_suffix(".gguf")
            .and_then(|stem| stem.rsplit_once("-of-"))
            .and_then(|(before_count, count)| before_count.rsplit_once('-').map(|(base, index)| (base, index, count)))
            .is_some_and(|(base, index, count)| {
                !base.is_empty()
                    && !index.is_empty()
                    && index.chars().all(|c| c.is_ascii_digit())
                    && !count.is_empty()
                    && count.chars().all(|c| c.is_ascii_digit())
            })
    }
    pub(crate) fn parts(filename: &str) -> Option<(String, u64, u64)> {
        filename
            .strip_suffix(".gguf")
            .and_then(|stem| stem.rsplit_once("-of-"))
            .filter(|(_, count)| !count.is_empty() && count.chars().all(|character| character.is_ascii_digit()))
            .and_then(|(before_count, count)| {
                let count_label = count.to_string();
                before_count
                    .rsplit_once('-')
                    .filter(|(base, index)| !base.is_empty() && !index.is_empty() && index.chars().all(|character| character.is_ascii_digit()))
                    .and_then(|(base, index)| {
                        index
                            .parse::<u64>()
                            .ok()
                            .zip(count.parse::<u64>().ok())
                            .filter(|(index, count)| *index > 0 && *count > 0 && index <= count)
                            .map(|(index, count)| (format!("{base}-of-{count_label}"), index, count))
                    })
            })
    }
}
impl Shards {
    fn candidates(self) -> ApiResult<Vec<PathBuf>> {
        match self.0.iter().find(|shard| shard.malformed) {
            | Some(_) => self.0.into_iter().find(|shard| shard.malformed).map_or_else(
                || Err(eyre!(ShardError::EmptyCandidate)),
                |shard| Err(eyre!(ShardError::Malformed(shard))),
            ),
            | None => self
                .0
                .into_iter()
                .fold(BTreeMap::<String, Self>::new(), |mut groups, shard| {
                    groups.entry(shard.key.clone()).or_default().0.push(shard);
                    groups
                })
                .into_values()
                .map(|group| {
                    let expected_count = group.0.iter().find_map(|shard| shard.count);
                    match expected_count {
                        | None => group
                            .0
                            .into_iter()
                            .map(|shard| shard.path)
                            .min()
                            .ok_or_else(|| eyre!(ShardError::EmptyCandidate)),
                        | Some(count) => {
                            let indexes = group.0.iter().filter_map(|shard| shard.index).collect::<BTreeSet<_>>();
                            let expected = (1..=count).collect::<BTreeSet<_>>();
                            match indexes == expected && usize::try_from(count).ok() == Some(group.0.len()) {
                                | true => group
                                    .0
                                    .into_iter()
                                    .min_by_key(|shard| shard.index)
                                    .map(|shard| shard.path)
                                    .ok_or_else(|| eyre!(ShardError::EmptyGroup)),
                                | false => Err(eyre!(ShardError::Incomplete {
                                    key: group.0.first().map_or_else(|| "unknown".to_string(), |shard| shard.key.clone()),
                                    count,
                                })),
                            }
                        }
                    }
                })
                .collect(),
        }
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
                        | false => Shards::from(filtered).candidates().map(|paths| {
                            paths
                                .into_iter()
                                .map(|path| {
                                    ModelDetails::init()
                                        .id(model_name)
                                        .name(model_name)
                                        .path(path.display().to_string())
                                        .build()
                                })
                                .collect()
                        }),
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
    use crate::io::config::{ApplicationConfiguration, ModelEntry, ModelEntryOptions};
    use crate::prelude::{create_dir_all, read_to_string, remove_dir_all, write};
    use crate::test::utils::temp_dir;
    use crate::{Location, Repository};

    fn detailed_model(name: &str, filter: Option<Vec<String>>) -> ModelEntry {
        ModelEntry::Entry(ModelEntryOptions {
            name: name.to_string(),
            source: Repository::HuggingFace {
                location: Location::Simple(format!("https://huggingface.co/example/{name}")),
            },
            revision: None,
            auth: None,
            filter,
            ignore: None,
            quantization: None,
            gpu_memory: None,
            copy: None,
            symlink: None,
        })
    }
    fn resolve_models(configuration: &ApplicationConfiguration, models_dir: &Path) -> ApiResult<Vec<ModelDetails>> {
        ModelEntry::requests(configuration.models.as_deref().unwrap_or_default())
            .and_then(|requests| requests.into_iter().map(|request| request.resolve(models_dir)).collect())
    }

    #[test]
    fn test_application_config_deserializes_sync_paths_without_serializing_them() {
        let configuration = ApplicationConfiguration::parse(
            r#"{
                "config": {
                    "llamaSwap": {"path": "./llama-swap.yaml"},
                    "opencode": {"path": "./opencode.jsonc"}
                }
            }"#,
        )
        .unwrap();
        let config = configuration.config.unwrap();
        assert_eq!(
            config.llama_swap.as_ref().and_then(|value| value.path.as_deref()),
            Some("./llama-swap.yaml")
        );
        assert_eq!(config.opencode.as_ref().and_then(|value| value.path.as_deref()), Some("./opencode.jsonc"));
        let serialized = serde_json::to_value(config).unwrap();
        assert!(serialized["llamaSwap"].get("path").is_none());
        assert!(serialized["opencode"].get("path").is_none());
    }
    #[test]
    fn test_application_config_rejects_unknown_model_override_fields() {
        let configuration = ApplicationConfiguration::parse(
            r#"{
                "config": {
                    "llamaSwap": {
                        "models": {"qwen": {"unknownOption": true}}
                    }
                }
            }"#,
        );
        assert!(configuration.is_err());
    }
    #[test]
    fn test_commit_creates_parent_directories() {
        let dir = temp_dir("sync-output");
        let path = dir.join("nested").join("config.yaml");
        let result = Config::commit(&[RenderedOutput {
            target: "test",
            path: path.clone(),
            before: String::new(),
            content: "models: {}".to_string(),
        }]);
        assert!(result.is_ok());
        assert!(path.is_file());
        let _ = remove_dir_all(dir);
    }
    #[test]
    fn test_commit_rolls_back_when_a_later_rename_fails() {
        let dir = temp_dir("sync-rollback");
        create_dir_all(&dir).unwrap();
        let first = dir.join("first.yaml");
        let second = dir.join("second.jsonc");
        write(&first, "original").unwrap();
        create_dir_all(&second).unwrap();
        let result = Config::commit(&[
            RenderedOutput {
                target: "first",
                path: first.clone(),
                before: "original".to_string(),
                content: "updated".to_string(),
            },
            RenderedOutput {
                target: "second",
                path: second.clone(),
                before: String::new(),
                content: "{}".to_string(),
            },
        ]);
        assert!(result.is_err());
        assert_eq!(read_to_string(&first).unwrap(), "original");
        assert!(second.is_dir());
        assert!(!PathBuf::from(format!("{}.acorn-sync-tmp", first.display())).exists());
        assert!(!PathBuf::from(format!("{}.acorn-sync-backup", first.display())).exists());
        let _ = remove_dir_all(dir);
    }
    #[test]
    fn test_dry_run_diff_is_colored_and_creates_no_directories() {
        let dir = temp_dir("sync-dry-run");
        let config = Config {
            llama_swap: Some(llama_swap::Config {
                path: Some(dir.join("nested").join("llama.yaml").display().to_string()),
                ..Default::default()
            }),
            opencode: Some(opencode::Config {
                path: Some(dir.join("nested").join("opencode.jsonc").display().to_string()),
                ..Default::default()
            }),
        };
        let models = [ModelDetails::init().id("qwen").name("qwen").path("/models/qwen.gguf").build()];
        let output = config
            .render_llama_swap(Options {
                models: &models,
                dry_run: true,
                ..Default::default()
            })
            .unwrap();
        let changes = text_diff_changes_with_color(&output.before, &output.content, true);
        assert!(changes
            .iter()
            .any(|(tag, line)| *tag == similar::ChangeTag::Insert && line.contains("+ models:")));
        assert!(changes
            .iter()
            .filter(|(tag, _)| *tag == similar::ChangeTag::Insert)
            .all(|(_, line)| line.contains("\u{1b}[32m")));
        assert!(text_diff_changes_with_color(&output.before, &output.content, false)
            .iter()
            .all(|(_, line)| !line.contains('\u{1b}')));
        assert!(config
            .sync(Options {
                models: &models,
                dry_run: true,
                ..Default::default()
            })
            .is_ok());
        assert!(!dir.exists());
    }
    #[test]
    fn test_render_failure_leaves_all_targets_unchanged() {
        let dir = temp_dir("sync-render-failure");
        create_dir_all(&dir).unwrap();
        let llama_path = dir.join("llama.yaml");
        let opencode_path = dir.join("opencode.jsonc");
        write(&llama_path, "models: {}\n").unwrap();
        write(&opencode_path, "{ invalid").unwrap();
        let config = Config {
            llama_swap: Some(llama_swap::Config {
                path: Some(llama_path.display().to_string()),
                ..Default::default()
            }),
            opencode: Some(opencode::Config {
                path: Some(opencode_path.display().to_string()),
                ..Default::default()
            }),
        };
        let models = [ModelDetails::init().id("qwen").name("qwen").path("/models/qwen.gguf").build()];
        assert!(config
            .sync(Options {
                models: &models,
                ..Default::default()
            })
            .is_err());
        assert_eq!(read_to_string(&llama_path).unwrap(), "models: {}\n");
        assert_eq!(read_to_string(&opencode_path).unwrap(), "{ invalid");
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
    fn test_resolve_models_applies_detailed_filters() {
        let dir = temp_dir("sync-detailed-filter");
        let model_dir = dir.join("qwen");
        create_dir_all(&model_dir).unwrap();
        write(model_dir.join("qwen-q4.gguf"), b"q4").unwrap();
        write(model_dir.join("qwen-q8.gguf"), b"q8").unwrap();
        let configuration = ApplicationConfiguration {
            models: Some(vec![detailed_model("qwen", Some(vec!["q4".to_string()]))]),
            ..Default::default()
        };
        let resolved = resolve_models(&configuration, &dir).unwrap();
        assert_eq!(resolved.len(), 1);
        assert!(resolved[0].path.as_deref().is_some_and(|path| path.ends_with("qwen-q4.gguf")));
        let _ = remove_dir_all(dir);
    }
    #[test]
    fn test_resolve_models_fails_for_missing_ambiguous_and_duplicate_models() {
        let dir = temp_dir("sync-resolution-errors");
        let ambiguous_dir = dir.join("ambiguous");
        create_dir_all(&ambiguous_dir).unwrap();
        write(ambiguous_dir.join("one.gguf"), b"one").unwrap();
        write(ambiguous_dir.join("two.gguf"), b"two").unwrap();
        let missing = ApplicationConfiguration {
            models: Some(vec![ModelEntry::Selector("missing".to_string())]),
            ..Default::default()
        };
        assert!(resolve_models(&missing, &dir).is_err());
        let missing_direct = ApplicationConfiguration {
            models: Some(vec![ModelEntry::Selector(dir.join("missing.gguf").display().to_string())]),
            ..Default::default()
        };
        assert!(resolve_models(&missing_direct, &dir).is_err());
        let ambiguous = ApplicationConfiguration {
            models: Some(vec![ModelEntry::Selector("ambiguous".to_string())]),
            ..Default::default()
        };
        assert!(resolve_models(&ambiguous, &dir).is_err());
        let duplicate = ApplicationConfiguration {
            models: Some(vec![detailed_model("duplicate", None), detailed_model("duplicate", None)]),
            ..Default::default()
        };
        assert!(resolve_models(&duplicate, &dir).is_err());
        let _ = remove_dir_all(dir);
    }
    #[test]
    fn test_resolve_models_uses_direct_local_gguf_source() {
        let dir = temp_dir("sync-direct-gguf");
        create_dir_all(&dir).unwrap();
        let path = dir.join("direct.gguf");
        write(&path, b"gguf").unwrap();
        let configuration = ApplicationConfiguration {
            models: Some(vec![ModelEntry::Selector(path.display().to_string())]),
            ..Default::default()
        };
        let resolved = resolve_models(&configuration, &dir).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].id.as_deref(), Some("direct"));
        assert_eq!(resolved[0].path.as_deref(), Some(path.canonicalize().unwrap().to_str().unwrap()));
        let _ = remove_dir_all(dir);
    }
    #[test]
    fn test_shard_candidates_keep_first_complete_shard() {
        let dir = temp_dir("dedup-shards");
        create_dir_all(&dir).unwrap();
        let files = vec![
            dir.join("model-00001-of-00003.gguf"),
            dir.join("model-00002-of-00003.gguf"),
            dir.join("model-00003-of-00003.gguf"),
            dir.join("standalone.gguf"),
            dir.join("mixture-of-8.gguf"),
        ];
        let candidates = Shards::from(files).candidates().unwrap();
        assert_eq!(candidates.len(), 3);
        assert!(candidates.iter().any(|path| path.ends_with("model-00001-of-00003.gguf")));
        assert!(candidates.iter().any(|path| path.ends_with("mixture-of-8.gguf")));
        let _ = remove_dir_all(dir);
    }
    #[test]
    fn test_shard_candidates_reject_incomplete_set() {
        let files = vec![PathBuf::from("model-00001-of-00003.gguf"), PathBuf::from("model-00003-of-00003.gguf")];
        assert!(Shards::from(files).candidates().is_err());
        assert!(Shards::from(vec![PathBuf::from("model-00004-of-00003.gguf")]).candidates().is_err());
    }
    #[test]
    fn test_sync_is_stable_after_successful_two_target_write() {
        let dir = temp_dir("sync-stable");
        let llama_path = dir.join("llama.yaml");
        let opencode_path = dir.join("opencode.jsonc");
        let config = Config {
            llama_swap: Some(llama_swap::Config {
                path: Some(llama_path.display().to_string()),
                ..Default::default()
            }),
            opencode: Some(opencode::Config {
                path: Some(opencode_path.display().to_string()),
                ..Default::default()
            }),
        };
        let models = [ModelDetails::init().id("qwen").name("qwen").path("/models/qwen.gguf").build()];
        assert!(config
            .sync(Options {
                models: &models,
                ..Default::default()
            })
            .is_ok());
        let first = (read_to_string(&llama_path).unwrap(), read_to_string(&opencode_path).unwrap());
        assert!(config
            .sync(Options {
                models: &models,
                ..Default::default()
            })
            .is_ok());
        assert_eq!(first, (read_to_string(&llama_path).unwrap(), read_to_string(&opencode_path).unwrap()));
        let _ = remove_dir_all(dir);
    }
    #[test]
    fn test_sync_selected_targets_use_defaults_when_unconfigured() {
        let config = Config::default();
        let both = config.selected(&Options::default());
        assert!(both.llama_swap.is_some());
        assert!(both.opencode.is_some());
        let llama_swap = config.selected(&Options {
            llama_swap: true,
            ..Default::default()
        });
        assert!(llama_swap.llama_swap.is_some());
        assert!(llama_swap.opencode.is_none());
        let opencode = config.selected(&Options {
            opencode: true,
            ..Default::default()
        });
        assert!(opencode.llama_swap.is_none());
        assert!(opencode.opencode.is_some());
    }
    #[test]
    fn test_sync_rejects_invalid_settings() {
        let invalid_config = Config {
            llama_swap: Some(llama_swap::Config {
                executable: Some(String::new()),
                ..Default::default()
            }),
            opencode: None,
        };
        assert!(invalid_config.sync(Options::default()).is_err());
        let invalid_config = Config {
            llama_swap: None,
            opencode: Some(opencode::Config {
                default_model: Some("missing".to_string()),
                ..Default::default()
            }),
        };
        let models = [ModelDetails::init().id("qwen").build()];
        assert!(invalid_config
            .sync(Options {
                models: &models,
                ..Default::default()
            })
            .is_err());
    }
    #[test]
    fn test_sync_rejects_reserved_args_and_unknown_overrides() {
        let reserved_args = Config {
            llama_swap: Some(llama_swap::Config {
                extra_args: Some(vec![llama_swap::Argument::from("--model")]),
                ..Default::default()
            }),
            opencode: None,
        };
        assert!(reserved_args.sync(Options::default()).is_err());
        let unknown_override = Config {
            llama_swap: Some(llama_swap::Config {
                models: Some([("missing".to_string(), llama_swap::ModelOverride::default())].into_iter().collect()),
                ..Default::default()
            }),
            opencode: None,
        };
        let models = [ModelDetails::init().id("qwen").build()];
        assert!(unknown_override
            .sync(Options {
                models: &models,
                ..Default::default()
            })
            .is_err());
    }
    #[test]
    fn test_sync_validation_ignores_unselected_target() {
        let invalid_llama_swap = Config {
            llama_swap: Some(llama_swap::Config {
                executable: Some(String::new()),
                ..Default::default()
            }),
            opencode: Some(opencode::Config::default()),
        };
        let invalid_opencode = Config {
            llama_swap: Some(llama_swap::Config::default()),
            opencode: Some(opencode::Config {
                default_model: Some("missing".to_string()),
                ..Default::default()
            }),
        };
        assert!(invalid_llama_swap
            .sync(Options {
                opencode: true,
                ..Default::default()
            })
            .is_ok());
        assert!(invalid_llama_swap.sync(Options::default()).is_err());
        assert!(invalid_opencode
            .sync(Options {
                llama_swap: true,
                ..Default::default()
            })
            .is_ok());
        assert!(invalid_opencode.sync(Options::default()).is_err());
    }
}
