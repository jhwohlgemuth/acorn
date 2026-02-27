//! Module for working with remote and local application programming interfaces (APIs)
//!
//! Interact with RESTful APIs, handling requests and responses, authentication, and data serialization.
//! Also supports communicating with interfaces like large language model (LLM) interfaces like Ollama.
use crate::io::config::ApplicationConfiguration;
use crate::io::{get, post, put};
use crate::prelude::io::ErrorKind;
use crate::prelude::Error;
use crate::util::constants::{ENDPOINTS_TEXT, URL_ENCODED_CARAT, URL_ENCODED_SPACE};
use crate::util::{detect_json, detect_xml};
use crate::{Location, Repository, Scheme};
use async_trait::async_trait;
use bon::Builder;
use core::fmt;
use derive_more::Display;
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashSet;
use tera::{Context, Tera};
use uriparse::URI;
use validator::Validate;

pub mod citeas;
pub mod geonames;
pub mod github;
pub mod gitlab;
pub mod orcid;
pub mod raid;
pub mod ror;
pub mod spdx;

lazy_static! {
    /// Vector of API endpoints used during the endeavor of scientific research, communication, and collaboration
    pub static ref INCLUDED_ENDPOINTS: Vec<Endpoint> = ApplicationConfiguration::parse(ENDPOINTS_TEXT).unwrap().endpoints.unwrap_or_default();
}

/// Helper trait for converting parameter collections into HTTP request body
pub trait IntoBody {
    /// Convert this value into a `serde_json::Value` for request body, using only body-style parameters.
    fn into_body(self) -> serde_json::Value;
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
#[async_trait]
pub trait RemoteResource {
    /// Query field type used for request building
    type Query: QueryField + ValueValidator;
    /// Field list type used for response selection
    type Field: QueryField;

