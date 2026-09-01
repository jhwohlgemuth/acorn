//! Document conversion utilities.
use crate::io::cache::Cache;
use crate::io::http::get;
use crate::io::{read_file, uri_to_path, ApiResult};
use crate::prelude::{fs, read, OnceLock, Path, PathBuf};
use crate::schema::pid::{Identifier, PID};
use crate::util::constants::app::{DOCUMENT_CACHE_LIMIT, MAX_DOCUMENT_BYTES, MAX_PDF_PAGES};
use crate::util::{MimeType, StringConversion};
use crate::{Location, Scheme};
use anydoc::Format;
use bon::Builder;
use color_eyre::eyre::{eyre, Report};
use core::fmt;
use derive_more::Display;
use jiff::Timestamp;
use pdf_inspector::{extract_pages_markdown_mem, PdfError};
use serde::{Deserialize, Serialize};

mod index;
mod matching;
mod research_activity;

pub(crate) use index::{DocumentEntry, DocumentParser};
pub use index::{DocumentExcerpt, DocumentIndex, DocumentMatch, DocumentPath, DocumentPathSegment, DocumentPosition, DocumentQuery, DocumentSpan};
pub(crate) use research_activity::MarkdownParser;

static DOCUMENT_CACHE: OnceLock<Cache<DocumentCacheKey, SourceDocument>> = OnceLock::new();

