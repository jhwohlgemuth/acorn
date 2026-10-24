#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    clippy::arithmetic_side_effects
)]
//! Shared async HTTP server helpers for `acorn-lib` integration-style tests.
#[cfg(feature = "agentic")]
use crate::io::api::gitlab::EventDetails;
use crate::io::ApiResult;
#[cfg(feature = "agentic")]
use alloc::sync::Arc;
use axum::Router;
#[cfg(feature = "agentic")]
use axum::{
    extract::{RawQuery, State},
    http::StatusCode,
    routing::get,
    Json,
};
use color_eyre::eyre::eyre;
#[cfg(feature = "agentic")]
use std::{collections::HashMap, sync::Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[cfg(feature = "agentic")]
#[derive(Default)]
pub(crate) struct GitLabEventRepository {
    pub(crate) events: Vec<EventDetails>,
    pub(crate) fail_page_two: bool,
    pub(crate) queries: Vec<HashMap<String, String>>,
}
/// Handle for a running test server.
pub(crate) struct TestServer {
    /// HTTP base URL, for example `http://127.0.0.1:54321`.
    pub base_url: String,
    shutdown: Option<oneshot::Sender<()>>,
    task: JoinHandle<()>,
}
impl TestServer {
    /// Start a server for the provided Axum router on an ephemeral local port.
    pub async fn start(router: Router) -> ApiResult<Self> {
        TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|why| eyre!("bind test server listener failed: {why}"))
            .and_then(|listener| {
                listener
                    .local_addr()
                    .map(|address| (listener, address))
                    .map_err(|why| eyre!("read test server local address failed: {why}"))
            })
            .map(|(listener, address)| {
                let base_url = format!("http://{address}");
                let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
                let task = tokio::spawn(async move {
                    let server = axum::serve(listener, router).with_graceful_shutdown(async {
                        let _ = shutdown_rx.await;
                    });
                    let _ = server.await;
                });
                Self {
                    base_url,
                    shutdown: Some(shutdown_tx),
                    task,
                }
            })
    }
    /// Stop the server and await shutdown completion.
    pub async fn stop(mut self) -> ApiResult<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.task.await.map_err(|why| eyre!("test server join failed: {why}")).map(|_| ())
    }
}
#[cfg(feature = "agentic")]
pub(crate) async fn gitlab_event_server(repository: Arc<Mutex<GitLabEventRepository>>) -> TestServer {
    let router = Router::new()
        .route(
            "/api/v4/projects/30/events",
            get(
                |State(repository): State<Arc<Mutex<GitLabEventRepository>>>, RawQuery(raw): RawQuery| async move {
                    let query = raw
                        .unwrap_or_default()
                        .split('&')
                        .filter_map(|part| part.split_once('='))
                        .map(|(key, value)| (key.to_string(), value.to_string()))
                        .collect::<HashMap<_, _>>();
                    let mut repository = repository.lock().unwrap();
                    repository.queries.push(query.clone());
                    let page = query.get("page").and_then(|value| value.parse::<usize>().ok()).unwrap_or_default();
                    assert_eq!(query.get("sort").map(String::as_str), Some("asc"));
                    assert_eq!(query.get("target_type").map(String::as_str), Some("note"));
                    assert_eq!(query.get("per_page").map(String::as_str), Some("100"));
                    match page == 2 && repository.fail_page_two {
                        | true => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"message":"temporary failure"})),
                        ),
                        | false => {
                            let start = page.saturating_sub(1).saturating_mul(100);
                            let events = repository.events.iter().skip(start).take(100).collect::<Vec<_>>();
                            (StatusCode::OK, Json(serde_json::json!(events)))
                        }
                    }
                },
            ),
        )
        .with_state(repository);
    TestServer::start(router).await.unwrap()
}
