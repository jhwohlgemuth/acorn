//! Bounded local configuration synchronization through the shared target renderer.
use super::super::{
    ArtifactPolicy, BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, InputLimits, RoutedInputPolicy, ToolRegistry,
};
use super::NativeEffect;
use crate::error::ApiResult;
use crate::io::config::ModelEntry;
use crate::io::sync::{self, Config, ConfigExt, ConfigPrivate};
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::SemanticVersion;
use acorn_schema::agent::tools::ToolEffects;
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Supported local inference configuration target.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyncTargetKind {
    /// Goose CLI configuration.
    Goose,
    /// llama-swap configuration.
    LlamaSwap,
    /// OpenCode configuration.
    OpenCode,
    /// VS Code configuration.
    VsCode,
}
/// One explicit, bounded target configuration request.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SyncInput {
    /// Local owner/name model identifiers already materialized under `models_dir`.
    pub models: Vec<String>,
    /// Existing models directory relative to the routed root.
    pub models_dir: PathBuf,
    /// One selected target application.
    pub target: SyncTargetKind,
    /// Exact target configuration path relative to the routed root.
    pub destination: PathBuf,
    /// Permit updating an existing target file during apply.
    #[serde(default)]
    pub overwrite: bool,
    /// Remove stale managed model entries from the selected target.
    #[serde(default)]
    pub prune: bool,
}
/// One rendered target change, returned before or after an authorized apply.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SyncOutput {
    /// Selected target application.
    pub target: String,
    /// Canonical target configuration path.
    pub destination: PathBuf,
    /// Existing configuration text.
    pub before: String,
    /// Proposed configuration text.
    pub content: String,
    /// Whether the target changed.
    pub changed: bool,
    /// Exact written target when apply changed the file.
    pub artifacts: Vec<String>,
}
impl SyncInput {
    fn plan(&self, root: &Path) -> ApiResult<(PathBuf, Vec<sync::RenderedOutput>)> {
        let valid_count = !self.models.is_empty() && self.models.len() <= 16 && self.models.iter().all(|model| model.len() <= 256);
        let valid_identifiers = !self.models.iter().any(|model| {
            let mut parts = model.split('/');
            match (parts.next(), parts.next(), parts.next()) {
                | (Some(owner), Some(name), None) => [owner, name].iter().any(|part| {
                    matches!(*part, "" | "." | "..")
                        || !part
                            .chars()
                            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
                }),
                | _ => true,
            }
        });
        match (valid_count, valid_identifiers) {
            | (true, true) => RoutedInputPolicy::new(root, InputLimits::default()).and_then(|input_policy| {
                root.canonicalize()
                    .map_err(|why| eyre!("Invalid sync root — {why}"))
                    .and_then(|canonical_root| {
                        input_policy
                            .resolve_path(&self.models_dir)
                            .and_then(|models_dir| match models_dir.is_dir() {
                                | false => Err(eyre!("Sync models directory must exist")),
                                | true => ArtifactPolicy::new(root, true, true).and_then(|policy| {
                                    policy
                                        .destination(&self.destination, &[])
                                        .and_then(|destination| match destination.starts_with(&models_dir) {
                                            | true => Err(eyre!("Sync configuration destination must be outside the models directory")),
                                            | false => {
                                                let size_check = if destination.is_file() {
                                                    destination
                                                        .metadata()
                                                        .map_err(|why| eyre!("Cannot inspect sync target — {why}"))
                                                        .and_then(|metadata| match InputLimits::default().max_total.can_contain(metadata.len()) {
                                                            | Some(true) => Ok(()),
                                                            | _ => Err(eyre!("Sync target exceeds the configuration byte limit")),
                                                        })
                                                } else {
                                                    Ok(())
                                                };
                                                size_check.and_then(|()| self.render_plan(&canonical_root, &models_dir, destination))
                                            }
                                        })
                                }),
                            })
                    })
            }),
            | (false, _) => Err(eyre!("Sync requires one to sixteen bounded local model identifiers")),
            | (_, false) => Err(eyre!("Sync model identifier must use a bounded owner/name form")),
        }
    }
    fn render_plan(&self, canonical_root: &Path, models_dir: &Path, destination: PathBuf) -> ApiResult<(PathBuf, Vec<sync::RenderedOutput>)> {
        let path = destination.display().to_string();
        let config = match self.target {
            | SyncTargetKind::Goose => Config {
                goose: Some(sync::goose::Config {
                    path: Some(path),
                    ..Default::default()
                }),
                ..Default::default()
            },
            | SyncTargetKind::LlamaSwap => Config {
                llama_swap: Some(sync::llama_swap::Config {
                    path: Some(path),
                    ..Default::default()
                }),
                ..Default::default()
            },
            | SyncTargetKind::OpenCode => Config {
                opencode: Some(sync::opencode::Config {
                    path: Some(path),
                    ..Default::default()
                }),
                ..Default::default()
            },
            | SyncTargetKind::VsCode => Config {
                vscode: Some(sync::vscode::Config {
                    path: Some(path),
                    ..Default::default()
                }),
                ..Default::default()
            },
        };
        let entries = self.models.iter().cloned().map(ModelEntry::Selector).collect::<Vec<_>>();
        let request = sync::ModelRequestOptions {
            models_dir,
            assume_models: false,
            fallbacks: Vec::new(),
        };
        ModelEntry::resolve(&entries, &request).and_then(|resolved| {
            let valid = resolved.len() == entries.len()
                && resolved.iter().all(|model| {
                    model.path.as_deref().is_some_and(|path| match Path::new(path).canonicalize() {
                        | Ok(path) => path.starts_with(canonical_root),
                        | Err(_) => false,
                    })
                });
            match valid {
                | true => {
                    let options = sync::Options::init()
                        .models(&resolved)
                        .entries(&entries)
                        .goose(matches!(self.target, SyncTargetKind::Goose))
                        .llama_swap(matches!(self.target, SyncTargetKind::LlamaSwap))
                        .opencode(matches!(self.target, SyncTargetKind::OpenCode))
                        .vscode(matches!(self.target, SyncTargetKind::VsCode))
                        .force(true)
                        .prune(self.prune)
                        .models_dir(models_dir)
                        .build();
                    config.plan(options).and_then(|outputs| match outputs.as_slice() {
                        | [output] if output.path() == destination => Ok((destination, outputs)),
                        | _ => Err(eyre!("Sync request must render exactly one authorized target")),
                    })
                }
                | false => Err(eyre!("Every synchronized model must resolve to a local path within the routed root")),
            }
        })
    }
    /// Preview one selected target without writing a file.
    pub fn preview(self, root: &Path) -> ApiResult<SyncOutput> {
        self.plan(root).and_then(|(destination, outputs)| {
            outputs
                .first()
                .map(|output| SyncOutput {
                    target: output.target().to_string(),
                    destination,
                    before: output.before().to_string(),
                    content: output.content().to_string(),
                    changed: output.changed(),
                    artifacts: Vec::new(),
                })
                .ok_or_else(|| eyre!("Sync target produced no output"))
        })
    }
    /// Apply the reviewed target through the shared coordinated commit operation.
    pub fn apply(self, root: &Path) -> ApiResult<SyncOutput> {
        self.plan(root)
            .and_then(|(destination, outputs)| match destination.exists() && !self.overwrite {
                | true => Err(eyre!("Sync target exists; explicit overwrite is required")),
                | false => ArtifactPolicy::new(root, true, self.overwrite)
                    .and_then(|policy| policy.destination(&self.destination, &[]))
                    .and_then(|_| {
                        outputs
                            .first()
                            .map(|output| SyncOutput {
                                target: output.target().to_string(),
                                destination,
                                before: output.before().to_string(),
                                content: output.content().to_string(),
                                changed: output.changed(),
                                artifacts: if output.changed() {
                                    vec![output.path().display().to_string()]
                                } else {
                                    Vec::new()
                                },
                            })
                            .ok_or_else(|| eyre!("Sync target produced no output"))
                    })
                    .and_then(|result| Config::commit(&outputs).map(|_| result)),
            })
    }
}
impl ToolRegistry {
    /// Register a read-only synchronization preview.
    pub fn register_sync_preview_capability(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let effects = ToolEffects::from(NativeEffect::LocalRead);
        let register = self.register_capability(CapabilityDefinition::new::<SyncInput, SyncOutput>(
            format!("{APPLICATION}.sync::preview"),
            version,
            "Preview one local inference target configuration change.",
            effects,
        ));
        register.and_then(|registry| {
            let definition = BindingDefinition::new::<SyncInput, SyncOutput>(
                format!("native:{APPLICATION}.sync::preview"),
                format!("{APPLICATION}.sync::preview"),
                version,
                BindingKind::Native,
                effects,
                BindingAvailability::Ready,
            );
            registry.register_capability_binding::<SyncInput, SyncOutput, _, _>(definition.with_offline_support(), move |input| {
                let root = root.clone();
                async move { input.preview(&root) }
            })
        })
    }
    /// Register exact-authority synchronization apply.
    pub fn register_sync_apply_capability(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let effects = ToolEffects::from(NativeEffect::LocalWrite);
        let register = self.register_capability(CapabilityDefinition::new::<SyncInput, SyncOutput>(
            format!("{APPLICATION}.sync::apply"),
            version,
            "Apply one authorized local inference target configuration change.",
            effects,
        ));
        register.and_then(|registry| {
            let definition = BindingDefinition::new::<SyncInput, SyncOutput>(
                format!("native:{APPLICATION}.sync::apply"),
                format!("{APPLICATION}.sync::apply"),
                version,
                BindingKind::Native,
                effects,
                BindingAvailability::Ready,
            );
            registry.register_capability_binding::<SyncInput, SyncOutput, _, _>(definition.with_offline_support(), move |input| {
                let root = root.clone();
                async move { input.apply(&root) }
            })
        })
    }
}
