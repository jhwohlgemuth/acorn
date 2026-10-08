//! # ACORN host library
//!
//! This crate provides ACORN services and host adapters. Portable primitives and
//! schemas are owned by `acorn-core` and `acorn-schema`, respectively.
#![cfg_attr(not(any(feature = "acp", feature = "analysis", feature = "std")), no_std)]

#[cfg(any(feature = "acp", feature = "analysis", feature = "mcp-client"))]
extern crate alloc;
#[cfg(any(feature = "acp", feature = "analysis"))]
pub mod agent;
#[cfg(feature = "analysis")]
pub mod analyzer;
pub mod error;
#[cfg(feature = "exec")]
pub mod exec;
#[cfg(feature = "analysis")]
pub mod io;
#[cfg(all(feature = "mcp-client", not(feature = "analysis")))]
#[path = "io/mcp_client/mod.rs"]
pub mod io;
#[cfg(all(feature = "acp", not(any(feature = "analysis", feature = "mcp-client"))))]
/// Narrow I/O surface for builds that enable only the ACP adapter.
pub mod io {
    /// Agent Client Protocol process, readiness, and inference adapter.
    pub mod acp;
}
pub mod util;
#[cfg(feature = "analysis")]
use crate::io::api::{github, gitlab, Configuration};
#[cfg(feature = "analysis")]
use crate::io::http::get;
#[cfg(feature = "analysis")]
use acorn_core::prelude::{format, String, Vec};
#[cfg(feature = "analysis")]
use acorn_core::util::MimeType;
#[cfg(feature = "analysis")]
use acorn_core::{Location, Release, Repository, Scheme};
#[cfg(feature = "analysis")]
use acorn_host::fs::file_uri_to_path;
#[cfg(feature = "analysis")]
use acorn_host::http::{HttpClient, HttpPolicy};
pub use acorn_macros::{Validate, With};
#[cfg(feature = "analysis")]
use acorn_schema::ControlledVocabulary;
#[cfg(feature = "analysis")]
use std::path::PathBuf;
#[cfg(feature = "analysis")]
use tracing::{debug, error, trace, warn};
/// Native filesystem and content-detection operations for [`Location`].
#[cfg(feature = "analysis")]
pub trait LocationExt {
    /// Convert the location URI to a normalized filesystem path.
    fn uri_as_path(&self) -> PathBuf;
    /// Return the path when this location identifies an available local source.
    fn local_path(&self) -> Option<PathBuf>;
    /// Resolve a source file type from content, falling back to the location path.
    fn resolve_file_type(&self, content: &[u8]) -> MimeType;
    /// Check whether this location is reachable.
    #[cfg(feature = "analysis")]
    fn exists(self) -> impl core::future::Future<Output = bool> + Send;
}
/// Native provider operations for [`Repository`].
#[cfg(feature = "analysis")]
pub trait RepositoryExt {
    /// Get metadata for the latest repository release.
    fn latest_release(self) -> impl core::future::Future<Output = Option<Release>> + Send;
    /// Fetch canonical programming-language names for the first supported repository.
    fn technology(repositories: &[Self], options: Option<gitlab::Options>) -> impl core::future::Future<Output = Option<Vec<String>>> + Send
    where
        Self: Sized;
    /// Get all releases supplied by the repository provider.
    #[doc(hidden)]
    fn releases(self) -> impl core::future::Future<Output = Vec<Release>> + Send;
}
#[cfg(feature = "analysis")]
impl LocationExt for Location {
    fn uri_as_path(&self) -> PathBuf {
        let value = self.uri().unwrap_or_default();
        file_uri_to_path(&value).unwrap_or_else(|_| PathBuf::from(value))
    }
    fn local_path(&self) -> Option<PathBuf> {
        let path = self.uri_as_path();
        (self.is_local() || path.exists()).then_some(path)
    }
    fn resolve_file_type(&self, content: &[u8]) -> MimeType {
        MimeType::infer(content)
            .or_else(|| self.path().as_deref().map(MimeType::from).filter(|mime| !mime.is_unknown()))
            .or_else(|| core::str::from_utf8(content).ok().map(MimeType::infer_from))
            .unwrap_or_else(|| MimeType::Unknown(self.path().unwrap_or_else(|| "unknown".to_string())))
    }
    #[cfg(feature = "analysis")]
    async fn exists(self) -> bool {
        let uri = self.uri();
        let scheme = self.scheme();
        if scheme == Scheme::HTTP {
            warn!("HTTP is supported but only advised in local development scenarios");
        }
        match scheme {
            | Scheme::HTTPS | Scheme::HTTP => match (uri, HttpClient::new(HttpPolicy::new())) {
                | (Some(uri), Ok(client)) => client.probe(&uri).await.is_ok(),
                | _ => false,
            },
            | Scheme::File => uri.is_some_and(|_| PathBuf::from(self.path().unwrap_or_default()).exists()),
            | Scheme::OCI | Scheme::SSH | Scheme::Unsupported => false,
        }
    }
}
#[cfg(feature = "analysis")]
impl RepositoryExt for Repository {
    async fn latest_release(self) -> Option<Release> {
        self.releases()
            .await
            .into_iter()
            .next()
            .inspect(|release| trace!(?release, "Latest release"))
    }
    async fn technology(repositories: &[Self], options: Option<gitlab::Options>) -> Option<Vec<String>> {
        let options = options.unwrap_or_else(gitlab::Options::from_env);
        match repositories.iter().find(|repository| repository.is_git_hub()) {
            | Some(repository) => github::languages(repository.project_path()?)
                .await
                .ok()
                .map(|values| ControlledVocabulary::normalize("technology", values).into_values()),
            | None => match repositories
                .iter()
                .find(|repository| repository.is_gitlab())
                .and_then(Repository::project_path)
                .map(|path| options.with_identifier(path))
            {
                | Some(options) => gitlab::language_use(&options).await.ok().map(|response| {
                    ControlledVocabulary::normalize("technology", response.languages.into_iter().map(|language| language.name)).into_values()
                }),
                | None => None,
            },
        }
    }
    async fn releases(self) -> Vec<Release> {
        let maybe_url = match &self {
            | Repository::Git { .. } | Repository::HuggingFace { .. } | Repository::OCI { .. } => None,
            | Repository::GitHub { location } => location
                .host()
                .zip(location.path())
                .map(|(host, path)| format!("https://api.{host}/repos{path}/releases")),
            | Repository::GitLab { location, .. } => self
                .id()
                .zip(location.host())
                .map(|(id, host)| format!("https://{host}/api/v4/projects/{id}/releases")),
        };
        match maybe_url {
            | Some(url) => {
                debug!(url, "Resolve releases");
                match get(url).send().await {
                    | Ok(response) => match response.text().await {
                        | Ok(text) if text.contains("API rate limit exceeded") => {
                            error!("GitHub API rate limit exceeded");
                            Vec::new()
                        }
                        | Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
                        | Err(why) => {
                            error!("Parse release response: {why}");
                            Vec::new()
                        }
                    },
                    | Err(why) => {
                        error!("Download releases: {why}");
                        Vec::new()
                    }
                }
            }
            | None => Vec::new(),
        }
    }
}
#[cfg(all(test, feature = "analysis"))]
mod test;
