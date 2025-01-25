use derive_more::Display;
use glob::glob;
use is_executable::IsExecutable;
use owo_colors::{OwoColorize, Style, Styled};
use serde::{Deserialize, Serialize};
// use sha2::Digest;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::copy;
use std::io::Cursor;
use std::io::Read;
use std::path::PathBuf;
use tracing::{debug, error, info};

const VALE_RELEASES_URL: &str = "https://github.com/errata-ai/vale/releases";
const VALE_VERSION: &str = "3.9.4";

/// Struct for using and sharing colorized logging labels
pub struct Label {}
/// Semantic version
///
/// see <https://semver.org/>
#[derive(Clone, Debug, Deserialize, Display, Serialize)]
#[display("{}.{}.{}", major, minor, patch)]
pub struct SemanticVersion {
    /// Version when you make incompatible API changes
    pub major: u32,
    /// Version when you add functionality in a backward compatible manner
    pub minor: u32,
    /// Version when you make backward compatible bug fixes
    pub patch: u32,
}
impl Label {
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
    pub fn using() -> String {
        Label::fmt_using("USING")
    }
    pub fn fmt_using(value: &str) -> String {
        let style = Style::new().cyan();
        value.style(style).to_string()
    }
}
impl SemanticVersion {
    pub fn from_string(version: &str) -> SemanticVersion {
        let mut parts = version.split('.');
        let major = parts.next().unwrap().parse::<u32>().unwrap();
        let minor = parts.next().unwrap().parse::<u32>().unwrap();
        let patch = parts.next().unwrap().parse::<u32>().unwrap();
        SemanticVersion {
            major,
            minor,
            patch,
        }
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
pub fn download_vale(version: Option<SemanticVersion>) -> Result<PathBuf, String> {
    // https://doc.rust-lang.org/std/env/consts/constant.OS.html
    let os = std::env::consts::OS.to_lowercase();
    let platform = match os.as_str() {
        "linux" => "Linux_64-bit.tar.gz",
        "macos" | "apple" => "macOS_64-bit.tar.gz",
        "windows" => "Windows_64-bit.zip",
        _ => {
            error!(os, "=> {}", Label::not_found());
            // TODO: Add exitcode here
            unimplemented!("Unrecognized operating system")
        }
    };
    let release = match version.clone() {
        Some(value) => value,
        None => SemanticVersion::from_string(VALE_VERSION),
    };
    let url = format!(
        "{}/download/v{}/vale_{}_{}",
        VALE_RELEASES_URL, release, release, platform
    );
    let destination = PathBuf::from(".");
    info!(url, "=> {} Vale release v{}", Label::using(), release);
    match download_binary(url.clone(), destination) {
        Ok(path) => {
            let _checksum = match download_vale_checksums(version) {
                Ok(value) => value.get(platform).unwrap().to_string(),
                Err(_) => "".to_string(),
            };
            // TODO: validate checksum using get_checksum
            // TODO: Un(zip|tar) vale (includes LICENSE, README.md, and vale)
            let vale = PathBuf::from("vale");
            if make_executable(&vale) {
                Ok(vale)
            } else {
                error!("=> {} Vale not executable", Label::invalid());
                Err(format!("Failed to make {} executable", path.display()))
            }
        }
        Err(error) => {
            error!(error, url, "=> {} Vale download", Label::fail());
            Err(error)
        }
    }
}
pub fn download_vale_checksums(
    version: Option<SemanticVersion>,
) -> Result<HashMap<String, String>, String> {
    let release = match version {
        Some(value) => value,
        None => SemanticVersion::from_string(VALE_VERSION),
    };
    let url = format!(
        "{}/download/v{}/vale_{}_checksums.txt",
        VALE_RELEASES_URL, release, release
    );
    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send().unwrap();
    let content = response.text().unwrap();
    for line in content.lines() {
        let mut values = line.split("  ").collect::<Vec<&str>>();
        let key = values.pop().unwrap();
        let value = values.pop().unwrap();
        println!("{} => Key: {:?}, Value: {:?}", Label::output(), key, value);
    }
    // TODO: Parse checksums
    unimplemented!()
}
/// Get SHA256 hash of a file
// pub fn get_checksum(path: PathBuf) -> String {
//     if let Ok(content) = read_file(path) {
//         let hash = Sha256::digest(content);
//         format!("{:x}", hash)
//     } else {
//         unreachable!()
//     }
// }
pub fn get_image_paths(root: PathBuf) -> Vec<PathBuf> {
    let extensions = ["jpg", "jpeg", "png", "svg"];
    let mut files = extensions
        .iter()
        .flat_map(|ext| glob(&format!("{}/**/*.{}", root.display(), ext)))
        .flat_map(|paths| paths.collect::<Vec<_>>())
        .flatten()
        .collect::<Vec<PathBuf>>();
    files.sort();
    files
}
pub fn get_parent(path: String) -> String {
    PathBuf::from(PathBuf::from(path).parent().unwrap())
        .display()
        .to_string()
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
pub fn tokio_runtime() -> tokio::runtime::Runtime {
    debug!("=> {} Tokio runtime", Label::using());
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}
