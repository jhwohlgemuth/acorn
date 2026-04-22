//! Check result types and helpers for analyzer output.

use super::error::{process, ErrorCode, ErrorKind, Location};
use super::readability::ReadabilityType;
use super::vale::{Vale, ValeConfig, ValeOutputItem, ValeOutputItemSeverity};
#[cfg(feature = "analysis")]
use super::{to_dataframe, StaticAnalyzer, StaticAnalyzerConfig};
use crate::io::InputOutput;
#[cfg(feature = "analysis")]
use crate::prelude::Cursor;
use crate::prelude::{HashMap, Path, PathBuf};
use crate::util::{Label, StringConversion, ToProse};
use crate::{check, check_err, check_ok};
use ariadne::{Color, Config, Report, ReportKind, Source};
use async_trait::async_trait;
use bon::Builder;
use color_eyre::owo_colors::OwoColorize;
use convert_case::{Case, Casing};
use core::fmt;
use core::ops::RangeInclusive;
use derive_more::Display;
use futures::future::join_all;
#[cfg(feature = "analysis")]
use polars::{
    frame::row::Row,
    io::csv::write::CsvWriter,
    prelude::{AnyValue, DataFrame, PolarsResult, SerWriter},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;
use tracing::{debug, error, info};
use validator::ValidationErrorsKind;

/// Trait for adding analysis capabilities
#[cfg(feature = "analysis")]
#[async_trait]
pub trait Analysis {
    /// Run analysis for a given category
    async fn check(category: CheckCategory, paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        match category {
            | CheckCategory::Link => Self::check_websites(paths, options).await,
            | CheckCategory::Prose => Self::check_prose(paths, options).await,
            | CheckCategory::Quality => Self::check_quality(paths, options),
            | CheckCategory::Readability => Self::check_readability(paths, options),
            | CheckCategory::Schema => Self::check_schema(paths),
        }
    }
    /// Perform analysis of prose
    async fn check_prose(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check>;
    /// Perform quality checks and verify data consistency
    fn check_quality(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check>;
    /// Calculate readability using a given metric
    fn check_readability(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check>;
    /// Execute validation checks for a given item
    fn check_schema(paths: &[PathBuf]) -> Vec<Check>;
    /// Check if URLs links are valid and readable
    async fn check_websites(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check>;
    /// Returns the metadata standard represented by the implementing type.
    fn standard() -> Standard;
    /// Returns the output path used for Vale analysis artifacts.
    fn prose_output_path(path: &Path, data: &Self) -> PathBuf;
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
    /// Dublin Core
    #[display("dublin-core")]
    DublinCore,
    /// HUBwise
    #[display("huwise")]
    Huwise,
    /// InvenioRDM
    #[display("invenio")]
    Invenio,
    /// Plain text
    #[display("text")]
    Text,
}
/// Various check categories available for validating research activity data
#[derive(Clone, Debug, Display, PartialEq)]
pub enum CheckCategory {
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
    /// Schema validation check
    #[display("schema")]
    Schema,
}
/// Severity level of a check result
#[derive(Clone, Debug, Default, Display, PartialEq)]
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
    /// Categories of checks to skip during processing
    #[builder(default)]
    pub skip: Vec<String>,
    /// Readability metric to use for readability checks
    pub readability_metric: ReadabilityType,
    /// Whether or not to skip checksum verification for prose analysis
    #[builder(default = false)]
    pub skip_verify_checksum: bool,
    /// Use compact output format instead of detailed output
    #[builder(default = false)]
    pub terse: bool,
    /// Vale configuration options
    pub vale_config: Option<ValeConfig>,
}
impl Check {
    /// Recursively count validation errors from a given ValidationErrorsKind
    pub fn count_validation_errors(kind: &ValidationErrorsKind) -> usize {
        match kind {
            | ValidationErrorsKind::Field(_) => 1,
            | ValidationErrorsKind::List(errors) => errors
                .clone()
                .into_values()
                .map(|error| Self::count_validation_errors(&ValidationErrorsKind::Struct(error)))
                .sum(),
            | ValidationErrorsKind::Struct(errors) => errors
                .clone()
                .into_errors()
                .into_values()
                .map(|nested| Self::count_validation_errors(&nested))
                .sum(),
        }
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
                        | ErrorKind::Validator(kind) => Self::count_validation_errors(kind),
                        | _ => 0,
                    }
                } else {
                    0
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
            errors,
            severity,
            status_code,
            uri,
            message,
            ..
        } = self.clone();
        match &category {
            | CheckCategory::Link => {
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
                let source_text = if text.is_empty() { " ".to_string() } else { text.clone() };
                let source = Source::from(source_text.clone());
                let span = 0..=source_text.len().saturating_sub(1);
                let _ = Report::build(kind, (&title, 1..=1))
                    .with_code(index)
                    .with_config(Config::default().with_compact(false))
                    .with_message("Website")
                    .with_label(
                        ariadne::Label::new((&title, span))
                            .with_message(format!("{}{code}", message.italic()))
                            .with_color(severity.clone().into()),
                    )
                    .finish()
                    .print((&title, source));
            }
            | CheckCategory::Prose => match &errors {
                | Some(ErrorKind::Vale(values)) => {
                    let title = uri.clone().unwrap_or("index.json".to_string());
                    if let Some(text) = &data {
                        values.iter().enumerate().for_each(|(i, item)| {
                            let ValeOutputItem {
                                check,
                                line,
                                message,
                                severity,
                                span,
                                ..
                            } = item;
                            let source = Source::from(text.clone());
                            let _ = Report::build(severity.into(), (&title, 1..=1))
                                .with_code(index + i)
                                .with_config(Config::default().with_compact(false))
                                .with_message(check)
                                .with_label(
                                    ariadne::Label::new((&title, highlighted_span(&source, span, *line)))
                                        .with_message(message.italic())
                                        .with_color(severity.into()),
                                )
                                .finish()
                                .print((&title, source));
                        });
                    }
                }
                | None | Some(_) => {
                    let message = format!("=> {} {} has {}", Label::pass(), message.underline(), "no prose issues".green());
                    info!("{}", message);
                }
            },
            | CheckCategory::Quality => unimplemented!(),
            | CheckCategory::Readability => match &errors {
                | Some(ErrorKind::Readability((_readability_index, readability_type))) => {
                    let kind: ReportKind<'static> = severity.clone().into();
                    let title = uri.as_deref().unwrap_or_default();
                    let metric = readability_type.to_string().to_uppercase();
                    let source_text = format!("Prose was analyzed for readability using the {metric} metric ({message})");
                    let source = Source::from(source_text.clone());
                    let start = source_text.find(&message).unwrap_or_default();
                    let end = start + message.len().saturating_sub(1);
                    let _ = Report::build(kind, (&title, 1..=1))
                        .with_code(index)
                        .with_config(Config::default().with_compact(false))
                        .with_message("Readability")
                        .with_label(
                            ariadne::Label::new((&title, start..=end))
                                .with_message("Simplify language for a general audience".to_string().italic())
                                .with_color(severity.clone().into()),
                        )
                        .finish()
                        .print((&title, source));
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
                    | Some(ErrorKind::Validator(validator_kind)) => {
                        let prefix = context.as_deref().or(locator.as_deref()).unwrap_or(&message);
                        process(prefix, validator_kind).iter().enumerate().for_each(|(issue_index, issue)| {
                            let locator = issue.locator();
                            let source_text = issue.source_text();
                            let source = Source::from(source_text.clone());
                            let span = match highlighted_span_from_params(&source, &issue.code, &issue.params) {
                                | Some(span) => span,
                                | None => 0..=source_text.len().saturating_sub(1),
                            };
                            let _ = Report::build(severity.clone().into(), (&title, 1..=1))
                                .with_code(index + issue_index)
                                .with_config(Config::default().with_compact(false))
                                .with_message(locator)
                                .with_label(
                                    ariadne::Label::new((&title, span))
                                        .with_message(issue.message.italic())
                                        .with_color(severity.clone().into()),
                                )
                                .finish()
                                .print((&title, source));
                        });
                    }
                    | None | Some(_) => {
                        info!("=> {} {title} has {}", Label::pass(), "no schema validation issues".green().bold());
                    }
                }
            }
        }
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
    /// Returns a new LinkCheckResult with the given index
    pub fn with_index(self, value: usize) -> Self {
        Self { index: value, ..self }
    }
    /// Returns a new LinkCheckResult with the given locator
    pub fn with_locator(self, value: Option<String>) -> Self {
        Self { locator: value, ..self }
    }
    /// Returns a new LinkCheckResult with the given URL
    pub fn with_uri(self, value: String) -> Self {
        Self { uri: Some(value), ..self }
    }
}
impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const INDENT: &str = " ";
        const COL_ONE_WIDTH: usize = 25;
        const COL_TWO_WIDTH: usize = 29;
        let format_row = |first: String, second: String, third: String| format!("{INDENT}{first:<COL_ONE_WIDTH$} {second:<COL_TWO_WIDTH$} {third}");
        match self.category {
            | CheckCategory::Link => {
                let Check {
                    severity,
                    context,
                    status_code,
                    message,
                    ..
                } = self;
                let code = match status_code.clone() {
                    | Some(value) if !value.is_empty() => format!(" ({value})").dimmed().to_string(),
                    | None | Some(_) => "".to_string(),
                };
                let context = context
                    .as_ref()
                    .map(|value| value.to_string().underline().italic().to_string())
                    .unwrap_or_else(|| "Missing".to_string());
                let severity = severity.colored();
                let details = format!("{context} {}{code}", message.dimmed());
                write!(f, "{}", format_row("".to_string(), severity, details))
            }
            | CheckCategory::Prose => match &self.errors {
                | Some(ErrorKind::Vale(values)) => {
                    let last = values.len().saturating_sub(1);
                    values.iter().enumerate().try_for_each(|(index, item)| {
                        let ValeOutputItem {
                            check, message, severity, ..
                        } = item;
                        let location = item.locator();
                        let severity = CheckSeverity::from(severity.clone());
                        let details = format!("{} {}{}", message, "rule=".dimmed(), check.dimmed());
                        f.write_fmt(format_args!("{}", format_row(location, severity.colored(), details)))
                            .and_then(|_| if index < last { writeln!(f) } else { Ok(()) })
                    })
                }
                | None | Some(_) => write!(f, ""),
            },
            | CheckCategory::Quality => unimplemented!(),
            | CheckCategory::Readability => {
                let Check {
                    errors, message, severity, ..
                } = self;
                match &errors {
                    | Some(ErrorKind::Readability((_readability_index, _readability_type))) => {
                        let details = format!("Simplify language for a general audience {}", message.dimmed());
                        f.write_fmt(format_args!("{}", format_row("".to_string(), severity.colored(), details)))
                    }
                    | None | Some(_) => write!(f, ""),
                }
            }
            | CheckCategory::Schema => match &self.errors {
                | Some(ErrorKind::Validator(kind)) => {
                    let Check {
                        context,
                        locator,
                        message,
                        severity,
                        ..
                    } = self;
                    let prefix = context.as_deref().or(locator.as_deref()).unwrap_or(message.as_str());
                    let rows = process(prefix, kind);
                    let last = rows.len().saturating_sub(1);
                    rows.iter().enumerate().try_for_each(|(index, issue)| {
                        let location = issue.locator();
                        let details = format!("{} {}{}", issue.message, "code=".dimmed(), issue.code.to_string().to_uppercase().dimmed());
                        f.write_fmt(format_args!("{}", format_row(location, severity.colored(), details)))
                            .and_then(|_| if index < last { writeln!(f) } else { Ok(()) })
                    })
                }
                | None | Some(_) => write!(f, ""),
            },
        }
    }
}
impl CheckSeverity {
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
impl From<ValeOutputItemSeverity> for CheckSeverity {
    fn from(value: ValeOutputItemSeverity) -> Self {
        match value {
            | ValeOutputItemSeverity::Error => CheckSeverity::Error,
            | ValeOutputItemSeverity::Suggestion => CheckSeverity::Suggestion,
            | ValeOutputItemSeverity::Warning => CheckSeverity::Warning,
        }
    }
}
impl From<String> for CheckCategory {
    fn from(value: String) -> Self {
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
            | "schema" => Self::Schema,
            | _ => panic!("Unsupported check category: {value}"),
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
pub(crate) async fn check_prose_for<T>(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check>
where
    T: Analysis + InputOutput + ToProse,
{
    let CheckOptions {
        offline,
        quiet,
        skip_verify_checksum,
        vale_config,
        ..
    } = options.cloned().unwrap_or_default();
    let config = vale_config.unwrap_or_default().save().await;
    let vale = Vale::resolve(config, offline, skip_verify_checksum).await;
    match vale.clone().sync(offline, quiet).await {
        | Ok(_) => {
            let results = paths.iter().map(|path| {
                let vale = vale.clone();
                let path = path.clone();
                async move {
                    match T::read(&path) {
                        | Ok(data) => {
                            let output = T::prose_output_path(&path, &data);
                            let content = data.to_prose();
                            vale.run(output, content, Some("JSON".into())).await
                        }
                        | Err(why) => {
                            error!("=> {} Read {} data — {why}", Label::fail(), T::standard());
                            vec![check_err!(CheckCategory::Prose)]
                        }
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
        .map(|path| match T::read(path) {
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
                    check!(
                        CheckCategory::Readability,
                        false,
                        severity: CheckSeverity::Warning,
                        uri: path.file_name_with_parent(),
                        errors: errors,
                        message: score,
                        data: content
                    )
                } else {
                    check_ok!(
                        CheckCategory::Readability,
                        uri: path.file_name_with_parent(),
                        message: score,
                        data: content
                    )
                }
            }
            | Err(_why) => check_err!(CheckCategory::Readability),
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
/// Calculate a highlighted byte span for a single line in a source report.
pub fn highlighted_span(source: &Source, span: &[u32], line_number: u32) -> RangeInclusive<usize> {
    let selected = (line_number as usize) - 1;
    let begin = source.lines().take(selected).fold((span[0] - 1) as usize, |acc, line| acc + line.len());
    let character_count = (span[1] as usize) - (span[0] as usize);
    let end = begin + character_count;
    begin..=end
}
/// Calculate a highlighted span for an indexed value in an array-like source.
pub fn highlighted_span_for_array(source: &Source, params: &HashMap<String, Value>) -> Option<RangeInclusive<usize>> {
    match params.get("index") {
        | Some(Value::Number(index)) => match index.as_u64() {
            | Some(index) => match params.get("value") {
                | Some(Value::Array(value)) => {
                    const PADDING: u64 = 2;
                    let end = (value[index as usize].to_string().len() + 1) as u32;
                    let span = &[4, end];
                    let line = (index + PADDING) as u32;
                    Some(highlighted_span(source, span, line))
                }
                | Some(_) | None => None,
            },
            | None => None,
        },
        | None | Some(_) => None,
    }
}
/// Select a highlighted span based on validator error code and parameters.
pub fn highlighted_span_from_params(source: &Source, code: &ErrorCode, params: &HashMap<String, Value>) -> Option<RangeInclusive<usize>> {
    match code {
        | ErrorCode::Latitude => {
            // TODO: Implement highlighting for latitude errors
            None
        }
        | ErrorCode::Length => match params.get("max") {
            | Some(Value::Number(max)) => match max.as_u64() {
                | Some(max) => Some(max as usize..=(source.text().len() - 2)),
                | None => None,
            },
            | None | Some(_) => None,
        },
        | ErrorCode::Longitude => {
            // TODO: Implement highlighting for longitude errors
            None
        }
        | ErrorCode::Section => match highlighted_span_for_array(source, params) {
            | Some(span) => Some(span),
            | None => None,
        },
        | _ if params.get("index").is_some() => match highlighted_span_for_array(source, params) {
            | Some(span) => Some(span),
            | None => None,
        },
        | _ => None,
    }
}
/// Create summary data table from given issues
pub fn summary(issues: Vec<Check>) -> Vec<Vec<String>> {
    [
        CheckCategory::Schema,
        CheckCategory::Link,
        CheckCategory::Prose,
        CheckCategory::Readability,
    ]
    .iter()
    .map(|category| {
        let count = issues
            .iter()
            .filter(|issue| issue.category == *category)
            .map(|issue| issue.issue_count())
            .sum::<usize>()
            .to_string();
        vec![format!("{} issues found", category.to_string().to_case(Case::Title)), count]
    })
    .collect::<Vec<_>>()
}
