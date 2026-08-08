//! Custom validation implementations for DCAT untagged enums and helper functions.
use super::{ConformsTo, ContactPoint, DocumentRef, OneOrMany, Publisher};
use crate::prelude::*;
use crate::schema::validate::{is_url, is_urls};
use validator::{Validate, ValidationError, ValidationErrors};

impl Validate for ConformsTo {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            | Self::Uri(_) => Ok(()),
            | Self::Standard(standard) => standard.validate(),
        }
    }
}
impl Validate for ContactPoint {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            | Self::Uri(_) => Ok(()),
            | Self::Kind(kind) => kind.validate(),
        }
    }
}
impl Validate for DocumentRef {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            | Self::Uri(_) => Ok(()),
            | Self::Document(document) => document.validate(),
        }
    }
}
impl Validate for Publisher {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            | Self::Organization(organization) => organization.validate(),
            | Self::Agent(agent) => agent.validate(),
        }
    }
}
pub(crate) fn is_document_refs_urls(value: &OneOrMany<DocumentRef>) -> Result<(), ValidationError> {
    value
        .iter()
        .filter_map(DocumentRef::url)
        .find_map(|url| is_url(url).err())
        .map_or(Ok(()), Err)
}
pub(crate) fn is_one_or_many_urls(value: &OneOrMany<String>) -> Result<(), ValidationError> {
    is_urls(value.as_slice())
}
