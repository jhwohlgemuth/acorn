//! Bounded OpenAPI endpoint import through the direct command's projection.
use super::super::{
    ArtifactPolicy, BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, InputLimits, InputReference, RoutedInputPolicy,
    ToolRegistry,
};
use super::NativeEffect;
use crate::error::ApiResult;
use crate::io::adapter::WeightsExt;
use crate::io::api::huggingface::{self, RepositoryResolution};
use crate::io::api::Endpoint;
use crate::io::database::{schema::Table, Database};
use crate::io::model::metadata::resolve_import_metadata;
use crate::io::{FromPath, Source};
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::{MimeType, SemanticVersion};
use acorn_schema::agent::tools::ToolEffects;
use acorn_schema::agent::Weights;
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs::{self, write};
use std::path::{Path, PathBuf};

/// One exact authorized local database destination for model metadata.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelImportApplyInput {
    /// Repository request to re-resolve before persistence.
    pub request: ModelImportInput,
    /// Exact relative SQLite database path under the routed root.
    pub destination: PathBuf,
    /// Permit updating an existing database file.
    #[serde(default)]
    pub overwrite: bool,
}
/// Persisted metadata record and authorized local database artifact.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelImportApplyOutput {
    /// Requested and resolved repository summary.
    pub metadata: ModelImportPreviewOutput,
    /// Canonical database path.
    pub destination: PathBuf,
    /// Bounded database artifact reference.
    pub artifacts: Vec<String>,
}
/// One explicit Hugging Face repository for metadata-only import.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelImportInput {
    /// Repository owner/name; URLs and local model files are not accepted.
    pub identifier: String,
}
/// Read-only result of resolving model weight metadata, without weight download.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelImportPreviewOutput {
    /// Requested Hugging Face repository.
    pub requested: String,
    /// Resolved repository after any fallback selection.
    pub resolved: String,
    /// Number of GGUF weight file metadata entries.
    pub weight_count: usize,
}
/// One exact authorized endpoint JSON destination.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpecImportApplyInput {
    /// Import arguments to revalidate before writing.
    pub request: SpecImportInput,
    /// Exact relative JSON destination.
    pub destination: PathBuf,
    /// Permit replacing an existing artifact.
    #[serde(default)]
    pub overwrite: bool,
}
/// Written endpoint JSON artifact.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpecImportApplyOutput {
    /// Canonical artifact path.
    pub destination: PathBuf,
    /// Bounded artifact reference for routed execution.
    pub artifacts: Vec<String>,
}
/// One explicit OpenAPI source and endpoint identity.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpecImportInput {
    /// Literal text, bounded local file, or explicit HTTP(S) URL.
    pub source: InputReference,
    /// Endpoint template name.
    pub name: String,
    /// Endpoint API domain.
    pub domain: String,
    /// Optional API root path.
    pub api_root: Option<String>,
}
/// Endpoint JSON produced without filesystem effects.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpecImportPreviewOutput {
    /// Pretty JSON endpoint template.
    pub content: String,
}
impl ModelImportApplyInput {
    /// Persist metadata only after exact destination authorization and re-resolution.
    pub async fn apply(self, root: &Path, domain: &str, offline: bool) -> ApiResult<ModelImportApplyOutput> {
        let prepared = match matches!(self.destination.extension().and_then(|value| value.to_str()), Some("sqlite" | "db")) {
            | false => Err(eyre!("Model metadata destination must end in .sqlite or .db")),
            | true => ArtifactPolicy::new(root, true, self.overwrite).and_then(|policy| policy.destination(&self.destination, &[]).map(|_| policy)),
        };
        match prepared {
            | Ok(policy) => self.request.resolve(domain, offline).await.and_then(|resolution| {
                policy.prepare_destination(&self.destination, &[]).and_then(|destination| {
                    let (requested, resolved, weights) = resolution.into_parts();
                    let weight_count = weights.0.len();
                    Database::<Table>::from_path(Some(&destination))
                        .migrate_table(Table::Models)
                        .and_then(|_| weights.persist(&requested, Some(destination.clone())))
                        .map(|_| ModelImportApplyOutput {
                            metadata: ModelImportPreviewOutput {
                                requested,
                                resolved,
                                weight_count,
                            },
                            artifacts: vec![destination.display().to_string()],
                            destination,
                        })
                })
            }),
            | Err(why) => Err(why),
        }
    }
}
impl ModelImportInput {
    fn is_valid_repository_character(character: char) -> bool {
        character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
    }
    fn is_valid_repository_part(part: &str) -> bool {
        !matches!(part, "." | "..") && part.chars().all(Self::is_valid_repository_character)
    }
    async fn resolve(&self, domain: &str, offline: bool) -> ApiResult<RepositoryResolution<Weights>> {
        let validated = match self.identifier.len() > 256 || self.identifier.trim() != self.identifier {
            | true => Err(eyre!("Model repository identifier exceeds the byte limit or contains outer whitespace")),
            | false => huggingface::parse_identifier(&self.identifier).and_then(|(owner, name)| {
                match [owner, name].iter().all(|part| Self::is_valid_repository_part(part)) {
                    | true => Ok(()),
                    | false => Err(eyre!("Model repository identifier contains unsupported characters")),
                }
            }),
        };
        match validated {
            | Ok(()) => {
                let options = huggingface::Options::init()
                    .domain(domain)
                    .identifier(&self.identifier)
                    .no_fallback(true)
                    .quiet(true)
                    .build();
                resolve_import_metadata(&options, offline).await
            }
            | Err(why) => Err(why),
        }
    }
    /// Preview explicit model metadata using the direct command's resolver.
    pub async fn preview(self, domain: &str, offline: bool) -> ApiResult<ModelImportPreviewOutput> {
        self.resolve(domain, offline).await.map(|resolution| ModelImportPreviewOutput {
            requested: resolution.requested().to_string(),
            resolved: resolution.resolved().to_string(),
            weight_count: resolution.value().0.len(),
        })
    }
}
impl SpecImportApplyInput {
    /// Write the reviewed endpoint template after revalidating its exact destination.
    pub async fn apply(self, root: &Path, offline: bool) -> ApiResult<SpecImportApplyOutput> {
        let sources = match MimeType::from_path(&self.destination) {
            | MimeType::Json => match &self.request.source {
                | InputReference::File(path) => RoutedInputPolicy::new(root, InputLimits::default())
                    .and_then(|policy| policy.resolve(&InputReference::File(path.clone())))
                    .map(|resolved| resolved.files.into_iter().map(|file| file.path).collect::<Vec<_>>()),
                | _ => Ok(Vec::new()),
            },
            | _ => Err(eyre!("OpenAPI endpoint destination must end in .json")),
        };
        let prepared = sources.and_then(|sources| {
            ArtifactPolicy::new(root, true, self.overwrite).and_then(|policy| {
                policy
                    .destination(&self.destination, &sources)
                    .and_then(|destination| match sources.contains(&destination) {
                        | true => Err(eyre!("OpenAPI endpoint destination must differ from the source")),
                        | false => Ok((sources, policy)),
                    })
            })
        });
        match prepared {
            | Ok((sources, policy)) => self.request.preview(root, offline).await.and_then(|preview| {
                policy.prepare_destination(&self.destination, &sources).and_then(|destination| {
                    write(&destination, format!("{}\n", preview.content))
                        .map_err(|why| eyre!("Write OpenAPI endpoint JSON {} — {why}", destination.display()))
                        .map(|()| SpecImportApplyOutput {
                            artifacts: vec![destination.display().to_string()],
                            destination,
                        })
                })
            }),
            | Err(why) => Err(why),
        }
    }
}
impl SpecImportInput {
    /// Preview one OpenAPI endpoint template with bounded local or remote input.
    pub async fn preview(self, root: &Path, offline: bool) -> ApiResult<SpecImportPreviewOutput> {
        let policy = RoutedInputPolicy::new(root, InputLimits::default());
        let spec = match (policy, &self.source) {
            | (Ok(policy), InputReference::Text(_) | InputReference::File(_)) => {
                policy
                    .resolve(&self.source)
                    .and_then(|resolved| match (resolved.text, resolved.files.as_slice()) {
                        | (Some(text), []) => Ok(text),
                        | (None, [file]) => {
                            fs::read_to_string(&file.path).map_err(|why| eyre!("Read OpenAPI specification {} — {why}", file.path.display()))
                        }
                        | _ => Err(eyre!("OpenAPI import requires one text or file source")),
                    })
            }
            | (Ok(_), InputReference::Url(url)) => {
                let validated = reqwest::Url::parse(url)
                    .map_err(|why| eyre!("Invalid OpenAPI URL — {why}"))
                    .and_then(|parsed| {
                        match matches!(parsed.scheme(), "http" | "https") && parsed.username().is_empty() && parsed.password().is_none() {
                            | true => Ok(()),
                            | false => Err(eyre!("OpenAPI source must be an HTTP(S) URL without embedded credentials")),
                        }
                    });
                match validated {
                    | Ok(()) => Source::read(url, offline).await,
                    | Err(why) => Err(why),
                }
            }
            | (Ok(_), _) => Err(eyre!("OpenAPI import supports only text, local file, or HTTP(S) URL sources")),
            | (Err(why), _) => Err(why),
        };
        spec.and_then(|spec| match InputLimits::default().max_total.can_contain(spec.len() as u64) {
            | Some(true) => Endpoint::from_openapi(&spec, &self.name, &self.domain, self.api_root.as_deref(), None)
                .and_then(|endpoint| serde_json::to_string_pretty(&endpoint).map_err(|why| eyre!("Serialize OpenAPI endpoint JSON — {why}")))
                .map(|content| SpecImportPreviewOutput { content }),
            | _ => Err(eyre!("OpenAPI specification exceeds the byte limit")),
        })
    }
}
impl ToolRegistry {
    /// Register read-only OpenAPI endpoint import preview.
    pub fn register_import_spec_preview_capability(self, root: impl AsRef<Path>, offline: bool) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let effects = ToolEffects::from(NativeEffect::RemoteRead);
        let register = self.register_capability(CapabilityDefinition::new::<SpecImportInput, SpecImportPreviewOutput>(
            format!("{APPLICATION}.import::spec::preview"),
            version,
            "Preview an endpoint template from explicit OpenAPI text, file, or URL.",
            effects,
        ));
        register.and_then(|registry| {
            let binding = BindingDefinition::new::<SpecImportInput, SpecImportPreviewOutput>(
                format!("native:{APPLICATION}.import::spec::preview"),
                format!("{APPLICATION}.import::spec::preview"),
                version,
                BindingKind::Native,
                effects,
                BindingAvailability::Ready,
            );
            let binding = if offline { binding.with_offline_support() } else { binding };
            registry.register_capability_binding::<SpecImportInput, SpecImportPreviewOutput, _, _>(binding, move |input| {
                let root = root.clone();
                async move { input.preview(&root, offline).await }
            })
        })
    }
    /// Register exact-authority OpenAPI endpoint artifact writing.
    pub fn register_import_spec_apply_capability(self, root: impl AsRef<Path>, offline: bool) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let effects = ToolEffects::from(NativeEffect::RemoteWrite);
        let register = self.register_capability(CapabilityDefinition::new::<SpecImportApplyInput, SpecImportApplyOutput>(
            format!("{APPLICATION}.import::spec::apply"),
            version,
            "Write one confirmed OpenAPI endpoint JSON artifact at an exact destination.",
            effects,
        ));
        register.and_then(|registry| {
            let binding = BindingDefinition::new::<SpecImportApplyInput, SpecImportApplyOutput>(
                format!("native:{APPLICATION}.import::spec::apply"),
                format!("{APPLICATION}.import::spec::apply"),
                version,
                BindingKind::Native,
                effects,
                BindingAvailability::Ready,
            );
            let binding = if offline { binding.with_offline_support() } else { binding };
            registry.register_capability_binding::<SpecImportApplyInput, SpecImportApplyOutput, _, _>(binding, move |input| {
                let root = root.clone();
                async move { input.apply(&root, offline).await }
            })
        })
    }
    /// Register read-only metadata import for an explicit Hugging Face repository.
    pub fn register_import_model_preview_capability(self, domain: String, offline: bool) -> ApiResult<Self> {
        let version = SemanticVersion::from((1, 0, 0));
        let effects = ToolEffects::from(NativeEffect::RemoteRead);
        let register = self.register_capability(CapabilityDefinition::new::<ModelImportInput, ModelImportPreviewOutput>(
            format!("{APPLICATION}.import::model::preview"),
            version,
            "Preview GGUF metadata for one explicit Hugging Face repository without downloading weights.",
            effects,
        ));
        register.and_then(|registry| {
            let binding = BindingDefinition::new::<ModelImportInput, ModelImportPreviewOutput>(
                format!("native:{APPLICATION}.import::model::preview"),
                format!("{APPLICATION}.import::model::preview"),
                version,
                BindingKind::Native,
                effects,
                BindingAvailability::Ready,
            );
            let binding = if offline { binding.with_offline_support() } else { binding };
            registry.register_capability_binding::<ModelImportInput, ModelImportPreviewOutput, _, _>(binding, move |input| {
                let domain = domain.clone();
                async move { input.preview(&domain, offline).await }
            })
        })
    }
    /// Register exact-authority persistence of one resolved model metadata record.
    pub fn register_import_model_apply_capability(self, root: impl AsRef<Path>, domain: String, offline: bool) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let effects = ToolEffects::from(NativeEffect::RemoteWrite);
        let register = self.register_capability(CapabilityDefinition::new::<ModelImportApplyInput, ModelImportApplyOutput>(
            format!("{APPLICATION}.import::model::apply"),
            version,
            "Persist GGUF metadata for one explicit Hugging Face repository in an authorized local database.",
            effects,
        ));
        register.and_then(|registry| {
            let binding = BindingDefinition::new::<ModelImportApplyInput, ModelImportApplyOutput>(
                format!("native:{APPLICATION}.import::model::apply"),
                format!("{APPLICATION}.import::model::apply"),
                version,
                BindingKind::Native,
                effects,
                BindingAvailability::Ready,
            );
            let binding = if offline { binding.with_offline_support() } else { binding };
            registry.register_capability_binding::<ModelImportApplyInput, ModelImportApplyOutput, _, _>(binding, move |input| {
                let root = root.clone();
                let domain = domain.clone();
                async move { input.apply(&root, &domain, offline).await }
            })
        })
    }
}
