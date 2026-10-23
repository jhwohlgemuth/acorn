//! Full local metadata analysis through the existing native service.
use super::super::{
    BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, InputFailure, InputLimits, InputReference, RoutedInputPolicy,
    ToolRegistry,
};
use crate::analyzer::host::context::{AnalysisContext, AnalysisOptions, ExecutionExtension, ExecutionOptions, ReportExtension, ReportOptions};
use crate::analyzer::host::service::{analyze_paths, AnalysisReport, ExecutionStatus};
use crate::error::ApiResult;
use acorn_analysis::CheckCategory;
use acorn_core::options::CommonRuntime;
use acorn_core::prelude::{String, Vec};
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::{MimeType, SemanticVersion};
use acorn_schema::agent::tools::ToolEffects;
use acorn_schema::standard::Standard;
use acorn_schema::validation::Validate;
use alloc::collections::BTreeSet;
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use strum::IntoEnumIterator;

const MAX_SUMMARY_BYTES: usize = 8192;
const MAX_SUMMARY_FINDINGS: usize = 20;
/// Execution state of one requested analysis category.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum AnalysisCategoryStatus {
    /// The analyzer ran this category.
    Executed,
    /// Runtime policy skipped this category.
    Skipped,
    /// The standard does not support this category.
    Unsupported,
}
/// One standard-specific group of analyzed paths.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct AnalysisBatchOutput {
    /// Detected or explicitly selected metadata standard.
    pub standard: Standard,
    /// Canonical analyzed paths.
    pub paths: Vec<PathBuf>,
    /// Every requested category and its execution state.
    pub categories: Vec<AnalysisCategoryOutput>,
}
/// Result and availability state of one requested category.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct AnalysisCategoryOutput {
    /// Category name.
    pub category: String,
    /// `executed`, `skipped`, or `unsupported`.
    pub status: AnalysisCategoryStatus,
    /// Reason for a skipped or unsupported category.
    pub reason: Option<String>,
    /// Ordered native findings.
    pub findings: Vec<AnalysisFinding>,
}
/// One structured finding from the native analyzer.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct AnalysisFinding {
    /// Check category.
    pub category: String,
    /// Whether this finding passed.
    pub success: bool,
    /// Native severity.
    pub severity: String,
    /// Human-readable finding.
    pub message: String,
    /// Source URI, when available.
    pub uri: Option<String>,
    /// File or field locator, when available.
    pub locator: Option<String>,
    /// Stable rule ID, when available.
    pub rule_id: Option<String>,
    /// HTTP status, when available.
    pub status_code: Option<String>,
}
/// Confirmed local input and check-category selection.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AnalysisInput {
    /// Caller-confirmed text, file, or directory.
    pub input: InputReference,
    /// Explicit metadata standard, or automatic local detection when omitted.
    pub standard: Option<Standard>,
    /// Known categories to omit from this analysis.
    #[serde(default)]
    pub skip: Vec<String>,
    /// Disable website checks even when online.
    #[serde(default)]
    pub disable_website_checks: bool,
}
/// Bounded structured output from the full check service.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct AnalysisOutput {
    /// Standard-specific batches in stable input order.
    pub batches: Vec<AnalysisBatchOutput>,
    /// Categories omitted by caller selection.
    pub skipped_categories: Vec<String>,
    /// Nonfatal directory entry failures.
    pub input_failures: Vec<InputFailure>,
}
impl AnalysisInput {
    /// Analyze this confirmed local reference with the same service used by `acorn check`.
    pub async fn analyze(self, root: &Path, offline: bool) -> ApiResult<AnalysisOutput> {
        let skip = self.skip.iter().cloned().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        let invalid_skip = skip
            .iter()
            .any(|name| !CheckCategory::iter().any(|category| category.to_string() == *name));
        let resolved = if self.standard == Some(Standard::Oci) {
            Err(eyre!("OCI artifact checks require the explicit OCI check path"))
        } else if invalid_skip {
            Err(eyre!("Unknown check category in skip selection"))
        } else {
            RoutedInputPolicy::new(root, InputLimits::default()).and_then(|policy| policy.resolve(&self.input))
        };
        let prepared = resolved.and_then(|resolved| {
            let temporary = match resolved.text.as_ref() {
                | Some(text) => {
                    let format = if self.standard == Some(Standard::Text) {
                        MimeType::Text
                    } else {
                        MimeType::Json
                    };
                    let suffix = format!(".{}", format.file_type());
                    tempfile::Builder::new()
                        .suffix(&suffix)
                        .tempfile()
                        .and_then(|mut file| file.write_all(text.as_bytes()).map(|()| file))
                        .map(Some)
                        .map_err(Into::into)
                }
                | None => Ok(None),
            };
            temporary.and_then(|temporary| {
                let paths = match temporary.as_ref() {
                    | Some(file) => vec![file.path().to_path_buf()],
                    | None => resolved.files.iter().map(|file| file.path.clone()).collect(),
                };
                if paths.is_empty() {
                    Err(eyre!("No supported local input files were found"))
                } else {
                    Ok((paths, temporary, resolved.failures))
                }
            })
        });
        match prepared {
            | Err(why) => Err(why),
            | Ok((paths, temporary, failures)) => {
                let analysis = AnalysisOptions::init().skip(skip).standard(self.standard.unwrap_or_default()).build();
                let execution = ExecutionOptions::init()
                    .common(CommonRuntime {
                        offline,
                        quiet: true,
                        ..Default::default()
                    })
                    .extension(
                        ExecutionExtension::init()
                            .disable_website_checks(offline || self.disable_website_checks)
                            .skip_sync(offline)
                            .build(),
                    )
                    .build();
                let report = ReportOptions::init()
                    .common(CommonRuntime {
                        quiet: true,
                        ..Default::default()
                    })
                    .extension(ReportExtension::init().all(true).build())
                    .build();
                let options = AnalysisContext::init().analysis(analysis).execution(execution).report(report).build();
                let result = AnalysisOutput::from_report(analyze_paths(&paths, &options).await, failures);
                drop(temporary);
                Ok(result)
            }
        }
    }
}
impl AnalysisOutput {
    fn from_report(report: AnalysisReport, input_failures: Vec<InputFailure>) -> Self {
        let batches = report
            .batches
            .into_iter()
            .map(|batch| AnalysisBatchOutput {
                standard: batch.standard,
                paths: batch.paths,
                categories: batch
                    .categories
                    .into_iter()
                    .map(|category| {
                        let (status, reason) = match category.status {
                            | ExecutionStatus::Executed => (AnalysisCategoryStatus::Executed, None),
                            | ExecutionStatus::Skipped { reason } => (AnalysisCategoryStatus::Skipped, Some(reason)),
                            | ExecutionStatus::Unsupported { reason } => (AnalysisCategoryStatus::Unsupported, Some(reason)),
                        };
                        AnalysisCategoryOutput {
                            category: category.category.to_string(),
                            status,
                            reason,
                            findings: category
                                .checks
                                .into_iter()
                                .map(|check| AnalysisFinding {
                                    category: check.category.to_string(),
                                    success: check.success,
                                    severity: check.severity.to_string(),
                                    message: check.message,
                                    uri: check.uri,
                                    locator: check.locator,
                                    rule_id: check.rule_id,
                                    status_code: check.status_code,
                                })
                                .collect(),
                        }
                    })
                    .collect(),
            })
            .collect();
        Self {
            batches,
            skipped_categories: report.skipped_categories.into_iter().map(|category| category.to_string()).collect(),
            input_failures,
        }
    }
    /// Render a bounded summary of failed checks for metadata conversations.
    pub fn summary(&self) -> String {
        let finding_lines = self
            .batches
            .iter()
            .enumerate()
            .flat_map(|(batch_index, batch)| {
                batch.categories.iter().enumerate().flat_map(move |(category_index, category)| {
                    category
                        .findings
                        .iter()
                        .filter(|finding| !finding.success)
                        .map(move |finding| (batch_index, category_index, category, finding))
                })
            })
            .take(MAX_SUMMARY_FINDINGS)
            .map(|(batch_index, category_index, category, finding)| {
                let message = finding.message.chars().take(300).collect::<String>();
                let source = finding.uri.as_deref().unwrap_or("unknown source");
                let locator = finding.locator.as_deref().map_or(String::new(), |value| format!("#{value}"));
                let rule = finding.rule_id.as_deref().map_or(String::new(), |value| format!(" ({value})"));
                let line = format!("{} [{}] {}{}{}: {}", category.category, finding.severity, source, locator, rule, message);
                (batch_index, category_index, line)
            })
            .collect::<Vec<_>>();
        let selected_findings = &finding_lines;
        let joined = self
            .batches
            .iter()
            .enumerate()
            .flat_map(|(batch_index, batch)| {
                let heading = core::iter::once(format!("Standard: {}. Files: {}.", batch.standard, batch.paths.len()));
                let categories = batch.categories.iter().enumerate().flat_map(move |(category_index, category)| {
                    let status = (category.status != AnalysisCategoryStatus::Executed).then(|| {
                        format!(
                            "{}: {} ({})",
                            category.category,
                            category.status,
                            category.reason.as_deref().unwrap_or("no reason")
                        )
                    });
                    let findings = selected_findings
                        .iter()
                        .filter(move |(selected_batch, selected_category, _)| *selected_batch == batch_index && *selected_category == category_index)
                        .map(|(_, _, line)| line.clone());
                    status.into_iter().chain(findings)
                });
                heading.chain(categories)
            })
            .chain((!self.input_failures.is_empty()).then(|| format!("{} input entries were skipped.", self.input_failures.len())))
            .collect::<Vec<_>>()
            .join("\n");
        let end = joined
            .char_indices()
            .take_while(|(index, character)| index.saturating_add(character.len_utf8()) <= MAX_SUMMARY_BYTES)
            .last()
            .map_or(0, |(index, character)| index.saturating_add(character.len_utf8()));
        joined[..end].to_string()
    }
}
impl ToolRegistry {
    /// Register the full local analysis service as a versioned native capability.
    pub fn register_analysis_capability(self, root: PathBuf, offline: bool) -> ApiResult<Self> {
        let effects = ToolEffects {
            read_only: true,
            destructive: false,
            idempotent: true,
            open_world: true,
        };
        let binding = BindingDefinition::new::<AnalysisInput, AnalysisOutput>(
            format!("native:{APPLICATION}.check"),
            format!("{APPLICATION}.check"),
            SemanticVersion::from((1, 0, 0)),
            BindingKind::Native,
            effects,
            BindingAvailability::Ready,
        );
        let binding = if offline { binding.with_offline_support() } else { binding };
        self.register_capability(CapabilityDefinition::new::<AnalysisInput, AnalysisOutput>(
            format!("{APPLICATION}.check"),
            SemanticVersion::from((1, 0, 0)),
            "Analyze confirmed local metadata files across supported check categories and report structured findings.",
            effects,
        ))
        .and_then(|registry| {
            registry.register_capability_binding::<AnalysisInput, AnalysisOutput, _, _>(binding, move |input| {
                let root = root.clone();
                async move { input.analyze(&root, offline).await }
            })
        })
    }
}
