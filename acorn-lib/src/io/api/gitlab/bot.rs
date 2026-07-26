//! GitLab bot server for polling GitLab note events.
//!
//! The bot combines a small Axum HTTP server with a background poller. The
//! poller repeatedly calls the GitLab project events API with
//! `target_type=note` and an optional `after` timestamp. Each returned merge
//! request note event is converted into a [`MergeRequestNoteEvent`] using
//! `event.note.noteable_iid` as the merge request number.
//!
//! # Starting The Polling Server
//!
//! ```ignore
//! use acorn::io::api::Configuration;
//! use acorn::io::api::gitlab::{self, bot};
//! use core::time::Duration;
//!
//! async fn start_bot() -> color_eyre::Result<()> {
//!     let options = gitlab::Options::from_env().with_identifier("16689");
//!     let config = bot::Config::new(options, "127.0.0.1:3000".parse()?)
//!         .with_after("2026-07-06T00:00:00Z")
//!         .with_poll_interval(Duration::from_secs(30));
//!
//!     bot::Server::new(config).run().await?;
//!     Ok(())
//! }
//! ```
//!
//! # Handling Merge Request Notes
//!
//! ```ignore
//! use acorn::io::api::Configuration;
//! use acorn::io::api::gitlab::{self, bot};
//! use alloc::sync::Arc;
//!
//! async fn start_bot() -> color_eyre::Result<()> {
//!     let handler: bot::MergeRequestNoteHandler = Arc::new(|event| {
//!         println!("MR !{} received note {}", event.merge_request_iid, event.note_id);
//!         Ok(())
//!     });
//!
//!     let options = gitlab::Options::from_env().with_identifier("16689");
//!     let config = bot::Config::new(options, "127.0.0.1:3000".parse()?).with_handler(handler);
//!
//!     bot::GitLabBot::new(config).run().await?;
//!     Ok(())
//! }
//! ```
use super::database::OperationQueue;
use super::webhook::{self, WebhookConfig};
pub use super::webhook::{HookActor, HookPayload, MergeRequestAction, WebhookDelivery, WebhookOperation, WebhookOperationHandler};
use super::{events, EventDetails, EventsResponse, Options, WebhookOptions};
#[cfg(feature = "analysis")]
use crate::analyzer::CheckOptions;
use crate::io::api::Configuration;
use crate::io::ApiResult;
use crate::param;
use crate::prelude::Mutex;
use crate::util::Label;
use alloc::sync::Arc;
use axum::extract::State as ServerState;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use bon::Builder;
use color_eyre::eyre::{eyre, WrapErr};
use core::{fmt, net::SocketAddr, time::Duration};
use serde::Serialize;
use tokio::net::TcpListener;
use tracing::{error, info};
/// Callback used to process merge request note events.
///
/// Handlers are called once for each merge request note event returned by a
/// poll. Returning an error records the error in bot state and prevents that
/// polling cycle from being marked successful.
///
/// # Example
///
/// ```ignore
/// use acorn::io::api::gitlab::bot::{MergeRequestNoteEvent, MergeRequestNoteHandler};
/// use alloc::sync::Arc;
///
/// let handler: MergeRequestNoteHandler = Arc::new(|event: MergeRequestNoteEvent| {
///     println!("processing MR !{}", event.merge_request_iid);
///     Ok(())
/// });
/// ```
pub type MergeRequestNoteHandler = Arc<dyn Fn(MergeRequestNoteEvent) -> ApiResult<()> + Send + Sync + 'static>;
/// Shared mutable state used by the Axum handlers and background poller.
type SharedState = Arc<Mutex<State>>;
/// Runtime configuration for the GitLab bot server and poller.
///
/// The configuration provides the bind address for the Axum server, GitLab API
/// options, initial `after` timestamp, poll interval, and merge request note
/// handler.
///
/// # Example
///
/// ```ignore
/// use acorn::io::api::Configuration;
/// use acorn::io::api::gitlab::{self, bot};
/// use core::time::Duration;
///
/// let options = gitlab::Options::from_env().with_identifier("16689");
/// let config = bot::Config::new(options, "127.0.0.1:3000".parse()?)
///     .with_after("2026-07-06T00:00:00Z")
///     .with_poll_interval(Duration::from_secs(10));
/// # Ok::<(), color_eyre::Report>(())
/// ```
#[derive(Clone)]
pub struct Config {
    /// Address where the Axum server listens.
    address: SocketAddr,
    /// Current `after` timestamp for the next GitLab events request.
    after: String,
    /// GitLab API options used for each polling request.
    pub(super) options: Options,
    /// Duration between polling attempts.
    poll_interval: Duration,
    /// Handler called for each merge request note event.
    handler: MergeRequestNoteHandler,
    /// Whether to run the legacy Events API reconciliation poller
    polling_enabled: bool,
    /// Expected `X-Gitlab-Token` header value for legacy webhook authentication.
    webhook_token: Option<String>,
    /// Base64-encoded HMAC-SHA256 signing secret for Standard Webhooks authentication.
    webhook_signing_token: Option<String>,
    /// Numeric GitLab project ID to accept deliveries for; rejects all others when set.
    project_id: Option<u64>,
    /// Durable delivery and operation queue
    operation_queue: OperationQueue,
    /// Handler invoked by workers for claimed webhook operations
    pub(super) operation_handler: Option<WebhookOperationHandler>,
    /// Options applied to merge request analysis
    #[cfg(feature = "analysis")]
    pub(super) analysis_options: CheckOptions,
}
/// Merge request note event extracted from a GitLab event.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MergeRequestNoteEvent {
    /// Numeric GitLab event ID.
    pub event_id: u64,
    /// Numeric GitLab project ID.
    pub project_id: u64,
    /// Merge request IID from `event.note.noteable_iid`.
    pub merge_request_iid: u64,
    /// Numeric GitLab note ID.
    pub note_id: u64,
    /// Event creation timestamp.
    pub created_at: String,
    /// Note body text.
    pub body: String,
    /// Username of the note author.
    pub author_username: String,
}
/// Summary returned after one GitLab polling cycle.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PollSummary {
    /// Number of events returned by GitLab.
    pub event_count: usize,
    /// Number of merge request note events processed.
    pub processed_count: usize,
    /// Latest event timestamp seen in the response.
    pub latest_after: Option<String>,
}
/// Axum-backed GitLab bot that polls project note events.
///
/// A [`GitLabBot`] owns the HTTP server configuration, current polling state,
/// and merge request note handler. Use [`GitLabBot::run`] to start both the
/// Axum server and the background polling loop.
///
/// # Example
///
/// ```ignore
/// use acorn::io::api::Configuration;
/// use acorn::io::api::gitlab::{self, bot};
///
/// async fn start_bot() -> color_eyre::Result<()> {
///     let options = gitlab::Options::from_env().with_identifier("16689");
///     let config = bot::Config::new(options, "127.0.0.1:3000".parse()?);
///
///     bot::GitLabBot::new(config).run().await?;
///     Ok(())
/// }
/// ```
#[derive(Builder, Clone)]
#[builder(builder_type(vis = ""), start_fn(name = init, vis = ""))]
pub struct Server {
    config: Config,
    state: SharedState,
}
/// Shared state used by the Axum handlers and background poller
#[derive(Clone, Debug)]
struct State {
    /// Current `after` timestamp used for the next GitLab events request.
    after: String,
    /// Number of successful polling cycles.
    poll_count: u64,
    /// Number of merge request note events processed successfully.
    processed_count: u64,
    /// Last polling or processing error, if any.
    last_error: Option<String>,
}
/// Serializable bot state returned by the HTTP server.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StateSnapshot {
    /// Current `after` timestamp used for the next GitLab events request.
    pub after: String,
    /// Number of completed polling cycles.
    pub poll_count: u64,
    /// Total number of merge request note events processed.
    pub processed_count: u64,
    /// Last polling or processing error, if any.
    pub last_error: Option<String>,
}
impl Config {
    /// Create a bot configuration from GitLab API options and a bind address.
    ///
    /// The returned configuration uses a 30 second poll interval, no initial
    /// `after` timestamp, and a default handler that logs each merge request
    /// note event.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use acorn::io::api::Configuration;
    /// use acorn::io::api::gitlab::{self, bot};
    ///
    /// let options = gitlab::Options::from_env().with_identifier("16689");
    /// let config = bot::Config::new(options, "127.0.0.1:3000".parse()?);
    /// # Ok::<(), color_eyre::Report>(())
    /// ```
    pub fn new(options: Options, address: SocketAddr) -> Self {
        Self {
            address,
            after: String::new(),
            options,
            poll_interval: Duration::from_secs(30),
            handler: Arc::new(default_merge_request_note_handler),
            polling_enabled: true,
            webhook_token: None,
            webhook_signing_token: None,
            project_id: None,
            operation_queue: OperationQueue::configured(),
            operation_handler: None,
            #[cfg(feature = "analysis")]
            analysis_options: CheckOptions::default(),
        }
    }
    /// Return a copy of the configuration with an initial `after` timestamp.
    ///
    /// GitLab accepts an ISO date such as `2026-07-06` or an RFC3339 timestamp
    /// such as `2026-07-06T00:00:00Z`. After each successful poll, the bot
    /// replaces this value with the latest event timestamp returned by GitLab.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use acorn::io::api::gitlab::{self, bot};
    /// # let options = gitlab::Options::default();
    /// let config = bot::Config::new(options, "127.0.0.1:3000".parse()?)
    ///     .with_after("2026-07-06T00:00:00Z");
    /// # Ok::<(), color_eyre::Report>(())
    /// ```
    pub fn with_after(self, after: impl Into<String>) -> Self {
        Self { after: after.into(), ..self }
    }
    /// Return a copy of the configuration with a polling interval.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use acorn::io::api::gitlab::{self, bot};
    /// use core::time::Duration;
    /// # let options = gitlab::Options::default();
    /// let config = bot::Config::new(options, "127.0.0.1:3000".parse()?)
    ///     .with_poll_interval(Duration::from_secs(15));
    /// # Ok::<(), color_eyre::Report>(())
    /// ```
    pub fn with_poll_interval(self, poll_interval: Duration) -> Self {
        Self { poll_interval, ..self }
    }
    /// Return a copy of the configuration with a custom merge request note handler.
    ///
    /// The handler receives a normalized [`MergeRequestNoteEvent`] for each
    /// GitLab event whose target is a note on a merge request.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use acorn::io::api::gitlab::{self, bot};
    /// use alloc::sync::Arc;
    ///
    /// # let options = gitlab::Options::default();
    /// let handler: bot::MergeRequestNoteHandler = Arc::new(|event| {
    ///     println!("MR !{} note: {}", event.merge_request_iid, event.body);
    ///     Ok(())
    /// });
    /// let config = bot::Config::new(options, "127.0.0.1:3000".parse()?).with_handler(handler);
    /// # Ok::<(), color_eyre::Report>(())
    /// ```
    pub fn with_handler(self, handler: MergeRequestNoteHandler) -> Self {
        Self { handler, ..self }
    }
    /// Return a copy of the configuration with polling enabled or disabled
    pub fn with_polling_enabled(self, polling_enabled: bool) -> Self {
        Self { polling_enabled, ..self }
    }
    /// Return a copy of the configuration with a legacy `X-Gitlab-Token` webhook credential.
    ///
    /// When set, every inbound delivery that does not carry a `webhook-signature` header must
    /// present this exact value in `X-Gitlab-Token`; deliveries without it are rejected with
    /// `401 Unauthorized`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use acorn::io::api::gitlab::{self, bot};
    /// # let options = gitlab::Options::default();
    /// let config = bot::Config::new(options, "127.0.0.1:3000".parse()?)
    ///     .with_webhook_token("my-secret-token");
    /// # Ok::<(), color_eyre::Report>(())
    /// ```
    pub fn with_webhook_token(self, token: impl Into<String>) -> Self {
        Self {
            webhook_token: Some(token.into()),
            ..self
        }
    }
    /// Return a copy of the configuration with a Standard Webhooks HMAC signing token.
    ///
    /// The value must be the base64-encoded signing secret.  When set, deliveries that carry
    /// a `webhook-signature` header are verified with HMAC-SHA256; an invalid signature is
    /// rejected with `401 Unauthorized` and the legacy `X-Gitlab-Token` path is not tried.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use acorn::io::api::gitlab::{self, bot};
    /// # let options = gitlab::Options::default();
    /// let config = bot::Config::new(options, "127.0.0.1:3000".parse()?)
    ///     .with_webhook_signing_token("whsec_base64encodedSecret=");
    /// # Ok::<(), color_eyre::Report>(())
    /// ```
    pub fn with_webhook_signing_token(self, token: impl Into<String>) -> Self {
        Self {
            webhook_signing_token: Some(token.into()),
            ..self
        }
    }
    /// Return a copy of the configuration restricted to deliveries from a single GitLab project.
    ///
    /// Deliveries whose `project.id` does not match are rejected with `403 Forbidden`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use acorn::io::api::gitlab::{self, bot};
    /// # let options = gitlab::Options::default();
    /// let config = bot::Config::new(options, "127.0.0.1:3000".parse()?)
    ///     .with_project_id(16689);
    /// # Ok::<(), color_eyre::Report>(())
    /// ```
    pub fn with_project_id(self, id: u64) -> Self {
        Self {
            project_id: Some(id),
            ..self
        }
    }
    /// Return a copy of the configuration with an explicit durable operation queue
    pub fn with_operation_queue(self, operation_queue: OperationQueue) -> Self {
        Self { operation_queue, ..self }
    }
    /// Return a copy of the configuration with a webhook operation handler
    pub fn with_operation_handler(self, operation_handler: WebhookOperationHandler) -> Self {
        Self {
            operation_handler: Some(operation_handler),
            ..self
        }
    }
    /// Return a copy with explicit merge request analysis options
    #[cfg(feature = "analysis")]
    pub fn with_analysis_options(self, analysis_options: CheckOptions) -> Self {
        Self { analysis_options, ..self }
    }
    /// Return a copy configured for authenticated webhook ingress
    pub fn with_webhook_options(self, options: &WebhookOptions, project_id: Option<u64>) -> Self {
        Self {
            webhook_token: options.webhook_token.clone(),
            webhook_signing_token: options.signing_token.clone(),
            project_id,
            ..self
        }
    }
}
impl fmt::Display for Config {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.address)
    }
}
impl<'a> TryFrom<&'a EventDetails> for MergeRequestNoteEvent {
    type Error = &'static str;

