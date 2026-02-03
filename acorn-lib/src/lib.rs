//! # 🌱 ACORN Library
//! > "Plant an ACORN and grow your research"
//!
//! `acorn-lib` is a one-stop-shop for everything related to building and maintaining research activity data (RAD)-related technology, including the Accessible Content Optimization for Research Needs (ACORN) tool.
//! The modules, structs, enums and constants found here support the ACORN CLI, which checks, analyzes, and exports research activity data into useable formats.
//!
#[cfg(feature = "std")]
use crate::analyzer::{link_check, Check};
#[cfg(feature = "std")]
use crate::io::network_get_request;
#[cfg(feature = "std")]
use crate::prelude::PathBuf;
use crate::util::Label;
use derive_more::Display;
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use tracing::debug;
use tracing::{error, trace, warn};
use uriparse::URI;
use urlencoding::encode;

#[cfg(feature = "analyzer")]
pub mod analyzer;
#[cfg(feature = "doctor")]
pub mod doctor;
#[cfg(feature = "std")]
pub mod io;
#[cfg(feature = "powerpoint")]
pub mod powerpoint;
pub mod prelude;
pub mod schema;
pub mod util;

/// Abstraction for file and folder locations that can be local (e.g., file:///path/to/project) or remote (e.g., <https://gitlab.com/project>)
#[derive(Clone, Debug, Display, Serialize, Deserialize)]
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
}
/// URI Scheme
///
/// See [RFC 8089] for more information
///
/// [RFC 8089]: https://datatracker.ietf.org/doc/rfc8089/
#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Scheme {
    /// Secure HTTP
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
impl Location {
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
        let uri = self.uri().unwrap();
        let host = match uri.host() {
            | Some(value) => value.clone().to_string().replace('.', "_"),
            | None => "".to_string(),
        };
        let segments = uri
            .path()
            .segments()
            .iter()
            .map(|s| s.to_string())
            .filter(|s| !(s.is_empty() || s.eq(".")))
            .collect::<Vec<_>>();
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
            | Location::Simple(value) => match URI::try_from(value.as_str()) {
                | Ok(uri) => match uri.scheme() {
                    | uriparse::Scheme::HTTPS => Scheme::HTTPS,
                    | uriparse::Scheme::HTTP => Scheme::HTTP,
                    | uriparse::Scheme::File => Scheme::File,
                    | _ => Scheme::Unsupported,
                },
                | Err(_) => Scheme::Unsupported,
            },
            | Location::Detailed { scheme, .. } => scheme.clone(),
        }
    }
    /// Check if a location exists (i.e., is reachable and accessible)
    #[cfg(feature = "std")]
    pub async fn exists(self) -> bool {
        let uri = self.uri();
        let scheme = self.scheme();
        if scheme == Scheme::HTTP {
            warn!("=> {} HTTP is supported but only advised in local development scenarios", Label::skip());
        }
        match scheme {
            | Scheme::HTTPS | Scheme::HTTP => match uri {
                | Some(uri) => match link_check(Some(uri.into())).await {
                    | Check { success, .. } if success => true,
                    | _ => false,
                },
                | None => false,
            },
            | Scheme::File => match uri {
                | Some(value) => PathBuf::from(value.path().to_string()).exists(),
                | None => false,
            },
            | Scheme::Unsupported => false,
        }
    }
    /// Extract and return URI from a location value
    pub fn uri(&self) -> Option<URI<'static>> {
        fn parse_uri(value: String) -> Option<URI<'static>> {
            let leaked: &'static str = Box::leak(value.into_boxed_str());
            match URI::try_from(leaked) {
                | Ok(value) => Some(value),
                | Err(why) => {
                    warn!("=> {} Parse URI - {why}", Label::fail());
                    None
                }
            }
        }
        match self {
            | Location::Simple(value) => parse_uri(value.clone()),
            | Location::Detailed { scheme, uri } => match URI::try_from(uri.as_str()) {
                | Ok(parsed) => {
                    let authority = parsed.authority().map(|auth| auth.to_string());
                    let path = parsed.path().to_string();
                    let query = parsed.query().map(|q| format!("?{q}")).unwrap_or_default();
                    let fragment = parsed.fragment().map(|f| format!("#{f}")).unwrap_or_default();
                    let rebuilt = match authority {
                        | Some(auth) if !auth.is_empty() => format!("{scheme}://{auth}{path}{query}{fragment}"),
                        | _ => format!("{scheme}:{path}{query}{fragment}"),
                    };
                    parse_uri(rebuilt)
                }
                | Err(_) => {
                    let rebuilt = format!("{scheme}://{uri}");
                    parse_uri(rebuilt)
                }
            },
        }
    }
}
impl Repository {
    /// Return whether or not the associated URI for a repository is local (e.g., has "file" scheme)
    pub fn is_local(self) -> bool {
        let local_schemes = [Scheme::File];
        local_schemes.contains(&self.location().scheme())
    }
    /// Get metadata for latest release of a Gitlab or GitHub repository
    #[cfg(feature = "std")]
    pub fn latest_release(self) -> Option<Release> {
        match self.releases() {
            | releases if releases.is_empty() => None,
            | releases => {
                let release = releases[0].clone();
                trace!("=> {} Latest {:#?}", Label::using(), release);
                Some(release)
            }
        }
    }
    /// Get repository location
    pub fn location(self) -> Location {
        match self {
            | Repository::Git { location, .. } => location,
            | Repository::GitHub { location, .. } => location,
            | Repository::GitLab { location, .. } => location,
        }
    }
    /// Get repository ID
    pub fn id(&self) -> Option<String> {
        match self {
            | Repository::Git { .. } => None,
            | Repository::GitHub { .. } => None,
            | Repository::GitLab { id, location } => match location.uri() {
                | Some(value) => {
                    let mut path = value.path().to_string();
                    path.remove(0);
                    let encoded = encode(&path).to_string();
                    trace!(encoded, "=> {} ID", Label::using());
                    Some(encoded)
                }
                | None => {
                    warn!("=> {} Parse GitLab URI", Label::fail());
                    match id {
                        | Some(value) => Some(value.to_string()),
                        | None => None,
                    }
                }
            },
        }
    }
    #[cfg(feature = "std")]
    fn releases(self) -> Vec<Release> {
        let maybe_url = match &self {
            | Repository::Git { .. } => None,
            | Repository::GitHub { location } => match location.uri() {
                | Some(uri) => {
                    let host = uri.host().unwrap().to_string();
                    let path = uri.path();
                    let endpoint = Some(format!("https://api.{host}/repos{path}/releases"));
                    endpoint
                }
                | None => {
                    error!("=> {} Parse GitHub URI", Label::fail());
                    None
                }
            },
            | Repository::GitLab { location, .. } => match self.id() {
                | Some(id) => match location.uri() {
                    | Some(uri) => {
                        let host = uri.host().unwrap().to_string();
                        Some(format!("https://{host}/api/v4/projects/{id}/releases"))
                    }
                    | None => {
                        error!("=> {} Parse GitLab URI", Label::fail());
                        None
                    }
                },
                | None => None,
            },
        };
        if let Some(url) = maybe_url {
            debug!(url, "=> {}", Label::using());
            match network_get_request(url).send() {
                | Ok(response) => {
                    let text = response.text();
                    match text {
                        | Ok(text) => {
                            if text.contains("API rate limit exceeded") {
                                println!("API rate limit exceeded");
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
            | Repository::GitHub { location, .. } => match location.uri() {
                | Some(ref value) => Some(format!("https://raw.githubusercontent.com{}/refs/heads/main/{path}", value.path())),
                | None => {
                    error!("=> {} Parse GitHub URI", Label::fail());
                    None
                }
            },
            | Repository::GitLab { location, .. } => Some(format!("{location}/-/raw/main/{path}")),
            | Repository::Git { .. } => None,
        }
    }
}
