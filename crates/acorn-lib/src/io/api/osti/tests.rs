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
#[test]
fn test_with_query_uses_organization_filter_for_organization_view() {
    let projects = Options::default().with_query("visualization");
    assert_eq!(projects.query, "visualization");
    assert_eq!(projects.organization, None);
    let organizations = Options {
        view: SearchView::Organizations,
        ..Options::default()
    }
    .with_query("ORNL");
    assert_eq!(organizations.query, "");
    assert_eq!(organizations.organization.as_deref(), Some("ORNL"));
}
