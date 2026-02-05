//! Module for interacting with ORCiD API
//!
//! Provides types and functions for constructing ORCiD API queries with field validation and output column selection.
use crate::schema::validate::{is_orcid, is_ror};
use crate::util::Label;
use bon::Builder;
use core::fmt;
use serde::{Deserialize, Serialize};
use tracing::warn;
use urlencoding::encode;

/// URL-encoded space character
const ENCODED_SPACE: &str = "%20";

/// ORCiD allowed search fields
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SearchField {
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
pub enum OutputColumn {
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
/// ORCiD search response
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SearchResponse {
    /// Number of results found
    #[serde(rename = "@num-found")]
    pub num_found: usize,
    /// XML namespace for expanded search
    #[builder(default = "http://www.orcid.org/ns/expanded-search".to_string())]
    #[serde(rename = "@xmlns:expanded-search")]
    pub namespace: String,
    /// List of expanded search results
    #[builder(default)]
    #[serde(rename = "expanded-result", default)]
    pub results: Vec<SearchResult>,
}
/// ORCiD search result
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SearchResult {
    /// [`ORCID`](crate::schema::pid::ORCID) identifier
    #[serde(rename = "orcid-id")]
    pub orcid_id: Option<String>,
    /// Given names (first name(s))
    #[serde(rename = "given-names", skip_serializing_if = "Option::is_none")]
    pub given_names: Option<String>,
    /// Family name (last name or surname)
    #[serde(rename = "family-names", skip_serializing_if = "Option::is_none")]
    pub family_names: Option<String>,
    /// Credit name (preferred display name)
    #[serde(rename = "credit-name", skip_serializing_if = "Option::is_none")]
    pub credit_name: Option<String>,
    /// Email addresses
    #[builder(default)]
    #[serde(rename = "email", default, skip_serializing_if = "Vec::is_empty")]
    pub emails: Vec<String>,
    /// Institution names
    #[builder(default)]
    #[serde(rename = "institution-name", default, skip_serializing_if = "Vec::is_empty")]
    pub institution_names: Vec<String>,
    /// Other names
    #[builder(default)]
    #[serde(rename = "other-name", default, skip_serializing_if = "Vec::is_empty")]
    pub other_name: Vec<String>,
}
impl SearchField {
    /// Validate certain types of ORCiD search field values
    ///
    /// Special validation is performed for `RorOrgId` and `Orcid` fields.
    pub fn is_valid(&self, value: &str) -> bool {
        match self {
            | SearchField::RorOrgId => is_ror(value).is_ok(),
            | SearchField::Orcid => is_orcid(value).is_ok(),
            | _ => true,
        }
    }
}
impl fmt::Display for SearchField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            | SearchField::AffiliationOrgName => "affiliation-org-name",
            | SearchField::CreditName => "credit-name",
            | SearchField::Email => "email",
            | SearchField::ExternalIdReference => "external-id-reference",
            | SearchField::FamilyName => "family-name",
            | SearchField::GivenNames => "given-names",
            | SearchField::Keyword => "keyword",
            | SearchField::Orcid => "orcid",
            | SearchField::OtherNames => "other-names",
            | SearchField::RorOrgId => "ror-org-id",
            | SearchField::Text => "text",
        };
        write!(f, "{}", s)
    }
}
impl fmt::Display for OutputColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            | OutputColumn::CreditName => "credit-name",
            | OutputColumn::CurrentInstitutionAffiliationName => "current-institution-affiliation-name",
            | OutputColumn::Email => "email",
            | OutputColumn::FamilyName => "family-name",
            | OutputColumn::GivenAndFamilyNames => "given-and-family-names",
            | OutputColumn::GivenName => "given-name",
            | OutputColumn::Orcid => "orcid",
            | OutputColumn::OtherName => "other-name",
            | OutputColumn::PastInstitutionAffiliationName => "past-institution-affiliation-name",
        };
        write!(f, "{}", s)
    }
}
impl TryFrom<&str> for SearchField {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            | "affiliation-org-name" => Ok(SearchField::AffiliationOrgName),
            | "credit-name" => Ok(SearchField::CreditName),
            | "email" => Ok(SearchField::Email),
            | "external-id-reference" => Ok(SearchField::ExternalIdReference),
            | "family-name" => Ok(SearchField::FamilyName),
            | "given-names" => Ok(SearchField::GivenNames),
            | "keyword" => Ok(SearchField::Keyword),
            | "orcid" => Ok(SearchField::Orcid),
            | "other-names" => Ok(SearchField::OtherNames),
            | "ror-org-id" => Ok(SearchField::RorOrgId),
            | "text" => Ok(SearchField::Text),
            | _ => Err(format!("Invalid ORCiD search field: {value}")),
        }
    }
}
impl TryFrom<&str> for OutputColumn {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            | "credit-name" => Ok(OutputColumn::CreditName),
            | "current-institution-affiliation-name" => Ok(OutputColumn::CurrentInstitutionAffiliationName),
            | "email" => Ok(OutputColumn::Email),
            | "family-name" => Ok(OutputColumn::FamilyName),
            | "given-and-family-names" => Ok(OutputColumn::GivenAndFamilyNames),
            | "given-name" => Ok(OutputColumn::GivenName),
            | "orcid" => Ok(OutputColumn::Orcid),
            | "other-name" => Ok(OutputColumn::OtherName),
            | "past-institution-affiliation-name" => Ok(OutputColumn::PastInstitutionAffiliationName),
            | _ => Err(format!("Invalid ORCiD output column: {value}")),
        }
    }
}
/// Create an ORCiD API URL query string from a lookup table of key-value pairs and a list of desired output columns
///
/// Optional boosting of certain fields can be specified to prioritize their relevance in search results
// TODO: Support OR statements for field values (e.g., affiliation-org-name:("University of Plymouth" OR "Plymouth University"))
pub fn search_query_string<S>(field_pairs: Vec<(S, S)>, output_columns: Vec<S>, boost: Option<Vec<S>>) -> String
where
    S: AsRef<str>,
{
    let fields = field_pairs
        .into_iter()
        .filter_map(|(k, v)| match SearchField::try_from(k.as_ref()) {
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
        .filter_map(|value| match OutputColumn::try_from(value.as_ref()) {
            | Ok(column) => Some(column.to_string()),
            | Err(_) => None,
        })
        .collect::<Vec<String>>()
        .join(",");
    let boosted = boost
        .map(|fields| {
            let valid_fields = fields
                .into_iter()
                .filter_map(|value| SearchField::try_from(value.as_ref()).ok())
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
