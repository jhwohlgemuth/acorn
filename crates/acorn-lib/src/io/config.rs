//! Application configuration functions and data structures
//!
//! ACORN is configured by a JSON, JSONC, or YAML file (typically named `.acorn.json`)
//!
//! The ACORN configuration file configures what buckets should be downloaded, readability analysis, <span title="Large Language Model">LLM</span> settings, and more.
//!
use crate::io::api::{github, gitlab, Configuration, Endpoint};
use crate::io::bagit::Bag;
use crate::io::database::schema::{ModelRow, Table};
use crate::io::database::{resolve_database_path, Database, Provenance, ResearchActivityCandidate, Row};
use crate::io::download::{DownloadItem, DownloadTask};
use crate::io::http::{self, download_with_progress};
use crate::io::{
    files_all, first_env_var, parse_jsonc_cst, read_file, sync, uri_to_path, with_progress, write_file, write_file_bytes, ApiResult,
    ArchiveCandidate, CstRootNode, CstValue, Executor, FromPath, InputOutput, ProgressType, TemporaryDirectory,
};
use crate::prelude::{self, env, exit, Arc, ErrorKind, HashMap, HashSet, Mutex, Path, PathBuf};
use crate::schema::{
    agent::{ModelDetails, Quantization},
    hardware::memory::Memory,
    pid::{Identifier, PID},
    research_activity::ResearchActivity,
    Migration, OneOrMany,
};
use crate::util::constants::app::{DEFAULT_CONFIG_FILENAMES, GITLAB_PRIVATE_TOKEN_HEADER, IGNORE, SUPPORTED_RAD_FILETYPES};
use crate::util::constants::env::GITLAB_TOKEN_VARIABLE_NAMES;
use crate::util::{detect_json, is_filetype, suffix, text_diff_changes_with_color, Label, MimeType, StringConversion};
use crate::validation::{Validate, ValidationError, ValidationReport};
use crate::{Location, Repository, Scheme};
use bon::Builder;
use color_eyre::eyre::{eyre, Report};
use core::fmt::{self, Debug};
use core::future::Future;
use core::iter::once;
use derive_more::Display;
use fancy_regex::Regex;
use futures::{future, stream, StreamExt, TryStreamExt};
use itertools::Itertools;
use jiff::Timestamp;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::path::Component;
use strum::EnumIs;
use tracing::{error, info, warn};

