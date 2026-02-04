//! Module for working with remote and local application programming interfaces (APIs)
//!
//! Interact with RESTful APIs, handling requests and responses, authentication, and data serialization.
//! Also supports communicating with interfaces like large language model (LLM) interfaces like Ollama.
use crate::prelude::HashMap;
use crate::schema::validate::{is_orcid, is_ror};
use crate::util::Label;
use crate::{Location, Repository};
use bon::Builder;
use core::fmt;
use serde::{Deserialize, Serialize};
use tracing::warn;
use uriparse::URI;
use urlencoding::encode;

pub mod citeas;

/// Authentication schemes supported for API requests
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum AuthenticationScheme {
    /// Bearer token authentication (e.g., JWT)
    #[default]
    Bearer,
    /// Basic authentication (username:password)
    Basic,
    /// API key authentication
    ApiKey,
    /// OAuth 2.0 authentication
    OAuth2,
    /// Custom authentication scheme
    Custom(String),
}
/// ORCiD allowed search fields
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrcidSearchField {
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
    /// ### Examples
    /// - <https://ror.org/01qz5mb56> (Oak Ridge National Laboratory)
    /// - <https://ror.org/05p915b28> (Oak Ridge Leadership Computing Facility)
    RorOrgId,
    /// Text field that contains all of the other fields
    Text,
}
/// ORCiD allowed output columns
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrcidOutputColumn {
    /// Email address
    Email,
    /// Credit name
    CreditName,
    /// Current institution affiliation name
    CurrentInstitutionAffiliationName,
    /// Given and family names
    GivenAndFamilyNames,
    /// Given name
    GivenName,
    /// Family name
    FamilyName,
    /// [`ORCID`](crate::schema::pid::ORCID) identifier
    Orcid,
    /// Other name
    OtherName,
    /// Past institution affiliation name
    PastInstitutionAffiliationName,
}
/// Represents authentication credentials for accessing an API
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init)]
pub struct Authentication {
    // TODO: Make token secret
    /// The token used for authenticating API requests
    pub token: Option<String>,
    /// The scheme used for authenticating API requests
    pub scheme: AuthenticationScheme,
}
/// Represents an API endpoint with a lookup table for calling various paths
/// ### Note
/// Paths use handlebars templating syntax for dynamic URL construction, powered by [Tera](https://keats.github.io/tera/)
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init)]
pub struct Endpoint {
    /// The name of the API endpoint (used mainly for logging and identification)
    #[builder(default = String::new())]
    pub name: String,
    /// The domain of the API endpoint
    pub domain: Option<String>,
    /// The port of the API endpoint
    pub port: Option<u16>,
    /// A lookup table mapping path names to their corresponding URL templates
    #[builder(default = HashMap::new())]
    pub lookup: HashMap<String, String>,
    /// Authentication credentials for accessing the API endpoint
    pub authentication: Option<Authentication>,
}
impl Default for Endpoint {
    fn default() -> Self {
        Endpoint::init().domain("https://example.com".to_string()).build()
    }
}
impl<'a> From<URI<'a>> for Endpoint {
    fn from(value: URI<'a>) -> Self {
        let domain: Option<String> = value.host().map(|h| format!("{}://{}", value.scheme(), h));
        let port: Option<u16> = value.authority().and_then(|auth| auth.port());
        Endpoint::init().maybe_domain(domain).maybe_port(port).build()
    }
}
impl From<Location> for Endpoint {
    fn from(value: Location) -> Self {
        match value.uri() {
            | Some(uri) => {
                let endpoint: Endpoint = uri.into();
                endpoint
            }
            | None => Endpoint::default(),
        }
    }
}
impl From<Repository> for Endpoint {
    fn from(value: Repository) -> Self {
        value.location().into()
    }
}
// TODO: Define lookup maps for ORCiD, ROR, Vale, GitLab, GitHub, and Citeas APIs
impl Endpoint {}
impl fmt::Display for OrcidSearchField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            | OrcidSearchField::AffiliationOrgName => "affiliation-org-name",
            | OrcidSearchField::CreditName => "credit-name",
            | OrcidSearchField::Email => "email",
            | OrcidSearchField::ExternalIdReference => "external-id-reference",
            | OrcidSearchField::FamilyName => "family-name",
            | OrcidSearchField::GivenNames => "given-names",
            | OrcidSearchField::Keyword => "keyword",
            | OrcidSearchField::Orcid => "orcid",
            | OrcidSearchField::OtherNames => "other-names",
            | OrcidSearchField::RorOrgId => "ror-org-id",
            | OrcidSearchField::Text => "text",
        };
        write!(f, "{}", s)
    }
}
impl fmt::Display for OrcidOutputColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            | OrcidOutputColumn::CreditName => "credit-name",
            | OrcidOutputColumn::CurrentInstitutionAffiliationName => "current-institution-affiliation-name",
            | OrcidOutputColumn::Email => "email",
            | OrcidOutputColumn::FamilyName => "family-name",
            | OrcidOutputColumn::GivenAndFamilyNames => "given-and-family-names",
            | OrcidOutputColumn::GivenName => "given-name",
            | OrcidOutputColumn::Orcid => "orcid",
            | OrcidOutputColumn::OtherName => "other-name",
            | OrcidOutputColumn::PastInstitutionAffiliationName => "past-institution-affiliation-name",
        };
        write!(f, "{}", s)
    }
}
impl TryFrom<&str> for OrcidSearchField {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            | "affiliation-org-name" => Ok(OrcidSearchField::AffiliationOrgName),
            | "credit-name" => Ok(OrcidSearchField::CreditName),
            | "email" => Ok(OrcidSearchField::Email),
            | "external-id-reference" => Ok(OrcidSearchField::ExternalIdReference),
            | "family-name" => Ok(OrcidSearchField::FamilyName),
            | "given-names" => Ok(OrcidSearchField::GivenNames),
            | "keyword" => Ok(OrcidSearchField::Keyword),
            | "orcid" => Ok(OrcidSearchField::Orcid),
            | "other-names" => Ok(OrcidSearchField::OtherNames),
            | "ror-org-id" => Ok(OrcidSearchField::RorOrgId),
            | "text" => Ok(OrcidSearchField::Text),
            | _ => Err(format!("Invalid ORCiD search field: {value}")),
        }
    }
}
impl TryFrom<&str> for OrcidOutputColumn {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            | "credit-name" => Ok(OrcidOutputColumn::CreditName),
            | "current-institution-affiliation-name" => Ok(OrcidOutputColumn::CurrentInstitutionAffiliationName),
            | "email" => Ok(OrcidOutputColumn::Email),
            | "family-name" => Ok(OrcidOutputColumn::FamilyName),
            | "given-and-family-names" => Ok(OrcidOutputColumn::GivenAndFamilyNames),
            | "given-name" => Ok(OrcidOutputColumn::GivenName),
            | "orcid" => Ok(OrcidOutputColumn::Orcid),
            | "other-name" => Ok(OrcidOutputColumn::OtherName),
            | "past-institution-affiliation-name" => Ok(OrcidOutputColumn::PastInstitutionAffiliationName),
            | _ => Err(format!("Invalid ORCiD output column: {value}")),
        }
    }
}
/// Create an ORCiD API URL query string from a lookup table of key-value pairs and a list of desired output columns
///
/// Optional boosting of certain fields can be specified to prioritize their relevance in search results
// TODO: Support OR statements for field values (e.g., affiliation-org-name:("University of Plymouth" OR "Plymouth University"))
pub fn orcid_query<S>(field_pairs: Vec<(S, S)>, output_columns: Vec<S>, boost: Option<Vec<S>>) -> String
where
    S: AsRef<str>,
{
    let fields = field_pairs
        .into_iter()
        .filter_map(|(k, v)| match OrcidSearchField::try_from(k.as_ref()) {
            | Ok(field) => {
                let value = v.as_ref().trim();
                let is_valid = match field {
                    | OrcidSearchField::RorOrgId => is_ror(value).is_ok(),
                    | OrcidSearchField::Orcid => is_orcid(value).is_ok(),
                    | _ => true,
                };
                if is_valid {
                    let encoded = encode(value);
                    Some(format!("{field}:{encoded}"))
                } else {
                    warn!("=> {} Invalid value {value} for field {field}", Label::fail());
                    None
                }
            }
            | Err(_) => None,
        })
        .collect::<Vec<String>>()
        .join("+AND+");
    let columns = output_columns
        .into_iter()
        .filter_map(|value| match OrcidOutputColumn::try_from(value.as_ref()) {
            | Ok(column) => Some(column.to_string()),
            | Err(_) => None,
        })
        .collect::<Vec<String>>()
        .join(",");
    let query = if fields.is_empty() { String::new() } else { format!("?q={fields}") };
    let query_with_output_columns = if columns.is_empty() {
        query.clone()
    } else {
        format!("{query}&fl={columns}")
    };
    let result = if query_with_output_columns.is_empty() {
        query.clone()
    } else {
        match boost {
            | Some(fields) => {
                let valid_boost_fields = fields
                    .into_iter()
                    .filter_map(|value| match OrcidSearchField::try_from(value.as_ref()) {
                        | Ok(field) => Some(field.to_string()),
                        | Err(_) => None,
                    })
                    .collect::<Vec<String>>();
                let count = valid_boost_fields.len();
                let boosted = valid_boost_fields
                    .into_iter()
                    .filter_map(|value| match OrcidSearchField::try_from(value.as_ref()) {
                        | Ok(field) => Some(field.to_string()),
                        | Err(_) => None,
                    })
                    .enumerate()
                    .filter_map(|(i, field)| {
                        let value = count + 1 - i;
                        if value > 0 {
                            Some(format!("{field}^{value}.0"))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("%20");
                format!("{query_with_output_columns}&qf={boosted}")
            }
            | None => query_with_output_columns.clone(),
        }
    };
    result
}

#[cfg(test)]
mod tests;
