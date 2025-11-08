//! # Common utilities
//!
//! This module contains common functions and data structures used to build the ACORN command line interface as well as support open science endeavors.
//!
//! ## Example Uses
//! ### Work with semantic versions
//! ```ignore
//! use acorn_lib::util::SemanticVersion;
//!
//! let version = SemanticVersion::from_string("1.2.3");
//! assert_eq!(version.minor, 2);
//!
//! if let Some(version) = SemanticVersion::from_command("cargo") {
//!     println!("cargo version: {version}");
//! }
//! ```
//!
//! ### Perform file read and write operations
//! ```ignore
//! use acorn_lib::util::{checksum, read_file, write_file};
//! use std::path::PathBuf;
//!
//! // Verify file integrity
//! assert_eq!(checksum(PathBuf::from("/path/to/file")), "somesha256hashvaluethatisreallylong");
//!
//! // Read file contents
//! let contents = read_file(PathBuf::from("/path/to/this/file"));
//!
//! // Write file contents
//! write_file(PathBuf::from("/path/to/that/file"), contents);
//! ```
//!
use crate::prelude::{canonicalize, set_permissions, var, HashMap, OsStr, Path, PathBuf, Permissions};
use bon::Builder;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use console::Emoji;
use convert_case::{Case, Casing};
use derive_more::Display;
use duct::cmd;
use fancy_regex::Regex;
use glob::glob;
use is_executable::IsExecutable;
use itertools::Itertools;
use nanoid::nanoid;
use owo_colors::{OwoColorize, Style, Styled};
use reqwest::header::USER_AGENT;
use rust_embed::Embed;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use similar::{
    ChangeTag::{self, Delete, Equal, Insert},
    TextDiff,
};
use tracing::{debug, error, warn};
use which::which;

pub mod citeas;
#[cfg_attr(not(feature = "std"), no_std)]
pub mod io;

