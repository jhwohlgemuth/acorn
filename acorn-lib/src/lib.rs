//! # 🌱 ACORN Library
//! > "Plant an ACORN and grow your research"
//!
//! `acorn-lib` is a one-stop-shop for everything related to building and maintaining research activity data (RAD)-related technology, including the Accessible Content Optimization for Research Needs (ACORN) tool.
//! The modules, structs, enums and constants found here support the ACORN CLI, which checks, analyzes, and exports research activity data into useable formats.
//!
// Policy: `wasm` marks portable allocation-capable APIs. Cargo feature
// unification means tooling such as `--all-features` may combine it with `std`;
// wasm-specific code should use `#[cfg(all(feature = "wasm", not(feature = "std")))]`.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
// Current schema derive macros emit `::std` paths even when host APIs are disabled.
#[cfg(not(feature = "std"))]
extern crate std;

#[doc(hidden)]
#[cfg(feature = "cmd")]
pub use acorn_macros::cmd_sh_words;

use core::convert::Infallible;
use core::str::FromStr;
use derive_more::Display;
use fluent_uri::{Uri, UriRef};
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use tracing::debug;
use tracing::{error, trace, warn};
use urlencoding::encode;

#[cfg(feature = "analysis")]
pub mod analyzer;
#[cfg(feature = "doctor")]
pub mod doctor;
#[cfg(feature = "std")]
pub mod io;
pub mod prelude;
pub mod schema;
pub mod util;

#[cfg(all(feature = "std", feature = "analysis"))]
use crate::analyzer::{link_check, Check};
#[cfg(feature = "std")]
use crate::io::http::get;
use crate::prelude::{format, String, ToString, Vec};
#[cfg(feature = "std")]
use crate::prelude::{Path, PathBuf};
use crate::util::Label;
use strum::EnumIs;

