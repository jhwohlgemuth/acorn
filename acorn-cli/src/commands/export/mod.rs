use crate::cli::arguments::FileFormat;
use crate::cli::{paths_from_options, CommandLineOptions};
use crate::template::Convert;
use acorn::io::{async_runtime, extract_zip, read_file, write_file};
use acorn::powerpoint::ooxml::{Relationship, Relationships};
use acorn::powerpoint::{archive, interpolate_values, read_xml_rel};
use acorn::prelude::{absolute, copy, create_dir_all, remove_file, File, PathBuf};
use acorn::schema::research_activity::{ResearchActivity, ResearchActivityMetadata};
use acorn::util::{suffix, Label, ToAbsoluteString};
use color_eyre::eyre::{Report, Result};
use indicatif::{ProgressBar, ProgressStyle};
use playwright::api::Page;
use playwright::Playwright;
use std::process::exit;
use tracing::{debug, error, info};

fn copy_image(index: usize, data: &ResearchActivity, source: &PathBuf, destination: &PathBuf) {
    let xml_rels_path = destination.join(format!("ppt/slides/_rels/slide{index}.xml.rels"));
    let image_path = match read_xml_rel(xml_rels_path.clone()) {
        | Some(Relationships { relationship }) => match relationship.clone().into_iter().find(|x| x.target.ends_with(".png")) {
            | Some(Relationship { target, .. }) => match destination.join("ppt/slides").join(target.clone()).canonicalize() {
                | Ok(value) => value,
                | Err(err) => {
                    error!(target = target.clone(), "=> {} Find image - {err}", Label::fail());
                    exit(exitcode::NOINPUT);
                }
            },
            | None => {
                error!(path = xml_rels_path.to_absolute_string(), "=> {} Find image relationship", Label::fail());
                exit(exitcode::DATAERR);
            }
        },
        | None => {
            error!(
                path = xml_rels_path.to_absolute_string(),
                "=> {} Read PowerPoint slide relationships",
                Label::fail()
            );
            exit(exitcode::NOINPUT);
        }
    };
    let from = source.join(data.clone().meta.first_image_content_url());
    copy(from, image_path).unwrap();
}
fn output_path_string(output: &Option<PathBuf>, data: &ResearchActivity) -> String {
    match output {
        | Some(ref value) => match create_dir_all(value.clone()) {
            | Ok(_) => {
                let ResearchActivityMetadata { identifier, .. } = data.meta.clone();
                let path = format!("{}/{}.pptx", value.display(), identifier);
                debug!(path, "=> {} Output", Label::using());
                path
            }
            | Err(err) => {
                let path = value.clone().to_absolute_string();
                error!(path, "=> {} Create directory - {err}", Label::fail());
                exit(exitcode::IOERR);
            }
        },
        | None => unreachable!(),
    }
}
pub async fn create_pdf(page: &Page, options: CommandLineOptions) {
    let CommandLineOptions { output, .. } = options;
    let path = match options.path {
        | Some(value) => value,
        | None => unimplemented!(),
    };
    let data = match ResearchActivity::read(path.clone()) {
        | Some(value) => value.format(Some(path.clone())),
        | None => {
            error!(path = path.to_absolute_string(), "=> {} Read data for PDF export", Label::fail());
            exit(exitcode::UNAVAILABLE);
        }
    };
    let content = data.to_html().to_string();
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
fn create_powerpoint(paths: Vec<PathBuf>, options: Option<CommandLineOptions>) {
    let CommandLineOptions { output, reference, .. } = options.clone().unwrap();
    for input_path in paths.clone() {
        let parent = input_path.parent().unwrap().canonicalize().unwrap();
        let reference_path = match reference {
            | Some(ref value) => {
                if value.as_path().is_absolute() {
                    value
                } else {
                    &parent.join(value)
                }
            }
            | None => &parent.join("reference.pptx"),
        };
        let data = match ResearchActivity::read(input_path.clone()) {
            | Some(value) => value.format(Some(input_path.clone())),
            | None => {
                error!(
                    path = input_path.to_absolute_string(),
                    "=> {} Read data for PowerPoint export",
                    Label::fail()
                );
                exit(exitcode::UNAVAILABLE);
            }
        };
        if let Ok(export_path) = extract_zip(reference_path.clone(), None) {
            for (index, _path) in paths.iter().enumerate() {
                let slide_number = index + 1;
                copy_image(slide_number, &data, &parent, reference_path);
                [
                    format!("ppt/slides/slide{slide_number}.xml"),
                    format!("ppt/notesSlides/notesSlide{slide_number}.xml"),
                ]
                .iter()
                .for_each(|fragment| {
                    let fragment_path = export_path.clone().join(fragment);
                    if !fragment_path.exists() {
                        let first_slide = export_path.clone().join("ppt/slides/slide1.xml");
                        let content = read_file(first_slide).unwrap();
                        let _ = write_file(fragment_path.clone(), content);
                    }
                    interpolate_values(fragment_path, data.clone());
                });
            }
            let _ = archive(export_path, Some(PathBuf::from(output_path_string(&output, &data))));
        }
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
    match format {
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
                    create_pdf(&page, pdf_options).await;
                    progress.inc(1);
                }
                progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                info!("=> {} Exported {} fact sheet PDF{}", Label::pass(), paths.len(), suffix(paths.len()));
                let _close = page.close(Some(false)).await;
            });
            Ok(())
        }
        | FileFormat::Powerpoint => {
            let paths = paths_from_options(path, &Some(options.clone()));
            if *combine {
                let progress = ProgressBar::new(1);
                progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
                let options = CommandLineOptions::init()
                    .combine(*combine)
                    .maybe_path(path.clone())
                    .maybe_output(output.clone())
                    .maybe_reference(reference.clone())
                    .build();
                progress.set_message("Generating combined PowerPoint artifact");
                create_powerpoint(paths, Some(options));
                progress.inc(1);
                progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                info!("=> {} Exported combined highlight PowerPoint file", Label::pass());
            } else {
                let paths = paths_from_options(path, &Some(options));
                let count = if *combine { 1 } else { paths.len() as u64 };
                let progress = ProgressBar::new(count);
                progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
                for path in &paths {
                    let options = CommandLineOptions::init()
                        .combine(*combine)
                        .path(path.clone())
                        .maybe_output(output.clone())
                        .maybe_reference(reference.clone())
                        .build();
                    progress.set_message(format!("Generating PowerPoint for {}", path.display()));
                    create_powerpoint(paths.clone(), Some(options));
                    progress.inc(1);
                }
                progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                info!(
                    "=> {} Exported {} highlight PowerPoint file{}",
                    Label::pass(),
                    paths.len(),
                    suffix(paths.len())
                );
            }
            Ok(())
        }
    }
}
