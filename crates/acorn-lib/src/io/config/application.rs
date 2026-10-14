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
        mcp::config::McpServerConfiguration,
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
    util::{detect_json, split_list, MimeType},
    validation::ValidationError,
    Location, Scheme,
};
use acorn_host::{config::resolve_config_path, terminal::Label};
use acorn_schema::{
    codec::{CodecDiagnostic, DocumentCodec},
    validation::{is_identifier, Validate, ValidationReport},
    OneOrMany,
};
use alloc::collections::{BTreeMap, BTreeSet};
use bon::Builder;
use color_eyre::eyre::{eyre, Report};
use core::iter::once;
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
    use acorn_core::OneOrMany;
    use acorn_service::WorkflowAction;
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
        /// GitLab generic-package name.
        pub package: String,
        /// GitLab project identifier.
        pub gitlab_project: String,
        /// Optional reporting timezone.
        #[serde(default)]
        pub timezone: Option<String>,
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
        pub actions: OneOrMany<WorkflowAction>,
        /// External effects allowed for this workflow.
        pub effects: OneOrMany<WorkflowEffect>,
        /// Input formats allowed for this workflow.
        pub formats: OneOrMany<MimeType>,
        /// Output behavior.
        pub outputs: WorkflowOutputs,
        /// Repository-relative input path patterns.
        pub paths: OneOrMany<String>,
    }
    /// Application-configuration source for executable-hook policy.
    #[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
    #[serde(untagged)]
    pub enum ScriptManifestSource {
        /// Inline executable-hook manifest.
        ///
        /// Boxed because `OneOrMany` adds a discriminant that pushes this enum past clippy's
        /// `large_enum_variant` threshold.
        Inline(Box<ScriptManifest>),
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
        pub hosts: OneOrMany<String>,
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
        pub hooks: OneOrMany<super::Hook>,
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
        pub scripts: OneOrMany<ScriptAuthorization>,
    }
}
/// Root attributes governed by `options.scripts`.
pub const SCRIPT_OPTION_ROOTS: [&str; 2] = ["scripts", "scriptManifest"];
/// Root attributes that have no `options` counterpart and are excluded from option coverage.
pub const UNGOVERNED_ROOT_ATTRIBUTES: [&str; 3] = ["schemaVersion", "config", "options"];
/// Supported Application Configuration for Research Enablement schema versions.
///
/// Version 1 documents are not supported. Migrate them to 2.0.0 before use.
#[derive(Clone, Copy, Debug, Default, Eq, eserde::Deserialize, JsonSchema, PartialEq, Serialize)]
pub enum AcreSchemaVersion {
    /// ACRE schema version 2.0.0.
    #[default]
    #[serde(rename = "2.0.0")]
    V2_0_0,
}
/// Reference to one configured bucket, by unique identifier or zero-based array index.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum BucketReference {
    /// Unique bucket identifier.
    Identifier(String),
    /// Zero-based index into the configured `buckets` array.
    Index(usize),
}
/// Resolved destination for one folder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Destination {
    /// Absolute local filesystem path.
    Local(PathBuf),
    /// Repository-relative path inside the configured GitLab project.
    GitLab {
        /// GitLab project identifier owning the file.
        project: String,
        /// Repository-relative path to the index file.
        path: String,
    },
    /// Absolute remote URI.
    Remote(String),
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
///     "schemaVersion": "2.0.0",
///     "buckets": [
///         {
///             "name": "big-science",
///             "source": {
///                 "provider": "github",
///                 "uri": "https://github.com/username/example"
///             }
///         },
///         {
///             "name": "does-it-scale",
///             "source": {
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
    /// ACRE schema version. Omitted files resolve to 2.0.0.
    #[serde(default, rename = "schemaVersion")]
    pub schema_version: AcreSchemaVersion,
    /// List of buckets
    #[eserde(compat)]
    #[serde(default)]
    pub buckets: Option<OneOrMany<Bucket>>,
    /// Synchronization targets for llama-swap, OpenCode, VS Code, and Goose
    #[eserde(compat)]
    #[serde(default)]
    pub config: Option<sync::Config>,
    /// Shared destination registry for intake and reporting routing
    #[eserde(compat)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folders: Option<OneOrMany<Folder>>,
    /// Global intake member allowlist applying to every configured folder
    #[eserde(compat)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members: Option<OneOrMany<String>>,
    /// Environment-variable references shared across the document
    #[eserde(compat)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<VariableReferences>,
    /// List of endpoints
    #[eserde(compat)]
    #[serde(default)]
    pub endpoints: Option<OneOrMany<Endpoint>>,
    /// Named outbound MCP servers and their explicit tool allowlists.
    #[eserde(compat)]
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mcp: BTreeMap<String, McpServerConfiguration>,
    /// List of models to download — bare IDs/paths or detailed download entries
    #[eserde(compat)]
    #[serde(default)]
    pub models: Option<OneOrMany<ModelEntry>>,
    /// Command defaults that may be overridden explicitly on the command line.
    #[eserde(compat)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<ApplicationOptions>,
    /// Named OCI registry profiles referenced by model entries.
    #[eserde(compat)]
    #[schemars(with = "Option<std::collections::BTreeMap<String, RegistryProfile>>")]
    #[serde(default)]
    pub registries: Option<HashMap<String, RegistryProfile>>,
    /// List of runners
    #[eserde(compat)]
    #[serde(default)]
    pub runners: Option<OneOrMany<RunnerDetails>>,
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
    pub scripts: Option<OneOrMany<Entry>>,
}
/// Command defaults stored in application configuration.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationOptions {
    /// Defaults used when importing metadata through a crosswalk.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crosswalk: Option<CrosswalkOptions>,
    /// Shared destination resolution for folder paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folders: Option<FolderOptions>,
    /// Deployment-owned PowerAutomate form intake and callback configuration.
    #[serde(default, rename = "powerAutomate", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<feature_configuration::PowerAutomateConfig>")]
    pub power_automate: Option<PowerAutomateConfig>,
    /// Options governing both root `scripts` and root `scriptManifest`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scripts: Option<ScriptOptions>,
}
/// Local identity defaults required by metadata imports.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CrosswalkOptions {
    /// JSON or YAML contact file, resolved relative to the configuration file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact: Option<PathBuf>,
    /// Local RAD identifier assigned to imported metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_identifier: Option<String>,
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
    pub hooks: OneOrMany<Hook>,
    /// When true, path is a platform-native absolute path.
    #[serde(default)]
    pub absolute: bool,
}
/// One registered destination folder shared by intake and reporting.
#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct Folder {
    /// Stable identifier. Aliases and the default path derive from this value.
    pub id: String,
    /// Routing aliases, including folder names, that resolve to this folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aliases: Option<OneOrMany<String>>,
    /// Destination bucket this folder belongs to. Association never alters path precedence or grants access.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bucket: Option<BucketReference>,
    /// Inline member allowlist for intake authorization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members: Option<OneOrMany<String>>,
    /// Destination index path, relative unless it is an absolute URI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Environment-variable references scoped to this folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<VariableReferences>,
}
/// Shared destination resolution settings.
#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct FolderOptions {
    /// Absolute local or remote URI used as the base for relative folder paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
}
/// Immutable routing snapshot derived from the ACRE document.
///
/// This is the routing half of [`ApplicationConfiguration`]: owned, `Send`, and `Sync`, so intake services can hold it without the JSONC comment-preserving tree.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FolderRegistry {
    folders: Vec<Folder>,
    members: Option<OneOrMany<String>>,
    project: Option<String>,
    root: Option<String>,
    variables: Option<VariableReferences>,
}
/// One folder resolved against its configured destination.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedFolder {
    /// The matched folder.
    pub folder: Folder,
    /// Destination derived from `options.folders.root`, the GitLab project, or the path itself.
    pub destination: Destination,
}
/// Options affecting both root `scripts` and root `scriptManifest`.
///
/// Root `scripts` registers hooks and root `scriptManifest` authorizes them.
/// They keep separate representations; this single option entry governs both so the two cannot drift apart.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptOptions {
    /// Whether script hooks may run. Both root attributes are ignored when false.
    #[serde(default = "scripts_enabled_by_default")]
    pub enabled: bool,
}
/// Environment-variable references declared at the document root or on one folder.
#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct VariableReferences {
    /// Name of one environment variable holding the member allowlist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members: Option<String>,
    /// Operator-defined references keyed by catalog name, valued by environment-variable name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub custom: BTreeMap<String, String>,
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
    pub buckets: Option<OneOrMany<String>>,
    /// Whitelisted model IDs/URLs, or one URL to a JSON whitelist document
    pub models: Option<OneOrMany<String>>,
}
impl ApplicationConfiguration {
    /// Return the routing snapshot consumed by intake and reporting services.
    pub fn registry(&self) -> FolderRegistry {
        FolderRegistry {
            folders: self.folders.clone().unwrap_or_default().into_vec(),
            members: self.members.clone(),
            project: self.power_automate().map(|automation| automation.gitlab_project.clone()),
            root: self
                .options
                .as_ref()
                .and_then(|options| options.folders.as_ref())
                .and_then(|folders| folders.root.clone()),
            variables: self.variables.clone(),
        }
    }
}
impl ApplicationConfiguration {
    /// Return whether any configured script subscribes to the requested hook.
    pub fn has_hook(&self, hook: Hook) -> bool {
        let enabled = self
            .options
            .as_ref()
            .and_then(|options| options.scripts.as_ref())
            .is_none_or(|scripts| scripts.enabled);
        let subscribed = self
            .scripts
            .as_deref()
            .unwrap_or_default()
            .iter()
            .any(|entry| entry.hooks.contains(&hook));
        enabled && subscribed
    }
    /// Return the configured PowerAutomate integration settings.
    pub fn power_automate(&self) -> Option<&PowerAutomateConfig> {
        self.options.as_ref().and_then(|options| options.power_automate.as_ref())
    }
    /// Return the bucket referenced by one folder association.
    ///
    /// A reference never alters path precedence or grants access; it only names the owning bucket.
    pub fn bucket(&self, reference: &BucketReference) -> ApiResult<&Bucket> {
        let buckets = self.buckets.as_ref().map(OneOrMany::as_slice).unwrap_or_default();
        let bucket = match reference {
            | BucketReference::Identifier(identifier) => buckets.iter().find(|bucket| bucket.name.as_deref() == Some(identifier.as_str())),
            | BucketReference::Index(index) => buckets.get(*index),
        };
        bucket.ok_or_else(|| eyre!("Bucket reference '{reference:?}' does not resolve to a configured bucket"))
    }
    /// Return every configured folder.
    pub fn folders(&self) -> impl Iterator<Item = &Folder> {
        self.folders.iter().flat_map(OneOrMany::iter)
    }
    /// Return the environment variable naming one built-in root variable reference.
    pub fn variable(&self, name: &str) -> Option<&str> {
        self.variables
            .as_ref()
            .and_then(|variables| variables.custom.get(name))
            .map(String::as_str)
    }
    /// Return the folder matching a canonical identifier or one configured alias.
    pub fn folder(&self, selector: &str) -> ApiResult<&Folder> {
        match self.folders().filter(|folder| folder.selects(selector)).collect::<Vec<_>>().as_slice() {
            | [folder] => Ok(folder),
            | [] => Err(eyre!("Unknown folder or alias '{selector}'")),
            | _ => Err(eyre!("Ambiguous folder selector '{selector}'")),
        }
    }
    /// Return one folder resolved against its configured destination.
    pub fn resolve_folder(&self, selector: &str) -> ApiResult<ResolvedFolder> {
        self.folder(selector).and_then(|folder| {
            let root = self
                .options
                .as_ref()
                .and_then(|options| options.folders.as_ref())
                .and_then(|folders| folders.root.clone());
            let project = self.power_automate().map(|automation| automation.gitlab_project.as_str());
            let path = folder.path.clone().unwrap_or_else(|| format!("{}/index.json", folder.id));
            Destination::new(&path, root.as_deref(), project).map(|destination| ResolvedFolder {
                folder: folder.clone(),
                destination,
            })
        })
    }
    /// Return the effective intake members for one folder, or for the whole document when `selector` is `None`.
    ///
    /// Inline and variable-sourced members are unioned, lowercased, and deduplicated.
    pub fn effective_members(&self, selector: Option<&str>, lookup: &dyn Fn(&str) -> Option<String>) -> BTreeSet<String> {
        let scoped = selector.and_then(|selector| self.folder(selector).ok());
        [self.members.as_ref(), scoped.and_then(|folder| folder.members.as_ref())]
            .into_iter()
            .flatten()
            .flat_map(OneOrMany::iter)
            .cloned()
            .chain(
                [self.variables.as_ref(), scoped.and_then(|folder| folder.variables.as_ref())]
                    .into_iter()
                    .flatten()
                    .filter_map(|variables| variables.members.as_deref())
                    .filter_map(lookup)
                    .flat_map(split_list),
            )
            .map(|member| member.to_lowercase())
            .collect()
    }
    /// Return whether one submitter may submit to the folder selected by `selector`.
    pub fn authorizes(&self, selector: &str, email: &str, lookup: &dyn Fn(&str) -> Option<String>) -> ApiResult<bool> {
        self.folder(selector).map(|folder| {
            let submitter = email.trim().to_lowercase();
            self.effective_members(Some(&folder.id), lookup).contains(&submitter)
        })
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
            self.models.clone().unwrap_or_default().into_vec(),
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
                configuration.models = Some(OneOrMany::Many(models));
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
        path.as_ref()
            .filter(|value| value.is_file())
            .cloned()
            .or_else(|| resolve_config_path(None, directory, &DEFAULT_CONFIG_FILENAMES))
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
#[cfg(feature = "analysis")]
impl DocumentCodec for ApplicationConfiguration {
    const NAME: &'static str = "ACRE configuration";
    fn decode(content: &str, format: &MimeType) -> Result<Self, Vec<CodecDiagnostic>> {
        match format {
            | MimeType::Json | MimeType::Jsonc | MimeType::Yaml => {
                Self::parse(content).map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())])
            }
            | _ => Err(vec![CodecDiagnostic::unsupported(format, Self::NAME)]),
        }
    }
    fn encode(&self, format: &MimeType) -> Result<String, CodecDiagnostic> {
        match format {
            | MimeType::Json | MimeType::Jsonc => serde_json::to_string_pretty(self).map_err(|error| CodecDiagnostic::serialize(error.to_string())),
            | MimeType::Yaml => serde_norway::to_string(self).map_err(|error| CodecDiagnostic::serialize(error.to_string())),
            | _ => Err(CodecDiagnostic::unsupported(format, Self::NAME)),
        }
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
        if let Some(power_automate) = self.power_automate() {
            if let Err(errors) = power_automate.validate() {
                report.append_prefixed("options.powerAutomate", errors);
            }
        }
        let folders = self.folders().cloned().collect::<Vec<_>>();
        folders.iter().enumerate().for_each(|(index, folder)| {
            if !is_identifier(&folder.id) {
                report.add(
                    format!("folders[{index}].id"),
                    ValidationError::new("identifier").with_message("Folder identifier must be a safe stable identifier"),
                );
            }
            if let Some(bucket) = folder.bucket.as_ref() {
                if let Err(why) = self.bucket(bucket) {
                    report.add(
                        format!("folders[{index}].bucket"),
                        ValidationError::new("bucket").with_message(why.to_string()),
                    );
                }
            }
            if let Err(why) = self.resolve_folder(&folder.id) {
                report.add(
                    format!("folders[{index}].path"),
                    ValidationError::new("destination").with_message(why.to_string()),
                );
            }
        });
        folders.iter().enumerate().fold(HashSet::new(), |mut identifiers, (index, folder)| {
            if !identifiers.insert(&folder.id) {
                report.add(
                    format!("folders[{index}].id"),
                    ValidationError::new("duplicate").with_message(format!("Duplicate folder identifier '{}'", folder.id)),
                );
            }
            identifiers
        });
        folders
            .iter()
            .flat_map(|folder| once(&folder.id).chain(folder.aliases.as_ref().into_iter().flat_map(OneOrMany::iter)))
            .fold(HashSet::new(), |mut selectors, selector| {
                if !selectors.insert(selector) {
                    report.add(
                        "folders",
                        ValidationError::new("duplicate").with_message(format!("Folder selector '{selector}' resolves to more than one folder")),
                    );
                }
                selectors
            });
        self.mcp.iter().for_each(|(name, server)| {
            if let Err(errors) = McpServerConfiguration::validate_name(name) {
                report.append_prefixed(&format!("mcp.{name}"), errors);
            }
            if let Err(errors) = server.validate() {
                report.append_prefixed(&format!("mcp.{name}"), errors);
            }
        });
        self.options
            .as_ref()
            .and_then(|options| options.crosswalk.as_ref())
            .into_iter()
            .for_each(|options| {
                if options.contact.as_ref().is_some_and(|path| path.as_os_str().is_empty()) {
                    report.add(
                        "options.crosswalk.contact",
                        ValidationError::new("required").with_message("Crosswalk contact path must not be empty"),
                    );
                }
                if options.local_identifier.as_deref().is_some_and(|identifier| !is_identifier(identifier)) {
                    report.add(
                        "options.crosswalk.localIdentifier",
                        ValidationError::new("format").with_message("Crosswalk local identifier must be a valid identifier"),
                    );
                }
            });
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
impl Destination {
    /// Return the repository-relative path when this destination lives in the configured GitLab project.
    pub fn repository_path(&self) -> Option<&str> {
        match self {
            | Self::GitLab { path, .. } => Some(path.as_str()),
            | Self::Local(_) | Self::Remote(_) => None,
        }
    }
    /// Return the destination for one folder path against its configured root and project.
    ///
    /// An absolute URI names its own destination. A relative path resolves against
    /// `options.folders.root`, then inside `options.powerAutomate.gitlab_project`.
    pub fn new(path: &str, root: Option<&str>, project: Option<&str>) -> ApiResult<Self> {
        let location = Location::from(path);
        match location.scheme() {
            | Scheme::File | Scheme::HTTP | Scheme::HTTPS => location.try_into(),
            | Scheme::Unsupported if location.is_path() => Self::from_relative(path, root, project),
            | scheme => Err(eyre!("Unsupported folder URI scheme '{scheme}' — '{path}'")),
        }
    }
    /// Return the destination for one relative folder path against its configured root and project.
    fn from_relative(path: &str, root: Option<&str>, project: Option<&str>) -> ApiResult<Self> {
        match (root, project) {
            | (Some(root), _) => safe_relative(path).and_then(|relative| {
                let resolved = format!("{}/{relative}", root.trim_end_matches('/'));
                Location::from(resolved.as_str()).try_into()
            }),
            | (None, Some(project)) => safe_relative(path).map(|relative| Self::GitLab {
                project: project.to_owned(),
                path: relative.to_owned(),
            }),
            | (None, None) => Err(eyre!(
                "Folder path '{path}' requires options.folders.root or options.powerAutomate.gitlab_project"
            )),
        }
    }
}
impl TryFrom<Location> for Destination {
    type Error = Report;

    /// Return the destination named by one absolute folder URI.
    ///
    /// `file` URIs become local paths, keeping a Windows UNC authority, and HTTP(S) URIs stay remote.
    fn try_from(uri: Location) -> Result<Self, Self::Error> {
        match uri.scheme() {
            | Scheme::File => Ok(Self::Local(file_path(&uri))),
            | Scheme::HTTP | Scheme::HTTPS => Ok(Self::Remote(uri.to_string())),
            | _ => Err(eyre!("Folder destinations require a file, http, or https URI — '{uri}'")),
        }
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
impl Folder {
    /// Return whether this folder answers to a canonical identifier or one configured alias.
    pub fn selects(&self, selector: &str) -> bool {
        self.id == selector || self.aliases.as_ref().is_some_and(|aliases| aliases.iter().any(|alias| alias == selector))
    }
}
impl FolderRegistry {
    /// Return every configured folder.
    pub fn folders(&self) -> impl Iterator<Item = &Folder> {
        self.folders.iter()
    }
    /// Return the environment variable naming one built-in root variable reference.
    pub fn variable(&self, name: &str) -> Option<&str> {
        self.variables
            .as_ref()
            .and_then(|variables| variables.custom.get(name))
            .map(String::as_str)
    }
    /// Return the folder matching a canonical identifier or one configured alias.
    pub fn folder(&self, selector: &str) -> ApiResult<&Folder> {
        match self.folders().filter(|folder| folder.selects(selector)).collect::<Vec<_>>().as_slice() {
            | [folder] => Ok(folder),
            | [] => Err(eyre!("Unknown folder or alias '{selector}'")),
            | _ => Err(eyre!("Ambiguous folder selector '{selector}'")),
        }
    }
    /// Return one folder resolved against its configured destination.
    pub fn resolve_folder(&self, selector: &str) -> ApiResult<ResolvedFolder> {
        self.folder(selector).and_then(|folder| {
            let path = folder.path.clone().unwrap_or_else(|| format!("{}/index.json", folder.id));
            Destination::new(&path, self.root.as_deref(), self.project.as_deref()).map(|destination| ResolvedFolder {
                folder: folder.clone(),
                destination,
            })
        })
    }
    /// Return the effective intake members for one folder, or for the whole document when `selector` is `None`.
    ///
    /// Inline and variable-sourced members are unioned, lowercased, and deduplicated.
    pub fn effective_members(&self, selector: Option<&str>, lookup: &dyn Fn(&str) -> Option<String>) -> BTreeSet<String> {
        let scoped = selector.and_then(|selector| self.folder(selector).ok());
        [self.members.as_ref(), scoped.and_then(|folder| folder.members.as_ref())]
            .into_iter()
            .flatten()
            .flat_map(OneOrMany::iter)
            .cloned()
            .chain(
                [self.variables.as_ref(), scoped.and_then(|folder| folder.variables.as_ref())]
                    .into_iter()
                    .flatten()
                    .filter_map(|variables| variables.members.as_deref())
                    .filter_map(lookup),
            )
            .flat_map(split_list)
            .map(|member| member.to_lowercase())
            .collect()
    }
    /// Return whether one submitter may submit to the folder selected by `selector`.
    pub fn authorizes(&self, selector: &str, email: &str, lookup: &dyn Fn(&str) -> Option<String>) -> ApiResult<bool> {
        self.folder(selector).map(|folder| {
            let submitter = email.trim().to_lowercase();
            self.effective_members(Some(&folder.id), lookup).contains(&submitter)
        })
    }
}
impl Default for ScriptOptions {
    fn default() -> Self {
        Self { enabled: true }
    }
}
/// Return the local filesystem path named by one `file` location.
///
/// A non-empty authority is a Windows UNC host, so it joins the path as a `\\host` prefix.
fn file_path(uri: &Location) -> PathBuf {
    let path = uri.path().unwrap_or_default();
    let host = uri.host().unwrap_or_default();
    match host.is_empty() {
        | true => PathBuf::from(path),
        | false => PathBuf::from(format!(r"\\{host}{path}")),
    }
}
/// Return one folder path as a canonical repository-relative path.
fn safe_relative(path: &str) -> ApiResult<&str> {
    match Location::from(path).is_canonical_repository_path() {
        | true => Ok(path),
        | false => Err(eyre!("Folder path '{path}' must be a safe relative path")),
    }
}
fn scripts_enabled_by_default() -> bool {
    true
}
