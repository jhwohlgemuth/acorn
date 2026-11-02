//! Explicit registration of installed local sidecar executors.
use super::{BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, InferenceTaskInput, InferenceTaskOutput, ToolRegistry};
use crate::error::ApiResult;
use crate::io::mcp::{ExecutorBindingConfiguration, ExecutorConfiguration};
use crate::io::sidecar::{ModelKitManifest, ResolvedAssets, Sidecar, SidecarAssets};
use crate::io::InputOutput;
use crate::util::constants::oci::MODELKIT_MANIFEST;
use acorn_core::util::SemanticVersion;
use acorn_schema::agent::tools::ToolEffects;
use acorn_schema::validation::Validate;
use color_eyre::eyre::eyre;
use core::time::Duration;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::env::consts::{ARCH, OS};
use std::path::Path;

#[cfg(unix)]
fn installed_executable(root: &Path, runner: &str) -> ApiResult<()> {
    use std::os::unix::fs::PermissionsExt;
    let runner = root.join(runner);
    runner.metadata().map_err(Into::into).and_then(|metadata: std::fs::Metadata| {
        (metadata.permissions().mode() & 0o100 != 0)
            .then_some(())
            .ok_or_else(|| eyre!("Installed sidecar runner is not executable"))
    })
}
#[cfg(not(unix))]
fn installed_executable(_root: &Path, _runner: &str) -> ApiResult<()> {
    Ok(())
}
fn inference_effects() -> ToolEffects {
    ToolEffects {
        read_only: true,
        destructive: false,
        idempotent: false,
        open_world: true,
    }
}
fn resolve_installed<S: SidecarAssets>(root: &Path) -> ApiResult<ResolvedAssets> {
    S::for_target(OS, ARCH).and_then(|target| {
        ModelKitManifest::read(root.join(MODELKIT_MANIFEST)).and_then(|manifest| {
            installed_executable(root, &manifest.runner.path).and_then(|()| S::resolve_modelkit(&manifest, root, S::platform(&target)))
        })
    })
}
impl ToolRegistry {
    /// Register one installed sidecar without acquiring assets or starting its process.
    pub fn register_sidecar_binding<S>(self, configured: &ExecutorBindingConfiguration, profile: &str, root: &Path) -> ApiResult<Self>
    where
        S: Sidecar + Send + 'static,
        S::Input: DeserializeOwned + JsonSchema + Send + 'static,
        S::Output: Serialize + JsonSchema + 'static,
    {
        let profile_matches = matches!(&configured.executor, ExecutorConfiguration::Sidecar { profile: selected } if selected == profile);
        configured
            .validate()
            .map_err(|why| eyre!("Invalid sidecar binding '{}' — {why}", configured.id))
            .and_then(|()| {
                profile_matches
                    .then_some(())
                    .ok_or_else(|| eyre!("Sidecar binding '{}' does not select profile '{profile}'", configured.id))
            })
            .and_then(|()| {
                let version = SemanticVersion::from((1, 0, 0));
                let effects = inference_effects();
                let capability = CapabilityDefinition::new::<S::Input, S::Output>(
                    &configured.capability_id,
                    version,
                    "Run one explicitly configured inference task.",
                    effects,
                );
                let assets = resolve_installed::<S>(root);
                let availability = match &assets {
                    | Ok(_) => BindingAvailability::Ready,
                    | Err(why) => BindingAvailability::Unavailable { reason: why.to_string() },
                };
                let definition = BindingDefinition::new::<S::Input, S::Output>(
                    &configured.id,
                    &configured.capability_id,
                    version,
                    BindingKind::Local,
                    effects,
                    availability,
                );
                let binding = definition.with_offline_support();
                let root = root.to_path_buf();
                self.register_dynamic_capability(capability).and_then(|registry| {
                    registry.register_capability_binding::<S::Input, S::Output, _, _>(binding, move |input| {
                        let root = root.clone();
                        async move {
                            let timed = tokio::time::timeout(Duration::from_secs(120), async {
                                match resolve_installed::<S>(&root) {
                                    | Ok(assets) => match S::start_with_assets(assets, true).await {
                                        | Ok(sidecar) => sidecar.infer(input).await,
                                        | Err(why) => Err(why),
                                    },
                                    | Err(why) => Err(why),
                                }
                            });
                            match timed.await {
                                | Ok(result) => result,
                                | Err(_) => Err(eyre!("Sidecar task timed out")),
                            }
                        }
                    })
                })
            })
    }
    /// Keep configured sidecar profiles visible until a concrete profile registers them.
    pub fn register_unavailable_sidecar_bindings(self, configured: &[ExecutorBindingConfiguration]) -> ApiResult<Self> {
        configured
            .iter()
            .filter_map(|binding| match &binding.executor {
                | ExecutorConfiguration::Sidecar { profile } => Some((binding, profile)),
                | _ => None,
            })
            .try_fold(self, |registry, (configured, profile)| {
                configured
                    .validate()
                    .map_err(|why| eyre!("Invalid sidecar binding '{}' — {why}", configured.id))
                    .and_then(|()| {
                        let version = SemanticVersion::from((1, 0, 0));
                        let effects = inference_effects();
                        let capability = CapabilityDefinition::new::<InferenceTaskInput, InferenceTaskOutput>(
                            &configured.capability_id,
                            version,
                            "Run one explicitly configured inference task.",
                            effects,
                        );
                        let definition = BindingDefinition::new::<InferenceTaskInput, InferenceTaskOutput>(
                            &configured.id,
                            &configured.capability_id,
                            version,
                            BindingKind::Local,
                            effects,
                            BindingAvailability::Unavailable {
                                reason: format!("Sidecar profile '{profile}' is not installed"),
                            },
                        );
                        let binding = definition.with_offline_support();
                        registry.register_dynamic_capability(capability).and_then(|registry| {
                            registry.register_capability_binding::<InferenceTaskInput, InferenceTaskOutput, _, _>(binding, |_| async {
                                Err(eyre!("Sidecar profile is unavailable"))
                            })
                        })
                    })
            })
    }
}
