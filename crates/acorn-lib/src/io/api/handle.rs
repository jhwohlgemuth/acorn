//! Handle proxy JSON REST API.
use super::{Endpoint, RemoteResource};
use crate::error::ApiResult;
use crate::param;
use acorn_core::prelude::{vec, String, Vec};
use acorn_schema::pid::{Handle, PersistentIdentifier};
use acorn_schema::util::constants::HTTP_URL;
use alloc::collections::BTreeSet;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

/// Public Handle record returned by the proxy REST API
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    /// Handle protocol response code
    pub response_code: u16,
    /// Handle value returned by the resolver
    pub handle: String,
    /// Public values stored in the Handle record
    #[serde(default)]
    pub values: Vec<Value>,
}
/// One public value in a Handle record
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Value {
    /// Value index within the Handle record
    pub index: u32,
    /// Handle value type, such as `URL` or `10320/loc`
    #[serde(rename = "type")]
    pub kind: String,
    /// Typed Handle value data
    pub data: ValueData,
    /// Public read/admin permissions when supplied by the proxy
    pub permissions: Option<String>,
    /// Time-to-live in seconds when supplied by the proxy
    pub ttl: Option<u64>,
    /// Value timestamp when supplied by the proxy
    pub timestamp: Option<String>,
}
/// Data representation for a Handle value
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ValueData {
    /// Handle data encoding, such as `string`, `base64`, or `hex`
    pub format: String,
    /// Lossless JSON representation of the value
    pub value: serde_json::Value,
}
impl Record {
    /// Return sorted, deduplicated direct public URL string values
    pub fn public_urls(&self) -> Vec<String> {
        self.values
            .iter()
            .filter(|value| value.kind.eq_ignore_ascii_case("URL") && value.data.format.eq_ignore_ascii_case("string"))
            .filter_map(|value| value.data.value.as_str())
            .filter(|value| HTTP_URL.is_match(value).unwrap_or(false))
            .map(str::to_string)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    pub(crate) fn validate(self) -> ApiResult<Self> {
        match self.response_code {
            | 1 => Ok(self),
            | code => Err(eyre!("Handle proxy could not resolve the Handle — response code {code}")),
        }
    }
}
/// Resolve a Handle through the public Handle proxy JSON REST API
pub async fn resolve(identifier: &Handle) -> ApiResult<Record> {
    match Endpoint::from_template("handle") {
        | Ok(endpoint) => resolve_with(&endpoint, identifier).await,
        | Err(why) => Err(eyre!("Handle API endpoint is unavailable — {why}")),
    }
}
pub(crate) async fn resolve_with(endpoint: &Endpoint, identifier: &Handle) -> ApiResult<Record> {
    let value = identifier.identifier();
    match value.is_empty() {
        | true => Err(eyre!("Handle resolver requires a valid Handle")),
        | false => {
            let encoded = value.split('/').map(urlencoding::encode).collect::<Vec<_>>().join("/");
            let data = Some(vec![param!(TemplateValue, "identifier", encoded.as_str())]);
            let response = endpoint.invoke("record", data).await;
            endpoint
                .handle::<Record>(response)
                .map_err(|why| eyre!("Handle proxy returned invalid resolver metadata — {why}"))
                .and_then(Record::validate)
        }
    }
}
