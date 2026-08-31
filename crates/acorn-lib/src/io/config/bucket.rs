//! Bucket transfer and local copy logic
use crate::io::api::{github, gitlab, Configuration};
use crate::io::bagit::Bag;
use crate::io::config::transfer::{transfer_bucket_files, DownloadOptions, TransferItem, TransferManifest, TransferPolicy};
use crate::io::config::{filter_paths, FilterSet};
use crate::io::download::{DownloadItem, DownloadTask};
use crate::io::http::{self, download_with_progress};
use crate::io::{files_all, first_env_var, uri_to_path, write_file_bytes, ApiResult, ArchiveCandidate, TemporaryDirectory};
use crate::prelude::{self, exit, Arc, ErrorKind, Mutex, Path, PathBuf};
use crate::util::constants::app::GITLAB_PRIVATE_TOKEN_HEADER;
use crate::util::constants::env::GITLAB_TOKEN_VARIABLE_NAMES;
use crate::util::{Label, MimeType, StringConversion};
use crate::{Location, Repository, Scheme};
use bon::Builder;
use color_eyre::eyre::{eyre, Report};
use core::future::Future;
use futures::{future, stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::{error, warn};

/// Struct for bucket data
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[builder(start_fn = init)]
pub struct Bucket {
    /// Bucket name
    ///
    /// See <https://schema.org/name>
    pub name: Option<String>,
    /// Bucket description
    ///
    /// See <https://schema.org/description>
    pub description: Option<String>,
    /// Code repository data of bucket
    ///
    /// See <https://schema.org/codeRepository>
    #[serde(alias = "repository")]
    pub code_repository: Repository,
}
/// Options for bucket transfers
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init)]
pub struct BucketOptions {
    /// Path to output directory
    pub output: Option<PathBuf>,
    /// Optional path to the activity database
    pub database_path: Option<PathBuf>,
    /// Disable ingestion into the local activity database
    #[builder(default)]
    pub no_local_database: bool,
    /// Reject deprecated research activity fields during ingestion
    #[builder(default)]
    pub strict: bool,
    /// Rewrite deprecated research activity fields before ingestion
    #[builder(default)]
    pub canonicalize: bool,
    /// Disable remote bucket operations
    #[builder(default)]
    pub offline: bool,
    /// Number of threads used for parallel processing
    #[builder(default = 10)]
    pub threads: usize,
    /// Suppress progress output
    #[builder(default)]
    pub quiet: bool,
    /// Regex pattern(s) of files to ignore
    #[builder(default)]
    pub ignore: Vec<String>,
    /// Regex pattern(s) of files to include
    #[builder(default)]
    pub filter: Vec<String>,
    /// Save transferred files directly beneath the output directory
    #[builder(default)]
    pub flatten: bool,
    /// Replace existing files or directories at selected output paths
    #[builder(default)]
    pub clobber: bool,
}
/// A collection of buckets that can be transferred together.
#[derive(Clone, Debug, Default)]
pub struct Buckets(pub Vec<Bucket>);
impl Bucket {
    /// Resolves and validates a BagIt archive as a local payload bucket.
    pub async fn archive(source: &str, format: Option<MimeType>, temporary: &TemporaryDirectory, offline: bool) -> ApiResult<(Self, String)> {
        let local = uri_to_path(source);
        let archive_path = match local.is_file() {
            | true => Ok(local),
            | false if offline => Err(eyre!("Offline mode cannot import remote archive: {source}")),
            | false => {
                let path = temporary.path().join("archive");
                download_with_progress(source, &path, |_, _| {}, None, None, None, None)
                    .await
                    .map(|_| path)
            }
        };
        archive_path
            .and_then(|path| ArchiveCandidate::from(path).extract(Some(temporary.path().join("extracted")), format))
            .and_then(|root| bag_root(&root))
            .and_then(|root| Bag::verify(&root).map(|()| root.join("data")))
            .map(|payload| (Self::from(payload.as_path()), source.to_string()))
    }
    /// Transfers this bucket and ingests supported research activity data.
    pub async fn transfer(self, options: &BucketOptions, policy: TransferPolicy, provenance: Option<&str>) -> ApiResult<usize> {
        let local = self.code_repository.is_local();
        match (policy, local, options.offline) {
            | (TransferPolicy::Download, true, _) => Err(eyre!("Bucket download requires a remote repository source")),
            | (_, false, true) => Err(eyre!("Offline mode cannot transfer remote bucket sources")),
            | _ => match local {
                | true => self.copy_files(options).await,
                | false => self.download_files(options).await,
            }
            .and_then(|manifest| {
                let manifest = match provenance {
                    | Some(source) => TransferManifest {
                        repository: source.to_string(),
                        ..manifest
                    },
                    | None => manifest,
                };
                let count = manifest.count();
                manifest
                    .canonicalize(options)
                    .and_then(|manifest| manifest.ingest(options))
                    .map(|()| count)
            }),
        }
    }
    /// Imports this bucket and ingests supported research activity data.
    pub async fn import(self, options: &BucketOptions, provenance: Option<&str>) -> ApiResult<usize> {
        self.transfer(options, TransferPolicy::Import, provenance).await
    }
    /// Get hosting domain from bucket struct
    pub(crate) fn domain(&self) -> ApiResult<String> {
        let location = match &self.code_repository {
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
    fn remove_destination(path: &Path) -> ApiResult<()> {
        match path.symlink_metadata() {
            | Ok(metadata) if metadata.file_type().is_symlink() || metadata.is_file() => {
                prelude::remove_file(path).map_err(|why| eyre!("Failed to remove existing output path {} — {why}", path.display()))
            }
            | Ok(_) => prelude::remove_dir_all(path).map_err(|why| eyre!("Failed to remove existing output directory {} — {why}", path.display())),
            | Err(why) if why.kind() == ErrorKind::NotFound => Ok(()),
            | Err(why) => Err(eyre!("Failed to inspect existing output path {} — {why}", path.display())),
        }
    }
    pub(crate) fn prepare_destination(output: &Path, destination: &Path) -> ApiResult<PathBuf> {
        let target = output.join(destination);
        destination
            .parent()
            .into_iter()
            .flat_map(Path::ancestors)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|parent| output.join(parent))
            .try_for_each(|parent| match parent.symlink_metadata() {
                | Ok(metadata) if metadata.is_dir() => Ok(()),
                | Ok(_) => Self::remove_destination(&parent).and_then(|()| {
                    prelude::create_dir_all(&parent).map_err(|why| eyre!("Failed to create output directory {} — {why}", parent.display()))
                }),
                | Err(why) if why.kind() == ErrorKind::NotFound => {
                    prelude::create_dir_all(&parent).map_err(|why| eyre!("Failed to create output directory {} — {why}", parent.display()))
                }
                | Err(why) => Err(eyre!("Failed to inspect output directory {} — {why}", parent.display())),
            })
            .and_then(|()| Self::remove_destination(&target))
            .map(|()| target)
    }
    pub(crate) async fn write_file<F, Fut, E>(output: &Path, destination: &Path, clobber: bool, write_lock: &Mutex<()>, get_bytes: F) -> ApiResult<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Vec<u8>, E>>,
        E: Into<Report>,
    {
        match clobber {
            | false => write_file_bytes(output.join(destination), get_bytes).await,
            | true => match get_bytes().await.map_err(Into::into) {
                | Ok(bytes) => match write_lock.lock() {
                    | Ok(_guard) => Self::prepare_destination(output, destination).and_then(|target| {
                        prelude::write(&target, bytes)
                            .map(|_| ())
                            .map_err(|why| eyre!("Failed to write output file {} — {why}", target.display()))
                    }),
                    | Err(why) => Err(eyre!("Failed to lock bucket output — {why}")),
                },
                | Err(why) => Err(why),
            },
        }
    }
    /// Copy files from (local) bucket to local directory
    /// ### Notes
    /// - Ignores files listed in [`IGNORE`]
    /// - Only copies files from local repositories
    pub async fn copy_files(self: Bucket, options: &BucketOptions) -> ApiResult<TransferManifest> {
        let BucketOptions { output, ignore, filter, .. } = options;
        let output = Arc::new(output.clone().unwrap_or_default());
        match FilterSet::compile(ignore, filter) {
            | Ok(filters) => {
                let Bucket { name, code_repository, .. } = self.clone();
                match code_repository.is_local() {
                    | true => {
                        let bucket_root = match code_repository.location().local_path() {
                            | Some(value) => value.to_absolute_path(),
                            | None => {
                                return Err(eyre!(
                                    "Bucket {} has no local path — cannot copy files",
                                    name.as_deref().unwrap_or("unknown")
                                ))
                            }
                        };
                        let items = filter_paths(
                            files_all(PathBuf::from(&bucket_root), None::<Vec<String>>)
                                .into_iter()
                                .map(|x| x.display().to_string())
                                .collect::<Vec<String>>(),
                            &filters,
                        )
                        .into_iter()
                        .filter(|path| PathBuf::from(path).is_file())
                        .filter_map(|path| {
                            PathBuf::from(&path)
                                .strip_prefix(&bucket_root)
                                .ok()
                                .map(|relative| relative.display().to_string())
                        })
                        .collect::<Vec<String>>();
                        match TransferItem::collect(items, options.flatten) {
                            | Ok(items) => {
                                let bucket_root = Arc::new(bucket_root);
                                let clobber = options.clobber;
                                let write_lock = Arc::new(Mutex::new(()));
                                let operation = {
                                    let bucket_root = Arc::clone(&bucket_root);
                                    let output = Arc::clone(&output);
                                    let write_lock = Arc::clone(&write_lock);
                                    move |item: TransferItem| {
                                        let bucket_root = Arc::clone(&bucket_root);
                                        let output = Arc::clone(&output);
                                        let write_lock = Arc::clone(&write_lock);
                                        async move {
                                            let source = PathBuf::from(bucket_root.as_str()).join(item.source);
                                            Self::write_file(output.as_path(), &item.destination, clobber, write_lock.as_ref(), || async {
                                                prelude::read(source)
                                            })
                                            .await
                                        }
                                    }
                                };
                                transfer_bucket_files(name, code_repository.location().to_string(), items, options, "Copying", operation).await
                            }
                            | Err(why) => Err(why),
                        }
                    }
                    | false => Ok(TransferManifest {
                        bucket: name,
                        repository: code_repository.location().to_string(),
                        files: Vec::new(),
                    }),
                }
            }
            | Err(why) => Err(why),
        }
    }
    /// Download files from bucket to local directory
    ///
    /// Ignores files listed in [`IGNORE`]
    ///
    /// Downloads files concurrently using streaming, resumable transfers for remote
    /// repositories. GitLab buckets stream from the repository-files raw API on the resolved
    /// default branch, falling back to the buffered repository-file API when streaming fails.
    pub async fn download_files(self: Bucket, options: &BucketOptions) -> ApiResult<TransferManifest> {
        let BucketOptions { filter, ignore, .. } = options;
        match FilterSet::compile(ignore, filter) {
            | Ok(filters) => match self.file_paths("").await {
                | Ok(paths) => match TransferItem::collect(filter_paths(paths, &filters), options.flatten) {
                    | Ok(items) => {
                        let clobber = options.clobber;
                        match self.resolve_branch().await {
                            | Ok(branch) => {
                                let repository_location = self.code_repository.location().to_string();
                                let operation = {
                                    let output = Arc::new(options.output.clone().unwrap_or_default());
                                    let repository = Arc::new(self.code_repository);
                                    let branch = Arc::new(branch);
                                    move |item: TransferItem| {
                                        let output = Arc::clone(&output);
                                        let repository = Arc::clone(&repository);
                                        let branch = Arc::clone(&branch);
                                        async move {
                                            let options = DownloadOptions {
                                                repository: repository.as_ref(),
                                                output: output.as_path(),
                                                source: &item.source,
                                                destination: &item.destination,
                                                branch: branch.as_deref(),
                                                clobber,
                                            };
                                            match Self::download_file(&options).await {
                                                | Ok(()) => Ok(()),
                                                | Err(stream_error) => match repository.as_ref() {
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
                                };
                                transfer_bucket_files(self.name, repository_location, items, options, "Downloading", operation).await
                            }
                            | Err(why) => Err(why),
                        }
                    }
                    | Err(why) => Err(why),
                },
                | Err(why) => {
                    error!("=> {} Get file paths for download — {why}", Label::fail());
                    Err(why)
                }
            },
            | Err(why) => Err(why),
        }
    }
    /// Resolve a repository's default branch, falling back to `None` when unavailable.
    async fn resolve_branch(&self) -> ApiResult<Option<String>> {
        match &self.code_repository {
            | Repository::GitLab { .. } => match (self.code_repository.domain(), self.code_repository.id()) {
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
    /// Build a streaming download item for a bucket file, resolving the provider-specific raw URL and credentials.
    pub(crate) fn download_item(options: &DownloadOptions<'_>) -> ApiResult<DownloadItem> {
        let path = options.source.display().to_string();
        let request = match options.repository {
            | Repository::GitLab { .. } => {
                let branch = options.branch.filter(|value| !value.is_empty()).unwrap_or("HEAD");
                let branch_ref = branch.percent_encode();
                let encoded_path = path.percent_encode();
                let headers = first_env_var(&GITLAB_TOKEN_VARIABLE_NAMES)
                    .filter(|value| !value.trim().is_empty())
                    .map(|token| http::headers([(GITLAB_PRIVATE_TOKEN_HEADER, token.as_str())]));
                match (options.repository.domain(), options.repository.id()) {
                    | (Some(domain), Some(identifier)) => Ok((
                        format!("https://{domain}/api/v4/projects/{identifier}/repository/files/{encoded_path}/raw?ref={branch_ref}"),
                        headers,
                    )),
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
            size: None,
            sha: None,
            headers,
        })
    }
    /// Stream a bucket file to disk with `.part` resume and atomic finalization.
    ///
    /// Existing files are left untouched unless `clobber` is set, in which case the
    /// destination is removed before the transfer starts.
    pub(crate) async fn download_file(options: &DownloadOptions<'_>) -> ApiResult<()> {
        match Self::download_item(options) {
            | Ok(item) => {
                let target = options.output.join(options.destination);
                match (options.clobber, item.is_complete_at(&target)) {
                    | (false, true) => Ok(()),
                    | (true, _) => match Self::remove_destination(&target) {
                        | Ok(()) => DownloadTask::new(item, options.output, true, true).download().await,
                        | Err(why) => Err(why),
                    },
                    | _ => DownloadTask::new(item, options.output, true, true).download().await,
                }
            }
            | Err(why) => Err(why),
        }
    }
    /// Fall back to the buffered GitLab repository-file API for a bucket file.
    async fn download_gitlab_via_api(options: &DownloadOptions<'_>) -> ApiResult<()> {
        match (options.repository.domain(), options.repository.id()) {
            | (Some(domain), Some(identifier)) => {
                let request = gitlab::Options::from_env()
                    .with_domain(domain)
                    .with_identifier(identifier)
                    .with_path(options.source.display().to_string())
                    .with_sha(options.branch.filter(|value| !value.is_empty()).unwrap_or("HEAD"));
                match gitlab::repository_file(&request).await.and_then(|file| file.decoded_content()) {
                    | Ok(bytes) if options.clobber => {
                        let write_lock = Mutex::new(());
                        Self::write_file(options.output, options.destination, true, &write_lock, || async {
                            Ok::<_, Report>(bytes)
                        })
                        .await
                    }
                    | Ok(bytes) => write_file_bytes(options.output.join(options.destination), || async { Ok::<_, Report>(bytes) }).await,
                    | Err(why) => Err(why),
                }
            }
            | _ => Err(eyre!("Failed to build GitLab API request for repository path")),
        }
    }
    async fn file_paths(&self, directory: &str) -> ApiResult<Vec<String>> {
        let code_repository = self.code_repository.clone();
        let bucket_name = self.name.clone().unwrap_or_else(|| "Bucket".to_string()).to_uppercase();
        match &code_repository {
            | Repository::Git { .. } => {
                let path = match code_repository.location().local_path() {
                    | Some(value) => value,
                    | None => return Err(eyre!("Git repository has no local path — cannot list files")),
                };
                Ok(files_all(path, None::<Vec<String>>)
                    .into_iter()
                    .map(|x| x.display().to_string())
                    .collect())
            }
            | Repository::GitHub { location } => match location.path() {
                | Some(path) => {
                    let path = path.trim_start_matches('/').to_string();
                    match self.domain() {
                        | Ok(host) => github::tree_paths(format!("api.{}", host), path, "main")
                            .await
                            .map_err(|why| eyre!("Failed to get file paths for {bucket_name} bucket - {why}")),
                        | Err(why) => Err(why),
                    }
                }
                | None => Err(eyre!("Failed to parse GitHub URI for {bucket_name} bucket")),
            },
            | Repository::GitLab { .. } => match code_repository.id() {
                | Some(id) => match self.domain() {
                    | Ok(host) => {
                        let options = gitlab::Options::from_env().with_domain(host).with_identifier(id).with_path(directory);
                        let mut page = 1_u32;
                        let mut all_paths: Vec<String> = vec![];
                        loop {
                            let page_options = options.clone().with_page(page);
                            match gitlab::repository_tree(&page_options).await {
                                | Ok(response) if response.entry_count == 0 => {
                                    break Ok(all_paths.clone());
                                }
                                | Ok(response) => {
                                    all_paths.extend(response.paths);
                                    page = page.saturating_add(1);
                                }
                                | Err(why) => {
                                    break Err(eyre!("Failed to get file paths for {bucket_name} bucket — {why}"));
                                }
                            }
                        }
                    }
                    | Err(why) => Err(why),
                },
                | None => Err(eyre!("Missing GitLab project id for {bucket_name} bucket")),
            },
            | Repository::HuggingFace { .. } => Err(eyre!("Hugging Face repositories are unsupported for bucket downloads")),
            | Repository::OCI { .. } => Err(eyre!("OCI artifacts are unsupported for bucket downloads")),
        }
    }
}
impl From<&Path> for Bucket {
    fn from(value: &Path) -> Self {
        let location = Location::Simple(format!("file:{}/", value.display()));
        Self::init()
            .code_repository(Repository::Git {
                location: Box::new(location),
            })
            .build()
    }
}
impl From<&str> for Bucket {
    fn from(value: &str) -> Self {
        let location = Location::Simple(value.to_string());
        if location.uri().is_none() {
            exit(exitcode::DATAERR);
        }
        let repository = match location.is_local() {
            | true => Repository::Git {
                location: Box::new(location),
            },
            | false => {
                let host = match location.host() {
                    | Some(value) => value.to_lowercase(),
                    | None => {
                        error!(value, "=> {} Parse URI - No host", Label::fail());
                        exit(exitcode::DATAERR);
                    }
                };
                if host.contains("github.com") {
                    Repository::GitHub {
                        location: Box::new(location),
                    }
                } else {
                    let id = None;
                    Repository::GitLab {
                        id,
                        location: Box::new(location),
                    }
                }
            }
        };
        Bucket::init().code_repository(repository).build()
    }
}
impl From<PathBuf> for Bucket {
    fn from(value: PathBuf) -> Self {
        Self::from(value.as_path())
    }
}
impl Default for BucketOptions {
    fn default() -> Self {
        Self {
            output: None,
            database_path: None,
            no_local_database: false,
            strict: false,
            canonicalize: false,
            offline: false,
            threads: 10,
            quiet: false,
            ignore: Vec::new(),
            filter: Vec::new(),
            flatten: false,
            clobber: false,
        }
    }
}
impl Buckets {
    /// Transfers all buckets and returns the total number of processed items.
    pub async fn transfer(self, options: &BucketOptions, policy: TransferPolicy) -> ApiResult<usize> {
        match options.output {
            | Some(_) => {
                stream::iter(self.0)
                    .then(|bucket| bucket.transfer(options, policy, None))
                    .try_fold(0_usize, |total, count| future::ready(Ok(total.saturating_add(count))))
                    .await
            }
            | None => Ok(0),
        }
    }
}
impl From<Vec<Bucket>> for Buckets {
    fn from(value: Vec<Bucket>) -> Self {
        Self(value)
    }
}
fn bag_root(root: &Path) -> ApiResult<PathBuf> {
    match root.join("bagit.txt").is_file() {
        | true => Ok(root.to_path_buf()),
        | false => root
            .read_dir()
            .map_err(Into::into)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter_map(|entry| entry.file_type().ok().filter(|kind| kind.is_dir()).map(|_| entry.path()))
                    .collect::<Vec<_>>()
            })
            .and_then(|directories| match directories.as_slice() {
                | [directory] if directory.join("bagit.txt").is_file() => Ok(directory.clone()),
                | _ => Err(eyre!("BagIt metadata must be at the archive root or beneath one enclosing directory")),
            }),
    }
}
