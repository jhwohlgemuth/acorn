//! Application configuration functions and data structures
//!
//! ACORN is configured by a JSON or YAML file (typically named `.acorn.json`)
//!
//! The ACORN configuration file configures what buckets should be downloaded, readability analysis, <span title="Large Language Model">LLM</span> settings, and more.
//!
use crate::io::api::github;
use crate::io::api::gitlab;
use crate::io::api::{Configuration, Endpoint};
use crate::io::{files_all, read_file, with_progress, write_file, write_file_bytes, FromPath, InputOutput, ProgressType, Source};
use crate::io::{ApiResult, Executor};
use crate::prelude::{self, env, exit, Arc, PathBuf};
use crate::util::constants::app::DEFAULT_CONFIG_FILENAMES;
use crate::util::{detect_json, suffix, Label, MimeType, StringConversion};
use crate::{Location, Repository, Scheme};
use bon::Builder;
use color_eyre::eyre::eyre;
use core::fmt::{self, Debug};
use core::future::Future;
use derive_more::Display;
use fancy_regex::Regex;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::error;

const IGNORE: [&str; 5] = [".gitignore", ".gitlab-ci.yml", ".gitkeep", ".DS_Store", "README.md"];

struct FilterSet {
    ignore: Vec<Regex>,
    filter: Vec<Regex>,
}

