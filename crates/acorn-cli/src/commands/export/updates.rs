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
use acorn::io::{zip_entries, ApiResult, InputOutput};
use acorn_core::util::MarkdownSupport;
use acorn_host::fs::SafePath;
use acorn_schema::research_activity::{LogbookUpdates, ResearchActivity};
use color_eyre::eyre::eyre;
use quick_xml::escape::escape;
use std::{
    fs::{create_dir_all, write},
    path::{Path, PathBuf},
};

pub(crate) trait LogbookUpdatesExport {
    fn artifact_path(&self, output: &Path, extension: &str) -> ApiResult<PathBuf>;
    fn write_docx(&self, path: &Path) -> ApiResult<()>;
    async fn write_pdf(&self, output_path: &Path, offline: bool, chrome_path: Option<&Path>, full_timestamp: bool) -> ApiResult<()>;
    async fn write_title_pdf(&self, output_path: &Path, offline: bool, chrome_path: Option<&Path>, full_timestamp: bool) -> ApiResult<()>;
}
impl LogbookUpdatesExport for LogbookUpdates {
    fn artifact_path(&self, output: &Path, extension: &str) -> ApiResult<PathBuf> {
        let since = self.since.replace(':', "-");
        SafePath::new(format!("{}-updates-{since}.{extension}", self.identifier))
            .map_err(Into::<color_eyre::Report>::into)
            .and_then(|path| path.materialize_under(output).map_err(Into::into))
    }
    fn write_docx(&self, path: &Path) -> ApiResult<()> {
        let paragraphs = self
            .to_markdown()
            .lines()
            .map(|line| format!("<w:p><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>", escape(line)))
            .collect::<String>();
        let document = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>{paragraphs}<w:sectPr/></w:body></w:document>"
        );
        let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/></Types>";
        let relationships = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/></Relationships>";
        zip_entries(&[
            ("[Content_Types].xml", content_types.as_bytes()),
            ("_rels/.rels", relationships.as_bytes()),
            ("word/document.xml", document.as_bytes()),
        ])
        .and_then(|bytes| write(path, bytes).map_err(|why| eyre!("Failed to write DOCX updates {} — {why}", path.display())))
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
                    | Some(output) => match create_dir_all(&output) {
                        | Ok(()) => match request.format {
                            | FileFormat::Docx => paths
                                .into_iter()
                                .map(|path| {
                                    render(&path, since)
                                        .and_then(|updates| updates.artifact_path(&output, "docx").and_then(|path| updates.write_docx(&path)))
                                })
                                .collect::<ApiResult<Vec<_>>>()
                                .map(|_| ()),
                            | FileFormat::Markdown => paths
                                .into_iter()
                                .map(|path| {
                                    render(&path, since).and_then(|updates| {
                                        updates.artifact_path(&output, "md").and_then(|path| {
                                            write(path, updates.to_markdown()).map_err(|why| eyre!("Failed to write Markdown updates — {why}"))
                                        })
                                    })
                                })
                                .collect::<ApiResult<Vec<_>>>()
                                .map(|_| ()),
                            | FileFormat::Pdf => export_pdf(paths, &output, since, &request).await,
                            | _ => Err(eyre!("Updates export supports only pdf, markdown, and docx formats")),
                        },
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
    ResearchActivity::read(path.to_path_buf())
        .map_err(|why| eyre!("Read research activity data for updates — {why}"))
        .and_then(|activity| activity.updates_since(since).map_err(|why| eyre!(why)))
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
