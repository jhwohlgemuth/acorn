use acorn_lib::schema::ResearchActivity;
use acorn_lib::util::{get_all_files, path_to_string, Label};
use color_eyre::eyre::{Report, Result};
use rayon::prelude::*;
use similar::{ChangeTag, TextDiff};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tracing::{error, info};

#[allow(clippy::too_many_arguments)]
pub fn run(path: &Option<PathBuf>) -> Result<(), Report> {
    if let Some(path) = path {
        let paths = get_all_files(path.into(), None);
        paths.par_iter().for_each(|path| match ResearchActivity::read(path.clone()) {
            | Ok(data) => {
                let filepath = path_to_string(path.clone());
                let mut file = match File::create(path) {
                    | Ok(file) => file,
                    | Err(why) => {
                        error!(file = filepath, "=> {} Create - {}", Label::fail(), why);
                        std::process::exit(exitcode::UNAVAILABLE);
                    }
                };
                let old_content = serde_json::to_string_pretty(&data.clone()).unwrap();
                let formatted = data.clone().format(Some(path.clone()));
                let new_content = serde_json::to_string_pretty(&formatted.clone().format(Some(path.clone()))).unwrap();
                let diff = TextDiff::from_lines(old_content.as_str(), new_content.as_str());
                for change in diff.iter_all_changes() {
                    let sign = match change.tag() {
                        | ChangeTag::Delete => "-",
                        | ChangeTag::Insert => "+",
                        | ChangeTag::Equal => " ",
                    };
                    print!("{}{}", sign, change);
                }
                match file.write_all(new_content.as_bytes()) {
                    | Ok(_) => {
                        info!(file = filepath, "=> {} Formatted", Label::pass());
                    }
                    | Err(why) => {
                        error!(file = filepath, "=> {} Write - {}", Label::fail(), why);
                        std::process::exit(exitcode::UNAVAILABLE);
                    }
                }
            }
            | Err(err) => {
                error!("=> {} Read research activity data at - {}", Label::fail(), err);
                std::process::exit(exitcode::DATAERR);
            }
        });
    }
    Ok(())
}
