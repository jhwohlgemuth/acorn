//! Structured research outputs associated with a research activity
use crate::research_activity::common::{ControlledSubject, StructuredRights, TypedActivityDate, TypedIdentifier, TypedRelatedResource};
use crate::standard::datacite::{self, Description, GeoLocation, Publisher, ResourceTypeGeneral};
use crate::validation::Validate;
use crate::{Keyword, Website};
use acorn_core::prelude::alloc::{String, Vec};
use bon::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_trim::{option_string_trim, string_trim};
use serde_with::skip_serializing_none;

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
    #[serde(default)]
    pub license: Option<String>,
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
    #[validate(ror)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub ror: Option<String>,
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
    #[validate(doi)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub doi: Option<String>,
    /// Resolvable award URL.
    #[validate(url)]
    pub url: Option<String>,
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
    #[validate(orcid)]
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
    #[validate(ror)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub ror: Option<String>,
    /// Awards from this funding organization associated with the output.
    #[validate(nested)]
    #[builder(default)]
    pub awards: Vec<Award>,
}
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
    #[validate(doi)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub doi: Option<String>,
    /// Human-readable output title.
    #[validate(length(min = 1))]
    #[serde(deserialize_with = "string_trim")]
    pub title: String,
    /// Provider or standard output type, such as `article`, `dataset`, or `software`.
    #[serde(rename = "type", default, deserialize_with = "option_string_trim")]
    pub kind: Option<String>,
    /// Publication year (deposit-ready, DataCite `publicationYear`).
    #[validate(year)]
    #[serde(rename = "publicationYear")]
    pub publication_year: Option<i32>,
    /// Publication and related dates with explicit types (e.g. `Issued`).
    #[validate(nested)]
    pub dates: Option<Vec<TypedActivityDate>>,
    /// Publisher with identifier (DataCite `publisher`, deposit mandatory).
    #[validate(nested)]
    pub publisher: Option<Publisher>,
    /// Controlled general resource type (e.g. `Poster`, `Dataset`).
    pub resource_type_general: Option<ResourceTypeGeneral>,
    /// Specific resource type free text.
    #[serde(default, deserialize_with = "option_string_trim")]
    pub resource_type: Option<String>,
    /// ISO 639-1 language code.
    #[validate(custom(function = "crate::validate::is_iso_639_1_language_code"))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub language: Option<String>,
    /// Alternate identifiers (typed).
    #[validate(nested)]
    pub alternate_identifiers: Option<Vec<TypedIdentifier>>,
    /// Typed related identifiers (DOI/RAiD/SWHID with `relationType`).
    #[validate(nested)]
    pub related_identifiers: Option<Vec<TypedRelatedResource>>,
    /// Related items (DataCite 4.7 prop 20, distinct from identifiers).
    #[validate(nested)]
    pub related_items: Option<Vec<datacite::RelatedItem>>,
    /// Semantic version.
    #[validate(version)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub version: Option<String>,
    /// Structured rights list (SPDX) — keep `access` for OA.
    #[validate(nested)]
    pub rights_list: Option<Vec<StructuredRights>>,
    /// Typed descriptions.
    #[validate(nested)]
    pub descriptions: Option<Vec<Description>>,
    /// Controlled subjects with scheme/valueURI (extends `keywords`).
    #[validate(nested)]
    pub subjects: Option<Vec<ControlledSubject>>,
    /// Formats (MIME/extension).
    pub formats: Option<Vec<String>>,
    /// Sizes (e.g. `15 pages`, `6 MB`).
    pub sizes: Option<Vec<String>>,
    /// Geographic locations.
    #[validate(nested)]
    pub geo_locations: Option<Vec<GeoLocation>>,
    /// Creators distinct from contributors (deposit mandatory `creators`).
    #[validate(nested)]
    pub creators: Option<Vec<Contributor>>,
    /// People credited on this output (DataCite `contributors`).
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
impl ResearchOutput {
    /// Merge candidate metadata into empty fields while preserving existing values.
    pub fn merge(self, candidate: Self) -> Self {
        Self {
            identifier: self.identifier.filter(|value| !value.trim().is_empty()).or(candidate.identifier),
            doi: self.doi.filter(|value| !value.trim().is_empty()).or(candidate.doi),
            title: match self.title.trim().is_empty() {
                | true => candidate.title,
                | false => self.title,
            },
            kind: self.kind.filter(|value| !value.trim().is_empty()).or(candidate.kind),
            publication_year: self.publication_year.or(candidate.publication_year),
            dates: self.dates.filter(|values| !values.is_empty()).or(candidate.dates),
            publisher: self.publisher.or(candidate.publisher),
            resource_type_general: self.resource_type_general.or(candidate.resource_type_general),
            resource_type: self.resource_type.filter(|value| !value.trim().is_empty()).or(candidate.resource_type),
            language: self.language.filter(|value| !value.trim().is_empty()).or(candidate.language),
            alternate_identifiers: self
                .alternate_identifiers
                .filter(|values| !values.is_empty())
                .or(candidate.alternate_identifiers),
            related_identifiers: self
                .related_identifiers
                .filter(|values| !values.is_empty())
                .or(candidate.related_identifiers),
            related_items: self.related_items.filter(|values| !values.is_empty()).or(candidate.related_items),
            version: self.version.filter(|value| !value.trim().is_empty()).or(candidate.version),
            rights_list: self.rights_list.filter(|values| !values.is_empty()).or(candidate.rights_list),
            descriptions: self.descriptions.filter(|values| !values.is_empty()).or(candidate.descriptions),
            subjects: self.subjects.filter(|values| !values.is_empty()).or(candidate.subjects),
            formats: self.formats.filter(|values| !values.is_empty()).or(candidate.formats),
            sizes: self.sizes.filter(|values| !values.is_empty()).or(candidate.sizes),
            geo_locations: self.geo_locations.filter(|values| !values.is_empty()).or(candidate.geo_locations),
            creators: self.creators.filter(|values| !values.is_empty()).or(candidate.creators),
            contributors: self.contributors.filter(|values| !values.is_empty()).or(candidate.contributors),
            funding: self.funding.filter(|values| !values.is_empty()).or(candidate.funding),
            websites: self.websites.filter(|values| !values.is_empty()).or(candidate.websites),
            keywords: match self.keywords.is_empty() {
                | true => candidate.keywords,
                | false => self.keywords,
            },
            access: self.access.or(candidate.access),
        }
    }
}
