//! # Vale interface
//!
//! Provides programmtic access to the [Vale prose analyzer](https://vale.sh/) - an open-source, command-line tool that brings your editorial style guide to life.
//!
use super::runtime::StaticAnalyzer;
use crate::analyzer::error::DocumentTarget;
pub use crate::analyzer::model::{ValeOutput, ValeOutputItem, ValeOutputItemAction, ValeOutputItemSeverity};
use crate::io::document::DocumentQuery;
use crate::io::ApiResult;
use crate::util::constants::app::{APPLICATION, ORGANIZATION, QUALIFIER};
use crate::util::constants::vale::{DISABLED_VALE_RULES, ENABLED_VALE_PACKAGES};
use crate::util::{is_uri_or_path, Label};
use crate::With;
use acorn_core::util::{SemanticVersion, StringExt};
use ariadne::{Color, ReportKind};
use bon::Builder;
use color_eyre::eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use derive_more::Display;
use directories::ProjectDirs;
use itertools::Itertools;
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
#[derive(Builder, Clone, Debug, Display, Eq, PartialEq, With)]
#[display("{:?}", path)]
#[builder(start_fn = init)]
pub struct ValeConfig {
    #[builder(default = false)]
    pub(super) custom_directory: bool,
    /// Path to Vale configuration
    #[builder(default)]
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
impl Default for ValeConfig {
    fn default() -> Self {
        let config = ValeConfig::init().build().normalize();
        trace!("=> {} Default - {:#?}", Label::using(), config.dimmed().cyan());
        config
    }
}
impl Vale {
    /// Return the platform-standard directory for ACORN-managed Vale artifacts.
    pub fn managed_directory(&self) -> ApiResult<PathBuf> {
        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .map(|directories| directories.cache_dir().join(self.command()))
            .ok_or_else(|| eyre!("Failed to determine the ACORN cache directory for Vale"))
    }
}
impl ValeConfig {
    /// Apply the deterministic Vale configuration policy in its declared order.
    ///
    /// Package names are normalized before defaults are merged, then all list fields are stably deduplicated.
    /// Applying the pipeline more than once produces the same configuration.
    pub fn normalize(self) -> Self {
        [Self::normalize_packages, Self::merge_policy, Self::deduplicate]
            .into_iter()
            .fold(self, |config, transform| transform(config))
    }
    /// Use an existing Vale installation without modifying or synchronizing it.
    pub fn with_directory(self, directory: impl Into<PathBuf>) -> Self {
        Self {
            path: directory.into().join(".vale.ini"),
            ..self
        }
        .with_custom_directory(true)
    }
    pub(crate) fn custom_directory(&self) -> bool {
        self.custom_directory
    }
    pub(crate) fn resolve_package(value: impl AsRef<str>) -> String {
        let value = value.as_ref().trim();
        match is_uri_or_path(value) {
            | true => value.to_string(),
            | false => format!("https://github.com/errata-ai/{value}/releases/latest/download/{value}.zip"),
        }
    }
    pub(crate) fn resolve_package_sources(self) -> Self {
        Self {
            packages: self.packages.into_iter().map(Self::resolve_package).collect(),
            ..self
        }
    }
    pub(crate) fn normalize_packages(self) -> Self {
        Self {
            packages: self.packages.trim_and_compact(),
            vocabularies: self.vocabularies.trim_and_compact(),
            disabled: self.disabled.trim_and_compact(),
            ..self
        }
    }
    pub(crate) fn merge_policy(self) -> Self {
        let policy = Self::policy();
        Self {
            custom_directory: self.custom_directory,
            path: self.path,
            packages: policy.packages.into_iter().chain(self.packages).collect(),
            vocabularies: policy.vocabularies.into_iter().chain(self.vocabularies).collect(),
            disabled: policy.disabled.into_iter().chain(self.disabled).collect(),
        }
    }
    pub(crate) fn deduplicate(self) -> Self {
        Self {
            packages: self.packages.into_iter().unique().collect(),
            vocabularies: self.vocabularies.into_iter().unique().collect(),
            disabled: self.disabled.into_iter().unique().collect(),
            ..self
        }
    }
    fn policy() -> Self {
        fn strings(values: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<String> {
            values.into_iter().map(|value| value.as_ref().to_string()).collect()
        }
        Self::init()
            .packages(strings(ENABLED_VALE_PACKAGES))
            .vocabularies(strings([ORGANIZATION.to_uppercase(), APPLICATION.to_string()]))
            .disabled(strings(DISABLED_VALE_RULES))
            .build()
    }
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
