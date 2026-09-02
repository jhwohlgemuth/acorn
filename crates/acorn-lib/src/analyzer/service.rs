//! Structured, reusable file analysis service
use super::{Analysis, AnalysisContext, Check, CheckCategory, DocumentIndexes, RenderContext, ReportOptions, Standard};
use crate::io::document::{DocumentIndex, MarkdownParser, SourceDocument};
use crate::io::read_file;
use crate::util::StringConversion;
use acorn_core::prelude::{Arc, HashMap};
use acorn_core::util::MimeType;
use acorn_schema::schema::pid::raid;
use acorn_schema::schema::research_activity::ResearchActivity;
use acorn_schema::schema::standard::cff::Cff;
use acorn_schema::schema::standard::text::{Document, Text};
use acorn_schema::schema::standard::{datacite, dcat, huwise, invenio};
use core::iter::once;
use std::path::PathBuf;
use strum::IntoEnumIterator;

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
    pub checks: Vec<Check>,
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
    pub fn checks(&self) -> Vec<Check> {
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
    pub(crate) fn with_checks(self, checks: Vec<Check>, options: &AnalysisContext) -> Self {
        let categories = CheckCategory::iter()
            .filter_map(|category| {
                let checks = checks.iter().filter(|check| check.category == category).cloned().collect::<Vec<_>>();
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
