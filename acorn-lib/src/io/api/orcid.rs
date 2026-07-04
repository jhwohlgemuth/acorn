//! Module for interacting with [ORCiD REST API](https://info.orcid.org/what-is-orcid/services/public-api/)
//!
//! Provides types and functions for constructing ORCiD API queries with field validation and output column selection
//!
//! ## Example Uses
//!
//! ### Get API status
//! ```ignore
//! use acorn_lib::io::api;
//!
//! println!("ORCiD REST API is healthy: {}", api::orcid::is_healthy().await);
//! ```
//!
//! ### Search for last names of people affiliated with Lyrasis and ORNL
//! ```ignore
//! use acorn_lib::param;
//! use acorn_lib::io::api;
//!
//! let params = vec![
//!     param!(
//!         QueryPair,
//!         "q",
//!         (("affiliation-org-name", "Lyrasis"), ("ror-org-id", "\"https://ror.org/01qz5mb56\""),)
//!     ),
//!     param!(FieldList, "fl", "family-name"),
//! ];
//! println!("ORCiD Search Response: {:#?}", api::orcid::search(params).await);
//! ```
use crate::io::api::{self, Configuration, Param, Params, RemoteResource, ValueValidator, INCLUDED_ENDPOINTS};
use crate::io::ApiResult;
use crate::prelude::var;
use crate::schema::validate::{is_orcid, is_ror};
use crate::util::{Label, Searchable};
use bon::Builder;
use color_eyre::eyre::{self, eyre};
use core::fmt;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::debug;

/// ORCiD API options
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = with_token, on(String, into))]
pub struct Options {
    /// Bearer token for authentication
    #[builder(start_fn)]
    pub token: String,
    /// Request body payload
    pub body: Option<String>,
    /// ORCiD API domain (defaults to pub.orcid.org)
    #[builder(default = String::from("pub.orcid.org"))]
    pub domain: String,
    /// ORCiD identifier
    pub identifier: Option<String>,
    /// Custom API parameters to include in every request
    #[builder(default = vec![])]
    pub custom_params: Vec<Param>,
}
impl Configuration for Options {
    /// Build options from ORCiD environment variables
    /// - `ORCID_API_TOKEN` -> `token` (optional — public API)
    /// - `ORCID_SERVER_HOST` -> `domain` (defaults to pub.orcid.org)
    fn from_env() -> Self {
        if let Err(why) = dotenvy::from_filename(".env") {
            debug!("=> {} Load .env — {why}", Label::skip());
        }
        Self {
            token: var("ORCID_API_TOKEN").unwrap_or_default(),
            body: None,
            domain: var("ORCID_SERVER_HOST").unwrap_or_else(|_| String::from("pub.orcid.org")),
            identifier: None,
            custom_params: vec![],
        }
    }
    fn with_body(self, value: impl Into<String>) -> Self {
        Self {
            body: Some(value.into()),
            ..self
        }
    }
    fn with_domain(self, value: impl Into<String>) -> Self {
        Self {
            domain: value.into(),
            ..self
        }
    }
    fn with_identifier(self, value: impl Into<String>) -> Self {
        Self {
            identifier: Some(value.into()),
            ..self
        }
    }
    fn token(&self) -> &str {
        &self.token
    }
    fn domain(&self) -> &str {
        &self.domain
    }
    fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }
    fn with_params(self, params: Vec<Param>) -> Self {
        Self {
            custom_params: params,
            ..self
        }
    }
    fn params(&self) -> &[Param] {
        &self.custom_params
    }
}

