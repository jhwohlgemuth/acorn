use crate::template::create_pdf;
use acorn_lib::util::cli::{Options, Size, Target};
use acorn_lib::util::{get_all_files, tokio_runtime, Label};
use color_eyre::eyre::{Report, Result};
use indicatif::{ProgressBar, ProgressStyle};
use playwright::Playwright;
use std::path::PathBuf;

#[allow(clippy::too_many_arguments)]
pub fn run(output: &Option<PathBuf>, path: &Option<PathBuf>, size: &Size, target: &Target) -> Result<(), Report> {
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
                let paths = get_all_files(input_path.clone(), None);
                let progress = ProgressBar::new(paths.len() as u64);
                progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
                for path in paths {
                    progress.set_message(format!("Generating PDF for {}", path.display()));
                    let options = Options::init()
                        .path(path.clone())
                        .maybe_output(output.clone())
                        .size(size.clone())
                        .target(*target)
                        .build();
                    create_pdf(&page, options.clone()).await;
                    progress.inc(1);
                }
                progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                let _close = page.close(Some(false)).await;
            });
            Ok(())
        }
        | Target::Poster => Ok(()),
        | Target::Highlight => {
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
                let paths = get_all_files(input_path.clone(), None);
                let progress = ProgressBar::new(paths.len() as u64);
                progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
                for path in paths {
                    progress.set_message(format!("Generating PDF for {}", path.display()));
                    let options = Options::init()
                        .path(path.clone())
                        .maybe_output(output.clone())
                        .size(size.clone())
                        .target(*target)
                        .build();
                    create_pdf(&page, options.clone()).await;
                    progress.inc(1);
                }
                progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                let _close = page.close(Some(false)).await;
            });
            Ok(())
        }
    }
}
