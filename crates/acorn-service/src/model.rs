//! Canonical requests and responses shared by direct and remote callers.
use acorn_analysis::readability::ReadabilityType;
use acorn_analysis::{CheckCategory, CheckSeverity};
use acorn_core::prelude::{format, String, Vec};
use acorn_core::util::{MarkdownSupport, MimeType};
use acorn_schema::standard::Standard;
use acorn_schema::vocabulary::VocabularyFuzzyMode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Kind of repository mutation proposed by a workflow.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, strum::EnumIs)]
#[serde(rename_all = "kebab-case")]
pub enum FileChangeKind {
    /// Create a new derived artifact.
    Create,
    /// Replace an existing source file.
    Update,
}
/// Registered repository processing action.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, strum::Display, strum::EnumIs)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum WorkflowAction {
    /// Check quality without changing content.
    Check,
    /// Invoke configured enrichment capabilities.
    Enrich,
    /// Normalize and deterministically serialize content.
    Format,
    /// Create a linked-data companion artifact.
    Link,
    /// Parse and validate content.
    Validate,
}
/// Calculate one readability metric over supplied text.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CheckReadabilityRequest {
    /// Readability metric to calculate.
    pub metric: ReadabilityType,
    /// Optional logical source identifier.
    pub source_uri: Option<String>,
    /// Complete supplied prose.
    pub text: String,
    /// Optional metric threshold; uses the canonical default when absent.
    pub threshold: Option<f64>,
}
/// Deterministic readability calculation and assessment.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CheckReadabilityResponse {
    /// Whether the text satisfies the configured threshold.
    pub acceptable: bool,
    /// Readability metric calculated.
    pub metric: ReadabilityType,
    /// Optional logical source identifier.
    pub source_uri: Option<String>,
    /// Rounded metric score.
    pub value: f64,
    /// Threshold used for assessment.
    pub threshold: f64,
}
/// One normalized user-facing diagnostic.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Diagnostic {
    /// Stable diagnostic code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Optional field or logical document location.
    pub path: Option<String>,
    /// Diagnostic severity.
    pub severity: CheckSeverity,
}
/// One deterministic repository mutation proposal.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileChange {
    /// Complete UTF-8 replacement content.
    pub content: String,
    /// Mutation kind.
    pub kind: FileChangeKind,
    /// Portable repository-relative path.
    pub path: String,
}
/// Process one caller-supplied immutable repository snapshot.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProcessRepositoryRequest {
    /// Ordered requested actions.
    pub actions: Vec<WorkflowAction>,
    /// Files read at the selected revision.
    pub files: Vec<RepositoryFileInput>,
    /// Provider project identifier.
    pub project: String,
    /// Exact source revision.
    pub revision: String,
}
/// Deterministic output of repository processing.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProcessRepositoryResponse {
    /// Exact source revision used to create the proposal.
    pub base_revision: String,
    /// Ordered proposed file mutations.
    pub files: Vec<FileChange>,
    /// Ordered per-file reports.
    pub reports: Vec<WorkflowFileReport>,
}
/// One immutable repository file supplied at a selected revision.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RepositoryFileInput {
    /// Complete UTF-8 content.
    pub content: String,
    /// Explicit format, or `None` to infer it from the path.
    pub format: Option<MimeType>,
    /// Portable repository-relative path.
    pub path: String,
}
/// Validate one supplied research activity document.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ValidateResearchActivityRequest {
    /// Complete UTF-8 document content.
    pub content: String,
    /// Explicit document format.
    pub format: MimeType,
    /// Optional logical source identifier used only in diagnostics.
    pub source_uri: Option<String>,
    /// Controlled-vocabulary matching policy; defaults to relaxed advisory matching.
    #[serde(default = "relaxed_vocabulary_fuzzy")]
    pub vocabulary_fuzzy: VocabularyFuzzyMode,
}
/// Complete validation outcome for supplied research activity data.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ValidateResearchActivityResponse {
    /// Every normalized parse or validation diagnostic.
    pub diagnostics: Vec<Diagnostic>,
    /// Whether parsing and schema validation succeeded.
    pub valid: bool,
    /// Effective controlled-vocabulary policy for research-activity validation.
    pub vocabulary_fuzzy: Option<VocabularyFuzzyMode>,
}
/// Validate one supplied document against a selected metadata standard.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ValidateStandardRequest {
    /// Complete UTF-8 document content.
    pub content: String,
    /// Explicit document format.
    pub format: MimeType,
    /// Optional logical source identifier used only in diagnostics.
    pub source_uri: Option<String>,
    /// Metadata standard the document must satisfy.
    pub standard: Standard,
}
/// Processing report for one repository file.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WorkflowFileReport {
    /// Whether normalized content differs from the supplied content.
    pub changed: bool,
    /// Effective format.
    pub format: MimeType,
    /// Portable repository-relative path.
    pub path: String,
}
impl From<WorkflowAction> for CheckCategory {
    fn from(value: WorkflowAction) -> Self {
        match value {
            | WorkflowAction::Validate => Self::Schema,
            | _ => Self::Quality,
        }
    }
}
impl MarkdownSupport for ProcessRepositoryResponse {
    fn to_markdown(&self) -> String {
        self.files
            .iter()
            .map(|change| format!("- `{}` ({})", change.path, <&'static str>::from(change.kind)))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
impl RepositoryFileInput {
    /// Return the explicit or path-inferred supported repository file format.
    pub fn format(&self) -> Option<MimeType> {
        let format = self.format.clone().unwrap_or_else(|| MimeType::from(self.path.as_str()));
        matches!(
            format,
            MimeType::Cff
                | MimeType::Json
                | MimeType::Jsonc
                | MimeType::LdJson
                | MimeType::Markdown
                | MimeType::RdfXml
                | MimeType::Turtle
                | MimeType::Yaml
                | MimeType::Zon
        )
        .then_some(format)
    }
}
impl From<FileChangeKind> for &'static str {
    fn from(value: FileChangeKind) -> Self {
        match value {
            | FileChangeKind::Create => "create",
            | FileChangeKind::Update => "update",
        }
    }
}
const fn relaxed_vocabulary_fuzzy() -> VocabularyFuzzyMode {
    VocabularyFuzzyMode::Relaxed
}
