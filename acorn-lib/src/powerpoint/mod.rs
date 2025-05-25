use crate::schema::{ContactPoint, Notes, Other, ResearchActivity};
use crate::util::{get_all_files, get_command_folder, path_to_string, read_file, write_file, Label};
use fancy_regex::Regex;
use std::error::Error;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use tracing::{debug, error, info};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub mod ooxml;
use ooxml::*;

pub fn archive(path: PathBuf, destination: Option<PathBuf>) -> Result<PathBuf, Box<dyn Error>> {
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let zip_file_path = match destination {
        | Some(value) => value,
        | None => path.with_extension("zip"),
    };
    let mut zip = match File::create(&zip_file_path) {
        | Ok(zip_file) => ZipWriter::new(zip_file),
        | Err(why) => {
            error!(file = path_to_string(path), "=> {} Create zip archive - {why}", Label::fail());
            std::process::exit(exitcode::IOERR);
        }
    };
    let files = get_all_files(path.clone(), None, None).into_iter().filter(|x| x.is_file());
    for file_path in files {
        if let Ok(file) = File::open(file_path.clone()) {
            let name = match path.canonicalize() {
                | Ok(relative) => file_path.strip_prefix(relative).unwrap_or_else(|_| &file_path),
                | Err(_) => &file_path,
            };
            debug!(file = path_to_string(name.to_path_buf()), "=> {} Add file to archive", Label::using());
            match zip.start_file_from_path(name, options) {
                | Ok(_) => {
                    let mut buffer = Vec::new();
                    match io::copy(&mut file.take(u64::MAX), &mut buffer) {
                        | Ok(_) => match zip.write_all(&buffer) {
                            | Ok(_) => {}
                            | Err(why) => {
                                error!(file = path_to_string(file_path), "=> {} Write zip archive - {why}", Label::fail())
                            }
                        },
                        | Err(why) => {
                            error!("=> {} Copy buffer - {why}", Label::fail())
                        }
                    }
                }
                | Err(why) => {
                    error!(file = path_to_string(file_path), "=> {} Start zip archive - {why}", Label::fail());
                }
            }
        }
    }
    match zip.finish() {
        | Ok(_) => Ok(zip_file_path),
        | Err(why) => {
            error!(file = path_to_string(path), "=> {} Finish zip archive - {why}", Label::fail());
            Err(why.into())
        }
    }
}
pub fn extract(path: PathBuf, destination: Option<PathBuf>) -> Result<PathBuf, Box<dyn Error>> {
    let root = get_command_folder("extract", destination);
    match File::open(path.clone()) {
        | Ok(zip_file) => match ZipArchive::new(zip_file) {
            | Ok(mut archive) => {
                for index in 0..archive.len() {
                    if let Ok(mut file) = archive.by_index(index) {
                        let target = root.join(file.name());
                        if let Some(parent) = target.parent() {
                            match create_dir_all(parent) {
                                | Ok(_) => {}
                                | Err(why) => error!(path = path_to_string(parent.to_path_buf()), "=> {} Create - {}", Label::fail(), why),
                            }
                        }
                        if let Ok(mut output_file) = File::create(&target) {
                            io::copy(&mut file, &mut output_file).unwrap();
                        } else {
                            error!(path = path_to_string(target), "=> {} Create file", Label::fail());
                            std::process::exit(exitcode::UNAVAILABLE);
                        }
                    }
                }
                info!(path = root.to_str().unwrap(), "=> {} Extract zip archive", Label::pass());
                Ok(root)
            }
            | Err(err) => {
                error!(
                    error = err.to_string(),
                    path = path.to_str().unwrap(),
                    "=> {} Read zip archive",
                    Label::fail()
                );
                std::process::exit(exitcode::UNAVAILABLE);
            }
        },
        | Err(err) => {
            error!(error = err.to_string(), path = path.to_str().unwrap(), "=> {} Read file", Label::fail());
            std::process::exit(exitcode::UNAVAILABLE);
        }
    }
}
pub fn interpolate_values(path: PathBuf, data: ResearchActivity) {
    let updated = match read_file(path.clone()) {
        | Ok(mut content) => {
            let ResearchActivity {
                contact,
                meta,
                notes,
                title,
                subtitle,
                ..
            } = data.clone();
            let ContactPoint {
                given_name: first,
                family_name: last,
                email,
                ..
            } = contact;
            let caption = meta.get_first_graphic_caption();
            let presentation_notes = match notes {
                | Some(value) => match value {
                    | Other::Formatted(Notes { presentation, .. }) => match presentation {
                        | Some(value) => value,
                        | None => "".to_string(),
                    },
                    | Other::Unformatted(notes) => notes,
                },
                | None => "".to_string(),
            };
            content = replace(&content, "title", &title);
            content = replace(&content, "subtitle", &subtitle.unwrap_or_else(|| "".to_string()));
            content = replace(&content, "first", &first);
            content = replace(&content, "last", &last);
            content = replace(&content, "email", &email);
            content = replace(&content, "caption", &caption);
            content = replace(&content, "notes", &presentation_notes);
            content
        }
        | Err(why) => {
            error!("=> {} Cannot read file for interpolation - {why}", Label::fail());
            std::process::exit(exitcode::IOERR);
        }
    };
    match write_file(path.clone(), updated) {
        | Ok(_) => {}
        | Err(why) => {
            error!("=> {} Cannot write file for interpolation - {why}", Label::fail());
            std::process::exit(exitcode::IOERR);
        }
    }
}
pub fn parse_ooxml_paragraph(content: &str) -> Result<TextParagraph, quick_xml::DeError> {
    let parsed = quick_xml::de::from_str::<TextParagraph>(content);
    debug!("=> {} OOXMLParagraph = {:#?}", Label::using(), parsed);
    parsed
}
pub fn read_xml_rel(path: PathBuf) -> Result<Relationships, quick_xml::DeError> {
    match read_file(path) {
        | Ok(content) => {
            let parsed = quick_xml::de::from_str::<Relationships>(&content);
            debug!("=> {} Relationships = {:#?}", Label::using(), parsed);
            parsed
        }
        | Err(why) => {
            error!("=> {} Cannot read xml.rels file - {why}", Label::fail());
            std::process::exit(exitcode::IOERR);
        }
    }
}
pub fn replace(content: &str, name: &str, value: &str) -> String {
    match Regex::new(&format!(r"{{{{\s*{}\s*}}}}", name)) {
        | Ok(re) => re.replace_all(content, value).to_string(),
        | Err(err) => {
            error!("=> {} Regex replacement - {err}", Label::fail());
            content.to_string()
        }
    }
}

#[cfg(test)]
mod tests;
