use super::datacite::{Affiliation, FunderIdentifier, FunderIdentifierType, FundingReference, NameIdentifier, Publisher};
use super::invenio::{PersonOrOrgIdentifier, PersonOrOrgIdentifierScheme};
use crate::prelude::alloc::String;
use crate::schema::validate::{is_gnd, is_isni, is_orcid, is_ror};
use validator::ValidationError;

fn identifier_validation_error(code: &'static str, message: impl Into<String>) -> ValidationError {
    ValidationError::new(code).with_message(message.into().into())
}
fn validate_named_identifier(identifier: &str, scheme: Option<&str>, code: &'static str) -> Result<(), ValidationError> {
    match scheme.map(str::trim).filter(|value| !value.is_empty()) {
        | Some(value) if value.eq_ignore_ascii_case("ISNI") => is_isni(identifier),
        | Some(value) if value.eq_ignore_ascii_case("ORCID") => is_orcid(identifier),
        | Some(value) if value.eq_ignore_ascii_case("ROR") => is_ror(identifier),
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
        | Some(FunderIdentifierType::Isni) => is_isni(identifier),
        | Some(FunderIdentifierType::Ror) => is_ror(identifier),
        | Some(_) => Ok(()),
        | None => Err(identifier_validation_error(
            "funder_identifier",
            "Provide a funder identifier type when a funder identifier is present",
        )),
    }
}
pub(super) fn validate_affiliation_identifier(value: &Affiliation) -> Result<(), ValidationError> {
    validate_optional_named_identifier(
        value.affiliation_identifier.as_ref(),
        value.affiliation_identifier_scheme.as_ref(),
        "affiliation_identifier",
    )
}
pub(super) fn validate_name_identifier(value: &NameIdentifier) -> Result<(), ValidationError> {
    validate_named_identifier(&value.name_identifier, value.name_identifier_scheme.as_deref(), "name_identifier")
}
pub(super) fn validate_publisher_identifier(value: &Publisher) -> Result<(), ValidationError> {
    validate_optional_named_identifier(
        value.publisher_identifier.as_ref(),
        value.publisher_identifier_scheme.as_ref(),
        "publisher_identifier",
    )
}
pub(super) fn validate_funder_identifier(value: &FunderIdentifier) -> Result<(), ValidationError> {
    validate_typed_funder_identifier(&value.value, value.funder_identifier_type.as_ref())
}
pub(super) fn validate_funding_reference_identifier(value: &FundingReference) -> Result<(), ValidationError> {
    match value.funder_identifier.as_ref() {
        | Some(identifier) => validate_typed_funder_identifier(identifier, value.funder_identifier_type.as_ref()),
        | None => Ok(()),
    }
}
pub(super) fn validate_person_or_org_identifier(value: &PersonOrOrgIdentifier) -> Result<(), ValidationError> {
    let result = match value.scheme {
        | PersonOrOrgIdentifierScheme::Gnd => is_gnd(&value.identifier),
        | PersonOrOrgIdentifierScheme::Isni => is_isni(&value.identifier),
        | PersonOrOrgIdentifierScheme::Orcid => is_orcid(&value.identifier),
        | PersonOrOrgIdentifierScheme::Ror => is_ror(&value.identifier),
    };
    result.map_err(|why| {
        let mut error = ValidationError::new("person_or_org_identifier")
            .with_message(why.message.unwrap_or_else(|| "Identifier does not match its scheme".into()));
        error.add_param("scheme".into(), &value.scheme);
        error.add_param("identifier".into(), &value.identifier);
        error
    })
}
