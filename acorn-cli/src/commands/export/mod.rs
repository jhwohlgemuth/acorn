use crate::template::get_html;
use crate::util::{FileFormat, Options, Size, Target};
use acorn_lib::powerpoint::{archive, extract};
use acorn_lib::schema::{Metadata, ResearchActivity};
use acorn_lib::util::{get_all_files, get_files_from_git, path_to_string, tokio_runtime, Label};
use color_eyre::eyre::{Report, Result};
use indicatif::{ProgressBar, ProgressStyle};
use playwright::api::Page;
use playwright::Playwright;
use std::fs::{create_dir_all, remove_file, File};
use std::path::{self, PathBuf};
use tracing::{debug, error};

pub async fn create_pdf(page: &Page, options: Options) {
    let Options { output, size, target, .. } = options;
    let path = match options.path {
        | Some(value) => value,
        | None => unimplemented!(),
    };
    let data = match ResearchActivity::read(path.clone()) {
        | Ok(value) => value.format(Some(path.clone())),
        | Err(err) => {
            error!(path = path_to_string(path), "=> {} {err}", Label::fail());
            std::process::exit(exitcode::UNAVAILABLE);
        }
    };
    let content = get_html(&data, target, size);
    // NOTE: Create temporary index.html because rust-playwright does not support setting baseURL
    let index = path::absolute(path.parent().unwrap().join("index.html")).unwrap();
    if File::create(&index).is_ok() {
        page.goto_builder(&format!("file://{}", index.display())).goto().await.unwrap();
    };
    page.set_content_builder(&content).set_content().await.unwrap();
    match output {
        | Some(output_dir) => match create_dir_all(output_dir.clone()) {
            | Ok(_) => {
                let id = data.meta.identifier;
                let output_path = format!("{}/{}.pdf", output_dir.display(), id);
                debug!(path = output_path, "=> {} Output", Label::using());
                page.pdf_builder()
                    .prefer_css_page_size(true)
                    .path(output_path.into())
                    .print_background(true)
                    .pdf()
                    .await
                    .unwrap();
            }
            | Err(err) => {
                error!(
                    path = output_dir.clone().into_os_string().into_string().unwrap(),
                    "=> {} Create directory - {err}",
                    Label::fail()
                );
            }
        },
        | None => unreachable!(),
    };
    let _remove = remove_file(index);
}
fn create_powerpoint(options: Options) {
    let Options { path, output, .. } = options;
    let input_path = match path {
        | Some(value) => value,
        | None => unimplemented!(),
    };
    let data = match ResearchActivity::read(input_path.clone()) {
        | Ok(value) => value.format(Some(input_path.clone())),
        | Err(err) => {
            error!(path = path_to_string(input_path), "=> {} {err}", Label::fail());
            std::process::exit(exitcode::UNAVAILABLE);
        }
    };
    let reference_path = match options.reference {
        | Some(value) => value,
        | None => {
            let root = input_path.parent().unwrap().to_path_buf().parent().unwrap().to_path_buf();
            root.join("reference.pptx")
        }
    };
    let output_path = match output {
        | Some(value) => match create_dir_all(value.clone()) {
            | Ok(_) => {
                let Metadata { identifier, .. } = data.meta;
                let path = format!("{}/{}.pptx", value.display(), identifier);
                debug!(path, "=> {} Output", Label::using());
                path
            }
            | Err(err) => {
                let path = path_to_string(value.clone());
                error!(path, "=> {} Create directory - {err}", Label::fail());
                std::process::exit(exitcode::IOERR);
            }
        },
        | None => unreachable!(),
    };
    if let Ok(path) = extract(reference_path, None) {
        // TODO: Copy image to ppt/media folder
        // TODO: Get ID of image
        // TODO: Interpolate data values
        let _ = archive(path, Some(PathBuf::from(output_path)));
    }
}
pub fn run(
    output: &Option<PathBuf>,
    path: &Option<PathBuf>,
    commit: &Option<String>,
    format: &FileFormat,
    reference: &Option<PathBuf>,
    size: &Size,
    target: &Target,
    schema: &bool,
) -> Result<(), Report> {
    fn get_paths_from_options(path: &Option<PathBuf>, commit: &Option<String>) -> Vec<PathBuf> {
        let extension = Some("json");
        match commit {
            | Some(hash) => get_files_from_git(hash, extension),
            | None => {
                let value = match path {
                    | Some(x) => x.clone(),
                    | None => PathBuf::from("."),
                };
                get_all_files(value, extension, None)
            }
        }
    }
    if *schema {
        ResearchActivity::to_schema();
        std::process::exit(exitcode::OK);
    }
    match target {
        | Target::FactSheet => {
            tokio_runtime().block_on(async {
                let playwright = Playwright::initialize().await.unwrap();
                playwright.install_chromium().unwrap(); // Install browsers
                let chromium = playwright.chromium();
                let browser = chromium.launcher().headless(true).launch().await.unwrap();
                let context = browser.context_builder().build().await.unwrap();
                let page = context.new_page().await.unwrap();
                let paths = get_paths_from_options(path, commit);
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
        | Target::Poster => todo!("Create poster from research activity data"),
        | Target::Highlight => {
            match format {
                | FileFormat::Pdf => {
                    tokio_runtime().block_on(async {
                        let playwright = Playwright::initialize().await.unwrap();
                        playwright.install_chromium().unwrap(); // Install browsers
                        let chromium = playwright.chromium();
                        let browser = chromium.launcher().headless(true).launch().await.unwrap();
                        let context = browser.context_builder().build().await.unwrap();
                        let page = context.new_page().await.unwrap();
                        let paths = get_paths_from_options(path, commit);
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
                | FileFormat::Powerpoint => {
                    let paths = get_paths_from_options(path, &None);
                    let progress = ProgressBar::new(paths.len() as u64);
                    progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
                    for path in paths {
                        progress.set_message(format!("Generating PowerPoint for {}", path.display()));
                        let options = Options::init()
                            .path(path.clone())
                            .maybe_output(output.clone())
                            .maybe_reference(reference.clone())
                            .size(size.clone())
                            .target(*target)
                            .build();
                        create_powerpoint(options);
                        progress.inc(1);
                    }
                    progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                    Ok(())
                }
            }
        }
    }
}
