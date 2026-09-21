//! Portable location and repository values.
use crate::{
    prelude::alloc::{format, Box, String, ToString, Vec},
    validation::digest,
};
use bon::bon;
use core::{convert::Infallible, str::FromStr};
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
    /// Secure Shell endpoint.
    #[display("ssh")]
    SSH,
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
impl<'a> From<&'a Location> for &'a str {
    fn from(value: &'a Location) -> Self {
        match value {
            | Location::Simple(value) | Location::Detailed { uri: value, .. } => value,
        }
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
impl FromStr for Location {
    type Err = Infallible;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(value))
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
        matches!(self, Self::Detailed { scheme: Scheme::File, .. }) || lexical || self.is_absolute()
    }
    /// Return whether this location is a local path, including an unprefixed relative path.
    pub fn is_path(&self) -> bool {
        let value: &str = self.into();
        let has_uri_scheme = UriRef::parse(value).ok().and_then(|uri| uri.scheme()).is_some();
        self.is_local() || !(has_uri_scheme || self.is_oci_reference())
    }
    /// Return whether this location is an absolute POSIX or Windows path.
    pub fn is_absolute(&self) -> bool {
        let value = <&str>::from(self);
        #[cfg(feature = "std")]
        let platform_absolute = std::path::Path::new(value).is_absolute();
        #[cfg(not(feature = "std"))]
        let platform_absolute = value.starts_with('/');
        let bytes = value.as_bytes();
        let windows_absolute =
            value.starts_with('\\') || matches!((bytes.first(), bytes.get(1)), (Some(first), Some(b':')) if first.is_ascii_alphabetic());
        platform_absolute || windows_absolute
    }
    /// Return whether this location uses the S3 URI scheme.
    pub fn is_s3(&self) -> bool {
        has_named_uri_scheme(self.into(), "s3")
    }
    /// Return whether this location is a syntactically valid OCI artifact reference.
    pub fn is_oci_reference(&self) -> bool {
        let value: &str = self.into();
        let has_reference_marker = value.contains(':') || value.contains('@');
        let selected = value.split(',').next().unwrap_or_default();
        let (repository, reference, digest_reference) = match selected.rsplit_once('@') {
            | Some((repository, reference)) => {
                let slash = repository.rfind('/').and_then(|index| index.checked_add(1)).unwrap_or(0);
                let repository = repository
                    .get(slash..)
                    .and_then(|suffix| suffix.rfind(':'))
                    .and_then(|index| slash.checked_add(index))
                    .and_then(|boundary| repository.get(..boundary))
                    .unwrap_or(repository);
                (repository, reference, true)
            }
            | None => {
                let slash = selected.rfind('/').and_then(|index| index.checked_add(1)).unwrap_or(0);
                selected
                    .get(slash..)
                    .and_then(|suffix| suffix.rfind(':'))
                    .and_then(|index| slash.checked_add(index))
                    .and_then(|boundary| boundary.checked_add(1).map(|start| (boundary, start)))
                    .and_then(|(boundary, start)| selected.get(..boundary).zip(selected.get(start..)))
                    .map_or((selected, "", false), |(repository, reference)| (repository, reference, false))
            }
        };
        let repository = repository.split_once('/').map_or(repository, |(first, rest)| {
            match first.contains('.') || first.contains(':') || first == "localhost" {
                | true => rest,
                | false => repository,
            }
        });
        let lowercase = repository == repository.to_ascii_lowercase();
        let valid_parts = repository.split('/').all(|part| {
            let valid_bytes = part
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-'));
            let valid_start = part.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric);
            let valid_end = part.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric);
            !part.is_empty() && valid_bytes && valid_start && valid_end
        });
        let valid_repository = !repository.is_empty() && lowercase && valid_parts;
        let valid_reference = match (reference.is_empty(), digest_reference) {
            | (true, false) => selected
                .split_once('/')
                .is_some_and(|(registry, repository)| registry.contains(':') && !repository.is_empty()),
            | (false, true) => digest(reference).is_ok(),
            | (false, false) => {
                !reference.is_empty()
                    && reference.len() <= 128
                    && reference
                        .bytes()
                        .enumerate()
                        .all(|(index, byte)| byte.is_ascii_alphanumeric() || byte == b'_' || (index > 0 && matches!(byte, b'.' | b'-')))
            }
            | (true, true) => false,
        };
        has_reference_marker && (digest(value).is_ok() || (valid_repository && valid_reference))
    }
    /// Normalize path separators and lexical `.` and `..` components.
    pub fn normalize(&self) -> String {
        let value: &str = self.into();
        let replaced = value.replace('\\', "/");
        let segments = replaced.split('/').fold(Vec::new(), |mut values: Vec<&str>, segment| {
            match segment {
                | "" | "." => (),
                | ".." if values.last().is_some_and(|value| *value != "..") => {
                    values.pop();
                }
                | _ => values.push(segment),
            }
            values
        });
        match segments.is_empty() {
            | true => ".".to_string(),
            | false => segments.join("/"),
        }
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
#[bon]
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
    /// Build a GitLab repository reference.
    #[builder]
    pub fn gitlab(id: Option<u64>, location: Location) -> Self {
        Self::GitLab {
            id,
            location: Box::new(location),
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
            | "ssh" => Self::SSH,
            | _ => Self::Unsupported,
        }
    }
}
fn has_named_uri_scheme(value: &str, expected: &str) -> bool {
    UriRef::parse(value)
        .ok()
        .and_then(|uri| uri.scheme().map(|scheme| scheme.as_str().eq_ignore_ascii_case(expected)))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_repository_is_local_git() {
        assert!(Repository::default().is_local());
    }
    #[test]
    fn test_location_classifies_and_normalizes_portable_paths_and_oci_references() {
        assert!(Location::from("model.gguf").is_path());
        assert!(Location::from("C:\\models\\model.gguf").is_absolute());
        assert_eq!(Location::from("docs/../README.md").normalize(), "README.md");
        assert!(Location::from("s3://bucket/key").is_s3());
        assert!(Location::from("registry.example.org/org/model:latest").is_oci_reference());
        assert!(!Location::from("https://example.org/model").is_local());
    }
    #[test]
    fn test_location_preserves_registry_ports_and_ipv6() {
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
    fn test_location_preserves_ssh_uri_components_and_serialization() {
        let value = "ssh://git@example.org:2222/group/project.git";
        let location = Location::from(value);
        assert_eq!(location.scheme(), Scheme::SSH);
        assert_eq!(location.uri().as_deref(), Some(value));
        assert_eq!(location.host().as_deref(), Some("example.org"));
        assert_eq!(location.port(), Some(2222));
        assert_eq!(location.path().as_deref(), Some("/group/project.git"));
        let serialized = serde_json::to_string(&location).expect("location serializes");
        let round_trip: Location = serde_json::from_str(&serialized).expect("location deserializes");
        assert_eq!(round_trip, location);
    }
    #[test]
    fn test_location_round_trips_supported_uri_schemes() {
        [
            ("file:///tmp/acorn.json", "file:/tmp/acorn.json", Scheme::File),
            ("http://user@example.org:8080/data", "http://user@example.org:8080/data", Scheme::HTTP),
            ("https://user@example.org:8443/data", "https://user@example.org:8443/data", Scheme::HTTPS),
            (
                "ssh://git@example.org:2222/group/project.git",
                "ssh://git@example.org:2222/group/project.git",
                Scheme::SSH,
            ),
        ]
        .into_iter()
        .for_each(|(value, canonical, scheme)| {
            let location = Location::from(value);
            assert_eq!(location.scheme(), scheme);
            assert_eq!(location.uri().as_deref(), Some(canonical));
        });
        assert_eq!(serde_json::to_string(&Scheme::SSH).expect("serialize SSH scheme"), "\"ssh\"");
        assert_eq!(serde_json::from_str::<Scheme>("\"ssh\"").expect("deserialize SSH scheme"), Scheme::SSH);
    }
    #[test]
    fn test_repository_classification_requires_provider_domain() {
        let github = Repository::from_remote("https://github.com/openai/codex", "gitlab.com").expect("GitHub repository");
        assert!(github.is_git_hub());
        assert_eq!(github.project_path().as_deref(), Some("openai/codex"));
        assert!(Repository::from_remote("https://example.org/group/project", "gitlab.com").is_none());
    }
    #[test]
    fn test_repository_gitlab_builder() {
        let repository = Repository::gitlab().location(Location::from("https://gitlab.com/group/project")).call();
        assert!(repository.is_git_lab());
        assert_eq!(repository.project_path().as_deref(), Some("group/project"));
    }
}
