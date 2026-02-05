//! Module for working with remote and local application programming interfaces (APIs)
//!
//! Interact with RESTful APIs, handling requests and responses, authentication, and data serialization.
//! Also supports communicating with interfaces like large language model (LLM) interfaces like Ollama.
use crate::prelude::HashMap;
use crate::{Location, Repository};
use bon::Builder;
use core::fmt;
use serde::{Deserialize, Serialize};
// use tera::{Context, Tera};
use uriparse::URI;

pub mod citeas;
pub mod orcid;

/// Trait for working with request-response cycle using HTTP methods like GET, POST, PUT, PATCH, and DELETE
/// against resource URLs that return structured data
pub trait RestfulInterface {
    /// Build context for endpoint paths
    fn build(&self) -> Result<Self, String>
    where
        Self: Sized;
    /// Handle a response from an endpoint request
    fn handle<R>(&self, response: R) -> Result<(), String>
    where
        R: Deserialize<'static>;
    /// Send data to the endpoint and receive a response
    fn send<R>(&self, data: Option<HashMap<String, String>>) -> Result<R, String>
    where
        R: Deserialize<'static>;
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
    /// Path data for generating full URLs using templates
    #[builder(default = HashMap::new())]
    pub paths: HashMap<String, String>,
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
    fn build(&self) -> Result<Self, String> {
        Ok(self.clone())
    }
    fn handle<R>(&self, _response: R) -> Result<(), String>
    where
        R: Deserialize<'static>,
    {
        Ok(())
    }
    fn send<R>(&self, _data: Option<HashMap<String, String>>) -> Result<R, String>
    where
        R: Deserialize<'static>,
    {
        Err("Not implemented".to_string())
    }
}
// TODO: Define lookup maps for ORCiD, ROR, Vale, GitLab, GitHub, and Citeas APIs
impl Endpoint {}

#[cfg(test)]
mod tests;
