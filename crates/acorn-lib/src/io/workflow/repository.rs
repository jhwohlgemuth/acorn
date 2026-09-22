//! Transport-neutral repository workflow contracts
use acorn_core::prelude::{String, Vec};
use acorn_core::util::{MarkdownSupport, MimeType};
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
    /// Check quality gate (read-only findings).
    Check,
    /// Invoke configured enrichment capabilities.
    Enrich,
    /// Normalize and deterministically serialize content.
    Format,
    /// Create linked-data companion artifacts.
    Link,
    /// Parse and validate content without requiring a rewrite.
    Validate,
}
/// One deterministic file mutation proposed by a workflow.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileChange {
    /// Complete UTF-8 replacement content.
    pub content: String,
    /// Mutation kind.
    pub kind: FileChangeKind,
    /// Repository-relative path.
    pub path: String,
}
/// Deterministic output of one repository workflow.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RepositoryChangeSet {
    /// Exact source revision used to create the changes.
    pub base_revision: String,
    /// Ordered file mutations.
    pub files: Vec<FileChange>,
    /// Ordered per-file reports.
    pub reports: Vec<WorkflowFileReport>,
}
/// One immutable repository file supplied at a selected revision.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RepositoryFileInput {
    /// Repository-relative path.
    pub path: String,
    /// UTF-8 file content read at `revision`.
    pub content: String,
    /// Explicit format, or `None` to infer it from the path and content.
    pub format: Option<MimeType>,
}
/// Immutable repository snapshot selected for workflow processing.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RepositoryInput {
    /// Requested processing actions.
    pub actions: Vec<WorkflowAction>,
    /// Files read at the selected revision.
    pub files: Vec<RepositoryFileInput>,
    /// Provider project identifier.
    pub project: String,
    /// Exact source commit SHA.
    pub revision: String,
}
/// Processing report for one repository file.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WorkflowFileReport {
    /// Whether the workflow proposed a source rewrite.
    pub changed: bool,
    /// Effective format.
    pub format: MimeType,
    /// Repository-relative path.
    pub path: String,
}
impl From<FileChangeKind> for &'static str {
    fn from(value: FileChangeKind) -> Self {
        match value {
            | FileChangeKind::Create => "create",
            | FileChangeKind::Update => "update",
        }
    }
}
impl MarkdownSupport for RepositoryChangeSet {
    fn to_markdown(&self) -> String {
        self.files
            .iter()
            .map(|change| {
                let kind: &'static str = change.kind.into();
                format!("- `{}` ({kind})", change.path)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
impl RepositoryFileInput {
    /// Return the explicit or path-inferred supported repository file type.
    pub fn format(&self) -> Option<MimeType> {
        let format = self.format.clone().unwrap_or_else(|| MimeType::from(self.path.as_str()));
        match format {
            | MimeType::Cff | MimeType::Json | MimeType::Jsonc | MimeType::Markdown | MimeType::Yaml => Some(format),
            | _ => None,
        }
    }
}
