//! PowerPoint export helpers
//!
//! This module generates PPTX output by copying slide templates, updating
//! OOXML relationships, and interpolating Research Activity data.
use crate::cli::CommandOptions;
use acorn::io::powerpoint::ooxml::{
    add_slide_to_presentation_xml, ensure_png_content_type, replace_aspect_placeholders_with_picture, update_relationship_target,
    validate_xml_well_formed, Relationship, Relationships,
};
use acorn::io::powerpoint::{interpolate_values, read_xml_rel};
use acorn::io::{archive, extract_zip, read_file, write_file, ApiResult, InputOutput};
use acorn::prelude::{copy, create_dir_all, write, Arc, Path, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{Label, StringConversion};
use color_eyre::eyre::eyre;
use itertools::izip;
use nanoid::nanoid;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use tracing::error;

const IMAGE_RELATIONSHIP: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";

fn add_slide(slide_number: usize, reference_extract_path: &Path) -> ApiResult<()> {
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
    if new_slide_paths.iter().all(|x| x.exists()) {
        return Ok(());
    }
    let relationship_replacements = [
        None,
        Some(("../notesSlides/notesSlide1.xml", format!("../notesSlides/notesSlide{slide_number}.xml"))),
        None,
        Some(("../slides/slide1.xml", format!("../slides/slide{slide_number}.xml"))),
    ];
    izip!(slide_paths(1, reference_extract_path), new_slide_paths, relationship_replacements)
        .map(|(first_slide_path, new_slide_path, replacement)| {
            read_file(&first_slide_path)
                .map_err(|why| eyre!("Failed to read PowerPoint part at {} — {why}", first_slide_path.to_absolute_string()))
                .and_then(|content| match replacement {
                    | Some((current_target, target)) => update_relationship_target(&content, current_target, &target),
                    | None => Ok(content),
                })
                .and_then(|content| match validate_xml_well_formed(&content) {
                    | true => Ok(content),
                    | false => Err(eyre!(
                        "Malformed OOXML content while copying PowerPoint part to {}",
                        new_slide_path.to_absolute_string()
                    )),
                })
                .and_then(|content| {
                    write_file(&new_slide_path, content)
                        .map_err(|why| eyre!("Failed to write PowerPoint part at {} — {why}", new_slide_path.to_absolute_string()))
                })
        })
        .collect::<ApiResult<Vec<_>>>()
        .and_then(|_| {
            let presentation_xml_rels_path = reference_extract_path.join("ppt/_rels/presentation.xml.rels");
            read_xml_rel(presentation_xml_rels_path.clone())
                .ok_or_else(|| {
                    eyre!(
                        "Failed to read PowerPoint presentation relationships at {}",
                        presentation_xml_rels_path.to_absolute_string()
                    )
                })
                .and_then(|presentation_xml_rels| {
                    let new_slide_revision_identifier = presentation_xml_rels.largest_revision_identifier().unwrap_or(0).saturating_add(1);
                    let new_slide_relationship = Relationship::init()
                        .id(format!("rId{new_slide_revision_identifier}"))
                        .target(format!("slides/slide{slide_number}.xml"))
                        .build();
                    presentation_xml_rels
                        .add_relationship(new_slide_relationship)
                        .to_string()
                        .map_err(|why| eyre!("Failed to serialize PowerPoint presentation relationships — {why}"))
                        .and_then(|updated| {
                            write_file(&presentation_xml_rels_path, updated).map_err(|why| {
                                eyre!(
                                    "Failed to write PowerPoint presentation relationships at {} — {why}",
                                    presentation_xml_rels_path.to_absolute_string()
                                )
                            })
                        })
                        .map(|_| new_slide_revision_identifier)
                })
        })
        .and_then(|new_slide_revision_identifier| {
            let presentation_xml_path = reference_extract_path.join("ppt/presentation.xml");
            read_file(&presentation_xml_path)
                .map_err(|why| {
                    eyre!(
                        "Failed to read PowerPoint presentation XML at {} — {why}",
                        presentation_xml_path.to_absolute_string()
                    )
                })
                .and_then(|presentation_xml| {
                    let new_slide_element_identifier = nanoid!(4, &['1', '2', '3', '4', '5', '6', '7', '8', '9'])
                        .parse::<u32>()
                        .unwrap_or(new_slide_revision_identifier.saturating_add(255));
                    add_slide_to_presentation_xml(&presentation_xml, new_slide_element_identifier, new_slide_revision_identifier)
                })
                .and_then(|updated| {
                    write_file(&presentation_xml_path, updated).map_err(|why| {
                        eyre!(
                            "Failed to write PowerPoint presentation XML at {} — {why}",
                            presentation_xml_path.to_absolute_string()
                        )
                    })
                })
        })
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
fn add_aspect_chart(index: usize, data: &ResearchActivity, chart: &[u8], destination: &Path) -> ApiResult<()> {
    let slide_path = destination.join(format!("ppt/slides/slide{index}.xml"));
    let relationships_path = destination.join(format!("ppt/slides/_rels/slide{index}.xml.rels"));
    let content_types_path = destination.join("[Content_Types].xml");
    let media_name = format!("{}-aspect.png", data.meta.identifier);
    let media_path = destination.join("ppt/media").join(&media_name);
    read_file(&slide_path)
        .map_err(|why| eyre!("Failed to read PowerPoint slide at {} — {why}", slide_path.to_absolute_string()))
        .and_then(|slide_xml| {
            read_xml_rel(relationships_path.clone())
                .ok_or_else(|| {
                    eyre!(
                        "Failed to read PowerPoint slide relationships at {}",
                        relationships_path.to_absolute_string()
                    )
                })
                .map(|relationships| (slide_xml, relationships))
        })
        .and_then(|(slide_xml, relationships)| {
            let revision = relationships.largest_revision_identifier().unwrap_or(0).saturating_add(1);
            let relationship_id = format!("rId{revision}");
            replace_aspect_placeholders_with_picture(&slide_xml, &relationship_id).and_then(|updated| match updated {
                | Some(updated_slide) => {
                    let relationship = Relationship::init()
                        .id(relationship_id)
                        .relationship_type(IMAGE_RELATIONSHIP.to_string())
                        .target(format!("../media/{media_name}"))
                        .build();
                    relationships
                        .add_relationship(relationship)
                        .to_string()
                        .map_err(|why| eyre!("Failed to serialize ASPECT chart relationship — {why}"))
                        .and_then(|xml| {
                            write_file(&relationships_path, xml).map_err(|why| {
                                eyre!(
                                    "Failed to write ASPECT chart relationship at {} — {why}",
                                    relationships_path.to_absolute_string()
                                )
                            })
                        })
                        .and_then(|_| {
                            write_file(&slide_path, updated_slide)
                                .map_err(|why| eyre!("Failed to write ASPECT chart picture at {} — {why}", slide_path.to_absolute_string()))
                        })
                        .and_then(|_| {
                            read_file(&content_types_path).map_err(|why| {
                                eyre!(
                                    "Failed to read PowerPoint content types at {} — {why}",
                                    content_types_path.to_absolute_string()
                                )
                            })
                        })
                        .and_then(|xml| ensure_png_content_type(&xml))
                        .and_then(|xml| {
                            write_file(&content_types_path, xml).map_err(|why| {
                                eyre!(
                                    "Failed to write PowerPoint content types at {} — {why}",
                                    content_types_path.to_absolute_string()
                                )
                            })
                        })
                        .and_then(|_| {
                            write(&media_path, chart)
                                .map_err(|why| eyre!("Failed to write ASPECT chart image at {} — {why}", media_path.to_absolute_string()))
                        })
                }
                | None => Ok(()),
            })
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
pub fn create(
    paths: impl IntoIterator<Item = PathBuf>,
    options: Option<Arc<CommandOptions>>,
    aspect_charts: Arc<Vec<Option<Vec<u8>>>>,
) -> ApiResult<PathBuf> {
    let options = options.unwrap_or_default();
    let CommandOptions { output, path, reference, .. } = options.as_ref();
    let paths: Arc<Vec<PathBuf>> = Arc::new(paths.into_iter().collect());
    let research_activity_data: Vec<ResearchActivity> = paths
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
                .map(|(index, _)| add_slide(index.saturating_add(1), &reference_extract_path))
                .collect::<ApiResult<Vec<_>>>()
                .and_then(|_| {
                    let reference_extract_path_for_workers = Arc::clone(&reference_extract_path);
                    izip!(research_activity_data, paths.iter().cloned())
                        .collect::<Vec<_>>()
                        .into_par_iter()
                        .enumerate()
                        .map(move |(index, (data, index_path))| {
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
                            match aspect_charts.get(index) {
                                | Some(Some(chart)) => add_aspect_chart(slide_number, &data, chart, &reference_extract_path).inspect_err(|why| {
                                    error!(
                                        path = index_path.to_absolute_string(),
                                        "=> {} Add ASPECT chart to slide {slide_number} — {why}",
                                        Label::fail()
                                    );
                                }),
                                | _ => Ok(()),
                            }
                        })
                        .collect::<ApiResult<Vec<_>>>()
                        .and_then(|_| {
                            let count = paths.len();
                            let folder = if count == 1 {
                                match paths.first() {
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
                })
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
    use super::{add_aspect_chart, add_slide, prepare_destination};
    use crate::test::util::{temp_test_dir, TestCleanup};
    use acorn::io::{read_file, write_file};
    use acorn::prelude::{create_dir_all, Path};
    use acorn::schema::research_activity::ResearchActivity;
    use std::env::temp_dir;

    fn write_slide_fixture(root: &Path, fragment: &str, content: &str) {
        let path = root.join(fragment);
        create_dir_all(path.parent().unwrap()).unwrap();
        write_file(path, content.to_string()).unwrap();
    }
    fn setup_slide_fixture(root: &Path) {
        let relationships = "http://schemas.openxmlformats.org/package/2006/relationships";
        write_slide_fixture(
            root,
            "ppt/slides/slide1.xml",
            r#"<p:sld xmlns:p="presentation" xmlns:a="drawing" xmlns:r="relationships"><p:cSld><p:spTree><p:sp><p:nvSpPr><p:cNvPr id="42" name="ASPECT placeholder"/></p:nvSpPr><p:spPr><a:xfrm><a:off x="10" y="20"/><a:ext cx="30" cy="40"/></a:xfrm></p:spPr><p:txBody><a:p><a:r><a:t>{{ aspect }}</a:t></a:r></a:p></p:txBody></p:sp></p:spTree></p:cSld></p:sld>"#,
        );
        write_slide_fixture(
            root,
            "ppt/slides/_rels/slide1.xml.rels",
            &format!(
                r#"<Relationships xmlns="{relationships}"><Relationship Id="rId1" Type="notes" Target="../notesSlides/notesSlide1.xml"/></Relationships>"#
            ),
        );
        write_slide_fixture(root, "ppt/notesSlides/notesSlide1.xml", r#"<p:notes xmlns:p="presentation"/>"#);
        write_slide_fixture(
            root,
            "ppt/notesSlides/_rels/notesSlide1.xml.rels",
            &format!(
                r#"<Relationships xmlns="{relationships}"><Relationship Id="rId1" Type="slide" Target="../slides/slide1.xml"/></Relationships>"#
            ),
        );
        write_slide_fixture(
            root,
            "ppt/_rels/presentation.xml.rels",
            &format!(r#"<Relationships xmlns="{relationships}"><Relationship Id="rId1" Type="slide" Target="slides/slide1.xml"/></Relationships>"#),
        );
        write_slide_fixture(
            root,
            "ppt/presentation.xml",
            r#"<p:presentation xmlns:p="presentation" xmlns:r="relationships"><p:sldIdLst><p:sldId id="256" r:id="rId1"/></p:sldIdLst></p:presentation>"#,
        );
        write_slide_fixture(
            root,
            "[Content_Types].xml",
            r#"<Types xmlns="content-types"><Default Extension="xml" ContentType="application/xml"/></Types>"#,
        );
    }

    #[test]
    fn test_prepare_destination() {
        let base = temp_dir();
        let output = Some(base.clone());
        let path = prepare_destination(&output, "example").unwrap();
        let expected = base.join("example.pptx");
        assert_eq!(path, expected);
    }
    #[test]
    fn test_add_slide_copies_parts_and_updates_relationships() {
        let root = temp_test_dir("powerpoint-add-slide");
        let _cleanup = TestCleanup::new(root.clone());
        setup_slide_fixture(&root);
        add_slide(2, &root).unwrap();
        let slide_relationships = read_file(root.join("ppt/slides/_rels/slide2.xml.rels")).unwrap();
        let notes_relationships = read_file(root.join("ppt/notesSlides/_rels/notesSlide2.xml.rels")).unwrap();
        let presentation_relationships = read_file(root.join("ppt/_rels/presentation.xml.rels")).unwrap();
        let presentation = read_file(root.join("ppt/presentation.xml")).unwrap();
        assert!(slide_relationships.contains("../notesSlides/notesSlide2.xml"));
        assert!(notes_relationships.contains("../slides/slide2.xml"));
        assert!(presentation_relationships.contains("slides/slide2.xml"));
        assert!(presentation.contains("rId2"));
    }
    #[test]
    fn test_add_slide_rejects_missing_source_part() {
        let root = temp_test_dir("powerpoint-add-slide-missing-source");
        let _cleanup = TestCleanup::new(root.clone());
        let result = add_slide(2, &root);
        assert!(result.is_err());
    }
    #[test]
    fn test_add_aspect_chart_packages_image_and_picture() {
        let root = temp_test_dir("powerpoint-aspect-chart");
        let _cleanup = TestCleanup::new(root.clone());
        setup_slide_fixture(&root);
        create_dir_all(root.join("ppt/media")).unwrap();
        let data = ResearchActivity::default();
        add_aspect_chart(1, &data, b"chart-png", &root).unwrap();
        let relationships = read_file(root.join("ppt/slides/_rels/slide1.xml.rels")).unwrap();
        let slide = read_file(root.join("ppt/slides/slide1.xml")).unwrap();
        let content_types = read_file(root.join("[Content_Types].xml")).unwrap();
        let media = acorn::prelude::read(root.join(format!("ppt/media/{}-aspect.png", data.meta.identifier))).unwrap();
        assert!(relationships.contains("relationships/image"));
        assert!(relationships.contains("-aspect.png"));
        assert!(slide.contains("ASPECT rose chart"));
        assert!(content_types.contains(r#"<Default Extension="png" ContentType="image/png"/>"#));
        assert_eq!(media, b"chart-png");
    }
}