/// Authentication requirement for a configured model download source
#[derive(Clone, Debug, Default, Display, EnumIs, Eq, PartialEq, Serialize, Deserialize)]
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
/// Registry-specific metadata provider for an OCI registry profile.
#[derive(Clone, Debug, Default, Display, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegistryKind {
    /// Generic OCI Distribution-compatible registry.
    #[default]
    OCI,
    /// Harbor registry with optional v2 API enrichment.
    Harbor,
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
/// Policy controlling which bucket sources a transfer accepts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferPolicy {
    /// Require a remote repository source.
    Download,
    /// Permit local and remote sources.
    Import,
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
    /// Named OCI registry profiles referenced by model entries.
    #[eserde(compat)]
    pub registries: Option<HashMap<String, RegistryProfile>>,
    /// List of runners
    #[eserde(compat)]
    pub runners: Option<Vec<RunnerDetails>>,
    /// Lookup object for whitelisted downloadable items
    #[eserde(compat)]
    pub whitelist: Option<WhitelistLookup>,
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
/// Options for bucket transfers
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init)]
pub struct BucketOptions {
    /// Path to output directory
    pub output: Option<PathBuf>,
    /// Optional path to the activity database
    pub database_path: Option<PathBuf>,
    /// Disable ingestion into the local activity database
    #[builder(default)]
    pub no_local_database: bool,
    /// Reject deprecated research activity fields during ingestion
    #[builder(default)]
    pub strict: bool,
    /// Rewrite deprecated research activity fields before ingestion
    #[builder(default)]
    pub canonicalize: bool,
    /// Disable remote bucket operations
    #[builder(default)]
    pub offline: bool,
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
/// A collection of buckets that can be transferred together.
#[derive(Clone, Debug, Default)]
pub struct Buckets(Vec<Bucket>);
/// Struct for filtering files based on regex patterns
#[derive(Clone, Debug)]
pub struct FilterSet {
    /// Regex patterns to ignore
    pub ignore: Vec<Regex>,
    /// Regex patterns to include
    pub filter: Vec<Regex>,
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
/// Named OCI registry configuration without embedded credentials.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegistryProfile {
    /// Registry implementation used for optional metadata enrichment.
    #[serde(default)]
    pub kind: RegistryKind,
    /// HTTPS registry/API endpoint.
    pub endpoint: String,
    /// Environment variable containing a token or password.
    pub credential_env: Option<String>,
    /// Username paired with the credential environment variable.
    pub username: Option<String>,
    /// Docker-compatible registry configuration path.
    pub registry_config: Option<PathBuf>,
    /// Custom certificate-authority path.
    pub ca_file: Option<PathBuf>,
    /// Client certificate path.
    pub client_cert: Option<PathBuf>,
    /// Client private-key path.
    pub client_key: Option<PathBuf>,
    /// Permit development-only plain HTTP.
    #[serde(default)]
    pub plain_http: bool,
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TransferItem {
    pub(super) source: PathBuf,
    pub(super) destination: PathBuf,
}
pub(super) struct DownloadOptions<'a> {
    pub(super) repository: &'a Repository,
    pub(super) output: &'a Path,
    pub(super) source: &'a Path,
    pub(super) destination: &'a Path,
    pub(super) branch: Option<&'a str>,
    pub(super) clobber: bool,
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
    /// Return configured OCI registry profiles after validating their public settings.
    pub fn registry_lookup(&self) -> ApiResult<HashMap<String, RegistryProfile>> {
        self.registries
            .clone()
            .unwrap_or_default()
            .into_iter()
            .try_fold(HashMap::new(), |mut profiles, (name, profile)| {
                let valid_characters = name.chars().all(|x| x.is_ascii_alphanumeric() || matches!(x, '-' | '_' | '.'));
                let is_valid_name = !name.is_empty() && valid_characters;
                match is_valid_name {
                    | false => Err(eyre!("Invalid OCI registry profile name '{name}'")),
                    | true => profile
                        .validate()
                        .map_err(|why| eyre!("Invalid OCI registry profile '{name}' — {why}"))
                        .map(|()| {
                            profiles.insert(name, profile);
                            profiles
                        }),
                }
            })
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
    pub(super) fn with_models(&self, entries: &[ModelEntry]) -> ApiResult<Self> {
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
    pub(super) fn write_or_preview(&self, path: &Path, dry_run: bool, no_color: bool) -> ApiResult<()> {
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
impl Bucket {
    /// Resolves and validates a BagIt archive as a local payload bucket.
    pub async fn archive(source: &str, format: Option<MimeType>, temporary: &TemporaryDirectory, offline: bool) -> ApiResult<(Self, String)> {
        let local = uri_to_path(source);
        let archive_path = match local.is_file() {
            | true => Ok(local),
            | false if offline => Err(eyre!("Offline mode cannot import remote archive: {source}")),
            | false => {
                let path = temporary.path().join("archive");
                download_with_progress(source, &path, |_, _| {}, None, None, None, None)
                    .await
                    .map(|_| path)
            }
        };
        archive_path
            .and_then(|path| ArchiveCandidate::from(path).extract(Some(temporary.path().join("extracted")), format))
            .and_then(|root| bag_root(&root))
            .and_then(|root| Bag::verify(&root).map(|()| root.join("data")))
            .map(|payload| (Self::from(payload.as_path()), source.to_string()))
    }
    /// Transfers this bucket and ingests supported research activity data.
    pub async fn transfer(self, options: &BucketOptions, policy: TransferPolicy, provenance: Option<&str>) -> ApiResult<usize> {
        let local = self.code_repository.is_local();
        match (policy, local, options.offline) {
            | (TransferPolicy::Download, true, _) => Err(eyre!("Bucket download requires a remote repository source")),
            | (_, false, true) => Err(eyre!("Offline mode cannot transfer remote bucket sources")),
            | _ => match local {
                | true => self.copy_files(options).await,
                | false => self.download_files(options).await,
            }
            .and_then(|manifest| {
                let manifest = match provenance {
                    | Some(source) => TransferManifest {
                        repository: source.to_string(),
                        ..manifest
                    },
                    | None => manifest,
                };
                let count = manifest.count();
                manifest
                    .canonicalize(options)
                    .and_then(|manifest| manifest.ingest(options))
                    .map(|()| count)
            }),
        }
    }
    /// Imports this bucket and ingests supported research activity data.
    pub async fn import(self, options: &BucketOptions, provenance: Option<&str>) -> ApiResult<usize> {
        self.transfer(options, TransferPolicy::Import, provenance).await
    }
    /// Get hosting domain from bucket struct
    pub(crate) fn domain(&self) -> ApiResult<String> {
        let location = match &self.code_repository {
            | Repository::GitHub { location } | Repository::GitLab { location, .. } => Ok(location),
            | Repository::Git { .. } => Err(eyre!("Domain is unsupported for generic Git repositories")),
            | Repository::HuggingFace { .. } => Err(eyre!("Domain is unsupported for Hugging Face repositories")),
            | Repository::OCI { .. } => Err(eyre!("Domain is unsupported for OCI artifacts used as buckets")),
        };
        location.and_then(|location| match location.scheme() {
            | Scheme::HTTPS => location.host().ok_or_else(|| eyre!("Failed to parse repository host from URI")),
            | _ => Err(eyre!("Unsupported repository URI scheme")),
        })
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
    pub(super) fn prepare_destination(output: &Path, destination: &Path) -> ApiResult<PathBuf> {
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
    pub(super) async fn write_file<F, Fut, E>(output: &Path, destination: &Path, clobber: bool, write_lock: &Mutex<()>, get_bytes: F) -> ApiResult<()>
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
                        let bucket_root = match code_repository.location().local_path() {
                            | Some(value) => value.to_absolute_path(),
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
    /// Downloads files concurrently using streaming, resumable transfers for remote
    /// repositories. GitLab buckets stream from the repository-files raw API on the resolved
    /// default branch, falling back to the buffered repository-file API when streaming fails.
    pub async fn download_files(self: Bucket, options: &BucketOptions) -> ApiResult<TransferManifest> {
        let BucketOptions { filter, ignore, .. } = options;
        match FilterSet::compile(ignore, filter) {
            | Ok(filters) => match self.file_paths("").await {
                | Ok(paths) => match TransferItem::collect(filter_paths(paths, &filters), options.flatten) {
                    | Ok(items) => {
                        let clobber = options.clobber;
                        match self.resolve_branch().await {
                            | Ok(branch) => {
                                let repository_location = self.code_repository.location().to_string();
                                let operation = {
                                    let output = Arc::new(options.output.clone().unwrap_or_default());
                                    let repository = Arc::new(self.code_repository);
                                    let branch = Arc::new(branch);
                                    move |item: TransferItem| {
                                        let output = Arc::clone(&output);
                                        let repository = Arc::clone(&repository);
                                        let branch = Arc::clone(&branch);
                                        async move {
                                            let options = DownloadOptions {
                                                repository: repository.as_ref(),
                                                output: output.as_path(),
                                                source: &item.source,
                                                destination: &item.destination,
                                                branch: branch.as_deref(),
                                                clobber,
                                            };
                                            match Self::download_file(&options).await {
                                                | Ok(()) => Ok(()),
                                                | Err(stream_error) => match repository.as_ref() {
                                                    | Repository::GitLab { .. } => match Self::download_gitlab_via_api(&options).await {
                                                        | Ok(()) => Ok(()),
                                                        | Err(fallback_error) => Err(eyre!(
                                                            "{stream_error}; and the buffered GitLab API fallback also failed — {fallback_error}"
                                                        )),
                                                    },
                                                    | _ => Err(stream_error),
                                                },
                                            }
                                        }
                                    }
                                };
                                transfer_bucket_files(self.name, repository_location, items, options, "Downloading", operation).await
                            }
                            | Err(why) => Err(why),
                        }
                    }
                    | Err(why) => Err(why),
                },
                | Err(why) => {
                    error!("=> {} Get file paths for download — {why}", Label::fail());
                    Err(why)
                }
            },
            | Err(why) => Err(why),
        }
    }
    /// Resolve a repository's default branch, falling back to `None` when unavailable.
    async fn resolve_branch(&self) -> ApiResult<Option<String>> {
        match &self.code_repository {
            | Repository::GitLab { .. } => match (self.code_repository.domain(), self.code_repository.id()) {
                | (Some(domain), Some(identifier)) => {
                    let options = gitlab::Options::from_env().with_domain(domain).with_identifier(identifier);
                    match gitlab::project_details(&options).await {
                        | Ok(details) => Ok(details.default_branch.filter(|value| !value.is_empty())),
                        | Err(why) => {
                            warn!("=> {} Resolve GitLab default branch — {why}", Label::fail());
                            Ok(None)
                        }
                    }
                }
                | _ => Ok(None),
            },
            | Repository::GitHub { .. } => Ok(Some("main".to_string())),
            | _ => Ok(None),
        }
    }
    /// Build a streaming download item for a bucket file, resolving the provider-specific raw URL and credentials.
    pub(super) fn download_item(options: &DownloadOptions<'_>) -> ApiResult<DownloadItem> {
        let path = options.source.display().to_string();
        let request = match options.repository {
            | Repository::GitLab { .. } => {
                let branch = options.branch.filter(|value| !value.is_empty()).unwrap_or("HEAD");
                let branch_ref = branch.percent_encode();
                let encoded_path = path.percent_encode();
                let headers = first_env_var(&GITLAB_TOKEN_VARIABLE_NAMES)
                    .filter(|value| !value.trim().is_empty())
                    .map(|token| http::headers([(GITLAB_PRIVATE_TOKEN_HEADER, token.as_str())]));
                match (options.repository.domain(), options.repository.id()) {
                    | (Some(domain), Some(identifier)) => Ok((
                        format!("https://{domain}/api/v4/projects/{identifier}/repository/files/{encoded_path}/raw?ref={branch_ref}"),
                        headers,
                    )),
                    | _ => Err(eyre!("Failed to build GitLab API request for repository path")),
                }
            }
            | _ => options
                .repository
                .raw_url(path)
                .map(|url| (url, None))
                .ok_or_else(|| eyre!("Failed to build raw URL for repository path")),
        };
        request.map(|(url, headers)| DownloadItem {
            url,
            path: options.destination.display().to_string(),
            size: None,
            sha: None,
            headers,
        })
    }
    /// Stream a bucket file to disk with `.part` resume and atomic finalization.
    ///
    /// Existing files are left untouched unless `clobber` is set, in which case the
    /// destination is removed before the transfer starts.
    pub(super) async fn download_file(options: &DownloadOptions<'_>) -> ApiResult<()> {
        match Self::download_item(options) {
            | Ok(item) => {
                let target = options.output.join(options.destination);
                match (options.clobber, item.is_complete_at(&target)) {
                    | (false, true) => Ok(()),
                    | (true, _) => match Self::remove_destination(&target) {
                        | Ok(()) => DownloadTask::new(item, options.output, true, true).download().await,
                        | Err(why) => Err(why),
                    },
                    | _ => DownloadTask::new(item, options.output, true, true).download().await,
                }
            }
            | Err(why) => Err(why),
        }
    }
    /// Fall back to the buffered GitLab repository-file API for a bucket file.
    async fn download_gitlab_via_api(options: &DownloadOptions<'_>) -> ApiResult<()> {
        match (options.repository.domain(), options.repository.id()) {
            | (Some(domain), Some(identifier)) => {
                let request = gitlab::Options::from_env()
                    .with_domain(domain)
                    .with_identifier(identifier)
                    .with_path(options.source.display().to_string())
                    .with_sha(options.branch.filter(|value| !value.is_empty()).unwrap_or("HEAD"));
                match gitlab::repository_file(&request).await.and_then(|file| file.decoded_content()) {
                    | Ok(bytes) if options.clobber => {
                        let write_lock = Mutex::new(());
                        Self::write_file(options.output, options.destination, true, &write_lock, || async {
                            Ok::<_, Report>(bytes)
                        })
                        .await
                    }
                    | Ok(bytes) => write_file_bytes(options.output.join(options.destination), || async { Ok::<_, Report>(bytes) }).await,
                    | Err(why) => Err(why),
                }
            }
            | _ => Err(eyre!("Failed to build GitLab API request for repository path")),
        }
    }
    async fn file_paths(&self, directory: &str) -> ApiResult<Vec<String>> {
        let code_repository = self.code_repository.clone();
        let bucket_name = self.name.clone().unwrap_or_else(|| "Bucket".to_string()).to_uppercase();
        match &code_repository {
            | Repository::Git { .. } => {
                let path = match code_repository.location().local_path() {
                    | Some(value) => value,
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
                            match gitlab::repository_tree(&page_options).await {
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
            | Repository::OCI { .. } => Err(eyre!("OCI artifacts are unsupported for bucket downloads")),
        }
    }
}
impl From<&Path> for Bucket {
    fn from(value: &Path) -> Self {
        let location = Location::Simple(format!("file:{}/", value.display()));
        Self::init()
            .code_repository(Repository::Git {
                location: Box::new(location),
            })
            .build()
    }
}
impl From<&str> for Bucket {
    fn from(value: &str) -> Self {
        let location = Location::Simple(value.to_string());
        if location.uri().is_none() {
            exit(exitcode::DATAERR);
        }
        let repository = match location.is_local() {
            | true => Repository::Git {
                location: Box::new(location),
            },
            | false => {
                let host = match location.host() {
                    | Some(value) => value.to_lowercase(),
                    | None => {
                        error!(value, "=> {} Parse URI - No host", Label::fail());
                        exit(exitcode::DATAERR);
                    }
                };
                if host.contains("github.com") {
                    Repository::GitHub {
                        location: Box::new(location),
                    }
                } else {
                    let id = None;
                    Repository::GitLab {
                        id,
                        location: Box::new(location),
                    }
                }
            }
        };
        Bucket::init().code_repository(repository).build()
    }
}
impl From<PathBuf> for Bucket {
    fn from(value: PathBuf) -> Self {
        Self::from(value.as_path())
    }
}
impl Default for BucketOptions {
    fn default() -> Self {
        Self {
            output: None,
            database_path: None,
            no_local_database: false,
            strict: false,
            canonicalize: false,
            offline: false,
            threads: 10,
            quiet: false,
            ignore: Vec::new(),
            filter: Vec::new(),
            flatten: false,
            clobber: false,
        }
    }
}
impl Buckets {
    /// Transfers all buckets and returns the total number of processed items.
    pub async fn transfer(self, options: &BucketOptions, policy: TransferPolicy) -> ApiResult<usize> {
        match options.output {
            | Some(_) => {
                stream::iter(self.0)
                    .then(|bucket| bucket.transfer(options, policy, None))
                    .try_fold(0_usize, |total, count| future::ready(Ok(total.saturating_add(count))))
                    .await
            }
            | None => Ok(0),
        }
    }
}
impl From<Vec<Bucket>> for Buckets {
    fn from(value: Vec<Bucket>) -> Self {
        Self(value)
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
impl Validate for RegistryProfile {
    fn validate(&self) -> Result<(), ValidationReport> {
        let location = Location::from(&self.endpoint);
        let scheme = location.scheme();
        let is_valid_scheme = scheme.is_https() || scheme.is_http() && self.plain_http;
        let is_valid_endpoint = location.host().is_some() && is_valid_scheme;
        let is_valid_environment = self.credential_env.as_ref().is_none_or(|value| {
            let valid_characters = value
                .chars()
                .enumerate()
                .all(|(index, character)| character == '_' || character.is_ascii_uppercase() || index > 0 && character.is_ascii_digit());
            !value.is_empty() && valid_characters
        });
        let has_paired_identity = matches!((&self.client_cert, &self.client_key), (Some(_), Some(_)) | (None, None));
        let pairs = [
            (!is_valid_endpoint).then(|| {
                let message = "Registry profile requires an HTTPS endpoint unless plainHttp is explicitly enabled".into();
                ("endpoint", ValidationError::new("scheme").with_message(message))
            }),
            (!is_valid_environment).then(|| {
                let message = "Registry profile has an invalid credentialEnv name".into();
                ("credential_env", ValidationError::new("format").with_message(message))
            }),
            (!has_paired_identity).then(|| {
                let message = "Registry profile must configure clientCert and clientKey together".into();
                ("client_cert", ValidationError::new("paired").with_message(message))
            }),
        ];
        let data = pairs.into_iter().flatten().fold(ValidationReport::new(), |mut errors, (field, error)| {
            errors.add(field, error);
            errors
        });
        match data.is_empty() {
            | true => Ok(()),
            | false => Err(data),
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
    pub(super) fn resolution_failure_reason(why: &impl fmt::Display, fallbacks_enabled: bool, fallbacks: &[String]) -> String {
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
impl fmt::Display for ModelEntryOptions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.name)
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
impl From<&Path> for TransferItem {
    fn from(source: &Path) -> Self {
        Self {
            source: source.to_path_buf(),
            destination: source.to_path_buf(),
        }
    }
}
impl TransferItem {
    pub(super) fn collect(paths: Vec<String>, flatten: bool) -> ApiResult<Vec<Self>> {
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
impl TransferManifest {
    /// Return the legacy reported count of transferred JSON and image files.
    pub fn count(&self) -> usize {
        let paths = self.files.iter().map(|path| path.display().to_string()).collect::<Vec<_>>();
        count_json_files(&paths).saturating_add(count_image_files(&paths))
    }
    fn canonicalize(self, options: &BucketOptions) -> ApiResult<Self> {
        let BucketOptions { canonicalize, output, .. } = options;
        match canonicalize {
            | false => Ok(self),
            | true => {
                let output = output.clone().unwrap_or_default();
                let canonicalized = self
                    .files
                    .iter()
                    .filter(is_filetype(SUPPORTED_RAD_FILETYPES))
                    .try_fold((), |(), relative| {
                        let path = output.join(relative);
                        ResearchActivity::migrate(path.clone())
                            .map(|_| ())
                            .map_err(|why| eyre!("Failed to canonicalize transferred RAD {} — {why}", path.display()))
                    });
                canonicalized.map(|()| self)
            }
        }
    }
    /// Ingest transferred RAD files into the canonical research activity table.
    pub fn ingest(&self, options: &BucketOptions) -> ApiResult<()> {
        let BucketOptions {
            database_path,
            no_local_database,
            output,
            strict,
            ..
        } = options;
        match no_local_database {
            | true => Ok(()),
            | false => {
                let output = output.clone().unwrap_or_default();
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
                        match strict {
                            | true => ResearchActivity::read_strict(path.clone()),
                            | false => ResearchActivity::read(path.clone()),
                        }
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
            (PID::Handle, rad.meta.handle.as_ref()),
            (PID::ISBN, rad.meta.books.as_ref()),
            (PID::Patent, rad.meta.patents.as_ref()),
            (PID::RAID, rad.meta.raid.as_ref()),
            (PID::SWHID, rad.meta.swhid.as_ref()),
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
fn bag_root(root: &Path) -> ApiResult<PathBuf> {
    match root.join("bagit.txt").is_file() {
        | true => Ok(root.to_path_buf()),
        | false => root
            .read_dir()
            .map_err(Into::into)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter_map(|entry| entry.file_type().ok().filter(|kind| kind.is_dir()).map(|_| entry.path()))
                    .collect::<Vec<_>>()
            })
            .and_then(|directories| match directories.as_slice() {
                | [directory] if directory.join("bagit.txt").is_file() => Ok(directory.clone()),
                | _ => Err(eyre!("BagIt metadata must be at the archive root or beneath one enclosing directory")),
            }),
    }
}
pub(super) fn count_image_files(paths: &[String]) -> usize {
    paths.iter().filter(|&x| has_image_extension(x)).count()
}
pub(super) fn count_json_files(paths: &[String]) -> usize {
    paths.iter().filter(|&path| path.to_lowercase().ends_with(".json")).count()
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
pub(super) fn has_image_extension(path: &String) -> bool {
    path.to_lowercase().ends_with(".png") || path.to_lowercase().ends_with(".jpg")
}
pub(super) fn is_filtered_path(path: &str, filter: &[Regex]) -> bool {
    filter.is_empty() || filter.iter().any(|pattern| pattern.is_match(path).unwrap_or(false))
}
pub(super) fn is_ignored_path(path: &str, ignore: &[Regex]) -> bool {
    let is_builtin_ignored = IGNORE.iter().any(|value| path.ends_with(value));
    let is_regex_ignored = ignore.iter().any(|pattern| pattern.is_match(path).unwrap_or(false));
    is_builtin_ignored || is_regex_ignored
}
pub(super) fn operations_complete_message(name: Option<String>, json_count: usize, image_count: usize) -> String {
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
