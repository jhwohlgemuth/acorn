//! # Schema validation helpers
//!
//! Generic validation functions and custom [validator](https://docs.rs/validator/latest/validator/) functions for validating schema data
//!
#[cfg(feature = "std")]
use crate::io::database::{schema::LicenseRow, Row};
#[cfg(feature = "std")]
use crate::io::License;
#[cfg(feature = "std")]
use crate::prelude::env;
use crate::prelude::*;
use crate::schema::geonames::{CodeFormat, GeonamesParser};
use crate::schema::pid::{Patent, PersistentIdentifierConvert};
use crate::schema::standard::datacite::GeoLocationPoint;
#[cfg(feature = "std")]
use crate::util::constants::env::DATABASE_PATH;
use crate::util::constants::{RE_FAKE_PHONE, RE_IMAGE_EXTENSION, RE_IP6, RE_ISO_8601_DATE, RE_PARTIAL_DATE, RE_PHONE, RE_RAID, RE_UNIX_EPOCH};
use crate::util::Constant;
#[cfg(not(feature = "std"))]
use crate::util::License;
use crate::util::SemanticVersion;
use crate::{list_validator, method_validator, regex_validator};
#[cfg(feature = "std")]
use chrono::SecondsFormat;
#[cfg(feature = "std")]
use chrono::Utc;
use chrono::{DateTime, Datelike, NaiveDate};
use convert_case::{Case, Casing};
use core::iter::once;
use fluent_uri::UriRef;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrors};

method_validator!(is_ark, "ARK");
method_validator!(is_doi, "DOI");
method_validator!(is_isbn, "ISBN");
method_validator!(is_orcid, "ORCiD");
method_validator!(is_ror, "ROR");
regex_validator!(is_ip6, RE_IP6, "IP6");
regex_validator!(is_raid, RE_RAID, "RAiD");
regex_validator!(
    has_image_extension,
    RE_IMAGE_EXTENSION,
    "image",
    "Provide path with a PNG, JPEG, GIF, WEBP, TIFF or SVG extension"
);
regex_validator!(is_date, RE_ISO_8601_DATE, "date", "Provide valid ISO 8601 date (e.g., YYYY-MM-DD)");
regex_validator!(
    is_partial_date,
    RE_PARTIAL_DATE,
    "date",
    "Provide valid date (e.g., YYYY-MM-DD or YYYY-MM)"
);
list_validator!(
    /// Custom validator function for validating list of state abbreviations
    is_states,
    is_state,
    "state",
    "Every state should be valid"
);
list_validator!(
    /// Custom validator function for validating list of URLs
    is_urls,
    is_url,
    "url",
    "Every URL should be valid"
);
/// Integer-or-string value used by selected schema fields.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum IntegerOrString {
    /// Integer representation.
    Number(i64),
    /// String representation.
    Text(String),
}

/// Month value represented as integer or string.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum MonthValue {
    /// Integer representation in range 1-12.
    Number(u8),
    /// String representation such as "1" through "12".
    Text(String),
}

/// Number-or-string value used by selected schema fields.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum NumberOrString {
    /// Numeric representation.
    Number(f64),
    /// String representation.
    Text(String),
}
/// Year value represented as text or integer.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum YearValue {
    /// Numeric year representation.
    Number(i64),
    /// String year representation.
    Text(String),
}

