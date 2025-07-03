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
use nanoid::nanoid;
use owo_colors::{OwoColorize, Style, Styled};
use ring::digest::{Context, SHA256};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use similar::{
    ChangeTag::{self, Delete, Equal, Insert},
    TextDiff,
};
use std::fs::create_dir_all;
use std::fs::File;
use std::io::{copy, BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, error, warn};
use which::which;

/// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/MIME_types/Common_types>
#[derive(Clone, Debug, Display)]
pub enum MimeType {
    #[display("text/csv")]
    Csv,
    #[display("application/ld+json")]
    LdJson,
    #[display("image/jpeg")]
    Jpeg,
    #[display("application/json")]
    Json,
    #[display("font/otf")]
    Otf,
    #[display("image/png")]
    Png,
    #[display("image/svg+xml")]
    Svg,
    #[display("text/plain")]
    Text,
}
#[derive(Clone, Copy, Debug, Display)]
pub enum ProgrammingLanguage {
    #[display("html")]
    Html,
    #[display("markdown")]
    Markdown,
    #[display("json")]
    Json,
    #[display("yaml")]
    Yaml,
}
#[derive(Embed)]
#[folder = "assets/constants/"]
pub struct Constant;
/// Struct for using and sharing colorized logging labels
pub struct Label {}
/// Semantic version
///
/// see <https://semver.org/>
#[derive(Builder, Clone, Copy, Debug, Deserialize, Display, Serialize)]
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
impl Label {
    pub const CAUTION: Emoji<'_, '_> = Emoji("⚠️ ", "!!! ");
    pub const CHECKMARK: Emoji<'_, '_> = Emoji("✅ ", "☑ ");
    pub const PROGRESS_BAR_TEMPLATE: &str = "  {spinner:.green}{pos:>5} of{len:^5}[{bar:40.green}] {msg}";
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
        format!(" ✗ {value} ").style(style).to_string()
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
    pub fn pass() -> String {
        Label::fmt_pass("SUCCESS")
    }
    pub fn fmt_pass(value: &str) -> String {
        let style = Style::new().green().bold().on_default_color();
        format!("{}{}", Label::CHECKMARK, value).style(style).to_string()
    }
    pub fn read() -> Styled<&'static &'static str> {
        let style = Style::new().green().on_default_color();
        "READ".style(style)
    }
    pub fn skip() -> String {
        Label::fmt_skip("SKIP")
    }
    pub fn fmt_skip(value: &str) -> String {
        let style = Style::new().yellow().on_default_color();
        format!("{}{} ", Label::CAUTION, value).style(style).to_string()
    }
    pub fn using() -> String {
        Label::fmt_using("USING")
    }
    pub fn fmt_using(value: &str) -> String {
        let style = Style::new().cyan();
        value.style(style).to_string()
    }
}
impl Constant {
    pub fn from_asset(file_name: &str) -> String {
        match Constant::get(file_name) {
            | Some(value) => String::from_utf8_lossy(value.data.as_ref()).into(),
            | None => {
                error!(file_name, "=> {} Import Constant asset", Label::fail());
                panic!("Unable to import {file_name}")
            }
        }
    }
    pub fn last_values(file_name: &str) -> impl Iterator<Item = String> {
        Constant::csv(file_name)
            .into_iter()
            .map(|x| match x.last() {
                | Some(value) => value.to_string(),
                | None => "".to_string(),
            })
            .filter(|x| !x.is_empty())
    }
    pub fn read_lines(file_name: &str) -> Vec<String> {
        let data = Constant::from_asset(file_name);
        data.lines().map(String::from).collect()
    }
    pub fn csv(file_name: &str) -> Vec<Vec<String>> {
        Constant::read_lines(format!("{file_name}.csv").as_str())
            .into_iter()
            .map(|x| x.split(",").map(String::from).collect())
            .collect()
    }
}
impl MimeType {
    pub fn from_path(file_name: &str) -> MimeType {
        let name = &file_name.to_lowercase();
        match extension(Path::new(name)).as_str() {
            | "csv" => MimeType::Csv,
            | "jpg" | "jpeg" => MimeType::Jpeg,
            | "json" => MimeType::Json,
            | "jsonld" => MimeType::LdJson,
            | "otf" => MimeType::Otf,
            | "png" => MimeType::Png,
            | "svg" => MimeType::Svg,
            | "txt" => MimeType::Text,
            | _ => unimplemented!("Unsupported MIME type"),
        }
    }
}
impl SemanticVersion {
    pub fn from_command(name: &str) -> Option<SemanticVersion> {
        if test_command(name.to_owned()) {
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
    pub fn from_string(value: &str) -> SemanticVersion {
        let value = match Regex::new(r"\d*[.]\d*[.]\d*") {
            | Ok(re) => match re.find(value) {
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
// Get SHA256 hash of a file
// https://rust-lang-nursery.github.io/rust-cookbook/cryptography/hashing.html
pub fn checksum(path: PathBuf) -> String {
    match File::open(path.clone()) {
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
            error!(error = err.to_string(), path = path_to_string(path), "=> {} Read file", Label::fail());
            "".to_string()
        }
    }
}
pub fn command_folder(task: &str, default: Option<PathBuf>) -> PathBuf {
    let root = match default {
        | Some(value) => value,
        | None => match ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
            | Some(dirs) => dirs.cache_dir().join(task).to_path_buf(),
            | None => PathBuf::from(format!("./{task}")),
        },
    };
    match create_dir_all(root.clone()) {
        | Ok(_) => {}
        | Err(why) => error!(directory = path_to_string(root.clone()), "=> {} Create - {}", Label::fail(), why),
    };
    root.join(generate_guid())
}
pub fn download_binary(url: String, destination: PathBuf) -> Result<PathBuf, String> {
    async fn download(url: String, destination: PathBuf) -> Result<(), String> {
        let client = reqwest::Client::new();
        let response = client.get(url.clone()).send();
        let filename = PathBuf::from(url.clone()).file_name().unwrap().to_str().unwrap().to_string();
        match response.await {
            | Ok(data) => match data.bytes().await {
                | Ok(content) => {
                    let mut output = File::create(destination.join(filename.clone())).unwrap();
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
    let _ = runtime.block_on(download(url.clone(), destination.clone()));
    let filename = PathBuf::from(url.clone()).file_name().unwrap().to_str().unwrap().to_string();
    Ok(destination.join(filename))
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
pub fn files_all(path: PathBuf, extension: Option<&str>, ignore: Option<String>) -> Vec<PathBuf> {
    fn paths_to_vec(paths: glob::Paths) -> Vec<PathBuf> {
        paths.collect::<Vec<_>>().into_iter().filter_map(|x| x.ok()).collect::<Vec<_>>()
    }
    if path.is_dir() {
        let pattern = match extension {
            | Some(value) => {
                let ext = &value.to_lowercase();
                format!("{}/**/*.{}", path_to_string(path), ext)
            }
            | None => {
                format!("{}/**/*", path_to_string(path))
            }
        };
        debug!(pattern, "=> {}", Label::using());
        let paths = glob(&pattern);
        match ignore {
            | Some(ignore_pattern) => match Regex::new(&ignore_pattern) {
                | Ok(re) => match paths {
                    | Ok(values) => paths_to_vec(values)
                        .into_iter()
                        .filter(|path| !re.is_match(&path_to_string(path.to_path_buf())).unwrap())
                        .collect(),
                    | Err(why) => {
                        error!("=> {} Get all files - {why}", Label::fail());
                        vec![]
                    }
                },
                | Err(why) => {
                    error!("=> {} get_all_files Regex - {why}", Label::fail());
                    vec![]
                }
            },
            | None => match paths {
                | Ok(values) => paths_to_vec(values),
                | Err(why) => {
                    error!("=> {} Get all files - {why}", Label::fail());
                    vec![]
                }
            },
        }
    } else {
        if extension.is_some() {
            warn!(
                path = path_to_string(path.clone()),
                "=> {} Extension passed with single file to get_all_files() - please make sure this is desired",
                Label::using()
            );
        }
        vec![path]
    }
}
pub fn files_from_git_branch(value: &str, extension: Option<&str>) -> Vec<PathBuf> {
    if test_command("git".to_owned()) {
        let default_branch = match git_default_branch_name() {
            | Some(value) => value,
            | None => "main".to_string(),
        };
        let args = vec!["diff", "--name-only", &default_branch, "--merge-base", value];
        let result = cmd("git", args).read();
        filter_git_command_result(result, extension)
    } else {
        vec![]
    }
}
pub fn files_from_git_commit(value: &str, extension: Option<&str>) -> Vec<PathBuf> {
    if test_command("git".to_owned()) {
        let args = vec!["diff-tree", "--no-commit-id", "--name-only", "-r", value];
        let result = cmd("git", args).read();
        filter_git_command_result(result, extension)
    } else {
        vec![]
    }
}
fn filter_git_command_result(result: Result<String, std::io::Error>, file_extension: Option<&str>) -> Vec<PathBuf> {
    match result {
        | Ok(value) => match file_extension {
            | Some(ext) => value
                .to_lowercase()
                .split("\n")
                .map(PathBuf::from)
                .filter(|x| extension(Path::new(x)) == ext)
                .collect::<Vec<_>>(),
            | None => value.to_lowercase().split("\n").map(PathBuf::from).collect::<Vec<_>>(),
        },
        | Err(_) => vec![],
    }
}
pub fn generate_guid() -> String {
    let alphabet = [
        '-', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'T', 'U', 'V', 'W', 'X', 'Y', 'a', 'b', 'c', 'd',
        'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 't', 'w', 'x', 'y', 'z', '3', '4', '6', '7', '8', '9',
    ];
    let id = nanoid!(10, &alphabet);
    debug!(id, "=> {}", Label::using());
    id
}
pub fn git_branch_name() -> Option<String> {
    if test_command("git".to_owned()) {
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
pub fn git_default_branch_name() -> Option<String> {
    if test_command("git".to_owned()) {
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
pub fn image_paths(root: PathBuf) -> Vec<PathBuf> {
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
#[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
pub fn make_executable(path: &PathBuf) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path.is_executable()
}
#[cfg(windows)]
pub fn make_executable(path: &Path) -> bool {
    // TODO: Add windows support...pass through?
    path.is_executable()
}
pub fn parent(path: String) -> String {
    PathBuf::from(PathBuf::from(path).parent().unwrap()).display().to_string()
}
pub fn path_to_string(path: PathBuf) -> String {
    // NOTE: fs::canonicalize might cause problems on Windows
    let result = match std::fs::canonicalize(path.as_path()) {
        | Ok(value) => value,
        | Err(_) => path,
    };
    result.to_str().unwrap().to_string()
}
pub fn pretty_print(text: &str, syntax: ProgrammingLanguage, highlight: Vec<usize>) {
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
pub fn read_file(path: PathBuf) -> Result<String, std::io::Error> {
    let mut content = String::new();
    let _ = match File::open(path.clone()) {
        | Ok(mut file) => {
            debug!(path = path_to_string(path), "=> {}", Label::read());
            file.read_to_string(&mut content)
        }
        | Err(why) => {
            error!(path = path_to_string(path), "=> {} Cannot read file", Label::fail());
            Err(why)
        }
    };
    Ok(content)
}
pub fn suffix(value: usize) -> String {
    (if value == 1 { "" } else { "s" }).to_string()
}
pub fn test_command(name: String) -> bool {
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
pub fn tokio_runtime() -> tokio::runtime::Runtime {
    debug!("=> {} Tokio runtime", Label::using());
    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap()
}
pub fn to_string(values: Vec<&str>) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}
pub fn write_file(path: PathBuf, content: String) -> Result<(), std::io::Error> {
    match File::create(path.clone()) {
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
