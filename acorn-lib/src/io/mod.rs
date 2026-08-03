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
#[cfg(unix)]
use crate::prelude;
use crate::prelude::{
    canonicalize, consts, create_dir_all, current_dir, io, remove_file, temp_dir, var, var_os, write, BufReader, CommandOutput, Cursor, File,
    HashSet, OpenOptions, OsString, Path, PathBuf, Read, Write,
};
#[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
use crate::prelude::{set_permissions, OpenOptionsExt, Permissions, PermissionsExt};
#[cfg(windows)]
use crate::prelude::{symlink_dir, symlink_file};
use crate::util::constants::app::{APPLICATION, DOCKER_SOCKET, LARGE_FILE_THRESHOLD_BYTES, ORGANIZATION, QUALIFIER};
#[cfg(windows)]
use crate::util::file_extension;
use crate::util::{generate_guid, suffix, Checksum, ChecksumAlgorithm, Label, MimeType, SemanticVersion, StringConversion, ToStrings};
use crate::{args, cmd, Location};
use chrono::Utc;
use color_eyre::eyre::{eyre, Report, Result};
use core::fmt;
use core::pin::Pin;
use core::time::Duration;
use data_encoding::HEXUPPER;
use directories::{BaseDirs, ProjectDirs};
use fancy_regex::Regex;
use fluent_uri::Uri;
use futures::stream::{self, StreamExt};
use futures::Future;
use glob::glob;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use is_executable::IsExecutable;
use jsonc_parser::{cst::CstInputValue, parse_to_serde_value, ParseOptions};
use lazy_static::lazy_static;
use nanoid::nanoid;
use rand::rngs::OsRng;
use ring::digest::{Context, SHA256};
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
use rsa::{RsaPrivateKey, RsaPublicKey};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use strum::EnumIs;
use tokio::runtime::{Builder, Runtime};
use tracing::{debug, error, info, trace, warn};
use which::which;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub mod api;
pub mod bagit;
#[cfg(feature = "chart")]
pub mod chart;
pub mod config;
pub mod database;
pub mod docx;
pub mod download;
pub mod http;
#[cfg(feature = "agentic")]
pub mod mcp;
pub mod model;
#[cfg(feature = "powerpoint")]
pub mod powerpoint;
pub mod source;
pub mod sync;

pub use jsonc_parser::cst::CstRootNode;
pub use model::ModelListFile;
pub use source::{Source, SourceAction};

