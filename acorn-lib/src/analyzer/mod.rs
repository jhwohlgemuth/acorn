//! # Prose analyzer module
//!
//! This is where we keep functions and interfaces necessary to execute ACORN's automated editorial style guide as well as content readability analyzer.
//!
//!
use crate::analyzer::error::Location as ErrorLocation;
use crate::analyzer::vale::{ValeOutput, ValeOutputItem};
#[cfg(feature = "std")]
use crate::io::http::get;
use crate::io::InputOutput;
use crate::io::{command_exists, download_binary, extract_zip, file_checksum, make_executable, standard_project_folder, ApiResult};
use crate::prelude::{self, create_dir_all, remove_file, write, Command, CommandOutput, File, HashMap, Path, PathBuf, Stdio};
use crate::schema::pid::{PersistentIdentifier, PersistentIdentifierParse, DOI};
use crate::schema::research_activity::ResearchActivity;
use crate::schema::standard::cff::{Cff, Identifier, IdentifierType, Reference};
use crate::schema::standard::text::{Docx, Text};
use crate::schema::{Organization, ProgrammingLanguage, Website};
use crate::util::constants::{APPLICATION, CUSTOM_VALE_PACKAGE_NAME, DEFAULT_VALE_PACKAGE_URL, DEFAULT_VALE_ROOT, VALE_RELEASES_URL, VALE_VERSION};
use crate::util::{is_uri_or_path, Constant, Label, SemanticVersion, StringConversion};
use crate::{check, check_err, check_ok};
use crate::{cmd, skip};
use crate::{Location, Repository};
use async_trait::async_trait;
use bat::PrettyPrinter;
use color_eyre::eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use convert_case::{Case, Casing};
use flate2::read::GzDecoder;
use futures::future::join_all;
use ini::Ini;
use lychee_lib::{CacheStatus, Response, Status};
#[cfg(feature = "analysis")]
use polars::datatypes::PlSmallStr;
#[cfg(feature = "analysis")]
use polars::prelude::{DataFrame, PolarsResult};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tar::Archive;
use tracing::{debug, error, info};
use validator::Validate;
use validator::ValidationErrorsKind;
use which::which;

pub mod check;
pub mod error;
pub mod readability;
pub mod vale;

#[cfg(feature = "analysis")]
pub use check::{checks_to_csv, checks_to_dataframe, summary, Analysis, Check, CheckCategory, CheckOptions, CheckSeverity, IntoRow, Standard};

#[cfg(feature = "analysis")]
use check::{check_prose_for, check_readability_for};
use error::{process, ErrorKind};
use vale::{Vale, ValeConfig};

