//! Source readers for local paths, file URIs, and remote HTTP(S) URLs.
use crate::io::{http, uri_to_path, ApiResult};
use crate::prelude::{read, PathBuf};
use crate::{Location, Scheme};
use color_eyre::eyre::eyre;

/// **Operational/transient** source — a one-shot parse-and-read type for I/O operations.
///
/// This is the complementary counterpart to [`Location`]: whereas `Location` is a descriptive
/// data type meant for configuration and serialization, `Source` is a lightweight runtime type
/// that drives actual byte reads from disk or HTTP(S).
///
/// Prefer to parse user-provided source strings with [`Source::read`] / [`Source::read_bytes`],
/// and reserve [`Location`] for stored configuration values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// Local filesystem path.
    Local(PathBuf),
    /// Remote HTTP(S) URL.
    Remote(String),
    /// URI scheme that cannot be read as an ACORN source.
    Unsupported(String),
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
        read_parsed_bytes(Self::parse(source), offline).await
    }
    /// Parses a user-provided source string into a source location.
    ///
    /// Delegates URI scheme detection to [`Location::from_str`].
    pub fn parse(source: &str) -> Self {
        // Infallible — Location::from_str always succeeds (Err = Infallible)
        let location: Location = source.parse().expect("Location::from_str is infallible");
        match location {
            | Location::Detailed { scheme: Scheme::File, .. } => {
                let local_source = source
                    .strip_prefix("file://localhost/")
                    .map(|value| format!("file:///{value}"))
                    .unwrap_or_else(|| source.to_string());
                Self::Local(uri_to_path(PathBuf::from(local_source)))
            }
            | Location::Detailed {
                scheme: Scheme::HTTPS | Scheme::HTTP,
                ..
            } => Self::Remote(source.to_string()),
            | Location::Detailed {
                scheme: Scheme::Unsupported, ..
            } => Self::Unsupported(source.to_string()),
            | Location::Simple(_) => Self::Local(PathBuf::from(source)),
        }
    }
}
impl From<Location> for Source {
    fn from(location: Location) -> Self {
        let scheme = location.scheme();
        let uri = location.uri().unwrap_or_default();
        match scheme {
            | Scheme::File => Self::Local(uri_to_path(PathBuf::from(uri))),
            | Scheme::HTTPS | Scheme::HTTP => Self::Remote(uri),
            | Scheme::Unsupported => Self::Unsupported(uri),
        }
    }
}
async fn read_parsed_bytes(source: Source, offline: bool) -> ApiResult<Vec<u8>> {
    match source {
        | Source::Local(path) => read(path).map_err(|why| eyre!("Failed to read source — {why}")),
        | Source::Remote(url) => read_remote_source_bytes(&url, offline).await,
        | Source::Unsupported(scheme) => Err(eyre!("Unsupported source URI scheme '{scheme}'")),
    }
}

async fn read_remote_source_bytes(url: &str, offline: bool) -> ApiResult<Vec<u8>> {
    match offline {
        | true => Err(eyre!("Cannot read remote source while offline")),
        | false => http::response_body_bytes(http::get(url).send().await, "Failed to download source").await,
    }
}
