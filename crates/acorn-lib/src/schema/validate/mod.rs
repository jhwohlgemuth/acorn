//! # Schema validation helpers
//!
//! Generic validation functions and custom ACORN rules for validating schema data.
//!
use crate::prelude::alloc::{format, String, ToString, Vec};
use crate::regex_validator;
use crate::schema::geonames::{CodeFormat, GeonamesParser};
use crate::schema::standard::datacite::{
    Affiliation, FunderIdentifier, FunderIdentifierType, FundingReference, GeoLocationPoint, NameIdentifier, Publisher,
};
use crate::schema::standard::invenio::{PersonOrOrgIdentifier, PersonOrOrgIdentifierScheme};
use crate::schema::OneOrMany;
use crate::util::constants::{RE_GND, RE_IMAGE_EXTENSION, RE_IP6, RE_PARTIAL_DATE, RE_PHONE};
use crate::util::Constant;
use crate::validation::{rules, Validate, ValidationError, ValidationReport};
use core::iter::once;
use jiff::{fmt::temporal::Pieces, tz::Offset, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn identifier_validation_error(code: &'static str, message: impl Into<String>) -> ValidationError {
    ValidationError::new(code).with_message(message.into().into())
}
fn validate_named_identifier(identifier: &str, scheme: Option<&str>, code: &'static str) -> Result<(), ValidationError> {
    match scheme.map(str::trim).filter(|value| !value.is_empty()) {
        | Some(value) if value.eq_ignore_ascii_case("ISNI") => rules::isni(identifier),
        | Some(value) if value.eq_ignore_ascii_case("ORCID") => rules::orcid(identifier),
        | Some(value) if value.eq_ignore_ascii_case("ROR") => rules::ror(identifier),
        | Some(_) => Ok(()),
        | None => Err(identifier_validation_error(
            code,
            "Provide an identifier scheme when an identifier is present",
        )),
    }
}
fn validate_optional_named_identifier(identifier: Option<&String>, scheme: Option<&String>, code: &'static str) -> Result<(), ValidationError> {
    match identifier {
        | Some(value) => validate_named_identifier(value, scheme.map(String::as_str), code),
        | None => Ok(()),
    }
}
fn validate_typed_funder_identifier(identifier: &str, kind: Option<&FunderIdentifierType>) -> Result<(), ValidationError> {
    match kind {
        | Some(FunderIdentifierType::Isni) => rules::isni(identifier),
        | Some(FunderIdentifierType::Ror) => rules::ror(identifier),
        | Some(_) => Ok(()),
        | None => Err(identifier_validation_error(
            "funder_identifier",
            "Provide a funder identifier type when a funder identifier is present",
        )),
    }
}
pub(in crate::schema) fn validate_affiliation_identifier(value: &Affiliation, _context: &()) -> Result<(), ValidationReport> {
    validate_optional_named_identifier(
        value.affiliation_identifier.as_ref(),
        value.affiliation_identifier_scheme.as_ref(),
        "affiliation_identifier",
    )
    .map_err(|error| error.into_report(""))
}
pub(in crate::schema) fn validate_name_identifier(value: &NameIdentifier, _context: &()) -> Result<(), ValidationReport> {
    validate_named_identifier(&value.name_identifier, value.name_identifier_scheme.as_deref(), "name_identifier")
        .map_err(|error| error.into_report(""))
}
pub(in crate::schema) fn validate_publisher_identifier(value: &Publisher, _context: &()) -> Result<(), ValidationReport> {
    validate_optional_named_identifier(
        value.publisher_identifier.as_ref(),
        value.publisher_identifier_scheme.as_ref(),
        "publisher_identifier",
    )
    .map_err(|error| error.into_report(""))
}
pub(in crate::schema) fn validate_funder_identifier(value: &FunderIdentifier, _context: &()) -> Result<(), ValidationReport> {
    validate_typed_funder_identifier(&value.value, value.funder_identifier_type.as_ref()).map_err(|error| error.into_report(""))
}
pub(in crate::schema) fn validate_funding_reference_identifier(value: &FundingReference, _context: &()) -> Result<(), ValidationReport> {
    match value.funder_identifier.as_ref() {
        | Some(identifier) => {
            validate_typed_funder_identifier(identifier, value.funder_identifier_type.as_ref()).map_err(|error| error.into_report(""))
        }
        | None => Ok(()),
    }
}
pub(in crate::schema) fn validate_person_or_org_identifier(value: &PersonOrOrgIdentifier, _context: &()) -> Result<(), ValidationReport> {
    let result = match value.scheme {
        | PersonOrOrgIdentifierScheme::Gnd => is_gnd(&value.identifier),
        | PersonOrOrgIdentifierScheme::Isni => rules::isni(&value.identifier),
        | PersonOrOrgIdentifierScheme::Orcid => rules::orcid(&value.identifier),
        | PersonOrOrgIdentifierScheme::Ror => rules::ror(&value.identifier),
    };
    result.map_err(|why| {
        let mut error = ValidationError::new("person_or_org_identifier")
            .with_message(why.message.unwrap_or_else(|| "Identifier does not match its scheme".into()).into());
        error.add_param("scheme".into(), &value.scheme);
        error.add_param("identifier".into(), &value.identifier);
        error.into_report("")
    })
}
regex_validator!(is_gnd, RE_GND, "GND");
regex_validator!(is_ip6, RE_IP6, "IP6");
regex_validator!(
    has_image_extension,
    RE_IMAGE_EXTENSION,
    "image",
    "Provide path with a PNG, JPEG, GIF, WEBP, TIFF or SVG extension"
);
regex_validator!(
    is_partial_date,
    RE_PARTIAL_DATE,
    "date",
    "Provide valid date (e.g., YYYY-MM-DD or YYYY-MM)"
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
impl Validate for IntegerOrString {
    fn validate(&self) -> Result<(), ValidationReport> {
        let valid = match self {
            | Self::Number(_) => true,
            | Self::Text(value) => !value.trim().is_empty(),
        };
        match valid {
            | true => Ok(()),
            | false => {
                let mut errors = ValidationReport::new();
                errors.add(
                    "integer_or_string",
                    ValidationError::new("required").with_message("Integer-or-string text should not be empty".into()),
                );
                Err(errors)
            }
        }
    }
}
pub(in crate::schema) fn validate_license(value: &OneOrMany<String>) -> Result<(), ValidationError> {
    value.iter().find_map(|value| is_license(value).err()).map_or(Ok(()), Err)
}
impl Validate for MonthValue {
    fn validate(&self) -> Result<(), ValidationReport> {
        let to_validation_errors = |why: ValidationError| {
            once(why).fold(ValidationReport::new(), |mut acc, err| {
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
    fn validate(&self) -> Result<(), ValidationReport> {
        let valid = match self {
            | Self::Number(value) => value.is_finite(),
            | Self::Text(value) => !value.trim().is_empty(),
        };
        match valid {
            | true => Ok(()),
            | false => {
                let mut errors = ValidationReport::new();
                errors.add(
                    "number_or_string",
                    ValidationError::new("format").with_message("Number-or-string should be finite or contain non-empty text".into()),
                );
                Err(errors)
            }
        }
    }
}
impl Validate for YearValue {
    fn validate(&self) -> Result<(), ValidationReport> {
        let to_validation_errors = |why: ValidationError| {
            once(why).fold(ValidationReport::new(), |mut acc, err| {
                acc.add("year", err);
                acc
            })
        };
        match self {
            | Self::Number(value) => rules::year(*value).map_err(to_validation_errors),
            | Self::Text(value) => rules::year(value).map_err(to_validation_errors),
        }
    }
}
impl Validate for PostalCode {
    fn validate(&self) -> Result<(), ValidationReport> {
        let to_validation_errors = |why: ValidationError| {
            once(why).fold(ValidationReport::new(), |mut acc, err| {
                acc.add("postal_code", err);
                acc
            })
        };
        let valid = match self {
            | Self::Number(value) => *value >= 0,
            | Self::Text(value) => {
                let trimmed = value.trim();
                let punctuation = [' ', '-'];
                let valid_characters = trimmed
                    .chars()
                    .all(|character| character.is_alphanumeric() || punctuation.contains(&character));
                !trimmed.is_empty() && trimmed.len() <= 20 && valid_characters
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
/// let expected = "2023-01-01T12:00:00.123456+00:00";
/// assert_eq!(format_timestamp(input), expected);
///
/// // Already in microsecond precision stays the same
/// let input = "2023-01-01T12:00:00.123456+00:00";
/// assert_eq!(format_timestamp(input), input);
///
/// // Invalid timestamp returns original string
/// let invalid = "invalid-timestamp";
/// assert_eq!(format_timestamp(invalid), invalid);
/// ```
pub fn format_timestamp(value: &str) -> String {
    #[cfg(feature = "std")]
    {
        parse_rfc3339(value)
            .map(|(timestamp, offset)| format!("{:.6}", timestamp.display_with_offset(offset)))
            .unwrap_or_else(|| value.to_string())
    }
    #[cfg(not(feature = "std"))]
    {
        value.to_string()
    }
}
/// Validate that at least one item in `values` satisfies `method`
///
/// This helper is intended for composing ACORN custom validation functions.
pub fn has_at_least_one_truthy<T>(values: &[T], method: fn(&T) -> bool, code: &'static str, message: &'static str) -> Result<(), ValidationError> {
    match values.iter().any(method) {
        | true => Ok(()),
        | false => Err(ValidationError::new(code).with_message(message.into())),
    }
}
/// Check if value is either an ISO 8601 date (`YYYY-MM-DD`) or an RFC3339 timestamp.
pub fn is_iso_date_or_rfc3339_timestamp(value: &str) -> bool {
    rules::date(value).is_ok() || parse_rfc3339(value).is_some()
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
/// Check if a value is a non-empty SPDX license identifier string.
pub fn is_license(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "license";
    const MESSAGE: &str = "Provide valid SPDX license identifier";
    let candidate = value.trim().to_string();
    let validation_error = |message: String| {
        let mut why = ValidationError::new(CODE).with_message(message.into());
        why.add_param("value".into(), &candidate);
        why
    };
    match candidate.is_empty() {
        | true => Err(validation_error(format!("{MESSAGE} (cannot be empty)"))),
        | false => Ok(()),
    }
}
/// Check whether a programming-language value is non-empty.
pub fn is_programming_language(value: &str) -> Result<(), ValidationError> {
    match value.trim().is_empty() {
        | true => Err(ValidationError::new("programming_language").with_message("Provide valid programming language (cannot be empty)".into())),
        | false => Ok(()),
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
fn parse_rfc3339(value: &str) -> Option<(Timestamp, Offset)> {
    let pieces = Pieces::parse(value).ok();
    pieces
        .filter(|parts| parts.time().is_some() && parts.time_zone_annotation().is_none() && has_compatible_offset(value))
        .and_then(|parts| parts.offset().map(|offset| (parts, offset.to_numeric_offset())))
        .and_then(|(_, offset)| value.parse::<Timestamp>().ok().map(|timestamp| (timestamp, offset)))
}
fn has_compatible_offset(value: &str) -> bool {
    let value = value.trim_end();
    value.ends_with('Z')
        || value
            .char_indices()
            .rev()
            .find(|(index, character)| *index > 9 && matches!(character, '+' | '-'))
            .map(|(index, _)| &value[index..])
            .is_some_and(|offset| {
                offset.get(1..).is_some_and(|body| match body.len() {
                    | 5 => body
                        .chars()
                        .enumerate()
                        .all(|(index, character)| if index == 2 { character == ':' } else { character.is_ascii_digit() }),
                    | 4 => body.chars().all(|character| character.is_ascii_digit()),
                    | _ => false,
                })
            })
}
#[cfg(test)]
mod tests;
