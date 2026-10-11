//! Vocabulary resolution data types.
use super::matching::stable_unique;
use acorn_core::prelude::alloc::{String, ToString, Vec};
use acorn_core::Noul;
use bon::bon;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::EnumIs;

/// Vocabulary resolution failure that applies to a whole prepared vocabulary or batch.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum VocabularyError {
    /// An embedded vocabulary asset could not be loaded or parsed.
    #[error("Failed to load the {vocabulary} vocabulary: {message}")]
    InvalidAsset {
        /// Asset failure detail.
        message: String,
        /// Vocabulary whose asset failed.
        vocabulary: VocabularyKind,
    },
    /// A configured batch or prepared-vocabulary limit was exceeded.
    #[error("Vocabulary {name} limit exceeded: {actual} > {limit}")]
    LimitExceeded {
        /// Observed count.
        actual: usize,
        /// Limit name.
        name: &'static str,
        /// Configured maximum.
        limit: usize,
    },
    /// A serialized vocabulary name is not recognized.
    #[error("Unknown vocabulary kind: {value}")]
    UnknownKind {
        /// Unrecognized serialized name.
        value: String,
    },
}
/// Fuzzy-matching policy selected by a caller.
#[derive(Clone, Copy, Debug, Default, Deserialize, EnumIs, Eq, JsonSchema, PartialEq, Serialize, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum VocabularyFuzzyMode {
    /// Disable similarity scoring while retaining exact and normalized matching.
    Off,
    /// Apply lower acceptance gates while preserving ambiguity and resource safeguards.
    Relaxed,
    /// Apply the default high-precision acceptance gates.
    #[default]
    Strict,
}
/// Embedded controlled-vocabulary identity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum VocabularyKind {
    /// Research keywords.
    Keywords,
    /// ORNL organization hierarchy used by contact fields.
    Organizations,
    /// Research partner organizations.
    Partners,
    /// Research sponsor organizations.
    Sponsors,
    /// Software and data technologies.
    Technology,
}
/// How an accepted or suggested candidate matched an entry.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum VocabularyMatchKind {
    /// Declared alias.
    Alias,
    /// Canonical value.
    Canonical,
    /// Fuzzy character or token comparison.
    Fuzzy,
    /// Typed identifier such as a ROR, classification path, or homepage.
    Identifier,
    /// Unique vocabulary-specific normalized equality.
    Normalized,
}
/// Per-input resolution status.
#[derive(Clone, Copy, Debug, Deserialize, EnumIs, Eq, JsonSchema, PartialEq, Serialize, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum VocabularyOutcome {
    /// More than one canonical identity is equally eligible.
    Ambiguous,
    /// A declared alias matched.
    ExactAlias,
    /// A canonical value matched.
    ExactCanonical,
    /// A typed identifier matched.
    ExactIdentifier,
    /// A unique candidate passed the selected fuzzy policy.
    FuzzyAccepted,
    /// The input is empty or otherwise unsuitable for matching.
    Invalid,
    /// A resource limit prevented a complete search.
    ResourceLimited,
    /// No candidate passed the selected policy.
    Unmatched,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct Entry {
    pub(super) aliases: Vec<String>,
    pub(super) canonical: String,
    pub(super) identifiers: Vec<String>,
    pub(super) locator: String,
}
#[derive(Clone, Copy)]
pub(super) struct FuzzyGate {
    pub(super) margin: f64,
    pub(super) maximum_distance: Option<usize>,
    pub(super) minimum_score: f64,
}
#[derive(Clone, Debug)]
pub(super) struct ScoredCandidate {
    pub(super) candidate: VocabularyCandidate,
    pub(super) distance: Option<usize>,
    pub(super) entry: usize,
}
/// Audit summary for a prepared vocabulary.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyAudit {
    /// Number of declared aliases.
    pub aliases: usize,
    /// Normalized aliases or canonical values shared by distinct entries.
    pub collisions: Vec<String>,
    /// Number of canonical entries.
    pub entries: usize,
}
/// One accepted or suggested canonical candidate.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyCandidate {
    /// Canonical output value.
    pub canonical: String,
    /// Edit distance when the selected method reports one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_distance: Option<usize>,
    /// Stable hierarchy or asset locator.
    pub locator: String,
    /// Term that produced this score.
    pub matched: String,
    /// Matching method or typed lookup used.
    pub method: String,
    /// Similarity score from zero to one.
    pub score: Noul,
}
/// A formatted value together with every vocabulary report produced while formatting it.
#[derive(Clone, Debug, PartialEq)]
pub struct VocabularyFormat<T> {
    /// Ordered vocabulary reports.
    pub reports: Vec<VocabularyReport>,
    /// Formatted value.
    pub value: T,
}
/// Resource bounds for vocabulary preparation and batch resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VocabularyLimits {
    /// Maximum aliases across all entries.
    pub max_aliases: usize,
    /// Maximum cumulative fuzzy work for a batch.
    pub max_batch_work: usize,
    /// Maximum returned candidates per input.
    pub max_candidates: usize,
    /// Maximum canonical entries.
    pub max_entries: usize,
    /// Maximum Unicode scalars in one input.
    pub max_input_scalars: usize,
    /// Maximum inputs in one batch.
    pub max_inputs: usize,
    /// Maximum fuzzy work for one input.
    pub max_work_per_input: usize,
}
/// Options for one deterministic vocabulary-resolution batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VocabularyOptions {
    /// Fuzzy acceptance profile.
    pub fuzzy: VocabularyFuzzyMode,
    /// Resource bounds.
    pub limits: VocabularyLimits,
}
/// Ordered report for one vocabulary batch.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyReport {
    /// Selected fuzzy profile.
    pub fuzzy: VocabularyFuzzyMode,
    /// Stable policy revision.
    pub policy_revision: String,
    /// Ordered per-input resolutions.
    pub resolutions: Vec<VocabularyResolution>,
    /// Embedded vocabulary revision.
    pub vocabulary_revision: String,
    /// Vocabulary identity.
    pub vocabulary: VocabularyKind,
}
/// Ordered result for one supplied input.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyResolution {
    /// Bounded candidates in deterministic score order.
    pub candidates: Vec<VocabularyCandidate>,
    /// Accepted canonical value, when any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical: Option<String>,
    /// Whether every eligible entry was searched.
    pub complete: bool,
    /// Edit distance for the leading candidate when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_distance: Option<usize>,
    /// Original caller-supplied value.
    pub input: String,
    /// Stable hierarchy or asset locator for an accepted value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
    /// Kind of accepted match.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_kind: Option<VocabularyMatchKind>,
    /// Term that matched the accepted value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched: Option<String>,
    /// Similarity method or typed lookup used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Per-input status.
    pub outcome: VocabularyOutcome,
    /// Difference between the best and runner-up distinct canonical scores.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runner_up_margin: Option<Noul>,
    /// Accepted or leading similarity score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<Noul>,
    /// Caller-supplied field or source locator for this input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_locator: Option<String>,
}
/// Prepared reusable resolver for one embedded vocabulary.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VocabularyResolver {
    pub(super) entries: Vec<Entry>,
    pub(super) kind: VocabularyKind,
    pub(super) revision: String,
}

