//! Application configuration functions and data structures
//!
//! ACORN is configured by a JSON, JSONC, or YAML file (typically named `.acorn.json`)
//!
//! The ACORN configuration file configures what buckets should be downloaded, readability analysis, <span title="Large Language Model">LLM</span> settings, and more.
//!
use crate::io::api::{github, gitlab, Configuration, Endpoint};
use crate::io::database::schema::ModelRow;
use crate::io::database::{resolve_database_path, Row};
use crate::io::{
    files_all, parse_jsonc_cst, read_file, sync, with_progress, write_file, write_file_bytes, ApiResult, CstRootNode, CstValue, Executor, FromPath,
    InputOutput, ProgressType, Source,
};
use crate::prelude::{self, env, exit, Arc, HashSet, Path, PathBuf};
use crate::schema::OneOrMany;
use crate::schema::{
    agent::{ModelDetails, Quantization},
    hardware::memory::Memory,
};
use crate::util::constants::app::{DEFAULT_CONFIG_FILENAMES, IGNORE};
use crate::util::{detect_json, suffix, text_diff_changes_with_color, Label, MimeType, StringConversion};
use crate::{Location, Repository, Scheme};
use bon::Builder;
use color_eyre::eyre::eyre;
use core::fmt::{self, Debug};
use core::future::Future;
use derive_more::Display;
use fancy_regex::Regex;
use itertools::Itertools;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
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
    /// Copy files from (local) bucket to local directory
    /// ### Notes
    /// - Ignores files listed in [`IGNORE`]
    /// - Only copies files from local repositories
    pub async fn copy_files(self: Bucket, options: &BucketOptions) -> ApiResult<usize> {
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
                            files_all(PathBuf::from(&bucket_root), None)
                                .into_iter()
                                .map(|x| x.display().to_string())
                                .collect::<Vec<String>>(),
                            &filters,
                        )
                        .into_iter()
                        .filter(|path| PathBuf::from(path).is_file())
                        .collect::<Vec<String>>();
                        let bucket_root = Arc::new(bucket_root);
                        let operation = {
                            let bucket_root = Arc::clone(&bucket_root);
                            let output = Arc::clone(&output);
                            move |path: String| {
                                let bucket_root = Arc::clone(&bucket_root);
                                let output = Arc::clone(&output);
                                async move {
                                    let path_buf = PathBuf::from(&path);
                                    match path_buf.strip_prefix(PathBuf::from(bucket_root.as_str())) {
                                        | Ok(relative) => write_file_bytes(output.join(relative), || async { prelude::read(path.clone()) }).await,
                                        | Err(_) => Err(eyre!("Failed to strip prefix {bucket_root} from {path}")),
                                    }
                                }
                            }
                        };
                        transfer_bucket_files(name, items, options, "Copying", operation).await
                    }
                    | false => Ok(0),
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
    pub async fn download_files(self: Bucket, options: &BucketOptions) -> ApiResult<usize> {
        let BucketOptions { filter, ignore, .. } = options;
        match FilterSet::compile(ignore, filter) {
            | Ok(filters) => {
                let name = self.name.clone();
                let code_repository = self.code_repository.clone();
                match self.file_paths("").await {
                    | Ok(paths) => {
                        let items = filter_paths(paths, &filters);
                        let operation = {
                            let code_repository = Arc::new(code_repository);
                            let output = Arc::new(options.output.clone().unwrap_or_default());
                            move |path: String| {
                                let output = Arc::clone(&output);
                                let repository = Arc::clone(&code_repository);
                                async move {
                                    match repository.raw_url(path.clone()) {
                                        | Some(url) => {
                                            let output_filepath = output.join(path);
                                            write_file_bytes(output_filepath, || async move { Source::read_bytes(&url, false).await }).await
                                        }
                                        | None => Err(eyre!("Failed to build raw URL for repository path")),
                                    }
                                }
                            }
                        };
                        transfer_bucket_files(name, items, options, "Downloading", operation).await
                    }
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
                Ok(files_all(path, None).into_iter().map(|x| x.display().to_string()).collect())
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
                                | Ok(response) if response.paths.is_empty() => {
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
            | Ok(FilterSet { ignore, filter }) => Ok(items
                .into_iter()
                .filter(|item| {
                    let value = value(item);
                    let ignored = ignore.iter().any(|pattern| pattern.is_match(&value).unwrap_or(false));
                    let filtered = filter.is_empty() || filter.iter().any(|pattern| pattern.is_match(&value).unwrap_or(false));
                    !ignored && filtered && keep(item)
                })
                .collect()),
            | Err(why) => Err(why),
        }
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
    items: Vec<String>,
    options: &BucketOptions,
    verb: &'static str,
    operation: F,
) -> ApiResult<usize>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = ApiResult<()>>,
{
    let BucketOptions { threads, quiet, .. } = options;
    let total_data = count_json_files(&items);
    let total_images = count_image_files(&items);
    let total = total_data.saturating_add(total_images);
    let message = move |path: &String| format!("{verb} {path}");
    let finish_message = |_| operations_complete_message(name, total_data, total_images);
    let progress_type = match quiet {
        | true => ProgressType::Silent,
        | false => ProgressType::Bar,
    };
    with_progress(items, message, operation, finish_message, Some(*threads), progress_type)
        .await
        .map(|_| total)
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
            .join("..")
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
