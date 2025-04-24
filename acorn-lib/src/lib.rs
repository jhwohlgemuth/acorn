use color_eyre::eyre;
use derive_more::Display;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rayon::prelude::*;
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fmt::Debug;
use std::fs::File;
use std::io::{copy, Cursor};
use std::path::PathBuf;
use tracing::{debug, error, info, trace};
use uriparse::URI;

pub mod analyzer;
pub mod constants;
pub mod doctor;
pub mod schema;
pub mod util;

use crate::schema::SchemaType;
use crate::util::*;

const SCHEMA_TYPES: [SchemaType; 3] = [SchemaType::Project, SchemaType::Organization, SchemaType::Highlight];

#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    #[display("tree")]
    Tree,
    #[display("blob")]
    Blob,
}
#[derive(Clone, Debug, Display, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum Repository {
    #[display("github")]
    GitHub { uri: String },
    #[display("gitlab")]
    GitLab {
        /// Integer ID of GitLab project
        ///
        /// See <https://docs.gitlab.com/api/projects/#get-a-single-project> for more information
        id: u64,
        uri: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BucketsConfig {
    pub buckets: Vec<Bucket>,
}
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitlabTreeEntry {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    pub path: String,
    pub mode: String,
}
impl Bucket {
    fn parse(response: reqwest::blocking::Response) -> Vec<String> {
        let content = response.text().unwrap();
        let data: Result<Vec<GitlabTreeEntry>> = serde_json::from_str(&content);
        debug!("=> {} {} GitLab tree entries", Label::found(), data.as_ref().unwrap().len());
        match data {
            | Ok(entries) => entries
                .into_iter()
                .filter(GitlabTreeEntry::is_blob)
                .map(GitlabTreeEntry::get_path)
                .collect(),
            | Err(_) => {
                error!("=> {} Failed to process GitLab tree entries", Label::fail());
                vec![]
            }
        }
    }
    fn get_domain(&self) -> String {
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
    fn get_tree(&self, schema_type: SchemaType, page: u32) -> eyre::Result<reqwest::blocking::Response, reqwest::Error> {
        let url = self.get_tree_url(schema_type, page);
        reqwest::blocking::get(url)
    }
    fn get_tree_url(&self, schema_type: SchemaType, page: u32) -> String {
        let id = match &self.code_repository {
            | Repository::GitHub { .. } => todo!(),
            | Repository::GitLab { id, .. } => id.to_string(),
        };
        let per_page = 100;
        let url = format!(
            "https://{}/api/v4/projects/{}/repository/tree?&per_page={}&page={}&recursive=true&path={}",
            self.get_domain(),
            id,
            per_page,
            page,
            schema_type
        );
        debug!(url = url.as_str(), "=> {}", Label::using());
        url
    }
    pub fn download_files(self: Bucket, output: PathBuf) -> usize {
        fn get_suffix(value: usize) -> String {
            (if value == 1 { "" } else { "s" }).to_string()
        }
        let counts = SCHEMA_TYPES.into_iter().map(|schema_type| match self.code_repository {
            | Repository::GitHub { .. } => todo!(),
            | Repository::GitLab { ref uri, .. } => {
                info!("=> Downloading {} research data from {}...", schema_type, uri.clone());
                let paths = self
                    .clone()
                    .get_file_paths(schema_type.clone())
                    .into_iter()
                    .filter(|path| !(path.ends_with(".gitkeep") || path.ends_with(".DS_Store")))
                    .collect::<Vec<String>>();
                let progress = ProgressBar::new(paths.len() as u64);
                progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
                let client = Client::new();
                paths.par_iter().for_each(|path| {
                    let url = format!("{}/-/raw/main/{}", uri, path);
                    progress.set_message(format!("Downloading {}", path));
                    let folder = format!("{}/{}", output.display(), get_parent(path.clone()));
                    std::fs::create_dir_all(folder.clone()).unwrap();
                    let mut file = File::create(format!("{}/{}", output.display(), path)).unwrap();
                    let response = client.get(url).send().unwrap();
                    let mut content = Cursor::new(response.bytes().unwrap());
                    let _ = copy(&mut content, &mut file);
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
                        get_suffix(total_data),
                        total_images.yellow(),
                        get_suffix(total_images),
                        recommendation.italic(),
                    )
                } else {
                    "".to_string()
                };
                progress.set_style(ProgressStyle::with_template("{msg}").unwrap());
                progress.finish_with_message(format!(
                    "  {}Downloaded {} {} {} file{}{}",
                    if total > 0 { Label::CHECKMARK } else { Label::CAUTION },
                    if total > 0 {
                        total.green().to_string()
                    } else {
                        total.yellow().to_string()
                    },
                    self.clone().name.to_uppercase(),
                    schema_type,
                    get_suffix(total),
                    message,
                ));
                total
            }
        });
        counts.into_iter().sum()
    }
    // https://docs.gitlab.com/ee/api/repositories.html
    fn get_file_paths(self: Bucket, schema_type: SchemaType) -> Vec<String> {
        const FIRST_PAGE: u32 = 1;
        match self.code_repository {
            | Repository::GitHub { .. } => unimplemented!(),
            | Repository::GitLab { .. } => {
                fn get_page_count(response: &reqwest::blocking::Response) -> u32 {
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
                match self.get_tree(schema_type.clone(), FIRST_PAGE) {
                    | Ok(response) if response.status().is_success() => (FIRST_PAGE..=get_page_count(&response))
                        .into_par_iter()
                        .map(|page| self.clone().get_file_paths_for_page(schema_type.clone(), page))
                        .reduce(std::vec::Vec::new, |a, b| [a, b].concat()),
                    | Ok(_) | Err(_) => {
                        let url = self.get_tree_url(schema_type.clone(), FIRST_PAGE);
                        debug!(url, "=> {}", Label::using());
                        error!(
                            "=> {} Failed to get file paths for {} {} bucket",
                            Label::fail(),
                            self.name.to_uppercase().red(),
                            schema_type.red()
                        );
                        vec![]
                    }
                }
            }
        }
    }
    fn get_file_paths_for_page(self: Bucket, schema_type: SchemaType, page: u32) -> Vec<String> {
        match self.get_tree(schema_type.clone(), page) {
            | Ok(response) if response.status().is_success() => match self.get_tree(schema_type.clone(), page) {
                | Ok(response) if response.status().is_success() => Bucket::parse(response),
                | Ok(_) | Err(_) => {
                    let url = self.get_tree_url(schema_type.clone(), 1);
                    error!(url, page, "=> {} Failed to get paths", Label::fail());
                    vec![]
                }
            },
            | Ok(_) | Err(_) => {
                let url = self.get_tree_url(schema_type, page);
                error!(url, page, "=> {} Failed to get paths", Label::fail());
                vec![]
            }
        }
    }
}
impl BucketsConfig {
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
    fn get_path(self) -> String {
        self.path
    }
    fn is_blob(&self) -> bool {
        self.entry_type.eq(&EntryType::Blob)
    }
}

#[cfg(test)]
mod tests;
