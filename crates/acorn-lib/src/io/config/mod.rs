//! Application configuration functions and data structures
//!
//! ACORN is configured by a JSON, JSONC, or YAML file (typically named `.acorn.json`)
//!
//! The ACORN configuration file configures what buckets should be downloaded, readability analysis, <span title="Large Language Model">LLM</span> settings, and more.
pub mod application;
pub mod bucket;
pub mod transfer;
use crate::{
    io::{
        api,
        database::{resolve_database_path, schema::ModelRow, Row},
        sync, ApiResult, Executor, Remote,
    },
    util::constants::app::IGNORE,
};
use acorn_core::OneOrMany;
use acorn_core::{prelude::HashSet, validation::ValidationError, Location, Repository};
use acorn_host::terminal::Label;
use acorn_schema::{
    agent::{ModelDetails, Quantization},
    hardware::memory::Memory,
    validation::{Validate, ValidationReport},
};
pub use application::{
    AcreSchemaVersion, ApplicationConfiguration, ApplicationOptions, BucketReference, CrosswalkOptions, DaemonBindConfiguration, DaemonConfiguration,
    DaemonJobConfiguration, DaemonTlsConfiguration, Destination, Folder, FolderOptions, FolderRegistry, Hook, ResolvedFolder, ScriptOptions,
    VariableReferences, WhitelistLookup,
};

pub use application::{SCRIPT_OPTION_ROOTS, UNGOVERNED_ROOT_ATTRIBUTES};
use bon::Builder;
pub use bucket::{
    Bucket, BucketExtension, BucketOptions, BucketSource, BucketSourceIdentity, BucketSourceKind, Buckets, ResolvedBucket, SourceInput,
};
use color_eyre::eyre::{eyre, Report};
use core::fmt::{self, Debug};
use derive_more::Display;
use fancy_regex::Regex;
use itertools::Itertools;
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::{
    de::{Error as _, MapAccess, Visitor},
    Deserialize, Serialize,
};
use serde_with::skip_serializing_none;
use std::path::PathBuf;
use strum::EnumIs;
use tracing::warn;
pub use transfer::{TransferManifest, TransferManifestItem, TransferPolicy};
use veil::Redact;

