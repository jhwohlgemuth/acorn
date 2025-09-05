//! # 🌱 ACORN Library
//!
//! `acorn-lib` is a one-stop-shop for everything related to building and maintaining research activity data (RAD)-related technology, including the Accessible Content Optimization for Research Needs (ACORN) tool.
//! The modules, structs, enums and constants found here support the ACORN CLI, which checks, analyzes, and exports research activity data into useable formats.
//!
use color_eyre::eyre;
use derive_more::Display;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rayon::prelude::*;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use serde_with::skip_serializing_none;
use std::fmt::Debug;
use std::fs::File;
use std::io::{copy, Cursor};
use std::path::PathBuf;
use std::vec;
use tracing::{debug, error, trace, warn};
use uriparse::URI;
use urlencoding::encode;

pub mod analyzer;
pub mod constants;
pub mod doctor;
pub mod powerpoint;
pub mod schema;
pub mod util;

use crate::util::*;

/// Files to ignore
///
/// - `.gitignore`
/// - `.gitkeep`
/// - `.DS_Store`
/// - `README.md`
pub const IGNORE: [&str; 5] = [".gitignore", ".gitlab-ci.yml", ".gitkeep", ".DS_Store", "README.md"];

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
/// Git hosting repository data
#[derive(Clone, Debug, Display, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum Repository {
    /// GitHub
    ///
    /// See <https://docs.github.com/en/rest/reference/repos>
    #[display("github")]
    GitHub {
        /// Repository URI
        uri: String,
    },
    /// GitLab
    ///
    /// See <https://docs.gitlab.com/api/repositories/#list-repository-tree>
    #[display("gitlab")]
    GitLab {
        /// Integer ID of GitLab project
        ///
        /// See <https://docs.gitlab.com/api/projects/#get-a-single-project> for more information
        id: Option<u64>,
        /// Repository URI
        uri: String,
    },
}
/// Struct for buckets configuration
///
/// ### Example buckets.json
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
pub struct BucketsConfig {
    /// List of buckets
    pub buckets: Vec<Bucket>,
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
    /// SHA1 of entry
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
/// Struct for release data from GitLab or GitHub
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Release {
    /// Name of release
    pub name: String,
    /// Tag name of release
    /// ### Example
    /// > `v1.0.0`
    pub tag_name: String,
    /// Prose description of release
    #[serde(alias = "body")]
    pub description: String,
    /// Date of release creation
    pub created_at: String,
    /// Date of release publication
    #[serde(alias = "published_at")]
    pub released_at: String,
}
impl Bucket {
    /// Parse GitHub tree entries
    fn parse_github_response(response: reqwest::blocking::Response) -> Vec<String> {
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
    fn parse_gitlab_response(response: reqwest::blocking::Response) -> Vec<String> {
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
            | Repository::GitHub { uri } => match URI::try_from(uri.as_str()) {
                | Ok(uri) => uri.host().unwrap().to_string(),
                | Err(_) => "github.com".to_string(),
            },
            | Repository::GitLab { uri, .. } => match URI::try_from(uri.as_str()) {
                | Ok(uri) => uri.host().unwrap().to_string(),
                | Err(_) => "gitlab.com".to_string(),
            },
        }
    }
    fn tree(&self, directory: &str, page: Option<u32>) -> eyre::Result<reqwest::blocking::Response, reqwest::Error> {
        let url = self.tree_url(directory, page);
        let client = Client::new();
        client.get(url.unwrap_or_default()).header(USER_AGENT, "rust-web-api-client").send()
    }
    fn tree_url(&self, directory: &str, page: Option<u32>) -> Option<String> {
        match &self.code_repository {
            | Repository::GitHub { uri } => {
                let parsed = match URI::try_from(uri.as_str()) {
                    | Ok(value) => value,
                    | Err(why) => {
                        warn!(uri, "=> {} Parse GitHub URI - {why}", Label::fail());
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
    /// Download files from bucket to local directory
    ///
    /// Ignores files listed in [`IGNORE`]
    pub fn download_files(self: Bucket, output: PathBuf) -> usize {
        fn count_json_files(paths: Vec<String>) -> usize {
            paths.clone().into_iter().filter(|path| path.to_lowercase().ends_with(".json")).count()
        }
        fn count_image_files(paths: Vec<String>) -> usize {
            paths.into_iter().filter(has_image_extension).count()
        }
        fn download_complete_message(name: String, json_count: usize, image_count: usize) -> String {
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
                "  {}Downloaded {} {} file{}{}",
                if total > 0 { Label::CHECKMARK } else { Label::CAUTION },
                if total > 0 {
                    total.green().to_string()
                } else {
                    total.yellow().to_string()
                },
                name.to_uppercase(),
                suffix(total),
                message,
            )
        }
        fn has_image_extension(path: &String) -> bool {
            path.to_lowercase().ends_with(".png") || path.to_lowercase().ends_with(".jpg")
        }
        let paths = self
            .clone()
            .file_paths("")
            .into_iter()
            .filter(|path| !IGNORE.iter().any(|x| path.ends_with(x)))
            .collect::<Vec<String>>();
        let total_data: usize = count_json_files(paths.clone());
        let total_images: usize = count_image_files(paths.clone());
        let message = download_complete_message(self.name, total_data, total_images);
        let progress = ProgressBar::new(paths.len() as u64);
        let client = Client::new();
        paths.par_iter().for_each(|path| {
            progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
            progress.set_message(format!("Downloading {path}"));
            let folder = format!("{}/{}", output.display(), parent(path.clone()).display());
            std::fs::create_dir_all(folder.clone()).unwrap();
            if let Ok(mut file) = File::create(format!("{}/{}", output.display(), path)) {
                if let Some(url) = self.code_repository.raw_url(path.to_string()) {
                    match client.get(url).header(USER_AGENT, "rust-web-api-client").send() {
                        | Ok(response) => match response.bytes() {
                            | Ok(bytes) => {
                                let mut content = Cursor::new(bytes);
                                let _ = copy(&mut content, &mut file);
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
        fn page_count(response: &reqwest::blocking::Response) -> u32 {
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
                        .reduce(std::vec::Vec::new, |a, b| [a, b].concat());
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
impl BucketsConfig {
    /// Read and parse buckets configuration file (JSON or YAML)
    pub fn read(path: PathBuf) -> Option<BucketsConfig> {
        let content = match MimeType::from_path(path.clone()) {
            | MimeType::Json => match BucketsConfig::read_json(path.clone()) {
                | Ok(value) => Some(value),
                | Err(_) => None,
            },
            | MimeType::Yaml => match BucketsConfig::read_yaml(path.clone()) {
                | Ok(value) => Some(value),
                | Err(_) => None,
            },
            | _ => unimplemented!("Unsupported configuration file extension"),
        };
        if let Some(content) = content {
            Some(content)
        } else {
            error!(path = path.to_str().unwrap(), "=> {} Import configuration", Label::fail());
            std::process::exit(exitcode::UNAVAILABLE);
        }
    }
    /// Read buckets configuration (e.g., `buckets.json`) using Serde and [`BucketsConfig`] struct
    fn read_json(path: PathBuf) -> Result<BucketsConfig> {
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
        let data: Result<BucketsConfig> = serde_json::from_str(&content);
        let label = match data {
            | Ok(_) => Label::using(),
            | Err(_) => Label::invalid(),
        };
        trace!("=> {} Bucket configuration = {:#?}", label, data.dimmed());
        data
    }
    /// Read buckets configuration (e.g., `buckets.yaml`) using Serde and [`BucketsConfig`] struct
    fn read_yaml(path: PathBuf) -> serde_yml::Result<BucketsConfig> {
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
        let data: serde_yml::Result<BucketsConfig> = serde_yml::from_str(&content);
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
impl Repository {
    /// Get metadata for latest release of a Gitlab or GitHub repository
    pub fn latest_release(self) -> Option<Release> {
        match self.releases() {
            | releases if releases.is_empty() => None,
            | releases => {
                let release = releases[0].clone();
                trace!("=> {} Latest {:#?}", Label::using(), release);
                Some(release)
            }
        }
    }
    fn id(&self) -> Option<String> {
        match self {
            | Repository::GitHub { .. } => None,
            | Repository::GitLab { id, uri } => match URI::try_from(uri.as_str()) {
                | Ok(value) => {
                    let mut path = value.path().to_string();
                    path.remove(0);
                    let encoded = encode(&path).to_string();
                    trace!(encoded, "=> {} ID", Label::using());
                    Some(encoded)
                }
                | Err(why) => {
                    warn!(uri, "=> {} Parse GitLab URI - {why}", Label::fail());
                    match id {
                        | Some(value) => Some(value.to_string()),
                        | None => None,
                    }
                }
            },
        }
    }
    fn releases(self) -> Vec<Release> {
        let maybe_url = match &self {
            | Repository::GitHub { uri } => match URI::try_from(uri.as_str()) {
                | Ok(uri) => {
                    let host = uri.host().unwrap().to_string();
                    let path = uri.path();
                    let endpoint = Some(format!("https://api.{host}/repos{path}/releases"));
                    println!("{endpoint:#?}");
                    endpoint
                }
                | Err(_) => {
                    error!(uri, "=> {} Parse GitHub URI", Label::fail());
                    None
                }
            },
            | Repository::GitLab { uri, .. } => match self.id() {
                | Some(id) => match URI::try_from(uri.as_str()) {
                    | Ok(uri) => {
                        let host = uri.host().unwrap().to_string();
                        Some(format!("https://{host}/api/v4/projects/{id}/releases"))
                    }
                    | Err(why) => {
                        error!(uri, "=> {} Parse GitLab URI - {why}", Label::fail());
                        None
                    }
                },
                | None => None,
            },
        };
        if let Some(url) = maybe_url {
            debug!(url, "=> {}", Label::using());
            let client = Client::new();
            match client.get(url).header(USER_AGENT, "rust-web-api-client").send() {
                | Ok(response) => match response.text() {
                    | Ok(text) => {
                        let releases: Vec<Release> = match serde_json::from_str(&text) {
                            | Ok(values) => values,
                            | Err(why) => {
                                error!("=> {} Parse {} API JSON response - {why}", self, Label::fail());
                                vec![]
                            }
                        };
                        releases
                    }
                    | Err(why) => {
                        error!("=> {} Parse {} API text response - {why}", self, Label::fail());
                        vec![]
                    }
                },
                | Err(why) => {
                    error!("=> {} Download {} releases - {why}", self, Label::fail());
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }
    /// Get URL for raw data of a file at a given path
    fn raw_url(&self, path: String) -> Option<String> {
        match self {
            | Repository::GitHub { uri, .. } => match URI::try_from(uri.clone().as_str()) {
                | Ok(ref value) => Some(format!("https://raw.githubusercontent.com{}/refs/heads/main/{path}", value.path())),
                | Err(why) => {
                    error!(uri, "=> {} Parse GitHub URI - {why}", Label::fail());
                    None
                }
            },
            | Repository::GitLab { uri, .. } => Some(format!("{uri}/-/raw/main/{path}")),
        }
    }
}

#[cfg(test)]
mod tests;
