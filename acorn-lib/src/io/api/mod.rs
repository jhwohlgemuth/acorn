//! Module for working with remote and local application programming interfaces (APIs)
//!
//! Interact with RESTful APIs, handling requests and responses, authentication, and data serialization.
//! Also supports communicating with interfaces like large language model (LLM) interfaces like Ollama.
use crate::io::network_get_request;
use crate::util::constants::URL_ENCODED_SPACE;
use crate::{Location, Repository, Scheme};
use bon::Builder;
use core::fmt;
use serde::{Deserialize, Serialize};
use tera::Context;
use uriparse::URI;

pub mod citeas;
pub mod orcid;

/// Helper trait combining common bounds for API query field types
pub trait QueryField: fmt::Display + for<'a> TryFrom<&'a str> {}
/// ORCiD API resources (similar to OpenAPI spec or WADL file)
// pub const ORCID_RESOURCES: &'static [Resource; 2] = &[
//     ("search", HttpMethod::Get, "{{ base }}/expanded-search/{{ query }}"),
//     ("status", HttpMethod::Get, "{{ base }}/pubStatus"),
// ];
/// Trait to enable validation of field values
pub trait Validate {
    /// Verify associated field value is valid
    fn is_valid(&self, _value: &str) -> bool {
        true
    }
}
/// Blanket implementation for all types that satisfy the bounds
impl<T> QueryField for T where T: fmt::Display + for<'a> TryFrom<&'a str> {}
/// Trait for working with request-response cycle using HTTP methods like GET, POST, PUT, PATCH, and DELETE
/// against resource URLs that return structured data
pub trait RestfulInterface {
    /// Build context for endpoint paths
    fn build(&self) -> Context {
        Context::new()
    }
    /// Handle a response from an endpoint request
    fn handle<R>(&self, response: Result<String, reqwest::Error>) -> Result<R, String>
    where
        R: for<'de> Deserialize<'de>;
    /// Send data to the endpoint and receive a response
    fn invoke_sync<R, S>(&self, action: S, data: Option<Vec<Param>>) -> Result<String, reqwest::Error>
    where
        R: for<'de> Deserialize<'de>,
        S: AsRef<str>;
    /// Parse response string content
    fn parse<R>(&self, content: &str) -> Result<R, String>
    where
        R: for<'de> Deserialize<'de>;
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
    Get,
    /// Submit data to create a new resource or process information — not idempotent
    Post,
    /// Replace all current representations of the target resource with the uploaded content — idempotent
    Put,
    /// Apply partial modifications to a resource — not necessarily idempotent
    Patch,
    /// Delete a specified resource — idempotent
    Delete,
}
/// Describes the location/type of a parameter for an API resource
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum ParamStyle {
    /// Query parameter key-value pair (e.g., "given-names:Jason")
    #[default]
    QueryPair,
    /// Query parameter with list of field values — used for specifying fields to boost
    QueryField,
    /// Specifies response fields (e.g., "given-names,family-name")
    FieldList,
}
/// Represents authentication credentials for accessing an API
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init)]
pub struct Authentication {
    // TODO: Make token secret
    /// The token used for authenticating API requests
    pub token: Option<String>,
    /// The scheme used for authenticating API requests
    #[builder(default)]
    pub scheme: AuthenticationScheme,
}
/// Represents an API endpoint with a lookup table for calling various paths
/// ### Note
/// Paths use handlebars templating syntax for dynamic URL construction, powered by [Tera](https://keats.github.io/tera/)
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
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
    pub resources: Vec<(String, HttpMethod, Vec<Param>)>,
}
/// Describes a parameter (path, query, header, etc.) for an API resource
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
pub struct Param {
    /// The type/location of the parameter
    #[builder(default)]
    pub style: ParamStyle,
    /// The name of the parameter (e.g., "q", "fl", etc.)
    pub name: String,
    /// Value(s) of the parameter
    #[builder(default = vec![])]
    pub values: Vec<(Option<String>, Option<String>)>,
    /// Whether or not the parameter is required
    #[builder(default = false)]
    pub required: bool,
}
impl Param {
    /// Render this parameter to a query string using the provided field types.
    /// - `Q` is used for query pairs and boosted query fields and must support validation (`Validate` trait).
    /// - `F` is used for field lists (e.g., output columns).
    pub fn to_string<Q: Validate + QueryField, F: QueryField>(&self) -> String {
        let key = self.name.as_str();
        let rendered: Option<String> = match self.style {
            | ParamStyle::QueryPair => {
                let separator = "+AND+";
                let pairs: Vec<(&str, &str)> = self
                    .values
                    .iter()
                    .filter_map(|(key, value)| match (key.as_deref(), value.as_deref()) {
                        | (Some(k), Some(v)) => Some((k, v)),
                        | _ => None,
                    })
                    .collect();
                param_from_query_pairs::<Q>(key, separator, pairs)
            }
            | ParamStyle::QueryField => {
                let separator = URL_ENCODED_SPACE;
                let fields: Vec<&str> = self
                    .values
                    .iter()
                    .filter_map(|(key, value)| key.as_deref().or(value.as_deref()))
                    .collect();
                param_from_query_fields::<Q>(key, separator, fields)
            }
            | ParamStyle::FieldList => {
                let separator = ",";
                let fields: Vec<&str> = self
                    .values
                    .iter()
                    .filter_map(|(key, value)| key.as_deref().or(value.as_deref()))
                    .collect();
                param_from_field_list::<F>(key, separator, fields)
            }
        };
        rendered.unwrap_or_default()
    }
}
/// Represents a resource for an API endpoint, which can be used to generate full paths for requests
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
pub struct Resource {
    /// Resource name (e.g., "search", "status")
    pub name: String,
    /// HTTP method to use when invoking this resource (e.g., GET, POST)
    pub method: HttpMethod,
    /// Template for the resource path (e.g., "/expanded-search/{{ query }}")
    pub template: String,
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
        Endpoint::init().domain("https://example.com").build()
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
        let mut context = Context::new();
        context.insert("base", &self.base());
        context
    }
    fn handle<R>(&self, response: Result<String, reqwest::Error>) -> Result<R, String>
    where
        R: for<'de> Deserialize<'de>,
    {
        match response {
            | Ok(text) => self.parse(&text),
            | Err(e) => Err(e.to_string()),
        }
    }
    /// Invoke an endpoint resource with data and receive a response
    /// ### Example
    /// ```ignore
    /// ```
    fn invoke_sync<R, S>(&self, _name: S, _data: Option<Vec<Param>>) -> Result<String, reqwest::Error>
    where
        R: for<'de> Deserialize<'de>,
        S: AsRef<str>,
    {
        let _search_params = [Param::init()
            .style(ParamStyle::QueryPair)
            .name("q")
            .values(vec![
                (Some("given-names".to_string()), Some("Jason".to_string())),
                (Some("family-name".to_string()), Some("Wohlgemuth".to_string())),
            ])
            .build()];
        let _resources = &[
            Resource::init()
                .name("search")
                .method(HttpMethod::Get)
                .template("{{ base }}/expanded-search/{{ query }}")
                .build(),
            Resource::init()
                .name("status")
                .method(HttpMethod::Get)
                .template("{{ base }}/pubStatus")
                .build(),
        ];
        let Self { scheme, domain, root, .. } = self;
        let resource = "expanded-search";
        let data = vec![("given-names", "Jason"), ("family-name", "Wohlgemuth")];
        let query = orcid::query_string(data, vec![], vec![]);
        let path = [domain.as_deref().unwrap_or(""), root.as_str(), resource, &query].join("/");
        match network_get_request(format!("{scheme}://{path}")).send() {
            | Err(e) => Err(e),
            | Ok(response) => response.text(),
        }
    }
    fn parse<R>(&self, content: &str) -> Result<R, String>
    where
        R: for<'de> Deserialize<'de>,
    {
        match quick_xml::de::from_str::<R>(content) {
            | Ok(response) => Ok(response),
            | Err(e) => Err(e.to_string()),
        }
    }
}
/// Create a query string from a lookup table of key-value pairs with field validation
pub fn param_from_query_pairs<T: Validate + QueryField>(key: &str, separator: &str, pairs: Vec<(&str, &str)>) -> Option<String> {
    let values: Vec<String> = pairs
        .into_iter()
        .filter_map(|(k, v)| {
            let key: &str = k;
            let value: &str = v.trim();
            match T::try_from(key) {
                | Ok(field) => {
                    if field.is_valid(value) {
                        Some(format!("{}:{}", field, urlencoding::encode(value)))
                    } else {
                        #[cfg(feature = "std")]
                        {
                            use crate::util::Label;
                            use tracing::warn;
                            warn!("=> {} Invalid value {value} for field {field}", Label::fail());
                        }
                        None
                    }
                }
                | Err(_) => None,
            }
        })
        .collect();
    if values.is_empty() {
        None
    } else {
        Some(format!("{}={}", key, values.join(separator)))
    }
}
/// Create a query string from a list of field values
pub fn param_from_field_list<T: QueryField>(key: &str, separator: &str, fields: Vec<&str>) -> Option<String> {
    let values: Vec<String> = fields
        .into_iter()
        .filter_map(|value: &str| {
            let val = value;
            match T::try_from(val) {
                | Ok(column) => Some(column.to_string()),
                | Err(_) => None,
            }
        })
        .collect();
    if values.is_empty() {
        None
    } else {
        Some(format!("{key}={}", values.join(separator)))
    }
}
/// Create a boosted query string from a list of fields with weighted relevance
pub fn param_from_query_fields<T: QueryField>(key: &str, separator: &str, fields: Vec<&str>) -> Option<String> {
    let valid_fields: Vec<T> = fields.into_iter().filter_map(|value| T::try_from(value).ok()).collect();
    if valid_fields.is_empty() {
        None
    } else {
        let count = valid_fields.len();
        Some(format!(
            "{}={}",
            key,
            valid_fields
                .into_iter()
                .enumerate()
                .map(|(i, field)| format!("{}^{}.0", field, count + 1 - i))
                .collect::<Vec<String>>()
                .join(separator),
        ))
    }
}
/// Parse API response content into a structured data type using `quick_xml` for XML deserialization
pub fn parse<R>(content: &str) -> Result<R, String>
where
    R: for<'de> Deserialize<'de>,
{
    match quick_xml::de::from_str::<R>(content) {
        | Ok(response) => Ok(response),
        | Err(e) => Err(format!("Failed to parse ORCiD search response: {e}")),
    }
}
// TODO: Support OR statements for field values (e.g., affiliation-org-name:("University of Plymouth" OR "Plymouth University"))
/// Construct a query string for an endpoint API query from a list of field-value pairs, a list of fields, and a list of fields with boosted relevance.
///
/// The query string is constructed by joining the following parts with "&":
///
/// - The field-value pairs, joined with "+AND+", prefixed with "?q=".
/// - The list of fields, joined with ",", prefixed with "&fl=".
/// - The list of fields with boosted relevance, joined with URL encoded space, prefixed with "&qf=".
///
/// If the list of field-value pairs is empty, an empty string is returned.
pub fn query_string<Q: QueryField + Validate, F: QueryField>(
    query_pairs: Vec<(&str, &str)>,
    field_list: Vec<&str>,
    query_fields: Vec<&str>,
) -> String {
    // TODO: Refactor to use `Param` struct for query construction and validation
    let fields: Option<String> = param_from_query_pairs::<Q>("q", "+AND+", query_pairs);
    let columns: Option<String> = param_from_field_list::<F>("fl", ",", field_list);
    let boosted: Option<String> = param_from_query_fields::<Q>("qf", URL_ENCODED_SPACE, query_fields);
    if let Some(fields) = fields {
        let mut parts: Vec<String> = vec![format!("?{fields}")];
        if let Some(columns) = columns {
            parts.push(columns.to_string());
        }
        if let Some(values) = boosted {
            parts.push(values.to_string());
        }
        parts.join("&")
    } else {
        String::new()
    }
}
// TODO: Define lookup maps for ORCiD, ROR, Vale, GitLab, GitHub, and Citeas APIs
// TODO: Import paths from OpenAPI spec
impl Endpoint {
    /// Get the base URL for the API endpoint, constructed from scheme, domain, and port if not provided
    pub fn base(&self) -> String {
        format!(
            "{}://{}{}/{}",
            self.scheme,
            self.domain.clone().unwrap_or_default(),
            self.port.map_or(String::new(), |p| format!(":{p}")),
            self.root,
        )
    }
}

#[cfg(test)]
mod tests;
