//! Managed subprocess sidecars
use crate::{
    io::{
        api::huggingface,
        download::{DownloadItem, DownloadItems},
        oci::{OciReference, OciTransportOptions, OrasTransport},
        read_file, verify_checksum, write_file, ApiResult, Fingerprint, InputOutput, Source,
    },
    util::{
        constants::{
            app::{APPLICATION, ORGANIZATION, QUALIFIER},
            oci::MODELKIT_MANIFEST,
        },
        Constant,
    },
};
use acorn_core::prelude::{canonicalize, Component, Path, PathBuf, String, Vec};
use acorn_core::util::assets::EmbeddedAssets;
use acorn_host::fs::{make_executable, SafePath};
use acorn_schema::modelkit::kitfile::MANIFEST_VERSION;
use alloc::collections::BTreeMap;
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use core::{fmt, iter::once};
use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    env::consts::{ARCH, OS},
    fs::remove_file,
};

#[cfg(feature = "needle")]
pub mod needle;
#[cfg(feature = "slm")]
pub mod slm;

/// Read-only view of a supporting asset in an official sidecar distribution.
pub trait DistributionAsset: Send {
    /// Stable asset name.
    fn name(&self) -> &str;
    /// Repository-relative asset path.
    fn path(&self) -> &str;
    /// Expected SHA-256 digest.
    fn sha256(&self) -> &str;
    /// Expected byte size.
    fn size(&self) -> u64;
}
/// Read-only view of a sidecar engine in an official distribution.
pub trait DistributionEngine: Clone + Send {
    /// Target architecture.
    fn architecture(&self) -> &str;
    /// Executable file name.
    fn executable(&self) -> &str;
    /// Target operating system.
    fn os(&self) -> &str;
    /// Distribution platform folder.
    fn platform(&self) -> &str;
    /// Expected SHA-256 digest.
    fn sha256(&self) -> &str;
    /// Expected byte size.
    fn size(&self) -> u64;
}
/// Read-only view of an official sidecar distribution manifest.
pub trait DistributionManifest: Send {
    /// Engine entry type.
    type Engine: DistributionEngine;
    /// Supporting asset entry type.
    type Supporting: DistributionAsset;

    /// Available engine entries.
    fn engines(&self) -> &[Self::Engine];
    /// Supporting asset entries.
    fn supporting(&self) -> &[Self::Supporting];
}
/// Complete runtime contract for a managed local inference sidecar.
///
/// New sidecars implement this trait and its asset and distribution supertraits in their own module.
/// The default [`Sidecar::start`] implementation resolves verified assets before delegating process startup to [`Sidecar::start_with_assets`].
#[async_trait]
pub trait Sidecar: SidecarAssets + SidecarDistribution {
    /// Typed request accepted by this sidecar.
    type Input: Send;
    /// Typed result returned by this sidecar.
    type Output: Send;
    /// Session-local tool-index implementation used by this sidecar.
    type ToolIndex: ToolIndex + 'static;

    /// Resolve verified assets and start the managed sidecar process.
    async fn start(custom_location: Option<&str>, offline: bool) -> ApiResult<Self> {
        match Self::resolve_assets(custom_location, offline, true).await {
            | Ok(assets) => Self::start_with_assets(assets, offline).await,
            | Err(why) => Err(why),
        }
    }
    /// Start the managed process from already verified assets.
    async fn start_with_assets(assets: ResolvedAssets, offline: bool) -> ApiResult<Self>;
    /// Run one typed request and consume the managed sidecar process.
    async fn infer(self, input: Self::Input) -> ApiResult<Self::Output>;
}
/// Identity and asset-acquisition contract shared by managed sidecars.
#[async_trait]
pub trait SidecarAssets: Sized + Send {
    /// Typed pinned-asset manifest used to acquire the official sidecar distribution.
    type AssetManifest: DeserializeOwned + DistributionManifest<Engine = Self::EngineAsset> + Send;
    /// Typed runner asset selected from the pinned distribution.
    type EngineAsset: DistributionEngine;

    /// Embedded asset filename containing the official distribution manifest.
    const ASSET_FILE_NAME: &'static str;
    /// Stable sidecar kind stored in ModelKit manifests and cache paths.
    const KIND: &'static str;
    /// Process protocol version accepted by this sidecar implementation.
    const PROTOCOL_VERSION: u32;

