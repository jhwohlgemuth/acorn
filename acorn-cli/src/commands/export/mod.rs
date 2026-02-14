use crate::cli::arguments::FileFormat;
use crate::cli::{paths_from_options, CommandLineOptions};
use acorn::io::bagit::{Bag, BagInfo, Save};
use acorn::io::{async_runtime, folder_size, FromPath, InputOutput};
use acorn::prelude::{exit, remove_dir_all, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{current_date, format_bytes, suffix, Label, MimeType};
use acorn::{fail, skip};
use color_eyre::eyre::{Report, Result};
use core::iter::once;
use indicatif::{ProgressBar, ProgressStyle};
use playwright::Playwright;
use tracing::{debug, info};

mod pdf;
mod powerpoint;

#[allow(clippy::too_many_arguments)]
pub fn run(
    output: &Option<PathBuf>,
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    format: &FileFormat,
    reference: &Option<PathBuf>,
    combine: &bool,
    merge_request: &bool,
    offline: &bool,
) -> Result<(), Report> {
    if *offline {
        println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
        unimplemented!("Offline mode is not implemented yet");
    }
    let options = CommandLineOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .merge_request(*merge_request)
        .build();
    let extension = format.to_string();
    match format {
        | FileFormat::Bag => {
            let base = path.clone().unwrap().display().to_string();
            let date = current_date();
            let size = format_bytes(folder_size(base.clone()));
            let info = BagInfo::init().date(date).size(size).build();
            debug!("=> {} {:#?}", Label::using(), info);
            let bag = Bag::init().base_directory(base).info(info).build();
            let destination = output.clone().unwrap();
            let save: Result<(), Report> = match bag.save(destination.clone()) {
                | Ok(_) => Ok(()),
                | Err(why) => Err(why.into()),
            };
            save.and_then(|_| Bag::verify(destination.clone()).map_err(|why| why.into()))
                .and_then(|_| remove_dir_all(destination).map_err(|why| why.into()))
        }
        | FileFormat::Json => {
            let from_options = paths_from_options(path, &Some(options.clone()));
            let (skipped, paths): (Vec<_>, Vec<_>) = from_options
                .iter()
                .partition(|path| MimeType::from_path(path) == MimeType::from(format.clone()));
            skipped.iter().for_each(|path| {
                skip!("{} is already in {} format", path.display(), extension);
            });
            let count = paths.len() as u64;
            let progress = ProgressBar::new(count);
            progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
            for path in &paths {
                progress.set_message(format!("Generating {} for {}", extension.clone(), path.display()));
                let output = path.with_extension(extension.clone());
                match ResearchActivity::read(path) {
                    | Ok(data) => match data.write(output) {
                        | Ok(_) => {}
                        | Err(_) => {
                            exit(exitcode::UNAVAILABLE);
                        }
                    },
                    | Err(why) => {
                        fail!("Read research activity data - {}", why);
                        exit(exitcode::DATAERR);
                    }
                };
                progress.inc(1);
            }
            progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
            info!("=> {} Exported {count} {} file{}", Label::pass(), extension, suffix(count));
            Ok(())
        }
        | FileFormat::Pdf => {
            async_runtime().block_on(async {
                let paths = paths_from_options(path, &Some(options));
                let playwright = Playwright::initialize().await.unwrap();
                playwright.install_chromium().unwrap(); // Install browsers
                let chromium = playwright.chromium();
                let browser = chromium.launcher().headless(true).launch().await.unwrap();
                let context = browser.context_builder().build().await.unwrap();
                let page = context.new_page().await.unwrap();
                let progress = ProgressBar::new(paths.len() as u64);
                progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
                for path in &paths {
                    progress.set_message(format!("Generating PDF for {}", path.display()));
                    let pdf_options = CommandLineOptions::init().path(path.clone()).maybe_output(output.clone()).build();
                    pdf::create(&page, pdf_options).await;
                    progress.inc(1);
                }
                progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                info!("=> {} Exported {} PDF{}", Label::pass(), paths.len(), suffix(paths.len()));
                let _close = page.close(Some(false)).await;
            });
            Ok(())
        }
        | FileFormat::Markdown => {
            let from_options = paths_from_options(path, &Some(options.clone()));
            let (skipped, paths): (Vec<_>, Vec<_>) = from_options
                .iter()
                .partition(|path| MimeType::from_path(path) == MimeType::from(format.clone()));
            skipped.iter().for_each(|path| {
                skip!("{} is already in {} format", path.display(), extension);
            });
            let count = paths.len() as u64;
            let progress = ProgressBar::new(count);
            progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
            for path in &paths {
                progress.set_message(format!("Generating {} for {}", extension.clone(), path.display()));
                let output = path.with_extension(extension.clone());
                match ResearchActivity::read(path) {
                    | Ok(data) => match data.write(output) {
                        | Ok(_) => {}
                        | Err(_) => {
                            exit(exitcode::UNAVAILABLE);
                        }
                    },
                    | Err(why) => {
                        fail!("Read research activity data - {}", why);
                        exit(exitcode::DATAERR);
                    }
                };
                progress.inc(1);
            }
            progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
            info!("=> {} Exported {count} {} file{}", Label::pass(), extension, suffix(count));
            Ok(())
        }
        | FileFormat::Powerpoint => {
            let paths = paths_from_options(path, &Some(options.clone()));
            let options = CommandLineOptions::init()
                .maybe_path(path.clone())
                .maybe_output(output.clone())
                .maybe_reference(reference.clone())
                .build();
            let count = if *combine { 1 } else { paths.len() as u64 };
            let progress = ProgressBar::new(count);
            progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
            if *combine {
                progress.set_message("Generating combined PowerPoint artifact");
                let _ = powerpoint::create(paths, Some(options));
                progress.inc(1);
                progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                info!("=> {} Exported combined PowerPoint file", Label::pass());
            } else {
                for path in &paths {
                    progress.set_message(format!("Generating PowerPoint for {}", path.display()));
                    let _ = powerpoint::create(once(path.to_path_buf()), Some(options.clone()));
                    progress.inc(1);
                }
                progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                info!("=> {} Exported {count} PowerPoint file{}", Label::pass(), suffix(count));
            }
            Ok(())
        }
        | FileFormat::Yaml => {
            let from_options = paths_from_options(path, &Some(options.clone()));
            let (skipped, paths): (Vec<_>, Vec<_>) = from_options
                .iter()
                .partition(|path| MimeType::from_path(path) == MimeType::from(format.clone()));
            skipped.iter().for_each(|path| {
                skip!("{} is already in {} format", path.display(), extension);
            });
            let count = paths.len() as u64;
            let progress = ProgressBar::new(count);
            progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
            for path in &paths {
                progress.set_message(format!("Generating {} for {}", extension.clone(), path.display()));
                let output = path.with_extension(extension.clone());
                match ResearchActivity::read(path) {
                    | Ok(data) => match data.write(output) {
                        | Ok(_) => {}
                        | Err(_) => {
                            exit(exitcode::UNAVAILABLE);
                        }
                    },
                    | Err(why) => {
                        fail!("Read research activity data - {}", why);
                        exit(exitcode::DATAERR);
                    }
                };
                progress.inc(1);
            }
            progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
            info!("=> {} Exported {count} {} file{}", Label::pass(), extension, suffix(count));
            Ok(())
        }
    }
}
