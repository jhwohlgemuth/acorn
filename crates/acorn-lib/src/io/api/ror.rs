//! Module for interacting with [ROR REST API](https://ror.readme.io/docs/rest-api)
//!
//! Provides types and functions for constructing API queries with field validation and output column selection
//!
//! The ROR REST API supports three primary parameters, each of which has different use cases:
//! - `query`
//!   - [`crate::io::api::ParamStyle::KeyValuePair`] (value should be URL encoded)
//!   - Best for finding organization records by name keywords or by non-ROR identifiers (GRID, Wikidata, Crossref Funder ID, ISNI)
//! - `filter`
//!   - [`crate::io::api::ParamStyle::QueryPair`]
//!   - Filter by status, type, country, or continent
//! - `affiliation`
//!   - [`crate::io::api::ParamStyle::KeyValuePair`] (value should be URL encoded)
//!   - Best for large-scale programmatic matching of complex, unstructured text strings to ROR IDs
//!
//! ## Example Use
//!
//! ### Get API status
//! ```ignore
//! use acorn_lib::io::api;
//!
//! println!("ROR REST API is healthy: {}", api::ror::is_healthy().await);
//! ```
use crate::io::api::{ApiResult, Configuration, Param, Params, RemoteResource, TextResponse, INCLUDED_ENDPOINTS};
use crate::prelude::var;
use crate::schema::pid::{PersistentIdentifierParse, ROR};
use crate::util::{Label, Searchable};
use bon::Builder;
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::debug;
use validator::Validate;

