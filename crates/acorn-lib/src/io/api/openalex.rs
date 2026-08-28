//! OpenAlex API access and research-output enrichment
//!
//! OpenAlex is used as a structured scholarly metadata provider. It complements
//! CiteAs, whose current ACORN integration remains focused on rendered citation
//! lookup rather than workflow enrichment.
use crate::io::api::{Configuration, Endpoint, Param, Params, RemoteResource};
use crate::io::enrichment::{Enrich, ResearchOutputMetadata};
use crate::io::{workflow, ApiResult};
use crate::prelude::{format, var, String, ToString, Vec};
use crate::schema::pid::{PersistentIdentifierParse, DOI};
use crate::schema::research_activity::output::{Access, Affiliation, Award, Contributor, Funding};
use crate::schema::research_activity::ResearchOutput;
use crate::schema::Website;
use crate::util::constants::app::{DEFAULT_OPENALEX_DOMAIN, OPENALEX_WORK_FIELDS};
use crate::util::constants::env::{OPENALEX_API_HOST, OPENALEX_API_KEY};
use crate::util::merge_unique_by;
use crate::Location;
use async_trait::async_trait;
use bon::Builder;
use color_eyre::eyre::eyre;
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};

/// A compact OpenAlex author identity.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Author {
    /// OpenAlex author identifier.
    pub id: String,
    /// Author display name.
    pub display_name: String,
    /// ORCID URL, when known.
    pub orcid: Option<String>,
}
/// An author-work relationship returned by OpenAlex.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Authorship {
    /// Author identity.
    pub author: Author,
    /// Position in the byline.
    pub author_position: String,
    /// Whether the author is identified as corresponding.
    pub is_corresponding: bool,
    /// Institutions asserted for this work.
    #[serde(default)]
    pub institutions: Vec<Institution>,
}
/// Funding award data associated with an OpenAlex work.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Grant {
    /// OpenAlex funder identifier.
    pub funder: Option<String>,
    /// Funder display name.
    pub funder_display_name: Option<String>,
    /// Provider-supplied award identifier.
    pub award_id: Option<String>,
}
/// A compact OpenAlex institution identity.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Institution {
    /// OpenAlex institution identifier.
    pub id: String,
    /// Institution display name.
    pub display_name: String,
    /// ROR URL, when known.
    pub ror: Option<String>,
}
/// A keyword inferred by OpenAlex.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Keyword {
    /// Keyword display value.
    pub display_name: String,
    /// Relevance score from zero to one.
    pub score: f64,
}
/// Open-access metadata associated with a work.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpenAccess {
    /// Whether an open copy is known.
    pub is_oa: bool,
    /// OpenAlex access classification.
    pub oa_status: String,
    /// Best known open URL.
    pub oa_url: Option<String>,
}
/// Options for one OpenAlex work lookup.
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct Options {
    /// API key used for authenticated OpenAlex requests.
    #[builder(default)]
    pub api_key: String,
    /// Request body payload.
    pub body: Option<String>,
    /// OpenAlex API host or base URL.
    #[builder(default = DEFAULT_OPENALEX_DOMAIN.to_string())]
    pub domain: String,
    /// DOI identifying the requested work.
    pub identifier: Option<String>,
    /// Custom API parameters to include in every request.
    #[builder(default = vec![])]
    pub custom_params: Vec<Param>,
}
/// OpenAlex-backed workflow enrichment provider.
#[derive(Clone, Debug)]
pub struct Provider {
    options: Options,
}
/// An OpenAlex work record used by ACORN enrichment.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Work {
    /// OpenAlex work identifier.
    pub id: String,
    /// Canonical DOI URL, when known.
    pub doi: Option<String>,
    /// Work title.
    pub title: String,
    /// OpenAlex work type.
    #[serde(rename = "type")]
    pub kind: String,
    /// ISO 8601 publication date.
    pub publication_date: Option<String>,
    /// Work-specific author and affiliation assertions.
    #[serde(default)]
    pub authorships: Vec<Authorship>,
    /// Funding awards associated with the work.
    #[serde(default)]
    pub grants: Vec<Grant>,
    /// Open-access summary.
    pub open_access: Option<OpenAccess>,
    /// Version-of-record or primary location.
    pub primary_location: Option<WorkLocation>,
    /// Best available open-access location.
    pub best_oa_location: Option<WorkLocation>,
    /// OpenAlex keywords and their relevance scores.
    #[serde(default)]
    pub keywords: Vec<Keyword>,
}
/// A publication or repository location.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkLocation {
    /// Landing-page URL.
    pub landing_page_url: Option<String>,
    /// Direct PDF URL.
    pub pdf_url: Option<String>,
    /// License identifier asserted for this location.
    pub license: Option<String>,
}
impl WorkLocation {
    fn websites(&self, description: &str) -> Vec<Website> {
        let Self {
            landing_page_url, pdf_url, ..
        } = self;
        merge_unique_by(
            [],
            [landing_page_url, pdf_url]
                .into_iter()
                .flatten()
                .map(|url| Website::at(url).description(description).build()),
            |left, right| left.url == right.url,
        )
    }
}
impl From<Institution> for Affiliation {
    fn from(institution: Institution) -> Self {
        Affiliation::init().name(institution.display_name).maybe_ror(institution.ror).build()
    }
}
impl From<Authorship> for Contributor {
    fn from(authorship: Authorship) -> Self {
        Contributor::init()
            .name(authorship.author.display_name)
            .maybe_orcid(
                authorship
                    .author
                    .orcid
                    .map(|value| value.rsplit('/').next().unwrap_or(&value).to_string()),
            )
            .corresponding(authorship.is_corresponding)
            .position(authorship.author_position)
            .affiliations(authorship.institutions.into_iter().map(Affiliation::from).collect())
            .build()
    }
}
impl From<Grant> for Option<Funding> {
    fn from(grant: Grant) -> Self {
        grant.funder_display_name.map(|name| {
            let awards = grant
                .award_id
                .filter(|value| !value.trim().is_empty())
                .map(|identifier| vec![Award::init().identifier(identifier).build()])
                .unwrap_or_default();
            Funding::init().name(name).awards(awards).build()
        })
    }
}
impl Configuration for Options {
    /// Build options from OpenAlex environment variables.
    fn from_env() -> Self {
        Self {
            api_key: var(OPENALEX_API_KEY).unwrap_or_default(),
            body: None,
            domain: var(OPENALEX_API_HOST).unwrap_or_else(|_| DEFAULT_OPENALEX_DOMAIN.to_string()),
            identifier: None,
            custom_params: vec![],
        }
    }
    fn with_body(self, value: impl Into<String>) -> Self {
        Self {
            body: Some(value.into()),
            ..self
        }
    }
    fn with_domain(self, value: impl Into<String>) -> Self {
        Self {
            domain: value.into(),
            ..self
        }
    }
    fn with_identifier(self, identifier: impl Into<String>) -> Self {
        Self {
            identifier: Some(identifier.into()),
            ..self
        }
    }
    fn token(&self) -> &str {
        &self.api_key
    }
    fn domain(&self) -> &str {
        &self.domain
    }
    fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }
    fn with_params(self, params: Vec<Param>) -> Self {
        Self {
            custom_params: params,
            ..self
        }
    }
    fn params(&self) -> &[Param] {
        &self.custom_params
    }
}
impl Default for Options {
    fn default() -> Self {
        Self::from_env()
    }
}
impl Provider {
    /// Create a provider using base request options.
    pub fn new(options: Options) -> Self {
        Self { options }
    }
    /// Create a provider from environment configuration.
    pub fn from_env() -> Self {
        Self::new(Options::from_env())
    }
}
#[async_trait]
impl<T> Enrich<(Location, T)> for Provider
where
    T: ResearchOutputMetadata + 'static,
{
    type Output = ApiResult<workflow::EnrichmentResult<T>>;
    async fn enrich(&self, (_, data): (Location, T)) -> Self::Output {
        let result = stream::iter(data.publication_identifiers())
            .fold(
                workflow::EnrichmentResult {
                    data: data.research_outputs(),
                    conflicts: Vec::new(),
                    failures: Vec::new(),
                },
                |result, identifier| async move {
                    let normalized = DOI::format(&identifier);
                    match normalized.is_empty() {
                        | true => result,
                        | false => {
                            let record = work(&self.options.clone().with_identifier(normalized.clone())).await;
                            merge_work_result(result, normalized, record)
                        }
                    }
                },
            )
            .await;
        Ok(workflow::EnrichmentResult {
            data: data.with_research_outputs(result.data),
            conflicts: result.conflicts,
            failures: result.failures,
        })
    }
}
impl From<Work> for ResearchOutput {
    fn from(work: Work) -> Self {
        let contributors = work.authorships.into_iter().map(Contributor::from).collect::<Vec<_>>();
        let funding = work.grants.into_iter().filter_map(Option::<Funding>::from).collect::<Vec<_>>();
        let websites = merge_unique_by(
            work.primary_location
                .as_ref()
                .into_iter()
                .flat_map(|location| location.websites("Primary location")),
            work.best_oa_location
                .as_ref()
                .into_iter()
                .flat_map(|location| location.websites("Open-access location")),
            |left, right| left.url == right.url,
        );
        let access = work.open_access.map(|value| {
            Access::init()
                .open(value.is_oa)
                .status(value.oa_status)
                .maybe_url(value.oa_url)
                .maybe_license(work.best_oa_location.and_then(|location| location.license))
                .build()
        });
        ResearchOutput::init()
            .identifier(work.id)
            .maybe_doi(work.doi.map(|value| DOI::format(&value)))
            .title(work.title)
            .kind(work.kind)
            .maybe_publication_date(work.publication_date)
            .maybe_contributors((!contributors.is_empty()).then_some(contributors))
            .maybe_funding((!funding.is_empty()).then_some(funding))
            .maybe_websites((!websites.is_empty()).then_some(websites))
            .keywords(work.keywords.into_iter().map(|value| value.display_name).collect())
            .maybe_access(access)
            .build()
    }
}
pub(crate) fn merge_work_result(
    result: workflow::EnrichmentResult<Vec<ResearchOutput>>,
    identifier: String,
    record: ApiResult<Work>,
) -> workflow::EnrichmentResult<Vec<ResearchOutput>> {
    match record {
        | Ok(record) => {
            let candidate = ResearchOutput::from(record);
            let exists = result
                .data
                .iter()
                .filter_map(|output| output.doi.as_ref())
                .any(|doi| DOI::format(doi).eq_ignore_ascii_case(&identifier));
            let data = match exists {
                | true => result
                    .data
                    .into_iter()
                    .map(
                        |output| match output.doi.as_ref().is_some_and(|doi| DOI::format(doi).eq_ignore_ascii_case(&identifier)) {
                            | true => output.merge(candidate.clone()),
                            | false => output,
                        },
                    )
                    .collect(),
                | false => result.data.into_iter().chain([candidate]).collect(),
            };
            workflow::EnrichmentResult { data, ..result }
        }
        | Err(why) => workflow::EnrichmentResult {
            failures: result.failures.into_iter().chain([format!("{identifier}: {why}")]).collect(),
            ..result
        },
    }
}
/// Retrieve one OpenAlex work using the DOI in `options`
pub async fn work(options: &Options) -> ApiResult<Work> {
    let template = "openalex";
    let action = "work";
    let identifier = options.identifier.as_ref().map(DOI::format).filter(|value| !value.is_empty());
    match identifier.ok_or_else(|| eyre!("An OpenAlex work lookup requires a DOI")) {
        | Ok(identifier) => {
            let params = Params::new()
                .with_template("identifier", Some(identifier.as_str()))
                .with_keyvalue("select", Some(OPENALEX_WORK_FIELDS))
                .with_api_key(&options.api_key)
                .with_custom(options.params())
                .build();
            match Endpoint::from_template(template).map(|endpoint| endpoint.with_domain(options.domain())) {
                | Ok(endpoint) => endpoint
                    .handle::<Work>(endpoint.invoke(action, Some(params)).await)
                    .map_err(|why| eyre!("Failed to retrieve OpenAlex work — {why}")),
                | Err(why) => Err(why),
            }
        }
        | Err(why) => Err(why),
    }
}
