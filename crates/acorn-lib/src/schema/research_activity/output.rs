//! Structured research outputs associated with a research activity.
use crate::prelude::alloc::{String, Vec};
use crate::schema::validate::{is_doi, is_orcid, is_partial_date, is_ror};
use crate::schema::{Keyword, Website};
use crate::validation::Validate;
use bon::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_trim::{option_string_trim, string_trim};
use serde_with::skip_serializing_none;

/// A publication, dataset, software package, or other product of a research activity.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResearchOutput {
    /// Provider-native or globally resolvable identifier for the output.
    #[serde(default, deserialize_with = "option_string_trim")]
    pub identifier: Option<String>,
    /// Digital Object Identifier for the output.
    #[validate(custom(function = "is_doi"))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub doi: Option<String>,
    /// Human-readable output title.
    #[validate(length(min = 1))]
    #[serde(deserialize_with = "string_trim")]
    pub title: String,
    /// Provider or standard output type, such as `article`, `dataset`, or `software`.
    #[serde(rename = "type", default, deserialize_with = "option_string_trim")]
    pub kind: Option<String>,
    /// Publication or release date as an ISO 8601 date.
    #[validate(custom(function = "is_partial_date"))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub publication_date: Option<String>,
    /// People credited on this output.
    #[validate(nested)]
    pub contributors: Option<Vec<Contributor>>,
    /// Funding associated specifically with this output.
    #[validate(nested)]
    pub funding: Option<Vec<Funding>>,
    /// Publisher, repository, landing-page, or open-access locations for this output.
    #[validate(nested)]
    pub websites: Option<Vec<Website>>,
    /// Output-level keywords, including inferred values retained with provider provenance.
    #[builder(default)]
    pub keywords: Vec<Keyword>,
    /// Open-access information associated with this output.
    #[validate(nested)]
    pub access: Option<Access>,
}

/// A person credited on a research output.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Contributor {
    /// Display name asserted for the contributor on this output.
    #[validate(length(min = 1))]
    #[serde(deserialize_with = "string_trim")]
    pub name: String,
    /// ORCID associated with the contributor.
    #[validate(custom(function = "is_orcid"))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub orcid: Option<String>,
    /// Whether the source identifies this person as a corresponding author.
    pub corresponding: Option<bool>,
    /// Position in the output byline, such as `first`, `middle`, or `last`.
    #[serde(default, deserialize_with = "option_string_trim")]
    pub position: Option<String>,
    /// Work-specific affiliations asserted for the contributor.
    #[validate(nested)]
    #[builder(default)]
    pub affiliations: Vec<Affiliation>,
}

/// An organization affiliation asserted for an output contributor.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Affiliation {
    /// Organization display name.
    #[validate(length(min = 1))]
    #[serde(deserialize_with = "string_trim")]
    pub name: String,
    /// Research Organization Registry identifier.
    #[validate(custom(function = "is_ror"))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub ror: Option<String>,
}

/// Funding organization and awards associated with a research output.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Funding {
    /// Funding organization display name.
    #[validate(length(min = 1))]
    #[serde(deserialize_with = "string_trim")]
    pub name: String,
    /// Research Organization Registry identifier for the funder.
    #[validate(custom(function = "is_ror"))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub ror: Option<String>,
    /// Awards from this funding organization associated with the output.
    #[validate(nested)]
    #[builder(default)]
    pub awards: Vec<Award>,
}

/// A grant or other funding award associated with a research output.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Award {
    /// Provider or funder identifier for the award.
    #[serde(deserialize_with = "string_trim")]
    pub identifier: String,
    /// Award title when supplied by the source.
    #[serde(default, deserialize_with = "option_string_trim")]
    pub title: Option<String>,
    /// DOI assigned to the award.
    #[validate(custom(function = "is_doi"))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub doi: Option<String>,
    /// Resolvable award URL.
    #[validate(url)]
    pub url: Option<String>,
}

/// Open-access metadata for a research output.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Access {
    /// Whether a free-to-read copy is known to exist.
    pub open: bool,
    /// Provider access classification, such as `gold`, `green`, or `closed`.
    #[serde(default, deserialize_with = "option_string_trim")]
    pub status: Option<String>,
    /// Best known open-access URL.
    #[validate(url)]
    pub url: Option<String>,
    /// License identifier or URI asserted for the selected location.
    #[serde(default, deserialize_with = "option_string_trim")]
    pub license: Option<String>,
}
