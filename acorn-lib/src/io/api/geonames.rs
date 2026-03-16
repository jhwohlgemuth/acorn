//! Module for downloading, parsing, and using GeoNames data
//!
//! See [Geonames web services documentation](https://www.geonames.org/export/web-services.html) for more information
use crate::io::api::{ApiResult, RemoteResource, Searchable, TextResponse, INCLUDED_ENDPOINTS};
use bon::Builder;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Type alias for GeoNames API reponse for country data, which is returned as TSV text that can be parsed into structured data
pub type SearchResponse = TextResponse;
/// Trait for parsing GeoNames country data
pub trait CountryParser {
    /// Parse a GeoNames TSV export into a list of GeoNames countries
    fn parse(text: &str) -> Countries;
}
/// Type alias for a list of GeoNames countries
pub type Countries = Vec<Country>;
/// Represents a single GeoNames country row parsed from a TSV file.
///
/// The fields map directly to the columns in the GeoNames `countryInfo.txt`/TSV export.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
pub struct Country {
    /// Two-letter ISO 3166-1 alpha-2 country code (e.g., "US")
    pub iso: String,
    /// Three-letter ISO 3166-1 alpha-3 country code (e.g., "USA")
    pub iso3: String,
    /// Numeric ISO 3166-1 code as a string (e.g., "840")
    pub iso_numeric: String,
    /// FIPS 10-4 country code (deprecated in favor of ISO codes)
    pub fips: String,
    /// Country name
    pub name: String,
    /// Capital city name
    pub capital: String,
    /// Country area in square kilometers
    pub area: Option<u64>,
    /// Country population
    pub population: Option<u64>,
    /// Continent code (e.g., "EU", "NA")
    pub continent: String,
    /// Top-level domain (TLD) for the country (e.g., ".us")
    pub top_level_domain: String,
    /// Currency code (e.g., "USD")
    pub currency_code: String,
    /// Currency name (e.g., "Dollar")
    pub currency_name: String,
    /// Country calling code or pattern
    pub country_code: String,
    /// Postal code format string
    pub postal_code_format: String,
    /// Postal code regular expression
    pub postal_code_regex: String,
    /// List of language tags ordered by number of speakers
    pub languages: Vec<String>,
    /// GeoNames identifier for the country
    pub identifier: Option<u64>,
    /// List of neighboring country ISO codes
    pub neighbours: Vec<String>,
    /// Equivalent FIPS code, if any (deprecated in favor of ISO codes)
    pub equivalent_fips_code: Option<String>,
}
impl Searchable<Country> for Vec<Country> {
    fn contains(&self, value: &str) -> bool {
        let trimmed = value.trim();
        !trimmed.is_empty()
            && self.iter().any(|country| {
                let Country { iso, iso3, .. } = country;
                iso.eq_ignore_ascii_case(trimmed) || iso3.eq_ignore_ascii_case(trimmed)
            })
    }
    fn find_by_iso(&self, value: impl Into<String>) -> Option<Country> {
        let trimmed = value.into().trim().to_string();
        self.iter()
            .find(|country| {
                let Country { iso, iso3, .. } = country;
                iso.eq_ignore_ascii_case(&trimmed) || iso3.eq_ignore_ascii_case(&trimmed)
            })
            .cloned()
    }
    fn find_by_name(&self, value: impl Into<String>) -> Option<Country> {
        let trimmed = value.into().trim().to_string();
        self.iter()
            .find(|country| {
                let Country { name, .. } = country;
                name.eq_ignore_ascii_case(&trimmed)
            })
            .cloned()
    }
}
impl SearchResponse {
    /// Parse GeoNames country TSV text response, removing comments and returning structured rows.
    ///
    /// Lines beginning with `#` are treated as comments and skipped. The first non-comment
    /// line that starts with the `ISO` header row is treated as the header and skipped.
    /// Remaining lines are split on tab characters and converted into [`Country`] values.
    /// Lines that do not contain at least 19 tab-separated columns are ignored.
    pub fn parse(&self) -> Countries {
        fn parse_optional_string_field(value: &str) -> Option<String> {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        fn parse_u64_field(value: &str) -> Option<u64> {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                trimmed.parse::<u64>().ok()
            }
        }
        fn list_field(value: &str) -> Vec<String> {
            value
                .split(',')
                .map(|item| item.trim())
                .filter(|item| !item.is_empty())
                .map(|item| item.to_string())
                .collect()
        }
        fn string_field(columns: &[&str], index: usize) -> String {
            columns.get(index).map(|s| s.trim().to_string()).unwrap_or_default()
        }
        self.content
            .lines()
            .filter(|line| {
                let line = line.trim();
                !line.is_empty() && !line.starts_with('#')
            })
            .filter(|line| !line.starts_with("ISO\t"))
            .map(|line| {
                let columns: Vec<&str> = line.split('\t').collect();
                Country {
                    iso: string_field(&columns, 0),
                    iso3: string_field(&columns, 1),
                    iso_numeric: string_field(&columns, 2),
                    fips: string_field(&columns, 3),
                    name: string_field(&columns, 4),
                    capital: string_field(&columns, 5),
                    area: parse_u64_field(columns[6]),
                    population: parse_u64_field(columns[7]),
                    continent: string_field(&columns, 8),
                    top_level_domain: string_field(&columns, 9),
                    currency_code: string_field(&columns, 10),
                    currency_name: string_field(&columns, 11),
                    country_code: string_field(&columns, 12),
                    postal_code_format: string_field(&columns, 13),
                    postal_code_regex: string_field(&columns, 14),
                    languages: list_field(columns[15]),
                    identifier: parse_u64_field(columns[16]),
                    neighbours: list_field(columns[17]),
                    equivalent_fips_code: parse_optional_string_field(columns[18]),
                }
            })
            .collect()
    }
}
/// Pull GeoNames country data from the API and parse it into structured data
pub async fn countries() -> ApiResult<Countries> {
    let name = "GeoNames";
    match INCLUDED_ENDPOINTS.find_by_name(name) {
        | Some(endpoint) => {
            let response = endpoint.invoke("countries", None).await;
            endpoint.handle::<TextResponse>(response).map(|text| text.parse())
        }
        | None => Err(eyre!("{name} endpoint not found")),
    }
}
/// Isolate languages data from GeoNames country data for use in other contexts
///
/// Returns a sorted list of unique language tags from ISO-639-(1|2|3)
pub async fn languages(data: Option<Vec<Country>>) -> Vec<String> {
    let countries: Vec<Country> = match data {
        | Some(countries) => countries,
        | None => countries().await.unwrap_or_default(),
    };
    let mut languages = countries
        .into_iter()
        .flat_map(|country: Country| {
            country
                .languages
                .into_iter()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect::<Vec<String>>()
        })
        .collect::<Vec<String>>();
    languages.sort();
    languages.dedup();
    languages
}
