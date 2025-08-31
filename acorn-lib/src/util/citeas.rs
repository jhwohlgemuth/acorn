//! Module for communicating with CiteAs API
//!
//! See <https://citeas.org/api> for more information
use serde::{Deserialize, Serialize};

/// Describes status of the CiteAs API
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Status {
    /// Where you can find documentation for this version
    /// ### Example
    /// > "https://citeas.org/api"
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
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    pub citation: String,
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Metadata {
    /// Valid DOI
    pub doi: String,
    /// Resource URL
    pub url: String,
    /// Author objects
    pub author: Vec<String>,
    /// ID for the resource
    /// <div class="warning">Always "ITEM-1"</div>
    pub id: String,
    /// Title of the resource
    pub title: String,
    /// Year of publication
    pub year: u16,
}
///
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Provenance {
    /// URL utilized to discover citation data
    pub content_url: String,
    /// Returns "doi" or "arXiv ID" if found via DOI or arXiv, else "null"
    pub found_via_proxy_type: String,
    /// Returns true if content was found at the URL
    pub has_content: bool,
    /// Host of the resource, such as crossref, github or pypi
    pub host: String,
    /// Name of the step taken to find citation data
    pub name: String,
    /// Name of the parent step
    pub parent_step_name: String,
    /// Name of the parent subject
    pub parent_subject: String,
    /// Subject of the current step
    /// ### Example
    /// > "GitHub repository main page"
    pub subject: String,
}
