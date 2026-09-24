//! Needle subprocess lifecycle
use super::super::{ResolvedAssets, ToolIndex};
use crate::io::{api::localhost::available_port, ApiResult};
use acorn_cmd::{args, cmd};
use acorn_core::prelude::{Box, Path, PathBuf, String};
use color_eyre::eyre::eyre;
use std::{fs::remove_dir_all, process::Child};
use tokio::{
    net::TcpStream,
    time::{sleep, Duration},
};

const MAX_RESPONSE_TOKENS: usize = 4096;
const MODEL_DEPTH: usize = 20;
const STARTUP_ATTEMPTS: u16 = 300;
/// Running Needle child process.
#[derive(Debug)]
pub struct Runner {
    child: Child,
    endpoint: String,
    session_root: PathBuf,
    tool_index: Box<dyn ToolIndex>,
}
impl Drop for Runner {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = self.tool_index.publish();
        let _ = remove_dir_all(&self.session_root);
    }
}
impl Runner {
    /// Start the standalone runner with a fixed tool catalog.
    pub async fn start(assets: &ResolvedAssets, tools: &Path, tool_index: Box<dyn ToolIndex>, session_root: PathBuf) -> ApiResult<Self> {
        let cleanup_root = session_root.clone();
        let runner = assets
            .assets
            .get("model")
            .ok_or_else(|| eyre!("Needle 3 model asset is unavailable"))
            .and_then(|model| available_port().map(|port| (model, port)))
            .and_then(|(model, port)| {
                let arguments = args![
                    ("--model", model.as_os_str()),
                    ("--tools", tools.as_os_str()),
                    ("--tool-index", tool_index.session().as_os_str()),
                    "--serve",
                    ("--port", port.to_string()),
                    ("--depth", MODEL_DEPTH.to_string()),
                    ("--max", MAX_RESPONSE_TOKENS.to_string()),
                    "--fail-input-overflow"
                ];
                cmd!(spawn &assets.runner, arguments; dir: &assets.root, env: [("DO_NOT_TRACK", "1"), ("NEEDLE_TELEMETRY", "0")])
                    .map(|child| Self {
                        child,
                        endpoint: format!("http://127.0.0.1:{port}"),
                        session_root,
                        tool_index,
                    })
                    .map_err(|why| eyre!("Failed to start Needle runner — {why}"))
            });
        let result = match runner {
            | Ok(mut runner) => match runner.wait_until_ready().await {
                | Ok(()) => Ok(runner),
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        };
        if result.is_err() {
            let _ = remove_dir_all(cleanup_root);
        }
        result
    }
    /// Return the private loopback endpoint selected for this process.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
    async fn wait_until_ready(&mut self) -> ApiResult<()> {
        let address = self.endpoint.trim_start_matches("http://");
        let mut remaining = STARTUP_ATTEMPTS;
        while remaining > 0 {
            match self.child.try_wait() {
                | Ok(Some(status)) => return Err(eyre!("Needle runner exited before becoming ready with status {status}")),
                | Err(why) => return Err(eyre!("Failed to inspect Needle runner status — {why}")),
                | Ok(None) if TcpStream::connect(address).await.is_ok() => return Ok(()),
                | Ok(None) => {
                    remaining = remaining.saturating_sub(1);
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
        Err(eyre!("Needle runner did not become ready at {} within 30 seconds", self.endpoint))
    }
}