    /// Resolve the persistent cache root owned by this sidecar.
    fn cache_root() -> ApiResult<PathBuf> {
        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .map(|directories| directories.cache_dir().join(Self::KIND))
            .ok_or_else(|| eyre!("Failed to resolve the ACORN cache directory"))
    }
    /// Resolve the pinned runner for an operating system and architecture.
    fn for_target(os: &str, architecture: &str) -> ApiResult<Self::EngineAsset> {
        Self::load_asset_manifest().and_then(|assets| {
            assets
                .engines()
                .iter()
                .find(|asset| asset.os() == os && asset.architecture() == architecture)
                .cloned()
                .ok_or_else(|| eyre!("Sidecar '{}' standalone inference is not available for {os}/{architecture}", Self::KIND))
        })
    }
    /// Return the ModelKit platform name for a runner asset.
    fn platform(asset: &Self::EngineAsset) -> &str {
        asset.platform()
    }
    /// Load the pinned asset manifest for the official sidecar distribution.
    fn load_asset_manifest() -> ApiResult<Self::AssetManifest> {
        Constant::from_asset(Self::ASSET_FILE_NAME)
            .ok_or_else(|| eyre!("Embedded sidecar '{}' asset manifest is unavailable", Self::KIND))
            .and_then(|content| {
                serde_json::from_str(&content).map_err(|why| eyre!("Invalid embedded sidecar '{}' asset manifest — {why}", Self::KIND))
            })
    }
    /// Resolve and verify the runner and supporting assets for this sidecar.
    async fn resolve_assets(custom_location: Option<&str>, offline: bool, quiet: bool) -> ApiResult<ResolvedAssets>
    where
        Self: SidecarDistribution,
    {
        match custom_location.map(str::trim).filter(|value| !value.is_empty()) {
            | None => Self::resolve_official(offline, quiet).await,
            | Some(location) => match Source::parse(location) {
                | Source::Remote { .. } if offline => Err(eyre!("Cannot download sidecar '{}' ModelKit while offline", Self::KIND)),
                | source @ (Source::Local { .. } | Source::Remote { .. }) => Self::resolve_location(source, quiet).await,
                | Source::Unsupported(identifier) if identifier.starts_with("oci://") && offline => {
                    Err(eyre!("Cannot pull sidecar '{}' ModelKit from OCI while offline", Self::KIND))
                }
                | Source::Unsupported(identifier) if identifier.starts_with("oci://") => Self::resolve_oci(&identifier),
                | Source::Unsupported(identifier) => Err(eyre!("Unsupported sidecar '{}' ModelKit URI '{identifier}'", Self::KIND)),
            },
        }
    }
    /// Resolve a ModelKit from a local path or remote manifest.
    async fn resolve_location(location: Source, quiet: bool) -> ApiResult<ResolvedAssets> {
        match location {
            | Source::Local { path, .. } => canonicalize(&path)
                .map_err(|why| eyre!("Failed to resolve local sidecar '{}' ModelKit {} — {why}", Self::KIND, path.display()))
                .and_then(from_materialized_modelkit::<Self>),
            | Source::Remote { identifier, .. } => match Source::read(&identifier, false)
                .await
                .and_then(|content| serde_json::from_str::<ModelKitManifest>(&content).map_err(|why| eyre!("Invalid ModelKit manifest — {why}")))
            {
                | Ok(manifest) => {
                    let assets_and_downloads = Self::for_target(OS, ARCH).and_then(|asset| {
                        modelkit_download_items::<Self>(&identifier, &manifest, Self::platform(&asset)).map(|downloads| (asset, downloads))
                    });
                    match assets_and_downloads {
                        | Ok((asset, (root, items))) => DownloadItems::new(&root, items, quiet, false)
                            .download()
                            .await
                            .and_then(|()| canonicalize(root).map_err(|why| eyre!("Failed to resolve downloaded ModelKit — {why}")))
                            .and_then(|root| Self::resolve_modelkit(&manifest, &root, Self::platform(&asset))),
                        | Err(why) => Err(why),
                    }
                }
                | Err(why) => Err(why),
            },
            | Source::Unsupported(identifier) => Err(eyre!("Unsupported sidecar '{}' ModelKit URI '{identifier}'", Self::KIND)),
        }
    }
    /// Validate and resolve a materialized ModelKit for this sidecar.
    fn resolve_modelkit(manifest: &ModelKitManifest, root: &Path, expected_platform: &str) -> ApiResult<ResolvedAssets> {
        Self::validate_modelkit(manifest).and_then(|()| manifest.resolve_for::<Self>(root, expected_platform))
    }
    /// Resolve the pinned official distribution for this sidecar.
    async fn resolve_official(offline: bool, quiet: bool) -> ApiResult<ResolvedAssets>
    where
        Self: SidecarDistribution,
    {
        let resolved = Self::for_target(OS, ARCH).and_then(|asset| {
            Self::load_asset_manifest().and_then(|manifest| {
                Self::cache_root().map(|root| {
                    let root = root.join(Self::REVISION).join(asset.platform());
                    let runner = root.join(asset.executable());
                    let assets = manifest
                        .supporting()
                        .iter()
                        .map(|asset| (asset.name().to_string(), root.join(asset.path())))
                        .collect();
                    (
                        ResolvedAssets {
                            assets,
                            runner,
                            tools: None,
                            root,
                        },
                        asset,
                    )
                })
            })
        });
        match resolved {
            | Ok((assets, asset)) => {
                let files_exist = assets.runner.exists() && assets.assets.values().all(|path| path.exists());
                match (offline, files_exist) {
                    | (true, false) => Err(eyre!(
                        "Sidecar '{}' protocol v{} distribution is not cached and ACORN is offline — {}",
                        Self::KIND,
                        Self::PROTOCOL_VERSION,
                        assets.root.display()
                    )),
                    | (_, false) => download_official::<Self>(assets, asset, quiet).await,
                    | (_, true) => match Self::validate_assets(&assets, &asset) {
                        | Ok(()) => Ok(assets),
                        | Err(_) if !offline => match clear_official::<Self>(&assets) {
                            | Ok(()) => download_official::<Self>(assets, asset, quiet).await,
                            | Err(why) => Err(why),
                        },
                        | Err(why) => Err(why),
                    },
                }
            }
            | Err(why) => Err(why),
        }
    }
    /// Resolve an OCI ModelKit for this sidecar.
    fn resolve_oci(location: &str) -> ApiResult<ResolvedAssets> {
        OciReference::parse(location).and_then(|reference| {
            let identity = Fingerprint::from_bytes(location).to_string();
            Self::cache_root()
                .map(|root| root.join("oci").join(identity))
                .and_then(|root| match root.exists() {
                    | true => from_materialized_modelkit::<Self>(root),
                    | false => OrasTransport::new(OciTransportOptions::default())
                        .and_then(|transport| transport.pull_artifact(&reference, &root))
                        .and_then(|()| from_materialized_modelkit::<Self>(root)),
                })
        })
    }
    /// Validate sidecar-specific ModelKit requirements beyond the shared manifest contract.
    fn validate_modelkit(manifest: &ModelKitManifest) -> ApiResult<()>;
}
/// Official distribution behavior supplied by a concrete sidecar implementation.
pub trait SidecarDistribution: SidecarAssets {
    /// Upstream repository containing the official distribution.
    const REPOSITORY: &'static str;
    /// Immutable upstream revision used for official downloads and cache identity.
    const REVISION: &'static str;