/// Trait for augmenting path value functionality with absolute path string conversion
pub trait ToAbsoluteString {
    /// Return a string representation of the absolute path
    fn to_absolute_string(&self) -> String;
}
/// Trait for adding chunking functionality
pub trait ToStringChunks<T>
where
    T: AsRef<str> + ToString,
{
    /// Chunk a string into substrings of a given size
    fn chunk(&self, size: usize) -> Vec<String>;
}
/// Trait for converting a vector of PathBufs to a vector of strings
pub trait ToStrings {
    /// Convert a vector of string slices to a vector of string values of paths
    ///
    /// This is a convenience that I find myself wanting to use in a lot of places.
    ///
    /// Adding a `to_strings` method to the `Vec<PathBuf>` types seems like a good idea.
    /// ### Example
    /// ```rust
    /// use acorn_lib::util::ToStrings;
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
/// SPDX compliant license identifier
///
/// See <https://spdx.org/licenses/>
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum License {
    /// MIT License
    Mit,
    /// Creative Commons
    #[serde(alias = "Creative Commons CC-0")]
    CreativeCommons,
    /// Unknown license
    Unknown,
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
/// Struct for using and sharing constants
///
/// See <https://git.sr.ht/~pyrossh/rust-embed>
#[derive(Embed)]
#[folder = "assets/constants/"]
pub struct Constant;
/// Struct for parsing GitLab API merge request diff responses
///
/// Used by [`files_from_gitlab_merge_request`]
///
/// ### Example Response JSON
/// ```json
/// [
///     {
///         "old_path": "README",
///         "new_path": "README",
///         "a_mode": "100644",
///         "b_mode": "100644",
///         "diff": "@@ -1 +1 @@\ -Title\ +README",
///         "collapsed": false,
///         "too_large": false,
///         "new_file": false,
///         "renamed_file": false,
///         "deleted_file": false,
///         "generated_file": false
///     },
///     {
///         "old_path": "VERSION",
///         "new_path": "VERSION",
///         "a_mode": "100644",
///         "b_mode": "100644",
///         "diff": "@@\ -1.9.7\ +1.9.8",
///         "collapsed": false,
///         "too_large": false,
///         "new_file": false,
///         "renamed_file": false,
///         "deleted_file": false,
///         "generated_file": false
///     }
/// ]
/// ```
///
/// See <https://docs.gitlab.com/api/merge_requests/#list-merge-request-diffs> for more information
#[derive(Debug, Deserialize)]
pub struct GitlabMergeRequestDiffResponse {
    new_path: String,
    // diff: String,
    // old_path: String,
    // too_large: Option<bool>,
    // new_file: bool,
    // renamed_file: bool,
    // deleted_file: bool,
    // generated_file: bool,
}
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
/// use acorn_lib::util::SemanticVersion;
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
/// Struct for adding ToStringList functionality
pub struct StringList<'a>(pub &'a Vec<PathBuf>);
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
impl MimeType {
    /// Returns the file type as a string
    /// ### Example
    /// ```rust
    /// use acorn_lib::util::MimeType;
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
    /// Returns a [`MimeType`] value based on the file extension of the given file name.
    ///
    /// Uses [`MimeType::from_string`].
    ///
    /// ```rust
    /// use acorn_lib::util::MimeType;
    /// use std::path::PathBuf;
    ///
    /// let mime = MimeType::from_path(PathBuf::from("test.cff"));
    /// assert_eq!(mime, MimeType::Yaml);
    /// ```
    pub fn from_path<P>(value: P) -> MimeType
    where
        P: Into<PathBuf>,
    {
        MimeType::from_string(value.into().display().to_string())
    }
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
    pub fn from_string<S>(value: S) -> MimeType
    where
        S: Into<String>,
    {
        let name = &value.into().to_lowercase();
        match extension(Path::new(name)).as_str() {
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
        }
    }
}
impl SemanticVersion {
    /// Returns a `SemanticVersion` value based on the output of the `--version` command-line flag
    /// of the given executable name. Tested with [cargo](https://rustup.rs/), [git](https://git-scm.com/book/en/v2/Getting-Started-The-Command-Line), and [pandoc](https://pandoc.org/).
    ///
    /// <div class="warning">this function only supports commands that provide a `--version` flag</div>
    ///
    /// ### Example
    /// ```ignore
    /// use acorn_lib::schema::validate::SemanticVersion;
    ///
    /// let version = SemanticVersion::from_command("cargo").to_string();
    /// assert_eq!(version, "1.90.0");
    /// ```
    pub fn from_command<S>(name: S) -> Option<SemanticVersion>
    where
        S: Into<String> + duct::IntoExecutablePath + core::marker::Copy,
    {
        if command_exists(name.into()) {
            let result = cmd(name, vec!["--version"]).read();
            match result {
                | Ok(value) => {
                    let first_line = value.lines().collect::<Vec<_>>().first().cloned();
                    match first_line {
                        | Some(line) => Some(SemanticVersion::from_string(line)),
                        | None => None,
                    }
                }
                | Err(_) => None,
            }
        } else {
            None
        }
    }
    /// Parses a string into a `SemanticVersion` value
    ///
    /// ### Example
    /// ```rust
    /// use acorn_lib::util::SemanticVersion;
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
impl ToAbsoluteString for PathBuf {
    fn to_absolute_string(&self) -> String {
        to_absolute_string(self.clone())
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
impl<P: Into<PathBuf> + Clone> ToStrings for Vec<P> {
    fn to_strings(&self) -> Vec<String> {
        self.iter()
            .map(|p| <P as Into<PathBuf>>::into(p.clone()).to_string_lossy().to_string())
            .collect()
    }
    fn to_absolute_strings(&self) -> Vec<String> {
        self.iter().map(|p| <P as Into<PathBuf>>::into(p.clone()).to_absolute_string()).collect()
    }
}
/// Checks if a given command exists in current terminal context.
///
/// # Arguments
///
/// * `name` - A string slice or `String` containing the name of the command to be checked.
///
/// # Return
///
/// A boolean indicating whether the command exists or not.
pub fn command_exists<S>(name: S) -> bool
where
    S: Into<String> + AsRef<OsStr> + tracing::Value,
{
    match which(&name) {
        | Ok(value) => {
            let path = to_absolute_string(value.clone());
            match value.try_exists() {
                | Ok(true) => {
                    debug!(path, "=> {} Command", Label::found());
                    true
                }
                | _ => {
                    debug!(path, "=> {} Command", Label::not_found());
                    false
                }
            }
        }
        | Err(_) => {
            warn!(name, "=> {} Command", Label::not_found());
            false
        }
    }
}
/// Get file extension
///
/// # Examples
/// ```
/// use std::path::Path;
/// use acorn_lib::util::extension;
///
/// assert_eq!("txt", extension(Path::new("hello.txt")));
/// assert_eq!("md", extension(Path::new("README.md")));
/// assert_eq!("", extension(Path::new(".dotfile")));
/// assert_eq!("", extension(Path::new("/path/to/folder")));
/// ```
pub fn extension(path: &Path) -> String {
    path.extension().unwrap_or_default().to_str().unwrap_or_default().to_string()
}
/// Returns a vector of `PathBuf` containing all files in a directory that match at least one of the given extensions.
///
/// # Arguments
///
/// * `path` - A `PathBuf` to the directory to search.
/// * `extensions` - An `Option` containing a list of string slice(s) representing the file extension(s) to search for.
///
/// # Returns
///
/// A `Vec` containing `PathBuf` values of all files in the given directory that match at least one of the given extensions.
// TODO: Add support for URI path
pub fn files_all(path: PathBuf, extensions: Option<Vec<&str>>) -> Vec<PathBuf> {
    fn paths_to_vec(paths: glob::Paths) -> Vec<PathBuf> {
        paths.collect::<Vec<_>>().into_iter().filter_map(|x| x.ok()).collect::<Vec<_>>()
    }
    fn pattern(path: PathBuf, extension: &str) -> String {
        let ext = &extension.to_lowercase();
        let result = format!("{}/**/*.{}", to_absolute_string(path), ext);
        debug!("=> {} {result}", Label::using());
        result
    }
    if path.is_dir() {
        match extensions {
            | Some(values) => values
                .into_iter()
                .map(|extension| {
                    let glob_pattern = pattern(path.clone(), extension);
                    glob(&glob_pattern)
                })
                .filter(|x| x.is_ok())
                .flat_map(|x| paths_to_vec(x.unwrap()))
                .unique()
                .collect::<Vec<PathBuf>>(),
            | None => match glob(&format!("{}/**/*", to_absolute_string(path))) {
                | Ok(paths) => paths_to_vec(paths),
                | Err(why) => {
                    error!("=> {} Get all files (Glob) - {why}", Label::fail());
                    vec![]
                }
            },
        }
    } else {
        if extensions.is_some() {
            warn!(
                path = to_absolute_string(path.clone()),
                "=> {} Extension passed with single file to files_all() - please make sure this is desired",
                Label::using()
            );
        }
        vec![path]
    }
}
/// Returns a vector of `PathBuf` containing all files changed in the given Git branch relative to the default branch.
///
/// # Arguments
///
/// * `value` - A string slice representing the name of the Git branch to check.
/// * `extension` - An `Option` containing a string slice representing the file extension to filter results by.
pub fn files_from_git_branch(value: &str, extensions: Option<Vec<&str>>) -> Vec<PathBuf> {
    if command_exists("git".to_owned()) {
        let default_branch = match git_default_branch_name() {
            | Some(value) => value,
            | None => "main".to_string(),
        };
        let args = vec!["diff", "--name-only", &default_branch, "--merge-base", value];
        let result = cmd("git", args).read();
        match result {
            | Ok(value) => filter_git_command_result(value, extensions),
            | Err(why) => {
                error!("=> {} Get files from Git branch - {why}", Label::fail());
                vec![]
            }
        }
    } else {
        vec![]
    }
}
/// Returns a vector of `PathBuf` containing all files changed in the given Git commit.
///
/// # Arguments
///
/// * `value` - A string slice representing the Git commit hash to check.
/// * `extension` - An `Option` containing a string slice representing the file extension to filter results by.
pub fn files_from_git_commit(value: &str, extensions: Option<Vec<&str>>) -> Vec<PathBuf> {
    if command_exists("git".to_owned()) {
        let args = vec!["diff-tree", "--no-commit-id", "--name-only", "-r", value];
        let result = cmd("git", args).read();
        debug!("=> {} Git command response - {result:?}", Label::using());
        let files = match result {
            | Ok(value) => filter_git_command_result(value, extensions),
            | Err(why) => {
                error!("=> {} Get files from Git commit - {why}", Label::fail());
                vec![]
            }
        };
        debug!(
            "=> {} Found {} file{} from Git commit - {files:?}",
            Label::using(),
            files.len(),
            suffix(files.len())
        );
        files
    } else {
        vec![]
    }
}
/// Returns a vector of `PathBuf` containing all files changed in a GitLab merge request, as determined by the `CI_API_V4_URL`, `CI_MERGE_REQUEST_PROJECT_ID`, and `CI_MERGE_REQUEST_IID` environment variables[^env].
///
/// See <https://docs.gitlab.com/api/merge_requests/#list-merge-request-diffs> for more information
///
/// [^env]: See <https://docs.gitlab.com/ci/variables/predefined_variables/> for more information about GitLab CI environment variables
pub fn files_from_gitlab_merge_request(extensions: Option<Vec<&str>>) -> Vec<PathBuf> {
    let root = var("CI_API_V4_URL").unwrap_or_default();
    let project_id = var("CI_MERGE_REQUEST_PROJECT_ID").unwrap_or_default();
    let merge_request_iid = var("CI_MERGE_REQUEST_IID").unwrap_or_default();
    let path = format!("/projects/{project_id}/merge_requests/{merge_request_iid}/diffs");
    let url = format!("{root}{path}");
    match reqwest_request(url).send() {
        | Ok(response) => {
            let content: serde_json::Result<Vec<GitlabMergeRequestDiffResponse>> = serde_json::from_str(&response.text().unwrap());
            match content {
                | Ok(data) => {
                    debug!("=> {} GitLab API merge request diff response - {data:#?}", Label::using());
                    let results = data.into_iter().map(|x| PathBuf::from(x.new_path)).collect::<Vec<PathBuf>>();
                    match extensions {
                        | Some(values) => results
                            .into_iter()
                            .filter(|path| values.iter().any(|ext| MimeType::from_path(path).file_type() == *ext.to_lowercase()))
                            .collect::<Vec<_>>(),
                        | None => results,
                    }
                }
                | Err(why) => {
                    error!("=> {} Parse GitLab API merge request diff response - {why}", Label::fail());
                    vec![]
                }
            }
        }
        | Err(why) => {
            error!("=> {} Get GitLab API merge request diff response - {why}", Label::fail());
            vec![]
        }
    }
}
fn filter_git_command_result(value: String, extensions: Option<Vec<&str>>) -> Vec<PathBuf> {
    match extensions {
        | Some(values) => value
            .to_lowercase()
            .split("\n")
            .map(PathBuf::from)
            .filter(|path| values.iter().any(|ext| MimeType::from_path(path).file_type() == *ext.to_lowercase()))
            .collect::<Vec<_>>(),
        | None => value.to_lowercase().split("\n").map(PathBuf::from).collect::<Vec<_>>(),
    }
}
/// Return file paths in a vector that don't match the ignore pattern
/// ### Example
/// ```rust
/// use acorn_lib::util::filter_ignored;
/// use std::path::PathBuf;
///
/// let paths = vec![PathBuf::from("/path/to/foo.txt"), PathBuf::from("/path/to/bar.txt")];
/// let ignore = Some("*.txt".to_string());
/// let result = filter_ignored(paths, ignore);
/// assert!(result.is_empty());
/// ```
pub fn filter_ignored(paths: Vec<PathBuf>, ignore: Option<String>) -> Vec<PathBuf> {
    match ignore {
        | Some(ignore_pattern) => match Regex::new(&ignore_pattern) {
            | Ok(re) => paths
                .into_iter()
                .map(to_absolute_string)
                .filter(|x| !re.is_match(x).unwrap())
                .map(PathBuf::from)
                .collect(),
            | Err(why) => {
                error!("=> {} Filter ignored - {why}", Label::fail());
                vec![]
            }
        },
        | None => paths,
    }
}
/// Return fisrt key/value pair with key that matches pattern
/// ### Example
/// ```rust
/// use acorn_lib::util::find_first;
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
/// Returns the current Git branch name if the `git` command is available and executed successfully.
///
/// This function executes the `git symbolic-ref --short HEAD` command to retrieve the name of
/// the current Git branch. If the command is successful, the branch name is extracted and returned
/// as a `String`. If the command fails or if `git` is not available, the function returns `None`.
pub fn git_branch_name() -> Option<String> {
    if command_exists("git".to_owned()) {
        let args = vec!["symbolic-ref", "--short", "HEAD"];
        let result = cmd("git", args).read();
        match result {
            | Ok(ref value) => {
                let name = match value.clone().split("/").last() {
                    | Some(x) => Some(x.to_string()),
                    | None => None,
                };
                name
            }
            | Err(_) => None,
        }
    } else {
        None
    }
}
/// Returns the default Git branch name if the `git` command is available and executed successfully.
///
/// This function executes the `git symbolic-ref refs/remotes/origin/HEAD --short` command to retrieve
/// the default Git branch name. If the command is successful, the branch name is extracted and returned
/// as a `String`. If the command fails or if `git` is not available, the function returns `None`.
pub fn git_default_branch_name() -> Option<String> {
    if command_exists("git".to_owned()) {
        let args = vec!["symbolic-ref", "refs/remotes/origin/HEAD", "--short"];
        let result = cmd("git", args).read();
        match result {
            | Ok(ref value) => {
                let name = match value.clone().split("/").last() {
                    | Some(x) => Some(x.to_string()),
                    | None => None,
                };
                name
            }
            | Err(_) => None,
        }
    } else {
        None
    }
}
/// Returns a vector of `PathBuf` representing paths to all images found in the given
/// directory and all of its subdirectories.
///
/// # Arguments
///
/// * `root` - A value that can be converted into a `PathBuf` and implements the `Clone` trait. This is the directory in which the search for images is performed.
///
/// # Returns
///
/// A vector of `PathBuf` representing paths to all images found in the given directory and
/// all of its subdirectories. The paths are sorted alphabetically.
///
/// # Notes
/// - Supported image formats are "JPEG", "PNG", "SVG", and "GIF"
pub fn image_paths<P>(root: P) -> Vec<PathBuf>
where
    P: Into<PathBuf> + Clone,
{
    let extensions = ["jpg", "jpeg", "png", "svg", "gif"];
    let mut files = extensions
        .iter()
        .flat_map(|ext| glob(&format!("{}/**/*.{}", root.clone().into().display(), ext)))
        .flat_map(|paths| paths.collect::<Vec<_>>())
        .flatten()
        .collect::<Vec<PathBuf>>();
    files.sort();
    files
}
/// Makes the given file executable.
///
/// # Parameters
///
/// * `path` - A `PathBuf` containing the path to the file to be made executable.
///
/// # Return
///
/// A boolean indicating whether the file is executable after calling this function.
#[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
pub fn make_executable<P>(path: P) -> bool
where
    P: Into<PathBuf> + Clone,
{
    use crate::prelude::PermissionsExt;
    set_permissions(path.clone().into(), Permissions::from_mode(0o755)).unwrap();
    path.into().is_executable()
}
/// Makes the given file executable.
///
/// # Parameters
///
/// * `path` - A `PathBuf` containing the path to the file to be made executable.
///
/// # Return
///
/// A boolean indicating whether the file is executable after calling this function.
#[cfg(windows)]
pub fn make_executable<P>(path: P) -> bool
where
    P: Into<PathBuf> + Clone,
{
    let binary = if extension(&path.clone().into()).is_empty() {
        path.into().with_extension("exe")
    } else {
        path.into()
    };
    println!("{binary:#?}");
    binary.is_executable()
}
/// Returns the absolute path of the parent directory for the given path.
pub fn parent<P>(path: P) -> PathBuf
where
    P: Into<PathBuf> + Clone,
{
    let default = PathBuf::from(".");
    match path.clone().into().canonicalize() {
        | Ok(value) => match value.parent() {
            | Some(value) => value.to_path_buf(),
            | None => {
                warn!("=> {} Resolve parent path", Label::fail());
                default
            }
        },
        | Err(why) => {
            debug!("=> {} Resolve absolute path - {why}", Label::fail());
            match path.into().parent() {
                | Some(value) if !to_absolute_string(value.to_path_buf()).is_empty() => value.to_path_buf(),
                | Some(_) | None => {
                    warn!("=> {} Parent path was empty or could not be resolved", Label::fail());
                    default
                }
            }
        }
    }
}
/// Converts a `PathBuf` into a `String` representation of the **absolute** path.
/// <div class="warning">Uses <code>fs::canonicalize</code>, which might cause problems on Windows</div>
///
/// This function attempts to canonicalize the provided path, which resolves any symbolic links
/// and returns an absolute path. If canonicalization fails, the original path is returned as a string.
///
/// # Arguments
///
/// * `path` - A `PathBuf` representing the file system path to be converted.
///
/// # Returns
///
/// A `String` containing the absolute path if canonicalization succeeds, or the original path as a string otherwise.
pub fn to_absolute_string<P>(path: P) -> String
where
    P: Into<PathBuf> + Clone,
{
    let result = match canonicalize(path.clone().into().as_path()) {
        | Ok(value) => value,
        | Err(_) => path.into(),
    };
    result.display().to_string()
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
/// ### Example
/// ```rust
/// use acorn_lib::util::regex_capture_lookup;
/// let lookup = regex_capture_lookup(
///     r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})",
///     "2023-06-30",
///     vec!["year", "month", "day"]
/// );
/// assert_eq!(lookup["year"], "2023");
/// assert_eq!(lookup["month"], "06");
/// assert_eq!(lookup["day"], "30");
/// ```
pub fn regex_capture_lookup<S>(pattern: S, value: S, names: Vec<S>) -> HashMap<S, String>
where
    S: Into<String> + AsRef<str> + Clone + core::cmp::Eq + core::hash::Hash,
{
    let re = Regex::new(pattern.as_ref()).unwrap();
    let mut lookup: HashMap<S, String> = HashMap::new();
    if let Some(capture_matches) = re.captures_iter(value.as_ref()).last() {
        match capture_matches {
            | Ok(captures) => {
                captures.iter().skip(1).enumerate().for_each(|(index, data)| {
                    if let Some(results) = data {
                        let key = names[index].clone();
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
/// Utility method to employ best practices when using a Reqwest client to make HTTP requests.
pub fn reqwest_request<U>(url: U) -> reqwest::blocking::RequestBuilder
where
    U: reqwest::IntoUrl,
{
    reqwest::blocking::Client::new().get(url).header(USER_AGENT, "rust-web-api-client")
}
/// Converts the given string to snake case.
/// ### Example
/// ```rust
/// use acorn_lib::util::snake_case;
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
/// use acorn_lib::util::suffix;
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
/// Create a new [Tokio](https://tokio.rs/) runtime
/// ### Example
/// ```ignore
/// tokio_runtime().block_on(async {
///     // ...async stuff
/// });
/// ```
pub fn tokio_runtime() -> tokio::runtime::Runtime {
    debug!("=> {} Tokio runtime", Label::using());
    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap()
}
/// Convert a vector of string slices to a vector of strings
pub fn to_string(values: Vec<&str>) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests;
