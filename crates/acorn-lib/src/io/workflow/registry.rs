//! Immutable registry of named repository workflows
use super::repository::{FileChange, FileChangeKind, RepositoryChangeSet, RepositoryFileInput, RepositoryInput, WorkflowAction, WorkflowFileReport};
use crate::io::{jsonc_parse_value, ApiResult, InputOutput};
use acorn_core::{
    prelude::{String, Vec},
    util::{MarkdownSupport, MimeType},
};
use acorn_host::fs::SafePath;
use acorn_schema::standard::zon;
use acorn_schema::{research_activity::ResearchActivity, standard::cff::Cff, validation::Validate};
use alloc::sync::Arc;
use color_eyre::eyre::eyre;
use core::fmt;
use serde_json::Value;

type WorkflowHandler = Arc<dyn Fn(RepositoryInput) -> ApiResult<RepositoryChangeSet> + Send + Sync>;
trait NormalizeRepositoryContent {
    fn normalize(&self, content: &str, actions: &[WorkflowAction]) -> ApiResult<(String, Option<String>)>;
}
trait NormalizeResearchActivity {
    fn normalize(self, mime: &MimeType, actions: &[WorkflowAction]) -> ApiResult<(String, Option<String>)>;
}
#[derive(Clone)]
struct RegisteredWorkflow {
    handler: WorkflowHandler,
    name: String,
}
/// Immutable registry of deployment-approved repository workflows.
#[derive(Clone, Default)]
pub struct RepositoryWorkflowRegistry {
    workflows: Vec<RegisteredWorkflow>,
}
impl NormalizeRepositoryContent for MimeType {
    fn normalize(&self, content: &str, actions: &[WorkflowAction]) -> ApiResult<(String, Option<String>)> {
        match actions.contains(&WorkflowAction::Enrich) {
            | true => Err(eyre!("Repository enrichment requires an explicitly configured capability")),
            | false => match self {
                | Self::Cff => serde_norway::from_str::<Cff>(content)
                    .map_err(|why| eyre!("Failed to parse {self} — {why}"))
                    .and_then(|cff| cff.validate().map(|()| cff).map_err(|why| eyre!("Invalid {self} — {why}")))
                    .and_then(|cff| serde_norway::to_string(&cff).map_err(|why| eyre!("Failed to serialize {self} — {why}")))
                    .map(|content| (content, None)),
                | Self::Json => match eserde::json::from_str::<ResearchActivity>(content) {
                    | Ok(activity) => activity.normalize(self, actions),
                    | Err(_) => serde_json::from_str::<Value>(content)
                        .and_then(|value| serde_json::to_string_pretty(&value))
                        .map(|content| (content, None))
                        .map_err(|why| eyre!("Failed to process {self} — {why}")),
                },
                | Self::Jsonc => jsonc_parse_value(content).and_then(|value| match serde_json::from_value::<ResearchActivity>(value.clone()) {
                    | Ok(activity) => activity.normalize(self, actions),
                    | Err(_) => serde_json::to_string_pretty(&value)
                        .map(|content| (content, None))
                        .map_err(|why| eyre!("Failed to serialize {self} — {why}")),
                }),
                | Self::Markdown => ResearchActivity::from_markdown(content)
                    .map_err(|why| eyre!(why))
                    .and_then(|activity| activity.normalize(self, actions))
                    .or_else(|_| {
                        let normalized = format!("{}\n", content.replace("\r\n", "\n").trim_end());
                        match normalized.trim().is_empty() {
                            | true => Err(eyre!("{self} repository file cannot be empty")),
                            | false => Ok((normalized, None)),
                        }
                    }),
                | Self::Yaml => match serde_norway::from_str::<ResearchActivity>(content) {
                    | Ok(activity) => activity.normalize(self, actions),
                    | Err(_) => serde_norway::from_str::<Value>(content)
                        .and_then(|value| serde_norway::to_string(&value))
                        .map(|content| (content, None))
                        .map_err(|why| eyre!("Failed to process {self} — {why}")),
                },
                | Self::Zon => match zon::decode::<ResearchActivity>(content) {
                    | Ok(activity) => activity.normalize(self, actions),
                    | Err(_) => zon::decode_value(content)
                        .and_then(|value| zon::encode(&value))
                        .map(|content| (content, None))
                        .map_err(|why| eyre!("Failed to process {self} — {why}")),
                },
                | _ => Err(eyre!("Unsupported repository MIME type '{self}'")),
            },
        }
    }
}
impl TryFrom<RepositoryInput> for RepositoryChangeSet {
    type Error = color_eyre::Report;
    fn try_from(input: RepositoryInput) -> Result<Self, Self::Error> {
        match (input.project.trim().is_empty(), input.revision.trim().is_empty()) {
            | (true, _) => Err(eyre!("Repository project identifier is required")),
            | (_, true) => Err(eyre!("Repository revision is required")),
            | _ => {
                let mut files = input.files;
                files.sort_by(|left, right| left.path.cmp(&right.path));
                files
                    .into_iter()
                    .map(|file| file.normalize(&input.actions))
                    .collect::<ApiResult<Vec<_>>>()
                    .map(|processed| RepositoryChangeSet {
                        base_revision: input.revision,
                        files: processed.iter().flat_map(|(changes, _)| changes.clone()).collect(),
                        reports: processed.into_iter().map(|(_, report)| report).collect(),
                    })
            }
        }
    }
}
impl RepositoryFileInput {
    fn normalize(self, actions: &[WorkflowAction]) -> ApiResult<(Vec<FileChange>, WorkflowFileReport)> {
        let safe_path = SafePath::new(&self.path).map_err(|why| eyre!(why));
        match (safe_path, self.format()) {
            | (Ok(safe_path), Some(format)) => format.normalize(&self.content, actions).map(|(content, linked)| {
                let path = safe_path.as_path().to_string_lossy().to_string();
                let changed = content != self.content;
                let source_change = (changed && actions.contains(&WorkflowAction::Format)).then(|| FileChange {
                    content,
                    kind: FileChangeKind::Update,
                    path: path.clone(),
                });
                let linked_change = linked.map(|content| FileChange {
                    content,
                    kind: FileChangeKind::Create,
                    path: safe_path.as_path().with_extension("jsonld").to_string_lossy().to_string(),
                });
                let changes = source_change.into_iter().chain(linked_change).collect();
                let report = WorkflowFileReport { changed, format, path };
                (changes, report)
            }),
            | (Err(why), _) => Err(why),
            | (_, None) => Err(eyre!("Unsupported repository file format for '{}'", self.path)),
        }
    }
}
impl RepositoryWorkflowRegistry {
    /// Build the conservative built-in repository workflow registry.
    pub fn acorn() -> ApiResult<Self> {
        Self::default().register("repository-quality", RepositoryChangeSet::try_from)
    }
    /// Return registered workflow names in deterministic order.
    pub fn names(&self) -> Vec<String> {
        self.workflows.iter().map(|workflow| workflow.name.clone()).collect()
    }
    /// Process a repository snapshot through a registered workflow.
    pub fn process(&self, name: &str, input: RepositoryInput) -> ApiResult<RepositoryChangeSet> {
        self.workflows
            .iter()
            .find(|workflow| workflow.name == name)
            .ok_or_else(|| eyre!("Unknown repository workflow '{name}'"))
            .and_then(|workflow| (workflow.handler)(input))
    }
    /// Register a deployment-approved repository workflow.
    pub fn register(
        self,
        name: impl Into<String>,
        handler: impl Fn(RepositoryInput) -> ApiResult<RepositoryChangeSet> + Send + Sync + 'static,
    ) -> ApiResult<Self> {
        let name = name.into();
        match name.trim().is_empty() || self.workflows.iter().any(|workflow| workflow.name == name) {
            | true => Err(eyre!("Repository workflow name is empty or already registered")),
            | false => {
                let mut workflows = self.workflows;
                workflows.push(RegisteredWorkflow {
                    handler: Arc::new(handler),
                    name,
                });
                workflows.sort_by(|left, right| left.name.cmp(&right.name));
                Ok(Self { workflows })
            }
        }
    }
}
impl fmt::Debug for RepositoryWorkflowRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RepositoryWorkflowRegistry")
            .field("names", &self.names())
            .finish()
    }
}
impl NormalizeResearchActivity for ResearchActivity {
    fn normalize(self, mime: &MimeType, actions: &[WorkflowAction]) -> ApiResult<(String, Option<String>)> {
        self.validate()
            .map_err(|why| eyre!("Invalid research activity data — {why}"))
            .and_then(|()| {
                let activity = match actions.contains(&WorkflowAction::Format) {
                    | true => InputOutput::format_with(self, None),
                    | false => self,
                };
                InputOutput::serialize_as(&activity, mime).and_then(|content| match actions.contains(&WorkflowAction::Link) {
                    | true => InputOutput::linked_content(&activity).map(|linked| (content, linked)),
                    | false => Ok((content, None)),
                })
            })
    }
}
