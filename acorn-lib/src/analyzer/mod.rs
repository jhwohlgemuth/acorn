//! # Prose analyzer module
//!
//! This is where we keep functions and interfaces necessary to execute ACORN's automated editorial style guide as well as content readability analyzer.
//!
use crate::analyzer::vale::{ValeOutput, ValeOutputItem};
use crate::io::{command_exists, download_binary, extract_zip, file_checksum, make_executable, standard_project_folder};
#[cfg(feature = "std")]
use crate::io::{get, InputOutput};
use crate::prelude::{self, create_dir_all, remove_file, Command, CommandOutput, Error, File, HashMap, PathBuf, Write};
use crate::schema::research_activity::ResearchActivity;
use crate::schema::{Organization, ProgrammingLanguage, Website};
use crate::util::constants::{
    APPLICATION, CUSTOM_VALE_PACKAGE_NAME, DEFAULT_VALE_PACKAGE_URL, DEFAULT_VALE_ROOT, DISABLED_VALE_RULES, ENABLED_VALE_PACKAGES, ORGANIZATION,
    VALE_RELEASES_URL, VALE_VERSION,
};
use crate::util::{Constant, Label, SemanticVersion, StringConversion, ToMarkdown};
use crate::{cmd, skip};
use crate::{Location, Repository};
use ariadne::{Config, Report, Source};
use async_trait::async_trait;
use bat::PrettyPrinter;
use bon::Builder;
use color_eyre::owo_colors::OwoColorize;
use convert_case::{Case, Casing};
use core::fmt;
use core::ops::RangeInclusive;
use derive_more::Display;
use flate2::read::GzDecoder;
use futures::future::join_all;
use ini::Ini;
use lychee_lib::{CacheStatus, Response, Status};
use polars::datatypes::PlSmallStr;
use polars::frame::row::Row;
use polars::prelude::{AnyValue, DataFrame, PolarsResult};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;
use tar::Archive;
use tracing::{debug, error, info, trace};
use validator::Validate;
use validator::ValidationErrorsKind;
use which::which;

pub mod error;
pub mod readability;
pub mod vale;

use error::{process, ErrorCode, ErrorKind};
use readability::ReadabilityType;
use vale::{Vale, ValeConfig};

