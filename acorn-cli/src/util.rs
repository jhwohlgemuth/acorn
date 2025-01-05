#![allow(dead_code)]
use is_executable::IsExecutable;
use is_terminal::IsTerminal;
use owo_colors::{OwoColorize, Style, Styled};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs::File;
use std::io::copy;
use std::io::Cursor;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs, io};
use tracing::{debug, error};

pub enum StdinIsPipedResult {
    StdinIsPiped,
    StdinIsNotPiped,
}
pub enum StdoutIsPipedResult {
    StdoutIsPiped,
    StdoutIsNotPiped,
}
/// Struct for using and sharing colorized logging labels
pub struct Label {}
impl Label {
    pub fn dry_run() -> Styled<&'static &'static str> {
        let style = Style::new().black().on_yellow();
        " DRY_RUN ■ ".style(style)
    }
    pub fn invalid() -> String {
        Label::fmt_invalid("INVALID")
    }
    pub fn fmt_invalid(value: &str) -> String {
        let style = Style::new().red().on_default_color();
        value.style(style).to_string()
    }
    pub fn fail() -> String {
        Label::fmt_fail("FAIL")
    }
    pub fn fmt_fail(value: &str) -> String {
        let style = Style::new().white().on_red();
        format!(" ✗ {} ", value).style(style).to_string()
    }
    pub fn found() -> Styled<&'static &'static str> {
        let style = Style::new().green().on_default_color();
        "FOUND".style(style)
    }
    pub fn not_found() -> String {
        Label::fmt_not_found("NOT_FOUND")
    }
    pub fn fmt_not_found(value: &str) -> String {
        let style = Style::new().red().on_default_color();
        value.style(style).to_string()
    }
    pub fn output() -> String {
        Label::fmt_output("OUTPUT")
    }
    pub fn fmt_output(value: &str) -> String {
        let style = Style::new().cyan().dimmed().on_default_color();
        value.style(style).to_string()
    }
    pub fn pass() -> Styled<&'static &'static str> {
        let style = Style::new().green().on_default_color();
        "☑  PASS".style(style)
    }
    pub fn read() -> Styled<&'static &'static str> {
        let style = Style::new().green().on_default_color();
        "READ".style(style)
    }
    pub fn rejected() -> Styled<&'static &'static str> {
        let style = Style::new().red().on_default_color();
        "🛑 REJECTED ".style(style)
    }
    pub fn run() -> Styled<&'static &'static str> {
        let style = Style::new().black().on_yellow();
        " RUN ▶ ".style(style)
    }
    pub fn using() -> String {
        Label::fmt_using("USING")
    }
    pub fn fmt_using(value: &str) -> String {
        let style = Style::new().cyan();
        value.style(style).to_string()
    }
}
/// Add forward slash to end of folder path
pub fn add_dot_slash(path: PathBuf) -> PathBuf {
    if path.starts_with("./") {
        path.to_path_buf()
    } else {
        PathBuf::from(format!("./{}", path.display()))
    }
}
/// Add forward slash to end of folder path
pub fn add_forward_slash(path: PathBuf) -> PathBuf {
    if path.is_dir() && !path.ends_with("/") {
        PathBuf::from(format!("{}/", path.display()))
    } else {
        path.to_path_buf()
    }
}
pub fn download_binary(url: String, destination: PathBuf) -> Result<PathBuf, String> {
    async fn download(url: String, destination: PathBuf) -> Result<(), String> {
        let client = reqwest::Client::new();
        let response = client.get(url.clone()).send();
        let filename = PathBuf::from(url.clone())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        match response.await {
            Ok(data) => match data.bytes().await {
                Ok(content) => {
                    let mut output = File::create(destination.join(filename.clone())).unwrap();
                    let _ = copy(&mut Cursor::new(content.clone()), &mut output);
                    debug!(filename = filename, "=> {} Downloaded", Label::output());
                    Ok(())
                }
                Err(_) => Err(format!("No content downloaded from {url}")),
            },
            Err(_) => Err(format!("Failed to download {url}")),
        }
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let _ = runtime.block_on(download(url.clone(), destination.clone()));
    let filename = PathBuf::from(url.clone())
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    Ok(destination.join(filename))
}
/// Get name of Conda virtual environment from manifest file
///
/// Not super sophisticated. "name" needs to be first line in environment.yml
///
/// Example:
///
/// `environment.yml`
/// ```yaml
/// name: module-a
/// channels:
///   - conda-forge
///   - defaults
/// dependencies:
///   - pip
///   - numpy
/// ```
///
/// Get the environment name
/// ```
/// use std::path::PathBuf;
/// use pipe_lib::util::get_conda_environment_name;
///
/// get_conda_environment_name(PathBuf::from("/path/to/environment.yml"));
/// ```
pub fn get_conda_environment_name(path: PathBuf) -> Option<String> {
    match path.try_exists() {
        Ok(true) => match read_file(path) {
            Ok(content) => {
                let name = content
                    .split('\n')
                    .collect::<Vec<_>>()
                    .clone()
                    .into_iter()
                    .next()?
                    .split("name: ")
                    .collect::<Vec<_>>()
                    .clone()
                    .into_iter()
                    .nth(1)?
                    .to_string();
                Some(name)
            }
            _ => None,
        },
        _ => None,
    }
}
/// Get SHA256 hash of a file
pub fn get_checksum(path: PathBuf) -> String {
    if let Ok(content) = read_file(path) {
        let hash = Sha256::digest(content);
        format!("{:x}", hash)
    } else {
        unreachable!()
    }
}
/// Get list of directory children
///
/// Only first level children are returned
///
pub fn get_children(path: String) -> Vec<String> {
    match fs::read_dir(path) {
        Ok(value) => value
            .flatten()
            .map(|x| path_to_string(x.path()))
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    }
}
/// Get file extension
///
/// # Examples
/// ```
/// use std::path::Path;
/// use pipe_lib::util::get_extension;
///
/// assert_eq!("txt", get_extension(Path::new("hello.txt")));
/// assert_eq!("md", get_extension(Path::new("README.md")));
/// assert_eq!("", get_extension(Path::new(".dotfile")));
/// assert_eq!("", get_extension(Path::new("/path/to/folder")));
/// ```
pub fn get_extension(path: &Path) -> String {
    path.extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string()
}
/// Get parent of a file/folder path
pub fn get_parent(path: PathBuf) -> PathBuf {
    PathBuf::from(path.parent().unwrap())
}
/// Check if a file has a given file extension
///
/// # Examples
/// ```
/// use std::path::Path;
/// use pipe_lib::util::has_extension;
///
/// assert!(has_extension("txt", Path::new("hello.txt")));
/// assert!(!has_extension("txt", Path::new("hello.md")));
/// ```
pub fn has_extension(extension: &str, path: &Path) -> bool {
    extension == get_extension(path)
}
/// Check if stdin is piped (e.g., `echo "hello world" | pipe`)
pub fn is_stdin_piped() -> StdinIsPipedResult {
    if !io::stdin().is_terminal() {
        StdinIsPipedResult::StdinIsPiped
    } else {
        StdinIsPipedResult::StdinIsNotPiped
    }
}
/// Check if stdout is piped (e.g., `pipe | process`)
pub fn is_stdout_piped() -> StdoutIsPipedResult {
    if !io::stdout().is_terminal() {
        StdoutIsPipedResult::StdoutIsPiped
    } else {
        StdoutIsPipedResult::StdoutIsNotPiped
    }
}
#[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
pub fn make_executable(path: &PathBuf) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path.is_executable()
}
#[cfg(windows)]
pub fn make_executable(path: &PathBuf) -> bool {
    // TODO: Add windows support...pass through?
    path.is_executable()
}
pub fn path_to_string(path: PathBuf) -> String {
    // NOTE: fs::canonicalize might cause problems on Windows
    let result = match fs::canonicalize(path.as_path()) {
        Ok(value) => value,
        Err(_) => path,
    };
    result.to_str().unwrap().to_string()
}
/// Read content of a file to a string
///
/// Fails when the file cannot be found or read
///
/// # Examples
/// ```
/// use std::path::PathBuf;
/// use pipe_lib::util::read_file;
///
/// let path = PathBuf::from("./tests/fixtures/template.json");
/// let content = read_file(path).unwrap();
pub fn read_file(path: PathBuf) -> Result<String, Box<dyn Error>> {
    let mut content = String::new();
    let _ = match File::open(path.clone()) {
        Ok(mut file) => {
            debug!(path = path.to_str().unwrap(), "=> {}", Label::read());
            file.read_to_string(&mut content)
        }
        Err(err) => {
            error!(
                path = path.to_str().unwrap(),
                "=> {} Cannot read file",
                Label::fail()
            );
            Err(err)
        }
    };
    Ok(content)
}
pub fn to_snake_case(value: String) -> String {
    value.replace(" ", "_").to_lowercase()
}
pub fn to_string(values: Vec<&str>) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}
