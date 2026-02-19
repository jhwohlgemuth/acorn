//! Module for working with remote and local application programming interfaces (APIs)
//!
//! Interact with RESTful APIs, handling requests and responses, authentication, and data serialization.
//! Also supports communicating with interfaces like large language model (LLM) interfaces like Ollama.
use crate::io::{network_get_request, network_post_request, network_put_request};
use crate::prelude::io::ErrorKind;
use crate::prelude::Error;
use crate::util::constants::{URL_ENCODED_CARAT, URL_ENCODED_SPACE};
use crate::util::{detect_json, detect_xml};
use crate::{Location, Repository, Scheme};
use bon::Builder;
use core::fmt;
use derive_more::Display;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tera::{Context, Tera};
use uriparse::URI;
use validator::Validate;

pub mod citeas;
pub mod github;
pub mod gitlab;
pub mod orcid;
pub mod ror;
pub mod spdx;

/// Helper trait for working with list of endpoints
pub trait EndpointSearch {
    /// Filter list of endpoints by name and return the first match
    fn find_by_name(&self, value: impl Into<String>) -> Option<Endpoint>;
}
/// Helper trait for converting parameter collections into HTTP headers
pub trait IntoHeaders {
    /// Convert this value into a `reqwest::HeaderMap`, using only header-style parameters.
    fn into_headers(self) -> HeaderMap;
}
/// Helper trait combining common bounds for API query field types
pub trait QueryField: fmt::Display + for<'a> TryFrom<&'a str> {}
/// Trait for working with request-response cycle using HTTP methods like GET, POST, PUT, PATCH, and DELETE
/// against resource URLs that return structured data
pub trait RestfulInterface {
    /// Query field type used for request building
    type Query: QueryField + ValueValidator;
    /// Field list type used for response selection
    type Field: QueryField;

    /// Build context for endpoint paths
    fn context(&self, _params: Option<Vec<Param>>) -> Context {
        Context::new()
    }
    /// Handle a response from an endpoint request
    fn handle<R>(&self, response: Result<ResponseContent, Error>) -> Result<R, String>
    where
        R: for<'de> Deserialize<'de>;
    /// Send data to the endpoint and receive a response
    fn invoke_sync(&self, action: impl Into<String> + Clone, data: Option<Vec<Param>>) -> Result<ResponseContent, Error>;
    /// Send data to the endpoint and receive a response using explicit query/field types
    fn invoke_sync_with<Q, F>(&self, action: impl Into<String> + Clone, data: Option<Vec<Param>>) -> Result<ResponseContent, Error>
    where
        Q: QueryField + ValueValidator,
        F: QueryField;
    /// Parse JSON response string content
    fn parse_json<R>(&self, content: &str) -> Result<R, String>
    where
        R: for<'de> Deserialize<'de>;
    /// Parse XML response string content
    fn parse_xml<R>(&self, content: &str) -> Result<R, String>
    where
        R: for<'de> Deserialize<'de>;
}
/// Trait to enable validation of field values
pub trait ValueValidator {
    /// Verify associated field value is valid
    fn is_valid(&self, _value: &str) -> bool {
        true
    }
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
#[serde(rename_all = "lowercase")]
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
    /// Header parameter
    Header,
    /// Body parameter (data sent via POST or PUT request)
    Body,
    /// Value to be substituted directly into the URL path template
    TemplateValue,
}
/// Wrapper enum for including response content MIME type with response body text
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ResponseContent {
    /// JSON response content
    Json(String),
    /// Raw text response content
    Raw(String),
    /// XML response content
    Xml(String),
}
/// Type for Git(Hub/Lab) tree entry
#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "lowercase")]
pub enum TreeEntryType {
    /// List of files and directories
    ///
    /// See <https://docs.gitlab.com/api/repositories/#list-repository-tree>
    #[display("tree")]
    Tree,
    /// Base64 encoded content
    ///
    /// See <https://docs.gitlab.com/api/repositories/#get-a-blob-from-repository>
    #[display("blob")]
    Blob,
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
/// Empty struct used for cases where no query fields are needed
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct EmptyField(String);
/// Represents an API endpoint with a lookup table for calling various paths
/// ### Note
/// Paths use handlebars templating syntax for dynamic URL construction, powered by [Tera](https://keats.github.io/tera/)
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Serialize, Validate)]
#[builder(start_fn = at, on(String, into))]
pub struct Endpoint {
    /// The domain of the API endpoint
    #[builder(start_fn)]
    pub domain: String,
    /// The name of the API endpoint (used mainly for logging and identification)
    #[builder(default = String::new())]
    pub name: String,
    /// The scheme of the API endpoint
    #[serde(default)]
    pub scheme: Option<Scheme>,
    /// The port of the API endpoint
    pub port: Option<u16>,
    /// Authentication credentials for accessing the API endpoint
    pub authentication: Option<Authentication>,
    /// Root path for the API endpoint
    /// ### Example
    /// "v3.0" for ORCiD API
    pub root: Option<String>,
    /// Resource data for generating full paths for the API endpoint using templates
    #[builder(default = vec![])]
    pub resources: Vec<Resource>,
}
/// Describes a parameter (path, query, header, etc.) for an API resource
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = of_type, finish_fn = with_key, on(String, into))]
pub struct Param {
    /// The type/location of the parameter
    #[builder(start_fn)]
    pub style: ParamStyle,
    /// The name of the parameter (e.g., "q", "fl", etc.)
    #[builder(finish_fn)]
    pub name: String,
    /// Value(s) of the parameter
    #[builder(
        default = vec![],
        with = |pairs: Vec<(Option<&str>, Option<&str>)>| {
            pairs
                .into_iter()
                .map(|(k, v)| (k.map(str::to_string), v.map(str::to_string)))
                .collect()
        }
    )]
    pub values: Vec<(Option<String>, Option<String>)>,
    /// Whether or not the parameter is required
    #[builder(default = false)]
    pub required: bool,
}
/// Represents a resource for an API endpoint, which can be used to generate full paths for requests
#[derive(Builder, Clone, Debug, Deserialize, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
pub struct Resource {
    /// Resource name (e.g., "search", "status")
    pub name: String,
    /// HTTP method to use when invoking this resource (e.g., GET, POST)
    #[builder(with = |method: &str| HttpMethod::from(method))]
    #[serde(default)]
    pub method: HttpMethod,
    /// Template for the resource path (e.g., "/expanded-search/{{ query }}")
    pub template: String,
}
/// Wrapper struct for raw text responses that cannot be parsed as JSON or XML
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextResponse {
    /// The raw text content from the response
    pub content: String,
}
// TODO: Define lookup maps for ORCiD, ROR, Vale, GitLab, GitHub, and Citeas APIs
// TODO: Import paths from OpenAPI spec
impl Endpoint {
    /// Get the base URL for the API endpoint, constructed from scheme, domain, and port if not provided
    pub fn base(&self) -> String {
        let Self { domain, root, .. } = self;
        let scheme = self.scheme.as_ref().map_or("https".to_string(), |s| s.to_string());
        let port = self.port.map_or(String::new(), |port| format!(":{port}"));
        let root = root.as_ref().map_or(String::new(), |root| format!("/{root}"));
        format!("{scheme}://{domain}{port}{root}")
    }

