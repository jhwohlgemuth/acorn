//! Plain-text standard model with no schema structure
//!
//! Primary use case is to leverage prose analysis and readability metrics on unstructured text content, such as abstracts or descriptions.

#[cfg(feature = "std")]
use crate::io::{read_file, write_file, ApiResult, InputOutput};
#[cfg(feature = "std")]
use crate::prelude::PathBuf;
#[cfg(feature = "std")]
use crate::util::MimeType;
use crate::util::{ToMarkdown, ToProse};
#[cfg(feature = "std")]
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Text document with unconstrained content
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(transparent)]
pub struct Text {
    /// Raw text content
    pub content: String,
}
impl ToMarkdown for Text {
    fn to_markdown(&self) -> String {
        self.content.clone()
    }
}
impl ToProse for Text {
    fn to_prose(&self) -> String {
        self.content.clone()
    }
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
impl Text {
    fn read_text(path: impl Into<PathBuf>) -> ApiResult<Self> {
        read_file(path.into()).map(|content| Self { content })
    }
    fn write_text(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        write_file(path, self.content.clone())
    }
}
