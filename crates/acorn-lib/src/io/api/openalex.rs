//! OpenAlex API access and research-output enrichment.
//!
//! OpenAlex is used as a structured scholarly metadata provider. It complements
//! CiteAs, whose current ACORN integration remains focused on rendered citation
//! lookup rather than workflow enrichment.

use crate::io::http::get;
use crate::io::workflow;
use crate::io::ApiResult;
use crate::prelude::{format, var, String, ToString, Vec};
use crate::schema::pid::{PersistentIdentifierParse, DOI};
use crate::schema::research_activity::output::{Access, Affiliation, Award, Contributor, Funding};
use crate::schema::research_activity::{ResearchActivity, ResearchActivityMetadata, ResearchOutput};
use crate::schema::Website;
use crate::util::constants::env::{OPENALEX_API_HOST, OPENALEX_API_KEY};
use async_trait::async_trait;
use bon::Builder;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

const DEFAULT_DOMAIN: &str = "api.openalex.org";
const WORK_FIELDS: &str = "id,doi,title,type,publication_date,authorships,grants,open_access,primary_location,best_oa_location,keywords";

/// Options for one OpenAlex work lookup.
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct Options {
    /// API key used for authenticated OpenAlex requests.
    #[builder(default)]
    pub api_key: String,
    /// OpenAlex API host or base URL.
    #[builder(default = DEFAULT_DOMAIN.to_string())]
    pub domain: String,
    /// DOI identifying the requested work.
    pub identifier: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self::from_env()
    }
}

impl Options {
    /// Build options from OpenAlex environment variables.
    pub fn from_env() -> Self {
        Self {
            api_key: var(OPENALEX_API_KEY).unwrap_or_default(),
            domain: var(OPENALEX_API_HOST).unwrap_or_else(|_| DEFAULT_DOMAIN.to_string()),
            identifier: None,
        }
    }

    /// Return options configured for one DOI.
    pub fn with_identifier(self, identifier: impl Into<String>) -> Self {
        Self {
            identifier: Some(identifier.into()),
            ..self
        }
    }
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
    pub primary_location: Option<Location>,
    /// Best available open-access location.
    pub best_oa_location: Option<Location>,
    /// OpenAlex keywords and their relevance scores.
    #[serde(default)]
    pub keywords: Vec<Keyword>,
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

/// A publication or repository location.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Location {
    /// Landing-page URL.
    pub landing_page_url: Option<String>,
    /// Direct PDF URL.
    pub pdf_url: Option<String>,
    /// License identifier asserted for this location.
    pub license: Option<String>,
}

/// A keyword inferred by OpenAlex.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Keyword {
    /// Keyword display value.
    pub display_name: String,
    /// Relevance score from zero to one.
    pub score: f64,
}

/// Construct the request URL for one OpenAlex work lookup.
///
/// # Errors
///
/// Returns an error when no DOI is configured.
pub fn endpoint(options: &Options) -> ApiResult<String> {
    let identifier = match options
        .identifier
        .as_ref()
        .map(|value| normalize_doi(value))
        .filter(|value| !value.is_empty())
    {
        | Some(value) => value,
        | None => return Err(eyre!("An OpenAlex work lookup requires a DOI")),
    };
    let domain = options.domain.trim().trim_end_matches('/');
    let base = if domain.starts_with("http://") || domain.starts_with("https://") {
        domain.to_string()
    } else {
        format!("https://{domain}")
    };
    let key = if options.api_key.trim().is_empty() {
        String::new()
    } else {
        format!("&api_key={}", urlencoding::encode(options.api_key.trim()))
    };
    Ok(format!("{base}/works/doi:{}?select={WORK_FIELDS}{key}", urlencoding::encode(&identifier)))
}

/// Retrieve one OpenAlex work using the DOI in `options`.
///
/// # Errors
///
/// Returns an error for invalid options, HTTP failures, or malformed responses.
pub async fn work(options: &Options) -> ApiResult<Work> {
    let url = match endpoint(options) {
        | Ok(value) => value,
        | Err(why) => return Err(why),
    };
    let response = match get(url).send().await {
        | Ok(value) => value,
        | Err(why) => return Err(why),
    };
    match response.text().await {
        | Ok(content) => serde_json::from_str(&content).map_err(|why| eyre!("Failed to deserialize OpenAlex work — {why}")),
        | Err(why) => Err(why),
    }
}

/// OpenAlex-backed workflow enrichment provider.
#[derive(Clone, Debug)]
pub struct Provider {
    options: Options,
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
impl workflow::Enrichment for Provider {
    async fn enrich(&self, _source: &workflow::Source, data: ResearchActivity) -> ApiResult<ResearchActivity> {
        let identifiers = data.meta.doi.clone().unwrap_or_default();
        let mut outputs = data.meta.outputs.clone().unwrap_or_default();
        for identifier in identifiers {
            let normalized = normalize_doi(&identifier);
            if normalized.is_empty() {
                continue;
            }
            let exists = outputs
                .iter()
                .filter_map(|output| output.doi.as_ref())
                .any(|doi| normalize_doi(doi).eq_ignore_ascii_case(&normalized));
            if !exists {
                let options = self.options.clone().with_identifier(normalized);
                let record = match work(&options).await {
                    | Ok(value) => value,
                    | Err(why) => return Err(why),
                };
                outputs.push(ResearchOutput::from(record));
            }
        }
        Ok(ResearchActivity {
            meta: ResearchActivityMetadata {
                outputs: (!outputs.is_empty()).then_some(outputs),
                ..data.meta
            },
            ..data
        })
    }
}

impl From<Work> for ResearchOutput {
    fn from(work: Work) -> Self {
        let contributors = work.authorships.into_iter().map(Contributor::from).collect::<Vec<_>>();
        let funding = work.grants.into_iter().filter_map(Option::<Funding>::from).collect::<Vec<_>>();
        let mut websites = Vec::new();
        append_location(&mut websites, work.primary_location.as_ref(), "Primary location");
        append_location(&mut websites, work.best_oa_location.as_ref(), "Open-access location");
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
            .maybe_doi(work.doi.map(|value| normalize_doi(&value)))
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

impl From<Authorship> for Contributor {
    fn from(authorship: Authorship) -> Self {
        Contributor::init()
            .name(authorship.author.display_name)
            .maybe_orcid(authorship.author.orcid.map(|value| trim_identifier(&value)))
            .corresponding(authorship.is_corresponding)
            .position(authorship.author_position)
            .affiliations(authorship.institutions.into_iter().map(Affiliation::from).collect())
            .build()
    }
}

impl From<Institution> for Affiliation {
    fn from(institution: Institution) -> Self {
        Affiliation::init().name(institution.display_name).maybe_ror(institution.ror).build()
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

fn append_location(websites: &mut Vec<Website>, location: Option<&Location>, description: &str) {
    if let Some(location) = location {
        for url in [&location.landing_page_url, &location.pdf_url].into_iter().flatten() {
            if !websites.iter().any(|website| website.url == *url) {
                websites.push(Website {
                    description: description.to_string(),
                    url: url.clone(),
                });
            }
        }
    }
}

fn normalize_doi(value: &str) -> String {
    DOI::format(value)
}

fn trim_identifier(value: &str) -> String {
    value.rsplit('/').next().unwrap_or(value).to_string()
}
