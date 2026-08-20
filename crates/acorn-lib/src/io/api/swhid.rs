//! Software Heritage SWHID resolution API.
use crate::error::ApiResult;
use crate::io::http::{get, response_body_bytes};
use crate::schema::pid::SWHID;
use crate::util::constants::app::SOFTWARE_HERITAGE_RESOLVER_URI;
use color_eyre::eyre::eyre;

/// Resolve a SWHID through the Software Heritage archive
///
/// The returned value is resolver metadata, not citation metadata.
pub async fn resolve(identifier: &SWHID) -> ApiResult<serde_json::Value> {
    let identifier = identifier.to_string();
    let encoded = urlencoding::encode(&identifier);
    let url = format!("{SOFTWARE_HERITAGE_RESOLVER_URI}/{encoded}/");
    response_body_bytes(get(url).send().await, "Software Heritage could not resolve the SWHID")
        .await
        .and_then(|body| serde_json::from_slice(&body).map_err(|why| eyre!("Software Heritage returned invalid resolver metadata — {why}")))
}
