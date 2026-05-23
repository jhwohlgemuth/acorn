//! Application configuration functions and data structures
//!
//! ACORN is configured by a JSON or YAML file (typically named `.acorn.json`)
//!
//! The ACORN configuration file configures what buckets should be downloaded, readability analysis, <span title="Large Language Model">LLM</span> settings, and more.
//!
use crate::io::api::github;
use crate::io::api::gitlab;
use crate::io::api::Endpoint;
use crate::io::http::get;
use crate::io::ApiResult;
use crate::io::{files_all, read_file, with_progress, write_file, write_file_bytes, FromPath, InputOutput, ProgressType};
use crate::prelude::{self, exit, Arc, PathBuf};
use crate::util::{detect_json, suffix, Label, MimeType, StringConversion};
use crate::{Location, Repository, Scheme};
use bon::Builder;
use color_eyre::eyre::eyre;
use core::fmt::Debug;
use derive_more::Display;
use fancy_regex::Regex;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::error;

const IGNORE: [&str; 5] = [".gitignore", ".gitlab-ci.yml", ".gitkeep", ".DS_Store", "README.md"];

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
#[derive(Clone, Debug, Default, Display, Serialize, Deserialize)]
pub enum RunnerType {
    /// Accessible to a specific group and its projects/subgroups
    #[default]
    #[display("group_type")]
    #[serde(rename = "group_type", alias = "group")]
    Group,
    /// Available to all projects and groups within an instance
    /// > Also called "shared runners" in GitLab
    #[display("instance_type")]
    #[serde(rename = "instance_type", alias = "instance")]
    Instance,
    /// Available to a specific project
    #[display("project_type")]
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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApplicationConfiguration {
    /// List of buckets
    pub buckets: Option<Vec<Bucket>>,
    /// List of endpoints
    pub endpoints: Option<Vec<Endpoint>>,
    /// List of runners
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
    /// Runner name
    pub name: String,
    /// Runner type (e.g., group, instance, project)
    #[builder(with = |method: &str| RunnerType::from(method))]
    #[builder(default)]
    #[serde(rename = "type")]
    pub runner_type: RunnerType,
    /// Optional description of runner
    pub description: Option<String>,
    /// Code repository project/group where runner will be used
    #[serde(alias = "repository")]
    pub code_repository: Repository,
    /// Does the runner need GPU capabilities?
    #[builder(default)]
    #[serde(default, alias = "gpu")]
    pub gpu_enabled: bool,
    /// List of tags associated with the runner
    #[serde(default, alias = "tag_list")]
    pub tags: Option<Vec<String>>,
    /// Whether the runner runs untagged jobs (GitLab-specific)
    #[builder(default)]
    #[serde(default)]
    pub run_untagged: bool,
}
impl ApplicationConfiguration {
    /// Parse ACORN configuration in JSON or YAML format
    pub fn parse(content: impl AsRef<str>) -> ApiResult<Self> {
        let content = content.as_ref();
        let result = if detect_json(content) {
            match Self::parse_json(content) {
                | Ok(value) => Ok(value),
                | Err(why) => Err(eyre!("Failed to parse ACORN configuration JSON — {why}")),
            }
        } else {
            match Self::parse_yaml(content) {
                | Ok(value) => Ok(value),
                | Err(why) => Err(eyre!("Failed to parse ACORN configuration YAML — {why}")),
            }
        };
        match result {
            | Ok(data) => Ok(data),
            | Err(why) => Err(why),
        }
    }
    fn parse_json(content: impl AsRef<str>) -> serde_json::Result<Self> {
        serde_json::from_str(content.as_ref())
    }
    fn parse_yaml(content: impl AsRef<str>) -> serde_yml::Result<Self> {
        serde_yml::from_str(content.as_ref())
    }
}
impl InputOutput for ApplicationConfiguration {
    /// Read and parse application configuration file (JSON or YAML)
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into();
        match MimeType::from_path(source.clone()) {
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
        Self::parse_json(content).map_err(|why| eyre!("Failed to parse JSON config — {why}"))
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
        serde_yml::to_string(&self)
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
        match location.uri() {
            | Some(uri) => match uri.scheme() {
                | uriparse::Scheme::HTTPS => uri
                    .host()
                    .map(|value| value.to_string())
                    .ok_or_else(|| eyre!("Failed to parse repository host from URI")),
                | _ => Err(eyre!("Unsupported repository URI scheme")),
            },
            | None => Err(eyre!("Failed to parse repository URI")),
        }
    }
    /// Copy files from (local) bucket to local directory
    /// ### Notes
    /// - Ignores files listed in [`IGNORE`]
    /// - Only copies files from local repositories
    pub async fn copy_files(self: Bucket, options: &BucketOptions) -> ApiResult<usize> {
        let BucketOptions {
            output,
            threads,
            quiet,
            ignore,
            filter,
        } = options;
        let output = Arc::new(output.clone().unwrap_or_default());
        let Bucket { name, code_repository, .. } = self.clone();
        if code_repository.is_local() {
            let uri = code_repository.location().uri();
            let bucket_root = match uri {
                | Some(value) => PathBuf::from(value.path().to_string()).to_absolute_string(),
                | None => {
                    unimplemented!()
                }
            };
            let files = files_all(PathBuf::from(&bucket_root), None)
                .into_iter()
                .map(|x| x.display().to_string())
                .collect::<Vec<String>>();
            let items = files
                .into_iter()
                .filter(|path| !is_ignored_path(path, ignore) && PathBuf::from(path).is_file() && is_filtered_path(path, filter))
                .collect::<Vec<String>>();
            let message: fn(&String) -> String = |path| format!("Copying {path}");
            let bucket_root = Arc::new(bucket_root);
            let operation = {
                let bucket_root = Arc::clone(&bucket_root);
                let output = Arc::clone(&output);
                move |path: String| {
                    let bucket_root = Arc::clone(&bucket_root);
                    let output = Arc::clone(&output);
                    async move {
                        let path_buf = PathBuf::from(&path);
                        let relative = match path_buf.strip_prefix(PathBuf::from(bucket_root.as_str())) {
                            | Ok(rel) => rel.to_path_buf(),
                            | Err(_) => return Err(eyre!("Failed to strip prefix {bucket_root} from {path}")),
                        };
                        let output_filepath = output.join(relative);
                        write_file_bytes(output_filepath, || async { prelude::read(path.clone()) }).await
                    }
                }
            };
            let total_data: usize = count_json_files(&items);
            let total_images: usize = count_image_files(&items);
            let finish_message = |_| operations_complete_message(name, total_data, total_images);
            let progress_type = match quiet {
                | true => ProgressType::Silent,
                | false => ProgressType::Bar,
            };
            match with_progress(items, message, operation, finish_message, Some(*threads), progress_type).await {
                | Ok(_) => Ok(total_data.saturating_add(total_images)),
                | Err(why) => {
                    error!("=> {} Copy files with progress bar — {why}", Label::fail());
                    Err(why)
                }
            }
        } else {
            Ok(0)
        }
    }
    /// Download files from bucket to local directory
    ///
    /// Ignores files listed in [`IGNORE`]
    ///
    /// Downloads files concurrently using buffered streams
    pub async fn download_files(self: Bucket, options: &BucketOptions) -> ApiResult<usize> {
        let BucketOptions {
            output,
            threads,
            quiet,
            filter,
            ignore,
        } = options;
        let output = Arc::new(output.clone().unwrap_or_default());
        let Bucket { name, code_repository, .. } = self.clone();
        let items = match self.clone().file_paths("").await {
            | Ok(paths) => paths
                .into_iter()
                .filter(|path| !is_ignored_path(path, ignore) && is_filtered_path(path, filter))
                .collect::<Vec<String>>(),
            | Err(why) => {
                error!("=> {} Get file paths for download — {why}", Label::fail());
                return Err(why);
            }
        };
        let message: fn(&String) -> String = |path| format!("Downloading {path}");
        let operation = {
            let output = Arc::clone(&output);
            let code_repository = Arc::new(code_repository);
            move |path: String| {
                let output = Arc::clone(&output);
                let repository = Arc::clone(&code_repository);
                async move {
                    let output_filepath = format!("{}/{}", output.display(), path);
                    let url = repository
                        .raw_url(path.clone())
                        .ok_or_else(|| eyre!("Failed to build raw URL for repository path"));
                    match url {
                        | Ok(url) => {
                            write_file_bytes(output_filepath, || async move {
                                match get(url).send().await {
                                    | Ok(response) => response
                                        .bytes()
                                        .await
                                        .map(|b| b.to_vec())
                                        .map_err(|why| eyre!("Failed to download file — {why}")),
                                    | Err(why) => Err(eyre!("Failed to download file — {why}")),
                                }
                            })
                            .await
                        }
                        | Err(why) => Err(why),
                    }
                }
            }
        };
        let total_data: usize = count_json_files(&items);
        let total_images: usize = count_image_files(&items);
        let finish_message = |_| operations_complete_message(name, total_data, total_images);
        let progress_type = match quiet {
            | true => ProgressType::Silent,
            | false => ProgressType::Bar,
        };
        match with_progress(items, message, operation, finish_message, Some(*threads), progress_type).await {
            | Ok(_) => Ok(total_data.saturating_add(total_images)),
            | Err(why) => Err(why),
        }
    }
    async fn file_paths(self: Bucket, directory: &str) -> ApiResult<Vec<String>> {
        let Bucket { code_repository, name, .. } = self.clone();
        let bucket_name = name.clone().unwrap_or_else(|| "Bucket".to_string()).to_uppercase();
        match &code_repository {
            | Repository::Git { .. } => {
                let path = match code_repository.location().uri() {
                    | Some(value) => PathBuf::from(value.path().to_string()),
                    | None => {
                        unimplemented!()
                    }
                };
                Ok(files_all(path, None).into_iter().map(|x| x.display().to_string()).collect())
            }
            | Repository::GitHub { location } => {
                let path = match location.uri() {
                    | Some(uri) => uri.path().to_string().trim_start_matches('/').to_string(),
                    | None => return Err(eyre!("Failed to parse GitHub URI for {bucket_name} bucket")),
                };
                let host = match self.domain() {
                    | Ok(value) => value,
                    | Err(why) => return Err(why),
                };
                github::tree_paths(format!("api.{}", host), path, "main")
                    .await
                    .map_err(|why| eyre!("Failed to get file paths for {bucket_name} bucket - {why}"))
            }
            | Repository::GitLab { .. } => match self.code_repository.id() {
                | Some(id) => {
                    let host = match self.domain() {
                        | Ok(value) => value,
                        | Err(why) => return Err(why),
                    };
                    let mut page = 1_u32;
                    let mut all_paths: Vec<String> = vec![];
                    loop {
                        match gitlab::tree_paths(host.clone(), id.to_string(), directory, page).await {
                            | Ok(paths) if paths.is_empty() => break,
                            | Ok(paths) => {
                                all_paths.extend(paths);
                                page = page.saturating_add(1);
                            }
                            | Err(why) => {
                                return Err::<Vec<String>, _>(why);
                            }
                        }
                    }
                    Ok(all_paths)
                }
                | None => Err(eyre!("Missing GitLab project id for {bucket_name} bucket")),
            },
        }
    }
}
impl From<&str> for Bucket {
    fn from(value: &str) -> Self {
        let location = Location::Simple(value.to_string());
        match location.uri() {
            | Some(uri) => {
                let repository = match location.scheme() {
                    | Scheme::File => Repository::Git { location },
                    | _ => {
                        let host = match uri.host() {
                            | Some(value) => value.to_string().to_lowercase(),
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
            | None => {
                exit(exitcode::DATAERR);
            }
        }
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
fn count_json_files(paths: &[String]) -> usize {
    paths.iter().filter(|&path| path.to_lowercase().ends_with(".json")).count()
}
fn count_image_files(paths: &[String]) -> usize {
    paths.iter().filter(|&x| has_image_extension(x)).count()
}
fn is_ignored_path(path: &str, ignore: &[String]) -> bool {
    let is_builtin_ignored = IGNORE.iter().any(|value| path.ends_with(value));
    let is_regex_ignored = ignore
        .iter()
        .filter_map(|pattern| match Regex::new(pattern) {
            | Ok(value) => Some(value),
            | Err(_) => None,
        })
        .any(|pattern| pattern.is_match(path).unwrap_or(false));
    is_builtin_ignored || is_regex_ignored
}
fn is_filtered_path(path: &str, filter: &[String]) -> bool {
    filter.is_empty()
        || filter
            .iter()
            .filter_map(|pattern| match Regex::new(pattern) {
                | Ok(value) => Some(value),
                | Err(_) => None,
            })
            .any(|pattern| pattern.is_match(path).unwrap_or(false))
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
#[allow(clippy::ptr_arg)]
fn has_image_extension(path: &String) -> bool {
    path.to_lowercase().ends_with(".png") || path.to_lowercase().ends_with(".jpg")
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
        let ignore = vec![r"\.jpeg$".to_string(), r"notes\.txt$".to_string()];
        assert!(is_ignored_path("/tmp/photo.jpeg", &ignore));
        assert!(is_ignored_path("/tmp/notes.txt", &ignore));
        assert!(!is_ignored_path("/tmp/index.json", &ignore));
        let ignore = vec!["[".to_string()];
        assert!(!is_ignored_path("/tmp/index.json", &ignore));
        let ignore: Vec<String> = vec![];
        assert!(is_ignored_path("/tmp/README.md", &ignore));
    }
    #[test]
    fn test_is_filtered_path() {
        let filter = vec![r"\.json$".to_string(), r"img/".to_string()];
        assert!(is_filtered_path("/tmp/data.json", &filter));
        assert!(is_filtered_path("/tmp/img/photo.jpg", &filter));
        assert!(!is_filtered_path("/tmp/README.md", &filter));
        let filter = vec!["[".to_string()];
        assert!(!is_filtered_path("/tmp/data.json", &filter));
        let filter: Vec<String> = vec![];
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
