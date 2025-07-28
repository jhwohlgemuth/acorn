//! # ACORN Library
//!
//! `acorn-lib` provides functions for working with [ACORN](https://acorn.ornl.gov) data and supports the ACORN CLI.
use color_eyre::eyre;
use derive_more::Display;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rayon::prelude::*;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fmt::Debug;
use std::fs::File;
use std::io::{copy, Cursor};
use std::path::PathBuf;
use std::vec;
use tracing::{debug, error, info, trace};
use uriparse::URI;

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
        // TODO: Accept URL encoded project path instead of numeric ID (https://docs.gitlab.com/api/rest/#namespaced-paths)
        /// Integer ID of GitLab project
        ///
        /// See <https://docs.gitlab.com/api/projects/#get-a-single-project> for more information
        id: u64,
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
/// Struct for GitLab tree entry
///
/// See <https://docs.gitlab.com/api/repositories/#list-repository-tree>
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitlabTreeEntry {
    /// Integer ID of GitLab project
    ///
    /// See <https://docs.gitlab.com/api/projects/#get-a-single-project>
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
/// Struct for GitLab/GitHub release data
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Release {
    pub name: String,
    pub tag_name: String,
    #[serde(alias = "body")]
    pub description: String,
    pub created_at: String,
    #[serde(alias = "published_at")]
    pub released_at: String,
}
impl Bucket {
    /// Parse GitLab tree entries
    fn parse(response: reqwest::blocking::Response) -> Vec<String> {
        let content = response.text().unwrap();
        let data: Result<Vec<GitlabTreeEntry>> = serde_json::from_str(&content);
        debug!("=> {} {} GitLab tree entries", Label::found(), data.as_ref().unwrap().len());
        match data {
            | Ok(entries) => entries.into_iter().filter(GitlabTreeEntry::is_blob).map(GitlabTreeEntry::path).collect(),
            | Err(_) => {
                error!("=> {} Failed to process GitLab tree entries", Label::fail());
                vec![]
            }
        }
    }
    /// Get hosting domain from bucket struct
    fn domain(&self) -> String {
        fn default_domain(repository: Repository) -> String {
            match repository {
                | Repository::GitHub { .. } => "github.com".to_string(),
                | Repository::GitLab { .. } => "gitlab.com".to_string(),
            }
        }
        match &self.code_repository {
            | Repository::GitHub { uri } => match URI::try_from(uri.as_str()) {
                | Ok(uri) => uri.host().unwrap().to_string(),
                | Err(_) => default_domain(self.code_repository.clone()),
            },
            | Repository::GitLab { uri, .. } => match URI::try_from(uri.as_str()) {
                | Ok(uri) => uri.host().unwrap().to_string(),
                | Err(_) => default_domain(self.code_repository.clone()),
            },
        }
    }
    fn tree(&self, directory: &str, page: u32) -> eyre::Result<reqwest::blocking::Response, reqwest::Error> {
        let url = self.tree_url(directory, page);
        reqwest::blocking::get(url)
    }
    fn tree_url(&self, directory: &str, page: u32) -> String {
        // TODO: Replace id with url-encoded project path - https://docs.gitlab.com/api/rest/#namespaced-paths
        let id = match &self.code_repository {
            | Repository::GitHub { .. } => todo!(),
            | Repository::GitLab { id, .. } => id.to_string(),
        };
        let per_page = 100;
        let url = format!(
            "https://{}/api/v4/projects/{}/repository/tree?&per_page={}&page={}&recursive=true&path={}",
            self.domain(),
            id,
            per_page,
            page,
            directory
        );
        debug!(url = url.as_str(), "=> {}", Label::using());
        url
    }
    /// Download files from bucket to local directory
    ///
    /// Ignores files listed in [`IGNORE`]
    pub fn download_files(self: Bucket, output: PathBuf) -> usize {
        match self.code_repository {
            | Repository::GitHub { ref uri, .. } => todo!("Add support for GitHub repositories like {uri}"),
            | Repository::GitLab { ref uri, .. } => {
                info!("=> Downloading research data from {}...", uri.clone());
                let paths = self
                    .clone()
                    .file_paths("")
                    .into_iter()
                    .filter(|path| !IGNORE.iter().any(|x| path.ends_with(x)))
                    .collect::<Vec<String>>();
                let progress = ProgressBar::new(paths.len() as u64);
                progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
                let client = Client::new();
                paths.par_iter().for_each(|path| {
                    let url = format!("{uri}/-/raw/main/{path}");
                    progress.set_message(format!("Downloading {path}"));
                    let folder = format!("{}/{}", output.display(), parent(path.clone()));
                    std::fs::create_dir_all(folder.clone()).unwrap();
                    if let Ok(mut file) = File::create(format!("{}/{}", output.display(), path)) {
                        match client.get(url).send() {
                            | Ok(response) => match response.bytes() {
                                | Ok(bytes) => {
                                    let mut content = Cursor::new(bytes);
                                    let _ = copy(&mut content, &mut file);
                                }
                                | Err(why) => {
                                    error!(path, "=> {} Failed to convert to bytes - {why}", Label::fail());
                                }
                            },
                            | Err(why) => {
                                error!(path, "=> {} Failed to download - {why}", Label::fail());
                            }
                        }
                    };
                    progress.inc(1);
                });
                let total_data: usize = paths.clone().into_iter().filter(|path| path.to_lowercase().ends_with(".json")).count();
                let total_images: usize = paths
                    .into_iter()
                    .filter(|path| path.to_lowercase().ends_with(".png") || path.to_lowercase().ends_with(".jpg"))
                    .count();
                let total = total_data + total_images;
                let message = if total_data != total_images {
                    let recommendation = if total_data > total_images {
                        "Do you need to add some images?"
                    } else {
                        "Do you need to add some JSON files?"
                    };
                    format!(
                        " ({} data file{}, {} image{} - {})",
                        total_data.yellow(),
                        suffix(total_data),
                        total_images.yellow(),
                        suffix(total_images),
                        recommendation.italic(),
                    )
                } else {
                    "".to_string()
                };
                progress.set_style(ProgressStyle::with_template("{msg}").unwrap());
                progress.finish_with_message(format!(
                    "  {}Downloaded {} {} file{}{}",
                    if total > 0 { Label::CHECKMARK } else { Label::CAUTION },
                    if total > 0 {
                        total.green().to_string()
                    } else {
                        total.yellow().to_string()
                    },
                    self.clone().name.to_uppercase(),
                    suffix(total),
                    message,
                ));
                total
            }
        }
    }
    fn file_paths(self: Bucket, directory: &str) -> Vec<String> {
        const FIRST_PAGE: u32 = 1;
        match self.code_repository {
            | Repository::GitHub { .. } => todo!(),
            | Repository::GitLab { .. } => {
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
                match self.tree(directory, FIRST_PAGE) {
                    | Ok(response) if response.status().is_success() => (FIRST_PAGE..=page_count(&response))
                        .into_par_iter()
                        .map(|page| self.clone().file_paths_for_page(directory, page))
                        .reduce(std::vec::Vec::new, |a, b| [a, b].concat()),
                    | Ok(_) | Err(_) => {
                        let url = self.tree_url(directory, FIRST_PAGE);
                        debug!(url, "=> {}", Label::using());
                        error!(
                            "=> {} Failed to get file paths for {} bucket",
                            Label::fail(),
                            self.name.to_uppercase().red(),
                        );
                        vec![]
                    }
                }
            }
        }
    }
    fn file_paths_for_page(self: Bucket, directory: &str, page: u32) -> Vec<String> {
        match self.tree(directory, page) {
            | Ok(response) if response.status().is_success() => match self.tree(directory, page) {
                | Ok(response) if response.status().is_success() => Bucket::parse(response),
                | Ok(_) | Err(_) => {
                    let url = self.tree_url(directory, 1);
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
    /// Read buckets configuration using Serde and [`BucketsConfig`] struct
    pub fn read_json(path: PathBuf) -> Result<BucketsConfig> {
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
    fn latest_release(self) -> Option<Release> {
        match self.releases() {
            | releases if releases.is_empty() => None,
            | releases => Some(releases[0].clone()),
        }
    }
    fn releases(self) -> Vec<Release> {
        let url = match &self {
            | Repository::GitHub { uri } => match URI::try_from(uri.as_str()) {
                | Ok(uri) => {
                    let host = uri.host().unwrap().to_string();
                    let path = uri.path();
                    format!("https://api.{}/repos{}/releases", host, path)
                }
                | Err(_) => {
                    error!(uri, "=> {} Parse GitHub URI", Label::fail());
                    uri.to_string()
                }
            },
            | Repository::GitLab { id, .. } => {
                format!("https://code.ornl.gov/api/v4/projects/{}/releases", id)
            }
        };
        debug!(url, "=> {}", Label::using());
        let client = Client::new();
        match client.get(url).header(USER_AGENT, "rust-web-api-client").send() {
            | Ok(response) => match response.text() {
                | Ok(text) => {
                    trace!(text, "=> {}", Label::using());
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
    }
}

#[cfg(test)]
mod tests;
