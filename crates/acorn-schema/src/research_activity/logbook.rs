//! Time-coded developments for a research activity.
use super::{ActivityContributor, ResearchActivity, ResearchActivityMetadata};
use crate::enrichment::Provider;
use crate::validation::{rules, Validate, ValidationReport};
use crate::{ClassificationLevel, OneOrMany};
use acorn_core::prelude::alloc::{format, String, ToString, Vec};
use acorn_core::util::{frontmatter_and_body, MarkdownDecode, MarkdownSupport};
use acorn_core::validation::ValidationError;
use bon::Builder;
use core::cmp::Reverse;
use core::fmt;
use jiff::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_trim::{option_string_trim, string_trim};
use serde_with::skip_serializing_none;
use sophia::api::ns::Namespace;

/// Supported canonical destination for a logbook graduation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GraduationTarget {
    /// `meta.classification`.
    Classification,
    /// `meta.contributors`.
    Contributors,
}
/// Subject category for a logbook entry.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogbookCategory {
    /// Project administration and governance developments.
    Administration,
    /// Partnership, working-group, or other collaborative developments and events.
    Collaboration,
    /// Data collection, curation, publication, or stewardship developments.
    Data,
    /// Public, sponsor, stakeholder, or scholarly engagement developments.
    #[serde(alias = "conference", alias = "tour")]
    Engagement,
    /// Funding application, award, or management developments.
    Funding,
    /// Facility, equipment, platform, or other infrastructure developments.
    #[serde(alias = "technical")]
    Infrastructure,
    /// Research method, protocol, or workflow developments.
    Method,
    /// Developments not covered by another category.
    #[serde(alias = "general")]
    Other,
    /// Personnel developments.
    Personnel,
    /// Publication developments.
    Publication,
    /// Experiment, analysis, finding, or other research developments.
    Research,
    /// Repository, notebook, workflow implementation, or software release developments.
    #[serde(alias = "code")]
    Software,
}
/// Origin of a logbook entry.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogbookEntryOrigin {
    /// Authored directly by a person.
    #[default]
    Manual,
    /// Proposed by an enrichment provider and accepted by a person.
    Discovered,
}
/// Failure to select or graduate logbook entries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LogbookError {
    /// A timestamp was not a complete RFC 3339 instant with an offset.
    InvalidTimestamp(String),
    /// A reporting window did not have an exclusive upper bound after its lower bound.
    InvalidWindow(String),
    /// An accepted entry identifier did not identify a graduation candidate.
    UnknownGraduationIdentifier(String),
}
/// Format of an event described by a logbook entry.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogbookEventType {
    /// Conference participation or organization.
    Conference,
    /// Exhibition or demonstration.
    Exhibition,
    /// Meeting or convening.
    Meeting,
    /// Event format not covered by another type.
    Other,
    /// Talk, poster, or other presentation.
    Presentation,
    /// Facility, laboratory, or site tour.
    Tour,
    /// Training or instructional event.
    Training,
    /// Workshop or working session.
    Workshop,
}
/// Temporal interpretation of a logbook timestamp.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogbookState {
    /// Work completed at the recorded instant.
    Completed,
    /// Work known to be underway at the recorded instant.
    InProgress,
    /// Work expected at the recorded instant.
    Upcoming,
}
/// A deterministic, reviewable proposal to graduate one logbook entry.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GraduationCandidate {
    /// Identifier of the source logbook entry.
    pub entry_identifier: String,
    /// Canonical destination field.
    pub target: GraduationTarget,
    /// Value proposed for the canonical destination.
    pub value: String,
}
/// Deterministic, reviewable proposals to graduate logbook entries.
#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct GraduationCandidates(pub Vec<GraduationCandidate>);
/// A logbook collection with an optional editorial overview.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "Logbook::is_valid", skip_on_field_errors = false))]
pub struct Logbook {
    /// Editorial overview of the logbook.
    #[validate(nonempty, length(max = 1000))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub summary: Option<String>,
    /// Ordered time-coded developments. Serialization preserves source order.
    #[validate(nested, length(min = 1, max = 1000))]
    pub entries: Vec<LogbookEntry>,
}
/// Discovery provenance for a logbook entry.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LogbookDiscovery {
    /// Provider that discovered the candidate.
    pub provider: Provider,
    /// Provider-native identifier or lookup value.
    #[validate(nonempty, length(max = 2048))]
    #[serde(deserialize_with = "string_trim")]
    pub source: String,
    /// Instant when the candidate was discovered.
    #[validate(timestamp)]
    #[serde(deserialize_with = "string_trim")]
    pub discovered_at: String,
}
/// One time-coded research development.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(LogbookCategory, into), on(LogbookEventType, into), on(LogbookState, into), on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "LogbookEntry::is_valid", skip_on_field_errors = false))]
pub struct LogbookEntry {
    /// Stable local identifier, unique within the logbook.
    #[validate(nonempty, kebabcase, length(max = 100))]
    #[serde(deserialize_with = "string_trim")]
    pub identifier: String,
    /// Stable project identifier used to route the entry to project metadata.
    #[validate(nonempty, kebabcase, length(max = 100))]
    #[serde(deserialize_with = "string_trim")]
    pub project_id: String,
    /// Temporal anchor interpreted according to [`LogbookState`].
    #[validate(timestamp)]
    #[serde(deserialize_with = "string_trim")]
    pub timestamp: String,
    /// Subject of the development.
    pub category: LogbookCategory,
    /// National security classification associated with this entry.
    pub classification: Option<ClassificationLevel>,
    /// Format when the development concerns an event.
    pub event_type: Option<LogbookEventType>,
    /// Temporal state of the development.
    pub state: LogbookState,
    /// How the entry entered the logbook.
    #[builder(default)]
    #[serde(default)]
    pub origin: LogbookEntryOrigin,
    /// Provenance required for discovered entries and omitted for manual entries.
    #[validate(nested)]
    pub discovery: Option<LogbookDiscovery>,
    /// Concise description of the development.
    #[validate(nonempty, length(max = 500))]
    #[serde(deserialize_with = "string_trim")]
    pub summary: String,
    /// Supporting statements.
    #[validate(length(min = 1, max = 20), each(nonempty), each(length(max = 1000)))]
    pub items: Option<Vec<String>>,
    /// Whether this entry has been graduated into canonical activity metadata.
    #[builder(default)]
    #[serde(default)]
    pub archived: bool,
}
/// A since-filtered, display-ordered logbook update set.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LogbookUpdates {
    /// Stable activity identifier.
    pub identifier: String,
    /// Activity title.
    pub title: String,
    /// Optional activity subtitle.
    pub subtitle: Option<String>,
    /// Inclusive RFC 3339 lower bound used for selection.
    pub since: String,
    /// Exclusive RFC 3339 upper bound used for selection, when bounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub through: Option<String>,
    /// Entries at or after `since`, ordered newest first.
    pub entries: Vec<LogbookEntry>,
}
/// Immutable half-open reporting interval used for automated artifacts.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[validate(schema(function = "ReportingWindow::is_valid", skip_on_field_errors = false))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReportingWindow {
    /// Inclusive RFC 3339 lower bound, or `None` for full history.
    #[validate(timestamp)]
    pub left: Option<String>,
    /// Exclusive RFC 3339 upper bound.
    #[validate(timestamp)]
    pub right: String,
}
impl From<&ResearchActivity> for GraduationCandidates {
    fn from(activity: &ResearchActivity) -> Self {
        Self(
            activity
                .logbook
                .as_ref()
                .into_iter()
                .flat_map(|logbook| &logbook.entries)
                .filter(|entry| !entry.archived)
                .flat_map(|entry| {
                    let classification = entry
                        .classification
                        .as_ref()
                        .filter(|_| entry.category == LogbookCategory::Administration && entry.event_type == Some(LogbookEventType::Other))
                        .map(|classification| GraduationCandidate {
                            entry_identifier: entry.identifier.clone(),
                            target: GraduationTarget::Classification,
                            value: classification.to_string(),
                        });
                    let contributor = (entry.category == LogbookCategory::Personnel).then(|| GraduationCandidate {
                        entry_identifier: entry.identifier.clone(),
                        target: GraduationTarget::Contributors,
                        value: entry.summary.clone(),
                    });
                    [classification, contributor].into_iter().flatten()
                })
                .collect(),
        )
    }
}
impl From<ResearchActivity> for GraduationCandidates {
    fn from(activity: ResearchActivity) -> Self {
        Self::from(&activity)
    }
}
impl GraduationCandidates {
    /// Render the deterministic merge-request note for the current candidates.
    pub fn render_note(&self) -> String {
        let candidates = self
            .0
            .iter()
            .map(|candidate| format!("- `{}` → `{:?}`: {}", candidate.entry_identifier, candidate.target, candidate.value))
            .collect::<Vec<_>>()
            .join("\n");
        let candidates = match candidates.is_empty() {
            | true => "_No graduation candidates._".to_string(),
            | false => candidates,
        };
        format!(
            "<!-- acorn:logbook-intake -->\n## ACORN logbook intake\n\n### Graduation candidates\n{candidates}\n\nUse `/acorn graduate <entry-id>...` to accept selected candidates."
        )
    }
}
impl Logbook {
    /// Select entries in a validated half-open reporting window, newest first.
    pub fn entries_within(&self, window: &ReportingWindow) -> Result<Vec<LogbookEntry>, LogbookError> {
        window.left_instant().and_then(|left| {
            window.right_instant().and_then(|right| {
                self.entries_all().map(|entries| {
                    entries
                        .into_iter()
                        .filter(|entry| {
                            rules::parse_timestamp(&entry.timestamp).is_ok_and(|instant| left.is_none_or(|left| instant >= left) && instant < right)
                        })
                        .collect()
                })
            })
        })
    }
    /// Select entries at or after an inclusive RFC 3339 instant, newest first.
    pub fn entries(&self, since: &str) -> Result<Vec<LogbookEntry>, LogbookError> {
        rules::parse_timestamp(since)
            .map_err(|_| LogbookError::InvalidTimestamp(since.to_string()))
            .and_then(|since_instant| {
                self.entries_all().map(|entries| {
                    entries
                        .into_iter()
                        .filter(|entry| rules::parse_timestamp(&entry.timestamp).is_ok_and(|instant| instant >= since_instant))
                        .collect()
                })
            })
    }
    /// Return all entries ordered newest first.
    pub fn entries_all(&self) -> Result<Vec<LogbookEntry>, LogbookError> {
        self.entries
            .iter()
            .map(|entry| {
                rules::parse_timestamp(&entry.timestamp)
                    .map(|instant| (instant, entry))
                    .map_err(|_| LogbookError::InvalidTimestamp(entry.timestamp.clone()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|mut entries| {
                entries.sort_by_key(|(instant, _)| Reverse(*instant));
                entries.into_iter().map(|(_, entry)| entry.clone()).collect()
            })
    }
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        self.entries
            .iter()
            .enumerate()
            .find_map(|(index, entry)| {
                self.entries
                    .get(..index)
                    .into_iter()
                    .flatten()
                    .any(|previous| previous.identifier == entry.identifier)
                    .then_some((index, &entry.identifier))
            })
            .map(|(index, identifier)| {
                let message = "Logbook entry identifiers must be unique";
                let mut error = ValidationError::new("logbook_identifier").with_message(message);
                error.add_param("identifier".into(), identifier);
                ValidationReport::from_error(format!("entries[{index}].identifier"), error)
            })
            .map_or(Ok(()), Err)
    }
}
impl MarkdownDecode for Logbook {
    fn from_markdown(content: &str) -> Result<Self, String> {
        frontmatter_and_body(content)
            .0
            .ok_or_else(|| "Logbook Markdown requires YAML frontmatter delimited by '---'".to_string())
            .and_then(|frontmatter| Self::from_front_matter(&frontmatter))
    }
}
impl MarkdownSupport for Logbook {
    fn to_markdown(&self) -> String {
        let summary = self.summary.as_ref().map(|value| format!("\n{value}\n")).unwrap_or_default();
        let entries = self.entries.iter().map(MarkdownSupport::to_markdown).collect::<Vec<_>>().join("\n\n");
        format!("# Logbook\n{summary}\n{entries}\n")
    }
}
impl From<&str> for LogbookCategory {
    fn from(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            | "administration" => Self::Administration,
            | "collaboration" => Self::Collaboration,
            | "data" => Self::Data,
            | "conference" | "engagement" | "tour" => Self::Engagement,
            | "funding" => Self::Funding,
            | "infrastructure" | "technical" => Self::Infrastructure,
            | "method" => Self::Method,
            | "personnel" => Self::Personnel,
            | "publication" => Self::Publication,
            | "research" => Self::Research,
            | "code" | "software" => Self::Software,
            | _ => Self::Other,
        }
    }
}
impl From<String> for LogbookCategory {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}
impl LogbookCategory {
    /// Return the published ontology namespace represented by this category.
    pub fn namespace(self) -> Namespace<&'static str> {
        Namespace::new_unchecked(match self {
            | Self::Administration => "https://schema.org/OrganizeAction",
            | Self::Collaboration => "http://www.w3.org/ns/prov#Association",
            | Self::Data => "http://www.w3.org/ns/dcat#Dataset",
            | Self::Engagement => "https://schema.org/Event",
            | Self::Funding => "https://schema.org/Grant",
            | Self::Infrastructure => "http://www.w3.org/ns/prov#Entity",
            | Self::Method => "http://www.w3.org/ns/prov#Plan",
            | Self::Other => "http://www.w3.org/2004/02/skos/core#Concept",
            | Self::Personnel => "http://www.w3.org/ns/prov#Person",
            | Self::Publication => "https://schema.org/CreativeWork",
            | Self::Research => "https://schema.org/ResearchProject",
            | Self::Software => "https://schema.org/SoftwareSourceCode",
        })
    }
}
impl LogbookDiscovery {
    /// Build a deterministic kebab-case entry identifier from provider and source.
    pub fn identifier(&self) -> String {
        self.provider.discovered_identifier(&self.source)
    }
}
impl LogbookEntry {
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        if !matches!(
            (self.origin, self.discovery.as_ref()),
            (LogbookEntryOrigin::Manual, None) | (LogbookEntryOrigin::Discovered, Some(_))
        ) {
            report.add(
                "discovery",
                ValidationError::new("logbook_origin").with_message("logbook_origin: manual entries omit discovery; discovered entries require it"),
            );
        }
        if self.classification.is_some() && self.category != LogbookCategory::Administration {
            report.add(
                "category",
                ValidationError::new("logbook_classification_category")
                    .with_message("Classification logbook entries must use the administration category"),
            );
        }
        if self.classification.is_some() && self.event_type != Some(LogbookEventType::Other) {
            report.add(
                "eventType",
                ValidationError::new("logbook_classification_event").with_message("Classification logbook entries must use the other event type"),
            );
        }
        report.finish()
    }
}
impl MarkdownDecode for LogbookEntry {
    fn from_markdown(content: &str) -> Result<Self, String> {
        frontmatter_and_body(content)
            .0
            .ok_or_else(|| "Logbook entry Markdown requires YAML frontmatter delimited by '---'".to_string())
            .and_then(|frontmatter| Self::from_front_matter(&frontmatter))
    }
}
impl MarkdownSupport for LogbookEntry {
    fn to_markdown(&self) -> String {
        let archived = if self.archived { " · archived" } else { Default::default() };
        let classification = self.classification.as_ref().map(|value| format!(" · {value}")).unwrap_or_default();
        let event_type = self.event_type.map(|value| format!(" · {value:?}")).unwrap_or_default();
        let items = self
            .items
            .as_ref()
            .map(|items| format!("\n{}", items.iter().map(|item| format!("- {item}")).collect::<Vec<_>>().join("\n")))
            .unwrap_or_default();
        format!(
            "## {} — {:?}\n\n**{:?}{classification}{event_type} · {:?}{archived}**\n\n{}{}",
            self.timestamp, self.state, self.category, self.origin, self.summary, items
        )
    }
}
impl core::error::Error for LogbookError {}
impl fmt::Display for LogbookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::UnknownGraduationIdentifier(identifier) => write!(formatter, "No graduation candidate has identifier '{identifier}'"),
            | Self::InvalidTimestamp(value) => write!(formatter, "Invalid RFC 3339 timestamp with explicit offset: {value}"),
            | Self::InvalidWindow(value) => write!(formatter, "Invalid reporting window: {value}"),
        }
    }
}
impl From<&str> for LogbookEventType {
    fn from(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            | "conference" => Self::Conference,
            | "exhibition" => Self::Exhibition,
            | "meeting" => Self::Meeting,
            | "presentation" => Self::Presentation,
            | "tour" => Self::Tour,
            | "training" => Self::Training,
            | "workshop" => Self::Workshop,
            | _ => Self::Other,
        }
    }
}
impl From<String> for LogbookEventType {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}
impl LogbookEventType {
    /// Return the published ontology namespace represented by this event type.
    pub fn namespace(self) -> Namespace<&'static str> {
        Namespace::new_unchecked(match self {
            | Self::Conference => "http://purl.org/ontology/bibo/Conference",
            | Self::Exhibition => "https://schema.org/ExhibitionEvent",
            | Self::Meeting | Self::Other | Self::Presentation | Self::Tour => "https://schema.org/Event",
            | Self::Training => "https://schema.org/EducationEvent",
            | Self::Workshop => "http://purl.org/ontology/bibo/Workshop",
        })
    }
}
impl From<&str> for LogbookState {
    fn from(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            | "completed" => Self::Completed,
            | "upcoming" => Self::Upcoming,
            | _ => Self::InProgress,
        }
    }
}
impl From<String> for LogbookState {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}
impl LogbookUpdates {
    /// Return the number of selected entries already archived into canonical metadata.
    pub fn archived_count(&self) -> usize {
        self.entries.iter().filter(|entry| entry.archived).count()
    }
}
impl MarkdownSupport for LogbookUpdates {
    fn to_markdown(&self) -> String {
        let subtitle = self.subtitle.as_ref().map(|value| format!("\n_{value}_\n")).unwrap_or_default();
        let entries = self.entries.iter().map(MarkdownSupport::to_markdown).collect::<Vec<_>>().join("\n\n");
        let period = self.through.as_ref().map_or_else(
            || format!("Since `{}`", self.since),
            |through| match self.since.as_str() {
                | "full-history" => format!("Through `{through}` (exclusive)"),
                | since => format!("From `{since}` through `{through}` (exclusive)"),
            },
        );
        format!(
            "# {} updates\n{subtitle}\n{period}\n\n{}\n",
            self.title,
            match entries.is_empty() {
                | true => "_No updates in this period._".to_string(),
                | false => entries,
            }
        )
    }
}
impl ReportingWindow {
    /// Construct a reporting window whose upper bound is exclusive.
    pub fn new(from: Option<&str>, through: &str) -> Result<Self, LogbookError> {
        let window = Self {
            left: from.map(str::to_string),
            right: through.to_string(),
        };
        window
            .validate()
            .map_err(|why| LogbookError::InvalidWindow(why.to_string()))
            .map(|()| window)
    }
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        let left = self.left.as_deref().map(rules::parse_timestamp).transpose();
        let right = rules::parse_timestamp(&self.right);
        if matches!((left, right), (Ok(Some(left)), Ok(right)) if left >= right) {
            report.add(
                "right",
                ValidationError::new("reporting_window").with_message("The exclusive right bound must be after the inclusive left bound"),
            );
        }
        report.finish()
    }
    fn left_instant(&self) -> Result<Option<Timestamp>, LogbookError> {
        self.left
            .as_deref()
            .map(|value| rules::parse_timestamp(value).map_err(|_| LogbookError::InvalidTimestamp(value.to_string())))
            .transpose()
    }
    fn right_instant(&self) -> Result<Timestamp, LogbookError> {
        rules::parse_timestamp(&self.right).map_err(|_| LogbookError::InvalidTimestamp(self.right.clone()))
    }
}
impl ResearchActivity {
    /// Apply accepted graduation candidates, retaining and archiving their source entries.
    pub fn graduate(self, accepted_ids: &[String]) -> Result<Self, LogbookError> {
        let candidates = GraduationCandidates::from(&self).0;
        accepted_ids
            .iter()
            .find(|identifier| {
                !candidates
                    .iter()
                    .any(|candidate| candidate.entry_identifier.as_str() == identifier.as_str())
            })
            .map(|identifier| LogbookError::UnknownGraduationIdentifier(identifier.clone()))
            .map_or_else(
                || {
                    let accepted = candidates
                        .into_iter()
                        .filter(|candidate| accepted_ids.contains(&candidate.entry_identifier))
                        .collect::<Vec<_>>();
                    let classification = self
                        .meta
                        .classification
                        .clone()
                        .into_iter()
                        .chain(
                            self.logbook
                                .as_ref()
                                .into_iter()
                                .flat_map(|logbook| &logbook.entries)
                                .filter(|entry| {
                                    accepted_ids.contains(&entry.identifier)
                                        && entry.category == LogbookCategory::Administration
                                        && entry.event_type == Some(LogbookEventType::Other)
                                })
                                .filter_map(|entry| entry.classification.clone()),
                        )
                        .max();
                    let existing = self.meta.contributors.clone().unwrap_or_default();
                    let additions = accepted
                        .into_iter()
                        .filter(|candidate| candidate.target == GraduationTarget::Contributors)
                        .map(|candidate| candidate.value)
                        .filter(|name| !existing.iter().any(|contributor| contributor.name.eq_ignore_ascii_case(name)))
                        .map(ActivityContributor::from)
                        .collect::<Vec<_>>();
                    let contributors =
                        (!existing.is_empty() || !additions.is_empty()).then(|| existing.into_iter().chain(additions).collect::<OneOrMany<_>>());
                    Ok(Self {
                        meta: ResearchActivityMetadata {
                            classification,
                            contributors,
                            ..self.meta
                        },
                        logbook: self.logbook.map(|logbook| Logbook {
                            entries: logbook
                                .entries
                                .into_iter()
                                .map(|entry| match accepted_ids.contains(&entry.identifier) {
                                    | true => LogbookEntry {
                                        archived: true,
                                        state: LogbookState::Completed,
                                        ..entry
                                    },
                                    | false => entry,
                                })
                                .collect(),
                            ..logbook
                        }),
                        ..self
                    })
                },
                Err,
            )
    }
    /// Select entries at or after an inclusive RFC 3339 instant and order them newest first.
    pub fn updates_since(&self, since: &str) -> Result<LogbookUpdates, LogbookError> {
        self.logbook
            .as_ref()
            .map_or(Ok(Vec::new()), |logbook| logbook.entries(since))
            .map(|entries| LogbookUpdates {
                identifier: self.meta.identifier.clone(),
                title: self.title.clone(),
                subtitle: self.subtitle.clone(),
                since: since.to_string(),
                through: None,
                entries,
            })
    }
    /// Select entries in a half-open reporting interval and order them newest first.
    pub fn updates_within(&self, window: &ReportingWindow) -> Result<LogbookUpdates, LogbookError> {
        window
            .validate()
            .map_err(|why| LogbookError::InvalidWindow(why.to_string()))
            .and_then(|()| {
                self.logbook
                    .as_ref()
                    .map_or(Ok(Vec::new()), |logbook| logbook.entries_within(window))
                    .map(|entries| LogbookUpdates {
                        identifier: self.meta.identifier.clone(),
                        title: self.title.clone(),
                        subtitle: self.subtitle.clone(),
                        since: window.left.clone().unwrap_or_else(|| "full-history".to_string()),
                        through: Some(window.right.clone()),
                        entries,
                    })
            })
    }
}
