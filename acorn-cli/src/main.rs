// #![deny(missing_docs)]
use clap::Parser;
use color_eyre::eyre;
use dotenvy::dotenv;
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use playwright::Playwright;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use tracing::instrument;
use tracing::{error, info};
use tracing_indicatif::IndicatifLayer;
use tracing_log::AsTrace;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

mod cli;
mod template;

use acorn_lib::analyzer::{StaticAnalyzer, Vale, ValeConfig};
use acorn_lib::schema::*;
use acorn_lib::util::*;
use acorn_lib::*;
use cli::{Arguments, Commands, Target};
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
        | Some(Commands::Check { path, .. }) => {
            fn check_file(path: PathBuf) -> usize {
                match ResearchActivity::read(path.clone()) {
                    | Ok(data) => match data.clone().check() {
                        | Ok(_) => {
                            info!("=> {} {} has no schema errors", Label::pass(), path.display());
                            0
                        }
                        | Err(found) => {
                            let count = get_error_count(found.clone());
                            error!("=> {} Found {} errors in {}: {:#?}", Label::fail(), count, path.display(), found,);
                            count
                        }
                    },
                    | Err(err) => {
                        error!("=> {} Read research activity data at {} - {}", Label::fail(), path.display(), err);
                        0
                    }
                }
            }
            fn analyze_file(path: &Path) {
                let config = ValeConfig::default().save();
                let vale = Vale::init().config(config).build().download(None);
                match vale.clone().sync() {
                    | Ok(_) => match ResearchActivity::read(path.into()) {
                        | Ok(data) => {
                            // TODO: Handle vale runs that exit with error code. Maybe use --no-exit?
                            vale.analyze(data.clone().meta.id, data.extract_prose());
                        }
                        | Err(err) => {
                            error!("=> {} Read research activity data at - {}", Label::fail(), err);
                        }
                    },
                    | Err(err) => {
                        error!("=> {} Vale sync - {}", Label::fail(), err);
                    }
                }
            }
            if let Some(path) = path {
                if path.is_dir() {
                    // TODO: Implement total error count that works with par_iter
                    // Schema validation
                    match glob(&format!("{}/**/*.json", path.to_str().unwrap())) {
                        | Ok(paths) => {
                            // let mut arr = [0, 7, 9, 11];
                            // arr.par_iter_mut().for_each(|p| *p -= 1);
                            // println!("{:?}", arr);
                            // let mut count = 0;
                            paths.collect::<Vec<_>>().par_iter_mut().filter_map(|x| x.as_mut().ok()).for_each(|path| {
                                let _error_count = check_file(path.clone());
                                // count += error_count;
                            })
                        }
                        | Err(err) => {
                            error!("=> {} Failed to read glob pattern - {}", Label::fail(), err);
                        }
                    };
                    // Prose static analysis
                    match glob(&format!("{}/**/*.json", path.to_str().unwrap())) {
                        | Ok(paths) => {
                            let config = ValeConfig::default().save();
                            let vale = Vale::init().config(config).build().download(None);
                            match vale.clone().sync() {
                                | Ok(_) => {
                                    paths
                                        .collect::<Vec<_>>()
                                        .iter()
                                        .for_each(|path| match ResearchActivity::read(path.as_ref().unwrap().into()) {
                                            | Ok(data) => {
                                                // TODO: Handle vale runs that exit with error code. Maybe use --no-exit?
                                                vale.clone().analyze(data.clone().meta.id, data.extract_prose());
                                            }
                                            | Err(err) => {
                                                error!("=> {} Read research activity data at - {}", Label::fail(), err);
                                            }
                                        })
                                }
                                | Err(err) => {
                                    error!("=> {} Vale sync - {}", Label::fail(), err);
                                }
                            }
                        }
                        | Err(err) => {
                            error!("=> {} Failed to read glob pattern - {}", Label::fail(), err);
                        }
                    };
                } else {
                    check_file(path.clone());
                    analyze_file(&path.clone());
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
                        playwright.prepare().unwrap(); // Install browsers
                        let chromium = playwright.chromium();
                        let browser = chromium.launcher().headless(true).launch().await.unwrap();
                        let context = browser.context_builder().build().await.unwrap();
                        let page = context.new_page().await.unwrap();
                        let input_path = match path.clone() {
                            | Some(value) => value,
                            | None => unimplemented!(),
                        };
                        if input_path.is_dir() {
                            match glob(&format!("{}/**/*.json", input_path.to_str().unwrap())) {
                                | Ok(values) => {
                                    let paths = values.flatten().collect::<Vec<_>>();
                                    let progress = ProgressBar::new(paths.len() as u64);
                                    progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
                                    for path in paths {
                                        progress.set_message(format!("Generating PDF for {}", path.display()));
                                        generate_factsheet_pdf(path, &page, output).await;
                                        progress.inc(1);
                                    }
                                    progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                                }
                                | Err(err) => {
                                    error!("=> {} Failed to read glob pattern - {}", Label::fail(), err);
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
                | Target::Poster => Ok(()),
                | Target::Highlight => Ok(()),
            }
        }
        | Some(Commands::Format { .. }) => Ok(()),
        | None => Ok(()),
    }
}
