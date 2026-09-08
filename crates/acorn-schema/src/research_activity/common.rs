//! Shared metadata primitives for research activity alignment
//!
//! Portable, validated types reused across `ResearchActivity`, `ResearchOutput`, RAiD, DataCite, DCAT and discovery adapters.
//! All types are `alloc`-compatible and perform no I/O. Matrix: `.wiki/assets/standards-field-matrix.md`.
use crate::pid::PID;
use crate::validate::is_license;
use crate::validation::{rules, IntoValidationReport, Validate, ValidationReport};
use acorn_core::prelude::alloc::{String, Vec};
use bon::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_trim::{option_string_trim, string_trim};
use serde_with::skip_serializing_none;

/// Type of date recorded for a research activity.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub enum ActivityDateType {
    /// Date on which the activity began.
    StartDate,
    /// Date on which the activity ended.
    EndDate,
    /// Date on which the activity was created.
    Created,
    /// Date on which the activity was last updated.
    Updated,
    /// Date on which the activity was issued.
    Issued,
    /// Date or interval covered by the activity.
    Coverage,
}
/// Legacy research activity field represented by a related resource.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RelatedResourceCategory {
    /// Resource associated with a sponsor.
    Sponsors,
    /// Resource associated with a partner.
    Partners,
    /// Resource associated through the general related-activity field.
    Related,
}
/// Access condition associated with a rights statement.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RightsAccess {
    /// The resource is openly accessible.
    Open,
    /// The resource is inaccessible until an embargo expires.
    Embargoed,
}
/// Structured activity contributor (person) with identifiers, affiliations and roles.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ActivityContributor {
    /// Display name.
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub name: String,
    /// Agent identifiers (ORCID, ISNI, etc.)
    #[validate(nested)]
    #[builder(default)]
    pub identifiers: Vec<AgentIdentifier>,
    /// Affiliations as typed organization identifiers
    #[validate(nested)]
    #[builder(default)]
    pub affiliations: Vec<TypedIdentifier>,
    /// Dated roles held during the activity
    #[validate(nested)]
    #[builder(default)]
    pub roles: Vec<DatedAgentRole>,
}
/// Structured activity organization with identifiers and roles.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ActivityOrganization {
    /// Organization display name
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub name: String,
    /// Organization identifiers (ROR, etc.)
    #[validate(nested)]
    #[builder(default)]
    pub identifiers: Vec<TypedIdentifier>,
    /// Dated roles within the activity
    #[validate(nested)]
    #[builder(default)]
    pub roles: Vec<DatedOrganizationRole>,
}
/// Agent identifier supporting ORCID, ROR, ISNI and extensible schemes
///
/// Distinct from `TypedIdentifier` to preserve agent-specific validation and to reuse `output::Affiliation` semantics where appropriate.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AgentIdentifier {
    /// Identifier value
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub value: String,
    /// Identifier scheme (e.g. `ORCID`, `ROR`, `ISNI`)
    #[serde(default, deserialize_with = "option_string_trim")]
    pub scheme: Option<String>,
    /// Scheme URI
    #[validate(url)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub scheme_uri: Option<String>,
}
/// Controlled subject with scheme and value URIs
///
/// Maps RAD `Keyword` / `technology` normalized terms to DataCite `Subject` with `subjectScheme`/`valueURI` and RAiD `subject` keywords.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ControlledSubject {
    /// Subject term (keyword).
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub subject: String,
    /// Subject scheme (e.g. `Field of Research`, `ANZSRC`, custom)
    #[serde(default, deserialize_with = "option_string_trim")]
    pub subject_scheme: Option<String>,
    /// Scheme URI
    #[validate(url)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub scheme_uri: Option<String>,
    /// Value URI for the controlled term
    #[validate(url)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub value_uri: Option<String>,
}
/// Dated agent role (contributor with role and active period)
///
/// Maps to RAiD `contributor[]` `role` + `position` date range and DataCite `creators`/`contributors` `contributorType` with `timestamp`
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DatedAgentRole {
    /// Agent identifier (ORCID/ISNI or name-based)
    #[validate(nested)]
    pub agent: AgentIdentifier,
    /// Role identifier (e.g. CRediT `Conceptualization`, DataCite `ContactPerson`, RAiD `Leader`)
    #[serde(default, deserialize_with = "option_string_trim")]
    pub role: Option<String>,
    /// Role scheme URI (e.g. `https://credit.niso.org/`)
    #[validate(url)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub role_scheme_uri: Option<String>,
    /// Active period start (RFC3339 `timestamp` with offset)
    #[validate(timestamp)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub start_date: Option<String>,
    /// Active period end (RFC3339 `timestamp` with offset)
    #[validate(timestamp)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub end_date: Option<String>,
    /// Whether the agent is the designated contact for the activity
    pub contact: Option<bool>,
    /// Whether the agent is the designated leader/PI
    pub leader: Option<bool>,
}
/// Dated organization role (organization with role and active period)
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DatedOrganizationRole {
    /// Organization identifier (typically ROR)
    #[validate(nested)]
    pub organization: TypedIdentifier,
    /// Role (e.g. `LeadResearchOrganisation`, `Funder`)
    #[serde(default, deserialize_with = "option_string_trim")]
    pub role: Option<String>,
    /// Role scheme URI
    #[validate(url)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub role_scheme_uri: Option<String>,
    /// Active period start
    #[validate(timestamp)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub start_date: Option<String>,
    /// Active period end
    #[validate(timestamp)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub end_date: Option<String>,
}
/// Structured rights and access statement
///
/// Covers DataCite `rightsList` / `rights` (`rightsURI`, `rightsIdentifier` SPDX) and RAiD `access` / `license` with `validate_license` parity.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StructuredRights {
    /// Rights statement text
    #[serde(default, deserialize_with = "option_string_trim")]
    pub rights: Option<String>,
    /// Rights URI
    #[validate(url)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub rights_uri: Option<String>,
    /// Rights identifier (e.g. SPDX `CC-BY-4.0`)
    #[validate(custom(function = "is_license"))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub rights_identifier: Option<String>,
    /// Rights identifier scheme (e.g. `SPDX`)
    #[serde(default, deserialize_with = "option_string_trim")]
    pub rights_identifier_scheme: Option<String>,
    /// Scheme URI
    #[validate(url)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub scheme_uri: Option<String>,
    /// Access statement (e.g. `open`, `embargoed`)
    pub access: Option<RightsAccess>,
}
/// Typed activity date with explicit `dateType` semantics.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TypedActivityDate {
    /// Date type (e.g. `StartDate`, `EndDate`, `Created`, `Updated`, `Issued`, `Coverage`).
    pub date_type: ActivityDateType,
    /// RFC3339 timestamp with offset (distinguishes instant vs interval).
    #[validate(timestamp)]
    #[serde(deserialize_with = "string_trim")]
    pub date: String,
    /// Optional free-text information about the date.
    #[serde(default, deserialize_with = "option_string_trim")]
    pub information: Option<String>,
}
/// Typed related resource with identifier, relation, resource type and category
///
/// Unifies DataCite `RelatedIdentifier`/`RelatedItem` for RAD. `relationType` includes `Other` with `relationTypeInformation`.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TypedRelatedResource {
    /// Related resource identifier.
    #[validate(nested)]
    pub identifier: TypedIdentifier,
    /// Relation type (e.g. `IsPartOf`, `References`, `Other`)
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub relation_type: String,
    /// Relation type information for `Other` or extended semantics
    #[serde(default, deserialize_with = "option_string_trim")]
    pub relation_type_information: Option<String>,
    /// General resource type (e.g. `Dataset`, `JournalArticle`, `Poster`, `Project`)
    #[serde(default, deserialize_with = "option_string_trim")]
    pub resource_type_general: Option<String>,
    /// Activity category for related activities (e.g. `sponsors`, `partners`, `related`)
    pub category: Option<RelatedResourceCategory>,
}
/// Typed identifier with value, scheme and scheme URI
///
/// Covers DOI, Handle, RAiD, SWHID, ROR, ORCID, ISNI, ARK etc. without network resolution. See `.wiki/assets/standards-field-matrix.md` identifiers.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "TypedIdentifier::is_valid", skip_on_field_errors = false))]
pub struct TypedIdentifier {
    /// Identifier value (e.g. `10.5072/example`, `swh:1:cnt:...`, `https://ror.org/...`).
    #[validate(nonempty)]
    #[serde(deserialize_with = "string_trim")]
    pub value: String,
    /// Identifier scheme (e.g. `DOI`, `RAiD`, `SWHID`, `ROR`, `ORCID`, `ISNI`).
    #[serde(default, deserialize_with = "option_string_trim")]
    pub scheme: Option<String>,
    /// Scheme URI (e.g. `https://doi.org`, `https://ror.org`).
    #[validate(url)]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub scheme_uri: Option<String>,
}
impl TypedIdentifier {
    pub(crate) fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        match self.scheme.as_deref().map(PID::from) {
            | Some(PID::RRID) => rules::rrid(&self.value).map_err(|error| error.into_report("value")),
            | _ => Ok(()),
        }
    }
}
