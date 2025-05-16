use crate::util::{Label, SemanticVersion};
use bon::Builder;
use color_eyre::owo_colors::OwoColorize;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::error;

#[derive(Clone, Debug, Default, Display, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValeOutputItemSeverity {
    #[display("warning")]
    Warning,
    #[display("error")]
    Error,
    #[default]
    #[display("suggestion")]
    Suggestion,
}
#[derive(Builder, Clone, Debug, Default, Display)]
#[display("{:?}", version)]
#[builder(start_fn = init)]
pub struct Vale {
    pub version: Option<SemanticVersion>,
    pub binary: Option<PathBuf>,
    pub config: Option<ValeConfig>,
}
#[derive(Builder, Clone, Debug, Display)]
#[display("{:?}", path)]
#[builder(start_fn = init)]
pub struct ValeConfig {
    #[builder(default = PathBuf::from("./.vale/.vale.ini"))]
    pub path: PathBuf,
    #[builder(default = Vec::<String>::new())]
    pub packages: Vec<String>,
    #[builder(default = Vec::<String>::new())]
    pub vocabularies: Vec<String>,
    #[builder(default = Vec::<String>::new())]
    pub disabled: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValeOutput {
    pub items: Vec<ValeOutputItem>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ValeOutputItem {
    action: ValeOutputItemAction,
    span: Vec<u32>,
    check: String,
    description: String,
    link: String,
    message: String,
    severity: ValeOutputItemSeverity,
    #[serde(rename = "Match")]
    word_match: String,
    pub line: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ValeOutputItemAction {
    name: String,
    params: Option<Vec<String>>,
}
impl ValeOutputItemSeverity {
    pub fn colored(&self) -> String {
        match self {
            | ValeOutputItemSeverity::Warning => self.to_string().yellow().to_string(),
            | ValeOutputItemSeverity::Error => self.to_string().red().to_string(),
            | ValeOutputItemSeverity::Suggestion => self.to_string().blue().to_string(),
        }
    }
}
#[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
pub fn preprocess_vale_output(path: PathBuf, output: &str) -> String {
    output.replace(path.to_str().unwrap(), "items")
}
#[cfg(windows)]
pub fn preprocess_vale_output(path: PathBuf, output: &str) -> String {
    let input = path.as_path().display().to_string().replace("\\", "/");
    output.replace("\\\\", "/").replace(&input, "items")
}
pub fn parse_vale_output(path: PathBuf, output: &str) -> Vec<ValeOutputItem> {
    let processed = preprocess_vale_output(path, output);
    if processed != "{}" {
        let parsed: serde_json::Result<ValeOutput> = serde_json::from_str(&processed);
        match parsed {
            | Ok(ValeOutput { items }) => items,
            | Err(why) => {
                error!("=> {} Parse Vale output - {why}", Label::fail());
                vec![]
            }
        }
    } else {
        vec![]
    }
}
pub fn print_vale_output(items: Vec<ValeOutputItem>) {
    for item in items {
        let ValeOutputItem {
            check,
            line,
            message,
            severity,
            span,
            ..
        } = item;
        let location = format!("Line {}, Character {}", line, span[0]);
        println!("{:<24} {:<21} {} {}", location, severity.colored(), message, check.dimmed());
    }
}
