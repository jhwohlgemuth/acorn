//! # Vale interface
//!
//! Provides programmtic access to [Vale prose analyzer](https://vale.sh/).
use crate::util::{Label, SemanticVersion};
use bon::Builder;
use color_eyre::owo_colors::OwoColorize;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::error;

/// Vale output severity
///
/// See <https://vale.sh/docs/keys/minalertlevel>
#[derive(Clone, Debug, Default, Display, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValeOutputItemSeverity {
    /// Warning
    ///
    /// Should strongly consider fixing
    #[display("warning")]
    Warning,
    /// Error
    ///
    /// Should fix
    #[display("error")]
    Error,
    /// Suggestion
    ///
    /// Should consider fixing or changing
    #[default]
    #[display("suggestion")]
    Suggestion,
}
/// Vale installation details
#[derive(Builder, Clone, Debug, Default, Display)]
#[display("{:?}", version)]
#[builder(start_fn = init)]
pub struct Vale {
    /// Vale version
    pub version: Option<SemanticVersion>,
    /// Path to `vale` binary
    pub binary: Option<PathBuf>,
    /// Path to Vale configuration
    pub config: Option<ValeConfig>,
}
/// Vale configuration
///
/// See <https://vale.sh/docs/vale-ini>
#[derive(Builder, Clone, Debug, Display)]
#[display("{:?}", path)]
#[builder(start_fn = init)]
pub struct ValeConfig {
    /// Path to Vale configuration
    #[builder(default = PathBuf::from("./.vale/.vale.ini"))]
    pub path: PathBuf,
    /// List of Vale packages
    ///
    /// See <https://vale.sh/docs/keys/packages>
    #[builder(default = Vec::<String>::new())]
    pub packages: Vec<String>,
    /// List of Vale vocabularies
    ///
    /// See <https://vale.sh/docs/keys/vocab>
    #[builder(default = Vec::<String>::new())]
    pub vocabularies: Vec<String>,
    /// List of Vale rules to disable
    ///
    /// See <https://vale.sh/docs/styles#rules>
    #[builder(default = Vec::<String>::new())]
    pub disabled: Vec<String>,
}
/// Vale output
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValeOutput {
    /// List of output items
    pub items: Vec<ValeOutputItem>,
}
/// Vale output item
///
/// The primary purpose of this struct is to enable presenting a custom view of Vale output
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
    /// Line number
    pub line: u32,
}
/// Vale output item action
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ValeOutputItemAction {
    /// Action name
    name: String,
    /// Action parameters
    params: Option<Vec<String>>,
}
impl ValeOutputItemSeverity {
    /// Returns colored output based on severity
    pub fn colored(&self) -> String {
        match self {
            | ValeOutputItemSeverity::Warning => self.to_string().yellow().to_string(),
            | ValeOutputItemSeverity::Error => self.to_string().red().to_string(),
            | ValeOutputItemSeverity::Suggestion => self.to_string().blue().to_string(),
        }
    }
}
/// Preprocess Vale output
#[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
pub fn preprocess_vale_output(path: PathBuf, output: &str) -> String {
    output.replace(path.to_str().unwrap(), "items")
}
/// Preprocess Vale output
#[cfg(windows)]
pub fn preprocess_vale_output(path: PathBuf, output: &str) -> String {
    let input = path.as_path().display().to_string().replace("\\", "/");
    output.replace("\\\\", "/").replace(&input, "items")
}
/// Parse Vale output
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
/// Print Vale output
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