/// Authentication requirement for a configured model download source
#[derive(Clone, Debug, Default, Deserialize, Display, EnumIs, Eq, JsonSchema, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(untagged)]
pub enum ModelEntry {
    /// Bare model ID / path / URL string
    Selector(String),
    /// Detailed model download entry
    Entry(ModelEntryOptions),
}
/// Registry-specific metadata provider for an OCI registry profile.
#[derive(Clone, Debug, Default, Deserialize, Display, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RegistryKind {
    /// Generic OCI Distribution-compatible registry.
    #[default]
    OCI,
    /// Harbor registry with optional v2 API enrichment.
    Harbor,
}
/// Runner status for CI/CD pipelines.
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
#[derive(Clone, Debug, Default, Deserialize, EnumIs, JsonSchema, Serialize)]
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
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize)]
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
    pub filter: Option<OneOrMany<String>>,
    /// Regex pattern(s) used to exclude model files for this entry
    #[serde(default)]
    pub ignore: Option<OneOrMany<String>>,
    /// Ordered exact GGUF quantization allowlist
    #[serde(default)]
    pub quantization: Option<acorn_schema::OneOrMany<Quantization>>,
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
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize)]
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
#[derive(Builder, Clone, Deserialize, JsonSchema, Redact, Serialize)]
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
    pub tags: Option<OneOrMany<String>>,
    /// Whether the runner runs untagged jobs (GitLab-specific)
    #[builder(default)]
    #[serde(default, alias = "run_untagged")]
    pub run_untagged: bool,
    /// Optional GitLab host/domain override (for self-managed instances)
    pub host: Option<String>,
    /// Container host used to run this runner
    pub target: Option<RunnerTarget>,
    /// Default Docker image for the runner itself
    #[builder(default = String::from("gitlab/gitlab-runner:latest"))]
    #[serde(default = "default_docker_image")]
    pub docker_image: String,
    /// Runner identifier assigned by GitLab during creation
    #[serde(default)]
    pub identifier: Option<u64>,
    /// Runner authentication token returned from GitLab API
    #[serde(skip, default)]
    #[redact]
    pub token: Option<api::Secret>,
}
/// Container execution target for a configured CI/CD runner.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
pub struct RunnerTarget {
    /// SSH URI or alias of the remote container host
    #[schemars(with = "String")]
    pub host: Remote,
}
struct RunnerTargetVisitor;
impl FilterSet {
    /// Compile the bucket options' ignore and filter regex patterns into a filter set.
    pub fn compile(options: &BucketOptions) -> ApiResult<Self> {
        Self::try_from((options.common.ignore.as_slice(), options.common.filter.as_slice()))
    }
    /// Filter items based on ignore and filter regex patterns
    pub fn filter<T>(
        items: Vec<T>,
        filter: &[String],
        ignore: &[String],
        value: impl Fn(&T) -> String,
        keep: impl Fn(&T) -> bool,
    ) -> ApiResult<Vec<T>> {
        match FilterSet::try_from((ignore, filter)) {
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
impl TryFrom<(&[String], &[String])> for FilterSet {
    type Error = Report;

    fn try_from((ignore, filter): (&[String], &[String])) -> Result<Self, Self::Error> {
        let compile = |patterns: &[String]| {
            patterns
                .iter()
                .map(|pattern| Regex::new(pattern).map_err(|why| eyre!("Invalid regex/filter pattern '{pattern}': {why}")))
                .collect::<ApiResult<Vec<Regex>>>()
        };
        compile(ignore).and_then(|ignore| compile(filter).map(|filter| Self { ignore, filter }))
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
    pub(crate) fn resolution_failure_reason(why: &impl fmt::Display, fallbacks_enabled: bool, fallbacks: &[String]) -> String {
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
                let message = "Registry profile requires an HTTPS endpoint unless plainHttp is explicitly enabled";
                ("endpoint", ValidationError::new("scheme").with_message(message))
            }),
            (!is_valid_environment).then(|| {
                let message = "Registry profile has an invalid credentialEnv name";
                ("credential_env", ValidationError::new("format").with_message(message))
            }),
            (!has_paired_identity).then(|| {
                let message = "Registry profile must configure clientCert and clientKey together";
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
        Self {
            token: value.map(api::Secret::from),
            ..self
        }
    }
}
impl<'de> Deserialize<'de> for RunnerTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(RunnerTargetVisitor)
    }
}
impl<'de> Visitor<'de> for RunnerTargetVisitor {
    type Value = RunnerTarget;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an SSH alias, SSH URI, or object containing a host field")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let key = map.next_key::<String>()?.ok_or_else(|| A::Error::missing_field("host"))?;
        if key != "host" {
            Err(A::Error::unknown_field(&key, &["host"]))
        } else {
            let host = map.next_value::<Remote>()?;
            match map.next_key::<String>()? {
                | None => Ok(RunnerTarget { host }),
                | Some(key) if key == "host" => Err(A::Error::duplicate_field("host")),
                | Some(key) => Err(A::Error::unknown_field(&key, &["host"])),
            }
        }
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        value.parse().map(|host| RunnerTarget { host }).map_err(E::custom)
    }
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&value)
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
fn default_docker_image() -> String {
    "gitlab/gitlab-runner:latest".to_string()
}
fn default_executor() -> Executor {
    Executor::Docker
}
pub(crate) fn is_filtered_path(path: &str, filter: &[Regex]) -> bool {
    filter.is_empty() || filter.iter().any(|pattern| pattern.is_match(path).unwrap_or(false))
}
pub(crate) fn is_ignored_path(path: &str, ignore: &[Regex]) -> bool {
    let is_builtin_ignored = IGNORE.iter().any(|value| path.ends_with(value));
    let is_regex_ignored = ignore.iter().any(|pattern| pattern.is_match(path).unwrap_or(false));
    is_builtin_ignored || is_regex_ignored
}

#[cfg(test)]
mod tests;
