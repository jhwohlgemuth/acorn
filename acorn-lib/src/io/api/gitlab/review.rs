//! Merge request analysis workflow
use super::{
    merge_request, merge_request_diffs, publish_commit_status, repository_file, upsert_merge_request_note, CommitStatusState, MergeRequestDetails,
    MergeRequestDiff, Options,
};
use crate::analyzer::discovery::discover_identifiers;
use crate::analyzer::{analyze_paths, Check, CheckOptions};
use crate::io::api::RepositoryFileMetadata;
use crate::io::ApiResult;
use crate::prelude::{create_dir_all, env, remove_dir_all, write, PathBuf};
use crate::schema::pid::Identifier;
use crate::util::constants::app::{APPLICATION, MAX_ANALYSIS_FILE_BYTES, MERGE_REQUEST_REPORT_MARKER};
use crate::util::{generate_guid, MimeType, ToMarkdown};
use color_eyre::eyre::{eyre, Report};
use core::iter::once;
use futures::stream::{self, StreamExt};
use serde::Serialize;

/// One merge request analysis input
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum Input {
    /// Supported input awaiting retrieval
    Pending {
        /// Repository-relative path
        path: String,
    },
    /// Materialized input included in analysis
    Checked {
        /// Repository-relative path
        path: String,
        /// Temporary local path
        local_path: PathBuf,
        /// UTF-8 text used for identifier discovery
        text: Option<String>,
    },
    /// Input intentionally excluded from analysis
    Skipped {
        /// Repository-relative path
        path: String,
        /// Reason the input was not analyzed
        reason: String,
    },
    /// Supported input that could not be analyzed
    Failed {
        /// Repository-relative path
        path: String,
        /// Failure description
        reason: String,
        /// Whether durable operation retry can recover the failure
        retryable: bool,
    },
}
impl Input {
    fn path(&self) -> &str {
        match self {
            | Self::Pending { path } | Self::Checked { path, .. } | Self::Skipped { path, .. } | Self::Failed { path, .. } => path,
        }
    }
    fn is_skipped(&self) -> bool {
        matches!(self, Self::Skipped { .. })
    }
    fn supported(path: &str) -> bool {
        match MimeType::from(path) {
            | MimeType::Cff | MimeType::Json | MimeType::Jsonc | MimeType::Markdown | MimeType::Text | MimeType::Yaml => true,
            | MimeType::Vendor(extension) => extension == "docx",
            | _ => false,
        }
    }
}
impl From<&MergeRequestDiff> for Input {
    fn from(diff: &MergeRequestDiff) -> Self {
        let reason = if diff.deleted_file {
            Some("deleted file")
        } else if diff.generated_file {
            Some("generated file")
        } else if !Self::supported(&diff.new_path) {
            Some("unsupported file type")
        } else {
            None
        };
        reason.map_or_else(
            || Self::Pending { path: diff.new_path.clone() },
            |reason| Self::Skipped {
                path: diff.new_path.clone(),
                reason: reason.to_string(),
            },
        )
    }
}
/// Structured merge request analysis result
#[derive(Clone, Debug)]
pub struct MergeRequestAnalysisReport {
    /// Analyzed merge request head SHA
    pub head_sha: String,
    /// Inputs considered for analysis
    pub inputs: Vec<Input>,
    /// Structured ACORN checks
    pub checks: Vec<Check>,
    /// Artifact identifiers discovered in analyzed text
    pub citation_candidates: Vec<Identifier>,
}
impl MergeRequestAnalysisReport {
    /// Whether the final commit status must fail
    pub fn failed(&self) -> bool {
        self.inputs.iter().any(|input| matches!(input, Input::Failed { .. })) || self.checks.iter().any(Check::is_failure)
    }
    /// Whether a durable operation retry may recover an input failure
    pub fn requires_retry(&self) -> bool {
        self.inputs.iter().any(|input| matches!(input, Input::Failed { retryable: true, .. }))
    }
    /// Render the idempotent merge request report note
    pub fn render(&self) -> String {
        let status = if self.failed() { "Failed" } else { "Passed" };
        let checked_files = self
            .inputs
            .iter()
            .filter(|input| matches!(input, Input::Checked { .. }))
            .map(Input::path)
            .map(|path| format!("`{path}`"))
            .collect::<Vec<_>>();
        let checked = if checked_files.is_empty() {
            "- Merge request title and description".to_string()
        } else {
            once("- Merge request title and description".to_string())
                .chain(checked_files.to_markdown().trim_start().lines().map(str::to_string))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let issues = self
            .checks
            .iter()
            .filter(|check| !check.success)
            .map(|check| {
                let Check {
                    uri,
                    locator,
                    context,
                    message,
                    severity,
                    category,
                    ..
                } = check;
                let location = uri.as_deref().or(locator.as_deref()).unwrap_or("merge request");
                let detail = context.as_deref().filter(|value| !value.trim().is_empty()).unwrap_or(message.as_str());
                format!("- **{severity} / {category}** `{location}` — {detail}")
            })
            .collect::<Vec<_>>();
        let issues = if issues.is_empty() {
            "- No reported issues".to_string()
        } else {
            issues.join("\n")
        };
        let candidates = if self.citation_candidates.is_empty() {
            "- None detected".to_string()
        } else {
            self.citation_candidates
                .iter()
                .map(|identifier| {
                    let kind: &str = identifier.into();
                    format!("- `{kind}`: `{}`", identifier.value)
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        let skipped_inputs = self
            .inputs
            .iter()
            .filter_map(|input| match input {
                | Input::Skipped { path, reason } => Some(format!("- `{path}` — {reason}")),
                | _ => None,
            })
            .collect::<Vec<_>>();
        let skipped = if skipped_inputs.is_empty() {
            "- None".to_string()
        } else {
            skipped_inputs.join("\n")
        };
        let failed_inputs = self
            .inputs
            .iter()
            .filter_map(|input| match input {
                | Input::Failed { path, reason, .. } => Some(format!("- `{path}` — {reason}")),
                | _ => None,
            })
            .collect::<Vec<_>>();
        let failures = if failed_inputs.is_empty() {
            "- None".to_string()
        } else {
            failed_inputs.join("\n")
        };
        let application = APPLICATION.to_ascii_uppercase();
        format!(
            "{MERGE_REQUEST_REPORT_MARKER}\n## {application} merge request analysis\n\n**Status:** {status}  \n**Analyzed SHA:** `{}`\n\n### Checked content\n{checked}\n\n### Checks\n{issues}\n\n### Citation candidates\n{candidates}\n\n### Skipped inputs\n{skipped}\n\n### Input failures\n{failures}",
            self.head_sha
        )
    }
}
/// Result of processing one head-specific merge request operation
#[derive(Clone, Debug)]
pub enum MergeRequestAnalysisOutcome {
    /// The queued SHA is no longer the merge request head
    Stale {
        /// Queued SHA
        queued_sha: String,
        /// Current SHA
        current_sha: String,
    },
    /// Analysis was published to GitLab
    Published(MergeRequestAnalysisReport),
}
struct Workspace {
    root: PathBuf,
}
impl Workspace {
    fn create(iid: u64, head_sha: &str) -> ApiResult<Self> {
        let short_sha = head_sha.chars().take(12).collect::<String>();
        let root = env::temp_dir().join(format!("acorn-mr-{iid}-{short_sha}-{}", generate_guid()));
        create_dir_all(&root)
            .map(|_| Self { root: root.clone() })
            .map_err(|why| eyre!("Failed to create merge request analysis workspace {} — {why}", root.display()))
    }
    fn write(&self, name: &str, content: &[u8]) -> ApiResult<PathBuf> {
        let path = self.root.join(name);
        write(&path, content)
            .map(|_| path.clone())
            .map_err(|why| eyre!("Failed to materialize merge request input {} — {why}", path.display()))
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        if let Err(why) = remove_dir_all(&self.root) {
            tracing::warn!("Failed to remove merge request analysis workspace {} — {why}", self.root.display());
        }
    }
}
/// Analyze one merge request head and publish its note and commit status
pub async fn analyze_merge_request(options: &Options, check_options: &CheckOptions) -> ApiResult<MergeRequestAnalysisOutcome> {
    match options.sha() {
        | Ok(sha) => match merge_request(options).await {
            | Ok(details) if details.sha != sha => Ok(MergeRequestAnalysisOutcome::Stale {
                queued_sha: sha.to_string(),
                current_sha: details.sha,
            }),
            | Ok(details) => {
                let source_project_id = details.source_project_id.unwrap_or(details.project_id);
                match publish_commit_status(
                    options,
                    Some(source_project_id),
                    sha,
                    CommitStatusState::Running,
                    &format!("{} merge request analysis is running", APPLICATION.to_ascii_uppercase()),
                    Some(&details.web_url),
                )
                .await
                {
                    | Ok(_) => match build_report(options, &details, sha, check_options).await {
                        | Ok(report) => publish_report(options, &details, report).await,
                        | Err(why) => fail_workflow(options, Some(source_project_id), sha, Some(&details.web_url), why).await,
                    },
                    | Err(why) => Err(why),
                }
            }
            | Err(why) => fail_workflow(options, None, sha, None, why).await,
        },
        | Err(why) => Err(why),
    }
}
async fn build_report(
    options: &Options,
    details: &MergeRequestDetails,
    head_sha: &str,
    check_options: &CheckOptions,
) -> ApiResult<MergeRequestAnalysisReport> {
    match merge_request_diffs(options).await {
        | Ok(diffs) => match Workspace::create(details.iid, head_sha) {
            | Ok(workspace) => {
                let merge_request_text = format!("# {}\n\n{}", details.title, details.description);
                match workspace.write("merge-request.md", merge_request_text.as_bytes()) {
                    | Ok(merge_request_path) => {
                        let classified_inputs = diffs.iter().map(Input::from).collect::<Vec<_>>();
                        let source_project_id = details.source_project_id.unwrap_or(details.project_id);
                        let fetches = diffs
                            .into_iter()
                            .filter(|diff| !Input::from(diff).is_skipped())
                            .enumerate()
                            .map(|(index, diff)| fetch_input(options, source_project_id, head_sha, &workspace, index, diff));
                        let fetched = stream::iter(fetches).buffered(8).collect::<Vec<_>>().await;
                        let paths = once(merge_request_path)
                            .chain(fetched.iter().filter_map(|input| match input {
                                | Input::Checked { local_path, .. } => Some(local_path.clone()),
                                | _ => None,
                            }))
                            .collect::<Vec<_>>();
                        let candidate_text = once(merge_request_text)
                            .chain(fetched.iter().filter_map(|input| match input {
                                | Input::Checked { text, .. } => text.clone(),
                                | _ => None,
                            }))
                            .collect::<Vec<_>>()
                            .join("\n");
                        let citation_candidates = discover_identifiers(&candidate_text);
                        let checks = analyze_paths(&paths, check_options).await.checks();
                        let inputs = classified_inputs.into_iter().filter(Input::is_skipped).chain(fetched).collect();
                        Ok(MergeRequestAnalysisReport {
                            head_sha: head_sha.to_string(),
                            inputs,
                            checks,
                            citation_candidates,
                        })
                    }
                    | Err(why) => Err(why),
                }
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
async fn fetch_input(
    options: &Options,
    source_project_id: u64,
    head_sha: &str,
    workspace: &Workspace,
    index: usize,
    diff: MergeRequestDiff,
) -> Input {
    match repository_file(options, source_project_id, &diff.new_path, head_sha).await {
        | Ok(file) if file.size().is_some_and(|size| size > MAX_ANALYSIS_FILE_BYTES) => Input::Failed {
            path: diff.new_path,
            reason: format!("file exceeds the {MAX_ANALYSIS_FILE_BYTES}-byte analysis limit"),
            retryable: false,
        },
        | Ok(file) => match file.decoded_content() {
            | Ok(content) => {
                let extension = PathBuf::from(&diff.new_path)
                    .extension()
                    .and_then(|value| value.to_str())
                    .map(str::to_ascii_lowercase)
                    .unwrap_or_else(|| "txt".to_string());
                let name = format!("input-{index:04}.{extension}");
                match workspace.write(&name, &content) {
                    | Ok(local_path) => Input::Checked {
                        path: diff.new_path,
                        local_path,
                        text: String::from_utf8(content).ok(),
                    },
                    | Err(why) => Input::Failed {
                        path: diff.new_path,
                        reason: why.to_string(),
                        retryable: true,
                    },
                }
            }
            | Err(why) => Input::Failed {
                path: diff.new_path,
                reason: why.to_string(),
                retryable: true,
            },
        },
        | Err(why) => Input::Failed {
            path: diff.new_path,
            reason: format!("failed to fetch content at `{head_sha}` — {why}"),
            retryable: true,
        },
    }
}
async fn publish_report(
    options: &Options,
    details: &MergeRequestDetails,
    report: MergeRequestAnalysisReport,
) -> ApiResult<MergeRequestAnalysisOutcome> {
    let state = CommitStatusState::from(!report.failed());
    let application = APPLICATION.to_ascii_uppercase();
    let description = if report.failed() {
        format!("{application} found merge request analysis failures")
    } else {
        format!("{application} merge request analysis passed")
    };
    match upsert_merge_request_note(options, MERGE_REQUEST_REPORT_MARKER, &report.render()).await {
        | Ok(_) => match publish_commit_status(
            options,
            Some(details.source_project_id.unwrap_or(details.project_id)),
            &report.head_sha,
            state,
            &description,
            Some(&details.web_url),
        )
        .await
        {
            | Ok(_) if report.requires_retry() => Err(eyre!("One or more merge request inputs could not be fetched or materialized")),
            | Ok(_) => Ok(MergeRequestAnalysisOutcome::Published(report)),
            | Err(why) => Err(why),
        },
        | Err(why) => {
            fail_workflow(
                options,
                Some(details.source_project_id.unwrap_or(details.project_id)),
                &report.head_sha,
                Some(&details.web_url),
                why,
            )
            .await
        }
    }
}
async fn fail_workflow(
    options: &Options,
    project_id: Option<u64>,
    head_sha: &str,
    target_url: Option<&str>,
    why: Report,
) -> ApiResult<MergeRequestAnalysisOutcome> {
    let description = why.to_string().chars().take(255).collect::<String>();
    match publish_commit_status(options, project_id, head_sha, CommitStatusState::Failed, &description, target_url).await {
        | Ok(_) => Err(why),
        | Err(status_error) => Err(eyre!("{why}; failed to publish terminal acorn/check status — {status_error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classifies_supported_and_skipped_inputs() {
        let deleted = MergeRequestDiff {
            old_path: "old.json".to_string(),
            new_path: "old.json".to_string(),
            new_file: false,
            renamed_file: false,
            deleted_file: true,
            generated_file: false,
            collapsed: false,
            too_large: false,
        };
        let unsupported = MergeRequestDiff {
            old_path: "main.rs".to_string(),
            new_path: "main.rs".to_string(),
            deleted_file: false,
            ..deleted.clone()
        };
        assert!(matches!(Input::from(&deleted), Input::Skipped { reason, .. } if reason == "deleted file"));
        assert!(matches!(Input::from(&unsupported), Input::Skipped { reason, .. } if reason == "unsupported file type"));
        assert!(Input::supported("metadata/CITATION.cff"));
        assert!(Input::supported("activity.JSONC"));
    }
}
