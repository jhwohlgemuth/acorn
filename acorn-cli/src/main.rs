// #![deny(missing_docs)]
use clap::Parser;
use color_eyre::eyre;
use console::Emoji;
use derive_more::Display;
use dotenvy::dotenv;
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use playwright::api::Page;
use playwright::Playwright;
use rayon::prelude::*;
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fmt::Debug;
use std::fs::{create_dir_all, remove_file, File};
use std::io::{copy, Cursor};
use std::path::{self, Path, PathBuf};
use tracing::instrument;
use tracing::{debug, error, info, trace};
use tracing_indicatif::IndicatifLayer;
use tracing_log::AsTrace;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use uriparse::URI;

mod cli;
mod schema;
mod template;
mod util;

use cli::{Arguments, Commands, Target};
use schema::*;
use util::*;

pub type Void = eyre::Result<(), eyre::Report>;

static CHECKMARK: Emoji<'_, '_> = Emoji("✅ ", "☑ ");

// Environment variables
const LOG_LEVEL: &str = "ACORN_LOG_LEVEL";
const DISPLAY_LEVEL: bool = true;
const DISPLAY_LINE_NUMBER: bool = false;
const DISPLAY_TARGET: bool = false;
const DISPLAY_THREAD_ID: bool = false;
const DISPLAY_THREAD_NAME: bool = false;
// Application constants
const PROGRESS_BAR_TEMPLATE: &str = "{spinner:.green} {pos}/{len} [{bar:.green}] {msg}";

