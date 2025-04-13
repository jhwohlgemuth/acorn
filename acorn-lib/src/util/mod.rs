use crate::constants::*;
use console::Emoji;
use data_encoding::HEXUPPER;
use derive_more::Display;
use duct::cmd;
use fancy_regex::Regex;
use glob::glob;
use is_executable::IsExecutable;
use owo_colors::{OwoColorize, Style, Styled};
use ring::digest::{Context, SHA256};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use similar::{
    ChangeTag::{self, Delete, Equal, Insert},
    TextDiff,
};
use std::error::Error;
use std::fs::File;
use std::io::copy;
use std::io::Cursor;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use tracing::{debug, error, warn};
use which::which;

#[cfg(feature = "cli")]
pub mod cli {

    use bon::Builder;
    use clap::ValueEnum;
    use derive_more::Display;
    use serde::Serialize;
    use std::path::PathBuf;

    #[derive(Clone, Debug, Display, PartialEq, ValueEnum, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Check {
        /// Static analysis of prose
        #[display("analysis")]
        Analysis,
        /// Schema validation via Rust type system
        #[display("validation")]
        Validation,
    }
    #[derive(Clone, Copy, Debug, Default, Display, PartialEq, ValueEnum, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Diagnostic {
        #[default]
        #[display("all")]
        All,
        /// System information (e.g. CPU count, OS, etc...)
        #[display("system")]
        System,
        /// Memory information and usage
        #[display("memory")]
        Memory,
        /// Network information
        #[display("network")]
        Network,
        /// Graphics Processing Unit (GPU) information
        #[display("gpu")]
        Gpu,
        /// Check for installed software
        #[display("software")]
        Software,
    }
    #[derive(Clone, Debug, Default, Display, ValueEnum, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Size {
        /// Standard size (4:3)
        #[display("standard")]
        Standard,
        /// Widescreen size (16:9)
        #[default]
        #[display("widescreen")]
        Widescreen,
    }
    #[derive(Clone, Copy, Debug, Default, Display, ValueEnum, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Target {
        #[default]
        #[display("fact-sheet")]
        FactSheet,
        #[display("highlight")]
        Highlight,
        #[display("poster")]
        Poster,
    }
    #[derive(Builder, Clone, Debug)]
    #[builder(start_fn = init)]
    pub struct Options {
        pub path: Option<PathBuf>,
        pub output: Option<PathBuf>,
        pub size: Option<Size>,
        pub target: Option<Target>,
    }
}

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
#[derive(Embed)]
#[folder = "assets/constants/"]
pub struct Constant;
/// Struct for using and sharing colorized logging labels
pub struct Label {}
/// Semantic version
///
/// see <https://semver.org/>
#[derive(Clone, Copy, Debug, Default, Deserialize, Display, Serialize)]
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
                unimplemented!()
            }
        }
    }
    pub fn read_lines(file_name: &str) -> Vec<String> {
        let data = Constant::from_asset(file_name);
        data.lines().map(String::from).collect()
    }
    pub fn csv(file_name: &str) -> Vec<Vec<String>> {
        Constant::read_lines(file_name)
            .into_iter()
            .map(|x| x.split(",").map(String::from).collect())
            .collect()
    }
}
impl MimeType {
    pub fn from_path(file_name: &str) -> MimeType {
        let name = &file_name.to_lowercase();
        match get_extension(Path::new(name)).as_str() {
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
    pub fn from_string(version: &str) -> SemanticVersion {
        let mut parts = version.split('.');
        let major = parts.next().unwrap().parse::<u32>().unwrap();
        let minor = parts.next().unwrap().parse::<u32>().unwrap();
        let patch = parts.next().unwrap().parse::<u32>().unwrap();
        SemanticVersion { major, minor, patch }
    }
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
pub fn get_all_files(path: PathBuf, extension: Option<&str>) -> Vec<PathBuf> {
    if path.is_dir() {
        let ext = match extension {
            | Some(value) => &value.to_lowercase(),
            | None => "json",
        };
        let pattern = format!("{}/**/*.{}", path_to_string(path), ext);
        debug!(pattern, "=> {}", Label::using());
        let paths = glob(&pattern);
        match paths {
            | Ok(values) => values.collect::<Vec<_>>().into_iter().filter_map(|x| x.ok()).collect::<Vec<_>>(),
            | Err(_) => vec![],
        }
    } else {
        debug!(
            path = path_to_string(path.clone()),
            "=> {} Extension passed with single file to get_all_files() - make sure this is desired",
            Label::using()
        );
        vec![path]
    }
}
pub fn get_changes(old: &str, new: &str) -> Vec<(ChangeTag, String)> {
    TextDiff::from_lines(old, new)
        .iter_all_changes()
        .map(|line| {
            let tag = line.tag();
            let text = match tag {
                | Delete => format!("- {}", line).red().to_string(),
                | Insert => format!("+ {}", line).green().to_string(),
                | Equal => format!("  {}", line).dimmed().to_string(),
            };
            (tag, text)
        })
        .collect::<Vec<_>>()
}
// Get SHA256 hash of a file
// https://rust-lang-nursery.github.io/rust-cookbook/cryptography/hashing.html
pub fn get_checksum(path: PathBuf) -> String {
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
/// Get file extension
///
/// # Examples
/// ```
/// use std::path::Path;
/// use acorn_lib::util::get_extension;
///
/// assert_eq!("txt", get_extension(Path::new("hello.txt")));
/// assert_eq!("md", get_extension(Path::new("README.md")));
/// assert_eq!("", get_extension(Path::new(".dotfile")));
/// assert_eq!("", get_extension(Path::new("/path/to/folder")));
/// ```
pub fn get_extension(path: &Path) -> String {
    path.extension().unwrap_or_default().to_str().unwrap_or_default().to_string()
}
pub fn get_files_from_git(commit: &str, extension: Option<&str>) -> Vec<PathBuf> {
    if test_command("git".to_owned()) {
        let args = vec!["diff-tree", "--no-commit-id", "--name-only", "-r", commit];
        let result = cmd("git", args).read();
        match result {
            | Ok(value) => {
                let ext = match extension {
                    | Some(value) => &value.to_lowercase(),
                    | None => "json",
                };
                value
                    .split("\n")
                    .map(PathBuf::from)
                    .filter(|x| get_extension(Path::new(x)) == ext)
                    .collect::<Vec<_>>()
            }
            | Err(_) => vec![],
        }
    } else {
        vec![]
    }
}
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
    PathBuf::from(PathBuf::from(path).parent().unwrap()).display().to_string()
}
/// Get the singular form of a word
///
/// Adapted from the PHP library, [Text-Statistics](https://github.com/DaveChild/Text-Statistics)
pub fn get_singular_form(word: &str) -> String {
    const SINGULAR: [(&str, &str); 28] = [
        (r#"(quiz)zes$"#, r#"${1}"#),
        (r#"(matr)ices$"#, r#"${1}ix"#),
        (r#"(vert|ind)ices$"#, r#"${1}ex"#),
        (r#"^(ox)en$"#, r#"${1}"#),
        (r#"(alias)es$"#, r#"${1}"#),
        (r#"(octop|vir)i$"#, r#"${1}us"#),
        (r#"(cris|ax|test)es$"#, r#"${1}is"#),
        (r#"(shoe)s$"#, r#"${1}"#),
        (r#"(o)es$"#, r#"${1}"#),
        (r#"(bus)es$"#, r#"${1}"#),
        (r#"([m|l])ice$"#, r#"${1}ouse"#),
        (r#"(x|ch|ss|sh)es$"#, r#"${1}"#),
        (r#"(m)ovies$"#, r#"${1}ovie"#),
        (r#"(s)eries$"#, r#"${1}eries"#),
        (r#"([^aeiouy]|qu)ies$"#, r#"${1}y"#),
        (r#"([lr])ves$"#, r#"${1}f"#),
        (r#"(tive)s$"#, r#"${1}"#),
        (r#"(hive)s$"#, r#"${1}"#),
        (r#"(li|wi|kni)ves$"#, r#"${1}fe"#),
        (r#"(shea|loa|lea|thie)ves$"#, r#"${1}f"#),
        (r#"(^analy)ses$"#, r#"${1}sis"#),
        (r#"((a)naly|(b)a|(d)iagno|(p)arenthe|(p)rogno|(s)ynop|(t)he)ses$"#, r#"${1}${2}sis"#),
        (r#"([ti])a$"#, r#"${1}um"#),
        (r#"(n)ews$"#, r#"${1}ews"#),
        (r#"(h|bl)ouses$"#, r#"${1}ouse"#),
        (r#"(corpse)s$"#, r#"${1}"#),
        (r#"(us)es$"#, r#"${1}"#),
        (r#"s$"#, r#""#),
    ];
    match word.to_lowercase().as_str() {
        | value if SAME_SINGULAR_PLURAL.contains(&value) => value.to_string(),
        | value if IRREGULAR_NOUNS.contains_key(&value) => value.to_string(),
        | value if IRREGULAR_NOUNS_INVERTED.contains_key(&value) => match IRREGULAR_NOUNS_INVERTED.get(value) {
            | Some(value) => value.to_string(),
            | None => value.to_string(),
        },
        | value => {
            let pair = SINGULAR.iter().find(|(pattern, _)| match Regex::new(pattern).unwrap().is_match(value) {
                | Ok(true) => true,
                | Ok(false) | Err(_) => false,
            });
            match pair {
                | Some((pattern, replacement)) => {
                    debug!(pattern, replacement, value, "=> {} Singular form conversion", Label::using());
                    let re = Regex::new(pattern).unwrap();
                    re.replace_all(value, *replacement).to_string()
                }
                | None => value.to_string(),
            }
        }
    }
}
pub fn get_syllable_count(text: &str) -> usize {
    fn tokenize(value: &str) -> Vec<String> {
        value
            .replace("é", "-e")
            .replace("ë", "-e")
            .split('-')
            .map(|x| NON_ALPHABETIC.replace_all(x, "").to_lowercase())
            .collect::<Vec<_>>()
    }
    fn get_syllables(word: String) -> usize {
        let singular = get_singular_form(&word);
        match word.as_str() {
            | "" => 0,
            | value if value.len() < 3 => 1,
            | value if PROBLEMATIC_WORDS.contains_key(value) => match PROBLEMATIC_WORDS.get(value) {
                | Some(x) => *x,
                | None => 0,
            },
            | _ if PROBLEMATIC_WORDS.contains_key(&singular.as_str()) => match PROBLEMATIC_WORDS.get(singular.as_str()) {
                | Some(x) => *x,
                | None => 0,
            },
            | value if NEED_TO_BE_FIXED.contains_key(value) => match NEED_TO_BE_FIXED.get(value) {
                | Some(x) => *x,
                | None => 0,
            },
            | _ if NEED_TO_BE_FIXED.contains_key(&singular.as_str()) => match NEED_TO_BE_FIXED.get(singular.as_str()) {
                | Some(x) => *x,
                | None => 0,
            },
            | _ => {
                let mut input = word;
                let mut count: isize = 0;
                // TODO: Combine SINGLE, DOUBLE, and TRIPLE regex operations
                count += 3 * TRIPLE.find_iter(&input).count() as isize;
                input = TRIPLE.replace_all(&input, "").to_string();
                count += 2 * DOUBLE.find_iter(&input).count() as isize;
                input = DOUBLE.replace_all(&input, "").to_string();
                count += SINGLE.find_iter(&input).count() as isize;
                input = SINGLE.replace_all(&input, "").to_string();
                count -= SINGLE_SYLLABIC_ONE.find_iter(&input).count() as isize;
                count -= SINGLE_SYLLABIC_TWO.find_iter(&input).count() as isize;
                count += DOUBLE_SYLLABIC_ONE.find_iter(&input).count() as isize;
                count += DOUBLE_SYLLABIC_TWO.find_iter(&input).count() as isize;
                count += DOUBLE_SYLLABIC_THREE.find_iter(&input).count() as isize;
                count += DOUBLE_SYLLABIC_FOUR.find_iter(&input).count() as isize;
                count += VOWEL.split(&input).filter(|x| !x.as_ref().unwrap().is_empty()).count() as isize;
                count as usize
            }
        }
    }
    let tokens = text.split_whitespace().flat_map(tokenize).collect::<Vec<String>>();
    tokens.into_iter().map(get_syllables).sum()
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
    let result = match std::fs::canonicalize(path.as_path()) {
        | Ok(value) => value,
        | Err(_) => path,
    };
    result.to_str().unwrap().to_string()
}
pub fn print_changes(old: &str, new: &str) {
    let changes = get_changes(old, new);
    let has_no_changes = changes.clone().into_iter().all(|(tag, _)| tag == Equal);
    if has_no_changes {
        debug!("=> {}No format changes", Label::skip());
    } else {
        for change in changes {
            print!("{}", change.1);
        }
    }
}
pub fn read_file(path: PathBuf) -> Result<String, Box<dyn Error>> {
    let mut content = String::new();
    let _ = match File::open(path.clone()) {
        | Ok(mut file) => {
            debug!(path = path.to_str().unwrap(), "=> {}", Label::read());
            file.read_to_string(&mut content)
        }
        | Err(err) => {
            error!(path = path.to_str().unwrap(), "=> {} Cannot read file", Label::fail());
            Err(err)
        }
    };
    Ok(content)
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
pub fn tokio_runtime() -> tokio::runtime::Runtime {
    debug!("=> {} Tokio runtime", Label::using());
    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap()
}
pub fn to_string(values: Vec<&str>) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests;
