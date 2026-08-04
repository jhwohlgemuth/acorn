//! GitLab API service operations.

use super::{
    eyre, first_env_var, require_non_empty_secret, ApiResult, BoxFuture, CommitStatus, CommitStatusState, Configuration, EmptyField, Endpoint,
    ErrorResponse, EventFilterKey, EventsResponse, Fallback, FutureExt, GitLabIdentity, GroupsResponse, InstanceVersion, MergeRequestDetails,
    MergeRequestDiffsResponse, Note, NoteMetadata, NotesResponse, Options, PaginationKey, Params, ProgrammingLanguageUseResponse,
    ProgrammingLanguagesResponse, ProjectMember, ProjectWebhook, ProjectWebhookRequest, ProjectWebhooksResponse, RemoteResource, RepositoryFile,
    ResponseContent, RunnerCreationResponse, RunnerMetadata, RunnersResponse, Searchable, TreeResponse, Validate, WebhookOptions,
    WebhookRegistration, WorkItem, GITLAB_TOKEN_VARIABLE_NAMES, INCLUDED_ENDPOINTS,
};
use serde::Deserialize;

pub(crate) fn handle_tree_paths_response(response: ApiResult<TreeResponse>, page: u32) -> ApiResult<TreeResponse> {
    match response {
        | Ok(value) => match value.error {
            | Some(why) if page > 1 && why.is_terminal_pagination_error() => Ok(TreeResponse::default()),
            | Some(why) => Err(eyre!(why.message())),
            | None => Ok(value),
        },
        | Err(why) => Err(why),
    }
}
/// Create a new runner using project or group identifier
///
/// See <https://docs.gitlab.com/api/users/#create-a-runner-linked-to-a-user> for more information on the GitLab API
pub async fn create_runner(options: &Options) -> ApiResult<RunnerCreationResponse> {
    #[derive(Deserialize)]
    struct StrictRunnerCreationResponse {
        #[serde(rename = "id")]
        identifier: u64,
        token: Option<String>,
        token_expires_at: Option<String>,
    }
    let template = "gitlab::api";
    let action = "runner::create";
    let path = format!("{template}::{action}");
    let runner_metadata = &options.runner_metadata;
    let runner_type = &runner_metadata.runner_type;
    let description = runner_metadata.description.as_deref().unwrap_or_default();
    let tags = runner_metadata.tags.as_deref().unwrap_or_default();
    let run_untagged = runner_metadata.run_untagged;
    let tag_list = if tags.is_empty() { None } else { Some(tags.join(",")) };
    match require_non_empty_secret(&options.token, &path, &GITLAB_TOKEN_VARIABLE_NAMES) {
        | Ok(token) => match Endpoint::from_template(template).map(|e| e.with_domain(options.domain())) {
            | Ok(endpoint) => {
                let params = Params::new()
                    .with_auth(&token, Some("PRIVATE-TOKEN"))
                    .with_body("description", description)
                    .with_body(&format!("{runner_type}_id"), options.identifier.as_deref().unwrap_or_default())
                    .with_body("runner_type", &format!("{runner_type}_type"))
                    .with_body("run_untagged", &run_untagged.to_string())
                    .with_body_maybe("tag_list", tag_list.as_deref())
                    .with_custom(options.params())
                    .build();
                let response = endpoint.invoke(action, Some(params)).await;
                match response {
                    | Ok(ResponseContent::Json(content)) => match serde_json::from_str::<StrictRunnerCreationResponse>(&content) {
                        | Ok(parsed) => Ok(RunnerCreationResponse {
                            identifier: parsed.identifier,
                            token: parsed.token,
                            token_expires_at: parsed.token_expires_at,
                        }),
                        | Err(_) => {
                            let rendered = serde_json::from_str::<serde_json::Value>(&content)
                                .ok()
                                .and_then(|value| serde_json::to_string_pretty(&value).ok())
                                .unwrap_or(content);
                            Err(eyre!("{rendered}"))
                        }
                    },
                    | Ok(other) => endpoint.handle::<RunnerCreationResponse>(Ok(other)),
                    | Err(why) => Err(why),
                }
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
/// Get events for a project
/// ### Example
///
/// ```ignore
/// use acorn::io::api::{gitlab, Configuration};
/// use acorn::param;
///
/// let project_id = "16689";
/// let options = gitlab::Options::from_env()
///     .with_identifier(project_id)
///     .with_params(vec![
///         param!(KeyValuePair, "target_type", "note"),
///         param!(KeyValuePair, "after", "2026-06-27"),
///         // 'before' value is invalid and will not be including in final URL
///         param!(KeyValuePair, "before", "2026-##-04"),
///     ]);
/// let events = gitlab::events(&options).await;
/// ```
///
/// See <https://docs.gitlab.com/api/events/#list-all-visible-events-for-a-project> for more information
pub async fn events(options: &Options) -> ApiResult<EventsResponse> {
    let template = "gitlab::api";
    let action = "events";
    let path = format!("{template}::{action}");
    match require_non_empty_secret(&options.token, &path, &GITLAB_TOKEN_VARIABLE_NAMES) {
        | Ok(token) => match Endpoint::from_template(template).map(|e| e.with_domain(options.domain())) {
            | Ok(endpoint) => {
                let params = Params::new()
                    .with_auth(&token, Some("PRIVATE-TOKEN"))
                    .with_template("identifier", options.identifier())
                    .with_custom(options.params())
                    .build();
                let response = endpoint.invoke_with::<EventFilterKey, EmptyField>(action, Some(params)).await;
                endpoint.handle::<EventsResponse>(response)
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
/// Get the GitLab instance version
pub async fn version(options: &Options) -> ApiResult<InstanceVersion> {
    let template = "gitlab::api";
    let action = "version";
    let path = format!("{template}::{action}");
    match require_non_empty_secret(&options.token, &path, &GITLAB_TOKEN_VARIABLE_NAMES) {
        | Ok(token) => match Endpoint::from_template(template).map(|endpoint| endpoint.with_domain(options.domain())) {
            | Ok(endpoint) => {
                let params = Params::new().with_auth(&token, Some("PRIVATE-TOKEN")).build();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle::<InstanceVersion>(response)
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
/// List all project webhooks for the configured project
pub async fn project_webhooks(options: &Options) -> ApiResult<ProjectWebhooksResponse> {
    project_webhooks_page(options).await
}
fn project_webhooks_page(options: &Options) -> BoxFuture<'_, ApiResult<ProjectWebhooksResponse>> {
    async move {
        let template = "gitlab::api";
        let action = "hooks";
        let path = format!("{template}::{action}");
        let page = options.page;
        match require_non_empty_secret(&options.token, &path, &GITLAB_TOKEN_VARIABLE_NAMES) {
            | Ok(token) => match Endpoint::from_template(template).map(|endpoint| endpoint.with_domain(options.domain())) {
                | Ok(endpoint) => {
                    let params = Params::new()
                        .with_auth(&token, Some("PRIVATE-TOKEN"))
                        .with_template("identifier", options.identifier())
                        .with_keyvalue("page", Some(&page.to_string()))
                        .with_keyvalue("per_page", Some("100"))
                        .build();
                    let response = endpoint.invoke_with::<PaginationKey, EmptyField>(action, Some(params)).await;
                    match endpoint.handle::<ProjectWebhooksResponse>(response) {
                        | Ok(hooks) if hooks.len() == 100 => match project_webhooks_page(&options.clone().with_page(page.saturating_add(1))).await {
                            | Ok(remaining) => Ok(hooks.into_iter().chain(remaining).collect()),
                            | Err(why) => Err(why),
                        },
                        | result => result,
                    }
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    .boxed()
}
/// Create or refresh the ACORN merge request and note webhook
pub async fn upsert_project_webhook(options: &Options, webhook: &WebhookOptions) -> ApiResult<WebhookRegistration> {
    match webhook.validate() {
        | Ok(()) => match version(options).await {
            | Ok(version) => {
                let registration = webhook.for_registration(version.supports_signing_tokens());
                match registration.public_url.as_deref() {
                    | Some(endpoint_url) => match project_webhooks(options).await {
                        | Ok(hooks) => match hooks.into_iter().find(|hook| hook.url.trim_end_matches('/') == endpoint_url) {
                            | Some(hook) => write_project_webhook(options, "hook::update", Some(hook.id), &registration)
                                .await
                                .map(|hook| WebhookRegistration { hook, created: false }),
                            | None => write_project_webhook(options, "hook::create", None, &registration)
                                .await
                                .map(|hook| WebhookRegistration { hook, created: true }),
                        },
                        | Err(why) => Err(why),
                    },
                    | None => Err(eyre!("Webhook mode requires --public-url")),
                }
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(eyre!(why)),
    }
}
async fn write_project_webhook(options: &Options, action: &str, hook_id: Option<u64>, webhook: &WebhookOptions) -> ApiResult<ProjectWebhook> {
    let template = "gitlab::api";
    let path = format!("{template}::{action}");
    match require_non_empty_secret(&options.token, &path, &GITLAB_TOKEN_VARIABLE_NAMES) {
        | Ok(token) => match Endpoint::from_template(template).map(|endpoint| endpoint.with_domain(options.domain())) {
            | Ok(endpoint) => {
                let request = ProjectWebhookRequest {
                    url: webhook.public_url.as_deref().unwrap_or_default(),
                    name: "ACORN bot",
                    description: "ACORN merge request analysis and citation intake",
                    merge_requests_events: true,
                    note_events: true,
                    enable_ssl_verification: true,
                    token: webhook.webhook_token.as_deref(),
                    signing_token: webhook.signing_token.as_deref(),
                };
                match serde_json::to_string(&request) {
                    | Ok(body) => {
                        let hook_id = hook_id.map(|value| value.to_string());
                        let params = Params::new()
                            .with_auth(&token, Some("PRIVATE-TOKEN"))
                            .with_template("identifier", options.identifier())
                            .with_template("hook_id", hook_id.as_deref())
                            .with(crate::param!(Body, &body))
                            .build();
                        let response = endpoint.invoke(action, Some(params)).await;
                        endpoint.handle::<ProjectWebhook>(response)
                    }
                    | Err(why) => Err(eyre!("Failed to encode GitLab webhook registration — {why}")),
                }
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
/// Get descendant groups of a group by identifier
///
/// See <https://docs.gitlab.com/api/groups/#list-descendant-groups> for more information
pub async fn groups(options: &Options) -> ApiResult<GroupsResponse> {
    let template = "gitlab::api";
    let action = "groups";
    let path = format!("{template}::{action}");
    match require_non_empty_secret(&options.token, &path, &GITLAB_TOKEN_VARIABLE_NAMES) {
        | Ok(token) => match Endpoint::from_template(template).map(|e| e.with_domain(options.domain())) {
            | Ok(endpoint) => {
                let params = Params::new()
                    .with_auth(&token, Some("PRIVATE-TOKEN"))
                    .with_template("identifier", options.identifier())
                    .with_keyvalue("page", Some("1"))
                    .with_keyvalue("per_page", Some("100"))
                    .with_custom(options.params())
                    .build();
                let response = endpoint.invoke_with::<PaginationKey, EmptyField>(action, Some(params)).await;
                endpoint.handle_or::<GroupsResponse, Fallback<ErrorResponse>>(response)
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
/// Get programming languages used by a GitLab project
///
/// See <https://docs.gitlab.com/api/projects/?utm_source=perplexity#retrieve-programming-language-usage-information> for more information on GitLab API.
pub async fn language_use(options: &Options) -> ApiResult<ProgrammingLanguageUseResponse> {
    let template = "gitlab::api";
    let action = "languages";
    let path = format!("{template}::{action}");
    match require_non_empty_secret(&options.token, &path, &GITLAB_TOKEN_VARIABLE_NAMES) {
        | Ok(token) => match Endpoint::from_template(template).map(|e| e.with_domain(options.domain())) {
            | Ok(endpoint) => {
                let params = Params::new()
                    .with_auth(&token, Some("PRIVATE-TOKEN"))
                    .with_template("identifier", options.identifier())
                    .with_custom(options.params())
                    .build();
                let response = endpoint.invoke_with::<PaginationKey, EmptyField>(action, Some(params)).await;
                endpoint.handle::<ProgrammingLanguageUseResponse>(response)
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
/// Download programming language metadata from GitLab linguist source file
pub async fn languages() -> ApiResult<ProgrammingLanguagesResponse> {
    let names = ["gitlab::org", "github::org"];
    let action = "languages";
    async fn fetch(name: &str, action: &str) -> ApiResult<ProgrammingLanguagesResponse> {
        match INCLUDED_ENDPOINTS.find_by_name(name) {
            | Some(endpoint) => {
                let response = endpoint.invoke(action, None).await.map(|content| match content {
                    | ResponseContent::Raw(content) => ResponseContent::Yaml(content),
                    | other => other,
                });
                endpoint.handle::<ProgrammingLanguagesResponse>(response)
            }
            | None => Err(eyre!("{name} API endpoint not found")),
        }
    }
    let mut response: Option<ProgrammingLanguagesResponse> = None;
    let mut errors = Vec::new();
    for name in names {
        let path = format!("{name}::{action}");
        match fetch(name, action).await {
            | Ok(value) => {
                response = Some(value);
                break;
            }
            | Err(why) => errors.push(format!("{path}={why}")),
        }
    }
    match response {
        | Some(value) => Ok(value),
        | None => Err(eyre!("Failed to download and parse language metadata — {}", errors.join("; "))),
    }
}
fn authenticated_endpoint(action: &str, options: &Options) -> ApiResult<(Endpoint, String)> {
    let path = format!("gitlab::api::{action}");
    match require_non_empty_secret(&options.token, &path, &GITLAB_TOKEN_VARIABLE_NAMES) {
        | Ok(token) => Endpoint::from_template("gitlab::api").map(|endpoint| (endpoint.with_domain(options.domain()), token)),
        | Err(why) => Err(why),
    }
}
/// Get merge request metadata by IID
pub async fn merge_request(options: &Options) -> ApiResult<MergeRequestDetails> {
    let action = "merge-requests";
    match authenticated_endpoint(action, options) {
        | Ok((endpoint, token)) => {
            let params = Params::new()
                .with_auth(&token, Some("PRIVATE-TOKEN"))
                .with_template("identifier", options.identifier())
                .with_template("internal_identifier", options.internal_identifier.as_deref())
                .build();
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle_or::<MergeRequestDetails, Fallback<ErrorResponse>>(response)
        }
        | Err(why) => Err(why),
    }
}
/// List every changed file in a merge request
pub fn merge_request_diffs(options: &Options) -> BoxFuture<'_, ApiResult<MergeRequestDiffsResponse>> {
    async move {
        let action = "merge-request-diffs";
        match authenticated_endpoint(action, options) {
            | Ok((endpoint, token)) => {
                let page_value = options.page.to_string();
                let params = Params::new()
                    .with_auth(&token, Some("PRIVATE-TOKEN"))
                    .with_template("identifier", options.identifier())
                    .with_template("internal_identifier", options.internal_identifier.as_deref())
                    .with_keyvalue("page", Some(&page_value))
                    .with_keyvalue("per_page", Some("100"))
                    .build();
                let response = endpoint.invoke_with::<PaginationKey, EmptyField>(action, Some(params)).await;
                match endpoint.handle_or::<MergeRequestDiffsResponse, Fallback<ErrorResponse>>(response) {
                    | Ok(diffs) if diffs.len() == 100 => {
                        match merge_request_diffs(&options.clone().with_page(options.page.saturating_add(1))).await {
                            | Ok(remaining) => Ok(diffs.into_iter().chain(remaining).collect()),
                            | Err(why) => Err(why),
                        }
                    }
                    | result => result,
                }
            }
            | Err(why) => Err(why),
        }
    }
    .boxed()
}
/// Fetch the configured repository file at an exact commit SHA
pub async fn repository_file(options: &Options) -> ApiResult<RepositoryFile> {
    let action = "repository-file";
    match (
        authenticated_endpoint(action, options),
        options.identifier(),
        options.path(),
        options.sha(),
    ) {
        | (Ok((endpoint, token)), Some(project_identifier), Ok(file_path), Ok(sha)) => {
            let encoded_identifier = urlencoding::encode(project_identifier);
            let encoded_path = urlencoding::encode(file_path);
            let params = Params::new()
                .with_auth(&token, Some("PRIVATE-TOKEN"))
                .with_template("identifier", Some(encoded_identifier.as_ref()))
                .with_template("file_path", Some(encoded_path.as_ref()))
                .with_keyvalue("ref", Some(sha))
                .build();
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle_or::<RepositoryFile, Fallback<ErrorResponse>>(response)
        }
        | (Err(why), _, _, _) | (_, _, Err(why), _) | (_, _, _, Err(why)) => Err(why),
        | (_, None, _, _) => Err(eyre!("GitLab project identifier is required")),
    }
}
/// Publish the `acorn/check` external commit status
pub async fn publish_commit_status(
    options: &Options,
    project_id: Option<u64>,
    sha: &str,
    state: CommitStatusState,
    description: &str,
    target_url: Option<&str>,
) -> ApiResult<CommitStatus> {
    let action = "commit-status";
    match authenticated_endpoint(action, options) {
        | Ok((endpoint, token)) => {
            let state = state.to_string();
            let project_id = project_id.map(|value| value.to_string());
            let params = Params::new()
                .with_auth(&token, Some("PRIVATE-TOKEN"))
                .with_template("identifier", project_id.as_deref().or(options.identifier()))
                .with_template("sha", Some(sha))
                .with_body("state", &state)
                .with_body("name", "acorn/check")
                .with_body("description", description)
                .with_body_maybe("target_url", target_url)
                .build();
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle_or::<CommitStatus, Fallback<ErrorResponse>>(response)
        }
        | Err(why) => Err(why),
    }
}
/// List every note on a merge request
pub async fn merge_request_notes(options: &Options) -> ApiResult<NotesResponse> {
    merge_request_notes_page(options).await
}
fn merge_request_notes_page(options: &Options) -> BoxFuture<'_, ApiResult<NotesResponse>> {
    async move {
        let action = "merge-request-notes";
        match authenticated_endpoint(action, options) {
            | Ok((endpoint, token)) => {
                let page_value = options.page.to_string();
                let params = Params::new()
                    .with_auth(&token, Some("PRIVATE-TOKEN"))
                    .with_template("identifier", options.identifier())
                    .with_template("internal_identifier", options.internal_identifier.as_deref())
                    .with_keyvalue("page", Some(&page_value))
                    .with_keyvalue("per_page", Some("100"))
                    .build();
                let response = endpoint.invoke_with::<PaginationKey, EmptyField>(action, Some(params)).await;
                match endpoint.handle_or::<NotesResponse, Fallback<ErrorResponse>>(response) {
                    | Ok(notes) if notes.len() == 100 => {
                        match merge_request_notes_page(&options.clone().with_page(options.page.saturating_add(1))).await {
                            | Ok(remaining) => Ok(notes.into_iter().chain(remaining).collect()),
                            | Err(why) => Err(why),
                        }
                    }
                    | result => result,
                }
            }
            | Err(why) => Err(why),
        }
    }
    .boxed()
}
/// Get the GitLab user authenticated by the outbound API token
pub async fn current_user(options: &Options) -> ApiResult<GitLabIdentity> {
    let action = "current-user";
    match authenticated_endpoint(action, options) {
        | Ok((endpoint, token)) => {
            let params = Params::new().with_auth(&token, Some("PRIVATE-TOKEN")).build();
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle_or::<GitLabIdentity, Fallback<ErrorResponse>>(response)
        }
        | Err(why) => Err(why),
    }
}
/// Get one GitLab Issue or Task by IID
pub async fn work_item(options: &Options) -> ApiResult<WorkItem> {
    let action = "issue";
    match authenticated_endpoint(action, options) {
        | Ok((endpoint, token)) => {
            let params = Params::new()
                .with_auth(&token, Some("PRIVATE-TOKEN"))
                .with_template("identifier", options.identifier())
                .with_template("internal_identifier", options.internal_identifier.as_deref())
                .build();
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle_or::<WorkItem, Fallback<ErrorResponse>>(response)
        }
        | Err(why) => Err(why),
    }
}
/// Get a user's effective project membership, including inherited membership
pub async fn project_member(options: &Options, user_id: u64) -> ApiResult<ProjectMember> {
    let action = "member";
    match authenticated_endpoint(action, options) {
        | Ok((endpoint, token)) => {
            let user_id = user_id.to_string();
            let params = Params::new()
                .with_auth(&token, Some("PRIVATE-TOKEN"))
                .with_template("identifier", options.identifier())
                .with_template("user_id", Some(&user_id))
                .build();
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle_or::<ProjectMember, Fallback<ErrorResponse>>(response)
        }
        | Err(why) => Err(why),
    }
}
/// List every note on one GitLab Issue or Task
pub fn work_item_notes(options: &Options) -> BoxFuture<'_, ApiResult<NotesResponse>> {
    async move {
        let action = "issue-notes";
        match authenticated_endpoint(action, options) {
            | Ok((endpoint, token)) => {
                let page = options.page.to_string();
                let params = Params::new()
                    .with_auth(&token, Some("PRIVATE-TOKEN"))
                    .with_template("identifier", options.identifier())
                    .with_template("internal_identifier", options.internal_identifier.as_deref())
                    .with_keyvalue("page", Some(&page))
                    .with_keyvalue("per_page", Some("100"))
                    .with_keyvalue("sort", Some("asc"))
                    .build();
                let response = endpoint.invoke_with::<PaginationKey, EmptyField>(action, Some(params)).await;
                match endpoint.handle_or::<NotesResponse, Fallback<ErrorResponse>>(response) {
                    | Ok(notes) if notes.len() == 100 => match work_item_notes(&options.clone().with_page(options.page.saturating_add(1))).await {
                        | Ok(remaining) => Ok(notes.into_iter().chain(remaining).collect()),
                        | Err(why) => Err(why),
                    },
                    | result => result,
                }
            }
            | Err(why) => Err(why),
        }
    }
    .boxed()
}
/// Create or update the bot-authored citation-intake report note
pub async fn upsert_work_item_note(options: &Options, marker: &str, body: &str) -> ApiResult<Note> {
    match current_user(options).await {
        | Ok(identity) => match work_item_notes(options).await {
            | Ok(notes) => {
                let existing = notes
                    .into_iter()
                    .find(|note| note.body().contains(marker) && note.author_id() == Some(identity.identifier));
                write_work_item_note(options, existing.map(|note| note.identifier()), body).await
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
async fn write_work_item_note(options: &Options, note_id: Option<u64>, body: &str) -> ApiResult<Note> {
    let action = if note_id.is_some() { "issue-note::update" } else { "issue-note" };
    match authenticated_endpoint(action, options) {
        | Ok((endpoint, token)) => {
            let note_id = note_id.map(|value| value.to_string());
            let params = Params::new()
                .with_auth(&token, Some("PRIVATE-TOKEN"))
                .with_template("identifier", options.identifier())
                .with_template("internal_identifier", options.internal_identifier.as_deref())
                .with_template("note_id", note_id.as_deref())
                .with_body("body", body)
                .build();
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle_or::<Note, Fallback<ErrorResponse>>(response)
        }
        | Err(why) => Err(why),
    }
}
/// Create or update the bot-authored merge request report note
pub async fn upsert_merge_request_note(options: &Options, marker: &str, body: &str) -> ApiResult<Note> {
    match current_user(options).await {
        | Ok(identity) => match merge_request_notes(options).await {
            | Ok(notes) => {
                let existing = notes
                    .into_iter()
                    .find(|note| note.body().contains(marker) && note.author_id() == Some(identity.identifier));
                write_merge_request_note(options, existing.map(|note| note.identifier()), body).await
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
async fn write_merge_request_note(options: &Options, note_id: Option<u64>, body: &str) -> ApiResult<Note> {
    let action = if note_id.is_some() {
        "merge-request-note::update"
    } else {
        "merge-request-note"
    };
    match authenticated_endpoint(action, options) {
        | Ok((endpoint, token)) => {
            let note_id = note_id.map(|value| value.to_string());
            let params = Params::new()
                .with_auth(&token, Some("PRIVATE-TOKEN"))
                .with_template("identifier", options.identifier())
                .with_template("internal_identifier", options.internal_identifier.as_deref())
                .with_template("note_id", note_id.as_deref())
                .with_body("body", body)
                .build();
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle_or::<Note, Fallback<ErrorResponse>>(response)
        }
        | Err(why) => Err(why),
    }
}
/// Add a note (comment) to a merge request using project identifier and merge request IID
///
/// When used in CI environment, the project identifier can be obtained from the `CI_PROJECT_ID` environment variable and the merge request IID can be obtained from the `CI_MERGE_REQUEST_IID` environment variable.
/// The GitLab token must be provided in the `CI_JOB_TOKEN` environment variable.
///
/// See <https://docs.gitlab.com/api/notes/#create-a-merge-request-note> for more information on this API endpoint and required parameters
pub async fn merge_request_note(options: &Options) -> ApiResult<NoteMetadata> {
    let template = "gitlab::api";
    let action = "merge-request-note";
    let path = format!("{template}::{action}");
    match require_non_empty_secret(&options.token, &path, &GITLAB_TOKEN_VARIABLE_NAMES) {
        | Ok(token) => match Endpoint::from_template(template).map(|e| e.with_domain(options.domain())) {
            | Ok(endpoint) => {
                let body = options.body.as_deref().unwrap_or_default();
                let params = Params::new()
                    .with_auth(&token, Some("PRIVATE-TOKEN"))
                    .with_body("body", body)
                    .with_template("identifier", options.identifier())
                    .with_template("internal_identifier", options.internal_identifier.as_deref())
                    .with_custom(options.params())
                    .build();
                let response = endpoint.invoke_with::<PaginationKey, EmptyField>(action, Some(params)).await;
                endpoint.handle::<NoteMetadata>(response)
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
/// Get runner details by identifier
pub async fn runner(options: &Options) -> ApiResult<RunnerMetadata> {
    let template = "gitlab::api";
    let action = "runner";
    let path = format!("{template}::{action}");
    match require_non_empty_secret(&options.token, &path, &GITLAB_TOKEN_VARIABLE_NAMES) {
        | Ok(token) => match Endpoint::from_template(template).map(|e| e.with_domain(options.domain())) {
            | Ok(endpoint) => {
                let params = Params::new()
                    .with_auth(&token, Some("PRIVATE-TOKEN"))
                    .with_template("identifier", options.identifier())
                    .with_custom(options.params())
                    .build();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle::<RunnerMetadata>(response)
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
/// Get runners visible to user associated with given token
pub async fn runners(options: &Options) -> ApiResult<RunnersResponse> {
    let template = "gitlab::api";
    let action = "runners";
    let path = format!("{template}::{action}");
    match require_non_empty_secret(&options.token, &path, &GITLAB_TOKEN_VARIABLE_NAMES) {
        | Ok(token) => match Endpoint::from_template(template).map(|e| e.with_domain(options.domain())) {
            | Ok(endpoint) => {
                let params = Params::new()
                    .with_auth(&token, Some("PRIVATE-TOKEN"))
                    .with_custom(options.params())
                    .build();
                let response = endpoint.invoke_with::<PaginationKey, EmptyField>(action, Some(params)).await;
                endpoint.handle_or::<RunnersResponse, Fallback<ErrorResponse>>(response)
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
/// Fetch one page of repository tree blob paths for a GitLab project
pub(crate) async fn tree_paths(options: &Options) -> ApiResult<TreeResponse> {
    let template = "gitlab::api";
    let action = "tree";
    match Endpoint::from_template(template).map(|e| e.with_domain(options.domain())) {
        | Ok(endpoint) => {
            let params = Params::new()
                .with_template("identifier", options.identifier())
                .with_keyvalue("per_page", Some("100"))
                .with_keyvalue("page", Some(&options.page.to_string()))
                .with_keyvalue("recursive", Some("true"))
                .with_keyvalue("path", options.path.as_deref())
                .with_custom(options.params());
            let token = if options.token.trim().is_empty() {
                first_env_var(&GITLAB_TOKEN_VARIABLE_NAMES)
            } else {
                Some(options.token.clone())
            };
            let params = match token.as_ref().filter(|v| !v.trim().is_empty()) {
                | Some(token) => params.with_auth(token, Some("PRIVATE-TOKEN")),
                | None => params,
            };
            let response = endpoint.invoke(action, Some(params.build())).await;
            handle_tree_paths_response(endpoint.handle_or::<TreeResponse, Fallback<ErrorResponse>>(response), options.page)
        }
        | Err(why) => Err(why),
    }
}
