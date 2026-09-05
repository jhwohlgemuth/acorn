//! Structured native file-analysis service
use super::render::{DocumentIndexes, RenderContext};
use crate::analyzer::check::{Check, CheckBatch, CheckCategory, CheckSeverity, Standard};
use crate::analyzer::host::context::{AnalysisContext, ReportOptions};
use crate::analyzer::host::runtime::StaticAnalyzer;
use crate::analyzer::host::vale::Vale;
use crate::analyzer::readability::score::{ReadabilityScore, ReadabilitySeverity};
use crate::io::document::{DocumentIndex, MarkdownParser, SourceDocument};
use crate::io::{read_file, InputOutput};
use crate::util::{Label, StringConversion};
use crate::{check, check_err, check_ok};
use acorn_core::prelude::{Arc, HashMap, Path, PathBuf};
use acorn_core::util::{MimeType, ToProse};
use acorn_schema::schema::pid::raid;
use acorn_schema::schema::research_activity::ResearchActivity;
use acorn_schema::schema::standard::cff::Cff;
use acorn_schema::schema::standard::text::{Document, Text};
use acorn_schema::schema::standard::{datacite, dcat, huwise, invenio};
use async_trait::async_trait;
use core::future::Future;
use core::iter::once;
use futures::future::{join_all, BoxFuture};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use strum::IntoEnumIterator;
use tracing::{debug, error};

/// Native analysis operations over filesystem paths.
#[async_trait]
pub trait Analysis: SchemaCheck {
    /// Run analysis for a given category.
    async fn check(category: CheckCategory, paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        match category {
            | CheckCategory::Link => Self::check_websites(paths, options).await,
            | CheckCategory::Prose => Self::check_prose(paths, options).await,
            | CheckCategory::Quality => Self::check_quality(paths, options).await,
            | CheckCategory::Readability => Self::check_readability(paths, options).await,
            | CheckCategory::Schema => Self::check_schema(paths, options).await,
            | CheckCategory::Crosswalk => CheckBatch::empty(),
        }
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
    /// Check whether URL links are valid and readable.
    async fn check_websites(_paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
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
    pub(crate) fn with_checks(self, checks: CheckBatch, options: &AnalysisContext) -> Self {
        let categories = CheckCategory::iter()
            .filter_map(|category| {
                let checks = checks.iter().filter(|check| check.category == category).cloned().collect::<CheckBatch>();
                (!checks.is_empty()).then_some(CategoryChecks { category, checks })
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
async fn analyze<T: Analysis + Send + Sync>(
    paths: &[PathBuf],
    options: &AnalysisContext,
    categories: &[CheckCategory],
    documents: &DocumentIndexes,
) -> Vec<CategoryChecks> {
    let mut results = Vec::with_capacity(categories.len());
    for category in categories {
        let checks = T::check(category.clone(), paths, Some(options))
            .await
            .into_iter()
            .map(|check| {
                let document = check.uri.as_ref().and_then(|uri| documents.get(uri)).cloned();
                match document {
                    | Some(document) => check.resolve_locator(&document),
                    | None => check,
                }
            })
            .collect();
        results.push(CategoryChecks {
            category: category.clone(),
            checks,
        });
    }
    results
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
            | Standard::CitationFileFormat => analyze::<Cff>(&paths, options, &categories, &documents).await,
            | Standard::Datacite => analyze::<datacite::Record>(&paths, options, &categories, &documents).await,
            | Standard::Dcat => analyze::<dcat::Dataset>(&paths, options, &categories, &documents).await,
            | Standard::Document => analyze::<Document>(&paths, options, &categories, &documents).await,
            | Standard::Huwise => analyze::<huwise::Dataset>(&paths, options, &categories, &documents).await,
            | Standard::Invenio => analyze::<invenio::Record>(&paths, options, &categories, &documents).await,
            | Standard::ResearchActivityData => analyze::<ResearchActivity>(&paths, options, &categories, &documents).await,
            | Standard::Raid => analyze::<raid::Metadata>(&paths, options, &categories, &documents).await,
            | Standard::Text => analyze::<Text>(&paths, options, &categories, &documents).await,
            | Standard::DublinCore => Vec::new(),
        };
        batches.push(AnalysisBatch { standard, paths, categories });
    }
    AnalysisReport {
        batches,
        documents,
        skipped_categories,
    }
}
pub(crate) async fn check_prose_for<T>(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch
where
    T: Analysis + InputOutput + ToProse,
{
    let resolved_options = options.cloned().unwrap_or_default();
    let analyzer = resolved_options
        .execution
        .resolve_analyzer::<Vale, _>(resolved_options.execution.vale_config.clone().unwrap_or_default())
        .await;
    let sync = match resolved_options.execution.skip_sync {
        | true => Ok(()),
        | false => {
            analyzer
                .clone()
                .sync(resolved_options.execution.offline, resolved_options.execution.quiet)
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
                        | Err(why) => CheckBatch::single(check_err!(CheckCategory::Prose, context: why.to_string()).with_uri(uri)),
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
            | Err(why) => vec![check_err!(CheckCategory::Readability, context: why.to_string())],
        })
        .collect::<Vec<Check>>()
        .into()
}
/// Classify paths by explicit standard or by file type for the default RAD mode
pub fn classify_paths(paths: &[PathBuf], standard: Standard) -> Vec<StandardPaths> {
    paths.iter().cloned().fold(Vec::<StandardPaths>::new(), |mut groups, path| {
        let selected = if standard == Standard::ResearchActivityData {
            match MimeType::from(path.display().to_string()) {
                | MimeType::Cff => Standard::CitationFileFormat,
                | MimeType::Docx | MimeType::Pdf => Standard::Document,
                | MimeType::Markdown if read_file(&path).is_ok_and(ResearchActivity::is_markdown) => Standard::ResearchActivityData,
                | MimeType::Text | MimeType::Markdown => Standard::Text,
                | _ => Standard::ResearchActivityData,
            }
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
fn skipped_categories(options: &AnalysisContext) -> Vec<CheckCategory> {
    options
        .analysis
        .skip
        .iter()
        .map(CheckCategory::from)
        .chain((options.execution.offline || options.execution.disable_website_checks).then_some(CheckCategory::Link))
        .fold(Vec::new(), |mut categories, category| {
            if !categories.contains(&category) {
                categories.push(category);
            }
            categories
        })
}
#[cfg(test)]
mod tests;
