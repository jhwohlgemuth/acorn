//! Module for working with GeoNames data
use crate::prelude::*;
use crate::util::{Constant, Searchable};
use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Trait for parsing GeoNames country data
pub trait GeonamesParser {
    /// Isolate country codes from GeoNames country data for use in other contexts
    ///
    /// Returns a sorted list of unique country code tags from ISO-3166-1
    fn country_codes(format: CodeFormat) -> Vec<String>;
    /// Parse GeoNames country TSV text response, removing comments and returning structured rows.
    ///
    /// Lines beginning with `#` are treated as comments and skipped. The first non-comment
    /// line that starts with the `ISO` header row is treated as the header and skipped.
    /// Remaining lines are split on tab characters and converted into [`Country`] values.
    /// Lines that do not contain at least 19 tab-separated columns are ignored.
    fn country_data() -> Countries;
    /// Isolate languages data from GeoNames country data for use in other contexts
    ///
    /// Returns a sorted list of unique language tags from ISO-639-(1|2|3)
    fn languages(format: CodeFormat) -> Vec<String>;
}
/// Wrapper type for a list of GeoNames countries
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Countries(pub Vec<Country>);
/// Selects code format to extract (either alpha-2 or alpha-3).
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum CodeFormat {
    /// Two-letter code (e.g., `US` for countries, `en` for languages).
    Alpha2,
    /// Three-letter or longer code (e.g., `USA` for countries, `eng` or `en-US` for languages).
    Alpha3,
}
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
impl Countries {
    /// Return the number of countries in the collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Return true when the collection contains no countries.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl Searchable<Country> for Countries {
    fn contains(&self, value: &str) -> bool {
        let trimmed = value.trim();
        !trimmed.is_empty()
            && self.0.iter().any(|country| {
                let Country { iso, iso3, .. } = country;
                iso.eq_ignore_ascii_case(trimmed) || iso3.eq_ignore_ascii_case(trimmed)
            })
    }
    fn find_by_iso(&self, value: impl Into<String>) -> Option<Country> {
        let trimmed = value.into().trim().to_string();
        self.0
            .iter()
            .find(|country| {
                let Country { iso, iso3, .. } = country;
                iso.eq_ignore_ascii_case(&trimmed) || iso3.eq_ignore_ascii_case(&trimmed)
            })
            .cloned()
    }
    fn find_by_name(&self, value: impl Into<String>) -> Option<Country> {
        let trimmed = value.into().trim().to_string();
        self.0
            .iter()
            .find(|country| {
                let Country { name, .. } = country;
                name.eq_ignore_ascii_case(&trimmed)
            })
            .cloned()
    }
}
impl From<Vec<Country>> for Countries {
    fn from(value: Vec<Country>) -> Self {
        Self(value)
    }
}
impl From<&str> for Countries {
    fn from(value: &str) -> Self {
        parse(value)
    }
}
impl From<String> for Countries {
    fn from(value: String) -> Self {
        parse(&value)
    }
}
impl GeonamesParser for Constant {
    fn country_codes(format: CodeFormat) -> Vec<String> {
        let data = Self::country_data();
        country_codes(data, format)
    }
    fn country_data() -> Countries {
        let text = Self::from_asset("geonames.tsv").unwrap_or_default();
        parse(&text)
    }
    fn languages(format: CodeFormat) -> Vec<String> {
        let data = Self::country_data();
        languages(data, format)
    }
}
fn country_codes(data: Countries, format: CodeFormat) -> Vec<String> {
    let mut codes = data
        .0
        .into_iter()
        .map(|country: Country| {
            let code = match format {
                | CodeFormat::Alpha2 => country.iso,
                | CodeFormat::Alpha3 => country.iso3,
            };
            code.trim().to_lowercase()
        })
        .filter(|value| !value.is_empty())
        .collect::<Vec<String>>();
    codes.sort();
    codes.dedup();
    codes
}
fn languages(data: Countries, format: CodeFormat) -> Vec<String> {
    let mut languages = data
        .0
        .into_iter()
        .flat_map(|country: Country| {
            country
                .languages
                .into_iter()
                .map(|value| value.trim().to_lowercase())
                .filter(|value| !value.is_empty())
                .collect::<Vec<String>>()
        })
        .filter(|value| match format {
            | CodeFormat::Alpha2 => value.len() == 2,
            | CodeFormat::Alpha3 => value.contains('-'),
        })
        .collect::<Vec<String>>();
    languages.sort();
    languages.dedup();
    languages
}
fn parse(data: &str) -> Countries {
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
    data.lines()
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
                area: columns.get(6).and_then(|s| parse_u64_field(s)),
                population: columns.get(7).and_then(|s| parse_u64_field(s)),
                continent: string_field(&columns, 8),
                top_level_domain: string_field(&columns, 9),
                currency_code: string_field(&columns, 10),
                currency_name: string_field(&columns, 11),
                country_code: string_field(&columns, 12),
                postal_code_format: string_field(&columns, 13),
                postal_code_regex: string_field(&columns, 14),
                languages: columns.get(15).map(|s| list_field(s)).unwrap_or_default(),
                identifier: columns.get(16).and_then(|s| parse_u64_field(s)),
                neighbours: columns.get(17).map(|s| list_field(s)).unwrap_or_default(),
                equivalent_fips_code: columns.get(18).and_then(|s| parse_optional_string_field(s)),
            }
        })
        .collect::<Vec<Country>>()
        .into()
}
