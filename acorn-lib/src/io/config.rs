//! Application configuration functions and data structures
//!
//! ACORN is configured by a JSON or YAML file (typically named `.acorn.json`)
//!
//! The ACORN configuration file configures what buckets should be downloaded, readability analysis, <span title="Large Language Model">LLM</span> settings, and more.
//!
use crate::io::api::github::{TreeEntry as GithubTreeEntry, TreeResponse as GithubTreeResponse};
use crate::io::api::gitlab::{TreeEntry as GitlabTreeEntry, TreeResponse as GitlabTreeResponse};
use crate::io::api::Endpoint;
use crate::io::ApiResult;
use crate::io::{files_all, get, parent, read_file, with_progress_bar, write_file, FromPath, InputOutput};
use crate::prelude::{self, create_dir_all, exit, io, Cursor, File, PathBuf};
use crate::util::{detect_json, suffix, Label, MimeType, StringConversion};
use crate::{Location, Repository, Scheme};
use bon::Builder;
use color_eyre::eyre::eyre;
use core::fmt::Debug;
use derive_more::Display;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reqwest::header::{HeaderMap, USER_AGENT};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::{debug, error, trace, warn};

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
    pub runners: Option<Vec<Runner>>,
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
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[builder(start_fn = at, on(String, into))]
#[serde(rename_all = "camelCase")]
/// CI/CD runner configuration entry
pub struct Runner {
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
}
impl ApplicationConfiguration {
    /// Parse ACORN configuration in JSON or YAML format
    pub fn parse(content: impl AsRef<str>) -> Option<Self> {
        let content = content.as_ref();
        let data = if detect_json(content) {
            match Self::parse_json(content) {
                | Ok(value) => Some(value),
                | Err(_) => None,
            }
        } else {
            match Self::parse_yaml(content) {
                | Ok(value) => Some(value),
                | Err(_) => None,
            }
        };
        if let Some(data) = data {
            debug!("=> {} Parse ACORN configuration = {:#?}", Label::using(), data.dimmed());
            Some(data)
        } else {
            error!("=> {} Parse ACORN configuration", Label::fail());
            None
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
                error!(path = path.to_str().unwrap(), "=> {} ACORN configuration JSON content", Label::fail());
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
                error!(path = path.to_str().unwrap(), "=> {} ACORN configuration YAML content", Label::fail());
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
    /// Parse GitHub tree entries
    async fn parse_github_response(response: Response) -> Vec<String> {
        let content = response.text().await.unwrap_or_default();
        let data: serde_json::Result<GithubTreeResponse> = serde_json::from_str(&content);
        match data {
            | Ok(GithubTreeResponse { tree, .. }) => {
                debug!("=> {} {} Tree entries", Label::found(), tree.len());
                tree.into_iter().filter(GithubTreeEntry::is_blob).map(GithubTreeEntry::path).collect()
            }
            | Err(why) => {
                error!("=> {} Process tree entries — {why}", Label::fail());
                vec![]
            }
        }
    }
    /// Parse GitLab tree entries
    async fn parse_gitlab_response(response: Response) -> Vec<String> {
        let content = response.text().await.unwrap_or_default();
        let data: serde_json::Result<GitlabTreeResponse> = serde_json::from_str(&content);
        match data {
            | Ok(entries) => {
                debug!("=> {} {} Tree entries", Label::found(), entries.len());
                entries.into_iter().filter(GitlabTreeEntry::is_blob).map(GitlabTreeEntry::path).collect()
            }
            | Err(why) => {
                error!("=> {} Process tree entries — {why}", Label::fail());
                vec![]
            }
        }
    }
    /// Get hosting domain from bucket struct
    pub(crate) fn domain(&self) -> String {
        match &self.code_repository {
            | Repository::GitHub { location } => match location.uri() {
                | Some(uri) => match uri.scheme() {
                    | uriparse::Scheme::HTTPS => uri.host().unwrap().to_string(),
                    | _ => todo!("Add support for file:///"),
                },
                | None => todo!("Handle invalid GitHub URI"),
            },
            | Repository::GitLab { location, .. } => match location.uri() {
                | Some(uri) => match uri.scheme() {
                    | uriparse::Scheme::HTTPS => uri.host().unwrap().to_string(),
                    | _ => todo!("Add support for file:///"),
                },
                | None => todo!("Handle invalid GitLab URI"),
            },
            | Repository::Git { .. } => todo!("Add support for generic repositories"),
        }
    }
    async fn tree(&self, directory: &str, page: Option<u32>) -> ApiResult<Response> {
        let url = self.tree_url(directory, page);
        let client = Client::new();
        client
            .get(url.unwrap_or_default())
            .header(USER_AGENT, "rust-web-api-client")
            .send()
            .await
            .map_err(|why| eyre!("Failed to send HTTP request — {why}"))
    }
    fn tree_url(&self, directory: &str, page: Option<u32>) -> Option<String> {
        match &self.code_repository {
            | Repository::Git { .. } => None,
            | Repository::GitHub { location } => {
                let parsed = match location.uri() {
                    | Some(value) => value,
                    | None => {
                        warn!("=> {} Parse GitHub URI", Label::fail());
                        return None;
                    }
                };
                let path = parsed.path();
                let url = format!("https://api.{}/repos{}/git/trees/main?recursive=1", self.domain(), path);
                debug!(url = url.as_str(), "=> {}", Label::using());
                Some(url)
            }
            | Repository::GitLab { .. } => {
                if let Some(id) = &self.code_repository.id() {
                    let per_page = 100;
                    let url = format!(
                        "https://{}/api/v4/projects/{}/repository/tree?&per_page={}&page={}&recursive=true&path= {}",
                        self.domain(),
                        id,
                        per_page,
                        page.unwrap_or_default(),
                        directory
                    );
                    debug!(url = url.as_str(), "=> {}", Label::using());
                    Some(url)
                } else {
                    None
                }
            }
        }
    }
    /// Copy files from (local) bucket to local directory
    /// ### Notes
    /// - Ignores files listed in [`IGNORE`]
    /// - Only copies files from local repositories
    // TODO: Refactor to return ApiResult<usize>
    pub fn copy_files(self: Bucket, output: PathBuf) -> usize {
        let Bucket { name, code_repository, .. } = self.clone();
        if code_repository.clone().is_local() {
            let uri = code_repository.clone().location().uri();
            let bucket_root = match uri {
                | Some(value) => PathBuf::from(value.path().to_string()).to_absolute_string(),
                | None => {
                    unimplemented!()
                }
            };
            let items = files_all(PathBuf::from(bucket_root.clone()), None)
                .into_iter()
                .map(|x| x.display().to_string())
                .filter(|path| !IGNORE.iter().any(|x| path.ends_with(x)))
                .filter(|path| PathBuf::from(path).is_file())
                .collect::<Vec<String>>();
            // TODO: Refactor to use with_progress_bar and async file copying for consistency with download_files
            let progress = ProgressBar::new(items.len() as u64);
            progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
            items.par_iter().for_each(|path| {
                progress.set_message(format!("Copying {path}"));
                let relative = path.strip_prefix(&format!("{}/", bucket_root.trim_end_matches("/"))).unwrap();
                let parent_path = PathBuf::from(relative).parent().unwrap().to_path_buf();
                let output_filepath = output.join(relative);
                let folder = format!("{}/{}", output.display(), parent_path.display());
                let _ = create_dir_all(folder.clone());
                if let Ok(mut file) = File::create(output_filepath) {
                    match prelude::read(path.clone()) {
                        | Ok(bytes) => {
                            let mut content = Cursor::new(bytes);
                            let _ = io::copy(&mut content, &mut file);
                            progress.inc(1);
                        }
                        | Err(why) => {
                            error!(path, "=> {} Read file as bytes - {why}", Label::fail());
                        }
                    }
                }
            });
            let total_data: usize = count_json_files(&items);
            let total_images: usize = count_image_files(&items);
            let finish_message = operations_complete_message(name, total_data, total_images);
            progress.set_style(ProgressStyle::with_template("{msg}").unwrap());
            progress.finish_with_message(finish_message);
            total_data + total_images
        } else {
            0
        }
    }
    /// Download files from bucket to local directory
    ///
    /// Ignores files listed in [`IGNORE`]
    ///
    /// Downloads files concurrently using buffered streams
    // TODO: Refactor to return ApiResult<usize>
    pub async fn download_files(self: Bucket, output: PathBuf, threads: usize) -> usize {
        let Bucket { name, code_repository, .. } = self.clone();
        let items = self
            .clone()
            .file_paths("")
            .await
            .into_iter()
            .filter(|path| !IGNORE.iter().any(|x| path.ends_with(x)))
            .collect::<Vec<String>>();
        let message: fn(&String) -> String = |path| format!("Downloading {path}");
        let operation = move |path: String| {
            let output = output.clone();
            let repository = code_repository.clone();
            async move {
                let folder = format!("{}/{}", output.display(), parent(path.clone()).display());
                match create_dir_all(folder.clone()) {
                    | Ok(_) => {}
                    | Err(why) => {
                        error!(path, "=> {} Create output folder — {why}", Label::fail());
                    }
                }
                if let Ok(mut file) = File::create(format!("{}/{}", output.display(), path)) {
                    if let Some(url) = repository.raw_url(path.to_string()) {
                        match get(url).send().await {
                            | Ok(response) => match response.bytes().await {
                                | Ok(bytes) => {
                                    let mut content = Cursor::new(bytes);
                                    let _ = io::copy(&mut content, &mut file);
                                }
                                | Err(why) => {
                                    error!(path, "=> {} Convert to bytes — {why}", Label::fail());
                                }
                            },
                            | Err(why) => {
                                error!(path, "=> {} Download file — {why}", Label::fail());
                            }
                        }
                    }
                }
                Ok(())
            }
        };
        let total_data: usize = count_json_files(&items);
        let total_images: usize = count_image_files(&items);
        let finish_message = |_| operations_complete_message(name, total_data, total_images);
        match with_progress_bar(items, message, operation, finish_message, Some(threads)).await {
            | Ok(_) => {}
            | Err(why) => {
                error!("=> {} Download files with progress bar - {why}", Label::fail());
            }
        }
        total_data + total_images
    }
    async fn file_paths(self: Bucket, directory: &str) -> Vec<String> {
        const FIRST_PAGE: Option<u32> = Some(1);
        fn page_count(response: &Response) -> u32 {
            fn parse_header(headers: &HeaderMap, key: &str) -> u32 {
                match headers.get(key) {
                    | Some(val) if !val.is_empty() => {
                        let value = val.to_str().unwrap().parse::<u32>().unwrap();
                        debug!("=> {} {} = {}", Label::using(), key, value);
                        value
                    }
                    | Some(_) | None => 0,
                }
            }
            let headers = response.headers();
            parse_header(headers, "x-total-pages")
        }
        match self.code_repository {
            | Repository::Git { .. } => {
                let path = match self.code_repository.clone().location().uri() {
                    | Some(value) => PathBuf::from(value.path().to_string()),
                    | None => {
                        unimplemented!()
                    }
                };
                files_all(path, None).into_iter().map(|x| x.display().to_string()).collect()
            }
            | Repository::GitHub { .. } => match self.tree(directory, None).await {
                | Ok(response) if response.status().is_success() => Bucket::parse_github_response(response).await,
                | Ok(_) | Err(_) => {
                    let url = self.tree_url(directory, None);
                    let name = self.name.unwrap_or_else(|| "Bucket".to_string());
                    debug!(url, "=> {}", Label::using());
                    error!("=> {} Get file paths for {} bucket", Label::fail(), name.to_uppercase().red());
                    vec![]
                }
            },
            | Repository::GitLab { .. } => match self.tree(directory, FIRST_PAGE).await {
                | Ok(response) if response.status().is_success() => {
                    let start = FIRST_PAGE.unwrap_or_default();
                    let end = page_count(&response);
                    let tasks = (start..=end).map(|page| self.clone().file_paths_for_page(directory, Some(page)));
                    let paths = join_all(tasks).await.into_iter().flatten().collect::<Vec<String>>();
                    trace!("{:#?}", response);
                    paths
                }
                | Ok(_) | Err(_) => {
                    let url = self.tree_url(directory, FIRST_PAGE);
                    let name = self.name.unwrap_or_else(|| "Bucket".to_string());
                    debug!(url, "=> {}", Label::using());
                    error!("=> {} Get file paths for {} bucket", Label::fail(), name.to_uppercase().red());
                    vec![]
                }
            },
        }
    }
    async fn file_paths_for_page(self: Bucket, directory: &str, page: Option<u32>) -> Vec<String> {
        match self.tree(directory, page).await {
            | Ok(response) if response.status().is_success() => Bucket::parse_gitlab_response(response).await,
            | Ok(_) | Err(_) => {
                let url = self.tree_url(directory, Some(1));
                error!(url, page, "=> {} Failed to get paths", Label::fail());
                vec![]
            }
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
fn count_json_files(paths: &[String]) -> usize {
    paths.iter().filter(|&path| path.to_lowercase().ends_with(".json")).cloned().count()
}
fn count_image_files(paths: &[String]) -> usize {
    paths.iter().filter(|&x| has_image_extension(x)).cloned().count()
}
fn operations_complete_message(name: Option<String>, json_count: usize, image_count: usize) -> String {
    let total = json_count + image_count;
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
        "  {}Obtained {} file{} from {bucket_description}{}",
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
