use crate::cli::arguments::{FileFormat, Size, Target};
use crate::cli::{paths_from_options, CommandLineOptions};
use crate::template::Convert;
use acorn_lib::io::{async_runtime, extract_zip};
use acorn_lib::powerpoint::ooxml::{Relationship, Relationships};
use acorn_lib::powerpoint::{archive, interpolate_values, read_xml_rel};
use acorn_lib::prelude::{absolute, copy, create_dir_all, remove_file, File, PathBuf};
use acorn_lib::schema::{Metadata, ResearchActivity};
use acorn_lib::util::{suffix, to_absolute_string, Label};
use color_eyre::eyre::{Report, Result};
use indicatif::{ProgressBar, ProgressStyle};
use playwright::api::Page;
use playwright::Playwright;
use std::process::exit;
use tracing::{debug, error, info};

pub async fn create_pdf(page: &Page, options: CommandLineOptions) {
    let CommandLineOptions { output, size, target, .. } = options;
    let path = match options.path {
        | Some(value) => value,
        | None => unimplemented!(),
    };
    let data = match ResearchActivity::read(path.clone()) {
        | Some(value) => value.format(Some(path.clone())),
        | None => {
            error!(path = to_absolute_string(path), "=> {} Read data for PDF export", Label::fail());
            exit(exitcode::UNAVAILABLE);
        }
    };
    let content = data.to_html(target, size).to_string();
    // NOTE: Create temporary index.html because rust-playwright does not support setting baseURL
    let index = absolute(path.parent().unwrap().join("index.html")).unwrap();
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
fn create_powerpoint(options: CommandLineOptions) {
    let CommandLineOptions { path, output, .. } = options;
    let input_path = match path {
        | Some(value) => value,
        | None => unimplemented!(),
    };
    let data = match ResearchActivity::read(input_path.clone()) {
        | Some(value) => value.format(Some(input_path.clone())),
        | None => {
            error!(
                path = to_absolute_string(input_path),
                "=> {} Read data for PowerPoint export",
                Label::fail()
            );
            exit(exitcode::UNAVAILABLE);
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
                let Metadata { identifier, .. } = data.meta.clone();
                let path = format!("{}/{}.pptx", value.display(), identifier);
                debug!(path, "=> {} Output", Label::using());
                path
            }
            | Err(err) => {
                let path = to_absolute_string(value.clone());
                error!(path, "=> {} Create directory - {err}", Label::fail());
                exit(exitcode::IOERR);
            }
        },
        | None => unreachable!(),
    };
    if let Ok(path) = extract_zip(reference_path, None) {
        let xml_rels_path = path.join("ppt/slides/_rels/slide1.xml.rels");
        let image_path = match read_xml_rel(xml_rels_path.clone()) {
            | Some(Relationships { relationship }) => match relationship.clone().into_iter().find(|x| x.target.ends_with(".png")) {
                | Some(Relationship { target, .. }) => match path.join("ppt/slides").join(target.clone()).canonicalize() {
                    | Ok(value) => value,
                    | Err(err) => {
                        error!(target = target.clone(), "=> {} Find image - {err}", Label::fail());
                        exit(exitcode::NOINPUT);
                    }
                },
                | None => {
                    error!(path = to_absolute_string(xml_rels_path), "=> {} Find image relationship", Label::fail());
                    exit(exitcode::DATAERR);
                }
            },
            | None => {
                error!(
                    path = to_absolute_string(xml_rels_path),
                    "=> {} Read PowerPoint slide relationships",
                    Label::fail()
                );
                exit(exitcode::NOINPUT);
            }
        };
        let from = input_path.parent().unwrap().join(data.clone().meta.first_image_content_url());
        copy(from, image_path).unwrap();
        ["ppt/slides/slide1.xml", "ppt/notesSlides/notesSlide1.xml"]
            .into_iter()
            .for_each(|fragment| {
                interpolate_values(path.clone().join(fragment), data.clone());
            });
        let _ = archive(path, Some(PathBuf::from(output_path)));
    }
}
#[allow(clippy::too_many_arguments)]
pub fn run(
    output: &Option<PathBuf>,
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    format: &FileFormat,
    reference: &Option<PathBuf>,
    size: &Size,
    target: &Target,
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
    let paths = paths_from_options(path, &Some(options));
    match target {
        | Target::FactSheet => {
            async_runtime().block_on(async {
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
                    let pdf_options = CommandLineOptions::init()
                        .path(path.clone())
                        .maybe_output(output.clone())
                        .size(size.clone())
                        .target(*target)
                        .build();
                    create_pdf(&page, pdf_options).await;
                    progress.inc(1);
                }
                progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                info!("=> {} Exported {} fact sheet PDF{}", Label::pass(), paths.len(), suffix(paths.len()));
                let _close = page.close(Some(false)).await;
            });
            Ok(())
        }
        | Target::Poster => todo!("Create poster from research activity data"),
        | Target::Highlight => {
            match format {
                | FileFormat::Pdf => {
                    async_runtime().block_on(async {
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
                            let pdf_options = CommandLineOptions::init()
                                .path(path.clone())
                                .maybe_output(output.clone())
                                .size(size.clone())
                                .target(*target)
                                .build();
                            create_pdf(&page, pdf_options).await;
                            progress.inc(1);
                        }
                        progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                        info!("=> {} Exported {} highlight PDF{}", Label::pass(), paths.len(), suffix(paths.len()));
                        let _close = page.close(Some(false)).await;
                    });
                    Ok(())
                }
                | FileFormat::Powerpoint => {
                    let progress = ProgressBar::new(paths.len() as u64);
                    progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
                    for path in &paths {
                        progress.set_message(format!("Generating PowerPoint for {}", path.display()));
                        let powerpoint_options = CommandLineOptions::init()
                            .path(path.clone())
                            .maybe_output(output.clone())
                            .maybe_reference(reference.clone())
                            .size(size.clone())
                            .target(*target)
                            .build();
                        create_powerpoint(powerpoint_options);
                        progress.inc(1);
                    }
                    progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                    info!(
                        "=> {} Exported {} highlight PowerPoint file{}",
                        Label::pass(),
                        paths.len(),
                        suffix(paths.len())
                    );
                    Ok(())
                }
            }
        }
    }
}
