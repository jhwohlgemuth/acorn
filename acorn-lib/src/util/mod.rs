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
use crate::prelude::HashMap;
use bon::Builder;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use console::Emoji;
use convert_case::{Case, Casing};
use derive_more::Display;
use fancy_regex::Regex;
use nanoid::nanoid;
use owo_colors::{OwoColorize, Style, Styled};
use rust_embed::Embed;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use similar::{
    ChangeTag::{self, Delete, Equal, Insert},
    TextDiff,
};
use tracing::{debug, error};

pub mod constants;

/// Trait for augmenting data with linked data context
pub trait LinkedData {
    /// Add linked data (e.g., JSON-LD) context
    fn with_context(&self) -> Self;
}
/// Add enhanced string interpolation functionality
pub trait StringInterpolation<T>
where
    T: AsRef<str> + ToString,
{
    /// Replace placeholder instances with a given value (basic interpolation based on handlebars template syntax)
    fn replace_placeholder_with_string(&self, placeholder: &str, value: &str) -> String;
}
/// Trait for augmenting path value functionality with absolute path string conversion
pub trait ToAbsoluteString {
    /// Return a string representation of the absolute path
    fn to_absolute_string(&self) -> String;
}
/// Format data structures as Markdown
pub trait ToMarkdown {
    /// Convert `self` to Markdown format string
    fn to_markdown(&self) -> String;
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
/// # License
/// SPDX compliant license identifier
///
/// See <https://spdx.org/licenses/>
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
    /// Markdown
    #[display("text/markdown")]
    Markdown,
    /// OpenType Font (OTF)
    #[display("font/otf")]
    Otf,
    /// Portable Network Graphic (PNG)
    #[display("image/png")]
    Png,
    /// Rust Source Code (RS)
    #[display("text/rust")]
    Rust,
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
    /// YAML Ain't Markup Language (YAML)
    ///
    /// See <https://yaml.org/>
    #[display("application/yaml")]
    Yaml,
    /// Unknown MIME type
    #[display("application/unknown")]
    Unknown,
}
/// Enumeration for hardware resources used by technology
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
pub enum Resource {
    /// Central Processing Unit for "classical" computing
    CPU,
    /// Graphics Processing Unit
    GPU,
    /// Tensor Processing Unit
    TPU,
    /// Neuromorphic compute
    Neuromorphic,
    /// Quantum computing (e.g., NISQ, etc.)
    Quantum,
    /// Unknown, unspecified, or otherwise unclassified resource
    Other,
}
/// Struct for using and sharing constants
///
/// See <https://git.sr.ht/~pyrossh/rust-embed>
#[derive(Embed)]
#[folder = "assets/constants/"]
pub struct Constant;
/// Struct for using and sharing colorized logging labels
///
/// ### Labels [^1]
/// | Name    | Example Output |
/// |---------|----------------|
/// | Dry run | "=> DRY_RUN ■ Pretending to do a thing" |
/// | Skip    | "=> ⚠️  Thing was skipped" |
/// | Pass    | "=> ✅ Thing passed " |
/// | Fail    | "=> ✗ Thing failed " |
///
/// [^1]: Incomplete list of examples without foreground/background coloring
pub struct Label {}
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
impl Constant {
    /// Reads a file from the asset folder and returns its contents as a UTF-8 string.
    ///
    /// # Panics
    ///
    /// Panics if the file does not exist in the asset folder.
    pub fn from_asset(file_name: &str) -> String {
        match Constant::get(file_name) {
            | Some(value) => String::from_utf8_lossy(value.data.as_ref()).into(),
            | None => {
                error!(file_name, "=> {} Import Constant asset", Label::fail());
                panic!("Unable to import {file_name}")
            }
        }
    }
    /// Returns an iterator over the last values of each row in the given file.
    ///
    /// If a row is empty, an empty string is returned.
    pub fn last_values(file_name: &str) -> impl Iterator<Item = String> {
        Constant::csv(file_name)
            .into_iter()
            .map(|x| match x.last() {
                | Some(value) => value.to_string(),
                | None => "".to_string(),
            })
            .filter(|x| !x.is_empty())
    }
    /// Reads a file from the asset folder and returns its contents as an iterator over individual lines.
    ///
    /// # Panics
    ///
    /// Panics if the file does not exist in the asset folder.
    pub fn read_lines(file_name: &str) -> Vec<String> {
        let data = Constant::from_asset(file_name);
        data.lines().map(String::from).collect()
    }
    /// Reads a CSV file from the asset folder and returns its contents as a `Vec` of `Vec<String>`,
    /// where each inner vector represents a row and each string within the inner vector represents a cell value.
    ///
    /// # Arguments
    ///
    /// * `file_name` - A string slice representing the name of the CSV file (without extension).
    ///
    /// # Panics
    ///
    /// Panics if the file does not exist in the asset folder.
    pub fn csv(file_name: &str) -> Vec<Vec<String>> {
        Constant::read_lines(format!("{file_name}.csv").as_str())
            .into_iter()
            .map(|x| x.split(",").map(String::from).collect())
            .collect()
    }
}
impl Label {
    /// Emoji for use when logging a warning, caution, etc.
    pub const CAUTION: Emoji<'_, '_> = Emoji("⚠️  ", "!!! ");
    /// Emoji for use when logging a success, pass, etc.
    pub const CHECKMARK: Emoji<'_, '_> = Emoji("✅ ", "☑ ");
    /// Template string to customize the progress bar
    ///
    /// See <https://docs.rs/indicatif/latest/indicatif/#templates>
    pub const PROGRESS_BAR_TEMPLATE: &str = "  {spinner:.green}{pos:>5} of{len:^5}[{bar:40.green}] {msg}";
    /// "Dry run" label
    pub fn dry_run() -> Styled<&'static &'static str> {
        let style = Style::new().black().on_yellow();
        " DRY_RUN ■ ".style(style)
    }
    /// "Invalid" label
    pub fn invalid() -> String {
        Label::fmt_invalid(" ✗ INVALID")
    }
    /// "Invalid" label formatting
    pub fn fmt_invalid(value: &str) -> String {
        let style = Style::new().red().on_default_color();
        value.style(style).to_string()
    }
    /// "Valid" label
    pub fn valid() -> String {
        Label::fmt_valid(" ✓ VALID  ")
    }
    /// "Invalid" label formatting
    pub fn fmt_valid(value: &str) -> String {
        let style = Style::new().green().on_default_color();
        value.style(style).to_string()
    }
    /// "Fail" label
    pub fn fail() -> String {
        Label::fmt_fail("FAIL")
    }
    /// "Fail" label formatting
    pub fn fmt_fail(value: &str) -> String {
        let style = Style::new().white().on_red();
        format!(" ✗ {value} ").style(style).to_string()
    }
    /// "Found" label
    pub fn found() -> String {
        Label::fmt_found("FOUND")
    }
    /// "Found" label formatting
    pub fn fmt_found(value: &str) -> String {
        let style = Style::new().green().on_default_color();
        value.to_string().style(style).to_string()
    }
    /// "Not found" label
    pub fn not_found() -> String {
        Label::fmt_not_found("NOT_FOUND")
    }
    /// "Not found" label formatting
    pub fn fmt_not_found(value: &str) -> String {
        let style = Style::new().red().on_default_color();
        value.style(style).to_string()
    }
    /// "Output" label
    pub fn output() -> String {
        Label::fmt_output("OUTPUT")
    }
    /// "Output" label formatting
    pub fn fmt_output(value: &str) -> String {
        let style = Style::new().cyan().dimmed().on_default_color();
        value.style(style).to_string()
    }
    /// "Pass" label
    pub fn pass() -> String {
        Label::fmt_pass("SUCCESS")
    }
    /// "Pass" label formatting
    pub fn fmt_pass(value: &str) -> String {
        let style = Style::new().green().bold().on_default_color();
        format!("{}{}", Label::CHECKMARK, value).style(style).to_string()
    }
    /// "Read" label
    pub fn read() -> Styled<&'static &'static str> {
        let style = Style::new().green().on_default_color();
        "READ".style(style)
    }
    /// "Rejected" label
    pub fn rejected() -> String {
        Label::fmt_rejected("REJECTED")
    }
    /// "Rejected" label formatting
    pub fn fmt_rejected(value: &str) -> String {
        let style = Style::new().red().on_default_color();
        format!("🛑 {value} ").style(style).to_string()
    }
    /// "Run" label
    pub fn run() -> String {
        Label::fmt_run("RUN")
    }
    /// "Run" label formatting
    pub fn fmt_run(value: &str) -> String {
        let style = Style::new().black().on_yellow();
        format!("{value} ▶ ").style(style).to_string()
    }
    /// "Skip" label
    pub fn skip() -> String {
        Label::fmt_skip("SKIP")
    }
    /// "Skip" label formatting
    pub fn fmt_skip(value: &str) -> String {
        let style = Style::new().yellow().on_default_color();
        format!("{}{} ", Label::CAUTION, value).style(style).to_string()
    }
    /// "Using" label
    pub fn using() -> String {
        Label::fmt_using("USING")
    }
    /// "Using" label formatting
    pub fn fmt_using(value: &str) -> String {
        let style = Style::new().cyan();
        value.style(style).to_string()
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
            | "bsd-2-clause" => License::Bsd3Clause,
            | "bsd-3-clause" => License::Bsd3Clause,
            | "cc0-1.0" | "creative commons cc-0" => License::CreativeCommons,
            | "gpl-1.0-or-later" => License::Gpl2Only,
            | "gpl-2.0-only" => License::Gpl2Only,
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
    fn from_technology(value: &str) -> Option<License> {
        let data = Constant::csv("technology");
        let result = data
            .into_iter()
            .map(|row| row.into_iter().take(5).collect::<Vec<String>>())
            .find(|pair| pair[0] == value);
        match result {
            | Some(pair) => Some(License::from(pair[4].clone())),
            | None => None,
        }
    }
    fn is_open_source(&self) -> bool {
        let data = Constant::csv("technology");
        let result = data
            .into_iter()
            .map(|row| row.into_iter().skip(4).take(2).collect::<Vec<String>>())
            .find(|pair| pair[0] == self.to_string());
        match result {
            | Some(value) => value[1] == "true",
            | None => false,
        }
    }
}
impl<T: AsRef<str>> From<T> for MimeType
where
    T: ToString,
{
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
    /// | jsonld | application/ld+json |
    /// | md | text/markdown |
    /// | otf | font/otf |
    /// | png | image/png |
    /// | rs | text/rust |
    /// | svg | image/svg+xml |
    /// | toml | application/toml |
    /// | txt | text/plain |
    /// | yaml | application/yaml |
    fn from(value: T) -> Self {
        let name = value.to_string().to_lowercase();
        match file_extension(name) {
            | Some(value) => match value.as_str() {
                | "csv" => MimeType::Csv,
                | "jpg" | "jpeg" => MimeType::Jpeg,
                | "json" => MimeType::Json,
                | "jsonld" | "json-ld" => MimeType::LdJson,
                | "md" | "markdown" => MimeType::Markdown,
                | "otf" => MimeType::Otf,
                | "png" => MimeType::Png,
                | "rs" => MimeType::Rust,
                | "svg" => MimeType::Svg,
                | "toml" => MimeType::Toml,
                | "txt" => MimeType::Text,
                | "yml" | "yaml" | "cff" => MimeType::Yaml,
                | _ => MimeType::Unknown,
            },
            | None => MimeType::Unknown,
        }
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
            | MimeType::Jpeg => "jpeg",
            | MimeType::Json => "json",
            | MimeType::LdJson => "jsonld",
            | MimeType::Markdown => "md",
            | MimeType::Otf => "otf",
            | MimeType::Png => "png",
            | MimeType::Rust => "rs",
            | MimeType::Svg => "svg",
            | MimeType::Text => "txt",
            | MimeType::Toml => "toml",
            | MimeType::Yaml => "yaml",
            | _ => "unknown-file-type",
        }
        .to_string()
    }
}
impl SemanticVersion {
    /// Parses a string into a `SemanticVersion` value
    ///
    /// ### Example
    /// ```rust
    /// use acorn::util::SemanticVersion;
    ///
    /// let version = SemanticVersion::from_string("1.2.3");
    /// assert_eq!(version.minor, 2);
    /// ```
    pub fn from_string<S>(value: S) -> SemanticVersion
    where
        S: Into<String>,
    {
        let value = match Regex::new(r"\d*[.]\d*[.]\d*") {
            | Ok(re) => match re.find(&value.into()) {
                | Ok(value) => match value {
                    | Some(value) => value.as_str().to_string(),
                    | None => unreachable!(),
                },
                | Err(_) => unreachable!(),
            },
            | Err(_) => unreachable!(),
        };
        let mut parts = value.split('.');
        let major = parts.next().unwrap().parse::<u32>().unwrap();
        let minor = parts.next().unwrap().parse::<u32>().unwrap();
        let patch = parts.next().unwrap().parse::<u32>().unwrap();
        SemanticVersion { major, minor, patch }
    }
}
impl Default for SemanticVersion {
    fn default() -> Self {
        SemanticVersion::init().build()
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
                error!("=> {} Regex replacement - {err}", Label::fail());
                self.to_string()
            }
        }
    }
}
impl<P: AsRef<str>> ToMarkdown for Vec<P> {
    fn to_markdown(&self) -> String {
        self.iter().map(|x| format!("- {}", x.as_ref())).collect::<Vec<String>>().join("\n")
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
            .map(|chunk| String::from_utf8(chunk.to_vec()).unwrap())
            .collect::<Vec<_>>()
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
        let last_segment = segments.clone().last().unwrap().to_string();
        let has_extension = filename.contains(".") && segments.len() > 1;
        let is_filename = !(last_segment.contains("/") || last_segment.is_empty());
        if has_extension && is_filename {
            Some(last_segment)
        } else {
            None
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
/// Generates a random GUID using a custom alphabet.
///
/// The generated GUID is a 10-character string composed of a mix of uppercase
/// letters, lowercase letters, digits, and a hyphen. The function uses the
/// [nanoid](https://github.com/ai/nanoid) library to ensure randomness and uniqueness of the GUID.
///
/// # Returns
///
/// A `String` representing a randomly generated GUID.
pub fn generate_guid() -> String {
    let alphabet = [
        '-', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'T', 'U', 'V', 'W', 'X', 'Y', 'a', 'b', 'c', 'd',
        'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 't', 'w', 'x', 'y', 'z', '3', '4', '6', '7', '8', '9',
    ];
    let id = nanoid!(10, &alphabet);
    debug!(id, "=> {}", Label::using());
    id
}
/// Prints a diff of changes between two strings.
///
/// If there are no changes between `old` and `new`, prints a debug message indicating so.
/// Otherwise, prints a unified diff of the changes, with `+` indicating lines that are
/// present in `new` but not `old`, `-` indicating lines that are present in `old` but
/// not `new`, and lines that are the same in both are prefixed with a space.
pub fn print_changes(old: &str, new: &str) {
    let changes = text_diff_changes(old, new);
    let has_no_changes = changes.clone().into_iter().all(|(tag, _)| tag == Equal);
    if has_no_changes {
        debug!("=> {}No format changes", Label::skip());
    } else {
        for change in changes {
            print!("{}", change.1);
        }
    }
}
// TODO: Improve flexibility (see https://rust-lang.github.io/api-guidelines/flexibility.html#c-generic)
/// Prints the given values as a table.
///
/// # Arguments
///
/// * `title` - The title of the table.
/// * `headers` - The headers of the table.
/// * `rows` - The rows of the table as a vector of vectors of strings.
pub fn print_values_as_table(title: &str, headers: Vec<&str>, rows: Vec<Vec<String>>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(headers);
    rows.into_iter().for_each(|row| {
        table.add_row(row);
    });
    println!("=> {} \n{table}", title.green().bold());
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
pub fn regex_capture_lookup<S>(pattern: S, value: S, groups: Vec<S>) -> HashMap<S, String>
where
    S: Into<String> + AsRef<str> + Clone + core::cmp::Eq + core::hash::Hash,
{
    let re = Regex::new(pattern.as_ref()).unwrap();
    let mut lookup = HashMap::new();
    if let Some(capture_matches) = re.captures_iter(value.as_ref()).last() {
        match capture_matches {
            | Ok(captures) => {
                captures.iter().skip(1).enumerate().for_each(|(index, data)| {
                    if let Some(results) = data {
                        let key = groups[index].clone();
                        let value = results.as_str().to_string();
                        lookup.insert(key, value);
                    }
                });
            }
            | Err(_) => (),
        }
    };
    lookup
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
/// Returns "s" if the given value is not 1, otherwise returns an empty string.
/// ### Example
/// ```
/// use acorn::util::suffix;
///
/// assert_eq!(suffix(1), "");
/// assert_eq!(suffix(2), "s");
/// ```
pub fn suffix(value: usize) -> String {
    (if value == 1 { "" } else { "s" }).to_string()
}
/// Computes the differences between two strings line by line and returns a vector of changes.
///
/// Each change is represented as a tuple containing a `ChangeTag` indicating the type of change
/// (deletion, insertion, or equality) and a `String` with the formatted line prefixed with a
/// symbol indicating the type of change (`-` for deletions, `+` for insertions, and a space for equal lines).
///
/// The formatted string is also colored: red for deletions, green for insertions, and dimmed for equal lines.
///
/// # Arguments
///
/// * `old` - A string slice representing the original text.
/// * `new` - A string slice representing the modified text.
///
/// # Returns
///
/// A vector of tuples, each containing a `ChangeTag` and a formatted `String` representing the changes.
pub fn text_diff_changes(old: &str, new: &str) -> Vec<(ChangeTag, String)> {
    TextDiff::from_lines(old, new)
        .iter_all_changes()
        .map(|line| {
            let tag = line.tag();
            let text = match tag {
                | Delete => format!("- {line}").red().to_string(),
                | Insert => format!("+ {line}").green().to_string(),
                | Equal => format!("  {line}").dimmed().to_string(),
            };
            (tag, text)
        })
        .collect::<Vec<_>>()
}
/// Convert a vector of string slices to a vector of strings
pub fn to_string(values: Vec<&str>) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests;
