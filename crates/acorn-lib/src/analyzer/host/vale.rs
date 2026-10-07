//! # Vale interface
//!
//! Provides programmtic access to the [Vale prose analyzer](https://vale.sh/) - an open-source, command-line tool that brings your editorial style guide to life.
//!
use super::runtime::StaticAnalyzer;
use crate::analyzer::host::DocumentTargetQuery;
pub use crate::analyzer::vale::{ValeOutput, ValeOutputItem, ValeOutputItemAction, ValeOutputItemSeverity};
use crate::{
    io::{document::DocumentQuery, ApiResult},
    util::{
        constants::{
            app::{APPLICATION, ORGANIZATION, QUALIFIER},
            vale::{DISABLED_VALE_RULES, ENABLED_VALE_PACKAGES},
        },
        is_uri_or_path,
    },
    With,
};
use acorn_core::util::{SemanticVersion, StringExt};
use acorn_host::terminal::Label;
use ariadne::{Color, ReportKind};
use bon::Builder;
use color_eyre::{eyre::eyre, owo_colors::OwoColorize};
use derive_more::Display;
use directories::ProjectDirs;
use itertools::Itertools;
use std::{env::consts, path::PathBuf};
use tracing::{error, trace};

pub(crate) trait ValeSeverityRender {
    fn ariadne_color(&self) -> Color;
    fn report_kind(&self) -> ReportKind<'static>;
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
impl Vale {
    /// Return the platform-standard directory for ACORN-managed Vale artifacts.
    pub fn managed_directory(&self) -> ApiResult<PathBuf> {
        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .map(|directories| directories.cache_dir().join(self.command()))
            .ok_or_else(|| eyre!("Failed to determine the ACORN cache directory for Vale"))
    }
}
impl Default for ValeConfig {
    fn default() -> Self {
        let config = ValeConfig::init().build().normalize();
        trace!("=> {} Default - {:#?}", Label::using(), config.dimmed().cyan());
        config
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
impl DocumentTargetQuery for ValeOutputItem {
    fn query(&self) -> DocumentQuery {
        match self.word_match.is_empty() {
            | true => DocumentQuery::new(),
            | false => DocumentQuery::new().with_needle(self.word_match.clone()),
        }
    }
}
impl ValeSeverityRender for ValeOutputItemSeverity {
    fn ariadne_color(&self) -> Color {
        match self {
            | ValeOutputItemSeverity::Warning => Color::Yellow,
            | ValeOutputItemSeverity::Error => Color::Red,
            | ValeOutputItemSeverity::Suggestion => Color::Blue,
        }
    }
    fn report_kind(&self) -> ReportKind<'static> {
        match self {
            | ValeOutputItemSeverity::Warning => ReportKind::Warning,
            | ValeOutputItemSeverity::Error => ReportKind::Error,
            | ValeOutputItemSeverity::Suggestion => ReportKind::Advice,
        }
    }
}
/// Parse Vale output, logging malformed native process output.
pub fn parse_vale_output(output: &str, path: PathBuf) -> Vec<ValeOutputItem> {
    match try_parse_vale_output(output, path) {
        | Ok(items) => items,
        | Err(why) => {
            error!("=> {} Parse Vale output - {why}", Label::fail());
            vec![]
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
pub(crate) fn try_parse_vale_output(output: &str, path: PathBuf) -> serde_json::Result<Vec<ValeOutputItem>> {
    let processed = preprocess_vale_output(path, output);
    if processed != "{}" {
        serde_json::from_str::<ValeOutput>(&processed).map(|ValeOutput { items }| items)
    } else {
        Ok(vec![])
    }
}
