//! # Schema validation helpers
//!
//! Generic validation functions and custom [validator](https://docs.rs/validator/latest/validator/) functions for validating schema data
//!
use crate::schema::pid::{Patent, PersistentIdentifierConvert};
use crate::util::constants::{
    MAX_LENGTH_APPROACH, MAX_LENGTH_CAPABILIY, MAX_LENGTH_IMPACT, MAX_LENGTH_RESEARCH_AREA, RE_FAKE_PHONE, RE_IMAGE_EXTENSION, RE_IP6,
    RE_ISO_8601_DATE, RE_ISO_8601_YEAR, RE_PHONE, RE_RAID, RE_UNIX_EPOCH,
};
use chrono::{DateTime, Datelike, Utc};
use convert_case::{Case, Casing};
use uriparse::URI;
use validator::ValidationError;

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
            | None => Err(ValidationError::new("telephone").with_message(MESSAGE.into())),
        },
        | _ => Err(ValidationError::new("telephone").with_message(MESSAGE.into())),
    }
}
/// Check if a path has a valid image extension
pub fn has_image_extension(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a path with a PNG, JPEG, GIF, WEBP, TIFF or SVG extension";
    match RE_IMAGE_EXTENSION.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("image").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid [`crate::schema::pid::ARK`]
pub fn is_ark(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid ARK";
    match value.is_ark() {
        | true => Ok(()),
        | _ => Err(ValidationError::new("ark").with_message(MESSAGE.into())),
    }
}
// TODO: Add chrono crate to handle dates
fn is_current_year(value: String) -> bool {
    let now: DateTime<Utc> = Utc::now();
    let year = now.year().to_string().parse::<i32>().unwrap_or_default();
    value.parse::<i32>().unwrap_or_default() <= year
}
/// Check if value is a valid [`crate::schema::pid::DOI`]
pub fn is_doi(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid DOI, by itself and without domain or 'doi:' prefix.";
    match value.is_doi() {
        | true => Ok(()),
        | _ => Err(ValidationError::new("doi").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid IP6 address
pub fn is_ip6(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid IP6 address";
    match RE_IP6.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("IP6").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid [`crate::schema::pid::ISBN`]
/// ### Example
/// > `978-3-16-148410-0`
pub fn is_isbn(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid ISBN";
    match value.is_isbn() {
        | true => Ok(()),
        | _ => Err(ValidationError::new("ISBN").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid ISO 8601 date (e.g., YYYY-MM-DD)
/// ### Example
/// > `2025-06-04`
pub fn is_iso8601_date(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid ISO 8601 date (e.g., YYYY-MM-DD)";
    // TODO: Make sure not in the future
    match RE_ISO_8601_DATE.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("ISO 8601 Date").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid ISO 8601 year (e.g., YYYY)
/// ### Examples
/// - `2025`
pub fn is_iso8601_year(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid ISO 8601 year (e.g., YYYY)";
    match RE_ISO_8601_YEAR.is_match(value) {
        | Ok(x) if x && is_current_year(value.to_string()) => Ok(()),
        | _ => Err(ValidationError::new("ISO 8601 Date").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid kebab-case (e.g. 'this-is-kebab-case')
pub fn is_kebabcase(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide an ID in kebab-case format";
    match value.to_case(Case::Kebab).eq(&value) {
        | true => Ok(()),
        | _ => Err(ValidationError::new("kebabcase").with_message(MESSAGE.into())),
    }
}
/// Custom validator function for validating list of URLs
pub fn is_list_url(value: &[String]) -> Result<(), ValidationError> {
    let is_valid = value.iter().all(|x| URI::try_from(x.as_str()).is_ok());
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("URLs").with_message("Every URL should be valid".to_string().into())),
    }
}
/// Check if value is a valid [`crate::schema::pid::ORCID`]
pub fn is_orcid(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid ORCiD";
    match value.is_orcid() {
        | true => Ok(()),
        | _ => Err(ValidationError::new("orcid").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid [`crate::schema::pid::Patent`]
pub fn is_patent_identifier(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid patent identifier";
    match Patent::is_valid(value) {
        | true => Ok(()),
        | _ => Err(ValidationError::new("patent").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid phone number
///
/// Uses same regex as `format_phone_number`
pub fn is_phone_number(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid phone number";
    let is_fake = match RE_FAKE_PHONE.is_match(value) {
        | Ok(value) if value => true,
        | _ => false,
    };
    match RE_PHONE.is_match(value) {
        | Ok(value) if value && !is_fake => Ok(()),
        | _ => Err(ValidationError::new("phone").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid [`crate::schema::pid::raid`]
pub fn is_raid(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid RAiD";
    match RE_RAID.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("raid").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid Research Organization Registry (ROR) value
///
/// See <https://www.ror.org/> for more information
pub fn is_ror(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid ROR";
    match value.is_ror() {
        | true => Ok(()),
        | _ => Err(ValidationError::new("ror").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid Unix epoch timestamp
pub fn is_unix_epoch(value: usize) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid Unix epoch timestamp";
    match RE_UNIX_EPOCH.is_match(&value.to_string()) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("unix epoch").with_message(MESSAGE.into())),
    }
}
/// Custom validator function for [approach](/acorn_lib/schema/struct.Sections.html#structfield.approach)
pub(crate) fn validate_attribute_approach(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_APPROACH;
    let message: String = format!("Each approach statement should be less than {MAX_LENGTH} characters");
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("approach").with_message(message.into())),
    }
}
/// Custom validator function for [research areas](/acorn_lib/schema/struct.Research.html#structfield.areas)
pub(crate) fn validate_attribute_areas(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_RESEARCH_AREA;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("area").with_message(format!("Each area should be less than {MAX_LENGTH} characters").into())),
    }
}
/// Custom validator function for [`ResearchActivity`] [patents](/acorn_lib/schema/struct.Metadata.html#structfield.patents)
///
/// [`ResearchActivity`]: ../struct.ResearchActivity.html
pub(crate) fn validate_attribute_books(value: &[String]) -> Result<(), ValidationError> {
    let is_valid = value.iter().all(|x| x.is_isbn());
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("books").with_message("Every book should be a valid ISBN".to_string().into())),
    }
}
/// Custom validator function for [`ResearchActivity`] [capabilities](/acorn_lib/schema/struct.Sections.html#structfield.capabilities)
///
/// [`ResearchActivity`]: ../struct.ResearchActivity.html
pub(crate) fn validate_attribute_capabilities(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_CAPABILIY;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("capability").with_message(format!("Each capability should be less than {MAX_LENGTH} characters").into())),
    }
}
/// Custom validator function for [`ResearchActivity`] [doi](/acorn_lib/schema/struct.Metadata.html#structfield.doi)
///
/// [`ResearchActivity`]: ../struct.ResearchActivity.html
pub(crate) fn validate_attribute_doi(value: &[String]) -> Result<(), ValidationError> {
    let is_valid = value.iter().all(|x| is_doi(x).is_ok());
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("DOIs").with_message("Every DOI should be valid".to_string().into())),
    }
}
/// Custom validator function for [`ResearchActivity`] [ror](/acorn_lib/schema/struct.Metadata.html#structfield.ror)
///
/// [`ResearchActivity`]: ../struct.ResearchActivity.html
pub fn validate_attribute_ror(value: &str) -> Result<(), ValidationError> {
    match is_ror(value).is_ok() {
        | true => Ok(()),
        | _ => Err(ValidationError::new("RORs").with_message("Every ROR should be valid".to_string().into())),
    }
}
/// Custom validator function for [`ResearchActivity`] [ror](/acorn_lib/schema/struct.Metadata.html#structfield.ror)
///
/// [`ResearchActivity`]: ../struct.ResearchActivity.html
pub(crate) fn validate_attribute_ror_list(value: &[String]) -> Result<(), ValidationError> {
    let is_valid = value.iter().all(|x| is_ror(x).is_ok());
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("RORs").with_message("Every ROR should be valid".to_string().into())),
    }
}
// TODO: Check that statments start with capital letter (use regex for period and captial?)
/// Custom validator function for [`ResearchActivity`] [impact](/acorn_lib/schema/struct.Sections.html#structfield.impact)
///
/// [`ResearchActivity`]: ../struct.ResearchActivity.html
pub(crate) fn validate_attribute_impact(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_IMPACT;
    match value.iter().all(|x| x.len() <= MAX_LENGTH) {
        | true => {
            let all_periods = value.iter().all(|x| x.trim().ends_with("."));
            let no_periods = value.iter().all(|x| !x.trim().ends_with("."));
            let is_valid = all_periods || no_periods;
            match is_valid {
                | true => Ok(()),
                | _ => Err(ValidationError::new("impact")
                    .with_message("Impact statements should be all sentences with periods or all phrases without periods".into())),
            }
        }
        | _ => Err(ValidationError::new("impact").with_message(format!("Each impact statement should be less than {MAX_LENGTH} characters").into())),
    }
}
