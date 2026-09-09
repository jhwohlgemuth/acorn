//! Configuration synchronization schemas
//!
//! Provides types for the `acorn sync` command, which resolves ACORN model
//! entries to downloaded GGUF files and merges the corresponding models into
//! llama-swap, OpenCode, VS Code, and Goose configuration.
use crate::io::oci::OciReference;
use crate::io::{
    command_exists,
    config::{FilterSet, ModelEntry},
    files_all, home_directory, write_file, ApiResult, PathConversion, Source,
};
use crate::util::constants::app::DEFAULT_MODELS_DIRECTORY;
use crate::util::{text_diff_changes_with_color, Label};
use acorn_schema::agent::ModelDetails;
use acorn_schema::validation::Validate;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use bon::Builder;
use color_eyre::eyre::eyre;
use core::fmt;
use core::ops::ControlFlow;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fs::{canonicalize, create_dir_all, remove_file, rename};
use std::path::{Path, PathBuf};
use tracing::info;
pub mod goose;
pub mod llama_swap;
pub mod opencode;
pub mod vscode;

trait McpSync {
    type Document;
    const TARGET: &'static str;
    fn update_mcp(&self, existing: Self::Document, executable: &Path, prune: bool) -> ApiResult<Self::Document>;
    fn upsert_mcp(&self, existing: Self::Document, executable: &Path, prune: bool) -> ApiResult<Self::Document> {
        match executable.is_absolute() {
            | true => self.update_mcp(existing, executable, prune),
            | false => Err(eyre!("{} MCP executable must be absolute", Self::TARGET)),
        }
    }
}
pub(crate) trait SyncTarget: Clone + Default {
    const COMMAND: &'static str;
    fn merge(self, overrides: Self) -> Self;
    fn merge_cli_overrides(self, overrides: Self) -> Self;
    fn resolve_path(explicit: Option<&str>) -> ApiResult<PathBuf>;
    fn render(&self, options: Options<'_>) -> ApiResult<RenderedOutput>;
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
#[derive(Debug)]
enum SyncError {
    Goose(color_eyre::Report),
    LlamaSwap(color_eyre::Report),
    OpenCode(color_eyre::Report),
    VsCode(color_eyre::Report),
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
///   vscode:
///     path: ./chatLanguageModels.json
///     url: http://localhost:8080/v1/chat/completions
///   goose:
///     path: ./goose.yaml
///     host: http://localhost:8080
/// ```
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Goose CLI configuration for local inference
    #[validate(nested)]
    pub goose: Option<goose::Config>,
    /// llama-swap configuration for local inference
    #[validate(nested)]
    pub llama_swap: Option<llama_swap::Config>,
    /// OpenCode configuration for local inference
    #[validate(nested)]
    pub opencode: Option<opencode::Config>,
    /// VS Code custom endpoint configuration for local inference
    #[validate(nested)]
    pub vscode: Option<vscode::Config>,
}
/// A normalized model entry ready for local GGUF resolution.
#[derive(Clone, Debug)]
pub struct ModelRequest {
    id: String,
    source: Source,
    filter: Vec<String>,
    ignore: Vec<String>,
}
/// Filesystem and fallback inputs used to resolve a model request.
#[derive(Clone, Debug)]
pub struct ModelRequestOptions<'a> {
    /// Root directory containing downloaded model repositories.
    pub models_dir: &'a Path,
    /// Assume the configured model path exists without checking the filesystem.
    pub assume_models: bool,
    /// Downloaded fallback repository identifiers to try after the configured identifier.
    pub fallbacks: Vec<String>,
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
    /// Synchronize VS Code configuration.
    #[builder(default)]
    pub vscode: bool,
    /// Synchronize Goose CLI configuration.
    #[builder(default)]
    pub goose: bool,
    /// Synchronize llama-swap configuration.
    #[builder(default)]
    pub llama_swap: bool,
    /// Synchronize the ACORN MCP server.
    #[builder(default)]
    pub mcp: bool,
    /// Absolute ACORN executable used by MCP clients.
    pub executable: Option<&'a Path>,
    /// Preview configuration without writing files.
    #[builder(default)]
    pub dry_run: bool,
    /// Disable color in dry-run output.
    #[builder(default)]
    pub no_color: bool,
    /// Remove stale managed models.
    #[builder(default)]
    pub prune: bool,
    /// Bypass command detection for explicitly selected targets.
    #[builder(default)]
    pub force: bool,
    /// Assume model paths from their identifiers without checking the filesystem.
    #[builder(default)]
    pub assume_models: bool,
    /// Models directory override used to resolve model entries.
    pub models_dir: Option<&'a Path>,
}
#[derive(Clone, Debug)]
pub(crate) struct RenderedOutput {
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
        let Self {
            goose,
            llama_swap,
            opencode,
            vscode,
        } = self;
        Self {
            goose: merge_target(goose, overrides.goose),
            llama_swap: merge_target(llama_swap, overrides.llama_swap),
            opencode: merge_target(opencode, overrides.opencode),
            vscode: merge_target(vscode, overrides.vscode),
        }
    }
    /// Merge CLI-supported overrides into configured synchronization targets
    pub fn merge_cli_overrides(self, overrides: Self) -> Self {
        let Self {
            goose,
            llama_swap,
            opencode,
            vscode,
        } = self;
        Self {
            goose: merge_target_cli(goose, overrides.goose),
            llama_swap: merge_target_cli(llama_swap, overrides.llama_swap),
            opencode: merge_target_cli(opencode, overrides.opencode),
            vscode: merge_target_cli(vscode, overrides.vscode),
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
        selected.models_for_sync(&options).and_then(|models| {
            let options = Options { models: &models, ..options };
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
                        .filter(|_| !(options.mcp && options.models.is_empty()))
                        .map_or(Ok(()), |default_model| match model_ids.iter().any(|model_id| model_id == default_model) {
                            | true => Ok(()),
                            | false => Err(eyre!("opencode.defaultModel references unknown model '{default_model}'")),
                        })
                })
                .and_then(|()| {
                    selected
                        .goose
                        .as_ref()
                        .and_then(|config| config.default_model.as_ref())
                        .filter(|_| !(options.mcp && options.models.is_empty()))
                        .map_or(Ok(()), |default_model| match model_ids.iter().any(|model_id| model_id == default_model) {
                            | true => Ok(()),
                            | false => Err(eyre!("goose.defaultModel references unknown model '{default_model}'")),
                        })
                })
                .and_then(|()| match (selected.is_empty(), options.models.is_empty(), options.mcp) {
                    | (true, _, _) => {
                        info!("{} No eligible synchronization targets detected — nothing to synchronize", Label::CAUTION);
                        Ok(())
                    }
                    | (_, true, false) => {
                        info!("{} No models resolved — nothing to synchronize", Label::CAUTION);
                        Ok(())
                    }
                    | _ => [
                        selected
                            .llama_swap
                            .as_ref()
                            .filter(|_| !options.models.is_empty())
                            .map(|config| SyncTarget::render(config, options).map_err(|why| eyre!(SyncError::LlamaSwap(why)))),
                        selected
                            .opencode
                            .as_ref()
                            .map(|config| SyncTarget::render(config, options).map_err(|why| eyre!(SyncError::OpenCode(why)))),
                        selected
                            .vscode
                            .as_ref()
                            .filter(|_| !options.models.is_empty())
                            .map(|config| SyncTarget::render(config, options).map_err(|why| eyre!(SyncError::VsCode(why)))),
                        selected
                            .vscode
                            .as_ref()
                            .filter(|_| options.mcp)
                            .map(|config| config.render_mcp(options).map_err(|why| eyre!(SyncError::VsCode(why)))),
                        selected
                            .goose
                            .as_ref()
                            .map(|config| SyncTarget::render(config, options).map_err(|why| eyre!(SyncError::Goose(why)))),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<ApiResult<Vec<_>>>()
                    .and_then(|outputs| match options.dry_run {
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
                    })
                    .map(|paths| {
                        paths
                            .iter()
                            .for_each(|path| info!("=> {} Updated {}", Label::pass(), path.display().cyan()));
                    }),
                })
        })
    }
    fn models_for_sync(&self, options: &Options<'_>) -> ApiResult<Vec<ModelDetails>> {
        let mcp_only = options.mcp && options.models.is_empty();
        let no_llama_swap = self.llama_swap.is_none();
        let no_entries = options.entries.is_empty();
        match (mcp_only, no_llama_swap, no_entries, options.models_dir) {
            | (true, _, _, _) => Ok(Vec::new()),
            | (_, true, false, Some(models_dir)) => ModelEntry::requests(options.entries).map(|requests| {
                options
                    .models
                    .iter()
                    .cloned()
                    .chain(
                        requests
                            .into_iter()
                            .filter(|request| !options.models.iter().any(|model| model.id.as_deref() == Some(request.id())))
                            .filter(|request| request.source_exists(models_dir))
                            .map(|request| ModelDetails::init().id(request.id().to_string()).name(request.id().to_string()).build()),
                    )
                    .collect()
            }),
            | _ => Ok(options.models.to_vec()),
        }
    }
    fn selected(&self, options: &Options<'_>) -> Self {
        let Options {
            goose,
            llama_swap,
            opencode,
            vscode,
            force,
            ..
        } = options;
        let explicit = *llama_swap || *opencode || *vscode || *goose;
        Self {
            goose: select_target(self.goose.as_ref(), *goose, explicit, *force),
            llama_swap: (!options.mcp)
                .then(|| select_target(self.llama_swap.as_ref(), *llama_swap, explicit, *force))
                .flatten(),
            opencode: select_target(self.opencode.as_ref(), *opencode, explicit, *force),
            vscode: select_target(self.vscode.as_ref(), *vscode, explicit, *force),
        }
    }
    fn is_empty(&self) -> bool {
        let Self {
            goose,
            llama_swap,
            opencode,
            vscode,
        } = self;
        goose.is_none() && llama_swap.is_none() && opencode.is_none() && vscode.is_none()
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
}
impl ModelRequest {
    fn repository(&self) -> String {
        OciReference::parse(&self.id)
            .and_then(|reference| reference.local_repository())
            .unwrap_or_else(|_| self.id.clone())
    }
    /// Build model details from the configured source or expected models-directory path without checking the filesystem.
    pub fn assume(&self, models_dir: &Path) -> ModelDetails {
        let path = match &self.source {
            | Source::Local { path, .. } => path.clone(),
            | _ => models_dir.join(self.repository()),
        };
        ModelDetails::init()
            .id(self.id.clone())
            .name(self.id.clone())
            .path(path.display().to_string())
            .build()
    }
    fn details(&self, path: PathBuf) -> ApiResult<ModelDetails> {
        canonicalize(&path)
            .map_err(|why| {
                eyre!(ShardError::Canonicalize {
                    model: self.id.clone(),
                    why: why.into(),
                })
            })
            .map(|path| ModelDetails {
                id: Some(self.id.clone()),
                name: Some(self.id.clone()),
                path: Some(path.cross_platform_display()),
                ..Default::default()
            })
    }
    /// Return the configured model identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
    fn source_exists(&self, models_dir: &Path) -> bool {
        match &self.source {
            | Source::Local { path, .. } => path.exists(),
            | _ => models_dir.join(self.repository()).is_dir(),
        }
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
    pub fn resolve(&self, options: &ModelRequestOptions<'_>) -> ApiResult<ModelDetails> {
        let ModelRequestOptions {
            models_dir,
            assume_models,
            fallbacks,
        } = options;
        match assume_models {
            | true => Ok(self.assume(models_dir)),
            | false => {
                let direct_path = match &self.source {
                    | Source::Local { path, .. } if path.is_file() || Self::is_gguf(path) => Some(path.clone()),
                    | _ => None,
                };
                match direct_path {
                    | Some(path) => self.resolve_direct(&path),
                    | None => match self.resolve_repository(models_dir, &self.repository()) {
                        | Ok(model) => Ok(model),
                        | Err(direct_error) if fallbacks.is_empty() => Err(direct_error),
                        | Err(direct_error) => {
                            match fallbacks.iter().try_fold(Vec::new(), |failures, repository| {
                                match self.resolve_repository(models_dir, repository) {
                                    | Ok(model) => ControlFlow::Break(model),
                                    | Err(why) => {
                                        ControlFlow::Continue(failures.into_iter().chain([(repository.clone(), why.to_string())]).collect::<Vec<_>>())
                                    }
                                }
                            }) {
                                | ControlFlow::Break(model) => Ok(model),
                                | ControlFlow::Continue(failures) => Err(eyre!(
                                    "{direct_error}; fallback attempts: {}",
                                    failures
                                        .into_iter()
                                        .map(|(repository, why)| format!("{repository} ({why})"))
                                        .collect::<Vec<_>>()
                                        .join("; ")
                                )),
                            }
                        }
                    },
                }
            }
        }
    }
    fn resolve_repository(&self, models_dir: &Path, repository: &str) -> ApiResult<ModelDetails> {
        let Self { id, filter, ignore, .. } = self;
        resolve_gguf(&models_dir.display().to_string(), repository, Some(filter), Some(ignore)).and_then(|models| match models.as_slice() {
            | [model] => model
                .path
                .as_ref()
                .map(PathBuf::from)
                .ok_or_else(|| eyre!("Resolved model '{id}' has no GGUF path"))
                .and_then(|path| self.details(path)),
            | [] => Err(eyre!("No GGUF candidates resolved for model '{id}'")),
            | _ => Err(eyre!(ShardError::Ambiguous {
                model: id.clone(),
                paths: models.iter().filter_map(|model| model.path.clone()).collect(),
            })),
        })
    }
    fn resolve_direct(&self, path: &Path) -> ApiResult<ModelDetails> {
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
impl fmt::Display for Shard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.path.display().fmt(formatter)
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
impl core::error::Error for ShardError {}
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
impl From<Vec<PathBuf>> for Shards {
    fn from(paths: Vec<PathBuf>) -> Self {
        Self(paths.into_iter().map(Shard::from).collect())
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
impl core::error::Error for SyncError {}
impl fmt::Display for SyncError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::Goose(why) => write!(formatter, "Synchronize Goose — {why}"),
            | Self::LlamaSwap(why) => write!(formatter, "Synchronize llama-swap — {why}"),
            | Self::OpenCode(why) => write!(formatter, "Synchronize OpenCode — {why}"),
            | Self::VsCode(why) => write!(formatter, "Synchronize VS Code — {why}"),
        }
    }
}
fn merge_target<T: SyncTarget>(config: Option<T>, overrides: Option<T>) -> Option<T> {
    match (config, overrides) {
        | (Some(config), Some(overrides)) => Some(config.merge(overrides)),
        | (config, overrides) => overrides.or(config),
    }
}
fn merge_target_cli<T: SyncTarget>(config: Option<T>, overrides: Option<T>) -> Option<T> {
    match (config, overrides) {
        | (Some(config), Some(overrides)) => Some(config.merge_cli_overrides(overrides)),
        | (config, overrides) => overrides.or(config),
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
fn select_target<T: SyncTarget>(config: Option<&T>, requested: bool, explicit: bool, force: bool) -> Option<T> {
    ((requested || !explicit) && (command_exists(T::COMMAND) || (requested && force))).then(|| config.cloned().unwrap_or_default())
}

#[cfg(test)]
mod tests;
