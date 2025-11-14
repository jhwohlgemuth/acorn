//! Module for communicating with RAiD service point API
//!
//! See <https://metadata.raid.org> for more information
//!
use crate::schema::pid::raid;
use crate::schema::validate::validate_attribute_ror;
use bon::Builder;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// RAiD service point API
pub trait ServicePointApi {
    /// Create and publish a new RAiD value
    fn mint_raid(&self, _metadata: raid::Metadata) -> Option<String> {
        None
    }
    /// Read a RAiD value
    fn read_raid(&self, _identifier: String) -> Option<String> {
        None
    }
}
/// RAiD service point
///
/// Primary entry point for interfacing with a RAiD service point
#[derive(Builder, Debug, Deserialize, Serialize, Validate)]
#[builder(start_fn = init)]
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
pub struct ServicePointResponse {
    /// Identifier of associated response
    #[serde(alias = "id")]
    pub identifier: String,
    /// Name of service point associated with response
    pub name: String,
    /// ROR of service point owner
    #[validate(custom(function = "validate_attribute_ror"))]
    #[serde(alias = "identifierOwner")]
    pub owner: String,
    /// Repository identifier
    #[serde(alias = "repositoryId")]
    pub repository: String,
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
