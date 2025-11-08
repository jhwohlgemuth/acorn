//! ## PowerPoint utilities
//!
//! Here you'll find functions for working with [`OOXML`] and creating PowerPoint files.
//!
//! [`OOXML`]: https://en.wikipedia.org/wiki/Office_Open_XML
use crate::prelude::{io, File, PathBuf, Read, Write};
use crate::schema::{ContactPoint, Metadata, Notes, Other, ResearchActivity, Sections};
use crate::util::io::{read_file, write_file};
use crate::util::{files_all, to_absolute_string, Label};
use core::error::Error;
use fancy_regex::Regex;
use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use tracing::{debug, error};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub mod ooxml;
use ooxml::{Relationships, TextParagraph, TextParagraphProperties, TextRun, TextString};

/// Creates zip archive from directory
pub fn archive(path: PathBuf, destination: Option<PathBuf>) -> Result<PathBuf, Box<dyn Error>> {
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let zip_file_path = match destination {
        | Some(value) => value,
        | None => path.with_extension("zip"),
    };
    let zip_file = match File::create(&zip_file_path) {
        | Ok(zip_file) => Some(ZipWriter::new(zip_file)),
        | Err(why) => {
            error!(file = to_absolute_string(path.clone()), "=> {} Create zip archive - {why}", Label::fail());
            None
        }
    };
    if let Some(mut zip) = zip_file {
        let files = files_all(path.clone(), None).into_iter().filter(|x| x.is_file());
        for file_path in files {
            if let Ok(file) = File::open(file_path.clone()) {
                let name = match path.canonicalize() {
                    | Ok(relative) => file_path.strip_prefix(relative).unwrap_or_else(|_| &file_path),
                    | Err(_) => &file_path,
                };
                debug!(file = to_absolute_string(name.to_path_buf()), "=> {} Add file to archive", Label::using());
                match zip.start_file_from_path(name, options) {
                    | Ok(_) => {
                        let mut buffer = Vec::new();
                        match io::copy(&mut file.take(u64::MAX), &mut buffer) {
                            | Ok(_) => match zip.write_all(&buffer) {
                                | Ok(_) => {}
                                | Err(why) => {
                                    error!(file = to_absolute_string(file_path), "=> {} Write zip archive - {why}", Label::fail())
                                }
                            },
                            | Err(why) => {
                                error!("=> {} Copy buffer - {why}", Label::fail())
                            }
                        }
                    }
                    | Err(why) => {
                        error!(file = to_absolute_string(file_path), "=> {} Start zip archive - {why}", Label::fail());
                    }
                }
            }
        }
        match zip.finish() {
            | Ok(_) => Ok(zip_file_path),
            | Err(why) => {
                error!(file = to_absolute_string(path), "=> {} Finish zip archive - {why}", Label::fail());
                Err(why.into())
            }
        }
    } else {
        Err("Unable to create zip archive".into())
    }
}
/// Interpolate research activity data into PowerPoint template
pub fn interpolate_values(path: PathBuf, data: ResearchActivity) {
    let updated = match read_file(path.clone()) {
        | Ok(mut content) => {
            let ResearchActivity {
                contact,
                meta,
                notes,
                title,
                subtitle,
                sections,
                ..
            } = data.clone();
            let Metadata { doi, partners, .. } = meta.clone();
            let Sections {
                achievement,
                impact,
                approach,
                ..
            } = sections;
            let ContactPoint {
                given_name: first,
                family_name: last,
                email,
                ..
            } = contact;
            let caption = meta.first_image_caption();
            let presentation_notes = match notes.clone() {
                | Some(value) => match value {
                    | Other::Formatted(Notes { presentation, .. }) => match presentation {
                        | Some(value) => value,
                        | None => "".to_string(),
                    },
                    | Other::Unformatted(notes) => notes,
                },
                | None => "".to_string(),
            };
            let managers = match notes.clone() {
                | Some(value) => match value {
                    | Other::Formatted(Notes { managers, .. }) => match managers {
                        | Some(value) => value,
                        | None => vec![],
                    },
                    | _ => vec![],
                },
                | None => vec![],
            };
            let programs = match notes {
                | Some(value) => match value {
                    | Other::Formatted(Notes { programs, .. }) => match programs {
                        | Some(value) => value,
                        | None => vec![],
                    },
                    | _ => vec![],
                },
                | None => vec![],
            };
            content = replace_placeholder_with_string(&content, "title", &title);
            content = replace_placeholder_with_string(&content, "subtitle", &subtitle.unwrap_or_else(|| "".to_string()));
            content = replace_placeholder_with_string(&content, "first", &first);
            content = replace_placeholder_with_string(&content, "last", &last);
            content = replace_placeholder_with_string(&content, "email", &email);
            content = replace_placeholder_with_string(&content, "partners", &partners.unwrap().join(", "));
            content = replace_placeholder_with_string(&content, "programs", &programs.join(" and "));
            content = replace_placeholder_with_string(&content, "managers", &managers.join(" and "));
            // TODO: Use CiteAs API to get in Chicago format
            content = replace_placeholder_with_string(&content, "citation", &doi.unwrap()[0]);
            content = replace_placeholder_with_string(&content, "caption", &caption);
            content = replace_placeholder_with_string(&content, "notes", &presentation_notes);
            content = replace_placeholder_with_bullets(&content, "achievement", achievement.unwrap_or_else(Vec::new));
            content = replace_placeholder_with_bullets(&content, "impact", impact);
            content = replace_placeholder_with_bullets(&content, "technical", approach);
            Some(content)
        }
        | Err(why) => {
            error!("=> {} Cannot read file for interpolation - {why}", Label::fail());
            None
        }
    };
    if let Some(content) = updated {
        match write_file(path.clone(), content) {
            | Ok(_) => {}
            | Err(why) => {
                error!("=> {} Cannot write file for interpolation - {why}", Label::fail());
            }
        }
    }
}
/// Parse OOXML paragraph object
pub fn parse_ooxml_paragraph(content: &str) -> Result<TextParagraph, quick_xml::DeError> {
    let parsed = quick_xml::de::from_str::<TextParagraph>(content);
    debug!("=> {} OOXMLParagraph = {:#?}", Label::using(), parsed);
    parsed
}
/// Prettify XML
pub fn prettify_xml(xml: &str) -> String {
    let mut buf: Vec<u8> = Vec::new();
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
    loop {
        let ev = reader.read_event();
        match ev {
            | Ok(Event::Eof) => break, // exits the loop when reaching end of file
            | Ok(event) => writer.write_event(event),
            | Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
        .expect("Failed to parse XML");
        buf.clear();
    }
    let result = core::str::from_utf8(&writer.into_inner())
        .expect("Failed to convert a slice of bytes to a string slice")
        .to_string();
    result
}
/// Read OOXML relationships XML file
pub fn read_xml_rel(path: PathBuf) -> Option<Relationships> {
    match read_file(path) {
        | Ok(content) => {
            let parsed = quick_xml::de::from_str::<Relationships>(&content);
            debug!("=> {} Relationships = {:#?}", Label::using(), parsed);
            match parsed {
                | Ok(value) => Some(value),
                | Err(why) => {
                    error!("=> {} Cannot parse relationships - {why}", Label::fail());
                    None
                }
            }
        }
        | Err(why) => {
            error!("=> {} Cannot read xml.rels file - {why}", Label::fail());
            None
        }
    }
}
/// Replace placeholder with bullet values in PowerPoint template
pub fn replace_placeholder_with_bullets<I: IntoIterator<Item = String>>(xml: &str, placeholder: &str, values: I) -> String {
    let paragraphs = match Regex::new(r#"<a:p>(?:(?!<a:p>|</a:p>)[\s\S])*</a:p>"#) {
        | Ok(re) => re
            .find_iter(xml)
            .flat_map(|m| m.ok())
            .map(|m| m.as_str().to_string())
            .collect::<Vec<String>>(),
        | Err(_) => unreachable!(),
    };
    let selected = match paragraphs
        .into_iter()
        .find(|x| match Regex::new(&format!(r"{{{{\s*{placeholder}\s*}}}}")) {
            | Ok(re) => re.is_match(x).unwrap_or_default(),
            | Err(_) => false,
        }) {
        | Some(value) => value,
        | None => "".to_string(),
    };
    match parse_ooxml_paragraph(&selected) {
        | Ok(paragraph) => {
            let TextParagraph {
                text_paragraph_properties,
                text_run,
                end_paragraph_run_properties,
                ..
            } = paragraph.clone();
            let bullet_properties = match text_paragraph_properties.first() {
                | Some(value) => value,
                | None => &TextParagraphProperties::init().build(),
            };
            let text_properties = match text_run.first() {
                | Some(first) => match first.text_run_properties.first() {
                    | Some(value) => value,
                    | None => todo!(),
                },
                | None => todo!(),
            };
            let content: Vec<String> = values
                .into_iter()
                .map(|value| {
                    let run = TextRun::init()
                        .text_run_properties(vec![text_properties.clone()])
                        .text(TextString { value })
                        .build();
                    let custom = TextParagraph::init()
                        .text_paragraph_properties(vec![bullet_properties.clone()])
                        .text_run(vec![run])
                        .maybe_end_paragraph_run_properties(end_paragraph_run_properties.clone())
                        .build();
                    prettify_xml(&quick_xml::se::to_string(&custom).unwrap())
                })
                .collect();
            let formatted = content.join("");
            xml.replace(&selected, &formatted)
        }
        | Err(why) => {
            debug!(selected, "=> {} Parse OOXML Paragraph - {why}", Label::fail());
            xml.to_string()
        }
    }
}
/// Replace placeholder with string value in PowerPoint template
pub fn replace_placeholder_with_string(content: &str, placeholder: &str, value: &str) -> String {
    match Regex::new(&format!(r"{{{{\s*{placeholder}\s*}}}}")) {
        | Ok(re) => re.replace_all(content, value).to_string(),
        | Err(err) => {
            error!("=> {} Regex replacement - {err}", Label::fail());
            content.to_string()
        }
    }
}

#[cfg(test)]
mod tests;
