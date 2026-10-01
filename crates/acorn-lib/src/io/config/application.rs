//! Application configuration I/O
//!
//! Handles discovery, parsing, rendering and synchronization of ACORN files.
#[cfg(feature = "exec")]
use crate::exec::manifest::ScriptManifestSource;
#[cfg(feature = "agentic")]
use crate::io::api::gitlab::DeploymentConfig;
#[cfg(feature = "agentic")]
use crate::io::api::powerautomate::PowerAutomateConfig;
use crate::{
    io::{
        api::Endpoint,
        config::{Bucket, ModelEntry, RegistryProfile, RunnerDetails, SourceInput},
        parse_jsonc_cst, read_file, sync,
        sync::ConfigExt,
        write_file, ApiResult, CstRootNode, CstValue, FromPath, InputOutput,
    },
    util::{
        constants::app::{APPLICATION, DEFAULT_CONFIG_FILENAMES},
        text_diff_changes_with_color,
    },
};
use acorn_core::{
    prelude::{HashMap, HashSet},
    util::{detect_json, MimeType},
    validation::ValidationError,
};
use acorn_host::terminal::Label;
use acorn_schema::{
    validation::{is_identifier, Validate, ValidationReport},
    OneOrMany,
};
use bon::Builder;
use color_eyre::eyre::eyre;
#[cfg(not(feature = "exec"))]
use feature_configuration::ScriptManifestSource;
#[cfg(not(feature = "agentic"))]
use feature_configuration::{DeploymentConfig, PowerAutomateConfig};
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    env,
    path::{Path, PathBuf},
};
use strum::{Display, EnumIs, EnumString};
use tracing::{error, info};

/// Configuration-only representations retained when optional execution features are disabled.
pub mod feature_configuration {
    use acorn_core::util::MimeType;
    use alloc::collections::BTreeMap;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;

