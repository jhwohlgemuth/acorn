//! Plain-text standard model with no schema structure
//!
//! Primary use case is to leverage prose analysis and readability metrics on unstructured text content, such as abstracts or descriptions.

#[cfg(feature = "std")]
use crate::io::docx::extract_text_from_path;
#[cfg(feature = "std")]
use crate::io::{read_file, write_file, ApiResult, InputOutput};
#[cfg(feature = "std")]
use crate::prelude::PathBuf;
use crate::prelude::*;
#[cfg(feature = "std")]
use crate::util::file_extension;
#[cfg(feature = "std")]
use crate::util::MimeType;
use crate::util::{ToMarkdown, ToProse, Unstructured};
#[cfg(feature = "std")]
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Text document sourced from DOCX or text-like content.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(transparent)]
pub struct Docx {
    /// Raw extracted text content.
    pub content: String,
}
/// Text document with unconstrained content
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(transparent)]
pub struct Text {
    /// Raw text content
    pub content: String,
}
#[cfg(feature = "std")]
impl InputOutput for Text {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into();
        match MimeType::from(source.display().to_string()) {
            | MimeType::Markdown | MimeType::Text => Self::read_text(source),
            | MimeType::Json => Self::read_json(source),
            | MimeType::Yaml => Self::read_yaml(source),
            | _ => Err(eyre!("Unsupported plain text file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        Self::read_text(path)
    }
    fn read_markdown(path: PathBuf) -> Option<Self> {
        Self::read_text(path).ok()
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        Self::read_text(path)
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match MimeType::from(output.display().to_string()) {
            | MimeType::Markdown | MimeType::Text => self.write_text(output),
            | MimeType::Json => self.write_json(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported plain text file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_text(path)
    }
    fn write_markdown(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_text(path)
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_text(path)
    }
}
#[cfg(feature = "std")]
impl InputOutput for Docx {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into();
        match file_extension(source.display().to_string()).as_deref() {
            | Some("docx") => Self::read_docx(source),
            | Some("md") | Some("markdown") | Some("txt") | Some("json") | Some("yml") | Some("yaml") => Self::read_text(source),
            | _ => Err(eyre!("Unsupported DOCX file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        Self::read_text(path)
    }
    fn read_markdown(path: PathBuf) -> Option<Self> {
        Self::read_text(path).ok()
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        Self::read_text(path)
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match file_extension(output.display().to_string()).as_deref() {
            | Some("md") | Some("markdown") | Some("txt") | Some("json") | Some("yml") | Some("yaml") => self.write_text(output),
            | Some("docx") => Err(eyre!("DOCX writing is not implemented")),
            | _ => Err(eyre!("Unsupported DOCX file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_text(path)
    }
    fn write_markdown(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_text(path)
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_text(path)
    }
}
impl ToMarkdown for Docx {
    fn to_markdown(&self) -> String {
        self.content().to_string()
    }
}
impl ToMarkdown for Text {
    fn to_markdown(&self) -> String {
        self.content().to_string()
    }
}
impl ToProse for Docx {
    fn to_prose(&self) -> String {
        self.content().to_string()
    }
}
impl ToProse for Text {
    fn to_prose(&self) -> String {
        self.content().to_string()
    }
}
impl Unstructured for Docx {
    fn content(&self) -> &str {
        &self.content
    }
}
impl Unstructured for Text {
    fn content(&self) -> &str {
        &self.content
    }
}
#[cfg(feature = "std")]
impl Docx {
    fn read_docx(path: impl Into<PathBuf>) -> ApiResult<Self> {
        extract_text_from_path(path.into()).map(|content| Self { content })
    }
    fn read_text(path: impl Into<PathBuf>) -> ApiResult<Self> {
        read_file(path.into()).map(|content| Self { content })
    }
    fn write_text(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        write_file(path, self.content().to_string())
    }
}
#[cfg(feature = "std")]
impl Text {
    fn read_text(path: impl Into<PathBuf>) -> ApiResult<Self> {
        read_file(path.into()).map(|content| Self { content })
    }
    fn write_text(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        write_file(path, self.content().to_string())
    }
}
