//! # ACORN host library
//!
//! This crate provides ACORN services and host adapters. Portable primitives and
//! schemas are owned by `acorn-core` and `acorn-schema`, respectively.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "std", feature = "analyzer"))]
extern crate alloc;

#[doc(hidden)]
#[cfg(feature = "cmd")]
pub use acorn_macros::cmd_sh_words;

#[cfg(feature = "analyzer")]
pub mod analyzer;
#[cfg(feature = "doctor")]
pub mod doctor;
pub mod error;
#[cfg(feature = "std")]
pub mod io;
pub mod util;

pub use acorn_macros::{Validate, With};

#[cfg(feature = "std")]
use crate::io::api::{github, gitlab, Configuration};
#[cfg(feature = "std")]
use crate::io::http::get;
#[cfg(feature = "std")]
use acorn_core::prelude::{format, String, Vec};
#[cfg(feature = "std")]
use acorn_core::util::MimeType;
#[cfg(feature = "std")]
use acorn_core::{Location, Release, Repository, Scheme};
#[cfg(feature = "std")]
use acorn_schema::ControlledVocabulary;
#[cfg(all(feature = "std", feature = "analysis"))]
use analyzer::host::runtime::link_check;
#[cfg(all(feature = "std", feature = "analysis"))]
use analyzer::Check;
#[cfg(feature = "std")]
use std::path::PathBuf;
#[cfg(feature = "std")]
use tracing::{debug, error, trace, warn};

/// Native filesystem and content-detection operations for [`Location`].
#[cfg(feature = "std")]
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
#[cfg(feature = "std")]
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
#[cfg(feature = "std")]
impl LocationExt for Location {
    fn uri_as_path(&self) -> PathBuf {
        io::uri_to_path(self.uri().unwrap_or_default())
    }
    fn local_path(&self) -> Option<PathBuf> {
        let path = self.uri_as_path();
        (self.is_local() || path.exists()).then_some(path)
    }
    fn resolve_file_type(&self, content: &[u8]) -> MimeType {
        MimeType::infer(content).unwrap_or_else(|| {
            self.path()
                .as_deref()
                .map(MimeType::from)
                .filter(|mime| !mime.is_unknown())
                .unwrap_or(MimeType::Json)
        })
    }
    #[cfg(feature = "analysis")]
    async fn exists(self) -> bool {
        let uri = self.uri();
        let scheme = self.scheme();
        if scheme == Scheme::HTTP {
            warn!("HTTP is supported but only advised in local development scenarios");
        }
        match scheme {
            | Scheme::HTTPS | Scheme::HTTP => match uri {
                | Some(uri) => matches!(link_check(Some(uri), None).await, Check { success: true, .. }),
                | None => false,
            },
            | Scheme::File => uri.is_some_and(|_| PathBuf::from(self.path().unwrap_or_default()).exists()),
            | Scheme::Unsupported => false,
        }
    }
}
#[cfg(feature = "std")]
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
                .find(|repository| repository.is_git_lab())
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

#[cfg(all(test, feature = "std"))]
mod test;
