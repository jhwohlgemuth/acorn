use super::*;
use crate::io::config::ApplicationConfiguration;
use crate::io::sidecar::{
    DistributionAsset, DistributionEngine, DistributionManifest, ModelKitManifest, ResolvedAssets, Sidecar, SidecarAssets, SidecarDistribution,
    ToolIndex,
};
use async_trait::async_trait;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};

static INVOCATIONS: AtomicUsize = AtomicUsize::new(0);
static DROPPED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Deserialize)]
struct FixtureEngine;
#[derive(Deserialize)]
struct FixtureManifest;
#[derive(Deserialize)]
struct FixtureAsset;
#[derive(Debug)]
struct FixtureIndex(PathBuf);
struct FixtureSidecar;
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct FixtureInput {
    text: String,
}
#[derive(JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct FixtureOutput {
    text: String,
}
struct DropSignal;
impl Drop for DropSignal {
    fn drop(&mut self) {
        DROPPED.store(true, Ordering::SeqCst);
    }
}
impl DistributionAsset for FixtureAsset {
    fn name(&self) -> &str {
        "unused"
    }
    fn path(&self) -> &str {
        "unused"
    }
    fn sha256(&self) -> &str {
        "unused"
    }
    fn size(&self) -> u64 {
        0
    }
}
impl DistributionEngine for FixtureEngine {
    fn architecture(&self) -> &str {
        std::env::consts::ARCH
    }
    fn executable(&self) -> &str {
        "runner"
    }
    fn os(&self) -> &str {
        std::env::consts::OS
    }
    fn platform(&self) -> &str {
        "fixture"
    }
    fn sha256(&self) -> &str {
        "f16d05ec6b29248d2c61adb1e9263f78e4f7bace1b955014a2d17872cfe4064d"
    }
    fn size(&self) -> u64 {
        7
    }
}
impl DistributionManifest for FixtureManifest {
    type Engine = FixtureEngine;
    type Supporting = FixtureAsset;
    fn engines(&self) -> &[Self::Engine] {
        &[]
    }
    fn supporting(&self) -> &[Self::Supporting] {
        &[]
    }
}
impl ToolIndex for FixtureIndex {
    fn new(_cached: &Path, session_root: &Path) -> Self {
        Self(session_root.to_path_buf())
    }
    fn prepare(&self) -> ApiResult<()> {
        Ok(())
    }
    fn publish(&self) -> ApiResult<()> {
        Ok(())
    }
    fn session(&self) -> &Path {
        &self.0
    }
}
#[async_trait]
impl SidecarAssets for FixtureSidecar {
    type AssetManifest = FixtureManifest;
    type EngineAsset = FixtureEngine;
    const ASSET_FILE_NAME: &'static str = "unused";
    const KIND: &'static str = "fixture";
    const PROTOCOL_VERSION: u32 = 1;
    fn for_target(os: &str, architecture: &str) -> ApiResult<Self::EngineAsset> {
        if os == std::env::consts::OS && architecture == std::env::consts::ARCH {
            Ok(FixtureEngine)
        } else {
            Err(eyre!("unsupported fixture host"))
        }
    }
    fn validate_modelkit(_manifest: &ModelKitManifest) -> ApiResult<()> {
        Ok(())
    }
}
impl SidecarDistribution for FixtureSidecar {
    const REPOSITORY: &'static str = "unused";
    const REVISION: &'static str = "unused";
}
#[async_trait]
impl Sidecar for FixtureSidecar {
    type Input = FixtureInput;
    type Output = FixtureOutput;
    type ToolIndex = FixtureIndex;
    async fn start_with_assets(_assets: ResolvedAssets, _offline: bool) -> ApiResult<Self> {
        INVOCATIONS.fetch_add(1, Ordering::SeqCst);
        Ok(Self)
    }
    async fn infer(self, input: Self::Input) -> ApiResult<Self::Output> {
        let _drop = DropSignal;
        if input.text == "pending" {
            core::future::pending().await
        } else {
            Ok(FixtureOutput { text: input.text })
        }
    }
}
fn binding() -> ExecutorBindingConfiguration {
    ApplicationConfiguration::parse(
        json!({"executors":[{"id":"sidecar:fixture","capabilityId":"acorn.fixture","executor":{"type":"sidecar","profile":"fixture"}}]}).to_string(),
    )
    .expect("fixture configuration")
    .executors
    .remove(0)
}
fn installed() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary ModelKit root");
    std::fs::write(root.path().join("runner"), b"fixture").expect("fixture runner");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(root.path().join("runner"), std::fs::Permissions::from_mode(0o700)).expect("executable runner");
    }
    let manifest = json!({
        "manifestVersion":"v1.0.0","sourceRevision":"fixture","platform":"fixture",
        "sidecar":{"kind":"fixture","protocolVersion":1},
        "runner":{"path":"runner","size":7,"sha256":"f16d05ec6b29248d2c61adb1e9263f78e4f7bace1b955014a2d17872cfe4064d"}
    });
    std::fs::write(root.path().join("needle-modelkit.json"), manifest.to_string()).expect("fixture manifest");
    root
}
#[tokio::test]
async fn test_installed_sidecar_runs_once_and_cancels() {
    INVOCATIONS.store(0, Ordering::SeqCst);
    DROPPED.store(false, Ordering::SeqCst);
    let root = installed();
    let registry = ToolRegistry::default()
        .register_sidecar_binding::<FixtureSidecar>(&binding(), "fixture", root.path())
        .expect("registered fixture");
    let selected = registry
        .select_capability_binding(
            "acorn.fixture",
            None,
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: false,
            },
        )
        .expect("selected fixture");
    let result = registry
        .execute_selected(
            &selected,
            json!({"text":"hello"}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::Completed);
    assert_eq!(result.content, Some(json!({"text":"hello"})));
    assert_eq!(result.provenance.binding_id, "sidecar:fixture");
    assert_eq!(INVOCATIONS.load(Ordering::SeqCst), 1);
    let cancelled = registry
        .execute_selected(
            &selected,
            json!({"text":"pending"}),
            ExecutionContext {
                grounded: true,
                cancellation: Some(Box::pin(async { tokio::time::sleep(core::time::Duration::from_millis(20)).await })),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(cancelled.outcome, ExecutionOutcome::Cancelled);
    assert!(DROPPED.load(Ordering::SeqCst));
}
#[test]
fn test_missing_or_altered_sidecar_assets_remain_unavailable() {
    let root = installed();
    std::fs::write(root.path().join("runner"), b"changed").expect("alter fixture runner");
    let registry = ToolRegistry::default()
        .register_sidecar_binding::<FixtureSidecar>(&binding(), "fixture", root.path())
        .expect("register unavailable fixture");
    assert!(registry
        .select_capability_binding("acorn.fixture", Some("sidecar:fixture"), None, CapabilitySelectionContext::default())
        .is_err());
    assert_eq!(std::fs::read(root.path().join("runner")).expect("runner unchanged"), b"changed");
}
#[tokio::test]
async fn test_sidecar_rechecks_assets_before_invocation() {
    let root = installed();
    let registry = ToolRegistry::default()
        .register_sidecar_binding::<FixtureSidecar>(&binding(), "fixture", root.path())
        .expect("registered fixture");
    let selected = registry
        .select_capability_binding("acorn.fixture", None, None, CapabilitySelectionContext::default())
        .expect("selected fixture");
    std::fs::write(root.path().join("runner"), b"changed").expect("alter runner after registration");
    let result = registry
        .execute_selected(
            &selected,
            json!({"text":"hello"}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::ToolFailed);
}
#[test]
fn test_unimplemented_sidecar_profile_is_explicitly_unavailable() {
    let registry = ToolRegistry::default()
        .register_unavailable_sidecar_bindings(&[binding()])
        .expect("register unavailable profile");
    let error = registry
        .select_capability_binding("acorn.fixture", Some("sidecar:fixture"), None, CapabilitySelectionContext::default())
        .expect_err("profile must be unavailable");
    assert!(error.to_string().contains("not installed"));
}
