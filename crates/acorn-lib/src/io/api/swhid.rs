//! Software Heritage SWHID resolution API.
use crate::error::ApiResult;
use crate::io::http::{get, response_body_bytes};
use acorn_schema::pid::SWHID;
use acorn_schema::util::constants::app::SOFTWARE_HERITAGE_RESOLVER_URI;
use color_eyre::eyre::eyre;

/// Resolve a SWHID through the Software Heritage archive
///
/// The returned value is resolver metadata, not citation metadata.
pub async fn resolve(identifier: &SWHID) -> ApiResult<serde_json::Value> {
    resolve_with(SOFTWARE_HERITAGE_RESOLVER_URI, identifier).await
}
/// Resolve a SWHID through a custom resolver base URI
///
/// `base_uri` is the resolver root without trailing slash, e.g. `https://archive.softwareheritage.org/api/1/resolve`.
/// This is the injection point for fixture-backed tests.
pub(crate) async fn resolve_with(base_uri: &str, identifier: &SWHID) -> ApiResult<serde_json::Value> {
    let identifier_str = identifier.to_string();
    match identifier_str.is_empty() || identifier.object_id.is_empty() {
        | true => Err(eyre!("SWHID resolver requires a valid SWHID")),
        | false => {
            let encoded = urlencoding::encode(&identifier_str);
            let url = format!("{}/{encoded}/", base_uri.trim_end_matches('/'));
            response_body_bytes(get(url).send().await, "Software Heritage could not resolve the SWHID")
                .await
                .and_then(|body| serde_json::from_slice(&body).map_err(|why| eyre!("Software Heritage returned invalid resolver metadata — {why}")))
        }
    }
}
