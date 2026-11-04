//! Since-filtered logbook updates artifacts.
#[cfg(feature = "pdf")]
use super::html::LogbookUpdatesHtml;
#[cfg(feature = "pdf")]
use super::pdf;
use super::ExportRequest;
use crate::cli::{
    arguments::{FileFormat, Target},
    resolve_paths,
};
use acorn::io::{export::updates, ApiResult};
use acorn_core::util::MimeType;
use acorn_host::terminal::Label;
use acorn_schema::research_activity::LogbookUpdates;
use color_eyre::eyre::eyre;
use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};
use tracing::info;

pub(crate) trait LogbookUpdatesExport {
    fn artifact_path(&self, output: &Path, extension: &str) -> ApiResult<PathBuf>;
    fn write_docx(&self, path: &Path) -> ApiResult<()>;
    async fn write_pdf(&self, output_path: &Path, offline: bool, chrome_path: Option<&Path>, full_timestamp: bool) -> ApiResult<()>;
    async fn write_title_pdf(&self, output_path: &Path, offline: bool, chrome_path: Option<&Path>, full_timestamp: bool) -> ApiResult<()>;
}
impl LogbookUpdatesExport for LogbookUpdates {
    fn artifact_path(&self, output: &Path, extension: &str) -> ApiResult<PathBuf> {
        updates::updates_artifact_path(self, output, extension)
    }
    fn write_docx(&self, path: &Path) -> ApiResult<()> {
        updates::write_docx_updates(self, path)
    }
    #[cfg(feature = "pdf")]
    async fn write_pdf(&self, output_path: &Path, offline: bool, chrome_path: Option<&Path>, full_timestamp: bool) -> ApiResult<()> {
        write_html_pdf(self.to_html(full_timestamp), output_path, offline, chrome_path).await
    }
    #[cfg(not(feature = "pdf"))]
    async fn write_pdf(&self, _output_path: &Path, _offline: bool, _chrome_path: Option<&Path>, _full_timestamp: bool) -> ApiResult<()> {
        Err(eyre!("PDF export requires the 'pdf' feature. Rebuild with --features pdf"))
    }
    #[cfg(feature = "pdf")]
    async fn write_title_pdf(&self, output_path: &Path, offline: bool, chrome_path: Option<&Path>, full_timestamp: bool) -> ApiResult<()> {
        write_html_pdf(self.to_title_html(full_timestamp), output_path, offline, chrome_path).await
    }
    #[cfg(not(feature = "pdf"))]
    async fn write_title_pdf(&self, _output_path: &Path, _offline: bool, _chrome_path: Option<&Path>, _full_timestamp: bool) -> ApiResult<()> {
        Err(eyre!("PDF export requires the 'pdf' feature. Rebuild with --features pdf"))
    }
}
pub(super) async fn export(request: ExportRequest) -> ApiResult<()> {
    match (request.target, request.since.as_deref(), &request.format) {
        | (Target::Updates, Some(since), FileFormat::Docx | FileFormat::Markdown | FileFormat::Pdf) => {
            match resolve_paths(&request.path, &request.options).await {
                | Ok(paths) => match request.output.clone() {
                    | Some(output) if request.dry_run => {
                        let extension = MimeType::from(&request.format).file_type();
                        paths
                            .into_iter()
                            .map(|path| {
                                render(&path, since).and_then(|updates| {
                                    updates.artifact_path(&output, &extension).map(|destination| {
                                        info!("=> {} Would write updates artifact {}", Label::dry_run(), destination.display());
                                    })
                                })
                            })
                            .collect::<ApiResult<Vec<_>>>()
                            .map(|_| ())
                    }
                    | Some(output) => match create_dir_all(&output) {
                        | Ok(()) => {
                            let extension = MimeType::from(&request.format).file_type();
                            match request.format {
                                | FileFormat::Docx => paths
                                    .into_iter()
                                    .map(|path| {
                                        render(&path, since)
                                            .and_then(|updates| updates.artifact_path(&output, &extension).and_then(|path| updates.write_docx(&path)))
                                    })
                                    .collect::<ApiResult<Vec<_>>>()
                                    .map(|_| ()),
                                | FileFormat::Markdown => paths
                                    .into_iter()
                                    .map(|path| {
                                        render(&path, since).and_then(|updates| {
                                            updates
                                                .artifact_path(&output, &extension)
                                                .and_then(|path| updates::write_markdown_updates(&updates, &path))
                                        })
                                    })
                                    .collect::<ApiResult<Vec<_>>>()
                                    .map(|_| ()),
                                | FileFormat::Pdf => export_pdf(paths, &output, since, &request).await,
                                | _ => Err(eyre!("Updates export supports only pdf, markdown, and docx formats")),
                            }
                        }
                        | Err(why) => Err(eyre!("Failed to create updates output directory {} — {why}", output.display())),
                    },
                    | None => Err(eyre!("Updates export requires an output directory")),
                },
                | Err(why) => Err(why),
            }
        }
        | (Target::Updates, None, _) => Err(eyre!("Updates export requires --since <TIMESTAMP>")),
        | (Target::Updates, _, _) => Err(eyre!("Updates export supports only pdf, markdown, and docx formats")),
        | (_, _, FileFormat::Docx) => Err(eyre!("DOCX export is supported only with --target updates")),
        | _ => Err(eyre!("Invalid updates export request")),
    }
}
#[cfg(feature = "pdf")]
async fn export_pdf(paths: Vec<PathBuf>, output: &Path, since: &str, request: &ExportRequest) -> ApiResult<()> {
    match pdf::initialize_page(request.options.offline, request.chrome_path.as_deref()).await {
        | Ok(session) => {
            let updates = paths.into_iter().map(|path| render(&path, since)).collect::<ApiResult<Vec<_>>>();
            let result = match updates {
                | Ok(updates) => {
                    let page = session.page();
                    let mut result = Ok(());
                    for update in updates {
                        if result.is_ok() {
                            result = match update.artifact_path(output, "pdf") {
                                | Ok(path) => match update.to_html(request.full_timestamp) {
                                    | Ok(content) => pdf::render_html_to_pdf(page, &content, &path, pdf::PdfMargins::Stylesheet).await,
                                    | Err(why) => Err(why),
                                },
                                | Err(why) => Err(why),
                            };
                        }
                    }
                    result
                }
                | Err(why) => Err(why),
            };
            session.close().await;
            result
        }
        | Err(why) => Err(why),
    }
}
#[cfg(not(feature = "pdf"))]
async fn export_pdf(_paths: Vec<PathBuf>, _output: &Path, _since: &str, _request: &ExportRequest) -> ApiResult<()> {
    Err(eyre!("PDF export requires the 'pdf' feature. Rebuild with --features pdf"))
}
fn render(path: &Path, since: &str) -> ApiResult<LogbookUpdates> {
    updates::render_updates(path, since)
}
#[cfg(feature = "pdf")]
async fn write_html_pdf(content: ApiResult<String>, output_path: &Path, offline: bool, chrome_path: Option<&Path>) -> ApiResult<()> {
    match pdf::initialize_page(offline, chrome_path).await {
        | Ok(session) => {
            let result = match content {
                | Ok(content) => pdf::render_html_to_pdf(session.page(), &content, output_path, pdf::PdfMargins::Stylesheet).await,
                | Err(why) => Err(why),
            };
            session.close().await;
            result
        }
        | Err(why) => Err(why),
    }
}
