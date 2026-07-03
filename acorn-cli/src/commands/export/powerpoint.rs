//! PowerPoint export helpers
//!
//! This module generates PPTX output by copying slide templates, updating
//! OOXML relationships, and interpolating Research Activity data.
use crate::cli::CommandOptions;
use acorn::io::powerpoint::ooxml::{
    add_slide_to_presentation_xml, update_relationship_target, validate_xml_well_formed, Relationship, Relationships,
};
use acorn::io::powerpoint::{interpolate_values, read_xml_rel};
use acorn::io::{archive, extract_zip, read_file, write_file, ApiResult, InputOutput};
use acorn::prelude::{copy, create_dir_all, Arc, Path, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{Label, StringConversion};
use color_eyre::eyre::eyre;
use itertools::izip;
use nanoid::nanoid;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use tracing::error;

fn add_slide(slide_number: usize, reference_extract_path: &Path) {
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
    let new_slide_paths = slide_paths(slide_number, reference_extract_path);
    if !new_slide_paths.iter().all(|x| x.exists()) {
        izip!(slide_paths(1, reference_extract_path), new_slide_paths).for_each(|(first_slide_path, new_slide_path)| {
            if let Ok(result) = read_file(first_slide_path) {
                let content = result
                    .replace("slides/slide1", &format!("slides/slide{}", slide_number))
                    .replace("notesSlides/notesSlide1", &format!("notesSlides/notesSlide{}", slide_number));
                if validate_xml_well_formed(&content) {
                    if let Err(why) = write_file(new_slide_path.clone(), content) {
                        error!(
                            path = new_slide_path.to_absolute_string(),
                            "=> {} Write slide file — {why}",
                            Label::fail()
                        );
                    }
                } else {
                    error!(
                        path = new_slide_path.to_absolute_string(),
                        "=> {} Validate copied slide XML",
                        Label::fail()
                    );
                }
            }
        });
        // Add new slide relationship to presentation.xml.rels
        let presentation_xml_rels_path = reference_extract_path.join("ppt/_rels/presentation.xml.rels");
        let presentation_xml_rels = match read_xml_rel(presentation_xml_rels_path.clone()) {
            | Some(value) => value,
            | None => Relationships::default(),
        };
        let largest_identifier = presentation_xml_rels.largest_revision_identifier().unwrap_or(0);
        let new_slide_revision_identifier = largest_identifier.saturating_add(1);
        let new_slide_relationship = Relationship::init()
            .id(format!("rId{}", new_slide_revision_identifier))
            .target(format!("slides/slide{}.xml", slide_number))
            .build();
        let updated_presentation_xml_rels = presentation_xml_rels.add_relationship(new_slide_relationship);
        if let Err(why) = write_file(presentation_xml_rels_path.clone(), format!("{updated_presentation_xml_rels}")) {
            error!(
                path = presentation_xml_rels_path.to_absolute_string(),
                "=> {} Write presentation relationships — {why}",
                Label::fail()
            );
        }
        // Add new slide element to presentation.xml
        let presentation_xml_path = reference_extract_path.join("ppt/presentation.xml");
        match read_file(&presentation_xml_path) {
            | Ok(presentation_xml) => {
                let new_slide_element_identifier = nanoid!(4, &['1', '2', '3', '4', '5', '6', '7', '8', '9'])
                    .parse::<u32>()
                    .unwrap_or(new_slide_revision_identifier.saturating_add(255));
                match add_slide_to_presentation_xml(&presentation_xml, new_slide_element_identifier, new_slide_revision_identifier) {
                    | Ok(updated_presentation_xml) => {
                        if let Err(why) = write_file(&presentation_xml_path, updated_presentation_xml) {
                            error!(
                                path = presentation_xml_path.to_absolute_string(),
                                "=> {} Write presentation XML — {why}",
                                Label::fail()
                            );
                        }
                    }
                    | Err(why) => {
                        error!(
                            path = presentation_xml_path.to_absolute_string(),
                            "=> {} Update presentation XML — {why}",
                            Label::fail()
                        );
                    }
                }
            }
            | Err(why) => {
                error!(
                    path = presentation_xml_path.to_absolute_string(),
                    "=> {} Read presentation XML — {why}",
                    Label::fail()
                );
            }
        }
    }
}
fn copy_image(index: usize, data: &ResearchActivity, source: &Path, destination: &Path) -> ApiResult<u64> {
    let xml_rels_path = destination.join(format!("ppt/slides/_rels/slide{index}.xml.rels"));
    read_xml_rel(xml_rels_path.clone())
        .map(|Relationships { relationship, .. }| relationship)
        .ok_or_else(|| {
            let path = xml_rels_path.to_absolute_string();
            error!(path, "=> {} Read PowerPoint slide relationships", Label::fail());
            eyre!("Failed to read PowerPoint slide relationships at {path}")
        })
        .and_then(|relationships| {
            relationships
                .into_iter()
                .find(|x| x.target.ends_with("png"))
                .map(|x| x.target)
                .ok_or_else(|| {
                    let path = xml_rels_path.to_absolute_string();
                    error!(path, "=> {} Find slide image relationship", Label::fail());
                    eyre!("No PNG image relationship found in PowerPoint slide at {path}")
                })
        })
        .and_then(|target| {
            let ResearchActivity { meta, .. } = data;
            let parent = source
                .parent()
                .map(Path::to_path_buf)
                .and_then(|value| value.canonicalize().ok())
                .unwrap_or_else(|| PathBuf::from("."));
            let image_url = meta.clone().first_image_content_url();
            let from = parent.join(image_url.clone());
            let name = meta.clone().identifier;
            let extension = Path::new(&image_url).extension().and_then(|e| e.to_str()).unwrap_or("png");
            let to = destination.join(format!("ppt/media/{name}.{extension}"));
            read_file(xml_rels_path.clone())
                .map_err(|why| eyre!("Failed to read slide relationship XML — {why}"))
                .and_then(|content| update_relationship_target(&content, &target, &format!("../media/{name}.{extension}")))
                .and_then(|updated| {
                    write_file(xml_rels_path.clone(), updated).map_err(|why| {
                        error!(
                            path = xml_rels_path.to_absolute_string(),
                            "=> {} Write slide relationship XML — {why}",
                            Label::fail()
                        );
                        eyre!("Failed to write slide relationship XML — {why}")
                    })
                })
                .and_then(|_| copy(from, to).map_err(|e| eyre!("Failed to copy image — {e}")))
        })
}
fn validate_ooxml_files(reference_extract_path: &Path, count: usize) -> ApiResult<()> {
    let presentation_targets = ["ppt/presentation.xml", "ppt/_rels/presentation.xml.rels"]
        .iter()
        .map(|fragment| reference_extract_path.join(fragment));
    let slide_targets = (1..=count).flat_map(|slide_number| {
        [
            format!("ppt/slides/slide{slide_number}.xml"),
            format!("ppt/notesSlides/notesSlide{slide_number}.xml"),
            format!("ppt/slides/_rels/slide{slide_number}.xml.rels"),
            format!("ppt/notesSlides/_rels/notesSlide{slide_number}.xml.rels"),
        ]
        .into_iter()
        .map(|fragment| reference_extract_path.join(fragment))
    });
    let invalid = presentation_targets
        .chain(slide_targets)
        .find(|path| read_file(path).map(|content| !validate_xml_well_formed(&content)).unwrap_or(true));
    match invalid {
        | Some(path) => Err(eyre!(
            "Malformed OOXML content detected before archive write at {}",
            path.to_absolute_string()
        )),
        | None => Ok(()),
    }
}
/// Create PowerPoint presentation from Research Activity data
pub fn create(paths: impl IntoIterator<Item = PathBuf>, options: Option<Arc<CommandOptions>>) -> ApiResult<PathBuf> {
    let options = options.unwrap_or_default();
    let CommandOptions { output, path, reference, .. } = options.as_ref();
    let _paths: Arc<Vec<PathBuf>> = Arc::new(paths.into_iter().collect());
    let research_activity_data: Vec<ResearchActivity> = _paths
        .iter()
        .filter_map(|path| match ResearchActivity::read(path.clone()) {
            | Ok(value) => Some(value.format_with(Some(path.clone()))),
            | Err(why) => {
                error!(
                    path = path.to_absolute_string(),
                    "=> {} Read data for PowerPoint export - {why}",
                    Label::fail(),
                );
                None
            }
        })
        .collect();
    path.clone()
        .ok_or_else(|| eyre!("Missing path for PowerPoint export — a path or reference must be provided"))
        .map(|path_from_options| {
            let parent = if path_from_options.is_dir() {
                path_from_options
            } else {
                path_from_options
                    .parent()
                    .map(|p| p.canonicalize().unwrap_or_else(|_| PathBuf::from(".")))
                    .unwrap_or_else(|| PathBuf::from("."))
            };
            match reference {
                | Some(ref value) => {
                    if value.as_path().is_absolute() {
                        value.to_path_buf()
                    } else {
                        parent.join(value).to_path_buf()
                    }
                }
                | None => parent.join("reference.pptx"),
            }
        })
        .and_then(|reference_path| {
            extract_zip(reference_path.clone(), None).map(Arc::new).map_err(|_| {
                error!(path = reference_path.to_absolute_string(), "=> {} Extract reference", Label::fail());
                eyre!("Failed to extract PowerPoint reference at {reference_path:?}")
            })
        })
        .and_then(move |reference_extract_path| {
            research_activity_data
                .iter()
                .enumerate()
                .for_each(|(index, _)| add_slide(index.saturating_add(1), &reference_extract_path));
            let reference_extract_path_for_workers = Arc::clone(&reference_extract_path);
            izip!(research_activity_data, _paths.iter().cloned())
                .collect::<Vec<_>>()
                .into_par_iter()
                .enumerate()
                .for_each(move |(index, (data, index_path))| {
                    let reference_extract_path = Arc::clone(&reference_extract_path_for_workers);
                    let slide_number = index.saturating_add(1);
                    let fragments = [
                        format!("ppt/slides/slide{slide_number}.xml"),
                        format!("ppt/notesSlides/notesSlide{slide_number}.xml"),
                        format!("ppt/slides/_rels/slide{slide_number}.xml.rels"),
                    ];
                    fragments
                        .iter()
                        .map(|fragment| reference_extract_path.join(fragment))
                        .for_each(|path| interpolate_values(path, &data));
                    if let Err(why) = copy_image(slide_number, &data, &index_path, &reference_extract_path) {
                        error!(
                            path = index_path.to_absolute_string(),
                            "=> {} Copy image for slide {slide_number} — {why}",
                            Label::fail()
                        );
                    }
                });
            let count = _paths.len();
            let folder = if count == 1 {
                match _paths.first() {
                    | Some(p) => match p.parent() {
                        | Some(parent_dir) => parent_dir.canonicalize().unwrap_or_else(|_| PathBuf::from(".")),
                        | None => PathBuf::from("."),
                    },
                    | None => PathBuf::from("."),
                }
            } else {
                path.clone().unwrap_or_else(|| PathBuf::from("."))
            };
            let folder_name = folder.file_name_with_parent();
            validate_ooxml_files(&reference_extract_path, count)
                .and_then(|_| prepare_destination(output, &folder_name))
                .and_then(|destination| archive(reference_extract_path.as_ref().clone(), Some(destination)))
        })
}
fn prepare_destination(output: &Option<PathBuf>, name: &str) -> ApiResult<PathBuf> {
    const EXTENSION: &str = "pptx";
    match output {
        | Some(ref value) => match create_dir_all(value.clone()) {
            | Ok(_) => Ok(value.join(name).with_extension(EXTENSION)),
            | Err(why) => Err(eyre!("Failed to create output directory for PowerPoint export — {why}")),
        },
        | None => Err(eyre!("Missing output directory")),
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )]
    use super::prepare_destination;

    #[test]
    fn test_prepare_destination() {
        let base = std::env::temp_dir();
        let output = Some(base.clone());
        let path = prepare_destination(&output, "example").unwrap();
        let expected = base.join("example.pptx");
        assert_eq!(path, expected);
    }
}