/// Trait for static analyzers (e.g. Vale)
#[async_trait]
pub trait StaticAnalyzer<Config: StaticAnalyzerConfig> {
    /// Get command name (e.g. "vale")
    fn command(self) -> String;
    /// Download binary
    async fn download(self, config: Option<Config>, skip_verify_checksum: bool) -> Self;
    /// Download checksum values
    async fn download_checksums(self) -> ApiResult<HashMap<String, String>>;
    /// Extract binary
    fn extract(self, path: PathBuf, destination: Option<PathBuf>) -> PathBuf;
    /// Resolve analyzer
    async fn resolve(_config: Config, _is_offline: bool, _skip_verify_checksum: bool) -> Self;
    /// Run analyzer on content
    async fn run(&self, path: PathBuf, content: String, output: Option<String>) -> Vec<Check>;
    /// Perform sync operation (only applies to Vale)
    async fn sync(self, is_offline: bool, quiet: bool) -> ApiResult<()>;
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
    /// Convert to INI
    async fn ini(self) -> Ini;
    /// Save configuration
    async fn save(self) -> Self;
    /// Set parent path of configuration
    fn with_path(self, path: PathBuf) -> Self;
    /// Resolve package source: convert bare package name to download URL or pass through existing URL/path
    fn resolve_package(value: impl AsRef<str>) -> String;
}
#[async_trait]
impl Analysis for Cff {
    fn standard() -> Standard {
        Standard::CitationFileFormat
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        paths
            .par_iter()
            .map(|path| match Self::read(path) {
                | Ok(_) => check_ok!(CheckCategory::Quality),
                | Err(why) => check_err!(
                    CheckCategory::Quality,
                    message: "Cannot read CFF data",
                    context: why.to_string()
                ),
            })
            .collect()
    }
    async fn check_readability(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        check_readability_for::<Self>(paths, options)
    }
    async fn check_schema(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        paths
            .par_iter()
            .flat_map(|path| match Self::read(path) {
                | Ok(data) => {
                    let uri = path.file_name_with_parent();
                    collect_validation_checks(data.clone())
                        .into_iter()
                        .map(|issue| issue.with_uri(uri.clone()))
                        .collect::<Vec<_>>()
                }
                | Err(why) => {
                    error!("=> {} Read CFF data at {} — {why}", Label::fail(), path.display());
                    vec![check_err!(CheckCategory::Schema)]
                }
            })
            .collect()
    }
    #[cfg(feature = "std")]
    async fn check_websites(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        let futures = paths.iter().map(|path| {
            let path = path.clone();
            async move {
                match Self::read(&path) {
                    | Ok(data) => {
                        let Cff {
                            identifiers,
                            license_url,
                            references,
                            repository,
                            repository_artifact,
                            repository_code,
                            url,
                            ..
                        } = data;
                        let root_futures = vec![
                            url.map(|u| link_check(Some(u), Some("url".into()))),
                            license_url.map(|u| link_check(Some(u), Some("license_url".into()))),
                            repository.map(|u| link_check(Some(u), Some("repository".into()))),
                            repository_artifact.map(|u| link_check(Some(u), Some("repository_artifact".into()))),
                            repository_code.map(|u| link_check(Some(u), Some("repository_code".into()))),
                        ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>();
                        let identifier_futures = identifiers
                            .unwrap_or_default()
                            .into_iter()
                            .enumerate()
                            .filter_map(|(i, Identifier { kind, value, .. })| match kind {
                                | IdentifierType::Doi => {
                                    let maybe_doi = DOI::from_string(value.clone()).url();
                                    let maybe_url = if maybe_doi.is_empty() { Some(value) } else { Some(maybe_doi) };
                                    Some(link_check(maybe_url, Some(format!("identifiers[{i}].value"))))
                                }
                                | IdentifierType::Url => Some(link_check(Some(value), Some(format!("identifiers[{i}].value")))),
                                | _ => None,
                            })
                            .collect::<Vec<_>>();
                        let reference_futures = references
                            .unwrap_or_default()
                            .into_iter()
                            .enumerate()
                            .flat_map(
                                |(
                                    i,
                                    Reference {
                                        doi,
                                        collection_doi,
                                        license_url,
                                        repository,
                                        repository_artifact,
                                        repository_code,
                                        url,
                                        ..
                                    },
                                )| {
                                    vec![
                                        doi.map(|d| link_check(Some(DOI::from_string(d).url()), Some(format!("references[{i}].doi")))),
                                        collection_doi
                                            .map(|d| link_check(Some(DOI::from_string(d).url()), Some(format!("references[{i}].collection_doi")))),
                                        url.map(|u| link_check(Some(u), Some(format!("references[{i}].url")))),
                                        license_url.map(|u| link_check(Some(u), Some(format!("references[{i}].license_url")))),
                                        repository.map(|u| link_check(Some(u), Some(format!("references[{i}].repository")))),
                                        repository_artifact.map(|u| link_check(Some(u), Some(format!("references[{i}].repository_artifact")))),
                                        repository_code.map(|u| link_check(Some(u), Some(format!("references[{i}].repository_code")))),
                                    ]
                                    .into_iter()
                                    .flatten()
                                },
                            )
                            .collect::<Vec<_>>();
                        let futures: Vec<_> = root_futures.into_iter().chain(identifier_futures).chain(reference_futures).collect();
                        join_all(futures)
                            .await
                            .into_iter()
                            .map(|check| check.with_uri(path.file_name_with_parent()))
                            .collect()
                    }
                    | Err(why) => {
                        error!("=> {} Read CFF data at {} — {why}", Label::fail(), path.display());
                        vec![check_err!(CheckCategory::Schema)]
                    }
                }
            }
        });
        join_all(futures).await.into_iter().flatten().collect()
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
}
#[async_trait]
impl Analysis for Docx {
    fn standard() -> Standard {
        Standard::Docx
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        paths
            .par_iter()
            .map(|path| match Self::read(path) {
                | Ok(_) => check_ok!(CheckCategory::Quality),
                | Err(why) => check_err!(
                    CheckCategory::Quality,
                    message: "Cannot read DOCX data",
                    context: why.to_string()
                ),
            })
            .collect()
    }
    async fn check_readability(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        check_readability_for::<Self>(paths, options)
    }
    async fn check_schema(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        vec![]
    }
    #[cfg(feature = "std")]
    async fn check_websites(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        vec![]
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
}
#[async_trait]
impl Analysis for ResearchActivity {
    fn standard() -> Standard {
        Standard::ResearchActivityData
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        paths
            .par_iter()
            .map(|path| match Self::read(path) {
                | Ok(_) => check_ok!(CheckCategory::Quality),
                | Err(why) => check_err!(
                    CheckCategory::Quality,
                    message: "Cannot read research activity data",
                    context: why.to_string()
                ),
            })
            .collect()
    }
    async fn check_readability(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        check_readability_for::<Self>(paths, options)
    }
    async fn check_schema(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        paths
            .par_iter()
            .flat_map(|path| match Self::read(path) {
                | Ok(data) => {
                    let uri = path.file_name_with_parent();
                    collect_validation_checks(data.clone())
                        .into_iter()
                        .map(|issue| issue.with_uri(uri.clone()))
                        .collect::<Vec<_>>()
                }
                | Err(why) => {
                    error!("=> {} Read research activity data at {} — {why}", Label::fail(), path.display());
                    vec![check_err!(CheckCategory::Schema)]
                }
            })
            .collect()
    }
    #[cfg(feature = "std")]
    async fn check_websites(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        let futures = paths.iter().map(|path| {
            let path = path.clone();
            async move {
                match Self::read(&path) {
                    | Ok(data) => {
                        let dois = match data.clone().meta.doi {
                            | Some(values) => values
                                .into_iter()
                                .enumerate()
                                .map(|(i, doi)| link_check(Some(format!("https://doi.org/{doi}")), Some(format!("meta.doi[{i}]"))))
                                .collect::<Vec<_>>(),
                            | None => vec![],
                        };
                        let websites = match data.clone().meta.websites {
                            | Some(values) => values
                                .into_iter()
                                .enumerate()
                                .map(|(i, Website { url, .. })| link_check(Some(url), Some(format!("meta.websites[{i}].url"))))
                                .collect::<Vec<_>>(),
                            | None => vec![],
                        };
                        let contact = [link_check(Some(data.clone().contact.url), Some("contact.url".into()))];
                        let links: Vec<_> = [].into_iter().chain(dois).chain(websites).chain(contact).collect();
                        join_all(links)
                            .await
                            .into_iter()
                            .map(|check| check.with_uri(path.file_name_with_parent()))
                            .collect()
                    }
                    | Err(why) => {
                        error!("=> {} Read research activity data at {} — {why}", Label::fail(), path.display());
                        vec![check_err!(CheckCategory::Schema)]
                    }
                }
            }
        });
        join_all(futures).await.into_iter().flatten().collect()
    }
    fn output_path(path: &Path, data: &Self) -> PathBuf {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("index.json");
        standard_project_folder("check", None)
            .join(data.meta.identifier.to_lowercase())
            .join(filename)
    }
}
#[async_trait]
impl Analysis for Text {
    fn standard() -> Standard {
        Standard::Text
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        paths
            .par_iter()
            .map(|path| match Self::read(path) {
                | Ok(_) => check_ok!(CheckCategory::Quality),
                | Err(why) => check_err!(
                    CheckCategory::Quality,
                    message: "Cannot read text data",
                    context: why.to_string()
                ),
            })
            .collect()
    }
    async fn check_readability(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        check_readability_for::<Self>(paths, options)
    }
    async fn check_schema(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        vec![]
    }
    #[cfg(feature = "std")]
    async fn check_websites(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        vec![]
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
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
    async fn run(&self, path: PathBuf, content: String, output_format: Option<String>) -> Vec<Check> {
        let root = path
            .parent()
            .map(|value| value.to_path_buf())
            .unwrap_or_else(|| standard_project_folder("check", None));
        match create_dir_all(root.clone()) {
            | Ok(_) => {}
            | Err(why) => error!(path = root.clone().to_absolute_string(), "=> {} Create — {why}", Label::fail()),
        }
        match write(&path, content.as_bytes()) {
            | Ok(_) => {}
            | Err(why) => panic!("=> {} Write file {} - {why}", Label::fail(), path.display()),
        }
        let binary = match &self.binary {
            | Some(value) => value,
            | None => {
                error!("=> {} {} binary", Label::not_found(), self.clone().command());
                &PathBuf::from("./.vale/vale")
            }
        };
        let uri = path.file_name_with_parent();
        match &self.config {
            | Some(config) => {
                let binary_path = binary.to_absolute_string();
                let config_path = config.clone().path.to_absolute_string();
                let path_string = path.clone().to_absolute_string();
                let args = [
                    "--no-exit".to_string(),
                    "--no-wrap".to_string(),
                    "--ext".to_string(),
                    ".md".to_string(),
                    "--config".to_string(),
                    config_path,
                    path_string,
                ];
                let result = match output_format {
                    | Some(value) => {
                        let args = args.into_iter().chain(["--output".to_string(), value]).collect::<Vec<String>>();
                        cmd!(binary_path, args)
                    }
                    | None => cmd!(binary_path, args),
                };
                match result {
                    | Ok(output) if output.status.success() => {
                        let parsed = ValeOutput::parse(&output.stdout(), path.clone());
                        if parsed.is_empty() {
                            info!("=> {} {} has {}", Label::pass(), uri.underline(), "no prose issues".green());
                            vec![check_ok!(CheckCategory::Prose, uri: uri)]
                        } else {
                            parsed
                                .into_iter()
                                .map(|item| {
                                    let ValeOutputItem { message, severity, .. } = item.clone();
                                    check!(
                                        CheckCategory::Prose,
                                        false,
                                        severity: severity.into(),
                                        uri: uri.clone(),
                                        locator: item.locator(),
                                        message: message,
                                        errors: ErrorKind::Vale(vec![item.clone()]),
                                        data: content.clone(),
                                    )
                                })
                                .collect()
                        }
                    }
                    | Ok(output) => {
                        let why = output.stderr();
                        let message = if why.is_empty() {
                            format!("process exited with status {}", output.status)
                        } else {
                            why
                        };
                        error!("=> {} Analyze with OK output — {message}", Label::fail());
                        vec![check_err!(CheckCategory::Prose, message: message)]
                    }
                    | Err(why) => {
                        error!("=> {} Analyze — {why}", Label::fail());
                        vec![check_err!(CheckCategory::Prose, uri: uri)]
                    }
                }
            }
            | None => {
                let title = self.clone().command().to_case(Case::Title);
                error!("=> {} {} configuration", Label::not_found(), title);
                vec![check_err!(CheckCategory::Prose, message: uri)]
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
            | Err(why) => {
                error!(url, "=> {} {} download — {why}", Label::fail(), self.command());
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
    async fn download_checksums(self) -> ApiResult<HashMap<String, String>> {
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
                error!("=> {} {command} extract — Unsupported format", Label::fail());
                PathBuf::from(DEFAULT_VALE_ROOT).join(command)
            }
        }
    }
    async fn sync(self, is_offline: bool, quiet: bool) -> ApiResult<()> {
        let command = self.clone().command();
        let binary_path = match self.binary {
            | Some(value) => value,
            | None => {
                error!("=> {} {} binary", Label::not_found(), command);
                PathBuf::from(DEFAULT_VALE_ROOT).join(command)
            }
        };
        let config_path = self.config.unwrap().path;
        let result: ApiResult<()> = if is_offline {
            skip!("Vale sync");
            Ok(())
        } else {
            let pipe = || if quiet { Stdio::null() } else { Stdio::inherit() };
            let status = Command::new(binary_path.clone())
                .arg("--config")
                .arg(config_path.clone())
                .arg("sync")
                .stdout(pipe())
                .stderr(pipe())
                .status();
            match status {
                | Ok(value) if value.success() => Ok(()),
                | Ok(value) => Err(eyre!("Vale sync failed ({value})")),
                | Err(why) => Err(eyre!("Vale sync failed — {why}")),
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
                let acronyms = Constant::last_values("acronyms");
                let partners = Constant::nth_values("partners", 1);
                let sponsors = Constant::nth_values("sponsors", 1);
                let abbreviations = Organization::alternative_names();
                let words = Constant::read_lines("accept.txt");
                let accept_content = acronyms
                    .chain(partners)
                    .chain(sponsors)
                    .chain(abbreviations)
                    .chain(words)
                    .collect::<Vec<String>>()
                    .join("\n");
                match write(format!("{parent}/accept.txt"), accept_content.as_bytes()) {
                    | Ok(_) => {}
                    | Err(why) => panic!("=> {} Write accept.txt — {why}", Label::fail()),
                }
                let reject_content = Constant::read_lines("reject.txt").join("\n");
                match write(format!("{parent}/reject.txt"), reject_content.as_bytes()) {
                    | Ok(_) => {}
                    | Err(why) => panic!("=> {} Write reject.txt — {why}", Label::fail()),
                }
                Ok(())
            }
            | Err(why) => {
                error!(config = config_path.to_absolute_string(), "=> {} Vale sync — {why}", Label::fail());
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
    async fn ini(self) -> Ini {
        let ValeConfig {
            packages,
            vocabularies,
            disabled,
            ..
        } = self;
        let package_names = packages.clone();
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
        let package_sources = package_names
            .iter()
            .map(Self::resolve_package)
            .chain(core::iter::once(package_url))
            .collect::<Vec<String>>();
        // CAUTION: Order of attributes in INI file matter. "StylesPath" must come before "Vocab"
        conf.with_section::<String>(None)
            .set("StylesPath", "styles")
            .set("Vocab", vocabularies.join(", "))
            .set("Packages", package_sources.join(", "));
        conf.with_section(Some("*")).set(
            "BasedOnStyles",
            format!("Vale, {}, {}", CUSTOM_VALE_PACKAGE_NAME, package_names.join(", ")),
        );
        disabled.iter().for_each(|rule| {
            conf.with_section(Some("*")).set(rule, "NO");
        });
        conf
    }
    fn resolve_package(value: impl AsRef<str>) -> String {
        let value = value.as_ref().trim();
        if is_uri_or_path(value) {
            value.to_string()
        } else {
            format!("https://github.com/errata-ai/{value}/releases/latest/download/{value}.zip")
        }
    }
    async fn save(self) -> ValeConfig {
        let path = self.clone().path;
        let parent = path.parent().unwrap().to_path_buf();
        match create_dir_all(parent.clone()) {
            | Ok(_) => {}
            | Err(why) => error!(directory = parent.to_absolute_string(), "=> {} Create — {why}", Label::fail()),
        }
        match self.clone().ini().await.write_to_file(path.clone()) {
            | Ok(_) => {
                debug!(path = path.to_absolute_string(), "=> {} Saved configuration", Label::using());
            }
            | Err(why) => {
                error!("=> {} Save configuration — {why}", Label::fail());
            }
        }
        self
    }
    fn with_path(mut self, path: PathBuf) -> Self {
        self.path = path;
        self
    }
}
pub(crate) fn collect_validation_checks<T>(value: T) -> Vec<Check>
where
    T: Validate,
{
    value
        .validate()
        .err()
        .into_iter()
        .flat_map(|err| {
            let kind = ValidationErrorsKind::Struct(Box::new(err));
            process("", &kind).into_iter().map(|issue| {
                let validator_error = validator::ValidationError {
                    code: issue.code.to_string().into(),
                    message: Some(issue.message.clone().into()),
                    params: issue.params.clone().into_iter().map(|(key, value)| (key.into(), value)).collect(),
                };
                let errors = ValidationErrorsKind::Field(vec![validator_error]);
                let prefix = issue.path.clone().unwrap_or_else(|| issue.locator());
                check_err!(
                    CheckCategory::Schema,
                    locator: issue.locator(),
                    context: prefix,
                    message: issue.message,
                    errors: ErrorKind::Validator(errors)
                )
            })
        })
        .collect()
}
/// Convert Lychee response to [`Check`]
pub(crate) fn convert_lychee_response(value: Response) -> Check {
    let body = value.body().to_string();
    let url = value.source().to_string();
    debug!(url, "=> {} Response: {:#?}", Label::using(), body.dimmed().cyan());
    let status_code = value.status().code().map(|c| c.to_string()).unwrap_or_default();
    match value.status() {
        | Status::Ok(_) | Status::Redirected(_, _) => check!(
            CheckCategory::Link,
            true,
            severity: CheckSeverity::Info,
            status_code: status_code.clone(),
            message: "Has no HTTP errors"
        ),
        | Status::Cached(status) => match status {
            | CacheStatus::Ok(_) => check!(
                CheckCategory::Link,
                true,
                severity: CheckSeverity::Info,
                status_code: status_code.clone(),
                message: "Has no HTTP errors"
            ),
            | CacheStatus::Error(Some(_)) => check_err!(
                CheckCategory::Link,
                status_code: status_code.clone(),
                message: "Has cached HTTP errors"
            ),
            | CacheStatus::Unsupported => check!(
                CheckCategory::Link,
                false,
                severity: CheckSeverity::Warning,
                status_code: status_code.clone(),
                message: "Unsupported cached response"
            ),
            | _ => check!(
                CheckCategory::Link,
                true,
                severity: CheckSeverity::Suggestion,
                status_code: status_code.clone(),
                message: "Ignored or otherwise successful (cached response)"
            ),
        },
        | Status::Error(_) => check_err!(
            CheckCategory::Link,
            status_code: status_code.clone(),
            message: "Has HTTP errors"
        ),
        | Status::Unsupported(why) => check!(
            CheckCategory::Link,
            false,
            severity: CheckSeverity::Warning,
            status_code: status_code.clone(),
            message: format!("Unsupported HTTP response — {why}")
        ),
        | Status::UnknownStatusCode(_) => check!(
            CheckCategory::Link,
            false,
            severity: CheckSeverity::Warning,
            status_code: status_code.clone(),
            message: "Unknown HTTP response"
        ),
        | Status::Timeout(_) => check!(
            CheckCategory::Link,
            false,
            severity: CheckSeverity::Warning,
            status_code: status_code.clone(),
            message: "HTTP timeout"
        ),
        | _ => check!(
            CheckCategory::Link,
            true,
            severity: CheckSeverity::Suggestion,
            status_code: status_code.clone(),
            message: "Ignored or otherwise successful"
        ),
    }
}
/// Perform link check on given URL using Lychee
pub async fn link_check(uri: Option<String>, locator: Option<String>) -> Check {
    match uri {
        | Some(value) => {
            let context = value.clone();
            let result = lychee_lib::check(value.as_str()).await;
            match result {
                | Ok(response) => convert_lychee_response(response).with_context(context).with_locator(locator),
                | Err(_) => check_err!(CheckCategory::Link, context: context, message: "Unreachable").with_locator(locator),
            }
        }
        | None => check_err!(CheckCategory::Link, message: "Missing URL").with_locator(locator),
    }
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
/// Convert vector of values of a given type to a Polars [DataFrame]
/// ### Example
/// ```ignore
/// let df = to_dataframe::<i32, _, str>(vec![1, 2, 3], ["a", "b", "c"]);
/// ```
///
/// [DataFrame]: https://docs.rs/polars/latest/polars/prelude/struct.DataFrame.html
#[cfg(feature = "analysis")]
pub fn to_dataframe<'a, T, I, H>(values: Vec<T>, names: I) -> PolarsResult<DataFrame>
where
    T: IntoRow<'a>,
    H: Into<PlSmallStr>,
    I: IntoIterator<Item = H>,
{
    let rows = values.into_iter().map(|value| value.to_row::<T>()).collect::<Vec<_>>();
    match DataFrame::from_rows(&rows) {
        | Ok(mut df) => match df.set_column_names(&names.into_iter().map(Into::into).collect::<Vec<PlSmallStr>>()) {
            | Ok(_) => Ok(df),
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}

#[cfg(test)]
mod tests;
