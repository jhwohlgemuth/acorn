//! PDF export helpers
//!
//! This module renders Research Activity data to HTML and uses ironpress
//! to generate a PDF output file.
use crate::cli::CommandOptions;
use crate::template::Convert;
use acorn::io::{ApiResult, InputOutput};
use acorn::prelude::{create_dir_all, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::schema::MediaObject;
use acorn::util::{Label, StringConversion};
use color_eyre::eyre::eyre;
use std::path::Path;
use ironpress::{HtmlConverter, Margin, PageSize};
use tracing::{debug, error};

/// Create PDF document from Research Activity data
pub async fn create(options: CommandOptions) -> ApiResult<()> {
    let CommandOptions { output, .. } = options;
    let validated =
        options
            .path
            .ok_or_else(|| eyre!("Missing path for PDF export"))
            .and_then(|path| -> ApiResult<(PathBuf, ResearchActivity)> {
                ResearchActivity::read(path.clone())
                    .map_err(|why| {
                        error!(path = path.to_absolute_string(), "=> {} Read data for PDF export — {why}", Label::fail());
                        eyre!("Read data for PDF export — {why}")
                    })
                    .map(|data| (path.clone(), data.format_with(Some(path))))
            });
    match validated {
        | Ok((source_path, data)) => {
            let source_parent = source_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
            let mut resolved_data = data.clone();
            if let Some(media) = resolved_data.meta.media.as_mut() {
                media.iter_mut().for_each(|item| {
                    if let MediaObject::Image(image) = item {
                        if let Some(url) = image.content_url.as_ref() {
                            if !url.is_empty() && !url.starts_with("data:") && !Path::new(url).is_absolute() {
                                image.content_url = Some(source_parent.join(url).to_string_lossy().to_string());
                            }
                        }
                    }
                });
            }
            let content = resolved_data.to_html().to_string();
            if std::env::var("ACORN_DEBUG_PDF_HTML").ok().as_deref() == Some("1") {
                let debug_path = source_parent.join("acorn-pdf-debug.html");
                println!("DEBUG_PDF_HTML_PATH={} ", debug_path.display());
                if let Err(why) = std::fs::write(&debug_path, content.as_bytes()) {
                    println!("DEBUG_PDF_HTML_WRITE_ERR={why}");
                }
            }
            match output {
                | Some(output_dir) => match create_dir_all(output_dir.clone()) {
                    | Ok(_) => {
                        let id = data.meta.identifier;
                        let output_path = output_dir.join(format!("{}.pdf", id));
                        debug!(path = output_path.to_absolute_string(), "=> {} Output", Label::using());
                        match HtmlConverter::new()
                            .page_size(PageSize::LETTER)
                            .margin(Margin::uniform(0.0))
                            .sanitize(false)
                            .base_path(&source_parent)
                            .convert(&content)
                        {
                            | Ok(pdf_bytes) => {
                                acorn::prelude::write(&output_path, pdf_bytes)
                                    .map_err(|why| {
                                        error!("=> {} Write PDF file — {why}", Label::fail());
                                        eyre!("Failed to write PDF file — {why}")
                                    })
                            }
                            | Err(err) => {
                                error!("=> {} Convert HTML to PDF — {err}", Label::fail());
                                Err(eyre!("Failed to convert HTML to PDF — {err}"))
                            }
                        }
                    }
                    | Err(err) => {
                        error!("=> {} Create directory — {err}", Label::fail());
                        Err(eyre!("Failed to create output directory for PDF — {err}"))
                    }
                },
                | None => Err(eyre!("Missing output directory for PDF export")),
            }
        }
        | Err(e) => Err(e),
    }
}
