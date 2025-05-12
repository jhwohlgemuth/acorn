use crate::util::{get_command_folder, path_to_string, read_file, Label};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::create_dir_all;
use std::fs::File;
use std::path::PathBuf;
use tracing::{debug, error, info};
use zip::ZipArchive;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Relationships {
    relationship: Vec<Relationship>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Relationship {
    #[serde(rename = "@Id")]
    id: String,
    #[serde(rename = "@Type")]
    relationship_type: String,
    #[serde(rename = "@Target")]
    target: String,
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
                            std::io::copy(&mut file, &mut output_file).unwrap();
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

#[cfg(test)]
mod tests;
