//! Module for interacting with ROR API
//!
//! Provides types and functions for constructing ROR API queries with field validation and output column selection.
// use crate::io::api::{self, ValueValidator};
use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// ROR API search response
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
pub struct SearchResponse {
    /// Number of results found
    pub number_of_results: usize,
    /// Time taken to perform the search
    pub time_taken: usize,
    /// List of search result items
    #[builder(default)]
    pub items: Vec<SearchResult>,
}
/// ROR API search result item
/// ### Example response (OLCF)
/// ```json
/// {
///   "admin": {
///     "created": {
///       "date": "2023-12-20",
///       "schema_version": "1.0"
///     },
///     "last_modified": {
///       "date": "2024-12-11",
///       "schema_version": "2.1"
///     }
///   },
///   "domains": [],
///   "established": 2005,
///   "external_ids": [
///     {
///       "all": [
///         "Q7073703"
///       ],
///       "preferred": "Q7073703",
///       "type": "wikidata"
///     }
///   ],
///   "id": "https://ror.org/05p915b28",
///   "links": [
///     {
///       "type": "website",
///       "value": "https://www.olcf.ornl.gov"
///     },
///     {
///       "type": "wikipedia",
///       "value": "https://en.wikipedia.org/wiki/Oak_Ridge_Leadership_Computing_Facility"
///     }
///   ],
///   "locations": [
///     {
///       "geonames_details": {
///         "continent_code": "NA",
///         "continent_name": "North America",
///         "country_code": "US",
///         "country_name": "United States",
///         "country_subdivision_code": "TN",
///         "country_subdivision_name": "Tennessee",
///         "lat": 36.01036,
///         "lng": -84.26964,
///         "name": "Oak Ridge"
///       },
///       "geonames_id": 4646571
///     }
///   ],
///   "names": [
///     {
///       "lang": null,
///       "types": [
///         "acronym"
///       ],
///       "value": "OLCF"
///     },
///     {
///       "lang": null,
///       "types": [
///         "acronym"
///       ],
///       "value": "ORNL OLCF"
///     },
///     {
///       "lang": "en",
///       "types": [
///         "ror_display",
///         "label"
///       ],
///       "value": "Oak Ridge Leadership Computing Facility"
///     },
///     {
///       "lang": "en",
///       "types": [
///         "alias"
///       ],
///       "value": "Oak Ridge National Laboratory Oak Ridge Leadership Computing Facility"
///     },
///     {
///       "lang": "en",
///       "types": [
///         "alias"
///       ],
///       "value": "U.S. Department of Energy Office of Science Oak Ridge National Laboratory Oak Ridge Leadership Computing Facility"
///     },
///     {
///       "lang": "en",
///       "types": [
///         "alias"
///       ],
///       "value": "United States Department of Energy Office of Science Oak Ridge National Laboratory Oak Ridge Leadership Computing Facility"
///     }
///   ],
///   "relationships": [
///     {
///       "label": "Oak Ridge National Laboratory",
///       "type": "parent",
///       "id": "https://ror.org/01qz5mb56"
///     }
///   ],
///   "status": "active",
///   "types": [
///     "facility"
///   ]
/// }
/// ```
///
/// See `https://ror.readme.io/docs/ror-data-structure` for more details on the ROR API response fields.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SearchResult {
    /// Status of associated ROR organization
    pub status: Status,
}
/// ROR item status
///
/// See `https://ror.readme.io/docs/ror-data-structure#status` for more information
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// An organization that is actively producing research outputs
    Active,
    /// An organization that has ceased operation or producing research outputs
    Inactive,
    /// A record that was created in error, such as a duplicate record or a record that is not in scope for the registry
    Withdrawn,
}
