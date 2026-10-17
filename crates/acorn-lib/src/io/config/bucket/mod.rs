//! Bucket transfer and local copy logic
use self::source::{ArchiveBucketSource, BucketInput, BucketSourceAdapter, RepositoryBucketSource};
use crate::io::{
    bagit::Bag,
    config::transfer::{transfer_bucket_files, TransferManifest, TransferPolicy},
    http::download_with_progress,
    ApiResult, ArchiveCandidate,
};
use crate::util::constants::app::APPLICATION;
use acorn_core::{options::RuntimeOptions, util::MimeType};
use acorn_host::fs::{file_uri_to_path, TemporaryDirectory};
use bon::Builder;
use color_eyre::eyre::eyre;
use futures::{future, stream, StreamExt, TryStreamExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
pub use source::{BagArchive, BucketSource, BucketSourceIdentity, BucketSourceKind};
use std::path::{Path, PathBuf};

pub mod source;
/// Options for bucket transfers
pub type BucketOptions = RuntimeOptions<BucketExtension, PathBuf>;
/// One import or download source taken from a configuration entry or command-line URI.
#[derive(Clone, Debug)]
pub enum SourceInput {
    /// Bucket already described by a configuration entry.
    Bucket(Bucket),
    /// URI or local path that must be classified before it can be transferred.
    Uri(String),
}
/// Configured collection of research activity files
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", try_from = "BucketInput")]
#[schemars(!try_from)]
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
    /// Provider used to enumerate and retrieve bucket items
    ///
    /// See <https://schema.org/codeRepository>
    pub source: BucketSource,
}
/// Bucket-specific transfer options
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BucketExtension {
    /// Rewrite deprecated research activity fields before ingestion
    pub canonicalize: bool,
    /// Replace existing files or directories at selected output paths
    pub clobber: bool,
    /// Optional path to the activity database
    pub database_path: Option<PathBuf>,
    /// Save transferred files directly beneath the output directory
    pub flatten: bool,
    /// Disable ingestion into the local activity database
    pub no_local_database: bool,
    /// Reject deprecated research activity fields during ingestion
    pub strict: bool,
}
/// A collection of buckets that can be transferred together
#[derive(Clone, Debug, Default)]
pub struct Buckets(pub Vec<Bucket>);
/// A bucket resolved for transfer, together with the temporary storage that must outlive it.
#[derive(Debug)]
pub struct ResolvedBucket {
    bucket: Bucket,
    /// Location this bucket was resolved from, reported as transfer provenance.
    provenance: Option<String>,
    /// Downloaded archive and extraction staging for archive-backed payloads.
    ///
    /// Dropping this directory before the transfer would delete the payload, so callers transfer through this type rather than extracting the bare bucket.
    _temporary: Option<TemporaryDirectory>,
}
impl Bucket {
    /// Resolves and verifies a local or remote BagIt archive as a payload bucket.
    ///
    /// The bag is resolved at the archive root or beneath one enclosing directory, verified in full before any payload is published, and reduced to its `data/` directory so BagIt tag files never reach the output.
    /// The returned location is the original archive URI, which callers pass along as transfer provenance.
    ///
    /// The `temporary` directory owns the downloaded archive and extraction staging, so it must outlive the transfer that consumes the returned bucket.
    pub async fn archive(source: &str, format: Option<MimeType>, temporary: &TemporaryDirectory, offline: bool) -> ApiResult<(Self, String)> {
        let local = file_uri_to_path(source).unwrap_or_else(|_| PathBuf::from(source));
        let archive_path = match local.is_file() {
            | true => Ok(local),
            | false if offline => Err(eyre!("Offline mode cannot transfer remote archive: {source}")),
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
            .and_then(Bag::verify_payload)
            .map(|payload| (Self::from_payload(source, payload), source.to_string()))
    }
    /// Copy files from a local bucket to a local directory
    pub async fn copy_files(self, options: &BucketOptions) -> ApiResult<TransferManifest> {
        let Self { name, source, .. } = self;
        match source.is_local() {
            | true => source.transfer_files(name, options).await,
            | false => Ok(TransferManifest::empty(name, source.identity())),
        }
    }
    /// Get the hosting domain for a repository-backed bucket
    #[cfg(test)]
    pub(crate) fn domain(&self) -> ApiResult<String> {
        match &self.source {
            | BucketSource::Repository(repository) => RepositoryBucketSource::new(repository).domain(),
            | BucketSource::Archive(_) => Err(eyre!("Archive payloads have no repository domain")),
        }
    }
    /// Downloads this bucket to a local output directory and ingests supported research activity data.
    pub async fn download(self, options: &BucketOptions, provenance: Option<&str>) -> ApiResult<usize> {
        self.transfer(options, TransferPolicy::Download, provenance).await
    }
    /// Download files from a remote bucket to a local directory
    pub async fn download_files(self, options: &BucketOptions) -> ApiResult<TransferManifest> {
        self.transfer_files(options).await
    }
    /// Imports this bucket and ingests supported research activity data.
    pub async fn import(self, options: &BucketOptions, provenance: Option<&str>) -> ApiResult<usize> {
        self.transfer(options, TransferPolicy::Import, provenance).await
    }
    /// Transfers this bucket and ingests supported research activity data.
    pub async fn transfer(self, options: &BucketOptions, policy: TransferPolicy, provenance: Option<&str>) -> ApiResult<usize> {
        match (policy, &self.source, options.common.offline) {
            | (TransferPolicy::Download, source, _) if source.is_download_restricted() => {
                Err(eyre!("Bucket download requires a remote bucket source"))
            }
            | (_, source, true) if !source.is_local() => Err(eyre!("Offline mode cannot transfer remote bucket sources")),
            | _ => self.transfer_files(options).await.and_then(|manifest| {
                let manifest = match provenance {
                    | Some(source) => manifest.with_source_locator(source),
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
    async fn transfer_files(self, options: &BucketOptions) -> ApiResult<TransferManifest> {
        let Self { name, source, .. } = self;
        source.transfer_files(name, options).await
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
impl BucketSource {
    pub(crate) fn identity(&self) -> BucketSourceIdentity {
        match self {
            | Self::Archive(archive) => ArchiveBucketSource::new(archive).source_identity(),
            | Self::Repository(repository) => RepositoryBucketSource::new(repository).source_identity(),
        }
    }
    async fn transfer_files(&self, name: Option<String>, options: &BucketOptions) -> ApiResult<TransferManifest> {
        match self {
            | Self::Archive(archive) => transfer_bucket_files(name, &ArchiveBucketSource::new(archive), options).await,
            | Self::Repository(repository) => transfer_bucket_files(name, &RepositoryBucketSource::new(repository), options).await,
        }
    }
}
impl ResolvedBucket {
    /// Transfers this bucket and ingests supported research activity data.
    ///
    /// This consumes the resolved source so the temporary directory that owns an archive payload cannot be dropped while its files are being read.
    pub async fn transfer(self, options: &BucketOptions, policy: TransferPolicy) -> ApiResult<usize> {
        self.bucket.transfer(options, policy, self.provenance.as_deref()).await
    }
}
impl From<Bucket> for SourceInput {
    fn from(value: Bucket) -> Self {
        Self::Bucket(value)
    }
}
impl From<String> for SourceInput {
    fn from(value: String) -> Self {
        Self::Uri(value)
    }
}
impl SourceInput {
    /// Resolves this source to a bucket under the caller's transfer policy.
    ///
    /// Local directories, local archive files, `file:` URIs, and URIs that name a supported archive resolve to a local directory bucket or to a verified BagIt archive payload.
    /// Any other URI resolves to a remote repository bucket.
    ///
    /// `format` forces archive handling instead of inferring it from the source.
    pub async fn resolve(self, format: Option<MimeType>, offline: bool) -> ApiResult<ResolvedBucket> {
        match self {
            | Self::Bucket(bucket) => Ok(ResolvedBucket {
                bucket,
                provenance: None,
                _temporary: None,
            }),
            | Self::Uri(source) => {
                let path = file_uri_to_path(&source).unwrap_or_else(|_| PathBuf::from(&source));
                let inferred = MimeType::from(source.as_str());
                let forced = format.is_some();
                match (path.is_dir(), path.is_file(), forced, inferred) {
                    | (true, _, false, _) => match path.canonicalize() {
                        | Ok(path) => Ok(ResolvedBucket {
                            bucket: Bucket::from(path.as_path()),
                            provenance: Some(source),
                            _temporary: None,
                        }),
                        | Err(why) => Err(why.into()),
                    },
                    | (_, true, _, _) | (_, _, true, _) | (_, _, _, MimeType::Gzip | MimeType::SevenZip | MimeType::Tar | MimeType::Zip) => {
                        match TemporaryDirectory::create(&format!("{APPLICATION}-bagit")) {
                            | Ok(temporary) => match Bucket::archive(&source, format, &temporary, offline).await {
                                | Ok((bucket, provenance)) => Ok(ResolvedBucket {
                                    bucket,
                                    provenance: Some(provenance),
                                    _temporary: Some(temporary),
                                }),
                                | Err(why) => Err(why),
                            },
                            | Err(why) => Err(why.into()),
                        }
                    }
                    | (_, _, _, _) if source.contains("://") => match Bucket::try_from(source.as_str()) {
                        | Ok(bucket) => Ok(ResolvedBucket {
                            bucket,
                            provenance: Some(source),
                            _temporary: None,
                        }),
                        | Err(why) => Err(why),
                    },
                    | _ => Err(eyre!("Source does not exist or has an unsupported format: {source}")),
                }
            }
        }
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