    fn try_from(event: &'a EventDetails) -> Result<Self, Self::Error> {
        match (
            event.note.as_ref(),
            event.target_type.is_note(),
            event.action_name.is_commented() || event.action_name.is_commented_on(),
        ) {
            | (Some(note), true, true) if note.noteable_type.eq_ignore_ascii_case("MergeRequest") => match note.noteable_iid {
                | Some(merge_request_iid) => Ok(Self {
                    event_id: event.identifier,
                    project_id: event.project_id,
                    merge_request_iid,
                    note_id: note.identifier,
                    created_at: event.created_at.clone(),
                    body: note.body.clone(),
                    author_username: note.author.username.clone(),
                }),
                | None => Err("Merge request note is missing noteable_iid"),
            },
            | (None, _, _) => Err("Event has no note"),
            | (_, false, _) => Err("Event is not a note event"),
            | (_, _, false) => Err("Event action is not a comment"),
            | (Some(_), true, true) => Err("Note is not on a merge request"),
        }
    }
}
impl Server {
    /// Create a GitLab bot from configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use acorn::io::api::gitlab::{self, bot};
    ///
    /// let config = bot::Config::new(gitlab::Options::default(), "127.0.0.1:3000".parse()?);
    /// let bot = bot::GitLabBot::new(config);
    /// # Ok::<(), color_eyre::Report>(())
    /// ```
    pub fn new(config: Config) -> Self {
        let state = State {
            after: config.after.clone(),
            poll_count: 0,
            processed_count: 0,
            last_error: None,
        };
        Self::init().config(config).state(Arc::new(Mutex::new(state))).build()
    }
    /// Build the Axum router for the bot server.
    ///
    /// The router exposes `GET /health` for a simple liveness response and
    /// `GET /state` for a JSON [`BotStateSnapshot`]. This is useful when a
    /// caller wants to compose the bot routes into a larger Axum application
    /// instead of using [`GitLabBot::run`].
    ///
    /// # Example
    ///
    /// ```ignore
    /// use acorn::io::api::gitlab::{self, bot};
    ///
    /// let config = bot::Config::new(gitlab::Options::default(), "127.0.0.1:3000".parse()?);
    /// let router = bot::GitLabBot::new(config).router();
    /// # Ok::<(), color_eyre::Report>(())
    /// ```
    pub fn router(&self) -> Router {
        let Server { config, state } = self;
        let webhook_config = Arc::new(
            WebhookConfig::init()
                .maybe_webhook_token(config.webhook_token.as_deref())
                .maybe_webhook_signing_token(config.webhook_signing_token.as_deref())
                .maybe_project_id(config.project_id)
                .operation_queue(config.operation_queue.clone())
                .build(),
        );
        Router::new()
            .route("/health", get(Self::health))
            .route("/state", get(Self::state_handler))
            .route("/webhooks/gitlab", post(webhook::receive))
            .with_state(Arc::clone(state))
            .layer(Extension(webhook_config))
    }
    /// Run the Axum server and background GitLab polling task.
    ///
    /// This method binds the configured address, spawns a background task that
    /// calls [`GitLabBot::poll_once`] every configured interval, and then serves
    /// the bot router until the server is shut down by the runtime.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use acorn::io::api::Configuration;
    /// use acorn::io::api::gitlab::{self, bot};
    /// use core::time::Duration;
    ///
    /// async fn start_bot() -> color_eyre::Result<()> {
    ///     let options = gitlab::Options::from_env().with_identifier("16689");
    ///     let config = bot::Config::new(options, "0.0.0.0:3000".parse()?)
    ///         .with_after("2026-07-06T00:00:00Z")
    ///         .with_poll_interval(Duration::from_secs(30));
    ///
    ///     bot::GitLabBot::new(config).run().await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when the configured address cannot be bound or when the
    /// Axum server exits with an error.
    pub async fn run(self) -> ApiResult<()> {
        let Server { config, state } = self;
        match TcpListener::bind(config.address)
            .await
            .wrap_err_with(|| format!("Failed to bind GitLab bot server to {config}"))
        {
            | Ok(listener) => {
                let server = Arc::new(Self::init().config(config).state(state).build());
                if server.config.polling_enabled {
                    let poller = Arc::clone(&server);
                    tokio::spawn(async move { poller.poll_forever().await });
                }
                let worker = Arc::clone(&server);
                tokio::spawn(async move { worker.work_forever().await });
                info!("GitLab bot server listening on {}", server.config);
                axum::serve(listener, server.router()).await.wrap_err("GitLab bot server failed")
            }
            | Err(why) => Err(why),
        }
    }
    /// Poll GitLab once and process every merge request note event in the response.
    ///
    /// The request is sent with `target_type=note`. If state contains a non-empty
    /// `after` value, the request also includes that timestamp. After the
    /// response is processed, state is advanced to the latest event timestamp in
    /// the response.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use acorn::io::api::Configuration;
    /// use acorn::io::api::gitlab::{self, bot};
    ///
    /// async fn poll_once() -> color_eyre::Result<()> {
    ///     let options = gitlab::Options::from_env().with_identifier("16689");
    ///     let bot = bot::GitLabBot::new(bot::Config::new(options, "127.0.0.1:0".parse()?));
    ///     let summary = bot.poll_once().await?;
    ///     println!("processed {} notes", summary.processed_count);
    ///     Ok(())
    /// }
    /// ```
    pub async fn poll_once(&self) -> ApiResult<PollSummary> {
        let Server { config, .. } = self;
        match self.snapshot() {
            | Ok(snapshot) => {
                let params = if snapshot.after.trim().is_empty() {
                    vec![param!(KeyValuePair, "target_type", "note")]
                } else {
                    vec![
                        param!(KeyValuePair, "target_type", "note"),
                        param!(KeyValuePair, "after", snapshot.after.as_str()),
                    ]
                };
                let options = config.options.clone().with_params(params);
                match events(&options).await {
                    | Ok(response) => self.process_events(response),
                    | Err(why) => Err(why),
                }
            }
            | Err(why) => Err(why),
        }
    }
    /// Return the current bot state snapshot.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use acorn::io::api::gitlab::{self, bot};
    ///
    /// let bot = bot::GitLabBot::new(bot::Config::new(gitlab::Options::default(), "127.0.0.1:0".parse()?));
    /// let state = bot.snapshot()?;
    /// println!("next after value: {}", state.after);
    /// # Ok::<(), color_eyre::Report>(())
    /// ```
    pub fn snapshot(&self) -> ApiResult<StateSnapshot> {
        let Server { state, .. } = self;
        state
            .lock()
            .map(|s| s.snapshot())
            .map_err(|why| eyre!("GitLab bot state lock is poisoned: {why}"))
    }
    /// Poll GitLab forever at the configured interval
    async fn poll_forever(&self) {
        let Server { config, .. } = &self;
        let mut interval = tokio::time::interval(config.poll_interval);
        loop {
            interval.tick().await;
            match self.poll_once().await {
                | Ok(summary) => {
                    // TODO: Process events
                    println!("=> [WIP] GitLab bot poll summary: {summary:?}")
                }
                | Err(why) => {
                    error!("GitLab bot polling failed: {why}");
                    if let Err(lock_error) = self.record_error(why.to_string()) {
                        error!("=> {} Failed to record GitLab bot error — {lock_error}", Label::fail());
                    }
                }
            }
        }
    }
    async fn work_forever(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            if let Err(why) = self.process_next().await {
                error!("=> {} GitLab bot worker — {why}", Label::fail());
                if let Err(lock_error) = self.record_error(why.to_string()) {
                    error!("=> {} Failed to record GitLab bot worker error — {lock_error}", Label::fail());
                }
            }
        }
    }
    /// Claim and process one durable webhook operation
    pub async fn process_next(&self) -> ApiResult<bool> {
        let Server { config, .. } = self;
        match config.operation_queue.claim_next(chrono::Duration::minutes(5)) {
            | Ok(Some(operation)) => {
                let result = match serde_json::from_str::<WebhookDelivery>(&operation.event_json) {
                    | Ok(delivery) => match delivery.process(config).await {
                        | Ok(()) => config.operation_queue.succeed(&operation.operation_key),
                        | Err(why) => Err(why),
                    },
                    | Err(why) => Err(eyre!("Failed to decode normalized webhook operation — {why}")),
                };
                result
                    .map(|_| true)
                    .or_else(|why| config.operation_queue.fail(&operation, &why.to_string()).and_then(|_| Err(why)))
            }
            | Ok(None) => Ok(false),
            | Err(why) => Err(why),
        }
    }
    /// Process a GitLab events response and update bot state.
    pub(crate) fn process_events(&self, response: EventsResponse) -> ApiResult<PollSummary> {
        let Server { config, .. } = self;
        let latest_after = response.iter().map(|event| event.created_at.as_str()).max().map(str::to_string);
        let events: Vec<_> = response.iter().filter_map(|e| MergeRequestNoteEvent::try_from(e).ok()).collect();
        let summary = PollSummary {
            event_count: response.len(),
            processed_count: events.len(),
            latest_after,
        };
        match events.iter().cloned().try_for_each(|event| (config.handler)(event)) {
            | Ok(()) => match self.record_success(&summary) {
                | Ok(()) => Ok(summary),
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    /// Record a successful polling cycle in shared state
    fn record_success(&self, summary: &PollSummary) -> ApiResult<()> {
        let Server { state, .. } = self;
        match state.lock() {
            | Ok(mut guard) => {
                let after = summary.latest_after.clone().unwrap_or_else(|| guard.after.clone());
                *guard = State {
                    after,
                    poll_count: guard.poll_count.saturating_add(1),
                    processed_count: guard.processed_count.saturating_add(summary.processed_count as u64),
                    last_error: None,
                };
                Ok(())
            }
            | Err(why) => Err(eyre!("GitLab bot state lock is poisoned: {why}")),
        }
    }
    /// Record a polling or processing error in shared state
    fn record_error(&self, error: String) -> ApiResult<()> {
        let Server { state, .. } = self;
        match state.lock() {
            | Ok(mut guard) => {
                *guard = State {
                    last_error: Some(error),
                    ..guard.clone()
                };
                Ok(())
            }
            | Err(why) => Err(eyre!("GitLab bot state lock is poisoned: {why}")),
        }
    }
    /// Axum health check handler
    async fn health() -> &'static str {
        "ok"
    }
    /// Axum state endpoint handler
    async fn state_handler(ServerState(state): ServerState<SharedState>) -> Result<Json<StateSnapshot>, (StatusCode, String)> {
        state
            .lock()
            .map(|s| Json(s.snapshot()))
            .map_err(|why| (StatusCode::INTERNAL_SERVER_ERROR, format!("GitLab bot state lock is poisoned — {why}")))
    }
}
impl State {
    /// Convert internal bot state into a serializable public snapshot
    fn snapshot(&self) -> StateSnapshot {
        let State {
            after,
            poll_count,
            processed_count,
            last_error,
        } = self;
        StateSnapshot {
            after: after.clone(),
            poll_count: *poll_count,
            processed_count: *processed_count,
            last_error: last_error.clone(),
        }
    }
}
/// Default merge request note handler used when no custom handler is provided.
fn default_merge_request_note_handler(event: MergeRequestNoteEvent) -> ApiResult<()> {
    info!(
        "Processing GitLab merge request note event {} for MR !{}",
        event.event_id, event.merge_request_iid
    );
    Ok(())
}
