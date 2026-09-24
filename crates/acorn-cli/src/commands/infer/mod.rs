//! Local inference commands
use crate::io::write_stdout;
use acorn::io::sidecar::needle::Needle;
use acorn::io::sidecar::Sidecar;
use acorn::io::ApiResult;
use color_eyre::eyre::eyre;

/// Run one bounded Needle inference request and print its native result as JSON.
pub async fn run(prompt: &str, custom_location: Option<&str>, threshold: f64, max_rounds: usize, offline: bool) -> ApiResult<()> {
    let configured = match Needle::start(custom_location, offline).await {
        | Ok(sidecar) => sidecar
            .with_confidence_threshold(threshold)
            .and_then(|sidecar| sidecar.with_max_rounds(max_rounds)),
        | Err(why) => Err(why),
    };
    match configured {
        | Ok(sidecar) => match sidecar.infer(prompt.to_string()).await {
            | Ok(result) => serde_json::to_string_pretty(&result)
                .map_err(|why| eyre!("Failed to serialize Needle inference result — {why}"))
                .map(|json| write_stdout(format!("{json}\n"))),
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