/// Trait for adding analysis capabilities
#[async_trait]
pub trait Analysis {
    /// Perform analysis of prose
    async fn prose_analysis(paths: Vec<PathBuf>, is_offline: bool, skip_verify_checksum: bool) -> Vec<Check>;
    /// Calculate readability using a given metric
    fn readability_analysis<R>(paths: Vec<PathBuf>, readability_type: R) -> Vec<Check>
    where
        R: Into<ReadabilityType>;
    /// Check if URLs links are valid and readable
    async fn check_websites(paths: Vec<PathBuf>, is_offline: bool) -> Vec<Check>;
}
/// Trait for converting to a ([Polars]) row
///
/// [Polars]: https://docs.rs/polars/latest/polars/
pub trait IntoRow<'a> {
    /// Convert to a (Polars) row
    fn to_row<T>(self) -> Row<'a>;
}
/// Trait for static analyzers (e.g. Vale)
#[async_trait]
pub trait StaticAnalyzer<Config: StaticAnalyzerConfig> {
    /// Get command name (e.g. "vale")
    fn command(self) -> String;
    /// Download binary
    async fn download(self, config: Option<Config>, skip_verify_checksum: bool) -> Self;
    /// Download checksum values
    async fn download_checksums(self) -> Result<HashMap<String, String>, String>;
    /// Extract binary
    fn extract(self, path: PathBuf, destination: Option<PathBuf>) -> PathBuf;
    /// Resolve analyzer
    async fn resolve(_config: Config, _is_offline: bool, _skip_verify_checksum: bool) -> Self;
    /// Run analyzer on content
    async fn run(&self, id: String, content: String, output: Option<String>) -> Check;
    /// Perform sync operation (only applies to Vale)
    async fn sync(self, is_offline: bool) -> Result<(), Error>;
    /// Set binary
    fn with_binary<P>(self, path: P) -> Self
    where
        P: Into<PathBuf>;
    /// Set config
    fn with_config(self, value: Config) -> Self;
    /// Set system command
    fn with_system_command(self) -> Self;
    /// Set version
    fn with_version(self, value: String) -> Self;
}
/// Trait for static analyzer configuration (e.g. .vale.ini)
#[async_trait]
pub trait StaticAnalyzerConfig {
    /// Get default configuration
    fn default() -> Self;
    /// Convert to INI
    async fn ini(self) -> Ini;
    /// Save configuration
    async fn save(self) -> Self;
    /// Set parent path of configuration
    fn with_path(self, path: PathBuf) -> Self;
}
/// Trait for schema validation
pub trait Validation {
    /// Execute validation checks for a given item
    fn check(paths: Vec<PathBuf>) -> Vec<Check>;
    /// Get validation issues for a given item
    fn validation_issues(self) -> Vec<Check>;
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
#[async_trait]
impl Analysis for ResearchActivity {
    /// Checks links associated with research activity (DOIs, websites, contact URL) for availability and readability
    #[cfg(feature = "std")]
    async fn check_websites(paths: Vec<PathBuf>, is_offline: bool) -> Vec<Check> {
        let futures = paths.into_iter().map(|path| async move {
            match ResearchActivity::read(&path) {
                | Ok(data) => {
                    let issues = if is_offline {
                        vec![]
                    } else {
                        let dois = match data.clone().meta.doi {
                            | Some(values) => values.into_iter().map(|doi| format!("https://doi.org/{doi}")).collect(),
                            | None => vec![],
                        };
                        let websites = match data.clone().meta.websites {
                            | Some(values) => values.into_iter().map(|Website { url, .. }| url).collect(),
                            | None => vec![],
                        };
                        let links: Vec<String> = dois.into_iter().chain(websites).chain([data.clone().contact.url]).collect();
                        let link_futures = links.into_iter().map(|url| link_check(Some(url)));
                        join_all(link_futures).await
                    };
                    issues.into_iter().map(|check| check.with_context(path.file_name_with_parent())).collect()
                }
                | Err(why) => {
                    error!("=> {} Read research activity data at {} - {why}", Label::fail(), path.display());
                    vec![Check::init().category(CheckCategory::Schema).success(false).build()]
                }
            }
        });
        join_all(futures).await.into_iter().flatten().collect()
    }
    async fn prose_analysis(paths: Vec<PathBuf>, is_offline: bool, skip_verify_checksum: bool) -> Vec<Check> {
        // Create and save Vale config asynchronously
        let config = ValeConfig::default().save().await;
        let vale = Vale::resolve(config, is_offline, skip_verify_checksum).await;
        match vale.clone().sync(is_offline).await {
            | Ok(_) => {
                let results = paths.into_iter().map(|path| {
                    let vale = vale.clone();
                    async move {
                        match ResearchActivity::read(&path) {
                            | Ok(data) => {
                                let identifier = data.clone().meta.identifier.to_lowercase();
                                vale.run(identifier, data.to_markdown(), Some("JSON".into())).await
                            }
                            | Err(why) => {
                                error!("=> {} Read research activity data - {why}", Label::fail());
                                Check::init().category(CheckCategory::Prose).success(false).build()
                            }
                        }
                    }
                });
                join_all(results).await
            }
            | Err(why) => {
                error!("=> {} Vale sync - {why}", Label::fail());
                vec![Check::init().category(CheckCategory::Prose).success(false).build()]
            }
        }
    }
    fn readability_analysis<R>(paths: Vec<PathBuf>, readability_type: R) -> Vec<Check>
    where
        R: Into<ReadabilityType>,
    {
        let rtype = readability_type.into();
        paths
            .par_iter()
            .map(|path| match ResearchActivity::read(path) {
                | Ok(data) => {
                    let index = rtype.calculate(&data.to_markdown());
                    let maximum = match rtype.maximum_allowed_from_env() {
                        | Some(value) => {
                            debug!(value, "=> {} Maximum allowed readability from .env", Label::using());
                            value
                        }
                        | None => rtype.maximum_allowed(),
                    };
                    debug!(value = index, "=> {} Readability index", Label::using());
                    if index > maximum {
                        let errors = ErrorKind::Readability((index, rtype));
                        Check::init()
                            .category(CheckCategory::Readability)
                            .success(false)
                            .message(path.file_name_with_parent())
                            .errors(errors)
                            .context(maximum.to_string())
                            .build()
                    } else {
                        let score = format!("({} = {}/{})", rtype.to_string().to_uppercase(), index, maximum);
                        Check::init()
                            .category(CheckCategory::Readability)
                            .success(true)
                            .message(path.file_name_with_parent())
                            .context(score)
                            .build()
                    }
                }
                | Err(why) => {
                    error!("=> {} Read research activity data - {why}", Label::fail());
                    Check::init().category(CheckCategory::Readability).success(false).build()
                }
            })
            .collect::<Vec<Check>>()
    }
}
impl Check {
    /// Recursively count validation errors from a given ValidationErrorsKind
    pub fn count_validation_errors(kind: &ValidationErrorsKind) -> usize {
        match kind {
            | ValidationErrorsKind::Field(_) => 1,
            | ValidationErrorsKind::Struct(errors) => errors
                .clone()
                .into_errors()
                .into_values()
                .map(|nested| Self::count_validation_errors(&nested))
                .sum(),
            | ValidationErrorsKind::List(errors) => errors
                .clone()
                .into_values()
                .map(|error| Self::count_validation_errors(&ValidationErrorsKind::Struct(error)))
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
                    success,
                    uri,
                    status_code,
                    ..
                } = self.clone();
                let title = context.unwrap_or_else(|| "index.json".to_string());
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
                    let _ = Report::build(ariadne::ReportKind::Error, (&title, 1..=1))
                        .with_config(Config::default().with_compact(false))
                        .with_message("Website")
                        .with_label(
                            ariadne::Label::new((&title, span))
                                .with_message(format!("{}{code}", message.italic()))
                                .with_color(ariadne::Color::Red),
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
                    context, errors, message, ..
                } = self;
                match errors {
                    | Some(ErrorKind::Readability((index, readability_type))) => {
                        let title = message.clone();
                        let threshold = context.unwrap_or_default();
                        let metric = readability_type.to_string().to_uppercase();
                        let source_text =
                            format!("Text was analyzed for readability using the {metric} metric. It has a score of {index} which is > {threshold}.");
                        let source = Source::from(source_text.clone());
                        let index_text = index.to_string();
                        let start = source_text.find(index_text.as_str()).unwrap_or_default();
                        let end = start + index_text.len().saturating_sub(1);
                        let _ = Report::build(ariadne::ReportKind::Error, (&title, 1..=1))
                            .with_config(Config::default().with_compact(false))
                            .with_message("Readability")
                            .with_label(
                                ariadne::Label::new((&title, start..=end))
                                    .with_message("Simplify language for a general audience".to_string().italic())
                                    .with_color(ariadne::Color::Red),
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
                let uri = self.clone().uri.unwrap();
                let title = uri;
                match &errors {
                    | Some(ErrorKind::Validator(kind)) => {
                        process(&prefix, kind.clone())
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
                                let _ = Report::build(ariadne::ReportKind::Error, (&title, 1..=1))
                                    .with_code(index)
                                    .with_config(Config::default().with_compact(false))
                                    .with_message(title_message)
                                    .with_label(
                                        ariadne::Label::new((&title, span))
                                            .with_message(message.italic())
                                            .with_color(ariadne::Color::Red),
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
            .uri(value)
            .message(self.message)
            .maybe_status_code(self.status_code)
            .maybe_errors(self.errors)
            .build()
    }
}
impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.category {
            | CheckCategory::Link => {
                let Check {
                    uri, status_code, message, ..
                } = self;
                let code = match status_code.clone() {
                    | Some(value) if !value.is_empty() => format!(" ({value})").dimmed().to_string(),
                    | None | Some(_) => "".to_string(),
                };
                let uri = uri
                    .as_ref()
                    .map(|value| value.to_string().underline().italic().to_string())
                    .unwrap_or_else(|| "Missing".to_string());
                let status = if self.success {
                    "valid".green().to_string()
                } else {
                    "invalid".red().to_string()
                };
                let indent = " ".repeat(28);
                write!(f, "{indent}{:<20} {uri} {}{code}", status.bold(), message.dimmed())
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
                        f.write_fmt(format_args!(
                            "  {:<25} {:<20} {} {}",
                            location,
                            severity.colored(),
                            message,
                            check.dimmed()
                        ))
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
                        let indent = " ".repeat(28);
                        f.write_fmt(format_args!(
                            "{indent}{:<10} Simplify language for a general audience ({} should be < {max}) {}{rtype} {}{}",
                            "error".red().bold(),
                            index.to_string().red().bold(),
                            "type=".dimmed(),
                            "path=".dimmed(),
                            path.dimmed(),
                        ))
                    }
                    | None | Some(_) => write!(f, ""),
                }
            }
            | CheckCategory::Schema => match &self.errors {
                | Some(ErrorKind::Validator(kind)) => {
                    let Check { message: prefix, .. } = self.clone();
                    let rows = process(&prefix, kind.clone());
                    let last = rows.len().saturating_sub(1);
                    rows.iter().enumerate().try_for_each(|(index, (code, path, message, _params))| {
                        let field = path.clone().unwrap_or_default();
                        f.write_fmt(format_args!(
                            "  {field:<25} {:<10} {message} {}{}",
                            "error".red().bold(),
                            "code=".dimmed(),
                            code.to_string().to_uppercase().dimmed(),
                        ))
                        .and_then(|_| if index < last { writeln!(f) } else { Ok(()) })
                    })
                }
                | None | Some(_) => write!(f, ""),
            },
        }
    }
}
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
#[async_trait]
impl StaticAnalyzer<ValeConfig> for Vale {
    fn command(self) -> String {
        "vale".to_string()
    }
    /// Resolve Vale
    /// ### Notes
    /// - Will use system `vale` command if available
    /// - Will use local `vale` binary if available (will expect local binary if offline)
    /// - Will download `vale` binary if not available by other means
    async fn resolve(config: ValeConfig, is_offline: bool, skip_verify_checksum: bool) -> Vale {
        fn any_exist<S>(paths: Vec<S>) -> bool
        where
            S: Into<PathBuf>,
        {
            paths.into_iter().any(|s| s.into().exists())
        }
        let root = DEFAULT_VALE_ROOT;
        let name = "vale";
        let init = Vale::init().build();
        let vale = if command_exists(name) {
            let init_with_config = init.with_config(config);
            init_with_config.with_system_command()
        } else if is_offline || any_exist(vec![format!("{root}{name}"), format!("{root}{name}.exe")]) {
            info!("=> {} Local {} binary", Label::using(), name.green().bold());
            #[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
            {
                init.with_config(config).with_binary(format!("{root}{name}"))
            }
            #[cfg(windows)]
            {
                init.with_config(config).with_binary(format!("{root}{name}.exe"))
            }
        } else {
            init.download(Some(config), skip_verify_checksum).await
        };
        vale
    }
    async fn run(&self, id: String, content: String, output: Option<String>) -> Check {
        let root = standard_project_folder("check", None).join(&id);
        match create_dir_all(root.clone()) {
            | Ok(_) => {}
            | Err(why) => error!(path = root.clone().to_absolute_string(), "=> {} Create - {why}", Label::fail()),
        }
        let path = root.join("index.json");
        let mut file = match File::create(&path) {
            | Ok(file) => file,
            | Err(why) => panic!("=> {} Create file {} - {why}", Label::fail(), path.display()),
        };
        file.write_all(content.as_bytes())
            .expect("Unable to write to cache directory project file");
        let binary = match &self.binary {
            | Some(value) => value,
            | None => {
                error!("=> {} {} binary", Label::not_found(), self.clone().command());
                &PathBuf::from("./.vale/vale")
            }
        };
        match &self.config {
            | Some(config) => {
                let result = match output {
                    | Some(value) => cmd!(
                        binary.to_absolute_string(),
                        [
                            "--no-wrap",
                            "--config",
                            &config.clone().path.to_absolute_string(),
                            "--output",
                            value.as_str(),
                            &path.clone().to_absolute_string(),
                            "--ext",
                            ".md",
                            "--no-exit"
                        ]
                    ),
                    | None => cmd!(
                        binary.to_absolute_string(),
                        [
                            "--no-wrap",
                            "--config",
                            &config.clone().path.to_absolute_string(),
                            &path.clone().to_absolute_string(),
                            "--ext",
                            ".md",
                            "--no-exit"
                        ]
                    ),
                };
                match result {
                    | Ok(output) if output.status.success() => {
                        let parsed = ValeOutput::parse(&output.stdout(), path.clone());
                        if parsed.is_empty() {
                            Check::init().category(CheckCategory::Prose).success(true).message(id).build()
                        } else {
                            Check::init()
                                .category(CheckCategory::Prose)
                                .success(false)
                                .message(id)
                                .errors(ErrorKind::Vale(parsed))
                                .context(content)
                                .uri(path.file_name_with_parent())
                                .build()
                        }
                    }
                    | Ok(output) => {
                        let why = output.stderr();
                        let message = if why.is_empty() {
                            format!("process exited with status {}", output.status)
                        } else {
                            why
                        };
                        error!("=> {} Analyze - {}", Label::fail(), message);
                        Check::init().category(CheckCategory::Prose).success(false).message(id).build()
                    }
                    | Err(why) => {
                        error!("=> {} Analyze - {}", Label::fail(), why);
                        Check::init().category(CheckCategory::Prose).success(false).message(id).build()
                    }
                }
            }
            | None => {
                let title = self.clone().command().to_case(Case::Title);
                error!("=> {} {} configuration", Label::not_found(), title);
                Check::init().category(CheckCategory::Prose).success(false).message(id).build()
            }
        }
    }
    async fn download(self, config: Option<ValeConfig>, skip_verify_checksum: bool) -> Vale {
        let platform = prelude::vale_release_filename();
        let release = match self.version {
            | Some(value) => value,
            | None => SemanticVersion::from(VALE_VERSION),
        };
        let url = format!("{VALE_RELEASES_URL}/download/v{release}/{}_{release}_{platform}", self.clone().command());
        info!(url, "=> {} Vale release v{release}", Label::using());
        let binary = match download_binary(&url, ".").await {
            | Ok(path) => {
                if !skip_verify_checksum {
                    let dowloaded_checksum = match self.clone().download_checksums().await {
                        | Ok(value) => value.get(&platform).unwrap().to_string(),
                        | Err(_) => "".to_string(),
                    };
                    if let Some(calculated) = file_checksum(path.clone(), None) {
                        if !dowloaded_checksum.eq(&calculated) {
                            error!(dowloaded_checksum, calculated, "=> {}", Label::invalid());
                            let _cleanup = remove_file(path.clone());
                        } else {
                            info!(checksum = dowloaded_checksum, "=> {} Checksum verification", Label::pass());
                        }
                    };
                } else {
                    skip!("Checksum verification");
                }
                let destination = match config.clone() {
                    | Some(value) => value.path.parent().unwrap().to_path_buf(),
                    | None => PathBuf::from("./.vale/"),
                };
                let binary = self.clone().extract(path.clone(), Some(destination));
                if make_executable(&binary) {
                    let _cleanup = remove_file(path);
                    Some(binary)
                } else {
                    error!("=> {} {} not executable", Label::fail(), self.command());
                    None
                }
            }
            | Err(error) => {
                error!(error, url, "=> {} {} download", Label::fail(), self.command());
                None
            }
        };
        let builder = Vale::init().version(release).maybe_binary(binary);
        match config {
            | Some(value) => builder.config(value).build(),
            | None => {
                let config = ValeConfig::default();
                builder.config(config).build()
            }
        }
    }
    async fn download_checksums(self) -> Result<HashMap<String, String>, String> {
        let release = match self.version {
            | Some(value) => value,
            | None => SemanticVersion::from(VALE_VERSION),
        };
        let url = format!(
            "{VALE_RELEASES_URL}/download/v{release}/{}_{release}_checksums.txt",
            self.clone().command()
        );
        let response = get(url).send().await.unwrap();
        let content = response.text().await.unwrap();
        let checksums = content.lines().clone().fold(HashMap::new(), |mut acc: HashMap<String, String>, line| {
            let mut values = line.split("  ").collect::<Vec<&str>>();
            let key = values.pop().unwrap()["vale_#.#.#_".len()..].to_string();
            let value = values.pop().unwrap().to_string();
            acc.insert(key, value);
            acc
        });
        debug!(
            "=> {} {} checksums {:#?}",
            Label::using(),
            self.command().to_case(Case::Title),
            checksums.dimmed().cyan()
        );
        Ok(checksums)
    }
    fn extract(self, path: PathBuf, destination: Option<PathBuf>) -> PathBuf {
        let command = self.clone().command();
        let parent = match destination {
            | Some(value) => value.to_absolute_string(),
            | None => format!("./.{command}/"),
        };
        let extension = path.extension().unwrap_or_default().to_str().unwrap_or_default().to_string();
        match extension.as_str() {
            | "zip" => match extract_zip(path, Some(parent.into())) {
                | Ok(value) => {
                    let path = value.join(command);
                    if cfg!(windows) {
                        path.with_extension("exe")
                    } else {
                        path
                    }
                }
                | Err(why) => {
                    error!("=> {} {command} extract - {why}", Label::fail());
                    let path = PathBuf::from(DEFAULT_VALE_ROOT).join(command);
                    if cfg!(windows) {
                        path.with_extension("exe")
                    } else {
                        path
                    }
                }
            },
            | "gz" => {
                let tar_gz = File::open(path).unwrap();
                let tar = GzDecoder::new(tar_gz);
                let mut archive = Archive::new(tar);
                let message = format!("Unable to extract {command} binary");
                archive.unpack(parent.clone()).expect(&message);
                debug!(parent, "=> {} Extracted {command} binary", Label::using());
                PathBuf::from(format!("{parent}/{command}"))
            }
            | _ => {
                error!("=> {} {command} extract - Unsupported format", Label::fail());
                PathBuf::from(DEFAULT_VALE_ROOT).join(command)
            }
        }
    }
    async fn sync(self, is_offline: bool) -> Result<(), Error> {
        let command = self.clone().command();
        let path = match self.binary {
            | Some(value) => value,
            | None => {
                error!("=> {} {} binary", Label::not_found(), command);
                PathBuf::from(DEFAULT_VALE_ROOT).join(command)
            }
        };
        let config_path = self.config.unwrap().path;
        let result: Result<(), Error> = if is_offline {
            skip!("Vale sync");
            Ok(())
        } else {
            match Command::new(path.clone()).arg("--config").arg(config_path.clone()).arg("sync").status() {
                | Ok(status) if status.success() => Ok(()),
                | Ok(status) => Err(Error::other(format!("Vale sync failed with status: {status}"))),
                | Err(why) => Err(why),
            }
        };
        match result {
            | Ok(_) => {
                let parent = format!("{}/styles/config/vocabularies/{}", config_path.parent().unwrap().display(), APPLICATION);
                debug!(parent, "=> {} Vocabularies", Label::using());
                match create_dir_all(parent.clone()) {
                    | Ok(_) => {}
                    | Err(why) => error!(directory = parent, "=> {} Create - {why}", Label::fail()),
                }
                match File::create(format!("{parent}/accept.txt")) {
                    | Ok(mut file) => {
                        let acronyms = Constant::last_values("acronyms");
                        let partners = Constant::last_values("partners");
                        let sponsors = Constant::last_values("sponsors");
                        let abbreviations = Organization::alternative_names();
                        let words = Constant::read_lines("accept.txt");
                        let content = acronyms
                            .chain(partners)
                            .chain(sponsors)
                            .chain(abbreviations)
                            .chain(words)
                            .collect::<Vec<String>>()
                            .join("\n");
                        file.write_all(content.as_bytes()).expect("Unable to write to accept.txt");
                    }
                    | Err(why) => panic!("=> {} Create accept.txt - {why}", Label::fail()),
                }
                match File::create(format!("{parent}/reject.txt")) {
                    | Ok(mut file) => {
                        let content = Constant::read_lines("reject.txt").join("\n");
                        file.write_all(content.as_bytes()).expect("Unable to write to reject.txt");
                    }
                    | Err(why) => panic!("=> {} Create reject.txt - {why}", Label::fail()),
                }
                Ok(())
            }
            | Err(why) => {
                error!(config = config_path.to_absolute_string(), "=> {} Vale sync - {why}", Label::fail());
                Err(why)
            }
        }
    }
    fn with_binary<P>(mut self, path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        self.binary = Some(path.into());
        self
    }
    fn with_config(mut self, value: ValeConfig) -> Self {
        self.config = Some(value);
        self
    }
    fn with_system_command(mut self) -> Self {
        let name = self.clone().command();
        if command_exists(name.clone()) {
            let path = which(name.clone()).unwrap().to_path_buf();
            self.binary = Some(path.clone());
            let offset = "vale version ".len();
            let output = cmd!(name.clone(), ["--version"]).unwrap();
            let stdout = output.stdout();
            let version = stdout[offset..].trim().to_string();
            self.version = Some(SemanticVersion::from(version.as_ref()));
            debug!(
                path = path.to_absolute_string(),
                "=> {} System {} (v{}) command",
                Label::using(),
                name.green().bold(),
                version
            );
        }
        self
    }
    fn with_version(mut self, value: String) -> Self {
        self.version = Some(SemanticVersion::from(value.as_ref()));
        self
    }
}
#[async_trait]
impl StaticAnalyzerConfig for ValeConfig {
    fn default() -> Self {
        fn to_string(values: Vec<&str>) -> Vec<String> {
            values.iter().map(|s| s.to_string()).collect()
        }
        let config = ValeConfig::init()
            .packages(to_string(ENABLED_VALE_PACKAGES.to_vec()))
            .vocabularies(to_string(vec![&ORGANIZATION.to_uppercase(), APPLICATION]))
            .disabled(to_string(DISABLED_VALE_RULES.to_vec()))
            .build();
        trace!("=> {} Default - {:#?}", Label::using(), config.dimmed().cyan());
        config
    }
    async fn ini(self) -> Ini {
        let ValeConfig {
            packages,
            vocabularies,
            disabled,
            ..
        } = self;
        let mut conf = Ini::new();
        let package_repository = Repository::GitLab {
            id: None,
            location: Location::Simple("https://code.ornl.gov/research-enablement/vale-package".to_string()),
        };
        let package_url = match package_repository.latest_release().await {
            | Some(release) => {
                let tag = release.tag_name;
                format!("https://code.ornl.gov/research-enablement/vale-package/-/archive/{tag}/vale-package-{tag}.zip")
            }
            | None => DEFAULT_VALE_PACKAGE_URL.to_string(),
        };
        // CAUTION: Order of attributes in INI file matter. "StylesPath" must come before "Vocab"
        conf.with_section::<String>(None)
            .set("StylesPath", "styles")
            .set("Vocab", vocabularies.join(", "))
            .set("Packages", format!("{}, {}", packages.join(", "), package_url));
        conf.with_section(Some("*"))
            .set("BasedOnStyles", format!("Vale, {}, {}", CUSTOM_VALE_PACKAGE_NAME, packages.join(", ")));
        disabled.iter().for_each(|rule| {
            conf.with_section(Some("*")).set(rule, "NO");
        });
        conf
    }
    async fn save(self) -> ValeConfig {
        let path = self.clone().path;
        let parent = path.parent().unwrap().to_path_buf();
        match create_dir_all(parent.clone()) {
            | Ok(_) => {}
            | Err(why) => error!(directory = parent.to_absolute_string(), "=> {} Create - {why}", Label::fail()),
        }
        match self.clone().ini().await.write_to_file(path.clone()) {
            | Ok(_) => {
                debug!(path = path.to_absolute_string(), "=> {} Saved configuration", Label::using());
            }
            | Err(why) => {
                error!("=> {} Save configuration - {why}", Label::fail());
            }
        }
        self
    }
    fn with_path(mut self, path: PathBuf) -> Self {
        self.path = path;
        self
    }
}
impl Validation for ResearchActivity {
    /// Checks a list of research activity files
    fn check(paths: Vec<PathBuf>) -> Vec<Check> {
        paths
            .par_iter()
            .flat_map(|path| match ResearchActivity::read(path) {
                | Ok(data) => {
                    let uri = path.file_name_with_parent();
                    data.clone()
                        .validation_issues()
                        .into_iter()
                        .map(|issue| issue.with_uri(uri.clone()))
                        .collect::<Vec<_>>()
                }
                | Err(why) => {
                    error!("=> {} Read research activity data at {} - {why}", Label::fail(), path.display());
                    vec![Check::init().category(CheckCategory::Schema).success(false).build()]
                }
            })
            .collect()
    }
    fn validation_issues(self) -> Vec<Check> {
        self.validate()
            .err()
            .into_iter()
            .map(|err| {
                Check::init()
                    .category(CheckCategory::Schema)
                    .success(false)
                    .errors(ErrorKind::Validator(ValidationErrorsKind::Struct(Box::new(err))))
                    .build()
            })
            .collect()
    }
}
pub(crate) fn highlighted_span(source: Source, span: &[u32], line_number: u32) -> RangeInclusive<usize> {
    let selected = (line_number as usize) - 1;
    let begin = source.lines().take(selected).fold((span[0] - 1) as usize, |acc, line| acc + line.len());
    let character_count = (span[1] as usize) - (span[0] as usize);
    let end = begin + character_count;
    begin..=end
}
pub(crate) fn highlighted_span_for_array(source: Source, params: HashMap<String, Value>) -> Option<RangeInclusive<usize>> {
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
pub(crate) fn highlighted_span_from_params(source: Source, code: ErrorCode, params: HashMap<String, Value>) -> Option<RangeInclusive<usize>> {
    match code {
        | ErrorCode::Length => match params.get("max") {
            | Some(Value::Number(max)) => match max.as_u64() {
                | Some(max) => Some(max as usize..=(source.text().len() - 2)),
                | None => None,
            },
            | None | Some(_) => None,
        },
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
/// Convert Lychee response to [`Check`]
pub fn convert_lychee_response(value: Response) -> Check {
    let body = value.body().to_string();
    let url = value.source().to_string();
    debug!(url, "=> {} Response: {:#?}", Label::using(), body.dimmed().cyan());
    let status_code = value.status().code().map(|c| c.to_string()).unwrap_or_default();
    let check = match value.status() {
        | Status::Ok(_) | Status::Redirected(_, _) => Check::init()
            .category(CheckCategory::Link)
            .success(true)
            .status_code(status_code.clone())
            .message("Has no HTTP errors"),
        | Status::Cached(status) => match status {
            | CacheStatus::Ok(_) => Check::init()
                .category(CheckCategory::Link)
                .success(true)
                .status_code(status_code.clone())
                .message("Has no HTTP errors"),
            | CacheStatus::Error(Some(_)) => Check::init()
                .category(CheckCategory::Link)
                .success(false)
                .status_code(status_code.clone())
                .message("Has cached HTTP errors"),
            | CacheStatus::Unsupported => Check::init()
                .category(CheckCategory::Link)
                .success(false)
                .status_code(status_code.clone())
                .message("Unsupported cached response"),
            | _ => Check::init()
                .category(CheckCategory::Link)
                .success(true)
                .status_code(status_code.clone())
                .message("Ignored or otherwise successful (cached response)"),
        },
        | Status::Error(_) => Check::init()
            .category(CheckCategory::Link)
            .success(false)
            .status_code(status_code.clone())
            .message("Has HTTP errors"),
        | Status::Unsupported(why) => Check::init()
            .category(CheckCategory::Link)
            .success(false)
            .status_code(status_code.clone())
            .message(format!("Unsupported HTTP response - {why}")),
        | Status::UnknownStatusCode(_) => Check::init()
            .category(CheckCategory::Link)
            .success(false)
            .status_code(status_code.clone())
            .message("Unknown HTTP response"),
        | Status::Timeout(_) => Check::init()
            .category(CheckCategory::Link)
            .success(false)
            .status_code(status_code.clone())
            .message("HTTP timeout"),
        | _ => Check::init()
            .category(CheckCategory::Link)
            .success(true)
            .status_code(status_code.clone())
            .message("Ignored or otherwise successful"),
    };
    check.build()
}
/// Perform link check on given URL using Lychee
pub async fn link_check(uri: Option<String>) -> Check {
    match uri {
        | Some(value) => {
            let result = lychee_lib::check(value.as_str()).await;
            match result {
                | Ok(response) => convert_lychee_response(response).with_uri(value),
                | Err(_) => Check::init()
                    .category(CheckCategory::Link)
                    .success(false)
                    .uri(value)
                    .message("Unreachable")
                    .build(),
            }
        }
        | None => Check::init().category(CheckCategory::Link).success(false).message("Missing URL").build(),
    }
}
/// Convert vector of [`Check`] values to a Polars [DataFrame]
pub fn checks_to_dataframe(values: Vec<Check>) -> PolarsResult<DataFrame> {
    let names = ["success", "category", "message", "uri", "status_code", "context"];
    to_dataframe::<Check, _, &str>(values, names)
}
/// Prints `text` to stdout using syntax highlighting for the specified `syntax`.
///
/// `highlight` is an iterator of line numbers to highlight in the output.
pub fn pretty_print<I: IntoIterator<Item = usize>>(text: &str, syntax: ProgrammingLanguage, highlight: I) {
    let input = format!("{text}\n");
    let language = syntax.to_string();
    let mut printer = PrettyPrinter::new();
    printer
        .input_from_bytes(input.as_bytes())
        .theme("zenburn")
        .language(&language)
        .line_numbers(true);
    for line in highlight {
        printer.highlight(line);
    }
    printer.print().unwrap();
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
/// Convert vector of values of a given type to a Polars [DataFrame]
/// ### Example
/// ```ignore
/// let df = to_dataframe::<i32, _, str>(vec![1, 2, 3], ["a", "b", "c"]);
/// ```
///
/// [DataFrame]: https://docs.rs/polars/latest/polars/prelude/struct.DataFrame.html
pub fn to_dataframe<'a, T, I, H>(values: Vec<T>, names: I) -> PolarsResult<DataFrame>
where
    T: IntoRow<'a>,
    H: Into<PlSmallStr>,
    I: IntoIterator<Item = H>,
{
    let rows = values.into_iter().map(|value| value.to_row::<T>()).collect::<Vec<_>>();
    match DataFrame::from_rows(&rows) {
        | Ok(mut df) => match df.set_column_names(names) {
            | Ok(_) => Ok(df),
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}

#[cfg(test)]
mod tests;
