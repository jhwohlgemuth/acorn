//! GitLab webhook authentication, normalization, and durable processing.

use super::bot::Config;
use super::database::OperationQueue;
#[cfg(feature = "analysis")]
use super::intake::analyze_work_item;
use crate::io::api;
use crate::io::{self, ApiResult};
use crate::util::constant_time_eq;
use alloc::sync::Arc;
use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::Extension;
use bon::Builder;
use color_eyre::eyre::{eyre, Report};
use core::{fmt, future::Future, pin::Pin};
use data_encoding::BASE64;
use ring::hmac;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Awaitable result produced by a durable webhook operation
pub type WebhookOperation = Pin<Box<dyn Future<Output = ApiResult<()>> + Send>>;
/// Callback used by durable webhook workers
pub type WebhookOperationHandler = Arc<dyn Fn(WebhookDelivery) -> WebhookOperation + Send + Sync + 'static>;
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
/// Immutable webhook config injected into the Axum router via [`Extension`].
#[derive(Builder, Clone)]
#[builder(start_fn = init, on(String, into))]
pub(super) struct WebhookConfig {
    webhook_token: Option<String>,
    webhook_signing_token: Option<String>,
    project_id: Option<u64>,
    operation_queue: OperationQueue,
}
/// An authenticated, normalized webhook delivery ready for processing.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebhookDelivery {
    /// Delivery identifier from `webhook-id`, `Idempotency-Key`, or `X-Gitlab-Webhook-UUID`.
    pub delivery_id: String,
    /// Normalized event payload.
    pub event: HookPayload,
}
#[derive(Debug)]
struct WebhookError {
    status: StatusCode,
    message: String,
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
    fn normalize(headers: &HeaderMap, raw_body: &[u8], expected_project_id: Option<u64>) -> ApiResult<Self> {
        let event_type = api::first_header(headers, &["x-gitlab-event"])
            .ok_or_else(|| WebhookError::report(StatusCode::BAD_REQUEST, "Missing X-Gitlab-Event header"))?;
        let delivery_id = api::first_header(headers, &["webhook-id", "idempotency-key", "x-gitlab-webhook-uuid"])
            .filter(|delivery_id| !delivery_id.trim().is_empty())
            .ok_or_else(|| WebhookError::report(StatusCode::BAD_REQUEST, "Missing webhook delivery identifier"))?;
        let event = match event_type {
            | "Merge Request Hook" => {
                let RawMergeRequestHook {
                    user,
                    project: api::Identifier { identifier: project_id },
                    object_attributes,
                } = serde_json::from_slice(raw_body)
                    .map_err(|why| WebhookError::report(StatusCode::BAD_REQUEST, format!("Invalid Merge Request Hook payload: {why}")))?;
                Self::validate_project(expected_project_id, project_id)?;
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
            }
            | "Note Hook" => {
                let RawNoteHook {
                    user,
                    project: api::Identifier { identifier: project_id },
                    object_attributes,
                    issue,
                    merge_request,
                } = serde_json::from_slice(raw_body)
                    .map_err(|why| WebhookError::report(StatusCode::BAD_REQUEST, format!("Invalid Note Hook payload: {why}")))?;
                Self::validate_project(expected_project_id, project_id)?;
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
            }
            | _ => return Ok(Self::Unsupported),
        };
        Ok(Self::Delivery(WebhookDelivery {
            delivery_id: delivery_id.to_string(),
            event,
        }))
    }
    fn validate_project(expected: Option<u64>, actual: u64) -> ApiResult<()> {
        if expected.is_some_and(|project_id| project_id != actual) {
            Err(WebhookError::report(StatusCode::FORBIDDEN, "Webhook from unexpected project"))
        } else {
            Ok(())
        }
    }
    fn verify(headers: &HeaderMap, raw_body: &[u8], signing_token: &str) -> ApiResult<()> {
        let webhook_id =
            api::first_header(headers, &["webhook-id"]).ok_or_else(|| WebhookError::report(StatusCode::BAD_REQUEST, "Missing webhook-id header"))?;
        let timestamp = api::first_header(headers, &["webhook-timestamp"])
            .ok_or_else(|| WebhookError::report(StatusCode::BAD_REQUEST, "Missing webhook-timestamp header"))?;
        timestamp
            .parse::<i64>()
            .map_err(|_| WebhookError::report(StatusCode::BAD_REQUEST, "Invalid webhook-timestamp value"))
            .and_then(|timestamp| {
                io::validate_unix_timestamp_window(timestamp, 300)
                    .map_err(|_| WebhookError::report(StatusCode::UNAUTHORIZED, "Webhook timestamp is outside the five-minute window"))
            })?;
        let signature_header = api::first_header(headers, &["webhook-signature"])
            .ok_or_else(|| WebhookError::report(StatusCode::UNAUTHORIZED, "Missing webhook-signature header"))?;
        let encoded_token = signing_token
            .strip_prefix("whsec_")
            .ok_or_else(|| WebhookError::report(StatusCode::UNAUTHORIZED, "Invalid webhook signing token format"))?;
        let key_bytes = BASE64
            .decode(encoded_token.as_bytes())
            .map_err(|_| WebhookError::report(StatusCode::UNAUTHORIZED, "Invalid webhook signing token encoding"))?;
        let key = hmac::Key::new(hmac::HMAC_SHA256, &key_bytes);
        let mut message = format!("{webhook_id}.{timestamp}.").into_bytes();
        message.extend_from_slice(raw_body);
        let verified = signature_header.split_whitespace().any(|part| {
            part.strip_prefix("v1,").is_some_and(|signature| {
                BASE64
                    .decode(signature.as_bytes())
                    .is_ok_and(|tag_bytes| hmac::verify(&key, &message, &tag_bytes).is_ok())
            })
        });
        if verified {
            Ok(())
        } else {
            Err(WebhookError::report(StatusCode::UNAUTHORIZED, "Webhook signature verification failed"))
        }
    }
}
impl Webhook {
    async fn receive(config: Arc<WebhookConfig>, request: Request) -> Result<StatusCode, (StatusCode, String)> {
        let (parts, body) = request.into_parts();
        to_bytes(body, 1024 * 1024)
            .await
            .map_err(|_| WebhookError::report(StatusCode::PAYLOAD_TOO_LARGE, "Request body exceeds 1 MiB limit"))
            .and_then(|raw| {
                config
                    .authenticate(&parts.headers, &raw)
                    .and_then(|_| Self::normalize(&parts.headers, &raw, config.project_id))
            })
            .and_then(|webhook| webhook.persist(&config))
            .map_err(WebhookError::response)
    }
    fn persist(self, config: &WebhookConfig) -> ApiResult<StatusCode> {
        match self {
            | Self::Unsupported => Ok(StatusCode::OK),
            | Self::Delivery(delivery) => match delivery.key() {
                | Some(operation_key) => serde_json::to_string(&delivery)
                    .map_err(|why| {
                        WebhookError::report(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to serialize normalized webhook: {why}"),
                        )
                    })
                    .and_then(|event_json| {
                        config
                            .operation_queue
                            .enqueue(&delivery.delivery_id, &operation_key, &event_json)
                            .map(|_| StatusCode::ACCEPTED)
                            .map_err(|why| {
                                WebhookError::report(StatusCode::SERVICE_UNAVAILABLE, format!("Failed to persist webhook operation: {why}"))
                            })
                    }),
                | None => Ok(StatusCode::ACCEPTED),
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
    fn authenticate(&self, headers: &HeaderMap, raw_body: &[u8]) -> ApiResult<()> {
        if headers.contains_key("webhook-signature") {
            self.webhook_signing_token
                .as_deref()
                .ok_or_else(|| WebhookError::report(StatusCode::UNAUTHORIZED, "No signing token configured for Standard Webhooks"))
                .and_then(|signing_token| Webhook::verify(headers, raw_body, signing_token))
        } else {
            match (self.webhook_token.as_deref(), api::first_header(headers, &["x-gitlab-token"])) {
                | (Some(expected), Some(received)) => {
                    if constant_time_eq(expected, received) {
                        Ok(())
                    } else {
                        Err(WebhookError::report(StatusCode::UNAUTHORIZED, "Invalid X-Gitlab-Token"))
                    }
                }
                | (Some(_), None) => Err(WebhookError::report(StatusCode::UNAUTHORIZED, "Missing X-Gitlab-Token header")),
                | (None, _) => Err(WebhookError::report(StatusCode::UNAUTHORIZED, "No webhook credential configured")),
            }
        }
    }
}
impl WebhookDelivery {
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
                let requested = check_requested(body);
                (human_authored && work_item && requested).then(|| format!("work-item-check:{}:{iid}:{note_id}", actor.project_id))
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
                head_sha: Some(head_sha),
                action: MergeRequestAction::Open | MergeRequestAction::Reopen | MergeRequestAction::Update,
                ..
            } => {
                let options = config.options.clone().with_internal_identifier(iid.to_string()).with_sha(head_sha);
                super::review::analyze_merge_request(&options, &config.analysis_options)
                    .await
                    .map(|outcome| {
                        info!("Processed GitLab merge request analysis operation: {outcome:?}");
                    })
            }
            | HookPayload::Note {
                actor,
                note_id,
                noteable_iid: Some(iid),
                noteable_type,
                ..
            } if noteable_type.eq_ignore_ascii_case("issue") || noteable_type.eq_ignore_ascii_case("task") => {
                let options = config.options.clone().with_internal_identifier(iid.to_string());
                analyze_work_item(&options, &actor, note_id).await.map(|report| {
                    info!("Processed GitLab work-item intake operation: {report:?}");
                })
            }
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
impl WebhookError {
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
impl fmt::Display for WebhookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}
impl core::error::Error for WebhookError {}
pub(super) fn check_requested(content: &str) -> bool {
    content.lines().any(|line| line.trim() == "/acorn check")
}
/// Authenticate and normalize an inbound GitLab webhook request into an HTTP status
pub(super) async fn receive(Extension(config): Extension<Arc<WebhookConfig>>, request: Request) -> Result<StatusCode, (StatusCode, String)> {
    Webhook::receive(config, request).await
}
