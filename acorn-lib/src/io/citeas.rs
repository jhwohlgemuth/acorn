//! Module for communicating with CiteAs API
//! > CiteAs is a way to get the correct citation for diverse research products including, software, datasets, preprints, and traditional articles. By making it easier to cite software and other "alternative" scholarly products, we aim to help the creators of such products get full credit for their work.
//!
//! See <https://citeas.org/api> for more information
use crate::io::network_get_request;
use crate::schema::pid::DOI;
use crate::util::Label;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::{debug, error};

/// Trait for objects that can be converted to citations (e.g., `DOI`)
pub trait ToCitations {
    /// Convert object to `Citations`
    fn to_citations(&self) -> Citations;
}
/// Author object
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Author {
    /// First name
    pub given: String,
    /// Last name
    pub family: String,
}
/// Describes status of the CiteAs API
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Status {
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
/// Get status of CiteAs API
pub fn status() -> Option<Status> {
    let url = "https://api.citeas.org?email=research@ornl.gov";
    debug!(url, "=> {}", Label::using());
    match network_get_request(url).send() {
        | Ok(response) => {
            let content: serde_json::Result<Status> = serde_json::from_str(&response.text().unwrap_or_default());
            match content {
                | Ok(status) => {
                    debug!("=> {} Status", Label::using());
                    Some(status)
                }
                | Err(why) => {
                    error!("=> {} Parse API status response - {why}", Label::fail());
                    None
                }
            }
        }
        | Err(why) => {
            error!("{} Get API status - {why}", Label::fail());
            None
        }
    }
}
impl Citations {
    /// Use CiteAs API to get citation data from DOI value
    pub fn from_doi(value: &str) -> Citations {
        let url = format!("https://api.citeas.org/product/{value}?email=research@ornl.gov");
        match network_get_request(url).send() {
            | Ok(response) => {
                let content: serde_json::Result<Citations> = serde_json::from_str(&response.text().unwrap());
                match content {
                    | Ok(results) => {
                        debug!("=> {} Response", Label::using());
                        results
                    }
                    | Err(why) => {
                        error!("=> {} Parse API status response - {why}", Label::fail());
                        Citations::default()
                    }
                }
            }
            | Err(why) => {
                error!("{} Get Citations data - {why}", Label::fail());
                Citations::default()
            }
        }
    }
    /// Get citation data with given citation style (ex. "APA")
    ///
    /// If citation with desired style is not found, will return first citation
    pub fn match_style(self, value: &str) -> Option<Citation> {
        let citations = self.citations;
        let result = citations
            .iter()
            .find(|&citation| citation.style_shortname.to_lowercase() == value.to_lowercase());
        match result {
            | Some(citation) => Some(citation.clone()),
            | None => {
                if citations.is_empty() {
                    None
                } else {
                    match citations.first() {
                        | Some(citation) => Some(citation.clone()),
                        | None => None,
                    }
                }
            }
        }
    }
}
impl DOI {
    /// Convert a `DOI` to a `Citations`
    pub fn to_citations(self) -> Citations {
        Citations::from_doi(&self.to_string())
    }
}