    /// Build context for endpoint paths using the associated query and field types
    fn context(&self, params: Option<Vec<Param>>) -> Context {
        self.context_with::<Self::Query, Self::Field>(params)
    }
    /// Build context for endpoint paths using explicit query and field types
    fn context_with<Q, F>(&self, data: Option<Vec<Param>>) -> Context
    where
        Q: QueryField + ValueValidator,
        F: QueryField;
    /// Handle a response from an endpoint request
    fn handle<R>(&self, response: Result<ResponseContent, Error>) -> Result<R, String>
    where
        R: for<'de> Deserialize<'de>;
    /// Send data to the endpoint and receive a response asynchronously
    async fn invoke(&self, action: impl Into<String> + Clone + Send, data: Option<Vec<Param>>) -> Result<ResponseContent, Error>;
    /// Send data to the endpoint and receive a response asynchronously using explicit query/field types
    async fn invoke_with<Q, F>(&self, action: impl Into<String> + Clone + Send, data: Option<Vec<Param>>) -> Result<ResponseContent, Error>
    where
        Q: QueryField + ValueValidator,
        F: QueryField;
}
/// Helper trait for searching lists of named API elements
pub trait Searchable<T> {
    /// Check if a certain value is present in the list
    /// ### Note
    /// This method will differ greatly on the implementation and type of T
    fn contains(&self, _value: &str) -> bool {
        false
    }
    /// Filter list by ISO or ISO3 and return the first match
    ///### Note
    /// This method is specific to the `Country` type in the GeoNames API, but is included in the trait for convenience and consistency with `find_by_name`
    fn find_by_iso(&self, _value: impl Into<String>) -> Option<T> {
        None
    }
    /// Filter list by name and return the first match
    fn find_by_name(&self, value: impl Into<String>) -> Option<T>;
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
    /// Key-value pair parameter (e.g., "key=value")
    KeyValuePair,
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
        with = |vecs: Vec<Vec<Option<&str>>>| {
            vecs
                .into_iter()
                .map(|vec| vec.into_iter().map(|opt| opt.map(str::to_string)).collect())
                .collect()
        }
    )]
    pub values: Vec<Vec<Option<String>>>,
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
impl Endpoint {
    /// Get the base URL for the API endpoint, constructed from scheme, domain, and port if not provided
    pub fn base(&self) -> String {
        let Self { domain, root, .. } = self;
        let scheme = self.scheme.as_ref().map_or("https".to_string(), |s| s.to_string());
        let port = self.port.map_or(String::new(), |port| format!(":{port}"));
        let root = root.as_ref().map_or(String::new(), |root| format!("/{root}"));
        format!("{scheme}://{domain}{port}{root}")
    }
}
impl Searchable<Endpoint> for Vec<Endpoint> {
    fn find_by_name(&self, value: impl Into<String>) -> Option<Endpoint> {
        let name = value.into();
        self.iter().find(|endpoint| endpoint.name.eq_ignore_ascii_case(&name)).cloned()
    }
}
impl Searchable<Resource> for Vec<Resource> {
    fn find_by_name(&self, value: impl Into<String>) -> Option<Resource> {
        let name = value.into();
        self.iter().find(|resource| resource.name.eq_ignore_ascii_case(&name)).cloned()
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
impl fmt::Display for TextResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
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
    /// Check if this parameter is a key-value parameter
    pub fn is_key_value(&self) -> bool {
        matches!(self.style, ParamStyle::KeyValuePair)
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
            .filter(|param| param.is_query() || param.is_key_value())
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
            .values(pairs.into_iter().map(|(k, v)| vec![Some(k), Some(v)]).collect())
            .with_key(key)
    }
    /// Create a field list parameter with a list of field names
    pub fn from_field_list(key: &str, fields: Vec<&str>) -> Self {
        Param::of_type(ParamStyle::FieldList)
            .values(fields.into_iter().map(|f| vec![Some(f)]).collect())
            .with_key(key)
    }
    /// Create a boosted query field parameter with a list of field names
    pub fn from_query_field(key: &str, fields: Vec<&str>) -> Self {
        Param::of_type(ParamStyle::QueryField)
            .values(fields.into_iter().map(|f| vec![Some(f)]).collect())
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
                    .filter_map(
                        |vec| match (vec.first().and_then(|o| o.as_deref()), vec.get(1).and_then(|o| o.as_deref())) {
                            | (Some(k), Some(v)) => Some((k, v)),
                            | _ => None,
                        },
                    )
                    .collect();
                param_from_query_pairs::<Q>(key, separator, pairs)
            }
            | ParamStyle::QueryField => {
                let separator = URL_ENCODED_SPACE;
                let fields: Vec<&str> = self.values.iter().filter_map(|vec| vec.first().and_then(|o| o.as_deref())).collect();
                param_from_query_fields::<Q>(key, separator, fields)
            }
            | ParamStyle::FieldList => {
                let separator = ",";
                let fields: Vec<&str> = self.values.iter().filter_map(|vec| vec.first().and_then(|o| o.as_deref())).collect();
                param_from_field_list::<F>(key, separator, fields)
            }
            | ParamStyle::KeyValuePair => {
                let value = self
                    .values
                    .iter()
                    .filter_map(|vec| vec.first().and_then(|o| o.as_deref()))
                    .collect::<String>();
                param_from_key_value_pair::<Q>(key, &value)
            }
            | _ => None,
        };
        rendered.unwrap_or_default()
    }
}
impl IntoBody for Vec<Param> {
    fn into_body(self) -> serde_json::Value {
        let body = self
            .into_iter()
            .filter(|param| param.is_body())
            .map(|param| {
                let Param { name, values, .. } = param;
                let flattened: Vec<String> = values.into_iter().flat_map(|vec| vec.into_iter().flatten()).collect();
                let value = if flattened.len() == 1 {
                    serde_json::Value::String(flattened.into_iter().next().unwrap())
                } else if flattened.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::Array(flattened.into_iter().map(serde_json::Value::String).collect())
                };
                (name, value)
            })
            .collect();
        serde_json::Value::Object(body)
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
            values.into_iter().for_each(|vec| {
                vec.into_iter().for_each(|opt_value| {
                    if let Some(raw) = opt_value {
                        if let Ok(mut header_value) = HeaderValue::from_str(&raw) {
                            header_value.set_sensitive(true);
                            headers.append(name.clone(), header_value);
                        }
                    }
                });
            });
        });
        headers
    }
}
#[async_trait]
impl RemoteResource for Endpoint {
    type Query = EmptyField;
    type Field = EmptyField;

