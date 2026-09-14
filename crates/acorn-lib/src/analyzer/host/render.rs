//! Native terminal rendering for analyzer results.
use crate::analyzer::check::{Check, CheckCategory, CheckKind, CheckSeverity, OutputFormat, Standard};
use crate::analyzer::error::{DocumentTarget, ErrorKind, ValidatorIssues};
use crate::analyzer::host::context::ReportOptions;
use crate::analyzer::model::ValeOutputItem;
use crate::io::document::{DocumentIndex, DocumentMatch, DocumentPath, DocumentPosition, DocumentQuery, DocumentSpan};
use crate::util::constants::MAX_LENGTH_REPORT_SPAN_PREFIX;
use crate::util::Label;
use acorn_core::prelude::{Arc, HashMap};
use acorn_schema::modelkit::kitfile;
use acorn_schema::pid::raid::Metadata;
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::standard::{cff, datacite, dcat, huwise, invenio, text};
use ariadne::{Color, Config, IndexType, Report, ReportKind, Source};
use bon::Builder;
use color_eyre::owo_colors::OwoColorize;
use convert_case::{Case, Casing};
use core::fmt;
use schemars::schema_for;
use tracing::info;

/// Document indexes available to host renderers, keyed by display URI.
#[cfg(any(feature = "analysis", feature = "std"))]
pub type DocumentIndexes = HashMap<String, Arc<DocumentIndex>>;
/// Renders analyzer checks using native terminal facilities.
pub trait Render {
    /// Render checks in deterministic detailed or compact form.
    fn render(&self, context: RenderContext<'_>);
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
/// Host rendering inputs kept separate from portable check values.
#[cfg(any(feature = "analysis", feature = "std"))]
#[derive(Clone, Copy, Debug)]
pub struct RenderContext<'a> {
    /// Source indexes used for precise diagnostics.
    pub documents: Option<&'a DocumentIndexes>,
    /// Output and failure-reporting policy.
    pub options: &'a ReportOptions,
}
impl Render for [Check] {
    fn render(&self, context: RenderContext<'_>) {
        let RenderContext { documents, options } = context;
        let is_visible = Check::is_visible_at(options.extension.verbosity);
        let mut sorted_checks = self
            .iter()
            .filter(|check| !options.extension.filter_by_verbosity || is_visible(check))
            .cloned()
            .collect::<Vec<_>>();
        sorted_checks.sort_by_cached_key(|check| {
            let fair = check.fair_details();
            (
                u8::from(&check.severity),
                u8::from(&check.category),
                fair.map(|finding| finding.resource.to_lowercase()).unwrap_or_default(),
                fair.map(|finding| finding.indicator.id),
                check.locator.clone().unwrap_or_default().to_lowercase(),
            )
        });
        if !options.common.quiet && options.extension.format == OutputFormat::Console && !sorted_checks.is_empty() {
            println!();
        }
        match (options.common.quiet, options.extension.format, options.extension.terse) {
            | (false, OutputFormat::Console, true) => sorted_checks
                .iter()
                .for_each(|check| println!("{}", check.terse(check.document_from(documents)))),
            | (false, OutputFormat::Console, false) => sorted_checks.into_iter().enumerate().for_each(|(index, check)| {
                let document = check.document_from(documents);
                check.with_index(index).report(document);
            }),
            | (false, OutputFormat::Raw, _) if options.extension.verbosity.is_some_and(|level| level > 1) => {
                sorted_checks.iter().for_each(|check| eprintln!("{check}"));
            }
            | _ => {}
        }
    }
}
impl Check {
    #[cfg(any(feature = "analysis", feature = "std"))]
    fn document_from<'a>(&self, documents: Option<&'a DocumentIndexes>) -> Option<&'a DocumentIndex> {
        self.uri
            .as_ref()
            .and_then(|uri| documents.and_then(|documents| documents.get(uri)))
            .map(Arc::as_ref)
    }
    /// Fancy output in contrast with more compressed output of Display impl
    #[cfg(any(feature = "analysis", feature = "std"))]
    pub fn report(&self, document: Option<&DocumentIndex>) {
        let index = self.index;
        let Check {
            category,
            locator,
            context,
            data,
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
                    let rendered = document.is_some_and(|document| document.print(&query, report));
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
                            .zip(document.and_then(|document| self.prose_span(document, item)))
                            .is_some_and(|(document, span)| document.print_span(&span, report));
                        if !rendered {
                            print_unlocated(index.saturating_add(i), &CheckSeverity::from(severity.clone()), check, message.clone());
                        }
                    });
                }
                | None | Some(_) => {}
            },
            | CheckCategory::Fair => match self.fair_details() {
                | Some(finding) => {
                    let title = uri.as_deref().unwrap_or(finding.resource.as_str());
                    let heading: String = finding.indicator.clone().into();
                    let source_text = finding.evidence.as_deref().unwrap_or(&finding.remediation);
                    let query = match finding.locator.as_deref() {
                        | Some(locator) => DocumentQuery::new().with_path(DocumentPath::parse(locator)).with_needle(source_text),
                        | None => DocumentQuery::new().with_needle(source_text),
                    };
                    let report = DocumentReport::init()
                        .title(title)
                        .index(index)
                        .kind(severity.clone().into())
                        .heading(&heading)
                        .message(finding.remediation.italic().to_string())
                        .color(severity.clone().into())
                        .build();
                    let rendered = document.is_some_and(|document| document.print(&query, report));
                    if !rendered {
                        print_unlocated(index, &severity, &heading, finding.remediation.clone());
                    }
                }
                | None => {
                    let heading = uri.as_deref().map_or_else(
                        || category.message().to_string(),
                        |uri| format!("{} — {}", category.message(), uri.cyan().underline()),
                    );
                    print_unlocated(index, &severity, &heading, context.as_deref().unwrap_or(&message).to_string());
                }
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
                | Some(ErrorKind::Readability(_)) => {
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
                                let rendered = document.is_some_and(|document| document.print(&query, report));
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
    #[cfg(not(any(feature = "analysis", feature = "std")))]
    pub fn report(&self) {
        // Portable fallback without document rendering or Vale/HTTP
        tracing::info!(category = %self.category, uri = ?self.uri, message = %self.message, "check report");
    }
    #[cfg(any(feature = "analysis", feature = "std"))]
    pub(crate) fn resolve_locator(self, document: &DocumentIndex) -> Self {
        let prose_locators = match (&self.category, &self.errors) {
            | (CheckCategory::Prose, Some(ErrorKind::Vale(values))) => values.iter().map(|item| self.prose_locator(document, item)).collect(),
            | _ => self.prose_locators.clone(),
        };
        let locator = prose_locators.first().cloned().flatten().or_else(|| self.locator.clone());
        Self {
            locator,
            prose_locators,
            ..self
        }
    }
    #[cfg(any(feature = "analysis", feature = "std"))]
    fn prose_locator(&self, document: &DocumentIndex, item: &ValeOutputItem) -> Option<String> {
        self.prose_position(document, item)
            .map(|position| format!("Line {}, Character {}", position.line, position.column))
    }
    #[cfg(any(feature = "analysis", feature = "std"))]
    fn prose_position(&self, document: &DocumentIndex, item: &ValeOutputItem) -> Option<DocumentPosition> {
        self.prose_span(document, item).and_then(|span| document.position(span.0.start))
    }
    #[cfg(any(feature = "analysis", feature = "std"))]
    fn prose_span(&self, document: &DocumentIndex, item: &ValeOutputItem) -> Option<DocumentSpan> {
        match self.data.as_deref() == Some(document.document().content.as_str()) {
            | true => document.span_from_line_columns(item.line, &item.span),
            | false => match document.resolve(&item.query()) {
                | DocumentMatch::Unique(span) => Some(span),
                | DocumentMatch::Missing | DocumentMatch::Ambiguous => None,
            },
        }
    }
    #[cfg(any(feature = "analysis", feature = "std"))]
    pub(crate) fn terse(&self, document: Option<&DocumentIndex>) -> String {
        let uri = self.uri.as_deref().unwrap_or_default();
        let positions = document.map_or_else(Vec::new, |document| match (&self.category, &self.kind, &self.errors) {
            | (CheckCategory::Link, CheckKind::Standard, _) => {
                let context = self.context.as_deref().map(str::trim).unwrap_or("No URL");
                let text = if context.is_empty() { " " } else { context };
                let query = match self.locator.as_deref() {
                    | Some(locator) => DocumentQuery::new().with_path(DocumentPath::parse(locator)).with_needle(text),
                    | None => DocumentQuery::new().with_needle(text),
                };
                vec![document.locate(&query)]
            }
            | (CheckCategory::Prose, _, Some(ErrorKind::Vale(values))) => values.iter().map(|item| self.prose_position(document, item)).collect(),
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
            | CheckCategory::Fair => match self.fair_details() {
                | Some(finding) => {
                    let location = finding
                        .locator
                        .clone()
                        .filter(|value| !value.is_empty())
                        .unwrap_or_else(|| finding.resource.clone());
                    let metadata = finding.metadata();
                    let details = format!("{} {}", finding.remediation, metadata.dimmed());
                    f.write_str(&format_row(location, self.severity.colored(), details))
                }
                | None => {
                    let location = self.uri.clone().or_else(|| self.locator.clone()).unwrap_or_default();
                    f.write_str(&format_row(location, self.severity.colored(), self.message.clone()))
                }
            },
            | CheckCategory::Prose => {
                #[cfg(any(feature = "analysis", feature = "std"))]
                {
                    match &self.errors {
                        | Some(ErrorKind::Vale(values)) => {
                            let last = values.len().saturating_sub(1);
                            values.iter().enumerate().try_for_each(|(index, item)| {
                                let ValeOutputItem {
                                    check, message, severity, ..
                                } = item;
                                let location = self
                                    .prose_locators
                                    .get(index)
                                    .cloned()
                                    .flatten()
                                    .or_else(|| self.locator.clone())
                                    .unwrap_or_else(|| "Unlocated".to_string());
                                let severity = CheckSeverity::from(severity.clone());
                                let details = format!("{} {}{}", message, "rule=".dimmed(), check.dimmed());
                                f.write_fmt(format_args!("{}", format_row(location, severity.colored(), details)))
                                    .and_then(|_| if index < last { writeln!(f) } else { Ok(()) })
                            })
                        }
                        | None | Some(_) => write!(f, ""),
                    }
                }
                #[cfg(not(any(feature = "analysis", feature = "std")))]
                {
                    let details = self.message.clone();
                    f.write_str(&format_row(self.locator.clone().unwrap_or_default(), self.severity.colored(), details))
                }
            }
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
                    | Some(ErrorKind::Readability(_)) => {
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
impl DocumentIndex {
    fn print(&self, query: &DocumentQuery, report: DocumentReport<'_>) -> bool {
        match self.resolve(query) {
            | DocumentMatch::Unique(span) => self.print_span(&span, report),
            | DocumentMatch::Missing | DocumentMatch::Ambiguous => false,
        }
    }
    fn print_span(&self, span: &DocumentSpan, report: DocumentReport<'_>) -> bool {
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
#[cfg(any(feature = "analysis", feature = "std"))]
impl<'a> RenderContext<'a> {
    /// Create a rendering context without source indexes.
    pub fn new(options: &'a ReportOptions) -> Self {
        Self { documents: None, options }
    }
    /// Create a rendering context with source indexes.
    pub fn with_documents(options: &'a ReportOptions, documents: &'a DocumentIndexes) -> Self {
        Self {
            documents: Some(documents),
            options,
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
impl Standard {
    /// Print JSON or YAML schema for this standard to stdout
    #[cfg(any(feature = "analysis", feature = "std"))]
    pub fn to_schema(&self, format: &str) {
        match *self {
            | Standard::ResearchActivityData => ResearchActivity::to_schema(format),
            | Standard::Raid => Metadata::to_schema(format),
            | Standard::CitationFileFormat => print_schema::<cff::Cff>(format),
            | Standard::Datacite => print_schema::<datacite::Record>(format),
            | Standard::Dcat => print_schema::<dcat::Dataset>(format),
            | Standard::Huwise => print_schema::<huwise::Dataset>(format),
            | Standard::Invenio => print_schema::<invenio::Record>(format),
            | Standard::Kitfile | Standard::ModelKit => print_schema::<kitfile::Kitfile>(format),
            | Standard::Document | Standard::Text => print_schema::<text::Text>(format),
            | Standard::DublinCore => eprintln!("Schema generation not supported for Dublin Core (no schema defined)"),
        }
    }
}
/// Print JSON or YAML schema for type `T` to stdout
#[cfg(any(feature = "analysis", feature = "std"))]
fn print_schema<T: schemars::JsonSchema>(format: &str) {
    let schema = schema_for!(T);
    let output = match format.to_lowercase().as_str() {
        | "yaml" | "yml" => serde_norway::to_string(&schema).unwrap_or_default(),
        | _ => serde_json::to_string_pretty(&schema).unwrap_or_default(),
    };
    println!("{output}");
}
fn print_unlocated(index: usize, severity: &CheckSeverity, heading: &str, message: String) {
    let name = severity.to_string().to_case(Case::Title);
    let code = format!("[{index:02}]");
    let colorized_code = severity.colorize(code);
    let colorized_name = severity.colorize(name);
    println!("{colorized_code} {colorized_name}: {heading}");
    println!("    {}\n", message.trim_end().italic());
}
