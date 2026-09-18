//! Portable descriptions of compute platforms, facilities, and environments.
//!
//! These values describe an observed configuration. They are distinct from
//! [`ResearchActivityMetadata::resources`](crate::research_activity::ResearchActivityMetadata::resources),
//! which records minimum compute requirements. This module composes hardware
//! resources with standards and OCP authority records without exposing
//! management endpoints, credentials, hostnames, serial numbers, or control
//! operations.
use crate::hardware::Resource;
use crate::standard::ocp::{ManagementProfileClaim, Recognition, RecognitionProgram};
use crate::standard::StandardsClaim;
use crate::types::{deserialize_optional_one_or_many, deserialize_optional_one_or_many_strings};
use crate::validation::{Validate, ValidationReport};
use crate::OneOrMany;
use acorn_core::prelude::alloc::{format, String};
use acorn_core::validation::ValidationError;
use alloc::collections::BTreeSet;
use bon::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_trim::{option_string_trim, string_trim};
use serde_with::skip_serializing_none;

/// Platforms and optional facility observed as one compute environment.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "ComputeEnvironment::is_valid", skip_on_field_errors = false))]
pub struct ComputeEnvironment {
    /// Stable ACORN-local environment identifier.
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub identifier: String,
    /// Human-readable environment name.
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub name: String,
    /// Installed compute platforms.
    #[builder(into)]
    #[validate(nested)]
    pub platforms: OneOrMany<ComputePlatform>,
    /// Facility containing the platforms, when relevant.
    #[validate(nested)]
    pub facility: Option<ComputeFacility>,
    /// OCP Integrated Solution or future environment-scoped recognition.
    #[builder(into)]
    #[validate(nested)]
    #[serde(default, deserialize_with = "deserialize_optional_one_or_many")]
    pub recognitions: Option<OneOrMany<Recognition>>,
    /// RFC 3339 instant at which this configuration was observed.
    #[validate(timestamp)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub observed_at: Option<String>,
}
/// Facility associated with a compute environment.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "ComputeFacility::is_valid", skip_on_field_errors = false))]
pub struct ComputeFacility {
    /// Stable ACORN-local facility identifier.
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub identifier: String,
    /// Human-readable facility name.
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub name: String,
    /// Facility operator.
    #[validate(nonempty)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub operator: Option<String>,
    /// Appropriately coarse facility location.
    #[validate(nonempty)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub location: Option<String>,
    /// Research Organization Registry identifier(s) for the facility operator.
    #[builder(into)]
    #[validate(each(ror))]
    #[serde(default, deserialize_with = "deserialize_optional_one_or_many_strings")]
    pub ror: Option<OneOrMany<String>>,
    /// Standards claims authored for the facility.
    #[builder(into)]
    #[validate(nested)]
    #[serde(default, deserialize_with = "deserialize_optional_one_or_many")]
    pub standards_claims: Option<OneOrMany<StandardsClaim>>,
    /// OCP Ready or future facility-scoped recognition.
    #[builder(into)]
    #[validate(nested)]
    #[serde(default, deserialize_with = "deserialize_optional_one_or_many")]
    pub recognitions: Option<OneOrMany<Recognition>>,
}
/// Configured compute platform and its installed hardware resources.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "ComputePlatform::is_valid", skip_on_field_errors = false))]
pub struct ComputePlatform {
    /// Stable ACORN-local platform identifier.
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub identifier: String,
    /// Human-readable platform name.
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub name: String,
    /// Platform manufacturer.
    #[validate(nonempty)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub manufacturer: Option<String>,
    /// Manufacturer model name.
    #[validate(nonempty)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub model: Option<String>,
    /// Installed hardware resources.
    #[builder(into)]
    #[validate(nested)]
    pub resources: OneOrMany<Resource>,
    /// Standards claims authored for the platform.
    #[builder(into)]
    #[validate(nested)]
    #[serde(default, deserialize_with = "deserialize_optional_one_or_many")]
    pub standards_claims: Option<OneOrMany<StandardsClaim>>,
    /// OCP product or device recognitions.
    #[builder(into)]
    #[validate(nested)]
    #[serde(default, deserialize_with = "deserialize_optional_one_or_many")]
    pub recognitions: Option<OneOrMany<Recognition>>,
    /// Evidence-backed hardware-management profile results.
    #[builder(into)]
    #[validate(nested)]
    #[serde(default, deserialize_with = "deserialize_optional_one_or_many")]
    pub management_profile_claims: Option<OneOrMany<ManagementProfileClaim>>,
}
impl ComputeEnvironment {
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        ValidationReport::empty("platforms", self.platforms.is_empty())
            .invalid_recognition_scopes(&self.recognitions, RecognitionProgram::is_environment_recognition)
            .finish()
    }
}
impl ComputeFacility {
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        ValidationReport::new()
            .duplicate_claims(&self.standards_claims)
            .invalid_recognition_scopes(&self.recognitions, RecognitionProgram::is_facility_recognition)
            .finish()
    }
}
impl ComputePlatform {
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        ValidationReport::empty("resources", self.resources.is_empty())
            .duplicate_claims(&self.standards_claims)
            .invalid_recognition_scopes(&self.recognitions, RecognitionProgram::is_platform_recognition)
            .finish()
    }
}
impl ValidationReport {
    fn duplicate_claims(self, claims: &Option<OneOrMany<StandardsClaim>>) -> Self {
        self.merge(
            "",
            claims.as_ref().map_or_else(ValidationReport::new, |claims| {
                claims
                    .iter()
                    .enumerate()
                    .fold(
                        (BTreeSet::new(), ValidationReport::new()),
                        |(mut identities, mut report), (index, claim)| {
                            let identity = (
                                claim.standard.authority.clone(),
                                claim.standard.identifier.clone(),
                                claim.standard.version.clone(),
                                claim.relationship.clone(),
                            );
                            if !identities.insert(identity) {
                                report.add(
                                    format!("standards_claims[{index}]"),
                                    ValidationError::new("duplicate_standard_claim").with_message("Duplicate standards claim"),
                                );
                            }
                            (identities, report)
                        },
                    )
                    .1
            }),
        )
    }
    fn invalid_recognition_scopes(self, recognitions: &Option<OneOrMany<Recognition>>, accepts: fn(&RecognitionProgram) -> bool) -> Self {
        self.merge(
            "",
            recognitions.as_ref().map_or_else(ValidationReport::new, |recognitions| {
                recognitions
                    .iter()
                    .enumerate()
                    .filter(|(_, recognition)| !accepts(&recognition.program))
                    .fold(ValidationReport::new(), |mut report, (index, _)| {
                        report.add(
                            format!("recognitions[{index}].program"),
                            ValidationError::new("recognition_scope").with_message("Recognition program is not valid for this infrastructure type"),
                        );
                        report
                    })
            }),
        )
    }
}
