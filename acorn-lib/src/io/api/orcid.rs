//! Module for interacting with ORCiD API
//!
//! Provides types and functions for constructing ORCiD API queries with field validation and output column selection.
use crate::schema::validate::{is_orcid, is_ror};
use crate::util::Label;
use core::fmt;
use serde::{Deserialize, Serialize};
use tracing::warn;
use urlencoding::encode;

/// URL-encoded space character
const ENCODED_SPACE: &str = "%20";

/// ORCiD allowed search fields
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrcidSearchField {
    /// Affiliation organization name
    /// ### Example
    /// > "Oak Ridge National Laboratory"
    AffiliationOrgName,
    /// Preferred form or display name, which can differ from legal or given/family names
    CreditName,
    /// Email address
    Email,
    /// External ID reference
    ExternalIdReference,
    /// Family name (i.e., last name or surname)
    FamilyName,
    /// Given names (i.e., first name(s))
    GivenNames,
    /// Keyword
    Keyword,
    /// [`ORCID`](crate::schema::pid::ORCID) identifier
    /// ### Examples
    /// - `0000-0002-2057-9115` (Jason Wohlgemuth)
    /// - `0009-0005-5568-6526` (Audrey Carson)
    Orcid,
    /// Other names
    OtherNames,
    /// [ROR](https://ror.org) organization ID
    /// ### Examples
    /// - <https://ror.org/01qz5mb56> (Oak Ridge National Laboratory)
    /// - <https://ror.org/05p915b28> (Oak Ridge Leadership Computing Facility)
    RorOrgId,
    /// Text field that contains all of the other fields
    Text,
}
/// ORCiD allowed output columns
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrcidOutputColumn {
    /// Email address
    Email,
    /// Credit name
    CreditName,
    /// Current institution affiliation name
    CurrentInstitutionAffiliationName,
    /// Given and family names
    GivenAndFamilyNames,
    /// Given name
    GivenName,
    /// Family name
    FamilyName,
    /// [`ORCID`](crate::schema::pid::ORCID) identifier
    Orcid,
    /// Other name
    OtherName,
    /// Past institution affiliation name
    PastInstitutionAffiliationName,
}