/// ROR API options
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = with_token, on(String, into))]
pub struct Options {
    /// Bearer token for authentication
    #[builder(start_fn)]
    pub token: String,
    /// Request body payload
    pub body: Option<String>,
    /// ROR API domain (defaults to api.ror.org)
    #[builder(default = String::from("api.ror.org"))]
    pub domain: String,
    /// ROR identifier
    pub identifier: Option<String>,
    /// Custom API parameters to include in every request
    #[builder(default = vec![])]
    pub custom_params: Vec<Param>,
}
impl Configuration for Options {
    /// Build options from ROR environment variables
    /// - `ROR_API_TOKEN` -> `token` (optional — public API)
    /// - `ROR_SERVER_HOST` -> `domain` (defaults to api.ror.org)
    fn from_env() -> Self {
        if let Err(why) = dotenvy::from_filename(".env") {
            debug!("=> {} Load .env — {why}", Label::skip());
        }
        Self {
            token: var("ROR_API_TOKEN").unwrap_or_default(),
            body: None,
            domain: var("ROR_SERVER_HOST").unwrap_or_else(|_| String::from("api.ror.org")),
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

/// ROR API status response is just "OK"
pub type StatusResponse = TextResponse;
/// External identifier type for ROR API search results
#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExternalIdentifierType {
    /// An identifier from [Wikidata](https://www.wikidata.org/wiki/)
    #[default]
    Wikidata,
    /// An identifier from the [International Standard Name Identifier (ISNI)](https://isni.org/) system
    Isni,
    /// An identifier from the [FundRef](https://www.crossref.org/services/funder-registry/) system
    Fundref,
    /// An identifier from the [Global Research Identifier Database (GRID)](https://grid.ac/)
    ///
    /// <div class="warning">The GRID dataset is no longer being updated and the GRID identifiers are being deprecated in favor of ROR IDs. However, many ROR records still include GRID identifiers as external identifiers, so this type is included here for completeness.</div>
    Grid,
}
/// Link type
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    /// Organization's website
    Website,
    /// Organization's Wikipedia page
    Wikipedia,
    /// Organization's Wikidata page
    Wikidata,
}
/// ROR organization name type
#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationNameType {
    /// Name of the organization displayed most prominently on records in ROR's web search
    #[default]
    RorDisplay,
    /// One or more official acronyms or initialisms for the organization, typically consisting of the first letters of the words in the organization name
    Acronym,
    /// An alternative name for the organization
    Alias,
    /// Displays equivalent forms of the organization name in one or more languages
    Label,
}
/// ROR organization type
///
/// See `https://ror.readme.io/docs/ror-data-structure#types` for more information
#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OrganizationType {
    /// A specialized facility where research takes place, such as a laboratory or telescope or dedicated research area
    #[default]
    Facility,
    /// An organization involved in stewarding research and cultural heritage materials. Includes libraries, museums, and zoos
    Archive,
    /// A private for-profit corporate entity involved in conducting or sponsoring research
    Company,
    /// A university or similar institution involved in providing education and educating/employing researchers
    Education,
    /// An organization that awards research funds or provides in-kind support
    Funder,
    /// A governmental organization
    Government,
    /// A medical care facility such as hospital or medical clinic
    Healthcare,
    /// A non-profit and non-governmental organization involved in conducting or funding research
    Nonprofit,
    /// An organization that does not fit into other categories
    Other,
}
/// Type of relationship between an organization and another organization
///
/// See `https://ror.readme.io/docs/ror-data-structure#relationships` for more information
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RelationshipType {
    /// Indicate a relationship where the parent exercises control (supervisory, administrative, or financial) over the child
    Child,
    /// The child is a component of the parent entity, like a research center within a university
    Parent,
    /// Denotes less defined connections, such as resource sharing or participation without direct control
    Related,
    /// Indicates that an organization continues the work of a predecessor organization that has ceased operations
    Successor,
    /// Track organizational continuity and are used when an entity ceases operations or to redirect from erroneous records to correct ones
    Predecessor,
}
/// ROR schema version
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
pub enum SchemaVersion {
    /// Permanently sunset in December 2025 and are no longer supported
    #[serde(rename = "1.0")]
    V1_0,
    /// Recommended schema (major) version for use in production applications
    #[serde(rename = "2.0")]
    V2_0,
    /// Current recommended, stable schema version — used by ROR API
    #[serde(rename = "2.1")]
    V2_1,
}
/// ROR item status
///
/// See `https://ror.readme.io/docs/ror-data-structure#status` for more information
#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// An organization that is actively producing research outputs
    #[default]
    Active,
    /// An organization that has ceased operation or producing research outputs
    Inactive,
    /// A record that was created in error, such as a duplicate record or a record that is not in scope for the registry
    Withdrawn,
}
/// Container for administrative information
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct AdminField {
    /// Date the ROR record was created
    pub created: DateField,
    /// Date the ROR record was last modified
    pub last_modified: DateField,
}
/// Container for date and schema version information
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct DateField {
    /// Date in ISO 8601 format (YYYY-MM-DD)
    pub date: String,
    /// Schema version of the date field
    pub schema_version: SchemaVersion,
}
/// Container for information about identifiers in other systems ("external identifiers") that are associated with a given organization in ROR
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct ExternalIdentifierField {
    /// All external identifiers of the type specified in `external_ids.type`
    pub all: Vec<String>,
    /// Preferred external identifier for the organization of the specified type
    pub preferred: Option<String>,
    /// Identifier system that the identifiers in `external_ids.all` and `external_ids.preferred` belong to
    #[serde(rename = "type")]
    pub external_identifier_type: ExternalIdentifierType,
}
/// Link data for a given organization in ROR
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct LinkField {
    /// URL for the link
    pub value: String,
    /// Type of the link
    #[serde(rename = "type")]
    pub link_type: LinkType,
}
/// Location details field
/// ### Example
/// ```json
/// {
///     "continent_code": "NA",
///     "continent_name": "North America",
///     "country_code": "US",
///     "country_name": "United States",
///     "country_subdivision_code": "TN",
///     "country_subdivision_name": "Tennessee",
///     "lat": 36.01036,
///     "lng": -84.26964,
///     "name": "Oak Ridge"
/// }
/// ```
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct LocationDetailsField {
    /// 2-character code for the continent that the organization is located in
    pub continent_code: String,
    /// Name of the continent that the organization is located in
    pub continent_name: String,
    /// Valid 2-character ISO 3166-2 country code (uppercase)
    pub country_code: String,
    /// Name of the country that the organization is located in
    pub country_name: String,
    /// Country subdivision code (derived from ISO-3166-2)
    pub country_subdivision_code: String,
    /// Country subdivision name (derived from ISO-3166-2)
    pub country_subdivision_name: String,
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lng: f64,
    /// Name of the location (e.g., city or town)
    pub name: String,
}
/// Container for location information
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct LocationField {
    /// GeoNames identifier for the city or most granular administrative region that the organization is located in
    /// ### Note
    /// > For most records, this ID represents a city, but for organizations not located in a city, the value in this field is ID of the most granular administrative region for the location available in [GeoNames](https://www.geonames.org/)
    ///
    /// See [`crate::io::api::geonames`] for additional GeoNames API functionality.
    #[serde(rename = "geonames_id")]
    pub identifier: u64,
    /// Container for details derived from the GeoNames record for the GeoNames ID specified in `geonames_id`
    #[serde(rename = "geonames_details")]
    pub details: LocationDetailsField,
}
/// Individual metadata item containing aggregation information
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct MetadataItem {
    /// Identifier for the metadata item
    #[serde(rename = "id")]
    pub identifier: String,
    /// Display title for the metadata item
    pub title: String,
    /// Count of organizations matching this metadata value in the search results
    pub count: usize,
}
/// Container for name information
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct NameField {
    /// The name of the organization
    pub value: String,
    /// The language of the name, in ISO 639-3 format (three-letter code)
    pub lang: Option<String>,
    /// Type(s) associated with the name value
    /// ### Notes
    /// - Each name must have at least 1 type
    /// - Exactly one name must have `ror_display` in its types
    pub types: Vec<OrganizationNameType>,
}
/// Container for relationship information
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct RelationshipField {
    /// Name of another organization identified in `relationships.id`, which is related to the organization
    pub label: String,
    /// Type of relationship between the organization and another organization identified in `relationships.id`
    #[serde(rename = "type")]
    pub relationship_type: RelationshipType,
    /// ROR ID of the related organization
    #[serde(rename = "id")]
    pub identifier: String,
}
/// Metadata for a search query to the ROR API
/// ### Example
/// ```json
/// {
///     "types": [
///         {
///             "id": "facility",
///             "title": "facility",
///             "count": 1
///         },
///         {
///             "id": "government",
///             "title": "government",
///             "count": 1
///         }
///     ],
///     "countries": [
///         {
///             "id": "us",
///             "title": "United States",
///             "count": 1
///         }
///     ],
///     "continents": [
///         {
///             "id": "na",
///             "title": "North America",
///             "count": 1
///         }
///     ],
///     "statuses": [
///         {
///             "id": "inactive",
///             "title": "inactive",
///             "count": 1
///         }
///     ]
/// }
/// ```
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct SearchMetadata {
    /// Listing and count of organization types contained within search results
    pub types: Vec<MetadataItem>,
    /// Listing and count of countries contained within search results
    pub countries: Vec<MetadataItem>,
    /// Listing and count of continents contained within search results
    pub continents: Vec<MetadataItem>,
    /// Listing and count of organization statuses contained within search results
    pub statuses: Vec<MetadataItem>,
}
/// ROR API search response
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct SearchResponse {
    /// Number of results found
    pub number_of_results: usize,
    /// Time taken to perform the search
    pub time_taken: usize,
    /// List of search result items
    #[builder(default)]
    pub items: Vec<SingleRecord>,
    /// Metadata about the search results
    pub meta: SearchMetadata,
}
/// ROR API search result item
///
/// See `https://ror.readme.io/docs/ror-data-structure` for more details on the ROR API response fields.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct SingleRecord {
    /// Unique ROR ID for the organization
    #[serde(rename = "id")]
    pub identifier: String,
    /// Fully-qualified domains that belong to the organization, using the smallest number of labels needed encompass the organization (excluding www)
    ///
    /// <div class="warning">Values cannot be subdomains of other domains listed in the same ROR record</div>
    pub domains: Vec<String>,
    /// Year the organization was established (CE)
    pub established: Option<usize>,
    /// External identifiers associated with the organization, such as Wikidata or ISNI
    pub external_ids: Vec<ExternalIdentifierField>,
    /// Links associated with the organization, such as the organization's website or Wikipedia page
    pub links: Vec<LinkField>,
    /// Associated locations for the organization
    ///
    /// Location data comes from [GeoNames](https://www.geonames.org/) (see also [`crate::io::api::geonames`])
    pub locations: Vec<LocationField>,
    /// Names associated with the organization
    pub names: Vec<NameField>,
    /// Relationships with other organizations
    pub relationships: Vec<RelationshipField>,
    /// Status of associated ROR organization
    pub status: Status,
    /// Types of associated ROR organization
    pub types: Vec<OrganizationType>,
}
/// Check if API is healthy
/// ### Example
/// ```ignore
/// use acorn_lib::io::api;
///
/// println!("ROR API is healthy: {}", api::ror::is_healthy().await);
/// ```
pub async fn is_healthy() -> bool {
    match status().await {
        | Ok(StatusResponse { content, .. }) => content.eq_ignore_ascii_case("OK"),
        | Err(_) => false,
    }
}
/// Get a single ROR record by identifier from options
/// ### Note
/// Will only make a request if the identifier is valid ROR
///
/// ### Example
/// ```ignore
/// use acorn_lib::io::api::ror::{self, Options, SingleRecord};
///
/// let options = Options::from_env().with_identifier("01qz5mb56");
/// let result: ApiResult<SingleRecord> = ror::record(&options).await;
/// ```
pub async fn record(options: &Options) -> ApiResult<SingleRecord> {
    let value = options.identifier().unwrap_or_default().to_string();
    if value.is_empty() || ROR::is_valid(&value) {
        let name = "ROR";
        let action = "search";
        let params = Params::new()
            .with_template("identifier", Some(&value))
            .with_custom(options.params())
            .build();
        let data = Some(params);
        match INCLUDED_ENDPOINTS.find_by_name(name) {
            | Some(endpoint) => {
                let response = endpoint.invoke(action, data).await;
                endpoint.handle::<SingleRecord>(response)
            }
            | None => Err(eyre!("{name} API endpoint not found")),
        }
    } else {
        Err(eyre!("Invalid ROR identifier: {}", &value))
    }
}
/// Search the ROR API with given options containing query parameters and output fields
///
/// ### Example
/// ```ignore
/// use acorn::param;
/// use acorn::io::api::ror::{self, Options, SearchResponse};
///
/// let options = Options::from_env()
///     .with_params(vec![
///         param!(FieldList, "query", "Oak Ridge"),
///         param!(QueryPair, "filter", ("status", "inactive")),
///     ]);
/// let result: ApiResult<SearchResponse> = ror::search(&options).await;
/// ```
pub async fn search(options: &Options) -> ApiResult<SearchResponse> {
    let name = "ROR";
    let action = "search";
    let params = Params::new().with_custom(options.params()).build();
    let data = Some(params);
    match INCLUDED_ENDPOINTS.find_by_name(name) {
        | Some(endpoint) => {
            let response = endpoint.invoke(action, data).await;
            endpoint.handle::<SearchResponse>(response)
        }
        | None => Err(eyre!("{name} API endpoint not found")),
    }
}
/// Get status of ROR API
pub async fn status() -> ApiResult<StatusResponse> {
    let name = "ROR";
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
