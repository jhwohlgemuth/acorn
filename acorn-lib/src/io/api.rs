//! Module for working with remote and local application programming interfaces (APIs)
//!
//! Interact with RESTful APIs, handling requests and responses, authentication, and data serialization.
//! Also supports communicating with interfaces like large language model (LLM) interfaces like Ollama.
use crate::prelude::HashMap;
use bon::Builder;
use serde::{Deserialize, Serialize};

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
    pub domain: String,
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
impl Endpoint {}
// TODO: Implement From<Repository> for Endpoint
// TODO: Implement From<URI> for Endpoint
// TODO: Define lookup maps for ORCiD, Vale, GitLab, GitHub, and Citeas APIs
