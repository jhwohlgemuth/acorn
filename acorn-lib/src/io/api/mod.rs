//! Module for working with remote and local application programming interfaces (APIs)
//!
//! Interact with RESTful APIs, handling requests and responses, authentication, and data serialization.
//! Also supports communicating with interfaces like large language model (LLM) interfaces like Ollama.
use crate::io::config::ApplicationConfiguration;
use crate::io::database::{schema::Table, Database};
use crate::io::http::HttpMethod;
use crate::io::http::{delete, get, patch, post, put};
use crate::io::ApiResult;
use crate::param;
use crate::util::constants::{URL_ENCODED_CARAT, URL_ENCODED_SPACE};
use crate::util::{detect_json, detect_xml, Constant, Label, Searchable};
use crate::{Location, Repository, Scheme};
use async_trait::async_trait;
use axum::http::header::{HeaderName, HeaderValue};
use axum::http::HeaderMap;
use bon::Builder;
use color_eyre::eyre::{self, eyre};
use core::iter::once;
use core::{fmt, marker::PhantomData};
use derive_more::Display;
use fluent_uri::{Uri, UriRef};
use itertools::Itertools;
use lazy_static::lazy_static;
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::EnumIs;
use tera::{Context, Tera};
use tracing::{debug, trace, warn};
use validator::Validate;

pub mod citeas;
pub mod geonames;
pub mod github;
pub mod gitlab;
pub mod huggingface;
pub mod models_dev;
pub mod openai;
pub mod openapi;
pub mod orcid;
pub mod raid;
pub mod ror;
pub mod spdx;

