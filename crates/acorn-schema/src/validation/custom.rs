//! Schema validation helpers
//!
//! Generic validation functions and custom rules for validating schema data.
//!
use super::{report_of, rules, IntoValidationReport, Validate, ValidationReport};
use crate::geonames::{CodeFormat, GeonamesParser};
use crate::namespaces::{DEFAULT_ISNI_SCHEMA_URI, DEFAULT_ORCID_SCHEMA_URI, DEFAULT_ROR_SCHEMA_URI};
use crate::regex_validator;
use crate::research_activity::AgentIdentifier;
use crate::standard::datacite::{Affiliation, FunderIdentifier, FunderIdentifierType, FundingReference, GeoLocationPoint, NameIdentifier, Publisher};
use crate::standard::invenio::{PersonOrOrgIdentifier, PersonOrOrgIdentifierScheme};
use crate::util::constants::{RE_GND, RE_IMAGE_EXTENSION, RE_PARTIAL_DATE, RE_PHONE};
use crate::util::Constant;
use crate::OneOrMany;
use acorn_core::prelude::alloc::{format, String, ToString, Vec};
use acorn_core::validation::ValidationError;
use alloc::collections::BTreeMap;
use core::iter::once;
use jiff::{fmt::temporal::Pieces, tz::Offset, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, PartialEq)]
enum IdentifierScheme {
    Isni,
    Orcid,
    Other,
    Ror,
}
#[derive(Clone, Copy)]
struct SchemeIdentifier<'a> {
    identifier: &'a str,
    scheme: IdentifierScheme,
}
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
/// Postal code represented as either text or integer.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PostalCode {
    /// Numeric postal code.
    Number(i64),
    /// String postal code.
    Text(String),
}
impl From<FunderIdentifierType> for IdentifierScheme {
    fn from(value: FunderIdentifierType) -> Self {
        match value {
            | FunderIdentifierType::Isni => Self::Isni,
            | FunderIdentifierType::Ror => Self::Ror,
            | FunderIdentifierType::CrossrefFunderId | FunderIdentifierType::Grid | FunderIdentifierType::Other => Self::Other,
        }
    }
}
impl IdentifierScheme {
    fn from_name(value: &str) -> Self {
        match value {
            | value if value.eq_ignore_ascii_case("ISNI") => Self::Isni,
            | value if value.eq_ignore_ascii_case("ORCID") => Self::Orcid,
            | value if value.eq_ignore_ascii_case("ROR") => Self::Ror,
            | _ => Self::Other,
        }
    }
    fn from_uri(value: &str) -> Option<Self> {
        let value = value.trim().trim_end_matches('/');
        let isni_root = DEFAULT_ISNI_SCHEMA_URI.strip_suffix("/isni");
        match value {
            | value if value == DEFAULT_ISNI_SCHEMA_URI.trim_end_matches('/') || isni_root == Some(value) => Some(Self::Isni),
            | value if value == DEFAULT_ORCID_SCHEMA_URI.trim_end_matches('/') => Some(Self::Orcid),
            | value if value == DEFAULT_ROR_SCHEMA_URI.trim_end_matches('/') => Some(Self::Ror),
            | _ => None,
        }
    }
    fn name(self) -> &'static str {
        match self {
            | Self::Isni => "ISNI",
            | Self::Orcid => "ORCID",
            | Self::Other => "OTHER",
            | Self::Ror => "ROR",
        }
    }
}
impl Validate for SchemeIdentifier<'_> {
    fn validate(&self) -> Result<(), ValidationReport> {
        match self.scheme {
            | IdentifierScheme::Isni => rules::isni(self.identifier),
            | IdentifierScheme::Orcid => rules::orcid(self.identifier),
            | IdentifierScheme::Other => Ok(()),
            | IdentifierScheme::Ror => rules::ror(self.identifier),
        }
        .map_err(|error| error.into_report(""))
    }
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
                    ValidationError::new("required").with_message("Integer-or-string text should not be empty"),
                );
                Err(errors)
            }
        }
    }
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
                ValidationError::new("range").with_message("Month should be an integer 1-12 or a string value from \"1\" to \"12\""),
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
                    ValidationError::new("format").with_message("Number-or-string should be finite or contain non-empty text"),
                );
                Err(errors)
            }
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
                "Postal code should be a non-negative integer or a non-empty string up to 20 characters using letters, digits, spaces, or hyphens",
            ))),
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
/// Format a phone number into a standard format
/// ### Note
/// > Output format is `000.000.0000`
///
/// ### Example
/// ```rust
/// use acorn_schema::validation::format_phone_number;
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
            | None => Err(ValidationError::new(CODE).with_message(MESSAGE)),
        },
        | _ => Err(ValidationError::new(CODE).with_message(MESSAGE)),
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
/// use acorn_schema::validation::format_timestamp;
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
        | false => Err(ValidationError::new(code).with_message(message)),
    }
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
regex_validator!(
    has_image_extension,
    RE_IMAGE_EXTENSION,
    "image",
    "Provide path with a PNG, JPEG, GIF, WEBP, TIFF or SVG extension"
);
fn identifier_validation_error(code: &'static str, message: impl Into<String>) -> ValidationError {
    ValidationError::new(code).with_message(message)
}
regex_validator!(is_gnd, RE_GND, "GND");
/// Return whether a stable identifier contains only ASCII letters, digits, hyphens, underscores, or periods.
pub fn is_identifier<T: AsRef<str> + ?Sized>(value: &T) -> bool {
    let value = value.as_ref();
    let is_only_valid_characters = value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'));
    !value.is_empty() && is_only_valid_characters
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
        | false => Err(ValidationError::new(CODE).with_message(MESSAGE)),
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
        | false => Err(ValidationError::new(CODE).with_message(MESSAGE)),
    }
}
/// Check if value is either an ISO 8601 date (`YYYY-MM-DD`) or an RFC3339 timestamp.
pub fn is_iso_date_or_rfc3339_timestamp(value: &str) -> bool {
    rules::date(value).is_ok() || parse_rfc3339(value).is_some()
}
/// Check if a value is a non-empty SPDX license identifier string.
pub fn is_license(value: &str) -> Result<(), ValidationError> {
    const CODE: &str = "license";
    const MESSAGE: &str = "Provide valid SPDX license identifier";
    let candidate = value.trim().to_string();
    let validation_error = |message: String| {
        let mut why = ValidationError::new(CODE).with_message(message);
        why.add_param("value".into(), &candidate);
        why
    };
    match candidate.is_empty() {
        | true => Err(validation_error(format!("{MESSAGE} (cannot be empty)"))),
        | false => Ok(()),
    }
}
regex_validator!(
    is_partial_date,
    RE_PARTIAL_DATE,
    "date",
    "Provide valid date (e.g., YYYY-MM-DD or YYYY-MM)"
);
/// Check if list of locations depict a valid polygon
pub fn is_polygon(value: &[GeoLocationPoint]) -> Result<(), ValidationError> {
    const CODE: &str = "polygon";
    const EMPTY_MESSAGE: &str = "Provide valid polygon — no points were provided";
    let length = value.len();
    let result = if length == 0 {
        Err(ValidationError::new(CODE).with_message(EMPTY_MESSAGE))
    } else if length < 4 {
        let length_message = format!("Provide valid polygon — at least 4 points are required (only {length} found)");
        Err(ValidationError::new(CODE).with_message(length_message))
    } else {
        match (value.first(), value.last()) {
            | (Some(first), Some(last)) if first == last => Ok(()),
            | (Some(first), Some(last)) => {
                let closed_message = format!("Provide valid polygon — first point must equal last point ({first} ≠ {last})");
                Err(ValidationError::new(CODE).with_message(closed_message))
            }
            | _ => Err(ValidationError::new(CODE).with_message(EMPTY_MESSAGE)),
        }
    };
    result
}
/// Check whether a programming-language value is non-empty.
pub fn is_programming_language(value: &str) -> Result<(), ValidationError> {
    match value.trim().is_empty() {
        | true => Err(ValidationError::new("programming_language").with_message("Provide valid programming language (cannot be empty)")),
        | false => Ok(()),
    }
}
fn parse_rfc3339(value: &str) -> Option<(Timestamp, Offset)> {
    let pieces = Pieces::parse(value).ok();
    pieces
        .filter(|parts| parts.time().is_some() && parts.time_zone_annotation().is_none() && has_compatible_offset(value))
        .and_then(|parts| parts.offset().map(|offset| (parts, offset.to_numeric_offset())))
        .and_then(|(_, offset)| value.parse::<Timestamp>().ok().map(|timestamp| (timestamp, offset)))
}
pub(crate) fn validate_affiliation_identifier(value: &Affiliation, _context: &()) -> Result<(), ValidationReport> {
    prefix_validation(
        "affiliation_identifier",
        validate_optional_named_identifier(
            "affiliation_identifier",
            value.affiliation_identifier.as_ref(),
            value.affiliation_identifier_scheme.as_ref(),
            value.scheme_uri.as_deref(),
        ),
    )
}
pub(crate) fn validate_agent_identifier(value: &AgentIdentifier, _context: &()) -> Result<(), ValidationReport> {
    prefix_validation(
        "value",
        validate_named_identifier(
            &value.value,
            true,
            "agent_identifier",
            value.scheme.as_deref(),
            value.scheme_uri.as_deref(),
        ),
    )
}
pub(crate) fn validate_funder_identifier(value: &FunderIdentifier, _context: &()) -> Result<(), ValidationReport> {
    let scheme = value.funder_identifier_type.clone().map(IdentifierScheme::from);
    prefix_validation(
        "value",
        validate_identifier_scheme(&value.value, true, "funder_identifier", scheme.map(IdentifierScheme::name), scheme, None),
    )
}
pub(crate) fn validate_funding_reference_identifier(value: &FundingReference, _context: &()) -> Result<(), ValidationReport> {
    match value.funder_identifier.as_ref() {
        | Some(identifier) => {
            let scheme = value.funder_identifier_type.clone().map(IdentifierScheme::from);
            prefix_validation(
                "funder_identifier",
                validate_identifier_scheme(
                    identifier,
                    true,
                    "funder_identifier",
                    scheme.map(IdentifierScheme::name),
                    scheme,
                    value.scheme_uri.as_deref(),
                ),
            )
        }
        | None => Ok(()),
    }
}
pub(crate) fn validate_license(value: &OneOrMany<String>) -> Result<(), ValidationError> {
    value.iter().find_map(|value| is_license(value).err()).map_or(Ok(()), Err)
}
pub(crate) fn validate_name_identifier(value: &NameIdentifier, _context: &()) -> Result<(), ValidationReport> {
    prefix_validation(
        "name_identifier",
        validate_named_identifier(
            &value.name_identifier,
            true,
            "name_identifier",
            value.name_identifier_scheme.as_deref(),
            value.scheme_uri.as_deref(),
        ),
    )
}
fn validate_named_identifier(
    identifier: &str,
    require_scheme: bool,
    code: &'static str,
    scheme: Option<&str>,
    scheme_uri: Option<&str>,
) -> Result<(), ValidationReport> {
    let scheme = scheme.map(str::trim).filter(|value| !value.is_empty());
    validate_identifier_scheme(
        identifier,
        require_scheme,
        code,
        scheme,
        scheme.map(IdentifierScheme::from_name),
        scheme_uri,
    )
}
fn validate_identifier_scheme(
    identifier: &str,
    require_scheme: bool,
    code: &'static str,
    scheme_name: Option<&str>,
    scheme: Option<IdentifierScheme>,
    scheme_uri: Option<&str>,
) -> Result<(), ValidationReport> {
    let uri_scheme = scheme_uri.and_then(IdentifierScheme::from_uri);
    match (scheme, scheme_uri, uri_scheme, require_scheme) {
        | (None, Some(_), Some(_), _) => Err(identifier_coherence_error(
            code,
            identifier,
            "Provide an identifier scheme for the known scheme URI",
            scheme_name,
            scheme_uri,
        )
        .into_report("")),
        | (None, _, _, true) => Err(identifier_coherence_error(
            code,
            identifier,
            "Provide an identifier scheme when an identifier is present",
            scheme_name,
            scheme_uri,
        )
        .into_report("")),
        | (None, _, _, false) | (Some(IdentifierScheme::Other), _, None, _) => Ok(()),
        | (Some(expected), Some(_), Some(actual), _) if expected != actual => {
            Err(
                identifier_coherence_error(code, identifier, "Identifier scheme and scheme URI do not agree", scheme_name, scheme_uri)
                    .into_report(""),
            )
        }
        | (Some(IdentifierScheme::Other), _, Some(_), _) => {
            Err(
                identifier_coherence_error(code, identifier, "Identifier scheme and scheme URI do not agree", scheme_name, scheme_uri)
                    .into_report(""),
            )
        }
        | (Some(_), Some(_), None, _) => Err(identifier_coherence_error(
            code,
            identifier,
            "Provide the canonical HTTPS URI for the identifier scheme",
            scheme_name,
            scheme_uri,
        )
        .into_report("")),
        | (Some(scheme), _, _, _) => SchemeIdentifier { identifier, scheme }
            .validate()
            .map_err(|report| map_validation_errors(report, |error| identifier_rule_error(error, identifier, scheme_name, scheme_uri))),
    }
}
fn validate_optional_named_identifier(
    code: &'static str,
    identifier: Option<&String>,
    scheme: Option<&String>,
    scheme_uri: Option<&str>,
) -> Result<(), ValidationReport> {
    match identifier {
        | Some(value) => validate_named_identifier(value, true, code, scheme.map(String::as_str), scheme_uri),
        | None => Ok(()),
    }
}
pub(crate) fn validate_person_or_org_identifier(value: &PersonOrOrgIdentifier, _context: &()) -> Result<(), ValidationReport> {
    let result = match value.scheme {
        | PersonOrOrgIdentifierScheme::Gnd => is_gnd(&value.identifier).map_err(|error| error.into_report("")),
        | PersonOrOrgIdentifierScheme::Isni => validate_named_identifier(&value.identifier, true, "person_or_org_identifier", Some("ISNI"), None),
        | PersonOrOrgIdentifierScheme::Orcid => validate_named_identifier(&value.identifier, true, "person_or_org_identifier", Some("ORCID"), None),
        | PersonOrOrgIdentifierScheme::Ror => validate_named_identifier(&value.identifier, true, "person_or_org_identifier", Some("ROR"), None),
    };
    let result = result.map_err(|report| {
        map_validation_errors(report, |why| ValidationError {
            code: "person_or_org_identifier".to_string(),
            message: Some(why.message.unwrap_or_else(|| "Identifier does not match its scheme".into())),
            params: why
                .params
                .into_iter()
                .chain([
                    ("identifier".to_string(), serde_json::json!(value.identifier)),
                    ("scheme".to_string(), serde_json::json!(value.scheme)),
                ])
                .collect(),
        })
    });
    prefix_validation("identifier", result)
}
pub(crate) fn validate_publisher_identifier(value: &Publisher, _context: &()) -> Result<(), ValidationReport> {
    prefix_validation(
        "publisher_identifier",
        validate_optional_named_identifier(
            "publisher_identifier",
            value.publisher_identifier.as_ref(),
            value.publisher_identifier_scheme.as_ref(),
            value.scheme_uri.as_deref(),
        ),
    )
}
fn map_validation_errors(report: ValidationReport, map_error: impl Fn(ValidationError) -> ValidationError) -> ValidationReport {
    report.into_issues().into_iter().fold(ValidationReport::new(), |report, issue| {
        report.merge("", ValidationReport::from_error(issue.path, map_error(issue.error)))
    })
}
fn prefix_validation(path: &str, result: Result<(), ValidationReport>) -> Result<(), ValidationReport> {
    ValidationReport::new().merge(path, report_of(result)).finish()
}
fn identifier_coherence_error(
    code: &'static str,
    identifier: &str,
    message: &'static str,
    scheme: Option<&str>,
    scheme_uri: Option<&str>,
) -> ValidationError {
    ValidationError {
        code: code.to_string(),
        message: Some(message.to_string()),
        params: BTreeMap::from([
            ("identifier".to_string(), serde_json::json!(identifier)),
            ("scheme".to_string(), serde_json::json!(scheme)),
            ("scheme_uri".to_string(), serde_json::json!(scheme_uri)),
        ]),
    }
}
fn identifier_rule_error(error: ValidationError, identifier: &str, scheme: Option<&str>, scheme_uri: Option<&str>) -> ValidationError {
    ValidationError {
        code: error.code,
        message: error.message,
        params: error
            .params
            .into_iter()
            .chain([
                ("identifier".to_string(), serde_json::json!(identifier)),
                ("scheme".to_string(), serde_json::json!(scheme)),
                ("scheme_uri".to_string(), serde_json::json!(scheme_uri)),
            ])
            .collect(),
    }
}
