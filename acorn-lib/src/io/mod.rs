//! # IO Utilities
//!
//! Module to isolate input/output operations to enhance portability
use crate::prelude::{create_dir_all, io, var, BufReader, Cursor, File, PathBuf, Read, Write};
#[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
use crate::prelude::{set_permissions, Permissions, PermissionsExt};
use crate::util::constants::{APPLICATION, ORGANIZATION, QUALIFIER};
#[cfg(windows)]
use crate::util::file_extension;
use crate::util::{command_exists, filter_git_command_result, generate_guid, suffix, Label, MimeType, ToAbsoluteString};
use data_encoding::HEXUPPER;
use directories::ProjectDirs;
use duct::cmd;
use is_executable::IsExecutable;
use reqwest::header::USER_AGENT;
use ring::digest::{Context, SHA256};
use serde::Deserialize;
use tokio::runtime::{Builder, Runtime};
use tracing::{debug, error, info};
use zip::ZipArchive;

pub mod citeas;
pub mod raid;

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
/// - Uses [`tokio_runtime`] for asynchronous operations.
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
        let response = client.get(url.clone()).header(USER_AGENT, "rust-web-api-client").send();
        let filename = PathBuf::from(url.clone()).file_name().unwrap().to_str().unwrap().to_string();
        match response.await {
            | Ok(data) => match data.bytes().await {
                | Ok(content) => {
                    let mut output = File::create(destination.into().join(filename.clone())).unwrap();
                    let _ = io::copy(&mut Cursor::new(content.clone()), &mut output);
                    debug!(filename = filename, "=> {} Downloaded", Label::output());
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
/// use acorn_lib::util::checksum;
///
/// let checksum = checksum("path/to/file");
/// assert!(checksum.is_some());
/// ```
pub fn file_checksum<P>(path: P) -> Option<String>
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
    println!("{binary:#?}");
    binary.is_executable()
}
/// Utility method to employ best practices when using a Reqwest client to make HTTP GET requests.
pub fn network_get_request<U>(url: U) -> reqwest::blocking::RequestBuilder
where
    U: reqwest::IntoUrl,
{
    reqwest::blocking::Client::new().get(url).header(USER_AGENT, "rust-web-api-client")
}
/// Utility method to employ best practices when using a Reqwest client to make HTTP POST requests.
pub fn network_post_request<U>(url: U) -> reqwest::blocking::RequestBuilder
where
    U: reqwest::IntoUrl,
{
    reqwest::blocking::Client::new().post(url).header(USER_AGENT, "rust-web-api-client")
}
/// Utility method to employ best practices when using a Reqwest client to make HTTP PUT requests.
pub fn network_put_request<U>(url: U) -> reqwest::blocking::RequestBuilder
where
    U: reqwest::IntoUrl,
{
    reqwest::blocking::Client::new().put(url).header(USER_AGENT, "rust-web-api-client")
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
        | Err(why) => error!(directory = root.clone().to_absolute_string(), "=> {} Create - {}", Label::fail(), why),
    };
    root.join(generate_guid())
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