impl TryFrom<&str> for VocabularyKind {
    type Error = VocabularyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            | "keywords" => Ok(Self::Keywords),
            | "organizations" => Ok(Self::Organizations),
            | "partners" => Ok(Self::Partners),
            | "sponsors" => Ok(Self::Sponsors),
            | "technology" => Ok(Self::Technology),
            | _ => Err(VocabularyError::UnknownKind { value: value.to_string() }),
        }
    }
}
impl Default for VocabularyLimits {
    fn default() -> Self {
        Self {
            max_aliases: 8_192,
            max_batch_work: 32_000_000,
            max_candidates: 3,
            max_entries: 4_096,
            max_input_scalars: 256,
            max_inputs: 256,
            max_work_per_input: 4_000_000,
        }
    }
}
impl VocabularyOutcome {
    /// Return whether the outcome supplies a canonical accepted value.
    pub fn is_accepted(self) -> bool {
        self.is_exact_alias() || self.is_exact_canonical() || self.is_exact_identifier() || self.is_fuzzy_accepted()
    }
}
impl VocabularyReport {
    /// Return accepted canonical values, stably deduplicated in input order.
    pub fn accepted_values(&self) -> Vec<String> {
        stable_unique(
            self.resolutions
                .iter()
                .filter_map(|resolution| resolution.outcome.is_accepted().then(|| resolution.canonical.clone()).flatten()),
        )
    }
    /// Return canonical accepted values and otherwise preserve non-empty original inputs.
    pub fn lossless_values(&self) -> Vec<String> {
        stable_unique(self.resolutions.iter().filter_map(|resolution| {
            resolution.canonical.clone().filter(|_| resolution.outcome.is_accepted()).or_else(|| {
                let value = resolution.input.trim();
                (!value.is_empty()).then(|| value.to_string())
            })
        }))
    }
}
#[bon]
impl VocabularyResolution {
    #[builder]
    pub(super) fn resolution(
        input: String,
        outcome: VocabularyOutcome,
        #[builder(default)] candidates: Vec<VocabularyCandidate>,
        #[builder(default = true)] complete: bool,
        edit_distance: Option<usize>,
        entry: Option<&Entry>,
        matched: Option<String>,
        match_kind: Option<VocabularyMatchKind>,
        method: Option<String>,
        score: Option<Noul>,
        runner_up_margin: Option<Noul>,
    ) -> Self {
        Self {
            candidates,
            canonical: entry.map(|entry| entry.canonical.clone()),
            complete,
            edit_distance,
            input,
            locator: entry.map(|entry| entry.locator.clone()),
            match_kind,
            matched,
            method,
            outcome,
            runner_up_margin,
            score,
            source_locator: None,
        }
    }
}
