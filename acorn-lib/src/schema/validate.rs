use crate::constants::*;
use fancy_regex::Regex;
use std::collections::HashMap;
use validator::ValidationError;
use validator::{Validate, ValidationErrorsKind};

pub fn format_errors(found: Vec<Option<HashMap<String, ValidationErrorsKind>>>) -> HashMap<String, ValidationErrorsKind> {
    found.clone().into_iter().flatten().fold(HashMap::new(), |mut acc, x| {
        for (key, value) in x.iter() {
            acc.insert(key.clone(), value.clone());
        }
        acc
    })
}
pub fn get_validation_errors<T: Validate>(attribute: T) -> Option<HashMap<String, ValidationErrorsKind>> {
    match attribute.validate() {
        | Ok(_) => None,
        | Err(err) => Some(
            err.into_errors()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect::<HashMap<String, ValidationErrorsKind>>(),
        ),
    }
}
pub fn get_error_count(errors: HashMap<String, ValidationErrorsKind>) -> usize {
    errors
        .clone()
        .into_values()
        .map(|v| match v {
            | ValidationErrorsKind::Field(_) => 1,
            | ValidationErrorsKind::Struct(errors) => errors.into_errors().len(),
            | ValidationErrorsKind::List(_) => 0,
        })
        .sum()
}
pub fn has_image_extension(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a path with a PNG, JPEG, or SVG extension";
    match RE_IMAGE_EXTENSION.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("image").with_message(MESSAGE.into())),
    }
}
pub fn is_doi(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid DOI, by itself and without domain or 'doi:' prefix.";
    match RE_DOI.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("doi").with_message(MESSAGE.into())),
    }
}
pub fn is_ip6(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid IP6 address";
    match RE_IP6.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("IP6").with_message(MESSAGE.into())),
    }
}
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
pub fn is_phone_number(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid phone number";
    match RE_PHONE.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("phone").with_message(MESSAGE.into())),
    }
}
pub fn is_raid(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid RAiD";
    match RE_DOI.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("raid").with_message(MESSAGE.into())),
    }
}
pub fn is_ror(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid ROR";
    match RE_ROR.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("ror").with_message(MESSAGE.into())),
    }
}
pub fn validate_attribute_areas(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_RESEARCH_AREA;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("area").with_message(format!("Each area should be less than {} characters", MAX_LENGTH).into())),
    }
}
pub fn validate_attribute_capabilities(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_CAPABILIY;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("capability").with_message(format!("Each capability should be less than {} characters", MAX_LENGTH).into())),
    }
}
pub fn validate_attribute_facilities(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_FACILITY;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("facility").with_message(format!("Each facility should be less than {} characters", MAX_LENGTH).into())),
    }
}
pub fn validate_attribute_impact(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_IMPACT;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("impact").with_message(format!("Each impact item should be less than {} characters", MAX_LENGTH).into())),
    }
}
pub fn validate_attribute_technical(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_TECHNICAL;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("technical approach")
            .with_message(format!("Each element of technical approach should be less than {} characters", MAX_LENGTH).into())),
    }
}
pub fn validate_attribute_outcomes(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_OUTCOME;
    match value.iter().all(|x| x.len() <= MAX_LENGTH) {
        | true => {
            let is_valid = value.iter().all(|x| !x.trim().ends_with("."));
            match is_valid {
                | true => Ok(()),
                | _ => Err(ValidationError::new("outcome").with_message("Outcomes should be phrases that do not end in periods".into())),
            }
        }
        | _ => Err(ValidationError::new("outcome").with_message(format!("Each outcome should be less than {} characters", MAX_LENGTH).into())),
    }
}
