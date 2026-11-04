//! Bounded linked-data previews and exact artifact writes.
use super::super::{
    ArtifactPolicy, BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, InputLimits, InputReference, RoutedInputPolicy,
    ToolRegistry,
};
use super::NativeEffect;
use crate::error::ApiResult;
use crate::io::InputOutput;
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::SemanticVersion;
use acorn_schema::agent::tools::ToolEffects;
use acorn_schema::research_activity::ResearchActivity;
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs::write;
use std::path::{Path, PathBuf};

/// Exact destination authorized by a caller for one linked-data artifact.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LinkApplyInput {
    /// Confirmed source file.
    pub input: LinkPreviewInput,
    /// Destination relative to the permitted root.
    pub destination: PathBuf,
    /// Permit replacing an existing artifact.
    #[serde(default)]
    pub overwrite: bool,
}
/// Written artifact and the reviewed linked-data projection.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LinkApplyOutput {
    /// Canonical artifact path written.
    pub destination: PathBuf,
    /// Artifact reference for the routed result.
    pub artifacts: Vec<String>,
    /// Projection that was written.
    pub projection: LinkPreviewOutput,
}
/// One confirmed local research activity file to project as linked data.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LinkPreviewInput {
    /// Local file below the permitted root.
    pub input: InputReference,
}
/// Reviewable linked-data projection from the direct link workflow.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LinkPreviewOutput {
    /// Source file used for the projection.
    pub source: PathBuf,
    /// Proposed linked-data artifact path.
    pub destination: PathBuf,
    /// Complete JSON-LD content.
    pub content: String,
}
impl LinkApplyInput {
    /// Revalidate the source and exact destination before writing one artifact.
    pub fn apply(self, root: &Path) -> ApiResult<LinkApplyOutput> {
        ArtifactPolicy::new(root, true, self.overwrite).and_then(|policy| {
            self.input.preview(root).and_then(|projection| {
                policy
                    .destination(&self.destination, core::slice::from_ref(&projection.source))
                    .and_then(|destination| {
                        if destination != projection.destination {
                            Err(eyre!("Linked-data destination must match the source companion path"))
                        } else {
                            write(&destination, &projection.content)
                                .map_err(|why| eyre!("Write linked-data artifact {} — {why}", destination.display()))
                                .map(|()| LinkApplyOutput {
                                    artifacts: vec![destination.display().to_string()],
                                    destination,
                                    projection,
                                })
                        }
                    })
            })
        })
    }
}
impl LinkPreviewInput {
    /// Project one bounded local file through the same formatter and linked-data operation as `acorn link`.
    pub fn preview(self, root: &Path) -> ApiResult<LinkPreviewOutput> {
        RoutedInputPolicy::new(root, InputLimits::default())
            .and_then(|policy| policy.resolve(&self.input))
            .and_then(|resolved| match (self.input, resolved.files.as_slice()) {
                | (InputReference::File(_), [file]) => {
                    let source = file.path.clone();
                    ResearchActivity::read(source.clone())
                        .map(|activity| activity.format_with(Some(source.clone())))
                        .and_then(|activity| activity.linked_content())
                        .and_then(|content| content.ok_or_else(|| eyre!("Research activity has no linked-data projection")))
                        .map(|content| LinkPreviewOutput {
                            destination: source.with_extension("jsonld"),
                            source,
                            content,
                        })
                }
                | _ => Err(eyre!("Linked-data projection requires one local file")),
            })
    }
}
impl ToolRegistry {
    /// Register linked-data preview and exact artifact application.
    pub fn register_link_capabilities(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let read = ToolEffects::from(NativeEffect::LocalRead);
        let write = ToolEffects {
            read_only: false,
            destructive: true,
            ..read
        };
        let preview_root = root.clone();
        self.register_capability(CapabilityDefinition::new::<LinkPreviewInput, LinkPreviewOutput>(
            format!("{APPLICATION}.link::preview"),
            version,
            "Preview one local linked-data artifact.",
            read,
        ))
        .and_then(|registry| {
            registry.register_capability_binding::<LinkPreviewInput, LinkPreviewOutput, _, _>(
                BindingDefinition::new::<LinkPreviewInput, LinkPreviewOutput>(
                    format!("native:{APPLICATION}.link::preview"),
                    format!("{APPLICATION}.link::preview"),
                    version,
                    BindingKind::Native,
                    read,
                    BindingAvailability::Ready,
                )
                .with_offline_support(),
                move |input| {
                    let root = preview_root.clone();
                    async move { input.preview(&root) }
                },
            )
        })
        .and_then(|registry| {
            registry.register_capability(CapabilityDefinition::new::<LinkApplyInput, LinkApplyOutput>(
                format!("{APPLICATION}.link::apply"),
                version,
                "Write one authorized linked-data artifact.",
                write,
            ))
        })
        .and_then(|registry| {
            registry.register_capability_binding::<LinkApplyInput, LinkApplyOutput, _, _>(
                BindingDefinition::new::<LinkApplyInput, LinkApplyOutput>(
                    format!("native:{APPLICATION}.link::apply"),
                    format!("{APPLICATION}.link::apply"),
                    version,
                    BindingKind::Native,
                    write,
                    BindingAvailability::Ready,
                )
                .with_offline_support(),
                move |input| {
                    let root = root.clone();
                    async move { input.apply(&root) }
                },
            )
        })
    }
}