    /// Build context for endpoint paths using explicit query/field types.
    pub fn context_with<Q, F>(&self, data: Option<Vec<Param>>) -> Context
    where
        Q: QueryField + ValueValidator,
        F: QueryField,
    {
        let mut context = Context::new();
        match data {
            | Some(params) => {
                let (query_params, other_params): (Vec<Param>, Vec<Param>) = params.into_iter().partition(|param| param.is_query());
                let query = Param::to_query_string::<Q, F>(query_params);
                context.insert("query", &query);
                other_params.into_iter().for_each(|param| {
                    if param.is_template() {
                        param.values.into_iter().filter_map(|(k, v)| k.or(v)).for_each(|value| {
                            let key = param.name.clone();
                            context.insert(&key, &value.clone());
                        });
                    }
                });
                // TODO: Add header/body params to request builder
                dbg!(&context);
            }
            | None => (),
        }
        context.insert("base", &self.base());
        context
    }
}
impl EndpointSearch for Vec<Endpoint> {
    fn find_by_name(&self, value: impl Into<String>) -> Option<Endpoint> {
        let name = value.into();
        self.iter().find(|endpoint| endpoint.name == name).cloned()
    }
}
/// Blanket implementation for all types that satisfy the bounds
impl<T> QueryField for T where T: fmt::Display + for<'a> TryFrom<&'a str> {}
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
impl fmt::Display for EmptyField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Default for Endpoint {
    fn default() -> Self {
        Endpoint::at("https://example.com").build()
    }
}
impl From<&str> for HttpMethod {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            | "GET" => HttpMethod::Get,
            | "POST" => HttpMethod::Post,
            | "PUT" => HttpMethod::Put,
            | "PATCH" => HttpMethod::Patch,
            | "DELETE" => HttpMethod::Delete,
            | _ => HttpMethod::Get,
        }
    }
}
impl<'a> From<URI<'a>> for Endpoint {
    fn from(value: URI<'a>) -> Self {
        let domain: String = value.host().map(|h| format!("{}://{}", value.scheme(), h)).unwrap_or_default();
        let port: Option<u16> = value.authority().and_then(|auth| auth.port());
        Endpoint::at(domain).maybe_port(port).build()
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
impl Param {
    /// Check if this parameter is a body parameter
    pub fn is_body(&self) -> bool {
        matches!(self.style, ParamStyle::Body)
    }
    /// Check if this parameter is a query parameter (either a query pair, boosted query field, or field list)
    pub fn is_query(&self) -> bool {
        matches!(self.style, ParamStyle::QueryPair | ParamStyle::QueryField | ParamStyle::FieldList)
    }
    /// Check if this parameter is a header parameter
    pub fn is_header(&self) -> bool {
        matches!(self.style, ParamStyle::Header)
    }
    /// Check if this parameter is a template value
    pub fn is_template(&self) -> bool {
        matches!(self.style, ParamStyle::TemplateValue)
    }
    /// Convert a list of params to a query string
    pub fn to_query_string<Q: QueryField + ValueValidator, F: QueryField>(params: Vec<Param>) -> String {
        let query = params
            .iter()
            .filter(|param| param.is_query())
            .map(|param| param.to_string::<Q, F>())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
            .join("&");
        if !query.is_empty() {
            format!("?{query}")
        } else {
            String::new()
        }
    }
    /// Create a query pair parameter with key-value pairs
    /// ### Example
    /// ```ignore
    /// let param = Param::from_query_pair("q", vec![("given-names", "Jason"), ("family-name", "Wohlgemuth")]);
    /// let rendered = param.to_string::<orcid::SearchField, orcid::OutputColumn>();
    /// let expected = "q=given-names:Jason+AND+family-name:Wohlgemuth";
    /// assert_eq!(rendered, expected);
    /// ```
    pub fn from_query_pair(key: &str, pairs: Vec<(&str, &str)>) -> Self {
        Param::of_type(ParamStyle::QueryPair)
            .values(pairs.into_iter().map(|(k, v)| (Some(k), Some(v))).collect())
            .with_key(key)
    }
    /// Create a field list parameter with a list of field names
    pub fn from_field_list(key: &str, fields: Vec<&str>) -> Self {
        Param::of_type(ParamStyle::FieldList)
            .values(fields.into_iter().map(|f| (Some(f), None)).collect())
            .with_key(key)
    }
    /// Create a boosted query field parameter with a list of field names
    pub fn from_query_field(key: &str, fields: Vec<&str>) -> Self {
        Param::of_type(ParamStyle::QueryField)
            .values(fields.into_iter().map(|f| (Some(f), None)).collect())
            .with_key(key)
    }

    /// Render this parameter to a query string using the provided field types.
    /// - `Q` is used for query pairs and boosted query fields and must support validation (`Validate` trait).
    /// - `F` is used for field lists (e.g., output columns).
    pub fn to_string<Q: QueryField + ValueValidator, F: QueryField>(&self) -> String {
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
            | _ => None,
        };
        rendered.unwrap_or_default()
    }
}
impl IntoHeaders for Vec<Param> {
    fn into_headers(self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        self.into_iter().filter(|param| param.is_header()).for_each(|param| {
            let Param { name, values, .. } = param;
            let name: HeaderName = match name.parse() {
                | Ok(header_name) => header_name,
                | Err(_) => return,
            };
            values.into_iter().for_each(|(_, value)| {
                if let Some(raw) = value {
                    if let Ok(mut header_value) = HeaderValue::from_str(&raw) {
                        header_value.set_sensitive(true);
                        headers.append(name.clone(), header_value);
                    }
                }
            });
        });
        headers
    }
}
impl RestfulInterface for Endpoint {
    type Query = EmptyField;
    type Field = EmptyField;

