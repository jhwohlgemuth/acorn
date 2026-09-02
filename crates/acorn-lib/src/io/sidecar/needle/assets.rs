//! Needle runner and ModelKit asset acquisition.

use crate::io::download::{DownloadItem, DownloadItems};
use crate::io::oci::{OciReference, OciTransportOptions, OrasTransport};
use crate::io::{file_checksum, make_executable, read_file, ApiResult, Fingerprint};
use crate::util::constants::app::{APPLICATION, ORGANIZATION, QUALIFIER};
use acorn_core::prelude::String;
use acorn_schema::schema::agent::tools::NeedleToolDefinition;
use color_eyre::eyre::eyre;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{canonicalize, create_dir_all, remove_file};
use std::path::{Component, Path, PathBuf};

/// Immutable upstream Needle 2 revision used by default acquisition.
pub const NEEDLE_REVISION: &str = "98fbd955b0347e78059be0c253cc1ffa09b87bc7";
const NEEDLE_REPOSITORY: &str = "Cactus-Compute/needle2";
const MODELKIT_MANIFEST: &str = "needle-modelkit.json";

/// One pinned official standalone runner.
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

/// Validated assets used to start a Needle sidecar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedAssets {
    /// Standalone runner path.
    pub runner: PathBuf,
    /// Tool catalog supplied by a custom bundle.
    pub tools: Option<PathBuf>,
    /// Validated bundle or cache root.
    pub root: PathBuf,
}

/// Needle-specific OCI-compatible ModelKit metadata.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ModelKitManifest {
    /// Manifest schema version.
    pub schema_version: u32,
    /// Needle engine source revision.
    pub source_revision: String,
    /// Platform folder name.
    pub platform: String,
    /// Bundle-relative runner path.
    pub runner: String,
    /// Runner SHA-256.
    pub runner_sha256: String,
    /// Bundle-relative Needle tool schema.
    pub tools: String,
}

impl EngineAsset {
    /// Resolve the pinned runner for the current desktop platform.
    pub fn current() -> ApiResult<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
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
    pub fn url(&self) -> String {
        format!(
            "https://huggingface.co/{NEEDLE_REPOSITORY}/resolve/{NEEDLE_REVISION}/{}/{}",
            self.platform, self.executable
        )
    }
}

impl ModelKitManifest {
    /// Validate and resolve this manifest within `root`.
    pub fn resolve(&self, root: &Path) -> ApiResult<ResolvedAssets> {
        let expected_platform = EngineAsset::current().map(|asset| asset.platform);
        match (
            self.schema_version,
            self.source_revision.trim().is_empty(),
            expected_platform,
            self.platform.as_str(),
        ) {
            | (1, false, Ok(expected), actual) if actual == expected => resolve_relative(root, &self.runner)
                .and_then(|runner| verify_sha256(&runner, &self.runner_sha256).map(|()| runner))
                .and_then(|runner| match make_executable(&runner) {
                    | true => Ok(runner),
                    | false => Err(eyre!("Needle ModelKit runner is not executable — {}", runner.display())),
                })
                .and_then(|runner| {
                    resolve_relative(root, &self.tools).and_then(|tools| match tools.is_file() {
                        | true => Ok((runner, tools)),
                        | false => Err(eyre!("Needle ModelKit tools file is missing — {}", tools.display())),
                    })
                })
                .map(|(runner, tools)| ResolvedAssets {
                    runner,
                    tools: Some(tools),
                    root: root.to_path_buf(),
                }),
            | (1, false, Ok(expected), actual) => Err(eyre!("Needle ModelKit platform '{actual}' does not match this host ('{expected}')")),
            | (_, _, Err(why), _) => Err(why),
            | _ => Err(eyre!("Needle ModelKit requires schemaVersion 1, sourceRevision, and platform")),
        }
    }
}

/// Resolve the pinned default runner or an explicitly supplied local/OCI ModelKit.
pub async fn resolve(custom_location: Option<&str>, offline: bool) -> ApiResult<ResolvedAssets> {
    match custom_location.map(str::trim).filter(|value| !value.is_empty()) {
        | Some(location) if location.starts_with("oci://") => resolve_oci(location, offline),
        | Some(location) => resolve_local(Path::new(location)),
        | None => resolve_default(offline).await,
    }
}

fn cache_root() -> ApiResult<PathBuf> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|directories| directories.cache_dir().join("needle"))
        .ok_or_else(|| eyre!("Failed to resolve the ACORN cache directory"))
}

/// Resolve a persistent Needle retrieval index keyed by the tool catalog.
pub fn tool_index(catalog: &str) -> ApiResult<PathBuf> {
    cache_root().map(|root| root.join("indexes")).and_then(|root| {
        create_dir_all(&root)
            .map_err(|why| eyre!("Failed to create the Needle tool-index cache — {why}"))
            .map(|()| root.join(format!("{}.idx", Fingerprint::from_bytes(catalog))))
    })
}

/// Require a custom ModelKit to carry the exact active ACORN tool catalog.
pub fn validate_tools(assets: &ResolvedAssets, expected: &[NeedleToolDefinition]) -> ApiResult<()> {
    match &assets.tools {
        | None => Ok(()),
        | Some(path) => read_file(path)
            .and_then(|content| {
                serde_json::from_str::<Vec<NeedleToolDefinition>>(&content).map_err(|why| eyre!("Invalid Needle ModelKit tools catalog — {why}"))
            })
            .and_then(|actual| match actual == expected {
                | true => Ok(()),
                | false => Err(eyre!("Needle ModelKit tools catalog does not match ACORN's active catalog")),
            }),
    }
}

