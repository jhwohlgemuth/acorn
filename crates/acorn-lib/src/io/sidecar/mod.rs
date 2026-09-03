//! Managed subprocess sidecars
use crate::io::{make_executable, read_file, verify_checksum, write_file, ApiResult, InputOutput};
use acorn_core::prelude::{canonicalize, Component, Path, PathBuf, String};
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
/// Common OCI-compatible metadata for an ACORN sidecar ModelKit.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ModelKitManifest {
    /// Manifest schema version.
    pub schema_version: u32,
    /// Upstream sidecar source revision.
    pub source_revision: String,
    /// Platform folder name.
    pub platform: String,
    /// Bundle-relative executable path.
    pub runner: String,
    /// Executable SHA-256.
    pub runner_sha256: String,
    /// Optional bundle-relative tool schema.
    #[serde(default)]
    pub tools: Option<String>,
}
/// Validated executable assets used to start a sidecar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedAssets {
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
    pub fn resolve(&self, root: &Path, expected_platform: &str, sidecar: &str) -> ApiResult<ResolvedAssets> {
        let has_revision = !self.source_revision.trim().is_empty();
        let matches_platform = self.platform == expected_platform;
        match (self.schema_version, has_revision, matches_platform) {
            | (1, true, true) => resolve_relative(root, &self.runner, sidecar)
                .and_then(|runner| verify_checksum(&runner, &self.runner_sha256, None).map(|()| runner))
                .and_then(|runner| match make_executable(&runner) {
                    | true => Ok(runner),
                    | false => Err(eyre!("{sidecar} ModelKit runner is not executable — {}", runner.display())),
                })
                .and_then(|runner| {
                    self.tools
                        .as_deref()
                        .map(|tools| resolve_relative(root, tools, sidecar))
                        .transpose()
                        .map(|tools| ResolvedAssets {
                            runner,
                            tools,
                            root: root.to_path_buf(),
                        })
                }),
            | (1, true, false) => Err(eyre!(
                "{sidecar} ModelKit platform '{}' does not match this host ('{expected_platform}')",
                self.platform
            )),
            | _ => Err(eyre!("{sidecar} ModelKit requires schemaVersion 1, sourceRevision, and platform")),
        }
    }
}
fn resolve_relative(root: &Path, value: &str, sidecar: &str) -> ApiResult<PathBuf> {
    let relative = Path::new(value);
    let safe = !value.trim().is_empty() && relative.components().all(|component| matches!(component, Component::Normal(_)));
    match safe {
        | false => Err(eyre!("{sidecar} ModelKit path must be a safe relative path: {value}")),
        | true => canonicalize(root.join(relative))
            .map_err(|why| eyre!("Failed to resolve {sidecar} ModelKit path '{value}' — {why}"))
            .and_then(|path| match path.starts_with(root) {
                | true => Ok(path),
                | false => Err(eyre!("{sidecar} ModelKit path escapes its bundle root: {value}")),
            }),
    }
}
