#![deny(unsafe_code)]
// #![deny(missing_docs)]
use clap::Parser;
use color_eyre::eyre::{Report, Result};
use console::Emoji;
use derive_more::Display;
use dotenvy::dotenv;
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use playwright::Playwright;
use rayon::prelude::*;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{copy, Cursor};
use std::path::{Path, PathBuf};
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

pub type Void = Result<(), Report>;

static CHECKMARK: Emoji<'_, '_> = Emoji("✅ ", "☑ ");

// Environment variables
const LOG_LEVEL: &str = "ACORN_LOG_LEVEL";
const DISPLAY_LEVEL: bool = true;
const DISPLAY_LINE_NUMBER: bool = false;
const DISPLAY_TARGET: bool = false;
const DISPLAY_THREAD_ID: bool = false;
const DISPLAY_THREAD_NAME: bool = false;

#[derive(Clone, Debug)]
pub struct Endpoint<'a> {
    pub provider: Provider,
    pub domain: String,
    pub queries: HashMap<&'a str, &'a str>,
}
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
    /// Return the domain name of the bucket's `uri` as a string. If the `uri`
    /// is not a valid URI, return a default domain name based on the `provider`.
    fn get_domain(&self) -> String {
        match URI::try_from(self.uri.as_str()) {
            Ok(uri) => uri.host().unwrap().to_string(),
            Err(_) => match self.provider {
                Provider::GitHub => "github.com".to_string(),
                Provider::GitLab => "gitlab.com".to_string(),
            },
        }
    }
    fn download_files(self, output: PathBuf) {
        match self.provider {
            Provider::GitHub => unimplemented!(),
            Provider::GitLab => {
                fn get_parent(path: String) -> String {
                    PathBuf::from(PathBuf::from(path).parent().unwrap())
                        .display()
                        .to_string()
                }
                let paths = self.clone().get_file_paths();
                let progress = ProgressBar::new(paths.len() as u64);
                let template = "{spinner:.green} {pos}/{len} [{bar:.green}] {msg}";
                progress.set_style(ProgressStyle::with_template(template).unwrap());
                info!(
                    "=> Downloading {} research data from {}...",
                    self.schema_type, &self.uri
                );
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
    fn get_file_paths(self) -> Vec<String> {
        match self.provider {
            Provider::GitHub => unimplemented!(),
            Provider::GitLab => {
                let id = self.id.to_string();
                let domain = self.get_domain();
                // let endpoint = Endpoint {
                //     provider: self.provider,
                //     domain: domain.clone(),
                //     queries: HashMap::from([
                //         ("path", self.schema_type.to_string().as_str()),
                //         ("per_page", "100"),
                //         ("recursive", "true"),
                //     ]),
                // };
                let url = format!(
                    "https://{}/api/v4/projects/{}/repository/tree?&per_page=100&recursive=true&path={}",
                    domain, id, self.schema_type
                );
                debug!(url = url.as_str(), "=> {} Endpoint", Label::using());
                match reqwest::blocking::get(url.clone()) {
                    Ok(response) if response.status().is_success() => {
                        // println!(
                        //     "Headers = {:#?}",
                        //     response.headers()//.get("x-total-pages")
                        // );
                        let content = response.text().unwrap();
                        let data: serde_json::Result<Vec<GitlabTreeEntry>> =
                            serde_json::from_str(&content);
                        match data {
                            Ok(entries) => entries
                                .into_iter()
                                .filter(|entry| entry.entry_type.eq(&EntryType::Blob))
                                .map(|entry| entry.path)
                                .collect(),
                            Err(_) => {
                                error!("=> {} Failed to deserialize GitLab data", Label::fail());
                                vec![]
                            }
                        }
                    }
                    Ok(_) | Err(_) => {
                        error!("=> {} Failed to get paths from {}", Label::fail(), url);
                        vec![]
                    }
                }
            }
        }
    }
}
impl BucketsConfig {
    pub fn read_json(path: PathBuf) -> serde_json::Result<BucketsConfig> {
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
        let data: serde_json::Result<BucketsConfig> = serde_json::from_str(&content);
        let label = match data {
            Ok(_) => Label::using(),
            Err(_) => Label::invalid(),
        };
        trace!("=> {} Bucket configuration = {:#?}", label, data.dimmed());
        data
    }
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
        Some(Commands::Lint { path, .. }) => {
            fn check_file(path: &Path) {
                match ResearchActivity::read(path.into()) {
                    Ok(data) => match data.lint() {
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
                    let id = "acorn";
                    let output_path = match output {
                        Some(path) => match create_dir_all(path) {
                            Ok(_) => Ok(format!("{}/{}.pdf", path.display(), id)),
                            Err(err) => {
                                error!(
                                    path = path.clone().into_os_string().into_string().unwrap(),
                                    "=> {} Failed to create directory",
                                    Label::fail()
                                );
                                Err(err)
                            }
                        },
                        None => unreachable!(),
                    };
                    let runtime = tokio_runtime();
                    info!("=> {} Tokio runtime", Label::using());
                    runtime.block_on(async {
                        match output_path {
                            Ok(output_path_string) => {
                                let data = match ResearchActivity::read(path.clone().unwrap()) {
                                    Ok(data) => data,
                                    Err(_) => unimplemented!(),
                                };
                                let content = template::factsheet(data).to_string();
                                debug!("=> {} Rendered template", Label::using());
                                println!("{}", content);
                                let playwright = Playwright::initialize().await.unwrap();
                                playwright.prepare().unwrap(); // Install browsers
                                let chromium = playwright.chromium();
                                let browser =
                                    chromium.launcher().headless(true).launch().await.unwrap();
                                let context = browser.context_builder().build().await.unwrap();
                                let page = context.new_page().await.unwrap();
                                page.set_content_builder(&content)
                                    .set_content()
                                    .await
                                    .unwrap();
                                page.pdf_builder()
                                    .path(output_path_string.into())
                                    // .height(playwright::api::Length::WithUnit("36in"))
                                    // .width(playwright::api::Length::WithUnit("24in"))
                                    .print_background(true)
                                    .pdf()
                                    .await
                                    .unwrap();
                                page.close(Some(false)).await.unwrap();
                            }
                            Err(_) => {
                                error!("=> {} Failed to create output directory", Label::fail());
                            }
                        }
                    });
                    Ok(())
                }
                Target::Poster => Ok(()),
                Target::Highlight => Ok(()),
            }
        }
        None => {
            println!("ACORN without a subcommand currently does nothing");
            Ok(())
        }
    }
}
