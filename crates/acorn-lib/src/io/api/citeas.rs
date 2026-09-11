//! Module for communicating with CiteAs API
//! > CiteAs is a way to get the correct citation for diverse research products including, software, datasets, preprints, and traditional articles. By making it easier to cite software and other "alternative" scholarly products, we aim to help the creators of such products get full credit for their work.
//!
//! See <https://citeas.org/api> for more information
use crate::analyzer::host::discovery::CitationFormat;
use crate::io::api::{ApiResult, Configuration, Param, Params, RemoteResource, INCLUDED_ENDPOINTS};
use crate::param;
use crate::util::constants::app::DEFAULT_CITEAS_DOMAIN;
use crate::util::constants::env::{CITEAS_API_TOKEN, CITEAS_SERVER_HOST};
use acorn_core::options::{ApiExtension, ApiOptions};
use acorn_core::util::Searchable;
use acorn_schema::pid::{Arxiv, PersistentIdentifier, PublicationIdentifierType, DOI};
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// CiteAs API options
pub type Options = ApiOptions<Extension, Param>;
/// Trait for things that can be converted to citations.
#[async_trait]
pub trait ToCitations {
    /// Convert to `Citations`
    async fn to_citations(&self) -> ApiResult<Citations>;
}
/// Author object
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Author {
    /// First name
    pub given: String,
    /// Last name
    pub family: String,
}
/// Citation API response object
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Citation {
    /// Citation entry
    #[serde(alias = "citation")]
    pub text: String,
    /// Full name of the citation style
    /// ### Example
    /// > "American Psychological Association 6th edition"
    pub style_fullname: String,
    /// Short name of the citation style
    /// ### Example
    /// > "APA"
    pub style_shortname: String,
}
/// Main response object for the CiteAs API, returning citations for a given input
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Citations {
    /// List of citation objects
    pub citations: Vec<Citation>,
    /// List of export objects
    pub exports: Vec<Export>,
    /// Metadata for listing all metadata found for a given resource.
    /// <div class="warning">Varies by source</div>
    pub metadata: Metadata,
    /// Name of referenced resource
    pub name: String,
    /// List of provenance objects describing sources utilized to find and build citation data
    pub provenance: Vec<Provenance>,
    /// URL for the given resource
    /// <div class="warning">If input is a keyword, the URL is the first Google search result for the given keyword</div>
    pub url: String,
}
/// Exported citation data
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Export {
    /// Citation export
    pub export: String,
    /// Export format
    /// ### Note
    /// > May include CSV, enw, [RIS], and [BibTeX].
    ///
    /// [RIS]: https://en.wikipedia.org/wiki/RIS_(file_format)
    /// [BibTeX]: https://www.bibtex.org/
    pub export_name: String,
}
/// CiteAs-specific API option defaults.
#[derive(Clone, Debug, Default)]
pub struct Extension;
/// Metadata for source
/// <div class="warning">Varies by source</div>
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Metadata {
    /// List of authors
    pub author: Vec<Author>,
    /// List of categories that resource applies to
    pub categories: Vec<String>,
    /// List of contributors
    pub contributor: Vec<Author>,
    /// Valid DOI
    #[serde(alias = "DOI")]
    pub doi: String,
    /// ID for the resource
    /// <div class="warning">Always "ITEM-1"</div>
    pub id: String,
    /// Publisher of resource
    pub publisher: String,
    /// Type of resource
    #[serde(rename = "type")]
    pub resource_type: String,
    /// Title of the resource
    /// ### Example
    /// > "Oak Ridge National Laboratory (ORNL), Oak Ridge, TN (United States)"
    pub title: String,
    /// Resource URL
    #[serde(alias = "URL")]
    pub url: String,
    /// Year of publication
    pub year: u16,
}
/// Citation provenance object
///
/// Describes steps taken to try and find citation data, and whether citation data was found
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Provenance {
    /// Additional URL utilized to discover citation data
    pub additional_content_url: Option<String>,
    /// URL utilized to discover citation data
    pub content_url: Option<String>,
    /// Original URL of the resource
    pub original_url: Option<String>,
    /// Returns "doi" or "arXiv ID" if found via DOI or arXiv, else "null"
    pub found_via_proxy_type: Option<String>,
    /// Returns true if content was found at the URL
    pub has_content: bool,
    /// Host of the resource, such as crossref, github or pypi
    pub host: Option<String>,
    /// Name of the step taken to find citation data
    pub name: String,
    /// Name of the parent step
    pub parent_step_name: String,
    /// Name of the parent subject
    pub parent_subject: Option<String>,
    /// Subject of the current step
    /// ### Example
    /// > "GitHub repository main page"
    pub subject: String,
    /// Resource keyword
    pub key_word: Option<String>,
}
/// Describes status of the CiteAs API
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StatusResponse {
    /// Where you can find documentation for this version
    /// ### Example
    /// > "<https://citeas.org/api>"
    pub documentation_url: String,
    /// Relevant messages
    /// ### Example
    /// > "Don't panic"
    pub msg: String,
    /// API version
    /// ### Example
    /// > "0.1"
    pub version: String,
}
#[async_trait]
impl ToCitations for Arxiv {
    async fn to_citations(&self) -> ApiResult<Citations> {
        PublicationIdentifierType::from(self).to_citations().await
    }
}
impl Citations {
    /// Get citation data with the requested citation style.
    ///
    /// If the requested style is not found, the first available citation is returned.
    pub fn match_style(self, value: CitationFormat) -> Option<Citation> {
        let citations = self.citations;
        let normalized_value = value.to_string();
        let result = citations
            .iter()
            .find(|citation| citation.style_shortname.eq_ignore_ascii_case(&normalized_value));
        result.cloned().or_else(|| citations.first().cloned())
    }
}
#[async_trait]
impl ToCitations for DOI {
    /// Convert a [`DOI`] to a [`Citations`]
    async fn to_citations(&self) -> ApiResult<Citations> {
        PublicationIdentifierType::from(self).to_citations().await
    }
}
impl ApiExtension for Extension {
    fn default_domain() -> String {
        String::from(DEFAULT_CITEAS_DOMAIN)
    }
    fn env_token_var() -> &'static str {
        CITEAS_API_TOKEN
    }
    fn env_domain_var() -> &'static str {
        CITEAS_SERVER_HOST
    }
}
#[async_trait]
impl ToCitations for PublicationIdentifierType {
    async fn to_citations(&self) -> ApiResult<Citations> {
        let value = match self {
            | Self::Doi(doi) => Some(doi.to_string()),
            | Self::Arxiv(arxiv) => Some(arxiv.url()),
            | Self::Unknown => None,
        };
        match value {
            | Some(value) => {
                let identifier = urlencoding::encode(&value);
                let options = Options::from_env().with_params(vec![param!(TemplateValue, "identifier", identifier.as_ref())]);
                search(&options).await
            }
            | None => Err(eyre!("Unsupported publication identifier")),
        }
    }
}
/// Check if API is healthy
/// ### Example
/// ```ignore
/// use acorn_lib::io::api;
///
/// println!("CiteAs API is healthy: {}", api::citeas::is_healthy().await);
/// ```
pub async fn is_healthy() -> bool {
    match status().await {
        | Ok(StatusResponse { msg, .. }) => msg.eq_ignore_ascii_case("Don't panic"),
        | Err(_) => false,
    }
}
/// Perform search on CiteAs API
///
/// The CiteAs API is simple and only has two endpoints. This accesses the endpoint for retrieving citation data for a product identifier.
///
/// ### Example
/// ```ignore
/// use acorn::param;
/// use acorn::io::api::citeas;
///
/// let doi = "10.11578/dc.20250604.1";
/// let options = citeas::Options::from_env()
///     .with_params(vec![param!(TemplateValue, "identifier", doi)]);
/// let citations = citeas::search(&options).await;
/// ```
pub async fn search(options: &Options) -> ApiResult<Citations> {
    let name = "CiteAs";
    let action = "record";
    let params = Params::new().with_custom(options.params()).build();
    let data = Some(params);
    match INCLUDED_ENDPOINTS.find_by_name(name) {
        | Some(endpoint) => {
            let response = endpoint.invoke(action, data).await;
            endpoint.handle::<Citations>(response)
        }
        | None => Err(eyre!("{name} API endpoint not found")),
    }
}
/// Get status of CiteAs API
///
/// See `https://citeas.org/api#api-status-object` for more information
pub async fn status() -> ApiResult<StatusResponse> {
    let name = "CiteAs";
    let action = "status";
    let data = None;
    match INCLUDED_ENDPOINTS.find_by_name(name) {
        | Some(endpoint) => {
            let response = endpoint.invoke(action, data).await;
            endpoint.handle::<StatusResponse>(response)
        }
        | None => Err(eyre!("{name} API endpoint not found")),
    }
}
