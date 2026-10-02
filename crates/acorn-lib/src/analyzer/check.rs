//! Check result types and helpers for analyzer output
use super::error::ErrorKind;
#[cfg(any(feature = "analysis", feature = "std"))]
use super::error::{DocumentTarget, ValidatorIssues};
#[cfg(feature = "analysis")]
use super::fair::FairState;
use super::fair::{FairDetails, FairIndicatorId};
use super::model::ValeOutputItemSeverity;
use super::readability::score::{ReadabilityScore, ReadabilitySeverity};
use super::readability::ReadabilityType;
#[cfg(any(feature = "analysis", feature = "std"))]
use crate::check_err;
use crate::{check, check_ok};
use acorn_core::prelude::{format, vec, Box, String, ToString, Vec};
#[cfg(any(feature = "analysis", feature = "std"))]
use acorn_core::util::MimeType;
#[cfg(any(feature = "analysis", feature = "std"))]
use acorn_core::validation::ValidationError as PortableValidationError;
#[cfg(any(feature = "analysis", feature = "std"))]
use acorn_schema::pid::raid;
#[cfg(any(feature = "analysis", feature = "std"))]
use acorn_schema::research_activity::ResearchActivity;
#[cfg(any(feature = "analysis", feature = "std"))]
use acorn_schema::standard::crosswalk::mapping::{
    datacite_to_dcat, datacite_to_huwise, datacite_to_invenio, dcat_to_datacite, huwise_to_datacite, invenio_to_datacite,
};
#[cfg(any(feature = "analysis", feature = "std"))]
use acorn_schema::standard::crosswalk::{Conversion, ConversionWarning, Crosswalk, CrosswalkError, FieldMapping, Fields, SchemaExtractor};
#[cfg(any(feature = "analysis", feature = "std"))]
use acorn_schema::standard::{ckan, datacite, dcat, huwise, invenio};
#[cfg(any(feature = "analysis", feature = "std"))]
use acorn_schema::validation::ValidationReport;
#[cfg(any(feature = "analysis", feature = "std"))]
use acorn_schema::OneOrMany;
use alloc::borrow::Cow;
use bon::Builder;
use core::str::FromStr;
use derive_more::Display;
use serde::{Deserialize, Serialize};
#[cfg(any(feature = "analysis", feature = "std"))]
use serde_json::{Map, Value};
use strum::{EnumIs, EnumIter, VariantNames};
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