/// Runner status for CI/CD pipelines
#[derive(Clone, Debug, Default, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnerStatus {
    /// Online and available to run jobs
    #[default]
    Online,
    /// Offline and/or unavailable to run jobs
    Offline,
    /// Runner has not contacted the server for a while
    Stale,
    /// Runner has never contacted the server
    NeverContacted,
    /// Deprecated
    Active,
    /// Deprecated
    Paused,
}
/// Runner types for CI/CD pipelines
///
/// Mostly for GitLab as GitHub only has two types: hosted (by GitHub) and self-hosted
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum RunnerType {
    /// Accessible to a specific group and its projects/subgroups
    #[default]
    #[serde(rename = "group_type", alias = "group")]
    Group,
    /// Available to all projects and groups within an instance
    /// > Also called "shared runners" in GitLab
    #[serde(rename = "instance_type", alias = "instance")]
    Instance,
    /// Available to a specific project
    #[serde(rename = "project_type", alias = "project")]
    Project,
}
/// Struct for application configuration
///
/// ### Example `buckets.json`
/// ```json
/// {
///     "buckets": [
///         {
///             "name": "example",
///             "repository": {
///                 "provider": "github",
///                 "uri": "https://github.com/username/example"
///             }
///         },
///         {
///             "name": "example",
///             "repository": {
///                 "provider": "gitlab",
///                 "id": 12345,
///                 "uri": "https://gitlab.com/username/example"
///             }
///         }
///     ]
/// }
/// ```
#[derive(Clone, Debug, Default, Serialize, eserde::Deserialize)]
pub struct ApplicationConfiguration {
    /// List of buckets
    #[eserde(compat)]
    pub buckets: Option<Vec<Bucket>>,
    /// List of endpoints
    #[eserde(compat)]
    pub endpoints: Option<Vec<Endpoint>>,
    /// List of runners
    #[eserde(compat)]
    pub runners: Option<Vec<RunnerDetails>>,
}
/// Struct for bucket data
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[builder(start_fn = init)]
pub struct Bucket {
    /// Bucket name
    ///
    /// See <https://schema.org/name>
    pub name: Option<String>,
    /// Bucket description
    ///
    /// See <https://schema.org/description>
    pub description: Option<String>,
    /// Code repository data of bucket
    ///
    /// See <https://schema.org/codeRepository>
    #[serde(alias = "repository")]
    pub code_repository: Repository,
}
/// Options for bucket file operations (copy and download)
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init)]
pub struct BucketOptions {
    /// Path to output directory
    pub output: Option<PathBuf>,
    /// Number of threads used for parallel processing
    #[builder(default = 10)]
    pub threads: usize,
    /// Suppress progress output
    #[builder(default)]
    pub quiet: bool,
    /// Regex pattern(s) of files to ignore
    #[builder(default)]
    pub ignore: Vec<String>,
    /// Regex pattern(s) of files to include
    #[builder(default)]
    pub filter: Vec<String>,
}
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[builder(start_fn = at, on(String, into))]
#[serde(rename_all = "camelCase")]
/// CI/CD runner configuration entry
pub struct RunnerDetails {
    /// Code repository project/group where runner will be used
    #[builder(start_fn)]
    #[serde(alias = "repository")]
    pub code_repository: Repository,
    /// Runner name
    /// ### Note
    /// Primarily for identification in the GitLab UI
    pub name: Option<String>,
    /// Runner type (e.g., group, instance, project)
    #[builder(default, with = |method: &str| RunnerType::from(method))]
    #[serde(rename = "type")]
    pub runner_type: RunnerType,
    /// Optional description of runner
    pub description: Option<String>,
    /// Runner executor type (e.g., Docker, Kubernetes, Shell)
    #[builder(default = Executor::Docker)]
    #[serde(default = "default_executor")]
    pub executor: Executor,
    /// Does the runner need GPU capabilities?
    #[builder(default)]
    #[serde(default, alias = "gpu")]
    pub gpu_enabled: bool,
    /// List of tags associated with the runner
    #[serde(default, alias = "tag_list")]
    pub tags: Option<Vec<String>>,
    /// Whether the runner runs untagged jobs (GitLab-specific)
    #[builder(default)]
    #[serde(default, alias = "run_untagged")]
    pub run_untagged: bool,
    /// Optional GitLab host/domain override (for self-managed instances)
    pub host: Option<String>,
    /// Default Docker image for the runner itself
    #[builder(default = String::from("gitlab/gitlab-runner:latest"))]
    #[serde(default = "default_docker_image")]
    pub docker_image: String,
    /// Runner identifier assigned by GitLab during creation
    #[serde(default)]
    pub identifier: Option<u64>,
    /// Runner authentication token returned from GitLab API
    #[serde(default)]
    pub token: Option<String>,
}
impl ApplicationConfiguration {
    /// Resolve application configuration path
    ///
    /// If `path` is an existing file, it is returned.
    /// Otherwise, the first existing default config file in the current working directory is returned.
    pub fn resolve(path: &Option<PathBuf>) -> Option<PathBuf> {
        path.as_ref().filter(|value| value.is_file()).cloned().or_else(|| {
            let directory = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            DEFAULT_CONFIG_FILENAMES
                .iter()
                .map(|name| directory.join(name))
                .find(|candidate| candidate.exists())
        })
    }
    /// Parse ACORN configuration in JSON or YAML format
    pub fn parse(content: impl AsRef<str>) -> ApiResult<Self> {
        let content = content.as_ref();
        if detect_json(content) {
            match Self::parse_json(content) {
                | Ok(value) => Ok(value),
                | Err(errors) => {
                    let details: Vec<String> = errors
                        .iter()
                        .map(|e| format!("{}: {}", e.path().map_or("root".into(), |p| p.to_string()), e.message()))
                        .collect();
                    Err(eyre!("{}", details.join("\n")))
                }
            }
        } else {
            match Self::parse_yaml(content) {
                | Ok(value) => Ok(value),
                | Err(why) => Err(eyre!("Failed to parse ACORN configuration YAML — {why}")),
            }
        }
    }
    fn parse_json(content: impl AsRef<str>) -> Result<Self, eserde::DeserializationErrors> {
        eserde::json::from_str(content.as_ref())
    }
    fn parse_yaml(content: impl AsRef<str>) -> serde_norway::Result<Self> {
        serde_norway::from_str(content.as_ref())
    }
}
impl InputOutput for ApplicationConfiguration {
    /// Read and parse application configuration file (JSON or YAML)
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into();
        match MimeType::from_path(&source) {
            | MimeType::Json => Self::read_json(source.clone()),
            | MimeType::Yaml => Self::read_yaml(source.clone()),
            | _ => Err(eyre!("Unsupported configuration file extension")),
        }
    }
    /// Read configuration (e.g., `.acorn.json`) using Serde and [`ApplicationConfiguration`] struct
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        let content = match read_file(path.clone()) {
            | Ok(value) if !value.is_empty() => value,
            | Ok(_) | Err(_) => {
                error!(
                    path = path.to_string_lossy().to_string(),
                    "=> {} ACORN configuration JSON content",
                    Label::fail()
                );
                "{}".to_owned()
            }
        };
        match Self::parse_json(content) {
            | Ok(config) => Ok(config),
            | Err(errors) => {
                let details: Vec<String> = errors
                    .iter()
                    .map(|e| format!("{}: {}", e.path().map_or("root".into(), |p| p.to_string()), e.message()))
                    .collect();
                Err(eyre!("{}", details.join("\n")))
            }
        }
    }
    /// Read configuration (e.g., `.acorn.yml`) using Serde and [`ApplicationConfiguration`] struct
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        let content = match read_file(path.clone()) {
            | Ok(value) => value,
            | Err(_) => {
                error!(
                    path = path.to_string_lossy().to_string(),
                    "=> {} ACORN configuration YAML content",
                    Label::fail()
                );
                "".to_owned()
            }
        };
        Self::parse_yaml(content).map_err(|why| eyre!("Failed to parse YAML config — {why}"))
    }
    /// Write configuration to specified path (detects JSON or YAML from extension)
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let target = path.into();
        match MimeType::from_path(&target) {
            | MimeType::Json => self.write_json(&target),
            | MimeType::Yaml => self.write_yaml(&target),
            | _ => Err(eyre!("Unsupported configuration file extension")),
        }
    }
    /// Write configuration as JSON to specified path
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let target = path.into();
        serde_json::to_string_pretty(&self)
            .map_err(|why| eyre!("Failed to serialize JSON config — {why}"))
            .and_then(|content| write_file(target.clone(), content))
    }
    /// Write configuration as YAML to specified path
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let target = path.into();
        serde_norway::to_string(&self)
            .map_err(|why| eyre!("Failed to serialize YAML config — {why}"))
            .and_then(|content| write_file(target.clone(), content))
    }
}
impl Bucket {
    /// Get hosting domain from bucket struct
    pub(crate) fn domain(&self) -> ApiResult<String> {
        let location = match &self.code_repository {
            | Repository::GitHub { location } | Repository::GitLab { location, .. } => location,
            | Repository::Git { .. } => return Err(eyre!("Domain is unsupported for generic Git repositories")),
        };
        match location.scheme() {
            | Scheme::HTTPS => location.host().ok_or_else(|| eyre!("Failed to parse repository host from URI")),
            | _ => Err(eyre!("Unsupported repository URI scheme")),
        }
    }
    /// Copy files from (local) bucket to local directory
    /// ### Notes
    /// - Ignores files listed in [`IGNORE`]
    /// - Only copies files from local repositories
    pub async fn copy_files(self: Bucket, options: &BucketOptions) -> ApiResult<usize> {
        let BucketOptions { output, ignore, filter, .. } = options;
        let output = Arc::new(output.clone().unwrap_or_default());
        match compile_filter_set(ignore, filter) {
            | Ok(filters) => {
                let Bucket { name, code_repository, .. } = self.clone();
                match code_repository.is_local() {
                    | true => {
                        let bucket_root = match code_repository.location().path() {
                            | Some(value) => PathBuf::from(value).to_absolute_string(),
                            | None => {
                                return Err(eyre!(
                                    "Bucket {} has no local path — cannot copy files",
                                    name.as_deref().unwrap_or("unknown")
                                ))
                            }
                        };
                        let items = filter_paths(
                            files_all(PathBuf::from(&bucket_root), None)
                                .into_iter()
                                .map(|x| x.display().to_string())
                                .collect::<Vec<String>>(),
                            &filters,
                        )
                        .into_iter()
                        .filter(|path| PathBuf::from(path).is_file())
                        .collect::<Vec<String>>();
                        let bucket_root = Arc::new(bucket_root);
                        let operation = {
                            let bucket_root = Arc::clone(&bucket_root);
                            let output = Arc::clone(&output);
                            move |path: String| {
                                let bucket_root = Arc::clone(&bucket_root);
                                let output = Arc::clone(&output);
                                async move {
                                    let path_buf = PathBuf::from(&path);
                                    match path_buf.strip_prefix(PathBuf::from(bucket_root.as_str())) {
                                        | Ok(relative) => write_file_bytes(output.join(relative), || async { prelude::read(path.clone()) }).await,
                                        | Err(_) => Err(eyre!("Failed to strip prefix {bucket_root} from {path}")),
                                    }
                                }
                            }
                        };
                        transfer_bucket_files(name, items, options, "Copying", operation).await
                    }
                    | false => Ok(0),
                }
            }
            | Err(why) => Err(why),
        }
    }
    /// Download files from bucket to local directory
    ///
    /// Ignores files listed in [`IGNORE`]
    ///
    /// Downloads files concurrently using buffered streams
    pub async fn download_files(self: Bucket, options: &BucketOptions) -> ApiResult<usize> {
        let BucketOptions { filter, ignore, .. } = options;
        match compile_filter_set(ignore, filter) {
            | Ok(filters) => {
                let name = self.name.clone();
                let code_repository = self.code_repository.clone();
                match self.file_paths("").await {
                    | Ok(paths) => {
                        let items = filter_paths(paths, &filters);
                        let operation = {
                            let code_repository = Arc::new(code_repository);
                            let output = Arc::new(options.output.clone().unwrap_or_default());
                            move |path: String| {
                                let output = Arc::clone(&output);
                                let repository = Arc::clone(&code_repository);
                                async move {
                                    match repository.raw_url(path.clone()) {
                                        | Some(url) => {
                                            let output_filepath = output.join(path);
                                            write_file_bytes(output_filepath, || async move { Source::read_bytes(&url, false).await }).await
                                        }
                                        | None => Err(eyre!("Failed to build raw URL for repository path")),
                                    }
                                }
                            }
                        };
                        transfer_bucket_files(name, items, options, "Downloading", operation).await
                    }
                    | Err(why) => {
                        error!("=> {} Get file paths for download — {why}", Label::fail());
                        Err(why)
                    }
                }
            }
            | Err(why) => Err(why),
        }
    }
    async fn file_paths(&self, directory: &str) -> ApiResult<Vec<String>> {
        let code_repository = self.code_repository.clone();
        let bucket_name = self.name.clone().unwrap_or_else(|| "Bucket".to_string()).to_uppercase();
        match &code_repository {
            | Repository::Git { .. } => {
                let path = match code_repository.location().path() {
                    | Some(value) => PathBuf::from(value),
                    | None => return Err(eyre!("Git repository has no local path — cannot list files")),
                };
                Ok(files_all(path, None).into_iter().map(|x| x.display().to_string()).collect())
            }
            | Repository::GitHub { location } => match location.path() {
                | Some(path) => {
                    let path = path.trim_start_matches('/').to_string();
                    match self.domain() {
                        | Ok(host) => github::tree_paths(format!("api.{}", host), path, "main")
                            .await
                            .map_err(|why| eyre!("Failed to get file paths for {bucket_name} bucket - {why}")),
                        | Err(why) => Err(why),
                    }
                }
                | None => Err(eyre!("Failed to parse GitHub URI for {bucket_name} bucket")),
            },
            | Repository::GitLab { .. } => match code_repository.id() {
                | Some(id) => match self.domain() {
                    | Ok(host) => {
                        let options = gitlab::Options::from_env().with_domain(host).with_identifier(id).with_path(directory);
                        let mut page = 1_u32;
                        let mut all_paths: Vec<String> = vec![];
                        loop {
                            let page_options = options.clone().with_page(page);
                            match gitlab::tree_paths(&page_options).await {
                                | Ok(response) if response.paths.is_empty() => {
                                    break Ok(all_paths.clone());
                                }
                                | Ok(response) => {
                                    all_paths.extend(response.paths);
                                    page = page.saturating_add(1);
                                }
                                | Err(why) => {
                                    break Err(eyre!("Failed to get file paths for {bucket_name} bucket — {why}"));
                                }
                            }
                        }
                    }
                    | Err(why) => Err(why),
                },
                | None => Err(eyre!("Missing GitLab project id for {bucket_name} bucket")),
            },
        }
    }
}
impl From<&str> for Bucket {
    fn from(value: &str) -> Self {
        let location = Location::Simple(value.to_string());
        if location.uri().is_none() {
            exit(exitcode::DATAERR);
        }
        let repository = match location.scheme() {
            | Scheme::File => Repository::Git { location },
            | _ => {
                let host = match location.host() {
                    | Some(value) => value.to_lowercase(),
                    | None => {
                        error!(value, "=> {} Parse URI - No host", Label::fail());
                        exit(exitcode::DATAERR);
                    }
                };
                if host.contains("github.com") {
                    Repository::GitHub { location }
                } else {
                    let id = None;
                    Repository::GitLab { id, location }
                }
            }
        };
        Bucket::init().code_repository(repository).build()
    }
}
impl Default for BucketOptions {
    fn default() -> Self {
        Self {
            output: None,
            threads: 10,
            quiet: false,
            ignore: Vec::new(),
            filter: Vec::new(),
        }
    }
}
impl BucketOptions {
    /// Set the output path
    pub fn with_output(self, output: impl Into<PathBuf>) -> Self {
        Self {
            output: Some(output.into()),
            ..self
        }
    }
}
impl fmt::Display for RunnerType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            | RunnerType::Group => "group",
            | RunnerType::Instance => "instance",
            | RunnerType::Project => "project",
        };
        formatter.write_str(value)
    }
}
impl RunnerDetails {
    /// Set the runner identifier (assigned by GitLab after creation)
    pub fn with_id(self, value: u64) -> Self {
        Self {
            identifier: Some(value),
            ..self
        }
    }
    /// Set the runner name (used as Docker container name)
    pub fn with_name(self, value: String) -> Self {
        Self { name: Some(value), ..self }
    }
    /// Set the runner authentication token
    pub fn with_token(self, value: Option<String>) -> Self {
        Self { token: value, ..self }
    }
}
impl From<&str> for RunnerType {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            | "GROUP" => RunnerType::Group,
            | "INSTANCE" => RunnerType::Instance,
            | "PROJECT" => RunnerType::Project,
            | _ => RunnerType::Group,
        }
    }
}
impl From<String> for RunnerType {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}
fn compile_filter_set(ignore: &[String], filter: &[String]) -> ApiResult<FilterSet> {
    compile_regex_patterns(ignore).and_then(|ignore| compile_regex_patterns(filter).map(|filter| FilterSet { ignore, filter }))
}
fn compile_regex_patterns(patterns: &[String]) -> ApiResult<Vec<Regex>> {
    patterns
        .iter()
        .map(|pattern| Regex::new(pattern).map_err(|why| eyre!("Invalid regex/filter pattern '{pattern}': {why}")))
        .collect()
}
fn count_json_files(paths: &[String]) -> usize {
    paths.iter().filter(|&path| path.to_lowercase().ends_with(".json")).count()
}
fn count_image_files(paths: &[String]) -> usize {
    paths.iter().filter(|&x| has_image_extension(x)).count()
}
fn default_docker_image() -> String {
    "gitlab/gitlab-runner:latest".to_string()
}
fn default_executor() -> Executor {
    Executor::Docker
}
fn filter_paths(paths: Vec<String>, filters: &FilterSet) -> Vec<String> {
    paths
        .into_iter()
        .filter(|path| !is_ignored_path(path, &filters.ignore) && is_filtered_path(path, &filters.filter))
        .collect()
}
#[allow(clippy::ptr_arg)]
fn has_image_extension(path: &String) -> bool {
    path.to_lowercase().ends_with(".png") || path.to_lowercase().ends_with(".jpg")
}
fn is_ignored_path(path: &str, ignore: &[Regex]) -> bool {
    let is_builtin_ignored = IGNORE.iter().any(|value| path.ends_with(value));
    let is_regex_ignored = ignore.iter().any(|pattern| pattern.is_match(path).unwrap_or(false));
    is_builtin_ignored || is_regex_ignored
}
fn is_filtered_path(path: &str, filter: &[Regex]) -> bool {
    filter.is_empty() || filter.iter().any(|pattern| pattern.is_match(path).unwrap_or(false))
}
fn operations_complete_message(name: Option<String>, json_count: usize, image_count: usize) -> String {
    let total = json_count.saturating_add(image_count);
    let message = if json_count != image_count {
        let recommendation = if json_count > image_count {
            "Do you need to add some images?"
        } else {
            "Do you need to add some JSON files?"
        };
        format!(
            " ({} data file{}, {} image{} - {})",
            json_count.yellow(),
            suffix(json_count),
            image_count.yellow(),
            suffix(image_count),
            recommendation.italic(),
        )
    } else {
        "".to_string()
    };
    let bucket_description = match name {
        | Some(value) => format!("{} bucket", value.to_uppercase().cyan()),
        | None => "<URL>".cyan().to_string(),
    };
    format!(
        "{}Obtained {} file{} from {bucket_description}{}",
        if total > 0 { Label::CHECKMARK } else { Label::CAUTION },
        if total > 0 {
            total.green().to_string()
        } else {
            total.yellow().to_string()
        },
        suffix(total),
        message,
    )
}
async fn transfer_bucket_files<F, Fut>(
    name: Option<String>,
    items: Vec<String>,
    options: &BucketOptions,
    verb: &'static str,
    operation: F,
) -> ApiResult<usize>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = ApiResult<()>>,
{
    let BucketOptions { threads, quiet, .. } = options;
    let total_data = count_json_files(&items);
    let total_images = count_image_files(&items);
    let total = total_data.saturating_add(total_images);
    let message = move |path: &String| format!("{verb} {path}");
    let finish_message = |_| operations_complete_message(name, total_data, total_images);
    let progress_type = match quiet {
        | true => ProgressType::Silent,
        | false => ProgressType::Bar,
    };
    with_progress(items, message, operation, finish_message, Some(*threads), progress_type)
        .await
        .map(|_| total)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )]
    use super::*;
    use crate::prelude::{create_dir_all, remove_dir_all, write};

    fn temp_resolve_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(core::time::Duration::from_nanos(0))
            .as_nanos();
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("target")
            .join("test_artifacts")
            .join(format!("{name}-{nanos}"))
    }

    #[test]
    fn test_resolve_returns_explicit_existing_path() {
        let directory = temp_resolve_dir("resolve-explicit");
        create_dir_all(&directory).unwrap();
        let directory = directory.canonicalize().unwrap();
        let provided = directory.join("config.yaml");
        let default = directory.join(".acorn.json");
        write(&provided, "{}\n").unwrap();
        write(&default, "{}\n").unwrap();
        let resolved = ApplicationConfiguration::resolve(&Some(provided.clone()));
        assert_eq!(resolved, Some(provided));
        let _ = remove_dir_all(directory);
    }
    #[test]
    fn test_resolve_falls_back_to_default_when_provided_path_missing() {
        let directory = temp_resolve_dir("resolve-fallback");
        create_dir_all(&directory).unwrap();
        let directory = directory.canonicalize().unwrap();
        let default = directory.join(".acorn.yml");
        write(&default, "{}\n").unwrap();
        let previous = env::current_dir().unwrap();
        env::set_current_dir(&directory).unwrap();
        let resolved = ApplicationConfiguration::resolve(&Some(directory.join("missing.json")));
        env::set_current_dir(previous).unwrap();
        assert_eq!(resolved, Some(default));
        let _ = remove_dir_all(directory);
    }
    #[test]
    fn test_count_image_files_counts_supported_extensions() {
        let paths = vec![
            "content/plot.png".to_string(),
            "content/photo.jpg".to_string(),
            "content/photo.jpeg".to_string(),
            "content/index.json".to_string(),
        ];
        assert_eq!(count_image_files(&paths), 2);
    }
    #[test]
    fn test_count_json_files_counts_case_insensitive_json_paths() {
        let paths = vec![
            "content/index.json".to_string(),
            "content/README.md".to_string(),
            "content/data.JSON".to_string(),
        ];
        assert_eq!(count_json_files(&paths), 2);
    }
    #[test]
    fn test_has_image_extension_matches_png_and_jpg() {
        assert!(has_image_extension(&"image.png".to_string()));
        assert!(has_image_extension(&"photo.JPG".to_string()));
        assert!(!has_image_extension(&"graphic.jpeg".to_string()));
    }
    #[test]
    fn test_is_ignored_path() {
        let ignore = compile_regex_patterns(&[r"\.jpeg$".to_string(), r"notes\.txt$".to_string()]).unwrap();
        assert!(is_ignored_path("/tmp/photo.jpeg", &ignore));
        assert!(is_ignored_path("/tmp/notes.txt", &ignore));
        assert!(!is_ignored_path("/tmp/index.json", &ignore));
        let invalid = compile_regex_patterns(&["[".to_string()]);
        assert!(invalid.is_err());
        let ignore: Vec<Regex> = vec![];
        assert!(is_ignored_path("/tmp/README.md", &ignore));
    }
    #[test]
    fn test_is_filtered_path() {
        let filter = compile_regex_patterns(&[r"\.json$".to_string(), r"img/".to_string()]).unwrap();
        assert!(is_filtered_path("/tmp/data.json", &filter));
        assert!(is_filtered_path("/tmp/img/photo.jpg", &filter));
        assert!(!is_filtered_path("/tmp/README.md", &filter));
        let invalid = compile_regex_patterns(&["[".to_string()]);
        assert!(invalid.is_err());
        let filter: Vec<Regex> = vec![];
        assert!(is_filtered_path("/tmp/README.md", &filter));
    }
    #[test]
    fn test_operations_complete_message_includes_bucket_name_and_guidance() {
        let message = operations_complete_message(Some("acorn".to_string()), 2, 1);
        assert!(message.contains("Obtained"));
        assert!(message.contains("ACORN"));
        assert!(message.contains(" bucket"));
        assert!(message.contains("data file"));
        assert!(message.contains("image"));
        assert!(message.contains("Do you need to add some images?"));
    }
    #[test]
    fn test_operations_complete_message_uses_url_placeholder_without_name() {
        let message = operations_complete_message(None, 0, 0);
        assert!(message.contains("Obtained"));
        assert!(message.contains("<URL>"));
    }
}
