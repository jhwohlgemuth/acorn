//! Structured native file-analysis service
use super::render::{CheckRender, DocumentIndexes, RenderContext};
use crate::{
    analyzer::{
        check::{Check, CheckBatch, CheckCategory, CheckSeverity},
        fair::{FairDetails, FairEvidence, FairProfile, FairState},
        host::{
            context::{AnalysisContext, ReportOptions},
            runtime::{AcreStandard, CheckBatchExt, KitfileStandard, LinkEvidence, ModelKitStandard, StaticAnalyzer},
            vale::Vale,
        },
        readability::score::{ReadabilityScore, ReadabilitySeverity},
    },
    io::{
        document::{DocumentIndex, MarkdownParser, SourceDocument},
        read_file, FromPath, InputOutput,
    },
    util::StringConversion,
};
use acorn_analysis::{check, check_err, check_ok, StandardCheckCapabilities};
use acorn_core::{
    prelude::{Arc, HashMap, Path, PathBuf},
    util::{MimeType, ToProse},
};
use acorn_host::terminal::Label;
use acorn_schema::{
    pid::{raid, DOI},
    research_activity::ResearchActivity,
    standard::{
        cff::Cff,
        ckan, datacite, dcat,
        dublin_core::DublinCoreDocument,
        huwise, invenio,
        text::{Document, Text},
        Standard,
    },
};
use async_trait::async_trait;
use core::{future::Future, iter::once};
use futures::future::{join_all, BoxFuture};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use strum::{EnumIs, IntoEnumIterator};
use tracing::{debug, error};

