//! Allocation-backed utilities without host-service ownership
//!
//! This module contains common functions and data structures used to build the ACORN command line interface as well as support open science endeavors.
//!
//! ## Example Uses
//! ### Work with semantic versions
//! ```ignore
//! use acorn_core::util::SemanticVersion;
//!
//! let version = SemanticVersion::from_string("1.2.3");
//! assert_eq!(version.minor, 2);
//!
//! assert_eq!(version.major, 1);
//! ```
//!
#[cfg(any(feature = "exec", feature = "std"))]
use super::constants::app::{EXEC_HEADER_PREFIX, MAX_EXEC_BYTES, SUPPORTED_EXEC_VERSION};
#[cfg(feature = "exec")]
use super::constants::app::{
    MAX_EXEC_ARRAY_SIZE, MAX_EXEC_EXPR_DEPTH, MAX_EXEC_FUNCTIONS, MAX_EXEC_FUNCTION_DEPTH, MAX_EXEC_INTERNED_STRINGS, MAX_EXEC_MAP_SIZE,
    MAX_EXEC_MODULES, MAX_EXEC_OPERATIONS, MAX_EXEC_PROBE_CACHE_ENTRIES, MAX_EXEC_STRING_SIZE, MAX_EXEC_VARIABLES,
};
use super::constants::{CROCKFORD_BASE32_ALPHABET, LINE_SEPARATOR};
use crate::prelude::alloc::{format, vec, HashMap, HashSet, String, ToString, Vec};
use crate::validation::ValidationError;
use ::alloc::collections::BTreeSet;
use aho_corasick::AhoCorasick;
#[cfg(feature = "exec")]
use alloc::collections::VecDeque;
use bon::Builder;
use core::fmt;
use core::iter::{once, successors};
#[cfg(feature = "std")]
use core::str::from_utf8;
use derive_more::Display;
use fancy_regex::Regex;
use fluent_uri::UriRef;
use jiff::{fmt::temporal::DateTimeParser, tz::Offset, Timestamp};
use jsonc_parser::{parse_to_serde_value, ParseOptions};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(feature = "std")]
use std::fs::canonicalize;
#[cfg(feature = "std")]
use std::path::{Path, PathBuf};
#[cfg(feature = "exec")]
use std::sync::{Mutex, OnceLock};
use strum::EnumIs;
use urlencoding::encode;

static DATE_TIME_PARSER: DateTimeParser = DateTimeParser::new();

