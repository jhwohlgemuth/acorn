//! Portable location and repository values.
use crate::prelude::alloc::{format, Box, String, ToString, Vec};
use core::convert::Infallible;
use core::str::FromStr;
use derive_more::Display;
use fluent_uri::{Uri, UriRef};
use serde::{Deserialize, Serialize};
use strum::EnumIs;
use urlencoding::encode;

/// Descriptive location reference for configuration and serialization.
#[derive(Clone, Debug, Deserialize, Display, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Location {
    /// Raw URI or path string.
    Simple(String),
    /// Structured location.
    #[display("{uri}")]
    Detailed {
        /// URI scheme.
        scheme: Scheme,
        /// Full URI value.
        uri: String,
        /// Optional branch, tag, or revision.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        revision: Option<String>,
    },
}
/// Git hosting repository data.
#[derive(Clone, Debug, Display, EnumIs, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum Repository {
    /// Generic Git repository.
    #[display("git")]
    Git {
        /// Repository location.
        #[serde(alias = "uri")]
        location: Box<Location>,
    },
    /// GitHub repository.
    #[display("github")]
    GitHub {
        /// Repository location.
        #[serde(alias = "uri")]
        location: Box<Location>,
    },
    /// GitLab repository.
    #[display("gitlab")]
    GitLab {
        /// Numeric GitLab project ID.
        id: Option<u64>,
        /// Repository location.
        #[serde(alias = "uri")]
        location: Box<Location>,
    },
    /// Hugging Face repository.
    #[display("huggingface")]
    HuggingFace {
        /// Repository location.
        #[serde(alias = "uri")]
        location: Box<Location>,
    },
    /// OCI artifact registry.
    #[display("oci")]
    OCI {
        /// OCI artifact location.
        #[serde(alias = "uri")]
        location: Box<Location>,
        /// Optional named registry profile.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        registry: Option<String>,
    },
}
/// URI scheme.
#[derive(Clone, Debug, Default, Deserialize, Display, EnumIs, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scheme {
    /// Secure HTTP.
    #[default]
    #[display("https")]
    HTTPS,
    /// Insecure HTTP.
    #[display("http")]
    HTTP,
    /// Local file or folder.
    #[display("file")]
    File,
    /// Unsupported scheme.
    Unsupported,
}
/// Release data returned by repository providers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Release {
    /// Release name.
    pub name: String,
    /// Release tag.
    pub tag_name: String,
    /// Release description.
    #[serde(alias = "body")]
    pub description: String,
    /// Creation date.
    pub created_at: String,
    /// Publication date.
    #[serde(alias = "published_at")]
    pub released_at: String,
    /// Provider response message.
    pub message: Option<String>,
}
impl FromStr for Location {
    type Err = Infallible;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(value))
    }
}
impl From<&str> for Location {
    fn from(value: &str) -> Self {
        match UriRef::parse(value).ok().and_then(|uri| uri.scheme()) {
            | Some(scheme) => Self::Detailed {
                scheme: Scheme::from(scheme.as_str()),
                uri: value.to_string(),
                revision: None,
            },
            | None => Self::Simple(value.to_string()),
        }
    }
}
impl From<&String> for Location {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}
impl<'a> From<&'a Location> for &'a str {
    fn from(value: &'a Location) -> Self {
        match value {
            | Location::Simple(value) | Location::Detailed { uri: value, .. } => value,
        }
    }
}
impl Location {
    /// Return whether this location uses HTTP or HTTPS.
    pub fn is_http_or_https(&self) -> bool {
        matches!(self.scheme(), Scheme::HTTP | Scheme::HTTPS)
    }
    /// Return whether this location represents a local path.
    pub fn is_local(&self) -> bool {
        let value = <&str>::from(self).trim();
        let lexical = value.starts_with("file:") || value.starts_with("./") || value.starts_with("../");
        #[cfg(feature = "std")]
        let absolute = std::path::Path::new(value).is_absolute();
        #[cfg(not(feature = "std"))]
        let absolute = false;
        matches!(self, Self::Detailed { scheme: Scheme::File, .. }) || lexical || absolute
    }
    /// Get a stable location hash suitable for path components.
    pub fn hash(&self) -> String {
        let host = self.host().unwrap_or_default().replace('.', "_");
        let segments = self
            .path()
            .map(|path| {
                path.split('/')
                    .filter(|segment| !(segment.is_empty() || *segment == "."))
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        [host, segments.join("_").to_lowercase()]
            .into_iter()
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join("_")
    }
    /// Get the location scheme.
    pub fn scheme(&self) -> Scheme {
        match self {
            | Self::Simple(value) => Uri::parse(value.as_str())
                .map(|uri| Scheme::from(uri.scheme().as_str()))
                .unwrap_or(Scheme::Unsupported),
            | Self::Detailed { scheme, .. } => scheme.clone(),
        }
    }
    /// Return the normalized URI string.
    pub fn uri(&self) -> Option<String> {
        match self {
            | Self::Simple(value) => Some(value.clone()),
            | Self::Detailed { scheme, uri, .. } => Uri::parse(uri.as_str()).ok().map_or_else(
                || Some(format!("{scheme}://{uri}")),
                |parsed| {
                    let authority = parsed.authority().map(|value| value.as_str().to_string());
                    let path = parsed.path();
                    let query = parsed.query().map(|value| format!("?{value}")).unwrap_or_default();
                    let fragment = parsed.fragment().map(|value| format!("#{value}")).unwrap_or_default();
                    Some(match authority {
                        | Some(authority) if !authority.is_empty() => format!("{scheme}://{authority}{path}{query}{fragment}"),
                        | _ => format!("{scheme}:{path}{query}{fragment}"),
                    })
                },
            ),
        }
    }
    /// Get the URI host.
    pub fn host(&self) -> Option<String> {
        self.uri().and_then(|value| {
            Uri::parse(value.as_str())
                .ok()
                .and_then(|uri| uri.authority().map(|auth| auth.host().to_string()))
        })
    }
    /// Get registry authority, preserving ports and IPv6 brackets.
    pub fn registry(&self) -> Option<String> {
        self.host().map(|host| match self.port() {
            | Some(port) if host.starts_with('[') => format!("{host}:{port}"),
            | Some(port) if host.contains(':') => format!("[{host}]:{port}"),
            | Some(port) => format!("{host}:{port}"),
            | None if host.starts_with('[') => host,
            | None if host.contains(':') => format!("[{host}]"),
            | None => host,
        })
    }
    /// Get the URI path.
    pub fn path(&self) -> Option<String> {
        self.uri()
            .and_then(|value| Uri::parse(value.as_str()).ok().map(|uri| uri.path().to_string()))
    }
    /// Get the URI port.
    pub fn port(&self) -> Option<u16> {
        self.uri().and_then(|value| {
            Uri::parse(value.as_str())
                .ok()
                .and_then(|uri| uri.authority().and_then(|auth| auth.port_to_u16().ok()).flatten())
        })
    }
}
impl Default for Repository {
    fn default() -> Self {
        Self::Git {
            location: Box::new(Location::Simple("file:///".to_string())),
        }
    }
}
impl Repository {
    /// Classify a repository URL by provider domain.
    pub fn from_remote(value: &str, domain: &str) -> Option<Self> {
        let location = Location::from(value);
        let host = location.host().map(|host| host.trim_start_matches("www.").to_ascii_lowercase());
        let configured = Location::from(
            if domain.contains("://") {
                domain.to_string()
            } else {
                format!("https://{domain}")
            }
            .as_str(),
        );
        let is_gitlab = configured.host() == location.host() && configured.port() == location.port();
        match (location.scheme(), host.as_deref()) {
            | (Scheme::HTTP | Scheme::HTTPS, Some("github.com")) => Some(Self::GitHub {
                location: Box::new(location),
            }),
            | (Scheme::HTTP | Scheme::HTTPS, Some("huggingface.co")) => Some(Self::HuggingFace {
                location: Box::new(location),
            }),
            | (Scheme::HTTP | Scheme::HTTPS, Some(_)) if is_gitlab => Some(Self::GitLab {
                id: None,
                location: Box::new(location),
            }),
            | _ => None,
        }
    }
    /// Get repository domain.
    pub fn domain(&self) -> Option<String> {
        self.location().host()
    }
    /// Return whether the repository is local.
    pub fn is_local(&self) -> bool {
        self.location().is_local()
    }
    /// Get repository location.
    pub fn location(&self) -> Location {
        match self.clone() {
            | Self::Git { location }
            | Self::GitHub { location }
            | Self::GitLab { location, .. }
            | Self::HuggingFace { location }
            | Self::OCI { location, .. } => *location,
        }
    }
    /// Get provider-native repository ID.
    pub fn id(&self) -> Option<String> {
        match self {
            | Self::Git { .. } | Self::GitHub { .. } | Self::OCI { .. } => None,
            | Self::HuggingFace { location } => location.path().map(|path| path.trim_start_matches('/').to_string()),
            | Self::GitLab { id: Some(id), .. } => Some(id.to_string()),
            | Self::GitLab { id: None, location } => location.path().and_then(|path| {
                path.strip_prefix('/')
                    .filter(|path| !path.is_empty())
                    .map(|path| encode(path).to_string())
            }),
        }
    }
    /// Return the provider-native project path.
    pub fn project_path(&self) -> Option<String> {
        match self {
            | Self::GitHub { location } | Self::HuggingFace { location } => location.path().and_then(|path| {
                let mut segments = path.trim_matches('/').split('/');
                segments
                    .next()
                    .zip(segments.next())
                    .map(|(owner, repository)| format!("{owner}/{}", repository.trim_end_matches(".git")))
            }),
            | Self::GitLab { id: Some(id), .. } => Some(id.to_string()),
            | Self::GitLab { id: None, location } => location.path().and_then(|path| {
                let project = path.trim_matches('/').split("/-/").next().unwrap_or_default().trim_end_matches(".git");
                (project.split('/').count() >= 2).then(|| project.to_string())
            }),
            | Self::Git { .. } | Self::OCI { .. } => None,
        }
    }
    /// Get a raw GitHub file URL.
    pub fn raw_url(&self, path: String) -> Option<String> {
        match self {
            | Self::GitHub { location } => location
                .path()
                .map(|value| format!("https://raw.githubusercontent.com{value}/refs/heads/main/{path}")),
            | _ => None,
        }
    }
}
impl From<&str> for Scheme {
    fn from(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            | "https" => Self::HTTPS,
            | "http" => Self::HTTP,
            | "file" => Self::File,
            | _ => Self::Unsupported,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_preserves_registry_ports_and_ipv6() {
        assert_eq!(
            Location::from("https://registry.example.org:5000").registry().as_deref(),
            Some("registry.example.org:5000")
        );
        assert_eq!(
            Location::from("https://[2001:db8::1]:5000").registry().as_deref(),
            Some("[2001:db8::1]:5000")
        );
    }
    #[test]
    fn repository_classification_requires_provider_domain() {
        let github = Repository::from_remote("https://github.com/openai/codex", "gitlab.com").expect("GitHub repository");
        assert!(github.is_git_hub());
        assert_eq!(github.project_path().as_deref(), Some("openai/codex"));
        assert!(Repository::from_remote("https://example.org/group/project", "gitlab.com").is_none());
    }
    #[test]
    fn default_repository_is_local_git() {
        assert!(Repository::default().is_local());
    }
}
