//! Bounded MCP routing wrappers around the existing semantic diff and portable similarity operations.
use super::super::{
    BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, InputLimits, InputReference, RoutedInputPolicy, ToolRegistry,
};
use super::NativeEffect;
use crate::analyzer::diff::{DiffFormat, DiffInput, SemanticDiff};
use crate::error::ApiResult;
use acorn_core::prelude::String;
use acorn_core::similarity::{compare, ComparisonOptions, SimilarityLimits, SimilarityMethod, SimilarityPreprocessing, SimilarityTokenization};
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::{MimeType, SemanticVersion};
use acorn_core::validation::ValidationError;
use acorn_core::Noul;
use acorn_schema::agent::tools::ToolEffects;
use acorn_schema::standard::Standard;
use acorn_schema::validation::{IntoValidationReport, Validate, ValidationReport};
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// One bounded literal or local file used in a comparison.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "value")]
pub enum DiffOperand {
    /// File below the caller's permitted root.
    File(PathBuf),
    /// Verbatim value.
    Text(String),
}
/// The requested comparison and its bounded options.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind", deny_unknown_fields)]
pub enum DiffOperation {
    /// Same-standard semantic comparison.
    Semantic {
        /// Produce RFC 6902 operations instead of unified text.
        json_patch: bool,
        /// Override automatic standard detection.
        standard: Option<Standard>,
    },
    /// Pairwise similarity with explicit resource limits.
    Similarity {
        /// Method name accepted by the direct diff command.
        method: String,
        /// Optional positive method cutoff.
        cutoff: Option<f64>,
        /// Optional prefix or suffix length.
        length: Option<usize>,
        /// Maximum collection items.
        max_collection_items: Option<usize>,
        /// Maximum input scalars.
        max_input_scalars: Option<usize>,
        /// Maximum work units.
        max_work_units: Option<usize>,
        /// Optional character n-gram width.
        ngram_width: Option<usize>,
        /// Text preprocessing name.
        preprocessing: Option<String>,
        /// Optional token match threshold.
        threshold: Option<Noul>,
    },
}
/// One caller-confirmed pairwise comparison.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct DiffOperationInput {
    /// Original file or literal value.
    pub before: DiffOperand,
    /// Revised file or literal value.
    pub after: DiffOperand,
    /// Comparison kind and its options.
    pub operation: DiffOperation,
}
/// Structured result of one comparison.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiffOperationOutput {
    /// `unified`, `json-patch`, or `similarity`.
    pub format: String,
    /// Unified text or the structured patch/comparison.
    pub result: Value,
}
#[derive(Clone, Copy)]
struct SimilarityRequestOptions {
    cutoff: Option<f64>,
    length: Option<usize>,
    max_collection_items: Option<usize>,
    max_input_scalars: Option<usize>,
    max_work_units: Option<usize>,
    method: SimilarityMethod,
    ngram_width: Option<usize>,
    preprocessing: SimilarityPreprocessing,
    threshold: Option<Noul>,
}
impl DiffOperationInput {
    /// Resolve bounded inputs and perform exactly one comparison.
    pub fn compare(self, root: &Path) -> ApiResult<DiffOperationOutput> {
        RoutedInputPolicy::new(root, InputLimits::default()).and_then(|policy| {
            read_operand(&policy, &self.before)
                .and_then(|before| read_operand(&policy, &self.after).map(|after| (before, after)))
                .and_then(|((before, before_path), (after, after_path))| self.compare_resolved(before, before_path, after, after_path))
        })
    }
    fn compare_resolved(
        self,
        before: String,
        before_path: Option<PathBuf>,
        after: String,
        after_path: Option<PathBuf>,
    ) -> ApiResult<DiffOperationOutput> {
        match self.operation {
            | DiffOperation::Semantic { json_patch, standard } => {
                let format = if json_patch { DiffFormat::JsonPatch } else { DiffFormat::Unified };
                let before_label = match &self.before {
                    | DiffOperand::Text(_) => "a/input".to_string(),
                    | DiffOperand::File(path) => format!("a/{}", path.display()),
                };
                let after_label = match &self.after {
                    | DiffOperand::Text(_) => "b/input".to_string(),
                    | DiffOperand::File(path) => format!("b/{}", path.display()),
                };
                let before_media = before_path
                    .as_ref()
                    .map_or_else(|| MimeType::infer_from(&before), |path| MimeType::from_path_or_content(path, &before));
                let after_media = after_path
                    .as_ref()
                    .map_or_else(|| MimeType::infer_from(&after), |path| MimeType::from_path_or_content(path, &after));
                SemanticDiff::compare(
                    &DiffInput {
                        content: &before,
                        label: &before_label,
                        media_type: before_media,
                    },
                    &DiffInput {
                        content: &after,
                        label: &after_label,
                        media_type: after_media,
                    },
                    format,
                    standard,
                )
                .map_err(|why| eyre!(why))
                .and_then(|result| match result {
                    | SemanticDiff::Unified(text) => Ok(DiffOperationOutput {
                        format: "unified".to_string(),
                        result: json!(text),
                    }),
                    | SemanticDiff::JsonPatch(patch) => {
                        serde_json::to_value(patch)
                            .map_err(|why| eyre!("Serialize JSON Patch — {why}"))
                            .map(|result| DiffOperationOutput {
                                format: "json-patch".to_string(),
                                result,
                            })
                    }
                })
            }
            | DiffOperation::Similarity {
                method,
                cutoff,
                length,
                max_collection_items,
                max_input_scalars,
                max_work_units,
                ngram_width,
                preprocessing,
                threshold,
            } => SimilarityMethod::try_from(method.as_str())
                .map_err(|why| eyre!(why))
                .and_then(|method| {
                    SimilarityPreprocessing::try_from(preprocessing.as_deref().unwrap_or("raw"))
                        .map_err(|why| eyre!(why))
                        .map(|preprocessing| (method, preprocessing))
                })
                .and_then(|(method, preprocessing)| {
                    SimilarityRequestOptions {
                        cutoff,
                        length,
                        max_collection_items,
                        max_input_scalars,
                        max_work_units,
                        method,
                        ngram_width,
                        preprocessing,
                        threshold,
                    }
                    .validate()
                    .map_err(|why| eyre!(why))
                    .map(|()| (method, preprocessing))
                })
                .and_then(|(method, preprocessing)| {
                    let defaults = SimilarityLimits::default();
                    SimilarityLimits::new(
                        max_input_scalars.unwrap_or(defaults.max_input_scalars()),
                        max_work_units.unwrap_or(defaults.max_work_units()),
                        max_collection_items.unwrap_or(defaults.max_collection_items()),
                    )
                    .map_err(|why| eyre!(why))
                    .map(|limits| {
                        ComparisonOptions::init()
                            .method(method)
                            .maybe_cutoff(cutoff)
                            .maybe_length(length)
                            .limits(limits)
                            .preprocessing(preprocessing)
                            .maybe_threshold(threshold)
                            .tokenization(
                                ngram_width.map_or(SimilarityTokenization::Words, |width| SimilarityTokenization::CharacterNgrams { width }),
                            )
                            .build()
                    })
                })
                .and_then(|options| compare(&before, &after, options).map_err(|why| eyre!(why)))
                .and_then(|comparison| serde_json::to_value(comparison).map_err(|why| eyre!("Serialize similarity result — {why}")))
                .map(|result| DiffOperationOutput {
                    format: "similarity".to_string(),
                    result,
                }),
        }
    }
}
impl Validate for SimilarityRequestOptions {
    fn validate(&self) -> Result<(), ValidationReport> {
        let Self {
            cutoff,
            length,
            max_collection_items,
            max_input_scalars,
            max_work_units,
            method,
            ngram_width,
            preprocessing,
            threshold,
        } = *self;
        let incompatible = match method {
            | SimilarityMethod::AbsoluteDifference | SimilarityMethod::PercentageDifference => [
                ("length", length.is_some()),
                ("max-collection-items", max_collection_items.is_some()),
                ("max-input-scalars", max_input_scalars.is_some()),
                ("max-work-units", max_work_units.is_some()),
                ("ngram-width", ngram_width.is_some()),
                ("preprocess", preprocessing != SimilarityPreprocessing::Raw),
                ("threshold", threshold.is_some()),
            ]
            .into_iter()
            .find_map(|(option, present)| present.then_some(option)),
            | SimilarityMethod::Jaccard | SimilarityMethod::Overlap | SimilarityMethod::SorensenDice => [
                ("cutoff", cutoff.is_some()),
                ("length", length.is_some()),
                ("threshold", threshold.is_some()),
            ]
            .into_iter()
            .find_map(|(option, present)| present.then_some(option)),
            | SimilarityMethod::ExtendedJaccard | SimilarityMethod::SoftTfidf => [("cutoff", cutoff.is_some()), ("length", length.is_some())]
                .into_iter()
                .find_map(|(option, present)| present.then_some(option)),
            | SimilarityMethod::Prefix | SimilarityMethod::Suffix => [
                ("cutoff", cutoff.is_some()),
                ("ngram-width", ngram_width.is_some()),
                ("threshold", threshold.is_some()),
            ]
            .into_iter()
            .find_map(|(option, present)| present.then_some(option)),
            | _ => [
                ("cutoff", cutoff.is_some()),
                ("length", length.is_some()),
                ("ngram-width", ngram_width.is_some()),
                ("threshold", threshold.is_some()),
            ]
            .into_iter()
            .find_map(|(option, present)| present.then_some(option)),
        };
        incompatible.map_or(Ok(()), |option| {
            Err(ValidationError::new("incompatible-option")
                .with_message(format!("Similarity method {method:?} does not support --{option}"))
                .into_report(option))
        })
    }
}
impl ToolRegistry {
    /// Register bounded semantic diff and similarity as native capabilities.
    pub fn register_diff_capability(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let effects = ToolEffects::from(NativeEffect::LocalRead);
        let version = SemanticVersion::from((1, 0, 0));
        self.register_capability(CapabilityDefinition::new::<DiffOperationInput, DiffOperationOutput>(
            format!("{APPLICATION}.diff"),
            version,
            "Compare two confirmed metadata files semantically or score two bounded values.",
            effects,
        ))
        .and_then(|registry| {
            registry.register_capability_binding::<DiffOperationInput, DiffOperationOutput, _, _>(
                BindingDefinition::new::<DiffOperationInput, DiffOperationOutput>(
                    format!("native:{APPLICATION}.diff"),
                    format!("{APPLICATION}.diff"),
                    version,
                    BindingKind::Native,
                    effects,
                    BindingAvailability::Ready,
                )
                .with_offline_support(),
                move |input| {
                    let root = root.clone();
                    async move { input.compare(&root) }
                },
            )
        })
    }
}
fn read_operand(policy: &RoutedInputPolicy, input: &DiffOperand) -> ApiResult<(String, Option<PathBuf>)> {
    let reference = match input {
        | DiffOperand::Text(text) => InputReference::Text(text.clone()),
        | DiffOperand::File(path) => InputReference::File(path.clone()),
    };
    policy
        .resolve(&reference)
        .and_then(|resolved| match (resolved.text, resolved.files.as_slice()) {
            | (Some(text), []) => Ok((text, None)),
            | (None, [file]) => fs::read_to_string(&file.path)
                .map_err(|why| eyre!("Read UTF-8 diff input {} — {why}", file.path.display()))
                .map(|text| (text, Some(file.path.clone()))),
            | _ => Err(eyre!("Diff requires exactly one file or literal value per operand")),
        })
}
