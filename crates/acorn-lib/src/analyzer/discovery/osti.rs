//! OSTI-backed remote discovery search support.
use super::{RemoteEntity, RemoteMatch, RemoteProvider, RemoteSearchRequest, RemoteSearchResponse};
use crate::io::api::osti;
use crate::io::api::osti::SearchResults;
use crate::io::ApiResult;
use color_eyre::eyre::Report as EyreReport;
impl From<RemoteSearchRequest> for osti::Options {
    fn from(request: RemoteSearchRequest) -> Self {
        Self {
            query: String::new(),
            view: request.entity.into(),
            organization: request.organization,
            organization_role: request.organization_role,
            limit: request.limit,
            start: request.offset,
            all: request.all,
        }
    }
}
impl RemoteSearchResponse {
    pub(super) fn from_osti(response: osti::SearchResponse) -> ApiResult<Self> {
        let matches = match response.results {
            | SearchResults::Projects(values) => values
                .into_iter()
                .map(|value| {
                    serde_json::to_value(&value).map_err(EyreReport::from).map(|metadata| RemoteMatch {
                        entity: RemoteEntity::Project,
                        identifier: value.code_id.to_string(),
                        title: value.software_title,
                        pid: value.doi,
                        url: value
                            .repository_link
                            .or(value.landing_page)
                            .or_else(|| value.links.iter().find(|link| link.rel == "citation").map(|link| link.href.clone())),
                        metadata,
                    })
                })
                .collect::<ApiResult<Vec<_>>>(),
            | SearchResults::People(values) => values
                .into_iter()
                .map(|value| {
                    serde_json::to_value(&value).map_err(EyreReport::from).map(|metadata| RemoteMatch {
                        entity: RemoteEntity::Person,
                        identifier: value.orcid.clone().or(value.email.clone()).unwrap_or_else(|| value.name.clone()),
                        title: value.name,
                        pid: value.orcid,
                        url: None,
                        metadata,
                    })
                })
                .collect::<ApiResult<Vec<_>>>(),
            | SearchResults::Organizations(values) => values
                .into_iter()
                .map(|value| {
                    serde_json::to_value(&value).map_err(EyreReport::from).map(|metadata| RemoteMatch {
                        entity: RemoteEntity::Organization,
                        identifier: value.aliases.first().cloned().unwrap_or_else(|| value.name.clone()),
                        title: value.name,
                        pid: None,
                        url: None,
                        metadata,
                    })
                })
                .collect::<ApiResult<Vec<_>>>(),
        };
        matches.map(|matches| Self {
            provider: RemoteProvider::Osti,
            total: response.project_total,
            offset: response.offset,
            has_more: response.has_more,
            matches,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_types_convert_to_osti_types() {
        assert_eq!(osti::SearchView::from(RemoteEntity::Person), osti::SearchView::People);
    }
    #[test]
    fn test_remote_request_converts_to_osti_options() {
        let request = RemoteSearchRequest::init()
            .provider(RemoteProvider::Osti)
            .entity(RemoteEntity::Organization)
            .organization_role(crate::schema::discovery::RemoteOrganizationRole::Research)
            .limit(50)
            .offset(25)
            .all(true)
            .build();
        let options = osti::Options::from(request).with_query("ORNL");
        assert_eq!(options.query, "");
        assert_eq!(options.view, osti::SearchView::Organizations);
        assert_eq!(options.organization.as_deref(), Some("ORNL"));
        assert_eq!(options.organization_role, crate::schema::discovery::RemoteOrganizationRole::Research);
        assert_eq!(options.limit, 50);
        assert_eq!(options.start, 25);
        assert!(options.all);
    }
}
