#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    clippy::arithmetic_side_effects
)]
//! Shared async HTTP server helpers for `acorn-lib` integration-style tests.
use crate::io::ApiResult;
use axum::Router;
use color_eyre::eyre::eyre;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

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
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|why| eyre!("bind test server listener failed: {why}"))?;
        let address = listener
            .local_addr()
            .map_err(|why| eyre!("read test server local address failed: {why}"))?;
        let base_url = format!("http://{address}");
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let task = tokio::spawn(async move {
            let server = axum::serve(listener, router).with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            });
            let _ = server.await;
        });
        Ok(Self {
            base_url,
            shutdown: Some(shutdown_tx),
            task,
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