/// Native analysis operations over filesystem paths.
#[async_trait]
pub trait Analysis: SchemaCheck {
    /// Run analysis for a given category.
    async fn check(category: CheckCategory, paths: &[PathBuf], options: Option<&AnalysisContext>) -> CategoryChecks {
        let status = Self::standard().execution_status(&category, options);
        let checks = match status {
            | ExecutionStatus::Skipped { .. } | ExecutionStatus::Unsupported { .. } => CheckBatch::empty(),
            | ExecutionStatus::Executed => match category {
                | CheckCategory::Compute => Self::check_compute(paths, options).await,
                | CheckCategory::Link => Self::check_links(paths, options).await,
                | CheckCategory::Fair => Self::check_fair(paths, options).await,
                | CheckCategory::Prose => Self::check_prose(paths, options).await,
                | CheckCategory::Quality => Self::check_quality(paths, options).await,
                | CheckCategory::Readability => Self::check_readability(paths, options).await,
                | CheckCategory::Schema => Self::check_schema(paths, options).await,
                | CheckCategory::Crosswalk => CheckBatch::empty(),
            },
        };
        CategoryChecks { category, checks, status }
    }
    /// Assess declared compute requirements against caller-supplied observations.
    async fn check_compute(_paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        CheckBatch::empty()
    }
    /// Assess evidence of support for the FAIR principles.
    async fn check_fair(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        paths
            .iter()
            .map(|path| {
                let standard = Self::standard();
                check!(
                    CheckCategory::Fair,
                    false,
                    severity: CheckSeverity::Suggestion,
                    uri: path.file_name_with_parent(),
                    message: format!("FAIR assessment is not supported for {standard} metadata")
                )
            })
            .collect()
    }
    /// Perform analysis of prose.
    async fn check_prose(_paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        CheckBatch::empty()
    }
    /// Perform quality checks and verify data consistency.
    async fn check_quality(_paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        CheckBatch::empty()
    }
    /// Calculate readability using a given metric.
    async fn check_readability(_paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        CheckBatch::empty()
    }
    /// Execute validation checks for the given paths.
    async fn check_schema(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        let futures = paths.par_iter().filter_map(<Self as SchemaCheck>::check_schema_path).collect::<Vec<_>>();
        join_all(futures).await.into_iter().collect()
    }
    /// Evaluate links extracted from the supplied paths.
    async fn check_links(_paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        CheckBatch::empty()
    }
    /// Return the output path used for Vale analysis artifacts.
    fn output_path(path: &Path, data: &Self) -> PathBuf;
    /// Return the metadata standard represented by the implementing type.
    fn standard() -> Standard;
    /// Flatten futures that each return checks.
    async fn flatten_checks<I, F>(futures: I) -> CheckBatch
    where
        I: IntoIterator<Item = F> + Send,
        F: Future<Output = CheckBatch> + Send,
    {
        join_all(futures).await.into_iter().collect()
    }
    /// Collect link checks and attach the source URI.
    async fn collect_checks<'a>(futures: impl IntoIterator<Item = futures::future::BoxFuture<'a, Check>> + Send, path: &Path) -> CheckBatch {
        let uri = Some(path.to_path_buf().file_name_with_parent());
        join_all(futures).await.into_iter().map(|check| check.with_uri(uri.clone())).collect()
    }
}
/// Supplies schema checks to native [`Analysis`] implementations.
pub trait SchemaCheck {
    /// Read and validate one schema document when the implementing type supports it.
    fn check_schema_path(_path: &PathBuf) -> Option<BoxFuture<'static, CheckBatch>> {
        None
    }
}
trait StandardHostExt {
    fn execution_status(self, category: &CheckCategory, options: Option<&AnalysisContext>) -> ExecutionStatus;
    fn from_path(path: &Path) -> Self;
    fn unsupported(self, categories: &[CheckCategory]) -> Vec<CategoryChecks>;
}
/// Execution state for one requested analysis category.
#[derive(Clone, Debug, EnumIs, Eq, PartialEq)]
pub enum ExecutionStatus {
    /// The analyzer executed the category, even if it produced no findings.
    Executed,
    /// Configuration or runtime policy deliberately prevented execution.
    Skipped {
        /// Human-readable explanation of the policy decision.
        reason: String,
    },
    /// The selected metadata standard does not implement the category.
    Unsupported {
        /// Human-readable explanation of the capability gap.
        reason: String,
    },
}
/// Results for one standard-specific group of paths.
#[derive(Clone, Debug)]
pub struct AnalysisBatch {
    /// Standard used by this batch.
    pub standard: Standard,
    /// Paths included in this batch.
    pub paths: Vec<PathBuf>,
    /// Results retained by category.
    pub categories: Vec<CategoryChecks>,
}
/// Complete structured analysis result.
#[derive(Clone, Debug)]
pub struct AnalysisReport {
    /// Standard-specific result batches.
    pub batches: Vec<AnalysisBatch>,
    /// Source indexes retained once for host rendering.
    documents: DocumentIndexes,
    /// Categories omitted by configuration.
    pub skipped_categories: Vec<CheckCategory>,
}
/// Checks returned for one category.
#[derive(Clone, Debug)]
pub struct CategoryChecks {
    /// Category that produced the checks.
    pub category: CheckCategory,
    /// Structured check results.
    pub checks: CheckBatch,
    /// Whether the category ran, was deliberately skipped, or is unsupported.
    pub status: ExecutionStatus,
}
/// Paths assigned to one analysis standard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardPaths {
    /// Standard selected for these paths.
    pub standard: Standard,
    /// Paths analyzed with the standard.
    pub paths: Vec<PathBuf>,
}
impl AnalysisReport {
    /// Flatten all structured checks without printing or exiting.
    pub fn checks(&self) -> CheckBatch {
        self.batches
            .iter()
            .flat_map(|batch| batch.categories.iter())
            .flat_map(|category| category.checks.iter().cloned())
            .collect()
    }
    /// Build a host rendering context for this report.
    pub fn render_context<'a>(&'a self, options: &'a ReportOptions) -> RenderContext<'a> {
        RenderContext::with_documents(options, &self.documents)
    }
    /// Assemble a report for a non-filesystem analysis target.
    pub(crate) fn remote(batch: AnalysisBatch, skipped_categories: Vec<CheckCategory>) -> Self {
        Self {
            batches: vec![batch],
            documents: HashMap::new(),
            skipped_categories,
        }
    }
    pub(crate) async fn with_link_evidence(self, evidence: Vec<LinkEvidence>, options: &AnalysisContext) -> Self {
        let reachability_enabled = !(options.execution.common.offline || options.execution.extension.disable_website_checks);
        let checks = CheckBatch::evaluate_link_evidence(evidence, reachability_enabled).await;
        let categories = CheckCategory::iter()
            .filter_map(|category| {
                let checks = checks.iter().filter(|check| check.category == category).cloned().collect::<CheckBatch>();
                let status = ExecutionStatus::Executed;
                (!checks.is_empty()).then_some(CategoryChecks { category, checks, status })
            })
            .collect();
        let batch = AnalysisBatch {
            standard: options.analysis.standard,
            paths: Vec::new(),
            categories,
        };
        Self {
            batches: self.batches.into_iter().chain(once(batch)).collect(),
            ..self
        }
    }
}
impl StandardHostExt for Standard {
    fn execution_status(self, category: &CheckCategory, options: Option<&AnalysisContext>) -> ExecutionStatus {
        let supported = self.supports_check(category);
        match supported {
            | false => ExecutionStatus::Unsupported {
                reason: format!("{self} does not support {category} checks"),
            },
            | true if category.is_link() && options.is_some_and(|options| options.execution.common.offline) => ExecutionStatus::Skipped {
                reason: "offline mode disables network link checks".to_string(),
            },
            | true if category.is_link() && options.is_some_and(|options| options.execution.extension.disable_website_checks) => {
                ExecutionStatus::Skipped {
                    reason: "website checks are disabled".to_string(),
                }
            }
            | true => ExecutionStatus::Executed,
        }
    }
    fn from_path(path: &Path) -> Self {
        let file_name = path.file_name().and_then(|value| value.to_str());
        let mime = MimeType::from_path(path);
        if file_name.is_some_and(|value| matches!(value, ".acorn.json" | ".acorn.jsonc" | ".acorn.yaml" | ".acorn.yml")) {
            Self::ApplicationConfigurationResearchEnablement
        } else if file_name == Some("Kitfile") {
            Self::Kitfile
        } else if file_name.is_some_and(|value| value.eq_ignore_ascii_case("modelkit.json")) {
            Self::ModelKit
        } else {
            match mime {
                | MimeType::Cff => Self::CitationFileFormat,
                | MimeType::Docx | MimeType::Pdf => Self::Document,
                | MimeType::LdJson | MimeType::RdfXml | MimeType::Turtle => Self::DublinCore,
                | MimeType::Json => read_file(path)
                    .ok()
                    .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                    .and_then(|value| value.as_object().map(Self::from))
                    .filter(|standard| standard.is_dublin_core() || standard.is_model_kit())
                    .unwrap_or(Self::ResearchActivityData),
                | MimeType::Markdown if read_file(path).is_ok_and(ResearchActivity::is_markdown) => Self::ResearchActivityData,
                | MimeType::Text | MimeType::Markdown => Self::Text,
                | MimeType::Yaml => read_file(path)
                    .ok()
                    .and_then(|content| serde_norway::from_str::<serde_json::Value>(&content).ok())
                    .and_then(|value| value.as_object().map(Self::from))
                    .filter(|standard| standard.is_dublin_core() || standard.is_kitfile())
                    .unwrap_or(Self::ResearchActivityData),
                | _ => Self::ResearchActivityData,
            }
        }
    }
    fn unsupported(self, categories: &[CheckCategory]) -> Vec<CategoryChecks> {
        categories
            .iter()
            .map(|category| CategoryChecks {
                category: category.clone(),
                checks: CheckBatch::empty(),
                status: self.execution_status(category, None),
            })
            .collect()
    }
}
impl From<&AnalysisReport> for Vec<FairProfile> {
    fn from(value: &AnalysisReport) -> Self {
        let mut grouped = value
            .batches
            .iter()
            .flat_map(|batch| batch.categories.iter())
            .flat_map(|category| category.checks.iter())
            .filter_map(Check::fair_details)
            .fold(Vec::<(String, Vec<FairDetails>)>::new(), |mut groups, finding| {
                match groups.iter_mut().find(|(resource, _)| resource == &finding.resource) {
                    | Some((_, findings)) => findings.push(finding.clone()),
                    | None => groups.push((finding.resource.clone(), vec![finding.clone()])),
                }
                groups
            });
        grouped.sort_by_cached_key(|(resource, _)| resource.to_lowercase());
        let profiles = grouped
            .into_iter()
            .map(|(resource, findings)| FairProfile::from_findings(resource, &findings))
            .collect::<Vec<_>>();
        let pooled = (profiles.len() > 1).then(|| FairProfile::pooled("All resources", &profiles));
        profiles.into_iter().chain(pooled).collect()
    }
}
impl From<AnalysisReport> for Vec<FairProfile> {
    fn from(value: AnalysisReport) -> Self {
        Self::from(&value)
    }
}
async fn analyze<T: Analysis + Send + Sync>(
    paths: &[PathBuf],
    options: &AnalysisContext,
    categories: &[CheckCategory],
    documents: &DocumentIndexes,
) -> Vec<CategoryChecks> {
    correlate_link_evidence(
        join_all(categories.iter().cloned().map(|category| async move {
            let result = T::check(category, paths, Some(options)).await;
            let checks = result
                .checks
                .into_iter()
                .map(|check| {
                    let document = check.uri.as_ref().and_then(|uri| documents.get(uri)).cloned();
                    match document {
                        | Some(document) => check.resolve_locator(&document),
                        | None => check,
                    }
                })
                .collect();
            CategoryChecks { checks, ..result }
        }))
        .await,
    )
}
/// Analyze paths and return structured results without rendering or process-exit behavior
pub async fn analyze_paths(paths: &[PathBuf], options: &AnalysisContext) -> AnalysisReport {
    let documents = document_indexes(paths);
    let skipped_categories = skipped_categories(options);
    let categories = CheckCategory::iter()
        .filter(|category| !skipped_categories.contains(category))
        .collect::<Vec<_>>();
    let groups = classify_paths(paths, options.analysis.standard);
    let mut batches = Vec::with_capacity(groups.len());
    for group in groups {
        let StandardPaths { standard, paths } = group;
        let categories = match standard {
            | Standard::ApplicationConfigurationResearchEnablement => analyze::<AcreStandard>(&paths, options, &categories, &documents).await,
            | Standard::CitationFileFormat => analyze::<Cff>(&paths, options, &categories, &documents).await,
            | Standard::Ckan => analyze::<ckan::Package>(&paths, options, &categories, &documents).await,
            | Standard::Datacite => analyze::<datacite::Record>(&paths, options, &categories, &documents).await,
            | Standard::Dcat => analyze::<dcat::Dataset>(&paths, options, &categories, &documents).await,
            | Standard::Document => analyze::<Document>(&paths, options, &categories, &documents).await,
            | Standard::DublinCore => analyze::<DublinCoreDocument>(&paths, options, &categories, &documents).await,
            | Standard::Huwise => analyze::<huwise::Dataset>(&paths, options, &categories, &documents).await,
            | Standard::Invenio => analyze::<invenio::Record>(&paths, options, &categories, &documents).await,
            | Standard::Kitfile => analyze::<KitfileStandard>(&paths, options, &categories, &documents).await,
            | Standard::ModelKit => analyze::<ModelKitStandard>(&paths, options, &categories, &documents).await,
            | Standard::ResearchActivityData => analyze::<ResearchActivity>(&paths, options, &categories, &documents).await,
            | Standard::Raid => analyze::<raid::Metadata>(&paths, options, &categories, &documents).await,
            | Standard::Text => analyze::<Text>(&paths, options, &categories, &documents).await,
            | Standard::Oci => standard.unsupported(&categories),
        };
        batches.push(AnalysisBatch { standard, paths, categories });
    }
    AnalysisReport {
        batches,
        documents,
        skipped_categories,
    }
}
/// Evaluate FAIR evidence from typed metadata values.
pub(crate) fn check_fair_for<T>(paths: &[PathBuf], standard: &str) -> CheckBatch
where
    T: InputOutput + serde::Serialize,
{
    paths
        .par_iter()
        .flat_map(|path| {
            let uri = path.file_name_with_parent();
            match T::read(path) {
                | Ok(data) => match serde_json::to_value(data) {
                    | Ok(value) => FairEvidence::from_value(uri.clone(), standard, &value)
                        .assess()
                        .into_iter()
                        .map(|finding| Check::from(finding).with_uri(Some(uri.clone())))
                        .collect(),
                    | Err(why) => CheckCategory::Fair.failure_checks(why, Some(uri)).into_vec(),
                },
                | Err(why) => CheckCategory::Fair.failure_checks(why, Some(uri)).into_vec(),
            }
        })
        .collect::<Vec<_>>()
        .into()
}
pub(crate) async fn check_prose_for<T>(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch
where
    T: Analysis + InputOutput + ToProse,
{
    let resolved_options = options.cloned().unwrap_or_default();
    let resolved_analyzer = resolved_options
        .execution
        .extension
        .resolve_analyzer::<Vale, _>(
            &resolved_options.execution.common,
            resolved_options.execution.extension.vale_config.clone().unwrap_or_default(),
        )
        .await;
    match resolved_analyzer {
        | Ok(analyzer) => {
            let sync = match resolved_options.execution.extension.skip_sync {
                | true => Ok(()),
                | false => {
                    analyzer
                        .clone()
                        .sync(resolved_options.execution.common.offline, resolved_options.execution.common.quiet)
                        .await
                }
            };
            match sync {
                | Ok(_) => {
                    let analyzer = Arc::new(analyzer);
                    let results = paths.iter().map(|path| {
                        let analyzer = Arc::clone(&analyzer);
                        let path = path.clone();
                        async move {
                            let uri = Some(path.file_name_with_parent());
                            match T::read(&path) {
                                | Ok(data) => {
                                    let output = T::output_path(&path, &data);
                                    let content = data.to_prose();
                                    analyzer
                                        .run(output, content, Some("JSON".into()))
                                        .await
                                        .into_iter()
                                        .map(|check| check.with_uri(uri.clone()))
                                        .collect()
                                }
                                | Err(why) => CheckCategory::Prose.failure_checks(why, uri),
                            }
                        }
                    });
                    join_all(results).await.into_iter().collect()
                }
                | Err(why) => {
                    error!("=> {} Vale sync — {why}", Label::fail());
                    CheckBatch::single(check_err!(CheckCategory::Prose))
                }
            }
        }
        | Err(why) => {
            error!("=> {} Resolve Vale — {why}", Label::fail());
            CheckBatch::single(check_err!(CheckCategory::Prose, message: why.to_string()))
        }
    }
}
pub(crate) fn check_quality_for<T>(paths: &[PathBuf]) -> CheckBatch
where
    T: InputOutput,
{
    paths
        .par_iter()
        .map(|path| match T::read(path) {
            | Ok(_) => CheckBatch::single(check_ok!(CheckCategory::Quality)),
            | Err(why) => CheckCategory::Quality.failure_checks(why, None),
        })
        .collect::<Vec<_>>()
        .into_iter()
        .collect()
}
pub(crate) fn check_readability_for<T>(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch
where
    T: InputOutput + ToProse,
{
    let analysis_options = options.cloned().unwrap_or_default().analysis;
    let readability_metric = analysis_options.readability_metric;
    paths
        .par_iter()
        .flat_map(|path| match T::read(path) {
            | Ok(data) => {
                let content = data.to_prose();
                let maximum = match analysis_options.readability_threshold {
                    | Some(value) => {
                        debug!(value, "=> {} Maximum allowed readability from .env", Label::using());
                        value
                    }
                    | None => readability_metric.maximum_allowed(),
                };
                let calculated = readability_metric.calculate(&content);
                debug!(calculated, "=> {} Readability index", Label::using());
                let assessment = ReadabilityScore::new(readability_metric, calculated, maximum);
                let message = assessment.to_string();
                let uri = path.file_name_with_parent();
                if assessment.is_excessive() {
                    let errors = crate::analyzer::error::ErrorKind::Readability(assessment);
                    let severity = match assessment.severity() {
                        | ReadabilitySeverity::Fail => CheckSeverity::Warning,
                        | ReadabilitySeverity::Pass => CheckSeverity::Info,
                    };
                    vec![check!(
                        CheckCategory::Readability,
                        false,
                        severity: severity,
                        uri: uri,
                        errors: errors,
                        message: message,
                        data: content
                    )]
                } else {
                    vec![check_ok!(
                        CheckCategory::Readability,
                        uri: uri,
                        message: message,
                        data: content
                    )]
                }
            }
            | Err(why) => CheckCategory::Readability.failure_checks(why, None).into_vec(),
        })
        .collect::<Vec<Check>>()
        .into()
}
/// Classify paths by explicit standard or by file type and packaging metadata in the default RAD mode.
pub fn classify_paths(paths: &[PathBuf], standard: Standard) -> Vec<StandardPaths> {
    paths.iter().cloned().fold(Vec::<StandardPaths>::new(), |mut groups, path| {
        let selected = if standard == Standard::ResearchActivityData {
            Standard::from_path(&path)
        } else {
            standard
        };
        match groups.iter_mut().find(|group| group.standard == selected) {
            | Some(group) => group.paths.push(path),
            | None => groups.push(StandardPaths {
                standard: selected,
                paths: vec![path],
            }),
        }
        groups
    })
}
fn correlate_link_evidence(categories: Vec<CategoryChecks>) -> Vec<CategoryChecks> {
    let links = categories
        .iter()
        .flat_map(|category| category.checks.iter())
        .filter(|check| check.category == CheckCategory::Link)
        .cloned()
        .collect::<Vec<_>>();
    categories
        .into_iter()
        .map(|category| CategoryChecks {
            category: category.category,
            status: category.status,
            checks: category
                .checks
                .into_iter()
                .map(|check| {
                    let finding_and_target = check
                        .fair_details()
                        .cloned()
                        .and_then(|finding| finding.target_uri.clone().map(|target| (finding, target)));
                    match finding_and_target {
                        | Some((finding, target)) => {
                            let matches_target = |link: &&Check| {
                                link.uri == check.uri && link.context.as_deref().is_some_and(|uri| DOI::normalize(uri) == DOI::normalize(&target))
                            };
                            match links.iter().find(matches_target) {
                                | Some(link) => {
                                    let state = if link.success { FairState::Supported } else { FairState::Unsupported };
                                    let evidence = link.context.clone();
                                    let remediation = match state {
                                        | FairState::Supported => format!("Evidence supports {}", finding.indicator.id),
                                        | FairState::Unsupported => {
                                            format!("Resolve {} before claiming support for {}", target, finding.indicator.id)
                                        }
                                        | FairState::Indeterminate | FairState::NotApplicable => finding.remediation.clone(),
                                    };
                                    Check::from(FairDetails {
                                        evidence,
                                        remediation,
                                        state,
                                        ..finding
                                    })
                                }
                                | None => check,
                            }
                        }
                        | None => check,
                    }
                })
                .collect(),
        })
        .collect()
}
fn document_indexes(paths: &[PathBuf]) -> HashMap<String, Arc<DocumentIndex>> {
    let parser = MarkdownParser;
    paths
        .iter()
        .filter_map(|path| {
            SourceDocument::from_path(path)
                .ok()
                .map(|document| (path.file_name_with_parent(), Arc::new(DocumentIndex::with_parsers(document, &[&parser]))))
        })
        .fold(HashMap::<String, Arc<DocumentIndex>>::new(), |mut documents, (name, document)| {
            // Keep first seen document for duplicate display keys instead of dropping all candidates, which removes highlighting entirely.
            documents.entry(name).or_insert(document);
            documents
        })
}
pub(crate) fn skipped_categories(options: &AnalysisContext) -> Vec<CheckCategory> {
    options
        .analysis
        .skip
        .iter()
        .map(CheckCategory::from)
        .fold(Vec::new(), |mut categories, category| {
            if !categories.contains(&category) {
                categories.push(category);
            }
            categories
        })
}
#[cfg(test)]
mod tests;
