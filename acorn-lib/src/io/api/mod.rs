//! Module for working with remote and local application programming interfaces (APIs)
//!
//! Interact with RESTful APIs, handling requests and responses, authentication, and data serialization.
//! Also supports communicating with interfaces like large language model (LLM) interfaces like Ollama.
use crate::{Location, Repository, Scheme};
use bon::Builder;
use core::fmt;
use serde::{Deserialize, Serialize};
use tera::Context;
use uriparse::URI;

pub mod citeas;
pub mod orcid;

type Resource = (String, HttpMethod, Vec<Param>);

/// Trait for working with request-response cycle using HTTP methods like GET, POST, PUT, PATCH, and DELETE
/// against resource URLs that return structured data
pub trait RestfulInterface {
    /// Build context for endpoint paths
    fn build(&self) -> Context {
        Context::new()
    }
    /// Handle a response from an endpoint request
    fn handle<R>(&self, response: R) -> Result<(), String>
    where
        R: Deserialize<'static>;
    /// Send data to the endpoint and receive a response
    fn invoke<R, S>(&self, action: S, data: Option<Vec<(&str, &str)>>) -> Result<R, String>
    where
        R: Deserialize<'static>,
        S: AsRef<str>;
}
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
/// HTTP methods supported for API requests
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum HttpMethod {
    /// Retrieve data from a server without modifying it — safe and idempotent
    #[default]
    GET,
    /// Submit data to create a new resource or process information — not idempotent
    POST,
    /// Replace all current representations of the target resource with the uploaded content — idempotent
    PUT,
    /// Apply partial modifications to a resource — not necessarily idempotent
    PATCH,
    /// Delete a specified resource — idempotent
    DELETE,
}
/// Describes the location/type of a parameter for an API resource
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ParamStyle {
    /// URI query string parameter
    Query,
    /// HTTP header parameter
    Header,
    /// URI path template parameter (e.g., `/widgets/{widgetId}`)
    Template,
}
/// Describes a parameter (path, query, header, etc.) for an API resource
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Param {
    /// The type/location of the parameter
    pub style: ParamStyle,
    /// The name of the parameter
    pub name: String,
    /// The value of the parameter
    pub value: String,
    /// Whether the parameter is required
    pub required: bool,
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
    /// The scheme of the API endpoint
    #[builder(default = Scheme::HTTPS)]
    pub scheme: Scheme,
    /// The domain of the API endpoint
    pub domain: Option<String>,
    /// The port of the API endpoint
    pub port: Option<u16>,
    /// Authentication credentials for accessing the API endpoint
    pub authentication: Option<Authentication>,
    /// Root path for the API endpoint
    /// ### Example
    /// "v3.0" for ORCiD API
    #[builder(default = String::new())]
    pub root: String,
    /// Resource data for generating full paths for the API endpoint using templates
    #[builder(default = vec![])]
    pub resources: Vec<Resource>,
}
impl fmt::Display for AuthenticationScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                | AuthenticationScheme::Bearer => "Bearer",
                | AuthenticationScheme::Basic => "Basic",
                | AuthenticationScheme::ApiKey => "ApiKey",
                | AuthenticationScheme::OAuth2 => "OAuth2",
                | AuthenticationScheme::Custom(scheme) => scheme,
            }
        )
    }
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
impl RestfulInterface for Endpoint {
    fn build(&self) -> Context {
        Context::new()
    }
    fn handle<R>(&self, _response: R) -> Result<(), String>
    where
        R: Deserialize<'static>,
    {
        Ok(())
    }
    /// Invoke an endpoint resource with data and receive a response
    /// ### Example
    /// ```ignore
    /// let api = Endpoint::default();
    /// let data = Vec::new();
    /// let response = api.invoke::<orcid::SearchResponse>("search", Some(data));
    /// ```
    fn invoke<R, S>(&self, _action: S, _data: Option<Vec<(&str, &str)>>) -> Result<R, String>
    where
        R: Deserialize<'static>,
        S: AsRef<str>,
    {
        Err("Not implemented".to_string())
    }
}
// TODO: Define lookup maps for ORCiD, ROR, Vale, GitLab, GitHub, and Citeas APIs
// TODO: Import paths from WADL file
impl Endpoint {}

#[cfg(test)]
mod tests;