/// ORCiD allowed search fields
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SearchField {
    /// Affiliation organization name
    /// ### Example
    /// > "Oak Ridge National Laboratory"
    AffiliationOrgName,
    /// Preferred form or display name, which can differ from legal or given/family names
    CreditName,
    /// Email address
    Email,
    /// External ID reference
    ExternalIdReference,
    /// Family name (i.e., last name or surname)
    FamilyName,
    /// Given names (i.e., first name(s))
    GivenNames,
    /// Keyword
    Keyword,
    /// [`ORCID`](crate::schema::pid::ORCID) identifier
    /// ### Examples
    /// - `0000-0002-2057-9115` (Jason Wohlgemuth)
    /// - `0009-0005-5568-6526` (Audrey Carson)
    Orcid,
    /// Other names
    OtherNames,
    /// [ROR](https://ror.org) organization ID
    /// ### Notes
    /// - Must include ror.org domain
    /// - Must be enclosed in double quotes
    /// ### Examples
    /// - "<https://ror.org/01qz5mb56>" (Oak Ridge National Laboratory)
    /// - "<https://ror.org/05p915b28>" (Oak Ridge Leadership Computing Facility)
    RorOrgId,
    /// Text field that contains all of the other fields
    Text,
}
/// ORCiD allowed output columns
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputColumn {
    /// Email address
    Email,
    /// Credit name
    CreditName,
    /// Current institution affiliation name
    CurrentInstitutionAffiliationName,
    /// Given names
    GivenNames,
    /// Family names
    FamilyName,
    /// [`ORCID`](crate::schema::pid::ORCID) identifier
    Orcid,
    /// Other name
    OtherName,
    /// Past institution affiliation name
    PastInstitutionAffiliationName,
}
/// ORCiD search response
/// ### Example response
/// ```xml
/// <expanded-search:expanded-search xmlns:expanded-search="http://www.orcid.org/ns/expanded-search" num-found="68">
///     ...results
/// </expanded-search:expanded-search>
/// ```
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SearchResponse {
    /// Number of results found
    #[serde(rename = "@num-found")]
    pub num_found: usize,
    /// XML namespace for expanded search
    #[builder(default = "http://www.orcid.org/ns/expanded-search".to_string())]
    #[serde(rename = "@xmlns:expanded-search")]
    pub namespace: String,
    /// List of expanded search results
    #[builder(default)]
    #[serde(rename = "expanded-result", default)]
    pub results: Vec<SearchResult>,
}
/// ORCiD search result
/// ### Example response
/// ```xml
/// <expanded-search:expanded-result>
///     <expanded-search:orcid-id>0000-0002-2057-9115</expanded-search:orcid-id>
///     <expanded-search:given-names>Jason</expanded-search:given-names>
///     <expanded-search:family-names>Wohlgemuth</expanded-search:family-names>
///     <expanded-search:credit-name>Jason Wohlgemuth</expanded-search:credit-name>
///     <expanded-search:institution-name>Lyrasis</expanded-search:institution-name>
///     <expanded-search:institution-name>Oak Ridge National Laboratory</expanded-search:institution-name>
///     <expanded-search:institution-name>USSTRATCOM</expanded-search:institution-name>
///     <expanded-search:institution-name>University of Nebraska Omaha</expanded-search:institution-name>
/// </expanded-search:expanded-result>
/// ```
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SearchResult {
    /// [`ORCID`](crate::schema::pid::ORCID) identifier
    #[serde(rename = "orcid-id")]
    pub orcid_id: Option<String>,
    /// Given names (first name(s))
    #[serde(rename = "given-names")]
    pub given_names: Option<String>,
    /// Family name (last name or surname)
    #[serde(rename = "family-names")]
    pub family_names: Option<String>,
    /// Credit name (preferred display name)
    #[serde(rename = "credit-name")]
    pub credit_name: Option<String>,
    /// Email addresses
    #[serde(rename = "email")]
    pub emails: Option<Vec<String>>,
    /// Institution names
    #[serde(rename = "institution-name")]
    pub institution_names: Option<Vec<String>>,
    /// Other names
    #[serde(rename = "other-name")]
    pub other_name: Option<Vec<String>>,
}
/// Describes status of the ORCiD API
/// ### Caution
/// > Limit status checks to once every 5 mins ([docs](https://info.orcid.org/ufaqs/how-do-i-check-the-server-status/))
/// ### Example response
/// ```json
/// {
///   "tomcatUp": true,
///   "dbConnectionOk": true,
///   "readOnlyDbConnectionOk": true,
///   "overallOk": true
/// }
/// ```
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StatusResponse {
    /// Application server status
    #[serde(rename = "tomcatUp")]
    pub application: bool,
    /// Database server status
    #[serde(rename = "dbConnectionOk")]
    pub database: bool,
    /// Read-only database server status
    #[serde(rename = "readOnlyDbConnectionOk")]
    pub database_readonly: bool,
    /// Overall API status
    #[serde(rename = "overallOk")]
    pub overall: bool,
}
impl ValueValidator for SearchField {
    /// Validate certain types of ORCiD search field values
    ///
    /// Special validation is performed for `RorOrgId` and `Orcid` fields.
    fn is_valid(&self, value: &str) -> bool {
        match self {
            | SearchField::RorOrgId => is_ror(value.replace("\"", "").as_str()).is_ok(),
            | SearchField::Orcid => is_orcid(value).is_ok(),
            | _ => true,
        }
    }
}
impl fmt::Display for SearchField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            | SearchField::AffiliationOrgName => "affiliation-org-name",
            | SearchField::CreditName => "credit-name",
            | SearchField::Email => "email",
            | SearchField::ExternalIdReference => "external-id-reference",
            | SearchField::FamilyName => "family-name",
            | SearchField::GivenNames => "given-names",
            | SearchField::Keyword => "keyword",
            | SearchField::Orcid => "orcid",
            | SearchField::OtherNames => "other-names",
            | SearchField::RorOrgId => "ror-org-id",
            | SearchField::Text => "text",
        };
        write!(f, "{}", s)
    }
}
impl fmt::Display for OutputColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            | OutputColumn::CreditName => "credit-name",
            | OutputColumn::CurrentInstitutionAffiliationName => "current-institution-affiliation-name",
            | OutputColumn::Email => "email",
            | OutputColumn::FamilyName => "family-name",
            | OutputColumn::GivenNames => "given-names",
            | OutputColumn::Orcid => "orcid",
            | OutputColumn::OtherName => "other-name",
            | OutputColumn::PastInstitutionAffiliationName => "past-institution-affiliation-name",
        };
        write!(f, "{}", s)
    }
}
impl TryFrom<&str> for SearchField {
    type Error = String;

