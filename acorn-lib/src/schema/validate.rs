#![deny(missing_docs)]
//! # Schema validation helpers
//!
//! Generic validation functions and custom [validator](https://docs.rs/validator/latest/validator/) functions for validating schema data
use crate::constants::*;
use fancy_regex::Regex;
use validator::ValidationError;

/// Format a phone number into a standard format
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
/// Check if value is a valid DOI
pub fn is_doi(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid DOI, by itself and without domain or 'doi:' prefix.";
    match RE_DOI.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("doi").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid IP6
pub fn is_ip6(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid IP6 address";
    match RE_IP6.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("IP6").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid kebab-case (e.g. 'this-is-kebab-case')
pub fn is_kebabcase(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide an ID in kebab-case format";
    let kebab = match Regex::new(r"[ *_./\!@#$%^&(){}]") {
        | Ok(re) => re.replace_all(value, "").trim().to_string(),
        | Err(err) => err.to_string(),
    };
    match kebab.to_lowercase().eq(&value) {
        | true => Ok(()),
        | _ => Err(ValidationError::new("kebabcase").with_message(MESSAGE.into())),
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
/// Check if value is a valid RAiD
pub fn is_raid(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid RAiD";
    match RE_RAID.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("raid").with_message(MESSAGE.into())),
    }
}
/// Check if value is a valid ROR
pub fn is_ror(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid ROR";
    match RE_ROR.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("ror").with_message(MESSAGE.into())),
    }
}
/// Custom validator function for [approach](/acorn_lib/schema/struct.Sections.html#structfield.approach)
pub fn validate_attribute_approach(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_APPROACH;
    let message: String = format!("Each approach statement should be less than {MAX_LENGTH} characters");
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("approach").with_message(message.into())),
    }
}
/// Custom validator function for [research areas](/acorn_lib/schema/struct.Research.html#structfield.areas)
pub fn validate_attribute_areas(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_RESEARCH_AREA;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("area").with_message(format!("Each area should be less than {MAX_LENGTH} characters").into())),
    }
}
/// Custom validator function for `ReseaarchActivity` [capabilities](/acorn_lib/schema/struct.Sections.html#structfield.capabilities)
pub fn validate_attribute_capabilities(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_CAPABILIY;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("capability").with_message(format!("Each capability should be less than {MAX_LENGTH} characters").into())),
    }
}
// TODO: Check that statments start with capital letter (use regex for period and captial?)
/// Custom validator function for `ReseaarchActivity` [impact](/acorn_lib/schema/struct.Sections.html#structfield.impact)
pub fn validate_attribute_impact(value: &[String]) -> Result<(), ValidationError> {
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
