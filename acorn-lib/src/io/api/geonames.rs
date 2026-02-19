//! Module for downloading, parsing, and using GeoNames data
//!
use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Represents a single GeoNames country row parsed from a TSV file.
///
/// The fields map directly to the columns in the GeoNames `countryInfo.txt`/TSV export.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
pub struct Row {
    /// Two-letter ISO 3166-1 alpha-2 country code (e.g., "US")
    pub iso: String,
    /// Three-letter ISO 3166-1 alpha-3 country code (e.g., "USA")
    pub iso3: String,
    /// Numeric ISO 3166-1 code as a string (e.g., "840")
    pub iso_numeric: String,
    /// FIPS 10-4 country code
    pub fips: String,
    /// Country name
    pub country: String,
    /// Capital city name
    pub capital: String,
    /// Country area in square kilometers
    pub area_sq_km: Option<u64>,
    /// Country population
    pub population: Option<u64>,
    /// Continent code (e.g., "EU", "NA")
    pub continent: String,
    /// Top-level domain (TLD) for the country (e.g., ".us")
    pub tld: String,
    /// Currency code (e.g., "USD")
    pub currency_code: String,
    /// Currency name (e.g., "Dollar")
    pub currency_name: String,
    /// Country calling code or pattern
    pub phone: String,
    /// Postal code format string
    pub postal_code_format: String,
    /// Postal code regular expression
    pub postal_code_regex: String,
    /// List of language tags ordered by number of speakers
    pub languages: Vec<String>,
    /// GeoNames identifier for the country
    pub geoname_id: Option<u64>,
    /// List of neighboring country ISO codes
    pub neighbours: Vec<String>,
    /// Equivalent FIPS code, if any
    pub equivalent_fips_code: Option<String>,
}
/// Parse GeoNames country TSV content, removing comments and returning structured rows.
///
/// Lines beginning with `#` are treated as comments and skipped. The first non-comment
/// line that starts with the `ISO` header row is treated as the header and skipped.
/// Remaining lines are split on tab characters and converted into [`Row`] values.
/// Lines that do not contain at least 19 tab-separated columns are ignored.
pub fn parse_tsv(content: &str) -> Vec<Row> {
    fn parse_u64_field(value: &str) -> Option<u64> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            trimmed.parse::<u64>().ok()
        }
    }
    content
        .lines()
        .filter(|line| {
            let line = line.trim();
            !line.is_empty() && !line.starts_with('#')
        })
        .filter(|line| !line.starts_with("ISO\t"))
        .filter_map(|line| {
            let columns: Vec<&str> = line.split('\t').collect();
            // Expect the 19 standard GeoNames country columns; skip malformed lines.
            if columns.len() < 19 {
                return None;
            }
            let iso = columns[0].trim().to_string();
            let iso3 = columns[1].trim().to_string();
            let iso_numeric = columns[2].trim().to_string();
            let fips = columns[3].trim().to_string();
            let country = columns[4].trim().to_string();
            let capital = columns[5].trim().to_string();
            let area_sq_km = parse_u64_field(columns[6]);
            let population = parse_u64_field(columns[7]);
            let continent = columns[8].trim().to_string();
            let tld = columns[9].trim().to_string();
            let currency_code = columns[10].trim().to_string();
            let currency_name = columns[11].trim().to_string();
            let phone = columns[12].trim().to_string();
            let postal_code_format = columns[13].trim().to_string();
            let postal_code_regex = columns[14].trim().to_string();
            let languages = columns[15]
                .split(',')
                .map(|lang| lang.trim())
                .filter(|lang| !lang.is_empty())
                .map(|lang| lang.to_string())
                .collect::<Vec<String>>();
            let geoname_id = parse_u64_field(columns[16]);
            let neighbours = columns[17]
                .split(',')
                .map(|code| code.trim())
                .filter(|code| !code.is_empty())
                .map(|code| code.to_string())
                .collect::<Vec<String>>();
            let equivalent_fips_code = {
                let value = columns[18].trim();
                if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                }
            };
            Some(Row {
                iso,
                iso3,
                iso_numeric,
                fips,
                country,
                capital,
                area_sq_km,
                population,
                continent,
                tld,
                currency_code,
                currency_name,
                phone,
                postal_code_format,
                postal_code_regex,
                languages,
                geoname_id,
                neighbours,
                equivalent_fips_code,
            })
        })
        .collect()
}
