//! PDF export helpers
//!
//! This module renders Research Activity data to HTML and uses Playwright to
//! generate a PDF output file.
use crate::cli::CommandLineOptions;
use crate::template::Convert;
use acorn::io::InputOutput;
use acorn::prelude::{absolute, create_dir_all, remove_file, File};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{Label, ToAbsoluteString};
use playwright::api::Page;
use std::process::exit;
use tracing::{debug, error};

/// Create PDF document from Research Activity data
pub async fn create(page: &Page, options: CommandLineOptions) {
    let CommandLineOptions { output, .. } = options;
    let path = match options.path {
        | Some(value) => value,
        | None => unimplemented!(),
    };
    let data = match ResearchActivity::read(path.clone()) {
        | Ok(value) => value.format(Some(path.clone())),
        | Err(why) => {
            error!(path = path.to_absolute_string(), "=> {} Read data for PDF export - {why}", Label::fail());
            exit(exitcode::UNAVAILABLE);
        }
    };
    let content = data.to_html().to_string();
    // NOTE: Create temporary index.html because rust-playwright does not support setting baseURL
    let index = absolute(path.parent().unwrap().join("index.html")).unwrap();
    if File::create(&index).is_ok() {
        page.goto_builder(&format!("file://{}", index.display())).goto().await.unwrap();
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
                    "=> {} Create directory - {err}",
                    Label::fail()
                );
            }
        },
        | None => unreachable!(),
    };
    let _remove = remove_file(index);
}
