//! GitLab bot server for polling GitLab note events.
//!
//! The bot combines a small HTTP server with a background poller. The
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
pub use super::webhook::{HookActor, HookPayload, MergeRequestAction, WebhookDelivery, WebhookOperation, WebhookOperationHandler};
use super::{
    events,
    webhook::{self, WebhookConfig},
    DeploymentConfig, EventDetails, EventsResponse, Options, Package, PackageLifecycle, PackageOptions, WebhookOptions,
};
#[cfg(feature = "analysis")]
use crate::analyzer::host::context::AnalysisContext;
use crate::{
    io::{
        api::{
            self,
            json_rpc::{InvocationContext, InvocationOrigin, JsonRpcOperationRegistry, MethodName, OperationDefinition, OperationEffects},
            powerautomate::{FormSubmission, PowerAutomateCallbackResolver, PowerAutomateConfig, PowerAutomateProvider},
            webhooks::{
                store::{CallbackWorker, OperationQueue, OperationWorker},
                ArtifactEvent, WebhookRuntime, WebhookRuntimeState,
            },
            Configuration,
        },
        http::ReqwestHttpService,
        ApiResult,
    },
    param,
    util::constants::app::WEBHOOK_STALE_CLAIM_AFTER,
};
use acorn_host::terminal::Label;
use alloc::sync::Arc;
use axum::{
    extract::State as ServerState,
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use bon::Builder;
use color_eyre::eyre::{eyre, WrapErr};
use core::{fmt, net::SocketAddr, time::Duration};
use serde::Serialize;
use serde_json::Value;
#[cfg(test)]
use std::env::temp_dir;
use std::sync::Mutex;
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
/// Shared mutable state used by the HTTP handlers and background poller.
type SharedState = Arc<Mutex<State>>;
/// Runtime configuration for the GitLab bot server and poller.
///
/// The configuration provides the bind address for the HTTP server, GitLab API
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
    /// Address where the HTTP server listens.
    address: SocketAddr,
    /// Current `after` timestamp for the next GitLab events request.
    after: String,
    /// GitLab API options used for each polling request.
    pub(super) options: Options,
    /// Deployment-owned workflow and effect policy.
    pub(super) deployment: DeploymentConfig,
    /// Duration between polling attempts.
    poll_interval: Duration,
    /// Maximum time to wait for claimed webhook work during shutdown.
    shutdown_grace: Duration,
    /// Handler called for each merge request note event.
    handler: MergeRequestNoteHandler,
    /// Whether to run the legacy Events API reconciliation poller
    polling_enabled: bool,
    /// Expected `X-Gitlab-Token` header value for legacy webhook authentication.
    webhook_token: Option<api::Secret>,
    /// Base64-encoded HMAC-SHA256 signing secret for Standard Webhooks authentication.
    webhook_signing_token: Option<api::Secret>,
    /// Numeric GitLab project ID to accept deliveries for; rejects all others when set.
    project_id: Option<u64>,
    /// Durable delivery and operation queue
    pub(super) operation_queue: OperationQueue,
    /// Handler invoked by workers for claimed webhook operations
    pub(super) operation_handler: Option<WebhookOperationHandler>,
    /// Optional signed PowerAutomate form provider.
    pub(super) powerautomate: Option<PowerAutomateProvider>,
    /// Options applied to merge request analysis
    #[cfg(feature = "analysis")]
    pub(super) analysis_options: AnalysisContext,
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
#[derive(Clone)]
struct RouterState {
    bot: SharedState,
    webhook: WebhookRuntime,
}
/// HTTP-backed GitLab bot that polls project note events.
///
/// A [`Server`] owns the HTTP server configuration, current polling state, and merge request note handler.
/// Use [`Server::run`] to start both the HTTP server and the background polling loop.
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
///     bot::GitLabBot::new(config).run().await?;
///     Ok(())
/// }
/// ```
#[derive(Builder, Clone)]
#[builder(builder_type(vis = ""), start_fn(name = init, vis = ""))]
pub struct Server {
    config: Config,
    runtime: WebhookRuntime,
    state: SharedState,
}
/// Shared state used by the HTTP handlers and background poller.
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
    /// Shared webhook readiness, drain, and outcome state.
    pub webhook: WebhookRuntimeState,
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
            options,
            ..DeploymentConfig::default().into()
        }
    }
    /// Return a copy of the configuration with a bind address.
    pub fn with_address(self, address: SocketAddr) -> Self {
        Self { address, ..self }
    }
    fn registry(&self) -> ApiResult<JsonRpcOperationRegistry> {
        let effects = OperationEffects {
            filesystem_read: true,
            filesystem_write: true,
            mutation: true,
            network_read: true,
            ..OperationEffects::default()
        };
        let registry = [
            MethodName::from(["webhooks", "gitlab", "merge-request"]),
            MethodName::from(["webhooks", "gitlab", "note"]),
        ]
        .into_iter()
        .try_fold(JsonRpcOperationRegistry::default(), |registry, method| {
            OperationDefinition::new::<Value, Value>(method, effects)
                .map_err(color_eyre::Report::from)
                .and_then(|definition| {
                    let config = self.clone();
                    registry.register(definition, move |params: Value, _| {
                        let config = config.clone();
                        async move {
                            match serde_json::from_value::<WebhookDelivery>(params) {
                                | Ok(delivery) => delivery.process(&config).await.map(|()| Value::Null),
                                | Err(why) => Err(eyre!("Failed to decode normalized GitLab webhook operation — {why}")),
                            }
                        }
                    })
                })
        });
        match (registry, self.powerautomate.clone()) {
            | (Ok(registry), Some(provider)) => {
                let definition = OperationDefinition::new::<Value, Value>(MethodName::from(["webhooks", "powerautomate", "form"]), effects)
                    .map_err(color_eyre::Report::from);
                match definition {
                    | Ok(definition) => {
                        let options = self.options.clone().with_identifier(provider.config().gitlab_project.clone());
                        let queue = self.operation_queue.clone();
                        registry.register(definition, move |params: Value, _| {
                            let options = options.clone();
                            let provider = provider.clone();
                            let queue = queue.clone();
                            async move {
                                match serde_json::from_value::<FormSubmission>(params) {
                                    | Ok(submission) => {
                                        let latest = match Package::new(
                                            options.clone(),
                                            PackageOptions::new(&provider.config().package, &submission.project_id),
                                        ) {
                                            | Ok(package) => package.download().await,
                                            | Err(why) => Err(why),
                                        };
                                        let prepared = match latest {
                                            | Ok(latest) => {
                                                let cutoff = latest.as_ref().map(|manifest| manifest.through.as_str());
                                                let folder = provider.routing().resolve_folder(&submission.project_id);
                                                let path = folder
                                                    .as_ref()
                                                    .ok()
                                                    .and_then(|folder| folder.destination.repository_path().map(str::to_owned))
                                                    .ok_or_else(|| {
                                                        eyre!(
                                                            "Folder '{}' does not resolve into the configured GitLab project",
                                                            submission.project_id
                                                        )
                                                    });
                                                let entry = submission.entry_after(provider.routing(), cutoff);
                                                let branch = submission.intake_branch();
                                                match (path, entry, branch) {
                                                    | (Ok(path), Ok(entry), Ok(branch)) => Ok((path, entry, branch)),
                                                    | (Err(why), _, _) | (_, Err(why), _) | (_, _, Err(why)) => Err(why),
                                                }
                                            }
                                            | Err(why) => Err(why),
                                        };
                                        match prepared {
                                            | Ok((path, entry, branch)) => options
                                                .clone()
                                                .with_branch(branch)
                                                .with_workflow("logbook-intake")
                                                .publish_logbook_entry(entry, &queue, &path)
                                                .await
                                                .and_then(|result| {
                                                    let merge_request_url = result.publication.merge_request.web_url.clone();
                                                    let created = ArtifactEvent::MergeRequestCreated {
                                                        merge_request_url: merge_request_url.clone(),
                                                        project_id: submission.project_id.clone(),
                                                    }
                                                    .enqueue(
                                                        &queue,
                                                        &provider.config().callback_destination,
                                                        &submission.submission_id,
                                                    );
                                                    let review = match result.candidates.0.is_empty() {
                                                        | true => Ok(false),
                                                        | false => ArtifactEvent::ReviewNeeded {
                                                            merge_request_url,
                                                            project_id: submission.project_id.clone(),
                                                        }
                                                        .enqueue(
                                                            &queue,
                                                            &provider.config().callback_destination,
                                                            &format!("{}:review", submission.submission_id),
                                                        ),
                                                    };
                                                    created.and(review).map(|_| Value::Null)
                                                }),
                                            | Err(why) => Err(why),
                                        }
                                    }
                                    | Err(why) => Err(eyre!("Failed to decode normalized PowerAutomate form operation — {why}")),
                                }
                            }
                        })
                    }
                    | Err(why) => Err(why),
                }
            }
            | (Ok(registry), None) => Ok(registry),
            | (Err(why), _) => Err(why),
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
    /// Return a copy of the configuration with a bounded webhook shutdown grace period.
    pub fn with_shutdown_grace(self, shutdown_grace: Duration) -> Self {
        Self { shutdown_grace, ..self }
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
    /// Return a copy with validated deployment-owned repository workflow policy.
    pub fn with_deployment(self, deployment: DeploymentConfig) -> ApiResult<Self> {
        deployment.validate().map(|()| Self { deployment, ..self })
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
            webhook_token: Some(api::Secret::from(token.into())),
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
            webhook_signing_token: Some(api::Secret::from(token.into())),
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
    /// Return a copy configured for signed PowerAutomate form intake and callbacks.
    pub fn with_powerautomate(self, config: PowerAutomateConfig, routing: crate::io::config::application::FolderRegistry) -> ApiResult<Self> {
        PowerAutomateProvider::from_config(config, routing).map(|powerautomate| Self {
            powerautomate: Some(powerautomate),
            ..self
        })
    }
    /// Return a copy with explicit merge request analysis options
    #[cfg(feature = "analysis")]
    pub fn with_analysis_options(self, analysis_options: AnalysisContext) -> Self {
        Self { analysis_options, ..self }
    }
    /// Return a copy of the configuration with GitLab API options.
    pub fn with_options(self, options: Options) -> Self {
        Self { options, ..self }
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
impl From<DeploymentConfig> for Config {
    fn from(deployment: DeploymentConfig) -> Self {
        Self {
            address: SocketAddr::from(([127, 0, 0, 1], 3000)),
            after: String::new(),
            options: Options::default(),
            deployment,
            poll_interval: Duration::from_secs(30),
            shutdown_grace: Duration::from_secs(30),
            handler: Arc::new(default_merge_request_note_handler),
            polling_enabled: true,
            webhook_token: None,
            webhook_signing_token: None,
            project_id: None,
            operation_queue: default_operation_queue(),
            operation_handler: None,
            powerautomate: None,
            #[cfg(feature = "analysis")]
            analysis_options: AnalysisContext::default(),
        }
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
    /// Stop accepting webhook work and wait for active claims up to the configured grace period.
    pub async fn drain(&self) -> ApiResult<WebhookRuntimeState> {
        self.runtime.drain(self.config.shutdown_grace).await
    }
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
        Self::init()
            .config(config)
            .runtime(WebhookRuntime::default())
            .state(Arc::new(Mutex::new(state)))
            .build()
    }
    /// Build the HTTP router for the bot server.
    ///
    /// The router exposes `GET /health` for a simple liveness response and `GET /state` for a JSON [`StateSnapshot`].
    /// This is useful when a caller wants to compose the bot routes into a larger HTTP application instead of using [`Server::run`].
    ///
    /// ### Example
    /// ```ignore
    /// use acorn::io::api::gitlab::{self, bot};
    ///
    /// let config = bot::Config::new(gitlab::Options::default(), "127.0.0.1:3000".parse()?);
    /// let router = bot::GitLabBot::new(config).router();
    /// # Ok::<(), color_eyre::Report>(())
    /// ```
    pub fn router(&self) -> Router {
        let Server { config, runtime, state } = self;
        if let Err(why) = runtime.initialize(&config.operation_queue, WEBHOOK_STALE_CLAIM_AFTER) {
            error!("Webhook runtime initialization failed: {why}");
        }
        let webhook_config = Arc::new(
            WebhookConfig::init()
                .maybe_webhook_token(config.webhook_token.clone())
                .maybe_webhook_signing_token(config.webhook_signing_token.clone())
                .maybe_project_id(config.project_id)
                .operation_queue(config.operation_queue.clone())
                .runtime(runtime.clone())
                .build(),
        );
        let router = Router::new()
            .route("/health", get(Self::health))
            .route("/ready", get(Self::ready))
            .route("/state", get(Self::state_handler))
            .route("/webhooks/gitlab", post(webhook::receive))
            .with_state(RouterState {
                bot: Arc::clone(state),
                webhook: runtime.clone(),
            })
            .layer(Extension(webhook_config));
        match &config.powerautomate {
            | Some(provider) => router.merge(provider.clone().router(config.operation_queue.clone(), runtime.clone())),
            | None => router,
        }
    }
    /// Run the HTTP server and background GitLab polling task.
    ///
    /// This method binds the configured address, spawns a background task that
    /// calls [`Server::poll_once`] every configured interval, and then serves
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
    /// HTTP server exits with an error.
    pub async fn run(self) -> ApiResult<()> {
        let Server { config, runtime, state } = self;
        match TcpListener::bind(config.address)
            .await
            .wrap_err_with(|| format!("Failed to bind GitLab bot server to {config}"))
        {
            | Ok(listener) => {
                let server = Arc::new(Self::init().config(config).runtime(runtime).state(state).build());
                if let Err(why) = server.runtime.initialize(&server.config.operation_queue, WEBHOOK_STALE_CLAIM_AFTER) {
                    error!("Webhook runtime initialization failed: {why}");
                }
                let mut tasks = Vec::new();
                if server.config.polling_enabled {
                    let poller = Arc::clone(&server);
                    tasks.push(tokio::spawn(async move { poller.poll_forever().await }));
                }
                let worker = Arc::clone(&server);
                tasks.push(tokio::spawn(async move { worker.work_forever().await }));
                info!("GitLab bot server listening on {}", server.config);
                let shutdown = Arc::clone(&server);
                let result = axum::serve(listener, server.router())
                    .with_graceful_shutdown(async move {
                        let _ = tokio::signal::ctrl_c().await;
                        let _ = shutdown.runtime.drain(shutdown.config.shutdown_grace).await;
                    })
                    .await
                    .wrap_err("GitLab bot server failed");
                tasks.into_iter().for_each(|task| task.abort());
                result
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
        let Server { runtime, state, .. } = self;
        state
            .lock()
            .map_err(|why| eyre!("GitLab bot state lock is poisoned: {why}"))
            .and_then(|state| runtime.state().map(|webhook| state.snapshot(webhook)))
    }
    /// Poll GitLab forever at the configured interval
    async fn poll_forever(&self) {
        let Server { config, .. } = &self;
        let mut interval = tokio::time::interval(config.poll_interval);
        while self.runtime.is_accepting() {
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
        while self.runtime.is_accepting() {
            interval.tick().await;
            if let Err(why) = self.process_next().await {
                error!("=> {} GitLab bot worker — {why}", Label::fail());
                if let Err(lock_error) = self.record_error(why.to_string()) {
                    error!("=> {} Failed to record GitLab bot worker error — {lock_error}", Label::fail());
                }
            }
            if let Err(why) = self.process_callback_next().await {
                error!("=> {} PowerAutomate callback worker — {why}", Label::fail());
                if let Err(lock_error) = self.record_error(why.to_string()) {
                    error!("=> {} Failed to record callback worker error — {lock_error}", Label::fail());
                }
            }
        }
    }
    async fn process_callback_next(&self) -> ApiResult<bool> {
        let Server { config, .. } = self;
        match &config.powerautomate {
            | Some(provider) => {
                CallbackWorker::new(
                    config.operation_queue.clone(),
                    ReqwestHttpService::default(),
                    PowerAutomateCallbackResolver::new(provider.config().callback_destination.clone(), provider.routing()),
                    WEBHOOK_STALE_CLAIM_AFTER,
                )
                .with_runtime(self.runtime.clone())
                .run_once()
                .await
            }
            | None => Ok(false),
        }
    }
    /// Claim and process one durable webhook operation
    pub async fn process_next(&self) -> ApiResult<bool> {
        let Server { config, .. } = self;
        let context = InvocationContext {
            allow_mutation: true,
            origin: InvocationOrigin::Webhook,
            principal: crate::io::api::json_rpc::Principal {
                identifier: "gitlab-webhook".to_string(),
            },
            ..InvocationContext::default()
        };
        match config.registry() {
            | Ok(registry) => {
                OperationWorker::new(config.operation_queue.clone(), registry, context, WEBHOOK_STALE_CLAIM_AFTER)
                    .with_runtime(self.runtime.clone())
                    .run_once()
                    .await
            }
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
    /// Health check handler.
    async fn health() -> &'static str {
        "ok"
    }
    /// Readiness handler backed by the shared webhook runtime.
    async fn ready(ServerState(state): ServerState<RouterState>) -> StatusCode {
        match state.webhook.is_ready() {
            | true => StatusCode::OK,
            | false => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
    /// State endpoint handler.
    async fn state_handler(ServerState(state): ServerState<RouterState>) -> Result<Json<StateSnapshot>, (StatusCode, String)> {
        state
            .bot
            .lock()
            .map_err(|why| (StatusCode::INTERNAL_SERVER_ERROR, format!("GitLab bot state lock is poisoned — {why}")))
            .and_then(|bot| {
                state
                    .webhook
                    .state()
                    .map(|webhook| Json(bot.snapshot(webhook)))
                    .map_err(|why| (StatusCode::INTERNAL_SERVER_ERROR, why.to_string()))
            })
    }
}
impl State {
    /// Convert internal bot state into a serializable public snapshot
    fn snapshot(&self, webhook: WebhookRuntimeState) -> StateSnapshot {
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
            webhook,
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
#[cfg(not(test))]
fn default_operation_queue() -> OperationQueue {
    OperationQueue::configured()
}
#[cfg(test)]
fn default_operation_queue() -> OperationQueue {
    OperationQueue::from(temp_dir().join(format!("acorn-gitlab-bot-{}.db", nanoid::nanoid!())))
}
