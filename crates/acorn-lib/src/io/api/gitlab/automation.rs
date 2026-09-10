//! Typed GitLab repository publication for registered workflows
use super::{merge_request, merge_request_diffs, repository_file, ErrorResponse, Fallback, MergeRequestDetails, Options};
use crate::io::api::{self, require_non_empty_secret, Configuration, EmptyField, Endpoint, Params, RemoteResource};
use crate::io::workflow::{FileChangeKind, RepositoryChangeSet, RepositoryFileInput, RepositoryInput, RepositoryWorkflowRegistry, WorkflowAction};
use crate::io::ApiResult;
use crate::util::constants::app::APPLICATION;
use crate::util::constants::env::GITLAB_TOKEN_VARIABLE_NAMES;
use acorn_core::prelude::{String, Vec};
use acorn_core::util::{MarkdownSupport, MimeType};
use color_eyre::eyre::eyre;
use futures::stream::{self, StreamExt, TryStreamExt};
use secrecy::ExposeSecret;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Result of processing and optionally publishing a merge-request repository snapshot.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AutomationResult {
    /// Exact merge-request head used as workflow input.
    pub revision: String,
    /// Publication details when the workflow proposed changes.
    pub publication: Option<PublishResult>,
}
/// Minimal GitLab branch response.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Branch {
    /// Branch head commit.
    pub commit: BranchCommit,
    /// Branch name.
    pub name: String,
}
/// Minimal commit nested in a branch response.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BranchCommit {
    /// Commit SHA.
    pub id: String,
}
#[derive(Serialize)]
struct BranchRequest<'a> {
    branch: &'a str,
    #[serde(rename = "ref")]
    reference: &'a str,
}
/// Minimal GitLab commit response.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Commit {
    /// Commit SHA.
    pub id: String,
    /// Browser URL when returned by GitLab.
    pub web_url: Option<String>,
}
#[derive(Serialize)]
struct CommitAction<'a> {
    action: &'static str,
    content: &'a str,
    encoding: &'static str,
    file_path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_commit_id: Option<&'a str>,
}
#[derive(Serialize)]
struct CommitRequest<'a> {
    actions: Vec<CommitAction<'a>>,
    branch: &'a str,
    commit_message: &'a str,
}
/// Minimal GitLab merge-request response used by automation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MergeRequest {
    /// Project-scoped merge-request identifier.
    pub iid: u64,
    /// Source branch name.
    pub source_branch: String,
    /// Merge-request state.
    pub state: String,
    /// Browser URL.
    pub web_url: String,
}
#[derive(Serialize)]
struct MergeRequestRequest<'a> {
    description: &'a str,
    remove_source_branch: bool,
    source_branch: &'a str,
    target_branch: &'a str,
    title: &'a str,
}
/// Result of publishing a repository workflow change set.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PublishResult {
    /// Bot branch containing the changes.
    pub branch: String,
    /// Commit created for the change set.
    pub commit: Commit,
    /// Draft merge request created or reused for review.
    pub merge_request: MergeRequest,
}
impl MergeRequestDetails {
    /// Process supported flat files at this merge-request head and publish a draft correction merge request.
    pub async fn process(self, workflow: &str, options: &Options) -> ApiResult<AutomationResult> {
        let details = self;
        match details.source_project_id.is_some_and(|source| source != details.project_id) {
            | true => Err(eyre!(
                "Repository automation for fork merge requests requires cross-project branch publication support"
            )),
            | false => {
                let revision = details.sha.clone();
                match merge_request_diffs(options).await {
                    | Ok(diffs) => {
                        let mut paths = diffs
                            .into_iter()
                            .filter(|diff| !diff.deleted_file && !diff.generated_file && MimeType::from(diff.new_path.as_str()).is_repository_file())
                            .map(|diff| diff.new_path)
                            .collect::<Vec<_>>();
                        paths.sort();
                        paths.dedup();
                        match options.repository_files(&revision, paths).await {
                            | Ok(files) => match options.identifier().map(str::to_string) {
                                | Some(project) => {
                                    let input = RepositoryInput {
                                        actions: vec![WorkflowAction::Format, WorkflowAction::Link, WorkflowAction::Validate],
                                        files,
                                        project,
                                        revision: revision.clone(),
                                    };
                                    match RepositoryWorkflowRegistry::acorn().and_then(|registry| registry.process(workflow, input)) {
                                        | Ok(change_set) if change_set.files.is_empty() => Ok(AutomationResult { publication: None, revision }),
                                        | Ok(change_set) => match merge_request(options).await {
                                            | Ok(current) if current.sha != revision => Err(eyre!(
                                                "GitLab merge request head changed from {revision} to {} before publication",
                                                current.sha
                                            )),
                                            | Ok(_) => {
                                                let target_branch = match details.source_branch.trim().is_empty() {
                                                    | true => details.target_branch.as_str(),
                                                    | false => details.source_branch.as_str(),
                                                };
                                                match target_branch.trim().is_empty() {
                                                    | true => Err(eyre!("GitLab merge request did not report a source or target branch")),
                                                    | false => change_set.publish(workflow, target_branch, options).await.map(|publication| {
                                                        AutomationResult {
                                                            publication: Some(publication),
                                                            revision,
                                                        }
                                                    }),
                                                }
                                            }
                                            | Err(why) => Err(why),
                                        },
                                        | Err(why) => Err(why),
                                    }
                                }
                                | None => Err(eyre!("GitLab project identifier is required")),
                            },
                            | Err(why) => Err(why),
                        }
                    }
                    | Err(why) => Err(why),
                }
            }
        }
    }
}
impl Options {
    /// Resolve one GitLab branch to its exact head commit SHA.
    pub async fn branch_head(&self, branch: &str) -> ApiResult<String> {
        self.find_branch(branch).await.map(|branch| branch.commit.id)
    }
    async fn ensure_branch(&self, branch: &str, reference: &str) -> ApiResult<Branch> {
        let request = BranchRequest { branch, reference };
        match serde_json::to_value(request) {
            | Ok(body) => match self.invoke_body("branch::create", body).await {
                | Ok(branch) => Ok(branch),
                | Err(_) => self.find_branch(branch).await,
            },
            | Err(why) => Err(eyre!("Failed to encode GitLab branch request — {why}")),
        }
    }
    async fn find_branch(&self, branch: &str) -> ApiResult<Branch> {
        let encoded = urlencoding::encode(branch);
        self.invoke("branch", Params::new().with_template("branch", Some(encoded.as_ref())).build())
            .await
    }
    async fn find_open_merge_request(&self, source_branch: &str) -> ApiResult<Option<MergeRequest>> {
        self.invoke::<Vec<MergeRequest>>(
            "merge-request::list",
            Params::new()
                .with_keyvalue("source_branch", Some(source_branch))
                .with_keyvalue("state", Some("opened"))
                .build(),
        )
        .await
        .map(|merge_requests| {
            merge_requests
                .into_iter()
                .find(|merge_request| merge_request.source_branch == source_branch)
        })
    }
    async fn invoke<T: DeserializeOwned>(&self, action: &str, custom: Vec<api::Param>) -> ApiResult<T> {
        let path = format!("gitlab::api::{action}");
        match require_non_empty_secret(&self.token, &path, &GITLAB_TOKEN_VARIABLE_NAMES) {
            | Ok(token) => match Endpoint::from_template("gitlab::api").map(|endpoint| endpoint.with_domain(self.domain())) {
                | Ok(endpoint) => {
                    let params = Params::new()
                        .with_auth(ExposeSecret::expose_secret(&token), Some(self.token_header()))
                        .with_template("identifier", self.identifier())
                        .with_custom(&custom)
                        .build();
                    let response = endpoint.invoke_with::<EmptyField, EmptyField>(action, Some(params)).await;
                    endpoint.handle_or::<T, Fallback<ErrorResponse>>(response)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    async fn invoke_body<T: DeserializeOwned>(&self, action: &str, body: serde_json::Value) -> ApiResult<T> {
        match serde_json::to_string(&body) {
            | Ok(body) => self.invoke(action, Params::new().with_body("", &body).build()).await,
            | Err(why) => Err(eyre!("Failed to encode GitLab request body — {why}")),
        }
    }
    async fn repository_files(&self, revision: &str, paths: Vec<String>) -> ApiResult<Vec<RepositoryFileInput>> {
        stream::iter(paths)
            .then(|path| async move {
                repository_file(&self.clone().with_path(&path).with_sha(revision))
                    .await
                    .and_then(|file| {
                        file.decoded_content().and_then(|content| {
                            String::from_utf8(content).map_err(|why| eyre!("GitLab repository file '{path}' is not UTF-8 — {why}"))
                        })
                    })
                    .map(|content| RepositoryFileInput { content, format: None, path })
            })
            .try_collect()
            .await
    }
}
impl RepositoryChangeSet {
    fn branch_name(&self, workflow: &str) -> String {
        let workflow = workflow
            .chars()
            .map(|character| match character.is_ascii_alphanumeric() || character == '-' {
                | true => character.to_ascii_lowercase(),
                | false => '-',
            })
            .collect::<String>();
        let revision = self
            .base_revision
            .chars()
            .filter(char::is_ascii_hexdigit)
            .take(12)
            .map(|character| character.to_ascii_lowercase())
            .collect::<String>();
        format!("{APPLICATION}/{workflow}/{revision}")
    }
    fn commit_request<'a>(&'a self, branch: &'a str, message: &'a str) -> CommitRequest<'a> {
        CommitRequest {
            actions: self
                .files
                .iter()
                .map(|change| CommitAction {
                    action: change.kind.into(),
                    content: &change.content,
                    encoding: "text",
                    file_path: &change.path,
                    last_commit_id: matches!(change.kind, FileChangeKind::Update).then_some(self.base_revision.as_str()),
                })
                .collect(),
            branch,
            commit_message: message,
        }
    }
    /// Publish this deterministic change set to a bot branch and draft merge request.
    pub async fn publish(&self, workflow: &str, target_branch: &str, options: &Options) -> ApiResult<PublishResult> {
        match self.files.is_empty() {
            | true => Err(eyre!("Repository workflow produced no file changes")),
            | false => {
                let branch_name = self.branch_name(workflow);
                match options.ensure_branch(&branch_name, &self.base_revision).await {
                    | Ok(branch) => {
                        let commit = match branch.commit.id == self.base_revision {
                            | true => {
                                let message = format!("Apply {} workflow {workflow}", APPLICATION.to_ascii_uppercase());
                                let request = self.commit_request(&branch_name, &message);
                                match serde_json::to_value(request) {
                                    | Ok(body) => options.invoke_body::<Commit>("commits::create", body).await,
                                    | Err(why) => Err(eyre!("Failed to encode GitLab commit request — {why}")),
                                }
                            }
                            | false => Ok(Commit {
                                id: branch.commit.id,
                                web_url: None,
                            }),
                        };
                        match commit {
                            | Ok(commit) => match options.find_open_merge_request(&branch_name).await {
                                | Ok(Some(merge_request)) => Ok(PublishResult {
                                    branch: branch_name,
                                    commit,
                                    merge_request,
                                }),
                                | Ok(None) => {
                                    let title = format!("Draft: Apply {} {workflow}", APPLICATION.to_ascii_uppercase());
                                    let description = self.render_description(workflow);
                                    let request = MergeRequestRequest {
                                        description: &description,
                                        remove_source_branch: true,
                                        source_branch: &branch_name,
                                        target_branch,
                                        title: &title,
                                    };
                                    match serde_json::to_value(request) {
                                        | Ok(body) => options
                                            .invoke_body("merge-request::create", body)
                                            .await
                                            .map(|merge_request| PublishResult {
                                                branch: branch_name,
                                                commit,
                                                merge_request,
                                            }),
                                        | Err(why) => Err(eyre!("Failed to encode GitLab merge request — {why}")),
                                    }
                                }
                                | Err(why) => Err(why),
                            },
                            | Err(why) => Err(why),
                        }
                    }
                    | Err(why) => Err(why),
                }
            }
        }
    }
    fn render_description(&self, workflow: &str) -> String {
        let files = self.to_markdown();
        format!(
            "<!-- {APPLICATION}:repository-workflow -->\n## {} repository workflow\n\nWorkflow: `{workflow}`  \nBase revision: `{}`\n\n### Proposed files\n{files}",
            APPLICATION.to_ascii_uppercase(),
            self.base_revision
        )
    }
}
