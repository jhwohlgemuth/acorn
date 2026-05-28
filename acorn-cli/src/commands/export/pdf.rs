//! PDF export helpers
//!
//! This module renders Research Activity data to HTML and uses Playwright to
//! generate a PDF output file.
use crate::cli::CommandOptions;
use crate::template::Convert;
use acorn::io::InputOutput;
use acorn::prelude::{absolute, create_dir_all, remove_file, ErrorKind, OpenOptions};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{Label, StringConversion};
use playwright::api::Page;
use std::process::exit;
use tracing::{debug, error};

/// Create PDF document from Research Activity data
#[allow(clippy::unwrap_used)]
pub async fn create(page: &Page, options: CommandOptions) {
    let CommandOptions { output, .. } = options;
    let path = match options.path {
        | Some(value) => value,
        | None => unimplemented!(),
    };
    let data = match ResearchActivity::read(path.clone()) {
        | Ok(value) => {
            let context = Some(path.clone());
            value.format_with(context)
        }
        | Err(why) => {
            error!(path = path.to_absolute_string(), "=> {} Read data for PDF export — {why}", Label::fail());
            exit(exitcode::UNAVAILABLE);
        }
    };
    let content = data.to_html().to_string();
    // NOTE: Create temporary index.html because rust-playwright does not support setting baseURL
    let parent = match path.parent() {
        | Some(value) => value.to_path_buf(),
        | None => {
            error!(
                path = path.to_absolute_string(),
                "=> {} Cannot resolve parent directory for PDF export input",
                Label::fail()
            );
            exit(exitcode::DATAERR);
        }
    };
    let cleanup = match absolute(parent.join("index.html")) {
        | Ok(index) => {
            let can_navigate = match OpenOptions::new().write(true).create_new(true).open(&index) {
                | Ok(_) => true,
                | Err(why) if why.kind() == ErrorKind::AlreadyExists => true,
                | Err(why) => {
                    error!(path = index.to_absolute_string(), "=> {} Create temporary index — {why}", Label::fail());
                    false
                }
            };
            if can_navigate {
                page.goto_builder(&format!("file://{}", index.display())).goto().await.unwrap();
            }
            Some(index)
        }
        | Err(why) => {
            error!(
                path = path.to_absolute_string(),
                "=> {} Resolve temporary index path — {why}",
                Label::fail()
            );
            None
        }
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
                    "=> {} Create directory — {err}",
                    Label::fail()
                );
            }
        },
        | None => unreachable!(),
    };
    if let Some(index) = cleanup {
        let _remove = remove_file(index);
    }
}
