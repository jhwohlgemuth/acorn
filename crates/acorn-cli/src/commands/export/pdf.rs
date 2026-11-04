//! PDF export helpers
//!
//! This module renders Research Activity data to HTML and uses Chromium to generate a PDF output file.
use crate::{cli::CommandOptions, template::Converter};
pub(crate) use acorn::io::export::pdf::{browser_executable, initialize_page, render_html_to_pdf, PdfMargins};
use acorn::{
    io::{chart::ChartOptions, ApiResult, InputOutput},
    util::StringConversion,
};
use acorn_host::{fs::SafePath, terminal::Label};
use acorn_schema::research_activity::ResearchActivity;
use chromiumoxide::Page;
use color_eyre::eyre::eyre;
use std::{
    fs::{create_dir_all, remove_file, OpenOptions},
    io,
    path::{absolute, PathBuf},
};
use tracing::{debug, error};

/// Create PDF document from Research Activity data
pub async fn create(page: &Page, options: CommandOptions, chart_options: ChartOptions) -> ApiResult<()> {
    let CommandOptions { output, .. } = options;
    let validated =
        options
            .path
            .ok_or_else(|| eyre!("Missing path for PDF export"))
            .and_then(|path| -> ApiResult<(PathBuf, PathBuf, ResearchActivity)> {
                ResearchActivity::read(path.clone())
                    .map_err(|why| {
                        error!(path = path.to_absolute_path(), "=> {} Read data for PDF export — {why}", Label::fail());
                        eyre!("Read data for PDF export — {why}")
                    })
                    .and_then(|data| {
                        let formatted = data.format_with(Some(path.clone()));
                        path.parent()
                            .map(|p| (p.to_path_buf(), formatted))
                            .ok_or_else(|| {
                                error!(
                                    path = path.to_absolute_path(),
                                    "=> {} Cannot resolve parent directory for PDF export input",
                                    Label::fail()
                                );
                                eyre!("Cannot resolve parent directory for PDF export input")
                            })
                            .map(|(parent, formatted)| (path, parent, formatted))
                    })
            });
    let rendered = validated.and_then(|(path, parent, data)| {
        chart_options.render(data.aspect.as_ref()).and_then(|aspect_chart| {
            data.to_html(aspect_chart.as_deref())
                .map(|content| (path, parent, data, content.to_string()))
        })
    });
    match rendered {
        | Ok((path, parent, data, content)) => {
            let index_path = absolute(parent.join("index.html"))
                .map_err(|why| {
                    error!(
                        path = path.to_absolute_path(),
                        "=> {} Resolve temporary index path — {why}",
                        Label::fail()
                    );
                    why
                })
                .ok();
            if let Some(ref index) = index_path {
                let can_navigate = match OpenOptions::new().write(true).create_new(true).open(index) {
                    | Ok(_) => true,
                    | Err(why) if why.kind() == io::ErrorKind::AlreadyExists => true,
                    | Err(why) => {
                        error!(path = index.to_absolute_path(), "=> {} Create temporary index — {why}", Label::fail());
                        false
                    }
                };
                if can_navigate {
                    let _goto = page.goto(format!("file://{}", index.display())).await;
                }
            }
            match output {
                | Some(output_dir) => match create_dir_all(output_dir.clone()) {
                    | Ok(_) => {
                        let id = data.meta.identifier;
                        let output_path = SafePath::new(format!("{id}.pdf"))
                            .map_err(Into::<color_eyre::Report>::into)
                            .and_then(|path| path.materialize_under(&output_dir).map_err(Into::into))
                            .and_then(|path| absolute(path).map_err(Into::into));
                        match output_path {
                            | Ok(output_path) => {
                                debug!(path = output_path.to_absolute_path(), "=> {} Output", Label::using());
                                render_html_to_pdf(page, &content, &output_path, PdfMargins::FullBleed).await
                            }
                            | Err(why) => Err(eyre!("Failed to resolve PDF output path — {why}")),
                        }
                    }
                    | Err(err) => {
                        error!("=> {} Create directory — {err}", Label::fail());
                        Err(eyre!("Failed to create output directory for PDF — {err}"))
                    }
                },
                | None => Err(eyre!("Missing output directory for PDF export")),
            }
            .inspect(|_| {
                if let Some(index) = &index_path {
                    let _remove = remove_file(index);
                }
            })
        }
        | Err(e) => Err(e),
    }
}
