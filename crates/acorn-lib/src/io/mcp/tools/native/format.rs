//! Bounded format and crosswalk previews with exact destination writes.
use super::super::{
    ArtifactPolicy, BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, CrosswalkInput, CrosswalkOutput, InputFailure,
    InputLimits, InputReference, RoutedInputPolicy, ToolRegistry,
};
use super::NativeEffect;
use crate::error::ApiResult;
use crate::io::config::application::ApplicationConfiguration;
use crate::io::enrichment::{Enrich, EnrichmentMetadata, Pipeline};
use crate::io::{FromPath, InputOutput};
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::{MimeType, SemanticVersion};
use acorn_core::Location;
use acorn_diff::unified_diff;
use acorn_schema::agent::tools::ToolEffects;
use acorn_schema::enrichment::Provider;
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::standard::crosswalk::CrosswalkStandard;
use acorn_schema::standard::{cff::Cff, datacite, dublin_core::DublinCoreDocument, invenio};
use acorn_schema::validation::Validate;
use acorn_schema::vocabulary::{VocabularyFuzzyMode, VocabularyReport};
use color_eyre::eyre::eyre;
use dprint_core::configuration::NewLineKind;
use dprint_plugin_json::{
    configuration::{ConfigurationBuilder, EofNewLineKind, TrailingCommaKind},
    format_text as format_json,
};
use pretty_yaml::{config::FormatOptions, format_text as format_yaml};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs::{self, read_to_string};
use std::path::{Path, PathBuf};
use strum::IntoEnumIterator;

