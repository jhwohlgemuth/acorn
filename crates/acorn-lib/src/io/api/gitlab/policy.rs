//! Deployment-owned GitLab repository workflow policy
use crate::io::workflow::{RepositoryWorkflowRegistry, WorkflowAction};
use crate::io::ApiResult;
use acorn_core::prelude::{String, Vec};
use acorn_core::util::{expand_brace_glob, MimeType};
use acorn_core::validation::ValidationError;
use acorn_schema::validation::{IntoValidationReport, Validate, ValidationReport};
use alloc::collections::BTreeMap;
use color_eyre::eyre::eyre;
use glob::{MatchOptions, Pattern};
use serde::{Deserialize, Serialize};

/// Allowed external effect produced by a repository workflow.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowEffect {
    /// Create or reuse a bot branch.
    Branch,
    /// Create a commit on a bot branch.
    Commit,
    /// Publish or reconcile an external commit status.
    CommitStatus,
    /// Post the citation-completion comment.
    CompletionComment,
    /// Create or reuse a draft merge request.
    DraftMergeRequest,
    /// Apply a work-item label.
    Label,
    /// Create or update a bot-authored note.
    Note,
}
/// Output behavior for a deployment workflow.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowOutputMode {
    /// Update source files and create configured derived artifacts.
    SourceAndArtifacts,
}
/// Deployment configuration for repository automation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct DeploymentConfig {
    /// Whether a merge request from a fork may publish into the explicit destination.
    #[serde(default)]
    allow_forks: bool,
    /// Explicit project identifier that receives bot branches and merge requests.
    destination: Option<String>,
    /// Deployment-approved named workflows.
    workflows: BTreeMap<String, WorkflowPolicy>,
}
/// Output configuration for a deployment workflow.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct WorkflowOutputs {
    /// Output behavior.
    mode: WorkflowOutputMode,
}
/// Deployment-owned policy for one registered workflow.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Validate)]
#[validate(schema(function = "WorkflowPolicy::is_valid"))]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct WorkflowPolicy {
    /// Processing actions allowed for this workflow.
    #[validate(length(min = 1))]
    actions: Vec<WorkflowAction>,
    /// External effects allowed for this workflow.
    #[validate(length(min = 1))]
    effects: Vec<WorkflowEffect>,
    /// Input formats allowed for this workflow.
    #[validate(length(min = 1), each(custom(function = "validate_workflow_format")))]
    formats: Vec<MimeType>,
    /// Output behavior.
    outputs: WorkflowOutputs,
    /// Repository-relative glob patterns allowed for input.
    #[validate(length(min = 1), each(custom(function = "validate_path_pattern")))]
    paths: Vec<String>,
}
impl DeploymentConfig {
    /// Parse and validate deployment policy from YAML.
    pub fn from_yaml(content: &str) -> ApiResult<Self> {
        serde_norway::from_str(content)
            .map_err(|why| eyre!("Failed to parse GitLab workflow configuration — {why}"))
            .and_then(|config: Self| config.validate().map(|()| config))
    }
    /// Return whether fork merge requests are explicitly authorized.
    pub const fn allows_forks(&self) -> bool {
        self.allow_forks
    }
    /// Return the configured publication destination, or the webhook project for same-project work.
    pub fn destination<'a>(&'a self, webhook_project: &'a str) -> &'a str {
        self.destination.as_deref().unwrap_or(webhook_project)
    }
    /// Return whether publication is constrained to an explicitly named destination.
    pub const fn has_explicit_destination(&self) -> bool {
        self.destination.is_some()
    }
    /// Resolve one deployment-approved workflow.
    pub fn workflow(&self, name: &str) -> ApiResult<&WorkflowPolicy> {
        self.workflows
            .get(name)
            .ok_or_else(|| eyre!("Unknown or undeclared repository workflow `{name}`"))
    }
    /// Validate registered workflow names and every deployment-controlled value.
    pub fn validate(&self) -> ApiResult<()> {
        match self.workflows.is_empty() {
            | true => Err(eyre!("GitLab workflow configuration must declare at least one workflow")),
            | false => RepositoryWorkflowRegistry::acorn().and_then(|registry| {
                let registered = registry.names();
                self.workflows.iter().try_for_each(|(name, policy)| match registered.contains(name) {
                    | true => Validate::validate(policy).map_err(|why| eyre!("Invalid repository workflow `{name}` — {why}")),
                    | false => Err(eyre!("Unknown repository workflow `{name}`")),
                })
            }),
        }
        .and_then(|()| match self.destination.as_deref() {
            | Some(destination) if destination.trim().is_empty() => Err(eyre!("GitLab workflow destination cannot be empty")),
            | None if self.allow_forks => Err(eyre!("Fork workflow policy requires an explicit destination")),
            | _ => Ok(()),
        })
    }
}
impl Default for DeploymentConfig {
    fn default() -> Self {
        let workflow = WorkflowPolicy {
            actions: vec![WorkflowAction::Format, WorkflowAction::Link, WorkflowAction::Validate],
            effects: vec![
                WorkflowEffect::Branch,
                WorkflowEffect::Commit,
                WorkflowEffect::CommitStatus,
                WorkflowEffect::CompletionComment,
                WorkflowEffect::DraftMergeRequest,
                WorkflowEffect::Label,
                WorkflowEffect::Note,
            ],
            formats: vec![
                MimeType::Cff,
                MimeType::Json,
                MimeType::Jsonc,
                MimeType::Markdown,
                MimeType::Yaml,
                MimeType::Zon,
            ],
            outputs: WorkflowOutputs {
                mode: WorkflowOutputMode::SourceAndArtifacts,
            },
            paths: vec!["**/*.cff".to_string(), "**/*.{json,jsonc,yaml,yml,md,zonf}".to_string()],
        };
        Self {
            allow_forks: false,
            destination: None,
            workflows: BTreeMap::from([("repository-quality".to_string(), workflow)]),
        }
    }
}
impl WorkflowPolicy {
    /// Return the immutable processing action list.
    pub fn actions(&self) -> Vec<WorkflowAction> {
        self.actions.clone()
    }
    /// Return whether an external effect is deployment-approved.
    pub fn allows(&self, effect: WorkflowEffect) -> bool {
        self.effects.contains(&effect)
    }
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        match (
            has_duplicates(&self.formats),
            has_duplicates(&self.actions),
            has_duplicates(&self.effects),
        ) {
            | (true, _, _) => Err(ValidationError::new("formats").with_message("Duplicate workflow formats are not allowed")),
            | (_, true, _) => Err(ValidationError::new("actions").with_message("Duplicate workflow actions are not allowed")),
            | (_, _, true) => Err(ValidationError::new("effects").with_message("Duplicate workflow effects are not allowed")),
            | _ => Ok(()),
        }
        .map_err(|error| error.into_report(""))
    }
    /// Return whether a repository path and its format are deployment-approved.
    pub fn accepts(&self, path: &str) -> ApiResult<bool> {
        let format = MimeType::from(path);
        let allowed_format = self.formats.contains(&format);
        match allowed_format {
            | false => Ok(false),
            | true => self.paths.iter().try_fold(false, |matched, pattern| {
                expand_brace_glob(pattern).map_err(|why| eyre!(why)).and_then(|patterns| {
                    patterns.into_iter().try_fold(matched, |matched, pattern| {
                        Pattern::new(&pattern)
                            .map(|pattern| {
                                let options = MatchOptions {
                                    case_sensitive: true,
                                    require_literal_leading_dot: false,
                                    require_literal_separator: true,
                                };
                                matched || pattern.matches_with(path, options)
                            })
                            .map_err(|why| eyre!("Invalid repository path pattern `{pattern}` — {why}"))
                    })
                })
            }),
        }
    }
}
fn has_duplicates<T: PartialEq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values.iter().take(index).any(|existing| existing == value))
}
fn validate_path_pattern(pattern: &str) -> Result<(), ValidationError> {
    let unsafe_component = pattern
        .as_bytes()
        .split(|byte| *byte == b'/')
        .any(|component| component == b".." || component.is_empty());
    match pattern.trim().is_empty()
        || pattern.starts_with(['/', '\\'])
        || pattern.contains('\\')
        || pattern.as_bytes().get(1) == Some(&b':')
        || unsafe_component
    {
        | true => Err(ValidationError::new("path").with_message(format!("Unsafe repository path pattern `{pattern}`"))),
        | false => expand_brace_glob(pattern)
            .map_err(|why| ValidationError::new("path").with_message(why))
            .and_then(|patterns| {
                patterns.into_iter().try_for_each(|expanded| {
                    Pattern::new(&expanded)
                        .map(|_| ())
                        .map_err(|why| ValidationError::new("path").with_message(format!("Invalid repository path pattern `{pattern}` — {why}")))
                })
            }),
    }
}
fn validate_workflow_format(format: &MimeType) -> Result<(), ValidationError> {
    matches!(
        format,
        MimeType::Cff | MimeType::Json | MimeType::Jsonc | MimeType::Markdown | MimeType::Yaml | MimeType::Zon
    )
    .then_some(())
    .ok_or_else(|| ValidationError::new("format").with_message(format!("Unsupported repository workflow format `{format}`")))
}