/// Trait for augmenting data with linked data context
pub trait LinkedData {
    /// Add linked data (e.g., JSON-LD) context
    fn with_context(&self) -> Self;
}
/// Read and write Markdown data, including YAML front matter and escaped text
pub trait MarkdownSupport {
    /// Convert `self` to Markdown format string
    fn to_markdown(&self) -> String;
    /// Parse Markdown content into a value.
    fn from_markdown(content: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        let _ = content;
        Err("Markdown parsing is not implemented for this type".to_string())
    }
    /// Deserialize YAML front matter while preserving nested enum representations.
    fn from_front_matter(value: &str) -> Result<Self, String>
    where
        Self: DeserializeOwned + Sized,
    {
        serde_norway::from_str::<Value>(value)
            .map_err(|why| why.to_string())
            .and_then(|value| serde_json::from_value(value).map_err(|why| why.to_string()))
    }
    /// Serialize YAML front matter while preserving nested enum representations.
    fn to_front_matter(&self) -> Result<String, String>
    where
        Self: Serialize,
    {
        serde_json::to_value(self)
            .map_err(|why| why.to_string())
            .and_then(|value| serde_norway::to_string(&value).map_err(|why| why.to_string()))
    }
    /// Encode a scalar value so Markdown structure cannot reinterpret its contents.
    fn to_markdown_text(&self) -> String
    where
        Self: AsRef<str>,
    {
        self.as_ref().replace('&', "&amp;").replace('\r', "&#13;").replace('\n', "&#10;")
    }
    /// Decode an encoded Markdown scalar value.
    fn decode_markdown_text(&self) -> String
    where
        Self: AsRef<str>,
    {
        self.as_ref().replace("&#13;", "\r").replace("&#10;", "\n").replace("&amp;", "&")
    }
    /// Normalize a vocabulary value for fuzzy matching.
    fn normalize(&self) -> String
    where
        Self: AsRef<str>,
    {
        self.as_ref()
            .chars()
            .filter(|character| !matches!(character, '-' | '_' | '.' | ','))
            .collect::<String>()
            .replace('&', "and")
            .trim()
            .to_string()
    }
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
/// Trait for augmenting and formatting string-like values for display.
pub trait StringConversion {
    /// Return a file name with its parent folder.
    #[cfg(feature = "std")]
    fn file_name_with_parent(&self) -> String;
    /// Collapse whitespace and convert text to lowercase.
    fn normalized(&self) -> String;
    /// Percent-encode the string for use in a URL component.
    fn percent_encode(&self) -> String;
    /// Return an absolute path string.
    #[cfg(feature = "std")]
    fn to_absolute_path(&self) -> String;
    /// Normalize text to lowercase ASCII alphanumeric characters.
    fn to_ascii_alphanumeric(&self) -> String;
    /// Return the value when it is a complete ISO 8601 calendar date.
    fn to_complete_date(&self) -> Option<&str> {
        None
    }
    /// Render a path string using native separators.
    #[cfg(feature = "std")]
    fn to_cross_platform_path(&self) -> String;
}
/// Add persistent-identifier utilities to string values.
pub trait StringExt {
    /// Check whether all characters in a string are numeric.
    fn is_numeric(&self) -> bool;
    /// Trim list values and discard empty strings when supported.
    fn trim_and_compact(self) -> Self
    where
        Self: Sized,
    {
        self
    }
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
    /// Prepend indentation while preserving indentation already present on each line
    fn with_additional_indent(&self, spaces: usize) -> String;
}
/// Format data structures as prose suitable for static analysis
pub trait ToProse {
    /// Convert `self` to prose format string
    fn to_prose(&self) -> String;
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
#[derive(Clone, Debug, Deserialize, Display, EnumIs, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
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
    /// Gzip-compressed data.
    #[display("application/gzip")]
    Gzip,
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
    /// OCI layer media type accepted for model artifacts.
    #[display("{_0}")]
    OciLayer(String),
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
    /// Rhai script.
    #[display("text/x-rhai")]
    Rhai,
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
    /// 7-Zip archive.
    #[display("application/x-7z-compressed")]
    SevenZip,
    /// Scalable Vector Graphic (SVG)
    #[display("image/svg+xml")]
    Svg,
    /// Tape archive.
    #[display("application/x-tar")]
    Tar,
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
    /// Zstandard-compressed data.
    #[display("application/zstd")]
    Zstd,
    /// Unknown MIME type
    #[display("application/vnd.{}", _0)]
    Vendor(String),
    /// Unknown MIME type
    #[display("application/octet-stream")]
    Unknown(String),
}
/// Cryptographic checksum value paired with its algorithm.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
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
/// use acorn_core::util::SemanticVersion;
///
/// let version = SemanticVersion::from_string("1.2.3");
/// assert_eq!(version.major, 1);
/// assert_eq!(version.to_string(), "1.2.3");
/// ```

#[derive(Builder, Clone, Copy, Debug, Deserialize, Display, Eq, Ord, PartialEq, PartialOrd, Serialize, JsonSchema)]
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
impl From<&str> for Checksum {
    fn from(value: &str) -> Self {
        value
            .split_once(':')
            .filter(|(_, checksum_value)| !checksum_value.is_empty())
            .map(|(algorithm, checksum_value)| Checksum {
                algorithm: ChecksumAlgorithm::from(algorithm),
                checksum_value: checksum_value.to_string(),
            })
            .unwrap_or_else(|| Checksum {
                algorithm: ChecksumAlgorithm::default(),
                checksum_value: value.to_string(),
            })
    }
}
impl From<String> for Checksum {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
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
#[cfg(feature = "std")]
impl AsRef<std::ffi::OsStr> for MimeType {
    fn as_ref(&self) -> &std::ffi::OsStr {
        std::ffi::OsStr::new(self.file_type_ref())
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
                | "gz" => MimeType::Gzip,
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
                | "rhai" => MimeType::Rhai,
                | "rtf" => MimeType::Rtf,
                | "rs" => MimeType::Rust,
                | "safetensors" => MimeType::Safetensors,
                | "7z" => MimeType::SevenZip,
                | "spdx.json" => MimeType::Sbom,
                | "svg" => MimeType::Svg,
                | "tar" => MimeType::Tar,
                | "toml" => MimeType::Toml,
                | "txt" => MimeType::Text,
                | "xls" | "xlsb" | "xlsm" | "xlsx" => MimeType::Excel,
                | "yml" | "yaml" => MimeType::Yaml,
                | "zst" | "tzst" => MimeType::Zstd,
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
#[cfg(feature = "std")]
impl From<anydoc::Format> for MimeType {
    fn from(value: anydoc::Format) -> Self {
        match value {
            | anydoc::Format::Csv => Self::Csv,
            | anydoc::Format::Doc => Self::Doc,
            | anydoc::Format::Docx => Self::Docx,
            | anydoc::Format::Epub => Self::Epub,
            | anydoc::Format::Excel => Self::Excel,
            | anydoc::Format::Odp => Self::Odp,
            | anydoc::Format::Ods => Self::Ods,
            | anydoc::Format::Odt => Self::Odt,
            | anydoc::Format::Pdf => Self::Pdf,
            | anydoc::Format::Ppt => Self::Ppt,
            | anydoc::Format::Pptx => Self::Powerpoint,
            | anydoc::Format::Rtf => Self::Rtf,
        }
    }
}
impl From<String> for MimeType {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}
impl MimeType {
    /// Infer a supported textual file type from in-memory content.
    pub fn infer_from(content: &str) -> Self {
        match serde_json::from_str::<Value>(content).is_ok() {
            | true => Self::Json,
            | false if detect_json(content) => Self::Jsonc,
            | false if is_research_activity_markdown(content) => Self::Markdown,
            | false if is_research_activity_yaml(content) => Self::Yaml,
            | false => Self::Text,
        }
    }
    /// Infer a MIME type from file content using host-backed format detection.
    #[cfg(feature = "std")]
    pub fn infer(bytes: &[u8]) -> Option<Self> {
        anydoc::Format::from_bytes(bytes)
            .map(Self::from)
            .or_else(|| {
                infer::get(bytes).and_then(|kind| match kind.mime_type() {
                    | "application/gzip" | "application/x-gzip" => Some(Self::Gzip),
                    | "application/pdf" => Some(Self::Pdf),
                    | "application/x-7z-compressed" => Some(Self::SevenZip),
                    | "application/zip" => Some(Self::Zip),
                    | "application/x-tar" => Some(Self::Tar),
                    | "application/zstd" | "application/x-zstd" => Some(Self::Zstd),
                    | "image/jpeg" => Some(Self::Jpeg),
                    | "image/png" => Some(Self::Png),
                    | _ => None,
                })
            })
            .or_else(|| {
                let candidate = is_rhai_source(bytes).then_some(Self::Rhai);
                #[cfg(feature = "exec")]
                let candidate = candidate.filter(|_| from_utf8(bytes).is_ok_and(rhai_probe_compiles));
                candidate
            })
    }
    /// Return whether this file type is accepted as OCI model content.
    pub fn is_model_resource(&self) -> bool {
        match self {
            | Self::Gguf | Self::Json | Self::Jsonc | Self::Onnx | Self::Pytorch | Self::Safetensors | Self::Text => true,
            | Self::Vendor(extension) => matches!(extension.as_str(), "model" | "tiktoken"),
            | Self::Unknown(name) => name.ends_with("tokenizer_config"),
            | _ => false,
        }
    }
    /// Return whether this MIME type is supported by repository automation.
    pub fn is_repository_file(&self) -> bool {
        self.is_cff() || self.is_json() || self.is_jsonc() || self.is_markdown() || self.is_yaml()
    }
    /// Returns the file type as a string
    /// ### Example
    /// ```rust
    /// use acorn_core::util::MimeType;
    ///
    /// let mime = MimeType::Cff;
    /// assert_eq!(mime.file_type(), "cff");
    /// ```
    pub fn file_type(&self) -> String {
        self.file_type_ref().to_string()
    }
    pub(super) fn file_type_ref(&self) -> &str {
        match self {
            | MimeType::Cff => "cff",
            | MimeType::Csv => "csv",
            | MimeType::Doc => "doc",
            | MimeType::Docx => "docx",
            | MimeType::Epub => "epub",
            | MimeType::Excel => "xlsx",
            | MimeType::Gguf => "gguf",
            | MimeType::Gzip => "tar.gz",
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
            | MimeType::Rhai => "rhai",
            | MimeType::Rtf => "rtf",
            | MimeType::Rust => "rs",
            | MimeType::Safetensors => "safetensors",
            | MimeType::Sbom => "spdx.json",
            | MimeType::SevenZip => "7z",
            | MimeType::Svg => "svg",
            | MimeType::Tar => "tar",
            | MimeType::Text => "txt",
            | MimeType::Toml => "toml",
            | MimeType::Yaml => "yaml",
            | MimeType::Zip => "zip",
            | MimeType::Zstd => "tar.zst",
            | MimeType::Vendor(extension) => extension,
            | _ => "unknown-file-type",
        }
    }
}
impl<P: AsRef<str>> MarkdownSupport for Option<Vec<P>> {
    fn to_markdown(&self) -> String {
        match &self {
            | Some(values) => values.to_markdown(),
            | None => "[]".to_string(),
        }
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
    /// use acorn_core::util::SemanticVersion;
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
impl From<(u64, u64, u64)> for SemanticVersion {
    fn from(value: (u64, u64, u64)) -> Self {
        Self::init()
            .major(u32::try_from(value.0).unwrap_or(u32::MAX))
            .minor(u32::try_from(value.1).unwrap_or(u32::MAX))
            .patch(u32::try_from(value.2).unwrap_or(u32::MAX))
            .build()
    }
}
impl PartialEq<(u64, u64, u64)> for SemanticVersion {
    fn eq(&self, other: &(u64, u64, u64)) -> bool {
        (u64::from(self.major), u64::from(self.minor), u64::from(self.patch)) == *other
    }
}
impl PartialOrd<(u64, u64, u64)> for SemanticVersion {
    fn partial_cmp(&self, other: &(u64, u64, u64)) -> Option<core::cmp::Ordering> {
        Some((u64::from(self.major), u64::from(self.minor), u64::from(self.patch)).cmp(other))
    }
}
impl SemanticVersion {
    /// Extract a canonical semantic version from the first line with the supplied prefix.
    pub fn extract(source: impl AsRef<str>, prefix: &str) -> Option<Self> {
        source
            .as_ref()
            .lines()
            .find(|line| line.trim_start().starts_with(prefix))
            .and_then(|line| line.trim().strip_prefix(prefix))
            .and_then(|version| Self::parse_exact(version).ok())
    }
    /// Parse a canonical numeric semantic version with one to three components.
    pub fn parse_exact(value: impl AsRef<str>) -> Result<Self, ValidationError> {
        let value = value.as_ref();
        let parts = value.split('.').collect::<Vec<_>>();
        let valid_component =
            |part: &str| !part.is_empty() && part.chars().all(|character| character.is_ascii_digit()) && (part.len() == 1 || !part.starts_with('0'));
        let parsed = match parts.as_slice() {
            | [major] if valid_component(major) => major.parse::<u32>().ok().map(|major| (major, 0, 0)),
            | [major, minor] if [major, minor].iter().all(|part| valid_component(part)) => major
                .parse::<u32>()
                .ok()
                .zip(minor.parse::<u32>().ok())
                .map(|(major, minor)| (major, minor, 0)),
            | [major, minor, patch] if [major, minor, patch].iter().all(|part| valid_component(part)) => major
                .parse::<u32>()
                .ok()
                .zip(minor.parse::<u32>().ok())
                .zip(patch.parse::<u32>().ok())
                .map(|((major, minor), patch)| (major, minor, patch)),
            | _ => None,
        };
        parsed
            .map(|(major, minor, patch)| SemanticVersion::init().major(major).minor(minor).patch(patch).build())
            .ok_or_else(|| ValidationError::new("version").with_message("Provide valid semantic version"))
    }
    /// Parse the numeric components of a semantic version string
    pub fn from_string(value: impl AsRef<str>) -> Self {
        Self::from(value.as_ref())
    }
    // TODO: Add support for resolve version globs (e.g., `1.2.*` or `1.*` or `*`)
    /// Validate that this version satisfies a required minimum version.
    pub fn is_supported(&self, minimum: Self) -> Result<(), ValidationError> {
        (*self >= minimum)
            .then_some(())
            .ok_or_else(|| ValidationError::new("version").with_message(format!("found {self}; minimum supported version is {minimum}")))
    }
}
impl MarkdownSupport for str {
    fn to_markdown(&self) -> String {
        self.to_string()
    }
}
impl StringExt for str {
    fn is_numeric(&self) -> bool {
        self.chars().all(char::is_numeric)
    }
}
impl MarkdownSupport for String {
    fn to_markdown(&self) -> String {
        self.clone()
    }
}
impl StringExt for String {
    fn is_numeric(&self) -> bool {
        self.as_str().is_numeric()
    }
}
impl<T: AsRef<str>> StringInterpolation<T> for T
where
    T: ToString,
{
    fn replace_placeholder_with_string(&self, placeholder: &str, value: &str) -> String {
        match Regex::new(&format!(r"{{{{\s*{placeholder}\s*}}}}")) {
            | Ok(re) => re.replace_all(self.as_ref(), value).to_string(),
            | Err(_) => self.to_string(),
        }
    }
    fn with_indent(&self, spaces: usize) -> String {
        self.to_string()
            .lines()
            .map(|line| " ".repeat(spaces) + line.trim_start())
            .collect::<Vec<_>>()
            .join(LINE_SEPARATOR)
    }
    fn with_additional_indent(&self, spaces: usize) -> String {
        let prefix = " ".repeat(spaces);
        self.to_string()
            .lines()
            .map(|line| prefix.clone() + line)
            .collect::<Vec<_>>()
            .join(LINE_SEPARATOR)
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
impl StringExt for Vec<&str> {
    fn is_numeric(&self) -> bool {
        self.iter().any(|value| value.is_numeric())
    }
    fn trim_and_compact(self) -> Self {
        self.into_iter().map(str::trim).filter(|value| !value.is_empty()).collect()
    }
}
impl StringExt for Vec<String> {
    fn is_numeric(&self) -> bool {
        self.iter().any(|value| value.is_numeric())
    }
    fn trim_and_compact(self) -> Self {
        self.into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect()
    }
}
impl<P: AsRef<str>> MarkdownSupport for Vec<P> {
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
/// Decode a base32 Crockford string into a u128 value.
///
/// ### Note
/// - Accepts lowercase/uppercase
/// - Treats `O` as `0` and `I`/`L` as `1`
/// - Ignores `-`, `_`, and whitespace separators
///
/// ### Example
/// ```rust
/// use acorn_core::util::base32_crockford_decode;
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
            crockford_digit(c).and_then(|digit| match acc.checked_mul(MODULUS) {
                | Some(value) => value.checked_add(digit),
                | None => None,
            })
        })
}
/// Returns a base32 encoded string using the [base 32 Crockford](https://www.crockford.com/base32.html) alphabet
/// ### Note
/// > Uses Crockford base32 alphabet (excludes I, L, O, U to avoid confusion)
///
/// ### Example
/// ```rust
/// use acorn_core::util::base32_crockford_encode;
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
/// Convert ASCII letters into a case-insensitive glob pattern.
pub fn case_insensitive_glob(value: &str) -> String {
    value
        .chars()
        .map(|character| match character.is_ascii_alphabetic() {
            | true => format!("[{}{}]", character.to_ascii_lowercase(), character.to_ascii_uppercase()),
            | false => character.to_string(),
        })
        .collect()
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
    match trimmed.starts_with('<') {
        | true => {
            let mut reader = quick_xml::Reader::from_str(trimmed);
            let mut buf = vec![];
            loop {
                match reader.read_event_into(&mut buf) {
                    | Ok(quick_xml::events::Event::Eof) => break true,
                    | Err(_) => break false,
                    | _ => buf.clear(),
                }
            }
        }
        | false => false,
    }
}
/// Expand one comma-separated brace expression in a glob pattern.
pub fn expand_brace_glob(pattern: &str) -> Result<Vec<String>, String> {
    match (pattern.find('{'), pattern.find('}')) {
        | (None, None) => Ok(vec![pattern.to_string()]),
        | (Some(open), Some(close)) if open < close => {
            let prefix = &pattern[..open];
            let suffix = &pattern[close.saturating_add(1)..];
            let choices = pattern[open.saturating_add(1)..close].split(',').collect::<Vec<_>>();
            match choices.iter().any(|choice| choice.is_empty()) || suffix.contains(['{', '}']) {
                | true => Err(format!("Invalid brace expression in glob pattern `{pattern}`")),
                | false => Ok(choices.into_iter().map(|choice| format!("{prefix}{choice}{suffix}")).collect()),
            }
        }
        | _ => Err(format!("Invalid brace expression in glob pattern `{pattern}`")),
    }
}
/// Returns the file extension of the given file name as a string.
/// ### Note
/// > The primary benefit of this function is to get file extension without using Path or PathBuf
///
/// ### Example
/// ```rust
/// use acorn_core::util::file_extension;
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
/// use acorn_core::util::find_first;
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
fn is_research_activity_markdown(content: &str) -> bool {
    frontmatter_and_body(content).0.is_some_and(|frontmatter| {
        serde_norway::from_str::<Value>(&frontmatter).ok().is_some_and(|value| match value {
            | Value::Object(fields) => {
                fields.get("schema").and_then(Value::as_str) == Some("acorn/research-activity") || is_research_activity_value(&fields)
            }
            | _ => false,
        })
    })
}
fn is_research_activity_value(fields: &serde_json::Map<String, Value>) -> bool {
    ["contact", "meta", "sections", "title"]
        .iter()
        .filter(|field| fields.contains_key(**field))
        .count()
        >= 2
}
fn is_research_activity_yaml(content: &str) -> bool {
    serde_norway::from_str::<Value>(content).ok().is_some_and(|value| match value {
        | Value::Object(fields) => is_research_activity_value(&fields),
        | _ => false,
    })
}
#[cfg(feature = "std")]
fn is_rhai_source(bytes: &[u8]) -> bool {
    from_utf8(bytes)
        .ok()
        .filter(|source| source.len() <= MAX_EXEC_BYTES)
        .is_some_and(|source| {
            let headers = source.lines().filter(|line| line.trim_start().starts_with(EXEC_HEADER_PREFIX));
            let header = headers.clone().next().map(str::trim);
            let one_header = headers.count() == 1;
            let first = source.lines().find(|line| !line.trim().is_empty()).map(str::trim);
            let supported = SemanticVersion::extract(source, EXEC_HEADER_PREFIX).is_some_and(|version| version == SUPPORTED_EXEC_VERSION);
            one_header && first == header && supported
        })
}
#[doc(hidden)]
pub fn is_uri_or_path_portable(value: &str) -> bool {
    value.starts_with('/') || value.starts_with("./") || value.starts_with("../") || UriRef::parse(value).is_ok_and(|uri| uri.scheme().is_some())
}
/// Merge two string collections, preserving first-seen order and removing duplicates
/// ## Note
/// Values are trimmed before comparison and output. Empty or whitespace-only values are skipped.
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
/// Merge two sequences into sorted unique values
pub fn merge_unique<T: Ord>(left: impl IntoIterator<Item = T>, right: impl IntoIterator<Item = T>) -> Vec<T> {
    left.into_iter().chain(right).collect::<BTreeSet<_>>().into_iter().collect()
}
/// Merge two sequences in first-seen order using a caller-defined equality predicate.
pub fn merge_unique_by<T>(left: impl IntoIterator<Item = T>, right: impl IntoIterator<Item = T>, equal: impl Fn(&T, &T) -> bool) -> Vec<T> {
    right.into_iter().fold(left.into_iter().collect::<Vec<_>>(), |values, value| {
        match values.iter().any(|existing| equal(existing, &value)) {
            | true => values,
            | false => values.into_iter().chain(once(value)).collect(),
        }
    })
}
/// Helper function to create a lookup dictionary for regex captures
/// ### Note
/// > This function is sensitive to "un-named" regex groups (e.g. the parentheses around `\d{4}` in `(?<year>(\d{4}))`).
/// > For best functionality, avoid creating such groups by omitting unnecessary parentheses.
/// ### Example
/// ```rust
/// use acorn_core::util::regex_capture_lookup;
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
    match Regex::new(pattern.as_ref()) {
        | Ok(re) => re
            .captures_iter(text.as_ref())
            .last()
            .and_then(Result::ok)
            .map(|captures| {
                captures
                    .iter()
                    .skip(1)
                    .enumerate()
                    .filter_map(|(index, data)| data.and_then(|results| groups.get(index).cloned().map(|key| (key, results.as_str().to_string()))))
                    .collect::<HashMap<S, String>>()
            })
            .unwrap_or_default(),
        | Err(_) => HashMap::new(),
    }
}
/// Invert a regex pattern using negative lookahead so matches become exclusions.
pub fn regex_inverse(pattern: impl AsRef<str>) -> String {
    format!("^(?!.*(?:{})).*$", pattern.as_ref())
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
#[cfg(feature = "exec")]
fn rhai_probe_compiles(source: &str) -> bool {
    static CACHE: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(VecDeque::new()));
    match cache.lock().is_ok_and(|entries| entries.iter().any(|entry| entry == source)) {
        | true => true,
        | false => {
            let mut engine = rhai::Engine::new_raw();
            engine.disable_symbol("eval");
            engine.set_max_array_size(MAX_EXEC_ARRAY_SIZE);
            engine.set_max_call_levels(MAX_EXEC_FUNCTION_DEPTH);
            engine.set_max_expr_depths(MAX_EXEC_EXPR_DEPTH, MAX_EXEC_FUNCTION_DEPTH);
            engine.set_max_functions(MAX_EXEC_FUNCTIONS);
            engine.set_max_map_size(MAX_EXEC_MAP_SIZE);
            engine.set_max_modules(MAX_EXEC_MODULES);
            engine.set_max_operations(MAX_EXEC_OPERATIONS);
            engine.set_max_string_size(MAX_EXEC_STRING_SIZE);
            engine.set_max_strings_interned(MAX_EXEC_INTERNED_STRINGS);
            engine.set_max_variables(MAX_EXEC_VARIABLES);
            let is_valid = engine.compile(source).is_ok_and(|ast| {
                let entry_points = ast.iter_functions().filter(|function| function.name == "evaluate").collect::<Vec<_>>();
                matches!(entry_points.as_slice(), [entry_point] if entry_point.params.len() == 1)
            });
            if is_valid {
                let _ = cache.lock().map(|mut entries| {
                    if entries.len() == MAX_EXEC_PROBE_CACHE_ENTRIES {
                        entries.pop_front();
                    }
                    entries.push_back(source.to_string());
                });
            }
            is_valid
        }
    }
}
/// Returns "s" if the given value is not 1, otherwise returns an empty string.
/// ### Example
/// ```rust
/// use acorn_core::util::suffix;
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
#[cfg(feature = "std")]
fn cross_platform_path(path: &Path) -> String {
    let value = path.display().to_string();
    #[cfg(windows)]
    let value = value.strip_prefix(r"\\?\").unwrap_or(&value).replace('/', "\\");
    value
}
#[cfg(feature = "std")]
fn file_name_with_parent(path: &Path) -> String {
    let name = path.file_name().and_then(|value| value.to_str()).unwrap_or_default().to_string();
    match path.is_dir() {
        | true => name,
        | false => {
            let parent_name = path
                .parent()
                .and_then(|value| value.file_name())
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            match parent_name.is_empty() {
                | true => name,
                | false => format!("{parent_name}/{name}"),
            }
        }
    }
}
#[cfg(feature = "std")]
fn to_absolute_path(path: &Path) -> String {
    let result = canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let value = result.display().to_string();
    #[cfg(windows)]
    let value = value.strip_prefix(r"\\?\").unwrap_or(&value).to_string();
    value
}
/// Normalize a string to lowercase ASCII alphanumeric characters
pub fn to_ascii_alphanumeric(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}
/// Format a timestamp as RFC 3339 with an explicit UTC offset
pub fn to_rfc3339(value: Timestamp) -> String {
    value.display_with_offset(Offset::UTC).to_string()
}
/// Convert a vector of string slices to a vector of strings
pub fn to_string(values: Vec<&str>) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}

impl StringConversion for str {
    #[cfg(feature = "std")]
    fn file_name_with_parent(&self) -> String {
        file_name_with_parent(Path::new(self))
    }
    fn normalized(&self) -> String {
        self.split_whitespace().collect::<Vec<_>>().join(" ").to_ascii_lowercase()
    }
    fn percent_encode(&self) -> String {
        encode(self).into_owned()
    }
    #[cfg(feature = "std")]
    fn to_absolute_path(&self) -> String {
        to_absolute_path(Path::new(self))
    }
    fn to_ascii_alphanumeric(&self) -> String {
        to_ascii_alphanumeric(self)
    }
    fn to_complete_date(&self) -> Option<&str> {
        (self.len() == 10 && DATE_TIME_PARSER.parse_date(self).is_ok()).then_some(self)
    }
    #[cfg(feature = "std")]
    fn to_cross_platform_path(&self) -> String {
        cross_platform_path(Path::new(self))
    }
}
impl StringConversion for String {
    #[cfg(feature = "std")]
    fn file_name_with_parent(&self) -> String {
        <str as StringConversion>::file_name_with_parent(self.as_str())
    }
    fn normalized(&self) -> String {
        <str as StringConversion>::normalized(self.as_str())
    }
    fn percent_encode(&self) -> String {
        <str as StringConversion>::percent_encode(self.as_str())
    }
    #[cfg(feature = "std")]
    fn to_absolute_path(&self) -> String {
        <str as StringConversion>::to_absolute_path(self.as_str())
    }
    fn to_ascii_alphanumeric(&self) -> String {
        <str as StringConversion>::to_ascii_alphanumeric(self.as_str())
    }
    fn to_complete_date(&self) -> Option<&str> {
        <str as StringConversion>::to_complete_date(self.as_str())
    }
    #[cfg(feature = "std")]
    fn to_cross_platform_path(&self) -> String {
        <str as StringConversion>::to_cross_platform_path(self.as_str())
    }
}
impl StringConversion for &str {
    #[cfg(feature = "std")]
    fn file_name_with_parent(&self) -> String {
        <str as StringConversion>::file_name_with_parent(self)
    }
    fn normalized(&self) -> String {
        <str as StringConversion>::normalized(self)
    }
    fn percent_encode(&self) -> String {
        <str as StringConversion>::percent_encode(self)
    }
    #[cfg(feature = "std")]
    fn to_absolute_path(&self) -> String {
        <str as StringConversion>::to_absolute_path(self)
    }
    fn to_ascii_alphanumeric(&self) -> String {
        <str as StringConversion>::to_ascii_alphanumeric(self)
    }
    fn to_complete_date(&self) -> Option<&str> {
        <str as StringConversion>::to_complete_date(self)
    }
    #[cfg(feature = "std")]
    fn to_cross_platform_path(&self) -> String {
        <str as StringConversion>::to_cross_platform_path(self)
    }
}
#[cfg(feature = "std")]
impl StringConversion for PathBuf {
    fn file_name_with_parent(&self) -> String {
        file_name_with_parent(self)
    }
    fn normalized(&self) -> String {
        self.to_string_lossy().as_ref().normalized()
    }
    fn percent_encode(&self) -> String {
        self.to_string_lossy().as_ref().percent_encode()
    }
    fn to_absolute_path(&self) -> String {
        to_absolute_path(self)
    }
    fn to_ascii_alphanumeric(&self) -> String {
        to_ascii_alphanumeric(self.to_string_lossy().as_ref())
    }
    fn to_cross_platform_path(&self) -> String {
        cross_platform_path(self)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "std")]
    use super::MimeType;
    use super::{Checksum, ChecksumAlgorithm, SemanticVersion, StringConversion, StringExt};
    use crate::prelude::alloc::{String, ToString};
    use crate::util::constants::app::EXEC_HEADER_PREFIX;

    #[cfg(all(feature = "std", not(feature = "exec")))]
    #[test]
    fn test_mime_type_infers_rhai_candidate_without_exec_compiler() {
        let source = b"// @acorn-exec-v0.2.0\nthis is not valid Rhai";
        assert_eq!(MimeType::infer(source), Some(MimeType::Rhai));
    }
    #[cfg(feature = "exec")]
    #[test]
    fn test_mime_type_rejects_rhai_candidate_that_does_not_compile() {
        let source = b"// @acorn-exec-v0.2.0\nthis is not valid Rhai";
        assert_eq!(MimeType::infer(source), None);
    }
    #[test]
    fn test_semantic_version_extracts_prefixed_version() {
        assert_eq!(
            SemanticVersion::extract("// @acorn-exec-v0.2.0", EXEC_HEADER_PREFIX),
            Some(SemanticVersion::init().major(0).minor(2).patch(0).build())
        );
    }
    #[test]
    fn test_string_ext_default_compaction_is_identity() {
        let value = " value ".to_string();
        assert_eq!(value.clone().trim_and_compact(), value);
    }
    #[test]
    fn test_string_conversion_complete_date() {
        assert_eq!("2024-02-29".to_complete_date(), Some("2024-02-29"));
        assert_eq!("2023-02-29".to_complete_date(), None);
        assert_eq!("2024-02".to_complete_date(), None);
        assert_eq!("2024-02-29T00:00:00Z".to_complete_date(), None);
    }
    #[test]
    fn test_checksum_from_parses_borrowed_and_owned_values() {
        let checksum = Checksum::from("SHA512:abc123");
        assert_eq!(checksum.algorithm, ChecksumAlgorithm::Sha512);
        assert_eq!(checksum.checksum_value, "abc123");
        let owned_checksum = Checksum::from("md5:def456".to_string());
        assert_eq!(owned_checksum.algorithm, ChecksumAlgorithm::Md5);
        assert_eq!(owned_checksum.checksum_value, "def456");
    }
    #[test]
    fn test_checksum_from_uses_default_algorithm_for_malformed_values() {
        ["abc123", "sha256:"].into_iter().for_each(|value| {
            let checksum = Checksum::from(value);
            assert_eq!(checksum.algorithm, ChecksumAlgorithm::Sha256);
            assert_eq!(checksum.checksum_value, value);
        });
    }
    #[test]
    fn test_string_ext_vectors_trim_compact_and_detect_numeric_values() {
        let borrowed = vec![" alpha ", "", " 42 "].trim_and_compact();
        assert_eq!(borrowed, vec!["alpha", "42"]);
        assert!(borrowed.is_numeric());
        let owned = vec![" alpha ".to_string(), String::new(), " 42 ".to_string()].trim_and_compact();
        assert_eq!(owned, vec!["alpha".to_string(), "42".to_string()]);
        assert!(owned.is_numeric());
    }
}
