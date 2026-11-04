//! Routed acquisition and status capabilities for managed sidecars.
use super::{NativeCapability, NativeEffect, Register};
use crate::error::ApiResult;
use crate::io::mcp::tools::ToolRegistry;
use crate::io::sidecar::{needle::Needle, ResolvedAssets, Sidecar, SidecarAssets};
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::env::consts::{ARCH, OS};
use std::path::PathBuf;

trait DownloadSidecar: Sidecar {
    fn acquisition_destination() -> ApiResult<PathBuf> {
        Self::for_target(OS, ARCH).and_then(|runner| Self::cache_root().map(|root| root.join(Self::REVISION).join(Self::platform(&runner))))
    }
}
/// Explicit authorization target for pinned Needle acquisition.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NeedleAcquireInput {
    /// Exact official cache directory, as returned by the status operation.
    pub destination: PathBuf,
}
/// Verified pinned Needle acquisition result.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NeedleAcquireOutput {
    /// Verified cache directory.
    pub destination: PathBuf,
    /// Verified standalone runner.
    pub runner: PathBuf,
}
/// Empty, closed input for the pinned Needle cache status operation.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NeedleStatusInput {}
/// Pinned Needle cache readiness without any acquisition effect.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NeedleStatusOutput {
    /// Whether all pinned files exist and pass verification.
    pub ready: bool,
    /// Exact cache directory required for explicit acquisition authority.
    pub destination: PathBuf,
    /// Verified runner path when ready.
    pub runner: Option<PathBuf>,
}
impl DownloadSidecar for Needle {}
impl NeedleAcquireInput {
    /// Resolve the official pinned distribution after matching its exact cache destination.
    pub async fn acquire(self, offline: bool) -> ApiResult<NeedleAcquireOutput> {
        match Needle::acquisition_destination() {
            | Err(why) => Err(why),
            | Ok(destination) => match self.destination == destination {
                | false => Err(eyre!("Needle acquisition destination must match the pinned cache directory")),
                | true => Needle::resolve_assets(None, offline, true)
                    .await
                    .and_then(|assets| match assets.root == destination {
                        | false => Err(eyre!("Pinned Needle assets resolved outside the authorized cache directory")),
                        | true => Ok(NeedleAcquireOutput {
                            destination,
                            runner: assets.runner,
                        }),
                    }),
            },
        }
    }
}
impl NeedleStatusOutput {
    /// Verify only the existing local pinned assets, without network access or writes.
    pub async fn inspect() -> ApiResult<Self> {
        match Needle::acquisition_destination() {
            | Err(why) => Err(why),
            | Ok(destination) => {
                let verified = Needle::resolve_assets(None, true, true).await.ok();
                Ok(Self {
                    ready: verified.is_some(),
                    destination,
                    runner: verified.map(|ResolvedAssets { runner, .. }| runner),
                })
            }
        }
    }
}
impl ToolRegistry {
    /// Register explicit acquisition of the pinned Needle distribution.
    pub fn register_download_needle_capability(self, offline: bool) -> ApiResult<Self> {
        let capability = NativeCapability::new(
            "download::needle",
            "Acquire or verify the pinned Needle distribution at its exact cache destination.",
            NativeEffect::RemoteWrite,
        );
        let capability = match offline {
            | true => capability.with_offline_support(),
            | false => capability,
        };
        self.register_native::<NeedleAcquireInput, NeedleAcquireOutput, _, _>(capability, move |input| async move { input.acquire(offline).await })
    }
    /// Register read-only status for the pinned Needle distribution.
    pub fn register_download_needle_status_capability(self) -> ApiResult<Self> {
        self.register_native::<NeedleStatusInput, NeedleStatusOutput, _, _>(
            NativeCapability::new(
                "download::needle::status",
                "Check the pinned Needle runner, model, and license in the local cache without downloading.",
                NativeEffect::LocalRead,
            )
            .with_offline_support(),
            |_: NeedleStatusInput| async { NeedleStatusOutput::inspect().await },
        )
    }
}