/// Utility type alias for I/O operation futures (e.g., read, write, copy, etc.)
pub type ApiFuture<'a> = Pin<Box<dyn Future<Output = ApiResult<()>> + 'a>>;
/// Utility type alias for I/O operation results (e.g., read, write, copy, etc.)
pub type ApiResult<T> = Result<T, Report>;
/// An RSA key pair consisting of a private key and its corresponding public key
pub type RsaKeyPair = (rsa::RsaPrivateKey, rsa::RsaPublicKey);
pub(crate) struct CstValue<'a>(pub(crate) &'a Value);
/// Add `from_command` trait to `SemanticVersion`
pub trait FromCommand {
    /// Convert a command name to a `SemanticVersion` value
    fn from_command<S>(name: S) -> Option<Self>
    where
        Self: Sized,
        S: Into<String> + core::marker::Copy;
}
/// Add `from_path` trait to a value (like `MimeType`)
pub trait FromPath {
    /// Convert a path to a value
    fn from_path<P>(value: &P) -> Self
    where
        P: AsRef<Path> + ?Sized;
}
/// Trait for I/O operations such as read and write
pub trait InputOutput: Sized {
    /// Read data from specified file path
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self>;
    /// Read data as CFF from specified path
    fn read_cff(_path: impl Into<PathBuf>) -> ApiResult<Self> {
        Err(eyre!("CFF read not implemented for this type"))
    }
    /// Read data from specified JSON file path
    fn read_json(path: PathBuf) -> ApiResult<Self>;
    /// Read data from specified JSONC file path
    fn read_jsonc(_path: PathBuf) -> ApiResult<Self> {
        Err(eyre!("JSONC read not implemented for this type"))
    }
    /// Read data as Markdown from specified path
    fn read_markdown(_path: PathBuf) -> Option<Self> {
        None
    }
    /// Read data from specified YAML file path
    fn read_yaml(path: PathBuf) -> ApiResult<Self>;
    /// Write data to specified path
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()>;
    /// Write data as CFF to specified path
    fn write_cff(&self, _path: impl Into<PathBuf>) -> ApiResult<()> {
        Err(eyre!("CFF write not implemented for this type"))
    }
    /// Write data as JSON to specified path
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()>;
    /// Write data as Markdown (MD) to specified path
    fn write_markdown(&self, _path: impl Into<PathBuf>) -> ApiResult<()> {
        Err(eyre!("Markdown write not implemented for this type"))
    }
    /// Write data as YAML to specified path
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()>;
}
/// The "engine" or "execution method" that determines where and how to run (e.g., "execute") code
/// ### Note
/// At a minimum, choosing an executor also determines level of isolation and security for code execution.
#[derive(Clone, Debug, Deserialize, EnumIs, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum Executor {
    /// Simplifies the creation and execution of containers, ensuring software components are encapsulated for portability and reproducibility
    /// ### Note
    /// Formerly Singularity
    #[serde(alias = "Singularity", alias = "singularity")]
    Apptainer,
    /// Container engine and ecosystem for building, running, and managing containers
    ///
    /// See <https://www.docker.com/products/docker-desktop> for more information
    Docker,
    /// Daemonless, open-source container engine for building, running, and managing containers, with a Docker-like command line
    /// ### Note
    /// Podman is rootless by default
    ///
    /// See <https://podman.io/> for more information
    Podman,
    /// Secure execution environment that leverages technology other than containerization to achieve isolation
    Sandbox,
    /// Local command-line interface that interprets and runs commands for the operating system directly
    #[serde(alias = "zsh", alias = "pwsh", alias = "cmd", alias = "local")]
    Shell,
    /// Secure Shell (SSH) protocol for remote command execution
    #[serde(alias = "remote")]
    Ssh,
    /// Portable, extensible, open source platform for managing containerized workloads and services that facilitate both declarative configuration and automation
    ///
    /// See <https://kubernetes.io/> for more information
    #[serde(alias = "k8s")]
    Kubernetes,
    /// Software-based computers that run an operating system inside another host system, with stronger isolation than containers
    /// ### Examples
    /// - [Oracle VirtualBox](https://www.oracle.com/virtualization/virtualbox/)
    /// - [Vagrant](https://www.vagrantup.com/)
    /// - [Amazon Firecracker](https://firecracker-microvm.github.io/)
    #[serde(alias = "vm")]
    VirtualMachine,
    /// Custom or unspecified execution method
    Other(String),
}
/// Root-level or reference-level license declaration.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum License {
    /// Multiple SPDX identifiers interpreted as OR.
    Multiple(Vec<String>),
    /// Single SPDX identifier.
    Single(String),
}
/// Progress indicator types for with_progress
#[derive(Clone, Copy, Debug, Default)]
pub enum ProgressType {
    /// Progress bar with spinner, count, and percentage indicator
    #[default]
    Bar,
    /// Indeterminate spinner for unknown item counts
    Spinner,
    /// Simple counter showing position of total (e.g., "5 of 100")
    Counter,
    /// No progress output
    Silent,
}
lazy_static! {
    static ref PROGRESS_RENDERER: MultiProgress = MultiProgress::new();
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
/// SSH endpoint for a remote Docker daemon.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Remote(Location);
/// Struct for adding ToStringList functionality
pub struct StringList<'a>(pub &'a Vec<PathBuf>);
impl From<CstValue<'_>> for CstInputValue {
    fn from(value: CstValue<'_>) -> Self {
        match value.0 {
            | Value::Null => Self::Null,
            | Value::Bool(value) => Self::Bool(*value),
            | Value::Number(value) => Self::Number(value.to_string()),
            | Value::String(value) => Self::String(value.clone()),
            | Value::Array(values) => Self::Array(values.iter().map(|value| CstValue(value).into()).collect()),
            | Value::Object(values) => Self::Object(values.iter().map(|(key, value)| (key.clone(), CstValue(value).into())).collect()),
        }
    }
}
impl AsRef<str> for Executor {
    fn as_ref(&self) -> &str {
        match self {
            | Executor::Apptainer => "apptainer",
            | Executor::Docker => "docker",
            | Executor::Podman => "podman",
            | Executor::Sandbox => "sandbox",
            | Executor::Shell => "shell",
            | Executor::Ssh => "ssh",
            | Executor::Kubernetes => "kubernetes",
            | Executor::VirtualMachine => "virtual_machine",
            | Executor::Other(value) => value.as_str(),
        }
    }
}
impl fmt::Display for Executor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}
impl From<&str> for Executor {
    /// Parses a string into a `Executor` value
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            | "apptainer" | "singularity" => Executor::Apptainer,
            | "docker" => Executor::Docker,
            | "podman" => Executor::Podman,
            | "sandbox" => Executor::Sandbox,
            | "shell" => Executor::Shell,
            | "ssh" => Executor::Ssh,
            | "kubernetes" | "k8s" => Executor::Kubernetes,
            | "virtual machine" | "virtual_machine" | "vm" => Executor::VirtualMachine,
            | other => Executor::Other(other.to_string()),
        }
    }
}
impl From<Executor> for std::ffi::OsString {
    fn from(value: Executor) -> Self {
        Self::from(value.to_string())
    }
}
impl From<Executor> for String {
    fn from(value: Executor) -> Self {
        value.to_string()
    }
}
impl Executor {
    /// Returns the default configuration directory on host for GitLab runners based on the operating system.
    pub fn default_gitlab_runner_config_directory() -> &'static str {
        match cfg!(target_os = "macos") {
            | true => "/Users/Shared/gitlab-runner/config",
            | false => "/srv/gitlab-runner/config",
        }
    }
    /// Returns the OS binary name used to manage this executor.
    ///
    /// Returns `"docker"`, `"podman"`, `"apptainer"` for container-based
    /// executors, or `"gitlab-runner"` for all others.
    pub fn command(&self) -> Option<&str> {
        match self {
            | Executor::Docker => Some("docker"),
            | Executor::Podman => Some("podman"),
            | Executor::Apptainer => Some("apptainer"),
            | Executor::Shell | Executor::Ssh | Executor::Kubernetes | Executor::Sandbox | Executor::VirtualMachine => None,
            | Executor::Other(value) => Some(value.as_str()),
        }
    }
    /// Returns the value passed to `gitlab-runner register --executor`.
    pub fn gitlab_runner_type(&self) -> &str {
        match self {
            | Executor::Docker | Executor::Podman | Executor::Apptainer | Executor::Sandbox | Executor::Other(_) => "docker",
            | Executor::Shell => "shell",
            | Executor::Ssh => "ssh",
            | Executor::Kubernetes => "kubernetes",
            | Executor::VirtualMachine => match consts::OS {
                | "macos" => "parallels",
                | _ => "virtualbox",
            },
        }
    }
    /// Returns whether the executable used to manage this executor is available.
    pub fn is_available(&self) -> bool {
        command_exists(self.as_ref())
    }
    /// Returns the path to the socket file used to manage this executor, if applicable.
    pub fn socket(&self) -> Option<String> {
        match self {
            | Executor::Docker | Executor::Apptainer => {
                // Assumes linux-based image is used for GitLab runner
                Some(DOCKER_SOCKET.to_string())
            }
            | Executor::Podman => {
                // TODO: Windows support
                if let Some(value) = var_os("XDG_RUNTIME_DIR") {
                    let path = PathBuf::from(value).join("podman/podman.sock");
                    if path.exists() {
                        Some(path.to_absolute_string())
                    } else {
                        None
                    }
                } else {
                    // root permission fallback
                    let path = PathBuf::from("/run/podman/podman.sock");
                    if path.exists() {
                        Some(path.to_absolute_string())
                    } else {
                        None
                    }
                }
            }
            | Executor::Shell | Executor::Ssh | Executor::Kubernetes | Executor::Sandbox | Executor::VirtualMachine | Executor::Other(_) => None,
        }
    }
    /// Validate this runtime and configured runners for an optional remote Docker daemon
    pub fn validate(&self, runners: Option<&[config::RunnerDetails]>, remote: Option<&Remote>) -> ApiResult<()> {
        match (remote, self.is_docker()) {
            | (Some(endpoint), false) => Err(eyre!("Remote Docker target '{endpoint}' requires the docker runtime, not {self}")),
            | (Some(endpoint), true) => runners
                .and_then(|values| values.iter().find(|runner| !runner.executor.is_docker()))
                .map_or(Ok(()), |runner| {
                    Err(eyre!(
                        "Remote Docker target '{endpoint}' requires docker runner executors, not {}",
                        runner.executor
                    ))
                }),
            | (None, _) => Ok(()),
        }
    }
}
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
        S: Into<String> + core::marker::Copy,
    {
        let command = name.into();
        if command_exists(command.clone()) {
            match cmd!(&command, ["--version"]) {
                | Ok(output) if output.status.success() => output.stdout().lines().next().map(SemanticVersion::from),
                | Ok(_) | Err(_) => None,
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
    /// use std::path::Path;
    ///
    /// let mime = MimeType::from_path(Path::new("test.cff"));
    /// assert_eq!(mime, MimeType::Yaml);
    /// ```
    fn from_path<P>(value: &P) -> MimeType
    where
        P: AsRef<Path> + ?Sized,
    {
        MimeType::from(value.as_ref().display().to_string())
    }
}
impl fmt::Display for Remote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl core::str::FromStr for Remote {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let invalid = || format!("invalid remote '{value}' — expected ssh://[user@]host[:port][/socket]");
        match Uri::parse(value) {
            | Ok(uri) => {
                let is_ssh = uri.scheme().as_str() == "ssh";
                let is_trimmed = value.trim() == value;
                let has_no_fragment = uri.fragment().is_none();
                let has_no_query = uri.query().is_none();
                let has_no_whitespace = !value.chars().any(char::is_whitespace);
                let is_valid_uri = is_ssh && is_trimmed && has_no_fragment && has_no_query && has_no_whitespace;
                match uri.authority() {
                    | Some(authority) => {
                        let has_host = !authority.host().is_empty();
                        let has_no_password = authority.userinfo().is_none_or(|userinfo| !userinfo.as_str().contains(':'));
                        let has_valid_port = authority.port_to_u16().is_ok();
                        let is_valid_authority = has_host && has_no_password && has_valid_port;
                        match is_valid_uri && is_valid_authority {
                            | true => Ok(Self(Location::from(value))),
                            | false => Err(invalid()),
                        }
                    }
                    | None => Err(invalid()),
                }
            }
            | Err(_) => Err(invalid()),
        }
    }
}
impl Remote {
    /// Return the validated SSH endpoint.
    pub fn as_str(&self) -> &str {
        (&self.0).into()
    }
    /// Copy a GPU runner template into a container on this remote Docker daemon.
    pub fn copy_gpu_template(&self, runtime: &Executor, name: &str, template: &Path) -> Result<(), Report> {
        let copy = self.docker_args(args!["cp", template, format!("{name}:/etc/gitlab-runner/gpu.template.toml")]);
        match cmd!(runtime, copy) {
            | Ok(output) if output.status.success() => Ok(()),
            | Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(eyre!("Failed to copy GitLab runner GPU template to {self} — {stderr}"))
            }
            | Err(why) => Err(eyre!("Failed to execute docker cp for {self} — {why}")),
        }
    }
    /// Create an optional GPU runner template for a local or remote Docker daemon.
    pub fn create_gpu_template(remote: Option<&Self>, config_host_dir: &str) -> io::Result<Option<PathBuf>> {
        let parent = remote.map_or_else(|| PathBuf::from(config_host_dir), |_| temp_dir());
        let filename = remote.map_or_else(|| "gpu.template.toml".to_string(), |_| format!("acorn-gpu-{}.template.toml", nanoid!()));
        let template = parent.join(filename);
        let content = "[[runners]]\n  [runners.docker]\n    gpus = \"all\"\n";
        match create_dir_all(parent).and_then(|_| write(&template, content)) {
            | Ok(()) => Ok(Some(template)),
            | Err(why) if remote.is_some() => Err(why),
            | Err(_) => Ok(None),
        }
    }
    /// Target this remote Docker daemon with command arguments.
    pub fn docker_args(&self, command: Vec<OsString>) -> Vec<OsString> {
        args!["--host", self.as_str(), ..command]
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
impl ProgressType {
    fn template(&self) -> Option<&'static str> {
        match self {
            | ProgressType::Bar => Some(Label::PROGRESS_BAR_TEMPLATE),
            | ProgressType::Spinner => Some(Label::PROGRESS_SPINNER_TEMPLATE),
            | ProgressType::Counter => Some(Label::PROGRESS_COUNTER_TEMPLATE),
            | ProgressType::Silent => None,
        }
    }
    fn is_indeterminate(&self) -> bool {
        matches!(self, ProgressType::Spinner)
    }
}
impl StringConversion for PathBuf {
    fn file_name_with_parent(&self) -> String {
        file_name_with_parent(self.clone())
    }
    fn to_absolute_string(&self) -> String {
        to_absolute_string(self.clone())
    }
}
/// Creates zip archive from directory
pub fn archive(path: PathBuf, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let zip_file_path = match destination {
        | Some(value) => value,
        | None => path.with_extension("zip"),
    };
    info!("=> {} Create archive at {}", Label::using(), zip_file_path.to_absolute_string());
    let prepared = if zip_file_path.exists() {
        match zip_file_path.symlink_metadata() {
            | Ok(metadata) => {
                let file_type = metadata.file_type();
                if file_type.is_symlink() {
                    error!("=> {} Create zip archive — destination cannot be a symlink", Label::fail());
                    false
                } else if metadata.is_file() {
                    match remove_file(zip_file_path.clone()) {
                        | Ok(_) => true,
                        | Err(why) => {
                            error!("=> {} Prepare zip destination — {why}", Label::fail());
                            false
                        }
                    }
                } else {
                    error!("=> {} Create zip archive — destination exists and is not a file", Label::fail());
                    false
                }
            }
            | Err(why) => {
                error!("=> {} Inspect zip destination — {why}", Label::fail());
                false
            }
        }
    } else {
        true
    };
    let zip_file = if prepared {
        match OpenOptions::new().write(true).create_new(true).open(&zip_file_path) {
            | Ok(zip_file) => Some(ZipWriter::new(zip_file)),
            | Err(why) => {
                error!("=> {} Create zip archive — {why}", Label::fail());
                None
            }
        }
    } else {
        None
    };
    if let Some(mut zip) = zip_file {
        let archive_root = path.canonicalize();
        let files = files_all(path.clone(), None).into_iter().filter(|x| x.is_file());
        for file_path in files {
            if let Ok(file) = File::open(file_path.clone()) {
                let name = archive_root.as_ref().ok().and_then(|root| {
                    file_path
                        .canonicalize()
                        .ok()
                        .and_then(|absolute| absolute.strip_prefix(root).ok().map(Path::to_path_buf))
                });
                match name {
                    | Some(name) => {
                        trace!(file = name.to_absolute_string(), "=> {} Add file to archive", Label::using());
                        match zip.start_file_from_path(name, options) {
                            | Ok(_) => {
                                let mut buffer = Vec::new();
                                match io::copy(&mut file.take(u64::MAX), &mut buffer) {
                                    | Ok(_) => match zip.write_all(&buffer) {
                                        | Ok(_) => {}
                                        | Err(why) => {
                                            error!(file = file_path.to_absolute_string(), "=> {} Write zip archive — {why}", Label::fail())
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
                    | None => {
                        error!(
                            file = file_path.to_absolute_string(),
                            "=> {} Resolve relative zip archive path",
                            Label::fail()
                        );
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
        Err(eyre!("Unable to create zip archive"))
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
    #[allow(clippy::unwrap_used)]
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
/// Create an RSA public/private key pair with 2048 bits of entropy using the `rsa` crate
pub fn create_rsa_keypair() -> ApiResult<RsaKeyPair> {
    let bits = 2048;
    let mut rng = OsRng;
    match RsaPrivateKey::new(&mut rng, bits) {
        | Ok(private_key) => {
            let public_key = RsaPublicKey::from(&private_key);
            Ok((private_key, public_key))
        }
        | Err(why) => {
            error!("=> {} Create RSA key pair — {why}", Label::fail());
            Err(eyre!("Failed to create RSA key pair — {why}"))
        }
    }
}
/// Returns the current date in ISO8601 format (YYYY-MM-DD).
/// ### Examples
/// ```rust
/// use acorn::io::current_date;
///
/// let date = current_date();
/// // Returns something like "2026-01-22"
/// assert_eq!(date.len(), 10);
/// assert!(date.contains("-"));
/// ```
pub fn current_date() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}
/// Returns `Ok(())` if `unix_seconds` is within `window_secs` of the current UTC time.
///
/// # Errors
///
/// Returns an error when `unix_seconds` is more than `window_secs` seconds away from now.
pub fn validate_unix_timestamp_window(unix_seconds: i64, window_secs: i64) -> ApiResult<()> {
    let now = Utc::now().timestamp();
    if u64::try_from(window_secs).map_or(true, |window| now.abs_diff(unix_seconds) > window) {
        Err(eyre!("Timestamp {unix_seconds} is outside the {window_secs}-second window"))
    } else {
        Ok(())
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
pub async fn download_binary<S, P>(url: S, destination: P) -> ApiResult<PathBuf>
where
    S: Into<String> + Clone + core::marker::Copy,
    P: Into<PathBuf> + Clone,
{
    let url_string: String = url.into();
    let dest: PathBuf = destination.clone().into();
    let filename = PathBuf::from(url_string.clone())
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("downloaded_file")
        .to_string();
    match http::get(url_string.clone()).send().await {
        | Ok(data) => match data.bytes().await {
            | Ok(content) => {
                let output = dest.clone().join(filename.clone());
                match write(output.clone(), content.as_slice()) {
                    | Ok(_) => {
                        debug!(filename, "=> {} Downloaded", Label::output());
                        Ok(output)
                    }
                    | Err(why) => Err(eyre!("Failed to write {filename} - {why}")),
                }
            }
            | Err(_) => Err(eyre!("No content downloaded from {url_string}")),
        },
        | Err(_) => Err(eyre!("Failed to download {url_string}")),
    }
}
/// Returns whether an environment variable is set to a truthy value.
///
/// Recognized truthy values are `1`, `true`, `yes`, and `on`, matched case-insensitively.
/// Returns `None` when the variable is not set.
pub fn env_var_is_truthy(name: impl AsRef<str>) -> Option<bool> {
    var(name.as_ref())
        .ok()
        .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
}
/// Extract zip archive
/// ### Note
/// If `destination` is not provided, the extracted files will be saved in a folder named "extract" an OS-specific cache location.
pub fn extract_zip(path: PathBuf, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
    let root = match destination {
        | Some(value) => value,
        | None => standard_project_folder("extract", None),
    };
    match File::open(path.clone()) {
        | Ok(zip_file) => match ZipArchive::new(zip_file) {
            | Ok(mut archive) => {
                let success = (0..archive.len()).all(|index| match archive.by_index(index) {
                    | Ok(mut file) => {
                        let target = root.join(file.name());
                        if let Some(parent) = target.parent() {
                            match create_dir_all(parent) {
                                | Ok(_) => {}
                                | Err(why) => error!(path = parent.to_path_buf().to_absolute_string(), "=> {} Create - {}", Label::fail(), why),
                            }
                        }
                        match OpenOptions::new().write(true).create_new(true).open(&target) {
                            | Ok(mut output_file) => match io::copy(&mut file, &mut output_file) {
                                | Ok(_) => true,
                                | Err(why) => {
                                    error!(path = target.to_absolute_string(), "=> {} Copy file content - {why}", Label::fail());
                                    false
                                }
                            },
                            | Err(_) => {
                                error!(path = target.to_absolute_string(), "=> {} Create file", Label::fail());
                                false
                            }
                        }
                    }
                    | Err(why) => {
                        error!(path = path.to_absolute_string(), "=> {} Extract file - {why}", Label::fail());
                        false
                    }
                });
                if success {
                    info!(path = root.to_absolute_string(), "=> {} Extract zip archive", Label::pass());
                    Ok(root)
                } else {
                    error!(path = root.to_absolute_string(), "=> {} Extract zip archive", Label::fail());
                    Err(eyre!("Failed to extract zip archive"))
                }
            }
            | Err(why) => {
                error!(path = path.to_absolute_string(), "=> {} Read zip archive - {why}", Label::fail());
                Err(eyre!("Failed to read zip archive - {why}"))
            }
        },
        | Err(why) => {
            error!(path = path.to_absolute_string(), "=> {} Read file - {why}", Label::fail());
            Err(eyre!("Failed to read file - {why}"))
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
pub fn file_checksum<P>(path: P, algorithm: Option<&'static ring::digest::Algorithm>) -> Option<Checksum>
where
    P: Into<PathBuf>,
{
    let value = path.into();
    let digest_algorithm = algorithm.unwrap_or(&SHA256);
    let checksum_algorithm = ChecksumAlgorithm::from(digest_algorithm);
    match File::open(value.clone()) {
        | Ok(file) => {
            let mut buffer = [0; 1024];
            let mut context = Context::new(digest_algorithm);
            let mut reader = BufReader::new(file);
            loop {
                let count = match reader.read(&mut buffer) {
                    | Ok(c) => c,
                    | Err(err) => {
                        error!(
                            error = err.to_string(),
                            path = value.to_absolute_string(),
                            "=> {} Read file checksum",
                            Label::fail()
                        );
                        return None;
                    }
                };
                if count == 0 {
                    break;
                }
                context.update(buffer.get(..count).unwrap_or(&[]));
            }
            let digest = context.finish();
            let result = HEXUPPER.encode(digest.as_ref());
            Some(Checksum {
                algorithm: checksum_algorithm,
                checksum_value: result.to_lowercase(),
            })
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
/// * `path` - A `PathBuf` to the directory to search — also accepts URI format paths (e.g., `"file:///path/to/directory"`).
/// * `extensions` - An `Option` containing a list of string slice(s) representing the file extension(s) to search for.
///
/// # Returns
/// A `Vec` containing `PathBuf` values of all files in the given directory that match at least one of the given extensions.
pub fn files_all(path: PathBuf, extensions: Option<Vec<&str>>) -> Vec<PathBuf> {
    let path = uri_to_path(path);
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
            | Some(values) => {
                values
                    .into_iter()
                    .map(|extension| {
                        let glob_pattern = pattern(path.clone(), extension);
                        glob(&glob_pattern)
                    })
                    .filter_map(|x| x.ok())
                    .flat_map(paths_to_vec)
                    .fold((HashSet::new(), Vec::new()), |(mut seen, mut ordered), path| {
                        if seen.insert(path.clone()) {
                            ordered.push(path);
                        }
                        (seen, ordered)
                    })
                    .1
            }
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
                "=> {} Extension passed with single file to files_all()...was this intended?",
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
        match cmd!("git", args) {
            | Ok(output) if output.status.success() => filter_git_command_result(output.stdout(), extensions),
            | Ok(output) => {
                let why = output.stderr();
                let message = if why.is_empty() {
                    format!("process exited with status {}", output.status)
                } else {
                    why
                };
                error!("=> {} Get files from Git branch - {}", Label::fail(), message);
                vec![]
            }
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
        let result = cmd!("git", args);
        debug!("=> {} Git command response - {result:?}", Label::using());
        let files = match result {
            | Ok(output) if output.status.success() => filter_git_command_result(output.stdout(), extensions),
            | Ok(output) => {
                let why = output.stderr();
                let message = if why.is_empty() {
                    format!("process exited with status {}", output.status)
                } else {
                    why
                };
                error!("=> {} Get files from Git commit - {}", Label::fail(), message);
                vec![]
            }
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
pub async fn files_from_gitlab_merge_request(extensions: Option<Vec<&str>>) -> Vec<PathBuf> {
    let root = var("CI_API_V4_URL").unwrap_or_default();
    let project_id = var("CI_MERGE_REQUEST_PROJECT_ID").unwrap_or_default();
    let merge_request_iid = var("CI_MERGE_REQUEST_IID").unwrap_or_default();
    let path = format!("/projects/{project_id}/merge_requests/{merge_request_iid}/diffs");
    let url = format!("{root}{path}");
    match http::get(url).send().await {
        | Ok(response) => {
            let content: serde_json::Result<Vec<GitlabMergeRequestDiffResponse>> = response.text().await.map_or_else(
                |_| Err(serde_json::Error::io(io::Error::other("Failed to read response text"))),
                |body| serde_json::from_str(&body),
            );
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
/// let ignore = Some(r"\.txt$".to_string());
/// let result = filter_ignored(paths, ignore);
/// assert!(result.unwrap().is_empty());
/// ```
pub fn filter_ignored(paths: Vec<PathBuf>, ignore: Option<String>) -> ApiResult<Vec<PathBuf>> {
    match ignore {
        | Some(ignore_pattern) => match Regex::new(&ignore_pattern) {
            | Ok(re) => Ok(paths
                .into_iter()
                .map(to_absolute_string)
                .filter(|x| !re.is_match(x).unwrap_or(false))
                .map(PathBuf::from)
                .collect()),
            | Err(why) => Err(eyre!("Invalid regex/filter pattern: {why}")),
        },
        | None => Ok(paths),
    }
}
/// Return file paths that do not match an ignore pattern relative to a local root path.
///
/// This applies root containment checks and normalized relative-path matching.
pub fn filter_ignored_with_root(paths: Vec<PathBuf>, ignore: Option<String>, root: PathBuf) -> ApiResult<Vec<PathBuf>> {
    match ignore {
        | Some(ignore_pattern) => match Regex::new(&ignore_pattern) {
            | Ok(re) => {
                let root = if root.is_file() {
                    root.parent().map(|value| value.to_path_buf()).unwrap_or(root)
                } else {
                    root
                };
                let normalized_root = canonicalize(root.clone()).unwrap_or(root);
                let mut filtered: Vec<PathBuf> = vec![];
                for path in paths {
                    let normalized_path = canonicalize(path.clone()).unwrap_or(path.clone());
                    match normalized_path.strip_prefix(&normalized_root) {
                        | Ok(relative) => {
                            let value = relative.to_string_lossy().to_string().replace('\\', "/");
                            if !re.is_match(&value).unwrap_or(false) {
                                filtered.push(path);
                            }
                        }
                        | Err(_) => {
                            return Err(eyre!(
                                "Path '{}' is outside resolved root '{}'",
                                normalized_path.to_absolute_string(),
                                normalized_root.to_absolute_string()
                            ));
                        }
                    }
                }
                Ok(filtered)
            }
            | Err(why) => Err(eyre!("Invalid regex/filter pattern: {why}")),
        },
        | None => Ok(paths),
    }
}
/// Returns the value of the first environment variable in the list that is set
///
/// ### Example
/// ```rust
/// use acorn::io::first_env_var;
/// use std::env;
///
/// env::set_var("ACORN_DOCTEST_FIRST_ENV_VAR", "config_value");
/// let result = first_env_var(&["ACORN_DOCTEST_MISSING_ENV_VAR", "ACORN_DOCTEST_FIRST_ENV_VAR"]);
/// assert_eq!(result, Some("config_value".to_string()));
/// env::remove_var("ACORN_DOCTEST_FIRST_ENV_VAR");
/// ```
pub fn first_env_var(names: &[&str]) -> Option<String> {
    names
        .iter()
        .filter_map(|name| var(name).ok().map(|value| value.trim().to_string()))
        .find(|value| !value.is_empty())
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
        match cmd!("git", args) {
            | Ok(output) if output.status.success() => output.stdout().split("/").last().map(|x| x.to_string()),
            | Ok(_) | Err(_) => None,
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
        match cmd!("git", args) {
            | Ok(output) if output.status.success() => output.stdout().split("/").last().map(|x| x.to_string()),
            | Ok(_) | Err(_) => None,
        }
    } else {
        None
    }
}
/// Resolve a child directory path under the user's home directory
pub fn home_directory(child: &str) -> ApiResult<PathBuf> {
    BaseDirs::new()
        .map(|dirs| dirs.home_dir().join(child))
        .ok_or_else(|| eyre!("Failed to resolve home directory"))
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
    let path = path.into();
    let create_with_mode = OpenOptions::new().write(true).create_new(true).mode(0o755).open(path.as_path());
    match create_with_mode {
        | Ok(_) => path.is_executable(),
        | Err(why) => {
            if why.kind() == io::ErrorKind::AlreadyExists {
                match set_permissions(path.as_path(), Permissions::from_mode(0o755)) {
                    | Ok(()) => path.is_executable(),
                    | Err(why) => {
                        debug!(path = path.to_absolute_string(), "=> {} Set permissions — {why}", Label::fail());
                        false
                    }
                }
            } else {
                debug!(path = path.to_absolute_string(), "=> {} Create executable file — {why}", Label::fail());
                false
            }
        }
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
/// Returns a string containing the file name with its parent directory.
///
/// If the `PathBuf` is a directory, only the file name is returned.
pub fn file_name_with_parent(value: impl Into<PathBuf>) -> String {
    let path = value.into();
    let name = path.file_name().and_then(|value| value.to_str()).unwrap_or_default().to_string();
    if path.is_dir() {
        name
    } else {
        let parent_name = path
            .parent()
            .and_then(|value| value.file_name())
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if parent_name.is_empty() {
            name
        } else {
            format!("{parent_name}/{name}")
        }
    }
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
/// This function is thread-safe and can be used with rayon's parallel iterators.
///
/// # Parameters
///
/// * `path` - A `PathBuf` or string slice containing the path to the file to be read.
///
/// # Return
///
/// A `Result` containing the contents of the file as a string if the file is readable, or an
/// `std::io::Error` otherwise.
///
/// # Example with rayon
///
/// ```ignore
/// use rayon::prelude::*;
///
/// let paths = vec![PathBuf::from("file1.txt"), PathBuf::from("file2.txt")];
/// let contents: Vec<_> = paths
///     .par_iter()
///     .filter_map(|path| read_file(path).ok())
///     .collect();
/// ```
pub fn read_file<P>(path: P) -> ApiResult<String>
where
    P: Into<PathBuf> + Clone + Send,
{
    let path_buf = path.into();
    let filename = path_buf.file_name().unwrap_or_default().to_string_lossy().to_string();
    let is_large_file = match path_buf.metadata() {
        | Ok(metadata) => metadata.len() >= LARGE_FILE_THRESHOLD_BYTES,
        | Err(_) => false,
    };
    if is_large_file {
        trace!(filename, "=> {} Read file with large-file strategy", Label::using());
        read_large_file(path_buf)
    } else {
        match File::open(&path_buf) {
            | Ok(file) => {
                let mut reader = BufReader::new(file);
                let mut content = String::new();
                match reader.read_to_string(&mut content) {
                    | Ok(_) => Ok(content),
                    | Err(why) => Err(eyre!("Failed to read file content — {why}")),
                }
            }
            | Err(why) => {
                error!(filename, "=> {} Read file", Label::fail());
                Err(eyre!("Failed to read file — {why}"))
            }
        }
    }
}
/// Reads large files and returns the contents as a string.
///
/// This function uses a larger buffered reader and pre-allocates the output string
/// using file metadata when available.
pub fn read_large_file<P>(path: P) -> ApiResult<String>
where
    P: Into<PathBuf> + Clone + Send,
{
    match File::open(path.into()) {
        | Ok(file) => {
            let capacity = file
                .metadata()
                .ok()
                .and_then(|metadata| usize::try_from(metadata.len()).ok())
                .unwrap_or(0);
            let mut reader = BufReader::with_capacity(1024 * 1024, file);
            let mut content = if capacity > 0 { String::with_capacity(capacity) } else { String::new() };
            match reader.read_to_string(&mut content) {
                | Ok(_) => Ok(content),
                | Err(why) => Err(eyre!("Failed to read large file content — {why}")),
            }
        }
        | Err(why) => Err(eyre!("Failed to read large file — {why}")),
    }
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
    let s = result.display().to_string();
    #[cfg(windows)]
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s).to_string();
    s
}
/// Returns a sorted list of unique lowercase file extensions from the given paths.
pub fn unique_file_extensions(paths: &[PathBuf]) -> Vec<String> {
    let mut extensions = paths
        .iter()
        .filter_map(|path| path.extension().map(|extension| extension.to_string_lossy().to_lowercase()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    extensions.sort_unstable();
    extensions
}
/// Converts `file:` source values to a [`PathBuf`].
/// ### Note
/// Supports shorthand paths like `file:./data.json`, URI forms like `file:///tmp/data.json` or `file://localhost/tmp/data.json`, and leaves non-file paths unchanged.
pub fn uri_to_path<P>(value: P) -> PathBuf
where
    P: Into<PathBuf>,
{
    let path: PathBuf = value.into();
    let s = path.to_string_lossy().into_owned();
    match s.as_str() {
        | source if source.starts_with("file://localhost/") => {
            let stripped = source.trim_start_matches("file://localhost/");
            uri_to_path(PathBuf::from(format!("file:///{stripped}")))
        }
        | source if source.starts_with("file://") => {
            let stripped = source.trim_start_matches("file://");
            #[cfg(windows)]
            let normalized = match stripped.get(1..3) {
                | Some(drive) if drive.contains(':') => &stripped[1..],
                | _ => stripped,
            };
            #[cfg(not(windows))]
            let normalized = stripped;
            PathBuf::from(normalized)
        }
        | source if source.starts_with("file:") => PathBuf::from(source.trim_start_matches("file:")),
        | _ => path,
    }
}
/// Creates a symbolic link from `target` to `source`.
///
/// On Windows, directory sources use directory symlinks and other sources use file symlinks.
#[cfg(unix)]
pub fn symlink(source: &Path, target: &Path) -> ApiResult<()> {
    match prelude::symlink(source, target) {
        | Ok(_) => Ok(()),
        | Err(why) => Err(why.into()),
    }
}
/// Creates a symbolic link from `target` to `source`
/// ### Note
/// On Windows, directory sources use directory symlinks and other sources use file symlinks.
#[cfg(windows)]
pub fn symlink(source: &Path, target: &Path) -> ApiResult<()> {
    let result = if source.is_dir() {
        symlink_dir(source, target)
    } else {
        symlink_file(source, target)
    };
    match result {
        | Ok(_) => Ok(()),
        | Err(why) => Err(why.into()),
    }
}
/// Creates a new progress bar with the specified count and progress type
pub fn create_progress_bar(count: usize, progress_type: ProgressType) -> ProgressBar {
    create_progress_bar_with_renderer(count, progress_type, &PROGRESS_RENDERER)
}
fn create_progress_bar_with_renderer(count: usize, progress_type: ProgressType, renderer: &MultiProgress) -> ProgressBar {
    if matches!(progress_type, ProgressType::Silent) {
        ProgressBar::hidden()
    } else {
        let progress = if progress_type.is_indeterminate() {
            let spinner = ProgressBar::new_spinner();
            spinner.enable_steady_tick(Duration::from_millis(120));
            spinner
        } else {
            ProgressBar::new(count as u64)
        };
        if let Some(template) = progress_type.template() {
            #[allow(clippy::unwrap_used)]
            progress.set_style(ProgressStyle::with_template(template).unwrap());
        }
        renderer.add(progress)
    }
}
/// Returns the shared progress renderer used by ACORN I/O operations.
pub fn progress_renderer() -> MultiProgress {
    PROGRESS_RENDERER.clone()
}
/// Applies a new style template to an existing progress bar
pub fn apply_progress_style(progress: &ProgressBar, template: &str) {
    #[allow(clippy::unwrap_used)]
    progress.set_style(ProgressStyle::with_template(template).unwrap());
}
/// Finishes a progress bar with a message, applying appropriate final style
pub fn finish_progress_bar(progress: &ProgressBar, message: String) {
    #[allow(clippy::unwrap_used)]
    progress.set_style(ProgressStyle::with_template("  {msg}").unwrap());
    progress.finish_with_message(message);
}
/// Process a collection of data items with progress indication.
///
/// # Arguments
/// * `items` - Collection of items to process
/// * `message` - Function to generate progress message for each item
/// * `operation` - Async function to apply to each item
/// * `finish_message` - Function to generate completion message
/// * `buffer_size` - Concurrency level for parallel processing
/// * `progress_type` - Type of progress indicator (Bar, Spinner, Counter, Silent)
///
/// # Example
/// ```ignore
/// let result = with_progress(
///     items,
///     |item| format!("Processing {}", item),
///     |item| async move { process(item) },
///     |count| format!("Done! Processed {} items", count),
///     Some(10),
///     ProgressType::Bar,
/// ).await;
/// ```
pub async fn with_progress<T, U, M, F, Fut>(
    items: Vec<T>,
    message: M,
    operation: F,
    finish_message: impl FnOnce(usize) -> String,
    buffer_size: Option<usize>,
    progress_type: ProgressType,
) -> ApiResult<Vec<U>>
where
    M: for<'a> Fn(&'a T) -> String,
    F: Fn(T) -> Fut,
    Fut: Future<Output = ApiResult<U>>,
{
    let concurrency = buffer_size.unwrap_or(10).max(1);
    let count = items.len();
    let progress = create_progress_bar(count, progress_type);
    if matches!(progress_type, ProgressType::Spinner) {
        progress.enable_steady_tick(Duration::from_millis(120));
    }
    let output = stream::iter(items)
        .map(|item| {
            let msg = message(&item);
            let future = operation(item);
            async move {
                let result = future.await;
                (msg, result)
            }
        })
        .buffer_unordered(concurrency)
        .map(|(msg, result)| {
            progress.set_message(msg);
            progress.inc(1);
            result
        })
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<ApiResult<Vec<_>>>();

    if !matches!(progress_type, ProgressType::Silent) {
        finish_progress_bar(&progress, finish_message(count));
    }
    output
}
/// Parse JSONC content to a [`serde_json::Value`]
/// ### Note
/// Supports JavaScript-style comments (`//` and `/* */`) and trailing commas.
pub fn jsonc_parse_value(content: &str) -> ApiResult<serde_json::Value> {
    let options = ParseOptions {
        allow_comments: true,
        allow_trailing_commas: true,
        allow_loose_object_property_names: false,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    };
    parse_to_serde_value(content, &options).map_err(|why| eyre!("JSONC parse error — {why}"))
}
/// Parse JSONC content into a typed config with a CST root for comment-preserving round-trips
/// ### Note
/// Returns the deserialized config and the CST root. The caller should store the CST
/// in the config's `cst` field for write-back.
pub fn parse_jsonc_cst<T: DeserializeOwned>(content: &str) -> ApiResult<(T, CstRootNode)> {
    let options = ParseOptions {
        allow_comments: true,
        allow_trailing_commas: true,
        allow_loose_object_property_names: false,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    };
    CstRootNode::parse(content, &options)
        .map_err(|why| eyre!("JSONC parse error — {why}"))
        .and_then(|cst| {
            cst.to_serde_value().ok_or_else(|| eyre!("JSONC conversion error")).and_then(|value| {
                serde_json::from_value::<T>(value)
                    .map_err(|why| eyre!("JSONC deserialize error — {why}"))
                    .map(|config| (config, cst))
            })
        })
}
/// Writes the given content to a file at the given path
///
/// # Arguments
/// * `path` - A `PathBuf` or string slice containing the path to the file to be written.
/// * `content` - A `String` containing the content to be written to the file.
///
/// # Returns
/// A `Result` containing a unit value if the file is written successfully, or an
/// `eyre::Report` otherwise.
pub fn write_file<P>(path: P, content: String) -> ApiResult<()>
where
    P: Into<PathBuf>,
{
    write(path.into(), content.as_bytes())
        .map(|_| ())
        .map_err(|why| eyre!("Failed to write file - {why}"))
}
/// Writes bytes to a file at the given path, creating parent directories as needed
///
/// # Arguments
/// * `path` - The output file path
/// * `get_bytes` - An async closure/future that returns the bytes to write
///
/// # Returns
/// A `Result` containing a unit value if the file is written successfully, or an
/// `eyre::Report` otherwise.
pub async fn write_file_bytes<P, F, Fut, E>(path: P, get_bytes: F) -> ApiResult<()>
where
    P: Into<PathBuf>,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<Vec<u8>, E>>,
    E: Into<Report>,
{
    let path = path.into();
    match path.parent() {
        | Some(parent) => {
            let folder = parent.display().to_string();
            match create_dir_all(folder.clone()) {
                | Ok(_) => match OpenOptions::new().write(true).create_new(true).open(&path) {
                    | Ok(mut file) => match get_bytes().await.map_err(Into::into) {
                        | Ok(bytes) => {
                            let mut content = Cursor::new(bytes);
                            match io::copy(&mut content, &mut file) {
                                | Ok(_) => Ok(()),
                                | Err(why) => Err(eyre!("Failed to write bytes — {why}")),
                            }
                        }
                        | Err(why) => Err(why),
                    },
                    | Err(why) => Err(eyre!("Failed to create output file — {why}")),
                },
                | Err(why) => Err(eyre!("Failed to create output folder — {why}")),
            }
        }
        | None => Err(eyre!("Output path has no parent directory")),
    }
}
/// Writes an RSA key pair to disk
///
/// The private key at `path` and the public key at `{path}.pub`.
///
/// When `path` is `None`, the current working directory is used with `id_rsa` as the base name.
pub fn write_rsa_keypair<P>(values: RsaKeyPair, path: Option<P>) -> ApiResult<(PathBuf, PathBuf)>
where
    P: Into<PathBuf>,
{
    let resolved = match path {
        | Some(p) => Ok(p.into()),
        | None => match current_dir() {
            | Ok(cwd) => Ok(cwd.join("id_rsa")),
            | Err(why) => Err(eyre!("Failed to get current directory — {why}")),
        },
    };
    match resolved {
        | Ok(path) => {
            let (private_key, public_key) = values;
            match private_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF) {
                | Ok(private_key_pem) => match public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF) {
                    | Ok(public_key_pem) => {
                        let public_key_path = PathBuf::from(format!("{}.pub", path.display()));
                        let private_key_path = path.clone();
                        match write_file(path, (*private_key_pem).clone()) {
                            | Ok(_) => match write_file(public_key_path.clone(), public_key_pem) {
                                | Ok(_) => Ok((private_key_path, public_key_path)),
                                | Err(why) => Err(why),
                            },
                            | Err(why) => Err(why),
                        }
                    }
                    | Err(why) => {
                        error!("=> {} Write RSA keypair (public key) — {why}", Label::fail());
                        Err(eyre!("Failed to serialize public key to PEM — {why}"))
                    }
                },
                | Err(why) => {
                    error!("=> {} Write RSA keypair (private key) — {why}", Label::fail());
                    Err(eyre!("Failed to serialize private key to PEM — {why}"))
                }
            }
        }
        | Err(why) => Err(why),
    }
}

#[cfg(test)]
mod tests;
