//! Portable standards records and crosswalks.
use crate::{
    codec::CodecDiagnostic,
    util::constants::app::DCAT_NAMESPACE,
    validation::{rules, IntoValidationReport, Validate, ValidationReport},
};
use acorn_core::validation::ValidationError;
use acorn_core::{
    prelude::alloc::{String, ToString, Vec},
    util::MimeType,
};
use bon::Builder;
use derive_more::Display;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use serde_trim::{option_string_trim, string_trim};
use serde_with::skip_serializing_none;

pub mod cff;
pub mod ckan;
pub mod crosswalk;
pub mod datacite;
#[cfg(feature = "std")]
pub mod dcat;
pub mod dublin_core;
#[cfg(feature = "std")]
pub mod huwise;
pub mod invenio;
pub mod ocp;
pub mod rdf;
pub mod relation;
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
/// Metadata standard used to gate parsing, validation, conversion, and schema export.
#[derive(Clone, Copy, Debug, Default, Deserialize, Display, Eq, JsonSchema, PartialEq, Serialize, strum::EnumIs)]
#[serde(rename_all = "kebab-case")]
pub enum Standard {
    /// Application Configuration for Research Enablement (ACRE).
    #[display("application-configuration-research-enablement")]
    ApplicationConfigurationResearchEnablement,
    /// Citation File Format (CFF).
    #[display("citation-file-format")]
    CitationFileFormat,
    /// Comprehensive Knowledge Archive Network (CKAN).
    #[display("ckan")]
    Ckan,
    /// DataCite.
    #[display("datacite")]
    Datacite,
    /// Data Catalog Vocabulary (DCAT).
    #[display("dcat")]
    Dcat,
    /// Document-derived text.
    #[display("document")]
    Document,
    /// Dublin Core.
    #[display("dublin-core")]
    DublinCore,
    /// HuWise.
    #[display("huwise")]
    Huwise,
    /// InvenioRDM.
    #[display("invenio")]
    Invenio,
    /// KitOps authoring Kitfile.
    #[display("kitfile")]
    Kitfile,
    /// Resolved KitOps ModelKit configuration.
    #[display("modelkit")]
    ModelKit,
    /// Open Container Initiative model artifact profile.
    #[display("oci")]
    Oci,
    /// Research Activity Identifier metadata (RAiD).
    #[display("raid")]
    Raid,
    /// Research Activity Data (RAD).
    #[default]
    #[display("research-activity-data")]
    ResearchActivityData,
    /// Plain text.
    #[display("text")]
    Text,
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
impl<'de> Deserialize<'de> for ContributionKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(Self::from)
    }
}
impl From<&Map<String, Value>> for Standard {
    fn from(object: &Map<String, Value>) -> Self {
        Self::detect_object(object).unwrap_or(Self::Dcat)
    }
}
impl Standard {
    /// Strictly detect a supported metadata standard from a JSON object.
    pub fn detect_object(object: &Map<String, Value>) -> Option<Self> {
        if object.contains_key("cff-version") {
            Some(Self::CitationFileFormat)
        } else if object.contains_key("success") && is_ckan(object) {
            Some(Self::Ckan)
        } else if object.get("@type").is_some_and(is_dcat_type) {
            Some(Self::Dcat)
        } else if is_dublin_core(object) {
            Some(Self::DublinCore)
        } else if object.contains_key("dataset_id") && object.contains_key("metas") {
            Some(Self::Huwise)
        } else if object.contains_key("attributes") && object.contains_key("id") {
            Some(Self::Datacite)
        } else if object.contains_key("manifestVersion") && object.values().any(contains_layer_identity) {
            Some(Self::ModelKit)
        } else if object.contains_key("manifestVersion") {
            Some(Self::Kitfile)
        } else if object.contains_key("contact") && object.contains_key("meta") && object.contains_key("title") {
            Some(Self::ResearchActivityData)
        } else if object.contains_key("access") && object.contains_key("contributor") && object.contains_key("date") && object.contains_key("title") {
            Some(Self::Raid)
        } else if object.contains_key("metadata") || object.contains_key("pids") || object.contains_key("resource_type") {
            Some(Self::Invenio)
        } else if is_ckan(object) {
            Some(Self::Ckan)
        } else {
            None
        }
    }
    /// Strictly detect a supported metadata standard from an RDF graph.
    pub fn detect_rdf(content: &str, format: &MimeType) -> Result<Option<Self>, Vec<CodecDiagnostic>> {
        rdf::decode(content, format).map(|document| {
            let terms = document
                .descriptions
                .iter()
                .flat_map(|description| description.types.iter().chain(description.properties.keys()));
            let mut terms = terms.peekable();
            if terms.clone().any(|value| value.starts_with(DCAT_NAMESPACE)) {
                Some(Self::Dcat)
            } else if terms.any(|value| dublin_core::term(value).is_some()) {
                Some(Self::DublinCore)
            } else {
                None
            }
        })
    }
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
fn contains_layer_identity(value: &Value) -> bool {
    match value {
        | Value::Object(object) => object.contains_key("digest") || object.contains_key("diffId") || object.values().any(contains_layer_identity),
        | Value::Array(values) => values.iter().any(contains_layer_identity),
        | _ => false,
    }
}
fn is_ckan(object: &Map<String, Value>) -> bool {
    let package = object.contains_key("id")
        && object.contains_key("name")
        && object.contains_key("title")
        && ["metadata_created", "metadata_modified", "resources", "tags"]
            .into_iter()
            .any(|field| object.contains_key(field));
    package || object.get("success").and_then(Value::as_bool).is_some() && object.get("result").and_then(Value::as_object).is_some_and(is_ckan)
}
fn is_dcat_type(value: &Value) -> bool {
    match value {
        | Value::Array(values) => values.iter().any(is_dcat_type),
        | Value::String(value) => value.contains("dcat:") || value.contains(DCAT_NAMESPACE),
        | _ => false,
    }
}
fn is_dublin_core(object: &Map<String, Value>) -> bool {
    let dcmi_key = |key: &str| {
        key.starts_with("dc:")
            || key.starts_with("dcterms:")
            || [
                dublin_core::DCAM_NAMESPACE,
                dublin_core::DCTERMS_NAMESPACE,
                dublin_core::ELEMENTS_NAMESPACE,
                dublin_core::TYPE_NAMESPACE,
            ]
            .into_iter()
            .any(|namespace| key.starts_with(namespace))
    };
    object.keys().any(|key| dcmi_key(key))
        || object.get("descriptions").and_then(Value::as_array).is_some_and(|descriptions| {
            descriptions.iter().any(|description| {
                description
                    .get("properties")
                    .and_then(Value::as_object)
                    .is_some_and(|properties| properties.keys().any(|key| dcmi_key(key)))
            })
        })
        || object
            .get("@context")
            .and_then(Value::as_object)
            .is_some_and(|context| context.values().filter_map(Value::as_str).any(dcmi_key))
}

#[cfg(all(test, feature = "std"))]
mod tests;
