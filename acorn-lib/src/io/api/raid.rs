//! Module for communicating with RAiD service point API
//!
//! See <https://metadata.raid.org> for more information
//!
use crate::io::api::{ApiResult, Configuration, Endpoint, Fallback, RemoteResource};
use crate::param;
use crate::prelude::var;
use crate::schema::pid::raid::Metadata;
use crate::schema::validate::is_ror;
use bon::Builder;
use color_eyre::eyre::eyre;
use dotenvy;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Server response for RAiD API errors
/// ### Example JSON output
/// ```json
/// {
///   "type": "https://raid.org.au/errors#InvalidDateException",
///   "title": "Invalid date",
///   "status": 400,
///   "detail": "2023-08-28; 2023-08; 2023 is an invalid date or has an unsupported format.",
///   "instance": "https://raid.org.au",
///   "failures": [
///     {
///       "fieldId": "access.statement.language.schemaUri",
///       "errorType": "invalidValue",
///       "message": "schema is unknown/unsupported"
///     }
///   ]
/// }
/// ```
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[builder(start_fn = with_token, on(String, into))]
pub struct ErrorResponse {
    /// Error type
    #[serde(rename = "type")]
    pub error_type: String,
    /// Error title
    pub title: String,
    /// HTTP status code
    pub status: u16,
    /// Detailed error message
    pub detail: String,
    /// Error instance
    pub instance: String,
    /// Validation failures associated with this error response
    pub failures: Option<Vec<ErrorResponseFailure>>,
}
/// RAiD API error response failure
/// ### Example JSON output
/// ```json
/// {
///   "fieldId": "access.statement.language.schemaUri",
///   "errorType": "invalidValue",
///   "message": "schema is unknown/unsupported"
/// }
/// ```
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[builder(start_fn = with_token, on(String, into))]
pub struct ErrorResponseFailure {
    /// Field associated with this validation failure
    #[serde(rename = "fieldId")]
    pub field_id: String,
    /// Error type for this failure
    #[serde(rename = "errorType")]
    pub error_type: String,
    /// Human-readable failure message
    pub message: String,
}
/// RAiD API options
///
/// Configuration options for RAiD API operations
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = with_token, on(String, into))]
pub struct Options {
    /// Bearer token for authentication
    #[builder(start_fn)]
    pub token: String,
    /// Request body payload
    pub body: Option<String>,
    /// RAiD server domain (defaults to api.raid.org)
    #[builder(default = String::from("api.raid.org"))]
    pub domain: String,
    /// Service point identifier
    pub identifier: Option<String>,
    /// Metadata for minting a new RAiD record
    pub metadata: Option<Metadata>,
    /// RAiD prefix (e.g., "10.83962")
    pub prefix: Option<String>,
    /// RAiD suffix (e.g., "000001")
    pub suffix: Option<String>,
}
/// RAiD service point
///
/// Primary entry point for interfacing with a RAiD service point
#[derive(Builder, Debug, Deserialize, Serialize, Validate)]
#[builder(start_fn = with_token, on(String, into))]
pub struct ServicePoint {
    /// Service point identifier
    pub identifier: String,
    /// Service point endpoint URL
    #[validate(url)]
    pub url: String,
    /// Service point bearer token for authentication
    pub token: Option<String>,
}
/// RAiD service point response
///
/// Primary entry point for interfacing with a RAiD service point
///
/// ###Example Response
/// ```json
/// {
///   "id": 20000033,
///   "name": "Oak Ridge National Laboratory",
///   "identifierOwner": "https://ror.org/01qz5mb56",
///   "repositoryId": "ATHH.AZKTIF",
///   "prefix": "10.83962",
///   "groupId": "212777f8-ecfe-43a6-a809-e6a551d393e3",
///   "techEmail": "research@ornl.gov",
///   "adminEmail": "raid@ornl.gov",
///   "enabled": true,
///   "appWritesEnabled": true
/// }
/// ```
#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ServicePointResponse {
    /// Identifier of associated response
    #[serde(alias = "id")]
    pub identifier: u64,
    /// Name of service point associated with response
    pub name: String,
    /// ROR of service point owner
    #[validate(custom(function = "is_ror"))]
    #[serde(alias = "identifierOwner")]
    pub owner: String,
    /// Repository identifier
    #[serde(alias = "repositoryId")]
    pub repository: Option<String>,
    /// RAiD prefix used by service point
    pub prefix: String,
    /// Group identifier
    #[serde(alias = "groupId")]
    pub group: String,
    /// Email address for technical support
    #[serde(alias = "techEmail")]
    pub tech_email: String,
    /// Email address for administrative support
    #[serde(alias = "adminEmail")]
    pub admin_email: String,
    /// Status of service point
    pub enabled: bool,
    /// Status of app writes
    #[serde(alias = "appWritesEnabled")]
    pub app_writes_enabled: bool,
}
impl Configuration for Options {
    /// Build options from RAiD environment variables
    /// - `RAID_API_TOKEN` -> `token` (defaults to empty string when unset)
    /// - `RAID_SERVER_HOST` -> `domain` (defaults to api.raid.org when unset)
    fn from_env() -> Self {
        let _ = dotenvy::from_filename(".env");
        Self {
            token: var("RAID_API_TOKEN").unwrap_or_default(),
            body: None,
            domain: var("RAID_SERVER_HOST").unwrap_or_else(|_| String::from("api.raid.org")),
            prefix: None,
            suffix: None,
            identifier: None,
            metadata: None,
        }
    }
    /// Return a copy of options with request body payload set
    fn with_body(self, value: impl Into<String>) -> Self {
        Self {
            body: Some(value.into()),
            ..self
        }
    }
    /// Return a copy of options with RAiD server domain set
    fn with_domain(self, value: impl Into<String>) -> Self {
        Self {
            domain: value.into(),
            ..self
        }
    }
}
impl Options {
    /// Return a copy of options with service point identifier set
    pub fn with_identifier(self, value: impl Into<String>) -> Self {
        Self {
            identifier: Some(value.into()),
            ..self
        }
    }
    /// Return a copy of options with RAiD metadata set
    pub fn with_metadata(self, value: impl Into<Metadata>) -> Self {
        Self {
            metadata: Some(value.into()),
            ..self
        }
    }
}
/// Create a RAiD record with provided metadata
pub async fn create_record(options: Options) -> ApiResult<Metadata> {
    let template = "raid::api";
    let action = "record::create";
    let Options { token, domain, metadata, .. } = options;
    match Endpoint::from_template(template).map(|e| e.with_domain(&domain)) {
        | Ok(endpoint) => match metadata {
            | Some(value) => match serde_json::to_string(&value) {
                | Ok(body) => {
                    let params = vec![param!(Header, "Authorization", &format!("Bearer {token}")), param!(Body, &body)];
                    let response = endpoint.invoke(action, Some(params)).await;
                    endpoint.handle_or::<Metadata, Fallback<ErrorResponse>>(response)
                }
                | Err(why) => Err(eyre!("Failed to serialize RAiD metadata to JSON — {why:#?}")),
            },
            | None => Err(eyre!("RAiD metadata is required for creating a record")),
        },
        | Err(why) => Err(why),
    }
}
/// Retrieve RAiD record(s) attached to associated service point of provided options
///
/// If no identifier is provided, will return all RAiD records attached to the associated service point.
///
/// ### Example
/// ```ignore
/// use acorn::io::api::raid;
///
/// let options = raid::Options::from_env();
/// let result = raid::record(options).await;
/// ```
pub async fn record(options: Options) -> ApiResult<Vec<Metadata>> {
    let template = "raid::api";
    let action = "record";
    let Options {
        identifier, token, domain, ..
    } = options;
    match Endpoint::from_template(template).map(|e| e.with_domain(&domain)) {
        | Ok(endpoint) => match identifier {
            | Some(value) if !value.is_empty() => {
                let params = vec![
                    param!(Header, "Authorization", &format!("Bearer {token}")),
                    param!(TemplateValue, "identifier", &value),
                ];
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle::<Metadata>(response).map(|r| vec![r])
            }
            | Some(_) | None => {
                let params = vec![param!(Header, "Authorization", &format!("Bearer {token}"))];
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle::<Vec<Metadata>>(response)
            }
        },
        | Err(why) => Err(why),
    }
}
/// Retrieve all service point(s) for a given server
///
/// If no identifier is provided, will return all service points for the associated server
///
/// ### Example
/// ```ignore
/// use acorn::io::api::raid;
///
/// let options = raid::Options::from_env();
/// let result = raid::service_point(options).await;
/// ```
pub async fn service_point(options: Options) -> ApiResult<Vec<ServicePointResponse>> {
    let template = "raid::api";
    let action = "service-point";
    let Options {
        token, domain, identifier, ..
    } = options;
    match Endpoint::from_template(template).map(|e| e.with_domain(&domain)) {
        | Ok(endpoint) => match identifier {
            | Some(value) if !value.is_empty() => {
                let params = vec![
                    param!(Header, "Authorization", &format!("Bearer {token}")),
                    param!(TemplateValue, "identifier", &value),
                ];
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle::<ServicePointResponse>(response).map(|r| vec![r])
            }
            | Some(_) | None => {
                let params = vec![param!(Header, "Authorization", &format!("Bearer {token}"))];
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle::<Vec<ServicePointResponse>>(response)
            }
        },
        | Err(why) => Err(why),
    }
}
