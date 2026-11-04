//! Shared since-filtered logbook projection and local document artifacts.
pub mod html;
use crate::io::{zip_entries, ApiResult, InputOutput};
use acorn_core::util::MarkdownSupport;
use acorn_host::fs::SafePath;
use acorn_schema::research_activity::{LogbookUpdates, ResearchActivity};
use color_eyre::eyre::eyre;
use quick_xml::escape::escape;
use std::{
    fs::write,
    path::{Path, PathBuf},
};

/// Resolve one research activity's logbook entries at an inclusive lower bound.
pub fn render_updates(path: &Path, since: &str) -> ApiResult<LogbookUpdates> {
    ResearchActivity::read(path.to_path_buf())
        .map_err(|why| eyre!("Read research activity data for updates — {why}"))
        .and_then(|activity| activity.updates_since(since).map_err(|why| eyre!(why)))
}
/// Resolve the exact local artifact name for one updates projection.
pub fn updates_artifact_path(updates: &LogbookUpdates, output: &Path, extension: &str) -> ApiResult<PathBuf> {
    let since = updates.since.replace(':', "-");
    SafePath::new(format!("{}-updates-{since}.{extension}", updates.identifier))
        .map_err(Into::<color_eyre::Report>::into)
        .and_then(|path| path.materialize_under(output).map_err(Into::into))
}
/// Write a minimal DOCX updates document at the exact selected path.
pub fn write_docx_updates(updates: &LogbookUpdates, path: &Path) -> ApiResult<()> {
    let paragraphs = updates
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
/// Write the shared Markdown projection at the exact selected path.
pub fn write_markdown_updates(updates: &LogbookUpdates, path: &Path) -> ApiResult<()> {
    write(path, updates.to_markdown()).map_err(|why| eyre!("Failed to write Markdown updates — {why}"))
}
