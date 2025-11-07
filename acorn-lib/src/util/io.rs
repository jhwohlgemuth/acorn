//! # IO Utilities
//!
//! Module to isolate input/output operations to enhance portability
use crate::constants::{APPLICATION, ORGANIZATION, QUALIFIER};
use crate::prelude::{create_dir_all, exit, io, BufReader, Cursor, File, PathBuf, Read, Write};
use crate::util::{generate_guid, tokio_runtime, Label, ToAbsoluteString};
use data_encoding::HEXUPPER;
use directories::ProjectDirs;
use reqwest::header::USER_AGENT;
use ring::digest::{Context, SHA256};
use tracing::{debug, error, info};
use zip::ZipArchive;

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
    let runtime = tokio_runtime();
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
                                        exit(exitcode::UNAVAILABLE);
                                    }
                                }
                            } else {
                                error!(path = target.to_absolute_string(), "=> {} Create file", Label::fail());
                                exit(exitcode::UNAVAILABLE);
                            }
                        }
                        | Err(why) => {
                            error!(path = path.to_absolute_string(), "=> {} Extract file - {why}", Label::fail());
                            exit(exitcode::UNAVAILABLE);
                        }
                    }
                }
                info!(path = root.to_absolute_string(), "=> {} Extract zip archive", Label::pass());
                Ok(root)
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
