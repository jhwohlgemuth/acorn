//! Bounded format and crosswalk previews with exact destination writes.
use super::super::{
    ArtifactPolicy, BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, CrosswalkInput, CrosswalkOutput, InputFailure,
    InputLimits, InputReference, RoutedInputPolicy, ToolRegistry,
};
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
use std::fs;
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
        let policy = ArtifactPolicy::new(root, true, self.overwrite)?;
        policy.destination(&self.destination, &[])?;
        let conversion = self.input.preview(root)?;
        let destination = policy.destination(&self.destination, &[])?;
        fs::write(&destination, &conversion.content).map_err(|why| eyre!("Write crosswalk output {} — {why}", destination.display()))?;
        Ok(CrosswalkApplyOutput { destination, conversion })
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
        let policy = RoutedInputPolicy::new(root, InputLimits::default())?;
        let resolved = policy.resolve(&self.input)?;
        let (content, inferred) = match (resolved.text, resolved.files.as_slice()) {
            | (Some(text), []) => (text, None),
            | (None, [file]) => (
                fs::read_to_string(&file.path).map_err(|why| eyre!("Read crosswalk input {} — {why}", file.path.display()))?,
                Some(CrosswalkFormat::try_from(MimeType::from_path(&file.path))?),
            ),
            | _ => return Err(eyre!("Crosswalk requires one literal or local file")),
        };
        if InputLimits::default().max_total.can_contain(content.len() as u64) != Some(true) {
            return Err(eyre!("Crosswalk input exceeds the byte limit"));
        }
        let output = CrosswalkOutput::try_from(CrosswalkInput {
            content,
            source: self.source,
            target: self.target,
            input_format: self.input_format.or(inferred).map(Into::into),
            output_format: self.output_format.map(Into::into),
        })?;
        Ok(CrosswalkPreviewOutput {
            content: output.content,
            warnings: output.warnings,
        })
    }
}
impl FormatApplyInput {
    /// Apply one reviewed normalization after revalidating its destination.
    pub async fn apply(self, root: &Path, offline: bool) -> ApiResult<FormatApplyOutput> {
        if !matches!(self.input.input, InputReference::File(_)) {
            return Err(eyre!("Format apply requires one explicit source file"));
        }
        let policy = ArtifactPolicy::new(root, true, self.overwrite)?;
        policy.destination(&self.destination, &[])?;
        let mut preview = self.input.preview(root, offline).await?;
        let change = preview.changes.pop().ok_or_else(|| eyre!("No metadata file was formatted"))?;
        let destination = policy.destination(&self.destination, core::slice::from_ref(&change.source))?;
        let applied = !change.diff.is_empty() || destination != change.source;
        if applied {
            fs::write(&destination, &change.content).map_err(|why| eyre!("Write formatted metadata {} — {why}", destination.display()))?;
        }
        Ok(FormatApplyOutput {
            destination,
            applied,
            change,
        })
    }
}
impl FormatPreviewInput {
    /// Normalize bounded local metadata through the same format trait as the CLI.
    pub async fn preview(self, root: &Path, offline: bool) -> ApiResult<FormatPreviewOutput> {
        if !self.enrich && !self.providers.is_empty() {
            return Err(eyre!("Providers require enrichment"));
        }
        if self.enrich && offline {
            return Err(eyre!("Enrichment is unavailable in offline mode"));
        }
        let policy = RoutedInputPolicy::new(root, InputLimits::default())?;
        let resolved = policy.resolve(&self.input)?;
        if resolved.files.is_empty() || resolved.text.is_some() {
            return Err(eyre!("Formatting requires a local file or directory"));
        }
        let mut changes = Vec::new();
        let mut failures = resolved.failures;
        for file in resolved.files {
            if matches!(self.input, InputReference::Directory(_)) && !selected_directory_format_file(&file.path, self.standard) {
                continue;
            }
            let result = self.preview_file(&file.path).await;
            match result {
                | Ok(change) => changes.push(change),
                | Err(_) if matches!(self.input, InputReference::Directory(_)) => failures.push(InputFailure {
                    path: file.path,
                    reason: "metadata formatting failed".to_string(),
                }),
                | Err(why) => return Err(why),
            }
        }
        Ok(FormatPreviewOutput { changes, failures })
    }
    async fn preview_file(&self, path: &Path) -> ApiResult<FormatChange> {
        let original = fs::read_to_string(path).map_err(|why| eyre!("Read format input {} — {why}", path.display()))?;
        if InputLimits::default().max_total.can_contain(original.len() as u64) != Some(true) {
            return Err(eyre!("Format input exceeds the byte limit"));
        }
        let (before, content, warnings, conflicts, failures) = match self.standard {
            | FormatStandard::ApplicationConfigurationResearchEnablement if !self.enrich => {
                (original.clone(), format_acre(path, &original)?, Vec::new(), Vec::new(), Vec::new())
            }
            | FormatStandard::CitationFileFormat => format_metadata::<Cff>(path, self).await?,
            | FormatStandard::Datacite => format_metadata::<datacite::Record>(path, self).await?,
            | FormatStandard::DublinCore if !self.enrich => {
                let normalized = normalize_metadata_file::<DublinCoreDocument>(path, self.fuzzy)?;
                (
                    normalized.before,
                    normalized.content,
                    normalized.reports.iter().map(|report| format!("{report:?}")).collect(),
                    Vec::new(),
                    Vec::new(),
                )
            }
            | FormatStandard::Invenio => format_metadata::<invenio::Record>(path, self).await?,
            | FormatStandard::ResearchActivityData => format_metadata::<ResearchActivity>(path, self).await?,
            | _ => return Err(eyre!("Formatting is unsupported for the selected standard or enrichment mode")),
        };
        let diff_before = if self.enrich { &original } else { &before };
        let diff = if diff_before == &content {
            String::new()
        } else {
            unified_diff(diff_before, &content, &format!("a/{}", path.display()), &format!("b/{}", path.display()))
        };
        Ok(FormatChange {
            source: path.to_path_buf(),
            original,
            content,
            diff,
            warnings,
            conflicts,
            failures,
        })
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
    pub fn register_format_capabilities(self, root: PathBuf, offline: bool) -> ApiResult<Self> {
        let version = SemanticVersion::from((1, 0, 0));
        let read = ToolEffects {
            read_only: true,
            destructive: false,
            idempotent: true,
            open_world: false,
        };
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
            format!("{APPLICATION}.format_preview"),
            version,
            "Preview bounded metadata normalization.",
            read,
        ))?
        .register_capability_binding::<FormatPreviewInput, FormatPreviewOutput, _, _>(
            BindingDefinition::new::<FormatPreviewInput, FormatPreviewOutput>(
                format!("native:{APPLICATION}.format_preview"),
                format!("{APPLICATION}.format_preview"),
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
                        return Err(eyre!("Use format_enrich_preview for provider access"));
                    }
                    input.preview(&root, offline).await
                }
            },
        )?
        .register_capability(CapabilityDefinition::new::<FormatApplyInput, FormatApplyOutput>(
            format!("{APPLICATION}.format_apply"),
            version,
            "Write one authorized normalized metadata file.",
            write,
        ))?
        .register_capability_binding::<FormatApplyInput, FormatApplyOutput, _, _>(
            BindingDefinition::new::<FormatApplyInput, FormatApplyOutput>(
                format!("native:{APPLICATION}.format_apply"),
                format!("{APPLICATION}.format_apply"),
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
                        return Err(eyre!("Use format_enrich_apply for provider access"));
                    }
                    input.apply(&root, offline).await
                }
            },
        )?
        .register_capability(CapabilityDefinition::new::<FormatPreviewInput, FormatPreviewOutput>(
            format!("{APPLICATION}.format_enrich_preview"),
            version,
            "Preview metadata normalization with provider enrichment.",
            enrich_read,
        ))?
        .register_capability_binding::<FormatPreviewInput, FormatPreviewOutput, _, _>(
            BindingDefinition::new::<FormatPreviewInput, FormatPreviewOutput>(
                format!("native:{APPLICATION}.format_enrich_preview"),
                format!("{APPLICATION}.format_enrich_preview"),
                version,
                BindingKind::Native,
                enrich_read,
                BindingAvailability::Ready,
            ),
            move |input: FormatPreviewInput| {
                let root = enrich_preview_root.clone();
                async move {
                    if !input.enrich {
                        return Err(eyre!("Enrichment route requires enrich=true"));
                    }
                    input.preview(&root, offline).await
                }
            },
        )?
        .register_capability(CapabilityDefinition::new::<FormatApplyInput, FormatApplyOutput>(
            format!("{APPLICATION}.format_enrich_apply"),
            version,
            "Write one authorized enriched metadata file.",
            enrich_write,
        ))?
        .register_capability_binding::<FormatApplyInput, FormatApplyOutput, _, _>(
            BindingDefinition::new::<FormatApplyInput, FormatApplyOutput>(
                format!("native:{APPLICATION}.format_enrich_apply"),
                format!("{APPLICATION}.format_enrich_apply"),
                version,
                BindingKind::Native,
                enrich_write,
                BindingAvailability::Ready,
            ),
            move |input: FormatApplyInput| {
                let root = enrich_apply_root.clone();
                async move {
                    if !input.input.enrich {
                        return Err(eyre!("Enrichment route requires enrich=true"));
                    }
                    input.apply(&root, offline).await
                }
            },
        )?
        .register_capability(CapabilityDefinition::new::<CrosswalkPreviewInput, CrosswalkPreviewOutput>(
            format!("{APPLICATION}.crosswalk_preview"),
            version,
            "Preview bounded standards conversion with field-loss warnings.",
            read,
        ))?
        .register_capability_binding::<CrosswalkPreviewInput, CrosswalkPreviewOutput, _, _>(
            BindingDefinition::new::<CrosswalkPreviewInput, CrosswalkPreviewOutput>(
                format!("native:{APPLICATION}.crosswalk_preview"),
                format!("{APPLICATION}.crosswalk_preview"),
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
        )?
        .register_capability(CapabilityDefinition::new::<CrosswalkApplyInput, CrosswalkApplyOutput>(
            format!("{APPLICATION}.crosswalk_apply"),
            version,
            "Write one authorized standards conversion with loss warnings.",
            write,
        ))?
        .register_capability_binding::<CrosswalkApplyInput, CrosswalkApplyOutput, _, _>(
            BindingDefinition::new::<CrosswalkApplyInput, CrosswalkApplyOutput>(
                format!("native:{APPLICATION}.crosswalk_apply"),
                format!("{APPLICATION}.crosswalk_apply"),
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
    }
}
/// Validate and format ACRE content while retaining comments and trailing commas.
pub fn format_acre(path: &Path, original: &str) -> ApiResult<String> {
    ApplicationConfiguration::read(path.to_path_buf())?
        .validate()
        .map_err(|why| eyre!("Validate ACRE configuration — {why}"))?;
    match MimeType::from_path(path) {
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
    }
}
async fn format_metadata<T>(path: &Path, input: &FormatPreviewInput) -> ApiResult<(String, String, Vec<String>, Vec<String>, Vec<String>)>
where
    T: Clone + DeserializeOwned + EnrichmentMetadata + InputOutput + Send + Serialize + Sync + Validate + 'static,
{
    let normalized = normalize_metadata_file::<T>(path, input.fuzzy)?;
    if InputLimits::default().max_total.can_contain(normalized.original.len() as u64) != Some(true) {
        return Err(eyre!("Format input exceeds the byte limit"));
    }
    let warnings = normalized.reports.iter().map(|report| format!("{report:?}")).collect();
    let (content, conflicts, failures) = if input.enrich {
        let data = T::read(path.to_path_buf())?.format_with_vocabulary(input.fuzzy, Some(path.to_path_buf()));
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
            return Err(eyre!("Only ORCID enrichment is supported for this standard"));
        }
        let result = Pipeline::new(providers)
            .enrich((Location::from(&path.display().to_string()), data))
            .await?;
        let content = result
            .data
            .format_with_vocabulary(input.fuzzy, Some(path.to_path_buf()))
            .serialize_as(&MimeType::from_path(path))?;
        (content, result.conflicts, result.failures)
    } else {
        (normalized.content, Vec::new(), Vec::new())
    };
    Ok((normalized.before, content, warnings, conflicts, failures))
}
/// Parse and normalize one local metadata file without writing it.
pub fn normalize_metadata_file<T>(path: &Path, fuzzy: VocabularyFuzzyMode) -> ApiResult<FormatNormalization>
where
    T: InputOutput + Serialize,
{
    let original = fs::read_to_string(path).map_err(|why| eyre!("Read original metadata — {why}"))?;
    let data = T::read(path.to_path_buf())?;
    let mime = MimeType::from_path(path);
    let before = data.serialize_as(&mime)?;
    let formatted = data.format_with_vocabulary_report(fuzzy, Some(path.to_path_buf()));
    let content = formatted.value.serialize_as(&mime)?;
    Ok(FormatNormalization {
        original,
        before,
        content,
        reports: formatted.reports,
    })
}
fn selected_directory_format_file(path: &Path, standard: FormatStandard) -> bool {
    if standard == FormatStandard::ApplicationConfigurationResearchEnablement {
        return path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| matches!(name, ".acorn.json" | ".acorn.jsonc" | ".acorn.yaml" | ".acorn.yml"));
    }
    matches!(
        MimeType::from_path(path),
        MimeType::Json | MimeType::Jsonc | MimeType::Markdown | MimeType::Pdf | MimeType::Yaml | MimeType::Zon
    )
}
