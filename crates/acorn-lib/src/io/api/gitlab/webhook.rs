//! GitLab webhook authentication, normalization, and durable processing
use super::bot::Config;
#[cfg(feature = "analysis")]
use super::intake::{complete_citation_merge, Report as IntakeReport};
#[cfg(feature = "analysis")]
use super::review;
use crate::agent::slash::{SlashCommand, SlashContext};
use crate::io::api;
use crate::io::api::json_rpc;
use crate::io::api::webhooks::store::OperationQueue;
use crate::io::api::webhooks::{self, Verifier, WebhookRuntime};
use crate::io::ApiResult;
use alloc::sync::Arc;
use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::Extension;
use bon::Builder;
use color_eyre::eyre::{eyre, Report};
use core::fmt;
use serde::{Deserialize, Serialize};
#[cfg(feature = "analysis")]
use tracing::info;

const DEVELOPER_ACCESS_LEVEL: u64 = 30;
/// An authenticated, normalized webhook delivery ready for processing.
pub type WebhookDelivery = webhooks::Delivery<HookPayload>;
/// Awaitable result produced by a durable webhook operation
pub type WebhookOperation = webhooks::Operation;
/// Callback used by durable webhook workers
pub type WebhookOperationHandler = webhooks::OperationHandler<HookPayload>;
/// Normalized payload from a GitLab webhook delivery, with struct-like enum variants per event kind.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum HookPayload {
    /// Merge request opened, reopened, updated, closed, merged, approved, or unapproved.
    MergeRequest {
        /// User and project context for this delivery.
        actor: HookActor,
        /// Merge request IID within the project.
        iid: u64,
        /// SHA of the head commit at the time of the event.
        head_sha: Option<String>,
        /// Action that triggered this event.
        action: MergeRequestAction,
        /// Merge request title.
        title: String,
        /// Merge request description.
        description: String,
    },
    /// Note created on an issue, merge request, or other noteable.
    Note {
        /// User and project context for this delivery.
        actor: HookActor,
        /// Numeric ID of the note.
        note_id: u64,
        /// Note body text.
        body: String,
        /// Type of the noteable object (`Issue`, `MergeRequest`, etc.).
        noteable_type: String,
        /// Numeric ID of the noteable object.
        noteable_id: Option<u64>,
        /// IID of the noteable object within its project.
        noteable_iid: Option<u64>,
        /// Whether this is a system-generated note.
        system: bool,
        /// Whether this note is confidential.
        confidential: bool,
        /// Whether this note is internal.
        internal: bool,
    },
}
/// Merge request action from a GitLab `Merge Request Hook` webhook event.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MergeRequestAction {
    /// Merge request was opened.
    Open,
    /// Merge request was reopened.
    Reopen,
    /// Merge request head commit was updated.
    Update,
    /// Merge request was closed.
    Close,
    /// Merge request was merged.
    Merge,
    /// Merge request was approved.
    Approve,
    /// Merge request approval was revoked.
    Unapprove,
    /// Other or unknown action.
    Other(String),
}
enum Webhook {
    Delivery(WebhookDelivery),
    Unsupported,
}
/// User and project context shared across webhook delivery payload variants.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HookActor {
    /// Numeric GitLab project ID.
    pub project_id: u64,
    /// Numeric ID of the user who triggered the event.
    pub user_id: u64,
    /// Username of the user who triggered the event.
    pub username: String,
    /// Whether the triggering user is a bot account.
    pub is_bot: bool,
}
#[derive(Deserialize)]
struct HookUser {
    #[serde(rename = "id")]
    id: u64,
    username: String,
    #[serde(default)]
    bot: bool,
}
#[derive(Deserialize)]
struct RawMergeRequestAttributes {
    iid: u64,
    title: String,
    #[serde(default)]
    description: String,
    action: Option<String>,
    last_commit: Option<api::Identifier<String>>,
}
#[derive(Deserialize)]
struct RawMergeRequestHook {
    user: HookUser,
    project: api::Identifier<u64>,
    object_attributes: RawMergeRequestAttributes,
}
#[derive(Deserialize)]
struct RawNoteAttributes {
    #[serde(rename = "id")]
    id: u64,
    note: String,
    noteable_type: String,
    noteable_id: Option<u64>,
    #[serde(default)]
    system: bool,
    #[serde(default)]
    confidential: bool,
    #[serde(default)]
    internal: bool,
}
#[derive(Deserialize)]
struct RawNoteHook {
    user: HookUser,
    project: api::Identifier<u64>,
    object_attributes: RawNoteAttributes,
    #[serde(default)]
    issue: Option<api::Identifier<u64>>,
    #[serde(default)]
    merge_request: Option<api::Identifier<u64>>,
}
/// Immutable webhook config injected into the HTTP router via [`Extension`].
#[derive(Builder, Clone)]
#[builder(start_fn = init)]
pub(super) struct WebhookConfig {
    webhook_token: Option<api::Secret>,
    webhook_signing_token: Option<api::Secret>,
    project_id: Option<u64>,
    operation_queue: OperationQueue,
    runtime: WebhookRuntime,
}
#[derive(Debug)]
struct WebhookError {
    status: StatusCode,
    message: String,
}
impl From<(u64, HookUser)> for HookActor {
    fn from((project_id, user): (u64, HookUser)) -> Self {
        let HookUser {
            id: user_id,
            username,
            bot: is_bot,
        } = user;
        Self {
            project_id,
            user_id,
            username,
            is_bot,
        }
    }
}
impl HookActor {
    /// Create webhook actor metadata.
    pub fn new(project_id: u64, user_id: u64, username: impl Into<String>, is_bot: bool) -> Self {
        Self {
            project_id,
            user_id,
            username: username.into(),
            is_bot,
        }
    }
}
impl From<&str> for MergeRequestAction {
    fn from(action: &str) -> Self {
        match action {
            | "open" => Self::Open,
            | "reopen" => Self::Reopen,
            | "update" => Self::Update,
            | "close" => Self::Close,
            | "merge" => Self::Merge,
            | "approve" => Self::Approve,
            | "unapprove" => Self::Unapprove,
            | other => Self::Other(other.to_string()),
        }
    }
}
impl From<Option<&str>> for MergeRequestAction {
    fn from(action: Option<&str>) -> Self {
        action.map_or_else(|| Self::Other(String::new()), Self::from)
    }
}
impl From<Webhook> for StatusCode {
    fn from(webhook: Webhook) -> Self {
        match webhook {
            | Webhook::Delivery(delivery) => delivery.into(),
            | Webhook::Unsupported => Self::OK,
        }
    }
}
impl From<WebhookDelivery> for StatusCode {
    fn from(_delivery: WebhookDelivery) -> Self {
        Self::ACCEPTED
    }
}
impl Webhook {
    /// Normalize a supported GitLab webhook payload and validate its project identifier
    fn normalize(delivery_id: &str, headers: &HeaderMap, raw_body: &[u8], expected_project_id: Option<u64>) -> ApiResult<Self> {
        match api::first_header(headers, &["x-gitlab-event"]) {
            | Some("Merge Request Hook") => serde_json::from_slice::<RawMergeRequestHook>(raw_body)
                .map_err(|why| WebhookError::report(StatusCode::BAD_REQUEST, format!("Invalid Merge Request Hook payload: {why}")))
                .and_then(|hook| {
                    let RawMergeRequestHook {
                        user,
                        project: api::Identifier { identifier: project_id },
                        object_attributes,
                    } = hook;
                    Self::validate_project(expected_project_id, project_id).map(|()| {
                        let RawMergeRequestAttributes {
                            iid,
                            title,
                            description,
                            action,
                            last_commit,
                        } = object_attributes;
                        HookPayload::MergeRequest {
                            actor: (project_id, user).into(),
                            iid,
                            head_sha: last_commit.map(|reference| reference.identifier),
                            action: action.as_deref().into(),
                            title,
                            description,
                        }
                    })
                })
                .map(|event| {
                    Self::Delivery(webhooks::Delivery {
                        delivery_id: delivery_id.to_string(),
                        event,
                    })
                }),
            | Some("Note Hook") => serde_json::from_slice::<RawNoteHook>(raw_body)
                .map_err(|why| WebhookError::report(StatusCode::BAD_REQUEST, format!("Invalid Note Hook payload: {why}")))
                .and_then(|hook| {
                    let RawNoteHook {
                        user,
                        project: api::Identifier { identifier: project_id },
                        object_attributes,
                        issue,
                        merge_request,
                    } = hook;
                    Self::validate_project(expected_project_id, project_id).map(|()| {
                        let RawNoteAttributes {
                            id: note_id,
                            note: body,
                            noteable_type,
                            noteable_id,
                            system,
                            confidential,
                            internal,
                        } = object_attributes;
                        HookPayload::Note {
                            actor: (project_id, user).into(),
                            note_id,
                            body,
                            noteable_type,
                            noteable_id,
                            noteable_iid: issue.or(merge_request).map(|reference| reference.identifier),
                            system,
                            confidential,
                            internal,
                        }
                    })
                })
                .map(|event| {
                    Self::Delivery(webhooks::Delivery {
                        delivery_id: delivery_id.to_string(),
                        event,
                    })
                }),
            | Some(_) => Ok(Self::Unsupported),
            | None => Err(WebhookError::report(StatusCode::BAD_REQUEST, "Missing X-Gitlab-Event header")),
        }
    }
    fn validate_project(expected: Option<u64>, actual: u64) -> ApiResult<()> {
        if expected.is_some_and(|project_id| project_id != actual) {
            Err(WebhookError::report(StatusCode::FORBIDDEN, "Webhook from unexpected project"))
        } else {
            Ok(())
        }
    }
}
impl Webhook {
    async fn receive(config: Arc<WebhookConfig>, request: Request) -> Result<StatusCode, (StatusCode, String)> {
        match config.runtime.is_ready() {
            | false => Err((StatusCode::SERVICE_UNAVAILABLE, "Webhook runtime is not ready".to_string())),
            | true => {
                let (parts, body) = request.into_parts();
                to_bytes(body, 1024 * 1024)
                    .await
                    .map_err(|_| WebhookError::report(StatusCode::PAYLOAD_TOO_LARGE, "Request body exceeds 1 MiB limit"))
                    .and_then(|raw| {
                        config
                            .authenticate(&parts.headers, &raw)
                            .and_then(|verified| Self::normalize(verified.delivery_id, &parts.headers, verified.raw_body, config.project_id))
                    })
                    .and_then(|webhook| webhook.persist(&config))
                    .map_err(WebhookError::response)
            }
        }
    }
    fn persist(self, config: &WebhookConfig) -> ApiResult<StatusCode> {
        match self {
            | Self::Unsupported => Ok(StatusCode::OK),
            | Self::Delivery(delivery) => match delivery.operation() {
                | Ok(Some(operation)) => match config.operation_queue.enqueue_spec(&delivery.delivery_id, &operation) {
                    | Ok(_) => Ok(StatusCode::ACCEPTED),
                    | Err(why) => {
                        config.runtime.mark_unavailable();
                        Err(WebhookError::report(
                            StatusCode::SERVICE_UNAVAILABLE,
                            format!("Failed to persist webhook operation: {why}"),
                        ))
                    }
                },
                | Err(why) => Err(WebhookError::report(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to normalize webhook operation: {why}"),
                )),
                | Ok(None) => Ok(StatusCode::ACCEPTED),
            },
        }
    }
}
impl WebhookConfig {
    /// Authenticate an inbound webhook delivery against the configured token or signing secret.
    ///
    /// When a `webhook-signature` header is present, Standard Webhooks HMAC-SHA256 verification
    /// is performed and the legacy `X-Gitlab-Token` path is bypassed entirely.  Without that
    /// header, the `X-Gitlab-Token` value (if any) is compared in constant time.
    fn authenticate<'a>(&self, headers: &'a HeaderMap, raw_body: &'a [u8]) -> ApiResult<webhooks::VerifiedRequest<'a>> {
        let request = webhooks::InboundRequest { headers, raw_body };
        if headers.contains_key("webhook-signature") {
            self.webhook_signing_token
                .clone()
                .ok_or_else(|| WebhookError::report(StatusCode::UNAUTHORIZED, "No signing token configured for Standard Webhooks"))
                .and_then(|signing_token| {
                    webhooks::StandardWebhooksVerifier::new(signing_token, 300)
                        .and_then(|verifier| verifier.verify(request, jiff::Timestamp::now()))
                        .map_err(WebhookError::from_shared)
                })
        } else {
            self.webhook_token.clone().map_or_else(
                || Err(WebhookError::report(StatusCode::UNAUTHORIZED, "No webhook credential configured")),
                |token| {
                    webhooks::HeaderSecretVerifier::new("x-gitlab-token", token, &["webhook-id", "idempotency-key", "x-gitlab-webhook-uuid"])
                        .and_then(|verifier| verifier.verify(request, jiff::Timestamp::now()))
                        .map_err(WebhookError::from_shared)
                },
            )
        }
    }
}
impl core::error::Error for WebhookError {}
impl fmt::Display for WebhookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}
impl WebhookError {
    fn from_shared(error: webhooks::WebhookError) -> Report {
        let status = match error.kind() {
            | webhooks::ErrorKind::BadRequest => StatusCode::BAD_REQUEST,
            | webhooks::ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
            | _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self::report(status, error.to_string())
    }
    fn report(status: StatusCode, message: impl Into<String>) -> Report {
        eyre!(Self {
            status,
            message: message.into(),
        })
    }
    fn response(error: Report) -> (StatusCode, String) {
        error.downcast_ref::<Self>().map_or_else(
            || (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            |webhook_error| (webhook_error.status, webhook_error.message.clone()),
        )
    }
}
impl webhooks::Delivery<HookPayload> {
    /// Normalize an actionable GitLab delivery into a registered operation notification.
    pub fn operation(&self) -> ApiResult<Option<webhooks::OperationSpec>> {
        self.key()
            .map(|key| {
                let method = match &self.event {
                    | HookPayload::MergeRequest { .. } => json_rpc::MethodName::from(["webhooks", "gitlab", "merge-request"]),
                    | HookPayload::Note { .. } => json_rpc::MethodName::from(["webhooks", "gitlab", "note"]),
                };
                serde_json::to_value(self)
                    .map_err(|why| eyre!("Failed to encode GitLab webhook delivery — {why}"))
                    .and_then(|params| json_rpc::Notification::new(method, params).map_err(color_eyre::Report::from))
                    .map(|notification| webhooks::OperationSpec::new("gitlab", key, notification))
            })
            .transpose()
    }
    /// Return the canonical operation key for an actionable normalized delivery
    pub fn key(&self) -> Option<String> {
        match &self.event {
            | HookPayload::MergeRequest {
                actor,
                iid,
                head_sha: Some(head_sha),
                action: MergeRequestAction::Open | MergeRequestAction::Reopen | MergeRequestAction::Update,
                ..
            } if !actor.is_bot => Some(format!("mr-check:{}:{iid}:{head_sha}", actor.project_id)),
            | HookPayload::MergeRequest {
                actor,
                iid,
                action: MergeRequestAction::Merge,
                ..
            } if !actor.is_bot => Some(format!("citation-complete:{}:{iid}", actor.project_id)),
            | HookPayload::Note {
                actor,
                note_id,
                body,
                noteable_type,
                noteable_iid: Some(iid),
                system,
                confidential,
                internal,
                ..
            } => {
                let human_authored = !actor.is_bot && !system && !confidential && !internal;
                let work_item = noteable_type.eq_ignore_ascii_case("issue") || noteable_type.eq_ignore_ascii_case("task");
                let merge_request = noteable_type.eq_ignore_ascii_case("mergerequest");
                if human_authored && work_item && check_requested(body) {
                    Some(format!("work-item-check:{}:{iid}:{note_id}", actor.project_id))
                } else if human_authored && merge_request && workflow_requested(body).is_some() {
                    Some(format!("repository-process:{}:{iid}:{note_id}", actor.project_id))
                } else if human_authored && merge_request && graduation_requested(body).is_some() {
                    Some(format!("logbook-graduate:{}:{iid}:{note_id}", actor.project_id))
                } else {
                    None
                }
            }
            | _ => None,
        }
    }
    pub(super) async fn process(self, config: &Config) -> ApiResult<()> {
        match &config.operation_handler {
            | Some(handler) => handler(self).await,
            | None => self.default_handler(config).await,
        }
    }
    #[cfg(feature = "analysis")]
    async fn default_handler(self, config: &Config) -> ApiResult<()> {
        match self.event {
            | HookPayload::MergeRequest {
                iid,
                action: MergeRequestAction::Merge,
                ..
            } => match config.deployment.workflow("repository-quality") {
                | Ok(policy) if policy.allows(super::WorkflowEffect::Label) && policy.allows(super::WorkflowEffect::CompletionComment) => {
                    let options = config
                        .options
                        .clone()
                        .with_effects(config.operation_queue.clone())
                        .with_merge_request_iid(iid);
                    complete_citation_merge(&options)
                        .await
                        .inspect(|completed| info!(completed, "Processed GitLab citation merge-completion operation"))
                        .map(|_| ())
                }
                | Ok(_) => Err(eyre!("Citation merge completion requires label and completion-comment effects")),
                | Err(why) => Err(why),
            },
            | HookPayload::MergeRequest {
                iid,
                head_sha: Some(head_sha),
                action: MergeRequestAction::Open | MergeRequestAction::Reopen | MergeRequestAction::Update,
                ..
            } => {
                let options = config
                    .options
                    .clone()
                    .with_effects(config.operation_queue.clone())
                    .with_internal_identifier(iid.to_string())
                    .with_sha(head_sha);
                let required_effects = [super::WorkflowEffect::CommitStatus, super::WorkflowEffect::Note];
                match config.deployment.workflow("repository-quality") {
                    | Ok(policy) if required_effects.into_iter().all(|effect| policy.allows(effect)) => {
                        review::analyze_merge_request(&options, &config.analysis_options)
                            .await
                            .inspect(|outcome| info!("Processed GitLab merge request analysis operation: {outcome:?}"))
                            .map(|_| ())
                    }
                    | Ok(_) => Err(eyre!("Merge request analysis requires commit-status and note effects")),
                    | Err(why) => Err(why),
                }
            }
            | HookPayload::Note {
                actor,
                note_id,
                noteable_iid: Some(iid),
                noteable_type,
                ..
            } if noteable_type.eq_ignore_ascii_case("issue") || noteable_type.eq_ignore_ascii_case("task") => {
                let options = config
                    .options
                    .clone()
                    .with_effects(config.operation_queue.clone())
                    .with_internal_identifier(iid.to_string());
                let required_effects = [
                    super::WorkflowEffect::Branch,
                    super::WorkflowEffect::Commit,
                    super::WorkflowEffect::DraftMergeRequest,
                    super::WorkflowEffect::Note,
                ];
                match config.deployment.workflow("repository-quality") {
                    | Ok(policy) if required_effects.into_iter().all(|effect| policy.allows(effect)) => {
                        IntakeReport::analyze(&options, &actor, note_id)
                            .await
                            .inspect(|report| info!("Processed GitLab work-item intake operation: {report:?}"))
                            .map(|_| ())
                    }
                    | Ok(_) => Err(eyre!("Citation intake requires branch, commit, draft-merge-request, and note effects")),
                    | Err(why) => Err(why),
                }
            }
            | HookPayload::Note {
                actor,
                body,
                noteable_iid: Some(iid),
                noteable_type,
                ..
            } if noteable_type.eq_ignore_ascii_case("mergerequest") => match (workflow_requested(&body), graduation_requested(&body)) {
                | (Some(workflow), _) => {
                    let options = config.options.clone().with_internal_identifier(iid.to_string());
                    match super::project_member(&options, actor.user_id).await.and_then(|member| {
                        match member.identifier == actor.user_id && member.access_level >= DEVELOPER_ACCESS_LEVEL {
                            | true => Ok(()),
                            | false => Err(eyre!("GitLab repository workflow commands require Developer access")),
                        }
                    }) {
                        | Ok(()) => match super::merge_request(&options).await {
                            | Ok(details) => details
                                .process(&workflow, &options, &config.deployment, Some(&config.operation_queue))
                                .await
                                .inspect(|result| info!("Processed GitLab repository workflow operation: {result:?}"))
                                .map(|_| ()),
                            | Err(why) => Err(why),
                        },
                        | Err(why) => Err(why),
                    }
                }
                | (None, Some(accepted_ids)) => {
                    let options = config.options.clone().with_internal_identifier(iid.to_string());
                    match super::project_member(&options, actor.user_id).await.and_then(|member| {
                        match member.identifier == actor.user_id && member.access_level >= DEVELOPER_ACCESS_LEVEL {
                            | true => Ok(()),
                            | false => Err(eyre!("GitLab logbook graduation commands require Developer access")),
                        }
                    }) {
                        | Ok(()) => match &config.powerautomate {
                            | Some(provider) => {
                                let paths = provider.routing().folders().filter_map(|folder| folder.path.clone()).collect::<Vec<_>>();
                                options
                                    .graduate_logbook(&config.operation_queue, &paths, &accepted_ids)
                                    .await
                                    .inspect(|result| info!("Processed GitLab logbook graduation operation: {result:?}"))
                                    .map(|_| ())
                            }
                            | None => Err(eyre!("PowerAutomate project mappings are not configured")),
                        },
                        | Err(why) => Err(why),
                    }
                }
                | (None, None) => Ok(()),
            },
            | _ => {
                info!("Ignoring unsupported GitLab webhook operation {}", self.delivery_id);
                Ok(())
            }
        }
    }
    #[cfg(not(feature = "analysis"))]
    async fn default_handler(self, _config: &Config) -> ApiResult<()> {
        Err(eyre!(
            "GitLab webhook operation {} requires the acorn-lib analysis feature",
            self.delivery_id
        ))
    }
}
pub(super) fn check_requested(content: &str) -> bool {
    content
        .lines()
        .any(|line| line.trim() == "/acorn check" && matches!(SlashCommand::parse(line, SlashContext::GitLab), Ok(SlashCommand::Check)))
}
fn graduation_requested(content: &str) -> Option<Vec<String>> {
    content.lines().find_map(|line| match SlashCommand::parse(line, SlashContext::GitLab) {
        | Ok(SlashCommand::Graduate(identifiers)) => Some(identifiers),
        | _ => None,
    })
}
/// Authenticate and normalize an inbound GitLab webhook request into an HTTP status
pub(super) async fn receive(Extension(config): Extension<Arc<WebhookConfig>>, request: Request) -> Result<StatusCode, (StatusCode, String)> {
    Webhook::receive(config, request).await
}
fn workflow_requested(content: &str) -> Option<String> {
    content.lines().find_map(|line| match SlashCommand::parse(line, SlashContext::GitLab) {
        | Ok(SlashCommand::Process(workflow)) => Some(workflow),
        | _ => None,
    })
}