async fn resolve_default(offline: bool) -> ApiResult<ResolvedAssets> {
    let asset = EngineAsset::current();
    match asset {
        | Ok(asset) => {
            let root = cache_root().map(|root| root.join(NEEDLE_REVISION).join(asset.platform));
            match root {
                | Ok(root) => {
                    let runner = root.join(asset.executable);
                    match (runner.exists(), offline) {
                        | (false, true) => Err(eyre!("Needle runner is not cached and ACORN is offline — {}", runner.display())),
                        | (false, false) => download_default(root, runner, asset).await,
                        | (true, true) => finish_default(root, runner, asset),
                        | (true, false) => match finish_default(root.clone(), runner.clone(), asset) {
                            | Ok(resolved) => Ok(resolved),
                            | Err(_) => match remove_file(&runner) {
                                | Ok(()) => download_default(root, runner, asset).await,
                                | Err(why) => Err(eyre!("Failed to replace an invalid cached Needle runner — {why}")),
                            },
                        },
                    }
                }
                | Err(why) => Err(why),
            }
        }
        | Err(why) => Err(why),
    }
}

async fn download_default(root: PathBuf, runner: PathBuf, asset: EngineAsset) -> ApiResult<ResolvedAssets> {
    DownloadItems::new(
        &root,
        vec![DownloadItem {
            url: asset.url(),
            path: asset.executable.to_string(),
            size: Some(asset.size),
            sha: Some(asset.sha256.to_string()),
            headers: None,
        }],
        false,
        false,
    )
    .download()
    .await
    .and_then(|()| finish_default(root, runner, asset))
}

fn finish_default(root: PathBuf, runner: PathBuf, asset: EngineAsset) -> ApiResult<ResolvedAssets> {
    verify_sha256(&runner, asset.sha256).and_then(|()| match make_executable(&runner) {
        | true => Ok(ResolvedAssets { runner, tools: None, root }),
        | false => Err(eyre!("Needle runner is not executable")),
    })
}

fn resolve_local(root: &Path) -> ApiResult<ResolvedAssets> {
    canonicalize(root)
        .map_err(|why| eyre!("Failed to resolve local Needle ModelKit {} — {why}", root.display()))
        .and_then(|root| read_manifest(&root).and_then(|manifest| manifest.resolve(&root)))
}

fn resolve_oci(location: &str, offline: bool) -> ApiResult<ResolvedAssets> {
    match offline {
        | true => Err(eyre!("Cannot pull a Needle ModelKit from OCI while offline")),
        | false => OciReference::parse(location).and_then(|reference| {
            let identity = Fingerprint::from_bytes(location).to_string();
            cache_root()
                .map(|root| root.join("oci").join(identity))
                .and_then(|root| match root.exists() {
                    | true => read_manifest(&root).and_then(|manifest| manifest.resolve(&root)),
                    | false => OrasTransport::new(OciTransportOptions::default())
                        .and_then(|transport| transport.pull_artifact(&reference, &root))
                        .and_then(|()| read_manifest(&root))
                        .and_then(|manifest| manifest.resolve(&root)),
                })
        }),
    }
}

fn read_manifest(root: &Path) -> ApiResult<ModelKitManifest> {
    read_file(root.join(MODELKIT_MANIFEST))
        .and_then(|content| serde_json::from_str(&content).map_err(|why| eyre!("Invalid Needle ModelKit manifest — {why}")))
}

fn resolve_relative(root: &Path, value: &str) -> ApiResult<PathBuf> {
    let relative = Path::new(value);
    let safe = !value.trim().is_empty() && relative.components().all(|component| matches!(component, Component::Normal(_)));
    match safe {
        | false => Err(eyre!("Needle ModelKit path must be a safe relative path: {value}")),
        | true => canonicalize(root.join(relative))
            .map_err(|why| eyre!("Failed to resolve Needle ModelKit path '{value}' — {why}"))
            .and_then(|path| match path.starts_with(root) {
                | true => Ok(path),
                | false => Err(eyre!("Needle ModelKit path escapes its bundle root: {value}")),
            }),
    }
}

fn verify_sha256(path: &Path, expected: &str) -> ApiResult<()> {
    file_checksum(path.to_path_buf(), None)
        .ok_or_else(|| eyre!("Failed to calculate Needle runner checksum — {}", path.display()))
        .and_then(|actual| match actual.checksum_value.eq_ignore_ascii_case(expected) {
            | true => Ok(()),
            | false => Err(eyre!(
                "Needle runner checksum mismatch (expected {expected}, got {})",
                actual.checksum_value
            )),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_asset_is_digest_pinned() {
        let asset = EngineAsset::current().expect("supported test platform");
        assert_eq!(asset.sha256.len(), 64);
        assert!(asset.url().contains(NEEDLE_REVISION));
        assert!(!asset.url().contains("/main/"));
    }

    #[test]
    fn modelkit_rejects_unsafe_paths() {
        let root = std::env::temp_dir();
        assert!(resolve_relative(&root, "../needle").is_err());
        assert!(resolve_relative(&root, "/needle").is_err());
    }

    #[test]
    fn custom_catalog_must_match_exactly() {
        let fixture = tempfile::tempdir().expect("fixture");
        let tools = fixture.path().join("tools.json");
        std::fs::write(&tools, "[]").expect("write tools");
        let assets = ResolvedAssets {
            runner: fixture.path().join("needle"),
            tools: Some(tools),
            root: fixture.path().to_path_buf(),
        };
        assert!(validate_tools(&assets, &[]).is_ok());
        assert!(validate_tools(
            &assets,
            &[NeedleToolDefinition {
                name: "acorn.version".to_string(),
                description: "Version".to_string(),
                parameters: serde_json::json!({ "type": "object" }),
            }]
        )
        .is_err());
    }
}
