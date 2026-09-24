//! Needle runner and ModelKit asset acquisition
use super::super::{
    DistributionAsset, DistributionEngine, DistributionManifest, ModelKitManifest, ResolvedAssets, Sidecar, SidecarAssets, SidecarDistribution,
    ToolIndex,
};
use super::Needle;
use crate::{
    io::{file_checksum, read_file, ApiResult, Fingerprint},
    util::constants::app::{NEEDLE_REPOSITORY, NEEDLE_REVISION},
};
use acorn_core::prelude::{Box, String};
use color_eyre::eyre::eyre;
use serde::{de::DeserializeOwned, Deserialize};
use std::{
    fs::{copy, create_dir_all, remove_file, rename},
    path::{Path, PathBuf},
};

const LICENSE_ASSET_NAME: &str = "license";
const MODEL_ASSET_NAME: &str = "model";
/// One pinned official standalone runner
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EngineAsset {
    os: String,
    architecture: String,
    /// Upstream platform folder.
    pub platform: String,
    /// Executable file name.
    pub executable: String,
    /// Expected byte size.
    pub size: u64,
    /// Expected SHA-256 digest.
    pub sha256: String,
}
/// Pinned runner and supporting assets for the official Needle distribution.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NeedleAssetManifest {
    engines: Vec<EngineAsset>,
    supporting: Vec<PinnedAsset>,
}
/// Needle-specific persistent and session-local tool-index paths.
#[derive(Debug)]
pub struct NeedleToolIndex {
    cached: PathBuf,
    session: PathBuf,
}
/// One pinned supporting asset in the official Needle distribution.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PinnedAsset {
    name: String,
    path: String,
    size: u64,
    sha256: String,
}
impl DistributionEngine for EngineAsset {
    fn architecture(&self) -> &str {
        &self.architecture
    }
    fn executable(&self) -> &str {
        &self.executable
    }
    fn os(&self) -> &str {
        &self.os
    }
    fn platform(&self) -> &str {
        &self.platform
    }
    fn sha256(&self) -> &str {
        &self.sha256
    }
    fn size(&self) -> u64 {
        self.size
    }
}
impl SidecarAssets for Needle {
    type AssetManifest = NeedleAssetManifest;
    type EngineAsset = EngineAsset;

    const ASSET_FILE_NAME: &'static str = "needle.json";
    const KIND: &'static str = "needle";
    const PROTOCOL_VERSION: u32 = 2;
    fn validate_modelkit(manifest: &ModelKitManifest) -> ApiResult<()> {
        let required = |name: &str| {
            manifest.assets.iter().find(|asset| asset.name == name).ok_or_else(|| {
                eyre!(
                    "Sidecar '{}' protocol {} ModelKit requires a named '{name}' asset",
                    Self::KIND,
                    Self::PROTOCOL_VERSION
                )
            })
        };
        match manifest.sidecar.protocol_version {
            | version if version < Self::PROTOCOL_VERSION => Err(eyre!(
                "Sidecar '{}' protocol {version} ModelKits are no longer supported; rebuild this bundle for protocol {} with a runner, .cact model, and license",
                Self::KIND,
                Self::PROTOCOL_VERSION
            )),
            | Self::PROTOCOL_VERSION => required(MODEL_ASSET_NAME).and_then(|model| match model.path.ends_with(".cact") {
                | true => required(LICENSE_ASSET_NAME).map(|_| ()),
                | false => Err(eyre!(
                    "Sidecar '{}' protocol {} ModelKit model asset must use the .cact format",
                    Self::KIND,
                    Self::PROTOCOL_VERSION
                )),
            }),
            | version => Err(eyre!(
                "Unsupported sidecar '{}' ModelKit protocol version {version}; expected {}",
                Self::KIND,
                Self::PROTOCOL_VERSION
            )),
        }
    }
}
impl SidecarDistribution for Needle {
    const REPOSITORY: &'static str = NEEDLE_REPOSITORY;
    const REVISION: &'static str = NEEDLE_REVISION;
}
impl DistributionManifest for NeedleAssetManifest {
    type Engine = EngineAsset;
    type Supporting = PinnedAsset;