/// Postal code represented as either text or integer.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PostalCode {
    /// Numeric postal code.
    Number(i64),
    /// String postal code.
    Text(String),
}
/// Format a phone number into a standard format
/// ### Note
/// > Output format is `000.000.0000`
///
/// ### Example
/// ```rust
/// use acorn::schema::validate::format_phone_number;
///
/// assert_eq!(format_phone_number("(123) 456-7890"), Ok("123.456.7890".to_string()));
/// ```
pub fn format_phone_number(value: &str) -> Result<String, ValidationError> {
    const CODE: &str = "phone";
    const MESSAGE: &str = "Unable to format telephone number";
    match RE_PHONE.captures(value) {
        | Ok(value) => match value {
            | Some(captures) => {
                let country_code = match captures.name("country") {
                    | Some(value) => Some(value.as_str().trim().to_string()),
                    | None => None,
                };
                let area_code = match captures.name("area") {
                    | Some(value) => Some(value.as_str().replace("(", "").replace(")", "")),
                    | None => None,
                };
                let prefix = match captures.name("prefix") {
                    | Some(value) => Some(value.as_str().to_string()),
                    | None => None,
                };
                let line = match captures.name("line") {
                    | Some(value) => Some(value.as_str().to_string()),
                    | None => None,
                };
                Ok([country_code, area_code, prefix, line]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<String>>()
                    .join("."))
            }
            | None => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
        },
        | _ => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Normalizes RFC3339 timestamps to microsecond precision for display.
///
/// This function parses an RFC3339 timestamp string and reformats it to have
/// microsecond precision. If parsing fails, it returns the original string.
///
/// # Arguments
///
/// * `value` - A string slice containing an RFC3339 timestamp
///
/// # Returns
///
/// A String containing the normalized timestamp with microsecond precision,
/// or the original string if parsing fails.
///
/// # Examples
///
/// ```
/// use acorn::schema::validate::format_timestamp;
///
/// // Valid RFC3339 timestamp with nanoseconds gets normalized to microseconds
/// let input = "2023-01-01T12:00:00.123456789Z";
/// let expected = "2023-01-01T12:00:00.123456Z";
/// assert_eq!(format_timestamp(input), expected);
///
/// // Already in microsecond precision stays the same
/// let input = "2023-01-01T12:00:00.123456Z";
/// assert_eq!(format_timestamp(input), input);
///
/// // Invalid timestamp returns original string
/// let invalid = "invalid-timestamp";
/// assert_eq!(format_timestamp(invalid), invalid);
/// ```
pub fn format_timestamp(value: &str) -> String {
    #[cfg(feature = "std")]
    {
        DateTime::parse_from_rfc3339(value)
            .map(|dt| dt.to_rfc3339_opts(SecondsFormat::Micros, false))
            .unwrap_or_else(|_| value.to_string())
    }
    #[cfg(not(feature = "std"))]
    {
        value.to_string()
    }
}
/// Validate that at least one item in `values` satisfies `method`
///
/// This helper is intended for composing `validator` custom functions.
pub fn has_at_least_one_truthy<T>(values: &[T], method: fn(&T) -> bool, code: &'static str, message: &'static str) -> Result<(), ValidationError> {
    match values.iter().any(method) {
        | true => Ok(()),
        | false => Err(ValidationError::new(code).with_message(message.into())),
    }
}
/// Check if value is a valid commit hash
pub fn is_commit(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "commit";
    const MESSAGE: &str = "Provide valid commit hash (7-64 hexadecimal characters)";
    let valid = (7..=64).contains(&value.len()) && value.chars().all(|character| character.is_ascii_hexdigit());
    match valid {
        | true => Ok(()),
        | false => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid ISO 3166 country code
///
/// ISO 3166 is the International Standard for country codes and codes for their subdivisions
///
/// See <https://www.iso.org/iso-3166-country-codes.html> for more information
pub fn is_country_code(value: impl ToString) -> Result<(), ValidationError> {
    const CODE: &str = "country_code";
    const MESSAGE: &str = "Provide valid ISO 3166-1 alpha-2 country code";
    let x = value.to_string();
    let valid = x.len() == 2 && Constant::country_codes(CodeFormat::Alpha2).contains(&x.to_lowercase());
    match valid {
        | true => Ok(()),
        | false => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid ISO 639 language code (two-letter code with optional region subtag, e.g., "en" or "en-US")
///
/// ISO 639 is the internationally recognized code for the representation of the world's languages and language groups
///
/// See <https://www.iso.org/iso-639-language-code> for more information
pub fn is_iso_639_language_code(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "language";
    const MESSAGE: &str = "Provide valid ISO 639 language code";
    let candidate = value.to_lowercase();
    let valid = Constant::languages(CodeFormat::Alpha2).contains(&candidate) || Constant::languages(CodeFormat::Alpha3).contains(&candidate);
    match valid {
        | true => Ok(()),
        | false => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid ISO 639-1 language code (two-letter code only, e.g., 'en', 'fr', etc.)
///
/// ISO 639 is the internationally recognized code for the representation of the world's languages and language groups
///
/// See <https://www.iso.org/iso-639-language-code> for more information
pub fn is_iso_639_1_language_code(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "language";
    const MESSAGE: &str = "Provide valid ISO 639-1 language code (two-letter code only)";
    let valid = value.len() == 2 && Constant::languages(CodeFormat::Alpha2).contains(&value.to_lowercase());
    match valid {
        | true => Ok(()),
        | false => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid kebab-case (e.g. 'this-is-kebab-case')
pub fn is_kebabcase(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "kebabcase";
    let valid = value.to_case(Case::Kebab);
    let message = format!("Provide ID in kebab-case format (e.g., {valid})");
    match valid.eq(&value) {
        | true => Ok(()),
        | _ => Err(ValidationError::new(CODE).with_message(message.into())),
    }
}
/// Check if value is a valid latitude
pub fn is_latitude(value: impl ToString) -> Result<(), ValidationError> {
    const CODE: &str = "latitude";
    const MESSAGE: &str = "Provide valid latitude (-90 to 90)";
    let parsed = value.to_string().parse::<f64>().ok();
    match parsed.filter(|x| (-90.0..=90.0).contains(x)) {
        | Some(_) => Ok(()),
        | None => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid license
/// ### Note
/// Value is validated against local database
#[cfg(feature = "std")]
pub fn is_license(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "license";
    const MESSAGE: &str = "Provide valid SPDX license identifier";
    let candidate = value.trim().to_string();
    let normalized = candidate.to_ascii_lowercase();
    let validation_error = |message: String| {
        let mut why = ValidationError::new(CODE).with_message(message.into());
        why.add_param("value".into(), &candidate);
        why
    };
    let predicate = |row: &LicenseRow| {
        let LicenseRow { identifier, name, .. } = row;
        let is_valid_identifier = identifier.as_ref().map(|id| id.eq_ignore_ascii_case(&normalized)).unwrap_or(false);
        let is_valid_name = name.as_ref().map(|name| name.eq_ignore_ascii_case(&normalized)).unwrap_or(false);
        is_valid_identifier || is_valid_name
    };
    if normalized.is_empty() {
        Err(validation_error(format!("{MESSAGE} (cannot be empty)")))
    } else {
        match env::var(DATABASE_PATH) {
            | Ok(path) => {
                let exists = LicenseRow::default().select(Some(path.into()), predicate).map(|row| row.is_some());
                match exists {
                    | Ok(true) => Ok(()),
                    | Ok(false) => Err(validation_error(MESSAGE.to_string())),
                    | Err(_) => Err(validation_error("Failed to access local database for license validation".to_string())),
                }
            }
            | Err(_) => Err(validation_error("Failed to access local database for license validation".to_string())),
        }
    }
}
#[cfg(not(feature = "std"))]
/// Check if value is a non-empty SPDX license identifier string in no-std mode.
pub fn is_license(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "license";
    const MESSAGE: &str = "Provide valid SPDX license identifier";
    let value = value.trim();
    if value.is_empty() {
        Err(ValidationError::new(CODE).with_message(format!("{MESSAGE} (cannot be empty)").into()))
    } else {
        Ok(())
    }
}
/// Check if value is a valid longitude
pub fn is_longitude<T>(value: T) -> Result<(), ValidationError>
where
    T: ToString,
{
    const CODE: &str = "longitude";
    const MESSAGE: &str = "Provide valid longitude (-180 to 180)";
    let parsed = value.to_string().parse::<f64>().ok();
    match parsed.filter(|x| (-180.0..=180.0).contains(x)) {
        | Some(_) => Ok(()),
        | None => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid [`crate::schema::pid::Patent`]
pub fn is_patent_identifier(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "patent";
    const MESSAGE: &str = "Provide valid patent identifier";
    match Patent::is_valid(value) {
        | true => Ok(()),
        | _ => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid phone number
///
/// Uses same regex as `format_phone_number`
pub fn is_phone_number(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "phone";
    let is_fake = RE_FAKE_PHONE.is_match(value).unwrap_or(false);
    let is_valid = RE_PHONE.is_match(value).unwrap_or(false);
    match (is_valid, is_fake) {
        | (true, false) => Ok(()),
        | (_, true) => Err(ValidationError::new(CODE).with_message("Provide real phone number, not a placeholder".into())),
        | _ => Err(ValidationError::new(CODE).with_message("Provide valid phone number".into())),
    }
}
/// Check if list of locations depict a valid polygon
pub fn is_polygon(value: &[GeoLocationPoint]) -> Result<(), ValidationError> {
    const CODE: &str = "polygon";
    const EMPTY_MESSAGE: &str = "Provide valid polygon — no points were provided";
    let length = value.len();
    let result = if length == 0 {
        Err(ValidationError::new(CODE).with_message(EMPTY_MESSAGE.into()))
    } else if length < 4 {
        let length_message = format!("Provide valid polygon — at least 4 points are required (only {length} found)");
        Err(ValidationError::new(CODE).with_message(length_message.into()))
    } else {
        match (value.first(), value.last()) {
            | (Some(first), Some(last)) if first == last => Ok(()),
            | (Some(first), Some(last)) => {
                let closed_message = format!("Provide valid polygon — first point must equal last point ({first} ≠ {last})");
                Err(ValidationError::new(CODE).with_message(closed_message.into()))
            }
            | _ => Err(ValidationError::new(CODE).with_message(EMPTY_MESSAGE.into())),
        }
    };
    result
}
/// Check if value is a valid RFC3339 timestamp (ISO 8601 compliant)
/// ### Examples
/// - `2026-03-12T15:04:05Z`
/// - `2026-03-12T15:04:05+00:00`
pub fn is_rfc3339(value: impl ToString) -> Result<(), ValidationError> {
    const CODE: &str = "date";
    const MESSAGE: &str = "Provide valid RFC 3339 timestamp (ISO 8601 compliant)";
    let x = value.to_string();
    let is_rfc3339 = DateTime::parse_from_rfc3339(&x).is_ok();
    let is_date = NaiveDate::parse_from_str(&x, "%Y-%m-%d").is_ok();
    let is_supported_date_value = |value: &str| DateTime::parse_from_rfc3339(value).is_ok() || NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok();
    let is_date_interval = x
        .split_once('/')
        .map(|(start, end)| is_supported_date_value(start) && is_supported_date_value(end))
        .unwrap_or(false);
    match is_rfc3339 || is_date || is_date_interval {
        | true => Ok(()),
        | false => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid semantic version (e.g., 1.2.3)
///
/// See [`crate::util::SemanticVersion`]
pub fn is_semantic_version(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "version";
    const MESSAGE: &str = "Provide valid semantic version (e.g., 1.2.3)";
    let parsed = SemanticVersion::from(value);
    match parsed.to_string().starts_with(value) {
        | true => Ok(()),
        | false => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid state abbreviation (or Canada)
pub fn is_state(value: impl ToString) -> Result<(), ValidationError> {
    const CODE: &str = "state";
    const MESSAGE: &str = "Provide valid US state or Canadian province abbreviation (e.g., CA, NY, ON)";
    let x = value.to_string();
    let valid = x.len() == 2 && x.chars().all(|c| c.is_ascii_alphabetic());
    match valid {
        | true => Ok(()),
        | false => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid Unix epoch timestamp
pub fn is_unix_epoch(value: usize) -> Result<(), ValidationError> {
    const CODE: &str = "epoch";
    const MESSAGE: &str = "Provide valid Unix epoch timestamp";
    match RE_UNIX_EPOCH.is_match(&value.to_string()) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid URL
pub fn is_url(value: impl ToString) -> Result<(), ValidationError> {
    const CODE: &str = "url";
    const MESSAGE: &str = "Provide valid URL";
    let x = value.to_string();
    match UriRef::parse(x.as_str()) {
        | Ok(_) => Ok(()),
        | Err(_) => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid ISO 8601 year (e.g., YYYY)
/// ### Examples
/// - `2025`
pub fn is_year(value: impl ToString) -> Result<(), ValidationError> {
    const CODE: &str = "year";
    const MESSAGE: &str = "Provide valid ISO 8601 year (e.g., YYYY)";
    let x = value.to_string();
    let parsed_year = (x.len() == 4)
        .then(|| format!("{x}-01-01"))
        .and_then(|date| NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok())
        .map(|date| date.year());
    match parsed_year {
        | Some(year) if is_viable_year(year) => Ok(()),
        | _ => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
#[cfg(feature = "std")]
fn is_viable_year(year: i32) -> bool {
    let now: DateTime<Utc> = Utc::now();
    year <= now.year()
}
#[cfg(not(feature = "std"))]
fn is_viable_year(_year: i32) -> bool {
    true
}
impl Validate for IntegerOrString {
    fn validate(&self) -> Result<(), ValidationErrors> {
        Ok(())
    }
}
impl Validate for License {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let to_validation_errors = |why: ValidationError| {
            once(why).fold(ValidationErrors::new(), |mut acc, err| {
                acc.add("license", err);
                acc
            })
        };
        #[cfg(feature = "std")]
        {
            let invalid = match self {
                | License::Single(value) => is_license(value).err(),
                | License::Multiple(values) => values.iter().find_map(|value| is_license(value).err()),
            };
            invalid.map_or(Ok(()), |why| Err(to_validation_errors(why)))
        }
        #[cfg(not(feature = "std"))]
        {
            is_license(&self.to_string()).map_err(to_validation_errors)
        }
    }
}
impl Validate for MonthValue {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let to_validation_errors = |why: ValidationError| {
            once(why).fold(ValidationErrors::new(), |mut acc, err| {
                acc.add("month", err);
                acc
            })
        };
        let valid = match self {
            | Self::Number(value) => (1..=12).contains(value),
            | Self::Text(value) => matches!(value.as_str(), "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" | "11" | "12"),
        };
        match valid {
            | true => Ok(()),
            | false => Err(to_validation_errors(
                ValidationError::new("range").with_message("Month should be an integer 1-12 or a string value from \"1\" to \"12\"".into()),
            )),
        }
    }
}
impl Validate for NumberOrString {
    fn validate(&self) -> Result<(), ValidationErrors> {
        Ok(())
    }
}
impl Validate for YearValue {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let to_validation_errors = |why: ValidationError| {
            once(why).fold(ValidationErrors::new(), |mut acc, err| {
                acc.add("year", err);
                acc
            })
        };
        match self {
            | Self::Number(value) => is_year(*value).map_err(to_validation_errors),
            | Self::Text(value) => is_year(value).map_err(to_validation_errors),
        }
    }
}

impl Validate for PostalCode {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let to_validation_errors = |why: ValidationError| {
            once(why).fold(ValidationErrors::new(), |mut acc, err| {
                acc.add("postal_code", err);
                acc
            })
        };
        let valid = match self {
            | Self::Number(value) => *value >= 0,
            | Self::Text(value) => {
                let trimmed = value.trim();
                !trimmed.is_empty()
                    && trimmed.len() <= 20
                    && trimmed
                        .chars()
                        .all(|character| character.is_alphanumeric() || character == ' ' || character == '-')
            }
        };
        match valid {
            | true => Ok(()),
            | false => Err(to_validation_errors(ValidationError::new("format").with_message(
                "Postal code should be a non-negative integer or a non-empty string up to 20 characters using letters, digits, spaces, or hyphens"
                    .into(),
            ))),
        }
    }
}
