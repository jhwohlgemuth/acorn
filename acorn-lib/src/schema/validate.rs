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
use uriparse::URI;
use validator::ValidationError;

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
pub fn is_latitude(value: f64) -> Result<(), ValidationError> {
    const CODE: &str = "latitude";
    const MESSAGE: &str = "Provide valid latitude (-90 to 90)";
    match (-90.0..=90.0).contains(&value) {
        | true => Ok(()),
        | false => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid longitude
pub fn is_longitude(value: f64) -> Result<(), ValidationError> {
    const CODE: &str = "longitude";
    const MESSAGE: &str = "Provide valid longitude (-180 to 180)";
    match (-180.0..=180.0).contains(&value) {
        | true => Ok(()),
        | false => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
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
pub fn is_polygon(value: &Vec<GeoLocationPoint>) -> Result<(), ValidationError> {
    const CODE: &str = "polygon";
    const MESSAGE: &str = "Provide valid polygon (at least 4 points; first point must equal last point)";
    let is_closed = value.first().zip(value.last()).map(|(first, last)| first == last).unwrap_or(false);
    match value.len() >= 4 && is_closed {
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
