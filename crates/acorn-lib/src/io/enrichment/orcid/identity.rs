use super::super::{
    format, param, warn, ApiResult, Configuration, Organization, Param, PersistentIdentifier, PersistentIdentifierParse, String, StringConversion,
    ToString, Vec, ORCID, ROR,
};
use super::{OrcidIdentity, OrcidResolution};
use crate::io::api::orcid;

impl OrcidIdentity {
    pub(in crate::io::enrichment) fn new(
        display_name: impl Into<String>,
        affiliations: Vec<String>,
        fill_names: bool,
        given_name: Option<String>,
        family_name: Option<String>,
        identifier: Option<String>,
        email: Option<String>,
    ) -> Option<Self> {
        let display_name = display_name.into().trim().to_string();
        let given_name = given_name.filter(|value| !value.trim().is_empty());
        let family_name = family_name.filter(|value| !value.trim().is_empty());
        let identifier = identifier
            .map(|value| ORCID::from_string(value).identifier())
            .filter(|value| !value.is_empty());
        let is_missing_name = given_name.is_none() || family_name.is_none();
        let is_searchable_name = (given_name.is_some() && family_name.is_some()) || !display_name.is_empty();
        match (identifier.is_some(), fill_names && is_missing_name, is_searchable_name) {
            | (true, true, _) | (false, _, true) => Some(Self {
                affiliations,
                display_name,
                email: email.filter(|value| !value.trim().is_empty()),
                family_name,
                given_name,
                identifier,
            }),
            | _ => None,
        }
    }
    fn affiliation_names(&self) -> Vec<String> {
        self.affiliations
            .iter()
            .cloned()
            .chain(
                self.affiliation_root()
                    .into_iter()
                    .flat_map(|organization| [Some(organization.name), organization.alternative_name].into_iter().flatten()),
            )
            .collect()
    }
    fn affiliation_root(&self) -> Option<Organization> {
        Organization::load().into_iter().find(|root| {
            [root.clone()].into_iter().chain(root.clone().members()).any(|member| {
                self.affiliations.iter().any(|affiliation| {
                    let normalized = affiliation.normalized();
                    let matches_name = normalized == member.name.normalized();
                    let matches_alias = member.alternative_name.as_ref().is_some_and(|name| normalized == name.normalized());
                    matches_name || matches_alias
                })
            })
        })
    }
    fn affiliation_ror(&self) -> Option<String> {
        self.affiliation_root().and_then(|organization| organization.ror).map(ROR::format)
    }
    fn email_domain(&self) -> Option<String> {
        self.email
            .as_deref()
            .map(str::trim)
            .and_then(|email| email.split_once('@'))
            .filter(|(local, domain)| !local.is_empty() && !domain.is_empty() && !domain.contains('@'))
            .map(|(_, domain)| domain.to_ascii_lowercase())
    }
    fn email_matches(&self, result: &orcid::SearchResult) -> bool {
        self.email.as_ref().is_some_and(|email| {
            result
                .emails
                .as_ref()
                .is_some_and(|values| values.iter().any(|value| value.trim().eq_ignore_ascii_case(email.trim())))
        })
    }
    pub(in crate::io) fn key(&self) -> String {
        let email = self
            .email
            .as_deref()
            .map(|email| email.normalized())
            .map(|email| format!("|email:{email}"))
            .unwrap_or_default();
        match &self.identifier {
            | Some(identifier) => format!("orcid:{identifier}"),
            | None => match (&self.given_name, &self.family_name) {
                | (Some(given_name), Some(family_name)) => {
                    format!("name:{}|{}{email}", given_name.normalized(), family_name.normalized())
                }
                | _ => format!("name:{}{email}", self.display_name.normalized()),
            },
        }
    }
    pub(super) fn source(&self) -> String {
        self.identifier.clone().unwrap_or_else(|| match (&self.given_name, &self.family_name) {
            | (Some(given_name), Some(family_name)) => format!("{given_name} {family_name}"),
            | _ => self.display_name.clone(),
        })
    }
    pub(in crate::io) fn email_query(&self) -> Option<Param> {
        let email = self.email.as_deref().map(str::trim).map(str::to_ascii_lowercase);
        match (&self.given_name, &self.family_name, email.as_deref()) {
            | (Some(given_name), Some(family_name), Some(email)) => Some(Param::from_query_pair(
                "q",
                vec![
                    ("given-names", given_name.as_str()),
                    ("family-name", family_name.as_str()),
                    ("email", email),
                ],
            )),
            | (_, _, Some(email)) => Some(Param::from_query_pair(
                "q",
                vec![("credit-name", self.display_name.as_str()), ("email", email)],
            )),
            | _ => None,
        }
    }
    pub(in crate::io) fn email_domain_query(&self) -> Option<Param> {
        let email = self.email_domain().map(|domain| format!("*@{domain}"));
        match (&self.given_name, &self.family_name, email.as_deref()) {
            | (Some(given_name), Some(family_name), Some(email)) => Some(Param::from_query_pair(
                "q",
                vec![
                    ("given-names", given_name.as_str()),
                    ("family-name", family_name.as_str()),
                    ("email", email),
                ],
            )),
            | (_, _, Some(email)) => Some(Param::from_query_pair(
                "q",
                vec![("credit-name", self.display_name.as_str()), ("email", email)],
            )),
            | _ => None,
        }
    }
    async fn fetch_query(&self, query: Param) -> ApiResult<orcid::SearchResponse> {
        let fields = param!(
            FieldList,
            "fl",
            vec![
                "orcid",
                "email",
                "credit-name",
                "given-names",
                "family-name",
                "other-name",
                "current-institution-affiliation-name",
                "past-institution-affiliation-name",
            ]
        );
        orcid::search(&orcid::Options::from_env().with_params(vec![query, fields])).await
    }
    async fn fetch_refinement(&self, response: orcid::SearchResponse, query: Option<Param>, description: &str) -> ApiResult<orcid::SearchResponse> {
        match query {
            | Some(query) => match self.fetch_query(query).await {
                | Ok(refined) => Ok(self.prefer_refined_response(response, refined)),
                | Err(why) => {
                    warn!("ORCID {description} refinement failed — {why}");
                    Ok(response)
                }
            },
            | None => Ok(response),
        }
    }
    async fn fetch_name_or_email(&self) -> ApiResult<orcid::SearchResponse> {
        match self.email_query() {
            | Some(query) => match self.fetch_query(query).await {
                | Ok(response) if matches!(self.select(&response), Ok(Some(_))) => Ok(response),
                | Ok(_) => self.fetch_query(self.search_query()).await,
                | Err(why) => {
                    warn!("ORCID email-and-name lookup failed — {why}");
                    self.fetch_query(self.search_query()).await
                }
            },
            | None => self.fetch_query(self.search_query()).await,
        }
    }
    pub(in crate::io) fn search_query(&self) -> Param {
        match (&self.given_name, &self.family_name) {
            | (Some(given_name), Some(family_name)) => {
                Param::from_query_pair("q", vec![("given-names", given_name.as_str()), ("family-name", family_name.as_str())])
            }
            | _ => Param::from_query_pair("q", vec![("credit-name", self.display_name.as_str())]),
        }
    }
    pub(super) async fn fetch(&self) -> ApiResult<OrcidResolution> {
        match &self.identifier {
            | Some(identifier) => orcid::record(&orcid::Options::from_env().with_identifier(identifier))
                .await
                .map(OrcidResolution::Resolved),
            | None => self.fetch_search().await.map(|response| self.resolve(&response)),
        }
    }
    async fn fetch_search(&self) -> ApiResult<orcid::SearchResponse> {
        match self.fetch_name_or_email().await {
            | Ok(response) => match matches!(self.select(&response), Ok(Some(_))) {
                | false => match self.fetch_refinement(response, self.email_domain_query(), "email-domain").await {
                    | Ok(refined) => match matches!(self.select(&refined), Ok(Some(_))) {
                        | false => self.fetch_refinement(refined, self.ror_query(), "affiliation-ROR").await,
                        | true => Ok(refined),
                    },
                    | Err(why) => Err(why),
                },
                | true => Ok(response),
            },
            | Err(why) => Err(why),
        }
    }
    fn exact_matches(&self, response: &orcid::SearchResponse) -> Vec<orcid::SearchResult> {
        response
            .results
            .iter()
            .filter(|result| {
                let email_matches = self.email_matches(result);
                let assembled = [result.given_names.as_deref(), result.family_names.as_deref()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" ");
                let display_name_matches = result
                    .credit_name
                    .as_deref()
                    .into_iter()
                    .chain(result.other_name.as_deref().unwrap_or_default().iter().map(String::as_str))
                    .chain([assembled.as_str()])
                    .any(|value| value.normalized() == self.display_name.normalized());
                let name_matches = match (&self.given_name, &self.family_name) {
                    | (Some(given_name), Some(family_name)) => {
                        let given_name_matches = result.given_names.as_deref().is_some_and(|value| {
                            value.normalized() == given_name.normalized()
                                || value
                                    .split_whitespace()
                                    .any(|component| component.normalized() == given_name.normalized())
                        });
                        let family_name_matches = result
                            .family_names
                            .as_deref()
                            .is_some_and(|value| value.normalized() == family_name.normalized());
                        (given_name_matches && family_name_matches) || display_name_matches
                    }
                    | _ => display_name_matches,
                };
                email_matches || name_matches
            })
            .cloned()
            .collect()
    }
    pub(in crate::io) fn prefer_refined_response(&self, response: orcid::SearchResponse, refined: orcid::SearchResponse) -> orcid::SearchResponse {
        match self.select(&refined) {
            | Ok(Some(_)) => refined,
            | Ok(None) | Err(()) => response,
        }
    }
    pub(in crate::io) fn resolve(&self, response: &orcid::SearchResponse) -> OrcidResolution {
        let matches = self.compatible_matches(response);
        match matches.as_slice() {
            | [profile] => OrcidResolution::Resolved(profile.clone()),
            | [] => OrcidResolution::Missing,
            | _ => OrcidResolution::Ambiguous(matches),
        }
    }
    pub(in crate::io) fn ror_query(&self) -> Option<Param> {
        let ror = self.affiliation_ror().map(|value| format!("\"{value}\""));
        match (&self.given_name, &self.family_name, ror.as_deref()) {
            | (Some(given_name), Some(family_name), Some(ror)) => Some(Param::from_query_pair(
                "q",
                vec![
                    ("given-names", given_name.as_str()),
                    ("family-name", family_name.as_str()),
                    ("ror-org-id", ror),
                ],
            )),
            | (_, _, Some(ror)) => Some(Param::from_query_pair(
                "q",
                vec![("credit-name", self.display_name.as_str()), ("ror-org-id", ror)],
            )),
            | _ => None,
        }
    }
    fn compatible_matches(&self, response: &orcid::SearchResponse) -> Vec<orcid::SearchResult> {
        let matches = self.exact_matches(response);
        let affiliation_names = self.affiliation_names();
        let has_discriminator = self.email.is_some() || !affiliation_names.is_empty();
        let narrowed = match matches.len() > 1 && has_discriminator {
            | true => matches
                .iter()
                .filter(|result| {
                    self.email_matches(result)
                        || affiliation_names.iter().any(|affiliation| {
                            result
                                .institution_names
                                .as_ref()
                                .is_some_and(|values| values.iter().any(|value| value.normalized() == affiliation.normalized()))
                        })
                })
                .cloned()
                .collect::<Vec<_>>(),
            | false => Vec::new(),
        };
        match narrowed.is_empty() {
            | true => matches,
            | false => narrowed,
        }
    }
    pub(in crate::io) fn select(&self, response: &orcid::SearchResponse) -> Result<Option<orcid::SearchResult>, ()> {
        match self.compatible_matches(response).as_slice() {
            | [profile] => Ok(Some(profile.clone())),
            | [] => Ok(None),
            | _ => Err(()),
        }
    }
}
