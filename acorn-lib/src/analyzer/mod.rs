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
use crate::prelude::{self, create_dir_all, remove_file, write, Arc, Command, CommandOutput, File, HashMap, Path, PathBuf, Stdio};
use crate::schema::pid::raid;
use crate::schema::pid::{PersistentIdentifier, PersistentIdentifierParse, DOI};
use crate::schema::research_activity::ResearchActivity;
use crate::schema::standard::cff::{Cff, Identifier, IdentifierType, Reference};
use crate::schema::standard::text::{Docx, Text};
use crate::schema::standard::{datacite, dcat, huwise, invenio};
use crate::schema::{Organization, ProgrammingLanguage, Website};
use crate::util::constants::app::APPLICATION;
use crate::util::constants::vale::{CUSTOM_VALE_PACKAGE_NAME, DEFAULT_VALE_PACKAGE_URL, DEFAULT_VALE_ROOT, VALE_RELEASES_URL, VALE_VERSION};
use crate::util::{is_uri_or_path, Constant, Label, SemanticVersion, StringConversion};
use crate::{check, check_err, check_ok};
use crate::{cmd, skip};
use crate::{Location, Repository};
use async_trait::async_trait;
#[cfg(feature = "analysis")]
use bat::PrettyPrinter;
use color_eyre::eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use convert_case::{Case, Casing};
use flate2::read::GzDecoder;
use futures::future::{BoxFuture, FutureExt};
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
#[cfg(feature = "analysis")]
pub mod discovery;
pub mod error;
pub mod readability;
#[cfg(feature = "analysis")]
mod service;
pub mod vale;

pub use check::{Check, CheckCategory, CheckOptions, CheckSeverity, IntoChecks, Standard};

#[cfg(feature = "analysis")]
pub use check::{checks_to_csv, checks_to_dataframe, summary, Analysis, IntoRow};
#[cfg(feature = "analysis")]
pub use service::{analyze_paths, classify_paths, AnalysisBatch, AnalysisReport, CategoryChecks, StandardPaths};

#[cfg(feature = "analysis")]
use check::{check_prose_for, check_readability_for};
use error::{process, ErrorKind};
use vale::{Vale, ValeConfig};

