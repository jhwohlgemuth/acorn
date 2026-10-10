//! # Prose analyzer module
//!
//! This is where we keep functions and interfaces necessary to execute ACORN's automated editorial style guide as well as content readability analyzer.
//!
use super::{
    context::{AnalysisContext, ExecutionOptions},
    service::{check_fair_for, check_prose_for, check_quality_for, check_readability_for, Analysis, SchemaCheck},
    vale::{self, try_parse_vale_output, Vale, ValeConfig},
};
use crate::io::config::application::ApplicationConfiguration;
#[cfg(any(feature = "analysis", feature = "std"))]
use crate::io::document::SourceDocument;
#[cfg(any(feature = "analysis", feature = "std"))]
use crate::io::http::get;
#[cfg(any(feature = "analysis", feature = "std"))]
use crate::io::InputOutput;
#[cfg(any(feature = "analysis", feature = "std"))]
use crate::io::{download_binary, extract, standard_project_folder, ApiResult};
use crate::{
    analyzer::{
        check::{Check, CheckBatch, CheckCategory, CheckKind, CheckSeverity, ValidationReportExt},
        error::ErrorKind,
        fair::FairEvidence,
        gpu::check_gpu_requirements,
        vale::ValeOutputItem,
    },
    skip,
    util::{
        constants::{
            app::{APPLICATION, ONLINE_SYNC_ATTEMPTS},
            vale::{CUSTOM_VALE_PACKAGE_NAME, DEFAULT_VALE_PACKAGE_URL, VALE_PACKAGE_REPOSITORY_URL, VALE_RELEASES_URL, VALE_VERSION},
        },
        Constant, StringConversion,
    },
    RepositoryExt,
};
use acorn_analysis::{check, check_err, check_ok};
use acorn_cmd::{args, cmd, command_exists, CommandOutput};
use acorn_core::{
    prelude::{Arc, HashMap},
    util::{assets::EmbeddedAssets, SemanticVersion},
    Location, Repository,
};
use acorn_host::{
    fs::{file_checksum, make_executable, SafePath},
    terminal::Label,
};
use acorn_schema::{
    modelkit::kitfile::{Kitfile, ModelKitError},
    pid::{raid, Identifier, PersistentIdentifier, PersistentIdentifierParse, DOI, PID},
    research_activity::ResearchActivity,
    standard::{
        cff::{self, Cff, IdentifierType, Reference},
        ckan, datacite, dcat, huwise, invenio,
        text::{Document, Text},
        Standard,
    },
    util::assets::Constant as SchemaConstant,
    validation::Validate,
    vocabulary::{VocabularyFuzzyMode, VocabularyKind},
    Organization, ProgrammingLanguage, Website,
};
use alloc::collections::BTreeSet;
use async_trait::async_trait;
#[cfg(feature = "analysis")]
use bat::PrettyPrinter;
use color_eyre::{eyre::eyre, owo_colors::OwoColorize};
use convert_case::{Case, Casing};
use core::iter::once;
use flate2::read::GzDecoder;
use futures::future::{join_all, BoxFuture, FutureExt};
use ini::Ini;
use is_executable::IsExecutable;
use lychee_lib::{CacheStatus, Response, Status};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs::{create_dir_all, remove_file, write, File},
    io::{ErrorKind as IoErrorKind, Read},
    path::{Path, PathBuf},
};
use tar::Archive;
#[cfg(any(feature = "analysis", feature = "std"))]
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use which::which;
use zip::ZipArchive;