/// **Descriptive/persistent** location reference for use in configuration (buckets, repositories, etc.).
///
/// This is a **data type** meant for storage and serialization — use [`io::Source`] when you
/// actually need to **read** bytes from a local path or URL at runtime.
///
/// Supports both raw URI strings (`Simple`) and structured scheme+URI pairs (`Detailed`).
///
/// See [`io::Source`] for the complementary **operational/transient** type that handles I/O.
#[derive(Clone, Debug, Deserialize, Display, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Location {
    /// Just the URI string (assumes remote location)
    Simple(String),
    /// Location defined by URI and scheme - intended for use with remote or local locations
    #[display("{uri}")]
    Detailed {
        /// URI Scheme
        ///
        /// See [RFC 8089] for more information
        ///
        /// [RFC 8089]: https://datatracker.ietf.org/doc/rfc8089/
        scheme: Scheme,
        /// Full URI value
        uri: String,
        /// Optional branch, tag, or revision for versioned locations
        #[serde(default, skip_serializing_if = "Option::is_none")]
        revision: Option<String>,
    },
}
/// Git hosting repository data
#[derive(Clone, Debug, Display, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum Repository {
    /// Generic Git repository
    /// ### Note
    /// > This repository type should be used for local and offline repositories. Having the associated data be version controlled by Git is recommended, but not required.
    #[display("git")]
    Git {
        /// Repository location information
        #[serde(alias = "uri")]
        location: Location,
    },
    /// GitHub
    ///
    /// See <https://docs.github.com/en/rest/reference/repos>
    #[display("github")]
    GitHub {
        /// Repository location information
        #[serde(alias = "uri")]
        location: Location,
    },
    /// GitLab
    ///
    /// See <https://docs.gitlab.com/api/repositories/#list-repository-tree>
    #[display("gitlab")]
    GitLab {
        /// Integer ID of GitLab project
        ///
        /// See <https://docs.gitlab.com/api/projects/#get-a-single-project> for more information
        id: Option<u64>,
        /// Repository location information
        #[serde(alias = "uri")]
        location: Location,
    },
    /// Hugging Face
    ///
    /// See <https://huggingface.co/docs/hub/repositories-getting-started>
    #[display("huggingface")]
    HuggingFace {
        /// Repository location information
        #[serde(alias = "uri")]
        location: Location,
    },
}
/// URI Scheme
///
/// See [RFC 8089] for more information
///
/// [RFC 8089]: https://datatracker.ietf.org/doc/rfc8089/
#[derive(Clone, Debug, Default, Deserialize, Display, EnumIs, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scheme {
    /// Secure HTTP
    #[default]
    #[display("https")]
    HTTPS,
    /// Insecure HTTP included primarily for contexts necessitating its use (ex., local development)
    #[display("http")]
    HTTP,
    /// Local file or folder
    #[display("file")]
    File,
    /// Unsupported scheme (e.g., insecure, not implemented, etc.)
    Unsupported,
}
/// Struct for release data from GitLab or GitHub
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Release {
    /// Name of release
    pub name: String,
    /// Tag name of release
    /// ### Example
    /// > `v1.0.0`
    pub tag_name: String,
    /// Prose description of release
    #[serde(alias = "body")]
    pub description: String,
    /// Date of release creation
    pub created_at: String,
    /// Date of release publication
    #[serde(alias = "published_at")]
    pub released_at: String,
    /// Release response message
    pub message: Option<String>,
}
impl FromStr for Location {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}
impl From<&str> for Location {
    fn from(s: &str) -> Self {
        match UriRef::parse(s).ok().and_then(|uri| uri.scheme()) {
            | Some(scheme) => Location::Detailed {
                scheme: Scheme::from(scheme.as_str()),
                uri: s.to_string(),
                revision: None,
            },
            | None => Location::Simple(s.to_string()),
        }
    }
}
impl<'a> From<&'a Location> for &'a str {
    fn from(value: &'a Location) -> Self {
        match value {
            | Location::Simple(value) | Location::Detailed { uri: value, .. } => value.as_str(),
        }
    }
}
impl Location {
    /// Returns true when a source string points to a local filesystem path.
    pub fn is_local(&self) -> bool {
        let value = match self {
            | Location::Simple(value) | Location::Detailed { uri: value, .. } => value.trim(),
        };
        let is_file_scheme = match self {
            | Location::Detailed { scheme, .. } => scheme.is_file(),
            | Location::Simple(_) => false,
        };
        let is_local_path = value.starts_with("file:") || value.starts_with("./") || value.starts_with("../") || {
            #[cfg(feature = "std")]
            {
                Path::new(value).is_absolute()
            }
            #[cfg(not(feature = "std"))]
            {
                false
            }
        };
        is_file_scheme || is_local_path
    }
    /// Get associated location hash
    /// > Useful for standardizing file path handling across local and remote contexts
    /// ### Example
    /// ```rust
    /// use acorn::Location;
    ///
    /// let location = Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".to_string());
    /// assert_eq!(location.hash(), "code_ornl_gov_research-enablement_buckets_nssd");
    /// ```
    pub fn hash(&self) -> String {
        let host = self.host().unwrap_or_default().replace('.', "_");
        let segments = self
            .path()
            .map(|p| {
                p.split('/')
                    .filter(|s| !(s.is_empty() || *s == "."))
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        [host, segments.join("_").to_lowercase()]
            .into_iter()
            .filter(|x| !x.is_empty())
            .collect::<Vec<String>>()
            .join("_")
    }
    /// Get associated location value scheme (e.g., https, file, etc.)
    /// ### Example
    /// ```rust
    /// use acorn::{Location, Scheme};
    ///
    /// let location = Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".to_string());
    /// assert_eq!(location.scheme(), Scheme::HTTPS);
    /// let location = Location::Simple("file://localhost/buckets/nssd".to_string());
    /// assert_eq!(location.scheme(), Scheme::File);
    /// ```
    pub fn scheme(&self) -> Scheme {
        match self {
            | Location::Simple(value) => Uri::parse(value.as_str())
                .map(|uri| Scheme::from(uri.scheme().as_str()))
                .unwrap_or(Scheme::Unsupported),
            | Location::Detailed { scheme, .. } => scheme.clone(),
        }
    }
    /// Check if a location exists (i.e., is reachable and accessible)
    #[cfg(all(feature = "std", feature = "analysis"))]
    pub async fn exists(self) -> bool {
        let uri = self.uri();
        let scheme = self.scheme();
        if scheme == Scheme::HTTP {
            warn!("=> {} HTTP is supported but only advised in local development scenarios", Label::skip());
        }
        match scheme {
            | Scheme::HTTPS | Scheme::HTTP => match uri {
                | Some(uri) => match link_check(Some(uri), None).await {
                    | Check { success, .. } if success => true,
                    | _ => false,
                },
                | None => false,
            },
            | Scheme::File => match uri {
                | Some(_) => PathBuf::from(self.path().unwrap_or_default()).exists(),
                | None => false,
            },
            | Scheme::Unsupported => false,
        }
    }
    /// Extract and return URI string from a location value
    pub fn uri(&self) -> Option<String> {
        match self {
            | Location::Simple(value) => Some(value.clone()),
            | Location::Detailed { scheme, uri, .. } => match Uri::parse(uri.as_str()) {
                | Ok(parsed) => {
                    let authority = parsed.authority().map(|auth| auth.as_str().to_string());
                    let path = parsed.path().to_string();
                    let query = parsed.query().map(|q| format!("?{q}")).unwrap_or_default();
                    let fragment = parsed.fragment().map(|f| format!("#{f}")).unwrap_or_default();
                    Some(match authority {
                        | Some(auth) if !auth.is_empty() => format!("{scheme}://{auth}{path}{query}{fragment}"),
                        | _ => format!("{scheme}:{path}{query}{fragment}"),
                    })
                }
                | Err(_) => {
                    warn!("=> {} Parse URI - {uri}", Label::fail());
                    Some(format!("{scheme}://{uri}"))
                }
            },
        }
    }
    /// Get host from location URI
    pub fn host(&self) -> Option<String> {
        match self.uri() {
            | Some(value) => Uri::parse(value.as_str())
                .ok()
                .and_then(|uri| uri.authority().map(|auth| auth.host().to_string())),
            | None => None,
        }
    }
    /// Get path from location URI
    pub fn path(&self) -> Option<String> {
        match self.uri() {
            | Some(value) => Uri::parse(value.as_str()).ok().map(|uri| uri.path().to_string()),
            | None => None,
        }
    }
    /// Get port from location URI
    pub fn port(&self) -> Option<u16> {
        match self.uri() {
            | Some(value) => Uri::parse(value.as_str())
                .ok()
                .and_then(|uri| uri.authority().and_then(|auth| auth.port_to_u16().ok()).flatten()),
            | None => None,
        }
    }
}
impl Default for Repository {
    fn default() -> Self {
        Self::Git {
            location: Location::Simple("file:///".to_string()),
        }
    }
}
impl Repository {
    /// Get repository domain (e.g., "github.com" or "code.ornl.gov")
    pub fn domain(&self) -> Option<String> {
        self.location().host()
    }
    /// Return whether or not the associated URI for a repository is local (e.g., has "file" scheme)
    pub fn is_local(&self) -> bool {
        let local_schemes = [Scheme::File];
        local_schemes.contains(&self.clone().location().scheme())
    }
    /// Get metadata for latest release of a Gitlab or GitHub repository
    #[cfg(feature = "std")]
    pub async fn latest_release(self) -> Option<Release> {
        match self.releases().await {
            | releases if releases.is_empty() => None,
            | releases => match releases.into_iter().next() {
                | Some(release) => {
                    trace!("=> {} Latest {:#?}", Label::using(), release);
                    Some(release)
                }
                | None => None,
            },
        }
    }
    /// Get repository location
    pub fn location(&self) -> Location {
        match self.clone() {
            | Repository::Git { location, .. }
            | Repository::GitHub { location, .. }
            | Repository::GitLab { location, .. }
            | Repository::HuggingFace { location, .. } => location,
        }
    }
    /// Get repository ID
    pub fn id(&self) -> Option<String> {
        match self {
            | Repository::Git { .. } | Repository::GitHub { .. } => None,
            | Repository::HuggingFace { location } => location.path().map(|path| path.trim_start_matches('/').to_string()),
            | Repository::GitLab { id, location } => match id {
                | Some(value) => Some(value.to_string()),
                | None => match location.path() {
                    | Some(path) => match path.strip_prefix('/') {
                        | Some(stripped) if !stripped.is_empty() => {
                            let encoded = encode(stripped).to_string();
                            trace!(encoded, "=> {} ID", Label::using());
                            Some(encoded)
                        }
                        | _ => None,
                    },
                    | None => {
                        warn!("=> {} Parse GitLab URI", Label::fail());
                        None
                    }
                },
            },
        }
    }
    #[cfg(feature = "std")]
    async fn releases(self) -> Vec<Release> {
        let maybe_url = match &self {
            | Repository::Git { .. } | Repository::HuggingFace { .. } => None,
            | Repository::GitHub { location } => {
                let host = location.host();
                let path = location.path();
                match (host, path) {
                    | (Some(host), Some(path)) => Some(format!("https://api.{host}/repos{path}/releases")),
                    | (None, _) => {
                        error!("=> {} Parse GitHub URI host", Label::fail());
                        None
                    }
                    | (_, None) => {
                        error!("=> {} Parse GitHub URI", Label::fail());
                        None
                    }
                }
            }
            | Repository::GitLab { location, .. } => match self.id() {
                | Some(id) => match location.host() {
                    | Some(host) => Some(format!("https://{host}/api/v4/projects/{id}/releases")),
                    | None => {
                        error!("=> {} Parse GitLab URI host", Label::fail());
                        None
                    }
                },
                | None => None,
            },
        };
        if let Some(url) = maybe_url {
            debug!(url, "=> {}", Label::using());
            match get(url).send().await {
                | Ok(response) => {
                    let text = response.text().await;
                    match text {
                        | Ok(text) => {
                            if text.contains("API rate limit exceeded") {
                                error!("=> {} GitHub API rate limit exceeded", Label::fail());
                                vec![]
                            } else {
                                let releases: Vec<Release> = match serde_json::from_str(&text) {
                                    | Ok(values) => values,
                                    | Err(why) => {
                                        error!("=> {} Parse {} API JSON response - {why}", self, Label::fail());
                                        vec![]
                                    }
                                };
                                releases
                            }
                        }
                        | Err(why) => {
                            error!("=> {} Parse {} API text response - {why}", self, Label::fail());
                            vec![]
                        }
                    }
                }
                | Err(why) => {
                    error!("=> {} Download {} releases - {why}", self, Label::fail());
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }
    /// Get URL for raw data of a file at a given path
    pub fn raw_url(&self, path: String) -> Option<String> {
        match self {
            | Repository::GitHub { location, .. } => match location.path() {
                | Some(ref value) => Some(format!("https://raw.githubusercontent.com{value}/refs/heads/main/{path}")),
                | None => {
                    error!("=> {} Parse GitHub URI", Label::fail());
                    None
                }
            },
            | Repository::GitLab { location, .. } => Some(format!("{location}/-/raw/main/{path}")),
            | Repository::Git { .. } | Repository::HuggingFace { .. } => None,
        }
    }
}
impl From<&str> for Scheme {
    fn from(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            | "https" => Scheme::HTTPS,
            | "http" => Scheme::HTTP,
            | "file" => Scheme::File,
            | _ => Scheme::Unsupported,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::arithmetic_side_effects,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::unwrap_used
    )]
    use super::{Location, Repository, Scheme};

    #[test]
    fn test_scheme_from_str() {
        assert_eq!(Scheme::from("https"), Scheme::HTTPS);
        assert_eq!(Scheme::from("HTTP"), Scheme::HTTP);
        assert_eq!(Scheme::from("file"), Scheme::File);
        assert_eq!(Scheme::from("ssh"), Scheme::Unsupported);
    }
    #[test]
    fn test_repository_default_is_local_git() {
        let repository = Repository::default();
        assert!(repository.is_local());
        match repository {
            | Repository::Git { location } => {
                assert_eq!(location.to_string(), "file:///");
            }
            | _ => panic!("Repository default should be Git with local file URI"),
        }
    }
    #[test]
    fn test_repository_id_prefers_explicit_gitlab_id() {
        let repository = Repository::GitLab {
            id: Some(16689),
            location: Location::Simple("https://code.ornl.gov/research-enablement/acorn".to_string()),
        };
        assert_eq!(repository.id(), Some("16689".to_string()));
    }
    #[test]
    fn test_repository_id_falls_back_to_encoded_gitlab_path() {
        let repository = Repository::GitLab {
            id: None,
            location: Location::Simple("https://code.ornl.gov/research-enablement/acorn".to_string()),
        };
        assert_eq!(repository.id(), Some("research-enablement%2Facorn".to_string()));
    }
    #[test]
    fn test_repository_id_returns_none_without_gitlab_id_or_valid_uri() {
        let repository = Repository::GitLab {
            id: None,
            location: Location::Simple("not a uri".to_string()),
        };
        assert_eq!(repository.id(), None);
    }
}

#[cfg(all(test, feature = "std"))]
mod test;
