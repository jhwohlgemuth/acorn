use clap::Parser;
use color_eyre::eyre::{Report, Result};
use derive_more::Display;
use dotenvy::dotenv;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use tracing::instrument;
use tracing::{debug, error};
use tracing_log::AsTrace;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use uriparse::URI;

mod cli;
mod schema;
mod util;

use cli::{Arguments, Commands};
use schema::{ResearchActivity, SchemaType};
use util::*;

pub type Void = Result<(), Report>;

// Environment variables
const LOG_LEVEL: &str = "ACORN_LOG_LEVEL";
const DISPLAY_LEVEL: bool = true;
const DISPLAY_LINE_NUMBER: bool = false;
const DISPLAY_TARGET: bool = false;
const DISPLAY_THREAD_ID: bool = false;
const DISPLAY_THREAD_NAME: bool = true;

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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitlabTreeEntry {
    id: String,
    name: String,
    #[serde(rename = "type")]
    entry_type: String,
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
    async fn download_files(&self) {
        match self.provider {
            Provider::GitHub => unimplemented!(),
            Provider::GitLab => {
                let files = self.get_files().await;
                for file in files {
                    let url = format!("{}/-/raw/main/{}/{}", self.uri, self.schema_type, file);
                    let response = reqwest::get(url).await.unwrap();
                    let content = response.text().await.unwrap();
                    let output = format!("./content/{}", self.schema_type);
                    std::fs::create_dir_all(output.clone()).unwrap();
                    let mut file = File::create(format!("{}/{}", output, file)).unwrap();
                    file.write_all(content.as_bytes()).unwrap();
                }
            }
        };
    }
    async fn get_files(&self) -> Vec<String> {
        match self.provider {
            Provider::GitHub => unimplemented!(),
            Provider::GitLab => {
                let id = self.id.to_string();
                let domain = self.get_domain();
                let url = format!(
                    "https://{}/api/v4/projects/{}/repository/tree?recursive=true&path={}",
                    domain, id, self.schema_type
                );
                let response = reqwest::get(url).await.unwrap();
                let content = response.text().await.unwrap();
                let data: serde_json::Result<Vec<GitlabTreeEntry>> = serde_json::from_str(&content);
                match data {
                    Ok(entries) => entries.into_iter().map(|entry| entry.name).collect(),
                    Err(_) => vec![],
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
        debug!("=> {} Bucket configuration = {:#?}", label, data.dimmed());
        data
    }
}
fn tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}
pub fn read_json(path: PathBuf) -> serde_json::Result<ResearchActivity> {
    let content = match read_file(path.clone()) {
        Ok(value) if !value.is_empty() => value,
        Ok(_) | Err(_) => {
            error!(
                path = path.to_str().unwrap(),
                "=> {} Project content is not valid",
                Label::fail()
            );
            "{}".to_owned()
        }
    };
    let data: serde_json::Result<ResearchActivity> = serde_json::from_str(&content);
    let label = match data {
        Ok(_) => Label::using(),
        Err(_) => Label::invalid(),
    };
    debug!("=> {} Project = {:#?}", label, data.dimmed());
    data
}

#[instrument]
fn main() -> Void {
    color_eyre::install()?;
    let args = Arguments::parse();
    match dotenv() {
        Ok(_) if !dotenvy::var(LOG_LEVEL).unwrap_or_default().is_empty() => {
            // Configure tracing with a .env file
            let format = fmt::layer()
                .with_level(DISPLAY_LEVEL)
                .with_line_number(DISPLAY_LINE_NUMBER)
                .with_target(DISPLAY_TARGET)
                .with_thread_ids(DISPLAY_THREAD_ID)
                .with_thread_names(DISPLAY_THREAD_NAME)
                .compact();
            tracing_subscriber::registry()
                .with(format)
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
            if let Some(path) = path {
                match read_json(PathBuf::from(path)) {
                    Ok(data) => {
                        data.check();
                    }
                    Err(err) => {
                        error!("Failed to read research activity data - {err}");
                    }
                }
            }
            Ok(())
        }
        Some(Commands::Download { config, .. }) => {
            let runtime = tokio_runtime();
            if let Some(path) = config {
                match BucketsConfig::read_json(path.clone()) {
                    Ok(content) => {
                        content.buckets.into_iter().for_each(|bucket| {
                            runtime.block_on(async {
                                bucket.download_files().await;
                            });
                        });
                    }
                    Err(err) => {
                        error!("Failed to download data - {err}");
                    }
                };
            }
            Ok(())
        }
        None => {
            println!("ACORN without a subcommand currently does nothing");
            Ok(())
        }
    }
}
