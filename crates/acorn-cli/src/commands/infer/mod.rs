//! Local inference commands

use crate::io::{read_stdin, write_stdout};
use acorn::io::sidecar::needle::Needle;
use acorn::io::sidecar::Sidecar;
use acorn::io::ApiResult;
use color_eyre::eyre::eyre;
pub(crate) mod prompt;
fn resolve_prompt<ReadStdin>(prompt: Option<&str>, read: ReadStdin) -> ApiResult<String>
where
    ReadStdin: FnOnce() -> Option<String>,
{
    prompt
        .map(str::to_string)
        .or_else(read)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| eyre!("Provide a prompt argument or pipe a prompt through standard input"))
}
/// Run one bounded Needle inference request and print its native result as JSON.
pub async fn run(prompt: Option<&str>, custom_location: Option<&str>, threshold: f64, max_rounds: usize, offline: bool) -> ApiResult<()> {
    match resolve_prompt(prompt, read_stdin) {
        | Err(why) => Err(why),
        | Ok(prompt) => {
            let configured = match Needle::start(custom_location, offline).await {
                | Ok(sidecar) => sidecar
                    .with_confidence_threshold(threshold)
                    .and_then(|sidecar| sidecar.with_max_rounds(max_rounds)),
                | Err(why) => Err(why),
            };
            match configured {
                | Ok(sidecar) => match sidecar.infer(prompt).await {
                    | Ok(result) => serde_json::to_string_pretty(&result)
                        .map_err(|why| eyre!("Failed to serialize Needle inference result — {why}"))
                        .map(|json| write_stdout(format!("{json}\n"))),
                    | Err(why) => Err(why),
                },
                | Err(why) => Err(why),
            }
        }
    }
}
#[cfg(test)]
mod tests;