    /// Build downloads for the pinned runner and its supporting assets.
    fn download_items(asset: &Self::EngineAsset) -> ApiResult<Vec<DownloadItem>> {
        let runner = huggingface::resolve_url(Self::REPOSITORY, Self::REVISION, &format!("{}/{}", asset.platform(), asset.executable())).map(|url| {
            DownloadItem::init()
                .url(url)
                .path(asset.executable().to_string())
                .size(asset.size())
                .sha(asset.sha256().to_string())
                .build()
        });
        let supporting = Self::load_asset_manifest().and_then(|assets| {
            assets
                .supporting()
                .iter()
                .map(|asset| {
                    huggingface::resolve_url(Self::REPOSITORY, Self::REVISION, asset.path()).map(|url| {
                        DownloadItem::init()
                            .url(url)
                            .path(asset.path().to_string())
                            .size(asset.size())
                            .sha(asset.sha256().to_string())
                            .build()
                    })
                })
                .collect::<ApiResult<Vec<_>>>()
        });
        runner.and_then(|runner| supporting.map(|supporting| once(runner).chain(supporting).collect()))
    }
    /// Validate materialized files from the pinned distribution.
    fn validate_assets(assets: &ResolvedAssets, asset: &Self::EngineAsset) -> ApiResult<()> {
        assets
            .runner
            .metadata()
            .map_err(|why| {
                eyre!(
                    "Failed to inspect cached sidecar '{}' protocol {} runner {} — {why}",
                    Self::KIND,
                    Self::PROTOCOL_VERSION,
                    assets.runner.display()
                )
            })
            .and_then(|metadata| match metadata.len() == asset.size() {
                | true => verify_checksum(&assets.runner, asset.sha256(), None),
                | false => Err(eyre!(
                    "Cached sidecar '{}' protocol {} runner size mismatch (expected {}, got {})",
                    Self::KIND,
                    Self::PROTOCOL_VERSION,
                    asset.size(),
                    metadata.len()
                )),
            })
            .and_then(|()| {
                Self::load_asset_manifest().and_then(|manifest| {
                    manifest.supporting().iter().try_for_each(|asset| {
                        assets
                            .assets
                            .get(asset.name())
                            .ok_or_else(|| {
                                eyre!(
                                    "Sidecar '{}' protocol {} asset '{}' is missing",
                                    Self::KIND,
                                    Self::PROTOCOL_VERSION,
                                    asset.name()
                                )
                            })
                            .and_then(|path| {
                                path.metadata()
                                    .map_err(|why| {
                                        eyre!(
                                            "Failed to inspect cached sidecar '{}' protocol {} {} {} — {why}",
                                            Self::KIND,
                                            Self::PROTOCOL_VERSION,
                                            asset.name(),
                                            path.display()
                                        )
                                    })
                                    .and_then(|metadata| match metadata.len() == asset.size() {
                                        | true => verify_checksum(path, asset.sha256(), None),
                                        | false => Err(eyre!(
                                            "Cached sidecar '{}' protocol {} {} size mismatch (expected {}, got {})",
                                            Self::KIND,
                                            Self::PROTOCOL_VERSION,
                                            asset.name(),
                                            asset.size(),
                                            metadata.len()
                                        )),
                                    })
                            })
                    })
                })
            })
            .and_then(|()| make_executable(&assets.runner).map_err(Into::into))
    }
}
/// Session-local tool index with an optional persistent cache.
pub trait ToolIndex: fmt::Debug + Send {
    /// Create index paths for a persistent cache and an isolated session.
    fn new(cached: &Path, session_root: &Path) -> Self
    where
        Self: Sized;
    /// Seed the session-local index from the persistent cache when available.
    fn prepare(&self) -> ApiResult<()>;
    /// Publish a completed session index to the persistent cache.
    fn publish(&self) -> ApiResult<()>;
    /// Return the session-local index path passed to the sidecar.
    fn session(&self) -> &Path;
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
impl ModelKitManifest {
    /// Validate and resolve this manifest within its materialized bundle root.
    pub fn resolve(&self, root: &Path, expected_platform: &str, expected_sidecar: &str, expected_protocol: u32) -> ApiResult<ResolvedAssets> {
        self.plan(expected_platform, expected_sidecar, expected_protocol)
            .and_then(|plan| plan.resolve(root))
    }
    /// Validate and resolve this manifest for a declared sidecar contract.
    pub fn resolve_for<S: SidecarAssets>(&self, root: &Path, expected_platform: &str) -> ApiResult<ResolvedAssets> {
        self.resolve(root, expected_platform, S::KIND, S::PROTOCOL_VERSION)
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
fn clear_official<S: SidecarAssets>(assets: &ResolvedAssets) -> ApiResult<()> {
    once(&assets.runner)
        .chain(assets.assets.values())
        .filter(|path| path.exists())
        .try_for_each(|path| {
            remove_file(path).map_err(|why| eyre!("Failed to replace invalid cached '{}' sidecar asset {} — {why}", S::KIND, path.display()))
        })
}
async fn download_official<S: SidecarDistribution>(assets: ResolvedAssets, asset: S::EngineAsset, quiet: bool) -> ApiResult<ResolvedAssets> {
    match S::download_items(&asset) {
        | Ok(items) => DownloadItems::new(&assets.root, items, quiet, false)
            .download()
            .await
            .and_then(|()| S::validate_assets(&assets, &asset))
            .map(|()| assets),
        | Err(why) => Err(why),
    }
}
fn from_materialized_modelkit<S: SidecarAssets>(root: PathBuf) -> ApiResult<ResolvedAssets> {
    ModelKitManifest::read(root.join(MODELKIT_MANIFEST))
        .and_then(|manifest| S::for_target(OS, ARCH).and_then(|asset| S::resolve_modelkit(&manifest, &root, S::platform(&asset))))
}
pub(super) fn modelkit_download_items<S: SidecarAssets>(
    location: &str,
    manifest: &ModelKitManifest,
    expected_platform: &str,
) -> ApiResult<(PathBuf, Vec<DownloadItem>)> {
    let base = location.rsplit_once('/').map(|(base, _)| base).unwrap_or(location);
    let root = S::cache_root().map(|root| root.join("remote").join(Fingerprint::from_bytes(location).to_string()));
    let paths = S::validate_modelkit(manifest)
        .and_then(|()| manifest.plan(expected_platform, S::KIND, S::PROTOCOL_VERSION))
        .and_then(|plan| {
            once(&plan.runner)
                .chain(plan.assets.values())
                .map(|file| match Source::parse(&file.path) {
                    | Source::Local { path: relative, .. } if SafePath::new(&relative).is_ok() => Ok(DownloadItem::init()
                        .url(format!("{base}/{}", file.path))
                        .path(file.path.clone())
                        .size(file.size)
                        .sha(file.sha256.clone())
                        .build()),
                    | _ => Err(eyre!("Sidecar '{}' ModelKit path must be a safe relative path: {}", S::KIND, file.path)),
                })
                .collect::<ApiResult<Vec<_>>>()
        });
    root.and_then(|root| paths.map(|paths| (root, paths)))
}
