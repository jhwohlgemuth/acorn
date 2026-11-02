//! Full local metadata analysis through the existing native service.
use super::super::{
    BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, InputFailure, InputLimits, InputReference, RoutedInputPolicy,
    ToolRegistry,
};
use crate::agent::InferenceBackend;
use crate::analyzer::host::context::{AnalysisContext, AnalysisOptions, ExecutionExtension, ExecutionOptions, ReportExtension, ReportOptions};
use crate::analyzer::host::oci::analyze as analyze_oci;
use crate::analyzer::host::service::{analyze_paths, AnalysisReport, ExecutionStatus};
use crate::analyzer::{
    diff::{DiffFormat, DiffInput, SemanticDiff},
    fix::{FixOptions, FixReport},
};
use crate::error::ApiResult;
use crate::io::config::application::ApplicationConfiguration;
use crate::io::{files_from_gitlab_merge_request, Source};
use crate::LocationExt;
use acorn_analysis::CheckCategory;
use acorn_cmd::args;
use acorn_core::options::CommonRuntime;
use acorn_core::prelude::{String, Vec};
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::{MimeType, SemanticVersion};
use acorn_core::Location;
use acorn_schema::agent::tools::ToolEffects;
use acorn_schema::standard::Standard;
use acorn_schema::validation::Validate;
use alloc::{collections::BTreeSet, sync::Arc};
use color_eyre::eyre::eyre;
use fancy_regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
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
/// One reviewable proposed edit, without applying it.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisFixDifference {
    /// IDs of fixes projected into this difference.
    pub fix_ids: Vec<String>,
    /// Target owned by the finding.
    pub target: String,
    /// Unified difference against the original content.
    pub unified: String,
}
/// One check request that also asks a configured backend for nonexecuting proposals.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisFixInput {
    /// Confirmed analysis request.
    pub analysis: AnalysisInput,
}
/// Structured findings, proposals, and nonexecuting previews.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisFixOutput {
    /// Check findings and category state.
    pub analysis: AnalysisOutput,
    /// Validated suggestions from the configured inference backend.
    pub fixes: FixReport,
    /// Projected changes that the caller can review.
    pub differences: Vec<AnalysisFixDifference>,
    /// Projection failures without mutating a source.
    pub preview_diagnostics: Vec<String>,
}
/// Confirmed local input and check-category selection.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AnalysisInput {
    /// Caller-confirmed text, file, or directory.
    pub input: InputReference,
    /// Explicit metadata standard, or automatic local detection when omitted.
    pub standard: Option<Standard>,
    /// Explicit format for supplied text; other sources detect their format.
    pub format: Option<MimeType>,
    /// Known categories to omit from this analysis.
    #[serde(default)]
    pub skip: Vec<String>,
    /// Include only paths matching every supplied regular expression.
    #[serde(default)]
    pub filter: Vec<String>,
    /// Omit paths matching any supplied regular expression.
    #[serde(default)]
    pub ignore: Vec<String>,
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
impl AnalysisFixInput {
    /// Analyze, suggest, and project changes without applying them.
    pub async fn analyze<B>(self, root: &Path, offline: bool, backend: &B, options: &FixOptions) -> ApiResult<AnalysisFixOutput>
    where
        B: InferenceBackend + Sync,
    {
        match self.analysis.input.is_oci() {
            | true => Err(eyre!("OCI artifact checks do not support fix previews")),
            | false => match self.analysis.analyze_report(root, offline).await {
                | Err(why) => Err(why),
                | Ok((report, failures, temporary)) => match report.suggest_fixes(backend, options).await {
                    | Err(why) => Err(why),
                    | Ok(fixes) => Ok(Self::project(report, failures, temporary.is_some(), fixes)),
                },
            },
        }
    }
    fn project(report: AnalysisReport, failures: Vec<InputFailure>, ephemeral: bool, fixes: FixReport) -> AnalysisFixOutput {
        let preview = fixes.build_preview(&report, ephemeral);
        let mut diagnostics = preview.diagnostics.into_iter().map(|item| item.message).collect::<Vec<_>>();
        let differences = preview
            .targets
            .into_iter()
            .filter_map(|target| {
                let before_label = format!("a/{}", target.target);
                let after_label = format!("b/{}", target.target);
                let before = DiffInput {
                    content: &target.original,
                    label: &before_label,
                    media_type: target.source_format.clone(),
                };
                let after = DiffInput {
                    content: &target.proposed,
                    label: &after_label,
                    media_type: target.source_format.clone(),
                };
                match SemanticDiff::compare(&before, &after, DiffFormat::Unified, None) {
                    | Ok(SemanticDiff::Unified(unified)) => Some(AnalysisFixDifference {
                        fix_ids: target.fix_ids,
                        target: target.target,
                        unified,
                    }),
                    | Ok(SemanticDiff::JsonPatch(_)) => None,
                    | Err(why) => {
                        diagnostics.push(format!("{}: {why}", target.target));
                        None
                    }
                }
            })
            .collect();
        AnalysisFixOutput {
            analysis: AnalysisOutput::from_report(report, failures),
            fixes,
            differences,
            preview_diagnostics: diagnostics,
        }
    }
}
impl AnalysisInput {
    /// Analyze a confirmed source with the same native services used by `acorn check`.
    pub async fn analyze(self, root: &Path, offline: bool) -> ApiResult<AnalysisOutput> {
        self.analyze_report(root, offline)
            .await
            .map(|(report, failures, _)| AnalysisOutput::from_report(report, failures))
    }
    pub(crate) async fn analyze_report(
        self,
        root: &Path,
        offline: bool,
    ) -> ApiResult<(AnalysisReport, Vec<InputFailure>, Option<tempfile::NamedTempFile>)> {
        let skip = self.skip.iter().cloned().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        let filters = self
            .filter
            .iter()
            .map(|pattern| Regex::new(pattern).map_err(|why| eyre!("Invalid check filter — {why}")))
            .collect::<ApiResult<Vec<_>>>();
        let ignores = self
            .ignore
            .iter()
            .map(|pattern| Regex::new(pattern).map_err(|why| eyre!("Invalid check ignore — {why}")))
            .collect::<ApiResult<Vec<_>>>();
        let selection = filters
            .and_then(|filters| ignores.map(|ignores| (filters, ignores)))
            .and_then(|selection| self.validate_selection(&skip).map(|()| selection));
        match selection {
            | Err(why) => Err(why),
            | Ok((filters, ignores)) => {
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
                match self.input {
                    | InputReference::Oci(reference) => {
                        if offline || !filters.is_empty() || !ignores.is_empty() {
                            Err(eyre!("OCI artifact checks require online access and do not support path filters"))
                        } else {
                            ApplicationConfiguration::load(&None)
                                .and_then(|configuration| analyze_oci(&reference, &options, &configuration))
                                .map(|report| (report, Vec::new(), None))
                        }
                    }
                    | input => match input.prepare(root, offline, self.format).await {
                        | Err(why) => Err(why),
                        | Ok((paths, temporary, failures)) => {
                            let selected = paths
                                .into_iter()
                                .map(|path| selected_path(&path, &filters, &ignores).map(|included| included.then_some(path)))
                                .collect::<ApiResult<Vec<_>>>()
                                .map(|paths| paths.into_iter().flatten().collect::<Vec<_>>());
                            match selected {
                                | Err(why) => Err(why),
                                | Ok(paths) if paths.is_empty() => Err(eyre!("No supported check input files were found")),
                                | Ok(paths) => Ok((analyze_paths(&paths, &options).await, failures, temporary)),
                            }
                        }
                    },
                }
            }
        }
    }
    fn validate_selection(&self, skip: &[String]) -> ApiResult<()> {
        let invalid_skip = skip
            .iter()
            .any(|name| !CheckCategory::iter().any(|category| category.to_string() == *name));
        match (
            invalid_skip,
            self.format.is_some() && !self.input.is_text(),
            self.input.is_oci(),
            self.standard,
        ) {
            | (true, _, _, _) => Err(eyre!("Unknown check category in skip selection")),
            | (_, true, _, _) => Err(eyre!("An explicit check format requires supplied text")),
            | (_, _, false, Some(Standard::Oci)) => Err(eyre!("OCI artifact checks require an explicit OCI reference")),
            | (_, _, true, Some(standard)) if standard != Standard::Oci => Err(eyre!("OCI check source conflicts with standard {standard}")),
            | _ => Ok(()),
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
impl InputReference {
    async fn prepare(
        self,
        root: &Path,
        offline: bool,
        format: Option<MimeType>,
    ) -> ApiResult<(Vec<PathBuf>, Option<tempfile::NamedTempFile>, Vec<InputFailure>)> {
        let limits = InputLimits::default();
        let git_branch = self.is_git_branch();
        match self {
            | InputReference::Url(url) => {
                let location = Location::from(url.as_str());
                if offline || !(location.scheme().is_http() || location.scheme().is_https()) {
                    Err(eyre!("Remote check input requires an HTTP(S) URL and online policy"))
                } else {
                    Source::read_bytes(&url, false).await.and_then(|content| {
                        if limits.max_total.can_contain(u64::try_from(content.len()).unwrap_or(u64::MAX)) != Some(true) {
                            Err(eyre!("Remote check input exceeds the byte limit"))
                        } else {
                            let suffix = format!(".{}", location.resolve_file_type(&content).file_type());
                            tempfile::Builder::new()
                                .suffix(&suffix)
                                .tempfile()
                                .map_err(|why| eyre!(why))
                                .and_then(|mut file| file.write_all(&content).map_err(|why| eyre!(why)).map(|()| file))
                                .map(|file| (vec![file.path().to_path_buf()], Some(file), Vec::new()))
                        }
                    })
                }
            }
            | InputReference::GitBranch(name) | InputReference::GitCommit(name) => git_changed_files(root, &name, git_branch)
                .and_then(|files| RoutedInputPolicy::new(root, limits.clone()).and_then(|policy| bounded_git_paths(&policy, &limits, files))),
            | InputReference::GitMergeRequest => {
                if offline {
                    Err(eyre!("GitLab merge request selection is unavailable offline"))
                } else {
                    let files = files_from_gitlab_merge_request::<MimeType>(None).await;
                    RoutedInputPolicy::new(root, limits.clone()).and_then(|policy| bounded_git_paths(&policy, &limits, files))
                }
            }
            | other => RoutedInputPolicy::new(root, limits)
                .and_then(|policy| policy.resolve(&other))
                .and_then(|resolved| {
                    let temporary = resolved.text.as_ref().map_or(Ok(None), |text| {
                        let format = format.unwrap_or_else(|| MimeType::infer_from(text));
                        let suffix = format!(".{}", format.file_type());
                        tempfile::Builder::new()
                            .suffix(&suffix)
                            .tempfile()
                            .map_err(|why| eyre!(why))
                            .and_then(|mut file| file.write_all(text.as_bytes()).map_err(|why| eyre!(why)).map(|()| Some(file)))
                    });
                    temporary.map(|temporary| {
                        let paths = match temporary.as_ref() {
                            | Some(file) => vec![file.path().to_path_buf()],
                            | None => resolved
                                .files
                                .into_iter()
                                .map(|file| file.path)
                                .filter(|path| !other.is_directory() || MimeType::is_supported_metadata_path(path))
                                .collect(),
                        };
                        (paths, temporary, resolved.failures)
                    })
                }),
        }
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
    /// Register inference-backed, nonexecuting check proposals and previews.
    pub fn register_analysis_fixes_capability<B>(self, root: PathBuf, offline: bool, backend: Arc<B>, options: FixOptions) -> ApiResult<Self>
    where
        B: InferenceBackend + Sync + 'static,
    {
        let effects = ToolEffects {
            read_only: true,
            destructive: false,
            idempotent: true,
            open_world: true,
        };
        let version = SemanticVersion::from((1, 0, 0));
        self.register_capability(CapabilityDefinition::new::<AnalysisFixInput, AnalysisFixOutput>(
            format!("{APPLICATION}.check_fixes"),
            version,
            "Analyze confirmed metadata and return bounded fix proposals with nonexecuting previews.",
            effects,
        ))
        .and_then(|registry| {
            let id = format!("native:{APPLICATION}.check_fixes");
            let capability_id = format!("{APPLICATION}.check_fixes");
            let definition = BindingDefinition::new::<AnalysisFixInput, AnalysisFixOutput>(
                id,
                capability_id,
                version,
                BindingKind::Native,
                effects,
                BindingAvailability::Ready,
            );
            registry.register_capability_binding::<AnalysisFixInput, AnalysisFixOutput, _, _>(definition.with_offline_support(), move |input| {
                let root = root.clone();
                let backend = Arc::clone(&backend);
                let options = options.clone();
                async move { input.analyze(&root, offline, backend.as_ref(), &options).await }
            })
        })
    }
}
fn bounded_git_paths(
    policy: &RoutedInputPolicy,
    limits: &InputLimits,
    files: Vec<PathBuf>,
) -> ApiResult<(Vec<PathBuf>, Option<tempfile::NamedTempFile>, Vec<InputFailure>)> {
    let resolved = files
        .into_iter()
        .filter(|path| MimeType::is_supported_metadata_path(path))
        .map(|path| policy.resolve(&InputReference::File(path)))
        .collect::<ApiResult<Vec<_>>>();
    resolved.and_then(|resolved| {
        resolved
            .iter()
            .flat_map(|entry| &entry.files)
            .try_fold(0_u64, |total, file| file.size.checked_bytes().and_then(|size| total.checked_add(size)))
            .ok_or_else(|| eyre!("Git check selection exceeds the input limits"))
            .and_then(|bytes| {
                if resolved.len() > limits.max_files || limits.max_total.can_contain(bytes) != Some(true) {
                    Err(eyre!("Git check selection exceeds the input limits"))
                } else {
                    Ok((
                        resolved
                            .into_iter()
                            .flat_map(|entry| entry.files.into_iter().map(|file| file.path))
                            .collect(),
                        None,
                        Vec::new(),
                    ))
                }
            })
    })
}
fn git_changed_files(root: &Path, reference: &str, branch: bool) -> ApiResult<Vec<PathBuf>> {
    git_output(root, args!["rev-parse", "--verify", "--end-of-options", reference]).and_then(|selected| {
        let selected = selected.trim();
        let changed = if branch {
            let default = git_output(root, args!["symbolic-ref", "refs/remotes/origin/HEAD", "--short"])
                .ok()
                .and_then(|value| value.trim().split('/').next_back().map(str::to_string))
                .unwrap_or_else(|| "main".to_string());
            git_output(root, args!["rev-parse", "--verify", "--end-of-options", &default])
                .and_then(|base| git_output(root, args!["diff", "--name-only", base.trim(), "--merge-base", selected]))
        } else {
            git_output(root, args!["diff-tree", "--no-commit-id", "--name-only", "-r", selected])
        };
        changed.map(|changed| changed.lines().map(PathBuf::from).collect())
    })
}
fn git_output(root: &Path, arguments: Vec<OsString>) -> ApiResult<String> {
    Command::new("git")
        .args(args!["-C", root.as_os_str(), ..arguments])
        .output()
        .map_err(|why| eyre!("Git check selection failed — {why}"))
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).map_err(|why| eyre!("Git check output is not UTF-8 — {why}"))
            } else {
                Err(eyre!("Git check selection failed: {}", String::from_utf8_lossy(&output.stderr).trim()))
            }
        })
}
fn selected_path(path: &Path, filters: &[Regex], ignores: &[Regex]) -> ApiResult<bool> {
    let value = path.to_string_lossy();
    filters
        .iter()
        .try_fold(true, |selected, filter| {
            filter.is_match(&value).map(|matches| selected && matches).map_err(|why| eyre!(why))
        })
        .and_then(|included| {
            ignores
                .iter()
                .try_fold(false, |selected, ignore| {
                    ignore.is_match(&value).map(|matches| selected || matches).map_err(|why| eyre!(why))
                })
                .map(|ignored| included && !ignored)
        })
}
