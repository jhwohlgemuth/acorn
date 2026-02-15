//! # IO Utilities
//!
//! Module to isolate input/output operations to enhance portability
//!
//! ## Example Uses
//!
//! ### Perform file read and write operations
//! ```ignore
//! use acorn::util::{checksum, read_file, write_file};
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
use crate::fail;
use crate::prelude::{canonicalize, create_dir_all, io, var, BufReader, Cursor, Error, File, PathBuf, Read, Write};
#[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
use crate::prelude::{set_permissions, Permissions, PermissionsExt};
use crate::util::constants::{APPLICATION, ORGANIZATION, QUALIFIER};
#[cfg(windows)]
use crate::util::file_extension;
use crate::util::{generate_guid, suffix, Label, MimeType, SemanticVersion, ToAbsoluteString, ToStrings};
use core::time::Duration;
use data_encoding::HEXUPPER;
use directories::ProjectDirs;
use duct::cmd;
use fancy_regex::Regex;
use glob::glob;
use is_executable::IsExecutable;
use itertools::Itertools;
use reqwest::header::USER_AGENT;
use ring::digest::{Context, SHA256};
use serde::Deserialize;
use tokio::runtime::{Builder, Runtime};
use tracing::{debug, error, info, trace, warn};
use which::which;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub mod api;
pub mod bagit;
pub mod raid;

/// See recommendation at <https://support.datacite.org/docs/api#api-versions>
#[cfg(feature = "std")]
const ACORN_USER_AGENT: &str = concat!("ACORN/", env!("CARGO_PKG_VERSION"), " (https://acorn.ornl.gov; mailto:research@ornl.gov)");
#[cfg(not(feature = "std"))]
const ACORN_USER_AGENT: &str = "ACORN (https://acorn.ornl.gov; mailto:research@ornl.gov)";

