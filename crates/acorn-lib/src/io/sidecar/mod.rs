//! Managed subprocess sidecars
use crate::io::{read_file, verify_checksum, write_file, ApiResult, InputOutput};
use acorn_core::prelude::{canonicalize, Component, Path, PathBuf, String, Vec};
use acorn_host::fs::make_executable;
use acorn_schema::modelkit::kitfile::MANIFEST_VERSION;
use alloc::collections::BTreeMap;
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

pub mod needle;

/// Common lifecycle for a managed local inference sidecar.
#[async_trait]
pub trait Sidecar: Sized + Send {
    /// Result returned by one sidecar inference request.
    type Output;

    /// Resolve assets and start the managed sidecar process.
    async fn start(custom_location: Option<&str>, offline: bool) -> ApiResult<Self>;
    /// Run one request and consume the managed sidecar process, when supported.
    async fn infer(self, _prompt: String) -> Option<ApiResult<Self::Output>> {
        None
    }
}
/// Named supporting file declared by a sidecar ModelKit.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ModelKitAsset {
    /// Stable asset name used by sidecar adapters.
    pub name: String,
    /// Bundle-relative file path.
    pub path: String,
    /// Expected file size in bytes.
    pub size: u64,
    /// Expected SHA-256 digest.
    pub sha256: String,
}
/// Runner file declared by a sidecar ModelKit.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ModelKitFile {
    /// Bundle-relative file path.
    pub path: String,
    /// Expected file size in bytes.
    pub size: u64,
    /// Expected SHA-256 digest.
    pub sha256: String,
}
/// OCI-compatible metadata for an ACORN sidecar ModelKit.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ModelKitManifest {
    /// Kitfile-compatible manifest format version.
    pub manifest_version: String,
    /// Upstream sidecar source revision.
    pub source_revision: String,
    /// Platform folder name.
    pub platform: String,
    /// Sidecar identity and process protocol.
    pub sidecar: ModelKitSidecar,
    /// Sidecar runner file.
    pub runner: ModelKitFile,
    /// Named files required by the sidecar.
    #[serde(default)]
    pub assets: Vec<ModelKitAsset>,
    /// Optional name of the asset containing the tool catalog.
    #[serde(default)]
    pub tools: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ModelKitPlan {
    pub(super) assets: BTreeMap<String, PlannedFile>,
    pub(super) runner: PlannedFile,
    sidecar: ModelKitSidecar,
    tools_asset: Option<String>,
}
/// Sidecar identity declared by a ModelKit.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ModelKitSidecar {
    /// Stable sidecar kind.
    pub kind: String,
    /// Process protocol version implemented by the runner.
    pub protocol_version: u32,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PlannedFile {
    pub(super) path: String,
    pub(super) sha256: String,
    pub(super) size: u64,
}
/// Validated executable assets used to start a sidecar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedAssets {
    /// Validated named assets supplied by the bundle.
    pub assets: BTreeMap<String, PathBuf>,
    /// Sidecar executable path.
    pub runner: PathBuf,
    /// Optional tool catalog supplied by a custom bundle.
    pub tools: Option<PathBuf>,
    /// Validated bundle or cache root.
    pub root: PathBuf,
}
impl ModelKitManifest {
    /// Validate and resolve this manifest within its materialized bundle root.
    pub fn resolve(&self, root: &Path, expected_platform: &str, expected_sidecar: &str, expected_protocol: u32) -> ApiResult<ResolvedAssets> {
        self.plan(expected_platform, expected_sidecar, expected_protocol)
            .and_then(|plan| plan.resolve(root))
    }
    pub(super) fn plan(&self, expected_platform: &str, expected_sidecar: &str, expected_protocol: u32) -> ApiResult<ModelKitPlan> {
        let sidecar = &self.sidecar;
        sidecar
            .validate_manifest(self, expected_platform, expected_sidecar, expected_protocol)
            .and_then(|()| sidecar.validate_file(&self.runner.path, &self.runner.sha256, "runner"))
            .and_then(|()| match self.runner.size {
                | 0 => Err(eyre!("{} ModelKit runner cannot be empty", sidecar.kind)),
                | _ => self.assets.iter().try_fold(BTreeMap::new(), |mut assets, asset| {
                    let valid_name = !asset.name.is_empty() && asset.name.trim() == asset.name;
                    let duplicate_name = assets.contains_key(&asset.name);
                    let duplicate_path = asset.path == self.runner.path || assets.values().any(|file: &PlannedFile| file.path == asset.path);
                    match (valid_name, duplicate_name, duplicate_path) {
                        | (false, _, _) => Err(eyre!("{} ModelKit asset names must be non-empty and trimmed", sidecar.kind)),
                        | (_, true, _) => Err(eyre!("{} ModelKit contains duplicate asset name '{}'", sidecar.kind, asset.name)),
                        | (_, _, true) => Err(eyre!("{} ModelKit contains duplicate file path '{}'", sidecar.kind, asset.path)),
                        | (true, false, false) => sidecar
                            .validate_file(&asset.path, &asset.sha256, &format!("asset '{}'", asset.name))
                            .map(|()| {
                                assets.insert(
                                    asset.name.clone(),
                                    PlannedFile {
                                        path: asset.path.clone(),
                                        sha256: asset.sha256.clone(),
                                        size: asset.size,
                                    },
                                );
                                assets
                            }),
                    }
                }),
            })
            .and_then(|assets| match self.tools.as_ref() {
                | Some(name) if !assets.contains_key(name) => Err(eyre!("{} ModelKit tools asset '{name}' is not declared", sidecar.kind)),
                | _ => Ok(ModelKitPlan {
                    assets,
                    runner: PlannedFile {
                        path: self.runner.path.clone(),
                        sha256: self.runner.sha256.clone(),
                        size: self.runner.size,
                    },
                    sidecar: sidecar.clone(),
                    tools_asset: self.tools.clone(),
                }),
            })
    }
}
impl InputOutput for ModelKitManifest {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        Self::read_json(path.into())
    }
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        read_file(path).and_then(|content| serde_json::from_str(&content).map_err(|why| eyre!("Invalid ModelKit manifest — {why}")))
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        read_file(path).and_then(|content| serde_norway::from_str(&content).map_err(|why| eyre!("Invalid ModelKit manifest — {why}")))
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_json(path)
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        serde_json::to_string_pretty(self)
            .map_err(|why| eyre!("Failed to serialize ModelKit manifest — {why}"))
            .and_then(|content| write_file(path, content))
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        serde_norway::to_string(self)
            .map_err(|why| eyre!("Failed to serialize ModelKit manifest — {why}"))
            .and_then(|content| write_file(path, content))
    }
}
impl ModelKitPlan {
    fn resolve(self, root: &Path) -> ApiResult<ResolvedAssets> {
        let sidecar = &self.sidecar;
        canonicalize(root)
            .map_err(|why| eyre!("Failed to resolve {} ModelKit root '{}' — {why}", sidecar.kind, root.display()))
            .and_then(|root| {
                sidecar.resolve_planned_file(&root, &self.runner, "runner").and_then(|runner| {
                    self.assets
                        .iter()
                        .map(|(name, file)| {
                            sidecar
                                .resolve_planned_file(&root, file, &format!("asset '{name}'"))
                                .map(|path| (name.clone(), path))
                        })
                        .collect::<ApiResult<BTreeMap<_, _>>>()
                        .and_then(|assets| {
                            let tools = match self.tools_asset.as_ref() {
                                | Some(name) => assets
                                    .get(name)
                                    .cloned()
                                    .ok_or_else(|| eyre!("{} ModelKit tools asset '{name}' is not resolved", sidecar.kind))
                                    .map(Some),
                                | None => Ok(None),
                            };
                            tools.and_then(|tools| {
                                make_executable(&runner)
                                    .map_err(|why| eyre!("{} ModelKit runner is not executable — {why}", sidecar.kind))
                                    .map(|()| ResolvedAssets { assets, runner, tools, root })
                            })
                        })
                })
            })
    }
}
impl ModelKitSidecar {
    fn resolve_planned_file(&self, root: &Path, file: &PlannedFile, label: &str) -> ApiResult<PathBuf> {
        self.resolve_relative(root, &file.path)
            .and_then(|path| self.verify_size(&path, file.size, label).map(|()| path))
            .and_then(|path| verify_checksum(&path, &file.sha256, None).map(|()| path))
    }
    fn resolve_relative(&self, root: &Path, value: &str) -> ApiResult<PathBuf> {
        self.validate_relative(value).and_then(|()| {
            canonicalize(root.join(Path::new(value)))
                .map_err(|why| eyre!("Failed to resolve {} ModelKit path '{value}' — {why}", self.kind))
                .and_then(|path| match path.starts_with(root) {
                    | true => Ok(path),
                    | false => Err(eyre!("{} ModelKit path escapes its bundle root: {value}", self.kind)),
                })
        })
    }
    fn validate_file(&self, path: &str, sha256: &str, label: &str) -> ApiResult<()> {
        self.validate_relative(path).and_then(|()| {
            let digest_valid = sha256.len() == 64 && sha256.bytes().all(|byte| byte.is_ascii_hexdigit());
            match digest_valid {
                | true => Ok(()),
                | false => Err(eyre!(
                    "{} ModelKit {label} SHA-256 must contain exactly 64 hexadecimal characters",
                    self.kind
                )),
            }
        })
    }
    fn validate_manifest(
        &self,
        manifest: &ModelKitManifest,
        expected_platform: &str,
        expected_sidecar: &str,
        expected_protocol: u32,
    ) -> ApiResult<()> {
        let version_matches = manifest.manifest_version == MANIFEST_VERSION;
        let has_revision = !manifest.source_revision.trim().is_empty();
        let platform_matches = manifest.platform == expected_platform;
        match (version_matches, has_revision, platform_matches) {
            | (false, _, _) => Err(eyre!(
                "{expected_sidecar} ModelKit manifestVersion must be {MANIFEST_VERSION}, got {}",
                manifest.manifest_version
            )),
            | (_, false, _) => Err(eyre!("{expected_sidecar} ModelKit sourceRevision cannot be empty")),
            | (_, _, false) => Err(eyre!(
                "{expected_sidecar} ModelKit platform '{}' does not match this host ('{expected_platform}')",
                manifest.platform
            )),
            | (true, true, true) => {
                let kind_matches = self.kind == expected_sidecar;
                let protocol_matches = self.protocol_version == expected_protocol;
                match (kind_matches, protocol_matches) {
                    | (false, _) => Err(eyre!(
                        "{expected_sidecar} ModelKit sidecar '{}' does not match the expected kind",
                        self.kind
                    )),
                    | (_, false) => Err(eyre!(
                        "{expected_sidecar} ModelKit protocol {} does not match the expected version ({expected_protocol})",
                        self.protocol_version
                    )),
                    | (true, true) => Ok(()),
                }
            }
        }
    }
    fn validate_relative(&self, value: &str) -> ApiResult<()> {
        let relative = Path::new(value);
        let path_is_safe = !value.trim().is_empty() && relative.components().all(|component| matches!(component, Component::Normal(_)));
        match path_is_safe {
            | true => Ok(()),
            | false => Err(eyre!("{} ModelKit path must be a safe relative path: {value}", self.kind)),
        }
    }
    fn verify_size(&self, path: &Path, expected: u64, label: &str) -> ApiResult<()> {
        path.metadata()
            .map_err(|why| eyre!("Failed to inspect {} ModelKit {label} '{}' — {why}", self.kind, path.display()))
            .and_then(|metadata| match metadata.len() == expected {
                | true => Ok(()),
                | false => Err(eyre!(
                    "{} ModelKit {label} size mismatch for {} (expected {expected}, got {})",
                    self.kind,
                    path.display(),
                    metadata.len()
                )),
            })
    }
}
