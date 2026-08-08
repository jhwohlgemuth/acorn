//! # Common utilities
//!
//! This module contains common functions and data structures used to build the ACORN command line interface as well as support open science endeavors.
//!
//! ## Example Uses
//! ### Work with semantic versions
//! ```ignore
//! use acorn::util::SemanticVersion;
//!
//! let version = SemanticVersion::from_string("1.2.3");
//! assert_eq!(version.minor, 2);
//!
//! if let Some(version) = SemanticVersion::from_command("cargo") {
//!     println!("cargo version: {version}");
//! }
//! ```
//!
use crate::fail;
use crate::prelude::HashMap;
#[cfg(feature = "std")]
use crate::prelude::Path;
use crate::prelude::*;
use aho_corasick::AhoCorasick;
use bon::Builder;
use convert_case::{Case, Casing};
use core::fmt;
use core::iter::successors;
use derive_more::Display;
use fancy_regex::Regex;
use fluent_uri::UriRef;
use jiff::{tz::Offset, Timestamp};
use jsonc_parser::{parse_to_serde_value, ParseOptions};
#[cfg(feature = "std")]
use nanoid::nanoid;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;
pub mod assets;
#[cfg(feature = "std")]
pub mod cmd;
pub mod constants;
pub mod macros;
pub mod portable;
pub mod terminal;
pub use assets::Constant;
use constants::{CROCKFORD_BASE32_ALPHABET, LINE_SEPARATOR};
pub use terminal::Label;
#[cfg(feature = "std")]
pub use terminal::{
    print_changes, print_changes_with_color, print_values_as_table, text_diff_changes, text_diff_changes_with_color, values_as_table,
};