/// Media types supported by the existing standards crosswalk.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CrosswalkFormat {
    /// JSON.
    Json,
    /// JSON-LD for Dublin Core.
    LdJson,
    /// RDF/XML for Dublin Core.
    RdfXml,
    /// Turtle for Dublin Core.
    Turtle,
    /// YAML.
    Yaml,
    /// ZON.
    Zon,
}
/// Standards currently supported by the direct format command.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FormatStandard {
    /// Application Configuration for Research Enablement.
    ApplicationConfigurationResearchEnablement,
    /// Citation File Format.
    CitationFileFormat,
    /// DataCite metadata.
    Datacite,
    /// Dublin Core metadata.
    DublinCore,
    /// InvenioRDM metadata.
    Invenio,
    /// Research Activity Data.
    ResearchActivityData,
}
/// An exact, authorized crosswalk output request.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CrosswalkApplyInput {
    /// One confirmed crosswalk request.
    pub input: CrosswalkPreviewInput,
    /// Destination relative to the permitted root.
    pub destination: PathBuf,
    /// Permit replacing an existing output.
    #[serde(default)]
    pub overwrite: bool,
}
/// Written crosswalk artifact and its field-loss warnings.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CrosswalkApplyOutput {
    /// Exact path written.
    pub destination: PathBuf,
    /// Conversion result.
    pub conversion: CrosswalkPreviewOutput,
}
/// A bounded content crosswalk request.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CrosswalkPreviewInput {
    /// Literal text or one local file.
    pub input: InputReference,
    /// Source standard.
    pub source: CrosswalkStandard,
    /// Target standard.
    pub target: CrosswalkStandard,
    /// Input media type, inferred from a file when omitted.
    pub input_format: Option<CrosswalkFormat>,
    /// Output media type.
    pub output_format: Option<CrosswalkFormat>,
}
/// Converted content and explicit field-loss warnings.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CrosswalkPreviewOutput {
    /// Converted metadata.
    pub content: String,
    /// Loss and approximation warnings.
    pub warnings: Vec<String>,
}
/// An exact, authorized format destination.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormatApplyInput {
    /// One file to normalize.
    pub input: FormatPreviewInput,
    /// Destination relative to the permitted root.
    pub destination: PathBuf,
    /// Permit replacing an existing file, including the source.
    #[serde(default)]
    pub overwrite: bool,
}
/// Result of writing a validated normalized file.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormatApplyOutput {
    /// Exact validated destination.
    pub destination: PathBuf,
    /// Whether normalized content differed and was written.
    pub applied: bool,
    /// The reviewed change that was applied.
    pub change: FormatChange,
}
/// One normalized document and reviewable difference.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormatChange {
    /// Canonical input file.
    pub source: PathBuf,
    /// Original file text.
    pub original: String,
    /// Proposed replacement text.
    pub content: String,
    /// Unified difference; empty when no changes are needed.
    pub diff: String,
    /// Vocabulary or conversion diagnostics.
    pub warnings: Vec<String>,
    /// Provider conflicts.
    pub conflicts: Vec<String>,
    /// Provider failures.
    pub failures: Vec<String>,
}
/// Shared plain-format calculation used by the CLI and native route.
pub struct FormatNormalization {
    /// Original file content for audit or rollback.
    pub original: String,
    /// Original parsed metadata serialized in its source format.
    pub before: String,
    /// Normalized metadata serialized in its source format.
    pub content: String,
    /// Controlled-vocabulary resolution reports.
    pub reports: Vec<VocabularyReport>,
}
/// One local metadata file or directory and its normalization options.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormatPreviewInput {
    /// Explicit file or directory below the permitted root.
    pub input: InputReference,
    /// Metadata standard to parse.
    pub standard: FormatStandard,
    /// Request provider enrichment after normalization.
    #[serde(default)]
    pub enrich: bool,
    /// Selected providers; an empty list uses the command defaults.
    #[serde(default)]
    pub providers: Vec<Provider>,
    /// Vocabulary normalization policy.
    #[serde(default)]
    pub fuzzy: VocabularyFuzzyMode,
}
/// Bounded normalization result, with nonfatal directory failures.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormatPreviewOutput {
    /// Changes in stable path order.
    pub changes: Vec<FormatChange>,
    /// Directory entries that could not be inspected.
    pub failures: Vec<InputFailure>,
}
impl CrosswalkApplyInput {
    /// Convert and write after revalidating the exact output path.
    pub fn apply(self, root: &Path) -> ApiResult<CrosswalkApplyOutput> {
        ArtifactPolicy::new(root, true, self.overwrite).and_then(|policy| {
            policy
                .destination(&self.destination, &[])
                .and_then(|_| self.input.preview(root))
                .and_then(|conversion| {
                    policy.destination(&self.destination, &[]).and_then(|destination| {
                        fs::write(&destination, &conversion.content)
                            .map_err(|why| eyre!("Write crosswalk output {} — {why}", destination.display()))
                            .map(|()| CrosswalkApplyOutput { destination, conversion })
                    })
                })
        })
    }
}
impl TryFrom<MimeType> for CrosswalkFormat {
    type Error = color_eyre::Report;
    fn try_from(mime: MimeType) -> ApiResult<Self> {
        match mime {
            | MimeType::Json => Ok(Self::Json),
            | MimeType::LdJson => Ok(Self::LdJson),
            | MimeType::RdfXml => Ok(Self::RdfXml),
            | MimeType::Turtle => Ok(Self::Turtle),
            | MimeType::Yaml => Ok(Self::Yaml),
            | MimeType::Zon => Ok(Self::Zon),
            | _ => Err(eyre!("Crosswalk input file must use a supported media type")),
        }
    }
}
impl CrosswalkPreviewInput {
    /// Convert one bounded value using the existing standards crosswalk and loss report.
    pub fn preview(self, root: &Path) -> ApiResult<CrosswalkPreviewOutput> {
        RoutedInputPolicy::new(root, InputLimits::default())
            .and_then(|policy| policy.resolve(&self.input))
            .and_then(|resolved| match (resolved.text, resolved.files.as_slice()) {
                | (Some(text), []) => Ok((text, None)),
                | (None, [file]) => read_to_string(&file.path)
                    .map_err(|why| eyre!("Read crosswalk input {} — {why}", file.path.display()))
                    .and_then(|content| CrosswalkFormat::try_from(MimeType::from_path(&file.path)).map(|format| (content, Some(format)))),
                | _ => Err(eyre!("Crosswalk requires one literal or local file")),
            })
            .and_then(|(content, inferred)| {
                if InputLimits::default().max_total.can_contain(content.len() as u64) == Some(true) {
                    CrosswalkOutput::try_from(CrosswalkInput {
                        content,
                        source: self.source,
                        target: self.target,
                        input_format: self.input_format.or(inferred).map(Into::into),
                        output_format: self.output_format.map(Into::into),
                    })
                    .map(|output| CrosswalkPreviewOutput {
                        content: output.content,
                        warnings: output.warnings,
                    })
                } else {
                    Err(eyre!("Crosswalk input exceeds the byte limit"))
                }
            })
    }
}
impl FormatApplyInput {
    /// Apply one reviewed normalization after revalidating its destination.
    pub async fn apply(self, root: &Path, offline: bool) -> ApiResult<FormatApplyOutput> {
        let policy = match self.input.input {
            | InputReference::File(_) => ArtifactPolicy::new(root, true, self.overwrite),
            | _ => Err(eyre!("Format apply requires one explicit source file")),
        }
        .and_then(|policy| policy.destination(&self.destination, &[]).map(|_| policy));
        match policy {
            | Ok(policy) => self.input.preview(root, offline).await.and_then(|mut preview| {
                preview
                    .changes
                    .pop()
                    .ok_or_else(|| eyre!("No metadata file was formatted"))
                    .and_then(|change| {
                        policy
                            .destination(&self.destination, core::slice::from_ref(&change.source))
                            .and_then(|destination| {
                                let applied = !change.diff.is_empty() || destination != change.source;
                                let write = if applied {
                                    fs::write(&destination, &change.content)
                                        .map_err(|why| eyre!("Write formatted metadata {} — {why}", destination.display()))
                                } else {
                                    Ok(())
                                };
                                write.map(|()| FormatApplyOutput {
                                    destination,
                                    applied,
                                    change,
                                })
                            })
                    })
            }),
            | Err(why) => Err(why),
        }
    }
}
impl FormatPreviewInput {
    /// Normalize bounded local metadata through the same format trait as the CLI.
    pub async fn preview(self, root: &Path, offline: bool) -> ApiResult<FormatPreviewOutput> {
        let validated = match (self.enrich, self.providers.is_empty(), offline) {
            | (false, false, _) => Err(eyre!("Providers require enrichment")),
            | (true, _, true) => Err(eyre!("Enrichment is unavailable in offline mode")),
            | _ => RoutedInputPolicy::new(root, InputLimits::default()).and_then(|policy| policy.resolve(&self.input)),
        }
        .and_then(|resolved| {
            if resolved.files.is_empty() || resolved.text.is_some() {
                Err(eyre!("Formatting requires a local file or directory"))
            } else {
                Ok(resolved)
            }
        });
        match validated {
            | Ok(resolved) => {
                let mut output = Ok(FormatPreviewOutput {
                    changes: Vec::new(),
                    failures: resolved.failures,
                });
                for file in resolved.files {
                    if !matches!(self.input, InputReference::Directory(_)) || selected_directory_format_file(&file.path, self.standard) {
                        output = match output {
                            | Ok(mut preview) => match self.preview_file(&file.path).await {
                                | Ok(change) => {
                                    preview.changes.push(change);
                                    Ok(preview)
                                }
                                | Err(_) if matches!(self.input, InputReference::Directory(_)) => {
                                    preview.failures.push(InputFailure {
                                        path: file.path,
                                        reason: "metadata formatting failed".to_string(),
                                    });
                                    Ok(preview)
                                }
                                | Err(why) => Err(why),
                            },
                            | Err(why) => Err(why),
                        };
                    }
                }
                output
            }
            | Err(why) => Err(why),
        }
    }
    async fn preview_file(&self, path: &Path) -> ApiResult<FormatChange> {
        match read_to_string(path).map_err(|why| eyre!("Read format input {} — {why}", path.display())) {
            | Ok(original) if InputLimits::default().max_total.can_contain(original.len() as u64) == Some(true) => {
                let formatted = match self.standard {
                    | FormatStandard::ApplicationConfigurationResearchEnablement if !self.enrich => {
                        format_acre(path, &original).map(|content| (original.clone(), content, Vec::new(), Vec::new(), Vec::new()))
                    }
                    | FormatStandard::CitationFileFormat => format_metadata::<Cff>(path, self).await,
                    | FormatStandard::Datacite => format_metadata::<datacite::Record>(path, self).await,
                    | FormatStandard::DublinCore if !self.enrich => {
                        normalize_metadata_file::<DublinCoreDocument>(path, self.fuzzy).map(|normalized| {
                            (
                                normalized.before,
                                normalized.content,
                                normalized.reports.iter().map(|report| format!("{report:?}")).collect(),
                                Vec::new(),
                                Vec::new(),
                            )
                        })
                    }
                    | FormatStandard::Invenio => format_metadata::<invenio::Record>(path, self).await,
                    | FormatStandard::ResearchActivityData => format_metadata::<ResearchActivity>(path, self).await,
                    | _ => Err(eyre!("Formatting is unsupported for the selected standard or enrichment mode")),
                };
                formatted.map(|(before, content, warnings, conflicts, failures)| {
                    let diff_before = if self.enrich { &original } else { &before };
                    let diff = if diff_before == &content {
                        String::new()
                    } else {
                        unified_diff(diff_before, &content, &format!("a/{}", path.display()), &format!("b/{}", path.display()))
                    };
                    FormatChange {
                        source: path.to_path_buf(),
                        original,
                        content,
                        diff,
                        warnings,
                        conflicts,
                        failures,
                    }
                })
            }
            | Ok(_) => Err(eyre!("Format input exceeds the byte limit")),
            | Err(why) => Err(why),
        }
    }
}
impl From<CrosswalkFormat> for MimeType {
    fn from(format: CrosswalkFormat) -> Self {
        match format {
            | CrosswalkFormat::Json => Self::Json,
            | CrosswalkFormat::LdJson => Self::LdJson,
            | CrosswalkFormat::RdfXml => Self::RdfXml,
            | CrosswalkFormat::Turtle => Self::Turtle,
            | CrosswalkFormat::Yaml => Self::Yaml,
            | CrosswalkFormat::Zon => Self::Zon,
        }
    }
}
impl ToolRegistry {
    /// Register format preview, authorized apply, and loss-aware crosswalk capabilities.
    pub fn register_format_capabilities(self, root: impl AsRef<Path>, offline: bool) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let read = ToolEffects::from(NativeEffect::LocalRead);
        let write = ToolEffects {
            read_only: false,
            destructive: true,
            ..read
        };
        let enrich_read = ToolEffects { open_world: true, ..read };
        let enrich_write = ToolEffects {
            read_only: false,
            destructive: true,
            open_world: true,
            ..read
        };
        let preview_root = root.clone();
        let apply_root = root.clone();
        let enrich_preview_root = root.clone();
        let enrich_apply_root = root.clone();
        let crosswalk_root = root.clone();
        self.register_capability(CapabilityDefinition::new::<FormatPreviewInput, FormatPreviewOutput>(
            format!("{APPLICATION}.format::preview"),
            version,
            "Preview bounded metadata normalization.",
            read,
        ))
        .and_then(|registry| {
            registry.register_capability_binding::<FormatPreviewInput, FormatPreviewOutput, _, _>(
                BindingDefinition::new::<FormatPreviewInput, FormatPreviewOutput>(
                    format!("native:{APPLICATION}.format::preview"),
                    format!("{APPLICATION}.format::preview"),
                    version,
                    BindingKind::Native,
                    read,
                    BindingAvailability::Ready,
                )
                .with_offline_support(),
                move |input: FormatPreviewInput| {
                    let root = preview_root.clone();
                    async move {
                        if input.enrich {
                            Err(eyre!("Use format_enrich_preview for provider access"))
                        } else {
                            input.preview(&root, offline).await
                        }
                    }
                },
            )
        })
        .and_then(|registry| {
            registry.register_capability(CapabilityDefinition::new::<FormatApplyInput, FormatApplyOutput>(
                format!("{APPLICATION}.format::apply"),
                version,
                "Write one authorized normalized metadata file.",
                write,
            ))
        })
        .and_then(|registry| {
            registry.register_capability_binding::<FormatApplyInput, FormatApplyOutput, _, _>(
                BindingDefinition::new::<FormatApplyInput, FormatApplyOutput>(
                    format!("native:{APPLICATION}.format::apply"),
                    format!("{APPLICATION}.format::apply"),
                    version,
                    BindingKind::Native,
                    write,
                    BindingAvailability::Ready,
                )
                .with_offline_support(),
                move |input: FormatApplyInput| {
                    let root = apply_root.clone();
                    async move {
                        if input.input.enrich {
                            Err(eyre!("Use format_enrich_apply for provider access"))
                        } else {
                            input.apply(&root, offline).await
                        }
                    }
                },
            )
        })
        .and_then(|registry| {
            registry.register_capability(CapabilityDefinition::new::<FormatPreviewInput, FormatPreviewOutput>(
                format!("{APPLICATION}.format::enrich::preview"),
                version,
                "Preview metadata normalization with provider enrichment.",
                enrich_read,
            ))
        })
        .and_then(|registry| {
            registry.register_capability_binding::<FormatPreviewInput, FormatPreviewOutput, _, _>(
                BindingDefinition::new::<FormatPreviewInput, FormatPreviewOutput>(
                    format!("native:{APPLICATION}.format::enrich::preview"),
                    format!("{APPLICATION}.format::enrich::preview"),
                    version,
                    BindingKind::Native,
                    enrich_read,
                    BindingAvailability::Ready,
                ),
                move |input: FormatPreviewInput| {
                    let root = enrich_preview_root.clone();
                    async move {
                        if input.enrich {
                            input.preview(&root, offline).await
                        } else {
                            Err(eyre!("Enrichment route requires enrich=true"))
                        }
                    }
                },
            )
        })
        .and_then(|registry| {
            registry.register_capability(CapabilityDefinition::new::<FormatApplyInput, FormatApplyOutput>(
                format!("{APPLICATION}.format::enrich::apply"),
                version,
                "Write one authorized enriched metadata file.",
                enrich_write,
            ))
        })
        .and_then(|registry| {
            registry.register_capability_binding::<FormatApplyInput, FormatApplyOutput, _, _>(
                BindingDefinition::new::<FormatApplyInput, FormatApplyOutput>(
                    format!("native:{APPLICATION}.format::enrich::apply"),
                    format!("{APPLICATION}.format::enrich::apply"),
                    version,
                    BindingKind::Native,
                    enrich_write,
                    BindingAvailability::Ready,
                ),
                move |input: FormatApplyInput| {
                    let root = enrich_apply_root.clone();
                    async move {
                        if input.input.enrich {
                            input.apply(&root, offline).await
                        } else {
                            Err(eyre!("Enrichment route requires enrich=true"))
                        }
                    }
                },
            )
        })
        .and_then(|registry| {
            registry.register_capability(CapabilityDefinition::new::<CrosswalkPreviewInput, CrosswalkPreviewOutput>(
                format!("{APPLICATION}.crosswalk::preview"),
                version,
                "Preview bounded standards conversion with field-loss warnings.",
                read,
            ))
        })
        .and_then(|registry| {
            registry.register_capability_binding::<CrosswalkPreviewInput, CrosswalkPreviewOutput, _, _>(
                BindingDefinition::new::<CrosswalkPreviewInput, CrosswalkPreviewOutput>(
                    format!("native:{APPLICATION}.crosswalk::preview"),
                    format!("{APPLICATION}.crosswalk::preview"),
                    version,
                    BindingKind::Native,
                    read,
                    BindingAvailability::Ready,
                )
                .with_offline_support(),
                move |input| {
                    let root = crosswalk_root.clone();
                    async move { input.preview(&root) }
                },
            )
        })
        .and_then(|registry| {
            registry.register_capability(CapabilityDefinition::new::<CrosswalkApplyInput, CrosswalkApplyOutput>(
                format!("{APPLICATION}.crosswalk::apply"),
                version,
                "Write one authorized standards conversion with loss warnings.",
                write,
            ))
        })
        .and_then(|registry| {
            registry.register_capability_binding::<CrosswalkApplyInput, CrosswalkApplyOutput, _, _>(
                BindingDefinition::new::<CrosswalkApplyInput, CrosswalkApplyOutput>(
                    format!("native:{APPLICATION}.crosswalk::apply"),
                    format!("{APPLICATION}.crosswalk::apply"),
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
/// Validate and format ACRE content while retaining comments and trailing commas.
pub fn format_acre(path: &Path, original: &str) -> ApiResult<String> {
    ApplicationConfiguration::read(path.to_path_buf())
        .and_then(|configuration| configuration.validate().map_err(|why| eyre!("Validate ACRE configuration — {why}")))
        .and_then(|()| match MimeType::from_path(path) {
            | MimeType::Json | MimeType::Jsonc => {
                let configuration = ConfigurationBuilder::new()
                    .use_tabs(false)
                    .indent_width(2)
                    .new_line_kind(NewLineKind::LineFeed)
                    .eof_new_line(EofNewLineKind::Always)
                    .package_json_apply_conventions(false)
                    .trailing_commas(TrailingCommaKind::Maintain)
                    .build();
                format_json(path, original, &configuration)
                    .map(|content| content.unwrap_or_else(|| original.to_string()))
                    .map_err(|why| eyre!("Format ACRE JSON/JSONC — {why}"))
            }
            | MimeType::Yaml => format_yaml(original, &FormatOptions::default())
                .map(|content| if content.ends_with('\n') { content } else { format!("{content}\n") })
                .map_err(|why| eyre!("Format ACRE YAML — {why}")),
            | _ => Err(eyre!("ACRE formatting supports JSON, JSONC, YAML, and YML files")),
        })
}
async fn format_metadata<T>(path: &Path, input: &FormatPreviewInput) -> ApiResult<(String, String, Vec<String>, Vec<String>, Vec<String>)>
where
    T: Clone + DeserializeOwned + EnrichmentMetadata + InputOutput + Send + Serialize + Sync + Validate + 'static,
{
    let normalized = normalize_metadata_file::<T>(path, input.fuzzy).and_then(|normalized| {
        if InputLimits::default().max_total.can_contain(normalized.original.len() as u64) == Some(true) {
            Ok(normalized)
        } else {
            Err(eyre!("Format input exceeds the byte limit"))
        }
    });
    match normalized {
        | Ok(normalized) if input.enrich => {
            let providers = if input.providers.is_empty() {
                if input.standard == FormatStandard::ResearchActivityData {
                    Provider::iter().collect()
                } else {
                    vec![Provider::Orcid]
                }
            } else {
                input.providers.clone()
            };
            if input.standard != FormatStandard::ResearchActivityData && providers.iter().any(|provider| *provider != Provider::Orcid) {
                Err(eyre!("Only ORCID enrichment is supported for this standard"))
            } else {
                match T::read(path.to_path_buf()) {
                    | Ok(data) => {
                        let data = data.format_with_vocabulary(input.fuzzy, Some(path.to_path_buf()));
                        Pipeline::new(providers)
                            .enrich((Location::from(&path.display().to_string()), data))
                            .await
                            .and_then(|result| {
                                result
                                    .data
                                    .format_with_vocabulary(input.fuzzy, Some(path.to_path_buf()))
                                    .serialize_as(&MimeType::from_path(path))
                                    .map(|content| {
                                        (
                                            normalized.before,
                                            content,
                                            normalized.reports.iter().map(|report| format!("{report:?}")).collect(),
                                            result.conflicts,
                                            result.failures,
                                        )
                                    })
                            })
                    }
                    | Err(why) => Err(why),
                }
            }
        }
        | Ok(normalized) => Ok((
            normalized.before,
            normalized.content,
            normalized.reports.iter().map(|report| format!("{report:?}")).collect(),
            Vec::new(),
            Vec::new(),
        )),
        | Err(why) => Err(why),
    }
}
/// Parse and normalize one local metadata file without writing it.
pub fn normalize_metadata_file<T>(path: &Path, fuzzy: VocabularyFuzzyMode) -> ApiResult<FormatNormalization>
where
    T: InputOutput + Serialize,
{
    read_to_string(path)
        .map_err(|why| eyre!("Read original metadata — {why}"))
        .and_then(|original| {
            T::read(path.to_path_buf()).and_then(|data| {
                let mime = MimeType::from_path(path);
                data.serialize_as(&mime).and_then(|before| {
                    let formatted = data.format_with_vocabulary_report(fuzzy, Some(path.to_path_buf()));
                    formatted.value.serialize_as(&mime).map(|content| FormatNormalization {
                        original,
                        before,
                        content,
                        reports: formatted.reports,
                    })
                })
            })
        })
}
fn selected_directory_format_file(path: &Path, standard: FormatStandard) -> bool {
    match standard {
        | FormatStandard::ApplicationConfigurationResearchEnablement => path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| matches!(name, ".acorn.json" | ".acorn.jsonc" | ".acorn.yaml" | ".acorn.yml")),
        | _ => {
            let ext = MimeType::from_path(path);
            ext.is_json() || ext.is_jsonc() || ext.is_markdown() || ext.is_pdf() || ext.is_yaml() || ext.is_zon()
        }
    }
}