macro_rules! impl_schema_check {
    ($($schema:ty),+ $(,)?) => {
        $(
            impl SchemaCheck for $schema {
                fn check_schema_path(path: &PathBuf) -> Option<BoxFuture<'static, CheckBatch>> {
                    Some(check_schema_path(path.clone(), Self::read(path.to_path_buf())).boxed())
                }
            }
        )+
    };
}
pub(crate) trait CheckBatchExt {
    fn evaluate_link_evidence(
        values: impl IntoIterator<Item = LinkEvidence>,
        reachability_enabled: bool,
    ) -> impl core::future::Future<Output = CheckBatch>;
}
trait KitfileSchemaCheck {
    fn check_schema_path(path: PathBuf, document: ApiResult<Self>) -> BoxFuture<'static, CheckBatch>
    where
        Self: Sized;
}
/// Trait for static analyzers (e.g. Vale)
#[async_trait]
pub trait StaticAnalyzer<Config: StaticAnalyzerConfig> {
    /// Get command name (e.g. "vale")
    fn command(&self) -> String;
    /// Download binary
    async fn download(self, config: Option<Config>, skip_verify_checksum: bool) -> ApiResult<Self>
    where
        Self: Sized;
    /// Download checksum values
    async fn download_checksums(self) -> ApiResult<HashMap<String, String>>;
    /// Extract binary
    fn extract(self, path: PathBuf, destination: Option<PathBuf>) -> ApiResult<PathBuf>;
    /// Resolve analyzer
    async fn resolve(_config: Config, _is_offline: bool, _skip_verify_checksum: bool) -> ApiResult<Self>
    where
        Self: Sized;
    /// Run analyzer on content
    async fn run(&self, path: PathBuf, content: String, output: Option<String>) -> CheckBatch;
    /// Perform synchronization when supported by the analyzer
    async fn sync(self, is_offline: bool, quiet: bool) -> ApiResult<()>;
    /// Retry synchronization when supported by the analyzer.
    #[cfg(any(feature = "analysis", feature = "std"))]
    async fn retry_sync(self, options: &ExecutionOptions) -> ApiResult<()>
    where
        Self: Sized,
    {
        self.sync(options.common.offline, options.common.quiet).await
    }
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
    /// Save configuration while respecting offline execution policy.
    async fn save(self, is_offline: bool) -> ApiResult<Self>
    where
        Self: Sized;
    /// Resolve package source: convert bare package name to download URL or pass through existing URL/path
    fn resolve_package(value: impl AsRef<str>) -> String;
}
impl_schema_check!(
    Cff,
    ckan::Package,
    Document,
    Text,
    datacite::Record,
    dcat::Dataset,
    invenio::Record,
    huwise::Dataset,
    ResearchActivity
);
pub(super) struct AcreStandard;
#[derive(Clone, Copy)]
struct ContentPaths<'a> {
    kitfile: &'a Kitfile,
}
pub(super) struct KitfileStandard;
#[derive(Clone, Debug)]
pub(crate) struct LinkEvidence {
    diagnostic: Option<String>,
    locator: Option<String>,
    outcome: Option<LinkOutcome>,
    source_uri: Option<String>,
    target: Option<String>,
}
#[derive(Clone, Debug)]
pub(crate) struct LinkOutcome {
    kind: CheckKind,
    message: String,
    severity: CheckSeverity,
    status_code: Option<String>,
    success: bool,
}
pub(super) struct ModelKitStandard;
#[cfg(feature = "analysis")]
#[async_trait]
impl Analysis for AcreStandard {
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
    fn standard() -> Standard {
        Standard::ApplicationConfigurationResearchEnablement
    }
}
impl SchemaCheck for AcreStandard {
    fn check_schema_path(path: &PathBuf) -> Option<BoxFuture<'static, CheckBatch>> {
        let uri = Some(path.file_name_with_parent());
        let checks = match ApplicationConfiguration::read(path.to_path_buf()) {
            | Ok(data) => match data.validate() {
                | Ok(()) => CheckBatch::empty(),
                | Err(report) => report.checks(uri),
            },
            | Err(why) => CheckBatch::schema_parse_failure_checks(why, uri),
        };
        Some(futures::future::ready(checks).boxed())
    }
}
#[cfg(feature = "analysis")]
#[async_trait]
impl Analysis for Cff {
    fn standard() -> Standard {
        Standard::CitationFileFormat
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        check_quality_for::<Self>(paths)
    }
    async fn check_readability(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_readability_for::<Self>(paths, options)
    }
    #[cfg(any(feature = "analysis", feature = "std"))]
    async fn check_links(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
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
                            .filter_map(|(i, cff::Identifier { kind, value, .. })| match kind {
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
                    | Err(why) => CheckCategory::Link.failure_checks(why, Some(path.file_name_with_parent())),
                }
            }
        });
        Self::flatten_checks(futures).await
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
}
impl From<LinkEvidence> for Check {
    fn from(value: LinkEvidence) -> Self {
        let LinkEvidence {
            diagnostic,
            locator,
            outcome,
            source_uri,
            ..
        } = value;
        let outcome = outcome.unwrap_or_else(|| LinkOutcome::failure("Missing URL"));
        Self::init()
            .category(CheckCategory::Link)
            .success(outcome.success)
            .severity(outcome.severity)
            .message(outcome.message)
            .kind(outcome.kind)
            .maybe_context(diagnostic)
            .maybe_locator(locator)
            .maybe_status_code(outcome.status_code)
            .maybe_uri(source_uri)
            .build()
    }
}
impl CheckBatchExt for CheckBatch {
    async fn evaluate_link_evidence(values: impl IntoIterator<Item = LinkEvidence>, reachability_enabled: bool) -> Self {
        let futures = values
            .into_iter()
            .filter_map(|value| match (value.outcome.is_some(), reachability_enabled) {
                | (true, _) => Some(async move { Check::from(value) }.boxed()),
                | (false, true) => Some(value.evaluate().boxed()),
                | (false, false) => None,
            });
        join_all(futures).await.into_iter().collect()
    }
}
#[cfg(feature = "analysis")]
#[async_trait]
impl Analysis for ckan::Package {
    fn standard() -> Standard {
        Standard::Ckan
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
}
impl<'a> ContentPaths<'a> {
    fn new(kitfile: &'a Kitfile) -> Self {
        Self { kitfile }
    }
    fn iter(self) -> impl Iterator<Item = (String, &'a str, bool)> + 'a {
        self.model()
            .chain(self.code())
            .chain(self.datasets())
            .chain(self.docs())
            .chain(self.prompts())
            .chain(self.mcp_servers())
    }
    fn model(self) -> impl Iterator<Item = (String, &'a str, bool)> + 'a {
        self.kitfile.model.iter().flat_map(|model| {
            let primary = (!model.path.trim().is_empty() && Location::from(&model.path).is_path())
                .then(|| ("model.path".to_string(), model.path.as_str(), false));
            let parts = Self::indexed("model.parts", model.parts.iter().enumerate(), |part| part.path.as_str(), false);
            primary.into_iter().chain(parts)
        })
    }
    fn code(self) -> impl Iterator<Item = (String, &'a str, bool)> + 'a {
        Self::indexed("code", self.kitfile.code.iter().enumerate(), |layer| layer.path.as_str(), false)
    }
    fn datasets(self) -> impl Iterator<Item = (String, &'a str, bool)> + 'a {
        Self::indexed(
            "datasets",
            self.kitfile
                .datasets
                .iter()
                .enumerate()
                .filter(|(_, layer)| layer.remote_path.trim().is_empty()),
            |layer| layer.path.as_str(),
            false,
        )
    }
    fn docs(self) -> impl Iterator<Item = (String, &'a str, bool)> + 'a {
        Self::indexed("docs", self.kitfile.docs.iter().enumerate(), |layer| layer.path.as_str(), false)
    }
    fn prompts(self) -> impl Iterator<Item = (String, &'a str, bool)> + 'a {
        Self::indexed("prompts", self.kitfile.prompts.iter().enumerate(), |layer| layer.path.as_str(), false)
    }
    fn mcp_servers(self) -> impl Iterator<Item = (String, &'a str, bool)> + 'a {
        Self::indexed(
            "mcpServers",
            self.kitfile.mcp_servers.iter().enumerate(),
            |layer| layer.path.as_str(),
            true,
        )
    }
    fn indexed<T: 'a>(
        section: &'static str,
        values: impl Iterator<Item = (usize, &'a T)> + 'a,
        path: impl Fn(&'a T) -> &'a str + 'a,
        mcp_bundle: bool,
    ) -> impl Iterator<Item = (String, &'a str, bool)> + 'a {
        values.map(move |(index, value)| (format!("{section}[{index}].path"), path(value), mcp_bundle))
    }
}
#[cfg(feature = "analysis")]
#[async_trait]
impl Analysis for datacite::Record {
    fn standard() -> Standard {
        Standard::Datacite
    }
    async fn check_fair(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        check_fair_for::<Self>(paths, "DataCite 4.7")
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        check_quality_for::<Self>(paths)
    }
    async fn check_readability(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_readability_for::<Self>(paths, options)
    }
    #[cfg(any(feature = "analysis", feature = "std"))]
    async fn check_links(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
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
                    | Err(why) => CheckCategory::Link.failure_checks(why, Some(path.file_name_with_parent())),
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
    async fn check_fair(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        check_fair_for::<Self>(paths, "DCAT 3")
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        check_quality_for::<Self>(paths)
    }
    async fn check_readability(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_readability_for::<Self>(paths, options)
    }
    #[cfg(any(feature = "analysis", feature = "std"))]
    async fn check_links(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
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
                    | Err(why) => CheckCategory::Link.failure_checks(why, Some(path.file_name_with_parent())),
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
impl Analysis for Document {
    fn standard() -> Standard {
        Standard::Document
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        paths
            .par_iter()
            .flat_map(|path| match SourceDocument::from_path(path) {
                | Ok(document) if document.warnings().is_empty() => vec![check_ok!(CheckCategory::Quality, uri: path.file_name_with_parent())],
                | Ok(document) => document
                    .warnings()
                    .iter()
                    .map(|warning| {
                        check!(
                            CheckCategory::Quality,
                            false,
                            severity: CheckSeverity::Warning,
                            message: warning.clone(),
                            uri: path.file_name_with_parent()
                        )
                    })
                    .collect(),
                | Err(why) => CheckCategory::Quality.failure_checks(why, Some(path.file_name_with_parent())).into_vec(),
            })
            .collect::<Vec<_>>()
            .into()
    }
    async fn check_readability(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_readability_for::<Self>(paths, options)
    }
    #[cfg(any(feature = "analysis", feature = "std"))]
    async fn check_links(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        let futures = paths.iter().map(|path| async move {
            match Self::read(path) {
                | Ok(document) => {
                    Self::collect_checks(
                        Identifier::find_all(&document.content)
                            .into_iter()
                            .filter_map(|identifier| (identifier.kind == PID::URL).then_some(identifier.value))
                            .map(|url| link_check(Some(url), None)),
                        path,
                    )
                    .await
                }
                | Err(why) => CheckCategory::Link.failure_checks(why, Some(path.file_name_with_parent())),
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
    async fn check_fair(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        check_fair_for::<Self>(paths, "Huwise")
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        check_quality_for::<Self>(paths)
    }
    async fn check_readability(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_readability_for::<Self>(paths, options)
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
    async fn check_fair(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        check_fair_for::<Self>(paths, "InvenioRDM")
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        check_quality_for::<Self>(paths)
    }
    async fn check_readability(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_readability_for::<Self>(paths, options)
    }
    #[cfg(any(feature = "analysis", feature = "std"))]
    async fn check_links(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
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
                    | Err(why) => CheckCategory::Link.failure_checks(why, Some(path.file_name_with_parent())),
                }
            }
        });
        Self::flatten_checks(futures).await
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
}
impl KitfileSchemaCheck for Kitfile {
    fn check_schema_path(path: PathBuf, document: ApiResult<Self>) -> BoxFuture<'static, CheckBatch> {
        async move {
            let uri = Some(path.file_name_with_parent());
            match document {
                | Ok(data) => collect_validation_checks(&data)
                    .await
                    .into_iter()
                    .map(|issue| issue.with_uri(uri.clone()))
                    .collect(),
                | Err(why) => match why.downcast::<ModelKitError>() {
                    | Ok(ModelKitError::Validation(report)) => report.checks(uri),
                    | Ok(why) => CheckBatch::schema_parse_failure_checks(why, uri),
                    | Err(why) => CheckBatch::schema_parse_failure_checks(why, uri),
                },
            }
        }
        .boxed()
    }
}
#[cfg(feature = "analysis")]
#[async_trait]
impl Analysis for KitfileStandard {
    async fn check_quality(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        paths
            .par_iter()
            .flat_map(|path| match Kitfile::read_yaml(path.to_path_buf()) {
                | Ok(data) => modelkit_quality_checks(path, &data, true),
                | Err(_) => Vec::new(),
            })
            .collect::<Vec<_>>()
            .into()
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
    fn standard() -> Standard {
        Standard::Kitfile
    }
}
impl SchemaCheck for KitfileStandard {
    fn check_schema_path(path: &PathBuf) -> Option<BoxFuture<'static, CheckBatch>> {
        Some(Kitfile::check_schema_path(path.clone(), Kitfile::read_yaml(path.to_path_buf())))
    }
}
impl LinkEvidence {
    pub(crate) fn observed(target: impl Into<String>, diagnostic: Option<String>, source_uri: Option<String>, outcome: LinkOutcome) -> Self {
        let target = target.into();
        Self {
            diagnostic,
            locator: Some(target.clone()),
            outcome: Some(outcome),
            source_uri,
            target: Some(target),
        }
    }
    pub(crate) fn pending(target: Option<String>, locator: Option<String>, source_uri: Option<String>) -> Self {
        Self {
            diagnostic: None,
            locator,
            outcome: None,
            source_uri,
            target,
        }
    }
    async fn evaluate(self) -> Check {
        match self.target.as_deref() {
            | Some(target) => {
                let outcome = match lychee_lib::check(target).await {
                    | Ok(response) => convert_lychee_response(response),
                    | Err(_) => LinkOutcome::failure("Unreachable"),
                };
                Self {
                    diagnostic: Some(target.to_string()),
                    outcome: Some(outcome),
                    ..self
                }
                .into()
            }
            | None => self.into(),
        }
    }
}
impl LinkOutcome {
    pub(crate) fn lifecycle(success: bool, severity: CheckSeverity, message: impl Into<String>) -> Self {
        Self {
            kind: CheckKind::Lifecycle,
            message: message.into(),
            severity,
            status_code: None,
            success,
        }
    }
    fn standard(success: bool, severity: CheckSeverity, message: impl Into<String>, status_code: Option<String>) -> Self {
        Self {
            kind: CheckKind::Standard,
            message: message.into(),
            severity,
            status_code,
            success,
        }
    }
    fn failure(message: impl Into<String>) -> Self {
        Self::standard(false, CheckSeverity::Error, message, None)
    }
}
#[cfg(feature = "analysis")]
#[async_trait]
impl Analysis for ModelKitStandard {
    async fn check_quality(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        paths
            .par_iter()
            .flat_map(|path| match Kitfile::read_json(path.to_path_buf()) {
                | Ok(data) => modelkit_quality_checks(path, &data, false),
                | Err(_) => Vec::new(),
            })
            .collect::<Vec<_>>()
            .into()
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
    fn standard() -> Standard {
        Standard::ModelKit
    }
}
impl SchemaCheck for ModelKitStandard {
    fn check_schema_path(path: &PathBuf) -> Option<BoxFuture<'static, CheckBatch>> {
        Some(Kitfile::check_schema_path(path.clone(), Kitfile::read_json(path.to_path_buf())))
    }
}
#[cfg(feature = "analysis")]
#[async_trait]
impl Analysis for raid::Metadata {
    fn standard() -> Standard {
        Standard::Raid
    }
    async fn check_fair(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        check_fair_for::<Self>(paths, "RAiD metadata 1.6")
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        check_quality_for::<Self>(paths)
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        standard_project_folder("check", None).join(path.to_path_buf().file_name_with_parent())
    }
}
impl SchemaCheck for raid::Metadata {
    fn check_schema_path(path: &PathBuf) -> Option<BoxFuture<'static, CheckBatch>> {
        Some(check_schema_path(path.clone(), Self::read(path.to_path_buf())).boxed())
    }
}
#[cfg(feature = "analysis")]
#[async_trait]
impl Analysis for ResearchActivity {
    fn standard() -> Standard {
        Standard::ResearchActivityData
    }
    async fn check_compute(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        match options.and_then(|options| options.analysis.gpu_inventory.as_ref()) {
            | Some(inventory) => paths
                .par_iter()
                .map(|path| {
                    let uri = path.file_name_with_parent();
                    match Self::read(path) {
                        | Ok(data) => data.meta.resources.as_ref().map_or_else(CheckBatch::empty, |resources| {
                            check_gpu_requirements(resources.as_slice(), inventory, Some(uri))
                        }),
                        | Err(why) => CheckCategory::Compute.failure_checks(why, Some(uri)),
                    }
                })
                .collect::<Vec<_>>()
                .into_iter()
                .collect(),
            | None => CheckBatch::empty(),
        }
    }
    async fn check_fair(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        paths
            .par_iter()
            .flat_map(|path| {
                let uri = path.file_name_with_parent();
                match Self::read(path) {
                    | Ok(data) => match data.meta.outputs.as_ref() {
                        | Some(outputs) if !outputs.is_empty() => {
                            let activity = serde_json::to_value(&data.meta).ok();
                            outputs
                                .iter()
                                .enumerate()
                                .flat_map(|(index, output)| {
                                    serde_json::to_value(output)
                                        .ok()
                                        .map(|mut value| {
                                            if let (Some(values), Some(activity)) = (value.as_object_mut(), activity.clone()) {
                                                values.insert("activityMetadata".to_string(), activity);
                                            }
                                            FairEvidence::from_value(format!("{uri}#meta.outputs[{index}]"), "Research Activity Data", &value)
                                                .assess()
                                                .into_iter()
                                                .map(|finding| Check::from(finding).with_uri(Some(uri.clone())))
                                                .collect::<Vec<_>>()
                                        })
                                        .unwrap_or_else(|| vec![check_err!(CheckCategory::Fair, uri: uri.clone())])
                                })
                                .collect()
                        }
                        | None | Some(_) => vec![check!(
                            CheckCategory::Fair,
                            false,
                            severity: CheckSeverity::Suggestion,
                            uri: uri,
                            message: "Add structured meta.outputs entries to assess FAIR support"
                        )],
                    },
                    | Err(why) => CheckCategory::Fair.failure_checks(why, Some(uri)).into_vec(),
                }
            })
            .collect::<Vec<_>>()
            .into()
    }
    async fn check_prose(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        let prose = check_prose_for::<Self>(paths, options).await;
        let fuzzy = options.map_or(VocabularyFuzzyMode::Relaxed, |value| value.analysis.vocabulary_fuzzy);
        let vocabulary = paths
            .par_iter()
            .map(|path| match Self::read(path) {
                | Ok(data) => vocabulary_checks(path, &data, fuzzy),
                | Err(_) => CheckBatch::empty(),
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect();
        prose.combine(vocabulary)
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        check_quality_for::<Self>(paths)
    }
    async fn check_readability(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_readability_for::<Self>(paths, options)
    }
    #[cfg(any(feature = "analysis", feature = "std"))]
    async fn check_links(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
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
                    | Err(why) => CheckCategory::Link.failure_checks(why, Some(path.file_name_with_parent())),
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
    async fn check_prose(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_prose_for::<Self>(paths, options).await
    }
    async fn check_quality(paths: &[PathBuf], _options: Option<&AnalysisContext>) -> CheckBatch {
        check_quality_for::<Self>(paths)
    }
    async fn check_readability(paths: &[PathBuf], options: Option<&AnalysisContext>) -> CheckBatch {
        check_readability_for::<Self>(paths, options)
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
    /// - Will use an explicitly configured Vale directory without modifying it
    /// - Will use the system `vale` command if available
    /// - Will use the cached binary if available
    /// - Will download `vale` only when online and not available by other means
    async fn resolve(config: ValeConfig, is_offline: bool, skip_verify_checksum: bool) -> ApiResult<Vale> {
        let init = Vale::init().build();
        let name = init.command();
        let custom_directory = config.custom_directory();
        let root = match custom_directory {
            | true => config
                .path
                .parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| eyre!("Custom Vale configuration has no parent directory")),
            | false => init.managed_directory(),
        };
        match root {
            | Ok(root) => {
                let config = match custom_directory {
                    | true => Ok(config.with_custom_directory(true)),
                    | false => {
                        config
                            .with_custom_directory(false)
                            .with_path(root.join(".vale.ini"))
                            .save(is_offline)
                            .await
                    }
                };
                match config {
                    | Ok(config) => {
                        let cached_binary = root.join(&name);
                        let cached_binary = match cfg!(windows) {
                            | true => cached_binary.with_extension("exe"),
                            | false => cached_binary,
                        };
                        if custom_directory {
                            match (cached_binary.is_executable(), config.path.is_file()) {
                                | (true, true) => Ok(init.with_config(config).with_binary(cached_binary)),
                                | (false, _) => Err(eyre!(
                                    "Custom Vale executable is missing or is not executable: {}",
                                    cached_binary.display()
                                )),
                                | (_, false) => Err(eyre!("Custom Vale configuration does not exist: {}", config.path.display())),
                            }
                        } else if command_exists(&name) {
                            let vale = init.with_config(config).with_system_command();
                            match vale.binary.as_ref().is_some_and(|binary| binary.is_executable()) {
                                | true => Ok(vale),
                                | false => Err(eyre!("Failed to resolve the system Vale executable")),
                            }
                        } else if cached_binary.is_executable() {
                            info!("=> {} Local {} binary", Label::using(), name.green().bold());
                            Ok(init.with_config(config).with_binary(cached_binary))
                        } else if is_offline {
                            Err(eyre!(
                                "Vale is unavailable in offline mode; install it on PATH, cache it at {}, or use --vale-directory",
                                root.display()
                            ))
                        } else {
                            init.download(Some(config), skip_verify_checksum).await
                        }
                    }
                    | Err(why) => Err(why),
                }
            }
            | Err(why) => Err(why),
        }
    }
    async fn run(&self, path: PathBuf, content: String, output_format: Option<String>) -> CheckBatch {
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
                )]
                .into();
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
                    | Ok(output) if output.status.success() => match try_parse_vale_output(&output.stdout(), path.clone()) {
                        | Ok(parsed) if parsed.is_empty() => {
                            info!("=> {} {} has {}", Label::pass(), uri.underline(), "no prose issues".green());
                            CheckBatch::single(check_ok!(CheckCategory::Prose, uri: uri))
                        }
                        | Ok(parsed) => parsed
                            .into_iter()
                            .map(|item| {
                                let ValeOutputItem { message, severity, .. } = item.clone();
                                check!(
                                    CheckCategory::Prose,
                                    false,
                                    severity: severity.into(),
                                    uri: uri.clone(),
                                    message: message,
                                    errors: ErrorKind::Vale(vec![item.clone()]),
                                    data: content.clone(),
                                )
                            })
                            .collect(),
                        | Err(why) => CheckCategory::Prose.failure_checks(why, Some(uri)),
                    },
                    | Ok(output) => {
                        let why = output.stderr();
                        let message = if why.is_empty() {
                            format!("process exited with status {}", output.status)
                        } else {
                            why
                        };
                        error!("=> {} Analyze with OK output — {message}", Label::fail());
                        CheckBatch::single(check_err!(CheckCategory::Prose, message: message))
                    }
                    | Err(why) => {
                        error!("=> {} Analyze — {why}", Label::fail());
                        CheckBatch::single(check_err!(CheckCategory::Prose, uri: uri))
                    }
                }
            }
            | None => {
                let title = self.command().to_case(Case::Title);
                error!("=> {} {} configuration", Label::not_found(), title);
                CheckBatch::single(check_err!(CheckCategory::Prose, message: uri))
            }
        }
    }
    async fn download(self, config: Option<ValeConfig>, skip_verify_checksum: bool) -> ApiResult<Vale> {
        let platform = vale::release_filename();
        let release = match self.version {
            | Some(value) => value,
            | None => SemanticVersion::from(VALE_VERSION),
        };
        let url = format!("{VALE_RELEASES_URL}/download/v{release}/{}_{release}_{platform}", self.command());
        info!(url, "=> {} Vale release v{release}", Label::using());
        match config {
            | Some(config) if config.custom_directory() => Err(eyre!("ACORN does not download into a custom Vale directory")),
            | Some(config) => match config.path.parent().map(Path::to_path_buf) {
                | Some(destination) => match download_binary(url.as_str(), destination.clone()).await {
                    | Ok(archive) => {
                        let verification = if skip_verify_checksum {
                            skip!("Checksum verification");
                            Ok(())
                        } else {
                            match self.clone().download_checksums().await {
                                | Ok(checksums) => match checksums.get(&platform) {
                                    | Some(expected) => match file_checksum(&archive, None) {
                                        | Ok(calculated) if expected == &calculated.checksum_value => {
                                            info!(checksum = expected, "=> {} Checksum verification", Label::pass());
                                            Ok(())
                                        }
                                        | Ok(calculated) => Err(eyre!(
                                            "Vale checksum mismatch: expected {expected}, calculated {}",
                                            calculated.checksum_value
                                        )),
                                        | Err(why) => Err(eyre!("Failed to calculate the Vale archive checksum — {why}")),
                                    },
                                    | None => Err(eyre!("Vale checksums do not include {platform}")),
                                },
                                | Err(why) => Err(why),
                            }
                        };
                        let result = match verification {
                            | Ok(()) => match self.clone().extract(archive.clone(), Some(destination)) {
                                | Ok(binary) => make_executable(&binary)
                                    .map(|()| Vale::init().version(release).binary(binary).config(config).build())
                                    .map_err(|why| eyre!("Downloaded Vale binary is not executable — {why}")),
                                | Err(why) => Err(why),
                            },
                            | Err(why) => Err(why),
                        };
                        let _cleanup = remove_file(archive);
                        result
                    }
                    | Err(why) => Err(eyre!("Failed to download Vale from {url} — {why}")),
                },
                | None => Err(eyre!("Vale configuration has no parent directory")),
            },
            | None => Err(eyre!("Vale configuration is required before downloading")),
        }
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
    fn extract(self, path: PathBuf, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
        let command = self.command();
        match destination.map(|value| value.to_absolute_path()) {
            | Some(parent) => {
                let extension = path.extension().unwrap_or_default().to_str().unwrap_or_default().to_string();
                match extension.as_str() {
                    | "7z" | "zip" | "gz" => match extract(path, Some(parent.into()), None) {
                        | Ok(value) => {
                            let path = value.join(command);
                            if cfg!(windows) {
                                Ok(path.with_extension("exe"))
                            } else {
                                Ok(path)
                            }
                        }
                        | Err(why) => Err(eyre!("Failed to extract {command} — {why}")),
                    },
                    | "tgz" => match File::open(path) {
                        | Ok(tar_gz) => {
                            let tar = GzDecoder::new(tar_gz);
                            let mut archive = Archive::new(tar);
                            match archive.unpack(parent.clone()) {
                                | Ok(_) => {
                                    debug!(parent, "=> {} Extracted {command} binary", Label::using());
                                    Ok(PathBuf::from(format!("{parent}/{command}")))
                                }
                                | Err(why) => Err(eyre!("Failed to extract {command} — {why}")),
                            }
                        }
                        | Err(why) => Err(eyre!("Failed to open the {command} archive — {why}")),
                    },
                    | _ => Err(eyre!("Unsupported {command} archive format: {extension}")),
                }
            }
            | None => Err(eyre!("No extraction directory was provided for {command}")),
        }
    }
    async fn sync(self, is_offline: bool, quiet: bool) -> ApiResult<()> {
        match (self.binary, self.config) {
            | (None, _) => Err(eyre!("Vale executable was not resolved")),
            | (Some(_), None) => Err(eyre!("Vale configuration was not resolved")),
            | (Some(binary_path), Some(_)) if !binary_path.is_executable() => {
                Err(eyre!("Vale executable is missing or is not executable: {}", binary_path.display()))
            }
            | (Some(_), Some(config)) if !config.path.is_file() => Err(eyre!("Vale configuration does not exist: {}", config.path.display())),
            | (Some(binary_path), Some(config)) => {
                let custom_directory = config.custom_directory();
                let config_path = config.path;
                let result: ApiResult<()> = if is_offline || custom_directory {
                    skip!("Vale sync");
                    Ok(())
                } else {
                    let arguments = args!["--config", config_path.clone(), "sync"];
                    let binary = binary_path.to_absolute_path();
                    let status = match quiet {
                        | true => cmd!(&binary, arguments).map(|output| output.status),
                        | false => cmd!(status & binary, arguments),
                    };
                    match status {
                        | Ok(value) if value.success() => Ok(()),
                        | Ok(value) => Err(eyre!("Vale sync failed ({value})")),
                        | Err(why) => Err(eyre!("Vale sync failed — {why}")),
                    }
                };
                match result {
                    | Ok(_) if custom_directory => Ok(()),
                    | Ok(_) => {
                        let parent_dir = config_path.parent().map(|p| p.display().to_string()).unwrap_or_default();
                        let parent = format!("{parent_dir}/styles/config/vocabularies/{APPLICATION}");
                        debug!(parent, "=> {} Vocabularies", Label::using());
                        match create_dir_all(parent.clone()) {
                            | Ok(_) => {}
                            | Err(why) => error!(directory = parent, "=> {} Create - {why}", Label::fail()),
                        }
                        let acronyms = Constant::last_values("acronyms");
                        let partners = SchemaConstant::nth_values("partners", 1);
                        let sponsors = SchemaConstant::nth_values("sponsors", 1);
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
        }
    }
    #[cfg(any(feature = "analysis", feature = "std"))]
    async fn retry_sync(self, options: &ExecutionOptions) -> ApiResult<()> {
        let command = self.command();
        let mut attempt = 1;
        loop {
            match self.clone().sync(options.common.offline, options.common.quiet).await {
                | Ok(()) => break Ok(()),
                | Err(why) if attempt < ONLINE_SYNC_ATTEMPTS => {
                    warn!(
                        attempt,
                        attempts = ONLINE_SYNC_ATTEMPTS,
                        interval = %options.extension.sync_retry_interval,
                        "{command} sync failed — {why}; retrying..."
                    );
                    sleep(options.extension.sync_retry_interval.unsigned_abs()).await;
                    attempt = attempt.saturating_add(1);
                }
                | Err(why) if options.extension.ignore_sync_failure => {
                    warn!(
                        attempts = ONLINE_SYNC_ATTEMPTS,
                        "{command} sync failed — {why}; continuing because --ignore-sync-failure is set"
                    );
                    break Ok(());
                }
                | Err(why) => break Err(eyre!("{command} sync failed after {ONLINE_SYNC_ATTEMPTS} attempts — {why}")),
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
        let config = self.normalize();
        let package_url = vale_package_url(false).await;
        config.materialize(&package_url)
    }
    fn resolve_package(value: impl AsRef<str>) -> String {
        ValeConfig::resolve_package(value)
    }
    async fn save(self, is_offline: bool) -> ApiResult<ValeConfig> {
        match self.custom_directory() {
            | true => Err(eyre!("ACORN does not write configuration into a custom Vale directory")),
            | false => {
                let config = self.normalize();
                let path = config.path.clone();
                match path.parent().map(Path::to_path_buf) {
                    | Some(parent) => match create_dir_all(&parent) {
                        | Ok(()) => {
                            let package_url = vale_package_url(is_offline).await;
                            match config.clone().materialize(&package_url).write_to_file(path.clone()) {
                                | Ok(()) => {
                                    debug!(path = path.to_absolute_path(), "=> {} Saved configuration", Label::using());
                                    Ok(config)
                                }
                                | Err(why) => Err(eyre!("Failed to save Vale configuration at {} — {why}", path.display())),
                            }
                        }
                        | Err(why) => Err(eyre!("Failed to create Vale directory at {} — {why}", parent.display())),
                    },
                    | None => Err(eyre!("Vale configuration has no parent directory")),
                }
            }
        }
    }
}
impl ValeConfig {
    pub(crate) fn materialize(self, package_url: &str) -> Ini {
        let ValeConfig {
            packages,
            vocabularies,
            disabled,
            ..
        } = self.clone();
        let package_sources = self
            .resolve_package_sources()
            .packages
            .into_iter()
            .chain(once(package_url.to_string()))
            .collect::<Vec<String>>();
        let mut conf = Ini::new();
        // CAUTION: Order of attributes in INI file matter. "StylesPath" must come before "Vocab"
        conf.with_section::<String>(None)
            .set("StylesPath", "styles")
            .set("Vocab", vocabularies.join(", "))
            .set("Packages", package_sources.join(", "));
        conf.with_section(Some("*"))
            .set("BasedOnStyles", format!("Vale, {}, {}", CUSTOM_VALE_PACKAGE_NAME, packages.join(", ")));
        disabled.iter().for_each(|rule| {
            conf.with_section(Some("*")).set(rule, "NO");
        });
        conf
    }
}
async fn check_schema_path<T, E>(path: PathBuf, document: Result<T, E>) -> CheckBatch
where
    T: Validate + Send,
    E: ToString + Send,
{
    let uri = Some(path.file_name_with_parent());
    match document {
        | Ok(data) => collect_validation_checks(&data)
            .await
            .into_iter()
            .map(|issue| issue.with_uri(uri.clone()))
            .collect(),
        | Err(why) => CheckBatch::schema_parse_failure_checks(why, uri),
    }
}
pub(crate) async fn collect_validation_checks<T>(value: &T) -> CheckBatch
where
    T: Validate,
{
    match value.validate_async().await {
        | Ok(()) => CheckBatch::empty(),
        | Err(report) => report.checks(None),
    }
}
/// Convert a Lychee response to a normalized link outcome.
pub(crate) fn convert_lychee_response(value: Response) -> LinkOutcome {
    let body = value.body().to_string();
    let url = value.source().to_string();
    debug!(url, "=> {} Response: {:#?}", Label::using(), body.dimmed().cyan());
    let status_code = value.status().code().map(|c| c.to_string()).unwrap_or_default();
    match value.status() {
        | Status::Ok(_) | Status::Redirected(_, _) => {
            LinkOutcome::standard(true, CheckSeverity::Info, "Has no HTTP errors", Some(status_code.clone()))
        }
        | Status::Cached(status) => match status {
            | CacheStatus::Ok(_) => LinkOutcome::standard(true, CheckSeverity::Info, "Has no HTTP errors", Some(status_code.clone())),
            | CacheStatus::Error(Some(_)) => LinkOutcome::standard(false, CheckSeverity::Error, "Has cached HTTP errors", Some(status_code.clone())),
            | CacheStatus::Unsupported => {
                LinkOutcome::standard(false, CheckSeverity::Warning, "Unsupported cached response", Some(status_code.clone()))
            }
            | _ => LinkOutcome::standard(
                true,
                CheckSeverity::Suggestion,
                "Ignored or otherwise successful (cached response)",
                Some(status_code.clone()),
            ),
        },
        | Status::Error(_) => LinkOutcome::standard(false, CheckSeverity::Error, "Has HTTP errors", Some(status_code.clone())),
        | Status::Unsupported(why) => LinkOutcome::standard(
            false,
            CheckSeverity::Warning,
            format!("Unsupported HTTP response — {why}"),
            Some(status_code.clone()),
        ),
        | Status::UnknownStatusCode(_) => LinkOutcome::standard(false, CheckSeverity::Warning, "Unknown HTTP response", Some(status_code.clone())),
        | Status::Timeout(_) => LinkOutcome::standard(false, CheckSeverity::Warning, "HTTP timeout", Some(status_code.clone())),
        | _ => LinkOutcome::standard(
            true,
            CheckSeverity::Suggestion,
            "Ignored or otherwise successful",
            Some(status_code.clone()),
        ),
    }
}
fn inspect_mcp_archive_entries(archive: &mut ZipArchive<File>) -> Result<BTreeSet<PathBuf>, String> {
    (0..archive.len())
        .try_fold(
            (BTreeSet::<PathBuf>::new(), BTreeSet::<PathBuf>::new()),
            |(mut entries, mut files), index| {
                archive
                    .by_index(index)
                    .map_err(|why| format!("Cannot inspect MCP bundle entry {index}: {why}"))
                    .and_then(|entry| {
                        let name = entry.name().to_string();
                        SafePath::new(&name)
                            .map_err(|why| format!("MCP bundle contains an unsafe entry {name}: {why}"))
                            .and_then(|safe| {
                                let normalized = safe.into_path_buf();
                                match (entries.insert(normalized.clone()), entry.is_dir()) {
                                    | (false, _) => Err(format!("MCP bundle contains a duplicate entry: {}", normalized.display())),
                                    | (true, true) => Ok((entries, files)),
                                    | (true, false) => {
                                        files.insert(normalized);
                                        Ok((entries, files))
                                    }
                                }
                            })
                    })
            },
        )
        .and_then(
            |(entries, files)| match (entries.is_empty(), files.contains(Path::new("manifest.json"))) {
                | (true, _) => Err("MCP bundle is empty".to_string()),
                | (false, false) => Err("MCP bundle requires manifest.json at the archive root".to_string()),
                | (false, true) => Ok(files),
            },
        )
}
fn inspect_mcp_bundle(path: &Path) -> Result<(), String> {
    File::open(path)
        .map_err(|why| format!("Cannot open MCP bundle: {why}"))
        .and_then(|file| ZipArchive::new(file).map_err(|why| format!("MCP bundle is not a valid ZIP archive: {why}")))
        .and_then(|mut archive| {
            inspect_mcp_archive_entries(&mut archive)
                .and_then(|files| read_mcp_manifest(&mut archive).map(|manifest| (files, manifest)))
                .and_then(|(files, manifest)| validate_mcp_manifest(&manifest, &files))
        })
}
fn kitfile_packaging_checks(path: &Path, value: &Kitfile) -> Vec<Check> {
    let uri = path.to_path_buf().file_name_with_parent();
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    root.canonicalize().map_or_else(
        |why| {
            vec![check!(
                CheckCategory::Quality,
                false,
                severity: CheckSeverity::Warning,
                message: format!("Cannot inspect Kitfile content directory: {why}"),
                uri: uri.clone()
            )]
        },
        |canonical_root| {
            ContentPaths::new(value)
                .iter()
                .filter_map(|(locator, content_path, mcp_bundle)| {
                    SafePath::new(content_path)
                        .map_err(|why| format!("Content path is not safe for packaging ({content_path}): {why}"))
                        .and_then(|safe| {
                            let target = canonical_root.join(safe.as_path());
                            target.canonicalize().map_err(|why| match why.kind() {
                                | IoErrorKind::NotFound => format!("Referenced content path does not exist: {content_path}"),
                                | _ => format!("Cannot inspect referenced content path {content_path}: {why}"),
                            })
                        })
                        .and_then(|target| match target.starts_with(&canonical_root) {
                            | false => Err(format!("Content path resolves outside the Kitfile directory: {content_path}")),
                            | true if mcp_bundle => inspect_mcp_bundle(&target).map(|()| target),
                            | true => Ok(target),
                        })
                        .err()
                        .map(|message| {
                            check!(
                                CheckCategory::Quality,
                                false,
                                severity: CheckSeverity::Warning,
                                locator: locator,
                                message: message,
                                uri: uri.clone()
                            )
                        })
                })
                .collect()
        },
    )
}
/// Perform link check on given URL using Lychee
pub fn link_check<'a, T>(uri: Option<T>, locator: Option<String>) -> BoxFuture<'a, Check>
where
    T: Into<String> + Send + 'a,
{
    let target = uri.map(Into::into);
    LinkEvidence::pending(target, locator, None).evaluate().boxed()
}
fn mcp_manifest_string<'a>(object: &'a serde_json::Map<String, serde_json::Value>, key: &str, path: &str) -> Result<&'a str, String> {
    object
        .get(key)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("MCP bundle manifest requires a non-empty {path}"))
}
fn modelkit_quality_checks(path: &Path, value: &Kitfile, inspect_packaging: bool) -> Vec<Check> {
    let uri = path.to_path_buf().file_name_with_parent();
    let checks = value
        .warnings()
        .into_iter()
        .map(|message| {
            check!(
                CheckCategory::Quality,
                false,
                severity: CheckSeverity::Warning,
                message: message,
                uri: uri.clone()
            )
        })
        .chain(inspect_packaging.then(|| kitfile_packaging_checks(path, value)).into_iter().flatten())
        .collect::<Vec<_>>();
    match checks.is_empty() {
        | true => vec![check_ok!(CheckCategory::Quality, uri: uri)],
        | false => checks,
    }
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
fn read_mcp_manifest(archive: &mut ZipArchive<File>) -> Result<serde_json::Value, String> {
    const MAX_MCP_MANIFEST_BYTES: u64 = 1_048_576;
    archive
        .by_name("manifest.json")
        .map_err(|why| format!("Cannot read MCP bundle manifest.json: {why}"))
        .and_then(|manifest_file| {
            let mut manifest = String::new();
            manifest_file
                .take(MAX_MCP_MANIFEST_BYTES + 1)
                .read_to_string(&mut manifest)
                .map_err(|why| format!("MCP bundle manifest.json is not valid UTF-8: {why}"))
                .and_then(|_| match manifest.len() as u64 > MAX_MCP_MANIFEST_BYTES {
                    | true => Err("MCP bundle manifest.json exceeds 1 MiB".to_string()),
                    | false => serde_json::from_str(&manifest).map_err(|why| format!("MCP bundle manifest.json is invalid JSON: {why}")),
                })
        })
}
async fn vale_package_url(is_offline: bool) -> String {
    match is_offline {
        | true => DEFAULT_VALE_PACKAGE_URL.to_string(),
        | false => {
            let package_repository = Repository::gitlab()
                .location(Location::Simple(VALE_PACKAGE_REPOSITORY_URL.to_string()))
                .call();
            match package_repository.latest_release().await {
                | Some(release) => {
                    let tag = release.tag_name;
                    format!("{VALE_PACKAGE_REPOSITORY_URL}/-/archive/{tag}/vale-package-{tag}.zip")
                }
                | None => DEFAULT_VALE_PACKAGE_URL.to_string(),
            }
        }
    }
}
fn validate_mcp_manifest(manifest: &serde_json::Value, files: &BTreeSet<PathBuf>) -> Result<(), String> {
    manifest
        .as_object()
        .ok_or_else(|| "MCP bundle manifest.json must contain a JSON object".to_string())
        .and_then(|object| {
            mcp_manifest_string(object, "manifest_version", "manifest_version")
                .and_then(|_| mcp_manifest_string(object, "name", "name"))
                .and_then(|_| mcp_manifest_string(object, "version", "version"))
                .and_then(|_| {
                    object
                        .get("server")
                        .and_then(serde_json::Value::as_object)
                        .ok_or_else(|| "MCP bundle manifest requires a server object".to_string())
                })
                .and_then(|server| {
                    mcp_manifest_string(server, "type", "server.type").and_then(|_| mcp_manifest_string(server, "entry_point", "server.entry_point"))
                })
                .and_then(|entry_point| SafePath::new(entry_point).map_err(|why| format!("MCP bundle server.entry_point is unsafe: {why}")))
                .and_then(|entry_point| match files.contains(entry_point.as_path()) {
                    | true => Ok(()),
                    | false => Err(format!(
                        "MCP bundle server.entry_point is missing from the archive: {}",
                        entry_point.as_path().display()
                    )),
                })
        })
}
fn vocabulary_checks(path: &Path, data: &ResearchActivity, mode: VocabularyFuzzyMode) -> CheckBatch {
    let uri = path.file_name_with_parent();
    let fields = [
        (
            VocabularyKind::Keywords,
            data.meta
                .keywords
                .iter()
                .enumerate()
                .map(|(index, value)| (format!("meta.keywords[{index}]"), value.clone()))
                .collect(),
        ),
        (
            VocabularyKind::Organizations,
            once(("contact.organization".to_string(), data.contact.organization.clone()))
                .chain(
                    data.contact
                        .affiliation
                        .iter()
                        .map(|value| ("contact.affiliation".to_string(), value.clone())),
                )
                .collect(),
        ),
        (
            VocabularyKind::Partners,
            data.meta
                .partners
                .iter()
                .flat_map(|values| values.iter())
                .enumerate()
                .map(|(index, value)| (format!("meta.partners[{index}]"), value.clone()))
                .collect(),
        ),
        (
            VocabularyKind::Sponsors,
            data.meta
                .funders
                .iter()
                .flat_map(|values| values.iter())
                .enumerate()
                .map(|(index, value)| (format!("meta.funders[{index}].name"), value.name.clone()))
                .collect(),
        ),
        (
            VocabularyKind::Technology,
            data.meta
                .technology
                .iter()
                .enumerate()
                .map(|(index, value)| (format!("meta.technology[{index}]"), value.clone()))
                .collect(),
        ),
    ];
    fields
        .into_iter()
        .map(|(kind, values)| CheckBatch::resolve_vocabulary(kind, values, mode, &uri))
        .collect()
}
#[cfg(test)]
mod tests;
