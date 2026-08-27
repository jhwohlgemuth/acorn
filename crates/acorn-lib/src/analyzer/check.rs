//! Check result types and helpers for analyzer output.
use super::error::{DocumentTarget, ErrorKind, ValidatorIssues};
use super::readability::ReadabilityType;
use super::vale::{Vale, ValeConfig, ValeOutputItem, ValeOutputItemSeverity};
#[cfg(feature = "analysis")]
use super::{to_dataframe, StaticAnalyzer, StaticAnalyzerConfig};
use crate::io::document::{DocumentIndex, DocumentMatch, DocumentPath, DocumentPosition, DocumentQuery, DocumentSpan};
use crate::io::InputOutput;
#[cfg(feature = "analysis")]
use crate::prelude::Cursor;
use crate::prelude::{Arc, HashMap, Path, PathBuf};
#[cfg(feature = "std")]
use crate::schema::pid::raid::Metadata;
#[cfg(feature = "std")]
use crate::schema::research_activity::ResearchActivity;
use crate::schema::standard::crosswalk::mapping::{
    datacite_to_dcat, datacite_to_huwise, datacite_to_invenio, dcat_to_datacite, huwise_to_datacite, invenio_to_datacite,
};
use crate::schema::standard::crosswalk::{ConversionWarning, CrosswalkError, FieldMapping, Fields, SchemaExtractor};
use crate::schema::standard::{cff, datacite, dcat, huwise, invenio, text};
use crate::schema::OneOrMany;
use crate::util::constants::MAX_LENGTH_REPORT_SPAN_PREFIX;
use crate::util::{Label, MimeType, StringConversion, ToProse};
use crate::{check, check_err, check_ok};
use ariadne::{Color, Config, IndexType, Report, ReportKind, Source};
use async_trait::async_trait;
use bon::Builder;
use color_eyre::owo_colors::OwoColorize;
use convert_case::{Case, Casing};
use core::fmt;
#[cfg(feature = "std")]
use core::future::Future;
use derive_more::Display;
use futures::future::join_all;
use jiff::SignedDuration;
#[cfg(feature = "analysis")]
use polars::{
    frame::row::Row,
    io::csv::write::CsvWriter,
    prelude::{AnyValue, DataFrame, PolarsResult, SerWriter},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
#[cfg(feature = "std")]
use schemars::schema_for;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use strum::{EnumIs, EnumIter, VariantNames};
use tracing::{debug, error, info};

/// Utility type alias for a vector of [`ConversionWarning`]
pub type ConversionWarnings = Vec<ConversionWarning>;
/// Collection of analyzer checks.
pub type Checks = Vec<Check>;
/// Render a collection of checks.
pub trait Render {
    /// Render checks in deterministic detailed or compact form.
    fn render(&self, options: &CheckOptions);
}
/// Trait for adding analysis capabilities
#[cfg(feature = "analysis")]
#[async_trait]
pub trait Analysis {
    /// Run analysis for a given category
    async fn check(category: CheckCategory, paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        match category {
            | CheckCategory::Link => Self::check_websites(paths, options).await,
            | CheckCategory::Prose => Self::check_prose(paths, options).await,
            | CheckCategory::Quality => Self::check_quality(paths, options).await,
            | CheckCategory::Readability => Self::check_readability(paths, options).await,
            | CheckCategory::Schema => Self::check_schema(paths, options).await,
            | CheckCategory::Crosswalk => Vec::new(),
        }
    }
    /// Perform analysis of prose
    async fn check_prose(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        Vec::new()
    }
    /// Perform quality checks and verify data consistency
    async fn check_quality(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        Vec::new()
    }
    /// Calculate readability using a given metric
    async fn check_readability(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        Vec::new()
    }
    /// Execute validation checks for a given item
    async fn check_schema(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        Vec::new()
    }
    /// Check if URLs links are valid and readable
    async fn check_websites(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        Vec::new()
    }
    /// Returns the output path used for Vale analysis artifacts.
    fn output_path(path: &Path, data: &Self) -> PathBuf;
    /// Returns the metadata standard represented by the implementing type.
    fn standard() -> Standard;
    /// Flatten a collection of futures that each return a vector of checks into a single vector
    #[cfg(feature = "std")]
    async fn flatten_checks<I, F>(futures: I) -> Vec<Check>
    where
        I: IntoIterator<Item = F> + Send,
        F: Future<Output = Vec<Check>> + Send,
    {
        join_all(futures).await.into_iter().flatten().collect()
    }
    /// Collect link checks from futures and attach URI to each check
    #[cfg(feature = "std")]
    async fn collect_checks<'a>(futures: impl IntoIterator<Item = futures::future::BoxFuture<'a, Check>> + Send, path: &Path) -> Vec<Check> {
        let uri = Some(path.to_path_buf().file_name_with_parent());
        join_all(futures).await.into_iter().map(|check| check.with_uri(uri.clone())).collect()
    }
}
/// Trait for converting to a vector of checks
pub trait IntoChecks {
    /// Convert to a vector of checks
    fn to_checks(&self, uri: Option<String>) -> Vec<Check>;
}
/// Trait for converting to a ([Polars]) row
///
/// [Polars]: https://docs.rs/polars/latest/polars/
#[cfg(feature = "analysis")]
pub trait IntoRow<'a> {
    /// Convert to a (Polars) row
    fn to_row<T>(self) -> Row<'a>;
}
/// Metadata standard used to gate processing behavior within commands
#[derive(Clone, Copy, Debug, Default, Display, PartialEq, Eq)]
pub enum Standard {
    /// Research Activity Data (RAD)
    #[default]
    #[display("research-activity-data")]
    ResearchActivityData,
    /// Citation File Format (CFF)
    #[display("citation-file-format")]
    CitationFileFormat,
    /// DataCite
    #[display("datacite")]
    Datacite,
    /// Data Catalog (DCAT)
    #[display("dcat")]
    Dcat,
    /// Document-derived text
    #[display("document")]
    Document,
    /// Dublin Core
    #[display("dublin-core")]
    DublinCore,
    /// HUBwise
    #[display("huwise")]
    Huwise,
    /// InvenioRDM
    #[display("invenio")]
    Invenio,
    /// Research Activity Identifier metadata (RAiD)
    #[display("raid")]
    Raid,
    /// Plain text
    #[display("text")]
    Text,
}
/// Various check categories available for validating research activity data
#[derive(Clone, Debug, Default, Deserialize, Display, EnumIter, PartialEq, Serialize, VariantNames)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum CheckCategory {
    /// Schema validation check
    #[default]
    #[display("schema")]
    Schema,
    /// Website availability check
    #[display("link")]
    Link,
    /// Static analysis of prose
    #[display("prose")]
    Prose,
    /// Quality control and data consistency check
    #[display("quality")]
    Quality,
    /// Readability check using one of several metrics
    #[display("readability")]
    Readability,
    /// Metadata crosswalk conversion warning
    #[display("crosswalk")]
    Crosswalk,
}
/// Severity level of a check result
#[derive(Clone, Debug, Default, Deserialize, Display, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckSeverity {
    /// Hard error requiring immediate attention
    #[default]
    #[display("error")]
    Error,
    /// Informational message
    #[display("info")]
    Info,
    /// Advisory suggestion to improve content quality
    #[display("suggestion")]
    Suggestion,
    /// Content issue that should be addressed
    #[display("warning")]
    Warning,
}
#[derive(Clone, Copy, Debug, Default, EnumIs, Eq, PartialEq)]
/// Rendering behavior associated with a check.
pub enum CheckKind {
    /// Standard category-specific rendering.
    #[default]
    Standard,
    /// Identifier discovery and persistence lifecycle rendering.
    Lifecycle,
}
/// Output format used when rendering checks and discovery reports.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OutputFormat {
    /// JavaScript Object Notation
    #[default]
    Json,
    /// Human-readable console output
    Console,
    /// GitHub-Flavored Markdown
    Markdown,
    /// Plain-text values, one per line
    Raw,
    /// YAML
    Yaml,
}
/// Data structure for holding the result of a schema validation check
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct Check {
    /// Position of this issue in rendered output
    #[builder(default = 0)]
    pub index: usize,
    /// Check category
    pub category: CheckCategory,
    /// Unified issue location for checks (prose location or schema path)
    pub locator: Option<String>,
    /// Textual context of check (e.g., paragraph where prose issues were found)
    pub context: Option<String>,
    /// Internal payload used by check-specific renderers (e.g., prose source text)
    data: Option<String>,
    /// Original document index used only by detailed renderers
    document: Option<Arc<DocumentIndex>>,
    #[builder(default)]
    kind: CheckKind,
    /// Whether or not the check was successful
    #[builder(default = false)]
    pub success: bool,
    /// Severity of the check result
    #[builder(default)]
    pub severity: CheckSeverity,
    /// HTTP status code
    status_code: Option<String>,
    /// Errors and issues found during check
    pub errors: Option<ErrorKind>,
    /// Path of file being validated
    pub uri: Option<String>,
    /// Message related to or description of validation issue (e.g., key name of invalid value, result of validation, etc.)
    #[builder(default = "".to_string())]
    pub message: String,
}
/// Data structure for passing options to check processing functions
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into), on(ReadabilityType, into))]
#[derive(Default)]
pub struct CheckOptions {
    /// Include non-failure results such as suggestions and info messages
    #[builder(default = false)]
    pub all: bool,
    /// Disable website checks that require internet access
    #[builder(default = false)]
    pub disable_website_checks: bool,
    /// Flag used to indicate if a single error should cause the process to exit
    #[builder(default = false)]
    pub exit_on_first_error: bool,
    /// Whether or not to run in offline mode (i.e., skip website checks)
    #[builder(default = false)]
    pub offline: bool,
    /// Suppress stdout/stderr from external commands (e.g., Vale sync)
    #[builder(default = false)]
    pub quiet: bool,
    /// Prevent non-zero exit behavior when failures are present
    #[builder(default = false)]
    pub no_fail: bool,
    /// Continue checking when analyzer synchronization retries are exhausted
    #[builder(default = false)]
    pub ignore_sync_failure: bool,
    /// Categories of checks to skip during processing
    #[builder(default)]
    pub skip: Vec<String>,
    /// Metadata standard used to resolve check processing behavior
    #[builder(default)]
    pub standard: Standard,
    /// Readability metric to use for readability checks
    #[builder(default)]
    pub readability_metric: ReadabilityType,
    /// Whether or not to skip checksum verification for prose analysis
    #[builder(default = false)]
    pub skip_verify_checksum: bool,
    /// Skip analyzer synchronization before prose analysis
    #[builder(default = false)]
    pub skip_sync: bool,
    /// Delay between analyzer synchronization attempts
    #[builder(default = SignedDuration::from_secs(1))]
    pub sync_retry_interval: SignedDuration,
    /// Use compact output format instead of detailed output
    #[builder(default = false)]
    pub terse: bool,
    /// Filter rendered checks by the effective verbosity level
    #[builder(default = false)]
    pub filter_by_verbosity: bool,
    /// Output format used by the current command
    #[builder(default)]
    pub format: OutputFormat,
    /// Effective check visibility level
    pub verbosity: Option<u8>,
    /// Vale configuration options
    pub vale_config: Option<ValeConfig>,
}
#[derive(Builder)]
#[builder(start_fn = init)]
struct DocumentReport<'a> {
    title: &'a str,
    index: usize,
    kind: ReportKind<'static>,
    heading: &'a str,
    message: String,
    color: Color,
}
impl CheckKind {
    fn link_row(&self, check: &Check) -> (String, String, String) {
        let Check {
            severity,
            context,
            locator,
            status_code,
            message,
            ..
        } = check;
        let details = if self.is_lifecycle() {
            context
                .as_ref()
                .map_or_else(|| message.clone(), |context| format!("{message} {}", context.dimmed()))
        } else {
            let code = match status_code {
                | Some(value) if !value.is_empty() => format!(" ({value})").dimmed().to_string(),
                | None | Some(_) => "".to_string(),
            };
            let context = context
                .as_ref()
                .map(|value| value.to_string().underline().italic().to_string())
                .unwrap_or_else(|| "Missing".to_string());
            format!("{context} {}{code}", message.dimmed())
        };
        (locator.clone().unwrap_or_default(), severity.colored(), details)
    }
}
impl CheckOptions {
    /// Resolve the configured static analyzer.
    #[cfg(feature = "analysis")]
    pub async fn resolve_analyzer<Analyzer, Config>(&self, config: Config) -> Analyzer
    where
        Analyzer: StaticAnalyzer<Config>,
        Config: StaticAnalyzerConfig,
    {
        Analyzer::resolve(config.save().await, self.offline, self.skip_verify_checksum).await
    }
}
impl Check {
    pub(crate) fn with_kind(self, kind: CheckKind) -> Self {
        Self { kind, ..self }
    }
    /// Returns whether this check represents a failure (not successful and severity warrants failure)
    pub fn is_failure(&self) -> bool {
        !self.success && self.severity.is_failure()
    }
    /// Returns a function that filters checks based on the given log level threshold
    pub fn is_visible_at(level: Option<u8>) -> impl Fn(&Check) -> bool {
        move |check: &Check| check.severity.is_visible_at(level)
    }
    /// Returns the number of errors
    pub fn issue_count(&self) -> usize {
        match self.category {
            | CheckCategory::Link | CheckCategory::Quality | CheckCategory::Readability => 1,
            | CheckCategory::Prose => {
                if let Some(kind) = &self.errors {
                    match kind {
                        | ErrorKind::Vale(values) => values.len(),
                        | _ => 1,
                    }
                } else if !self.message.is_empty() {
                    1
                } else {
                    0
                }
            }
            | CheckCategory::Schema => {
                if let Some(kind) = &self.errors {
                    match kind {
                        | ErrorKind::Validator(content) => content.len(),
                        | _ => 0,
                    }
                } else {
                    0
                }
            }
            | CheckCategory::Crosswalk => {
                if self.success {
                    0
                } else {
                    1
                }
            }
        }
    }
    /// Fancy output in contrast with more compressed output of Display impl
    pub fn report(&self) {
        let index = self.index;
        let Check {
            category,
            locator,
            context,
            data,
            document,
            errors,
            kind,
            severity,
            status_code,
            uri,
            message,
            ..
        } = self.clone();
        match &category {
            | CheckCategory::Link => match kind {
                | CheckKind::Lifecycle => {
                    let title = uri.as_deref().or(locator.as_deref()).unwrap_or("resolution");
                    let source_text = context.as_deref().unwrap_or(&message).to_string();
                    let source = Source::from(source_text.clone());
                    let span = 0..=source_text.len().saturating_sub(1);
                    let _ = Report::build(severity.clone().into(), (title, 1..=1))
                        .with_code(index)
                        .with_config(Config::default().with_compact(false))
                        .with_message(locator.clone().unwrap_or("Resolution".to_string()))
                        .with_label(
                            ariadne::Label::new((title, span))
                                .with_message(message.italic())
                                .with_color(severity.clone().into()),
                        )
                        .finish()
                        .print((title, source));
                }
                | CheckKind::Standard => {
                    let title = uri.clone().unwrap_or_else(|| "index.json".to_string());
                    let kind: ReportKind<'static> = severity.clone().into();
                    let code = match status_code.as_ref() {
                        | Some(value) if !value.is_empty() => format!(" ({value})").dimmed().to_string(),
                        | None | Some(_) => "".to_string(),
                    };
                    let text = match context.as_ref() {
                        | Some(value) => value.trim().to_string(),
                        | None => "No URL".to_string(),
                    };
                    let source_text = if text.is_empty() { " ".to_string() } else { text };
                    let query = match locator.as_deref() {
                        | Some(locator) => DocumentQuery::new()
                            .with_path(DocumentPath::parse(locator))
                            .with_needle(source_text.clone()),
                        | None => DocumentQuery::new().with_needle(source_text.clone()),
                    };
                    let heading = locator.as_deref().unwrap_or(category.message());
                    let report = DocumentReport::init()
                        .title(&title)
                        .index(index)
                        .kind(kind)
                        .heading(heading)
                        .message(format!("{}{code}", message.italic()))
                        .color(severity.clone().into())
                        .build();
                    let rendered = document.as_ref().is_some_and(|document| print_document_report(document, &query, report));
                    if !rendered {
                        print_unlocated(index, &severity, heading, format!("{message}{code}"));
                    }
                }
            },
            | CheckCategory::Prose => match &errors {
                | Some(ErrorKind::Vale(values)) => {
                    let title = uri.clone().unwrap_or("index.json".to_string());
                    values.iter().enumerate().for_each(|(i, item)| {
                        let ValeOutputItem {
                            check, message, severity, ..
                        } = item;
                        let report = DocumentReport::init()
                            .title(&title)
                            .index(index.saturating_add(i))
                            .kind(severity.into())
                            .heading(check)
                            .message(message.italic().to_string())
                            .color(severity.into())
                            .build();
                        let rendered = document
                            .as_ref()
                            .zip(self.prose_span(item))
                            .is_some_and(|(document, span)| document.print(&span, report));
                        if !rendered {
                            print_unlocated(index.saturating_add(i), &CheckSeverity::from(severity.clone()), check, message.clone());
                        }
                    });
                }
                | None | Some(_) => {}
            },
            | CheckCategory::Quality => {
                let title = uri.as_deref().unwrap_or_default();
                if self.success {
                    info!("=> {} {title} has {}", Label::pass(), "no quality issues".green().bold());
                } else {
                    let details = context.as_deref().unwrap_or(&message).to_string();
                    let heading = uri.as_deref().map_or_else(
                        || category.message().to_string(),
                        |uri| format!("{} — {}", category.message(), uri.cyan().underline()),
                    );
                    print_unlocated(index, &severity, &heading, details);
                }
            }
            | CheckCategory::Readability => match &errors {
                | Some(ErrorKind::Readability((_readability_index, _readability_type))) => {
                    let score = severity.colorize(&message);
                    let heading = uri.as_deref().map_or_else(
                        || category.message().to_string(),
                        |uri| format!("{} — {}", category.message(), uri.cyan().underline()),
                    );
                    let details = format!("Simplify language for a general audience — {score}\n");
                    print_unlocated(index, &severity, &heading, details);
                }
                | None | Some(_) => {
                    let title = uri.as_deref().unwrap_or_default();
                    if let Some(value) = &data {
                        info!(
                            "=> {} {title} has {} {}",
                            Label::pass(),
                            "no readability issues".green().bold(),
                            value.dimmed()
                        );
                    } else {
                        info!("=> {} {title} has {}", Label::pass(), "no readability issues".green().bold(),);
                    }
                }
            },
            | CheckCategory::Schema => {
                let title = uri.clone().unwrap_or_default();
                match &errors {
                    | Some(ErrorKind::Validator(report)) => {
                        let prefix = context.as_deref().or(locator.as_deref()).unwrap_or(&message);
                        ValidatorIssues::from(report.clone())
                            .with_prefix(prefix)
                            .iter()
                            .enumerate()
                            .for_each(|(issue_index, issue)| {
                                let locator = issue.locator();
                                let query = issue.query();
                                let report = DocumentReport::init()
                                    .title(&title)
                                    .index(index.saturating_add(issue_index))
                                    .kind(severity.clone().into())
                                    .heading(&locator)
                                    .message(issue.message.italic().to_string())
                                    .color(severity.clone().into())
                                    .build();
                                let rendered = document.as_ref().is_some_and(|document| print_document_report(document, &query, report));
                                if !rendered {
                                    print_unlocated(index.saturating_add(issue_index), &severity, &locator, issue.message.clone());
                                }
                            });
                    }
                    | None if !self.success => {
                        let error_text = context.as_deref().unwrap_or(&message);
                        print_unlocated(index, &severity, category.message(), error_text.to_string());
                    }
                    | None | Some(_) => {
                        info!("=> {} {title} has {}", Label::pass(), "no schema validation issues".green().bold());
                    }
                }
            }
            | CheckCategory::Crosswalk => {
                print_unlocated(index, &severity, locator.as_deref().unwrap_or(category.message()), message);
            }
        }
    }
    pub(crate) fn attach_document(self, documents: &HashMap<String, Arc<DocumentIndex>>) -> Self {
        let document = self.uri.as_ref().and_then(|uri| documents.get(uri)).cloned();
        match document {
            | Some(document) => {
                let attached = self.with_document(document);
                let locator = match (&attached.category, &attached.errors) {
                    | (CheckCategory::Prose, Some(ErrorKind::Vale(values))) => values.first().and_then(|item| attached.prose_locator(item)),
                    | _ => attached.locator.clone(),
                };
                Self { locator, ..attached }
            }
            | None => self,
        }
    }
    fn prose_locator(&self, item: &ValeOutputItem) -> Option<String> {
        self.prose_position(item)
            .map(|position| format!("Line {}, Character {}", position.line, position.column))
    }
    fn prose_position(&self, item: &ValeOutputItem) -> Option<DocumentPosition> {
        self.document
            .as_ref()
            .zip(self.prose_span(item))
            .and_then(|(document, span)| document.position(span.0.start))
    }
    fn prose_span(&self, item: &ValeOutputItem) -> Option<DocumentSpan> {
        self.document
            .as_ref()
            .and_then(|document| match self.data.as_deref() == Some(document.document().content.as_str()) {
                | true => document.span_from_line_columns(item.line, &item.span),
                | false => match document.resolve(&item.query()) {
                    | DocumentMatch::Unique(span) => Some(span),
                    | DocumentMatch::Missing | DocumentMatch::Ambiguous => None,
                },
            })
    }
    pub(crate) fn terse(&self) -> String {
        let uri = self.uri.as_deref().unwrap_or_default();
        let positions = self
            .document
            .as_ref()
            .map_or_else(Vec::new, |document| match (&self.category, &self.kind, &self.errors) {
                | (CheckCategory::Link, CheckKind::Standard, _) => {
                    let context = self.context.as_deref().map(str::trim).unwrap_or("No URL");
                    let text = if context.is_empty() { " " } else { context };
                    let query = match self.locator.as_deref() {
                        | Some(locator) => DocumentQuery::new().with_path(DocumentPath::parse(locator)).with_needle(text),
                        | None => DocumentQuery::new().with_needle(text),
                    };
                    vec![document.locate(&query)]
                }
                | (CheckCategory::Prose, _, Some(ErrorKind::Vale(values))) => values.iter().map(|item| self.prose_position(item)).collect(),
                | (CheckCategory::Schema, _, Some(ErrorKind::Validator(report))) => {
                    let prefix = self.context.as_deref().or(self.locator.as_deref()).unwrap_or(&self.message);
                    ValidatorIssues::from(report.clone())
                        .with_prefix(prefix)
                        .iter()
                        .map(|issue| document.locate(&issue.query()))
                        .collect()
                }
                | _ => Vec::new(),
            });
        self.to_string()
            .split('\n')
            .enumerate()
            .map(|(index, row)| {
                let path = positions
                    .get(index)
                    .and_then(|position| *position)
                    .map_or_else(|| format!(", path={uri}"), |position| format!(", path={uri}:{position}"));
                format!("{row}{}", path.dimmed())
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    /// Returns a new LinkCheckResult with the given context
    pub fn with_context(self, value: String) -> Self {
        Self {
            context: Some(value),
            ..self
        }
    }
    /// Returns a new check with renderer data attached.
    pub fn with_data(self, value: String) -> Self {
        Self { data: Some(value), ..self }
    }
    pub(crate) fn with_document(self, value: Arc<DocumentIndex>) -> Self {
        Self {
            document: Some(value),
            ..self
        }
    }
    /// Returns a new LinkCheckResult with the given index
    pub fn with_index(self, value: usize) -> Self {
        Self { index: value, ..self }
    }
    /// Returns a new LinkCheckResult with the given locator
    pub fn with_locator(self, value: Option<String>) -> Self {
        Self { locator: value, ..self }
    }
    /// Returns a new LinkCheckResult with the given URL
    pub fn with_uri(self, value: Option<String>) -> Self {
        Self { uri: value, ..self }
    }
}
impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const INDENT: &str = " ";
        const COL_ONE_WIDTH: usize = 28;
        const COL_TWO_WIDTH: usize = 29;
        let format_row = |first: String, second: String, third: String| format!("{INDENT}{first:<COL_ONE_WIDTH$} {second:<COL_TWO_WIDTH$} {third}");
        match self.category {
            | CheckCategory::Link => {
                let (locator, severity, details) = self.kind.link_row(self);
                f.write_str(&format_row(locator, severity, details))
            }
            | CheckCategory::Prose => match &self.errors {
                | Some(ErrorKind::Vale(values)) => {
                    let last = values.len().saturating_sub(1);
                    values.iter().enumerate().try_for_each(|(index, item)| {
                        let ValeOutputItem {
                            check, message, severity, ..
                        } = item;
                        let location = self.prose_locator(item).unwrap_or_else(|| "Unlocated".to_string());
                        let severity = CheckSeverity::from(severity.clone());
                        let details = format!("{} {}{}", message, "rule=".dimmed(), check.dimmed());
                        f.write_fmt(format_args!("{}", format_row(location, severity.colored(), details)))
                            .and_then(|_| if index < last { writeln!(f) } else { Ok(()) })
                    })
                }
                | None | Some(_) => write!(f, ""),
            },
            | CheckCategory::Quality => {
                let location = self.uri.clone().or_else(|| self.locator.clone()).unwrap_or_default();
                let severity = match self.success {
                    | true => CheckSeverity::Info,
                    | false => self.severity.clone(),
                };
                let details = self
                    .context
                    .as_ref()
                    .filter(|value| !value.is_empty())
                    .or_else(|| (!self.message.is_empty()).then_some(&self.message))
                    .map_or_else(|| "No quality issues".to_string(), Clone::clone);
                f.write_str(&format_row(location, severity.colored(), details))
            }
            | CheckCategory::Readability => {
                let Check {
                    errors, message, severity, ..
                } = self;
                match &errors {
                    | Some(ErrorKind::Readability((_readability_index, _readability_type))) => {
                        let details = format!("Simplify language for a general audience {}", message.dimmed());
                        f.write_fmt(format_args!(
                            "{}",
                            format_row("Overall reading level".to_string(), severity.colored(), details)
                        ))
                    }
                    | None | Some(_) => write!(f, ""),
                }
            }
            | CheckCategory::Schema => match &self.errors {
                | Some(ErrorKind::Validator(report)) => {
                    let Check {
                        context,
                        locator,
                        message,
                        severity,
                        ..
                    } = self;
                    let prefix = context.as_deref().or(locator.as_deref()).unwrap_or(message.as_str());
                    let rows = ValidatorIssues::from(report.clone()).with_prefix(prefix);
                    let last = rows.len().saturating_sub(1);
                    rows.iter().enumerate().try_for_each(|(index, issue)| {
                        let location = issue.locator();
                        let details = format!("{} {}{}", issue.message, "code=".dimmed(), issue.code.to_string().to_uppercase().dimmed());
                        f.write_fmt(format_args!("{}", format_row(location, severity.colored(), details)))
                            .and_then(|_| if index < last { writeln!(f) } else { Ok(()) })
                    })
                }
                | None if !self.success => {
                    let error_text = self.context.as_deref().unwrap_or(&self.message);
                    write!(
                        f,
                        "{}",
                        format_row(
                            "Deserialization error".to_string(),
                            self.severity.colored(),
                            error_text.to_string().dimmed().to_string()
                        )
                    )
                }
                | None | Some(_) => write!(f, ""),
            },
            | CheckCategory::Crosswalk => {
                let details = self.message.dimmed().to_string();
                f.write_fmt(format_args!(
                    "{}",
                    format_row(self.locator.clone().unwrap_or_default(), self.severity.colored(), details)
                ))
            }
        }
    }
}

impl CheckCategory {
    /// Returns `true` if this category appears in the given list of category items
    pub fn is_in(&self, items: &[String]) -> bool {
        items.iter().any(|value| Self::from(value.as_str()) == *self)
    }
    fn message(&self) -> &'static str {
        match self {
            | Self::Link => "URL",
            | Self::Prose => "Prose",
            | Self::Quality => "Quality",
            | Self::Readability => "Readability",
            | Self::Schema => "Deserialization error",
            | Self::Crosswalk => "Crosswalk warning",
        }
    }
}
impl From<String> for CheckCategory {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}
impl From<&String> for CheckCategory {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}
impl From<&str> for CheckCategory {
    fn from(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            | "link" => Self::Link,
            | "prose" => Self::Prose,
            | "quality" => Self::Quality,
            | "readability" => Self::Readability,
            | "crosswalk" => Self::Crosswalk,
            | "schema" => Self::Schema,
            | _ => CheckCategory::default(),
        }
    }
}
impl From<&CheckCategory> for u8 {
    fn from(value: &CheckCategory) -> Self {
        match value {
            | CheckCategory::Schema | CheckCategory::Link => 0,
            | CheckCategory::Prose => 1,
            | CheckCategory::Quality => 2,
            | CheckCategory::Readability => 3,
            | CheckCategory::Crosswalk => 4,
        }
    }
}
impl CheckSeverity {
    fn colorize(&self, value: impl fmt::Display) -> String {
        match self {
            | Self::Error => value.red().to_string(),
            | Self::Info => value.cyan().to_string(),
            | Self::Suggestion => value.blue().to_string(),
            | Self::Warning => value.yellow().to_string(),
        }
    }
    /// Returns a colored string representation of the severity level
    pub fn colored(&self) -> String {
        match self {
            | Self::Error => self.to_string().red().bold().to_string(),
            | Self::Info => self.to_string().cyan().bold().to_string(),
            | Self::Suggestion => self.to_string().blue().bold().to_string(),
            | Self::Warning => self.to_string().yellow().bold().to_string(),
        }
    }
    /// Returns whether this severity should cause check failure.
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Error | Self::Warning)
    }
}
impl Render for Checks {
    fn render(&self, options: &CheckOptions) {
        let is_visible = Check::is_visible_at(options.verbosity);
        let mut sorted_checks = self
            .iter()
            .filter(|check| !options.filter_by_verbosity || is_visible(check))
            .cloned()
            .collect::<Vec<_>>();
        sorted_checks.sort_by_cached_key(|check| {
            (
                u8::from(&check.severity),
                u8::from(&check.category),
                check.locator.clone().unwrap_or_default().to_lowercase(),
            )
        });
        match (options.quiet, options.format, options.terse) {
            | (false, OutputFormat::Console, true) => sorted_checks.iter().for_each(|check| println!("{}", check.terse())),
            | (false, OutputFormat::Console, false) => sorted_checks
                .into_iter()
                .enumerate()
                .for_each(|(index, check)| check.with_index(index).report()),
            | (false, OutputFormat::Raw, _) if options.verbosity.is_some_and(|level| level > 1) => {
                sorted_checks.iter().for_each(|check| eprintln!("{check}"));
            }
            | _ => {}
        }
    }
}
impl IntoChecks for ConversionWarnings {
    /// Convert crosswalk conversion warnings into Check objects with Warning severity.
    ///
    /// Each ConversionWarning from a field-level loss during schema crosswalk produces
    /// a single Check with the field name as locator and a human-readable message.
    fn to_checks(&self, uri: Option<String>) -> Vec<Check> {
        self.iter()
            .map(|w| {
                let context = format!("{} → {}", w.from, w.to);
                check!(
                    CheckCategory::Crosswalk,
                    false,
                    severity: CheckSeverity::Warning,
                    message: w.to_string(),
                    context: context,
                    locator: w.field.clone(),
                )
                .with_uri(uri.clone())
            })
            .collect()
    }
}
impl CheckSeverity {
    /// Returns whether this severity level is visible at the given log level.
    ///
    /// - [`CheckSeverity::Error`] and [`CheckSeverity::Warning`] are always visible.
    /// - [`CheckSeverity::Suggestion`] is visible at `warn` level and above.
    /// - [`CheckSeverity::Info`] is visible at `info` level and above.
    pub fn is_visible_at(&self, level: Option<u8>) -> bool {
        match self {
            | Self::Error | Self::Warning => true,
            | Self::Suggestion => matches!(level, Some(2..=u8::MAX)),
            | Self::Info => matches!(level, Some(3..=u8::MAX)),
        }
    }
}
impl From<&CheckSeverity> for u8 {
    fn from(value: &CheckSeverity) -> Self {
        match value {
            | CheckSeverity::Error => 0,
            | CheckSeverity::Warning => 1,
            | CheckSeverity::Suggestion => 2,
            | CheckSeverity::Info => 3,
        }
    }
}
impl From<ValeOutputItemSeverity> for CheckSeverity {
    fn from(value: ValeOutputItemSeverity) -> Self {
        match value {
            | ValeOutputItemSeverity::Error => CheckSeverity::Error,
            | ValeOutputItemSeverity::Suggestion => CheckSeverity::Suggestion,
            | ValeOutputItemSeverity::Warning => CheckSeverity::Warning,
        }
    }
}
impl From<CheckSeverity> for Color {
    fn from(value: CheckSeverity) -> Self {
        match value {
            | CheckSeverity::Error => Color::Red,
            | CheckSeverity::Info => Color::Cyan,
            | CheckSeverity::Suggestion => Color::Blue,
            | CheckSeverity::Warning => Color::Yellow,
        }
    }
}
impl From<CheckSeverity> for ReportKind<'_> {
    fn from(value: CheckSeverity) -> Self {
        match value {
            | CheckSeverity::Error => ReportKind::Error,
            | CheckSeverity::Info => ReportKind::Custom("Info", Color::Cyan),
            | CheckSeverity::Suggestion => ReportKind::Custom("Suggestion", Color::Blue),
            | CheckSeverity::Warning => ReportKind::Warning,
        }
    }
}
#[cfg(feature = "analysis")]
impl<'a> IntoRow<'a> for Check {
    fn to_row<Check>(self) -> Row<'a> {
        let Self {
            index,
            category,
            severity,
            message,
            locator,
            uri,
            context,
            ..
        } = self;
        let data = [
            &index.to_string(),
            &severity.to_string(),
            &category.to_string(),
            &uri.unwrap_or_default(),
            &locator.unwrap_or_default(),
            &message,
            &context.unwrap_or_default(),
        ];
        Row::new(data.into_iter().map(|x| AnyValue::String(x).into_static()).collect::<Vec<_>>())
    }
}
impl DocumentIndex {
    fn print(&self, span: &DocumentSpan, report: DocumentReport<'_>) -> bool {
        self.excerpt(span, MAX_LENGTH_REPORT_SPAN_PREFIX).is_some_and(|excerpt| {
            let page = self.page_number(span.0.start);
            let DocumentSpan(span) = excerpt.span;
            let source = Source::from(excerpt.content);
            let message = match page {
                | Some(page) => format!("{} (PDF page {page})", report.message),
                | None => report.message,
            };
            let _ = Report::build(report.kind, (report.title, span.clone()))
                .with_code(report.index)
                .with_config(Config::default().with_compact(false).with_index_type(IndexType::Byte))
                .with_message(report.heading)
                .with_label(ariadne::Label::new((report.title, span)).with_message(message).with_color(report.color))
                .finish()
                .print((report.title, source));
            true
        })
    }
}
impl From<&Map<String, Value>> for Standard {
    fn from(object: &Map<String, Value>) -> Self {
        if object.get("@type").and_then(Value::as_str).is_some_and(|value| value.contains("dcat:")) {
            Standard::Dcat
        } else if object.contains_key("dataset_id") && object.contains_key("metas") {
            Standard::Huwise
        } else if object.contains_key("attributes") && object.contains_key("id") {
            Standard::Datacite
        } else if object.contains_key("metadata") || object.contains_key("pids") || object.contains_key("resource_type") {
            Standard::Invenio
        } else {
            Standard::Dcat
        }
    }
}
impl Standard {
    /// Convert serialized content from one metadata standard to another.
    ///
    /// Parses `content` as the schema identified by `self` (source standard),
    /// converts each record to the `target` standard using `TryFrom` pipeline,
    /// and serializes the result as `output_mime` (JSON or YAML).
    ///
    /// ### Errors
    /// Returns [`CrosswalkError`] when:
    /// - Content cannot be parsed as the source standard
    /// - A required conversion path is not supported
    /// - A record fails conversion
    /// - Output serialization fails
    pub fn crosswalk(&self, content: &str, mime: MimeType, target: Standard, target_mime: MimeType) -> Result<String, CrosswalkError> {
        match *self {
            | Standard::Datacite => OneOrMany::<datacite::Record>::parse(content, mime).and_then(|source| match target {
                | Standard::Datacite => source.serialize(target_mime),
                | Standard::Dcat => source.map(dcat::Dataset::try_from).and_then(|b| b.serialize(target_mime)),
                | Standard::Invenio => source.map(invenio::Record::try_from).and_then(|b| b.serialize(target_mime)),
                | Standard::Huwise => source.map(huwise::Dataset::try_from).and_then(|b| b.serialize(target_mime)),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | Standard::Dcat => OneOrMany::<dcat::Dataset>::parse(content, mime).and_then(|source| match target {
                | Standard::Dcat => source.serialize(target_mime),
                | Standard::Datacite => source.map(datacite::Record::try_from).and_then(|b| b.serialize(target_mime)),
                | Standard::Invenio => source
                    .map(|v| datacite::Record::try_from(v).and_then(invenio::Record::try_from))
                    .and_then(|b| b.serialize(target_mime)),
                | Standard::Huwise => source
                    .map(|v| datacite::Record::try_from(v).and_then(huwise::Dataset::try_from))
                    .and_then(|b| b.serialize(target_mime)),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | Standard::Invenio => OneOrMany::<invenio::Record>::parse(content, mime).and_then(|source| match target {
                | Standard::Invenio => source.serialize(target_mime),
                | Standard::Datacite => source.map(datacite::Record::try_from).and_then(|b| b.serialize(target_mime)),
                | Standard::Dcat => source
                    .map(|v| datacite::Record::try_from(v).and_then(dcat::Dataset::try_from))
                    .and_then(|b| b.serialize(target_mime)),
                | Standard::Huwise => source
                    .map(|v| datacite::Record::try_from(v).and_then(huwise::Dataset::try_from))
                    .and_then(|b| b.serialize(target_mime)),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | Standard::Huwise => OneOrMany::<huwise::Dataset>::parse(content, mime).and_then(|source| match target {
                | Standard::Huwise => source.serialize(target_mime),
                | Standard::Datacite => source.map(datacite::Record::try_from).and_then(|b| b.serialize(target_mime)),
                | Standard::Dcat => source
                    .map(|v| datacite::Record::try_from(v).and_then(dcat::Dataset::try_from))
                    .and_then(|b| b.serialize(target_mime)),
                | Standard::Invenio => source
                    .map(|v| datacite::Record::try_from(v).and_then(invenio::Record::try_from))
                    .and_then(|b| b.serialize(target_mime)),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | _ => Err(self.unsupported_crosswalk(target)),
        }
    }
    /// Convert serialized content from one metadata standard to another, returning both the converted content and any field-level warnings.
    ///
    /// Warnings indicate optional fields in the source that have no equivalent in the target schema, or fields whose semantics differ between schemas.
    pub fn crosswalk_with_warnings(
        &self,
        content: &str,
        mime: MimeType,
        target: Standard,
        target_mime: MimeType,
    ) -> Result<(String, ConversionWarnings), CrosswalkError> {
        let warnings = match self.collect_crosswalk_warnings(content, &mime, target) {
            | Ok(w) => w,
            | Err(_) => Vec::new(),
        };
        self.crosswalk(content, mime, target, target_mime).map(|content| (content, warnings))
    }
    /// Run the FieldMapping pipeline to detect field-level losses for a given pair.
    fn collect_crosswalk_warnings(&self, content: &str, mime: &MimeType, target: Standard) -> Result<ConversionWarnings, CrosswalkError> {
        match (*self, target) {
            | (Standard::Datacite, Standard::Dcat) => {
                process_crosswalk_warnings::<datacite::Record>(content, mime, "datacite", "dcat", datacite_to_dcat())
            }
            | (Standard::Datacite, Standard::Invenio) => {
                process_crosswalk_warnings::<datacite::Record>(content, mime, "datacite", "invenio", datacite_to_invenio())
            }
            | (Standard::Datacite, Standard::Huwise) => {
                process_crosswalk_warnings::<datacite::Record>(content, mime, "datacite", "huwise", datacite_to_huwise())
            }
            | (Standard::Dcat, Standard::Datacite) => {
                process_crosswalk_warnings::<dcat::Dataset>(content, mime, "dcat", "datacite", dcat_to_datacite())
            }
            | (Standard::Invenio, Standard::Datacite) => {
                process_crosswalk_warnings::<invenio::Record>(content, mime, "invenio", "datacite", invenio_to_datacite())
            }
            | (Standard::Huwise, Standard::Datacite) => {
                process_crosswalk_warnings::<huwise::Dataset>(content, mime, "huwise", "datacite", huwise_to_datacite())
            }
            | _ => Ok(Vec::new()),
        }
    }
    fn unsupported_crosswalk(&self, target: Standard) -> CrosswalkError {
        CrosswalkError::BuildFailed(format!(
            "Unsupported metadata crosswalk pair: {self} -> {target} (supported: datacite, dcat, invenio, huwise)"
        ))
    }
    /// Print JSON or YAML schema for this standard to stdout
    #[cfg(feature = "std")]
    pub fn to_schema(&self, format: &str) {
        match *self {
            | Standard::ResearchActivityData => ResearchActivity::to_schema(format),
            | Standard::Raid => Metadata::to_schema(format),
            | Standard::CitationFileFormat => print_schema::<cff::Cff>(format),
            | Standard::Datacite => print_schema::<datacite::Record>(format),
            | Standard::Dcat => print_schema::<dcat::Dataset>(format),
            | Standard::Huwise => print_schema::<huwise::Dataset>(format),
            | Standard::Invenio => print_schema::<invenio::Record>(format),
            | Standard::Document | Standard::Text => print_schema::<text::Text>(format),
            | Standard::DublinCore => eprintln!("Schema generation not supported for Dublin Core (no schema defined)"),
        }
    }
}
#[cfg(feature = "analysis")]
pub(crate) async fn check_prose_for<T>(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check>
where
    T: Analysis + InputOutput + ToProse,
{
    let resolved_options = options.cloned().unwrap_or_default();
    let analyzer = resolved_options
        .resolve_analyzer::<Vale, _>(resolved_options.vale_config.clone().unwrap_or_default())
        .await;
    let sync = match resolved_options.skip_sync {
        | true => Ok(()),
        | false => analyzer.clone().sync(resolved_options.offline, resolved_options.quiet).await,
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
                        | Err(why) => vec![check_err!(CheckCategory::Prose, context: why.to_string()).with_uri(uri)],
                    }
                }
            });
            join_all(results).await.into_iter().flatten().collect()
        }
        | Err(why) => {
            error!("=> {} Vale sync — {why}", Label::fail());
            vec![check_err!(CheckCategory::Prose)]
        }
    }
}
pub(super) fn check_readability_for<T>(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check>
where
    T: InputOutput + ToProse,
{
    let CheckOptions { readability_metric, .. } = options.cloned().unwrap_or_default();
    paths
        .par_iter()
        .flat_map(|path| match T::read(path) {
            | Ok(data) => {
                let content = data.to_prose();
                let calculated_index = readability_metric.calculate(&content);
                let maximum = match readability_metric.maximum_allowed_from_env() {
                    | Some(value) => {
                        debug!(value, "=> {} Maximum allowed readability from .env", Label::using());
                        value
                    }
                    | None => readability_metric.maximum_allowed(),
                };
                debug!(value = calculated_index, "=> {} Readability index", Label::using());
                let score = format!("{} = {calculated_index}/{maximum}", readability_metric.to_string().to_uppercase());
                if calculated_index > maximum {
                    let errors = ErrorKind::Readability((calculated_index, readability_metric));
                    vec![check!(
                        CheckCategory::Readability,
                        false,
                        severity: CheckSeverity::Warning,
                        uri: path.file_name_with_parent(),
                        errors: errors,
                        message: score,
                        data: content
                    )]
                } else {
                    vec![check_ok!(
                        CheckCategory::Readability,
                        uri: path.file_name_with_parent(),
                        message: score,
                        data: content
                    )]
                }
            }
            | Err(why) => vec![check_err!(CheckCategory::Readability, context: why.to_string())],
        })
        .collect::<Vec<Check>>()
}
/// Convert vector of [`Check`] values to a Polars [DataFrame]
#[cfg(feature = "analysis")]
pub fn checks_to_dataframe(values: &[Check]) -> PolarsResult<DataFrame> {
    let names = ["index", "severity", "category", "uri", "locator", "message", "context"];
    to_dataframe::<Check, _, &str>(
        values
            .iter()
            .enumerate()
            .map(|(index, check)| check.clone().with_index(index))
            .collect::<Vec<_>>(),
        names,
    )
}
/// Convert vector of [`Check`] values to a CSV string
#[cfg(feature = "analysis")]
pub fn checks_to_csv(values: &[Check], include_headers: bool) -> PolarsResult<String> {
    match checks_to_dataframe(values) {
        | Ok(mut dataframe) => {
            let mut buf = Cursor::new(Vec::new());
            match CsvWriter::new(&mut buf).include_header(include_headers).finish(&mut dataframe) {
                | Ok(_) => Ok(String::from_utf8_lossy(&buf.into_inner()).into_owned()),
                | Err(why) => Err(why),
            }
        }
        | Err(why) => Err(why),
    }
}
fn print_document_report(document: &DocumentIndex, query: &DocumentQuery, report: DocumentReport<'_>) -> bool {
    match document.resolve(query) {
        | DocumentMatch::Unique(span) => document.print(&span, report),
        | DocumentMatch::Missing | DocumentMatch::Ambiguous => false,
    }
}
fn print_unlocated(index: usize, severity: &CheckSeverity, heading: &str, message: String) {
    let name = severity.to_string().to_case(Case::Title);
    let code = format!("[{index:02}]");
    let colorized_code = severity.colorize(code);
    let colorized_name = severity.colorize(name);
    println!("{colorized_code} {colorized_name}: {heading}");
    println!("    {}", message.italic());
}
/// Print JSON or YAML schema for type `T` to stdout
#[cfg(feature = "std")]
fn print_schema<T: schemars::JsonSchema>(format: &str) {
    let schema = schema_for!(T);
    let output = match format.to_lowercase().as_str() {
        | "yaml" | "yml" => serde_norway::to_string(&schema).unwrap_or_default(),
        | _ => serde_json::to_string_pretty(&schema).unwrap_or_default(),
    };
    println!("{output}");
}
/// Parse source content and apply a FieldMapping to detect missing optional fields.
fn process_crosswalk_warnings<T>(
    content: &str,
    mime: &MimeType,
    from: &'static str,
    to: &'static str,
    mapping: FieldMapping,
) -> Result<ConversionWarnings, CrosswalkError>
where
    T: serde::de::DeserializeOwned + SchemaExtractor,
{
    OneOrMany::<T>::parse(content, mime.clone()).map(|batch| {
        let mapped_fields: Vec<&str> = mapping.rules.iter().map(|r| r.source).collect();
        let missing_warnings = |fields: &Fields, prefix: &str| -> ConversionWarnings {
            let mut target_fields = Fields::new();
            mapping
                .apply(fields, &mut target_fields)
                .unwrap_or_default()
                .into_iter()
                .map(|field| ConversionWarning::no_equivalent(from, to, format!("{prefix}{field}")))
                .collect()
        };
        let unmapped_warnings = |fields: &Fields, prefix: &str| -> ConversionWarnings {
            fields
                .keys()
                .filter(|key| !mapped_fields.contains(&key.as_str()))
                .map(|key| ConversionWarning::no_equivalent(from, to, format!("{prefix}{key}")))
                .collect()
        };
        match batch {
            | OneOrMany::One(record) => {
                let fields = record.extract_fields();
                [missing_warnings(&fields, ""), unmapped_warnings(&fields, "")].concat()
            }
            | OneOrMany::Many(records) => records
                .into_iter()
                .enumerate()
                .flat_map(|(i, record)| {
                    let fields = record.extract_fields();
                    let prefix = format!("record[{i}].");
                    missing_warnings(&fields, &prefix).into_iter().chain(unmapped_warnings(&fields, &prefix))
                })
                .collect(),
        }
    })
}
/// Create summary data table from given issues
pub fn summary(issues: Vec<Check>) -> Vec<Vec<String>> {
    [
        CheckCategory::Schema,
        CheckCategory::Link,
        CheckCategory::Prose,
        CheckCategory::Quality,
        CheckCategory::Readability,
        CheckCategory::Crosswalk,
    ]
    .iter()
    .map(|category| {
        let count = issues
            .iter()
            .filter(|issue| issue.category == *category)
            .map(|issue| issue.issue_count())
            .sum::<usize>()
            .to_string();
        vec![format!("{} items found", category.to_string().to_case(Case::Title)), count]
    })
    .collect::<Vec<_>>()
}
