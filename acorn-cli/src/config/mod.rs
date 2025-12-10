//! Application configuration functions and data structures
//!
//! ACORN is configured by a JSON or YAML file (typically named `.acorn.json`)
//!
//! The ACORN configuration file configures what buckets should be downloaded, readability analysis, <span title="Large Language Model">LLM</span> settings, and more.
//!
use acorn_lib::io::{files_all, parent, read_file, FromPath};
use acorn_lib::prelude::{self, create_dir_all, io, Cursor, File, PathBuf};
use acorn_lib::util::{suffix, Label, MimeType, ToAbsoluteString};
use acorn_lib::Repository;
use color_eyre::eyre;
use core::fmt::Debug;
use derive_more::Display;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use serde_with::skip_serializing_none;
use tracing::{debug, error, trace, warn};

const IGNORE: [&str; 5] = [".gitignore", ".gitlab-ci.yml", ".gitkeep", ".DS_Store", "README.md"];

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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationConfiguration {
    /// List of buckets
    pub buckets: Vec<Bucket>,
}
/// Type for GitLab tree entry
#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    /// List of files and directories
    ///
    /// See <https://docs.gitlab.com/api/repositories/#list-repository-tree>
    #[display("tree")]
    Tree,
    /// Base64 enoded content
    ///
    /// See <https://docs.gitlab.com/api/repositories/#get-a-blob-from-repository>
    #[display("blob")]
    Blob,
}
/// Struct for bucket data
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    /// Bucket name
    ///
    /// See <https://schema.org/name>
    pub name: String,
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
/// Struct for [GitHub] tree entry
///
/// [GitHub]: https://docs.github.com/en/rest
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GithubTreeEntry {
    /// Path of tree entry
    ///
    /// The path inside the repository. Used to get content of subdirectories.
    pub path: String,
    /// Mode of tree entry
    pub mode: String,
    /// Type of tree entry
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    /// [SHA1] of entry
    ///
    /// [SHA1]: https://en.wikipedia.org/wiki/SHA-1
    pub sha: String,
    /// Size of associated data
    /// ### Note
    /// > Not included for "tree" type entries
    pub size: Option<u64>,
    /// URL of associated data API endpoint
    ///
    /// Basically, a combination of the API endpoint and the SHA
    pub url: String,
}
/// Struct for [GitHub] tree API response
///
/// GitHub API endpoint for trees returns
/// ```json
/// {
///   "sha": "...",
///   "url": "<endpoint>/repos/<owner>/<repo>/git/trees/<sha>",
///   "tree": [...],
///   "truncated": false
/// }
/// ```
/// where `"tree"` is a list of [GithubTreeEntry].
///
/// ### Example Endpoint
/// > `https://api.github.com/repos/jhwohlgemuth/pwsh-prelude/git/trees/master?recursive=1`
///
/// See [documentation] for more information
///
/// [GitHub]: https://docs.github.com/en/rest
/// [documentation]: https://docs.github.com/en/rest/git/trees?apiVersion=2022-11-28#get-a-tree
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GithubTreeResponse {
    /// SHA1 of tree
    pub sha: String,
    /// URL of associated data API endpoint
    pub url: String,
    /// List of [GithubTreeEntry]
    pub tree: Vec<GithubTreeEntry>,
    /// Whether tree is truncated
    pub truncated: bool,
}
/// Struct for GitLab tree entry
///
/// See <https://docs.gitlab.com/api/repositories/#list-repository-tree>
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitlabTreeEntry {
    /// Integer ID of GitLab project
    ///
    /// See <https://docs.gitlab.com/api/projects/#get-a-single-project> for more information
    pub id: String,
    /// Name of tree entry
    pub name: String,
    /// Type of tree entry
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    /// Path of tree entry
    ///
    /// The path inside the repository. Used to get content of subdirectories.
    pub path: String,
    /// Mode of tree entry
    pub mode: String,
}
impl Bucket {
    /// Parse GitHub tree entries
    fn parse_github_response(response: Response) -> Vec<String> {
        let content = response.text().unwrap();
        let data: Result<GithubTreeResponse> = serde_json::from_str(&content);
        match data {
            | Ok(GithubTreeResponse { tree, .. }) => {
                debug!("=> {} {} Tree entries", Label::found(), tree.len());
                tree.into_iter().filter(GithubTreeEntry::is_blob).map(GithubTreeEntry::path).collect()
            }
            | Err(why) => {
                error!("=> {} Process tree entries - {why}", Label::fail());
                vec![]
            }
        }
    }
    /// Parse GitLab tree entries
    fn parse_gitlab_response(response: Response) -> Vec<String> {
        let content = response.text().unwrap();
        let data: Result<Vec<GitlabTreeEntry>> = serde_json::from_str(&content);
        debug!("=> {} {} Tree entries", Label::found(), data.as_ref().unwrap().len());
        match data {
            | Ok(entries) => entries.into_iter().filter(GitlabTreeEntry::is_blob).map(GitlabTreeEntry::path).collect(),
            | Err(why) => {
                error!("=> {} Process tree entries - {why}", Label::fail());
                vec![]
            }
        }
    }
    /// Get hosting domain from bucket struct
    fn domain(&self) -> String {
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
    fn tree(&self, directory: &str, page: Option<u32>) -> eyre::Result<Response, reqwest::Error> {
        let url = self.tree_url(directory, page);
        let client = Client::new();
        client.get(url.unwrap_or_default()).header(USER_AGENT, "rust-web-api-client").send()
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
                        "https://{}/api/v4/projects/{}/repository/tree?&per_page={}&page={}&recursive=true&path={}",
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
    pub fn copy_files(self: Bucket, output: PathBuf) -> usize {
        if self.code_repository.clone().is_local() {
            let location = self.code_repository.clone().location();
            let bucket_root = match location.uri() {
                | Some(value) => PathBuf::from(value.path().to_string()).to_absolute_string(),
                | None => {
                    unimplemented!()
                }
            };
            let paths = self
                .clone()
                .file_paths("")
                .into_iter()
                .filter(|path| !IGNORE.iter().any(|x| path.ends_with(x)))
                .filter(|path| PathBuf::from(path).is_file())
                .collect::<Vec<String>>();
            let total_data: usize = count_json_files(paths.clone());
            let total_images: usize = count_image_files(paths.clone());
            let message = operations_complete_message(self.name, total_data, total_images);
            let progress = ProgressBar::new(paths.len() as u64);
            paths.par_iter().for_each(|path| {
                progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
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
            progress.set_style(ProgressStyle::with_template("{msg}").unwrap());
            progress.finish_with_message(message);
            total_data + total_images
        } else {
            0
        }
    }
    /// Download files from bucket to local directory
    ///
    /// Ignores files listed in [`IGNORE`]
    pub fn download_files(self: Bucket, output: PathBuf) -> usize {
        let paths = self
            .clone()
            .file_paths("")
            .into_iter()
            .filter(|path| !IGNORE.iter().any(|x| path.ends_with(x)))
            .collect::<Vec<String>>();
        let total_data: usize = count_json_files(paths.clone());
        let total_images: usize = count_image_files(paths.clone());
        let message = operations_complete_message(self.name, total_data, total_images);
        let progress = ProgressBar::new(paths.len() as u64);
        let client = Client::new();
        paths.par_iter().for_each(|path| {
            progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
            progress.set_message(format!("Downloading {path}"));
            let folder = format!("{}/{}", output.display(), parent(path.clone()).display());
            create_dir_all(folder.clone()).unwrap();
            if let Ok(mut file) = File::create(format!("{}/{}", output.display(), path)) {
                if let Some(url) = self.code_repository.raw_url(path.to_string()) {
                    match client.get(url).header(USER_AGENT, "rust-web-api-client").send() {
                        | Ok(response) => match response.bytes() {
                            | Ok(bytes) => {
                                let mut content = Cursor::new(bytes);
                                let _ = io::copy(&mut content, &mut file);
                            }
                            | Err(why) => {
                                error!(path, "=> {} Convert to bytes - {why}", Label::fail());
                            }
                        },
                        | Err(why) => {
                            error!(path, "=> {} Download file - {why}", Label::fail());
                        }
                    }
                }
            };
            progress.inc(1);
        });
        progress.set_style(ProgressStyle::with_template("{msg}").unwrap());
        progress.finish_with_message(message);
        total_data + total_images
    }
    fn file_paths(self: Bucket, directory: &str) -> Vec<String> {
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
            | Repository::GitHub { .. } => match self.tree(directory, None) {
                | Ok(response) if response.status().is_success() => Bucket::parse_github_response(response),
                | Ok(_) | Err(_) => {
                    let url = self.tree_url(directory, None);
                    debug!(url, "=> {}", Label::using());
                    error!("=> {} Get file paths for {} bucket", Label::fail(), self.name.to_uppercase().red());
                    vec![]
                }
            },
            | Repository::GitLab { .. } => match self.tree(directory, FIRST_PAGE) {
                | Ok(response) if response.status().is_success() => {
                    let paths = (FIRST_PAGE.unwrap_or_default()..=page_count(&response))
                        .into_par_iter()
                        .map(|page| self.clone().file_paths_for_page(directory, Some(page)))
                        .reduce(Vec::new, |a, b| [a, b].concat());
                    trace!("{:#?}", response);
                    paths
                }
                | Ok(_) | Err(_) => {
                    let url = self.tree_url(directory, FIRST_PAGE);
                    debug!(url, "=> {}", Label::using());
                    error!("=> {} Get file paths for {} bucket", Label::fail(), self.name.to_uppercase().red());
                    vec![]
                }
            },
        }
    }
    fn file_paths_for_page(self: Bucket, directory: &str, page: Option<u32>) -> Vec<String> {
        match self.tree(directory, page) {
            | Ok(response) if response.status().is_success() => match self.tree(directory, page) {
                | Ok(response) if response.status().is_success() => Bucket::parse_gitlab_response(response),
                | Ok(_) | Err(_) => {
                    let url = self.tree_url(directory, Some(1));
                    error!(url, page, "=> {} Failed to get paths", Label::fail());
                    vec![]
                }
            },
            | Ok(_) | Err(_) => {
                let url = self.tree_url(directory, page);
                error!(url, page, "=> {} Failed to get paths", Label::fail());
                vec![]
            }
        }
    }
}
impl ApplicationConfiguration {
    /// Read and parse application configuration file (JSON or YAML)
    pub fn read(path: PathBuf) -> Option<ApplicationConfiguration> {
        let content = match MimeType::from_path(path.clone()) {
            | MimeType::Json => match ApplicationConfiguration::read_json(path.clone()) {
                | Ok(value) => Some(value),
                | Err(_) => None,
            },
            | MimeType::Yaml => match ApplicationConfiguration::read_yaml(path.clone()) {
                | Ok(value) => Some(value),
                | Err(_) => None,
            },
            | _ => unimplemented!("Unsupported configuration file extension"),
        };
        if let Some(content) = content {
            Some(content)
        } else {
            error!(path = path.to_str().unwrap(), "=> {} Import configuration", Label::fail());
            None
        }
    }
    /// Read configuration (e.g., `.acorn.json`) using Serde and [`ApplicationConfiguration`] struct
    fn read_json(path: PathBuf) -> Result<ApplicationConfiguration> {
        let content = match read_file(path.clone()) {
            | Ok(value) if !value.is_empty() => value,
            | Ok(_) | Err(_) => {
                error!(
                    path = path.to_str().unwrap(),
                    "=> {} Bucket configuration content is not valid",
                    Label::fail()
                );
                "{}".to_owned()
            }
        };
        let data: Result<ApplicationConfiguration> = serde_json::from_str(&content);
        let label = match data {
            | Ok(_) => Label::using(),
            | Err(_) => Label::invalid(),
        };
        trace!("=> {} Bucket configuration = {:#?}", label, data.dimmed());
        data
    }
    /// Read configuration (e.g., `.acorn.yml`) using Serde and [`ApplicationConfiguration`] struct
    fn read_yaml(path: PathBuf) -> serde_yml::Result<ApplicationConfiguration> {
        let content = match read_file(path.clone()) {
            | Ok(value) => value,
            | Err(_) => {
                error!(
                    path = path.to_str().unwrap(),
                    "=> {} Bucket configuration content is not valid",
                    Label::fail()
                );
                "".to_owned()
            }
        };
        let data: serde_yml::Result<ApplicationConfiguration> = serde_yml::from_str(&content);
        let label = match data {
            | Ok(_) => Label::output(),
            | Err(_) => Label::fail(),
        };
        debug!("=> {} Bucket configuration = {:#?}", label, data.dimmed());
        data
    }
}
impl GithubTreeEntry {
    fn path(self) -> String {
        self.path
    }
    fn is_blob(&self) -> bool {
        self.entry_type.eq(&EntryType::Blob)
    }
}
impl GitlabTreeEntry {
    fn path(self) -> String {
        self.path
    }
    fn is_blob(&self) -> bool {
        self.entry_type.eq(&EntryType::Blob)
    }
}
fn count_json_files(paths: Vec<String>) -> usize {
    paths.clone().into_iter().filter(|path| path.to_lowercase().ends_with(".json")).count()
}
fn count_image_files(paths: Vec<String>) -> usize {
    paths.into_iter().filter(has_image_extension).count()
}
fn operations_complete_message(name: String, json_count: usize, image_count: usize) -> String {
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
    format!(
        "  {}Obtained {} file{} from {} bucket{}",
        if total > 0 { Label::CHECKMARK } else { Label::CAUTION },
        if total > 0 {
            total.green().to_string()
        } else {
            total.yellow().to_string()
        },
        suffix(total),
        name.to_uppercase(),
        message,
    )
}
#[allow(clippy::ptr_arg)]
fn has_image_extension(path: &String) -> bool {
    path.to_lowercase().ends_with(".png") || path.to_lowercase().ends_with(".jpg")
}

#[cfg(test)]
mod tests;