    fn context(&self, data: Option<Vec<Param>>) -> Context {
        self.context_with::<Self::Query, Self::Field>(data)
    }
    fn handle<R>(&self, response: Result<ResponseContent, Error>) -> Result<R, String>
    where
        R: for<'de> Deserialize<'de>,
    {
        match response {
            | Ok(content) => match content {
                | ResponseContent::Json(content) => self.parse_json(&content),
                | ResponseContent::Xml(content) => self.parse_xml(&content),
                | ResponseContent::Raw(content) => {
                    let raw = TextResponse { content };
                    serde_json::to_string(&raw)
                        .map_err(|e| e.to_string())
                        .and_then(|json| self.parse_json(&json))
                }
            },
            | Err(e) => Err(e.to_string()),
        }
    }
    /// Invoke an endpoint resource with data and receive a response using [`EmptyField`] as the default query and field types.
    /// ### Example
    /// ```ignore
    /// let ror = endpoints.find_by_name("ror");
    /// let text = match &ror {
    ///     | Some(endpoint) => {
    ///         let response = endpoint.invoke_sync("status", None);
    ///         endpoint.handle::<api::TextResponse>(response)
    ///     }
    ///     | None => Err("No ROR endpoint found".into()),
    /// };
    /// println!("ROR Status: {text:#?}");
    /// ```
    fn invoke_sync(&self, name: impl Into<String> + Clone, data: Option<Vec<Param>>) -> Result<ResponseContent, Error> {
        self.invoke_sync_with::<Self::Query, Self::Field>(name, data)
    }
    /// Invoke an endpoint resource with data and receive a response using explicit query and field types.
    /// ### Example
    /// ```ignore
    /// let orcid = endpoints.find_by_name("orcid");
    /// let text = match &orcid {
    ///     | Some(endpoint) => {
    ///         let data = vec![
    ///             api::Param::of_type(api::ParamStyle::QueryPair)
    ///                 .values(vec![
    ///                     (Some("affiliation-org-name"), Some("Lyrasis")),
    ///                     (Some("ror-org-id"), Some("\"https://ror.org/01qz5mb56\"")),
    ///                 ])
    ///                 .with_key("q"),
    ///             api::Param::of_type(api::ParamStyle::FieldList)
    ///                 .values(vec![(Some("family-name"), None)])
    ///                 .with_key("fl"),
    ///         ];
    ///         let response = endpoint.invoke_sync_with::<api::orcid::SearchField, api::orcid::OutputColumn>("search", Some(data));
    ///         endpoint.handle::<api::orcid::SearchResponse>(response)
    ///     }
    ///     | None => Err("No ORCiD endpoint found".into()),
    /// };
    /// println!("ORCiD Search Response: {text:#?}");
    /// ```
    fn invoke_sync_with<Q, F>(&self, name: impl Into<String> + Clone, data: Option<Vec<Param>>) -> Result<ResponseContent, Error>
    where
        Q: QueryField + ValueValidator,
        F: QueryField,
    {
        let Self { resources, .. } = self;
        let mut tera = Tera::default();
        let context = self.context_with::<Q, F>(data.clone());
        let resource = resources.iter().find(|resource| resource.name == name.clone().into());
        match resource {
            | Some(Resource { method, template, .. }) => {
                let path = tera.render_str(template, &context).unwrap_or_default();
                let request = match method {
                    | HttpMethod::Get => network_get_request(path),
                    // TODO: Pass Body params for POST and PUT requests
                    | HttpMethod::Post => network_post_request(path),
                    | HttpMethod::Put => network_put_request(path),
                    | _ => unimplemented!("Only GET, POST, and PUT methods are currently implemented"),
                };
                let headers = data.unwrap_or_default().into_headers();
                match request.headers(headers).send() {
                    | Ok(response) => match response.text() {
                        | Ok(text) => {
                            let content = if detect_json(&text) {
                                ResponseContent::Json(text)
                            } else if detect_xml(&text) {
                                ResponseContent::Xml(text)
                            } else {
                                ResponseContent::Raw(text)
                            };
                            Ok(content)
                        }
                        | Err(why) => Err(Error::other(why.to_string())),
                    },
                    | Err(why) => Err(Error::other(why.to_string())),
                }
            }
            | None => Err(Error::new(ErrorKind::NotFound, "Resource not found")),
        }
    }
    fn parse_json<R>(&self, content: &str) -> Result<R, String>
    where
        R: for<'de> Deserialize<'de>,
    {
        match serde_json::from_str::<R>(content) {
            | Ok(response) => Ok(response),
            | Err(why) => Err(why.to_string()),
        }
    }
    fn parse_xml<R>(&self, content: &str) -> Result<R, String>
    where
        R: for<'de> Deserialize<'de>,
    {
        match quick_xml::de::from_str::<R>(content) {
            | Ok(response) => Ok(response),
            | Err(why) => Err(why.to_string()),
        }
    }
}
impl TryFrom<&str> for EmptyField {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(EmptyField(value.to_string()))
    }
}
impl ValueValidator for EmptyField {
    fn is_valid(&self, _value: &str) -> bool {
        true
    }
}
/// Create a query string from a lookup table of key-value pairs with field validation
pub fn param_from_query_pairs<T: QueryField + ValueValidator>(key: &str, separator: &str, pairs: Vec<(&str, &str)>) -> Option<String> {
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
                .map(|(i, field)| format!("{}{URL_ENCODED_CARAT}{}.0", field, count + 1 - i))
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
pub fn query_string<Q: QueryField + ValueValidator, F: QueryField>(
    query_pairs: Vec<(&str, &str)>,
    field_list: Vec<&str>,
    query_fields: Vec<&str>,
) -> String {
    let params = vec![
        Param::from_query_pair("q", query_pairs),
        Param::from_field_list("fl", field_list),
        Param::from_query_field("qf", query_fields),
    ];
    Param::to_query_string::<Q, F>(params)
}

#[cfg(test)]
mod tests;
