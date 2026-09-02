//! # Vale interface
//!
//! Provides programmtic access to the [Vale prose analyzer](https://vale.sh/) - an open-source, command-line tool that brings your editorial style guide to life.
//!
pub use super::model::{ValeOutput, ValeOutputItem, ValeOutputItemAction, ValeOutputItemSeverity};
use crate::analyzer::error::DocumentTarget;
use crate::io::document::DocumentQuery;
use crate::util::constants::app::{APPLICATION, ORGANIZATION};
use crate::util::constants::vale::{DISABLED_VALE_RULES, ENABLED_VALE_PACKAGES};
use crate::util::Label;
use acorn_core::util::SemanticVersion;
use ariadne::{Color, ReportKind};
use bon::Builder;
use color_eyre::owo_colors::OwoColorize;
use derive_more::Display;
use std::env::consts;
use std::path::PathBuf;
use tracing::{error, trace};

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
/// See <https://vale.sh/docs/vale-ini> for more information
#[derive(Builder, Clone, Debug, Display)]
#[display("{:?}", path)]
#[builder(start_fn = init)]
pub struct ValeConfig {
    /// Path to Vale configuration
    #[builder(default = PathBuf::from("./.vale/.vale.ini"))]
    pub path: PathBuf,
    /// List of Vale packages
    ///
    /// See <https://vale.sh/docs/keys/packages> for more information
    #[builder(default = Vec::<String>::new())]
    pub packages: Vec<String>,
    /// List of Vale vocabularies
    ///
    /// See <https://vale.sh/docs/keys/vocab> for more information
    #[builder(default = Vec::<String>::new())]
    pub vocabularies: Vec<String>,
    /// List of Vale rules to disable
    ///
    /// See <https://vale.sh/docs/styles#rules> for more information
    #[builder(default = Vec::<String>::new())]
    pub disabled: Vec<String>,
}
impl ValeOutput {
    /// Parse Vale output
    pub fn parse(output: &str, path: PathBuf) -> Vec<ValeOutputItem> {
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
}
impl Default for ValeConfig {
    fn default() -> Self {
        fn to_string(values: Vec<&str>) -> Vec<String> {
            values.iter().map(|s| s.to_string()).collect()
        }
        let config = ValeConfig::init()
            .packages(to_string(ENABLED_VALE_PACKAGES.to_vec()))
            .vocabularies(to_string(vec![&ORGANIZATION.to_uppercase(), APPLICATION]))
            .disabled(to_string(DISABLED_VALE_RULES.to_vec()))
            .build();
        trace!("=> {} Default - {:#?}", Label::using(), config.dimmed().cyan());
        config
    }
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
impl From<&ValeOutputItemSeverity> for Color {
    fn from(value: &ValeOutputItemSeverity) -> Self {
        match value {
            | ValeOutputItemSeverity::Warning => Color::Yellow,
            | ValeOutputItemSeverity::Error => Color::Red,
            | ValeOutputItemSeverity::Suggestion => Color::Blue,
        }
    }
}
impl From<&ValeOutputItemSeverity> for ReportKind<'_> {
    fn from(value: &ValeOutputItemSeverity) -> Self {
        match value {
            | ValeOutputItemSeverity::Warning => ReportKind::Warning,
            | ValeOutputItemSeverity::Error => ReportKind::Error,
            | ValeOutputItemSeverity::Suggestion => ReportKind::Custom("Suggestion", Color::Blue),
        }
    }
}
impl DocumentTarget for ValeOutputItem {
    fn locator(&self) -> String {
        let character = self.span.first().copied().unwrap_or_default();
        format!("Line {}, Character {}", self.line, character)
    }
    fn source_text(&self) -> String {
        match self.word_match.is_empty() {
            | true => " ".to_string(),
            | false => self.word_match.clone(),
        }
    }
    fn query(&self) -> DocumentQuery {
        match self.word_match.is_empty() {
            | true => DocumentQuery::new(),
            | false => DocumentQuery::new().with_needle(self.word_match.clone()),
        }
    }
}
/// Preprocess Vale output
#[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
pub(crate) fn preprocess_vale_output(path: PathBuf, output: &str) -> String {
    let input = path.display().to_string();
    output.replace(&input, "items")
}
/// Preprocess Vale output
#[cfg(windows)]
pub(crate) fn preprocess_vale_output(path: PathBuf, output: &str) -> String {
    let input = path.as_path().display().to_string().replace("\\", "/");
    // First replace double backslashes with forward slashes
    let normalized = output.replace("\\\\", "/");
    // Then handle the extended-length path prefix and replace the full path
    normalized.replace(&format!("//?/{}", input), "items").replace(&input, "items")
}
pub(super) fn release_filename() -> String {
    match consts::OS.to_lowercase().as_str() {
        | "linux" => "Linux_64-bit.tar.gz",
        | "macos" | "apple" => "macOS_64-bit.tar.gz",
        | "windows" => "Windows_64-bit.zip",
        | _ => "unknown",
    }
    .to_string()
}
