//! Local inference commands.

use crate::io::write_stdout;
use acorn::io::sidecar::needle::Sidecar;
use acorn::io::ApiResult;
use color_eyre::eyre::eyre;

/// Run one bounded Needle inference request and print its native result as JSON.
pub async fn run(prompt: &str, custom_location: Option<&str>, confidence_threshold: f64, max_rounds: usize, offline: bool) -> ApiResult<()> {
    let sidecar = Sidecar::start(custom_location, offline).await;
    match sidecar {
        | Ok(sidecar) => sidecar
            .with_confidence_threshold(confidence_threshold)
            .and_then(|sidecar| sidecar.with_max_rounds(max_rounds)),
        | Err(why) => Err(why),
    }
    .map(|mut sidecar| async move { sidecar.infer(prompt).await })?
    .await
    .and_then(|result| serde_json::to_string_pretty(&result).map_err(|why| eyre!("Failed to serialize Needle inference result — {why}")))
    .map(|json| write_stdout(format!("{json}\n")))
}
