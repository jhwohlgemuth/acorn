//! Portable standards records and crosswalks.
use crate::validation::{rules, IntoValidationReport, Validate, ValidationReport};
use acorn_core::prelude::alloc::{String, ToString};
use acorn_core::validation::ValidationError;
use bon::Builder;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_trim::{option_string_trim, string_trim};
use serde_with::skip_serializing_none;

pub mod cff;
pub mod crosswalk;
pub mod datacite;
#[cfg(feature = "std")]
pub mod dcat;
#[cfg(feature = "std")]
pub mod dublin_core;
#[cfg(feature = "std")]
pub mod huwise;
pub mod invenio;
pub mod ocp;
#[cfg(all(test, feature = "std"))]
mod tests;
pub mod text;
pub mod zon;

/// How evidence supports a standards claim.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ClaimAssessment {
    /// The subject author declares the relationship.
    Declared,
    /// The relationship was inferred by an identified process.
    Inferred,
    /// Evidence independently verifies the relationship.
    Verified,
}
/// Result of testing a subject for conformance.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConformanceResult {
    /// The tested subject did not conform.
    Fail,
    /// No conformance test was performed.
    NotTested,
    /// The tested subject conformed only in part.
    Partial,
    /// The tested subject conformed.
    Pass,
}
/// Kind assigned to a standards contribution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContributionKind {
    /// Design files.
    DesignFiles,
    /// A contribution kind not represented by another variant.
    Other(String),
    /// Reference architecture.
    ReferenceArchitecture,
    /// Requirements document.
    RequirementsDocument,
    /// Software contribution.
    Software,
    /// Specification.
    Specification,
    /// Tested configuration.
    TestedConfiguration,
    /// White paper.
    WhitePaper,
}
/// Relationship between a subject and a referenced standard or contribution.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StandardsRelationship {
    /// The subject is based on the reference.
    BasedOn,
    /// The subject conforms to the reference.
    ConformsTo,
    /// The subject implements the reference.
    Implements,
    /// The subject references the standard without asserting conformance.
    References,
    /// The subject was tested against the reference.
    TestedAgainst,
}
/// Stable identity for a standard, specification, or authoritative contribution.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StandardReference {
    /// Organization or authority responsible for the reference.
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub authority: String,
    /// Authority-assigned identifier.
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub identifier: String,
    /// Human-readable title.
    #[validate(nonempty)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub title: Option<String>,
    /// Version asserted by the authority.
    #[validate(nonempty)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub version: Option<String>,
    /// Canonical HTTP(S) record URI.
    #[validate(uri)]
    #[serde(deserialize_with = "string_trim")]
    pub canonical_uri: String,
}
/// Authored relationship between a subject and a standard reference.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "StandardsClaim::is_valid", skip_on_field_errors = false))]
pub struct StandardsClaim {
    /// Referenced standard or authoritative contribution.
    #[validate(nested)]
    pub standard: StandardReference,
    /// Relationship asserted by the claim author.
    pub relationship: StandardsRelationship,
    /// Strength and provenance of the assertion.
    pub assessment: ClaimAssessment,
    /// HTTP(S) evidence supporting the assertion.
    #[validate(uri)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub evidence_uri: Option<String>,
}
impl StandardsClaim {
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        match (&self.assessment, &self.evidence_uri) {
            | (ClaimAssessment::Verified, None) => ValidationError::new("evidence_required")
                .with_message("Verified standards claims require evidenceUri")
                .into_report("evidence_uri")
                .finish(),
            | _ => Ok(()),
        }
    }
}
impl<'de> Deserialize<'de> for ContributionKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(Self::from)
    }
}
impl From<String> for ContributionKind {
    fn from(value: String) -> Self {
        let value = value.trim().to_string();
        match value.as_str() {
            | "designFiles" | "Design Files" => Self::DesignFiles,
            | "referenceArchitecture" | "Reference Architecture" => Self::ReferenceArchitecture,
            | "requirementsDocument" | "Requirements Document" => Self::RequirementsDocument,
            | "software" | "Software" => Self::Software,
            | "specification" | "Specification" => Self::Specification,
            | "testedConfiguration" | "Tested Configuration" => Self::TestedConfiguration,
            | "whitePaper" | "White Paper" => Self::WhitePaper,
            | _ => Self::Other(value),
        }
    }
}
impl JsonSchema for ContributionKind {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        "ContributionKind".into()
    }
    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        String::json_schema(generator)
    }
}
impl Serialize for ContributionKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            | Self::DesignFiles => "designFiles",
            | Self::Other(value) => value,
            | Self::ReferenceArchitecture => "referenceArchitecture",
            | Self::RequirementsDocument => "requirementsDocument",
            | Self::Software => "software",
            | Self::Specification => "specification",
            | Self::TestedConfiguration => "testedConfiguration",
            | Self::WhitePaper => "whitePaper",
        })
    }
}
impl Validate for ContributionKind {
    fn validate(&self) -> Result<(), ValidationReport> {
        match self {
            | Self::Other(value) => rules::nonempty(value).map_err(|error| error.into_report("")),
            | _ => Ok(()),
        }
    }
}