/// Trait for static analyzers (e.g. Vale)
#[async_trait]
pub trait StaticAnalyzer<Config: StaticAnalyzerConfig> {
    /// Get command name (e.g. "vale")
    fn command(&self) -> String;
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
#[cfg(feature = "analysis")]
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
                | Err(why) => check_err!(CheckCategory::Quality, context: why.to_string()),
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
                    let uri = Some(path.file_name_with_parent());
                    let data = Arc::new(data);
                    collect_validation_checks(data.as_ref())
                        .into_iter()
                        .map(|issue| issue.with_uri(uri.clone()))
                        .collect::<Vec<_>>()
                }
                | Err(why) => why
                    .to_string()
                    .lines()
                    .map(|line| check_err!(CheckCategory::Schema, context: line.to_string()).with_uri(Some(path.file_name_with_parent())))
                    .collect(),
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
                        let _data = Arc::new(data);
                        let Cff {
                            identifiers,
                            license_url,
                            references,
                            repository,
                            repository_artifact,
                            repository_code,
                            url,
                            ..
                        } = _data.as_ref();
                        let root_futures = vec![
                            url.as_deref().map(|u| link_check(Some(u), Some("url".into()))),
                            license_url.as_deref().map(|u| link_check(Some(u), Some("license_url".into()))),
                            repository.as_deref().map(|u| link_check(Some(u), Some("repository".into()))),
                            repository_artifact
                                .as_deref()
                                .map(|u| link_check(Some(u), Some("repository_artifact".into()))),
                            repository_code.as_deref().map(|u| link_check(Some(u), Some("repository_code".into()))),
                        ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>();
                        let identifier_futures = identifiers
                            .as_ref()
                            .into_iter()
                            .flatten()
                            .enumerate()
                            .filter_map(|(i, Identifier { kind, value, .. })| match kind {
                                | IdentifierType::Doi => {
                                    let maybe_doi = DOI::from_string(value).url();
                                    let url = if maybe_doi.is_empty() { value.to_owned() } else { maybe_doi };
                                    Some(link_check(Some(url), Some(format!("identifiers[{i}].value"))))
                                }
                                | IdentifierType::Url => Some(link_check(Some(value.as_str()), Some(format!("identifiers[{i}].value")))),
                                | _ => None,
                            })
                            .collect::<Vec<_>>();
                        let reference_futures = references
                            .as_ref()
                            .into_iter()
                            .flatten()
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
                                        doi.as_deref().map(|d| {
                                            let url = DOI::from_string(d).url();
                                            link_check(Some(url), Some(format!("references[{i}].doi")))
                                        }),
                                        collection_doi.as_deref().map(|d| {
                                            let url = DOI::from_string(d).url();
                                            link_check(Some(url), Some(format!("references[{i}].collection_doi")))
                                        }),
                                        license_url
                                            .as_deref()
                                            .map(|u| link_check(Some(u), Some(format!("references[{i}].license_url")))),
                                        repository
                                            .as_deref()
                                            .map(|u| link_check(Some(u), Some(format!("references[{i}].repository")))),
                                        repository_artifact
                                            .as_deref()
                                            .map(|u| link_check(Some(u), Some(format!("references[{i}].repository_artifact")))),
                                        repository_code
                                            .as_deref()
                                            .map(|u| link_check(Some(u), Some(format!("references[{i}].repository_code")))),
                                        url.as_deref().map(|u| link_check(Some(u), Some(format!("references[{i}].url")))),
                                    ]
                                    .into_iter()
                                    .flatten()
                                },
                            )
                            .collect::<Vec<_>>();
                        let futures: Vec<_> = root_futures.into_iter().chain(identifier_futures).chain(reference_futures).collect();
                        Self::collect_checks(futures, &path).await
                    }
                    | Err(why) => vec![check_err!(CheckCategory::Link, context: why.to_string()).with_uri(Some(path.file_name_with_parent()))],
                }
            }
        });
        Self::flatten_checks(futures).await
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
}
#[cfg(feature = "analysis")]
#[async_trait]
impl Analysis for datacite::Record {
    fn standard() -> Standard {
        Standard::Datacite
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        paths
            .par_iter()
            .map(|path| match Self::read(path) {
                | Ok(_) => check_ok!(CheckCategory::Quality),
                | Err(why) => check_err!(CheckCategory::Quality, context: why.to_string()),
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
                    let uri = Some(path.file_name_with_parent());
                    let data = Arc::new(data);
                    collect_validation_checks(data.as_ref())
                        .into_iter()
                        .map(|issue| issue.with_uri(uri.clone()))
                        .collect::<Vec<_>>()
                }
                | Err(why) => why
                    .to_string()
                    .lines()
                    .map(|line| check_err!(CheckCategory::Schema, context: line.to_string()).with_uri(Some(path.file_name_with_parent())))
                    .collect(),
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
                        let data = Arc::new(data);
                        let attributes = &data.attributes;
                        let root_futures = attributes
                            .url
                            .as_deref()
                            .map(|value| link_check(Some(value), Some("attributes.url".into())))
                            .into_iter();
                        let rights_futures = attributes.rights_list.iter().flatten().enumerate().filter_map(|(index, value)| {
                            value
                                .rights_uri
                                .as_deref()
                                .map(|uri| link_check(Some(uri), Some(format!("attributes.rights_list[{index}].rights_uri"))))
                        });
                        let related_futures = attributes.related_identifiers.iter().flatten().enumerate().filter_map(|(index, value)| {
                            match value.related_identifier_type {
                                | Some(datacite::RelatedIdentifierType::Doi) => {
                                    let url = DOI::from_string(value.related_identifier.as_str()).url();
                                    let target = if url.is_empty() { value.related_identifier.clone() } else { url };
                                    Some(link_check(
                                        Some(target),
                                        Some(format!("attributes.related_identifiers[{index}].related_identifier")),
                                    ))
                                }
                                | Some(datacite::RelatedIdentifierType::Url) => Some(link_check(
                                    Some(value.related_identifier.as_str()),
                                    Some(format!("attributes.related_identifiers[{index}].related_identifier")),
                                )),
                                | _ => None,
                            }
                        });
                        Self::collect_checks(root_futures.chain(rights_futures).chain(related_futures).collect::<Vec<_>>(), &path).await
                    }
                    | Err(why) => vec![check_err!(CheckCategory::Link, context: why.to_string()).with_uri(Some(path.file_name_with_parent()))],
                }
            }
        });
        Self::flatten_checks(futures).await
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
}
#[cfg(feature = "analysis")]
#[async_trait]
impl Analysis for dcat::Dataset {
    fn standard() -> Standard {
        Standard::Dcat
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        paths
            .par_iter()
            .map(|path| match Self::read(path) {
                | Ok(_) => check_ok!(CheckCategory::Quality),
                | Err(why) => check_err!(CheckCategory::Quality, context: why.to_string()),
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
                    let uri = Some(path.file_name_with_parent());
                    let data = Arc::new(data);
                    collect_validation_checks(data.as_ref())
                        .into_iter()
                        .map(|issue| issue.with_uri(uri.clone()))
                        .collect::<Vec<_>>()
                }
                | Err(why) => why
                    .to_string()
                    .lines()
                    .map(|line| check_err!(CheckCategory::Schema, context: line.to_string()).with_uri(Some(path.file_name_with_parent())))
                    .collect(),
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
                        let data = Arc::new(data);
                        let root_futures = data
                            .landing_page
                            .iter()
                            .flatten()
                            .enumerate()
                            .map(|(index, value)| link_check(value.url(), Some(format!("landing_page[{index}]"))));
                        let distribution_futures = data
                            .distribution
                            .iter()
                            .flatten()
                            .enumerate()
                            .flat_map(|(distribution_index, distribution)| {
                                distribution
                                    .access_url
                                    .iter()
                                    .enumerate()
                                    .map(move |(index, value)| {
                                        link_check(
                                            Some(value.as_str()),
                                            Some(format!("distribution[{distribution_index}].access_url[{index}]")),
                                        )
                                    })
                                    .chain(distribution.download_url.iter().flatten().enumerate().map(move |(index, value)| {
                                        link_check(
                                            Some(value.as_str()),
                                            Some(format!("distribution[{distribution_index}].download_url[{index}]")),
                                        )
                                    }))
                            });
                        Self::collect_checks(root_futures.chain(distribution_futures).collect::<Vec<_>>(), &path).await
                    }
                    | Err(why) => vec![check_err!(CheckCategory::Link, context: why.to_string()).with_uri(Some(path.file_name_with_parent()))],
                }
            }
        });
        Self::flatten_checks(futures).await
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
}
#[cfg(feature = "analysis")]
#[async_trait]
impl Analysis for invenio::Record {
    fn standard() -> Standard {
        Standard::Invenio
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        paths
            .par_iter()
            .map(|path| match Self::read(path) {
                | Ok(_) => check_ok!(CheckCategory::Quality),
                | Err(why) => check_err!(CheckCategory::Quality, context: why.to_string()),
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
                    let uri = Some(path.file_name_with_parent());
                    let data = Arc::new(data);
                    collect_validation_checks(data.as_ref())
                        .into_iter()
                        .map(|issue| issue.with_uri(uri.clone()))
                        .collect::<Vec<_>>()
                }
                | Err(why) => why
                    .to_string()
                    .lines()
                    .map(|line| check_err!(CheckCategory::Schema, context: line.to_string()).with_uri(Some(path.file_name_with_parent())))
                    .collect(),
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
                        let data = Arc::new(data);
                        let pid_futures = data.pids.iter().flat_map(|pids| {
                            [
                                pids.doi.as_ref().map(|value| {
                                    link_check(
                                        Some(DOI::from_string(value.identifier.as_str()).url()),
                                        Some("pids.doi.identifier".into()),
                                    )
                                }),
                                pids.concept_doi.as_ref().map(|value| {
                                    link_check(
                                        Some(DOI::from_string(value.identifier.as_str()).url()),
                                        Some("pids.concept_doi.identifier".into()),
                                    )
                                }),
                            ]
                            .into_iter()
                            .flatten()
                        });
                        let related_futures = data.metadata.iter().flat_map(|metadata| {
                            metadata.related_identifiers.iter().flatten().enumerate().filter_map(|(index, value)| {
                                match value.scheme.to_lowercase().as_str() {
                                    | "doi" => {
                                        let url = DOI::from_string(value.identifier.as_str()).url();
                                        let target = if url.is_empty() { value.identifier.clone() } else { url };
                                        Some(link_check(
                                            Some(target),
                                            Some(format!("metadata.related_identifiers[{index}].identifier")),
                                        ))
                                    }
                                    | "url" => Some(link_check(
                                        Some(value.identifier.as_str()),
                                        Some(format!("metadata.related_identifiers[{index}].identifier")),
                                    )),
                                    | _ => None,
                                }
                            })
                        });
                        let rights_futures = data.metadata.iter().flat_map(|metadata| {
                            metadata.rights.iter().flatten().enumerate().filter_map(|(index, value)| {
                                value
                                    .link
                                    .as_deref()
                                    .map(|uri| link_check(Some(uri), Some(format!("metadata.rights[{index}].link"))))
                            })
                        });
                        Self::collect_checks(pid_futures.chain(related_futures).chain(rights_futures).collect::<Vec<_>>(), &path).await
                    }
                    | Err(why) => vec![check_err!(CheckCategory::Link, context: why.to_string()).with_uri(Some(path.file_name_with_parent()))],
                }
            }
        });
        Self::flatten_checks(futures).await
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
}
#[cfg(feature = "analysis")]
#[async_trait]
impl Analysis for huwise::Dataset {
    fn standard() -> Standard {
        Standard::Huwise
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&CheckOptions>) -> Vec<Check> {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        paths
            .par_iter()
            .map(|path| match Self::read(path) {
                | Ok(_) => check_ok!(CheckCategory::Quality),
                | Err(why) => check_err!(CheckCategory::Quality, context: why.to_string()),
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
                    let uri = Some(path.file_name_with_parent());
                    let data = Arc::new(data);
                    collect_validation_checks(data.as_ref())
                        .into_iter()
                        .map(|issue| issue.with_uri(uri.clone()))
                        .collect::<Vec<_>>()
                }
                | Err(why) => why
                    .to_string()
                    .lines()
                    .map(|line| check_err!(CheckCategory::Schema, context: line.to_string()).with_uri(Some(path.file_name_with_parent())))
                    .collect(),
            })
            .collect()
    }
    #[cfg(feature = "std")]
    async fn check_websites(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        vec![]
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
}
#[cfg(feature = "analysis")]
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
                | Err(why) => check_err!(CheckCategory::Quality, context: why.to_string()),
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
#[cfg(feature = "analysis")]
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
                | Err(why) => check_err!(CheckCategory::Quality, context: why.to_string()),
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
                    let uri = Some(path.file_name_with_parent());
                    let data = Arc::new(data);
                    collect_validation_checks(data.as_ref())
                        .into_iter()
                        .map(|issue| issue.with_uri(uri.clone()))
                        .collect::<Vec<_>>()
                }
                | Err(why) => why
                    .to_string()
                    .lines()
                    .map(|line| check_err!(CheckCategory::Schema, context: line.to_string()).with_uri(Some(path.file_name_with_parent())))
                    .collect(),
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
                        let data = Arc::new(data);
                        let ResearchActivity { meta, contact, .. } = data.as_ref();
                        let dois = match &meta.doi {
                            | Some(values) => values
                                .iter()
                                .enumerate()
                                .map(|(i, doi)| {
                                    let url = format!("https://doi.org/{doi}");
                                    link_check(Some(url), Some(format!("meta.doi[{i}]")))
                                })
                                .collect::<Vec<_>>(),
                            | None => vec![],
                        };
                        let websites = match &meta.websites {
                            | Some(values) => values
                                .iter()
                                .enumerate()
                                .map(|(i, Website { url, .. })| link_check(Some(url.as_str()), Some(format!("meta.websites[{i}].url"))))
                                .collect::<Vec<_>>(),
                            | None => vec![],
                        };
                        let contact = [link_check(Some(contact.url.as_str()), Some("contact.url".into()))];
                        let links: Vec<_> = [].into_iter().chain(dois).chain(websites).chain(contact).collect();
                        Self::collect_checks(links, &path).await
                    }
                    | Err(why) => vec![check_err!(CheckCategory::Link, context: why.to_string()).with_uri(Some(path.file_name_with_parent()))],
                }
            }
        });
        Self::flatten_checks(futures).await
    }
    fn output_path(path: &Path, data: &Self) -> PathBuf {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("index.json");
        standard_project_folder("check", None)
            .join(data.meta.identifier.to_lowercase())
            .join(filename)
    }
}
#[cfg(feature = "analysis")]
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
                | Err(why) => check_err!(CheckCategory::Quality, context: why.to_string()),
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
#[cfg(feature = "analysis")]
#[async_trait]
impl Analysis for raid::Metadata {
    fn standard() -> Standard {
        Standard::Raid
    }
    async fn check_prose(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        vec![]
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        paths
            .par_iter()
            .map(|path| match Self::read(path.to_path_buf()) {
                | Ok(_) => check_ok!(CheckCategory::Quality),
                | Err(why) => check_err!(CheckCategory::Quality, context: why.to_string()),
            })
            .collect()
    }
    async fn check_readability(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        vec![]
    }
    async fn check_schema(paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        paths
            .par_iter()
            .flat_map(|path| match Self::read(path.to_path_buf()) {
                | Ok(data) => {
                    let uri = Some(path.file_name_with_parent());
                    collect_validation_checks(&data)
                        .into_iter()
                        .map(|issue| issue.with_uri(uri.clone()))
                        .collect::<Vec<_>>()
                }
                | Err(why) => why
                    .to_string()
                    .lines()
                    .map(|line| check_err!(CheckCategory::Schema, context: line.to_string()).with_uri(Some(path.file_name_with_parent())))
                    .collect(),
            })
            .collect()
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
    fn command(&self) -> String {
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
        let uri = path.file_name_with_parent();
        let root = path
            .parent()
            .map(|value| value.to_path_buf())
            .unwrap_or_else(|| standard_project_folder("check", None));
        match create_dir_all(root.clone()) {
            | Ok(_) => {}
            | Err(why) => error!(path = root.clone().to_absolute_path(), "=> {} Create — {why}", Label::fail()),
        }
        match write(&path, content.as_bytes()) {
            | Ok(_) => {}
            | Err(why) => {
                error!(path = path.to_absolute_path(), "=> {} Write file — {why}", Label::fail());
                return vec![check_err!(
                    CheckCategory::Prose,
                    uri: uri,
                    message: format!("Cannot write analyzer input at {}", path.display())
                )];
            }
        }
        let binary = match &self.binary {
            | Some(value) => value,
            | None => {
                error!("=> {} {} binary", Label::not_found(), self.command());
                &PathBuf::from("./.vale/vale")
            }
        };
        match &self.config {
            | Some(config) => {
                let binary_path = binary.to_absolute_path();
                let config_path = config.clone().path.to_absolute_path();
                let path_string = path.clone().to_absolute_path();
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
                let title = self.command().to_case(Case::Title);
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
        let url = format!("{VALE_RELEASES_URL}/download/v{release}/{}_{release}_{platform}", self.command());
        info!(url, "=> {} Vale release v{release}", Label::using());
        let binary = match download_binary(&url, ".").await {
            | Ok(path) => {
                if !skip_verify_checksum {
                    let dowloaded_checksum = match self.clone().download_checksums().await {
                        | Ok(value) => value.get(&platform).unwrap_or(&String::new()).to_string(),
                        | Err(_) => "".to_string(),
                    };
                    if let Some(calculated) = file_checksum(path.clone(), None) {
                        if !dowloaded_checksum.eq(&calculated.checksum_value) {
                            error!(dowloaded_checksum, calculated = calculated.checksum_value, "=> {}", Label::invalid());
                            let _cleanup = remove_file(path.clone());
                        } else {
                            info!(checksum = dowloaded_checksum, "=> {} Checksum verification", Label::pass());
                        }
                    };
                } else {
                    skip!("Checksum verification");
                }
                let destination = match config.clone() {
                    | Some(value) => value.path.parent().map(Path::to_path_buf).unwrap_or(PathBuf::from("./.vale/")),
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
        builder.config(config.unwrap_or_default()).build()
    }
    async fn download_checksums(self) -> ApiResult<HashMap<String, String>> {
        let release = match self.version {
            | Some(value) => value,
            | None => SemanticVersion::from(VALE_VERSION),
        };
        let url = format!("{VALE_RELEASES_URL}/download/v{release}/{}_{release}_checksums.txt", self.command());
        let checksums = match get(url).send().await {
            | Ok(response) => match response.text().await {
                | Ok(content) => Ok(content.lines().clone().fold(HashMap::new(), |mut acc: HashMap<String, String>, line| {
                    let mut values: Vec<&str> = line.split("  ").collect();
                    if let (Some(raw_key), Some(raw_value)) = (values.pop(), values.pop()) {
                        let key = raw_key["vale_#.#.#_".len()..].to_string();
                        acc.insert(key, raw_value.to_string());
                    }
                    acc
                })),
                | Err(why) => Err(eyre!("Failed to read checksums response — {why}")),
            },
            | Err(why) => Err(eyre!("Failed to download checksums — {why}")),
        };
        match checksums {
            | Ok(checksums) => {
                debug!(
                    "=> {} {} checksums {:#?}",
                    Label::using(),
                    self.command().to_case(Case::Title),
                    checksums.dimmed().cyan()
                );
                Ok(checksums)
            }
            | Err(why) => Err(why),
        }
    }
    fn extract(self, path: PathBuf, destination: Option<PathBuf>) -> PathBuf {
        let command = self.command();
        let parent = match destination {
            | Some(value) => value.to_absolute_path(),
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
                    error!("=> {} {command} extract — {why}", Label::fail());
                    let path = PathBuf::from(DEFAULT_VALE_ROOT).join(command);
                    if cfg!(windows) {
                        path.with_extension("exe")
                    } else {
                        path
                    }
                }
            },
            | "gz" => match File::open(path) {
                | Ok(tar_gz) => {
                    let tar = GzDecoder::new(tar_gz);
                    let mut archive = Archive::new(tar);
                    match archive.unpack(parent.clone()) {
                        | Ok(_) => {
                            debug!(parent, "=> {} Extracted {command} binary", Label::using());
                            PathBuf::from(format!("{parent}/{command}"))
                        }
                        | Err(why) => {
                            error!("=> {} {command} extract — {why}", Label::fail());
                            let path = PathBuf::from(DEFAULT_VALE_ROOT).join(command);
                            if cfg!(windows) {
                                path.with_extension("exe")
                            } else {
                                path
                            }
                        }
                    }
                }
                | Err(why) => {
                    error!("=> {} {command} extract — {why}", Label::fail());
                    let path = PathBuf::from(DEFAULT_VALE_ROOT).join(command);
                    if cfg!(windows) {
                        path.with_extension("exe")
                    } else {
                        path
                    }
                }
            },
            | _ => {
                error!("=> {} {command} extract — Unsupported format", Label::fail());
                PathBuf::from(DEFAULT_VALE_ROOT).join(command)
            }
        }
    }
    async fn sync(self, is_offline: bool, quiet: bool) -> ApiResult<()> {
        let command = self.command();
        let binary_path = match self.binary {
            | Some(value) => value,
            | None => {
                error!("=> {} {} binary", Label::not_found(), command);
                PathBuf::from(DEFAULT_VALE_ROOT).join(command)
            }
        };
        let config_path = self.config.unwrap_or_default().path;
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
                let parent_dir = config_path.parent().map(|p| p.display().to_string()).unwrap_or_default();
                let parent = format!("{parent_dir}/styles/config/vocabularies/{APPLICATION}");
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
                    | Ok(_) => {
                        let reject_content = Constant::read_lines("reject.txt").join("\n");
                        match write(format!("{parent}/reject.txt"), reject_content.as_bytes()) {
                            | Ok(_) => Ok(()),
                            | Err(why) => Err(eyre!("Write reject.txt failed — {why}")),
                        }
                    }
                    | Err(why) => Err(eyre!("Write accept.txt failed — {why}")),
                }
            }
            | Err(why) => {
                error!(config = config_path.to_absolute_path(), "=> {} Vale sync — {why}", Label::fail());
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
        let name = self.command();
        if command_exists(name.clone()) {
            match which(name.clone()) {
                | Ok(path) => {
                    let path = path.to_path_buf();
                    self.binary = Some(path.clone());
                    match cmd!(name.clone(), ["--version"]) {
                        | Ok(output) => {
                            let stdout = output.stdout();
                            let version = stdout.strip_prefix("vale version ").unwrap_or(stdout.as_str()).trim().to_string();
                            self.version = Some(SemanticVersion::from(version.as_ref()));
                            debug!(
                                path = path.to_absolute_path(),
                                "=> {} System {} (v{version}) command",
                                Label::using(),
                                name.green().bold(),
                            );
                        }
                        | Err(why) => {
                            error!("=> {} Resolve {name} version — {why}", Label::fail());
                        }
                    }
                }
                | Err(why) => {
                    error!("=> {} Resolve {name} binary — {why}", Label::fail());
                }
            }
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
        let parent = path.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));
        match create_dir_all(parent.clone()) {
            | Ok(_) => {}
            | Err(why) => error!(directory = parent.to_absolute_path(), "=> {} Create — {why}", Label::fail()),
        }
        match self.clone().ini().await.write_to_file(path.clone()) {
            | Ok(_) => {
                debug!(path = path.to_absolute_path(), "=> {} Saved configuration", Label::using());
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
pub(crate) fn collect_validation_checks<T>(value: &T) -> Vec<Check>
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
pub fn link_check<'a, T>(uri: Option<T>, locator: Option<String>) -> BoxFuture<'a, Check>
where
    T: Into<String> + Send + 'a,
{
    async move {
        match uri {
            | Some(value) => {
                let context = value.into();
                let result = lychee_lib::check(context.as_str()).await;
                match result {
                    | Ok(response) => convert_lychee_response(response).with_context(context).with_locator(locator),
                    | Err(_) => check_err!(CheckCategory::Link, context: context, message: "Unreachable").with_locator(locator),
                }
            }
            | None => check_err!(CheckCategory::Link, message: "Missing URL").with_locator(locator),
        }
    }
    .boxed()
}
/// Prints `text` to stdout using syntax highlighting for the specified `syntax`.
///
/// `highlight` is an iterator of line numbers to highlight in the output.
#[cfg(feature = "analysis")]
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
    #[allow(clippy::unwrap_used)]
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