#[derive(Clone, Debug, Display, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[display("github")]
    GitHub,
    #[display("gitlab")]
    GitLab,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BucketsConfig {
    buckets: Vec<Bucket>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bucket {
    id: u64,
    parent: String,
    provider: Provider,
    #[serde(alias = "type")]
    schema_type: SchemaType,
    uri: String,
}
#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "lowercase")]
enum EntryType {
    #[display("tree")]
    Tree,
    #[display("blob")]
    Blob,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitlabTreeEntry {
    id: String,
    name: String,
    #[serde(rename = "type")]
    entry_type: EntryType,
    path: String,
    mode: String,
}
impl Bucket {
    fn parse(response: reqwest::blocking::Response) -> Vec<String> {
        let content = response.text().unwrap();
        let data: Result<Vec<GitlabTreeEntry>> = serde_json::from_str(&content);
        debug!(
            "=> {} {} GitLab tree entries",
            Label::found(),
            data.as_ref().unwrap().len()
        );
        match data {
            Ok(entries) => entries
                .into_iter()
                .filter(GitlabTreeEntry::is_blob)
                .map(GitlabTreeEntry::get_path)
                .collect(),
            Err(_) => {
                error!("=> {} Failed to process GitLab tree entries", Label::fail());
                vec![]
            }
        }
    }
    fn get_domain(&self) -> String {
        match URI::try_from(self.uri.as_str()) {
            Ok(uri) => uri.host().unwrap().to_string(),
            Err(_) => match self.provider {
                Provider::GitHub => "github.com".to_string(),
                Provider::GitLab => "gitlab.com".to_string(),
            },
        }
    }
    fn get_tree(&self, page: u32) -> eyre::Result<reqwest::blocking::Response, reqwest::Error> {
        let url = self.get_tree_url(page);
        reqwest::blocking::get(url)
    }
    fn get_tree_url(&self, page: u32) -> String {
        let Self {
            id, schema_type, ..
        } = self;
        let per_page = 3;
        let url = format!(
            "https://{}/api/v4/projects/{}/repository/tree?&per_page={}&page={}&recursive=true&path={}",
            self.get_domain(), id, per_page, page, schema_type
        );
        debug!(url = url.as_str(), "=> {}", Label::using());
        url
    }
    fn download_files(self: Bucket, output: PathBuf) {
        match self.provider {
            Provider::GitHub => unimplemented!(),
            Provider::GitLab => {
                info!(
                    "=> Downloading {} research data from {}...",
                    self.schema_type, &self.uri
                );
                let paths = self.clone().get_file_paths();
                let progress = ProgressBar::new(paths.len() as u64);
                progress.set_style(ProgressStyle::with_template(PROGRESS_BAR_TEMPLATE).unwrap());
                let client = Client::new();
                paths.par_iter().for_each(|path| {
                    progress.set_message(format!("Downloading {}", path));
                    let folder = format!("{}/{}", output.display(), get_parent(path.clone()));
                    std::fs::create_dir_all(folder.clone()).unwrap();
                    let mut file = File::create(format!("{}/{}", output.display(), path)).unwrap();
                    let url = format!("{}/-/raw/main/{}", self.uri, path);
                    let response = client.get(url).send().unwrap();
                    let mut content = Cursor::new(response.bytes().unwrap());
                    let _ = copy(&mut content, &mut file);
                    progress.inc(1);
                });
                progress.finish_with_message(format!("{}Done", CHECKMARK));
            }
        };
    }
    // https://docs.gitlab.com/ee/api/repositories.html
    fn get_file_paths(self: Bucket) -> Vec<String> {
        match self.provider {
            Provider::GitHub => unimplemented!(),
            Provider::GitLab => {
                fn get_page_count(response: &reqwest::blocking::Response) -> u32 {
                    fn parse_header(headers: &HeaderMap, key: &str) -> u32 {
                        match headers.get(key) {
                            Some(val) if !val.is_empty() => {
                                let value = val.to_str().unwrap().parse::<u32>().unwrap();
                                debug!("=> {} {} = {}", Label::using(), key, value);
                                value
                            }
                            Some(_) | None => 0,
                        }
                    }
                    let headers = response.headers();
                    parse_header(headers, "x-total-pages")
                }
                match self.get_tree(1) {
                    Ok(response) if response.status().is_success() => {
                        (1..=get_page_count(&response))
                            .into_par_iter()
                            .map(|page| self.clone().get_file_paths_for_page(page))
                            .reduce(std::vec::Vec::new, |a, b| [a, b].concat())
                    }
                    Ok(_) | Err(_) => {
                        let url = self.get_tree_url(1);
                        error!("=> {} Failed to get paths from {}", Label::fail(), url);
                        vec![]
                    }
                }
            }
        }
    }
    fn get_file_paths_for_page(self: Bucket, page: u32) -> Vec<String> {
        match self.get_tree(page) {
            Ok(response) if response.status().is_success() => match self.get_tree(page) {
                Ok(response) if response.status().is_success() => Bucket::parse(response),
                Ok(_) | Err(_) => {
                    let url = self.get_tree_url(1);
                    error!(url, page, "=> {} Failed to get paths", Label::fail());
                    vec![]
                }
            },
            Ok(_) | Err(_) => {
                let url = self.get_tree_url(page);
                error!(url, page, "=> {} Failed to get paths", Label::fail());
                vec![]
            }
        }
    }
}
impl BucketsConfig {
    pub fn read_json(path: PathBuf) -> Result<BucketsConfig> {
        let content = match read_file(path.clone()) {
            Ok(value) if !value.is_empty() => value,
            Ok(_) | Err(_) => {
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
            Ok(_) => Label::using(),
            Err(_) => Label::invalid(),
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
async fn generate_factsheet_pdf(path: PathBuf, page: &Page, output: &Option<PathBuf>) {
    let data = match ResearchActivity::read(path.clone()) {
        Ok(value) => value.format(Some(path.clone())),
        Err(err) => {
            error!(path = path.to_str().unwrap(), "=> {} {err}", Label::fail());
            unimplemented!()
        }
    };
    let content = template::factsheet(data.clone()).to_string();
    // NOTE: Create temporary index.html because rust-playwright does not support setting baseURL
    let index = path::absolute(path.parent().unwrap().join("index.html")).unwrap();
    if File::create(&index).is_ok() {
        page.goto_builder(&format!("file://{}", index.display()))
            .goto()
            .await
            .unwrap();
    };
    page.set_content_builder(&content)
        .set_content()
        .await
        .unwrap();
    match output {
        Some(output_dir) => match create_dir_all(output_dir.clone()) {
            Ok(_) => {
                let id = data.meta.id;
                let output_path = format!("{}/{}.pdf", output_dir.display(), id);
                debug!(path = output_path, "=> {} Output", Label::using());
                page.pdf_builder()
                    .path(output_path.into())
                    .print_background(true)
                    .pdf()
                    .await
                    .unwrap();
            }
            Err(err) => {
                error!(
                    path = output_dir.clone().into_os_string().into_string().unwrap(),
                    "=> {} Create directory - {err}",
                    Label::fail()
                );
            }
        },
        None => unreachable!(),
    };
    let _remove = remove_file(index);
}
#[instrument]
fn main() -> Void {
    color_eyre::install()?;
    let args = Arguments::parse();
    match dotenv() {
        Ok(_) if !dotenvy::var(LOG_LEVEL).unwrap_or_default().is_empty() => {
            // Configure tracing with a .env file
            let indicatif_layer = IndicatifLayer::new();
            let format = fmt::layer()
                .with_level(DISPLAY_LEVEL)
                .with_line_number(DISPLAY_LINE_NUMBER)
                .with_target(DISPLAY_TARGET)
                .with_thread_ids(DISPLAY_THREAD_ID)
                .with_thread_names(DISPLAY_THREAD_NAME)
                .compact();
            tracing_subscriber::registry()
                .with(format)
                .with(indicatif_layer)
                .with(EnvFilter::from_env(LOG_LEVEL))
                .init();
        }
        _ => {
            // Configure tracing with command line argument
            let format = fmt::format()
                .with_level(DISPLAY_LEVEL)
                .with_line_number(DISPLAY_LINE_NUMBER)
                .with_target(DISPLAY_TARGET)
                .with_thread_ids(DISPLAY_THREAD_ID)
                .with_thread_names(DISPLAY_THREAD_NAME)
                .without_time()
                .compact();
            tracing_subscriber::fmt()
                .with_max_level(args.verbose.log_level_filter().as_trace())
                .event_format(format)
                .init();
        }
    }
    match &args.command {
        Some(Commands::Check { path, .. }) => {
            fn check_file(path: &Path) {
                match ResearchActivity::read(path.into()) {
                    Ok(data) => match data.check() {
                        Ok(_) => {
                            info!("=> {} {} has no errors", Label::pass(), path.display());
                        }
                        Err(found) => {
                            error!(
                                "=> {} Found {} errors in {}: {:#?}",
                                Label::fail(),
                                get_error_count(found.clone()),
                                path.display(),
                                found,
                            );
                        }
                    },
                    Err(err) => {
                        error!(
                            "=> {} Read research activity data at {} - {}",
                            Label::fail(),
                            path.display(),
                            err
                        );
                    }
                }
            }
            if let Some(path) = path {
                if path.is_dir() {
                    match glob(&format!("{}/**/*.json", path.to_str().unwrap())) {
                        Ok(paths) => paths.collect::<Vec<_>>().par_iter().for_each(|path| {
                            check_file(path.as_ref().unwrap());
                        }),
                        Err(err) => {
                            error!("=> {} Failed to read glob pattern - {}", Label::fail(), err);
                        }
                    };
                } else {
                    check_file(&path.clone());
                }
            }
            Ok(())
        }
        Some(Commands::Download {
            buckets, output, ..
        }) => {
            if let Some(path) = buckets {
                match BucketsConfig::read_json(path.clone()) {
                    Ok(content) => {
                        content.buckets.into_iter().for_each(|bucket| {
                            if let Some(path) = output {
                                bucket.download_files(path.to_path_buf());
                            }
                        });
                    }
                    Err(err) => {
                        error!("=> {} Failed to download data - {}", Label::fail(), err);
                    }
                };
            }
            Ok(())
        }
        Some(Commands::Export {
            output,
            path,
            target,
            ..
        }) => {
            match target {
                Target::FactSheet => {
                    tokio_runtime().block_on(async {
                        let playwright = Playwright::initialize().await.unwrap();
                        playwright.prepare().unwrap(); // Install browsers
                        let chromium = playwright.chromium();
                        let browser = chromium.launcher().headless(true).launch().await.unwrap();
                        let context = browser.context_builder().build().await.unwrap();
                        let page = context.new_page().await.unwrap();
                        let input_path = match path.clone() {
                            Some(value) => value,
                            None => unimplemented!(),
                        };
                        if input_path.is_dir() {
                            match glob(&format!("{}/**/*.json", input_path.to_str().unwrap())) {
                                Ok(values) => {
                                    let paths = values.flatten().collect::<Vec<_>>();
                                    let progress = ProgressBar::new(paths.len() as u64);
                                    progress.set_style(
                                        ProgressStyle::with_template(PROGRESS_BAR_TEMPLATE)
                                            .unwrap(),
                                    );
                                    for path in paths {
                                        progress.set_message(format!(
                                            "Generating PDF for {}",
                                            path.display()
                                        ));
                                        generate_factsheet_pdf(path, &page, output).await;
                                        progress.inc(1);
                                    }
                                    progress.finish_with_message(format!("{}Done", CHECKMARK));
                                }
                                Err(err) => {
                                    error!(
                                        "=> {} Failed to read glob pattern - {}",
                                        Label::fail(),
                                        err
                                    );
                                }
                            };
                        } else {
                            info!("=> Generating PDF for {}...", input_path.display());
                            generate_factsheet_pdf(input_path, &page, output).await;
                        }
                        let _close = page.close(Some(false)).await;
                    });
                    Ok(())
                }
                Target::Poster => Ok(()),
                Target::Highlight => Ok(()),
            }
        }
        Some(Commands::Format { .. }) => Ok(()),
        None => {
            let _ = download_vale(None);
            Ok(())
        }
    }
}
