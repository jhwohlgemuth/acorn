//! Bucket transfer and local copy logic
pub mod source;
use self::source::{BucketInput, BucketSourceAdapter, RepositoryBucketSource};
use crate::io::{
    bagit::Bag,
    config::transfer::{transfer_bucket_files, TransferManifest, TransferPolicy},
    http::download_with_progress,
    ApiResult, ArchiveCandidate,
};
use acorn_core::{options::RuntimeOptions, util::MimeType};
use acorn_host::fs::{file_uri_to_path, TemporaryDirectory};
use bon::Builder;
use color_eyre::eyre::eyre;
use futures::{future, stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
pub use source::BucketSource;
use std::path::{Path, PathBuf};

/// Options for bucket transfers
pub type BucketOptions = RuntimeOptions<BucketExtension, PathBuf>;
/// Configured collection of research activity files
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", try_from = "BucketInput")]
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
    #[serde(rename = "codeRepository")]
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
impl Bucket {
    /// Resolves and validates a BagIt archive as a local payload bucket.
    pub async fn archive(source: &str, format: Option<MimeType>, temporary: &TemporaryDirectory, offline: bool) -> ApiResult<(Self, String)> {
        let local = file_uri_to_path(source).unwrap_or_else(|_| PathBuf::from(source));
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
        let local = self.source.is_local();
        match (policy, local, options.common.offline) {
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
    /// Get the hosting domain for a repository-backed bucket
    #[cfg(test)]
    pub(crate) fn domain(&self) -> ApiResult<String> {
        match &self.source {
            | BucketSource::Repository(repository) => RepositoryBucketSource::new(repository).domain(),
        }
    }
    /// Copy files from a local bucket to a local directory
    pub async fn copy_files(self, options: &BucketOptions) -> ApiResult<TransferManifest> {
        let Self { name, source, .. } = self;
        match source {
            | BucketSource::Repository(repository) => {
                let adapter = RepositoryBucketSource::new(&repository);
                match adapter.is_local() {
                    | true => transfer_bucket_files(name, &adapter, options).await,
                    | false => Ok(TransferManifest {
                        bucket: name,
                        repository: adapter.manifest_identity(),
                        files: Vec::new(),
                    }),
                }
            }
        }
    }
    /// Download files from a remote bucket to a local directory
    pub async fn download_files(self, options: &BucketOptions) -> ApiResult<TransferManifest> {
        let Self { name, source, .. } = self;
        match source {
            | BucketSource::Repository(repository) => {
                let adapter = RepositoryBucketSource::new(&repository);
                transfer_bucket_files(name, &adapter, options).await
            }
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
