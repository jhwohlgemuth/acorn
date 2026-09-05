//! Check result types and helpers for analyzer output
use super::error::ErrorKind;
use super::model::ValeOutputItemSeverity;
use super::readability::score::{ReadabilityScore, ReadabilitySeverity};
use super::readability::ReadabilityType;
use crate::{check, check_ok};
use acorn_core::prelude::{format, vec, String, ToString, Vec};
#[cfg(feature = "std")]
use acorn_core::util::MimeType;
#[cfg(feature = "std")]
use acorn_schema::schema::standard::crosswalk::mapping::{
    datacite_to_dcat, datacite_to_huwise, datacite_to_invenio, dcat_to_datacite, huwise_to_datacite, invenio_to_datacite,
};
#[cfg(feature = "std")]
use acorn_schema::schema::standard::crosswalk::{ConversionWarning, CrosswalkError, FieldMapping, Fields, SchemaExtractor};
#[cfg(feature = "std")]
use acorn_schema::schema::standard::{datacite, dcat, huwise, invenio};
#[cfg(feature = "std")]
use acorn_schema::schema::OneOrMany;
use bon::Builder;
use convert_case::{Case, Casing};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use strum::{EnumIs, EnumIter, VariantNames};

/// Collection of analyzer checks.
pub type Checks = Vec<Check>;
/// Utility type alias for a vector of [`ConversionWarning`]
#[cfg(feature = "std")]
pub type ConversionWarnings = Vec<ConversionWarning>;
/// Trait for converting to a vector of checks
#[cfg(feature = "std")]
pub trait IntoChecks {
    /// Convert to a vector of checks
    fn to_checks(&self, uri: Option<String>) -> Vec<Check>;
}
/// Various check categories available for validating research activity data
#[derive(Clone, Debug, Default, Deserialize, Display, EnumIter, PartialEq, Serialize, VariantNames)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum CheckCategory {
    /// Schema validation check
    #[default]
    #[display("schema")]
    Schema,
    /// Website availability check
    #[display("link")]
    Link,
    /// Static analysis of prose
    #[display("prose")]
    Prose,
    /// Quality control and data consistency check
    #[display("quality")]
    Quality,
    /// Readability check using one of several metrics
    #[display("readability")]
    Readability,
    /// Metadata crosswalk conversion warning
    #[display("crosswalk")]
    Crosswalk,
}
#[derive(Clone, Copy, Debug, Default, EnumIs, Eq, PartialEq)]
/// Rendering behavior associated with a check.
pub enum CheckKind {
    /// Standard category-specific rendering.
    #[default]
    Standard,
    /// Identifier discovery and persistence lifecycle rendering.
    Lifecycle,
}
/// Severity level of a check result
#[derive(Clone, Debug, Default, Deserialize, Display, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckSeverity {
    /// Hard error requiring immediate attention
    #[default]
    #[display("error")]
    Error,
    /// Informational message
    #[display("info")]
    Info,
    /// Advisory suggestion to improve content quality
    #[display("suggestion")]
    Suggestion,
    /// Content issue that should be addressed
    #[display("warning")]
    Warning,
}
/// Output format used when rendering checks and discovery reports.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OutputFormat {
    /// JavaScript Object Notation
    #[default]
    Json,
    /// Human-readable console output
    Console,
    /// GitHub-Flavored Markdown
    Markdown,
    /// Plain-text values, one per line
    Raw,
    /// YAML
    Yaml,
}
/// Metadata standard used to gate processing behavior within commands
#[derive(Clone, Copy, Debug, Default, Display, PartialEq, Eq)]
pub enum Standard {
    /// Research Activity Data (RAD)
    #[default]
    #[display("research-activity-data")]
    ResearchActivityData,
    /// Citation File Format (CFF)
    #[display("citation-file-format")]
    CitationFileFormat,
    /// DataCite
    #[display("datacite")]
    Datacite,
    /// Data Catalog (DCAT)
    #[display("dcat")]
    Dcat,
    /// Document-derived text
    #[display("document")]
    Document,
    /// Dublin Core
    #[display("dublin-core")]
    DublinCore,
    /// HUBwise
    #[display("huwise")]
    Huwise,
    /// InvenioRDM
    #[display("invenio")]
    Invenio,
    /// Research Activity Identifier metadata (RAiD)
    #[display("raid")]
    Raid,
    /// Plain text
    #[display("text")]
    Text,
}
/// Data structure for holding the result of a schema validation check
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
#[cfg_attr(not(feature = "analysis"), allow(dead_code))]
pub struct Check {
    /// Position of this issue in rendered output
    #[builder(default = 0)]
    pub index: usize,
    /// Check category
    pub category: CheckCategory,
    /// Unified issue location for checks (prose location or schema path)
    pub locator: Option<String>,
    /// Per-item normalized prose locations retained without host document state.
    #[builder(default)]
    pub(super) prose_locators: Vec<Option<String>>,
    /// Textual context of check (e.g., paragraph where prose issues were found)
    pub context: Option<String>,
    /// Internal payload used by check-specific renderers (e.g., prose source text)
    pub(super) data: Option<String>,
    #[builder(default)]
    pub(super) kind: CheckKind,
    /// Whether or not the check was successful
    #[builder(default = false)]
    pub success: bool,
    /// Severity of the check result
    #[builder(default)]
    pub severity: CheckSeverity,
    /// HTTP status code
    pub(super) status_code: Option<String>,
    /// Errors and issues found during check
    pub errors: Option<ErrorKind>,
    /// Path of file being validated
    pub uri: Option<String>,
    /// Message related to or description of validation issue (e.g., key name of invalid value, result of validation, etc.)
    #[builder(default = "".to_string())]
    pub message: String,
}
impl Check {
    #[cfg_attr(not(feature = "analysis"), allow(dead_code))]
    pub(crate) fn with_kind(self, kind: CheckKind) -> Self {
        Self { kind, ..self }
    }
    /// Returns whether this check represents a failure (not successful and severity warrants failure)
    pub fn is_failure(&self) -> bool {
        !self.success && self.severity.is_failure()
    }
    /// Returns a function that filters checks based on the given log level threshold
    pub fn is_visible_at(level: Option<u8>) -> impl Fn(&Check) -> bool {
        move |check: &Check| check.severity.is_visible_at(level)
    }
    /// Returns the number of errors
    pub fn issue_count(&self) -> usize {
        match self.category {
            | CheckCategory::Link | CheckCategory::Quality | CheckCategory::Readability => 1,
            | CheckCategory::Prose => {
                if let Some(kind) = &self.errors {
                    match kind {
                        | ErrorKind::Vale(values) => values.len(),
                        | _ => 1,
                    }
                } else if !self.message.is_empty() {
                    1
                } else {
                    0
                }
            }
            | CheckCategory::Schema => {
                if let Some(kind) = &self.errors {
                    match kind {
                        | ErrorKind::Validator(content) => content.len(),
                        | _ => 0,
                    }
                } else {
                    0
                }
            }
            | CheckCategory::Crosswalk => {
                if self.success {
                    0
                } else {
                    1
                }
            }
        }
    }
    /// Returns a new LinkCheckResult with the given context
    pub fn with_context(self, value: String) -> Self {
        Self {
            context: Some(value),
            ..self
        }
    }
    /// Returns a new check with renderer data attached.
    pub fn with_data(self, value: String) -> Self {
        Self { data: Some(value), ..self }
    }
    /// Returns a new LinkCheckResult with the given index
    pub fn with_index(self, value: usize) -> Self {
        Self { index: value, ..self }
    }
    /// Returns a new LinkCheckResult with the given locator
    pub fn with_locator(self, value: Option<String>) -> Self {
        Self { locator: value, ..self }
    }
    /// Returns a new LinkCheckResult with the given URL
    pub fn with_uri(self, value: Option<String>) -> Self {
        Self { uri: value, ..self }
    }
}
impl CheckCategory {
    /// Returns `true` if this category appears in the given list of category items
    pub fn is_in(&self, items: &[String]) -> bool {
        items.iter().any(|value| Self::from(value.as_str()) == *self)
    }
    #[cfg_attr(not(feature = "analysis"), allow(dead_code))]
    pub(super) fn message(&self) -> &'static str {
        match self {
            | Self::Link => "URL",
            | Self::Prose => "Prose",
            | Self::Quality => "Quality",
            | Self::Readability => "Readability",
            | Self::Schema => "Deserialization error",
            | Self::Crosswalk => "Crosswalk warning",
        }
    }
}
impl From<&str> for CheckCategory {
    fn from(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            | "link" => Self::Link,
            | "prose" => Self::Prose,
            | "quality" => Self::Quality,
            | "readability" => Self::Readability,
            | "crosswalk" => Self::Crosswalk,
            | "schema" => Self::Schema,
            | _ => CheckCategory::default(),
        }
    }
}
impl From<&String> for CheckCategory {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}
impl From<String> for CheckCategory {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}
impl CheckSeverity {
    /// Returns whether this severity should cause check failure.
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Error | Self::Warning)
    }
}
impl CheckSeverity {
    /// Returns whether this severity level is visible at the given log level.
    ///
    /// - [`CheckSeverity::Error`] and [`CheckSeverity::Warning`] are always visible.
    /// - [`CheckSeverity::Suggestion`] is visible at `warn` level and above.
    /// - [`CheckSeverity::Info`] is visible at `info` level and above.
    pub fn is_visible_at(&self, level: Option<u8>) -> bool {
        match self {
            | Self::Error | Self::Warning => true,
            | Self::Suggestion => matches!(level, Some(2..=u8::MAX)),
            | Self::Info => matches!(level, Some(3..=u8::MAX)),
        }
    }
}
impl From<ValeOutputItemSeverity> for CheckSeverity {
    fn from(value: ValeOutputItemSeverity) -> Self {
        match value {
            | ValeOutputItemSeverity::Error => CheckSeverity::Error,
            | ValeOutputItemSeverity::Suggestion => CheckSeverity::Suggestion,
            | ValeOutputItemSeverity::Warning => CheckSeverity::Warning,
        }
    }
}
#[cfg(feature = "std")]
impl IntoChecks for ConversionWarnings {
    /// Convert crosswalk conversion warnings into Check objects with Warning severity.
    ///
    /// Each ConversionWarning from a field-level loss during schema crosswalk produces
    /// a single Check with the field name as locator and a human-readable message.
    fn to_checks(&self, uri: Option<String>) -> Vec<Check> {
        self.iter()
            .map(|w| {
                let context = format!("{} → {}", w.from, w.to);
                check!(
                    CheckCategory::Crosswalk,
                    false,
                    severity: CheckSeverity::Warning,
                    message: w.to_string(),
                    context: context,
                    locator: w.field.clone(),
                )
                .with_uri(uri.clone())
            })
            .collect()
    }
}
impl From<&Map<String, Value>> for Standard {
    fn from(object: &Map<String, Value>) -> Self {
        if object.get("@type").and_then(Value::as_str).is_some_and(|value| value.contains("dcat:")) {
            Standard::Dcat
        } else if object.contains_key("dataset_id") && object.contains_key("metas") {
            Standard::Huwise
        } else if object.contains_key("attributes") && object.contains_key("id") {
            Standard::Datacite
        } else if object.contains_key("metadata") || object.contains_key("pids") || object.contains_key("resource_type") {
            Standard::Invenio
        } else {
            Standard::Dcat
        }
    }
}
#[cfg(feature = "std")]
impl Standard {
    /// Convert serialized content from one metadata standard to another.
    ///
    /// Parses `content` as the schema identified by `self` (source standard),
    /// converts each record to the `target` standard using `TryFrom` pipeline,
    /// and serializes the result as `output_mime` (JSON or YAML).
    ///
    /// ### Errors
    /// Returns [`CrosswalkError`] when:
    /// - Content cannot be parsed as the source standard
    /// - A required conversion path is not supported
    /// - A record fails conversion
    /// - Output serialization fails
    pub fn crosswalk(&self, content: &str, mime: MimeType, target: Standard, target_mime: MimeType) -> Result<String, CrosswalkError> {
        match *self {
            | Standard::Datacite => OneOrMany::<datacite::Record>::parse(content, mime).and_then(|source| match target {
                | Standard::Datacite => source.serialize(target_mime),
                | Standard::Dcat => source.map(dcat::Dataset::try_from).and_then(|b| b.serialize(target_mime)),
                | Standard::Invenio => source.map(invenio::Record::try_from).and_then(|b| b.serialize(target_mime)),
                | Standard::Huwise => source.map(huwise::Dataset::try_from).and_then(|b| b.serialize(target_mime)),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | Standard::Dcat => OneOrMany::<dcat::Dataset>::parse(content, mime).and_then(|source| match target {
                | Standard::Dcat => source.serialize(target_mime),
                | Standard::Datacite => source.map(datacite::Record::try_from).and_then(|b| b.serialize(target_mime)),
                | Standard::Invenio => source
                    .map(|v| datacite::Record::try_from(v).and_then(invenio::Record::try_from))
                    .and_then(|b| b.serialize(target_mime)),
                | Standard::Huwise => source
                    .map(|v| datacite::Record::try_from(v).and_then(huwise::Dataset::try_from))
                    .and_then(|b| b.serialize(target_mime)),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | Standard::Invenio => OneOrMany::<invenio::Record>::parse(content, mime).and_then(|source| match target {
                | Standard::Invenio => source.serialize(target_mime),
                | Standard::Datacite => source.map(datacite::Record::try_from).and_then(|b| b.serialize(target_mime)),
                | Standard::Dcat => source
                    .map(|v| datacite::Record::try_from(v).and_then(dcat::Dataset::try_from))
                    .and_then(|b| b.serialize(target_mime)),
                | Standard::Huwise => source
                    .map(|v| datacite::Record::try_from(v).and_then(huwise::Dataset::try_from))
                    .and_then(|b| b.serialize(target_mime)),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | Standard::Huwise => OneOrMany::<huwise::Dataset>::parse(content, mime).and_then(|source| match target {
                | Standard::Huwise => source.serialize(target_mime),
                | Standard::Datacite => source.map(datacite::Record::try_from).and_then(|b| b.serialize(target_mime)),
                | Standard::Dcat => source
                    .map(|v| datacite::Record::try_from(v).and_then(dcat::Dataset::try_from))
                    .and_then(|b| b.serialize(target_mime)),
                | Standard::Invenio => source
                    .map(|v| datacite::Record::try_from(v).and_then(invenio::Record::try_from))
                    .and_then(|b| b.serialize(target_mime)),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | _ => Err(self.unsupported_crosswalk(target)),
        }
    }
    /// Convert serialized content from one metadata standard to another, returning both the converted content and any field-level warnings.
    ///
    /// Warnings indicate optional fields in the source that have no equivalent in the target schema, or fields whose semantics differ between schemas.
    pub fn crosswalk_with_warnings(
        &self,
        content: &str,
        mime: MimeType,
        target: Standard,
        target_mime: MimeType,
    ) -> Result<(String, ConversionWarnings), CrosswalkError> {
        self.collect_crosswalk_warnings(content, &mime, target)
            .and_then(|warnings| self.crosswalk(content, mime, target, target_mime).map(|content| (content, warnings)))
    }
    /// Run the FieldMapping pipeline to detect field-level losses for a given pair.
    fn collect_crosswalk_warnings(&self, content: &str, mime: &MimeType, target: Standard) -> Result<ConversionWarnings, CrosswalkError> {
        match (*self, target) {
            | (Standard::Datacite, Standard::Dcat) => {
                process_crosswalk_warnings::<datacite::Record>(content, mime, "datacite", "dcat", datacite_to_dcat())
            }
            | (Standard::Datacite, Standard::Invenio) => {
                process_crosswalk_warnings::<datacite::Record>(content, mime, "datacite", "invenio", datacite_to_invenio())
            }
            | (Standard::Datacite, Standard::Huwise) => {
                process_crosswalk_warnings::<datacite::Record>(content, mime, "datacite", "huwise", datacite_to_huwise())
            }
            | (Standard::Dcat, Standard::Datacite) => {
                process_crosswalk_warnings::<dcat::Dataset>(content, mime, "dcat", "datacite", dcat_to_datacite())
            }
            | (Standard::Invenio, Standard::Datacite) => {
                process_crosswalk_warnings::<invenio::Record>(content, mime, "invenio", "datacite", invenio_to_datacite())
            }
            | (Standard::Huwise, Standard::Datacite) => {
                process_crosswalk_warnings::<huwise::Dataset>(content, mime, "huwise", "datacite", huwise_to_datacite())
            }
            | _ => Ok(Vec::new()),
        }
    }
    fn unsupported_crosswalk(&self, target: Standard) -> CrosswalkError {
        CrosswalkError::BuildFailed(format!(
            "Unsupported metadata crosswalk pair: {self} -> {target} (supported: datacite, dcat, invenio, huwise)"
        ))
    }
}
impl From<&CheckCategory> for u8 {
    fn from(value: &CheckCategory) -> Self {
        match value {
            | CheckCategory::Schema | CheckCategory::Link => 0,
            | CheckCategory::Prose => 1,
            | CheckCategory::Quality => 2,
            | CheckCategory::Readability => 3,
            | CheckCategory::Crosswalk => 4,
        }
    }
}
impl From<&CheckSeverity> for u8 {
    fn from(value: &CheckSeverity) -> Self {
        match value {
            | CheckSeverity::Error => 0,
            | CheckSeverity::Warning => 1,
            | CheckSeverity::Suggestion => 2,
            | CheckSeverity::Info => 3,
        }
    }
}
/// Portable readability check over supplied text (no filesystem or env)
#[cfg(feature = "alloc")]
pub fn check_readability_for_text(text: &str, metric: ReadabilityType, uri: Option<String>) -> Vec<Check> {
    let maximum = metric.maximum_allowed();
    let calculated = metric.calculate(text);
    let assessment = ReadabilityScore::new(metric, calculated, maximum);
    let message = assessment.to_string();
    if assessment.is_excessive() {
        let severity = match assessment.severity() {
            | ReadabilitySeverity::Fail => CheckSeverity::Warning,
            | ReadabilitySeverity::Pass => CheckSeverity::Info,
        };
        if let Some(uri_val) = uri {
            vec![check!(
                CheckCategory::Readability,
                false,
                severity: severity,
                uri: uri_val,
                errors: ErrorKind::Readability(assessment),
                message: message,
                data: text.to_string()
            )]
        } else {
            vec![check!(
                CheckCategory::Readability,
                false,
                severity: severity,
                errors: ErrorKind::Readability(assessment),
                message: message,
                data: text.to_string()
            )]
        }
    } else if let Some(uri_val) = uri {
        vec![check_ok!(
            CheckCategory::Readability,
            uri: uri_val,
            message: message,
            data: text.to_string()
        )]
    } else {
        vec![check_ok!(
            CheckCategory::Readability,
            message: message,
            data: text.to_string()
        )]
    }
}
/// Parse source content and apply a FieldMapping to detect missing optional fields.
#[cfg(feature = "std")]
fn process_crosswalk_warnings<T>(
    content: &str,
    mime: &MimeType,
    from: &'static str,
    to: &'static str,
    mapping: FieldMapping,
) -> Result<ConversionWarnings, CrosswalkError>
where
    T: serde::de::DeserializeOwned + SchemaExtractor,
{
    OneOrMany::<T>::parse(content, mime.clone()).map(|batch| {
        let mapped_fields: Vec<&str> = mapping.rules.iter().map(|r| r.source).collect();
        let missing_warnings = |fields: &Fields, prefix: &str| -> ConversionWarnings {
            let mut target_fields = Fields::new();
            mapping
                .apply(fields, &mut target_fields)
                .unwrap_or_default()
                .into_iter()
                .map(|field| ConversionWarning::no_equivalent(from, to, format!("{prefix}{field}")))
                .collect()
        };
        let unmapped_warnings = |fields: &Fields, prefix: &str| -> ConversionWarnings {
            fields
                .keys()
                .filter(|key| !mapped_fields.contains(&key.as_str()))
                .map(|key| ConversionWarning::no_equivalent(from, to, format!("{prefix}{key}")))
                .collect()
        };
        match batch {
            | OneOrMany::One(record) => {
                let fields = record.extract_fields();
                [missing_warnings(&fields, ""), unmapped_warnings(&fields, "")].concat()
            }
            | OneOrMany::Many(records) => records
                .into_iter()
                .enumerate()
                .flat_map(|(i, record)| {
                    let fields = record.extract_fields();
                    let prefix = format!("record[{i}].");
                    missing_warnings(&fields, &prefix).into_iter().chain(unmapped_warnings(&fields, &prefix))
                })
                .collect(),
        }
    })
}
/// Create summary data table from given issues
pub fn summary(issues: Vec<Check>) -> Vec<Vec<String>> {
    [
        CheckCategory::Schema,
        CheckCategory::Link,
        CheckCategory::Prose,
        CheckCategory::Quality,
        CheckCategory::Readability,
        CheckCategory::Crosswalk,
    ]
    .iter()
    .map(|category| {
        let count = issues
            .iter()
            .filter(|issue| issue.category == *category)
            .map(|issue| issue.issue_count())
            .sum::<usize>()
            .to_string();
        vec![format!("{} items found", category.to_string().to_case(Case::Title)), count]
    })
    .collect::<Vec<_>>()
}
