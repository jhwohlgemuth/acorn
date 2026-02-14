//! PowerPoint export helpers
//!
//! This module generates PPTX output by copying slide templates, updating
//! OOXML relationships, and interpolating Research Activity data.
use crate::cli::CommandLineOptions;
use acorn::io::{archive, extract_zip, read_file, write_file, InputOutput};
use acorn::powerpoint::ooxml::{Relationship, Relationships};
use acorn::powerpoint::{interpolate_values, read_xml_rel};
use acorn::prelude::{copy, create_dir_all, Error, Path, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{Label, ToAbsoluteString};
use color_eyre::eyre::Result;
use fancy_regex::Regex;
use itertools::izip;
use nanoid::nanoid;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use std::process::exit;
use tracing::{debug, error};

fn add_slide(slide_number: usize, reference_extract_path: PathBuf) {
    fn add_slide_element(xml: &str, revision_identifier: u32) -> String {
        let pattern = r#"<p:sldIdLst>(?<first_slide><p:sldId id="(?<identifier>\d+)" r:id="rId(?<revision>\d+)"/>)(?<new_slides>.*)</p:sldIdLst>"#;
        let re = Regex::new(pattern).unwrap();
        let new_slide_element_identifier: u32 = nanoid!(4, &['1', '2', '3', '4', '5', '6', '7', '8', '9']).parse().unwrap();
        let new_slide_element = format!(r#"<p:sldId id="{}" r:id="rId{revision_identifier}" />"#, new_slide_element_identifier);
        re.replace(xml, format!(r#"<p:sldIdLst>$1$4{new_slide_element}</p:sldIdLst>"#))
            .to_string()
    }
    fn slide_paths(slide_number: usize, root: &Path) -> Vec<PathBuf> {
        [
            format!("ppt/slides/slide{slide_number}.xml"),
            format!("ppt/slides/_rels/slide{slide_number}.xml.rels"),
            format!("ppt/notesSlides/notesSlide{slide_number}.xml"),
            format!("ppt/notesSlides/_rels/notesSlide{slide_number}.xml.rels"),
        ]
        .iter()
        .map(|x| root.join(x))
        .collect::<Vec<PathBuf>>()
    }
    let root = reference_extract_path.clone();
    let new_slide_paths = slide_paths(slide_number, &root);
    if !new_slide_paths.iter().all(|x| x.exists()) {
        izip!(slide_paths(1, &root), new_slide_paths).for_each(|(first_slide_path, new_slide_path)| {
            if let Ok(result) = read_file(first_slide_path) {
                let content = result
                    .replace("slides/slide1", &format!("slides/slide{}", slide_number))
                    .replace("notesSlides/notesSlide1", &format!("notesSlides/notesSlide{}", slide_number));
                let _ = write_file(new_slide_path, content);
            }
        });
        // Add new slide relationship to presentation.xml.rels
        let presentation_xml_rels_path = reference_extract_path.clone().join("ppt/_rels/presentation.xml.rels");
        let presentation_xml_rels = match read_xml_rel(presentation_xml_rels_path.clone()) {
            | Some(value) => value,
            | None => Relationships::default(),
        };
        let largest_identifier = presentation_xml_rels.largest_revision_identifier().unwrap();
        let new_slide_revision_identifier = largest_identifier + 1;
        let new_slide_relationship = Relationship::init()
            .id(format!("rId{}", new_slide_revision_identifier))
            .target(format!("slides/slide{}.xml", slide_number))
            .build();
        let updated_presentation_xml_rels = presentation_xml_rels.add_relationship(new_slide_relationship);
        let _ = write_file(presentation_xml_rels_path, format!("{updated_presentation_xml_rels}"));
        // Add new slide element to presentation.xml
        let presentation_xml_path = reference_extract_path.clone().join("ppt/presentation.xml");
        let presentation_xml = read_file(&presentation_xml_path).unwrap();
        let updated_presentation_xml = add_slide_element(&presentation_xml, new_slide_revision_identifier);
        let _ = write_file(&presentation_xml_path, updated_presentation_xml.to_string());
    }
}
fn copy_image(index: usize, data: &ResearchActivity, source: &Path, destination: &Path) -> std::io::Result<u64> {
    let ResearchActivity { meta, .. } = data;
    let xml_rels_path = destination.join(format!("ppt/slides/_rels/slide{index}.xml.rels"));
    let relationships = match read_xml_rel(xml_rels_path.clone()) {
        | Some(Relationships { relationship, .. }) => relationship,
        | None => {
            let path = xml_rels_path.to_absolute_string();
            error!(path, "=> {} Read PowerPoint slide relationships", Label::fail());
            exit(exitcode::NOINPUT);
        }
    };
    let target = match relationships.into_iter().find(|x| x.target.ends_with("png")) {
        | Some(Relationship { target, .. }) => target,
        | None => {
            let path = xml_rels_path.to_absolute_string();
            error!(path, "=> {} Find slide image relationship", Label::fail());
            exit(exitcode::NOINPUT);
        }
    };
    let parent = source.parent().unwrap().canonicalize().unwrap();
    let image_url = meta.clone().first_image_content_url();
    let from = parent.join(image_url.clone());
    let name = meta.clone().identifier;
    let extension = Path::new(&image_url).extension().unwrap().to_str().unwrap();
    let to = destination.join(format!("ppt/media/{name}.{extension}"));
    let re = Regex::new(&target).unwrap();
    let content = read_file(xml_rels_path.clone()).unwrap();
    let updated = re.replace(&content, format!("../media/{name}.{extension}")).to_string();
    let _ = write_file(xml_rels_path, updated);
    copy(from, to)
}
/// Create PowerPoint presentation from Research Activity data
pub fn create(paths: impl IntoIterator<Item = PathBuf>, options: Option<CommandLineOptions>) -> Result<PathBuf, Error> {
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
    let CommandLineOptions { output, path, reference, .. } = options.clone().unwrap();
    let paths: Vec<PathBuf> = paths.into_iter().collect();
    let count = paths.clone().len();
    let research_activity_data = paths
        .clone()
        .iter()
        .map(|x| match ResearchActivity::read(x.clone()) {
            | Ok(value) => value.format(Some(x.clone())),
            | Err(why) => {
                error!(
                    path = x.to_absolute_string(),
                    "=> {} Read data for PowerPoint export - {why}",
                    Label::fail(),
                );
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
    research_activity_data
        .iter()
        .enumerate()
        .for_each(|(index, _)| add_slide(index + 1, reference_extract_path.clone()));
    izip!(research_activity_data, paths.clone())
        .collect::<Vec<_>>()
        .into_par_iter()
        .enumerate()
        .for_each(|(index, (data, index_path))| {
            let slide_number = index + 1;
            let fragments = [
                format!("ppt/slides/slide{slide_number}.xml"),
                format!("ppt/notesSlides/notesSlide{slide_number}.xml"),
                format!("ppt/slides/_rels/slide{slide_number}.xml.rels"),
            ];
            fragments
                .iter()
                .map(|fragment| reference_extract_path.join(fragment))
                .for_each(|path| interpolate_values(path, data.clone()));
            let _ = copy_image(slide_number, &data, &index_path, &reference_extract_path);
        });
    let folder = if count == 1 {
        paths[0].clone().parent().unwrap().canonicalize().unwrap()
    } else {
        path.as_ref().unwrap().to_path_buf()
    };
    let name = folder.file_name().unwrap().to_string_lossy().to_string();
    archive(reference_extract_path.clone(), Some(PathBuf::from(output_path_string(&output, &name))))
}
