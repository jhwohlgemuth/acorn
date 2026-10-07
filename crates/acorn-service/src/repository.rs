//! Pure repository snapshot processing and lexical path validation.
use crate::model::{
    FileChange, FileChangeKind, ProcessRepositoryRequest, ProcessRepositoryResponse, RepositoryFileInput, WorkflowAction, WorkflowFileReport,
};
use crate::{ServiceError, ServiceErrorKind, ServiceResult};
use acorn_core::prelude::{format, String, ToString, Vec};
use acorn_core::util::{LinkedData, MarkdownSupport, MimeType};
use acorn_core::Location;
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::standard::{cff::Cff, zon};
use acorn_schema::validation::Validate;
use serde_json::Value;

trait NormalizeRepositoryContent {
    fn normalize(&self, content: &str, actions: &[WorkflowAction]) -> ServiceResult<NormalizedContent>;
    fn normalize_structured(&self, value: Value, actions: &[WorkflowAction]) -> ServiceResult<NormalizedContent>;
}
trait NormalizeResearchActivity {
    fn normalize(self, format: &MimeType, actions: &[WorkflowAction]) -> ServiceResult<NormalizedContent>;
    fn serialize(&self, format: &MimeType) -> ServiceResult<String>;
}
struct NormalizedContent {
    content: String,
    linked: Option<String>,
}

