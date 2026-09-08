//! Needle runner and ModelKit asset acquisition
use super::super::{ModelKitManifest, ResolvedAssets};
use crate::io::api::huggingface;
use crate::io::download::{DownloadItem, DownloadItems};
use crate::io::oci::{OciReference, OciTransportOptions, OrasTransport};
use crate::io::{make_executable, read_file, verify_checksum, ApiResult, Fingerprint, InputOutput, SafePath, Source};
use crate::util::constants::app::{APPLICATION, NEEDLE_REPOSITORY, NEEDLE_REVISION, ORGANIZATION, QUALIFIER};
use crate::util::constants::oci::MODELKIT_MANIFEST;
use acorn_core::prelude::String;
use acorn_schema::agent::tools::NeedleToolDefinition;
use color_eyre::eyre::eyre;
use core::iter::once;
use directories::ProjectDirs;
use std::env::consts::{ARCH, OS};
use std::fs::{canonicalize, create_dir_all, remove_file};
use std::path::PathBuf;

/// One pinned official standalone runner
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EngineAsset {
    /// Upstream platform folder.
    pub platform: &'static str,
    /// Executable file name.
    pub executable: &'static str,
    /// Expected byte size.
    pub size: u64,
    /// Expected SHA-256 digest.
    pub sha256: &'static str,
}
impl EngineAsset {
    /// Resolve the pinned runner for the current desktop platform.
    pub fn current() -> ApiResult<Self> {
        match (OS, ARCH) {
            | ("windows", "x86_64") => Ok(Self {
                platform: "windows-x86_64",
                executable: "needle.exe",
                size: 14_924_800,
                sha256: "2b5d0fb453fa85f345002a7053c067fa373dcc0b6e876ad7fa11f5691c3dbcdd",
            }),
            | ("windows", "aarch64") => Ok(Self {
                platform: "windows-arm64",
                executable: "needle.exe",
                size: 14_774_784,
                sha256: "609edc8e97cc6f77cfacdcd624242218450217dd943c1f807274d3d866e98c5a",
            }),
            | ("linux", "x86_64") => Ok(Self {
                platform: "linux-x86_64",
                executable: "needle",
                size: 14_884_256,
                sha256: "d5e86d6256e0c83670e36612f3ee3242a3cf6cf2b72e5b5fa079a711ac285af0",
            }),
            | ("linux", "aarch64") => Ok(Self {
                platform: "linux-arm64",
                executable: "needle",
                size: 14_815_112,
                sha256: "a75c4b37c7d04a62e0010d99b78063f6814f2c1208b26875fa2a1eac8bf3b992",
            }),
            | ("macos", "aarch64") => Ok(Self {
                platform: "macos-arm64",
                executable: "needle",
                size: 14_610_568,
                sha256: "ed86e8d6971a5a041fbfbc41923ce799997ca475df63fc794f5f95e990c4e428",
            }),
            | (os, architecture) => Err(eyre!("Needle standalone inference is not available for {os}/{architecture}")),
        }
    }
    /// Immutable Hugging Face URL for this runner.
    pub fn url(&self) -> ApiResult<String> {
        huggingface::resolve_url(NEEDLE_REPOSITORY, NEEDLE_REVISION, &format!("{}/{}", self.platform, self.executable))
    }
}
impl ResolvedAssets {
    async fn official(offline: bool, quiet: bool) -> ApiResult<Self> {
        let resolved = EngineAsset::current().and_then(|asset| {
            cache_root().map(|root| {
                let root = root.join(NEEDLE_REVISION).join(asset.platform);
                let runner = root.join(asset.executable);
                (Self { runner, tools: None, root }, asset)
            })
        });
        match resolved {
            | Ok((assets, _)) if offline && !assets.runner.exists() => {
                Err(eyre!("Needle runner is not cached and ACORN is offline — {}", assets.runner.display()))
            }
            | Ok((assets, asset)) if !assets.runner.exists() => assets.download(asset, quiet).await,
            | Ok((assets, asset)) => match assets.clone().validate_official(asset) {
                | Ok(valid) => Ok(valid),
                | Err(_) if !offline => match remove_file(&assets.runner) {
                    | Ok(()) => assets.download(asset, quiet).await,
                    | Err(why) => Err(eyre!("Failed to replace an invalid cached Needle runner — {why}")),
                },
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    async fn download(self, asset: EngineAsset, quiet: bool) -> ApiResult<Self> {
        match asset.url() {
            | Ok(url) => DownloadItems::new(
                &self.root,
                vec![DownloadItem::init()
                    .url(url)
                    .path(asset.executable.to_string())
                    .size(asset.size)
                    .sha(asset.sha256.to_string())
                    .build()],
                quiet,
                false,
            )
            .download()
            .await
            .and_then(|()| self.validate_official(asset)),
            | Err(why) => Err(why),
        }
    }
    fn from_oci(location: &str) -> ApiResult<Self> {
        OciReference::parse(location).and_then(|reference| {
            let identity = Fingerprint::from_bytes(location).to_string();
            cache_root()
                .map(|root| root.join("oci").join(identity))
                .and_then(|root| match root.exists() {
                    | true => Self::from_materialized_modelkit(root),
                    | false => OrasTransport::new(OciTransportOptions::default())
                        .and_then(|transport| transport.pull_artifact(&reference, &root))
                        .and_then(|()| Self::from_materialized_modelkit(root)),
                })
        })
    }
    async fn from_location(location: Source, quiet: bool) -> ApiResult<Self> {
        match location {
            | Source::Local { path, .. } => canonicalize(&path)
                .map_err(|why| eyre!("Failed to resolve local Needle ModelKit {} — {why}", path.display()))
                .and_then(Self::from_materialized_modelkit),
            | Source::Remote { identifier, .. } => match Source::read(&identifier, false)
                .await
                .and_then(|content| serde_json::from_str::<ModelKitManifest>(&content).map_err(|why| eyre!("Invalid ModelKit manifest — {why}")))
            {
                | Ok(manifest) => match download_items(&identifier, &manifest) {
                    | Ok((root, items)) => DownloadItems::new(&root, items, false, quiet)
                        .download()
                        .await
                        .and_then(|()| canonicalize(root).map_err(|why| eyre!("Failed to resolve downloaded ModelKit — {why}")))
                        .and_then(|root| EngineAsset::current().and_then(|asset| manifest.resolve(&root, asset.platform, "Needle"))),
                    | Err(why) => Err(why),
                },
                | Err(why) => Err(why),
            },
            | Source::Unsupported(identifier) => Err(eyre!("Unsupported Needle ModelKit URI '{identifier}'")),
        }
    }
    fn from_materialized_modelkit(root: PathBuf) -> ApiResult<Self> {
        ModelKitManifest::read(root.join(MODELKIT_MANIFEST))
            .and_then(|manifest| EngineAsset::current().and_then(|asset| manifest.resolve(&root, asset.platform, "Needle")))
    }
    fn validate_official(self, asset: EngineAsset) -> ApiResult<Self> {
        verify_checksum(&self.runner, asset.sha256, None).and_then(|()| match make_executable(&self.runner) {
            | true => Ok(self),
            | false => Err(eyre!("Needle runner is not executable")),
        })
    }
    /// Require a custom ModelKit to carry the exact active tool catalog
    pub fn validate_tools(self, expected: &[NeedleToolDefinition]) -> ApiResult<Self> {
        match self.tools.clone() {
            | None => Ok(self),
            | Some(path) => read_file(path)
                .and_then(|content| {
                    serde_json::from_str::<Vec<NeedleToolDefinition>>(&content).map_err(|why| eyre!("Invalid Needle ModelKit tools catalog — {why}"))
                })
                .and_then(|actual| match actual == expected {
                    | true => Ok(self),
                    | false => Err(eyre!("Needle ModelKit tools catalog does not match ACORN's active catalog")),
                }),
        }
    }
}
fn cache_root() -> ApiResult<PathBuf> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|directories| directories.cache_dir().join("needle"))
        .ok_or_else(|| eyre!("Failed to resolve the ACORN cache directory"))
}
pub(super) fn download_items(location: &str, manifest: &ModelKitManifest) -> ApiResult<(PathBuf, Vec<DownloadItem>)> {
    let base = location.rsplit_once('/').map(|(base, _)| base).unwrap_or(location);
    let root = cache_root().map(|root| root.join("remote").join(Fingerprint::from_bytes(location).to_string()));
    let paths = once((&manifest.runner, Some(&manifest.runner_sha256)))
        .chain(manifest.tools.as_ref().map(|path| (path, None)))
        .map(|(path, sha)| match Source::parse(path) {
            | Source::Local { path: relative, .. } if !path.trim().is_empty() && SafePath::new(&relative).is_ok() => Ok(DownloadItem::init()
                .url(format!("{base}/{path}"))
                .path(path.clone())
                .maybe_sha(sha.cloned())
                .build()),
            | _ => Err(eyre!("Needle ModelKit path must be a safe relative path: {path}")),
        })
        .collect::<ApiResult<Vec<_>>>();
    root.and_then(|root| paths.map(|paths| (root, paths)))
}
/// Resolve the pinned default runner or an explicitly supplied local/OCI ModelKit.
pub async fn resolve(location: Option<&str>, offline: bool, quiet: bool) -> ApiResult<ResolvedAssets> {
    match location.map(str::trim).filter(|value| !value.is_empty()) {
        | None => ResolvedAssets::official(offline, quiet).await,
        | Some(location) => match Source::parse(location) {
            | Source::Remote { .. } if offline => Err(eyre!("Cannot download a Needle ModelKit while offline")),
            | source @ (Source::Local { .. } | Source::Remote { .. }) => ResolvedAssets::from_location(source, quiet).await,
            | Source::Unsupported(identifier) if identifier.starts_with("oci://") && offline => {
                Err(eyre!("Cannot pull a Needle ModelKit from OCI while offline"))
            }
            | Source::Unsupported(identifier) if identifier.starts_with("oci://") => ResolvedAssets::from_oci(&identifier),
            | Source::Unsupported(identifier) => Err(eyre!("Unsupported Needle ModelKit URI '{identifier}'")),
        },
    }
}
/// Resolve a persistent Needle retrieval index keyed by the tool catalog.
pub fn tool_index(catalog: &str) -> ApiResult<PathBuf> {
    cache_root().map(|root| root.join("indexes")).and_then(|root| {
        create_dir_all(&root)
            .map_err(|why| eyre!("Failed to create the Needle tool-index cache — {why}"))
            .map(|()| root.join(format!("{}.idx", Fingerprint::from_bytes(catalog))))
    })
}
