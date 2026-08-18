//! Application configuration functions and data structures
//!
//! ACORN is configured by a JSON, JSONC, or YAML file (typically named `.acorn.json`)
//!
//! The ACORN configuration file configures what buckets should be downloaded, readability analysis, <span title="Large Language Model">LLM</span> settings, and more.
//!
use crate::io::api::{github, gitlab, Configuration, Endpoint};
use crate::io::database::schema::{ModelRow, Table};
use crate::io::database::{resolve_database_path, Database, Provenance, ResearchActivityCandidate, Row};
use crate::io::{
    files_all, parse_jsonc_cst, read_file, sync, with_progress, write_file, write_file_bytes, ApiResult, CstRootNode, CstValue, Executor, FromPath,
    InputOutput, ProgressType, Source,
};
use crate::prelude::{self, env, exit, Arc, ErrorKind, HashMap, HashSet, Mutex, Path, PathBuf};
use crate::schema::pid::{Identifier, PID};
use crate::schema::research_activity::ResearchActivity;
use crate::schema::OneOrMany;
use crate::schema::{
    agent::{ModelDetails, Quantization},
    hardware::memory::Memory,
};
use crate::util::constants::app::{DEFAULT_CONFIG_FILENAMES, IGNORE, SUPPORTED_RAD_FILETYPES};
use crate::util::{detect_json, is_filetype, suffix, text_diff_changes_with_color, Label, MimeType, StringConversion};
use crate::{Location, Repository, Scheme};
use bon::Builder;
use color_eyre::eyre::{eyre, Report};
use core::fmt::{self, Debug};
use core::future::Future;
use core::iter::once;
use derive_more::Display;
use fancy_regex::Regex;
use itertools::Itertools;
use jiff::Timestamp;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::path::Component;
use tracing::{error, info, warn};