impl NormalizeRepositoryContent for MimeType {
    fn normalize(&self, content: &str, actions: &[WorkflowAction]) -> ServiceResult<NormalizedContent> {
        match self {
            | MimeType::Cff => serde_norway::from_str::<Cff>(content)
                .map_err(|error| invalid(error.to_string()))
                .and_then(|value| value.validate().map(|()| value).map_err(|error| invalid(error.to_string())))
                .and_then(|value| serde_norway::to_string(&value).map_err(|error| internal(error.to_string())))
                .map(|content| NormalizedContent { content, linked: None }),
            | MimeType::Json => serde_json::from_str::<Value>(content)
                .map_err(|error| invalid(error.to_string()))
                .and_then(|value| self.normalize_structured(value, actions)),
            | MimeType::Jsonc => jsonc_parser::parse_to_serde_value::<Value>(content, &Default::default())
                .map_err(|error| invalid(error.to_string()))
                .and_then(|value| self.normalize_structured(value, actions)),
            | MimeType::Markdown => ResearchActivity::from_markdown(content)
                .map_err(invalid)
                .and_then(|activity| activity.normalize(self, actions))
                .or_else(|_| {
                    let normalized = format!("{}\n", content.replace("\r\n", "\n").trim_end());
                    (!normalized.trim().is_empty())
                        .then_some(NormalizedContent {
                            content: normalized,
                            linked: None,
                        })
                        .ok_or_else(|| invalid("Markdown repository file cannot be empty"))
                }),
            | MimeType::Yaml => serde_norway::from_str::<Value>(content)
                .map_err(|error| invalid(error.to_string()))
                .and_then(|value| self.normalize_structured(value, actions)),
            | MimeType::Zon => zon::decode_value(content)
                .map_err(|error| invalid(error.to_string()))
                .and_then(|value| self.normalize_structured(value, actions)),
            | _ => Err(ServiceError::new(ServiceErrorKind::Unsupported, "Unsupported repository format")),
        }
    }
    fn normalize_structured(&self, value: Value, actions: &[WorkflowAction]) -> ServiceResult<NormalizedContent> {
        match serde_json::from_value::<ResearchActivity>(value.clone()) {
            | Ok(activity) => activity.normalize(self, actions),
            | Err(_) => match self {
                | MimeType::Json | MimeType::Jsonc => serde_json::to_string_pretty(&value).map_err(|error| internal(error.to_string())),
                | MimeType::Yaml => serde_norway::to_string(&value).map_err(|error| internal(error.to_string())),
                | MimeType::Zon => zon::encode(&value).map_err(|error| internal(error.to_string())),
                | _ => Err(ServiceError::new(
                    ServiceErrorKind::Unsupported,
                    "Unsupported structured repository format",
                )),
            }
            .map(|content| NormalizedContent { content, linked: None }),
        }
    }
}
impl NormalizeResearchActivity for ResearchActivity {
    fn normalize(mut self, format: &MimeType, actions: &[WorkflowAction]) -> ServiceResult<NormalizedContent> {
        self.validate().map_err(|error| invalid(error.to_string())).and_then(|()| {
            self = match actions.contains(&WorkflowAction::Format) {
                | true => self.format(),
                | false => self,
            };
            self.serialize(format).and_then(|content| {
                actions
                    .contains(&WorkflowAction::Link)
                    .then(|| serde_json::to_string_pretty(&self.with_context()).map_err(|error| internal(error.to_string())))
                    .transpose()
                    .map(|linked| NormalizedContent { content, linked })
            })
        })
    }
    fn serialize(&self, format: &MimeType) -> ServiceResult<String> {
        match format {
            | MimeType::Json | MimeType::Jsonc => serde_json::to_string_pretty(self).map_err(|error| internal(error.to_string())),
            | MimeType::Markdown => Ok(self.to_markdown()),
            | MimeType::Yaml => serde_norway::to_string(self).map_err(|error| internal(error.to_string())),
            | MimeType::Zon => zon::encode(self).map_err(|error| internal(error.to_string())),
            | _ => Err(ServiceError::new(ServiceErrorKind::Unsupported, "Unsupported research activity format")),
        }
    }
}
fn internal(message: impl Into<String>) -> ServiceError {
    ServiceError::new(ServiceErrorKind::Internal, message)
}
fn invalid(message: impl Into<String>) -> ServiceError {
    ServiceError::new(ServiceErrorKind::InvalidInput, message)
}
fn process_file(file: RepositoryFileInput, actions: &[WorkflowAction]) -> ServiceResult<(Vec<FileChange>, WorkflowFileReport)> {
    file.format()
        .ok_or_else(|| {
            ServiceError::new(
                ServiceErrorKind::Unsupported,
                format!("Unsupported repository file format for '{}'", file.path),
            )
        })
        .and_then(|format| {
            format
                .normalize(&file.content, actions)
                .map_err(|error| error.with_path(file.path.clone()))
                .map(|normalized| (format, normalized))
        })
        .map(|(format, normalized)| {
            let changed = normalized.content != file.content;
            let source = (changed && actions.contains(&WorkflowAction::Format)).then(|| FileChange {
                content: normalized.content,
                kind: FileChangeKind::Update,
                path: file.path.clone(),
            });
            let linked = normalized.linked.map(|content| FileChange {
                content,
                kind: FileChangeKind::Create,
                path: file
                    .path
                    .rsplit_once('.')
                    .map_or_else(|| format!("{}.jsonld", file.path), |(stem, _)| format!("{stem}.jsonld")),
            });
            (
                source.into_iter().chain(linked).collect(),
                WorkflowFileReport {
                    changed,
                    format,
                    path: file.path,
                },
            )
        })
}
pub(crate) fn process_repository(mut request: ProcessRepositoryRequest) -> ServiceResult<ProcessRepositoryResponse> {
    validate_request(&request).and_then(|()| {
        request.files.sort_by(|left, right| left.path.cmp(&right.path));
        request
            .files
            .into_iter()
            .map(|file| process_file(file, &request.actions))
            .collect::<ServiceResult<Vec<_>>>()
            .map(|processed| ProcessRepositoryResponse {
                base_revision: request.revision,
                files: processed.iter().flat_map(|(changes, _)| changes.clone()).collect(),
                reports: processed.into_iter().map(|(_, report)| report).collect(),
            })
    })
}
fn validate_request(request: &ProcessRepositoryRequest) -> ServiceResult<()> {
    let mut paths = request.files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>();
    paths.sort_unstable();
    let duplicate_path = paths.windows(2).any(|pair| matches!(pair, [left, right] if left == right));
    match (
        request.project.trim().is_empty(),
        request.revision.trim().is_empty(),
        request.actions.contains(&WorkflowAction::Enrich),
        duplicate_path,
    ) {
        | (true, _, _, _) => Err(ServiceError::new(ServiceErrorKind::InvalidInput, "Repository project identifier is required").with_path("project")),
        | (false, true, _, _) => Err(ServiceError::new(ServiceErrorKind::InvalidInput, "Repository revision is required").with_path("revision")),
        | (false, false, true, _) => Err(ServiceError::new(
            ServiceErrorKind::PolicyDenied,
            "Repository enrichment requires an explicitly configured capability",
        )),
        | (false, false, false, true) => {
            Err(ServiceError::new(ServiceErrorKind::InvalidInput, "Repository file paths must be unique").with_path("files"))
        }
        | (false, false, false, false) => request.files.iter().try_for_each(|file| {
            let location = Location::from(&file.path);
            location.is_canonical_repository_path().then_some(()).ok_or_else(|| {
                ServiceError::new(
                    ServiceErrorKind::InvalidInput,
                    "Repository path must be relative and use canonical forward-slash form",
                )
                .with_path(file.path.clone())
            })
        }),
    }
}
