//! PowerPoint export helpers
//!
//! This module generates PPTX output by copying slide templates, updating
//! OOXML relationships, and interpolating Research Activity data.
use crate::{assets::draft_placeholder_bytes, cli::CommandOptions};
use acorn::{
    io::{
        archive,
        export::{ExportContext, ImageSource},
        powerpoint::{
            interpolate_values,
            ooxml::{
                add_slide_to_presentation_xml, ensure_png_content_type, replace_aspect_placeholders_with_picture, update_relationship_target,
                validate_xml_well_formed, Relationship,
            },
            read_xml_rel,
        },
        ApiResult, ArchiveCandidate, ArchiveExtraction, InputOutput,
    },
    util::StringConversion,
};
use acorn_core::{prelude::Arc, util::MimeType};
use acorn_host::{
    fs::{read_text, SafePath},
    terminal::Label,
};
use acorn_schema::research_activity::ResearchActivity;
use color_eyre::eyre::eyre;
use itertools::izip;
use nanoid::nanoid;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use std::{
    fs::{create_dir_all, read, write},
    path::{Path, PathBuf},
};
use tracing::error;

const IMAGE_RELATIONSHIP: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
fn add_aspect_chart(index: usize, context: &ExportContext, chart: &[u8], destination: &Path) -> ApiResult<()> {
    let slide_path = destination.join(format!("ppt/slides/slide{index}.xml"));
    let relationships_path = destination.join(format!("ppt/slides/_rels/slide{index}.xml.rels"));
    let content_types_path = destination.join("[Content_Types].xml");
    let media_name = format!("{}-aspect.png", context.identifier);
    SafePath::new(format!("ppt/media/{media_name}"))
        .map_err(Into::<color_eyre::Report>::into)
        .and_then(|path| path.materialize_under(destination).map_err(Into::into))
        .and_then(|media_path| {
            read_text(&slide_path)
                .map_err(|why| eyre!("Failed to read PowerPoint part at {} — {why}", slide_path.display()))
                .and_then(|slide_xml| {
                    read_xml_rel(relationships_path.clone()).and_then(|relationships| {
                        let revision = relationships.largest_revision_identifier().unwrap_or(0).saturating_add(1);
                        let relationship_id = format!("rId{revision}");
                        replace_aspect_placeholders_with_picture(&slide_xml, &relationship_id)
                            .map_err(|why| eyre!("Failed to transform PowerPoint part at {} — {why}", slide_path.display()))
                            .and_then(|updated| match updated {
                                | Some(updated_slide) => {
                                    let relationship = Relationship::init()
                                        .id(relationship_id)
                                        .relationship_type(IMAGE_RELATIONSHIP.to_string())
                                        .target(format!("../media/{media_name}"))
                                        .build();
                                    relationships
                                        .add_relationship(relationship)
                                        .to_string()
                                        .map_err(|why| eyre!("Failed to serialize PowerPoint part at {} — {why}", relationships_path.display()))
                                        .and_then(|updated_relationships| {
                                            read_text(&content_types_path)
                                                .map_err(|why| eyre!("Failed to read PowerPoint part at {} — {why}", content_types_path.display()))
                                                .and_then(|content_types| {
                                                    ensure_png_content_type(&content_types).map_err(|why| {
                                                        eyre!("Failed to transform PowerPoint part at {} — {why}", content_types_path.display())
                                                    })
                                                })
                                                .and_then(|updated_content_types| {
                                                    [
                                                        staged_xml(relationships_path, updated_relationships),
                                                        staged_xml(slide_path, updated_slide),
                                                        staged_xml(content_types_path, updated_content_types),
                                                    ]
                                                    .into_iter()
                                                    .collect::<ApiResult<Vec<_>>>()
                                                    .and_then(|mut parts| {
                                                        parts.push((media_path, chart.to_vec()));
                                                        write_staged(parts)
                                                    })
                                                })
                                        })
                                }
                                | None => Ok(()),
                            })
                    })
                })
        })
}
pub(crate) fn add_slide(slide_number: usize, root: &Path) -> ApiResult<()> {
    fn slide_paths(slide_number: usize, root: &Path) -> Vec<PathBuf> {
        [
            format!("ppt/slides/slide{slide_number}.xml"),
            format!("ppt/slides/_rels/slide{slide_number}.xml.rels"),
            format!("ppt/notesSlides/notesSlide{slide_number}.xml"),
            format!("ppt/notesSlides/_rels/notesSlide{slide_number}.xml.rels"),
        ]
        .iter()
        .map(|fragment| root.join(fragment))
        .collect()
    }
    let destination_paths = slide_paths(slide_number, root);
    match destination_paths.iter().filter(|path| path.exists()).count() {
        | count if count == destination_paths.len() => Ok(()),
        | count if count > 0 => Err(eyre!("PowerPoint slide {slide_number} has incomplete existing parts")),
        | _ => {
            let replacements = [
                None,
                Some(("../notesSlides/notesSlide1.xml", format!("../notesSlides/notesSlide{slide_number}.xml"))),
                None,
                Some(("../slides/slide1.xml", format!("../slides/slide{slide_number}.xml"))),
            ];
            izip!(slide_paths(1, root), destination_paths, replacements)
                .map(|(source, destination, replacement)| {
                    read_text(&source)
                        .map_err(|why| eyre!("Failed to read PowerPoint part at {} — {why}", source.display()))
                        .and_then(|content| match replacement {
                            | Some((old, new)) => update_relationship_target(&content, old, &new)
                                .map_err(|why| eyre!("Failed to transform PowerPoint part at {} — {why}", source.display())),
                            | None => Ok(content),
                        })
                        .and_then(|updated| staged_xml(destination, updated))
                })
                .collect::<ApiResult<Vec<_>>>()
                .and_then(|mut staged| {
                    let relationships_path = root.join("ppt/_rels/presentation.xml.rels");
                    read_xml_rel(relationships_path.clone()).and_then(|relationships| {
                        let revision = relationships.largest_revision_identifier().unwrap_or(0).saturating_add(1);
                        let relationship = Relationship::init()
                            .id(format!("rId{revision}"))
                            .target(format!("slides/slide{slide_number}.xml"))
                            .build();
                        relationships
                            .add_relationship(relationship)
                            .to_string()
                            .map_err(|why| eyre!("Failed to serialize PowerPoint part at {} — {why}", relationships_path.display()))
                            .and_then(|updated| staged_xml(relationships_path, updated))
                            .and_then(|part| {
                                staged.push(part);
                                let presentation_path = root.join("ppt/presentation.xml");
                                read_text(&presentation_path)
                                    .map_err(|why| eyre!("Failed to read PowerPoint part at {} — {why}", presentation_path.display()))
                                    .and_then(|presentation| {
                                        let slide_id = nanoid!(4, &['1', '2', '3', '4', '5', '6', '7', '8', '9'])
                                            .parse::<u32>()
                                            .unwrap_or(revision.saturating_add(255));
                                        add_slide_to_presentation_xml(&presentation, slide_id, revision)
                                            .map_err(|why| eyre!("Failed to transform PowerPoint part at {} — {why}", presentation_path.display()))
                                    })
                                    .and_then(|updated| staged_xml(presentation_path, updated))
                                    .map(|part| {
                                        staged.push(part);
                                        staged
                                    })
                            })
                    })
                })
                .and_then(write_staged)
        }
    }
}
pub(crate) fn copy_image(index: usize, context: &ExportContext, source: &Path, destination: &Path) -> ApiResult<u64> {
    let relationships_path = destination.join(format!("ppt/slides/_rels/slide{index}.xml.rels"));
    read_xml_rel(relationships_path.clone()).and_then(|relationships| {
        relationships
            .relationship
            .iter()
            .find(|relationship| relationship.relationship_type == IMAGE_RELATIONSHIP)
            .map(|relationship| relationship.target.as_str())
            .ok_or_else(|| eyre!("No image relationship found in PowerPoint part at {}", relationships_path.display()))
            .and_then(|target| {
                let mime = match &context.image_source {
                    | ImageSource::Authored(url) => MimeType::from(url.as_str()),
                    | ImageSource::DraftFallback => MimeType::Png,
                };
                let extension = match mime {
                    | MimeType::Unknown(_) => MimeType::Png.file_type(),
                    | mime => mime.file_type(),
                };
                SafePath::new(format!("ppt/media/{}.{extension}", context.identifier))
                    .map_err(Into::<color_eyre::Report>::into)
                    .and_then(|path| path.materialize_under(destination).map_err(Into::into))
                    .and_then(|media_path| {
                        let media = match &context.image_source {
                            | ImageSource::Authored(url) => {
                                let parent = source.parent().unwrap_or_else(|| Path::new("."));
                                read(parent.join(url)).map_err(|why| eyre!("Failed to copy image for {} — {why}", media_path.display()))
                            }
                            | ImageSource::DraftFallback => draft_placeholder_bytes(),
                        };
                        media.and_then(|media| {
                            read_text(&relationships_path)
                                .map_err(|why| eyre!("Failed to read PowerPoint part at {} — {why}", relationships_path.display()))
                                .and_then(|relationship_xml| {
                                    update_relationship_target(&relationship_xml, target, &format!("../media/{}.{extension}", context.identifier))
                                        .map_err(|why| eyre!("Failed to transform PowerPoint part at {} — {why}", relationships_path.display()))
                                })
                                .and_then(|updated| staged_xml(relationships_path, updated))
                                .and_then(|relationship| {
                                    let size = media.len() as u64;
                                    write_staged(vec![relationship, (media_path, media)]).map(|()| size)
                                })
                        })
                    })
            })
    })
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
    let research_activity_data = paths
        .iter()
        .map(|path| {
            ResearchActivity::read(path.clone())
                .map(|value| value.format_with(Some(path.clone())))
                .map_err(|why| eyre!("Failed to read research activity at {} — {why}", path.display()))
        })
        .collect::<ApiResult<Vec<ResearchActivity>>>();
    research_activity_data.and_then(|research_activity_data| {
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
                ArchiveCandidate::from(reference_path.clone())
                    .extract_zip(None)
                    .map(Arc::new)
                    .map_err(|why| {
                        error!(path = reference_path.to_absolute_path(), "=> {} Extract reference", Label::fail());
                        eyre!("Failed to extract PowerPoint reference at {reference_path:?} — {why}")
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
                                ExportContext::project(&data)
                                    .map_err(|why| eyre!("Failed to project research activity for slide {slide_number} — {why}"))
                                    .and_then(|context| {
                                        let fragments = [
                                            format!("ppt/slides/slide{slide_number}.xml"),
                                            format!("ppt/notesSlides/notesSlide{slide_number}.xml"),
                                            format!("ppt/slides/_rels/slide{slide_number}.xml.rels"),
                                        ];
                                        fragments
                                            .iter()
                                            .map(|fragment| reference_extract_path.join(fragment))
                                            .try_for_each(|part| interpolate_values(part, &context))
                                            .and_then(|_| copy_image(slide_number, &context, &index_path, &reference_extract_path))
                                            .map(|_| ())
                                            .inspect_err(|why| {
                                                error!(
                                                    path = index_path.to_absolute_path(),
                                                    "=> {} Copy image for slide {slide_number} — {why}",
                                                    Label::fail()
                                                )
                                            })
                                            .and_then(|_| match aspect_charts.get(index) {
                                                | Some(Some(chart)) => add_aspect_chart(slide_number, &context, chart, &reference_extract_path)
                                                    .inspect_err(|why| {
                                                        error!(
                                                            path = index_path.to_absolute_path(),
                                                            "=> {} Add ASPECT chart to slide {slide_number} — {why}",
                                                            Label::fail()
                                                        );
                                                    }),
                                                | _ => Ok(()),
                                            })
                                    })
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
                                    .and_then(|destination| archive(reference_extract_path.as_ref().clone(), Some(destination), MimeType::Zip))
                            })
                    })
            })
    })
}
fn prepare_destination(output: &Option<PathBuf>, name: &str) -> ApiResult<PathBuf> {
    match output {
        | Some(ref value) => match create_dir_all(value.clone()) {
            | Ok(_) => SafePath::new(PathBuf::from(name).with_extension(MimeType::Powerpoint.file_type()))
                .map_err(Into::<color_eyre::Report>::into)
                .and_then(|path| path.materialize_under(value).map_err(Into::into)),
            | Err(why) => Err(eyre!("Failed to create output directory for PowerPoint export — {why}")),
        },
        | None => Err(eyre!("Missing output directory")),
    }
}
fn staged_xml(path: PathBuf, content: String) -> ApiResult<(PathBuf, Vec<u8>)> {
    validate_xml_well_formed(&content)
        .then(|| (path.clone(), content.into_bytes()))
        .ok_or_else(|| eyre!("Malformed PowerPoint part at {}", path.display()))
}
pub(crate) fn validate_ooxml_files(reference_extract_path: &Path, count: usize) -> ApiResult<()> {
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
        .find(|path| read_text(path).map(|content| !validate_xml_well_formed(&content)).unwrap_or(true));
    match invalid {
        | Some(path) => Err(eyre!(
            "Malformed OOXML content detected before archive write at {}",
            path.to_absolute_path()
        )),
        | None => Ok(()),
    }
}
fn write_staged(parts: Vec<(PathBuf, Vec<u8>)>) -> ApiResult<()> {
    parts
        .into_iter()
        .try_for_each(|(path, content)| write(&path, content).map_err(|why| eyre!("Failed to write PowerPoint part at {} — {why}", path.display())))
}

#[cfg(test)]
mod tests;
