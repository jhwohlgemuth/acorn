// #![deny(missing_docs)]
use clap::Parser;
use color_eyre::eyre;
use dotenvy::dotenv;
use indicatif::{ProgressBar, ProgressStyle};
use playwright::Playwright;
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;
use tracing::instrument;
use tracing::{error, info, warn};
use tracing_indicatif::IndicatifLayer;
use tracing_log::AsTrace;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

mod cli;
mod template;

use acorn_lib::schema::*;
use acorn_lib::util::*;
use acorn_lib::*;
use cli::{Arguments, Check, Commands, Target};
use template::generate_factsheet_pdf;

pub type Void = eyre::Result<(), eyre::Report>;

// Environment variables
const LOG_LEVEL: &str = "ACORN_LOG_LEVEL";
const DISPLAY_LEVEL: bool = true;
const DISPLAY_LINE_NUMBER: bool = false;
const DISPLAY_TARGET: bool = false;
const DISPLAY_THREAD_ID: bool = false;
const DISPLAY_THREAD_NAME: bool = false;

#[instrument]
fn main() -> Void {
    color_eyre::install()?;
    let args = Arguments::parse();
    match dotenv() {
        | Ok(_) if !dotenvy::var(LOG_LEVEL).unwrap_or_default().is_empty() => {
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
        | _ => {
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
        | Some(Commands::Check { path, skip, .. }) => {
            if let Some(path) = path {
                let paths = if path.is_dir() {
                    get_all_files(path.into(), None)
                } else {
                    vec![path.into()]
                };
                if skip.contains(&Check::Validation) {
                    warn!("=> {} Schema validation", Label::skip());
                } else {
                    let total: usize = ResearchActivity::check(paths.clone());
                    if total > 0 {
                        error!("=> {} Found {} schema errors in total", Label::fail(), total);
                    }
                }
                if skip.contains(&Check::Analysis) {
                    warn!("=> {} Prose analysis", Label::skip());
                } else {
                    // TODO: Use JSON output to create customized and consolidated error count and output
                    ResearchActivity::analyze(paths);
                }
            }
            Ok(())
        }
        | Some(Commands::Download { buckets, output, .. }) => {
            if let Some(path) = buckets {
                match BucketsConfig::read_json(path.clone()) {
                    | Ok(content) => {
                        content.buckets.into_iter().for_each(|bucket| {
                            if let Some(path) = output {
                                bucket.download_files(path.to_path_buf());
                            }
                        });
                    }
                    | Err(err) => {
                        error!("=> {} Failed to download data - {}", Label::fail(), err);
                    }
                };
            }
            Ok(())
        }
        | Some(Commands::Export {
            output,
            path,
            size: _,
            target,
            ..
        }) => {
            match target {
                | Target::FactSheet => {
                    tokio_runtime().block_on(async {
                        let playwright = Playwright::initialize().await.unwrap();
                        playwright.install_chromium().unwrap(); // Install browsers
                        let chromium = playwright.chromium();
                        let browser = chromium.launcher().headless(true).launch().await.unwrap();
                        let context = browser.context_builder().build().await.unwrap();
                        let page = context.new_page().await.unwrap();
                        let input_path = match path.clone() {
                            | Some(value) => value,
                            | None => unimplemented!(),
                        };
                        let paths = if input_path.is_dir() {
                            get_all_files(input_path, None)
                        } else {
                            vec![input_path]
                        };
                        let progress = ProgressBar::new(paths.len() as u64);
                        progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
                        for path in paths {
                            progress.set_message(format!("Generating PDF for {}", path.display()));
                            generate_factsheet_pdf(path, &page, output).await;
                            progress.inc(1);
                        }
                        progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                        let _close = page.close(Some(false)).await;
                    });
                    Ok(())
                }
                | Target::Poster => Ok(()),
                | Target::Highlight => Ok(()),
            }
        }
        | Some(Commands::Format { path, .. }) => {
            if let Some(path) = path {
                let paths = if path.is_dir() {
                    get_all_files(path.into(), None)
                } else {
                    vec![path.into()]
                };
                paths.par_iter().for_each(|path| match ResearchActivity::read(path.clone()) {
                    | Ok(data) => {
                        let formatted = data.format(Some(path.clone()));
                        let filepath = path_to_string(path.clone());
                        let mut file = match File::create(path) {
                            | Ok(file) => file,
                            | Err(why) => {
                                error!(file = filepath, "=> {} Create - {}", Label::fail(), why);
                                std::process::exit(exitcode::UNAVAILABLE);
                            }
                        };
                        let content = serde_json::to_string_pretty(&formatted).unwrap();
                        match file.write_all(content.as_bytes()) {
                            | Ok(_) => {
                                info!(file = filepath, "=> {} Formatted", Label::pass());
                            }
                            | Err(why) => {
                                error!(file = filepath, "=> {} Write - {}", Label::fail(), why);
                                std::process::exit(exitcode::UNAVAILABLE);
                            }
                        }
                    }
                    | Err(err) => {
                        error!("=> {} Read research activity data at - {}", Label::fail(), err);
                    }
                });
            }
            Ok(())
        }
        | None => Ok(()),
    }
}