    fn context_with<Q, F>(&self, data: Option<Vec<Param>>) -> Context
    where
        Q: QueryField + ValueValidator,
        F: QueryField,
    {
        let mut context = Context::new();
        match data {
            | Some(params) => {
                let (query_params, other_params): (Vec<Param>, Vec<Param>) =
                    params.into_iter().partition(|param| param.is_query() || param.is_key_value());
                let query = Param::to_query_string::<Q, F>(query_params);
                context.insert("query", &query);
                other_params.into_iter().for_each(|param| {
                    if param.is_template() {
                        param.values.into_iter().for_each(|vec| {
                            vec.into_iter().flatten().for_each(|value| {
                                let key = param.name.clone();
                                context.insert(&key, &value.clone());
                            });
                        });
                    }
                });
            }
            | None => (),
        }
        context.insert("base", &self.base());
        context
    }
    fn handle<R>(&self, response: Result<ResponseContent, Error>) -> Result<R, String>
    where
        R: for<'de> Deserialize<'de>,
    {
        match response {
            | Ok(content) => match content {
                | ResponseContent::Json(content) => parse_json(&content),
                | ResponseContent::Xml(content) => parse_xml(&content),
                | ResponseContent::Raw(content) => {
                    let raw = TextResponse { content };
                    serde_json::to_string(&raw).map_err(|e| e.to_string()).and_then(|json| parse_json(&json))
                }
            },
            | Err(e) => Err(e.to_string()),
        }
    }
    /// Invoke an endpoint resource asynchronously with data and receive a response using [`EmptyField`] as the default query and field types.
    /// ### Example
    /// ```ignore
    /// let ror = endpoints.find_by_name("ror");
    /// let text = match &ror {
    ///     | Some(endpoint) => {
    ///         let response = endpoint.invoke("status", None).await;
    ///         endpoint.handle::<api::TextResponse>(response)
    ///     }
    ///     | None => Err("No ROR endpoint found".into()),
    /// };
    /// println!("ROR Status: {text:#?}");
    /// ```
    async fn invoke(&self, name: impl Into<String> + Clone + Send, data: Option<Vec<Param>>) -> Result<ResponseContent, Error> {
        self.invoke_with::<Self::Query, Self::Field>(name, data).await
    }
    /// Invoke an endpoint resource asynchronously with data and receive a response using explicit query and field types.
    /// ### Example
    /// ```ignore
    /// use acorn::io::api::{self, Searchable, INCLUDED_ENDPOINTS};
    ///
    /// let orcid = INCLUDED_ENDPOINTS.find_by_name("orcid");
    /// let text = match &orcid {
    ///     | Some(endpoint) => {
    ///         let data = vec![
    ///             param!(
    ///                 QueryPair,
    ///                 "q",
    ///                 (("affiliation-org-name", "Lyrasis"), ("ror-org-id", "\"https://ror.org/01qz5mb56\""),)
    ///             ),
    ///             param!(FieldList, "fl", "family-name"),
    ///         ];
    ///         let response = endpoint.invoke_with::<api::orcid::SearchField, api::orcid::OutputColumn>("search", Some(data)).await;
    ///         endpoint.handle::<api::orcid::SearchResponse>(response)
    ///     }
    ///     | None => Err("No ORCiD endpoint found".into()),
    /// };
    /// println!("ORCiD Search Response: {text:#?}");
    /// ```
    async fn invoke_with<Q, F>(&self, name: impl Into<String> + Clone + Send, data: Option<Vec<Param>>) -> Result<ResponseContent, Error>
    where
        Q: QueryField + ValueValidator,
        F: QueryField,
    {
        let Self { resources, .. } = self;
        let mut context = self.context_with::<Q, F>(data.clone());
        let resource = resources.find_by_name(name);
        match resource {
            | Some(Resource { method, template, .. }) => {
                let path = render(&template, &mut context);
                let params = data.unwrap_or_default();
                let headers = params.clone().into_headers();
                let body = params.into_body();
                dbg!(&path);
                let request = match method {
                    | HttpMethod::Get => get(path),
                    | HttpMethod::Post => post(path).json(&body),
                    | HttpMethod::Put => put(path).json(&body),
                    | _ => unimplemented!("Only GET, POST, and PUT methods are currently implemented"),
                };
                match request.headers(headers).send().await {
                    | Ok(response) => match response.text().await {
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
pub(crate) fn extract_template_keys(template: &str) -> Vec<String> {
    fn extract_key(expression: &str) -> Option<String> {
        let trimmed = expression.trim().trim_matches('-');
        let mut base_parts = trimmed.split('|');
        let base = match base_parts.next() {
            | Some(value) => value.trim(),
            | None => return None,
        };
        let mut key_parts = base.split_whitespace();
        let key = match key_parts.next() {
            | Some(value) => value.trim(),
            | None => return None,
        };
        if key.is_empty() {
            None
        } else {
            Some(key.to_string())
        }
    }
    let mut keys = Vec::new();
    let mut seen = HashSet::new();
    let mut remaining = template;
    while let Some(start) = remaining.find("{{") {
        remaining = &remaining[start + 2..];
        let end = match remaining.find("}}") {
            | Some(value) => value,
            | None => break,
        };
        let raw = &remaining[..end];
        if let Some(key) = extract_key(raw) {
            let key_value = key.to_string();
            if seen.insert(key_value.clone()) {
                keys.push(key_value);
            }
        }
        remaining = &remaining[end + 2..];
    }
    keys
}
/// Create a query string component from a key-value pair with key and value validation
pub(crate) fn param_from_key_value_pair<T: QueryField + ValueValidator>(key: &str, value: &str) -> Option<String> {
    match T::try_from(key) {
        | Ok(field) => {
            if field.is_valid(value) {
                Some(format!("{}={}", field, urlencoding::encode(value)))
            } else {
                None
            }
        }
        | Err(_) => None,
    }
}
/// Create a query string from a lookup table of key-value pairs with field validation
pub(crate) fn param_from_query_pairs<T: QueryField + ValueValidator>(key: &str, separator: &str, pairs: Vec<(&str, &str)>) -> Option<String> {
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
pub(crate) fn param_from_field_list<T: QueryField>(key: &str, separator: &str, fields: Vec<&str>) -> Option<String> {
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
pub(crate) fn param_from_query_fields<T: QueryField>(key: &str, separator: &str, fields: Vec<&str>) -> Option<String> {
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
pub(crate) fn parse_json<R>(content: &str) -> Result<R, String>
where
    R: for<'de> Deserialize<'de>,
{
    match serde_json::from_str::<R>(content) {
        | Ok(response) => Ok(response),
        | Err(why) => Err(why.to_string()),
    }
}
pub(crate) fn parse_xml<R>(content: &str) -> Result<R, String>
where
    R: for<'de> Deserialize<'de>,
{
    match quick_xml::de::from_str::<R>(content) {
        | Ok(response) => Ok(response),
        | Err(why) => Err(why.to_string()),
    }
}
/// Construct a query string for an endpoint API query from a list of field-value pairs, a list of fields, and a list of fields with boosted relevance.
///
/// The query string is constructed by joining the following parts with "&":
///
/// - The field-value pairs, joined with "+AND+", prefixed with "?q=".
/// - The list of fields, joined with ",", prefixed with "&fl=".
/// - The list of fields with boosted relevance, joined with URL encoded space, prefixed with "&qf=".
///
/// If the list of field-value pairs is empty, an empty string is returned.
pub(crate) fn query_string<Q: QueryField + ValueValidator, F: QueryField>(
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
pub(crate) fn render(template: &str, context: &mut Context) -> String {
    let mut tera = Tera::default();
    let keys = extract_template_keys(template);
    keys.into_iter().for_each(|key| {
        if !context.contains_key(&key) {
            context.insert(&key, "");
        }
    });
    tera.render_str(template, context).unwrap_or_default()
}

#[cfg(test)]
mod tests;
