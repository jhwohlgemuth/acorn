//! Search the public [DOE CODE API](https://www.osti.gov/doecodeapi/services/docs/search).
use crate::io::api::{ApiResult, Param, Params, RemoteResource, INCLUDED_ENDPOINTS};
pub use crate::schema::discovery::{Organization, Person, ProjectOrganization, ProjectPerson};
use crate::schema::discovery::{RemoteEntity, RemoteOrganizationRole};
use crate::schema::pid::{Identifier, PersistentIdentifier, PersistentIdentifierParse, DOI, ORCID, PID};
use crate::schema::research_activity::{ResearchActivity, ResearchActivityMetadata};
use crate::util::Searchable;
use alloc::collections::{BTreeMap, BTreeSet};
use color_eyre::eyre::eyre;
use core::fmt;
use futures::{stream, TryStreamExt};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

enum Query {
    Term(String),
    Doi(String),
    Orcid(String),
}
/// Typed DOE CODE results.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "view", content = "items", rename_all = "lowercase")]
pub enum SearchResults {
    /// Project results.
    Projects(Vec<Project>),
    /// Person results.
    People(Vec<Person>),
    /// Organization results.
    Organizations(Vec<Organization>),
}
/// Kind of DOE CODE entity returned by a search.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchView {
    /// DOE CODE software projects.
    #[default]
    Projects,
    /// Developers and contributors projected from matching projects.
    People,
    /// Organizations projected from matching projects.
    Organizations,
}
/// Link attached to a DOE CODE project.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Link {
    /// Link relationship.
    #[serde(default)]
    pub rel: String,
    /// Link target.
    #[serde(default)]
    pub href: String,
}
/// Options for one DOE CODE search.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Options {
    /// Search expression. Empty is allowed when `organization` is present.
    pub query: String,
    /// Entity view to return.
    pub view: SearchView,
    /// Optional organization filter.
    pub organization: Option<String>,
    /// Organization relationship to match.
    pub organization_role: RemoteOrganizationRole,
    /// Number of upstream project records per request.
    pub limit: usize,
    /// Zero-based upstream project start.
    pub start: usize,
    /// Retrieve every upstream page.
    pub all: bool,
}
/// Public DOE CODE software record.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Project {
    /// DOE CODE identifier.
    pub code_id: u64,
    /// Submitting site code.
    #[serde(default)]
    pub site_ownership_code: String,
    /// Software title.
    #[serde(default)]
    pub software_title: String,
    /// Software DOI.
    #[serde(default)]
    pub doi: Option<String>,
    /// Release date.
    #[serde(default)]
    pub release_date: Option<String>,
    /// Project description.
    #[serde(default)]
    pub description: Option<String>,
    /// Source repository URL.
    #[serde(default)]
    pub repository_link: Option<String>,
    /// Landing-page URL.
    #[serde(default)]
    pub landing_page: Option<String>,
    /// Developers.
    #[serde(default)]
    pub developers: Vec<ProjectPerson>,
    /// Contributors.
    #[serde(default)]
    pub contributors: Vec<ProjectPerson>,
    /// Research organizations.
    #[serde(default)]
    pub research_organizations: Vec<ProjectOrganization>,
    /// Sponsoring organizations.
    #[serde(default)]
    pub sponsoring_organizations: Vec<ProjectOrganization>,
    /// Contributing organizations.
    #[serde(default)]
    pub contributing_organizations: Vec<ProjectOrganization>,
    /// Developing organizations.
    #[serde(default)]
    pub developing_organizations: Vec<ProjectOrganization>,
    /// Related links.
    #[serde(default)]
    pub links: Vec<Link>,
}
#[derive(Clone, Debug, Deserialize)]
struct RawSearchResponse {
    num_found: usize,
    start: usize,
    #[serde(default)]
    docs: Vec<Project>,
}
#[derive(Clone, Debug, Deserialize)]
struct RecordResponse {
    metadata: Project,
}
/// DOE CODE search results and pagination information.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchResponse {
    /// Total matching upstream projects.
    pub project_total: usize,
    /// Upstream project offset.
    pub offset: usize,
    /// Whether another upstream page is available.
    pub has_more: bool,
    /// Typed results.
    pub results: SearchResults,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            query: String::new(),
            view: SearchView::Projects,
            organization: None,
            organization_role: RemoteOrganizationRole::Any,
            limit: 20,
            start: 0,
            all: false,
        }
    }
}
impl Query {
    fn from_value(value: &str) -> ApiResult<Self> {
        let trimmed = value.trim();
        let normalized = Identifier::new(trimmed).normalize();
        match normalized {
            | Some(identifier) if identifier.kind == PID::DOI => Ok(Self::Doi(DOI::format(trimmed))),
            | Some(identifier) if identifier.kind == PID::ORCID => {
                let orcid = ORCID::from_string(trimmed);
                Ok(Self::Orcid(orcid.identifier()))
            }
            | _ if trimmed.to_ascii_lowercase().starts_with("doi:") || trimmed.to_ascii_lowercase().contains("doi.org/") => {
                Err(eyre!("Invalid DOI search value: {trimmed}"))
            }
            | _ if trimmed.to_ascii_lowercase().starts_with("orcid:") || trimmed.to_ascii_lowercase().contains("orcid.org/") => {
                Err(eyre!("Invalid ORCID search value: {trimmed}"))
            }
            | _ => Ok(Self::Term(trimmed.to_string())),
        }
    }
    fn pair(&self, view: SearchView) -> (&'static str, &str) {
        match self {
            | Self::Doi(value) => ("identifiers", value),
            | Self::Orcid(value) => ("orcid", value),
            | Self::Term(value) if view == SearchView::People => ("developers_contributors", value),
            | Self::Term(value) => ("all_fields", value),
        }
    }
}
impl From<Project> for ResearchActivity {
    fn from(value: Project) -> Self {
        let Project {
            code_id,
            description,
            doi,
            software_title,
            ..
        } = value;
        let title = match software_title.trim().is_empty() {
            | true => format!("DOE CODE project {code_id}"),
            | false => software_title,
        };
        let meta = ResearchActivityMetadata::init()
            .identifier(format!("osti-{code_id}"))
            .maybe_doi(doi.map(|identifier| vec![identifier]))
            .build();
        ResearchActivity::init().title(title).maybe_subtitle(description).meta(meta).build()
    }
}
impl fmt::Display for SearchView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::Projects => "projects",
            | Self::People => "people",
            | Self::Organizations => "organizations",
        })
    }
}
impl From<RemoteEntity> for SearchView {
    fn from(value: RemoteEntity) -> Self {
        match value {
            | RemoteEntity::Project | RemoteEntity::Repository => Self::Projects,
            | RemoteEntity::Person => Self::People,
            | RemoteEntity::Organization => Self::Organizations,
        }
    }
}
fn acronym(value: &str) -> Option<String> {
    value.match_indices('(').find_map(|(index, _)| {
        value.get(index.saturating_add(1)..).and_then(|suffix| {
            let candidate = suffix.split_once(')').map_or(suffix, |(candidate, _)| candidate).trim();
            (!candidate.is_empty() && candidate.len() <= 12 && candidate.chars().all(|character| character.is_ascii_alphanumeric()))
                .then(|| candidate.to_ascii_uppercase())
        })
    })
}
fn matches_organization(project: &Project, filter: Option<&str>, role: RemoteOrganizationRole) -> bool {
    filter.is_none_or(|filter| {
        let needle = normalized(filter);
        organization_values(project, role)
            .iter()
            .any(|(value, _)| normalized(value).contains(&needle))
    })
}
fn normalized(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
fn organizations_with_role(
    organizations: &[ProjectOrganization],
    role: RemoteOrganizationRole,
) -> impl Iterator<Item = (String, RemoteOrganizationRole)> + '_ {
    organizations
        .iter()
        .map(move |organization| (organization.organization_name.clone(), role))
}
fn organization_values(project: &Project, role: RemoteOrganizationRole) -> Vec<(String, RemoteOrganizationRole)> {
    let site = (!project.site_ownership_code.is_empty())
        .then(|| (project.site_ownership_code.clone(), RemoteOrganizationRole::SiteOwner))
        .into_iter();
    let values = site
        .chain(organizations_with_role(&project.research_organizations, RemoteOrganizationRole::Research))
        .chain(organizations_with_role(
            &project.sponsoring_organizations,
            RemoteOrganizationRole::Sponsor,
        ))
        .chain(organizations_with_role(
            &project.contributing_organizations,
            RemoteOrganizationRole::Contributor,
        ))
        .chain(organizations_with_role(
            &project.developing_organizations,
            RemoteOrganizationRole::Developer,
        ))
        .filter(|(name, _)| !name.is_empty());
    values
        .filter(|(_, value_role)| role == RemoteOrganizationRole::Any || role == *value_role)
        .collect()
}
fn organizations(projects: &[Project], role: RemoteOrganizationRole, filter: Option<&str>) -> Vec<Organization> {
    let needle = filter.map(normalized);
    projects
        .iter()
        .flat_map(|project| organization_values(project, role).into_iter().map(move |value| (project, value)))
        .filter(|(_, (name, _))| needle.as_ref().is_none_or(|needle| normalized(name).contains(needle)))
        .fold(BTreeMap::<String, Organization>::new(), |mut values, (project, (name, value_role))| {
            let key = acronym(&name).map_or_else(|| normalized(&name), |value| normalized(&value));
            let entry = values.entry(key.clone()).or_insert_with(|| Organization {
                name: name.clone(),
                ..Organization::default()
            });
            if name.len() > entry.name.len() {
                entry.name.clone_from(&name);
            }
            entry.aliases.extend([name, key]);
            entry.roles.push(value_role.to_string());
            entry.project_ids.push(project.code_id);
            entry.project_titles.push(project.software_title.clone());
            values
        })
        .into_values()
        .map(|organization| Organization {
            aliases: sorted_unique(organization.aliases),
            project_ids: sorted_unique(organization.project_ids),
            project_titles: sorted_unique(organization.project_titles),
            roles: sorted_unique(organization.roles),
            ..organization
        })
        .collect()
}
async fn page(options: &Options) -> ApiResult<RawSearchResponse> {
    match INCLUDED_ENDPOINTS.find_by_name("osti") {
        | Some(endpoint) => match request_params(options) {
            | Ok(params) => {
                let response = endpoint.invoke("search", Some(params)).await;
                endpoint.handle::<RawSearchResponse>(response)
            }
            | Err(why) => Err(why),
        },
        | None => Err(eyre!("OSTI API endpoint not found")),
    }
}
fn people(projects: &[Project]) -> Vec<Person> {
    let people = projects.iter().flat_map(|project| {
        project
            .developers
            .iter()
            .map(move |person| (project, person, "developer".to_string()))
            .chain(project.contributors.iter().map(move |person| {
                let role = person.contributor_type.clone().unwrap_or_else(|| "contributor".to_string());
                (project, person, role)
            }))
    });
    people
        .fold(BTreeMap::<String, Person>::new(), |mut values, (project, person, role)| {
            let entry = values.entry(person_key(person)).or_insert_with(|| Person {
                name: person.name(),
                orcid: (!person.orcid.trim().is_empty()).then(|| ORCID::from_string(&person.orcid).identifier()),
                email: (!person.email.trim().is_empty()).then(|| person.email.clone()),
                ..Person::default()
            });
            entry.affiliations.extend(person.affiliations.clone());
            entry.roles.push(role);
            entry.project_ids.push(project.code_id);
            entry.project_titles.push(project.software_title.clone());
            values
        })
        .into_values()
        .map(|person| Person {
            affiliations: sorted_unique(person.affiliations),
            project_ids: sorted_unique(person.project_ids),
            project_titles: sorted_unique(person.project_titles),
            roles: sorted_unique(person.roles),
            ..person
        })
        .collect()
}
fn person_key(person: &ProjectPerson) -> String {
    let orcid = ORCID::from_string(&person.orcid).identifier();
    match (orcid.is_empty(), person.email.trim().is_empty()) {
        | (false, _) => format!("orcid:{orcid}"),
        | (true, false) => format!("email:{}", person.email.trim().to_ascii_lowercase()),
        | (true, true) => format!("name:{}:{}", normalized(&person.name()), normalized(&person.affiliations.join(" "))),
    }
}
async fn project_pages(options: &Options, first: RawSearchResponse) -> ApiResult<(usize, usize, Vec<Project>)> {
    let next_start = first.start.saturating_add(first.docs.len());
    match options.all {
        | false => Ok((first.num_found, next_start, first.docs)),
        | true => {
            let seen = first.docs.iter().map(|project| project.code_id).collect::<BTreeSet<_>>();
            stream::try_unfold((next_start, first.num_found, seen), |(start, project_total, seen)| async move {
                match start >= project_total {
                    | true => Ok(None),
                    | false => match page(&Options { start, ..options.clone() }).await {
                        | Err(why) => Err(why),
                        | Ok(response) => {
                            let response_start = response.start;
                            let response_total = response.num_found;
                            let response_length = response.docs.len();
                            let docs = response
                                .docs
                                .into_iter()
                                .filter(|project| !seen.contains(&project.code_id))
                                .unique_by(|project| project.code_id)
                                .collect::<Vec<_>>();
                            let next_seen = seen.into_iter().chain(docs.iter().map(|project| project.code_id)).collect();
                            let next_start = match docs.is_empty() {
                                | true => response_total,
                                | false => response_start.saturating_add(response_length),
                            };
                            let response = RawSearchResponse {
                                num_found: response_total,
                                start: response_start,
                                docs,
                            };
                            Ok(Some((response, (next_start, response_total, next_seen))))
                        }
                    },
                }
            })
            .try_collect::<Vec<_>>()
            .await
            .map(|remaining| {
                let project_total = remaining.last().map_or(first.num_found, |response| response.num_found);
                let final_start = remaining
                    .last()
                    .map_or(next_start, |response| response.start.saturating_add(response.docs.len()));
                let projects = core::iter::once(first)
                    .chain(remaining)
                    .flat_map(|response| response.docs)
                    .unique_by(|project| project.code_id)
                    .collect();
                (project_total, final_start, projects)
            })
        }
    }
}
fn request_params(options: &Options) -> ApiResult<Vec<Param>> {
    match (options.query.trim().is_empty(), options.organization.as_deref()) {
        | (false, _) => Ok(options.query.as_str()),
        | (true, Some(value)) if !value.trim().is_empty() => Ok(value),
        | _ => Err(eyre!("OSTI search requires a query or organization")),
    }
    .and_then(Query::from_value)
    .map(|query| {
        let (field, value) = query.pair(options.view);
        let params = match (&options.organization, options.organization_role, options.query.trim().is_empty()) {
            | (Some(organization), RemoteOrganizationRole::SiteOwner, false) => Params::new()
                .with_keyvalue(field, Some(value))
                .with_keyvalue("site_ownership_code", Some(organization)),
            | (Some(organization), RemoteOrganizationRole::SiteOwner, true) => Params::new().with_keyvalue("site_ownership_code", Some(organization)),
            | _ => Params::new().with_keyvalue(field, Some(value)),
        };
        let rows = options.limit.max(1).to_string();
        let start = options.start.to_string();
        params.with_keyvalue("rows", Some(&rows)).with_keyvalue("start", Some(&start)).build()
    })
}
/// Retrieve a single DOE CODE project by Code ID
/// ## Examples
/// ```no_run
/// use acorn::io::api::osti;
///
/// let project = osti::record(156286).await;
/// assert!(project.is_ok());
/// ```
pub async fn record(code_id: u64) -> ApiResult<Project> {
    match INCLUDED_ENDPOINTS.find_by_name("osti") {
        | Some(endpoint) => {
            let identifier = code_id.to_string();
            let params = Params::new().with_template("identifier", Some(&identifier)).build();
            let response = endpoint.invoke("record", Some(params)).await;
            endpoint.handle::<RecordResponse>(response).map(|response| response.metadata)
        }
        | None => Err(eyre!("OSTI API endpoint not found")),
    }
}
/// Search DOE CODE using one page or sequentially retrieve all pages
/// ## Examples
/// ```no_run
/// use acorn::io::api::osti::{self, Options, SearchView};
/// use acorn::schema::discovery::RemoteOrganizationRole;
///
/// let options = Options {
///     query: "scientific visualization".into(),
///     view: SearchView::Projects,
///     organization: Some("ORNL".into()),
///     organization_role: RemoteOrganizationRole::SiteOwner,
///     ..Options::default()
/// };
/// let response = osti::search(&options).await;
/// assert!(response.is_ok());
/// ```
pub async fn search(options: &Options) -> ApiResult<SearchResponse> {
    match page(options).await {
        | Err(why) => Err(why),
        | Ok(first) => match project_pages(options, first).await {
            | Err(why) => Err(why),
            | Ok((project_total, start, projects)) => {
                let projects = projects
                    .into_iter()
                    .filter(|project| matches_organization(project, options.organization.as_deref(), options.organization_role))
                    .collect::<Vec<_>>();
                let results = match options.view {
                    | SearchView::Projects => SearchResults::Projects(projects),
                    | SearchView::People => SearchResults::People(people(&projects)),
                    | SearchView::Organizations => {
                        SearchResults::Organizations(organizations(&projects, options.organization_role, options.organization.as_deref()))
                    }
                };
                Ok(SearchResponse {
                    project_total,
                    offset: options.start,
                    has_more: !options.all && start < project_total,
                    results,
                })
            }
        },
    }
}
fn sorted_unique<T: Ord>(values: Vec<T>) -> Vec<T> {
    values.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::api::{EmptyField, Param};

    fn project() -> Project {
        Project {
            code_id: 156286,
            site_ownership_code: "ORNL".to_string(),
            software_title: "ACORN".to_string(),
            developers: vec![ProjectPerson {
                orcid: "0000-0002-2057-9115".to_string(),
                first_name: "Jason".to_string(),
                last_name: "Wohlgemuth".to_string(),
                affiliations: vec!["Oak Ridge National Laboratory (ORNL)".to_string()],
                ..ProjectPerson::default()
            }],
            research_organizations: vec![ProjectOrganization {
                organization_name: "Oak Ridge National Laboratory (ORNL)".to_string(),
                doe: true,
                primary_award: None,
            }],
            ..Project::default()
        }
    }
    #[test]
    fn test_organization_filter_matches_role() {
        let project = project();
        assert!(matches_organization(&project, Some("ORNL"), RemoteOrganizationRole::Research));
        assert!(!matches_organization(&project, Some("ANL"), RemoteOrganizationRole::Any));
    }
    #[test]
    fn test_people_and_organizations_are_projected() {
        let projects = vec![project(), project()];
        assert_eq!(people(&projects).len(), 1);
        let all_organizations = organizations(&projects, RemoteOrganizationRole::Any, None);
        assert!(all_organizations
            .iter()
            .any(|organization| organization.aliases.iter().any(|alias| alias == "ORNL")));
        let filtered_organizations = organizations(&projects, RemoteOrganizationRole::Any, Some("ORNL"));
        assert_eq!(filtered_organizations.len(), 1);
        assert_eq!(
            filtered_organizations.first().map(|organization| organization.name.as_str()),
            Some("Oak Ridge National Laboratory (ORNL)")
        );
        assert_eq!(
            acronym("Oak Ridge National Laboratory (ORNL), Oak Ridge, TN (United States)"),
            Some("ORNL".to_string())
        );
    }
    #[test]
    fn test_project_converts_to_research_activity() {
        let activity = ResearchActivity::from(project());
        assert_eq!(activity.title, "ACORN");
        assert_eq!(activity.meta.identifier, "osti-156286");
    }
    #[test]
    fn test_request_url_normalizes_doi_and_orcid() {
        let doi = Options {
            query: "https://doi.org/10.11578/dc.20250604.1".to_string(),
            start: 40,
            ..Options::default()
        };
        let doi_query = Param::to_query_string::<EmptyField, EmptyField>(request_params(&doi).unwrap());
        assert!(doi_query.contains("identifiers=10.11578%2Fdc.20250604.1"));
        assert!(doi_query.contains("start=40"));
        let orcid = Options {
            query: "https://orcid.org/0000-0002-2057-9115".to_string(),
            view: SearchView::People,
            ..Options::default()
        };
        let orcid_query = Param::to_query_string::<EmptyField, EmptyField>(request_params(&orcid).unwrap());
        assert!(orcid_query.contains("orcid=0000-0002-2057-9115"));
    }
}
