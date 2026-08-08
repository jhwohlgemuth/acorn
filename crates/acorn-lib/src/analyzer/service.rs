//! Structured, reusable file analysis service.
use super::{Analysis, Check, CheckCategory, CheckOptions, Standard};
use crate::prelude::PathBuf;
use crate::schema::pid::raid;
use crate::schema::research_activity::ResearchActivity;
use crate::schema::standard::cff::Cff;
use crate::schema::standard::text::{Docx, Text};
use crate::schema::standard::{datacite, dcat, huwise, invenio};
use core::iter::once;
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
    /// Categories omitted by configuration.
    pub skipped_categories: Vec<CheckCategory>,
    /// Standard-specific result batches.
    pub batches: Vec<AnalysisBatch>,
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
}
async fn analyze<T: Analysis + Send + Sync>(paths: &[PathBuf], options: &CheckOptions, categories: &[CheckCategory]) -> Vec<CategoryChecks> {
    let mut results = Vec::with_capacity(categories.len());
    for category in categories {
        results.push(CategoryChecks {
            category: category.clone(),
            checks: T::check(category.clone(), paths, Some(options)).await,
        });
    }
    results
}
/// Analyze paths and return structured results without rendering or process-exit behavior
pub async fn analyze_paths(paths: &[PathBuf], options: &CheckOptions) -> AnalysisReport {
    let skipped_categories = skipped_categories(options);
    let categories = CheckCategory::iter()
        .filter(|category| !skipped_categories.contains(category))
        .collect::<Vec<_>>();
    let groups = classify_paths(paths, options.standard);
    let mut batches = Vec::with_capacity(groups.len());
    for group in groups {
        let StandardPaths { standard, paths } = group;
        let categories = match standard {
            | Standard::CitationFileFormat => analyze::<Cff>(&paths, options, &categories).await,
            | Standard::Datacite => analyze::<datacite::Record>(&paths, options, &categories).await,
            | Standard::Dcat => analyze::<dcat::Dataset>(&paths, options, &categories).await,
            | Standard::Docx => analyze::<Docx>(&paths, options, &categories).await,
            | Standard::Huwise => analyze::<huwise::Dataset>(&paths, options, &categories).await,
            | Standard::Invenio => analyze::<invenio::Record>(&paths, options, &categories).await,
            | Standard::ResearchActivityData => analyze::<ResearchActivity>(&paths, options, &categories).await,
            | Standard::Raid => analyze::<raid::Metadata>(&paths, options, &categories).await,
            | Standard::Text => analyze::<Text>(&paths, options, &categories).await,
            | Standard::DublinCore => Vec::new(),
        };
        batches.push(AnalysisBatch { standard, paths, categories });
    }
    AnalysisReport { skipped_categories, batches }
}
/// Classify paths by explicit standard or by file type for the default RAD mode
pub fn classify_paths(paths: &[PathBuf], standard: Standard) -> Vec<StandardPaths> {
    paths.iter().cloned().fold(Vec::<StandardPaths>::new(), |mut groups, path| {
        let selected = if standard == Standard::ResearchActivityData {
            match path.extension().and_then(|value| value.to_str()).map(str::to_ascii_lowercase).as_deref() {
                | Some("cff") => Standard::CitationFileFormat,
                | Some("docx") => Standard::Docx,
                | Some("txt" | "md" | "markdown") => Standard::Text,
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
fn skipped_categories(options: &CheckOptions) -> Vec<CheckCategory> {
    options
        .skip
        .iter()
        .map(CheckCategory::from)
        .chain(once(CheckCategory::Quality))
        .chain((options.offline || options.disable_website_checks).then_some(CheckCategory::Link))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_mixed_supported_paths_without_mutating_order() {
        let groups = classify_paths(
            &[
                PathBuf::from("CITATION.cff"),
                PathBuf::from("activity.json"),
                PathBuf::from("notes.md"),
                PathBuf::from("activity.jsonc"),
                PathBuf::from("report.docx"),
                PathBuf::from("activity.yaml"),
            ],
            Standard::ResearchActivityData,
        );
        assert_eq!(
            groups,
            vec![
                StandardPaths {
                    standard: Standard::CitationFileFormat,
                    paths: vec![PathBuf::from("CITATION.cff")],
                },
                StandardPaths {
                    standard: Standard::ResearchActivityData,
                    paths: vec![
                        PathBuf::from("activity.json"),
                        PathBuf::from("activity.jsonc"),
                        PathBuf::from("activity.yaml"),
                    ],
                },
                StandardPaths {
                    standard: Standard::Text,
                    paths: vec![PathBuf::from("notes.md")],
                },
                StandardPaths {
                    standard: Standard::Docx,
                    paths: vec![PathBuf::from("report.docx")],
                },
            ]
        );
    }

    #[test]
    fn explicit_standard_applies_to_every_path() {
        let groups = classify_paths(&[PathBuf::from("one.json"), PathBuf::from("two.yaml")], Standard::Datacite);
        assert_eq!(
            groups,
            vec![StandardPaths {
                standard: Standard::Datacite,
                paths: vec![PathBuf::from("one.json"), PathBuf::from("two.yaml")],
            }]
        );
    }
}
