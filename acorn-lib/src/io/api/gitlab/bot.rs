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
//!     bot::GitLabBot::new(config).run().await?;
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
use super::{events, EventAction, EventDetails, EventsResponse, Options, TargetType};
use crate::io::api::Configuration;
use crate::io::ApiResult;
use crate::param;
use alloc::sync::Arc;
use axum::extract::State as ServerState;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use color_eyre::eyre::{eyre, WrapErr};
use core::net::SocketAddr;
use core::time::Duration;
use serde::Serialize;
use std::sync::Mutex;
use tokio::net::TcpListener;
use tracing::{debug, error, info};

/// Shared mutable state used by the Axum handlers and background poller.
type SharedState = Arc<Mutex<State>>;
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
#[derive(Clone)]
pub struct GitLabBot {
    config: Config,
    state: SharedState,
}
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
    options: Options,
    /// Duration between polling attempts.
    poll_interval: Duration,
    /// Handler called for each merge request note event.
    handler: MergeRequestNoteHandler,
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
}
impl GitLabBot {
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
        Self {
            config,
            state: Arc::new(Mutex::new(state)),
        }
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
        Router::new()
            .route("/health", get(health))
            .route("/state", get(state))
            .with_state(self.state.clone())
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
        match TcpListener::bind(self.config.address)
            .await
            .wrap_err_with(|| format!("Failed to bind GitLab bot server to {}", self.config.address))
        {
            | Ok(listener) => {
                let poller = self.clone();
                tokio::spawn(async move { poller.poll_forever().await });
                info!("GitLab bot server listening on {}", self.config.address);
                axum::serve(listener, self.router()).await.wrap_err("GitLab bot server failed")
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
    ///
    /// # Errors
    ///
    /// Returns an error when GitLab cannot be queried, the bot state cannot be
    /// accessed, or the configured merge request note handler fails.
    pub async fn poll_once(&self) -> ApiResult<PollSummary> {
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
                let options = self.config.options.clone().with_params(params);
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
    ///
    /// # Errors
    ///
    /// Returns an error if the shared bot state lock is poisoned.
    pub fn snapshot(&self) -> ApiResult<StateSnapshot> {
        self.state
            .lock()
            .map(|state| state.snapshot())
            .map_err(|why| eyre!("GitLab bot state lock is poisoned: {why}"))
    }
    /// Poll GitLab forever at the configured interval.
    async fn poll_forever(self) {
        let mut interval = tokio::time::interval(self.config.poll_interval);
        loop {
            interval.tick().await;
            match self.poll_once().await {
                | Ok(summary) => debug!("GitLab bot poll summary: {summary:?}"),
                | Err(why) => {
                    error!("GitLab bot polling failed: {why}");
                    if let Err(lock_error) = self.record_error(why.to_string()) {
                        error!("Failed to record GitLab bot error: {lock_error}");
                    }
                }
            }
        }
    }
    /// Process a GitLab events response and update bot state.
    fn process_events(&self, response: EventsResponse) -> ApiResult<PollSummary> {
        let latest_after = response.iter().map(|event| event.created_at.as_str()).max().map(str::to_string);
        let events: Vec<_> = response.iter().filter_map(|e| MergeRequestNoteEvent::try_from(e).ok()).collect();
        let summary = PollSummary {
            event_count: response.len(),
            processed_count: events.len(),
            latest_after,
        };
        match events.iter().cloned().try_for_each(|event| (self.config.handler)(event)) {
            | Ok(()) => match self.record_success(&summary) {
                | Ok(()) => Ok(summary),
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    /// Record a successful polling cycle in shared state.
    fn record_success(&self, summary: &PollSummary) -> ApiResult<()> {
        match self.state.lock() {
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
    /// Record a polling or processing error in shared state.
    fn record_error(&self, error: String) -> ApiResult<()> {
        match self.state.lock() {
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
}
impl<'a> TryFrom<&'a EventDetails> for MergeRequestNoteEvent {
    type Error = &'static str;

    fn try_from(event: &'a EventDetails) -> Result<Self, Self::Error> {
        match (
            event.note.as_ref(),
            matches!(event.target_type, TargetType::Note),
            matches!(event.action_name, EventAction::Commented | EventAction::CommentedOn),
        ) {
            | (Some(note), true, true) if note.noteable_type.eq_ignore_ascii_case("MergeRequest") => Ok(Self {
                event_id: event.identifier,
                project_id: event.project_id,
                merge_request_iid: note.noteable_iid,
                note_id: note.identifier,
                created_at: event.created_at.clone(),
                body: note.body.clone(),
                author_username: note.author.username.clone(),
            }),
            | (None, _, _) => Err("Event has no note"),
            | (_, false, _) => Err("Event is not a note event"),
            | (_, _, false) => Err("Event action is not a comment"),
            | (Some(_), true, true) => Err("Note is not on a merge request"),
        }
    }
}
impl State {
    /// Convert internal bot state into a serializable public snapshot.
    fn snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            after: self.after.clone(),
            poll_count: self.poll_count,
            processed_count: self.processed_count,
            last_error: self.last_error.clone(),
        }
    }
}
/// Axum health check handler.
async fn health() -> &'static str {
    "ok"
}
/// Axum state endpoint handler.
async fn state(ServerState(state): ServerState<SharedState>) -> Result<Json<StateSnapshot>, (StatusCode, String)> {
    state
        .lock()
        .map(|state| Json(state.snapshot()))
        .map_err(|why| (StatusCode::INTERNAL_SERVER_ERROR, format!("GitLab bot state lock is poisoned — {why}")))
}
/// Default merge request note handler used when no custom handler is provided.
fn default_merge_request_note_handler(event: MergeRequestNoteEvent) -> ApiResult<()> {
    info!(
        "Processing GitLab merge request note event {} for MR !{}",
        event.event_id, event.merge_request_iid
    );
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::io::api::gitlab::{NoteMetadata, UserMetadata};

    fn user() -> UserMetadata {
        UserMetadata {
            avatar_url: "https://example.com/avatar.png".to_string(),
            identifier: 1,
            locked: false,
            name: "Test User".to_string(),
            email: None,
            state: "active".to_string(),
            username: "tester".to_string(),
            url: "https://example.com/tester".to_string(),
        }
    }
    fn note(noteable_type: &str, noteable_iid: u64) -> NoteMetadata {
        NoteMetadata {
            identifier: 10,
            note_type: None,
            body: "hello".to_string(),
            author: user(),
            created_at: "2026-07-06T00:00:01Z".to_string(),
            updated_at: "2026-07-06T00:00:01Z".to_string(),
            system: false,
            noteable_id: 20,
            noteable_iid,
            noteable_type: noteable_type.to_string(),
            project_id: 30,
            resolvable: false,
            confidential: false,
            internal: false,
            imported: false,
            imported_from: "none".to_string(),
            commands_changes: serde_json::json!({}),
        }
    }
    fn event(created_at: &str, noteable_type: &str) -> EventDetails {
        EventDetails {
            identifier: 100,
            project_id: 30,
            action_name: EventAction::Commented,
            target_id: 10,
            target_iid: 11,
            target_type: TargetType::Note,
            author_id: 1,
            target_title: "note".to_string(),
            created_at: created_at.to_string(),
            author: user(),
            imported: false,
            imported_from: "none".to_string(),
            push_data: None,
            author_username: "tester".to_string(),
            note: Some(note(noteable_type, 42)),
        }
    }

    #[test]
    fn extracts_merge_request_iid_from_note_event() {
        let extracted = MergeRequestNoteEvent::try_from(&event("2026-07-06T00:00:01Z", "MergeRequest")).unwrap();
        assert_eq!(extracted.merge_request_iid, 42);
    }
    #[test]
    fn ignores_non_merge_request_note_event() {
        let extracted = MergeRequestNoteEvent::try_from(&event("2026-07-06T00:00:01Z", "Issue"));
        assert!(extracted.is_err());
    }
    #[test]
    fn processing_updates_after_to_latest_event_timestamp() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap());
        let bot = GitLabBot::new(config);
        let summary = bot
            .process_events(vec![
                event("2026-07-06T00:00:01Z", "MergeRequest"),
                event("2026-07-06T00:00:03Z", "Issue"),
            ])
            .unwrap();
        assert_eq!(summary.latest_after, Some("2026-07-06T00:00:03Z".to_string()));
        assert_eq!(summary.processed_count, 1);
        assert_eq!(bot.snapshot().unwrap().after, "2026-07-06T00:00:03Z");
    }
}
