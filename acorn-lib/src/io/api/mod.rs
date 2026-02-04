//! Module for working with remote and local application programming interfaces (APIs)
//!
//! Interact with RESTful APIs, handling requests and responses, authentication, and data serialization.
//! Also supports communicating with interfaces like large language model (LLM) interfaces like Ollama.
use crate::prelude::HashMap;
use crate::{Location, Repository};
use bon::Builder;
use serde::{Deserialize, Serialize};
use uriparse::URI;

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
    AffiliationOrgName,
    /// Credit name
    CreditName,
    /// Email address
    Email,
    /// External ID reference
    ExternalIdReference,
    /// Family name
    FamilyName,
    /// Given names
    GivenNames,
    /// Keyword
    Keyword,
    /// ORCiD identifier
    Orcid,
    /// Other names
    OtherNames,
    /// ROR organization ID
    RorOrgId,
    /// Text field
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
    /// ORCiD identifier
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

/// Create an ORCiD API URL query string from a lookup table of key-value pairs and a list of desired output columns
/// 
/// Optional boosting of certain fields can be specified to prioritize their relevance in search results
// TODO: Support OR statements for field values (e.g., affiliation-org-name:("University of Plymouth" OR "Plymouth University"))
pub fn orcid_query<S>(_field_lookup: HashMap<S, S>, _output_columns: Vec<S>, _boost: Option<Vec<S>>) -> String where S: AsRef<str> {
    // TODO: Filter fields based on allowed list and associated validators (e.g., is_ror, is_orcid, etc.)
    String::new()
}

#[cfg(test)]
mod tests;