impl OrcidSearchField {
    /// Validate certain types of ORCiD search field values
    ///
    /// Special validation is performed for `RorOrgId` and `Orcid` fields.
    pub fn is_valid(&self, value: &str) -> bool {
        match self {
            | OrcidSearchField::RorOrgId => is_ror(value).is_ok(),
            | OrcidSearchField::Orcid => is_orcid(value).is_ok(),
            | _ => true,
        }
    }
}
impl fmt::Display for OrcidSearchField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            | OrcidSearchField::AffiliationOrgName => "affiliation-org-name",
            | OrcidSearchField::CreditName => "credit-name",
            | OrcidSearchField::Email => "email",
            | OrcidSearchField::ExternalIdReference => "external-id-reference",
            | OrcidSearchField::FamilyName => "family-name",
            | OrcidSearchField::GivenNames => "given-names",
            | OrcidSearchField::Keyword => "keyword",
            | OrcidSearchField::Orcid => "orcid",
            | OrcidSearchField::OtherNames => "other-names",
            | OrcidSearchField::RorOrgId => "ror-org-id",
            | OrcidSearchField::Text => "text",
        };
        write!(f, "{}", s)
    }
}
impl fmt::Display for OrcidOutputColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            | OrcidOutputColumn::CreditName => "credit-name",
            | OrcidOutputColumn::CurrentInstitutionAffiliationName => "current-institution-affiliation-name",
            | OrcidOutputColumn::Email => "email",
            | OrcidOutputColumn::FamilyName => "family-name",
            | OrcidOutputColumn::GivenAndFamilyNames => "given-and-family-names",
            | OrcidOutputColumn::GivenName => "given-name",
            | OrcidOutputColumn::Orcid => "orcid",
            | OrcidOutputColumn::OtherName => "other-name",
            | OrcidOutputColumn::PastInstitutionAffiliationName => "past-institution-affiliation-name",
        };
        write!(f, "{}", s)
    }
}
impl TryFrom<&str> for OrcidSearchField {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            | "affiliation-org-name" => Ok(OrcidSearchField::AffiliationOrgName),
            | "credit-name" => Ok(OrcidSearchField::CreditName),
            | "email" => Ok(OrcidSearchField::Email),
            | "external-id-reference" => Ok(OrcidSearchField::ExternalIdReference),
            | "family-name" => Ok(OrcidSearchField::FamilyName),
            | "given-names" => Ok(OrcidSearchField::GivenNames),
            | "keyword" => Ok(OrcidSearchField::Keyword),
            | "orcid" => Ok(OrcidSearchField::Orcid),
            | "other-names" => Ok(OrcidSearchField::OtherNames),
            | "ror-org-id" => Ok(OrcidSearchField::RorOrgId),
            | "text" => Ok(OrcidSearchField::Text),
            | _ => Err(format!("Invalid ORCiD search field: {value}")),
        }
    }
}
impl TryFrom<&str> for OrcidOutputColumn {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            | "credit-name" => Ok(OrcidOutputColumn::CreditName),
            | "current-institution-affiliation-name" => Ok(OrcidOutputColumn::CurrentInstitutionAffiliationName),
            | "email" => Ok(OrcidOutputColumn::Email),
            | "family-name" => Ok(OrcidOutputColumn::FamilyName),
            | "given-and-family-names" => Ok(OrcidOutputColumn::GivenAndFamilyNames),
            | "given-name" => Ok(OrcidOutputColumn::GivenName),
            | "orcid" => Ok(OrcidOutputColumn::Orcid),
            | "other-name" => Ok(OrcidOutputColumn::OtherName),
            | "past-institution-affiliation-name" => Ok(OrcidOutputColumn::PastInstitutionAffiliationName),
            | _ => Err(format!("Invalid ORCiD output column: {value}")),
        }
    }
}
/// Create an ORCiD API URL query string from a lookup table of key-value pairs and a list of desired output columns
///
/// Optional boosting of certain fields can be specified to prioritize their relevance in search results
// TODO: Support OR statements for field values (e.g., affiliation-org-name:("University of Plymouth" OR "Plymouth University"))
pub fn query<S>(field_pairs: Vec<(S, S)>, output_columns: Vec<S>, boost: Option<Vec<S>>) -> String
where
    S: AsRef<str>,
{
    let fields = field_pairs
        .into_iter()
        .filter_map(|(k, v)| match OrcidSearchField::try_from(k.as_ref()) {
            | Ok(field) => {
                let value = v.as_ref().trim();
                if field.is_valid(value) {
                    Some(format!("{field}:{}", encode(value)))
                } else {
                    warn!("=> {} Invalid value {value} for field {field}", Label::fail());
                    None
                }
            }
            | Err(_) => None,
        })
        .collect::<Vec<String>>()
        .join("+AND+");
    let columns = output_columns
        .into_iter()
        .filter_map(|value| match OrcidOutputColumn::try_from(value.as_ref()) {
            | Ok(column) => Some(column.to_string()),
            | Err(_) => None,
        })
        .collect::<Vec<String>>()
        .join(",");
    let boosted = boost
        .map(|fields| {
            let valid_fields = fields
                .into_iter()
                .filter_map(|value| OrcidSearchField::try_from(value.as_ref()).ok())
                .collect::<Vec<_>>();
            let count = valid_fields.len();
            valid_fields
                .into_iter()
                .enumerate()
                .map(|(i, field)| format!("{field}^{}.0", count + 1 - i))
                .collect::<Vec<String>>()
                .join(ENCODED_SPACE)
        })
        .filter(|s| !s.is_empty());
    if fields.is_empty() {
        String::new()
    } else {
        let mut parts = vec![format!("?q={fields}")];
        if !columns.is_empty() {
            parts.push(format!("fl={columns}"));
        }
        if let Some(boost_str) = boosted {
            parts.push(format!("qf={boost_str}"));
        }
        parts.join("&")
    }
}
