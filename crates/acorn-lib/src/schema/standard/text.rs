//! Plain-text standard model with no schema structure
//!
//! Primary use case is to leverage prose analysis and readability metrics on unstructured text content, such as abstracts or descriptions.

use crate::prelude::alloc::{String, ToString};
use crate::util::{MarkdownSupport, ToProse, Unstructured};
use crate::validation::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Text document sourced from a supported document format or text-like content.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(transparent)]
pub struct Document {
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
impl From<&str> for Text {
    fn from(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }
}
impl From<String> for Text {
    fn from(content: String) -> Self {
        Self { content }
    }
}
impl MarkdownSupport for Document {
    fn to_markdown(&self) -> String {
        self.content().to_string()
    }
}
impl MarkdownSupport for Text {
    fn to_markdown(&self) -> String {
        self.content().to_string()
    }
}
impl ToProse for Document {
    fn to_prose(&self) -> String {
        self.content().to_string()
    }
}
impl ToProse for Text {
    fn to_prose(&self) -> String {
        self.content().to_string()
    }
}
impl Unstructured for Document {
    fn content(&self) -> &str {
        &self.content
    }
}
impl Unstructured for Text {
    fn content(&self) -> &str {
        &self.content
    }
}