    fn engines(&self) -> &[Self::Engine] {
        &self.engines
    }
    fn supporting(&self) -> &[Self::Supporting] {
        &self.supporting
    }
}
impl ToolIndex for NeedleToolIndex {
    fn new(cached: &Path, session_root: &Path) -> Self {
        Self {
            cached: cached.to_path_buf(),
            session: session_root.join("tools.idx"),
        }
    }
    fn prepare(&self) -> ApiResult<()> {
        match self.cached.exists() {
            | true => copy(&self.cached, &self.session)
                .map(|_| ())
                .map_err(|why| eyre!("Failed to prepare the Needle session tool index — {why}")),
            | false => Ok(()),
        }
    }
    fn publish(&self) -> ApiResult<()> {
        let suffix = self
            .session
            .parent()
            .and_then(Path::file_name)
            .and_then(|value| value.to_str())
            .unwrap_or("session");
        let part = self.cached.with_extension(format!("idx.{suffix}.part"));
        match (self.cached.exists(), self.session.exists()) {
            | (true, _) => Ok(()),
            | (_, false) => Err(eyre!("Needle did not create its session tool index")),
            | (false, true) => copy(&self.session, &part)
                .map_err(|why| eyre!("Failed to stage the Needle tool index cache — {why}"))
                .and_then(|_| match rename(&part, &self.cached) {
                    | Ok(()) => Ok(()),
                    | Err(_) if self.cached.exists() => {
                        remove_file(&part).map_err(|why| eyre!("Failed to remove a redundant Needle tool index staging file — {why}"))
                    }
                    | Err(why) => {
                        let _ = remove_file(&part);
                        Err(eyre!("Failed to publish the Needle tool index cache — {why}"))
                    }
                }),
        }
    }
    fn session(&self) -> &Path {
        &self.session
    }
}
impl DistributionAsset for PinnedAsset {
    fn name(&self) -> &str {
        &self.name
    }
    fn path(&self) -> &str {
        &self.path
    }
    fn sha256(&self) -> &str {
        &self.sha256
    }
    fn size(&self) -> u64 {
        self.size
    }
}
/// Resolve a persistent retrieval index keyed by the engine, model, protocol, and tool catalog.
pub(super) fn tool_index<S: Sidecar>(assets: &ResolvedAssets, catalog: &str, session_root: &Path) -> ApiResult<Box<dyn ToolIndex>> {
    let identity = assets
        .assets
        .get(MODEL_ASSET_NAME)
        .ok_or_else(|| eyre!("Sidecar '{}' model asset is unavailable", S::KIND))
        .and_then(|model| {
            file_checksum(&assets.runner, None).map_err(|why| eyre!(why)).and_then(|runner| {
                file_checksum(model, None).map_err(|why| eyre!(why)).map(|model| {
                    Fingerprint::from_bytes(format!(
                        "{}:{}:{}:{catalog}",
                        S::PROTOCOL_VERSION,
                        runner.checksum_value,
                        model.checksum_value
                    ))
                })
            })
        });
    S::cache_root().map(|root| root.join("indexes")).and_then(|root| {
        create_dir_all(&root)
            .map_err(|why| eyre!("Failed to create the '{}' sidecar tool-index cache — {why}", S::KIND))
            .and_then(|()| {
                identity.map(|identity| Box::new(S::ToolIndex::new(&root.join(format!("{identity}.idx")), session_root)) as Box<dyn ToolIndex>)
            })
    })
}
/// Require a custom ModelKit to carry the exact active tool catalog.
pub(super) fn validate_tools<S, T>(assets: ResolvedAssets, expected: &[T]) -> ApiResult<ResolvedAssets>
where
    S: SidecarAssets,
    T: DeserializeOwned + PartialEq,
{
    match assets.tools.clone() {
        | None => Ok(assets),
        | Some(path) => read_file(path)
            .and_then(|content| {
                serde_json::from_str::<Vec<T>>(&content).map_err(|why| eyre!("Invalid sidecar '{}' ModelKit tools catalog — {why}", S::KIND))
            })
            .and_then(|actual| match actual == expected {
                | true => Ok(assets),
                | false => Err(eyre!(
                    "Sidecar '{}' ModelKit tools catalog does not match ACORN's active catalog",
                    S::KIND
                )),
            }),
    }
}
