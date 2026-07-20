//! Source readers for local paths, file URIs, and remote HTTP(S) URLs.
use crate::io::{http, symlink, uri_to_path, ApiResult};
use crate::prelude::{copy, read, Path, PathBuf};
use crate::schema::agent::ModelDetails;
use crate::util::Label;
use crate::{Location, Repository, Scheme};
use color_eyre::eyre::eyre;
use core::fmt;
use strum::EnumIs;
use tracing::error;

/// **Operational/transient** source — a one-shot parse-and-read type for I/O operations.
///
/// This is the complementary counterpart to [`Location`]: whereas `Location` is a descriptive
/// data type meant for configuration and serialization, `Source` is a lightweight runtime type
/// that drives actual byte reads from disk or HTTP(S).
///
/// Prefer to parse user-provided source strings with [`Source::read`] / [`Source::read_bytes`],
/// and reserve [`Location`] for stored configuration values.
#[derive(Clone, Debug, PartialEq, Eq, EnumIs)]
pub enum Source {
    /// Local filesystem path.
    Local {
        /// Optional display name for the source
        name: Option<String>,
        /// Local filesystem path
        path: PathBuf,
        /// Optional action to use when materializing the source
        action: Option<SourceAction>,
    },
    /// Remote HTTP(S) URL
    Remote {
        /// Optional display name for the source
        name: Option<String>,
        /// Remote identifier, repository ID, or URL
        identifier: String,
    },
    /// URI scheme that cannot be read as an ACORN source
    Unsupported(String),
}
/// Action to use when materializing a local source into an output directory
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SourceAction {
    /// Reference the source in place
    #[default]
    Reference,
    /// Copy the source into the output directory
    Copy,
    /// Create a symlink to the source in the output directory
    Symlink,
}
impl SourceAction {
    /// Resolve CLI `--copy` / `--symlink` flags into a source action.
    pub fn from_options(copy: bool, symlink: bool) -> Option<Self> {
        match (copy, symlink) {
            | (true, false) => Some(Self::Copy),
            | (false, true) => Some(Self::Symlink),
            | _ => None,
        }
    }
    /// Materialize ("reify" or "persist") source path locally according to a given action
    pub fn materialize(self, path: &Path, target: &Path, name: &str) -> ApiResult<String> {
        let result = match self {
            | SourceAction::Copy if target.exists() => Ok(format!("Skipping copy - '{name}' already exists at {}", target.display())),
            | SourceAction::Copy => match copy(path, target) {
                | Ok(_) => Ok(format!("Copied local model '{name}' -> {}", target.display())),
                | Err(why) => {
                    error!("=> {} Copy local model '{}' to {} - {why}", Label::fail(), name, target.display());
                    Err(why.into())
                }
            },
            | SourceAction::Symlink if target.exists() || target.symlink_metadata().map(|metadata| metadata.is_symlink()).unwrap_or(false) => {
                Ok(format!("Skipping symlink - '{name}' already exists at {}", target.display()))
            }
            | SourceAction::Symlink => match symlink(path, target) {
                | Ok(_) => Ok(format!("Symlinked local model '{name}' -> {}", target.display())),
                | Err(why) => {
                    error!("=> {} Symlink local model '{}' to {} - {why}", Label::fail(), name, target.display());
                    Err(why)
                }
            },
            | SourceAction::Reference => Ok(format!("Local model '{name}' referenced in place at {}", path.display())),
        };
        result
    }
}
impl Source {
    /// Reads source content from a URL or local file path.
    ///
    /// When `source` starts with `http://` or `https://`, the content is downloaded.
    /// Otherwise, the source is treated as a local file path and read from disk.
    ///
    /// Returns an error if URL access is requested while `offline` is enabled.
    pub async fn read(source: &str, offline: bool) -> ApiResult<String> {
        Self::read_bytes(source, offline)
            .await
            .and_then(|bytes| String::from_utf8(bytes).map_err(|why| eyre!("Failed to decode source as UTF-8 — {why}")))
    }
    /// Reads source bytes from a URL, file URI, or local file path.
    ///
    /// HTTP(S) sources are rejected when `offline` is enabled.
    pub async fn read_bytes(source: &str, offline: bool) -> ApiResult<Vec<u8>> {
        Source::read_parsed_bytes(Self::parse(source), offline).await
    }
    /// Parses a user-provided source string into a source location.
    /// ### Note
    /// Delegates URI scheme detection to [`Location::from_str`].
    pub fn parse(source: &str) -> Self {
        // Infallible — Location::from_str always succeeds (Err = Infallible)
        let location: Location = source.parse().expect("Location::from_str is infallible");
        match location {
            | Location::Detailed { scheme: Scheme::File, .. } => {
                let path = uri_to_path(source);
                Self::Local {
                    name: None,
                    path,
                    action: None,
                }
            }
            | Location::Detailed {
                scheme: Scheme::HTTPS | Scheme::HTTP,
                ..
            } => Self::Remote {
                name: None,
                identifier: source.to_string(),
            },
            | Location::Detailed {
                scheme: Scheme::Unsupported, ..
            } => Self::Unsupported(source.to_string()),
            | Location::Simple(_) => Self::Local {
                name: None,
                path: PathBuf::from(source),
                action: None,
            },
        }
    }
    /// Return the user-facing source name.
    pub fn name(&self) -> String {
        match self {
            | Source::Local { name: Some(name), .. } | Source::Remote { name: Some(name), .. } => name.clone(),
            | Source::Local { path, .. } => path.file_stem().and_then(|s| s.to_str()).unwrap_or("model").to_string(),
            | Source::Remote { identifier, .. } | Source::Unsupported(identifier) => identifier.clone(),
        }
    }
    /// Return a stable identifier for deduplication or output paths.
    pub fn identifier(&self) -> String {
        match self {
            | Source::Local { path, .. } => path.display().to_string(),
            | Source::Remote { identifier, .. } | Source::Unsupported(identifier) => identifier.clone(),
        }
    }
    /// Return a source with a materialization action.
    pub fn with_action(self, action: Option<SourceAction>) -> Self {
        match self {
            | Source::Local { name, path, .. } => Source::Local { name, path, action },
            | other => other,
        }
    }
    /// Return a source with a display name.
    pub fn with_name(self, value: impl Into<String>) -> Self {
        let binding = value.into();
        let trimmed = binding.trim();
        let name = (!trimmed.is_empty()).then(|| trimmed.to_string());
        match self {
            | Source::Local { path, action, .. } => Source::Local { name, path, action },
            | Source::Remote { identifier, .. } => Source::Remote { name, identifier },
            | Source::Unsupported(value) => Source::Unsupported(value),
        }
    }
    async fn read_parsed_bytes(source: Source, offline: bool) -> ApiResult<Vec<u8>> {
        match source {
            | Source::Local { path, .. } => read(path).map_err(|why| eyre!("Failed to read source — {why}")),
            | Source::Remote { identifier, .. } => Source::read_remote_bytes(&identifier, offline).await,
            | Source::Unsupported(scheme) => Err(eyre!("Unsupported source URI scheme '{scheme}'")),
        }
    }
    async fn read_remote_bytes(url: &str, offline: bool) -> ApiResult<Vec<u8>> {
        match offline {
            | true => Err(eyre!("Cannot read remote source while offline")),
            | false => http::response_body_bytes(http::get(url).send().await, "Failed to download source").await,
        }
    }
}
impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name();
        let identifier = self.identifier();
        if name == identifier {
            write!(f, "{name}")
        } else {
            write!(f, "{name} ({identifier})")
        }
    }
}
impl From<&str> for Source {
    fn from(selector: &str) -> Self {
        let trimmed = selector.trim();
        let location = Location::from(trimmed);
        if location.is_local() {
            let path = uri_to_path(trimmed);
            Self::Local {
                name: None,
                path,
                action: None,
            }
        } else {
            Self::Remote {
                name: Some(trimmed.to_string()),
                identifier: trimmed.to_string(),
            }
        }
    }
}
impl From<Location> for Source {
    fn from(location: Location) -> Self {
        let scheme = location.scheme();
        let uri = location.uri().unwrap_or_default();
        match scheme {
            | Scheme::File => Self::Local {
                name: None,
                path: uri_to_path(&uri),
                action: None,
            },
            | Scheme::HTTPS | Scheme::HTTP => Self::Remote { name: None, identifier: uri },
            | Scheme::Unsupported => Self::Unsupported(uri),
        }
    }
}
impl From<&Repository> for Source {
    fn from(repository: &Repository) -> Self {
        match repository {
            | Repository::HuggingFace { location } => Self::Remote {
                name: None,
                identifier: repository.id().unwrap_or_else(|| location.uri().unwrap_or_default()),
            },
            | _ => Self::from(repository.location()),
        }
    }
}
impl From<ModelDetails> for Option<Source> {
    fn from(details: ModelDetails) -> Self {
        details.weights.and_then(|weights| weights.to_source(details.name.or(details.id)))
    }
}
