//! Document conversion utilities.
use crate::io::http::get;
use crate::io::{read_file, uri_to_path, ApiResult};
use crate::prelude::PathBuf;
use crate::schema::pid::Identifier;
use crate::util::MimeType;
use crate::{Location, Scheme};
use anydoc::Format;
use bon::Builder;
use color_eyre::eyre::eyre;
use derive_more::Display;
use serde::{Deserialize, Serialize};

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
impl From<Format> for DocumentFormat {
    fn from(value: Format) -> Self {
        match value {
            | Format::Csv => Self::Csv,
            | Format::Doc => Self::Doc,
            | Format::Docx => Self::Docx,
            | Format::Epub => Self::Epub,
            | Format::Excel => Self::Excel,
            | Format::Odp => Self::Odp,
            | Format::Ods => Self::Ods,
            | Format::Odt => Self::Odt,
            | Format::Pdf => Self::Pdf,
            | Format::Ppt => Self::Ppt,
            | Format::Pptx => Self::Pptx,
            | Format::Rtf => Self::Rtf,
        }
    }
}
impl From<DocumentFormat> for MimeType {
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
            | DocumentFormat::Pptx => Self::Powerpoint,
            | DocumentFormat::Rtf => Self::Rtf,
        }
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
/// Convert a supported document file to GitHub-Flavored Markdown.
pub fn to_markdown(path: impl Into<PathBuf>) -> ApiResult<String> {
    let path = path.into();
    anydoc::to_markdown(&path).map_err(|why| eyre!("Failed to convert document {} - {why}", path.display()))
}
/// Convert supported document bytes to GitHub-Flavored Markdown.
pub fn to_markdown_bytes(bytes: &[u8], format: Option<DocumentFormat>) -> ApiResult<String> {
    anydoc::to_markdown_bytes(bytes, format.map(Format::from)).map_err(|why| eyre!("Failed to convert document - {why}"))
}
impl SourceDocument {
    /// Create an unloaded document source at a local location.
    pub fn at(location: impl Into<PathBuf>) -> Self {
        let source = location.into().display().to_string();
        let format = MimeType::from(source.as_str()).file_type();
        Self {
            content: String::new(),
            format,
            source,
        }
    }
    /// Load a local path, remote URI, or persistent identifier.
    pub async fn load(value: impl Into<Location>, offline: bool) -> ApiResult<Self> {
        let location = value.into();
        let value: &str = (&location).into();
        let persistent = !Identifier::find_all(value).is_empty();
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
        let mime = MimeType::from(source);
        match DocumentFormat::try_from(&mime) {
            | Ok(format) => to_markdown_bytes(bytes, Some(format)),
            | Err(_) => String::from_utf8(bytes.to_vec()).map_err(color_eyre::Report::from),
        }
        .map(|content| Self::init().content(content).format(mime.file_type()).source(source).build())
    }
    /// Read a local document, converting supported formats to Markdown.
    pub fn from_path(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let path = path.into();
        let source = path.display().to_string();
        let mime = MimeType::from(source.as_str());
        let content = match DocumentFormat::try_from(&mime) {
            | Ok(_) => to_markdown(path),
            | Err(_) => read_file(path),
        };
        content.map(|content| Self::init().content(content).format(mime.file_type()).source(source).build())
    }
    /// Read and convert this document to GitHub-Flavored Markdown.
    pub fn extract(&self) -> ApiResult<String> {
        match self.content.is_empty() {
            | true => Self::from_path(self.source.as_str()).map(|document| document.content),
            | false => Ok(self.content.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_anydoc_format_round_trip() {
        let formats = [
            DocumentFormat::Csv,
            DocumentFormat::Doc,
            DocumentFormat::Docx,
            DocumentFormat::Epub,
            DocumentFormat::Excel,
            DocumentFormat::Odp,
            DocumentFormat::Ods,
            DocumentFormat::Odt,
            DocumentFormat::Pdf,
            DocumentFormat::Ppt,
            DocumentFormat::Pptx,
            DocumentFormat::Rtf,
        ];
        formats
            .into_iter()
            .for_each(|format| assert_eq!(DocumentFormat::from(Format::from(format)), format));
    }
    #[test]
    fn test_document_format_mime_round_trip() {
        let formats = [
            DocumentFormat::Csv,
            DocumentFormat::Doc,
            DocumentFormat::Docx,
            DocumentFormat::Epub,
            DocumentFormat::Excel,
            DocumentFormat::Odp,
            DocumentFormat::Ods,
            DocumentFormat::Odt,
            DocumentFormat::Pdf,
            DocumentFormat::Ppt,
            DocumentFormat::Pptx,
            DocumentFormat::Rtf,
        ];
        formats.into_iter().for_each(|format| {
            let mime = MimeType::from(format);
            assert_eq!(DocumentFormat::try_from(&mime).ok(), Some(format));
        });
    }
    #[test]
    fn test_docx_converts_to_markdown() {
        let converted = to_markdown("../../tests/fixtures/acorn.docx");
        assert!(converted.is_ok());
        assert!(!converted.unwrap_or_default().is_empty());
    }
    #[test]
    fn test_source_document_extracts_docx() {
        let converted = SourceDocument::at("../../tests/fixtures/acorn.docx").extract();
        assert!(converted.is_ok());
        assert!(!converted.unwrap_or_default().is_empty());
    }
}