/// Document formats supported by Anydoc.
#[derive(Clone, Copy, Debug, Deserialize, Display, Eq, PartialEq, Serialize)]
pub enum DocumentFormat {
    /// Comma-separated values.
    #[display("csv")]
    Csv,
    /// Binary Word document.
    #[display("doc")]
    Doc,
    /// OOXML Word document.
    #[display("docx")]
    Docx,
    /// EPUB publication.
    #[display("epub")]
    Epub,
    /// Excel workbook.
    #[display("excel")]
    Excel,
    /// OpenDocument presentation.
    #[display("odp")]
    Odp,
    /// OpenDocument spreadsheet.
    #[display("ods")]
    Ods,
    /// OpenDocument text.
    #[display("odt")]
    Odt,
    /// Portable Document Format.
    #[display("pdf")]
    Pdf,
    /// Binary PowerPoint presentation.
    #[display("ppt")]
    Ppt,
    /// OOXML PowerPoint presentation.
    #[display("pptx")]
    Pptx,
    /// Rich Text Format.
    #[display("rtf")]
    Rtf,
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct DocumentCacheKey {
    length: u64,
    modified: Option<Timestamp>,
    path: PathBuf,
}
#[derive(Debug)]
struct PdfDocumentError(String);
#[derive(Clone, Debug)]
/// Normalized text and one-based page number for an extracted PDF page.
pub struct DocumentPage {
    content: String,
    number: usize,
}
/// Loaded document content and its source metadata.
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct SourceDocument {
    /// Extracted or plain-text document content.
    pub content: String,
    /// Source file type.
    pub format: String,
    /// Original path, URI, persistent identifier, or input label.
    pub source: String,
    #[builder(default)]
    pages: Vec<DocumentPage>,
    #[builder(default)]
    warnings: Vec<String>,
}
/// Ordered collection of loaded source documents.
#[derive(Clone, Debug, Default)]
pub struct SourceDocuments(
    /// Documents in source order.
    pub Vec<SourceDocument>,
);
impl From<Format> for DocumentFormat {
    fn from(value: Format) -> Self {
        #[allow(clippy::expect_used)]
        Self::try_from(&MimeType::from(value)).expect("anydoc formats always map to document formats")
    }
}
impl From<Vec<SourceDocument>> for SourceDocuments {
    fn from(value: Vec<SourceDocument>) -> Self {
        Self(value)
    }
}
impl From<SourceDocuments> for Vec<SourceDocument> {
    fn from(value: SourceDocuments) -> Self {
        value.0
    }
}
impl From<&Path> for DocumentCacheKey {
    fn from(path: &Path) -> Self {
        let metadata = fs::metadata(path).ok();
        Self {
            length: metadata.as_ref().map_or(0, fs::Metadata::len),
            modified: metadata
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|modified| Timestamp::try_from(modified).ok()),
            path: path.to_path_buf(),
        }
    }
}
impl From<PdfError> for PdfDocumentError {
    fn from(why: PdfError) -> Self {
        Self(match why {
            | PdfError::Encrypted => "PDF is encrypted; remove password protection before checking it".to_string(),
            | PdfError::InvalidStructure => "PDF has an invalid structure".to_string(),
            | PdfError::Io(why) => format!("Failed to read PDF - {why}"),
            | PdfError::NotAPdf(why) => format!("Invalid PDF - {why}"),
            | PdfError::Parse(why) => format!("Failed to parse PDF - {why}"),
        })
    }
}
impl TryFrom<&MimeType> for DocumentFormat {
    type Error = color_eyre::Report;
    fn try_from(value: &MimeType) -> Result<Self, Self::Error> {
        match value {
            | MimeType::Csv => Ok(Self::Csv),
            | MimeType::Doc => Ok(Self::Doc),
            | MimeType::Docx => Ok(Self::Docx),
            | MimeType::Epub => Ok(Self::Epub),
            | MimeType::Excel => Ok(Self::Excel),
            | MimeType::Odp => Ok(Self::Odp),
            | MimeType::Ods => Ok(Self::Ods),
            | MimeType::Odt => Ok(Self::Odt),
            | MimeType::Pdf => Ok(Self::Pdf),
            | MimeType::Ppt => Ok(Self::Ppt),
            | MimeType::Powerpoint => Ok(Self::Pptx),
            | MimeType::Rtf => Ok(Self::Rtf),
            | _ => Err(eyre!("Unsupported Anydoc MIME type: {value}")),
        }
    }
}
impl From<DocumentFormat> for Format {
    fn from(value: DocumentFormat) -> Self {
        match value {
            | DocumentFormat::Csv => Self::Csv,
            | DocumentFormat::Doc => Self::Doc,
            | DocumentFormat::Docx => Self::Docx,
            | DocumentFormat::Epub => Self::Epub,
            | DocumentFormat::Excel => Self::Excel,
            | DocumentFormat::Odp => Self::Odp,
            | DocumentFormat::Ods => Self::Ods,
            | DocumentFormat::Odt => Self::Odt,
            | DocumentFormat::Pdf => Self::Pdf,
            | DocumentFormat::Ppt => Self::Ppt,
            | DocumentFormat::Pptx => Self::Pptx,
            | DocumentFormat::Rtf => Self::Rtf,
        }
    }
}
impl From<DocumentFormat> for MimeType {
    fn from(value: DocumentFormat) -> Self {
        Self::from(Format::from(value))
    }
}
impl fmt::Display for PdfDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl core::error::Error for PdfDocumentError {}
impl SourceDocument {
    pub(crate) fn is_markdown(&self) -> bool {
        MimeType::from(self.source.as_str()) == MimeType::Markdown || MimeType::from(self.format.as_str()) == MimeType::Markdown
    }
    fn is_yaml(&self) -> bool {
        let mime = MimeType::from(self.source.as_str());
        mime.is_yaml() || mime.is_cff()
    }
    /// Create an unloaded document source at a local location.
    pub fn at(location: impl Into<PathBuf>) -> Self {
        let source = location.into().display().to_string();
        let format = MimeType::from(source.as_str()).file_type();
        Self {
            content: String::new(),
            format,
            source,
            pages: Vec::new(),
            warnings: Vec::new(),
        }
    }
    /// Load a local path, remote URI, or persistent identifier.
    pub async fn load(value: impl Into<Location>, offline: bool) -> ApiResult<Self> {
        let location = value.into();
        let value: &str = (&location).into();
        let persistent = Identifier::find_all(value).into_iter().any(|identifier| identifier.kind != PID::URL);
        let scheme = location.scheme();
        let source = location.uri().unwrap_or_default();
        match (persistent, scheme, offline) {
            | (true, _, _) => Ok(Self::init().content(value).format("pid").source(value).build()),
            | (false, Scheme::HTTP | Scheme::HTTPS, true) => Err(eyre!("Remote input is unavailable in offline mode: {value}")),
            | (false, Scheme::HTTP | Scheme::HTTPS, false) => get(source.clone())
                .send()
                .await
                .map(|response| response.body)
                .and_then(|bytes| Self::from_bytes(&bytes, &source)),
            | (false, Scheme::File | Scheme::Unsupported, _) => Self::from_path(uri_to_path(&source)),
        }
    }
    /// Convert document bytes to Markdown when supported, otherwise decode them as UTF-8.
    pub fn from_bytes(bytes: &[u8], source: &str) -> ApiResult<Self> {
        let mime = MimeType::infer(bytes).unwrap_or_else(|| MimeType::from(source));
        match DocumentFormat::try_from(&mime) {
            | Ok(DocumentFormat::Pdf) => Self::pdf(bytes, source),
            | Ok(format) => validate_document_size(bytes.len(), source)
                .and_then(|()| to_markdown_bytes(bytes, Some(format)))
                .map(|content| Self::init().content(content).format(mime.file_type()).source(source).build()),
            | Err(_) => String::from_utf8(bytes.to_vec())
                .map_err(color_eyre::Report::from)
                .map(|content| Self::init().content(content).format(mime.file_type()).source(source).build()),
        }
    }
    /// Read a local document, converting supported formats to Markdown.
    pub fn from_path(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let path = path.into();
        let source = path.display().to_string();
        let mime = MimeType::from(source.as_str());
        match DocumentFormat::try_from(&mime) {
            | Ok(_) => {
                let key = DocumentCacheKey::from(path.as_path());
                validate_document_size(key.length.try_into().unwrap_or(usize::MAX), &source).and_then(|()| {
                    match DOCUMENT_CACHE.get_or_init(|| Cache::new(DOCUMENT_CACHE_LIMIT)).get(&key) {
                        | Some(document) => Ok(document),
                        | None => read(&path)
                            .map_err(|why| eyre!("Failed to read document {} - {why}", path.display()))
                            .and_then(|bytes| Self::from_bytes(&bytes, &source))
                            .map(|document| DOCUMENT_CACHE.get_or_init(|| Cache::new(DOCUMENT_CACHE_LIMIT)).insert(key, document)),
                    }
                })
            }
            | Err(_) => read_file(path).map(|content| Self::init().content(content).format(mime.file_type()).source(source).build()),
        }
    }
    /// Read and convert this document to GitHub-Flavored Markdown.
    pub fn extract(&self) -> ApiResult<String> {
        match self.content.is_empty() {
            | true => Self::from_path(self.source.as_str()).map(|document| document.content),
            | false => Ok(self.content.clone()),
        }
    }
    /// Return the one-based PDF page containing an extracted byte offset.
    pub fn page_number(&self, byte: usize) -> Option<usize> {
        let start = self
            .content
            .get(..byte)
            .and_then(|content| content.rfind('\n').map(|index| index.saturating_add(1)))
            .unwrap_or_default();
        let end = self
            .content
            .get(byte..)
            .and_then(|content| content.find('\n').map(|index| byte.saturating_add(index)))
            .unwrap_or(self.content.len());
        self.content
            .get(start..end)
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .and_then(|line| {
                let normalized = line.to_ascii_alphanumeric();
                let mut matches = self
                    .pages
                    .iter()
                    .filter(|page| page.content.contains(&normalized))
                    .map(|page| page.number);
                matches.next().filter(|_| matches.next().is_none())
            })
    }
    fn pdf(bytes: &[u8], source: &str) -> ApiResult<Self> {
        validate_document_size(bytes.len(), source)
            .and_then(|()| {
                pdf_inspector::classify_pdf_mem(bytes)
                    .map_err(PdfDocumentError::from)
                    .map_err(Report::from)
            })
            .and_then(|classification| validate_pdf_pages(classification.page_count, source))
            .and_then(|()| {
                extract_pages_markdown_mem(bytes, None)
                    .map_err(PdfDocumentError::from)
                    .map_err(Report::from)
            })
            .and_then(|extraction| {
                let pages = extraction
                    .pages
                    .into_iter()
                    .filter(|page| !page.markdown.trim().is_empty())
                    .map(|page| DocumentPage {
                        content: page.markdown.to_ascii_alphanumeric(),
                        number: page.page.saturating_add(1) as usize,
                    })
                    .collect();
                let warnings = match extraction.pages_needing_ocr.is_empty() {
                    | true => Vec::new(),
                    | false => vec![format!(
                        "PDF pages {} require OCR and were skipped; OCR is not supported",
                        extraction.pages_needing_ocr.iter().map(u32::to_string).collect::<Vec<_>>().join(", ")
                    )],
                };
                anydoc::to_markdown_bytes(bytes, Some(Format::Pdf))
                    .map_err(|why| eyre!("Failed to extract PDF '{source}' - {why}"))
                    .and_then(|content| match content.trim().is_empty() {
                        | true => Err(eyre!("PDF '{source}' has no extractable text; OCR is required and is not supported")),
                        | false => Ok(Self::init()
                            .content(content)
                            .format(MimeType::Pdf.file_type())
                            .source(source)
                            .pages(pages)
                            .warnings(warnings)
                            .build()),
                    })
            })
    }
    pub(crate) fn has_pages(&self) -> bool {
        !self.pages.is_empty()
    }
    /// Return non-fatal document extraction warnings.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
}
/// Convert a supported document file to GitHub-Flavored Markdown.
pub fn to_markdown(path: impl Into<PathBuf>) -> ApiResult<String> {
    let path = path.into();
    read(&path)
        .map_err(|why| eyre!("Failed to read document {} - {why}", path.display()))
        .and_then(|bytes| {
            let format = Format::from_bytes(&bytes).or_else(|| Format::from_path(&path)).map(DocumentFormat::from);
            to_markdown_bytes(&bytes, format).map_err(|why| eyre!("Failed to convert document {} - {why}", path.display()))
        })
}
/// Convert supported document bytes to GitHub-Flavored Markdown.
pub fn to_markdown_bytes(bytes: &[u8], format: Option<DocumentFormat>) -> ApiResult<String> {
    validate_document_size(bytes.len(), "in-memory document").and_then(|()| match format {
        | Some(DocumentFormat::Pdf) => SourceDocument::pdf(bytes, "in-memory.pdf").map(|document| document.content),
        | format => anydoc::to_markdown_bytes(bytes, format.map(Format::from)).map_err(|why| eyre!("Failed to convert document - {why}")),
    })
}
fn validate_document_size(size: usize, source: &str) -> ApiResult<()> {
    match size <= MAX_DOCUMENT_BYTES {
        | true => Ok(()),
        | false => Err(eyre!(
            "Document '{source}' is too large to check safely: {size} bytes exceeds the {MAX_DOCUMENT_BYTES}-byte limit"
        )),
    }
}
fn validate_pdf_pages(page_count: u32, source: &str) -> ApiResult<()> {
    match page_count <= MAX_PDF_PAGES {
        | true => Ok(()),
        | false => Err(eyre!(
            "PDF '{source}' has too many pages to check safely: {page_count} exceeds the {MAX_PDF_PAGES}-page limit"
        )),
    }
}

#[cfg(test)]
mod tests;