    /// Deployment-owned configuration for PowerAutomate intake and callbacks.
    #[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct PowerAutomateConfig {
        /// Named callback destination.
        pub callback_destination: String,
        /// Runtime environment-variable names.
        pub variables: PowerAutomateVariables,
        /// GitLab generic-package name.
        pub package: String,
        /// GitLab project identifier.
        pub gitlab_project: String,
        /// Stable project-to-path mappings.
        pub projects: Vec<ProjectIntakeConfig>,
        /// Optional reporting timezone.
        #[serde(default)]
        pub timezone: Option<String>,
    }
    /// Runtime environment-variable names used by the PowerAutomate integration.
    #[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct PowerAutomateVariables {
        /// Environment variable containing the callback URL.
        pub callback_url: String,
        /// Environment variable containing the inbound secret.
        pub inbound_secret: String,
    }
    /// Configuration for one stable PowerApp project identifier.
    #[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ProjectIntakeConfig {
        /// Project member email allowlist.
        pub members: Vec<String>,
        /// Safe path to the project index.
        pub path: String,
        /// Stable form project identifier.
        pub project_id: String,
    }
    /// Allowed external effect produced by a repository workflow.
    #[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum WorkflowEffect {
        /// Create or reuse a bot branch.
        Branch,
        /// Create a commit on a bot branch.
        Commit,
        /// Publish or reconcile an external commit status.
        CommitStatus,
        /// Post a completion comment.
        CompletionComment,
        /// Create or reuse a draft merge request.
        DraftMergeRequest,
        /// Apply a work-item label.
        Label,
        /// Create or update a bot-authored note.
        Note,
    }
    /// Processing action allowed for a repository workflow.
    #[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum WorkflowAction {
        /// Check quality gates.
        Check,
        /// Invoke configured enrichment capabilities.
        Enrich,
        /// Format documents.
        Format,
        /// Add linked-data artifacts.
        Link,
        /// Validate documents.
        Validate,
    }
    /// Output behavior for a deployment workflow.
    #[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum WorkflowOutputMode {
        /// Update source files and create configured artifacts.
        SourceAndArtifacts,
    }
    /// Deployment configuration for repository automation.
    #[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "kebab-case")]
    pub struct DeploymentConfig {
        /// Whether fork merge requests may publish into the destination.
        #[serde(default)]
        pub allow_forks: bool,
        /// Explicit publication destination.
        pub destination: Option<String>,
        /// Deployment-approved workflows.
        pub workflows: BTreeMap<String, WorkflowPolicy>,
    }
    /// Output configuration for a deployment workflow.
    #[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "kebab-case")]
    pub struct WorkflowOutputs {
        /// Output behavior.
        pub mode: WorkflowOutputMode,
    }
    /// Deployment-owned policy for one registered workflow.
    #[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "kebab-case")]
    pub struct WorkflowPolicy {
        /// Processing actions allowed for this workflow.
        pub actions: Vec<WorkflowAction>,
        /// External effects allowed for this workflow.
        pub effects: Vec<WorkflowEffect>,
        /// Input formats allowed for this workflow.
        pub formats: Vec<MimeType>,
        /// Output behavior.
        pub outputs: WorkflowOutputs,
        /// Repository-relative input path patterns.
        pub paths: Vec<String>,
    }
    /// Application-configuration source for executable-hook policy.
    #[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(untagged)]
    pub enum ScriptManifestSource {
        /// Inline executable-hook manifest.
        Inline(ScriptManifest),
        /// Safe relative deployment manifest name.
        Path(String),
    }
    /// Filesystem access granted beneath named roots.
    #[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    pub struct FilesystemCapability {
        /// Maximum aggregate bytes transferred per invocation.
        #[serde(default)]
        pub max_bytes: usize,
        /// Maximum files inspected or proposed per invocation.
        #[serde(default)]
        pub max_files: usize,
        /// Named absolute directory roots.
        pub roots: BTreeMap<String, PathBuf>,
    }
    /// Network access granted to exact HTTPS authorities.
    #[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    pub struct NetworkCapability {
        /// Exact host names, optionally followed by a port.
        pub hosts: Vec<String>,
        /// Maximum requests per invocation.
        #[serde(default)]
        pub max_requests: usize,
        /// Maximum response bytes per request.
        #[serde(default)]
        pub max_response_bytes: usize,
        /// Per-request timeout in milliseconds.
        #[serde(default)]
        pub timeout: u64,
    }
    /// Per-script evaluation ceilings.
    #[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(default, deny_unknown_fields, rename_all = "camelCase")]
    pub struct ResourceLimits {
        /// Maximum registered host capability calls.
        pub max_calls: usize,
        /// Maximum Rhai operations.
        pub max_operations: u64,
        /// Maximum wall-clock runtime in milliseconds.
        pub max_runtime: u64,
    }
    /// Capabilities granted to one script.
    #[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(default, deny_unknown_fields, rename_all = "camelCase")]
    pub struct ScriptCapabilities {
        /// Named filesystem read roots.
        pub read: Option<FilesystemCapability>,
        /// Named filesystem write roots.
        pub write: Option<FilesystemCapability>,
        /// Permission to replace the proposed workflow document.
        pub mutation: bool,
        /// Restricted outbound HTTPS access.
        pub network: Option<NetworkCapability>,
    }
    /// One digest-pinned script authorization.
    #[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    pub struct ScriptAuthorization {
        /// Rhai executable-extension ABI version.
        pub version: String,
        /// Capabilities granted to this script.
        #[serde(default)]
        pub capabilities: ScriptCapabilities,
        /// Expected SHA-256 source identity.
        pub digest: String,
        /// Required script function name.
        #[serde(default)]
        pub entry: String,
        /// Workflow hooks this script may handle.
        pub hooks: Vec<super::Hook>,
        /// Stable script identifier.
        pub id: String,
        /// Evaluation ceilings.
        #[serde(default)]
        pub limits: ResourceLimits,
    }
    /// Versioned deployment manifest for executable hooks.
    #[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    pub struct ScriptManifest {
        /// Manifest schema version.
        pub version: String,
        /// Authorized scripts.
        pub scripts: Vec<ScriptAuthorization>,
    }
}
/// Supported Application Configuration for Research Enablement schema versions.
#[derive(Clone, Copy, Debug, Default, Eq, eserde::Deserialize, JsonSchema, PartialEq, Serialize)]
pub enum AcreSchemaVersion {
    /// ACRE schema version 1.0.0.
    #[default]
    #[serde(rename = "1.0.0")]
    V1_0_0,
}
/// Supported hook for constrained Rhai extensions.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, JsonSchema, PartialEq, Serialize, Display, EnumIs, EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Hook {
    /// Check quality gate.
    Check,
    /// Format workflow.
    Format,
    /// Link workflow.
    Link,
    /// Enrich workflow.
    Enrich,
    /// Validate workflow.
    Validate,
}
/// Application Configuration for Research Enablement.
///
/// ### Example `.acorn.json` configuration file
///
/// ```json
/// {
///     "schemaVersion": "1.0.0",
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
///             "name": "oklahoma-runner",
///             "description": "Oklahoma Docker Runner",
///             "repository": {
///                 "provider": "gitlab",
///                 "id": 24758,
///                 "uri": "https://code.ornl.gov/research-enablement"
///             },
///             "type": "group",
///             "executor": "docker",
///             "runUntagged": true,
///             "target": {
///                 "host": "oklahoma"
///             }
///         }
///     ]
/// }
/// ```
#[derive(Clone, Debug, Default, JsonSchema, Serialize, eserde::Deserialize)]
#[schemars(title = "Application Configuration for Research Enablement")]
#[serde(deny_unknown_fields)]
pub struct ApplicationConfiguration {
    /// CST root for JSONC comment-preserving round-trips
    #[schemars(skip)]
    #[serde(default, skip)]
    pub cst: Option<CstRootNode>,
    /// ACRE schema version. Omitted legacy files use version 1.0.0.
    #[serde(default, rename = "schemaVersion")]
    pub schema_version: AcreSchemaVersion,
    /// List of buckets
    #[eserde(compat)]
    #[serde(default)]
    pub buckets: Option<Vec<Bucket>>,
    /// Synchronization targets for llama-swap, OpenCode, VS Code, and Goose
    #[eserde(compat)]
    #[serde(default)]
    pub config: Option<sync::Config>,
    /// List of endpoints
    #[eserde(compat)]
    #[serde(default)]
    pub endpoints: Option<Vec<Endpoint>>,
    /// List of models to download — bare IDs/paths or detailed download entries
    #[eserde(compat)]
    #[serde(default)]
    pub models: Option<Vec<ModelEntry>>,
    /// Deployment-owned PowerAutomate form intake and callback configuration.
    #[eserde(compat)]
    #[schemars(with = "Option<feature_configuration::PowerAutomateConfig>")]
    #[serde(default, rename = "powerAutomate")]
    pub power_automate: Option<PowerAutomateConfig>,
    /// Named OCI registry profiles referenced by model entries.
    #[eserde(compat)]
    #[schemars(with = "Option<std::collections::BTreeMap<String, RegistryProfile>>")]
    #[serde(default)]
    pub registries: Option<HashMap<String, RegistryProfile>>,
    /// List of runners
    #[eserde(compat)]
    #[serde(default)]
    pub runners: Option<Vec<RunnerDetails>>,
    /// Lookup object for whitelisted downloadable items
    #[eserde(compat)]
    #[serde(default)]
    pub whitelist: Option<WhitelistLookup>,
    /// Deployment-owned repository workflow policy.
    #[eserde(compat)]
    #[schemars(with = "Option<feature_configuration::DeploymentConfig>")]
    #[serde(default)]
    pub workflow: Option<DeploymentConfig>,
    /// Deployment manifest name or an inline executable-hook manifest.
    #[eserde(compat)]
    #[schemars(with = "Option<feature_configuration::ScriptManifestSource>")]
    #[serde(default, rename = "scriptManifest")]
    pub script_manifest: Option<ScriptManifestSource>,
    /// Optional ordered list of constrained Rhai script hooks.
    #[eserde(compat)]
    #[serde(default)]
    pub scripts: Option<Vec<Entry>>,
}
/// One entry in the top-level `scripts` attribute.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Entry {
    /// Stable identifier used to match a deployment manifest entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Path to the script, relative to `<project-root>/.acorn/` unless `absolute` is true.
    pub path: String,
    /// Non-empty, duplicate-free hooks.
    pub hooks: Vec<Hook>,
    /// When true, path is a platform-native absolute path.
    #[serde(default)]
    pub absolute: bool,
}
/// Whitelist entry for downloadable items
/// ### Note
/// An empty whitelist is interpreted to mean there are no restrictions and any item will be allowed
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(start_fn = init)]
pub struct WhitelistLookup {
    /// Whitelisted buckets
    pub buckets: Option<Vec<String>>,
    /// Whitelisted model IDs/URLs, or one URL to a JSON whitelist document
    pub models: Option<OneOrMany<String>>,
}
impl ApplicationConfiguration {
    /// Return whether any configured script subscribes to the requested hook.
    pub fn has_hook(&self, hook: Hook) -> bool {
        self.scripts
            .as_deref()
            .unwrap_or_default()
            .iter()
            .any(|entry| entry.hooks.contains(&hook))
    }
    /// Load explicit or discovered application configuration, rejecting a missing explicit path.
    pub fn load(path: &Option<PathBuf>) -> ApiResult<Self> {
        match path {
            | Some(path) if !path.is_file() => Err(eyre!("Configuration file does not exist — {}", path.display())),
            | _ => Self::resolve(path).map_or_else(|| Ok(Self::default()), Self::read),
        }
    }
    /// Load configuration and return the resolved source path alongside the parsed value.
    pub fn load_with_path(path: &Option<PathBuf>) -> ApiResult<(Self, Option<PathBuf>)> {
        match path {
            | Some(path) if !path.is_file() => Err(eyre!("Configuration file does not exist — {}", path.display())),
            | _ => {
                let resolved = Self::resolve(path);
                match &resolved {
                    | Some(p) => Self::read(p.clone()).map(|cfg| (cfg, resolved)),
                    | None => Ok((Self::default(), None)),
                }
            }
        }
    }
    /// Return the project root directory containing the resolved configuration.
    pub fn root(resolved: &Option<PathBuf>) -> PathBuf {
        resolved
            .as_ref()
            .and_then(|p| p.parent())
            .map_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")), |p| p.to_path_buf())
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
    pub(crate) fn with_models(&self, entries: &[ModelEntry]) -> ApiResult<Self> {
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
    pub(crate) fn write_or_preview(&self, path: &Path, dry_run: bool, no_color: bool) -> ApiResult<()> {
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
        match MimeType::from_path(path) {
            | MimeType::Json | MimeType::Jsonc => self.render_json(),
            | MimeType::Yaml => serde_norway::to_string(self).map_err(|why| eyre!("Failed to serialize YAML config — {why}")),
            | _ => Err(eyre!("Unsupported configuration file extension")),
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
    /// Resolve transfer sources from an explicit or discovered configuration file, or from values.
    ///
    /// A configuration file takes precedence over `values`, so a discovered file in the working directory shadows positional sources.
    /// The result may be empty; callers decide whether an empty selection is an error because the message is specific to each command.
    pub fn sources(config: &Option<PathBuf>, values: &[String]) -> ApiResult<Vec<SourceInput>> {
        match Self::resolve(config) {
            | Some(path) => Self::read(path).map(|content| content.buckets.unwrap_or_default().into_iter().map(SourceInput::from).collect()),
            | None => Ok(values.iter().cloned().map(SourceInput::from).collect()),
        }
    }
    /// Resolve an application configuration path relative to an explicit directory.
    pub fn resolve_in(path: &Option<PathBuf>, directory: &Path) -> Option<PathBuf> {
        path.as_ref().filter(|value| value.is_file()).cloned().or_else(|| {
            DEFAULT_CONFIG_FILENAMES
                .iter()
                .map(|name| directory.join(name))
                .find(|candidate| candidate.is_file())
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
        match MimeType::from_path(&source) {
            | MimeType::Json => Self::read_json(source.clone()),
            | MimeType::Jsonc => Self::read_jsonc(source.clone()),
            | MimeType::Yaml => Self::read_yaml(source.clone()),
            | _ => Err(eyre!("Unsupported configuration file extension")),
        }
    }
    /// Read configuration (e.g., `.acorn.json`) using Serde and [`ApplicationConfiguration`] struct
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        let content = match read_file(&path) {
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
        let content = match read_file(&path) {
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
        let content = match read_file(&path) {
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
        match MimeType::from_path(&target) {
            | MimeType::Json | MimeType::Jsonc => self.write_json(&target),
            | MimeType::Yaml => self.write_yaml(&target),
            | _ => Err(eyre!("Unsupported configuration file extension")),
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
impl Validate for ApplicationConfiguration {
    fn validate(&self) -> Result<(), ValidationReport> {
        let entries = self.scripts.as_deref().unwrap_or_default();
        let mut report = ValidationReport::new();
        #[cfg(feature = "agentic")]
        if let Some(power_automate) = &self.power_automate {
            if let Err(errors) = power_automate.validate() {
                report.append_prefixed("powerAutomate", errors);
            }
        }
        entries.iter().enumerate().for_each(|(index, entry)| {
            if let Err(errors) = entry.validate() {
                report.append_prefixed(&format!("scripts[{index}]"), errors);
            }
        });
        entries.iter().enumerate().fold(HashSet::<&str>::new(), |mut paths, (index, entry)| {
            if !paths.insert(entry.path.as_str()) {
                report.add(
                    format!("scripts[{index}].path"),
                    ValidationError::new("duplicate").with_message(format!("Duplicate script path `{}`", entry.path)),
                );
            }
            paths
        });
        entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| entry.id.as_deref().map(|id| (index, id)))
            .fold(HashSet::<&str>::new(), |mut identifiers, (index, id)| {
                if !identifiers.insert(id) {
                    report.add(
                        format!("scripts[{index}].id"),
                        ValidationError::new("duplicate").with_message(format!("Duplicate script identifier `{id}`")),
                    );
                }
                identifiers
            });
        report.finish()
    }
}
impl Validate for Entry {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut hooks = HashSet::new();
        let duplicate_hook = self.hooks.iter().find(|hook| !hooks.insert(**hook));
        let path = Path::new(&self.path);
        let invalid_extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .filter(|extension| !extension.eq_ignore_ascii_case("rhai"));
        let identifier_error = match (self.id.as_deref(), self.id.as_deref().filter(is_identifier)) {
            | (Some(id), None) => Some((
                "id",
                ValidationError::new("format").with_message(format!("Invalid script identifier `{id}`")),
            )),
            | _ => None,
        };
        let error = self
            .path
            .trim()
            .is_empty()
            .then(|| ("path", ValidationError::new("required").with_message("Script path must not be empty")))
            .or(identifier_error)
            .or_else(|| {
                self.hooks.is_empty().then(|| {
                    (
                        "hooks",
                        ValidationError::new("required").with_message(format!("Script `{}` hooks must not be empty", self.path)),
                    )
                })
            })
            .or_else(|| {
                duplicate_hook.map(|hook| {
                    (
                        "hooks",
                        ValidationError::new("duplicate").with_message(format!("Duplicate hook `{hook}` in script `{}`", self.path)),
                    )
                })
            })
            .or_else(|| {
                (self.path.contains('*') || self.path.contains('?') || self.path.contains('[')).then(|| {
                    (
                        "path",
                        ValidationError::new("pattern").with_message(format!("Script path `{}` must not contain glob patterns", self.path)),
                    )
                })
            })
            .or_else(|| {
                (self.path.split('/').any(|segment| segment == "..") || self.path.split('\\').any(|segment| segment == "..")).then(|| {
                    (
                        "path",
                        ValidationError::new("traversal").with_message(format!("Script path `{}` must not contain `..`", self.path)),
                    )
                })
            })
            .or_else(|| {
                let unix_prefix = format!(".{APPLICATION}/");
                let windows_prefix = format!(".{APPLICATION}\\");
                (!self.absolute && (self.path.starts_with(&unix_prefix) || self.path.starts_with(&windows_prefix))).then(|| {
                    (
                        "path",
                        ValidationError::new("root").with_message(format!(
                            "Script path `{}` must be relative to `.{APPLICATION}/` without repeating the directory prefix",
                            self.path,
                        )),
                    )
                })
            })
            .or_else(|| {
                (!self.absolute && path.is_absolute()).then(|| {
                    (
                        "absolute",
                        ValidationError::new("required").with_message(format!("Absolute path `{}` requires `absolute: true`", self.path)),
                    )
                })
            })
            .or_else(|| {
                (self.absolute && !path.is_absolute()).then(|| {
                    (
                        "path",
                        ValidationError::new("absolute").with_message(format!("`absolute: true` requires absolute path, found `{}`", self.path)),
                    )
                })
            })
            .or_else(|| {
                invalid_extension.map(|extension| {
                    (
                        "path",
                        ValidationError::new("extension").with_message(format!("Script path `{}` has unsupported extension .{extension}", self.path)),
                    )
                })
            });
        error.map_or(Ok(()), |(path, error)| Err(ValidationReport::from_error(path, error)))
    }
}