    fn try_from(value: &str) -> eyre::Result<Self, Self::Error> {
        match value {
            | "affiliation-org-name" => Ok(SearchField::AffiliationOrgName),
            | "credit-name" => Ok(SearchField::CreditName),
            | "email" => Ok(SearchField::Email),
            | "external-id-reference" => Ok(SearchField::ExternalIdReference),
            | "family-name" => Ok(SearchField::FamilyName),
            | "given-names" => Ok(SearchField::GivenNames),
            | "keyword" => Ok(SearchField::Keyword),
            | "orcid" => Ok(SearchField::Orcid),
            | "other-names" => Ok(SearchField::OtherNames),
            | "ror-org-id" => Ok(SearchField::RorOrgId),
            | "text" => Ok(SearchField::Text),
            | _ => Err(format!("Invalid ORCiD search field: {value}")),
        }
    }
}
impl TryFrom<&str> for OutputColumn {
    type Error = String;

    fn try_from(value: &str) -> eyre::Result<Self, Self::Error> {
        match value {
            | "credit-name" => Ok(OutputColumn::CreditName),
            | "current-institution-affiliation-name" => Ok(OutputColumn::CurrentInstitutionAffiliationName),
            | "email" => Ok(OutputColumn::Email),
            | "family-name" => Ok(OutputColumn::FamilyName),
            | "given-names" => Ok(OutputColumn::GivenNames),
            | "orcid" => Ok(OutputColumn::Orcid),
            | "other-name" => Ok(OutputColumn::OtherName),
            | "past-institution-affiliation-name" => Ok(OutputColumn::PastInstitutionAffiliationName),
            | _ => Err(format!("Invalid ORCiD output column: {value}")),
        }
    }
}
/// Check if API is healthy
/// ### Example
/// ```ignore
/// use acorn_lib::io::api;
///
/// println!("ORCiD API is healthy: {}", api::orcid::is_healthy().await);
/// ```
pub async fn is_healthy() -> bool {
    match status().await {
        | Ok(StatusResponse { overall, .. }) => overall,
        | Err(_) => false,
    }
}
/// Construct query string for ORCiD API search endpoint
pub fn query_string(query_pairs: Vec<(&str, &str)>, field_list: Vec<&str>, query_fields: Vec<&str>) -> String {
    api::query_string::<SearchField, OutputColumn>(query_pairs, field_list, query_fields)
}
/// Search the ORCiD API with given options containing query parameters and output fields
///
/// ### Example
/// ```ignore
/// use acorn::param;
/// use acorn::io::api::orcid::{self, Options, SearchResponse};
///
/// let options = Options::from_env()
///     .with_params(vec![
///         param!(
///             QueryPair,
///             "q",
///             (("affiliation-org-name", "Lyrasis"), ("ror-org-id", "\"https://ror.org/01qz5mb56\""),)
///         ),
///         param!(FieldList, "fl", "family-name"),
///     ]);
/// let result: ApiResult<SearchResponse> = orcid::search(&options).await;
/// ```
pub async fn search(options: &Options) -> ApiResult<SearchResponse> {
    let name = "ORCiD";
    let action = "search";
    let params = Params::new().with_custom(options.params()).build();
    let data = Some(params);
    match INCLUDED_ENDPOINTS.find_by_name(name) {
        | Some(endpoint) => {
            let response = endpoint.invoke_with::<SearchField, OutputColumn>(action, data).await;
            endpoint.handle::<SearchResponse>(response)
        }
        | None => Err(eyre!("{name} API endpoint not found")),
    }
}
/// Get status of ORCiD API
pub async fn status() -> ApiResult<StatusResponse> {
    let name = "ORCiD";
    let action = "status";
    let data = None;
    match INCLUDED_ENDPOINTS.find_by_name(name) {
        | Some(endpoint) => {
            let response = endpoint.invoke(action, data).await;
            endpoint.handle::<StatusResponse>(response)
        }
        | None => Err(eyre!("{name} API endpoint not found")),
    }
}
