//! # Schema validation helpers
//!
//! Generic validation functions and custom [validator](https://docs.rs/validator/latest/validator/) functions for validating schema data
//!
use crate::schema::pid::{Patent, PersistentIdentifierConvert};
use crate::schema::standard::datacite::GeoLocationPoint;
use crate::util::constants::{RE_FAKE_PHONE, RE_IMAGE_EXTENSION, RE_IP6, RE_ISO_8601_DATE, RE_PHONE, RE_RAID, RE_UNIX_EPOCH};
use crate::util::SemanticVersion;
use crate::{method_validator, regex_validator};
use chrono::{DateTime, Datelike, NaiveDate, SecondsFormat, Utc};
use convert_case::{Case, Casing};
use core::iter::once;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uriparse::URI;
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
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.to_rfc3339_opts(SecondsFormat::Micros, false))
        .unwrap_or_else(|_| value.to_string())
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
/// Custom validator function for validating list of URLs
pub fn is_list_url(value: &[String]) -> Result<(), ValidationError> {
    const CODE: &str = "url";
    value
        .iter()
        .position(|x| URI::try_from(x.as_str()).is_err())
        .map(|index| {
            let mut err = ValidationError::new(CODE).with_message("Every URL should be valid".to_string().into());
            err.add_param("index".into(), &index);
            err
        })
        .map_or(Ok(()), Err)
}
/// Check if value is a valid latitude
pub fn is_latitude<T>(value: T) -> Result<(), ValidationError>
where
    T: ToString,
{
    const CODE: &str = "latitude";
    const MESSAGE: &str = "Provide valid latitude (-90 to 90)";
    let parsed = value.to_string().parse::<f64>().ok();
    match parsed.filter(|x| (-90.0..=90.0).contains(x)) {
        | Some(_) => Ok(()),
        | None => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
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
    let length_message = format!("Provide valid polygon — at least 4 points are required (only {} found)", value.len());
    let first = value.first().unwrap();
    let last = value.last().unwrap();
    let closed_message = format!("Provide valid polygon — first point must equal last point ({first} ≠ {last})");
    let length = value.len();
    let is_closed = value.first().zip(value.last()).map(|(first, last)| first == last).unwrap_or(false);
    match (length, is_closed) {
        | (0, _) => Err(ValidationError::new(CODE).with_message(EMPTY_MESSAGE.into())),
        | (1..=3, _) => Err(ValidationError::new(CODE).with_message(length_message.into())),
        | (_, false) => Err(ValidationError::new(CODE).with_message(closed_message.into())),
        | _ => Ok(()),
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
/// Check if value is a valid Unix epoch timestamp
pub fn is_unix_epoch(value: usize) -> Result<(), ValidationError> {
    const CODE: &str = "epoch";
    const MESSAGE: &str = "Provide valid Unix epoch timestamp";
    match RE_UNIX_EPOCH.is_match(&value.to_string()) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
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
fn is_viable_year(year: i32) -> bool {
    let now: DateTime<Utc> = Utc::now();
    year <= now.year()
}

impl Validate for IntegerOrString {
    fn validate(&self) -> Result<(), ValidationErrors> {
        Ok(())
    }
}

impl Validate for NumberOrString {
    fn validate(&self) -> Result<(), ValidationErrors> {
        Ok(())
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
