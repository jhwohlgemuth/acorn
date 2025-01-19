use owo_colors::{OwoColorize, Style, Styled};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use tracing::{debug, error};

/// Struct for using and sharing colorized logging labels
pub struct Label {}
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
pub fn get_parent(path: String) -> String {
    PathBuf::from(PathBuf::from(path).parent().unwrap())
        .display()
        .to_string()
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
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}
