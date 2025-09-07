//! # Common utilities
//!
//! This module contains common functions and data structures used to build the ACORN command line interface as well support open science endeavors.
//!
use crate::constants::{APPLICATION, ORGANIZATION, QUALIFIER};
use bat::PrettyPrinter;
use bon::Builder;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use console::Emoji;
use data_encoding::HEXUPPER;
use derive_more::Display;
use directories::ProjectDirs;
use duct::cmd;
use fancy_regex::Regex;
use glob::glob;
use is_executable::IsExecutable;
use itertools::Itertools;
use lychee_lib::{CacheStatus, Response, Status};
use nanoid::nanoid;
use owo_colors::{OwoColorize, Style, Styled};
use ring::digest::{Context, SHA256};
use rust_embed::Embed;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use similar::{
    ChangeTag::{self, Delete, Equal, Insert},
    TextDiff,
};
use std::fs::create_dir_all;
use std::fs::File;
use std::io::{copy, BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use titlecase::titlecase;
use tracing::{debug, error, info, warn};
use validator::ValidationErrorsKind;
use which::which;

pub mod citeas;
#[cfg(feature = "cli")]
pub mod cli;

/// SPDX compliant license identifier
///
/// See <https://spdx.org/licenses/>
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum License {
    /// MIT License
    Mit,
    /// Creative Commons
    CreativeCommons,
    /// Unknown license
    Unknown,
}
/// MIME types
///
/// Supports an incomplete list of common MIME types
///
/// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/MIME_types/Common_types>
#[derive(Clone, Debug, Display, PartialEq)]
pub enum MimeType {
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
    /// OpenType Font (OTF)
    #[display("font/otf")]
    Otf,
    /// Portable Network Graphic (PNG)
    #[display("image/png")]
    Png,
    /// Scalable Vector Graphic (SVG)
    #[display("image/svg+xml")]
    Svg,
    /// Plain Text
    ///
    /// Just plain old text
    #[display("text/plain")]
    Text,
    /// YAML Ain't Markup Language (YAML)
    ///
    /// See <https://yaml.org/>
    #[display("application/yaml")]
    Yaml,
    /// Unknown MIME type
    #[display("application/unknown")]
    Unknown,
}
/// Programming languages
///
/// Provides a small subset of common programming languages available for syntax highlighting
#[derive(Clone, Copy, Debug, Display)]
pub enum ProgrammingLanguage {
    /// HyperText Markup Language (HTML)
    #[display("html")]
    Html,
    /// Markdown
    ///
    /// See <https://www.markdownguide.org/>
    #[display("markdown")]
    Markdown,
    /// JavaScript Object Notation (JSON)
    ///
    /// See <https://www.json.org/json-en.html>
    #[display("json")]
    Json,
    /// YAM Ain't Markup Language (YAML)
    ///
    /// See <https://yaml.org/>
    #[display("yaml")]
    Yaml,
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
/// Data structure for holding the result of a link check
#[derive(Builder, Clone, Debug, Display)]
#[builder(start_fn = init)]
#[display("{message}")]
pub struct LinkCheck {
    /// Whether or not the check was successful
    #[builder(default = false)]
    pub success: bool,
    /// HTTP status code
    pub code: Option<String>,
    /// URL
    // TODO: Normalize URL as URI
    pub url: Option<String>,
    /// Message describing the HTTP status code
    pub message: String,
}
/// Data structure for holding the result of a schema validation check
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init)]
pub struct SchemaCheck {
    /// Whether or not the check was successful
    #[builder(default = false)]
    pub success: bool,
    /// Errors found during validation
    pub errors: Option<ValidationErrorsKind>,
    /// Path of file being validated
    // TODO: Normalize path as URI (file://)
    pub path: Option<PathBuf>,
    /// Message related to or description of validation issue (e.g., key name of invalid value, result of validation, etc.)
    pub message: String,
}
/// Semantic version
///
/// see <https://semver.org/>
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
    pub const CAUTION: Emoji<'_, '_> = Emoji("⚠️ ", "!!! ");
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
    pub fn found() -> Styled<&'static &'static str> {
        let style = Style::new().green().on_default_color();
        "FOUND".style(style)
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
impl LinkCheck {
    /// Converts a Lychee response into a LinkCheckResult
    pub fn from_lychee(response: Response) -> Self {
        match response.status() {
            | Status::Ok(code) | Status::Redirected(code) => LinkCheck::init()
                .success(true)
                .code(code.to_string())
                .message("has no HTTP errors".to_string())
                .build(),
            | Status::Cached(status) => match status {
                | CacheStatus::Ok(code) => LinkCheck::init()
                    .success(true)
                    .code(code.to_string())
                    .message("has no HTTP errors".to_string())
                    .build(),
                | CacheStatus::Error(Some(code)) => LinkCheck::init()
                    .success(false)
                    .code(code.to_string())
                    .message("has cached HTTP errors".to_string())
                    .build(),
                | CacheStatus::Unsupported => LinkCheck::init()
                    .success(false)
                    .message("unsupported cached response".to_string())
                    .build(),
                | _ => LinkCheck::init()
                    .success(true)
                    .message("ignored or otherwise successful (cached response)".to_string())
                    .build(),
            },
            | Status::Error(code) => LinkCheck::init()
                .success(false)
                .code(code.to_string())
                .message("has HTTP errors".to_string())
                .build(),
            | Status::Unsupported(why) => LinkCheck::init()
                .success(false)
                .message(format!("unsupported HTTP response - {why}"))
                .build(),
            | Status::UnknownStatusCode(code) => LinkCheck::init()
                .success(false)
                .code(code.to_string())
                .message("unknown HTTP response".to_string())
                .build(),
            | Status::Timeout(_) => LinkCheck::init().success(false).message("HTTP timeout".to_string()).build(),
            | _ => LinkCheck::init()
                .success(true)
                .message("ignored or otherwise successful".to_string())
                .build(),
        }
    }
    /// Perform link check on given URL
    pub async fn run(url: Option<String>) -> LinkCheck {
        match url {
            | Some(url) => {
                let response = lychee_lib::check(url.as_str()).await;
                match response {
                    | Ok(response) => LinkCheck::from_lychee(response).with_url(url),
                    | Err(_) => LinkCheck::init().success(false).url(url).message("unreachable".to_string()).build(),
                }
            }
            | None => LinkCheck::init().success(false).message("missing URL".to_string()).build(),
        }
    }
    /// Print the link check results
    pub fn print(self) {
        let code = match self.code {
            | Some(code) => format!(" ({code})").dimmed().to_string(),
            | None => "".to_string(),
        };
        let url = match self.url {
            | Some(url) => url.underline().italic().to_string(),
            | None => "Missing".italic().to_string(),
        };
        if self.success {
            let message = titlecase(&self.message).green().bold().to_string();
            info!("=> {} \"{url}\" {message}{code}", Label::valid());
        } else {
            let message = titlecase(&self.message).red().bold().to_string();
            error!("=> {} \"{url}\" {message}{code}", Label::invalid());
        }
    }
    /// Returns a new LinkCheckResult with the given URL
    pub fn with_url(self, value: String) -> Self {
        LinkCheck::init()
            .success(self.success)
            .url(value)
            .maybe_code(self.code)
            .message(self.message)
            .build()
    }
}
impl MimeType {
    /// Returns the file type as a string
    pub fn file_type(self) -> String {
        match self {
            | MimeType::Csv => "csv",
            | MimeType::Jpeg => "jpeg",
            | MimeType::Json => "json",
            | MimeType::LdJson => "jsonld",
            | MimeType::Otf => "otf",
            | MimeType::Png => "png",
            | MimeType::Svg => "svg",
            | MimeType::Text => "txt",
            | MimeType::Yaml => "yaml",
            | _ => "unknown-file-type",
        }
        .to_string()
    }
    /// Returns a `MimeType` value based on the file extension of the given file name.
    ///
    /// Uses [`MimeType::from_string`].
    pub fn from_path<P>(value: P) -> MimeType
    where
        P: Into<PathBuf>,
    {
        MimeType::from_string(path_to_string(value.into()))
    }
    /// Returns a `MimeType` value based on the file extension of the given file name.
    ///
    /// # Supported MIME types
    ///
    /// * `csv` - `text/csv`
    /// * `jpg` - `image/jpeg`
    /// * `jpeg` - `image/jpeg`
    /// * `json` - `application/json`
    /// * `jsonld` - `application/ld+json`
    /// * `otf` - `font/otf`
    /// * `png` - `image/png`
    /// * `svg` - `image/svg+xml`
    /// * `txt` - `text/plain`
    /// * `yaml` - `application/yaml`
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
            | "otf" => MimeType::Otf,
            | "png" => MimeType::Png,
            | "svg" => MimeType::Svg,
            | "txt" => MimeType::Text,
            | "yml" | "yaml" => MimeType::Yaml,
            | _ => MimeType::Unknown,
        }
    }
}
impl SchemaCheck {
    /// Returns the number of errors
    pub fn issue_count(&self) -> usize {
        if let Some(errors) = &self.errors {
            match errors.clone() {
                | ValidationErrorsKind::Field(_) => 1,
                | ValidationErrorsKind::Struct(errors) => errors.into_errors().len(),
                | ValidationErrorsKind::List(_) => 0,
            }
        } else {
            0
        }
    }
    /// Print the schema check results
    pub fn print(self) {
        let path = self.clone().path.unwrap().display().to_string();
        if self.success {
            info!("=> {} {} has {}", Label::pass(), path, "no schema validation issues".green().bold());
        } else {
            let count = self.issue_count();
            error!(
                "=> {} Found {} schema validation issue{} in {}: \n{:#?}",
                Label::fail(),
                count.red(),
                suffix(count),
                path.italic().underline(),
                self.errors.unwrap()
            );
        }
    }
    /// Returns a new LinkCheckResult with the given URL
    pub fn with_path(self, value: PathBuf) -> Self {
        SchemaCheck::init()
            .success(self.success)
            .maybe_errors(self.errors)
            .path(value)
            .message(self.message)
            .build()
    }
}
impl SemanticVersion {
    /// Returns a `SemanticVersion` value based on the output of the `--version` command-line flag
    /// of the given executable name.
    ///
    /// <div class="warning">this function only supports commands that provide a `--version` flag</div>
    pub fn from_command<S>(name: S) -> Option<SemanticVersion>
    where
        S: Into<String> + duct::IntoExecutablePath + std::marker::Copy,
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
/// Get SHA256 hash of a file
///
/// See <https://rust-lang-nursery.github.io/rust-cookbook/cryptography/hashing.html>
pub fn checksum<P>(path: P) -> String
where
    P: Into<PathBuf>,
{
    let value = path.into();
    match File::open(value.clone()) {
        | Ok(file) => {
            let mut buffer = [0; 1024];
            let mut context = Context::new(&SHA256);
            let mut reader = BufReader::new(file);
            loop {
                let count = reader.read(&mut buffer).unwrap();
                if count == 0 {
                    break;
                }
                context.update(&buffer[..count]);
            }
            let digest = context.finish();
            let result = HEXUPPER.encode(digest.as_ref());
            result.to_lowercase()
        }
        | Err(err) => {
            error!(error = err.to_string(), path = path_to_string(value), "=> {} Read file", Label::fail());
            "".to_string()
        }
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
    S: Into<String> + AsRef<std::ffi::OsStr> + tracing::Value,
{
    match which(&name) {
        | Ok(value) => {
            let path = path_to_string(value.clone());
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
/// Downloads a binary file from the given URL to the destination path.
///
/// # Arguments
///
/// * `url` - A string slice representing the URL of the binary to download.
/// * `destination` - A path to the root directory where the file should be saved.
///
/// # Returns
///
/// A `Result` containing a `PathBuf` to the downloaded file on success, or a string error message on failure.
pub fn download_binary<S, P>(url: S, destination: P) -> Result<PathBuf, String>
where
    S: Into<String> + Clone + std::marker::Copy,
    P: Into<PathBuf> + Clone,
{
    async fn download<P>(url: String, destination: P) -> Result<(), String>
    where
        P: Into<PathBuf>,
    {
        let client = reqwest::Client::new();
        let response = client.get(url.clone()).send();
        let filename = PathBuf::from(url.clone()).file_name().unwrap().to_str().unwrap().to_string();
        match response.await {
            | Ok(data) => match data.bytes().await {
                | Ok(content) => {
                    let mut output = File::create(destination.into().join(filename.clone())).unwrap();
                    let _ = copy(&mut Cursor::new(content.clone()), &mut output);
                    debug!(filename = filename, "=> {} Downloaded", Label::output());
                    Ok(())
                }
                | Err(_) => Err(format!("No content downloaded from {url}")),
            },
            | Err(_) => Err(format!("Failed to download {url}")),
        }
    }
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let _ = runtime.block_on(download(url.into(), destination.clone()));
    let filename = PathBuf::from(url.into()).file_name().unwrap().to_str().unwrap().to_string();
    Ok(destination.into().join(filename))
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
/// Returns a vector of `PathBuf` containing all files in a directory that match the given extension and do not match the ignore pattern.
///
/// # Arguments
///
/// * `path` - A `PathBuf` to the directory to search.
/// * `extension` - An `Option` containing a string slice representing the file extension to search for.
/// * `ignore` - An `Option` containing a string representing a regex pattern to ignore files matching.
///
/// # Returns
///
/// A `Vec` containing `PathBuf` values of all files in the given directory that match the given extension and do not match the ignore pattern.
pub fn files_all(path: PathBuf, extensions: Option<Vec<&str>>, ignore: Option<String>) -> Vec<PathBuf> {
    fn paths_to_vec(paths: glob::Paths) -> Vec<PathBuf> {
        paths.collect::<Vec<_>>().into_iter().filter_map(|x| x.ok()).collect::<Vec<_>>()
    }
    fn pattern(path: PathBuf, extension: &str) -> String {
        let ext = &extension.to_lowercase();
        let result = format!("{}/**/*.{}", path_to_string(path), ext);
        debug!("=> {} {result}", Label::using());
        result
    }
    if path.is_dir() {
        let paths = match extensions {
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
            | None => match glob(&format!("{}/**/*", path_to_string(path))) {
                | Ok(paths) => paths_to_vec(paths),
                | Err(why) => {
                    error!("=> {} Get all files (Glob) - {why}", Label::fail());
                    vec![]
                }
            },
        };
        match ignore {
            | Some(ignore_pattern) => match Regex::new(&ignore_pattern) {
                | Ok(re) => paths
                    .into_iter()
                    .map(path_to_string)
                    .filter(|x| !re.is_match(x).unwrap())
                    .map(PathBuf::from)
                    .collect(),
                | Err(why) => {
                    error!("=> {} Get all files (Regex) - {why}", Label::fail());
                    vec![]
                }
            },
            | None => paths,
        }
    } else {
        if extensions.is_some() {
            warn!(
                path = path_to_string(path.clone()),
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
        filter_git_command_result(result, extensions)
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
        filter_git_command_result(result, extensions)
    } else {
        vec![]
    }
}
// TODO: Add support for multiple extensions
fn filter_git_command_result(result: Result<String, std::io::Error>, extensions: Option<Vec<&str>>) -> Vec<PathBuf> {
    match result {
        | Ok(value) => match extensions {
            | Some(values) => value
                .to_lowercase()
                .split("\n")
                .map(PathBuf::from)
                .filter(|path| {
                    println!("{values:#?}");
                    values.iter().any(|ext| MimeType::from_path(path).to_string() == *ext)
                })
                .collect::<Vec<_>>(),
            | None => value.to_lowercase().split("\n").map(PathBuf::from).collect::<Vec<_>>(),
        },
        | Err(_) => vec![],
    }
}
/// Return fisrt key/value pair with key that matches pattern
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
/// # Platform support
///
/// Platforms that support this function are:
///
/// * Unix
/// * WASI
/// * Redox
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
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path.clone().into(), std::fs::Permissions::from_mode(0o755)).unwrap();
    path.into().is_executable()
}
/// Makes the given file executable.
///
/// # Platform support
///
/// Platforms that support this function are:
///
/// * Windows
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
    // TODO: Add windows support...pass through?
    path.into().is_executable()
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
                | Some(value) if !path_to_string(value.to_path_buf()).is_empty() => value.to_path_buf(),
                | Some(_) | None => {
                    warn!("=> {} Parent path was empty or could not be resolved", Label::fail());
                    default
                }
            }
        }
    }
}
/// Converts a `PathBuf` into a `String` representation of the absolute path.
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
pub fn path_to_string(path: PathBuf) -> String {
    // NOTE: fs::canonicalize might cause problems on Windows
    let result = match std::fs::canonicalize(path.as_path()) {
        | Ok(value) => value,
        | Err(_) => path,
    };
    result.to_str().unwrap().to_string()
}
/// Prints `text` to stdout using syntax highlighting for the specified `syntax`.
///
/// `highlight` is an iterator of line numbers to highlight in the output.
pub fn pretty_print<I: IntoIterator<Item = usize>>(text: &str, syntax: ProgrammingLanguage, highlight: I) {
    let input = format!("{text}\n");
    let language = syntax.to_string();
    let mut printer = PrettyPrinter::new();
    printer
        .input_from_bytes(input.as_bytes())
        .theme("zenburn")
        .language(&language)
        .line_numbers(true);
    for line in highlight {
        printer.highlight(line);
    }
    printer.print().unwrap();
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
/// Reads the given file and returns its contents as a string.
///
/// # Parameters
///
/// * `path` - A `PathBuf` or string slice containing the path to the file to be read.
///
/// # Return
///
/// A `Result` containing the contents of the file as a string if the file is readable, or an
/// `std::io::Error` otherwise.
pub fn read_file<P>(path: P) -> Result<String, std::io::Error>
where
    P: Into<PathBuf> + Clone,
{
    let mut content = String::new();
    let _ = match File::open(path.clone().into()) {
        | Ok(mut file) => {
            debug!(path = path_to_string(path.into()), "=> {}", Label::read());
            file.read_to_string(&mut content)
        }
        | Err(why) => {
            error!(path = path_to_string(path.into()), "=> {} Read file", Label::fail());
            Err(why)
        }
    };
    Ok(content)
}
/// Returns path to a folder in the operating system's cache directory that is unique to the given
/// `namespace` with a random UUID as the name of the final folder.
///
/// The folder is ***not*** created.
///
/// Used primarily by ACORN CLI where `namespace` is of a subcommand task. e.g. "check", "extract", etc.
///
/// # Arguments
///
/// * `namespace` - A string slice representing the name of the namespace.
/// * `default` - An optional `PathBuf` to use as the root directory instead of the cache directory.
///
/// # Returns
///
/// A `PathBuf` to the folder.
pub fn standard_project_folder(namespace: &str, default: Option<PathBuf>) -> PathBuf {
    let root = match default {
        | Some(value) => value,
        | None => match ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
            | Some(dirs) => dirs.cache_dir().join(namespace).to_path_buf(),
            | None => PathBuf::from(format!("./{namespace}")),
        },
    };
    match create_dir_all(root.clone()) {
        | Ok(_) => {}
        | Err(why) => error!(directory = path_to_string(root.clone()), "=> {} Create - {}", Label::fail(), why),
    };
    root.join(generate_guid())
}
/// Returns "s" if the given value is not 1, otherwise returns an empty string.
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
///     ...async stuff
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
/// Writes the given content to a file at the given path.
///
/// # Arguments
///
/// * `path` - A `PathBuf` or string slice containing the path to the file to be written.
/// * `content` - A `String` containing the content to be written to the file.
///
/// # Return
///
/// A `Result` containing a unit value if the file is written successfully, or an
/// `std::io::Error` otherwise.
pub fn write_file<P>(path: P, content: String) -> Result<(), std::io::Error>
where
    P: Into<PathBuf>,
{
    match File::create(path.into().clone()) {
        | Ok(mut file) => {
            file.write_all(content.as_bytes()).unwrap();
            file.flush()
        }
        | Err(why) => {
            error!("=> {} Cannot create file - {why}", Label::fail());
            Err(why)
        }
    }
}

#[cfg(test)]
mod tests;