/// Add `from_command` trait to `SemanticVersion`
pub trait FromCommand {
    /// Convert a command name to a `SemanticVersion` value
    fn from_command<S>(name: S) -> Option<Self>
    where
        Self: Sized,
        S: Into<String> + duct::IntoExecutablePath + core::marker::Copy;
}
/// Add `from_path` trait to a value (like `MimeType`)
pub trait FromPath {
    /// Convert a path to a value
    fn from_path<P>(value: P) -> Self
    where
        P: Into<PathBuf>;
}
/// Trait for I/O operations such as read and write
pub trait InputOutput: Sized {
    /// Read data from specified file path
    fn read(path: impl Into<PathBuf>) -> Result<Self, Error>;
    /// Read data from specified JSON file path
    fn read_json(path: PathBuf) -> Result<Self, Error>;
    /// Read data as Markdown from specified path
    fn read_markdown(_path: PathBuf) -> Option<Self> {
        None
    }
    /// Read data from specified YAML file path
    fn read_yaml(path: PathBuf) -> Result<Self, Error>;
    /// Write data to specified path
    fn write(&self, path: impl Into<PathBuf>) -> Result<(), Error>;
    /// Write data as JSON to specified path
    fn write_json(&self, path: impl Into<PathBuf>) -> Result<(), Error>;
    /// Write data as Markdown (MD) to specified path
    fn write_markdown(&self, _path: impl Into<PathBuf>) -> Result<(), Error> {
        Ok(())
    }
    /// Write data as YAML to specified path
    fn write_yaml(&self, path: impl Into<PathBuf>) -> Result<(), Error>;
}
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
/// Struct for adding ToStringList functionality
pub struct StringList<'a>(pub &'a Vec<PathBuf>);
impl FromCommand for SemanticVersion {
    /// Returns a `SemanticVersion` value based on the output of the `--version` command-line flag
    /// of the given executable name. Tested with [cargo](https://rustup.rs/), [git](https://git-scm.com/book/en/v2/Getting-Started-The-Command-Line), and [pandoc](https://pandoc.org/).
    ///
    /// <div class="warning">this function only supports commands that provide a `--version` flag</div>
    ///
    /// ### Example
    /// ```ignore
    /// use acorn::schema::validate::SemanticVersion;
    ///
    /// let version = SemanticVersion::from_command("cargo").to_string();
    /// assert_eq!(version, "1.90.0");
    /// ```
    #[cfg(feature = "std")]
    fn from_command<S>(name: S) -> Option<SemanticVersion>
    where
        S: Into<String> + duct::IntoExecutablePath + core::marker::Copy,
    {
        if command_exists(name.into()) {
            let result = cmd(name, vec!["--version"]).read();
            match result {
                | Ok(value) => {
                    let first_line = value.lines().collect::<Vec<_>>().first().cloned();
                    match first_line {
                        | Some(line) => Some(SemanticVersion::from(line)),
                        | None => None,
                    }
                }
                | Err(_) => None,
            }
        } else {
            None
        }
    }
}
impl FromPath for MimeType {
    /// Returns a [`MimeType`] value based on the file extension of the given file name.
    ///
    /// Uses [`MimeType::from_string`].
    ///
    /// ```ignore
    /// use acorn::util::MimeType;
    /// use std::path::PathBuf;
    ///
    /// let mime = MimeType::from_path(PathBuf::from("test.cff"));
    /// assert_eq!(mime, MimeType::Yaml);
    /// ```
    fn from_path<P>(value: P) -> MimeType
    where
        P: Into<PathBuf>,
    {
        MimeType::from(value.into().display().to_string())
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
impl ToAbsoluteString for PathBuf {
    fn to_absolute_string(&self) -> String {
        to_absolute_string(self.clone())
    }
}
/// Creates zip archive from directory
pub fn archive(path: PathBuf, destination: Option<PathBuf>) -> Result<PathBuf, Error> {
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let zip_file_path = match destination {
        | Some(value) => value,
        | None => path.with_extension("zip"),
    };
    let zip_file = match File::create(&zip_file_path) {
        | Ok(zip_file) => Some(ZipWriter::new(zip_file)),
        | Err(why) => {
            error!(
                file = path.clone().to_absolute_string(),
                "=> {} Create zip archive - {why}",
                Label::fail()
            );
            None
        }
    };
    if let Some(mut zip) = zip_file {
        let files = files_all(path.clone(), None).into_iter().filter(|x| x.is_file());
        for file_path in files {
            if let Ok(file) = File::open(file_path.clone()) {
                let name = match path.canonicalize() {
                    | Ok(relative) => file_path.strip_prefix(relative).unwrap_or_else(|_| &file_path),
                    | Err(_) => &file_path,
                };
                trace!(
                    file = name.to_path_buf().to_absolute_string(),
                    "=> {} Add file to archive",
                    Label::using()
                );
                match zip.start_file_from_path(name, options) {
                    | Ok(_) => {
                        let mut buffer = Vec::new();
                        match io::copy(&mut file.take(u64::MAX), &mut buffer) {
                            | Ok(_) => match zip.write_all(&buffer) {
                                | Ok(_) => {}
                                | Err(why) => {
                                    error!(file = file_path.to_absolute_string(), "=> {} Write zip archive - {why}", Label::fail())
                                }
                            },
                            | Err(why) => {
                                error!("=> {} Copy buffer - {why}", Label::fail())
                            }
                        }
                    }
                    | Err(why) => {
                        error!(file = file_path.to_absolute_string(), "=> {} Start zip archive - {why}", Label::fail());
                    }
                }
            }
        }
        match zip.finish() {
            | Ok(_) => Ok(zip_file_path),
            | Err(why) => {
                error!(file = path.to_absolute_string(), "=> {} Finish zip archive - {why}", Label::fail());
                Err(why.into())
            }
        }
    } else {
        Err(io::Error::other("Unable to create zip archive"))
    }
}
/// Create a new [Tokio](https://tokio.rs/) runtime
/// ### Example
/// ```ignore
/// async_runtime().block_on(async {
///     // ...async stuff
/// });
/// ```
pub fn async_runtime() -> Runtime {
    debug!("=> {} Async runtime", Label::using());
    Builder::new_current_thread().enable_all().build().unwrap()
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
    S: Into<String>,
{
    let command = name.into();
    match which(&command) {
        | Ok(value) => {
            let path = value.clone().to_absolute_string();
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
            warn!("=> {} Command {}", Label::not_found(), command);
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
///
/// # Notes
/// - Uses [`async_runtime`] for asynchronous operations.
pub fn download_binary<S, P>(url: S, destination: P) -> Result<PathBuf, String>
where
    S: Into<String> + Clone + core::marker::Copy,
    P: Into<PathBuf> + Clone,
{
    async fn download<P>(url: String, destination: P) -> Result<(), String>
    where
        P: Into<PathBuf>,
    {
        let client = reqwest::Client::new();
        let response = client.get(url.clone()).header(USER_AGENT, ACORN_USER_AGENT).send();
        let filename = PathBuf::from(url.clone()).file_name().unwrap().to_str().unwrap().to_string();
        match response.await {
            | Ok(data) => match data.bytes().await {
                | Ok(content) => {
                    let mut output = File::create(destination.into().join(filename.clone())).unwrap();
                    let _ = io::copy(&mut Cursor::new(content.clone()), &mut output);
                    debug!(filename, "=> {} Downloaded", Label::output());
                    Ok(())
                }
                | Err(_) => Err(format!("No content downloaded from {url}")),
            },
            | Err(_) => Err(format!("Failed to download {url}")),
        }
    }
    let runtime = async_runtime();
    let _ = runtime.block_on(download(url.into(), destination.clone()));
    let filename = PathBuf::from(url.into()).file_name().unwrap().to_str().unwrap().to_string();
    Ok(destination.into().join(filename))
}
/// Extract zip archive
/// ### Note
/// If `destination` is not provided, the extracted files will be saved in a folder named "extract" an OS-specific cache location.
pub fn extract_zip(path: PathBuf, destination: Option<PathBuf>) -> Result<PathBuf, io::Error> {
    let root = match destination {
        | Some(value) => value,
        | None => standard_project_folder("extract", None),
    };
    match File::open(path.clone()) {
        | Ok(zip_file) => match ZipArchive::new(zip_file) {
            | Ok(mut archive) => {
                let mut success = true;
                for index in 0..archive.len() {
                    match archive.by_index(index) {
                        | Ok(mut file) => {
                            let target = root.join(file.name());
                            if let Some(parent) = target.parent() {
                                match create_dir_all(parent) {
                                    | Ok(_) => {}
                                    | Err(why) => error!(path = parent.to_path_buf().to_absolute_string(), "=> {} Create - {}", Label::fail(), why),
                                }
                            }
                            if let Ok(mut output_file) = File::create(&target) {
                                match io::copy(&mut file, &mut output_file) {
                                    | Ok(_) => {}
                                    | Err(why) => {
                                        error!(path = target.to_absolute_string(), "=> {} Copy file content - {why}", Label::fail());
                                        success = false
                                    }
                                }
                            } else {
                                error!(path = target.to_absolute_string(), "=> {} Create file", Label::fail());
                                success = false
                            }
                        }
                        | Err(why) => {
                            error!(path = path.to_absolute_string(), "=> {} Extract file - {why}", Label::fail());
                            success = false
                        }
                    }
                }
                if success {
                    info!(path = root.to_absolute_string(), "=> {} Extract zip archive", Label::pass());
                    Ok(root)
                } else {
                    error!(path = root.to_absolute_string(), "=> {} Extract zip archive", Label::fail());
                    Err(io::Error::from(io::ErrorKind::Other))
                }
            }
            | Err(why) => {
                error!(path = path.to_absolute_string(), "=> {} Read zip archive - {why}", Label::fail());
                Err(why.into())
            }
        },
        | Err(why) => {
            error!(path = path.to_absolute_string(), "=> {} Read file - {why}", Label::fail());
            Err(why)
        }
    }
}
/// Get SHA256 hash of a file
///
/// See <https://rust-lang-nursery.github.io/rust-cookbook/cryptography/hashing.html>
///
/// ### Example
/// ```ignore
/// use ring::digest::SHA512;
/// use acorn::io::file_checksum;
///
/// let checksum = file_checksum("path/to/file", Some(&SHA512));
/// assert!(checksum.is_some());
/// ```
pub fn file_checksum<P>(path: P, algorithm: Option<&'static ring::digest::Algorithm>) -> Option<String>
where
    P: Into<PathBuf>,
{
    let value = path.into();
    let digest_algorithm = algorithm.unwrap_or(&SHA256);
    match File::open(value.clone()) {
        | Ok(file) => {
            let mut buffer = [0; 1024];
            let mut context = Context::new(digest_algorithm);
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
            Some(result.to_lowercase())
        }
        | Err(err) => {
            error!(
                error = err.to_string(),
                path = value.to_absolute_string(),
                "=> {} Read file",
                Label::fail()
            );
            None
        }
    }
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
        let result = format!("{}/**/*.{}", path.to_absolute_string(), ext);
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
            | None => match glob(&format!("{}/**/*", path.to_absolute_string())) {
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
                path = path.clone().to_absolute_string(),
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
    match network_get_request(url).send() {
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
/// Filter Git command result by file extension
pub fn filter_git_command_result(value: String, extensions: Option<Vec<&str>>) -> Vec<PathBuf> {
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
/// use acorn::io::filter_ignored;
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
/// Returns the size of a folder in bytes
pub fn folder_size<P: Into<PathBuf>>(path: P) -> u64 {
    files_all(path.into(), None)
        .into_iter()
        .filter_map(|p| p.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
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
    let binary = match file_extension(path.clone().into().to_absolute_string()) {
        | None => path.into().with_extension("exe"),
        | _ => path.into(),
    };
    debug!("=> {} {binary:#?}", Label::using());
    binary.is_executable()
}
/// Create a configured HTTP client with proxy support and appropriate timeouts
fn create_sync_http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}
/// Utility method to employ best practices when using a Reqwest client to make HTTP GET requests.
pub fn network_get_request<U>(url: U) -> reqwest::blocking::RequestBuilder
where
    U: reqwest::IntoUrl,
{
    create_sync_http_client().get(url).header(USER_AGENT, ACORN_USER_AGENT)
}
/// Utility method to employ best practices when using a Reqwest client to make HTTP POST requests.
pub fn network_post_request<U>(url: U) -> reqwest::blocking::RequestBuilder
where
    U: reqwest::IntoUrl,
{
    create_sync_http_client().post(url).header(USER_AGENT, ACORN_USER_AGENT)
}
/// Utility method to employ best practices when using a Reqwest client to make HTTP PUT requests.
pub fn network_put_request<U>(url: U) -> reqwest::blocking::RequestBuilder
where
    U: reqwest::IntoUrl,
{
    create_sync_http_client().put(url).header(USER_AGENT, ACORN_USER_AGENT)
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
                | Some(value) if !value.to_path_buf().to_absolute_string().is_empty() => value.to_path_buf(),
                | Some(_) | None => {
                    warn!("=> {} Parent path was empty or could not be resolved", Label::fail());
                    default
                }
            }
        }
    }
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
pub fn read_file<P>(path: P) -> Result<String, io::Error>
where
    P: Into<PathBuf> + Clone,
{
    let mut content = String::new();
    let _ = match File::open(path.clone().into()) {
        | Ok(mut file) => {
            debug!(path = path.into().to_absolute_string(), "=> {}", Label::read());
            file.read_to_string(&mut content)
        }
        | Err(why) => {
            error!(path = path.into().to_absolute_string(), "=> {} Read file", Label::fail());
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
        | Err(why) => error!(directory = root.clone().to_absolute_string(), "=> {} Create - {why}", Label::fail()),
    };
    root.join(generate_guid())
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
pub fn write_file<P>(path: P, content: String) -> Result<(), io::Error>
where
    P: Into<PathBuf>,
{
    match File::create(path.into().clone()) {
        | Ok(mut file) => match file.write_all(content.as_bytes()) {
            | Ok(_) => file.flush(),
            | Err(why) => {
                fail!("Write file - {}", why);
                Err(why)
            }
        },
        | Err(why) => {
            error!("=> {} Cannot create file - {why}", Label::fail());
            Err(why)
        }
    }
}

#[cfg(test)]
mod tests;
