//! # Prose analyzer module
//!
//! This is where we keep functions and interfaces necessary to execute ACORN's automated editorial style guide as well as content readability analyzer.
//!
use crate::analyzer::vale::{ValeOutput, ValeOutputItem};
#[cfg(feature = "std")]
use crate::io::async_runtime;
use crate::io::{command_exists, download_binary, extract_zip, file_checksum, make_executable, network_get_request, standard_project_folder};
use crate::prelude;
use crate::prelude::{create_dir_all, remove_file, Error, File, HashMap, PathBuf, Write};
use crate::schema::research_activity::ResearchActivity;
use crate::schema::{ImageObject, MediaObject, ProgrammingLanguage, VideoObject, Website};
use crate::util::constants::{
    APPLICATION, CUSTOM_VALE_PACKAGE_NAME, DEFAULT_VALE_PACKAGE_URL, DEFAULT_VALE_ROOT, DISABLED_VALE_RULES, ENABLED_VALE_PACKAGES, ORGANIZATION,
    VALE_RELEASES_URL, VALE_VERSION,
};
use crate::util::{suffix, Constant, Label, SemanticVersion, ToAbsoluteString, ToMarkdown};
use crate::{Location, Repository};
use bat::PrettyPrinter;
use bon::Builder;
use color_eyre::owo_colors::OwoColorize;
use convert_case::{Case, Casing};
use derive_more::Display;
use duct::cmd;
use flate2::read::GzDecoder;
use ini::Ini;
use lychee_lib::{CacheStatus, Response, Status};
use polars::datatypes::PlSmallStr;
use polars::frame::row::Row;
use polars::prelude::{AnyValue, DataFrame, PolarsResult};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tar::Archive;
use tracing::{debug, error, info, trace, warn};
use validator::Validate;
use validator::ValidationErrorsKind;
use which::which;

pub mod readability;
pub mod vale;

use readability::ReadabilityType;
use vale::{Vale, ValeConfig};

