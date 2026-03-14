//! Check result types and helpers for analyzer output.

use super::error::{process, ErrorCode, ErrorKind};
#[cfg(feature = "analysis")]
use super::to_dataframe;
use super::vale::{ValeOutputItem, ValeOutputItemSeverity};
use ariadne::{Color, Config, Report, ReportKind, Source};
use bon::Builder;
use color_eyre::owo_colors::OwoColorize;
use core::fmt;
use core::ops::RangeInclusive;
use derive_more::Display;
#[cfg(feature = "analysis")]
use polars::frame::row::Row;
#[cfg(feature = "analysis")]
use polars::prelude::{AnyValue, DataFrame, PolarsResult};
use serde_json::Value;
use tracing::info;
use validator::ValidationErrorsKind;

use crate::prelude::HashMap;
use crate::util::{Label, ToMarkdown};

/// Trait for converting to a ([Polars]) row
///
/// [Polars]: https://docs.rs/polars/latest/polars/
#[cfg(feature = "analysis")]
pub trait IntoRow<'a> {
    /// Convert to a (Polars) row
    fn to_row<T>(self) -> Row<'a>;
}
/// Various check categories available for validating research activity data
#[derive(Clone, Debug, Display, PartialEq)]
pub enum CheckCategory {
    /// Website avaialability check
    #[display("link")]
    Link,
    /// Static analysis of prose
    #[display("prose")]
    Prose,
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
    /// Check category
    pub category: CheckCategory,
    /// Textual context of check (e.g., paragraph where prose issues were found)
    pub context: Option<String>,
    /// Whether or not the check was successful
    #[builder(default = false)]
    pub success: bool,
    /// Severity of the check result
    #[builder(default)]
    pub severity: CheckSeverity,
    /// HTTP status code
    pub status_code: Option<String>,
    /// Errors and issues found during check
    pub errors: Option<ErrorKind>,
    /// Path of file being validated
    pub uri: Option<String>,
    /// Message related to or description of validation issue (e.g., key name of invalid value, result of validation, etc.)
    #[builder(default = "".to_string())]
    pub message: String,
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
    /// Returns the number of errors
    pub fn issue_count(&self) -> usize {
        match self.category {
            | CheckCategory::Link => 1,
            | CheckCategory::Prose => {
                if let Some(kind) = &self.errors {
                    match kind {
                        | ErrorKind::Vale(values) => values.len(),
                        | _ => 0,
                    }
                } else {
                    0
                }
            }
            | CheckCategory::Readability => 1,
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
    pub fn report(self) {
        match self.category {
            | CheckCategory::Link => {
                let Check {
                    context,
                    message,
                    severity,
                    success,
                    uri,
                    status_code,
                    ..
                } = self.clone();
                let title = context.unwrap_or_else(|| "index.json".to_string());
                let kind: ReportKind<'static> = severity.clone().into();
                let code = match status_code.clone() {
                    | Some(value) if !value.is_empty() => format!(" ({value})").dimmed().to_string(),
                    | None | Some(_) => "".to_string(),
                };
                if !success {
                    let text = match uri {
                        | Some(uri) => uri.trim().to_string(),
                        | None => "No URL".to_string(),
                    };
                    let span = 0..=(text.len() - 1);
                    let source = Source::from(text.clone());
                    let _ = Report::build(kind, (&title, 1..=1))
                        .with_config(Config::default().with_compact(false))
                        .with_message("Website")
                        .with_label(
                            ariadne::Label::new((&title, span))
                                .with_message(format!("{}{code}", message.italic()))
                                .with_color(severity.into()),
                        )
                        .finish()
                        .print((&title, source));
                }
            }
            | CheckCategory::Prose => {
                let Check {
                    context, errors, message, ..
                } = self;
                match &errors {
                    | Some(ErrorKind::Vale(values)) => {
                        let title = self.uri.unwrap_or("index.json".to_string());
                        if let Some(text) = &context {
                            values.iter().enumerate().for_each(|(index, item)| {
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
                                    .with_code(index)
                                    .with_config(Config::default().with_compact(false))
                                    .with_message(check)
                                    .with_label(
                                        ariadne::Label::new((&title, highlighted_span(source.clone(), span, *line)))
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
                }
            }
            | CheckCategory::Readability => {
                let Check {
                    context,
                    errors,
                    message,
                    severity,
                    ..
                } = self;
                match errors {
                    | Some(ErrorKind::Readability((index, readability_type))) => {
                        let kind: ReportKind<'static> = severity.clone().into();
                        let title = message.clone();
                        let threshold = context.unwrap_or_default();
                        let metric = readability_type.to_string().to_uppercase();
                        let source_text =
                            format!("Text was analyzed for readability using the {metric} metric. It has a score of {index} which is > {threshold}.");
                        let source = Source::from(source_text.clone());
                        let index_text = index.to_string();
                        let start = source_text.find(index_text.as_str()).unwrap_or_default();
                        let end = start + index_text.len().saturating_sub(1);
                        let _ = Report::build(kind, (&title, 1..=1))
                            .with_config(Config::default().with_compact(false))
                            .with_message("Readability")
                            .with_label(
                                ariadne::Label::new((&title, start..=end))
                                    .with_message("Simplify language for a general audience".to_string().italic())
                                    .with_color(severity.into()),
                            )
                            .finish()
                            .print((&title, source));
                    }
                    | None | Some(_) => {
                        if let Some(value) = context {
                            info!(
                                "=> {} {} has {} {}",
                                Label::pass(),
                                message,
                                "no readability issues".green().bold(),
                                value.dimmed()
                            );
                        } else {
                            info!("=> {} {} has {}", Label::pass(), message, "no readability issues".green().bold(),);
                        }
                    }
                }
            }
            | CheckCategory::Schema => {
                let Check { errors, message: prefix, .. } = self.clone();
                let severity = self.severity.clone();
                let report_kind: ReportKind<'static> = severity.clone().into();
                let uri = self.clone().uri.unwrap();
                let title = uri;
                match &errors {
                    | Some(ErrorKind::Validator(validator_kind)) => {
                        process(&prefix, validator_kind.clone())
                            .iter()
                            .enumerate()
                            .for_each(|(index, (code, path, message, params))| {
                                let title_message = match path {
                                    | Some(value) => match params.get("index") {
                                        | Some(Value::Number(index)) => format!("{}[{}]", value, index),
                                        | None | Some(_) => value.to_string(),
                                    },
                                    | None => code.to_string().to_uppercase(),
                                };
                                let text = match params.get("value") {
                                    | Some(Value::Array(items)) => items.iter().map(|item| item.to_string()).collect::<Vec<_>>().to_markdown(),
                                    | Some(value) => value.to_string(),
                                    | None => "".to_string(),
                                };
                                let source = Source::from(text);
                                let span = match highlighted_span_from_params(source.clone(), code.clone(), params.clone()) {
                                    | Some(span) => span,
                                    | None => 0..=(source.text().len() - 1),
                                };
                                let _ = Report::build(report_kind, (&title, 1..=1))
                                    .with_code(index)
                                    .with_config(Config::default().with_compact(false))
                                    .with_message(title_message)
                                    .with_label(
                                        ariadne::Label::new((&title, span))
                                            .with_message(message.italic())
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
        Check::init()
            .category(self.category)
            .context(value)
            .success(self.success)
            .severity(self.severity)
            .maybe_uri(self.uri)
            .message(self.message)
            .maybe_status_code(self.status_code)
            .maybe_errors(self.errors)
            .build()
    }
    /// Returns a new LinkCheckResult with the given URL
    pub fn with_uri(self, value: String) -> Self {
        Check::init()
            .category(self.category)
            .success(self.success)
            .severity(self.severity)
            .uri(value)
            .message(self.message)
            .maybe_status_code(self.status_code)
            .maybe_errors(self.errors)
            .build()
    }
}
impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const INDENT: &str = " ";
        const COL_ONE_WIDTH: usize = 25;
        const COL_TWO_WIDTH: usize = 29;
        let format_row = |first: String, second: String, third: String| format!("{INDENT}{first:<COL_ONE_WIDTH$} {second:<COL_TWO_WIDTH$} {third}");
        let severity_colored = |severity: &CheckSeverity| match severity {
            | CheckSeverity::Error => severity.to_string().red().bold().to_string(),
            | CheckSeverity::Suggestion => severity.to_string().blue().bold().to_string(),
            | CheckSeverity::Warning => severity.to_string().yellow().bold().to_string(),
        };
        match self.category {
            | CheckCategory::Link => {
                let Check {
                    severity,
                    uri,
                    status_code,
                    message,
                    ..
                } = self;
                let code = match status_code.clone() {
                    | Some(value) if !value.is_empty() => format!(" ({value})").dimmed().to_string(),
                    | None | Some(_) => "".to_string(),
                };
                let uri = uri
                    .as_ref()
                    .map(|value| value.to_string().underline().italic().to_string())
                    .unwrap_or_else(|| "Missing".to_string());
                let severity = severity_colored(severity);
                let details = format!("{uri} {}{code}", message.dimmed());
                write!(f, "{}", format_row("".to_string(), severity, details))
            }
            | CheckCategory::Prose => match &self.errors {
                | Some(ErrorKind::Vale(values)) => {
                    let last = values.len().saturating_sub(1);
                    values.iter().enumerate().try_for_each(|(index, item)| {
                        let ValeOutputItem {
                            check,
                            line,
                            message,
                            severity,
                            span,
                            ..
                        } = item;
                        let location = format!("Line {}, Character {}", line, span[0]);
                        let severity = CheckSeverity::from(severity.clone());
                        let details = format!("{} {}", message, check.dimmed());
                        f.write_fmt(format_args!("{}", format_row(location, severity_colored(&severity), details)))
                            .and_then(|_| if index < last { writeln!(f) } else { Ok(()) })
                    })
                }
                | None | Some(_) => write!(f, ""),
            },
            | CheckCategory::Readability => {
                let Check {
                    context,
                    errors,
                    message: path,
                    ..
                } = self;
                match &errors {
                    | Some(ErrorKind::Readability(values)) => {
                        let (index, readability_type) = values;
                        let rtype = readability_type.to_string().to_uppercase().dimmed().to_string();
                        let max = context.as_deref().unwrap_or("").cyan().to_string();
                        let severity = severity_colored(&self.severity);
                        let details = format!(
                            "Simplify language for a general audience ({} should be < {max}) {}{rtype} {}{}",
                            index.to_string().red().bold(),
                            "type=".dimmed(),
                            "path=".dimmed(),
                            path.dimmed(),
                        );
                        f.write_fmt(format_args!("{}", format_row("".to_string(), severity, details)))
                    }
                    | None | Some(_) => write!(f, ""),
                }
            }
            | CheckCategory::Schema => match &self.errors {
                | Some(ErrorKind::Validator(kind)) => {
                    let Check { message: prefix, .. } = self.clone();
                    let severity = severity_colored(&self.severity);
                    let rows = process(&prefix, kind.clone());
                    let last = rows.len().saturating_sub(1);
                    rows.iter().enumerate().try_for_each(|(index, (code, path, message, _params))| {
                        let field = path.clone().unwrap_or_default();
                        let details = format!("{message} {}{}", "code=".dimmed(), code.to_string().to_uppercase().dimmed());
                        f.write_fmt(format_args!("{}", format_row(field, severity.clone(), details)))
                            .and_then(|_| if index < last { writeln!(f) } else { Ok(()) })
                    })
                }
                | None | Some(_) => write!(f, ""),
            },
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
            | CheckSeverity::Suggestion => Color::Blue,
            | CheckSeverity::Warning => Color::Yellow,
        }
    }
}
impl From<CheckSeverity> for ReportKind<'_> {
    fn from(value: CheckSeverity) -> Self {
        match value {
            | CheckSeverity::Error => ReportKind::Error,
            | CheckSeverity::Suggestion => ReportKind::Custom("Suggestion", Color::Blue),
            | CheckSeverity::Warning => ReportKind::Warning,
        }
    }
}
#[cfg(feature = "analysis")]
impl<'a> IntoRow<'a> for Check {
    fn to_row<Check>(self) -> Row<'a> {
        let Self {
            success,
            category,
            message,
            uri,
            status_code,
            context,
            ..
        } = self;
        let data = [
            if success { "pass" } else { "fail" },
            &category.to_string(),
            &message,
            &uri.unwrap_or_default(),
            &status_code.unwrap_or_default(),
            &context.unwrap_or_default(),
        ];
        Row::new(data.into_iter().map(|x| AnyValue::String(x).into_static()).collect::<Vec<_>>())
    }
}
/// Convert vector of [`Check`] values to a Polars [DataFrame]
#[cfg(feature = "analysis")]
pub fn checks_to_dataframe(values: Vec<Check>) -> PolarsResult<DataFrame> {
    let names = ["success", "category", "message", "uri", "status_code", "context"];
    to_dataframe::<Check, _, &str>(values, names)
}
/// Calculate a highlighted byte span for a single line in a source report.
pub fn highlighted_span(source: Source, span: &[u32], line_number: u32) -> RangeInclusive<usize> {
    let selected = (line_number as usize) - 1;
    let begin = source.lines().take(selected).fold((span[0] - 1) as usize, |acc, line| acc + line.len());
    let character_count = (span[1] as usize) - (span[0] as usize);
    let end = begin + character_count;
    begin..=end
}
/// Calculate a highlighted span for an indexed value in an array-like source.
pub fn highlighted_span_for_array(source: Source, params: HashMap<String, Value>) -> Option<RangeInclusive<usize>> {
    match params.get("index") {
        | Some(Value::Number(index)) => match index.as_u64() {
            | Some(index) => match params.get("value") {
                | Some(Value::Array(value)) => {
                    const PADDING: u64 = 2;
                    let end = (value[index as usize].to_string().len() + 1) as u32;
                    let span = &[4, end];
                    let line = (index + PADDING) as u32;
                    Some(highlighted_span(source.clone(), span, line))
                }
                | Some(_) | None => None,
            },
            | None => None,
        },
        | None | Some(_) => None,
    }
}
/// Select a highlighted span based on validator error code and parameters.
pub fn highlighted_span_from_params(source: Source, code: ErrorCode, params: HashMap<String, Value>) -> Option<RangeInclusive<usize>> {
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
        | ErrorCode::Section => match highlighted_span_for_array(source.clone(), params.clone()) {
            | Some(span) => Some(span),
            | None => None,
        },
        | _ if params.get("index").is_some() => match highlighted_span_for_array(source.clone(), params.clone()) {
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
        vec![category.to_string(), count]
    })
    .collect::<Vec<_>>()
}
