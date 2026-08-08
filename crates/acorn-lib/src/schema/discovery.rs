//! Provider-neutral schema types for remote metadata discovery.
use core::fmt;
use serde::{Deserialize, Serialize};

/// Normalized entity returned by a remote discovery provider
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RemoteEntity {
    /// Research software project
    Project,
    /// Person
    Person,
    /// Organization
    Organization,
    /// Source-code repository
    Repository,
}
/// Provider-neutral relationship between an organization and a discovered entity
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteOrganizationRole {
    /// Any credited relationship
    #[default]
    Any,
    /// Submitting site
    SiteOwner,
    /// Research organization
    Research,
    /// Sponsoring organization
    Sponsor,
    /// Contributing organization
    Contributor,
    /// Developing organization
    Developer,
}
/// Provider-neutral aggregated organization search result
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Organization {
    /// Preferred display name
    pub name: String,
    /// Aliases and site codes
    pub aliases: Vec<String>,
    /// Relationships across matching projects
    pub roles: Vec<String>,
    /// Associated provider-native project identifiers
    pub project_ids: Vec<u64>,
    /// Associated project titles
    pub project_titles: Vec<String>,
}
/// Provider-neutral aggregated person search result.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Person {
    /// Display name
    pub name: String,
    /// ORCID identifier
    pub orcid: Option<String>,
    /// Email address
    pub email: Option<String>,
    /// Affiliations across matching projects
    pub affiliations: Vec<String>,
    /// Roles across matching projects
    pub roles: Vec<String>,
    /// Associated provider-native project identifiers
    pub project_ids: Vec<u64>,
    /// Associated project titles
    pub project_titles: Vec<String>,
}
/// An organization credited by a discovered project
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectOrganization {
    /// Organization display name
    #[serde(default)]
    pub organization_name: String,
    /// Whether the source identifies this as a DOE organization
    #[serde(default, rename = "DOE")]
    pub doe: bool,
    /// Primary award number, when supplied for sponsors
    #[serde(default)]
    pub primary_award: Option<String>,
}
/// A person credited by a discovered project
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectPerson {
    /// Email address.
    #[serde(default)]
    pub email: String,
    /// ORCID identifier.
    #[serde(default)]
    pub orcid: String,
    /// Given name.
    #[serde(default)]
    pub first_name: String,
    /// Family name.
    #[serde(default)]
    pub last_name: String,
    /// Middle name.
    #[serde(default)]
    pub middle_name: String,
    /// Contributor type when this is a contributor entry.
    #[serde(default)]
    pub contributor_type: Option<String>,
    /// Organization affiliations.
    #[serde(default)]
    pub affiliations: Vec<String>,
}
impl ProjectPerson {
    /// Return the person's display name.
    pub fn name(&self) -> String {
        [&self.first_name, &self.middle_name, &self.last_name]
            .into_iter()
            .filter(|part| !part.trim().is_empty())
            .map(|part| part.trim())
            .collect::<Vec<_>>()
            .join(" ")
    }
}
impl fmt::Display for RemoteEntity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::Project => "project",
            | Self::Person => "person",
            | Self::Organization => "organization",
            | Self::Repository => "repository",
        })
    }
}
impl fmt::Display for RemoteOrganizationRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::Any => "any",
            | Self::SiteOwner => "site-owner",
            | Self::Research => "research",
            | Self::Sponsor => "sponsor",
            | Self::Contributor => "contributor",
            | Self::Developer => "developer",
        })
    }
}