/// Authentication requirement for a configured model download source
#[derive(Clone, Debug, Default, Display, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationRequirement {
    /// Authentication is not expected for this model source
    None,
    /// Authentication may be used when a token is configured
    #[default]
    Optional,
    /// Authentication is required for this model source
    Required,
}
/// Model config entry — either a bare selector string or a detailed download entry
/// ### Notes
/// Models can be specified in several ways in the `models` array:
///
/// **Bare selector string** — a model ID, path, or URL:
/// ```json
/// {
///     "models": [
///         "meta-llama/Llama-2-7b-hf",
///         "microsoft/phi-2"
///     ]
/// }
/// ```
///
/// **Detailed entry with Hugging Face source** (simple location string):
/// ```json
/// {
///     "models": [
///         {
///             "name": "tiny-bert",
///             "source": {
///                 "provider": "huggingface",
///                 "location": "https://huggingface.co/hf-internal-testing/tiny-random-bert"
///             }
///         }
///     ]
/// }
/// ```
///
/// **Detailed entry with revision and auth**:
/// ```json
/// {
///     "models": [
///         {
///             "name": "tiny-bert",
///             "source": {
///                 "provider": "huggingface",
///                 "location": "https://huggingface.co/hf-internal-testing/tiny-random-bert"
///             },
///             "revision": "refs/pr/1",
///             "auth": "required",
///             "filter": ["Q4_K_M.*\\.gguf$"],
///             "ignore": ["Q2_", "Q3_"]
///         }
///     ]
/// }
/// ```
///
/// **Detailed entry with Hugging Face source** (detailed location with scheme and revision):
/// ```json
/// {
///     "models": [
///         {
///             "name": "tiny-bert",
///             "source": {
///                 "provider": "huggingface",
///                 "location": {
///                     "scheme": "https",
///                     "uri": "https://huggingface.co/hf-internal-testing/tiny-random-bert",
///                     "revision": "main"
///                 }
///             }
///         }
///     ]
/// }
/// ```
///
/// **Detailed entry with local Git repository source**:
/// ```json
/// {
///     "models": [
///         {
///             "name": "qwen-local",
///             "source": {
///                 "provider": "git",
///                 "location": "file:./models/qwen.gguf"
///             }
///         }
///     ]
/// }
/// ```
///
/// **Mixed array** — bare strings and detailed entries together:
/// ```json
/// {
///     "models": [
///         "meta-llama/Llama-2-7b-hf",
///         {
///             "name": "qwen-local",
///             "source": {
///                 "provider": "git",
///                 "location": "file:./models/qwen.gguf"
///             }
///         }
///     ]
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelEntry {
    /// Bare model ID / path / URL string
    Selector(String),
    /// Detailed model download entry
    Entry(ModelEntryOptions),
}
/// Runner status for CI/CD pipelines
#[derive(Clone, Debug, Default, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnerStatus {
    /// Online and available to run jobs
    #[default]
    Online,
    /// Offline and/or unavailable to run jobs
    Offline,
    /// Runner has not contacted the server for a while
    Stale,
    /// Runner has never contacted the server
    NeverContacted,
    /// Deprecated
    Active,
    /// Deprecated
    Paused,
}
/// Runner types for CI/CD pipelines
///
/// Mostly for GitLab as GitHub only has two types: hosted (by GitHub) and self-hosted
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum RunnerType {
    /// Accessible to a specific group and its projects/subgroups
    #[default]
    #[serde(rename = "group_type", alias = "group")]
    Group,
    /// Available to all projects and groups within an instance
    /// > Also called "shared runners" in GitLab
    #[serde(rename = "instance_type", alias = "instance")]
    Instance,
    /// Available to a specific project
    #[serde(rename = "project_type", alias = "project")]
    Project,
}
/// Struct for application configuration
/// ### Example `.acorn` configuration file
/// ```json
/// {
///     "buckets": [
///         {
///             "name": "big-science",
///             "repository": {
///                 "provider": "github",
///                 "uri": "https://github.com/username/example"
///             }
///         },
///         {
///             "name": "does-it-scale",
///             "repository": {
///                 "provider": "gitlab",
///                 "id": 12345,
///                 "uri": "https://gitlab.com/username/example"
///             }
///         }
///     ],
///     "models": [
///         "meta-llama/Llama-2-7b-hf",
///         "microsoft/phi-2"
///     ],
///     "runners": [
///         {
///             "name": "ACORN Runner B",
///             "repository": {
///                 "provider": "gitlab",
///                 "id": 24758,
///                 "uri": "https://code.ornl.gov/research-enablement"
///             },
///             "type": "group",
///             "runUntagged": true
///         }
///     ]
/// }
/// ```
#[derive(Clone, Debug, Default, Serialize, eserde::Deserialize)]
pub struct ApplicationConfiguration {
    /// CST root for JSONC comment-preserving round-trips
    #[serde(skip)]
    pub cst: Option<CstRootNode>,
    /// List of buckets
    #[eserde(compat)]
    pub buckets: Option<Vec<Bucket>>,
    /// Synchronization targets for llama-swap, OpenCode, VS Code, and Goose
    #[eserde(compat)]
    pub config: Option<sync::Config>,
    /// List of endpoints
    #[eserde(compat)]
    pub endpoints: Option<Vec<Endpoint>>,
    /// List of models to download — bare IDs/paths or detailed download entries
    #[eserde(compat)]
    pub models: Option<Vec<ModelEntry>>,
    /// List of runners
    #[eserde(compat)]
    pub runners: Option<Vec<RunnerDetails>>,
    /// Lookup object for whitelisted downloadable items
    #[eserde(compat)]
    pub whitelist: Option<WhitelistLookup>,
}
/// Detailed model download entry for use in `ApplicationConfiguration.models`
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[builder(start_fn = init)]
pub struct ModelEntryOptions {
    /// User-facing model name
    pub name: String,
    /// Model weight source (local path, URL, or Hugging Face repository)
    pub source: Repository,
    /// Optional repository revision, branch, or tag to resolve
    #[serde(default)]
    pub revision: Option<String>,
    /// Authentication requirement for this model source
    #[serde(default)]
    pub auth: Option<AuthenticationRequirement>,
    /// Regex pattern(s) used to include model files for this entry
    #[serde(default)]
    pub filter: Option<Vec<String>>,
    /// Regex pattern(s) used to exclude model files for this entry
    #[serde(default)]
    pub ignore: Option<Vec<String>>,
    /// Ordered exact GGUF quantization allowlist
    #[serde(default)]
    pub quantization: Option<OneOrMany<Quantization>>,
    /// Maximum GPU memory available for model weights
    #[serde(default)]
    pub gpu_memory: Option<Memory>,
    /// Copy local model files into the model directory instead of referencing in place
    #[serde(default)]
    pub copy: Option<bool>,
    /// Symlink local model files into the model directory instead of referencing in place
    #[serde(default)]
    pub symlink: Option<bool>,
}
/// Struct for bucket data
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[builder(start_fn = init)]
pub struct Bucket {
    /// Bucket name
    ///
    /// See <https://schema.org/name>
    pub name: Option<String>,
    /// Bucket description
    ///
    /// See <https://schema.org/description>
    pub description: Option<String>,
    /// Code repository data of bucket
    ///
    /// See <https://schema.org/codeRepository>
    #[serde(alias = "repository")]
    pub code_repository: Repository,
}
/// Options for bucket file operations (copy and download)
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init)]
pub struct BucketOptions {
    /// Path to output directory
    pub output: Option<PathBuf>,
    /// Number of threads used for parallel processing
    #[builder(default = 10)]
    pub threads: usize,
    /// Suppress progress output
    #[builder(default)]
    pub quiet: bool,
    /// Regex pattern(s) of files to ignore
    #[builder(default)]
    pub ignore: Vec<String>,
    /// Regex pattern(s) of files to include
    #[builder(default)]
    pub filter: Vec<String>,
    /// Save transferred files directly beneath the output directory
    #[builder(default)]
    pub flatten: bool,
    /// Replace existing files or directories at selected output paths
    #[builder(default)]
    pub clobber: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct TransferItem {
    source: PathBuf,
    destination: PathBuf,
}
impl From<&Path> for TransferItem {
    fn from(source: &Path) -> Self {
        Self {
            source: source.to_path_buf(),
            destination: source.to_path_buf(),
        }
    }
}
impl TransferItem {
    fn collect(paths: Vec<String>, flatten: bool) -> ApiResult<Vec<Self>> {
        let items = paths
            .into_iter()
            .map(PathBuf::from)
            .map(|path| Self::from(path.as_path()))
            .map(|item| match flatten {
                | true => item.flatten(),
                | false => Ok(item),
            })
            .collect::<ApiResult<Vec<_>>>();
        match items {
            | Ok(items) => items
                .iter()
                .try_fold(HashMap::<PathBuf, PathBuf>::new(), |mut destinations, item| {
                    let destination = &item.destination;
                    let safe = !destination.as_os_str().is_empty() && destination.components().all(|part| matches!(part, Component::Normal(_)));
                    match safe {
                        | false => Err(eyre!("Output path is unsafe — '{}'", item.destination.display())),
                        | true => match destinations.insert(item.destination.clone(), item.source.clone()) {
                            | Some(source) => Err(eyre!(
                                "Output path collision for '{}' — '{}' and '{}'",
                                item.destination.display(),
                                source.display(),
                                item.source.display()
                            )),
                            | None => Ok(destinations),
                        },
                    }
                })
                .map(|_| items),
            | Err(why) => Err(why),
        }
    }
    fn flatten(self) -> ApiResult<Self> {
        match self.source.file_name().map(PathBuf::from) {
            | Some(destination) => Ok(Self { destination, ..self }),
            | None => Err(eyre!("Cannot flatten repository path without a filename — {}", self.source.display())),
        }
    }
}
/// Files successfully transferred from one configured bucket.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferManifest {
    /// Configured bucket name.
    pub bucket: Option<String>,
    /// Repository location used for the transfer.
    pub repository: String,
    /// Relative paths written beneath the output directory.
    pub files: Vec<PathBuf>,
}
/// Struct for filtering files based on regex patterns
#[derive(Clone, Debug)]
pub struct FilterSet {
    /// Regex patterns to ignore
    pub ignore: Vec<Regex>,
    /// Regex patterns to include
    pub filter: Vec<Regex>,
}
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[builder(start_fn = at, on(String, into))]
#[serde(rename_all = "camelCase")]
/// CI/CD runner configuration entry
pub struct RunnerDetails {
    /// Code repository project/group where runner will be used
    #[builder(start_fn)]
    #[serde(alias = "repository")]
    pub code_repository: Repository,
    /// Runner name
    /// ### Note
    /// Primarily for identification in the GitLab UI
    pub name: Option<String>,
    /// Runner type (e.g., group, instance, project)
    #[builder(default, with = |method: &str| RunnerType::from(method))]
    #[serde(rename = "type")]
    pub runner_type: RunnerType,
    /// Optional description of runner
    pub description: Option<String>,
    /// Runner executor type (e.g., Docker, Kubernetes, Shell)
    #[builder(default = Executor::Docker)]
    #[serde(default = "default_executor")]
    pub executor: Executor,
    /// Does the runner need GPU capabilities?
    #[builder(default)]
    #[serde(default, alias = "gpu")]
    pub gpu_enabled: bool,
    /// List of tags associated with the runner
    #[serde(default, alias = "tag_list")]
    pub tags: Option<Vec<String>>,
    /// Whether the runner runs untagged jobs (GitLab-specific)
    #[builder(default)]
    #[serde(default, alias = "run_untagged")]
    pub run_untagged: bool,
    /// Optional GitLab host/domain override (for self-managed instances)
    pub host: Option<String>,
    /// Default Docker image for the runner itself
    #[builder(default = String::from("gitlab/gitlab-runner:latest"))]
    #[serde(default = "default_docker_image")]
    pub docker_image: String,
    /// Runner identifier assigned by GitLab during creation
    #[serde(default)]
    pub identifier: Option<u64>,
    /// Runner authentication token returned from GitLab API
    #[serde(default)]
    pub token: Option<String>,
}
/// Whitelist entry for downloadable items
/// ### Note
/// An empty whitelist is interpreted to mean there are no restrictions and any item will be allowed
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[builder(start_fn = init)]
pub struct WhitelistLookup {
    /// Whitelisted buckets
    pub buckets: Option<Vec<String>>,
    /// Whitelisted model IDs/URLs, or one URL to a JSON whitelist document
    pub models: Option<OneOrMany<String>>,
}
impl InputOutput for ApplicationConfiguration {
    /// Read and parse application configuration file (JSON, JSONC, or YAML)
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into();
        match source.file_name().and_then(|name| name.to_str()) {
            | Some(".acorn") => Self::read_jsonc(source),
            | _ => match MimeType::from_path(&source) {
                | MimeType::Json => Self::read_json(source.clone()),
                | MimeType::Jsonc => Self::read_jsonc(source.clone()),
                | MimeType::Yaml => Self::read_yaml(source.clone()),
                | _ => Err(eyre!("Unsupported configuration file extension")),
            },
        }
    }
    /// Read configuration (e.g., `.acorn.json`) using Serde and [`ApplicationConfiguration`] struct
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        let content = match read_file(path.clone()) {
            | Ok(value) if !value.is_empty() => value,
            | Ok(_) | Err(_) => {
                error!(
                    path = path.to_string_lossy().to_string(),
                    "=> {} ACORN configuration JSON content",
                    Label::fail()
                );
                "{}".to_owned()
            }
        };
        match Self::parse_json(content) {
            | Ok(config) => Ok(config),
            | Err(errors) => {
                let details: Vec<String> = errors
                    .iter()
                    .map(|e| format!("{}: {}", e.path().map_or("root".into(), |p| p.to_string()), e.message()))
                    .collect();
                Err(eyre!("{}", details.join("\n")))
            }
        }
    }
    /// Read configuration (e.g., `.acorn.jsonc`) using JSONC parser
    ///
    /// Supports comments (`//` and `/* */`) and trailing commas.
    fn read_jsonc(path: PathBuf) -> ApiResult<Self> {
        let content = match read_file(path.clone()) {
            | Ok(value) if !value.is_empty() => value,
            | Ok(_) | Err(_) => {
                error!(
                    path = path.to_string_lossy().to_string(),
                    "=> {} ACORN configuration JSONC content",
                    Label::fail()
                );
                "{}".to_owned()
            }
        };
        Self::parse_jsonc(&content).map_err(|why| eyre!("Failed to read JSONC config `{}` — {}", path.display(), why))
    }
    /// Read configuration (e.g., `.acorn.yml`) using Serde and [`ApplicationConfiguration`] struct
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        let content = match read_file(path.clone()) {
            | Ok(value) => value,
            | Err(_) => {
                error!(
                    path = path.to_string_lossy().to_string(),
                    "=> {} ACORN configuration YAML content",
                    Label::fail()
                );
                "".to_owned()
            }
        };
        Self::parse_yaml(content).map_err(|why| eyre!("Failed to parse YAML config — {why}"))
    }
    /// Write configuration to specified path (detects JSON, JSONC, or YAML from extension)
    ///
    /// JSONC files are written as strict JSON (no comments generated).
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let target = path.into();
        match target.file_name().and_then(|name| name.to_str()) {
            | Some(".acorn") => self.write_json(&target),
            | _ => match MimeType::from_path(&target) {
                | MimeType::Json | MimeType::Jsonc => self.write_json(&target),
                | MimeType::Yaml => self.write_yaml(&target),
                | _ => Err(eyre!("Unsupported configuration file extension")),
            },
        }
    }
    /// Write configuration as JSON to specified path
    ///
    /// If the config was parsed from JSONC, preserves comments via CST.
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let target = path.into();
        match &self.cst {
            | Some(cst) => write_file(target, cst.to_string()),
            | None => serde_json::to_string_pretty(&self)
                .map_err(|why| eyre!("Failed to serialize JSON config — {why}"))
                .and_then(|content| write_file(target, content)),
        }
    }
    /// Write configuration as YAML to specified path
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let target = path.into();
        serde_norway::to_string(&self)
            .map_err(|why| eyre!("Failed to serialize YAML config — {why}"))
            .and_then(|content| write_file(target.clone(), content))
    }
}
impl ApplicationConfiguration {
    /// Load explicit or discovered application configuration, rejecting a missing explicit path.
    pub fn load(path: &Option<PathBuf>) -> ApiResult<Self> {
        match path {
            | Some(path) if !path.is_file() => Err(eyre!("Configuration file does not exist — {}", path.display())),
            | _ => Self::resolve(path).map_or_else(|| Ok(Self::default()), Self::read),
        }
    }
    /// Merge configured synchronization settings with runtime overrides.
    pub fn resolve_sync_config(&self, overrides: sync::Config) -> sync::Config {
        self.config.clone().unwrap_or_default().merge(overrides)
    }
    /// Return configured model entries and their download whitelist.
    pub fn model_entries_and_whitelist(&self) -> (Vec<ModelEntry>, Option<OneOrMany<String>>) {
        (
            self.models.clone().unwrap_or_default(),
            self.whitelist.as_ref().and_then(|lookup| lookup.models.clone()),
        )
    }
    /// Synchronize selected model entries with configured local inference targets.
    pub fn sync(&self, options: sync::Options<'_>) -> ApiResult<()> {
        let sync_config = self.config.clone().unwrap_or_default();
        sync_config.resolve_models_dir(options.models_dir).and_then(|models_dir| {
            info!("{} Resolving selected models for synchronization", Label::run());
            let request_options = sync::ModelRequestOptions {
                models_dir: &models_dir,
                assume_models: options.assume_models,
                fallbacks: Vec::new(),
            };
            ModelEntry::resolve(options.entries, &request_options).and_then(|resolved| {
                sync_config.sync(sync::Options {
                    models: &resolved,
                    models_dir: Some(&models_dir),
                    ..options
                })
            })
        })
    }
    /// Synchronize selected models and add their unique identifiers to ACORN configuration.
    pub fn sync_and_update(&self, path: &Option<PathBuf>, options: sync::Options<'_>) -> ApiResult<()> {
        let path = Self::resolve(path)
            .or_else(|| path.clone())
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILENAMES[0]));
        self.sync(options)
            .and_then(|()| self.with_models(options.entries))
            .and_then(|configuration| configuration.write_or_preview(&path, options.dry_run, options.no_color))
    }
    fn with_models(&self, entries: &[ModelEntry]) -> ApiResult<Self> {
        self.models
            .clone()
            .unwrap_or_default()
            .into_iter()
            .chain(entries.iter().cloned())
            .try_fold((HashSet::new(), Vec::new()), |(mut identifiers, mut models), entry| {
                sync::ModelRequest::try_from(&entry).map(|request| {
                    if identifiers.insert(request.id().to_string()) {
                        models.push(entry);
                    }
                    (identifiers, models)
                })
            })
            .and_then(|(_, models)| {
                let mut configuration = self.clone();
                configuration.models = Some(models);
                match configuration.cst.clone() {
                    | Some(cst) => serde_json::to_value(&configuration.models)
                        .map_err(|why| eyre!("Failed to serialize ACORN model configuration — {why}"))
                        .map(|models| {
                            let root = cst.object_value_or_set();
                            match root.get("models") {
                                | Some(property) => property.set_value(CstValue(&models).into()),
                                | None => {
                                    root.append("models", CstValue(&models).into());
                                }
                            }
                            root.array_value_or_set("models").ensure_multiline();
                            configuration
                        }),
                    | None => Ok(configuration),
                }
            })
    }
    fn write_or_preview(&self, path: &Path, dry_run: bool, no_color: bool) -> ApiResult<()> {
        let before = path
            .is_file()
            .then(|| read_file(path))
            .transpose()
            .map(|content| content.unwrap_or_default());
        before.and_then(|before| {
            self.render(path).and_then(|content| match (dry_run, before == content) {
                | (_, true) => {
                    info!("=> {} No changes for {}", Label::CAUTION, path.display());
                    Ok(())
                }
                | (true, false) => {
                    match no_color {
                        | true => println!("\n{}", path.display()),
                        | false => println!("\n{}", path.display().cyan().bold()),
                    }
                    text_diff_changes_with_color(&before, &content, !no_color)
                        .iter()
                        .for_each(|(_, line)| print!("{line}"));
                    Ok(())
                }
                | (false, false) => self
                    .write(path)
                    .inspect(|()| info!("=> {} Updated {}", Label::pass(), path.display().cyan())),
            })
        })
    }
    fn render(&self, path: &Path) -> ApiResult<String> {
        match path.file_name().and_then(|name| name.to_str()) {
            | Some(".acorn") => self.render_json(),
            | _ => match MimeType::from_path(path) {
                | MimeType::Json | MimeType::Jsonc => self.render_json(),
                | MimeType::Yaml => serde_norway::to_string(self).map_err(|why| eyre!("Failed to serialize YAML config — {why}")),
                | _ => Err(eyre!("Unsupported configuration file extension")),
            },
        }
    }
    fn render_json(&self) -> ApiResult<String> {
        match &self.cst {
            | Some(cst) => Ok(cst.to_string()),
            | None => serde_json::to_string_pretty(self).map_err(|why| eyre!("Failed to serialize JSON config — {why}")),
        }
    }
    /// Resolve application configuration path
    pub fn resolve(path: &Option<PathBuf>) -> Option<PathBuf> {
        let directory = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::resolve_in(path, &directory)
    }
    /// Resolve an application configuration path relative to an explicit directory.
    pub fn resolve_in(path: &Option<PathBuf>, directory: &Path) -> Option<PathBuf> {
        path.as_ref().filter(|value| value.is_file()).cloned().or_else(|| {
            DEFAULT_CONFIG_FILENAMES
                .iter()
                .map(|name| directory.join(name))
                .find(|candidate| candidate.exists())
        })
    }
    /// Parse ACORN configuration in JSON, JSONC, or YAML format
    /// ### Note
    /// If the content is valid JSON, it is parsed as JSON.
    /// If it is not valid JSON but starts with `{` or `[`, it is first attempted to be parsed as JSON, and if that fails, it is attempted to be parsed as YAML.
    /// If the content does not start with `{` or `[`, it is parsed as YAML.
    pub fn parse(content: impl AsRef<str>) -> ApiResult<Self> {
        let trimmed = content.as_ref().trim();
        if detect_json(trimmed) {
            match Self::parse_json(trimmed) {
                | Ok(value) => Ok(value),
                | Err(json_errors) => match Self::parse_jsonc(trimmed) {
                    | Ok(value) => Ok(value),
                    | Err(_) => {
                        let details: Vec<String> = json_errors
                            .iter()
                            .map(|e| format!("{}: {}", e.path().map_or("root".into(), |p| p.to_string()), e.message()))
                            .collect();
                        Err(eyre!("{}", details.join("\n")))
                    }
                },
            }
        } else if trimmed.starts_with('{') || trimmed.starts_with('[') {
            match Self::parse_json(trimmed) {
                | Ok(value) => Ok(value),
                | Err(json_errors) => match Self::parse_yaml(trimmed) {
                    | Ok(value) => Ok(value),
                    | Err(why) => {
                        let details: Vec<String> = json_errors
                            .iter()
                            .map(|e| format!("{}: {}", e.path().map_or("root".into(), |p| p.to_string()), e.message()))
                            .collect();
                        Err(eyre!(
                            "Failed to parse ACORN configuration as JSON or YAML.\nJSON errors:\n{}\nYAML error: {why}",
                            details.join("\n")
                        ))
                    }
                },
            }
        } else {
            match Self::parse_yaml(trimmed) {
                | Ok(value) => Ok(value),
                | Err(why) => Err(eyre!("Failed to parse ACORN configuration YAML — {why}")),
            }
        }
    }
    fn parse_json(content: impl AsRef<str>) -> Result<Self, eserde::DeserializationErrors> {
        eserde::json::from_str(content.as_ref())
    }
    fn parse_jsonc(content: impl AsRef<str>) -> ApiResult<Self> {
        parse_jsonc_cst::<ApplicationConfiguration>(content.as_ref()).map(|(mut config, cst)| {
            config.cst = Some(cst);
            config
        })
    }
    fn parse_yaml(content: impl AsRef<str>) -> serde_norway::Result<Self> {
        serde_norway::from_str(content.as_ref())
    }
}
impl Bucket {
    /// Get hosting domain from bucket struct
    pub(crate) fn domain(&self) -> ApiResult<String> {
        let location = match &self.code_repository {
            | Repository::GitHub { location } | Repository::GitLab { location, .. } => location,
            | Repository::Git { .. } => return Err(eyre!("Domain is unsupported for generic Git repositories")),
            | Repository::HuggingFace { .. } => return Err(eyre!("Domain is unsupported for Hugging Face repositories")),
        };
        match location.scheme() {
            | Scheme::HTTPS => location.host().ok_or_else(|| eyre!("Failed to parse repository host from URI")),
            | _ => Err(eyre!("Unsupported repository URI scheme")),
        }
    }
    fn remove_destination(path: &Path) -> ApiResult<()> {
        match path.symlink_metadata() {
            | Ok(metadata) if metadata.file_type().is_symlink() || metadata.is_file() => {
                prelude::remove_file(path).map_err(|why| eyre!("Failed to remove existing output path {} — {why}", path.display()))
            }
            | Ok(_) => prelude::remove_dir_all(path).map_err(|why| eyre!("Failed to remove existing output directory {} — {why}", path.display())),
            | Err(why) if why.kind() == ErrorKind::NotFound => Ok(()),
            | Err(why) => Err(eyre!("Failed to inspect existing output path {} — {why}", path.display())),
        }
    }
    fn prepare_destination(output: &Path, destination: &Path) -> ApiResult<PathBuf> {
        let target = output.join(destination);
        destination
            .parent()
            .into_iter()
            .flat_map(Path::ancestors)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|parent| output.join(parent))
            .try_for_each(|parent| match parent.symlink_metadata() {
                | Ok(metadata) if metadata.is_dir() => Ok(()),
                | Ok(_) => Self::remove_destination(&parent).and_then(|()| {
                    prelude::create_dir_all(&parent).map_err(|why| eyre!("Failed to create output directory {} — {why}", parent.display()))
                }),
                | Err(why) if why.kind() == ErrorKind::NotFound => {
                    prelude::create_dir_all(&parent).map_err(|why| eyre!("Failed to create output directory {} — {why}", parent.display()))
                }
                | Err(why) => Err(eyre!("Failed to inspect output directory {} — {why}", parent.display())),
            })
            .and_then(|()| Self::remove_destination(&target))
            .map(|()| target)
    }
    async fn write_file<F, Fut, E>(output: &Path, destination: &Path, clobber: bool, write_lock: &Mutex<()>, get_bytes: F) -> ApiResult<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Vec<u8>, E>>,
        E: Into<Report>,
    {
        match clobber {
            | false => write_file_bytes(output.join(destination), get_bytes).await,
            | true => match get_bytes().await.map_err(Into::into) {
                | Ok(bytes) => match write_lock.lock() {
                    | Ok(_guard) => Self::prepare_destination(output, destination).and_then(|target| {
                        prelude::write(&target, bytes)
                            .map(|_| ())
                            .map_err(|why| eyre!("Failed to write output file {} — {why}", target.display()))
                    }),
                    | Err(why) => Err(eyre!("Failed to lock bucket output — {why}")),
                },
                | Err(why) => Err(why),
            },
        }
    }
    /// Copy files from (local) bucket to local directory
    /// ### Notes
    /// - Ignores files listed in [`IGNORE`]
    /// - Only copies files from local repositories
    pub async fn copy_files(self: Bucket, options: &BucketOptions) -> ApiResult<TransferManifest> {
        let BucketOptions { output, ignore, filter, .. } = options;
        let output = Arc::new(output.clone().unwrap_or_default());
        match FilterSet::compile(ignore, filter) {
            | Ok(filters) => {
                let Bucket { name, code_repository, .. } = self.clone();
                match code_repository.is_local() {
                    | true => {
                        let bucket_root = match code_repository.location().path() {
                            | Some(value) => PathBuf::from(value).to_absolute_path(),
                            | None => {
                                return Err(eyre!(
                                    "Bucket {} has no local path — cannot copy files",
                                    name.as_deref().unwrap_or("unknown")
                                ))
                            }
                        };
                        let items = filter_paths(
                            files_all(PathBuf::from(&bucket_root), None::<Vec<String>>)
                                .into_iter()
                                .map(|x| x.display().to_string())
                                .collect::<Vec<String>>(),
                            &filters,
                        )
                        .into_iter()
                        .filter(|path| PathBuf::from(path).is_file())
                        .filter_map(|path| {
                            PathBuf::from(&path)
                                .strip_prefix(&bucket_root)
                                .ok()
                                .map(|relative| relative.display().to_string())
                        })
                        .collect::<Vec<String>>();
                        match TransferItem::collect(items, options.flatten) {
                            | Ok(items) => {
                                let bucket_root = Arc::new(bucket_root);
                                let clobber = options.clobber;
                                let write_lock = Arc::new(Mutex::new(()));
                                let operation = {
                                    let bucket_root = Arc::clone(&bucket_root);
                                    let output = Arc::clone(&output);
                                    let write_lock = Arc::clone(&write_lock);
                                    move |item: TransferItem| {
                                        let bucket_root = Arc::clone(&bucket_root);
                                        let output = Arc::clone(&output);
                                        let write_lock = Arc::clone(&write_lock);
                                        async move {
                                            let source = PathBuf::from(bucket_root.as_str()).join(item.source);
                                            Self::write_file(output.as_path(), &item.destination, clobber, write_lock.as_ref(), || async {
                                                prelude::read(source)
                                            })
                                            .await
                                        }
                                    }
                                };
                                transfer_bucket_files(name, code_repository.location().to_string(), items, options, "Copying", operation).await
                            }
                            | Err(why) => Err(why),
                        }
                    }
                    | false => Ok(TransferManifest {
                        bucket: name,
                        repository: code_repository.location().to_string(),
                        files: Vec::new(),
                    }),
                }
            }
            | Err(why) => Err(why),
        }
    }
    /// Download files from bucket to local directory
    ///
    /// Ignores files listed in [`IGNORE`]
    ///
    /// Downloads files concurrently using buffered streams
    pub async fn download_files(self: Bucket, options: &BucketOptions) -> ApiResult<TransferManifest> {
        let BucketOptions { filter, ignore, .. } = options;
        match FilterSet::compile(ignore, filter) {
            | Ok(filters) => {
                let name = self.name.clone();
                let code_repository = self.code_repository.clone();
                match self.file_paths("").await {
                    | Ok(paths) => match TransferItem::collect(filter_paths(paths, &filters), options.flatten) {
                        | Ok(items) => {
                            let repository = code_repository.location().to_string();
                            let clobber = options.clobber;
                            let write_lock = Arc::new(Mutex::new(()));
                            let operation = {
                                let code_repository = Arc::new(code_repository);
                                let output = Arc::new(options.output.clone().unwrap_or_default());
                                let write_lock = Arc::clone(&write_lock);
                                move |item: TransferItem| {
                                    let output = Arc::clone(&output);
                                    let repository = Arc::clone(&code_repository);
                                    let write_lock = Arc::clone(&write_lock);
                                    async move {
                                        let source = item.source.display().to_string();
                                        let bytes = match repository.as_ref() {
                                            | Repository::GitLab { .. } => match (repository.domain(), repository.project_path()) {
                                                | (Some(domain), Some(identifier)) => {
                                                    let options = gitlab::Options::from_env()
                                                        .with_domain(domain)
                                                        .with_identifier(identifier)
                                                        .with_path(source)
                                                        .with_sha("HEAD");
                                                    gitlab::repository_file(&options).await.and_then(|file| file.decoded_content())
                                                }
                                                | _ => Err(eyre!("Failed to build GitLab API request for repository path")),
                                            },
                                            | _ => match repository.raw_url(source) {
                                                | Some(url) => Source::read_bytes(&url, false).await,
                                                | None => Err(eyre!("Failed to build raw URL for repository path")),
                                            },
                                        };
                                        Self::write_file(output.as_path(), &item.destination, clobber, write_lock.as_ref(), || async { bytes }).await
                                    }
                                }
                            };
                            transfer_bucket_files(name, repository, items, options, "Downloading", operation).await
                        }
                        | Err(why) => Err(why),
                    },
                    | Err(why) => {
                        error!("=> {} Get file paths for download — {why}", Label::fail());
                        Err(why)
                    }
                }
            }
            | Err(why) => Err(why),
        }
    }
    async fn file_paths(&self, directory: &str) -> ApiResult<Vec<String>> {
        let code_repository = self.code_repository.clone();
        let bucket_name = self.name.clone().unwrap_or_else(|| "Bucket".to_string()).to_uppercase();
        match &code_repository {
            | Repository::Git { .. } => {
                let path = match code_repository.location().path() {
                    | Some(value) => PathBuf::from(value),
                    | None => return Err(eyre!("Git repository has no local path — cannot list files")),
                };
                Ok(files_all(path, None::<Vec<String>>)
                    .into_iter()
                    .map(|x| x.display().to_string())
                    .collect())
            }
            | Repository::GitHub { location } => match location.path() {
                | Some(path) => {
                    let path = path.trim_start_matches('/').to_string();
                    match self.domain() {
                        | Ok(host) => github::tree_paths(format!("api.{}", host), path, "main")
                            .await
                            .map_err(|why| eyre!("Failed to get file paths for {bucket_name} bucket - {why}")),
                        | Err(why) => Err(why),
                    }
                }
                | None => Err(eyre!("Failed to parse GitHub URI for {bucket_name} bucket")),
            },
            | Repository::GitLab { .. } => match code_repository.id() {
                | Some(id) => match self.domain() {
                    | Ok(host) => {
                        let options = gitlab::Options::from_env().with_domain(host).with_identifier(id).with_path(directory);
                        let mut page = 1_u32;
                        let mut all_paths: Vec<String> = vec![];
                        loop {
                            let page_options = options.clone().with_page(page);
                            match gitlab::tree_paths(&page_options).await {
                                | Ok(response) if response.entry_count == 0 => {
                                    break Ok(all_paths.clone());
                                }
                                | Ok(response) => {
                                    all_paths.extend(response.paths);
                                    page = page.saturating_add(1);
                                }
                                | Err(why) => {
                                    break Err(eyre!("Failed to get file paths for {bucket_name} bucket — {why}"));
                                }
                            }
                        }
                    }
                    | Err(why) => Err(why),
                },
                | None => Err(eyre!("Missing GitLab project id for {bucket_name} bucket")),
            },
            | Repository::HuggingFace { .. } => Err(eyre!("Hugging Face repositories are unsupported for bucket downloads")),
        }
    }
}
impl From<&str> for Bucket {
    fn from(value: &str) -> Self {
        let location = Location::Simple(value.to_string());
        if location.uri().is_none() {
            exit(exitcode::DATAERR);
        }
        let repository = match location.scheme() {
            | Scheme::File => Repository::Git { location },
            | _ => {
                let host = match location.host() {
                    | Some(value) => value.to_lowercase(),
                    | None => {
                        error!(value, "=> {} Parse URI - No host", Label::fail());
                        exit(exitcode::DATAERR);
                    }
                };
                if host.contains("github.com") {
                    Repository::GitHub { location }
                } else {
                    let id = None;
                    Repository::GitLab { id, location }
                }
            }
        };
        Bucket::init().code_repository(repository).build()
    }
}
impl Default for BucketOptions {
    fn default() -> Self {
        Self {
            output: None,
            threads: 10,
            quiet: false,
            ignore: Vec::new(),
            filter: Vec::new(),
            flatten: false,
            clobber: false,
        }
    }
}
impl BucketOptions {
    /// Set the output path
    pub fn with_output(self, output: impl Into<PathBuf>) -> Self {
        Self {
            output: Some(output.into()),
            ..self
        }
    }
}
impl FilterSet {
    /// Compile ignore and filter regex patterns into a filter set
    pub fn compile(ignore: &[String], filter: &[String]) -> ApiResult<Self> {
        let compile = |patterns: &[String]| {
            patterns
                .iter()
                .map(|pattern| Regex::new(pattern).map_err(|why| eyre!("Invalid regex/filter pattern '{pattern}': {why}")))
                .collect::<ApiResult<Vec<Regex>>>()
        };
        compile(ignore).and_then(|ignore| compile(filter).map(|filter| Self { ignore, filter }))
    }
    /// Filter items based on ignore and filter regex patterns
    pub fn filter<T>(
        items: Vec<T>,
        filter: &[String],
        ignore: &[String],
        value: impl Fn(&T) -> String,
        keep: impl Fn(&T) -> bool,
    ) -> ApiResult<Vec<T>> {
        match FilterSet::compile(ignore, filter) {
            | Ok(filters) => Ok(items.into_iter().filter(|item| filters.matches(&value(item)) && keep(item)).collect()),
            | Err(why) => Err(why),
        }
    }
    /// Determine whether a value satisfies the compiled include and ignore patterns
    pub fn matches(&self, value: &str) -> bool {
        let ignored = self.ignore.iter().any(|pattern| pattern.is_match(value).unwrap_or(false));
        let filtered = self.filter.is_empty() || self.filter.iter().any(|pattern| pattern.is_match(value).unwrap_or(false));
        !ignored && filtered
    }
}
impl ModelEntry {
    /// Normalize model entries into unique synchronization requests
    pub fn requests(entries: &[Self]) -> ApiResult<Vec<sync::ModelRequest>> {
        entries
            .iter()
            .map(sync::ModelRequest::try_from)
            .try_fold((HashSet::new(), Vec::new()), |(mut identifiers, mut requests), request| {
                request.and_then(|request| match identifiers.insert(request.id().to_string()) {
                    | true => {
                        requests.push(request);
                        Ok((identifiers, requests))
                    }
                    | false => Err(eyre!("Duplicate generated model ID '{}'", request.id())),
                })
            })
            .map(|(_, requests)| requests)
    }
    /// Resolve model entries from a local models directory, skipping entries that cannot be resolved.
    pub fn resolve(entries: &[Self], options: &sync::ModelRequestOptions<'_>) -> ApiResult<Vec<ModelDetails>> {
        Self::resolve_using(entries, options, false, |_| Vec::new())
    }
    /// Resolve model entries using fallback repository metadata from the local model database.
    pub fn resolve_with_fallbacks(
        entries: &[Self],
        options: &sync::ModelRequestOptions<'_>,
        database_path: Option<PathBuf>,
    ) -> ApiResult<Vec<ModelDetails>> {
        Self::resolve_using(entries, options, true, |model_id| {
            Self::fallback_repositories(model_id, database_path.as_ref())
        })
    }
    fn resolve_using(
        entries: &[Self],
        options: &sync::ModelRequestOptions<'_>,
        fallbacks_enabled: bool,
        fallback: impl Fn(&str) -> Vec<String>,
    ) -> ApiResult<Vec<ModelDetails>> {
        Self::requests(entries).map(|requests| {
            requests
                .into_iter()
                .filter_map(|request| {
                    let id = request.id().to_string();
                    let request_options = sync::ModelRequestOptions {
                        fallbacks: fallback(&id),
                        ..options.clone()
                    };
                    match request.resolve(&request_options) {
                        | Ok(model) => Some(model),
                        | Err(why) => {
                            let reason = Self::resolution_failure_reason(&why, fallbacks_enabled, &request_options.fallbacks);
                            warn!("=> {} Could not resolve {} {}", Label::skip(), id.yellow(), reason.dimmed());
                            None
                        }
                    }
                })
                .collect()
        })
    }
    fn resolution_failure_reason(why: &impl fmt::Display, fallbacks_enabled: bool, fallbacks: &[String]) -> String {
        match (fallbacks_enabled, fallbacks.is_empty()) {
            | (true, true) => format!("({why}; no fallback repositories found in the local model database)"),
            | _ => format!("({why})"),
        }
    }
    fn fallback_repositories(model_id: &str, database_path: Option<&PathBuf>) -> Vec<String> {
        resolve_database_path(database_path)
            .ok()
            .filter(|path| path.is_file())
            .and_then(|path| {
                ModelRow::init()
                    .model_id(model_id.to_string())
                    .build()
                    .select(Some(path), |row| row.model_id.as_deref() == Some(model_id))
                    .ok()
                    .flatten()
            })
            .and_then(|row| row.parsed_weights())
            .map(|weights| weights.groups().0.into_iter().map(|group| group.repository).unique().collect())
            .unwrap_or_default()
    }
}
impl fmt::Display for RunnerType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            | RunnerType::Group => "group",
            | RunnerType::Instance => "instance",
            | RunnerType::Project => "project",
        };
        formatter.write_str(value)
    }
}
impl RunnerDetails {
    /// Set the runner identifier (assigned by GitLab after creation)
    pub fn with_id(self, value: u64) -> Self {
        Self {
            identifier: Some(value),
            ..self
        }
    }
    /// Set the runner name (used as Docker container name)
    pub fn with_name(self, value: String) -> Self {
        Self { name: Some(value), ..self }
    }
    /// Set the runner authentication token
    pub fn with_token(self, value: Option<String>) -> Self {
        Self { token: value, ..self }
    }
}
impl From<&str> for RunnerType {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            | "INSTANCE" => RunnerType::Instance,
            | "PROJECT" => RunnerType::Project,
            | _ => RunnerType::Group,
        }
    }
}
impl From<String> for RunnerType {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}
impl TransferManifest {
    /// Return the legacy reported count of transferred JSON and image files.
    pub fn count(&self) -> usize {
        let paths = self.files.iter().map(|path| path.display().to_string()).collect::<Vec<_>>();
        count_json_files(&paths).saturating_add(count_image_files(&paths))
    }
    /// Ingest transferred RAD files into the canonical research activity table.
    pub fn ingest(&self, options: &BucketOptions, database_path: &Option<PathBuf>, no_local_database: bool) -> ApiResult<()> {
        match no_local_database {
            | true => Ok(()),
            | false => {
                let output = options.output.clone().unwrap_or_default();
                let database = Database::<Table>::from_path(database_path.clone());
                self.files
                    .iter()
                    .filter(is_filetype(SUPPORTED_RAD_FILETYPES))
                    .filter(|relative| {
                        let path = output.join(relative);
                        MimeType::from_path(&path) != MimeType::Markdown || ResearchActivity::is_markdown(path.as_path())
                    })
                    .try_fold((), |(), relative| {
                        let path = output.join(relative);
                        ResearchActivity::read(path.clone())
                            .and_then(|rad| {
                                serde_json::to_value(&rad)
                                    .map_err(Report::from)
                                    .and_then(|rad_json| database.create_or_enrich(self.candidate(&rad, rad_json, relative)).map(|_| ()))
                            })
                            .map_err(|why| eyre!("Failed to ingest transferred RAD {} — {why}", path.display()))
                    })
            }
        }
    }
    fn candidate(&self, rad: &ResearchActivity, rad_json: serde_json::Value, relative: &Path) -> ResearchActivityCandidate {
        let pairs = [
            (PID::DOI, rad.meta.doi.as_ref()),
            (PID::ISBN, rad.meta.books.as_ref()),
            (PID::Patent, rad.meta.patents.as_ref()),
            (PID::RAID, rad.meta.raid.as_ref()),
        ];
        let pid_keys = pairs.into_iter().flat_map(|(kind, values)| {
            values.into_iter().flatten().filter_map(move |value| {
                Identifier::init()
                    .kind(kind.clone())
                    .value(value)
                    .build()
                    .normalized()
                    .map(|identifier| format!("{}:{}", identifier.kind.as_str(), identifier.value))
            })
        });
        let rad_key = format!("rad:{}:{}", self.repository, rad.meta.identifier);
        let prov = Provenance::Bucket {
            bucket: self.bucket.clone(),
            repository: self.repository.clone(),
            relative_path: relative.display().to_string(),
            observed_at: Timestamp::now().to_string(),
        };
        ResearchActivityCandidate::new(
            rad_json,
            pid_keys.chain(once(rad_key)).collect(),
            vec![serde_json::to_value(prov).unwrap_or_default()],
        )
    }
}
fn count_json_files(paths: &[String]) -> usize {
    paths.iter().filter(|&path| path.to_lowercase().ends_with(".json")).count()
}
fn count_image_files(paths: &[String]) -> usize {
    paths.iter().filter(|&x| has_image_extension(x)).count()
}
fn default_docker_image() -> String {
    "gitlab/gitlab-runner:latest".to_string()
}
fn default_executor() -> Executor {
    Executor::Docker
}
fn filter_paths(paths: Vec<String>, filters: &FilterSet) -> Vec<String> {
    paths
        .into_iter()
        .filter(|path| !is_ignored_path(path, &filters.ignore) && is_filtered_path(path, &filters.filter))
        .collect()
}
#[allow(clippy::ptr_arg)]
fn has_image_extension(path: &String) -> bool {
    path.to_lowercase().ends_with(".png") || path.to_lowercase().ends_with(".jpg")
}
fn is_ignored_path(path: &str, ignore: &[Regex]) -> bool {
    let is_builtin_ignored = IGNORE.iter().any(|value| path.ends_with(value));
    let is_regex_ignored = ignore.iter().any(|pattern| pattern.is_match(path).unwrap_or(false));
    is_builtin_ignored || is_regex_ignored
}
fn is_filtered_path(path: &str, filter: &[Regex]) -> bool {
    filter.is_empty() || filter.iter().any(|pattern| pattern.is_match(path).unwrap_or(false))
}
fn operations_complete_message(name: Option<String>, json_count: usize, image_count: usize) -> String {
    let total = json_count.saturating_add(image_count);
    let message = if json_count != image_count {
        let recommendation = if json_count > image_count {
            "Do you need to add some images?"
        } else {
            "Do you need to add some JSON files?"
        };
        format!(
            " ({} data file{}, {} image{} - {})",
            json_count.yellow(),
            suffix(json_count),
            image_count.yellow(),
            suffix(image_count),
            recommendation.italic(),
        )
    } else {
        "".to_string()
    };
    let bucket_description = match name {
        | Some(value) => format!("{} bucket", value.to_uppercase().cyan()),
        | None => "<URL>".cyan().to_string(),
    };
    format!(
        "{}Obtained {} file{} from {bucket_description}{}",
        if total > 0 { Label::CHECKMARK } else { Label::CAUTION },
        if total > 0 {
            total.green().to_string()
        } else {
            total.yellow().to_string()
        },
        suffix(total),
        message,
    )
}
async fn transfer_bucket_files<F, Fut>(
    name: Option<String>,
    repository: String,
    items: Vec<TransferItem>,
    options: &BucketOptions,
    verb: &'static str,
    operation: F,
) -> ApiResult<TransferManifest>
where
    F: Fn(TransferItem) -> Fut,
    Fut: Future<Output = ApiResult<()>>,
{
    let BucketOptions { threads, quiet, .. } = options;
    let source_paths = items.iter().map(|item| item.source.display().to_string()).collect::<Vec<_>>();
    let total_data = count_json_files(&source_paths);
    let total_images = count_image_files(&source_paths);
    let message = move |item: &TransferItem| format!("{verb} {}", item.source.display());
    let finish_name = name.clone();
    let finish_message = |_| operations_complete_message(finish_name, total_data, total_images);
    let progress_type = match quiet {
        | true => ProgressType::Silent,
        | false => ProgressType::Bar,
    };
    let files = items.iter().map(|item| item.destination.clone()).collect::<Vec<_>>();
    with_progress(items, message, operation, finish_message, Some(*threads), progress_type)
        .await
        .map(|_| TransferManifest {
            bucket: name,
            repository,
            files,
        })
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
    use crate::prelude::{create_dir_all, read_to_string, remove_dir_all, write};

    #[test]
    fn test_resolution_failure_reason_reports_fallback_lookup_status() {
        assert_eq!(
            ModelEntry::resolution_failure_reason(&"missing", true, &[]),
            "(missing; no fallback repositories found in the local model database)"
        );
        assert_eq!(ModelEntry::resolution_failure_reason(&"missing", false, &[]), "(missing)");
        assert_eq!(
            ModelEntry::resolution_failure_reason(&"missing", true, &["fallback/model".to_string()]),
            "(missing)"
        );
    }

    fn temp_resolve_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(core::time::Duration::from_nanos(0))
            .as_nanos();
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("target")
            .join("test_artifacts")
            .join(format!("{name}-{nanos}"))
    }

    #[test]
    fn test_load_rejects_missing_explicit_path() {
        let missing = temp_resolve_dir("load-missing").join("missing.json");
        let result = ApplicationConfiguration::load(&Some(missing.clone()));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            format!("Configuration file does not exist — {}", missing.display())
        );
    }
    #[test]
    fn test_with_models_keeps_unique_identifiers() {
        let configuration = ApplicationConfiguration::parse(r#"{"models":["acme/existing","acme/existing"]}"#).unwrap();
        let entries = vec![
            ModelEntry::Selector("acme/existing".to_string()),
            ModelEntry::Selector("acme/added".to_string()),
            ModelEntry::Selector("acme/added".to_string()),
        ];
        let updated = configuration.with_models(&entries).unwrap();
        let identifiers = updated
            .models
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| match entry {
                | ModelEntry::Selector(identifier) => Some(identifier),
                | ModelEntry::Entry(_) => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(identifiers, vec!["acme/existing", "acme/added"]);
    }
    #[test]
    fn test_model_update_preserves_jsonc_comments_and_dry_run() {
        let directory = temp_resolve_dir("sync-acorn-config");
        create_dir_all(&directory).unwrap();
        let path = directory.join("config.jsonc");
        let before = "{\n  // Keep this comment\n  \"models\": [\"acme/existing\"]\n}\n";
        write(&path, before).unwrap();
        let configuration = ApplicationConfiguration::read(path.clone()).unwrap();
        let entries = vec![ModelEntry::Selector("acme/added".to_string())];
        let updated = configuration.with_models(&entries).unwrap();
        updated.write_or_preview(&path, true, true).unwrap();
        assert_eq!(read_file(&path).unwrap(), before);
        updated.write_or_preview(&path, false, true).unwrap();
        let content = read_file(&path).unwrap();
        assert_eq!(
            content,
            "{\n  // Keep this comment\n  \"models\": [\n    \"acme/existing\",\n    \"acme/added\"\n  ]\n}\n"
        );
        let _ = remove_dir_all(directory);
    }
    #[test]
    fn test_resolve_returns_explicit_existing_path() {
        let directory = temp_resolve_dir("resolve-explicit");
        create_dir_all(&directory).unwrap();
        let directory = directory.canonicalize().unwrap();
        let provided = directory.join("config.yaml");
        let default = directory.join(".acorn.json");
        write(&provided, "{}\n").unwrap();
        write(&default, "{}\n").unwrap();
        let resolved = ApplicationConfiguration::resolve(&Some(provided.clone()));
        assert_eq!(resolved, Some(provided));
        let _ = remove_dir_all(directory);
    }
    #[test]
    fn test_resolve_falls_back_to_default_when_provided_path_missing() {
        let directory = temp_resolve_dir("resolve-fallback");
        create_dir_all(&directory).unwrap();
        let directory = directory.canonicalize().unwrap();
        let default = directory.join(".acorn.yml");
        write(&default, "{}\n").unwrap();
        let resolved = ApplicationConfiguration::resolve_in(&Some(directory.join("missing.json")), &directory);
        assert_eq!(resolved, Some(default));
        let _ = remove_dir_all(directory);
    }
    #[test]
    fn test_extensionless_config_is_last_and_read_as_jsonc() {
        assert_eq!(DEFAULT_CONFIG_FILENAMES.last(), Some(&".acorn"));
        let directory = temp_resolve_dir("extensionless-jsonc");
        create_dir_all(&directory).unwrap();
        let directory = directory.canonicalize().unwrap();
        let extensionless = directory.join(".acorn");
        write(&extensionless, "{\n  // Comment\n}\n").unwrap();
        let config = ApplicationConfiguration::read(extensionless.clone()).unwrap();
        assert!(config.write(extensionless).is_ok());
        let _ = remove_dir_all(directory);
    }
    #[test]
    fn test_count_image_files_counts_supported_extensions() {
        let paths = vec![
            "content/plot.png".to_string(),
            "content/photo.jpg".to_string(),
            "content/photo.jpeg".to_string(),
            "content/index.json".to_string(),
        ];
        assert_eq!(count_image_files(&paths), 2);
    }
    #[test]
    fn test_count_json_files_counts_case_insensitive_json_paths() {
        let paths = vec![
            "content/index.json".to_string(),
            "content/README.md".to_string(),
            "content/data.JSON".to_string(),
        ];
        assert_eq!(count_json_files(&paths), 2);
    }
    #[test]
    fn test_has_image_extension_matches_png_and_jpg() {
        assert!(has_image_extension(&"image.png".to_string()));
        assert!(has_image_extension(&"photo.JPG".to_string()));
        assert!(!has_image_extension(&"graphic.jpeg".to_string()));
    }
    #[test]
    fn test_is_ignored_path() {
        let ignore = FilterSet::compile(&[r"\.jpeg$".to_string(), r"notes\.txt$".to_string()], &[])
            .unwrap()
            .ignore;
        assert!(is_ignored_path("/tmp/photo.jpeg", &ignore));
        assert!(is_ignored_path("/tmp/notes.txt", &ignore));
        assert!(!is_ignored_path("/tmp/index.json", &ignore));
        let invalid = FilterSet::compile(&["[".to_string()], &[]);
        assert!(invalid.is_err());
        let ignore: Vec<Regex> = vec![];
        assert!(is_ignored_path("/tmp/README.md", &ignore));
    }
    #[test]
    fn test_is_filtered_path() {
        let filter = FilterSet::compile(&[], &[r"\.json$".to_string(), r"img/".to_string()]).unwrap().filter;
        assert!(is_filtered_path("/tmp/data.json", &filter));
        assert!(is_filtered_path("/tmp/img/photo.jpg", &filter));
        assert!(!is_filtered_path("/tmp/README.md", &filter));
        let invalid = FilterSet::compile(&[], &["[".to_string()]);
        assert!(invalid.is_err());
        let filter: Vec<Regex> = vec![];
        assert!(is_filtered_path("/tmp/README.md", &filter));
    }
    #[test]
    fn test_transfer_item_collect_preserves_or_flattens_paths() {
        let paths = vec!["docs/quest/index.json".to_string(), "docs/quest/image.png".to_string()];
        let preserved = TransferItem::collect(paths.clone(), false).unwrap();
        let flattened = TransferItem::collect(paths, true).unwrap();
        assert_eq!(
            preserved.iter().map(|item| item.destination.clone()).collect::<Vec<_>>(),
            vec![PathBuf::from("docs/quest/index.json"), PathBuf::from("docs/quest/image.png")]
        );
        assert_eq!(
            flattened.iter().map(|item| item.destination.clone()).collect::<Vec<_>>(),
            vec![PathBuf::from("index.json"), PathBuf::from("image.png")]
        );
    }
    #[test]
    fn test_transfer_item_collect_rejects_flattened_filename_collisions() {
        let result = TransferItem::collect(vec!["one/index.json".to_string(), "two/index.json".to_string()], true);
        let message = result.unwrap_err().to_string();
        assert!(message.contains("index.json"));
        assert!(message.contains("one/index.json"));
        assert!(message.contains("two/index.json"));
    }
    #[test]
    fn test_transfer_item_collect_rejects_unsafe_destination() {
        let result = TransferItem::collect(vec!["../outside.json".to_string()], false);
        assert!(result.unwrap_err().to_string().contains("unsafe"));
    }
    #[tokio::test]
    async fn test_copy_files_flattens_destinations_and_manifest() {
        let source = temp_resolve_dir("flatten-source");
        let output = temp_resolve_dir("flatten-output");
        create_dir_all(source.join("docs/quest")).unwrap();
        write(source.join("docs/quest/index.json"), "{}").unwrap();
        write(source.join("docs/quest/image.png"), "image").unwrap();
        let bucket = Bucket::init()
            .code_repository(Repository::Git {
                location: Location::Simple(format!("file:{}", source.display())),
            })
            .build();
        let options = BucketOptions::init().output(output.clone()).quiet(true).flatten(true).build();
        let manifest = bucket.copy_files(&options).await.unwrap();
        assert_eq!(manifest.files.len(), 2);
        assert!(manifest.files.contains(&PathBuf::from("index.json")));
        assert!(manifest.files.contains(&PathBuf::from("image.png")));
        assert!(output.join("index.json").is_file());
        assert!(output.join("image.png").is_file());
        assert!(!output.join("docs").exists());
        let _ = remove_dir_all(source);
        let _ = remove_dir_all(output);
    }
    #[tokio::test]
    async fn test_copy_files_preserves_existing_destination_without_clobber() {
        let source = temp_resolve_dir("clobber-disabled-source");
        let output = temp_resolve_dir("clobber-disabled-output");
        create_dir_all(&source).unwrap();
        create_dir_all(&output).unwrap();
        write(source.join("index.json"), "new").unwrap();
        write(output.join("index.json"), "old").unwrap();
        let bucket = Bucket::init()
            .code_repository(Repository::Git {
                location: Location::Simple(format!("file:{}", source.display())),
            })
            .build();
        let options = BucketOptions::init().output(output.clone()).quiet(true).build();
        assert!(bucket.copy_files(&options).await.is_err());
        assert_eq!(read_to_string(output.join("index.json")).unwrap(), "old");
        let _ = remove_dir_all(source);
        let _ = remove_dir_all(output);
    }
    #[tokio::test]
    async fn test_copy_files_clobbers_selected_path_conflicts_only() {
        let source = temp_resolve_dir("clobber-enabled-source");
        let output = temp_resolve_dir("clobber-enabled-output");
        create_dir_all(source.join("nested")).unwrap();
        create_dir_all(output.join("directory.json")).unwrap();
        write(source.join("existing.json"), "new file").unwrap();
        write(source.join("directory.json"), "new directory replacement").unwrap();
        write(source.join("nested/index.json"), "new nested file").unwrap();
        write(source.join("nested/other.json"), "new sibling file").unwrap();
        write(output.join("existing.json"), "old file").unwrap();
        write(output.join("directory.json/old.json"), "old directory content").unwrap();
        write(output.join("nested"), "old parent file").unwrap();
        write(output.join("unrelated.json"), "keep").unwrap();
        let bucket = Bucket::init()
            .code_repository(Repository::Git {
                location: Location::Simple(format!("file:{}", source.display())),
            })
            .build();
        let options = BucketOptions::init().output(output.clone()).quiet(true).clobber(true).build();
        let result = bucket.copy_files(&options).await;
        assert!(result.is_ok());
        assert_eq!(read_to_string(output.join("existing.json")).unwrap(), "new file");
        assert_eq!(read_to_string(output.join("directory.json")).unwrap(), "new directory replacement");
        assert_eq!(read_to_string(output.join("nested/index.json")).unwrap(), "new nested file");
        assert_eq!(read_to_string(output.join("nested/other.json")).unwrap(), "new sibling file");
        assert_eq!(read_to_string(output.join("unrelated.json")).unwrap(), "keep");
        let _ = remove_dir_all(source);
        let _ = remove_dir_all(output);
    }
    #[tokio::test]
    async fn test_clobber_preserves_destination_when_source_read_fails() {
        let output = temp_resolve_dir("clobber-source-failure-output");
        create_dir_all(&output).unwrap();
        write(output.join("index.json"), "old").unwrap();
        let write_lock = Mutex::new(());
        let result = Bucket::write_file(&output, Path::new("index.json"), true, &write_lock, || async {
            Err::<Vec<u8>, Report>(eyre!("source failure"))
        })
        .await;
        assert!(result.is_err());
        assert_eq!(read_to_string(output.join("index.json")).unwrap(), "old");
        let _ = remove_dir_all(output);
    }
    #[cfg(unix)]
    #[test]
    fn test_prepare_destination_replaces_parent_symlink_without_following_it() {
        let output = temp_resolve_dir("clobber-symlink-output");
        let external = temp_resolve_dir("clobber-symlink-external");
        create_dir_all(&output).unwrap();
        create_dir_all(&external).unwrap();
        write(external.join("index.json"), "outside").unwrap();
        crate::prelude::symlink(&external, output.join("linked")).unwrap();
        let target = Bucket::prepare_destination(&output, Path::new("linked/index.json")).unwrap();
        assert_eq!(target, output.join("linked/index.json"));
        assert!(output.join("linked").is_dir());
        assert!(!output.join("linked").symlink_metadata().unwrap().file_type().is_symlink());
        assert_eq!(read_to_string(external.join("index.json")).unwrap(), "outside");
        let _ = remove_dir_all(output);
        let _ = remove_dir_all(external);
    }
    #[test]
    fn test_operations_complete_message_includes_bucket_name_and_guidance() {
        let message = operations_complete_message(Some("acorn".to_string()), 2, 1);
        assert!(message.contains("Obtained"));
        assert!(message.contains("ACORN"));
        assert!(message.contains(" bucket"));
        assert!(message.contains("data file"));
        assert!(message.contains("image"));
        assert!(message.contains("Do you need to add some images?"));
    }
    #[test]
    fn test_operations_complete_message_uses_url_placeholder_without_name() {
        let message = operations_complete_message(None, 0, 0);
        assert!(message.contains("Obtained"));
        assert!(message.contains("<URL>"));
    }
    #[test]
    fn test_parse_supports_yaml_flow_mapping_when_json_detection_fails() {
        let content = "{endpoints: []}";
        let result = ApplicationConfiguration::parse(content);
        assert!(result.is_ok());
    }
}