lazy_static! {
    /// Vector of API endpoints used during the endeavor of scientific research, communication, and collaboration
    pub static ref INCLUDED_ENDPOINTS: Vec<Endpoint> = Constant::json::<ApplicationConfiguration>("application").endpoints.unwrap_or_default();
}
/// Returns the value of the first header in `names` that is present and valid UTF-8.
pub fn first_header<'a>(headers: &'a HeaderMap, names: &[&str]) -> Option<&'a str> {
    names.iter().filter_map(|name| headers.get(*name)).find_map(|value| value.to_str().ok())
}
/// Normalize an external username into a stable path component
pub fn sluggify(username: &str, user_id: u64) -> String {
    let (slug, _) = username
        .trim()
        .to_ascii_lowercase()
        .chars()
        .fold((String::new(), false), |(mut value, separator), character| {
            if character.is_ascii_alphanumeric() {
                value.push(character);
                (value, false)
            } else if !value.is_empty() && !separator {
                value.push('-');
                (value, true)
            } else {
                (value, separator)
            }
        });
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        format!("user-{user_id}")
    } else {
        slug.to_string()
    }
}
/// Trait to standardize configuration loading from environment variables and modification with new values
pub trait Configuration {
    /// Populate values from environment (e.g., `.env` file or environment variables)
    fn from_env() -> Self;
    /// Return a copy of this configuration with the specified request body payload set
    fn with_body(self, value: impl Into<String>) -> Self;
    /// Return a copy of this configuration with the specified domain set
    fn with_domain(self, value: impl Into<String>) -> Self;
    /// Return a copy of this configuration with the specified resource identifier set
    fn with_identifier(self, value: impl Into<String>) -> Self;
    /// Return the authentication token
    fn token(&self) -> &str;
    /// Return the API domain
    fn domain(&self) -> &str;
    /// Return the optional resource identifier
    fn identifier(&self) -> Option<&str>;
    /// Return a copy of this configuration with custom API parameters set.
    /// These are appended to internally-constructed parameters before each request.
    fn with_params(self, params: Vec<Param>) -> Self;
    /// Return any custom API parameters
    fn params(&self) -> &[Param];
}
/// Trait for objects that can be persisted in a database
#[async_trait]
pub trait DatabasePersistence {
    /// Persist data to database
    async fn persist(self, database: Database<Table>) -> ApiResult<usize>;
}
/// Helper trait for converting parameter collections into HTTP request body
pub trait IntoBody {
    /// Convert this value into a `serde_json::Value` for request body, using only body-style parameters.
    fn into_body(self) -> serde_json::Value;
}
/// Helper trait for converting parameter collections into HTTP headers
pub trait IntoHeaders {
    /// Convert this value into a `HeaderMap`, using only header-style parameters
    fn into_headers(self) -> HeaderMap;
}
/// Helper trait combining common bounds for API query field types
pub trait QueryField: fmt::Display + for<'a> TryFrom<&'a str> {}
/// Common repository file metadata exposed by provider APIs
pub trait RepositoryFileMetadata {
    /// Return the repository-relative path
    fn path(&self) -> &str;
    /// Return the reported file size when available
    fn size(&self) -> Option<u64>;
}
/// Generic API response containing a single identifier
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Identifier<T> {
    /// Provider-specific identifier
    #[serde(rename = "id", alias = "iid")]
    pub identifier: T,
}
/// Trait for types that can serve as a fallback error response parser
///
/// When `handle_or` fails to parse the primary response type, it calls
/// `into_error` on the fallback type to attempt an alternative parse
/// and surface a more descriptive error. Use [`NoFallback`] (the default)
/// when no fallback is needed, or [`FallbackFor<T>`] to wrap a concrete
/// error-response type.
pub trait FallbackResponse {
    /// Attempt to parse `content` as a fallback error type and return an
    /// error report, or `None` if parsing also fails.
    fn into_error(content: &str) -> Option<eyre::Report>;
    /// Attempt to parse and pretty-print JSON content for readable fallback output.
    fn to_string(content: &str) -> Option<String> {
        serde_json::from_str::<serde_json::Value>(content).ok().and_then(|value| match value {
            | serde_json::Value::String(inner) => serde_json::from_str::<serde_json::Value>(&inner)
                .ok()
                .and_then(|nested| serde_json::to_string_pretty(&nested).ok())
                .or_else(|| serde_json::to_string_pretty(&serde_json::Value::String(inner)).ok()),
            | other => serde_json::to_string_pretty(&other).ok(),
        })
    }
}
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
    fn handle<R>(&self, response: ApiResult<ResponseContent>) -> ApiResult<R>
    where
        R: for<'de> Deserialize<'de>,
    {
        match response {
            | Ok(content) => match content {
                | ResponseContent::Json(content) => parse_json(&content),
                | ResponseContent::Xml(content) => parse_xml(&content),
                | ResponseContent::Yaml(content) => parse_yaml(&content),
                | ResponseContent::Raw(content) => {
                    let raw = TextResponse { content };
                    serde_json::to_string(&raw).map_err(|e| eyre!(e)).and_then(|json| parse_json(&json))
                }
            },
            | Err(e) => Err(eyre!(e)),
        }
    }
    /// Handle a response from an endpoint request, trying to parse as `R` first,
    /// then falling back to `E` on parse failure to surface a richer error message
    fn handle_or<R, E>(&self, response: ApiResult<ResponseContent>) -> ApiResult<R>
    where
        R: for<'de> Deserialize<'de>,
        E: FallbackResponse;
    /// Send data to the endpoint and receive a response asynchronously
    async fn invoke(&self, action: impl Into<String> + Clone + Send, data: Option<Vec<Param>>) -> ApiResult<ResponseContent>;
    /// Send data to the endpoint and receive a response asynchronously using explicit query/field types
    async fn invoke_with<Q, F>(&self, action: impl Into<String> + Clone + Send, data: Option<Vec<Param>>) -> ApiResult<ResponseContent>
    where
        Q: QueryField + ValueValidator,
        F: QueryField;
}
/// Trait to enable validation of field values
pub trait ValueValidator {
    /// Verify associated field value is valid
    fn is_valid(&self, _value: &str) -> bool {
        true
    }
}
/// Authentication schemes supported for API requests
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize)]
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
    /// AWS Signature V4 request signing
    AwsSignatureV4,
    /// Google Cloud Application Default Credentials
    GoogleCloud,
    /// Custom authentication scheme
    Custom(String),
}
/// Describes the location/type of a parameter for an API resource
#[derive(Clone, Debug, Default, Deserialize, EnumIs, Serialize)]
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
    /// YAML response content
    Yaml(String),
    /// XML response content
    Xml(String),
}
/// Generic repository tree entry shared across provider APIs
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TreeEntry {
    /// Repository-relative path
    pub path: String,
    /// Entry type normalized across provider naming
    #[serde(rename = "type")]
    pub entry_type: TreeEntryType,
    /// File size in bytes, when provided
    pub size: Option<u64>,
    /// Provider-specific identifier, when provided
    pub id: Option<String>,
    /// Entry basename, when provided
    pub name: Option<String>,
    /// File mode, when provided
    pub mode: Option<String>,
    /// Git object SHA, when provided
    pub sha: Option<String>,
    /// Provider URL, when provided
    pub url: Option<String>,
}
/// Type for repository tree entry
#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "lowercase")]
pub enum TreeEntryType {
    /// File-like repository entry
    #[serde(alias = "blob")]
    #[display("file")]
    File,
    /// Directory-like repository entry
    #[serde(alias = "tree")]
    #[display("directory")]
    Directory,
}
impl TreeEntry {
    /// Whether this tree entry is a file
    pub fn is_file(&self) -> bool {
        self.entry_type == TreeEntryType::File
    }
    /// Whether this tree entry is a directory
    pub fn is_directory(&self) -> bool {
        self.entry_type == TreeEntryType::Directory
    }
    /// Consume the entry and return its repository-relative path
    pub fn path(self) -> String {
        self.path
    }
}
/// Wrapper that enables any `Deserialize + fmt::Debug` type as a fallback
/// error response parser
///
/// ### Example
/// ```ignore
/// endpoint.handle_or::<Metadata, Fallback<ErrorResponse>>(response)
/// ```
pub struct Fallback<T>(PhantomData<T>);
/// Default pass-through fallback — no secondary parse is attempted
pub struct NoFallback;
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
/// Builder for constructing `Vec<Param>` with a fluent, immutable API.
///
/// Methods consume `self` and return `Self` — no mutation, always chainable.
///
/// # Examples
///
/// ```ignore
/// use acorn::io::api::Params;
///
/// let params = Params::new()
///     .with_auth("sk-xxx", None)
///     .with_template("identifier", Some("chat-123"))
///     .with_keyvalue("limit", Some("10"))
///     .build();
/// ```
pub struct Params(Vec<Param>);
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
    /// Create an endpoint from explicit domain, scheme, and port parts.
    ///
    /// Unspecified parts fall back to the same defaults as [`Endpoint::default`].
    pub fn from_parts(domain: String, scheme: Option<Scheme>, port: Option<u16>) -> Self {
        Self {
            domain,
            scheme,
            port,
            ..Self::default()
        }
    }
    /// Get the base URL for the API endpoint, constructed from scheme, domain, and port if not provided
    pub fn base(&self) -> String {
        let Self { domain, root, .. } = self;
        let scheme = self.scheme.as_ref().map_or("https".to_string(), |s| s.to_string());
        let port = self.port.map_or(String::new(), |port| format!(":{port}"));
        let root = root.as_ref().map_or(String::new(), |root| format!("/{root}"));
        format!("{scheme}://{domain}{port}{root}")
    }
    /// Create a new endpoint with a custom domain while preserving all other properties
    pub fn with_domain(&self, domain: impl Into<String>) -> Self {
        let (domain, scheme, port) = Self::split_domain(domain.into().as_str());
        Self {
            domain,
            scheme: scheme.or_else(|| self.scheme.clone()),
            port: port.or(self.port),
            ..self.clone()
        }
    }
    fn split_domain(value: &str) -> (String, Option<Scheme>, Option<u16>) {
        Uri::parse(value)
            .ok()
            .and_then(|uri| {
                uri.authority().map(|authority| {
                    (
                        authority.host().to_string(),
                        Some(Scheme::from(uri.scheme().as_str())).filter(|scheme| *scheme != Scheme::Unsupported),
                        authority.port_to_u16().ok().flatten(),
                    )
                })
            })
            .or_else(|| {
                let authority = format!("//{}", value.trim());
                UriRef::parse(authority.as_str()).ok().and_then(|uri| {
                    uri.authority()
                        .map(|parsed| (parsed.host().to_string(), None, parsed.port_to_u16().ok().flatten()))
                })
            })
            .unwrap_or_else(|| (value.trim().to_string(), None, None))
    }
    /// Find an endpoint template by name from [`INCLUDED_ENDPOINTS`]
    ///
    /// # Example
    /// ```ignore
    /// let endpoint = Endpoint::from_template("gitlab")?.with_domain("my-gitlab.example.com");
    /// ```
    pub fn from_template(name: impl Into<String>) -> ApiResult<Self> {
        let endpoint_name = name.into();
        INCLUDED_ENDPOINTS
            .find_by_name(&endpoint_name)
            .ok_or_else(|| eyre!("Endpoint template '{endpoint_name}' not found in application configuration"))
    }
}
impl Searchable<Endpoint> for Vec<Endpoint> {
    fn find_by_name(&self, value: impl Into<String>) -> Option<Endpoint> {
        let name = value.into();
        self.iter().find(|endpoint| endpoint.name.eq_ignore_ascii_case(&name)).cloned()
    }
}
impl FallbackResponse for NoFallback {
    fn into_error(_: &str) -> Option<eyre::Report> {
        None
    }
    fn to_string(_: &str) -> Option<String> {
        None
    }
}
impl<T> FallbackResponse for Fallback<T>
where
    T: for<'de> Deserialize<'de> + fmt::Debug,
{
    fn into_error(content: &str) -> Option<eyre::Report> {
        serde_json::from_str::<T>(content).ok().map(|why| {
            let message = Self::to_string(content).unwrap_or_else(|| format!("{why:#?}"));
            eyre!("{message}")
        })
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
                | AuthenticationScheme::AwsSignatureV4 => "AWS Signature V4",
                | AuthenticationScheme::GoogleCloud => "Google Cloud",
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
        Endpoint::at("example.com").scheme(Scheme::default()).build()
    }
}
impl<'a> From<Uri<&'a str>> for Endpoint {
    fn from(value: Uri<&'a str>) -> Self {
        let domain = value.authority().map(|auth| auth.host().to_string()).unwrap_or_default();
        let port = value.authority().and_then(|auth| auth.port_to_u16().ok()).flatten();
        Self::from_parts(
            domain,
            Some(Scheme::from(value.scheme().as_str())).filter(|scheme| *scheme != Scheme::Unsupported),
            port,
        )
    }
}
impl From<Location> for Endpoint {
    fn from(value: Location) -> Self {
        let domain = value.host().unwrap_or_default();
        let port = value.port();
        Self::from_parts(domain, Some(value.scheme()), port)
    }
}
impl From<Repository> for Endpoint {
    fn from(value: Repository) -> Self {
        value.location().into()
    }
}
impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                | HttpMethod::Get => "GET",
                | HttpMethod::Post => "POST",
                | HttpMethod::Put => "PUT",
                | HttpMethod::Patch => "PATCH",
                | HttpMethod::Delete => "DELETE",
            }
        )
    }
}
impl Param {
    /// Check if this parameter is a query parameter (either a query pair, boosted query field, or field list)
    pub fn is_query(&self) -> bool {
        self.style.is_query_pair() | self.style.is_query_field() | self.style.is_field_list()
    }
    /// Convert a list of params to a query string
    pub fn to_query_string<Q: QueryField + ValueValidator, F: QueryField>(params: Vec<Param>) -> String {
        let query = params
            .iter()
            .filter(|param| param.is_query() || param.style.is_key_value_pair())
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
impl Default for Params {
    fn default() -> Self {
        Self::new()
    }
}
impl Params {
    /// Start building an empty param list
    pub fn new() -> Self {
        Self(Vec::new())
    }
    /// Build and return the underlying `Vec<Param>`
    pub fn build(self) -> Vec<Param> {
        self.0
    }
    /// Add any pre-built `Param` value
    pub fn with(self, param: Param) -> Self {
        Self(self.0.into_iter().chain(once(param)).collect())
    }
    /// Populate with Bearer auth header and identifier template value from a
    /// [`Configuration`] provider.
    pub fn from_config(config: &impl Configuration) -> Self {
        Self::new()
            .with_auth(config.token(), None)
            .with_template("identifier", config.identifier())
    }
    /// Add an authentication header if `token` is non-empty.
    ///
    /// When `name` is `None`, adds `Authorization: Bearer {token}` (Bearer auth).
    /// When `name` is `Some(header_name)`, adds `{header_name}: {token}` (custom header).
    ///
    /// Used by Bearer providers (OpenAI, RAiD) and custom-header providers (GitLab's `PRIVATE-TOKEN`).
    pub fn with_auth(self, token: &str, name: Option<&str>) -> Self {
        let value = token.trim();
        if !value.is_empty() {
            let (header_name, header_value): (&str, String) = match name {
                | None => ("Authorization", format!("Bearer {value}")),
                | Some(name) => (name, value.to_string()),
            };
            self.with(param!(Header, header_name, header_value.as_str()))
        } else {
            self
        }
    }
    /// Add a template-value parameter, skipping if the value is `None` or empty
    pub fn with_template(self, key: &str, value: Option<&str>) -> Self {
        match value {
            | Some(v) if !v.is_empty() => self.with(param!(ParamStyle::TemplateValue, key, v)),
            | _ => self,
        }
    }
    /// Add a query key-value pair, skipping if the value is `None` or empty
    pub fn with_keyvalue(self, key: &str, value: Option<&str>) -> Self {
        match value {
            | Some(v) if !v.is_empty() => self.with(param!(ParamStyle::KeyValuePair, key, v)),
            | _ => self,
        }
    }
    /// Add a named body parameter (always added)
    pub fn with_body(self, key: &str, value: &str) -> Self {
        self.with(param!(ParamStyle::Body, key, value))
    }
    /// Add a named body parameter, skipping if the value is `None` or empty
    pub fn with_body_maybe(self, key: &str, value: Option<&str>) -> Self {
        match value {
            | Some(v) if !v.is_empty() => self.with(param!(ParamStyle::Body, key, v)),
            | _ => self,
        }
    }
    /// Add a field-list parameter (always added)
    pub fn with_field(self, key: &str, value: &str) -> Self {
        self.with(param!(ParamStyle::FieldList, key, value))
    }
    /// Merge custom parameters into this param list.
    /// Skipped when `custom` is empty to avoid unnecessary allocation.
    pub fn with_custom(self, custom: &[Param]) -> Self {
        if custom.is_empty() {
            self
        } else {
            Self(self.0.iter().chain(custom.iter()).cloned().collect())
        }
    }
}
impl IntoBody for Vec<Param> {
    fn into_body(self) -> serde_json::Value {
        let params: Vec<Param> = self.into_iter().filter(|Param { style, .. }| style.is_body()).collect();
        match params.as_slice() {
            | [Param { name, values, .. }] if name.is_empty() => {
                let flattened: Vec<String> = values.iter().cloned().flat_map(|vec| vec.into_iter().flatten()).collect();
                if flattened.len() == 1 {
                    let raw = flattened.into_iter().next().unwrap_or_default();
                    serde_json::from_str::<serde_json::Value>(&raw).unwrap_or(serde_json::Value::String(raw))
                } else if flattened.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::Array(flattened.into_iter().map(serde_json::Value::String).collect())
                }
            }
            | _ => {
                let body = params
                    .into_iter()
                    .map(|param| {
                        let Param { name, values, .. } = param;
                        let flattened: Vec<String> = values.into_iter().flat_map(|vec| vec.into_iter().flatten()).collect();
                        let value = if flattened.len() == 1 {
                            #[allow(clippy::unwrap_used)]
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
    }
}
impl IntoHeaders for Vec<Param> {
    fn into_headers(self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        self.into_iter().filter(|Param { style, .. }| style.is_header()).for_each(|param| {
            let Param { name, values, .. } = param;
            if let Ok(header_name) = name.parse::<HeaderName>() {
                values.into_iter().for_each(|vec| {
                    vec.into_iter().for_each(|opt_value| {
                        if let Some(raw) = opt_value {
                            if let Ok(mut header_value) = HeaderValue::from_str(&raw) {
                                header_value.set_sensitive(true);
                                headers.append(header_name.clone(), header_value);
                            }
                        }
                    });
                });
            }
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
                    params.into_iter().partition(|param| param.is_query() || param.style.is_key_value_pair());
                let query = Param::to_query_string::<Q, F>(query_params);
                context.insert("query", &query);
                other_params.into_iter().for_each(|Param { name, style, values, .. }| {
                    if style.is_template_value() {
                        values.into_iter().for_each(|vec| {
                            vec.into_iter().flatten().for_each(|value| {
                                let key = name.clone();
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
    fn handle_or<R, E>(&self, response: ApiResult<ResponseContent>) -> ApiResult<R>
    where
        R: for<'de> Deserialize<'de>,
        E: FallbackResponse,
    {
        match response {
            | Ok(content) => {
                let raw_text = match &content {
                    | ResponseContent::Json(s) | ResponseContent::Xml(s) | ResponseContent::Yaml(s) | ResponseContent::Raw(s) => s.clone(),
                };
                let result: ApiResult<R> = match content {
                    | ResponseContent::Json(s) => parse_json(&s),
                    | ResponseContent::Xml(s) => parse_xml(&s),
                    | ResponseContent::Yaml(s) => parse_yaml(&s),
                    | ResponseContent::Raw(s) => {
                        let raw = TextResponse { content: s };
                        serde_json::to_string(&raw).map_err(|e| eyre!(e)).and_then(|json| parse_json(&json))
                    }
                };
                result.map_err(|err| E::into_error(&raw_text).unwrap_or(err))
            }
            | Err(why) => Err(eyre!(why)),
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
    ///     | None => Err(eyre!("No ROR endpoint found")),
    /// };
    /// println!("ROR Status: {text:#?}");
    /// ```
    async fn invoke(&self, name: impl Into<String> + Clone + Send, data: Option<Vec<Param>>) -> ApiResult<ResponseContent> {
        self.invoke_with::<Self::Query, Self::Field>(name, data).await
    }
    /// Invoke an endpoint resource asynchronously with data and receive a response using explicit query and field types.
    /// ### Example
    /// ```ignore
    /// use acorn::io::api::{self, INCLUDED_ENDPOINTS};
    /// use acorn::util::Searchable;
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
    ///     | None => Err(eyre!("No ORCiD endpoint found")),
    /// };
    /// println!("ORCiD Search Response: {text:#?}");
    /// ```
    async fn invoke_with<Q, F>(&self, name: impl Into<String> + Clone + Send, data: Option<Vec<Param>>) -> ApiResult<ResponseContent>
    where
        Q: QueryField + ValueValidator,
        F: QueryField,
    {
        let Self { resources, .. } = self;
        let context = self.context_with::<Q, F>(data.clone());
        let resource = resources.find_by_name(name);
        match resource {
            | Some(Resource { method, template, .. }) => {
                let path = render(&template, &context);
                let params = data.unwrap_or_default();
                let headers = params.clone().into_headers();
                let body = params.into_body();
                let request = match method {
                    | HttpMethod::Delete => delete(path),
                    | HttpMethod::Get => get(path),
                    | HttpMethod::Patch => patch(path).json(&body),
                    | HttpMethod::Post => post(path).json(&body),
                    | HttpMethod::Put => put(path).json(&body),
                };
                debug!("=> {} {}", Label::run(), request.cyan());
                match request.headers(headers).send().await {
                    | Ok(response) => match response.text().await {
                        | Ok(text) => {
                            trace!("=> {} Response {text}", Label::using());
                            let content = if detect_json(&text) {
                                ResponseContent::Json(text)
                            } else if detect_xml(&text) {
                                ResponseContent::Xml(text)
                            } else {
                                ResponseContent::Raw(text)
                            };
                            Ok(content)
                        }
                        | Err(why) => Err(eyre!(why)),
                    },
                    | Err(why) => Err(eyre!(why)),
                }
            }
            | None => Err(eyre!("Resource not found")),
        }
    }
}
impl TryFrom<&str> for EmptyField {
    type Error = String;

    fn try_from(value: &str) -> eyre::Result<Self, Self::Error> {
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
        trimmed
            .split('|')
            .next()
            .map(str::trim)
            .and_then(|base| base.split_whitespace().next().map(str::trim))
            .and_then(|key| (!key.is_empty()).then(|| key.to_string()))
    }
    template
        .split("{{")
        .skip(1)
        .filter_map(|segment| segment.split_once("}}").map(|(before, _)| before))
        .filter_map(extract_key)
        .unique()
        .collect()
}
/// Create a query string component from a key-value pair with key and value validation
pub(crate) fn param_from_key_value_pair<T: QueryField + ValueValidator>(key: &str, value: &str) -> Option<String> {
    match T::try_from(key) {
        | Ok(field) => {
            if field.is_valid(value) {
                Some(format!("{}={}", field, urlencoding::encode(value)))
            } else {
                warn!("=> {} Invalid key value ({}{})", Label::using(), format!("{key}=").dimmed(), value.red());
                None
            }
        }
        | Err(_) => {
            warn!("=> {} Invalid key ({}{})", Label::using(), key.red(), format!("={value}").dimmed());
            None
        }
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
                        warn!(
                            "=> {} Invalid query value ({}{})",
                            Label::using(),
                            format!("{key}=").dimmed(),
                            value.red()
                        );
                        None
                    }
                }
                | Err(_) => {
                    warn!("=> {} Invalid query key ({}{})", Label::using(), key.red(), format!("={value}").dimmed());
                    None
                }
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
                .map(|(i, field)| format!("{}{URL_ENCODED_CARAT}{}.0", field, count.saturating_add(1).saturating_sub(i)))
                .collect::<Vec<String>>()
                .join(separator),
        ))
    }
}
pub(crate) fn parse_json<R>(content: &str) -> ApiResult<R>
where
    R: for<'de> Deserialize<'de>,
{
    match serde_json::from_str::<R>(content) {
        | Ok(response) => Ok(response),
        | Err(why) => Err(eyre!(why)),
    }
}
pub(crate) fn parse_xml<R>(content: &str) -> ApiResult<R>
where
    R: for<'de> Deserialize<'de>,
{
    match quick_xml::de::from_str::<R>(content) {
        | Ok(response) => Ok(response),
        | Err(why) => Err(eyre!(why)),
    }
}
pub(crate) fn parse_yaml<R>(content: &str) -> ApiResult<R>
where
    R: for<'de> Deserialize<'de>,
{
    match serde_norway::from_str::<R>(content) {
        | Ok(response) => Ok(response),
        | Err(why) => Err(eyre!(why)),
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
pub(crate) fn render(template: &str, context: &Context) -> String {
    let missing_values = extract_template_keys(template)
        .into_iter()
        .filter(|key| !context.contains_key(key))
        .map(|key| (key, serde_json::Value::String(String::new())));
    let merged = match context.clone().into_json() {
        | serde_json::Value::Object(existing) => serde_json::Value::Object(existing.into_iter().chain(missing_values).collect()),
        | _ => serde_json::Value::Object(missing_values.collect()),
    };
    Context::from_serialize(merged)
        .ok()
        .and_then(|context| Tera::one_off(template, &context, false).ok())
        .unwrap_or_default()
}
/// Validate that a required secret is present and non-empty
/// ### Note
/// The returned error intentionally excludes secret values
pub(crate) fn require_non_empty_secret(secret: &str, path: &str, names: &[&str]) -> ApiResult<String> {
    let value = secret.trim();
    if value.is_empty() {
        let env_list = names.join(", ");
        Err(eyre!("Missing required token for {path} request. Set one of: {env_list}"))
    } else {
        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests;