/// Utility type alias for a vector of [`ConversionWarning`]
#[cfg(any(feature = "analysis", feature = "std"))]
pub type ConversionWarnings = Vec<ConversionWarning>;
/// Trait for converting values to an ordered batch of checks.
#[cfg(any(feature = "analysis", feature = "std"))]
pub trait IntoChecks {
    /// Convert to an ordered batch of checks.
    fn to_checks(&self, uri: Option<String>) -> CheckBatch;
}
/// Detect supported metadata standards from JSON objects.
#[cfg(any(feature = "analysis", feature = "std"))]
pub trait StandardDetection {
    /// Whether this object is Citation File Format metadata.
    fn is_citation_file_format(&self) -> bool;
    /// Whether this object is a CKAN package or Action API package response.
    fn is_ckan(&self) -> bool;
    /// Whether this object is DataCite metadata.
    fn is_datacite(&self) -> bool;
    /// Whether this object is DCAT metadata.
    fn is_dcat(&self) -> bool;
    /// Whether this object is Huwise metadata.
    fn is_huwise(&self) -> bool;
    /// Whether this object is InvenioRDM metadata.
    fn is_invenio(&self) -> bool;
    /// Whether this object is a KitOps authoring Kitfile.
    fn is_kitfile(&self) -> bool;
    /// Whether this object is a resolved KitOps ModelKit configuration.
    fn is_modelkit(&self) -> bool;
    /// Whether this object is RAiD metadata.
    fn is_raid(&self) -> bool;
    /// Whether this object is Research Activity Data metadata.
    fn is_research_activity_data(&self) -> bool;
}
#[cfg(any(feature = "analysis", feature = "std"))]
pub(crate) trait ValidationReportExt {
    fn checks(self, uri: Option<String>) -> CheckBatch;
}
/// Various check categories available for validating research activity data
#[derive(Clone, Debug, Default, Deserialize, Display, EnumIs, EnumIter, PartialEq, Serialize, VariantNames)]
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
    /// Compute resource compatibility check
    #[display("compute")]
    Compute,
    /// FAIR data support assessment
    #[display("fair")]
    Fair,
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
#[derive(Clone, Copy, Debug, Default, Deserialize, Display, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Standard {
    /// Research Activity Data (RAD)
    #[default]
    #[display("research-activity-data")]
    ResearchActivityData,
    /// Application Configuration for Research Enablement (ACRE)
    #[display("application-configuration-research-enablement")]
    ApplicationConfigurationResearchEnablement,
    /// Citation File Format (CFF)
    #[display("citation-file-format")]
    CitationFileFormat,
    /// DataCite
    #[display("datacite")]
    Datacite,
    /// Comprehensive Knowledge Archive Network (CKAN)
    #[display("ckan")]
    Ckan,
    /// Data Catalog (DCAT)
    #[display("dcat")]
    Dcat,
    /// Document-derived text
    #[display("document")]
    Document,
    /// Dublin Core
    #[display("dublin-core")]
    DublinCore,
    /// Huwise
    #[display("huwise")]
    Huwise,
    /// InvenioRDM
    #[display("invenio")]
    Invenio,
    /// KitOps authoring Kitfile.
    #[display("kitfile")]
    Kitfile,
    /// Resolved KitOps ModelKit OCI configuration.
    #[display("modelkit")]
    ModelKit,
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
    /// Structured FAIR assessment payload.
    pub(super) fair_details: Option<Box<FairDetails>>,
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
/// Ordered collection of analyzer checks.
#[derive(Clone, Debug, Default)]
pub struct CheckBatch(Vec<Check>);
/// Additive issue counts for analyzer checks.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CheckSummary {
    /// Compute resource compatibility issue count.
    pub compute: usize,
    /// FAIR assessment issue count.
    pub fair: usize,
    /// Schema validation issue count.
    pub schema: usize,
    /// Link issue count.
    pub link: usize,
    /// Prose issue count.
    pub prose: usize,
    /// Quality issue count.
    pub quality: usize,
    /// Readability issue count.
    pub readability: usize,
    /// Metadata crosswalk warning count.
    pub crosswalk: usize,
}
/// Portable analyzer finding emitted by built-in and extension checks.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Finding {
    /// Analyzer category responsible for this finding.
    pub category: CheckCategory,
    /// Textual evidence or source context.
    pub context: Option<String>,
    /// Structured FAIR details when the category is [`CheckCategory::Fair`].
    pub fair: Option<Box<FairDetails>>,
    /// Optional source or metadata locator.
    pub locator: Option<String>,
    /// Human-readable result or remediation.
    pub message: String,
    /// Stable rule identifier.
    pub rule_id: String,
    /// Finding severity.
    pub severity: CheckSeverity,
    /// Whether supplied evidence satisfies the check.
    pub success: bool,
    /// Optional resource URI.
    pub uri: Option<String>,
}
/// Ordered collection of analyzer findings.
#[cfg_attr(feature = "exec", derive(validator::Validate))]
#[cfg_attr(feature = "exec", validate(schema(function = "Findings::is_valid", skip_on_field_errors = false)))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Findings {
    #[cfg_attr(feature = "exec", validate(nested))]
    values: Vec<Finding>,
}
impl Check {
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
            | CheckCategory::Fair => usize::from(!self.success),
            | CheckCategory::Compute | CheckCategory::Link | CheckCategory::Quality | CheckCategory::Readability => 1,
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
    /// Return the structured FAIR finding associated with this check.
    pub fn fair_details(&self) -> Option<&FairDetails> {
        self.fair_details.as_deref()
    }
    /// Replace the structured FAIR finding associated with this check.
    #[cfg(feature = "analysis")]
    pub(crate) fn with_fair_details(self, finding: FairDetails) -> Self {
        let context = finding.evidence.clone();
        let locator = finding.locator.clone().or_else(|| Some(finding.indicator.id.to_string()));
        let message = finding.remediation.clone();
        let severity = CheckSeverity::from(finding.clone());
        let success = FairState::is_supported(&finding.state) || FairState::is_not_applicable(&finding.state);
        Self {
            context,
            fair_details: Some(Box::new(finding)),
            locator,
            message,
            severity,
            success,
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
impl From<Finding> for Check {
    fn from(value: Finding) -> Self {
        Self {
            index: 0,
            category: value.category,
            locator: value.locator,
            prose_locators: Vec::new(),
            context: value.context,
            data: None,
            fair_details: value.fair,
            kind: CheckKind::Standard,
            success: value.success,
            severity: value.severity,
            status_code: None,
            errors: None,
            uri: value.uri,
            message: value.message,
        }
    }
}
impl CheckBatch {
    /// Create an empty batch.
    pub fn empty() -> Self {
        Self(Vec::new())
    }
    /// Create a batch containing one check.
    pub fn single(check: Check) -> Self {
        Self(vec![check])
    }
    /// Convert schema parse diagnostics into checks.
    #[cfg(any(feature = "analysis", feature = "std"))]
    pub(crate) fn schema_parse_failure_checks(diagnostic: impl ToString, uri: Option<String>) -> Self {
        diagnostic
            .to_string()
            .lines()
            .map(|line| check_err!(CheckCategory::Schema, context: line.to_string()).with_uri(uri.clone()))
            .collect()
    }
    /// Combine two batches while preserving their left-to-right order.
    pub fn combine(self, other: Self) -> Self {
        Self(self.0.into_iter().chain(other.0).collect())
    }
    /// Resolve command completion from issue state and failure policy.
    pub fn finish_status(&self, no_fail: bool) -> Result<(), usize> {
        match no_fail || !self.iter().any(Check::is_failure) {
            | true => Ok(()),
            | false => Err(self.len()),
        }
    }
    /// Borrow the checks in this batch as a slice.
    pub fn as_slice(&self) -> &[Check] {
        &self.0
    }
    /// Iterate over borrowed checks in insertion order.
    pub fn iter(&self) -> core::slice::Iter<'_, Check> {
        self.0.iter()
    }
    /// Return the number of checks in this batch.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Return whether this batch contains no checks.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Convert this batch into its ordered vector representation.
    pub fn into_vec(self) -> Vec<Check> {
        self.0
    }
}
impl core::iter::FromIterator<Check> for CheckBatch {
    fn from_iter<T: IntoIterator<Item = Check>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
impl core::iter::FromIterator<CheckBatch> for CheckBatch {
    fn from_iter<T: IntoIterator<Item = CheckBatch>>(iter: T) -> Self {
        Self(iter.into_iter().flat_map(CheckBatch::into_iter).collect())
    }
}
impl From<Vec<Check>> for CheckBatch {
    fn from(checks: Vec<Check>) -> Self {
        Self(checks)
    }
}
impl<'a> IntoIterator for &'a CheckBatch {
    type Item = &'a Check;
    type IntoIter = core::slice::Iter<'a, Check>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl IntoIterator for CheckBatch {
    type Item = Check;
    type IntoIter = alloc::vec::IntoIter<Check>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl CheckCategory {
    /// Convert a category-specific diagnostic into one failed check.
    #[cfg(any(feature = "analysis", feature = "std"))]
    pub(crate) fn failure_checks(self, diagnostic: impl ToString, uri: Option<String>) -> CheckBatch {
        CheckBatch::single(check_err!(self, context: diagnostic.to_string()).with_uri(uri))
    }
    /// Returns `true` if this category appears in the given list of category items
    pub fn is_in(&self, items: &[String]) -> bool {
        items.iter().any(|value| Self::from(value.as_str()) == *self)
    }
    /// Return the user-facing category label.
    pub const fn label(&self) -> &'static str {
        match self {
            | Self::Compute => "Compute",
            | Self::Crosswalk => "Crosswalk",
            | Self::Fair => "Fair",
            | Self::Link => "Link",
            | Self::Prose => "Prose",
            | Self::Quality => "Quality",
            | Self::Readability => "Readability",
            | Self::Schema => "Schema",
        }
    }
    #[cfg_attr(not(feature = "analysis"), allow(dead_code))]
    pub(super) fn message(&self) -> &'static str {
        match self {
            | Self::Compute => "Compute",
            | Self::Link => "URL",
            | Self::Prose => "Prose",
            | Self::Quality => "Quality",
            | Self::Fair => "FAIR support",
            | Self::Readability => "Readability",
            | Self::Schema => "Deserialization error",
            | Self::Crosswalk => "Crosswalk warning",
        }
    }
}
impl From<&str> for CheckCategory {
    fn from(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            | "compute" => Self::Compute,
            | "link" => Self::Link,
            | "prose" => Self::Prose,
            | "quality" => Self::Quality,
            | "fair" => Self::Fair,
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
impl FromStr for CheckSeverity {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_lowercase().as_str() {
            | "error" => Ok(Self::Error),
            | "info" => Ok(Self::Info),
            | "suggestion" => Ok(Self::Suggestion),
            | "warning" => Ok(Self::Warning),
            | other => Err(format!("unknown severity `{other}`")),
        }
    }
}
impl CheckSummary {
    /// Create the additive identity containing no issues.
    pub const fn zero() -> Self {
        Self {
            compute: 0,
            fair: 0,
            schema: 0,
            link: 0,
            prose: 0,
            quality: 0,
            readability: 0,
            crosswalk: 0,
        }
    }
    /// Combine two summaries by adding corresponding category counts.
    pub fn combine(self, other: Self) -> Self {
        Self {
            compute: self.compute.saturating_add(other.compute),
            fair: self.fair.saturating_add(other.fair),
            schema: self.schema.saturating_add(other.schema),
            link: self.link.saturating_add(other.link),
            prose: self.prose.saturating_add(other.prose),
            quality: self.quality.saturating_add(other.quality),
            readability: self.readability.saturating_add(other.readability),
            crosswalk: self.crosswalk.saturating_add(other.crosswalk),
        }
    }
    /// Return category counts in stable display order.
    pub fn counts(&self) -> [(CheckCategory, usize); 8] {
        [
            (CheckCategory::Schema, self.schema),
            (CheckCategory::Link, self.link),
            (CheckCategory::Prose, self.prose),
            (CheckCategory::Quality, self.quality),
            (CheckCategory::Compute, self.compute),
            (CheckCategory::Fair, self.fair),
            (CheckCategory::Readability, self.readability),
            (CheckCategory::Crosswalk, self.crosswalk),
        ]
    }
    /// Convert category counts to presentation rows.
    pub fn rows(&self) -> Vec<Vec<String>> {
        self.counts()
            .into_iter()
            .map(|(category, count)| vec![format!("{} items found", category.label()), count.to_string()])
            .collect()
    }
}
impl From<&[Check]> for CheckSummary {
    fn from(checks: &[Check]) -> Self {
        checks.iter().fold(Self::zero(), |summary, check| summary.combine(check.into()))
    }
}
impl From<&Check> for CheckSummary {
    fn from(check: &Check) -> Self {
        let count = check.issue_count();
        match check.category {
            | CheckCategory::Compute => Self {
                compute: count,
                ..Self::zero()
            },
            | CheckCategory::Fair => Self { fair: count, ..Self::zero() },
            | CheckCategory::Schema => Self {
                schema: count,
                ..Self::zero()
            },
            | CheckCategory::Link => Self { link: count, ..Self::zero() },
            | CheckCategory::Prose => Self {
                prose: count,
                ..Self::zero()
            },
            | CheckCategory::Quality => Self {
                quality: count,
                ..Self::zero()
            },
            | CheckCategory::Readability => Self {
                readability: count,
                ..Self::zero()
            },
            | CheckCategory::Crosswalk => Self {
                crosswalk: count,
                ..Self::zero()
            },
        }
    }
}
impl From<&CheckBatch> for CheckSummary {
    fn from(checks: &CheckBatch) -> Self {
        Self::from(checks.as_slice())
    }
}
impl From<CheckBatch> for CheckSummary {
    fn from(checks: CheckBatch) -> Self {
        Self::from(&checks)
    }
}
#[cfg(any(feature = "analysis", feature = "std"))]
impl IntoChecks for ConversionWarnings {
    /// Convert crosswalk conversion warnings into Check objects with Warning severity.
    ///
    /// Each ConversionWarning from a field-level loss during schema crosswalk produces
    /// a single Check with the field name as locator and a human-readable message.
    fn to_checks(&self, uri: Option<String>) -> CheckBatch {
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
impl Validate for Finding {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let rule_id_error = match (1..=128).contains(&self.rule_id.chars().count()) {
            | false => Some(ValidationError::new("length")),
            | true => (!FairIndicatorId::validate(&self.rule_id)).then(|| ValidationError::new("rule_id")),
        };
        let data = [
            self.locator
                .as_ref()
                .is_some_and(|value| !(1..=1024).contains(&value.chars().count()))
                .then(|| ("locator", ValidationError::new("length"))),
            (!(1..=4096).contains(&self.message.chars().count())).then(|| ("message", ValidationError::new("length"))),
            rule_id_error.map(|error| ("rule_id", error)),
        ];
        let errors = data
            .into_iter()
            .flatten()
            .map(|(field, error)| (Cow::Borrowed(field), ValidationErrorsKind::Field(vec![error])))
            .collect();
        let errors = ValidationErrors(errors);
        match errors.is_empty() {
            | true => Ok(()),
            | false => Err(errors),
        }
    }
}
impl Findings {
    /// Return whether the collection contains no findings.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    /// Iterate over findings in stable output order.
    pub fn iter(&self) -> core::slice::Iter<'_, Finding> {
        self.values.iter()
    }
    /// Return the number of findings.
    pub fn len(&self) -> usize {
        self.values.len()
    }
}
impl From<Vec<Finding>> for Findings {
    fn from(value: Vec<Finding>) -> Self {
        Self { values: value }
    }
}
impl IntoIterator for Findings {
    type Item = Finding;
    type IntoIter = alloc::vec::IntoIter<Finding>;
    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}
#[cfg(any(feature = "analysis", feature = "std"))]
impl StandardDetection for Map<String, Value> {
    fn is_citation_file_format(&self) -> bool {
        self.contains_key("cff-version")
    }
    fn is_ckan(&self) -> bool {
        let package = self.contains_key("id")
            && self.contains_key("name")
            && self.contains_key("title")
            && ["metadata_created", "metadata_modified", "resources", "tags"]
                .into_iter()
                .any(|field| self.contains_key(field));
        package
            || self.get("success").and_then(Value::as_bool).is_some()
                && self.get("result").and_then(Value::as_object).is_some_and(StandardDetection::is_ckan)
    }
    fn is_datacite(&self) -> bool {
        self.contains_key("attributes") && self.contains_key("id")
    }
    fn is_dcat(&self) -> bool {
        self.get("@type").and_then(Value::as_str).is_some_and(|value| value.contains("dcat:"))
    }
    fn is_huwise(&self) -> bool {
        self.contains_key("dataset_id") && self.contains_key("metas")
    }
    fn is_invenio(&self) -> bool {
        self.contains_key("metadata") || self.contains_key("pids") || self.contains_key("resource_type")
    }
    fn is_kitfile(&self) -> bool {
        self.contains_key("manifestVersion") && !self.is_modelkit()
    }
    fn is_modelkit(&self) -> bool {
        self.contains_key("manifestVersion") && self.values().any(contains_layer_identity)
    }
    fn is_raid(&self) -> bool {
        self.contains_key("access") && self.contains_key("contributor") && self.contains_key("date") && self.contains_key("title")
    }
    fn is_research_activity_data(&self) -> bool {
        self.contains_key("contact") && self.contains_key("meta") && self.contains_key("title")
    }
}
#[cfg(any(feature = "analysis", feature = "std"))]
impl From<&Map<String, Value>> for Standard {
    fn from(object: &Map<String, Value>) -> Self {
        Self::detect_object(object).unwrap_or(Self::Dcat)
    }
}
#[cfg(any(feature = "analysis", feature = "std"))]
impl Standard {
    /// Strictly detect a supported metadata standard from an object.
    pub fn detect_object(object: &Map<String, Value>) -> Option<Self> {
        if object.is_citation_file_format() {
            Some(Self::CitationFileFormat)
        } else if object.contains_key("success") && object.is_ckan() {
            Some(Self::Ckan)
        } else if object.is_dcat() {
            Some(Self::Dcat)
        } else if object.is_huwise() {
            Some(Self::Huwise)
        } else if object.is_datacite() {
            Some(Self::Datacite)
        } else if object.is_modelkit() {
            Some(Self::ModelKit)
        } else if object.is_kitfile() {
            Some(Self::Kitfile)
        } else if object.is_research_activity_data() {
            Some(Self::ResearchActivityData)
        } else if object.is_raid() {
            Some(Self::Raid)
        } else if object.is_invenio() {
            Some(Self::Invenio)
        } else if object.is_ckan() {
            Some(Self::Ckan)
        } else {
            None
        }
    }
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
        let source_mime = mime.clone();
        let label = |index: usize| match source_mime {
            | MimeType::JsonLines => format!("record at index {index} (line {})", index.saturating_add(1)),
            | _ => format!("record at index {index}"),
        };
        match *self {
            | Standard::Ckan => ckan::Package::parse(content, mime).and_then(|source| match target {
                | Standard::Dcat => crosswalk_batch::<ckan::Package, dcat::Dataset>(source, &source_mime, target_mime).map(Conversion::into_value),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | Standard::Datacite => OneOrMany::<datacite::Record>::parse(content, mime).and_then(|source| match target {
                | Standard::Datacite => source.serialize(target_mime),
                | Standard::Dcat => source
                    .try_map_with_label(dcat::Dataset::try_from, label)
                    .and_then(|batch| batch.serialize(target_mime)),
                | Standard::Invenio => source
                    .try_map_with_label(invenio::Record::try_from, label)
                    .and_then(|batch| batch.serialize(target_mime)),
                | Standard::Huwise => source
                    .try_map_with_label(huwise::Dataset::try_from, label)
                    .and_then(|batch| batch.serialize(target_mime)),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | Standard::Dcat => OneOrMany::<dcat::Dataset>::parse(content, mime).and_then(|source| match target {
                | Standard::Dcat => source.serialize(target_mime),
                | Standard::Datacite => source
                    .try_map_with_label(datacite::Record::try_from, label)
                    .and_then(|batch| batch.serialize(target_mime)),
                | Standard::Invenio => source
                    .try_map_with_label(|value| datacite::Record::try_from(value).and_then(invenio::Record::try_from), label)
                    .and_then(|batch| batch.serialize(target_mime)),
                | Standard::Huwise => source
                    .try_map_with_label(|value| datacite::Record::try_from(value).and_then(huwise::Dataset::try_from), label)
                    .and_then(|batch| batch.serialize(target_mime)),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | Standard::Invenio => OneOrMany::<invenio::Record>::parse(content, mime).and_then(|source| match target {
                | Standard::Invenio => source.serialize(target_mime),
                | Standard::Datacite => source
                    .try_map_with_label(datacite::Record::try_from, label)
                    .and_then(|batch| batch.serialize(target_mime)),
                | Standard::Dcat => source
                    .try_map_with_label(|value| datacite::Record::try_from(value).and_then(dcat::Dataset::try_from), label)
                    .and_then(|batch| batch.serialize(target_mime)),
                | Standard::Huwise => source
                    .try_map_with_label(|value| datacite::Record::try_from(value).and_then(huwise::Dataset::try_from), label)
                    .and_then(|batch| batch.serialize(target_mime)),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | Standard::Huwise => OneOrMany::<huwise::Dataset>::parse(content, mime).and_then(|source| match target {
                | Standard::Huwise => source.serialize(target_mime),
                | Standard::Datacite => source
                    .try_map_with_label(datacite::Record::try_from, label)
                    .and_then(|batch| batch.serialize(target_mime)),
                | Standard::Dcat => source
                    .try_map_with_label(|value| datacite::Record::try_from(value).and_then(dcat::Dataset::try_from), label)
                    .and_then(|batch| batch.serialize(target_mime)),
                | Standard::Invenio => source
                    .try_map_with_label(|value| datacite::Record::try_from(value).and_then(invenio::Record::try_from), label)
                    .and_then(|batch| batch.serialize(target_mime)),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | Standard::Raid => OneOrMany::<raid::Metadata>::parse(content, mime).and_then(|source| match target {
                | Standard::Raid => source.serialize(target_mime),
                | Standard::ResearchActivityData => {
                    crosswalk_batch::<raid::Metadata, ResearchActivity>(source, &source_mime, target_mime).map(Conversion::into_value)
                }
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | Standard::ResearchActivityData => OneOrMany::<ResearchActivity>::parse(content, mime).and_then(|source| match target {
                | Standard::Datacite => {
                    crosswalk_batch::<ResearchActivity, datacite::Record>(source, &source_mime, target_mime).map(Conversion::into_value)
                }
                | Standard::Raid => {
                    crosswalk_batch::<ResearchActivity, raid::Metadata>(source, &source_mime, target_mime).map(Conversion::into_value)
                }
                | Standard::ResearchActivityData => source.serialize(target_mime),
                | _ => Err(self.unsupported_crosswalk(target)),
            }),
            | Standard::ApplicationConfigurationResearchEnablement
            | Standard::Kitfile
            | Standard::ModelKit
            | Standard::CitationFileFormat
            | Standard::Document
            | Standard::DublinCore
            | Standard::Text => Err(self.unsupported_crosswalk(target)),
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
    ) -> Result<Conversion<String>, CrosswalkError> {
        match (*self, target) {
            | (Standard::Ckan, Standard::Dcat) => ckan::Package::parse(content, mime.clone())
                .and_then(|source| crosswalk_batch::<ckan::Package, dcat::Dataset>(source, &mime, target_mime)),
            | (Standard::Raid, Standard::ResearchActivityData) => OneOrMany::<raid::Metadata>::parse(content, mime.clone())
                .and_then(|source| crosswalk_batch::<raid::Metadata, ResearchActivity>(source, &mime, target_mime)),
            | (Standard::ResearchActivityData, Standard::Datacite) => OneOrMany::<ResearchActivity>::parse(content, mime.clone())
                .and_then(|source| crosswalk_batch::<ResearchActivity, datacite::Record>(source, &mime, target_mime)),
            | (Standard::ResearchActivityData, Standard::Raid) => OneOrMany::<ResearchActivity>::parse(content, mime.clone())
                .and_then(|source| crosswalk_batch::<ResearchActivity, raid::Metadata>(source, &mime, target_mime)),
            | _ => self.collect_crosswalk_warnings(content, &mime, target).and_then(|warnings| {
                self.crosswalk(content, mime, target, target_mime)
                    .map(|content| Conversion::new(content, warnings))
            }),
        }
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
            "Unsupported metadata crosswalk pair: {self} -> {target} (supported: ckan, rads, raid, datacite, dcat, invenio, huwise)"
        ))
    }
}
impl From<&CheckCategory> for u8 {
    fn from(value: &CheckCategory) -> Self {
        match value {
            | CheckCategory::Schema | CheckCategory::Link => 0,
            | CheckCategory::Prose => 1,
            | CheckCategory::Quality => 2,
            | CheckCategory::Compute => 3,
            | CheckCategory::Fair => 4,
            | CheckCategory::Readability => 5,
            | CheckCategory::Crosswalk => 6,
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
#[cfg(any(feature = "analysis", feature = "std"))]
impl ValidationReportExt for ValidationReport {
    fn checks(self, uri: Option<String>) -> CheckBatch {
        ValidatorIssues::from(self)
            .into_iter()
            .map(|issue| {
                let errors = ValidationReport::from_error(
                    "",
                    PortableValidationError {
                        code: issue.code.to_string(),
                        message: Some(issue.message.clone()),
                        params: issue.params.clone().into_iter().collect(),
                    },
                );
                let prefix = issue.path.clone().unwrap_or_else(|| issue.locator());
                check_err!(
                    CheckCategory::Schema,
                    locator: issue.locator(),
                    context: prefix,
                    message: issue.message,
                    errors: ErrorKind::Validator(errors)
                )
                .with_uri(uri.clone())
            })
            .collect()
    }
}
impl From<CheckBatch> for Vec<Check> {
    fn from(checks: CheckBatch) -> Self {
        checks.into_vec()
    }
}
impl From<Findings> for Vec<Finding> {
    fn from(value: Findings) -> Self {
        value.values
    }
}
/// Portable readability check over supplied text (no filesystem or env)
#[cfg(feature = "alloc")]
pub fn check_readability_for_text(text: &str, metric: ReadabilityType, uri: Option<String>) -> CheckBatch {
    let maximum = metric.maximum_allowed();
    let calculated = metric.calculate(text);
    let assessment = ReadabilityScore::new(metric, calculated, maximum);
    let message = assessment.to_string();
    CheckBatch::from(if assessment.is_excessive() {
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
    })
}
#[cfg(any(feature = "analysis", feature = "std"))]
fn contains_layer_identity(value: &Value) -> bool {
    match value {
        | Value::Object(object) => object.contains_key("digest") || object.contains_key("diffId") || object.values().any(contains_layer_identity),
        | Value::Array(values) => values.iter().any(contains_layer_identity),
        | _ => false,
    }
}
#[cfg(any(feature = "analysis", feature = "std"))]
fn crosswalk_batch<S, T>(source: OneOrMany<S>, source_mime: &MimeType, target_mime: MimeType) -> Result<Conversion<String>, CrosswalkError>
where
    S: Crosswalk<T>,
    T: Serialize,
{
    match source {
        | OneOrMany::One(source) => source.crosswalk().and_then(|(target, warnings)| {
            OneOrMany::One(target)
                .serialize(target_mime)
                .map(|content| Conversion::new(content, warnings))
        }),
        | OneOrMany::Many(sources) => sources
            .into_iter()
            .enumerate()
            .map(|(index, source)| {
                let label = match source_mime {
                    | MimeType::JsonLines => format!("record at index {index} (line {})", index.saturating_add(1)),
                    | _ => format!("record at index {index}"),
                };
                source
                    .crosswalk()
                    .map_err(|error| CrosswalkError::BuildFailed(format!("Failed to convert {label} — {error}")))
            })
            .collect::<Result<Vec<_>, _>>()
            .and_then(|converted| {
                let (targets, warnings): (Vec<_>, Vec<_>) = converted.into_iter().unzip();
                OneOrMany::Many(targets)
                    .serialize(target_mime)
                    .map(|content| Conversion::new(content, warnings.into_iter().flatten().collect()))
            }),
    }
}
/// Parse source content and apply a FieldMapping to detect missing optional fields.
#[cfg(any(feature = "analysis", feature = "std"))]
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
                    let prefix = match mime {
                        | MimeType::JsonLines => format!("record[{i}].line[{}].", i.saturating_add(1)),
                        | _ => format!("record[{i}]."),
                    };
                    missing_warnings(&fields, &prefix).into_iter().chain(unmapped_warnings(&fields, &prefix))
                })
                .collect(),
        }
    })
}
