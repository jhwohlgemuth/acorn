//! Open Compute Project contribution, recognition, and management-profile records.
//!
//! The types preserve authored authority records and test evidence.
//!
//! They do NOT
//! - Resolve OCP catalogs
//! - Contact Redfish services
//! - Grant recognition
//! - Infer recognition or conformance from hardware vendors and architectures.
use super::{ConformanceResult, ContributionKind, StandardReference};
use crate::validation::{rules, IntoValidationReport, Validate, ValidationReport};
use acorn_core::prelude::alloc::{String, ToString};
use acorn_core::validation::ValidationError;
use bon::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_trim::{option_string_trim, string_trim};
use serde_with::skip_serializing_none;
use strum::EnumIs;

/// OCP recognition program attached to an authoritative record.
#[derive(Clone, Debug, EnumIs, Eq, PartialEq)]
pub enum RecognitionProgram {
    /// OCP Accepted product recognition.
    Accepted,
    /// OCP Inspired product recognition.
    Inspired,
    /// OCP Integrated Solution recognition.
    IntegratedSolution,
    /// A recognition program not represented by another variant.
    Other(String),
    /// OCP Ready facility recognition.
    Ready,
    /// OCP Security Appraisal Framework and Enablement recognition.
    Safe,
}
/// One contribution published by the Open Compute Project.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Contribution {
    /// OCP contribution identifier.
    #[validate(nested)]
    pub identifier: ContributionId,
    /// Human-readable contribution title.
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub title: String,
    /// OCP project or workstream.
    #[validate(nested)]
    pub project: Project,
    /// Contribution kind assigned by OCP.
    #[validate(nested)]
    pub kind: ContributionKind,
    /// Contribution version.
    #[validate(nonempty, version)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub version: Option<String>,
    /// Publication or review status supplied by OCP.
    #[validate(nonempty)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub status: Option<String>,
    /// Canonical OCP contribution record.
    #[validate(uri)]
    #[serde(deserialize_with = "string_trim")]
    pub canonical_uri: String,
    /// Contribution license or license URI.
    #[validate(nonempty)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub license: Option<String>,
}
/// Identifier assigned to an OCP contribution.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ContributionId(#[serde(deserialize_with = "string_trim")] pub String);
/// Evidence-backed result for an OCP hardware-management profile.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "ManagementProfileClaim::is_valid", skip_on_field_errors = false))]
pub struct ManagementProfileClaim {
    /// OCP management profile being evaluated.
    #[validate(nested)]
    pub profile: StandardReference,
    /// Underlying protocol or standard used by the profile.
    #[validate(nested)]
    pub underlying_standard: StandardReference,
    /// Validator implementation identity.
    #[validate(nonempty)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub validator_identity: Option<String>,
    /// Validator implementation version.
    #[validate(nonempty)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub validator_version: Option<String>,
    /// RFC 3339 instant at which the test ran.
    #[validate(timestamp)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub tested_at: Option<String>,
    /// Recorded conformance result.
    pub result: ConformanceResult,
    /// HTTP(S) validator report or authoritative record.
    #[validate(uri)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub evidence_uri: Option<String>,
}
/// Open OCP project or workstream name.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Project(#[serde(deserialize_with = "string_trim")] pub String);
/// Recognition granted by the Open Compute Project.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "Recognition::is_valid", skip_on_field_errors = false))]
pub struct Recognition {
    /// Recognition program.
    #[validate(nested)]
    pub program: RecognitionProgram,
    /// Authority-facing identity of the recognized subject.
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub subject_identifier: String,
    /// Authoritative HTTP(S) recognition record.
    #[validate(uri)]
    #[serde(deserialize_with = "string_trim")]
    pub source_uri: String,
    /// RFC 3339 instant at which recognition was awarded.
    #[validate(timestamp)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub awarded_at: Option<String>,
    /// RFC 3339 instant at which recognition expires.
    #[validate(timestamp)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub expires_at: Option<String>,
}
impl From<&str> for ContributionId {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}
impl From<String> for ContributionId {
    fn from(value: String) -> Self {
        Self(value.trim().to_string())
    }
}
impl Validate for ContributionId {
    fn validate(&self) -> Result<(), ValidationReport> {
        self.0
            .strip_prefix('S')
            .filter(|digits| !digits.is_empty() && digits.chars().all(|character| character.is_ascii_digit()))
            .map(|_| ())
            .ok_or_else(|| ValidationError::new("ocp_contribution_id").with_message("Provide an OCP identifier such as S0806"))
            .map_err(|error| error.into_report(""))
    }
}
impl ManagementProfileClaim {
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        match self.result {
            | ConformanceResult::NotTested => Ok(()),
            | ConformanceResult::Fail | ConformanceResult::Partial | ConformanceResult::Pass => {
                let report = [
                    self.tested_at.is_none().then(|| {
                        ValidationReport::from_error(
                            "tested_at",
                            ValidationError::new("tested_at_required").with_message("Tested results require testedAt"),
                        )
                    }),
                    self.evidence_uri.is_none().then(|| {
                        ValidationReport::from_error(
                            "evidence_uri",
                            ValidationError::new("evidence_required").with_message("Tested results require evidenceUri"),
                        )
                    }),
                ]
                .into_iter()
                .flatten()
                .fold(ValidationReport::new(), |report, issue| report.merge("", issue));
                report.finish()
            }
        }
    }
}
impl From<&str> for Project {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}
impl From<String> for Project {
    fn from(value: String) -> Self {
        Self(value.trim().to_string())
    }
}
impl Validate for Project {
    fn validate(&self) -> Result<(), ValidationReport> {
        rules::nonempty(&self.0).map_err(|error| error.into_report(""))
    }
}
impl Recognition {
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        match (&self.awarded_at, &self.expires_at) {
            | (Some(awarded_at), Some(expires_at)) => match (rules::parse_timestamp(awarded_at), rules::parse_timestamp(expires_at)) {
                | (Ok(awarded_at), Ok(expires_at)) if expires_at <= awarded_at => ValidationError::new("timestamp_order")
                    .with_message("expiresAt must follow awardedAt")
                    .into_report("expires_at")
                    .finish(),
                | _ => Ok(()),
            },
            | _ => Ok(()),
        }
    }
}
impl RecognitionProgram {
    pub(crate) fn is_environment_recognition(&self) -> bool {
        self.is_integrated_solution() || self.is_other()
    }
    pub(crate) fn is_facility_recognition(&self) -> bool {
        self.is_other() || self.is_ready()
    }
    pub(crate) fn is_platform_recognition(&self) -> bool {
        self.is_accepted() || self.is_inspired() || self.is_other() || self.is_safe()
    }
}
impl<'de> Deserialize<'de> for RecognitionProgram {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(Self::from)
    }
}
impl From<String> for RecognitionProgram {
    fn from(value: String) -> Self {
        let value = value.trim().to_string();
        match value.as_str() {
            | "accepted" | "Accepted" => Self::Accepted,
            | "inspired" | "Inspired" => Self::Inspired,
            | "integratedSolution" | "Integrated Solution" => Self::IntegratedSolution,
            | "ready" | "Ready" => Self::Ready,
            | "safe" | "SAFE" | "S.A.F.E." => Self::Safe,
            | _ => Self::Other(value),
        }
    }
}
impl JsonSchema for RecognitionProgram {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        "RecognitionProgram".into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        String::json_schema(generator)
    }
}
impl Serialize for RecognitionProgram {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            | Self::Accepted => "accepted",
            | Self::Inspired => "inspired",
            | Self::IntegratedSolution => "integratedSolution",
            | Self::Other(value) => value,
            | Self::Ready => "ready",
            | Self::Safe => "safe",
        })
    }
}
impl Validate for RecognitionProgram {
    fn validate(&self) -> Result<(), ValidationReport> {
        match self {
            | Self::Other(value) => rules::nonempty(value).map_err(|error| error.into_report("")),
            | _ => Ok(()),
        }
    }
}
impl From<&Contribution> for StandardReference {
    fn from(contribution: &Contribution) -> Self {
        Self {
            authority: "Open Compute Project".to_string(),
            identifier: contribution.identifier.0.clone(),
            title: Some(contribution.title.clone()),
            version: contribution.version.clone(),
            canonical_uri: contribution.canonical_uri.clone(),
        }
    }
}
