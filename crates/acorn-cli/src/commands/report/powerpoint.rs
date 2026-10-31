//! Category slide inputs for project logbook presentations.
use crate::commands::export::powerpoint::{add_slide, copy_image, validate_ooxml_files};
use acorn::io::{
    archive,
    export::ExportContext,
    powerpoint::{interpolate_values, ooxml::validate_xml_well_formed},
    ApiResult, ArchiveCandidate, ArchiveExtraction,
};
use acorn_core::{util::interpolation::ReplacementPlan, util::MimeType};
use acorn_host::fs::read_text;
use acorn_schema::research_activity::logbook::MAX_LOGBOOK_SUMMARY_CHARACTERS;
use acorn_schema::research_activity::{LogbookCategory, LogbookUpdates, ResearchActivity};
use color_eyre::eyre::eyre;
use core::ops::Range;
use quick_xml::{events::Event, Reader};
use std::{fs::write, path::Path};
use strum::IntoEnumIterator;

// Conservative fit for the retained reference's 14-point body, 7-inch column, and expanded 5.5-inch box.
const CATEGORY_LINE_CHARACTERS: usize = 58;
const CATEGORY_TEXT_HEIGHT: &str = "5000000";
const MAX_CATEGORY_LINES: usize = 22;

/// Report-specific PowerPoint behavior for a project research activity.
pub(super) trait PowerPointReporter {
    /// Compose selected logbook entries into ordered category slide data.
    fn compose(&self, updates: &LogbookUpdates) -> ApiResult<Vec<CategorySlide>>;
    /// Render selected category slides into a local PowerPoint deck.
    fn render(&self, updates: &LogbookUpdates, index_path: &Path, reference: &Path, destination: &Path) -> ApiResult<Option<std::path::PathBuf>>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CategorySlide {
    pub category: LogbookCategory,
    pub project_id: String,
    pub title: String,
    pub subtitle: String,
    pub achievement: String,
    pub notes: String,
    pub keywords: Vec<String>,
    pub principal_investigators: Vec<String>,
    pub sponsors: Vec<String>,
    pub image_url: Option<String>,
}

impl PowerPointReporter for ResearchActivity {
    fn compose(&self, updates: &LogbookUpdates) -> ApiResult<Vec<CategorySlide>> {
        LogbookCategory::iter()
            .filter_map(|category| {
                let entries = updates
                    .entries
                    .iter()
                    .filter(|entry| !entry.global && entry.project_id == updates.identifier && entry.category == category)
                    .collect::<Vec<_>>();
                (!entries.is_empty()).then_some((category, entries))
            })
            .map(|(category, entries)| {
                let oversized = entries
                    .iter()
                    .find(|entry| entry.summary.chars().count() > MAX_LOGBOOK_SUMMARY_CHARACTERS);
                let achievement = entries
                    .iter()
                    .map(|entry| {
                        let items = entry.items.as_ref().map_or(String::new(), |items| {
                            items.iter().map(|item| format!("\n  {item}")).collect::<String>()
                        });
                        format!("{} — {}{}", entry.timestamp, entry.summary, items)
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n");
                let window = updates.through.as_deref().unwrap_or("unspecified");
                match oversized {
                    | Some(entry) => Err(eyre!(
                        "PowerPoint source '{}' for project '{}' category '{category:?}' exceeds the {MAX_LOGBOOK_SUMMARY_CHARACTERS}-character logbook summary limit",
                        entry.identifier,
                        updates.identifier
                    )),
                    | None => Ok(CategorySlide {
                        category,
                        project_id: updates.identifier.clone(),
                        title: updates.title.clone(),
                        subtitle: format!("{category:?} | {} to {window}", updates.since),
                        achievement,
                        notes: entries.iter().map(|entry| entry.identifier.as_str()).collect::<Vec<_>>().join(", "),
                        keywords: updates.keywords.clone(),
                        principal_investigators: updates.principal_investigators.clone(),
                        sponsors: updates.sponsors.clone(),
                        image_url: self.authored_image_url(),
                    }),
                }
            })
            .collect()
    }
    fn render(&self, updates: &LogbookUpdates, index_path: &Path, reference: &Path, destination: &Path) -> ApiResult<Option<std::path::PathBuf>> {
        self.compose(updates).and_then(|slides| {
            if slides.is_empty() {
                Ok(None)
            } else {
                slides
                    .iter()
                    .find(|slide| estimated_lines(&slide.achievement) > MAX_CATEGORY_LINES)
                    .map(|slide| {
                        Err(eyre!(
                            "PowerPoint category '{:?}' for project '{}' needs {} lines; template capacity is {MAX_CATEGORY_LINES}",
                            slide.category,
                            slide.project_id,
                            estimated_lines(&slide.achievement)
                        ))
                    })
                    .unwrap_or(Ok(()))
                    .and_then(|()| ArchiveCandidate::from(reference.to_path_buf()).extract_zip(None))
                    .and_then(|root| {
                        let first_slide = root.join("ppt/slides/slide1.xml");
                        read_text(&first_slide)
                            .map_err(|why| eyre!("Read PowerPoint reference {} — {why}", first_slide.display()))
                            .and_then(|xml| category_layout(&xml))
                            .and_then(|xml| write(&first_slide, xml).map_err(|why| eyre!("Write PowerPoint slide layout — {why}")))
                            .and_then(|()| {
                                let notes = root.join("ppt/notesSlides/notesSlide1.xml");
                                read_text(&notes)
                                    .map_err(|why| eyre!("Read PowerPoint notes {} — {why}", notes.display()))
                                    .and_then(|xml| {
                                        xml.contains("{{ notes }}")
                                            .then_some(())
                                            .ok_or_else(|| eyre!("PowerPoint reference notes lack the notes placeholder"))
                                    })
                            })
                            .and_then(|()| (2..=slides.len()).try_for_each(|number| add_slide(number, &root)))
                            .and_then(|()| {
                                slides.iter().enumerate().try_for_each(|(index, slide)| {
                                    let number = index.saturating_add(1);
                                    slide_context(self, slide).and_then(|context| {
                                        [
                                            root.join(format!("ppt/slides/slide{number}.xml")),
                                            root.join(format!("ppt/notesSlides/notesSlide{number}.xml")),
                                            root.join(format!("ppt/slides/_rels/slide{number}.xml.rels")),
                                        ]
                                        .into_iter()
                                        .try_for_each(|path| interpolate_values(path, &context))
                                        .and_then(|()| copy_image(number, &context, index_path, &root).map(|_| ()))
                                    })
                                })
                            })
                            .and_then(|()| validate_ooxml_files(&root, slides.len()))
                            .and_then(|()| archive(root, Some(destination.to_path_buf()), MimeType::Zip))
                            .map(Some)
                    })
            }
        })
    }
}

fn category_layout(xml: &str) -> ApiResult<String> {
    if !validate_xml_well_formed(xml)
        || !["{{ title }}", "{{ subtitle }}", "{{ achievement }}"]
            .iter()
            .all(|token| xml.contains(token))
    {
        Err(eyre!(
            "PowerPoint reference slide is malformed or lacks title, subtitle, or achievement placeholders"
        ))
    } else {
        shape_ranges(xml).and_then(|ranges| {
            let achievement_count = ranges
                .iter()
                .filter(|range| xml.get((*range).clone()).is_some_and(|shape| shape.contains("{{ achievement }}")))
                .count();
            if achievement_count != 1 {
                Err(eyre!("PowerPoint reference must have one achievement shape"))
            } else {
                ranges.into_iter().rev().try_fold(xml.to_string(), |mut output, range| {
                    xml.get(range.clone())
                        .ok_or_else(|| eyre!("PowerPoint shape range is invalid"))
                        .and_then(|shape| {
                            if shape.contains("{{ achievement }}") {
                                enlarged_achievement(shape).map(|expanded| output.replace_range(range, &expanded))
                            } else {
                                if shape.contains("{{ impact }}") || shape.contains("{{ technical }}") {
                                    output.replace_range(range, "");
                                }
                                Ok(())
                            }
                        })
                        .map(|()| output)
                })
            }
        })
    }
}

fn enlarged_achievement(shape: &str) -> ApiResult<String> {
    shape
        .find("<a:xfrm")
        .and_then(|transform| {
            shape
                .get(transform..)
                .and_then(|tail| tail.find("<a:ext ").map(|offset| transform.saturating_add(offset)))
        })
        .and_then(|start| {
            shape
                .get(start..)
                .and_then(|tail| tail.find("/>").map(|end| start..start.saturating_add(end)))
        })
        .and_then(|range| {
            shape
                .get(range.clone())
                .and_then(|tag| tag.find("cy=\"").map(|offset| range.start.saturating_add(offset).saturating_add(4)))
        })
        .and_then(|start| {
            shape
                .get(start..)
                .and_then(|tail| tail.find('"').map(|end| start..start.saturating_add(end)))
        })
        .map(|range| {
            let mut resized = shape.to_string();
            resized.replace_range(range, CATEGORY_TEXT_HEIGHT);
            resized.replace("Scientific Achievement", "Category Updates")
        })
        .ok_or_else(|| eyre!("PowerPoint achievement shape has no size to expand"))
}

fn estimated_lines(text: &str) -> usize {
    text.lines()
        .map(|line| {
            line.split_whitespace()
                .fold((1_usize, 0_usize), |(lines, used), word| {
                    let length = word.chars().count();
                    let next = used.saturating_add(length).saturating_add(usize::from(used > 0));
                    if next > CATEGORY_LINE_CHARACTERS {
                        (
                            lines.saturating_add(length.div_ceil(CATEGORY_LINE_CHARACTERS)),
                            length % CATEGORY_LINE_CHARACTERS,
                        )
                    } else {
                        (lines, next)
                    }
                })
                .0
        })
        .sum()
}

fn shape_ranges(xml: &str) -> ApiResult<Vec<Range<usize>>> {
    let mut reader = Reader::from_str(xml);
    let mut start = None;
    let mut depth = 0_usize;
    let mut ranges = Vec::new();
    loop {
        let position = reader.buffer_position() as usize;
        match reader.read_event() {
            | Ok(Event::Start(event)) if event.name().as_ref() == b"p:sp" => {
                start = Some(position);
                depth = 1;
            }
            | Ok(Event::Start(_)) if start.is_some() => depth = depth.saturating_add(1),
            | Ok(Event::End(_)) if start.is_some() => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    if let Some(begin) = start.take() {
                        ranges.push(begin..reader.buffer_position() as usize);
                    }
                }
            }
            | Ok(Event::Eof) => break Ok(ranges),
            | Ok(_) => {}
            | Err(why) => break Err(eyre!("Parse PowerPoint slide shapes — {why}")),
        }
    }
}

fn slide_context(activity: &ResearchActivity, slide: &CategorySlide) -> ApiResult<ExportContext> {
    ExportContext::project(activity).map(|mut context| {
        let base = context.replacements.iter().map(|(name, value)| (name.to_string(), value.clone()));
        let overrides = [
            (
                "achievement".to_string(),
                slide.achievement.split("\n\n").map(str::to_string).collect::<Vec<_>>().into(),
            ),
            ("notes".to_string(), format!("Source entries: {}", slide.notes).into()),
            ("subtitle".to_string(), slide.subtitle.clone().into()),
            ("title".to_string(), slide.title.clone().into()),
        ];
        context.replacements = ReplacementPlan::new(base.chain(overrides));
        context
    })
}