/// Format a timestamp as RFC 3339 with an explicit UTC offset.
pub fn to_rfc3339(value: Timestamp) -> String {
    value.display_with_offset(Offset::UTC).to_string()
}
/// Trait for augmenting data with linked data context
pub trait LinkedData {
    /// Add linked data (e.g., JSON-LD) context
    fn with_context(&self) -> Self;
}
/// Helper trait for searching lists of named elements
pub trait Searchable<T> {
    /// Check if a certain value is present in the list
    /// ### Note
    /// This method will differ greatly on the implementation and type of T
    fn contains(&self, _value: &str) -> bool {
        false
    }
    /// Filter list by ISO or ISO3 and return the first match
    /// ### Note
    /// This method is specific to the `Country` type in the GeoNames API, but is included in the trait for convenience and consistency with `find_by_name`
    fn find_by_iso(&self, _value: impl Into<String>) -> Option<T> {
        None
    }
    /// Filter list by name and return the first match
    fn find_by_name(&self, value: impl Into<String>) -> Option<T>;
}
/// Trait for augmenting and formatting data for display
pub trait StringConversion {
    /// Render a path string using native separators without a Windows extended-length prefix
    fn to_cross_platform_path(&self) -> String;
    /// Return a string representation of the file_name with its parent folder (or just the folder name if it is a folder)
    fn file_name_with_parent(&self) -> String;
    /// Return a string representation of the absolute path
    fn to_absolute_path(&self) -> String;
}
/// Add enhanced string interpolation functionality
pub trait StringInterpolation<T>
where
    T: AsRef<str> + ToString,
{
    /// Replace placeholder instances with a given value (basic interpolation based on handlebars template syntax)
    fn replace_placeholder_with_string(&self, placeholder: &str, value: &str) -> String;
    /// Prepend indentation of a given number of spaces to each line of a text
    fn with_indent(&self, spaces: usize) -> String;
}
/// Format data structures as Markdown
pub trait ToMarkdown {
    /// Convert `self` to Markdown format string
    fn to_markdown(&self) -> String;
}
/// Format data structures as prose suitable for static analysis
pub trait ToProse {
    /// Convert `self` to prose format string
    fn to_prose(&self) -> String;
}
/// Trait for converting a vector of non-string values to a vector of strings
pub trait ToStrings {
    /// Convert a vector of string slices to a vector of string values of paths
    ///
    /// This is a convenience that I find myself wanting to use in a lot of places.
    ///
    /// Adding a `to_strings` method to the `Vec<PathBuf>` types seems like a good idea.
    /// ### Example
    /// ```ignore
    /// use acorn::util::ToStrings;
    ///
    /// let paths = vec![PathBuf::from("foo"), PathBuf::from("bar"), PathBuf::from("baz")];
    /// assert!(paths.to_strings().contains(&"foo".to_string()));
    /// ```
    fn to_strings(&self) -> Vec<String>;
    /// Convert a vector of string slices to a vector of string values of absolute paths
    fn to_absolute_strings(&self) -> Vec<String> {
        vec![]
    }
}
/// Trait for adding chunking functionality
pub trait ToStringChunks<T>
where
    T: AsRef<str> + ToString,
{
    /// Chunk a string into substrings of a given size
    fn chunk(&self, size: usize) -> Vec<String>;
}
/// Expose raw text content from structures with a content field
pub trait Unstructured {
    /// Return raw content as a string slice
    fn content(&self) -> &str;
}
/// Cryptographic hash algorithm used across ACORN metadata and packaging.
///
/// This enum is shared by DCAT checksum metadata and BagIt manifest workflows.
#[derive(Clone, Debug, Default, Display, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    /// SHA-256 secure hash algorithm (default)
    #[default]
    #[display("sha256")]
    #[serde(rename = "SHA256", alias = "sha256")]
    Sha256,
    /// MD2 message-digest algorithm
    #[display("md2")]
    #[serde(rename = "MD2", alias = "md2")]
    Md2,
    /// MD4 message-digest algorithm
    #[display("md4")]
    #[serde(rename = "MD4", alias = "md4")]
    Md4,
    /// MD5 message-digest algorithm
    #[display("md5")]
    #[serde(rename = "MD5", alias = "md5")]
    Md5,
    /// MD6 message-digest algorithm
    #[display("md6")]
    #[serde(rename = "MD6", alias = "md6")]
    Md6,
    /// SHA-1 secure hash algorithm
    #[display("sha1")]
    #[serde(rename = "SHA1", alias = "sha1")]
    Sha1,
    /// SHA-224 secure hash algorithm
    #[display("sha224")]
    #[serde(rename = "SHA224", alias = "sha224")]
    Sha224,
    /// SHA-384 secure hash algorithm
    #[display("sha384")]
    #[serde(rename = "SHA384", alias = "sha384")]
    Sha384,
    /// SHA-512 secure hash algorithm
    #[display("sha512")]
    #[serde(rename = "SHA512", alias = "sha512")]
    Sha512,
}
/// SPDX compliant license identifier
///
/// See <https://spdx.org/licenses/> for more information
#[derive(Clone, Debug, Display, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum License {
    /// GNU Affero General Public License v3.0 only
    #[display("AGPL-3.0-only")]
    #[serde(alias = "AGPL-3.0-only")]
    Agpl3Only,
    /// Apache License 2.0
    #[display("Apache-2.0")]
    #[serde(alias = "Apache-2.0")]
    Apache2,
    /// BSD 3-Clause "New" or "Revised" License
    #[display("BSD-3-Clause")]
    #[serde(alias = "BSD-3-Clause")]
    Bsd3Clause,
    /// Creative Commons Zero v1.0 Universal
    #[display("CC0-1.0")]
    #[serde(alias = "CC0-1.0", alias = "Creative Commons CC-0")]
    CreativeCommons,
    /// GNU General Public License v2.0 only
    #[display("GPL-2.0-only")]
    #[serde(alias = "GPL-2.0-only")]
    Gpl2Only,
    /// GNU General Public License v2.0 with Classpath exception
    #[display("GPL-2.0-with-classpath-exception")]
    #[serde(alias = "GPL-2.0-with-classpath-exception")]
    Gpl2WithClasspathException,
    /// GNU General Public License v3.0 only
    #[display("GPL-3.0-only")]
    #[serde(alias = "GPL-3.0-only")]
    Gpl3Only,
    /// GNU General Public License v3.0 or later
    #[display("GPL-3.0-or-later")]
    #[serde(alias = "GPL-3.0-or-later")]
    Gpl3OrLater,
    /// GNU Lesser General Public License v2.1 only
    #[display("LGPL-2.1-only")]
    #[serde(alias = "LGPL-2.1-only")]
    Lgpl21Only,
    /// LaTeX Project Public License v1.3c
    #[display("LPPL-1.3c")]
    #[serde(alias = "LPPL-1.3c")]
    Lppl13c,
    /// MIT License
    #[display("MIT")]
    #[serde(alias = "MIT")]
    Mit,
    /// PostgreSQL License
    #[display("PostgreSQL")]
    #[serde(alias = "PostgreSQL")]
    PostgreSql,
    /// Custom license reference for proprietary software
    #[display("Proprietary")]
    #[serde(alias = "LicenseRef-Proprietary")]
    Proprietary,
    /// Python Software Foundation License (based on PSF)
    #[display("PSF-based")]
    #[serde(alias = "PSF-based")]
    PsfBased,
    /// Python Software Foundation License 2.0
    #[display("PSF-2.0")]
    #[serde(alias = "PSF-2.0")]
    Psf2,
    /// Public domain (i.e., no license)
    #[display("Public Domain")]
    #[serde(alias = "Public Domain")]
    PublicDomain,
    /// Unknown license
    #[display("Unknown")]
    Unknown,
    /// Various licenses (mixed or unspecified)
    #[display("Various")]
    #[serde(alias = "Various")]
    Various,
    /// World Wide Web Consortium License
    #[display("W3C")]
    #[serde(alias = "W3C")]
    W3C,
}
/// Supports an incomplete list of common <span title="Multipurpose Internet Mail Extension">MIME</span> types
///
/// See listing of [common HTTP MIME types](https://developer.mozilla.org/en-US/docs/Web/HTTP/MIME_types/Common_types) and <https://mimetype.io/all-types> for more information
#[derive(Clone, Debug, Display, PartialEq)]
pub enum MimeType {
    /// Citation File Format (CFF)
    /// ### Note
    /// > CFF does not have a standard MIME type, but is valid YAML
    ///
    /// See <https://citation-file-format.github.io/> for more information
    #[display("application/yaml")]
    Cff,
    /// Comma Separated Values (CSV)
    #[display("text/csv")]
    Csv,
    /// Binary Microsoft Word document.
    #[display("application/msword")]
    Doc,
    /// OOXML Microsoft Word document.
    #[display("application/vnd.openxmlformats-officedocument.wordprocessingml.document")]
    Docx,
    /// EPUB publication.
    #[display("application/epub+zip")]
    Epub,
    /// Microsoft Excel workbook.
    #[display("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")]
    Excel,
    /// GPT-Generated Unified Format (GGUF) model
    ///
    /// See <https://github.com/ggml-org/ggml/blob/master/docs/gguf.md> for more information
    #[display("application/vnd.gguf.model")]
    Gguf,
    /// Linked Data [JSON](https://www.json.org/json-en.html)
    ///
    /// See <https://json-ld.org/>
    #[display("application/ld+json")]
    LdJson,
    /// Joint Photographic Experts Group (JPEG)
    #[display("image/jpeg")]
    Jpeg,
    /// JavaScript Object Notation (JSON)
    ///
    /// See <https://www.json.org/json-en.html>
    #[display("application/json")]
    Json,
    /// JSON with Comments (JSONC)
    ///
    /// See <https://code.visualstudio.com/docs/languages/json#_json-with-comments>
    #[display("application/jsonc")]
    Jsonc,
    /// Markdown
    #[display("text/markdown")]
    Markdown,
    /// Model card
    #[display("application/vnd.ai.modelcard.v1+json")]
    ModelCard,
    /// ONNX (Open Neural Network Exchange) model
    ///
    /// See <https://onnx.ai/> for more information
    #[display("application/vnd.onnx.model")]
    Onnx,
    /// OpenDocument presentation.
    #[display("application/vnd.oasis.opendocument.presentation")]
    Odp,
    /// OpenDocument spreadsheet.
    #[display("application/vnd.oasis.opendocument.spreadsheet")]
    Ods,
    /// OpenDocument text.
    #[display("application/vnd.oasis.opendocument.text")]
    Odt,
    /// OpenType Font (OTF)
    #[display("font/otf")]
    Otf,
    /// Parquet format
    ///
    /// See <https://parquet.apache.org> for more information
    #[display("application/x-parquet")]
    Parquet,
    /// Portable Document Format (PDF)
    #[display("application/pdf")]
    Pdf,
    /// Portable Network Graphic (PNG)
    #[display("image/png")]
    Png,
    /// Binary Microsoft PowerPoint presentation.
    #[display("application/vnd.ms-powerpoint")]
    Ppt,
    /// PyTorch model
    ///
    /// Commonly used for `.pt` and `.pth` model files.
    #[display("application/vnd.pytorch.model")]
    Pytorch,
    /// PowerPoint Presentation (modern format)
    ///
    /// See <https://en.wikipedia.org/wiki/Office_Open_XML>
    #[display("application/vnd.openxmlformats-officedocument.presentationml.presentation")]
    Powerpoint,
    /// LLM Prompt template
    ///
    /// This includes .prompt files (see <https://google.github.io/dotprompt/>)
    #[display("application/vnd.ai.prompt.v1+json")]
    Prompt,
    /// Rich Text Format.
    #[display("application/rtf")]
    Rtf,
    /// Rust Source Code (RS)
    #[display("text/rust")]
    Rust,
    /// Safetensors weights
    ///
    /// See <https://github.com/huggingface/safetensors> for more information
    #[display("application/vnd.safetensors")]
    Safetensors,
    /// SBOM (Software Bill of Materials)
    ///
    /// See <https://cyclonedx.org/> for more information
    #[display("application/spdx+json")]
    Sbom,
    /// Scalable Vector Graphic (SVG)
    #[display("image/svg+xml")]
    Svg,
    /// Plain Text
    ///
    /// Just plain old text
    #[display("text/plain")]
    Text,
    /// Tom's Obvious Minimal Language (TOML)
    ///
    /// See <https://toml.io/>
    #[display("application/toml")]
    Toml,
    /// TrueType Font (TTF)
    #[display("font/ttf")]
    Ttf,
    /// YAML Ain't Markup Language (YAML)
    ///
    /// See <https://yaml.org/>
    #[display("application/yaml")]
    Yaml,
    /// ZIP Archive
    ///
    /// See <https://en.wikipedia.org/wiki/ZIP_(file_format)>
    #[display("application/zip")]
    Zip,
    /// Unknown MIME type
    #[display("application/vnd.{}", _0)]
    Vendor(String),
    /// Unknown MIME type
    #[display("application/octet-stream")]
    Unknown(String),
}
/// Cryptographic checksum value paired with its algorithm.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Checksum {
    /// The algorithm used to produce the checksum.
    pub algorithm: ChecksumAlgorithm,
    /// Lowercase hexadecimal digest value.
    #[serde(rename = "checksumValue")]
    pub checksum_value: String,
}
/// Semantic version
///
/// see <https://semver.org/>
///
/// ```rust
/// use acorn::util::SemanticVersion;
///
/// let version = SemanticVersion::from_string("1.2.3");
/// assert_eq!(version.major, 1);
/// assert_eq!(version.to_string(), "1.2.3");
/// ```

