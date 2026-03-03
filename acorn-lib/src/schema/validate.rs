//! # Schema validation helpers
//!
//! Generic validation functions and custom [validator](https://docs.rs/validator/latest/validator/) functions for validating schema data
//!
use crate::schema::pid::{Patent, PersistentIdentifierConvert};
use crate::util::constants::{RE_FAKE_PHONE, RE_IMAGE_EXTENSION, RE_IP6, RE_ISO_8601_DATE, RE_ISO_8601_YEAR, RE_PHONE, RE_RAID, RE_UNIX_EPOCH};
use crate::{method_validator, regex_validator};
use chrono::{DateTime, Datelike, Utc};
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
    "Please provide a path with a PNG, JPEG, GIF, WEBP, TIFF or SVG extension"
);
regex_validator!(
    is_iso8601_date,
    RE_ISO_8601_DATE,
    "ISO 8601 Date",
    "Please provide a valid ISO 8601 date (e.g., YYYY-MM-DD)"
);
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
fn is_current_year(value: String) -> bool {
    let now: DateTime<Utc> = Utc::now();
    let year = now.year().to_string().parse::<i32>().unwrap_or_default();
    value.parse::<i32>().unwrap_or_default() <= year
}
/// Check if value is a valid ISO 8601 year (e.g., YYYY)
/// ### Examples
/// - `2025`
pub fn is_iso8601_year(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "year";
    const MESSAGE: &str = "Please provide a valid ISO 8601 year (e.g., YYYY)";
    match RE_ISO_8601_YEAR.is_match(value) {
        | Ok(x) if x && is_current_year(value.to_string()) => Ok(()),
        | _ => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid kebab-case (e.g. 'this-is-kebab-case')
pub fn is_kebabcase(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "kebabcase";
    let valid = value.to_case(Case::Kebab);
    let message = format!("Please provide an ID in kebab-case format (e.g., {valid})");
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
/// Check if value is a valid [`crate::schema::pid::Patent`]
pub fn is_patent_identifier(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "patent";
    const MESSAGE: &str = "Please provide a valid patent identifier";
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
        | (_, true) => Err(ValidationError::new(CODE).with_message("Please provide a real phone number, not a placeholder".into())),
        | _ => Err(ValidationError::new(CODE).with_message("Please provide a valid phone number".into())),
    }
}
/// Check if value is a valid Unix epoch timestamp
pub fn is_unix_epoch(value: usize) -> Result<(), ValidationError> {
    const CODE: &str = "epoch";
    const MESSAGE: &str = "Please provide a valid Unix epoch timestamp";
    match RE_UNIX_EPOCH.is_match(&value.to_string()) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new(CODE).with_message(MESSAGE.into())),
    }
}
