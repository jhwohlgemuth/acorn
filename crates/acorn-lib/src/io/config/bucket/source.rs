//! Bucket source configuration and repository adapters.
use super::Bucket;
use crate::{
    io::{
        api::{github, gitlab, Authenticator, Configuration, Credential, Endpoint, Params},
        download::{DownloadItem, DownloadTask},
        files_all, http, write_file_bytes, ApiResult, Parse, PathExt,
    },
    util::{constants::app::GITLAB_CREDENTIAL_VARIABLE_PAIRS, StringConversion},
    LocationExt,
};
use acorn_core::{prelude::HashSet, Location, Repository, Scheme, SimpleOrDetailed};
use acorn_host::{fs::SafePath, terminal::Label};
use color_eyre::eyre::{eyre, Report};
use fluent_uri::Uri;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::{
    fs::read,
    path::{Path, PathBuf},
};
use tracing::warn;

pub(super) type BucketInput = SimpleOrDetailed<String, BucketDetailsInput>;
type BucketSourceInput = SimpleOrDetailed<String, BucketSource>;
#[allow(async_fn_in_trait)]
pub(crate) trait BucketSourceAdapter {
    fn is_local(&self) -> bool;
    fn manifest_identity(&self) -> String;
    async fn items(&self) -> ApiResult<Vec<BucketSourceItem>>;
    async fn retrieve(&self, item: &BucketSourceItem, destination: &Path) -> ApiResult<()>;
}
/// Configured origin of one bucket.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum BucketSource {
    /// Existing local or remote repository source.
    Repository(Repository),
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BucketDetailsInput {
    name: Option<String>,
    description: Option<String>,
    #[serde(rename = "codeRepository", alias = "repository", alias = "source")]
    source: BucketSourceInput,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BucketSourceItem {
    pub(crate) key: String,
    pub(crate) path: SafePath,
    pub(crate) revision: Option<String>,
    pub(crate) size: Option<u64>,
}
pub(crate) struct DownloadOptions<'a> {
    pub(crate) repository: &'a Repository,
    pub(crate) output: &'a Path,
    pub(crate) source: &'a Path,
    pub(crate) destination: &'a Path,
    pub(crate) branch: Option<&'a str>,
    pub(crate) size: Option<u64>,
}
#[derive(Clone, Copy)]
pub(crate) struct RepositoryBucketSource<'a> {
    repository: &'a Repository,
}
impl Bucket {
    fn from_parts(name: Option<String>, description: Option<String>, source: BucketSource) -> Self {
        Self { name, description, source }
    }
}
impl From<&Path> for Bucket {
    fn from(value: &Path) -> Self {
        let location = Location::Simple(format!("file:{}/", value.display()));
        Self::from_parts(
            None,
            None,
            Repository::Git {
                location: Box::new(location),
            }
            .into(),
        )
    }
}
impl From<PathBuf> for Bucket {
    fn from(value: PathBuf) -> Self {
        Self::from(value.as_path())
    }
}
impl TryFrom<&str> for Bucket {
    type Error = Report;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        BucketSource::try_from(value).map(|source| Self::from_parts(None, None, source))
    }
}
impl TryFrom<BucketInput> for Bucket {
    type Error = Report;
    fn try_from(value: BucketInput) -> Result<Self, Self::Error> {
        match value {
            | SimpleOrDetailed::Simple(source) => Self::try_from(source),
            | SimpleOrDetailed::Detailed(BucketDetailsInput { name, description, source }) => {
                BucketSource::try_from(source).map(|source| Self::from_parts(name, description, source))
            }
        }
    }
}
impl TryFrom<String> for Bucket {
    type Error = Report;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
impl BucketSource {
    pub(crate) fn is_local(&self) -> bool {
        match self {
            | Self::Repository(repository) => repository.is_local(),
        }
    }
}
impl From<Repository> for BucketSource {
    fn from(value: Repository) -> Self {
        Self::Repository(value)
    }
}
impl TryFrom<&str> for BucketSource {
    type Error = Report;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Repository::parse(value).map(Self::Repository)
    }
}
impl TryFrom<BucketSourceInput> for BucketSource {
    type Error = Report;
    fn try_from(value: BucketSourceInput) -> Result<Self, Self::Error> {
        match value {
            | SimpleOrDetailed::Simple(source) => Self::try_from(source),
            | SimpleOrDetailed::Detailed(source) => Ok(source),
        }
    }
}
impl TryFrom<String> for BucketSource {
    type Error = Report;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
impl BucketSourceItem {
    pub(crate) fn new(key: String, path: impl AsRef<Path>, revision: Option<String>, size: Option<u64>) -> ApiResult<Self> {
        match key.trim().is_empty() {
            | true => Err(eyre!("Bucket source item key cannot be empty")),
            | false => SafePath::new(path).map_err(Report::from).map(|path| Self { key, path, revision, size }),
        }
    }
    pub(crate) fn validate(items: Vec<Self>) -> ApiResult<Vec<Self>> {
        items
            .iter()
            .try_fold(HashSet::new(), |mut paths, item| match paths.insert(item.path.clone()) {
                | true => Ok(paths),
                | false => Err(eyre!("Bucket source contains duplicate path — '{}'", item.path.as_path().display())),
            })
            .map(|_| items)
    }
}
impl Parse<&str> for Repository {
    fn parse(value: &str) -> ApiResult<Self> {
        let invalid = |reason: &str| eyre!("Invalid bucket source '{value}' — {reason}");
        let scheme = value.split_once(':').and_then(|(scheme, _)| {
            let valid = scheme.as_bytes().split_first().is_some_and(|(first, remaining)| {
                first.is_ascii_alphabetic()
                    && remaining
                        .iter()
                        .all(|character| character.is_ascii_alphanumeric() || matches!(*character, b'+' | b'-' | b'.'))
            });
            valid.then_some(scheme)
        });
        match (value.is_empty(), value.trim() == value, scheme, Path::new(value).is_windows()) {
            | (true, _, _, _) => Err(invalid("source cannot be empty")),
            | (_, false, _, _) => Err(invalid("remove surrounding whitespace")),
            | (_, true, None, _) | (_, true, _, true) => Ok(Self::Git {
                location: Box::new(Location::Simple(value.to_string())),
            }),
            | (_, true, Some(_), false) => match Uri::parse(value) {
                | Ok(uri) => {
                    let authority = uri.authority();
                    let has_userinfo = authority.is_some_and(|value| value.userinfo().is_some());
                    let host = authority.map(|value| value.host().to_ascii_lowercase());
                    match (Scheme::from(uri.scheme().as_str()), has_userinfo, host) {
                        | (_, true, _) => Err(invalid("embedded URL credentials are unsupported")),
                        | (Scheme::File, false, _) => Ok(Self::Git {
                            location: Box::new(Location::Simple(value.to_string())),
                        }),
                        | (Scheme::HTTPS, false, Some(host)) if matches!(host.as_str(), "github.com" | "www.github.com") => Ok(Self::GitHub {
                            location: Box::new(Location::Simple(value.to_string())),
                        }),
                        | (Scheme::HTTPS, false, Some(host)) if !host.is_empty() => Ok(Self::GitLab {
                            id: None,
                            location: Box::new(Location::Simple(value.to_string())),
                        }),
                        | (scheme, false, _) => Err(invalid(&format!("unsupported or hostless URI scheme '{scheme}'"))),
                    }
                }
                | Err(_) => Err(invalid("malformed URI")),
            },
        }
    }
}
impl BucketSourceAdapter for RepositoryBucketSource<'_> {
    fn is_local(&self) -> bool {
        self.repository.is_local()
    }
    fn manifest_identity(&self) -> String {
        self.repository.location().to_string()
    }
    async fn items(&self) -> ApiResult<Vec<BucketSourceItem>> {
        match self.file_paths("").await {
            | Ok(paths) => match self.resolve_branch().await {
                | Ok(revision) => paths
                    .into_iter()
                    .map(|path| BucketSourceItem::new(path.clone(), path, revision.clone(), None))
                    .collect::<ApiResult<Vec<_>>>()
                    .and_then(BucketSourceItem::validate),
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    async fn retrieve(&self, item: &BucketSourceItem, destination: &Path) -> ApiResult<()> {
        match self.repository {
            | Repository::Git { .. } => match self.local_root() {
                | Ok(root) => {
                    let source = root.join(&item.key);
                    write_file_bytes(destination.to_path_buf(), || async { read(source) }).await
                }
                | Err(why) => Err(why),
            },
            | Repository::GitHub { .. } | Repository::GitLab { .. } => {
                let output = destination.parent().unwrap_or_else(|| Path::new(""));
                let destination = destination.file_name().map(PathBuf::from);
                match destination {
                    | None => Err(eyre!("Bucket output path has no filename")),
                    | Some(destination) => {
                        let source = PathBuf::from(&item.key);
                        let options = DownloadOptions {
                            repository: self.repository,
                            output,
                            source: &source,
                            destination: &destination,
                            branch: item.revision.as_deref(),
                            size: item.size,
                        };
                        match Self::download_file(&options).await {
                            | Ok(()) => Ok(()),
                            | Err(stream_error) => match self.repository {
                                | Repository::GitLab { .. } => match Self::download_gitlab_via_api(&options).await {
                                    | Ok(()) => Ok(()),
                                    | Err(fallback_error) => Err(eyre!(
                                        "{stream_error}; and the buffered GitLab API fallback also failed — {fallback_error}"
                                    )),
                                },
                                | _ => Err(stream_error),
                            },
                        }
                    }
                }
            }
            | Repository::HuggingFace { .. } => Err(eyre!("Hugging Face repositories are unsupported for bucket downloads")),
            | Repository::OCI { .. } => Err(eyre!("OCI artifacts are unsupported for bucket downloads")),
        }
    }
}
impl<'a> RepositoryBucketSource<'a> {
    pub(crate) const fn new(repository: &'a Repository) -> Self {
        Self { repository }
    }
    pub(crate) fn domain(&self) -> ApiResult<String> {
        let location = match self.repository {
            | Repository::GitHub { location } | Repository::GitLab { location, .. } => Ok(location),
            | Repository::Git { .. } => Err(eyre!("Domain is unsupported for generic Git repositories")),
            | Repository::HuggingFace { .. } => Err(eyre!("Domain is unsupported for Hugging Face repositories")),
            | Repository::OCI { .. } => Err(eyre!("Domain is unsupported for OCI artifacts used as buckets")),
        };
        location.and_then(|location| match location.scheme() {
            | Scheme::HTTPS => location.host().ok_or_else(|| eyre!("Failed to parse repository host from URI")),
            | _ => Err(eyre!("Unsupported repository URI scheme")),
        })
    }
    pub(crate) fn download_item(options: &DownloadOptions<'_>) -> ApiResult<DownloadItem> {
        let path = options.source.display().to_string();
        let request = match options.repository {
            | Repository::GitLab { .. } => {
                let branch = options.branch.filter(|value| !value.is_empty()).unwrap_or("HEAD");
                let encoded_path = path.percent_encode();
                let headers = Credential::from_env(&GITLAB_CREDENTIAL_VARIABLE_PAIRS)
                    .map(Credential::into_header_pair)
                    .map(|(header, token)| http::headers([(header, ExposeSecret::expose_secret(&token))]));
                match (options.repository.domain(), options.repository.id()) {
                    | (Some(domain), Some(identifier)) => {
                        let params = Params::new()
                            .with_template("identifier", Some(identifier.as_str()))
                            .with_template("file_path", Some(encoded_path.as_str()))
                            .with_keyvalue("ref", Some(branch))
                            .build();
                        Endpoint::from_template("gitlab::api")
                            .map(|endpoint| endpoint.with_domain(domain))
                            .and_then(|endpoint| endpoint.resource_url("repository-file-raw", Some(params)))
                            .map(|url| (url, headers))
                    }
                    | _ => Err(eyre!("Failed to build GitLab API request for repository path")),
                }
            }
            | _ => options
                .repository
                .raw_url(path)
                .map(|url| (url, None))
                .ok_or_else(|| eyre!("Failed to build raw URL for repository path")),
        };
        request.map(|(url, headers)| DownloadItem {
            url,
            path: options.destination.display().to_string(),
            size: options.size,
            sha: None,
            headers,
        })
    }
    pub(crate) async fn download_file(options: &DownloadOptions<'_>) -> ApiResult<()> {
        match Self::download_item(options) {
            | Ok(item) => match item.target(options.output) {
                | Ok(target) if item.is_complete_at(&target) => Ok(()),
                | Ok(_) => match DownloadTask::new(item, options.output, true, true) {
                    | Ok(task) => task.download().await,
                    | Err(why) => Err(why),
                },
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    async fn download_gitlab_via_api(options: &DownloadOptions<'_>) -> ApiResult<()> {
        match (options.repository.domain(), options.repository.id()) {
            | (Some(domain), Some(identifier)) => {
                let request = gitlab::Options::from_env()
                    .with_domain(domain)
                    .with_identifier(identifier)
                    .with_path(options.source.display().to_string())
                    .with_sha(options.branch.filter(|value| !value.is_empty()).unwrap_or("HEAD"));
                match gitlab::repository_file(&request).await.and_then(|file| file.decoded_content()) {
                    | Ok(bytes) => write_file_bytes(options.output.join(options.destination), || async { Ok::<_, Report>(bytes) }).await,
                    | Err(why) => Err(why),
                }
            }
            | _ => Err(eyre!("Failed to build GitLab API request for repository path")),
        }
    }
    async fn file_paths(&self, directory: &str) -> ApiResult<Vec<String>> {
        match self.repository {
            | Repository::Git { .. } => self.local_root().map(|root| {
                files_all(root.clone(), None::<Vec<String>>)
                    .into_iter()
                    .filter(|path| path.is_file())
                    .filter_map(|path| path.strip_prefix(&root).ok().map(|relative| relative.display().to_string()))
                    .collect()
            }),
            | Repository::GitHub { location } => match (location.path(), self.domain()) {
                | (Some(path), Ok(host)) => github::tree_paths(format!("api.{host}"), path.trim_start_matches('/').to_string(), "main").await,
                | (None, Ok(_)) => Err(eyre!("Failed to parse GitHub repository URI")),
                | (_, Err(why)) => Err(why),
            },
            | Repository::GitLab { .. } => match self.repository.id() {
                | Some(identifier) => match self.domain() {
                    | Ok(domain) => {
                        let options = gitlab::Options::from_env()
                            .with_domain(domain)
                            .with_identifier(identifier)
                            .with_path(directory);
                        let mut page = 1_u32;
                        let mut paths = Vec::new();
                        loop {
                            match gitlab::repository_tree(&options.clone().with_page(page)).await {
                                | Ok(response) if response.entry_count == 0 => break Ok(paths),
                                | Ok(response) => {
                                    paths.extend(response.paths);
                                    page = page.saturating_add(1);
                                }
                                | Err(why) => break Err(why),
                            }
                        }
                    }
                    | Err(why) => Err(why),
                },
                | None => Err(eyre!("Missing GitLab project id")),
            },
            | Repository::HuggingFace { .. } => Err(eyre!("Hugging Face repositories are unsupported for bucket downloads")),
            | Repository::OCI { .. } => Err(eyre!("OCI artifacts are unsupported for bucket downloads")),
        }
    }
    fn local_root(&self) -> ApiResult<PathBuf> {
        self.repository
            .location()
            .local_path()
            .map(|path| PathBuf::from(path.to_absolute_path()))
            .ok_or_else(|| eyre!("Git repository has no local path — cannot list files"))
    }
    async fn resolve_branch(&self) -> ApiResult<Option<String>> {
        match self.repository {
            | Repository::GitLab { .. } => match (self.repository.domain(), self.repository.id()) {
                | (Some(domain), Some(identifier)) => {
                    let options = gitlab::Options::from_env().with_domain(domain).with_identifier(identifier);
                    match gitlab::project_details(&options).await {
                        | Ok(details) => Ok(details.default_branch.filter(|value| !value.is_empty())),
                        | Err(why) => {
                            warn!("=> {} Resolve GitLab default branch — {why}", Label::fail());
                            Ok(None)
                        }
                    }
                }
                | _ => Ok(None),
            },
            | Repository::GitHub { .. } => Ok(Some("main".to_string())),
            | _ => Ok(None),
        }
    }
}