#[derive(Builder, Clone, Copy, Debug, Deserialize, Display, Serialize, JsonSchema)]
#[builder(start_fn = init)]
#[display("{}.{}.{}", major, minor, patch)]
pub struct SemanticVersion {
    /// Version when you make incompatible API changes
    #[builder(default = 0)]
    pub major: u32,
    /// Version when you add functionality in a backward compatible manner
    #[builder(default = 0)]
    pub minor: u32,
    /// Version when you make backward compatible bug fixes
    #[builder(default = 0)]
    pub patch: u32,
}
impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.checksum_value)
    }
}
impl From<&str> for ChecksumAlgorithm {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            | "sha256" => ChecksumAlgorithm::Sha256,
            | "md5" => ChecksumAlgorithm::Md5,
            | "sha1" => ChecksumAlgorithm::Sha1,
            | "sha512" => ChecksumAlgorithm::Sha512,
            | "md2" => ChecksumAlgorithm::Md2,
            | "md4" => ChecksumAlgorithm::Md4,
            | "md6" => ChecksumAlgorithm::Md6,
            | "sha224" => ChecksumAlgorithm::Sha224,
            | "sha384" => ChecksumAlgorithm::Sha384,
            | _ => ChecksumAlgorithm::default(),
        }
    }
}
impl From<String> for ChecksumAlgorithm {
    fn from(value: String) -> Self {
        ChecksumAlgorithm::from(value.as_str())
    }
}
impl<T: AsRef<str>> From<T> for License
where
    T: ToString,
{
    /// Convert SPDX standard indentifier to associated `License` value
    /// ### Notes
    /// - Custom license identifiers (i.e., start with `LicenseRef-`) are mapped to `License::Proprietary`
    /// - `"Public Domain"`, which is not a valid SPDX identifier is mapped to `License::PublicDomain`
    /// - `"Unknown"`, which is not a valid SPDX identifier is mapped to `License::Unknown`
    /// - `"Various"`, which is not a valid SPDX identifier is mapped to `License::Various`
    fn from(value: T) -> Self {
        match value.as_ref().to_lowercase().as_str() {
            | "agpl-3.0-only" => License::Agpl3Only,
            | "apache-2.0" => License::Apache2,
            | "bsd-2-clause" | "bsd-3-clause" => License::Bsd3Clause,
            | "cc0-1.0" | "creative commons cc-0" => License::CreativeCommons,
            | "gpl-1.0-or-later" | "gpl-2.0-only" => License::Gpl2Only,
            | "gpl-2.0-with-classpath-exception" => License::Gpl2WithClasspathException,
            | "gpl-3.0-only" => License::Gpl3Only,
            | "gpl-3.0-or-later" => License::Gpl3OrLater,
            | "lgpl-2.1-only" => License::Lgpl21Only,
            | "lppl-1.3c" => License::Lppl13c,
            | "mit" => License::Mit,
            | "postgresql" => License::PostgreSql,
            | "proprietary" | "licenseref-proprietary" => License::Proprietary,
            | "psf-based" => License::PsfBased,
            | "psf-2.0" => License::Psf2,
            | "public-domain" | "public domain" => License::PublicDomain,
            | "various" => License::Various,
            | "w3c" => License::W3C,
            | _ => License::Unknown,
        }
    }
}
impl License {
    #[allow(dead_code)]
    fn from_technology(value: &str) -> Option<License> {
        let data = Constant::csv("technology");
        let result = data
            .into_iter()
            .map(|row| row.into_iter().take(5).collect::<Vec<String>>())
            .find(|pair| pair.first().map(|s| s.as_str()) == Some(value));
        match result {
            | Some(pair) => pair.get(4).map(|s| License::from(s.clone())),
            | None => None,
        }
    }
    #[allow(dead_code)]
    fn is_open_source(&self) -> bool {
        let data = Constant::csv("technology");
        let result = data
            .into_iter()
            .map(|row| row.into_iter().skip(4).take(2).collect::<Vec<String>>())
            .find(|pair| pair.first().map(|s| s.as_str()) == Some(self.to_string().as_str()));
        match result {
            | Some(value) => value.get(1).map(|s| s.as_str()) == Some("true"),
            | None => false,
        }
    }
}
impl From<&str> for MimeType {
    /// Returns a `MimeType` value based on the file extension of the given file name.
    ///
    /// # Supported MIME types
    ///
    /// | File Extension | MIME Type |
    /// | --- | --- |
    /// | cff | application/yaml |
    /// | csv | text/csv |
    /// | jpg | image/jpeg |
    /// | jpeg | image/jpeg |
    /// | json | application/json |
    /// | jsonc | application/jsonc |
    /// | jsonld | application/ld+json |
    /// | md | text/markdown |
    /// | otf | font/otf |
    /// | ttf | font/ttf |
    /// | pdf | application/pdf |
    /// | png | image/png |
    /// | pt | application/vnd.pytorch.model |
    /// | pth | application/vnd.pytorch.model |
    /// | pptx | application/vnd.openxmlformats-officedocument.presentationml.presentation |
    /// | rs | text/rust |
    /// | svg | image/svg+xml |
    /// | toml | application/toml |
    /// | txt | text/plain |
    /// | yaml | application/yaml |
    /// | zip | application/zip |
    fn from(value: &str) -> Self {
        let name = value.to_lowercase();
        match file_extension(name.clone()) {
            | Some(value) => match value.as_str() {
                | "cff" => MimeType::Cff,
                | "csv" => MimeType::Csv,
                | "doc" => MimeType::Doc,
                | "docx" | "docm" => MimeType::Docx,
                | "epub" => MimeType::Epub,
                | "gguf" => MimeType::Gguf,
                | "jpg" | "jpeg" => MimeType::Jpeg,
                | "json" => MimeType::Json,
                | "jsonc" => MimeType::Jsonc,
                | "jsonld" | "json-ld" => MimeType::LdJson,
                | "md" | "markdown" => MimeType::Markdown,
                | "onnx" => MimeType::Onnx,
                | "odp" => MimeType::Odp,
                | "ods" => MimeType::Ods,
                | "odt" => MimeType::Odt,
                | "otf" => MimeType::Otf,
                | "ttf" => MimeType::Ttf,
                | "parquet" => MimeType::Parquet,
                | "pdf" => MimeType::Pdf,
                | "png" => MimeType::Png,
                | "pt" | "pth" => MimeType::Pytorch,
                | "ppt" | "pps" | "pot" => MimeType::Ppt,
                | "pptx" | "pptm" | "ppsx" | "ppsm" => MimeType::Powerpoint,
                | "prompt" => MimeType::Prompt,
                | "rtf" => MimeType::Rtf,
                | "rs" => MimeType::Rust,
                | "safetensors" => MimeType::Safetensors,
                | "spdx.json" => MimeType::Sbom,
                | "svg" => MimeType::Svg,
                | "toml" => MimeType::Toml,
                | "txt" => MimeType::Text,
                | "xls" | "xlsb" | "xlsm" | "xlsx" => MimeType::Excel,
                | "yml" | "yaml" => MimeType::Yaml,
                | value => MimeType::Vendor(value.to_string()),
            },
            | None => MimeType::Unknown(name),
        }
    }
}
impl From<&String> for MimeType {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}
impl From<String> for MimeType {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}
impl MimeType {
    /// Returns the file type as a string
    /// ### Example
    /// ```rust
    /// use acorn::util::MimeType;
    ///
    /// let mime = MimeType::Cff;
    /// assert_eq!(mime.file_type(), "cff");
    /// ```
    pub fn file_type(self) -> String {
        match self {
            | MimeType::Cff => "cff",
            | MimeType::Csv => "csv",
            | MimeType::Doc => "doc",
            | MimeType::Docx => "docx",
            | MimeType::Epub => "epub",
            | MimeType::Excel => "xlsx",
            | MimeType::Gguf => "gguf",
            | MimeType::Jpeg => "jpeg",
            | MimeType::Json => "json",
            | MimeType::Jsonc => "jsonc",
            | MimeType::LdJson => "jsonld",
            | MimeType::Markdown => "md",
            | MimeType::ModelCard => "modelcard",
            | MimeType::Onnx => "onnx",
            | MimeType::Odp => "odp",
            | MimeType::Ods => "ods",
            | MimeType::Odt => "odt",
            | MimeType::Otf => "otf",
            | MimeType::Ttf => "ttf",
            | MimeType::Parquet => "parquet",
            | MimeType::Pdf => "pdf",
            | MimeType::Png => "png",
            | MimeType::Ppt => "ppt",
            | MimeType::Pytorch => "pt",
            | MimeType::Powerpoint => "pptx",
            | MimeType::Prompt => "prompt",
            | MimeType::Rtf => "rtf",
            | MimeType::Rust => "rs",
            | MimeType::Safetensors => "safetensors",
            | MimeType::Sbom => "spdx.json",
            | MimeType::Svg => "svg",
            | MimeType::Text => "txt",
            | MimeType::Toml => "toml",
            | MimeType::Yaml => "yaml",
            | MimeType::Zip => "zip",
            | _ => "unknown-file-type",
        }
        .to_string()
    }
}
impl Default for SemanticVersion {
    fn default() -> Self {
        SemanticVersion::init().build()
    }
}
impl From<&str> for SemanticVersion {
    /// Parses a string into a `SemanticVersion` value
    ///
    /// ### Example
    /// ```rust
    /// use acorn::util::SemanticVersion;
    ///
    /// let version = SemanticVersion::from("1.2.3");
    /// assert_eq!(version.minor, 2);
    /// ```
    fn from(value: &str) -> Self {
        let token = value
            .split(|c: char| !(c.is_ascii_digit() || c == '.'))
            .find(|x: &&str| x.chars().any(|c: char| c.is_ascii_digit()))
            .unwrap_or("");
        let parts = token
            .split('.')
            .filter(|x: &&str| !x.is_empty())
            .map(|x: &str| x.parse::<u32>())
            .collect::<Vec<_>>();
        match parts.as_slice() {
            | [Ok(major), Ok(minor), Ok(patch)] => SemanticVersion::init().major(*major).minor(*minor).patch(*patch).build(),
            | [Ok(major), Ok(minor)] => SemanticVersion::init().major(*major).minor(*minor).build(),
            | [Ok(major)] => SemanticVersion::init().major(*major).build(),
            | _ => SemanticVersion::default(),
        }
    }
}
impl SemanticVersion {
    /// Parse the numeric components of a semantic version string
    pub fn from_string(value: impl AsRef<str>) -> Self {
        Self::from(value.as_ref())
    }
}
impl<T: AsRef<str>> StringInterpolation<T> for T
where
    T: ToString,
{
    fn replace_placeholder_with_string(&self, placeholder: &str, value: &str) -> String {
        match Regex::new(&format!(r"{{{{\s*{placeholder}\s*}}}}")) {
            | Ok(re) => re.replace_all(self.as_ref(), value).to_string(),
            | Err(err) => {
                fail!("Regex replacement - {}", err);
                self.to_string()
            }
        }
    }
    fn with_indent(&self, spaces: usize) -> String {
        self.to_string()
            .lines()
            .map(|line| " ".repeat(spaces) + line.trim_start())
            .collect::<Vec<_>>()
            .join(LINE_SEPARATOR)
    }
}
impl<P: AsRef<str>> ToMarkdown for Vec<P> {
    fn to_markdown(&self) -> String {
        if self.is_empty() {
            "[]".to_string()
        } else {
            self.iter()
                .map(|x| format!("{LINE_SEPARATOR}- {}", x.as_ref()))
                .collect::<Vec<String>>()
                .join("")
        }
    }
}
impl<P: AsRef<str> + ToMarkdown> ToMarkdown for Option<Vec<P>> {
    fn to_markdown(&self) -> String {
        match &self {
            | Some(values) => format!("{LINE_SEPARATOR}{}", values.to_markdown()),
            | None => "[]".to_string(),
        }
    }
}
impl<T: AsRef<str>> ToStringChunks<T> for T
where
    T: ToString,
{
    fn chunk(&self, size: usize) -> Vec<String> {
        self.as_ref()
            .as_bytes()
            .chunks(size)
            .filter_map(|chunk| String::from_utf8(chunk.to_vec()).ok())
            .collect::<Vec<_>>()
    }
}
/// Returns a base32 encoded string using the [base 32 Crockford](https://www.crockford.com/base32.html) alphabet
/// ### Note
/// > Uses Crockford base32 alphabet (excludes I, L, O, U to avoid confusion)
///
/// ### Example
/// ```rust
/// use acorn::util::base32_crockford_encode;
///
/// let encoded = base32_crockford_encode(1234);
/// assert_eq!(encoded, "16j");
/// ```
pub fn base32_crockford_encode(value: u128) -> String {
    if value == 0 {
        "0".to_string()
    } else {
        const MODULUS: u128 = CROCKFORD_BASE32_ALPHABET.len() as u128;
        successors(Some(value), |&n| (n >= MODULUS).then_some(n / MODULUS))
            .map(|n| char::from(*CROCKFORD_BASE32_ALPHABET.get((n % MODULUS) as usize).unwrap_or(&0)))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>()
            .to_ascii_lowercase()
    }
}
/// Decode a base32 Crockford string into a u128 value.
///
/// ### Note
/// - Accepts lowercase/uppercase
/// - Treats `O` as `0` and `I`/`L` as `1`
/// - Ignores `-`, `_`, and whitespace separators
///
/// ### Example
/// ```rust
/// use acorn::util::base32_crockford_decode;
///
/// let decoded = base32_crockford_decode("16j").unwrap();
/// assert_eq!(decoded, 1234);
/// ```
pub fn base32_crockford_decode(value: impl AsRef<str>) -> Option<u128> {
    const MODULUS: u128 = CROCKFORD_BASE32_ALPHABET.len() as u128;
    value
        .as_ref()
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '_')
        .try_fold(0u128, |acc, c| {
            let digit = crockford_digit(c)?;
            match acc.checked_mul(MODULUS) {
                | Some(value) => value.checked_add(digit),
                | None => None,
            }
        })
}
/// Returns `true` if `a` and `b` have the same length and equal contents, without short-circuiting on the first
/// differing byte. Suitable for comparing secrets where timing side-channels must be avoided.
pub fn constant_time_eq<A: AsRef<[u8]>, B: AsRef<[u8]>>(a: A, b: B) -> bool {
    let (a, b) = (a.as_ref(), b.as_ref());
    a.len() == b.len() && a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
/// Returns true when any pattern exists in the haystack.
pub fn contains_any(patterns: &[&str], haystack: &str) -> bool {
    match AhoCorasick::new(patterns) {
        | Ok(matcher) => matcher.is_match(haystack),
        | Err(_) => false,
    }
}
/// Returns true when haystack contains a prefix and any suffix pattern.
pub fn contains_any_with_prefix(haystack: &str, prefix: &str, suffixes: &[&str]) -> bool {
    haystack.contains(prefix) && contains_any(suffixes, haystack)
}
fn crockford_digit(value: char) -> Option<u128> {
    let upper = value.to_ascii_uppercase();
    let normalized = if upper == 'O' {
        '0'
    } else if upper == 'I' || upper == 'L' {
        '1'
    } else {
        upper
    };
    let byte = normalized as u8;
    CROCKFORD_BASE32_ALPHABET.iter().position(|&b| b == byte).map(|index| index as u128)
}
/// Try to parse text as JSON or JSONC and return `true` if successful, `false` otherwise
pub fn detect_json(text: impl ToString) -> bool {
    let text = text.to_string();
    let options = ParseOptions {
        allow_comments: true,
        allow_trailing_commas: true,
        allow_loose_object_property_names: false,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    };
    serde_json::from_str::<Value>(&text).is_ok() || parse_to_serde_value::<Value>(&text, &options).is_ok()
}
/// Try to parse text as XML and return `true` if successful, `false` otherwise
pub fn detect_xml(text: impl ToString) -> bool {
    let content = text.to_string();
    let trimmed = content.trim();
    if trimmed.starts_with('<') {
        let mut reader = quick_xml::Reader::from_str(trimmed);
        let mut buf = vec![];
        loop {
            match reader.read_event_into(&mut buf) {
                | Ok(quick_xml::events::Event::Eof) => return true,
                | Err(_) => return false,
                | _ => buf.clear(),
            }
        }
    } else {
        false
    }
}
/// Returns the file extension of the given file name as a string.
/// ### Note
/// > The primary benefit of this function is to get file extension without using Path or PathBuf
///
/// ### Example
/// ```rust
/// use acorn::util::file_extension;
///
/// let extension = file_extension("test.cff");
/// assert_eq!(extension, Some("cff".to_string()));
/// ```
pub fn file_extension<S>(value: S) -> Option<String>
where
    S: Into<String>,
{
    let filename = value.into();
    let segments = filename.split('.').filter(|x| !x.is_empty()).collect::<Vec<_>>();
    if !segments.is_empty() {
        let last_segment = segments.last().map(|value| (*value).to_string());
        let has_extension = filename.contains(".") && segments.len() > 1;
        match last_segment {
            | Some(value) => {
                let is_filename = !(value.contains("/") || value.is_empty());
                if has_extension && is_filename {
                    Some(value)
                } else {
                    None
                }
            }
            | None => None,
        }
    } else {
        None
    }
}
/// Return fisrt key/value pair with key that matches pattern
/// ### Example
/// ```rust
/// use acorn::util::find_first;
///
/// let values = vec![("foo".to_string(), "bar".to_string()), ("baz".to_string(), "qux".to_string())];
/// let pattern = "ba";
/// let result = find_first(values, pattern);
/// assert_eq!(result, Some(("baz".to_string(), "qux".to_string())));
/// ```
pub fn find_first(values: Vec<(String, String)>, pattern: &str) -> Option<(String, String)> {
    let results = values
        .clone()
        .into_iter()
        .filter(|x| !x.1.is_empty())
        .find(|(key, _)| key.starts_with(pattern));
    match results {
        | Some(value) => Some(value),
        | None => None,
    }
}
/// Formats a number of bytes into a human-readable string with appropriate units (B, KB, MB, GB, TB)
pub fn format_bytes(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let (size, index) = successors(Some((bytes as f64, 0usize)), |(size, index)| {
        if *size >= 1024.0 && *index < units.len().saturating_sub(1) {
            Some((size / 1024.0, index.saturating_add(1)))
        } else {
            None
        }
    })
    .last()
    .unwrap_or((bytes as f64, 0));
    if index == 0 {
        format!("{} {}", bytes, units.get(index).unwrap_or(&""))
    } else {
        format!("{:.2} {}", size, units.get(index).unwrap_or(&""))
    }
}
/// Parse frontmatter and body from content that contains YAML frontmatter (e.g., Markdown, dotprompt, etc.)
/// ### Example
/// Input
/// ```markdown
/// ---
/// title: This is frontmatter
/// ---
/// This is the body
/// ```
/// Output
/// ```yaml
/// title: This is frontmatter
/// ```
/// ```markdown
/// This is the body
/// ```
pub fn frontmatter_and_body<S>(value: S) -> (Option<String>, String)
where
    S: AsRef<str>,
{
    let content = value.as_ref();
    let pattern = r"(?s)---\s*(?<frontmatter>.*?)\s*---\s*(?<body>.*)";
    let groups = vec!["frontmatter", "body"];
    let lookup = regex_capture_lookup(pattern, content, groups);
    (
        lookup.get("frontmatter").cloned().filter(|s| !s.is_empty()),
        lookup.get("body").cloned().unwrap_or_else(|| content.trim().to_string()),
    )
}
/// Generate a random ten-character ACORN identifier
pub fn generate_guid() -> String {
    #[cfg(feature = "std")]
    {
        let alphabet = [
            '-', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'T', 'U', 'V', 'W', 'X', 'Y', 'a', 'b', 'c',
            'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 't', 'w', 'x', 'y', 'z', '3', '4', '6', '7', '8', '9',
        ];
        nanoid!(10, &alphabet)
    }
    #[cfg(not(feature = "std"))]
    String::new()
}
/// Simple glob pattern matching supporting `*` (any sequence) and `?` (any single char).
pub fn glob_matches(path: &str, pattern: &str) -> bool {
    fn go(path: &[u8], pattern: &[u8]) -> bool {
        match pattern.split_first() {
            | None => path.is_empty(),
            | Some((b'*', rest)) => go(path, rest) || path.split_first().is_some_and(|(_, tail)| go(tail, pattern)),
            | Some((b'?', rest)) => path.split_first().is_some_and(|(_, tail)| go(tail, rest)),
            | Some((p, rest)) => path.split_first().is_some_and(|(q, tail)| p == q && go(tail, rest)),
        }
    }
    go(path.as_bytes(), pattern.as_bytes())
}
/// Check if value is a URI or filesystem path
pub fn is_uri_or_path(value: &str) -> bool {
    value.starts_with('/')
        || value.starts_with("./")
        || value.starts_with("../")
        || {
            #[cfg(feature = "std")]
            {
                Path::new(value).is_absolute()
            }
            #[cfg(not(feature = "std"))]
            {
                false
            }
        }
        || (if let Ok(uri) = UriRef::parse(value) {
            uri.scheme().is_some()
        } else {
            false
        })
}
/// Merge two string collections, preserving first-seen order and removing duplicates.
///
/// Values are trimmed before comparison and output. Empty or whitespace-only values
/// are skipped.
pub fn merge<A, B, S, T>(a: A, b: Option<B>) -> Vec<String>
where
    A: IntoIterator<Item = S>,
    B: IntoIterator<Item = T>,
    S: AsRef<str>,
    T: AsRef<str>,
{
    let mut seen = HashSet::new();
    a.into_iter()
        .map(|value| value.as_ref().trim().to_string())
        .chain(b.into_iter().flatten().map(|value| value.as_ref().trim().to_string()))
        .filter(|value| !value.is_empty() && seen.insert(value.clone()))
        .collect()
}
/// Helper function to create a lookup dictionary for regex captures
/// ### Note
/// > This function is sensitive to "un-named" regex groups (e.g. the parentheses around `\d{4}` in `(?<year>(\d{4}))`).
/// > For best functionality, avoid creating such groups by omitting unnecessary parentheses.
/// ### Example
/// ```rust
/// use acorn::util::regex_capture_lookup;
/// let lookup = regex_capture_lookup(
///     r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})",
///     "2023-06-30",
///     vec!["year", "month", "day"]
/// );
/// assert_eq!(lookup["year"], "2023");
/// assert_eq!(lookup["month"], "06");
/// assert_eq!(lookup["day"], "30");
/// ```
pub fn regex_capture_lookup<S>(pattern: S, text: S, groups: Vec<S>) -> HashMap<S, String>
where
    S: Into<String> + AsRef<str> + Clone + core::cmp::Eq + core::hash::Hash,
{
    #[allow(clippy::unwrap_used)]
    let re = Regex::new(pattern.as_ref()).unwrap();
    let mut lookup = HashMap::new();
    if let Some(capture_matches) = re.captures_iter(text.as_ref()).last() {
        match capture_matches {
            | Ok(captures) => {
                captures.iter().skip(1).enumerate().for_each(|(index, data)| {
                    if let Some(results) = data {
                        if let Some(key) = groups.get(index) {
                            let value = results.as_str().to_string();
                            lookup.insert(key.clone(), value);
                        }
                    }
                });
            }
            | Err(_) => (),
        }
    };
    lookup
}
/// Combine a list of regex patterns into a single alternation regex string.
pub fn regex_join(patterns: &[String]) -> Option<String> {
    let groups = patterns
        .iter()
        .filter(|pattern| !pattern.is_empty())
        .map(|pattern| format!("(?:{pattern})"))
        .collect::<Vec<String>>();
    match groups.is_empty() {
        | true => None,
        | false => Some(groups.join("|")),
    }
}
/// Invert a regex pattern using negative lookahead so matches become exclusions.
pub fn regex_inverse(pattern: impl AsRef<str>) -> String {
    format!("^(?!.*(?:{})).*$", pattern.as_ref())
}
/// Attempt to convert a safe regex pattern to a glob pattern.
///
/// Returns `Some(glob)` if the regex is simple enough for safe conversion, `None` otherwise.
///
/// Handles common model filter patterns:
/// - `\\.gguf$` → `*.gguf`
/// - `Q4_K_M.*\\.gguf$` → `*Q4_K_M*.gguf`
/// - `gguf$` → `*gguf`
/// - `tiny\\.gguf` → `*tiny.gguf*`
pub fn regex_to_glob(pattern: impl AsRef<str>) -> Option<String> {
    let mut bytes = pattern.as_ref().as_bytes().iter().peekable();
    let mut glob = String::new();
    let mut ok = true;
    let mut anchored_start = false;
    let mut anchored_end = false;
    if bytes.peek() == Some(&&b'^') {
        anchored_start = true;
        bytes.next();
    }
    while let Some(&byte) = bytes.next() {
        match byte {
            | b'\\' => match bytes.next() {
                | Some(&b'.') => glob.push('.'),
                | Some(b'd' | b'D' | b'w' | b'W' | b's' | b'S' | b'b' | b'B') | None => ok = false,
                | Some(&other) => glob.push(other as char),
            },
            | b'.' if bytes.peek() == Some(&&b'*') => {
                bytes.next();
                glob.push('*');
            }
            | b'.' if bytes.peek() == Some(&&b'+') => {
                bytes.next();
                glob.push('?');
                glob.push('*');
            }
            | b'.' | b'?' | b'+' | b'[' | b'(' | b'|' | b'{' => ok = false,
            | b'$' if bytes.peek().is_some() => ok = false,
            | b'$' => anchored_end = true,
            | other => glob.push(other as char),
        }
    }
    if ok && !glob.is_empty() {
        if !anchored_start {
            glob.insert(0, '*');
        }
        if !anchored_end {
            glob.push('*');
        }
        Some(glob)
    } else {
        None
    }
}
/// Converts the given string to snake case.
/// ### Example
/// ```rust
/// use acorn::util::snake_case;
///
/// let snake = snake_case("CamelCase");
/// assert_eq!(snake, "camel_case");
/// ```
pub fn snake_case<S>(value: S) -> String
where
    S: Into<String>,
{
    value.into().to_case(Case::Snake)
}
/// Remove the first matching suffix from a string slice.
pub fn strip_suffixes<'a>(values: &[&str], name: &'a str) -> &'a str {
    values.iter().find_map(|value| name.strip_suffix(value)).unwrap_or(name)
}
/// Returns "s" if the given value is not 1, otherwise returns an empty string.
/// ### Example
/// ```rust
/// use acorn::util::suffix;
///
/// assert_eq!(suffix(1_usize), "");
/// assert_eq!(suffix(2_usize), "s");
/// assert_eq!(suffix(1_u64), "");
/// assert_eq!(suffix(5_u64), "s");
/// ```
pub fn suffix<T>(value: T) -> String
where
    T: PartialEq + From<u8>,
{
    (if value == T::from(1) { "" } else { "s" }).to_string()
}
/// Normalize a string to lowercase ASCII alphanumeric characters.
pub fn to_ascii_alphanumeric(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}
/// Convert a vector of string slices to a vector of strings
pub fn to_string(values: Vec<&str>) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests;
