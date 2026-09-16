//! Deployment policy for capability-brokered Rhai hooks.
use crate::io::{jsonc_parse_value, read_file, read_file_snapshot, ApiResult, FromPath, SafePath};
use crate::util::constants::app::{APPLICATION, ORGANIZATION, QUALIFIER};
use acorn_core::prelude::{String, Vec};
use acorn_core::util::constants::app::{
    MAX_EXEC_CAPABILITY_CALLS, MAX_EXEC_NETWORK_REQUESTS, MAX_EXEC_OPERATIONS, MAX_EXEC_STRING_SIZE, MAX_EXEC_TOTAL_BYTES,
    MAX_EXEC_WALL_TIME_MILLISECONDS, SUPPORTED_EXEC_VERSION,
};
use acorn_core::util::MimeType;
use acorn_core::validation::ValidationError;
use acorn_schema::validation::{is_identifier, Validate, ValidationReport};
use alloc::collections::BTreeMap;
use color_eyre::eyre::{eyre, Report};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::Metadata;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Current executable-hook manifest schema version.
pub const SCRIPT_MANIFEST_VERSION: &str = "1.0.0";
/// Application-configuration source for executable-hook policy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ScriptManifestSource {
    /// Inline manifest, trusted for effects only from an explicitly selected configuration.
    Inline(ScriptManifest),
    /// Safe relative manifest name beneath the deployment manifest directory.
    Path(String),
}
/// Filesystem access granted beneath named directory roots.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FilesystemCapability {
    /// Maximum aggregate bytes transferred through this capability per invocation.
    #[serde(default = "default_total_bytes")]
    pub max_bytes: usize,
    /// Maximum distinct files inspected or proposed per invocation.
    #[serde(default = "default_file_count")]
    pub max_files: usize,
    /// Named absolute directory roots.
    pub roots: BTreeMap<String, PathBuf>,
}
/// Network access granted to exact HTTPS authorities.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NetworkCapability {
    /// Exact host names, optionally followed by `:port`.
    pub hosts: Vec<String>,
    /// Maximum requests per invocation.
    #[serde(default = "default_network_requests")]
    pub max_requests: usize,
    /// Maximum response bytes per request.
    #[serde(default = "default_string_bytes")]
    pub max_response_bytes: usize,
    /// Per-request timeout in milliseconds.
    #[serde(default = "default_network_timeout")]
    pub timeout: u64,
}
/// Resolved manifest and its trust provenance.
#[derive(Clone, Debug)]
pub struct ResolvedManifest {
    /// Validated manifest.
    pub manifest: ScriptManifest,
    /// Human-readable policy source for audit records.
    pub source: String,
}
/// Per-script evaluation ceilings.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ResourceLimits {
    /// Maximum registered host capability calls.
    pub max_calls: usize,
    /// Maximum Rhai operations.
    pub max_operations: u64,
    /// Maximum wall-clock runtime in milliseconds.
    pub max_runtime: u64,
}
/// One digest-pinned script authorization.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
    #[serde(default = "default_entry")]
    pub entry: String,
    /// Workflow hooks this script may handle.
    pub hooks: Vec<crate::io::config::application::Hook>,
    /// Stable identifier matched from application configuration.
    pub id: String,
    /// Evaluation ceilings that may only narrow host defaults.
    #[serde(default)]
    pub limits: ResourceLimits,
}
/// Capabilities granted to one script.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ScriptCapabilities {
    /// Named filesystem read roots.
    pub read: Option<FilesystemCapability>,
    /// Named filesystem write roots.
    pub write: Option<FilesystemCapability>,
    /// Permission to replace the current proposed workflow document.
    pub mutation: bool,
    /// Restricted outbound HTTPS access.
    pub network: Option<NetworkCapability>,
}
/// Versioned deployment manifest for executable hooks.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScriptManifest {
    /// Manifest schema version.
    pub version: String,
    /// Authorized scripts.
    pub scripts: Vec<ScriptAuthorization>,
}
impl Validate for FilesystemCapability {
    fn validate(&self) -> Result<(), ValidationReport> {
        let roots = !self.roots.is_empty()
            && self.roots.iter().all(|(name, root)| {
                !name.trim().is_empty()
                    && root.is_absolute()
                    && root
                        .symlink_metadata()
                        .ok()
                        .is_some_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
            });
        let bytes = self.max_bytes <= MAX_EXEC_TOTAL_BYTES;
        let files = self.max_files <= MAX_EXEC_CAPABILITY_CALLS;
        match (roots, bytes, files) {
            | (true, true, true) => Ok(()),
            | _ => Err(ValidationReport::from_error(
                "filesystem",
                ValidationError::new("capability").with_message("Invalid filesystem capability roots or limits"),
            )),
        }
    }
}
impl Validate for NetworkCapability {
    fn validate(&self) -> Result<(), ValidationReport> {
        let hosts = !self.hosts.is_empty()
            && self.hosts.iter().all(|host| {
                let candidate = format!("https://{host}");
                reqwest::Url::parse(&candidate)
                    .ok()
                    .and_then(|url| url.host_str().map(str::to_string))
                    .is_some_and(|parsed| !parsed.is_empty() && !host.contains(['/', '@']))
            });
        let requests = self.max_requests <= MAX_EXEC_NETWORK_REQUESTS;
        let response = self.max_response_bytes <= MAX_EXEC_STRING_SIZE;
        let timeout = self.timeout <= MAX_EXEC_WALL_TIME_MILLISECONDS;
        match (hosts, requests, response, timeout) {
            | (true, true, true, true) => Ok(()),
            | _ => Err(ValidationReport::from_error(
                "network",
                ValidationError::new("capability").with_message("Invalid network capability allowlist or limits"),
            )),
        }
    }
}
impl TryFrom<&str> for ResolvedManifest {
    type Error = Report;
    fn try_from(name: &str) -> Result<Self, Self::Error> {
        SafePath::new(name)
            .map_err(|why| eyre!("invalid deployment manifest name — {why}"))
            .and_then(|relative| deployment_manifest_directory().map(|root| root.join(relative.as_path())))
            .and_then(|path| {
                deployment_manifest_directory()
                    .and_then(|root| {
                        root.canonicalize()
                            .map_err(|why| eyre!("failed to resolve deployment manifest directory — {why}"))
                    })
                    .and_then(|root| {
                        read_file_snapshot(&path, MAX_EXEC_TOTAL_BYTES)
                            .map_err(|why| eyre!("failed to read deployment manifest `{}` — {why}", path.display()))
                            .and_then(|(canonical, content, metadata)| match canonical.starts_with(&root) {
                                | false => Err(eyre!("deployment manifest escapes `{}`", root.display())),
                                | true => validate_manifest_permissions(&path, &metadata).map(|()| (canonical, content)),
                            })
                    })
                    .and_then(|(canonical, content)| ScriptManifest::parse(&canonical, &content).map(|manifest| (canonical, manifest)))
                    .and_then(|(canonical, manifest)| Validate::validate(&manifest).map_err(Report::new).map(|()| (canonical, manifest)))
                    .map(|(canonical, manifest)| Self {
                        manifest,
                        source: canonical.display().to_string(),
                    })
            })
    }
}
impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_calls: MAX_EXEC_CAPABILITY_CALLS,
            max_operations: MAX_EXEC_OPERATIONS,
            max_runtime: MAX_EXEC_WALL_TIME_MILLISECONDS,
        }
    }
}
impl Validate for ResourceLimits {
    fn validate(&self) -> Result<(), ValidationReport> {
        let calls = self.max_calls <= MAX_EXEC_CAPABILITY_CALLS;
        let operations = self.max_operations <= MAX_EXEC_OPERATIONS;
        let runtime = self.max_runtime <= MAX_EXEC_WALL_TIME_MILLISECONDS;
        match (calls, operations, runtime) {
            | (true, true, true) => Ok(()),
            | _ => Err(ValidationReport::from_error(
                "limits",
                ValidationError::new("range").with_message("Script resource limits exceed ACORN host ceilings"),
            )),
        }
    }
}
impl ScriptAuthorization {
    /// Return the names of effects granted by this authorization.
    pub fn granted_effects(&self) -> Vec<String> {
        let effects = [
            self.capabilities.read.as_ref().map(|_| "read"),
            self.capabilities.write.as_ref().map(|_| "write"),
            self.capabilities.mutation.then_some("mutation"),
            self.capabilities.network.as_ref().map(|_| "network"),
        ];
        effects.into_iter().flatten().map(str::to_string).collect()
    }
    /// Return whether this authorization grants any host effect.
    pub const fn is_effectful(&self) -> bool {
        self.capabilities.read.is_some() || self.capabilities.write.is_some() || self.capabilities.mutation || self.capabilities.network.is_some()
    }
}
impl Validate for ScriptAuthorization {
    fn validate(&self) -> Result<(), ValidationReport> {
        let hooks = self.hooks.iter().collect::<HashSet<_>>();
        let valid_identity = is_identifier(&self.id)
            && self.version == SUPPORTED_EXEC_VERSION.to_string()
            && self.entry == "evaluate"
            && acorn_core::validation::digest(&self.digest).is_ok()
            && !self.hooks.is_empty()
            && hooks.len() == self.hooks.len();
        match valid_identity {
            | false => Err(ValidationReport::from_error(
                "authorization",
                ValidationError::new("invalid").with_message(format!("Invalid script authorization `{}`", self.id)),
            )),
            | true => self
                .limits
                .validate()
                .and_then(|()| self.capabilities.read.as_ref().map_or(Ok(()), Validate::validate))
                .and_then(|()| self.capabilities.write.as_ref().map_or(Ok(()), Validate::validate))
                .and_then(|()| self.capabilities.network.as_ref().map_or(Ok(()), Validate::validate)),
        }
    }
}
impl ScriptManifest {
    /// Parse a manifest from JSON, JSONC, or YAML content according to its path.
    pub fn parse(path: &Path, content: &str) -> ApiResult<Self> {
        match MimeType::from_path(path) {
            | MimeType::Cff | MimeType::Yaml => serde_norway::from_str(content).map_err(|why| eyre!("failed to parse script manifest — {why}")),
            | _ => jsonc_parse_value(content)
                .and_then(|value| serde_json::from_value(value).map_err(|why| eyre!("failed to parse script manifest — {why}"))),
        }
    }
    /// Read and validate a manifest from an explicitly supplied local path.
    pub fn read(path: &Path) -> ApiResult<Self> {
        read_file(path)
            .map_err(|why| eyre!("failed to read script manifest `{}` — {why}", path.display()))
            .and_then(|content| Self::parse(path, &content))
            .and_then(|manifest| Validate::validate(&manifest).map_err(Report::new).map(|()| manifest))
    }
    /// Find one authorization by stable script identifier.
    pub fn find(&self, id: &str) -> Option<&ScriptAuthorization> {
        self.scripts.iter().find(|script| script.id == id)
    }
}
impl Validate for ScriptManifest {
    fn validate(&self) -> Result<(), ValidationReport> {
        let unique = self.scripts.iter().map(|script| script.id.as_str()).collect::<HashSet<_>>().len() == self.scripts.len();
        match (self.version == SCRIPT_MANIFEST_VERSION, unique) {
            | (false, _) => Err(ValidationReport::from_error(
                "version",
                ValidationError::new("version").with_message(format!(
                    "Unsupported script manifest version `{}` — supported is `{SCRIPT_MANIFEST_VERSION}`",
                    self.version
                )),
            )),
            | (_, false) => Err(ValidationReport::from_error(
                "scripts",
                ValidationError::new("duplicate").with_message("Script manifest contains duplicate script identifiers"),
            )),
            | _ => self.scripts.validate(),
        }
    }
}
impl ScriptManifestSource {
    /// Resolve and validate a manifest using application-configuration provenance.
    pub fn resolve(&self, explicit_configuration: bool) -> ApiResult<ResolvedManifest> {
        match self {
            | Self::Inline(manifest) => match explicit_configuration || manifest.scripts.iter().all(|script| !script.is_effectful()) {
                | true => Validate::validate(manifest).map_err(Report::new).map(|()| ResolvedManifest {
                    manifest: manifest.clone(),
                    source: "inline".to_string(),
                }),
                | false => Err(eyre!(
                    "an inline script manifest may grant effects only when configuration is selected with `--config`"
                )),
            },
            | Self::Path(name) => ResolvedManifest::try_from(name.as_str()),
        }
    }
}
fn default_entry() -> String {
    "evaluate".to_string()
}
fn default_file_count() -> usize {
    MAX_EXEC_CAPABILITY_CALLS
}
fn default_network_requests() -> usize {
    MAX_EXEC_NETWORK_REQUESTS
}
fn default_network_timeout() -> u64 {
    10_000
}
fn default_string_bytes() -> usize {
    MAX_EXEC_STRING_SIZE
}
fn default_total_bytes() -> usize {
    MAX_EXEC_TOTAL_BYTES
}
fn deployment_manifest_directory() -> ApiResult<PathBuf> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|directories| directories.config_dir().join("manifests"))
        .ok_or_else(|| eyre!("ACORN deployment manifest directory is unavailable"))
}
#[cfg(unix)]
fn validate_manifest_permissions(path: &Path, metadata: &Metadata) -> ApiResult<()> {
    match metadata.permissions().mode() & 0o022 == 0 {
        | true => Ok(()),
        | false => Err(eyre!("deployment manifest `{}` is group/world-writable", path.display())),
    }
}
#[cfg(not(unix))]
fn validate_manifest_permissions(_path: &Path, _metadata: &Metadata) -> ApiResult<()> {
    Ok(())
}