/// Trait for adding analysis capabilities
pub trait Analysis {
    /// Perform analysis of prose
    fn prose_analysis(paths: Vec<PathBuf>, is_offline: bool, skip_verify_checksum: bool) -> Vec<Check>;
    /// Calculate readability using a given metric
    fn readability_analysis<R>(paths: Vec<PathBuf>, readability_type: R) -> Vec<Check>
    where
        R: Into<ReadabilityType>;
    /// Check if URLs links are valid and readable
    fn url_analysis(paths: Vec<PathBuf>, is_offline: bool) -> Vec<Check>;
}
/// Trait for converting to a ([Polars]) row
///
/// [Polars]: https://docs.rs/polars/latest/polars/
pub trait IntoRow<'a> {
    /// Convert to a (Polars) row
    fn to_row<T>(self) -> Row<'a>;
}
/// Trait for static analyzers (e.g. Vale)
pub trait StaticAnalyzer<Config: StaticAnalyzerConfig> {
    /// Get command name (e.g. "vale")
    fn command(self) -> String;
    /// Download binary
    fn download(self, config: Option<Config>, skip_verify_checksum: bool) -> Self;
    /// Download checksum values
    fn download_checksums(self) -> Result<HashMap<String, String>, String>;
    /// Extract binary
    fn extract(self, path: PathBuf, destination: Option<PathBuf>) -> PathBuf;
    /// Resolve analyzer
    fn resolve(_config: Config, _is_offline: bool, _skip_verify_checksum: bool) -> Self;
    /// Run analyzer on content
    fn run(&self, id: String, content: String, output: Option<String>) -> Check;
    /// Perform sync operation (only applies to Vale)
    fn sync(self, is_offline: bool) -> Result<(), Error>;
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
pub trait StaticAnalyzerConfig {
    /// Get default configuration
    fn default() -> Self;
    /// Convert to INI
    fn ini(self) -> Ini;
    /// Save configuration
    fn save(self) -> Self;
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
/// Error kind
#[derive(Clone, Debug)]
pub enum ErrorKind {
    /// Readability issue where calculated index exceeds threshold of associated metric
    Readability((f64, ReadabilityType)),
    /// Prose issue found by Vale
    Vale(Vec<ValeOutputItem>),
    /// Schema validation issue found by [validator crate]
    ///
    /// [validator crate]: https://crates.io/crates/validator
    Validator(ValidationErrorsKind),
}
/// Data structure for holding the result of a schema validation check
#[derive(Builder, Clone, Debug, Display)]
#[builder(start_fn = init)]
#[display("{message}")]
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
impl Analysis for ResearchActivity {
    /// Checks links associated with research activity
    #[cfg(feature = "std")]
    fn url_analysis(paths: Vec<PathBuf>, is_offline: bool) -> Vec<Check> {
        let runtime = async_runtime();
        paths
            .par_iter()
            .map(|path| match ResearchActivity::read(path.into()) {
                | Some(data) => {
                    let issues = runtime.block_on(async {
                        let mut _issues = vec![];
                        if !is_offline {
                            let dois = match data.clone().meta.doi {
                                | Some(values) => values.into_iter().map(|doi| format!("https://doi.org/{doi}")).collect(),
                                | None => vec![],
                            };
                            let websites = match data.clone().meta.websites {
                                | Some(values) => values.into_iter().map(|Website { url, .. }| url).collect(),
                                | None => vec![],
                            };
                            let links = dois.into_iter().chain(websites);
                            for url in links {
                                let result = link_check(Some(url)).await;
                                _issues.push(result);
                            }
                        }
                        _issues
                    });
                    issues
                }
                | None => {
                    error!("=> {} Read research activity data at {}", Label::fail(), path.display());
                    vec![Check::init().category(CheckCategory::Schema).success(false).build()]
                }
            })
            .flatten()
            .collect()
    }
    fn prose_analysis(paths: Vec<PathBuf>, is_offline: bool, skip_verify_checksum: bool) -> Vec<Check> {
        let config = ValeConfig::default().save();
        let vale = Vale::resolve(config, is_offline, skip_verify_checksum);
        match vale.clone().sync(is_offline) {
            | Ok(_) => {
                let results = paths.iter().map(|path| match ResearchActivity::read(path.into()) {
                    | Some(data) => vale.clone().run(data.clone().meta.identifier, data.to_markdown(), Some("JSON".into())),
                    | None => {
                        error!("=> {} Read research activity data", Label::fail());
                        Check::init().category(CheckCategory::Prose).success(false).build()
                    }
                });
                results.collect()
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
            .map(|path| match ResearchActivity::read(path.into()) {
                | Some(data) => {
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
                            .message(path.display().to_string())
                            .errors(errors)
                            .context(maximum.to_string())
                            .build()
                    } else {
                        let score = format!("({} = {}/{})", rtype.to_string().to_uppercase(), index, maximum);
                        Check::init()
                            .category(CheckCategory::Readability)
                            .success(true)
                            .message(path.display().to_string())
                            .context(score)
                            .build()
                    }
                }
                | None => {
                    error!("=> {} Read research activity data", Label::fail());
                    Check::init().category(CheckCategory::Readability).success(false).build()
                }
            })
            .collect::<Vec<Check>>()
    }
}
impl Check {
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
                        | ErrorKind::Validator(values) => match values {
                            | ValidationErrorsKind::Field(_) => 1,
                            | ValidationErrorsKind::Struct(values) => values.clone().into_errors().len(),
                            | ValidationErrorsKind::List(_) => 0,
                        },
                        | _ => 0,
                    }
                } else {
                    0
                }
            }
        }
    }
    /// Print the schema check results
    pub fn print(self) {
        match self.category {
            | CheckCategory::Link => {
                let code = match self.status_code {
                    | Some(code) => format!(" ({code})").dimmed().to_string(),
                    | None => "".to_string(),
                };
                let url = match self.uri {
                    | Some(value) => value.underline().italic().to_string(),
                    | None => "Missing".italic().to_string(),
                };
                if self.success {
                    let message = &self.message.to_case(Case::Title).green().bold().to_string();
                    info!("=> {} \"{url}\" {message}{code}", Label::valid());
                } else {
                    let message = &self.message.to_case(Case::Title).red().bold().to_string();
                    error!("=> {} \"{url}\" {message}{code}", Label::invalid());
                }
            }
            | CheckCategory::Prose => {
                let Check {
                    context, errors, message, ..
                } = self;
                match &errors {
                    | Some(ErrorKind::Vale(values)) => {
                        error!("=> {} {} issues found in {}", Label::fail(), values.len(), message.underline());
                        for item in values {
                            let ValeOutputItem {
                                check,
                                line,
                                message,
                                severity,
                                span,
                                ..
                            } = item;
                            let location = format!("Line {}, Character {}", line, span[0]);
                            println!("  {:<24} {:<21} {} {}", location, severity.colored(), message, check.dimmed());
                        }
                        let highlight = values.clone().into_iter().map(|item| item.line as usize).collect::<Vec<_>>();
                        if let Some(content) = &context {
                            println!();
                            pretty_print(content, ProgrammingLanguage::Markdown, highlight);
                            println!("\n");
                        }
                    }
                    | None | Some(_) => {
                        let message = format!("=> {} {} has {}", Label::pass(), message.underline(), "no prose issues".green(),);
                        info!("{}", message);
                    }
                }
            }
            | CheckCategory::Readability => {
                let Check {
                    context, errors, message, ..
                } = self;
                match &errors {
                    | Some(ErrorKind::Readability(values)) => {
                        let (index, readability_type) = values;
                        error!(
                            "=> {} {} has {} value of {} (should be less than {})",
                            Label::fail(),
                            message,
                            readability_type.to_string().to_uppercase(),
                            index.red().bold(),
                            context.unwrap().cyan(),
                        );
                    }
                    | None | Some(_) => {
                        if let Some(context) = &context {
                            info!(
                                "=> {} {} has {} {}",
                                Label::pass(),
                                message,
                                "no readability issues".green().bold(),
                                context.dimmed()
                            );
                        }
                    }
                }
            }
            | CheckCategory::Schema => {
                let path = self.clone().uri.unwrap();
                if self.success {
                    info!("=> {} {} has {}", Label::pass(), path, "no schema validation issues".green().bold());
                } else {
                    let count = self.issue_count();
                    error!(
                        "=> {} Found {} schema validation issue{} in {}: \n{:#?}",
                        Label::fail(),
                        count.red(),
                        suffix(count),
                        path.italic().underline(),
                        self.errors.unwrap()
                    );
                }
            }
        }
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
impl StaticAnalyzer<ValeConfig> for Vale {
    fn command(self) -> String {
        "vale".to_string()
    }
    /// Resolve Vale
    /// ### Notes
    /// - Will use system `vale` command if available
    /// - Will use local `vale` binary if available (will expect local binary if offline)
    /// - Will download `vale` binary if not available by other means
    fn resolve(config: ValeConfig, is_offline: bool, skip_verify_checksum: bool) -> Vale {
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
            init.with_config(config).with_system_command()
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
            init.download(Some(config), skip_verify_checksum)
        };
        vale
    }
    fn run(&self, id: String, content: String, output: Option<String>) -> Check {
        let root = standard_project_folder("check", None);
        match create_dir_all(root.clone()) {
            | Ok(_) => {}
            | Err(why) => error!(path = root.clone().to_absolute_string(), "=> {} Create - {}", Label::fail(), why),
        }
        let path = root.join(&id);
        let mut file = match File::create(&path) {
            | Ok(file) => file,
            | Err(why) => panic!("=> {} Create file {} - {}", Label::fail(), path.display(), why),
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
                        binary,
                        "--no-wrap",
                        "--config",
                        config.clone().path,
                        "--output",
                        value,
                        path.clone(),
                        "--ext",
                        ".md",
                        "--no-exit",
                    )
                    .read(),
                    | None => cmd!(
                        binary,
                        "--no-wrap",
                        "--config",
                        config.clone().path,
                        path.clone(),
                        "--ext",
                        ".md",
                        "--no-exit"
                    )
                    .read(),
                };
                match result {
                    | Ok(output) => {
                        let parsed = ValeOutput::parse(&output, path);
                        if parsed.is_empty() {
                            Check::init().category(CheckCategory::Prose).success(true).message(id).build()
                        } else {
                            Check::init()
                                .category(CheckCategory::Prose)
                                .success(false)
                                .message(id)
                                .errors(ErrorKind::Vale(parsed))
                                .context(content)
                                .build()
                        }
                    }
                    | Err(output) => {
                        error!("=> {} Analyze - {}", Label::fail(), output);
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
    fn download(self, config: Option<ValeConfig>, skip_verify_checksum: bool) -> Vale {
        let platform = prelude::vale_release_filename();
        let release = match self.version {
            | Some(value) => value,
            | None => SemanticVersion::from_string(VALE_VERSION),
        };
        let url = format!("{VALE_RELEASES_URL}/download/v{release}/{}_{release}_{platform}", self.clone().command());
        info!(url, "=> {} Vale release v{release}", Label::using());
        let binary = match download_binary(&url, ".") {
            | Ok(path) => {
                if !skip_verify_checksum {
                    let dowloaded_checksum = match self.clone().download_checksums() {
                        | Ok(value) => value.get(&platform).unwrap().to_string(),
                        | Err(_) => "".to_string(),
                    };
                    if let Some(calculated) = file_checksum(path.clone()) {
                        if !dowloaded_checksum.eq(&calculated) {
                            error!(dowloaded_checksum, calculated, "=> {}", Label::invalid());
                            let _cleanup = remove_file(path.clone());
                        } else {
                            info!(checksum = dowloaded_checksum, "=> {} Checksum verification", Label::pass());
                        }
                    };
                } else {
                    warn!("=> {} Checksum verification", Label::skip());
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
    fn download_checksums(self) -> Result<HashMap<String, String>, String> {
        let release = match self.version {
            | Some(value) => value,
            | None => SemanticVersion::from_string(VALE_VERSION),
        };
        let url = format!(
            "{VALE_RELEASES_URL}/download/v{release}/{}_{release}_checksums.txt",
            self.clone().command()
        );
        let response = network_get_request(url).send().unwrap();
        let content = response.text().unwrap();
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
    fn sync(self, is_offline: bool) -> Result<(), Error> {
        let command = self.clone().command();
        let path = match self.binary {
            | Some(value) => value,
            | None => {
                error!("=> {} {} binary", Label::not_found(), command);
                PathBuf::from(DEFAULT_VALE_ROOT).join(command)
            }
        };
        let config_path = self.config.unwrap().path;
        let result = if is_offline {
            println!("=> {} Not running vale sync in offline mode", Label::skip());
            cmd!("").run()
        } else {
            cmd!(path.clone(), "--config", config_path.clone(), "sync").run()
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
                        // TODO: Concatenate organization alternative names to accept file
                        let acronyms = Constant::last_values("acronyms");
                        let partners = Constant::last_values("partners");
                        let sponsors = Constant::last_values("sponsors");
                        let words = Constant::read_lines("accept.txt");
                        let content = acronyms.chain(partners).chain(sponsors).chain(words).collect::<Vec<String>>().join("\n");
                        file.write_all(content.as_bytes()).expect("Unable to write to accept.txt");
                    }
                    | Err(why) => panic!("=> {} Create accept.txt - {}", Label::fail(), why),
                }
                match File::create(format!("{parent}/reject.txt")) {
                    | Ok(mut file) => {
                        let content = Constant::read_lines("reject.txt").join("\n");
                        file.write_all(content.as_bytes()).expect("Unable to write to reject.txt");
                    }
                    | Err(why) => panic!("=> {} Create reject.txt - {}", Label::fail(), why),
                }
                Ok(())
            }
            | Err(why) => {
                error!(config = config_path.to_absolute_string(), "=> {} Vale sync - {}", Label::fail(), why);
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
            let version = cmd!(name.clone(), "--version").read().unwrap()[offset..].to_string();
            self.version = Some(SemanticVersion::from_string(&version));
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
        self.version = Some(SemanticVersion::from_string(&value));
        self
    }
}
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
    fn ini(self) -> Ini {
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
        let package_url = match package_repository.latest_release() {
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
    fn save(self) -> ValeConfig {
        let path = self.clone().path;
        let parent = path.parent().unwrap().to_path_buf();
        match create_dir_all(parent.clone()) {
            | Ok(_) => {}
            | Err(why) => error!(directory = parent.to_absolute_string(), "=> {} Create - {why}", Label::fail()),
        }
        match self.clone().ini().write_to_file(path.clone()) {
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
            .map(|path| match ResearchActivity::read(path.into()) {
                | Some(data) => data
                    .clone()
                    .validation_issues()
                    .into_iter()
                    .map(|issue| issue.with_uri(path.display().to_string()))
                    .collect(),
                | None => {
                    error!("=> {} Read research activity data at {}", Label::fail(), path.display());
                    vec![Check::init().category(CheckCategory::Schema).success(false).build()]
                }
            })
            .flatten()
            .collect()
    }
    fn validation_issues(self) -> Vec<Check> {
        fn errors_collect<T: Validate>(attribute: T) -> Option<Vec<Check>> {
            match attribute.validate() {
                | Ok(_) => None,
                | Err(err) => Some(
                    err.into_errors()
                        .into_iter()
                        .map(|(key, value)| {
                            Check::init()
                                .category(CheckCategory::Schema)
                                .success(false)
                                .errors(ErrorKind::Validator(value))
                                .message(key.to_string())
                                .build()
                        })
                        .collect::<Vec<Check>>(),
                ),
            }
        }
        let mut found = vec![errors_collect::<ResearchActivity>(self.clone())];
        match self.meta.media {
            | Some(values) => values.iter().for_each(|media| match media {
                | MediaObject::Image(x) => found.push(errors_collect::<ImageObject>(x.clone())),
                | MediaObject::Video(x) => found.push(errors_collect::<VideoObject>(x.clone())),
            }),
            | None => {}
        }
        found.into_iter().flatten().flatten().collect::<Vec<_>>()
    }
}
/// Convert Lychee response to [`Check`]
pub fn convert_lychee_response(value: Response) -> Check {
    match value.status() {
        | Status::Ok(code) | Status::Redirected(code, _) => Check::init()
            .category(CheckCategory::Link)
            .success(true)
            .status_code(code.to_string())
            .message("has no HTTP errors".to_string())
            .build(),
        | Status::Cached(status) => match status {
            | CacheStatus::Ok(code) => Check::init()
                .category(CheckCategory::Link)
                .success(true)
                .status_code(code.to_string())
                .message("has no HTTP errors".to_string())
                .build(),
            | CacheStatus::Error(Some(code)) => Check::init()
                .category(CheckCategory::Link)
                .success(false)
                .status_code(code.to_string())
                .message("has cached HTTP errors".to_string())
                .build(),
            | CacheStatus::Unsupported => Check::init()
                .category(CheckCategory::Link)
                .success(false)
                .message("unsupported cached response".to_string())
                .build(),
            | _ => Check::init()
                .category(CheckCategory::Link)
                .success(true)
                .message("ignored or otherwise successful (cached response)".to_string())
                .build(),
        },
        | Status::Error(code) => Check::init()
            .category(CheckCategory::Link)
            .success(false)
            .status_code(code.to_string())
            .message("has HTTP errors".to_string())
            .build(),
        | Status::Unsupported(why) => Check::init()
            .category(CheckCategory::Link)
            .success(false)
            .message(format!("unsupported HTTP response - {why}"))
            .build(),
        | Status::UnknownStatusCode(code) => Check::init()
            .category(CheckCategory::Link)
            .success(false)
            .status_code(code.to_string())
            .message("unknown HTTP response".to_string())
            .build(),
        | Status::Timeout(_) => Check::init()
            .category(CheckCategory::Link)
            .success(false)
            .message("HTTP timeout".to_string())
            .build(),
        | _ => Check::init()
            .category(CheckCategory::Link)
            .success(true)
            .message("ignored or otherwise successful".to_string())
            .build(),
    }
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
                    .message("unreachable".to_string())
                    .build(),
            }
        }
        | None => Check::init()
            .category(CheckCategory::Link)
            .success(false)
            .message("missing URL".to_string())
            .build(),
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
