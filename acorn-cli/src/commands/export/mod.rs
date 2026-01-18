use crate::cli::arguments::FileFormat;
use crate::cli::{paths_from_options, CommandLineOptions};
use crate::template::Convert;
use acorn::io::{async_runtime, extract_zip, read_file, write_file};
use acorn::powerpoint::ooxml::{Relationship, Relationships};
use acorn::powerpoint::{archive, interpolate_values, read_xml_rel};
use acorn::prelude::{absolute, copy, create_dir_all, remove_file, File, Path, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{suffix, Label, ToAbsoluteString};
use color_eyre::eyre::{Report, Result};
use core::iter::once;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::izip;
use playwright::api::Page;
use playwright::Playwright;
use std::process::exit;
use tracing::{debug, error, info};

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
    let CommandLineOptions { output, path, reference, .. } = options.clone().unwrap();
    let count = paths.clone().len();
    let research_activity_data = paths
        .clone()
        .iter()
        .map(|x| match ResearchActivity::read(x.clone()) {
            | Some(value) => value.format(Some(x.clone())),
            | None => {
                error!(path = x.to_absolute_string(), "=> {} Read data for PowerPoint export", Label::fail());
                exit(exitcode::UNAVAILABLE);
            }
        })
        .collect::<Vec<ResearchActivity>>();
    let reference_path = match path.clone() {
        | Some(path_from_options) => {
            let parent = if path_from_options.is_dir() {
                path_from_options
            } else {
                path_from_options.parent().unwrap().canonicalize().unwrap()
            };
            match reference {
                | Some(ref value) => {
                    if value.as_path().is_absolute() {
                        value.to_path_buf()
                    } else {
                        parent.join(value).to_path_buf()
                    }
                }
                | None => parent.join("reference.pptx").to_path_buf(),
            }
        }
        | None => unimplemented!(),
    };
    let reference_extract_path = match extract_zip(reference_path.clone(), None) {
        | Ok(value) => value,
        | Err(_) => {
            error!(path = reference_path.to_absolute_string(), "=> {} Extract reference", Label::fail());
            exit(exitcode::UNAVAILABLE);
        }
    };
    izip!(research_activity_data, paths.clone())
        .enumerate()
        .for_each(|(index, (data, index_path))| {
            let slide_number = index + 1;
            add_slide(slide_number, reference_extract_path.clone());
            [
                format!("ppt/slides/slide{slide_number}.xml"),
                format!("ppt/notesSlides/notesSlide{slide_number}.xml"),
                format!("ppt/slides/_rels/slide{slide_number}.xml.rels"),
            ]
            .iter()
            .map(|fragment| reference_extract_path.join(fragment))
            .for_each(|path| interpolate_values(path, data.clone()));
            copy_image(slide_number, &data, &index_path, &reference_extract_path);
        });
    let folder = if count == 1 {
        paths[0].clone().parent().unwrap().canonicalize().unwrap()
    } else {
        path.as_ref().unwrap().to_path_buf()
    };
    let name = folder.file_name().unwrap().to_string_lossy().to_string();
    let _ = archive(reference_extract_path.clone(), Some(PathBuf::from(output_path_string(&output, &name))));
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
        | FileFormat::Bag => {
            todo!("Implement BagIt export")
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
                create_powerpoint(paths, Some(options));
                progress.inc(1);
                progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                info!("=> {} Exported combined highlight PowerPoint file", Label::pass());
            } else {
                for path in &paths {
                    progress.set_message(format!("Generating PowerPoint for {}", path.display()));
                    create_powerpoint(vec![path.to_path_buf()], Some(options.clone()));
                    progress.inc(1);
                }
                progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                info!("=> {} Exported {count} highlight PowerPoint file{}", Label::pass(), suffix(count));
            }
            Ok(())
        }
    }
}
fn add_slide(slide_number: usize, reference_extract_path: PathBuf) {
    fn slide_paths(slide_number: usize, root: PathBuf) -> Vec<PathBuf> {
        [
            format!("ppt/slides/slide{slide_number}.xml"),
            format!("ppt/slides/_rels/slide{slide_number}.xml.rels"),
            format!("ppt/notesSlides/notesSlide{slide_number}.xml"),
            format!("ppt/notesSlides/_rels/notesSlide{slide_number}.xml.rels"),
            format!("ppt/slideLayouts/slideLayout{slide_number}.xml"),
            format!("ppt/slideLayouts/_rels/slideLayout{slide_number}.xml.rels"),
        ]
        .iter()
        .map(|x| root.join(x))
        .collect::<Vec<PathBuf>>()
    }
    let first_slide_paths = slide_paths(1, reference_extract_path.clone());
    let new_slide_paths = slide_paths(slide_number, reference_extract_path.clone());
    if !new_slide_paths.iter().all(|x| x.exists()) {
        izip!(first_slide_paths, new_slide_paths).for_each(|(first_slide_path, new_slide_path)| {
            if let Ok(result) = read_file(first_slide_path) {
                let content = result
                    .replace("slides/slide1", &format!("slides/slide{}", slide_number))
                    .replace("notesSlides/notesSlide1", &format!("notesSlides/notesSlide{}", slide_number))
                    .replace("slideLayouts/slideLayout1", &format!("slideLayouts/slideLayout{}", slide_number));
                let _ = write_file(new_slide_path, content);
            }
        });
        let presentation_rels = match read_xml_rel(reference_extract_path.clone().join("ppt/_rels/presentation.xml.rels")) {
            | Some(Relationships { relationship }) => relationship,
            | None => vec![],
        };
        let revision_identifiers = presentation_rels
            .iter()
            .map(|x| x.id.clone().trim_start_matches("rId").to_string().parse::<u32>().unwrap())
            .collect::<Vec<u32>>();
        let largest_identifier = *revision_identifiers.iter().max().unwrap();
        let new_slide_revision_identifier = largest_identifier + 1;
        let new_slide_relationship = Relationship {
            id: format!("rId{}", new_slide_revision_identifier),
            target: format!("slides/slide{}.xml", slide_number),
            target_mode: None,
            r#relationship_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide".to_string(),
        };
        let new_slide_xml_rel = Relationships {
            relationship: presentation_rels.into_iter().chain(once(new_slide_relationship)).collect::<Vec<_>>(),
        };
        // TODO: Update ppt/_rels/presentation.xml.rels with new_slide_xml_rel.to_string()
        // TODO: Add slide to ppt/presentation.xml
        // TODO: add <p:sldId id="###" r:id="rId#" /> using new_slide_revision_identifier
    }
}
fn copy_image(index: usize, data: &ResearchActivity, source: &Path, destination: &Path) {
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
    let parent = source.parent().unwrap().canonicalize().unwrap();
    let from = parent.join(data.clone().meta.first_image_content_url());
    copy(from, image_path).unwrap();
}
fn output_path_string(output: &Option<PathBuf>, name: &str) -> String {
    match output {
        | Some(ref value) => match create_dir_all(value.clone()) {
            | Ok(_) => {
                let path = format!("{}/{name}.pptx", value.display());
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
