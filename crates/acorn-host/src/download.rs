//! Bounded streaming downloads with verified, no-clobber installation.
use crate::{
    fs::{file_checksum, FsError},
    http::{redact_url, HttpClient, HttpError},
};
use acorn_core::util::{Checksum, ChecksumAlgorithm};
use futures::StreamExt;
use reqwest::{Method, Response};
use ring::digest::{Algorithm as RingAlgorithm, SHA1_FOR_LEGACY_USE_ONLY, SHA256, SHA384, SHA512};
use std::{
    fs::{hard_link, remove_file as remove_file_sync},
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::{
    fs::{create_dir_all, remove_file, File, OpenOptions},
    io::AsyncWriteExt,
};

/// Streaming download failure.
#[derive(Debug, Error)]
pub enum DownloadError {
    /// The destination already exists and was not replaced.
    #[error("download destination already exists: {0:?}")]
    DestinationExists(PathBuf),
    /// The expected checksum algorithm is not available in the native adapter.
    #[error("unsupported download checksum algorithm: {0}")]
    UnsupportedChecksum(ChecksumAlgorithm),
    /// The downloaded checksum did not match the expectation.
    #[error("download checksum mismatch (expected {expected}, got {actual})")]
    ChecksumMismatch {
        /// Expected hexadecimal checksum.
        expected: String,
        /// Actual hexadecimal checksum.
        actual: String,
    },
    /// The response exceeded its configured byte limit.
    #[error("download from {url} exceeded {maximum} bytes")]
    TooLarge {
        /// Maximum accepted byte count.
        maximum: u64,
        /// Redacted source URL.
        url: String,
    },
    /// The destination has no usable sibling partial filename.
    #[error("download destination has no filename: {0:?}")]
    InvalidDestination(PathBuf),
    /// Filesystem operation failed.
    #[error(transparent)]
    File(#[from] FsError),
    /// HTTP operation failed.
    #[error(transparent)]
    Http(#[from] HttpError),
    /// Asynchronous file I/O failed.
    #[error("download file operation failed for {path:?}: {source}")]
    Io {
        /// Path involved in the operation.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}
struct Algorithm(&'static RingAlgorithm);
/// Optional download integrity and size expectations.
#[derive(Clone, Debug, Default)]
pub struct DownloadExpectation {
    checksum: Option<Checksum>,
    maximum_bytes: Option<u64>,
}
/// Metadata for an installed download.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DownloadReceipt {
    /// Number of bytes written.
    pub bytes: u64,
    /// Installed destination path.
    pub path: PathBuf,
}
struct PartialCleanup(PathBuf);
impl TryFrom<ChecksumAlgorithm> for Algorithm {
    type Error = DownloadError;
    fn try_from(algorithm: ChecksumAlgorithm) -> Result<Self, Self::Error> {
        match algorithm {
            | ChecksumAlgorithm::Sha1 => Ok(Self(&SHA1_FOR_LEGACY_USE_ONLY)),
            | ChecksumAlgorithm::Sha256 => Ok(Self(&SHA256)),
            | ChecksumAlgorithm::Sha384 => Ok(Self(&SHA384)),
            | ChecksumAlgorithm::Sha512 => Ok(Self(&SHA512)),
            | unsupported => Err(DownloadError::UnsupportedChecksum(unsupported)),
        }
    }
}
impl DownloadExpectation {
    /// Create an unconstrained expectation.
    pub fn new() -> Self {
        Self::default()
    }
    /// Require a matching checksum.
    pub fn with_checksum(mut self, checksum: Checksum) -> Self {
        self.checksum = Some(checksum);
        self
    }
    /// Reject a response larger than `maximum` bytes.
    pub fn with_maximum_bytes(mut self, maximum: u64) -> Self {
        self.maximum_bytes = Some(maximum);
        self
    }
}
impl Drop for PartialCleanup {
    fn drop(&mut self) {
        let _ = remove_file_sync(&self.0);
    }
}
/// Stream a URL to a sibling partial file and install it without replacing a destination.
pub async fn download_to(
    client: &HttpClient,
    url: &str,
    destination: impl AsRef<Path>,
    expectation: DownloadExpectation,
) -> Result<DownloadReceipt, DownloadError> {
    let destination = destination.as_ref().to_path_buf();
    match destination.exists() {
        | true => Err(DownloadError::DestinationExists(destination)),
        | false => match partial_path(&destination) {
            | Ok(partial) => {
                let _cleanup = PartialCleanup(partial.clone());
                let result = download_partial(client, url, &destination, &partial, expectation).await;
                if result.is_err() {
                    let _ = remove_file(&partial).await;
                }
                result
            }
            | Err(error) => Err(error),
        },
    }
}
async fn download_partial(
    client: &HttpClient,
    url: &str,
    destination: &Path,
    partial: &Path,
    expectation: DownloadExpectation,
) -> Result<DownloadReceipt, DownloadError> {
    let response = match client.parse_and_authorize(url) {
        | Ok(parsed) => {
            let diagnostic = redact_url(parsed.as_str());
            client
                .response(Method::GET, parsed)
                .await
                .map(|response| (response, diagnostic))
                .map_err(Into::into)
        }
        | Err(error) => Err(error.into()),
    }
    .and_then(|(response, diagnostic)| {
        validate_download_response(response, &diagnostic, expectation.maximum_bytes).map(|response| (response, diagnostic))
    });
    match response {
        | Ok((response, diagnostic)) => match create_destination_parent(destination).await {
            | Ok(()) => match open_partial(partial).await {
                | Ok(file) => {
                    let result = write_response(file, response, partial, &diagnostic, expectation.maximum_bytes)
                        .await
                        .and_then(|bytes| {
                            verify_checksum(partial, expectation.checksum)
                                .and_then(|()| install_partial(partial, destination))
                                .map(|()| DownloadReceipt {
                                    bytes,
                                    path: destination.to_path_buf(),
                                })
                        });
                    match result {
                        | Ok(receipt) => {
                            let _ = remove_file(partial).await;
                            Ok(receipt)
                        }
                        | Err(error) => Err(error),
                    }
                }
                | Err(error) => Err(error),
            },
            | Err(error) => Err(error),
        },
        | Err(error) => Err(error),
    }
}
async fn create_destination_parent(destination: &Path) -> Result<(), DownloadError> {
    match destination.parent() {
        | Some(parent) => create_dir_all(parent).await.map_err(|source| DownloadError::Io {
            path: parent.to_path_buf(),
            source,
        }),
        | None => Ok(()),
    }
}
fn install_partial(partial: &Path, destination: &Path) -> Result<(), DownloadError> {
    hard_link(partial, destination).map_err(|source| {
        if source.kind() == io::ErrorKind::AlreadyExists {
            DownloadError::DestinationExists(destination.to_path_buf())
        } else {
            DownloadError::Io {
                path: destination.to_path_buf(),
                source,
            }
        }
    })
}
async fn open_partial(partial: &Path) -> Result<File, DownloadError> {
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(partial)
        .await
        .map_err(|source| DownloadError::Io {
            path: partial.to_path_buf(),
            source,
        })
}
fn partial_path(destination: &Path) -> Result<PathBuf, DownloadError> {
    destination
        .file_name()
        .map(|name| destination.with_file_name(format!(".{}.{}.acorn-part", name.to_string_lossy(), nanoid::nanoid!(8))))
        .ok_or_else(|| DownloadError::InvalidDestination(destination.to_path_buf()))
}
fn validate_download_response(response: Response, diagnostic: &str, maximum: Option<u64>) -> Result<Response, DownloadError> {
    if !response.status().is_success() {
        Err(HttpError::Status {
            status: response.status(),
            url: diagnostic.to_string(),
        }
        .into())
    } else if maximum
        .zip(response.content_length())
        .is_some_and(|(maximum, content_length)| content_length > maximum)
    {
        Err(DownloadError::TooLarge {
            maximum: maximum.unwrap_or_default(),
            url: diagnostic.to_string(),
        })
    } else {
        Ok(response)
    }
}
fn verify_checksum(partial: &Path, expectation: Option<Checksum>) -> Result<(), DownloadError> {
    match expectation {
        | Some(expected) => Algorithm::try_from(expected.algorithm.clone())
            .and_then(|algorithm| file_checksum(partial, Some(algorithm.0)).map_err(Into::into))
            .and_then(|actual| match actual.checksum_value.eq_ignore_ascii_case(&expected.checksum_value) {
                | true => Ok(()),
                | false => Err(DownloadError::ChecksumMismatch {
                    expected: expected.checksum_value,
                    actual: actual.checksum_value,
                }),
            }),
        | None => Ok(()),
    }
}
async fn write_response(mut file: File, response: Response, partial: &Path, diagnostic: &str, maximum: Option<u64>) -> Result<u64, DownloadError> {
    let mut bytes = 0_u64;
    let mut stream = response.bytes_stream();
    let written = loop {
        match stream.next().await {
            | Some(Ok(chunk)) => {
                let next_bytes = bytes.saturating_add(u64::try_from(chunk.len()).unwrap_or(u64::MAX));
                if maximum.is_some_and(|maximum| next_bytes > maximum) {
                    break Err(DownloadError::TooLarge {
                        maximum: maximum.unwrap_or_default(),
                        url: diagnostic.to_string(),
                    });
                }
                match file.write_all(&chunk).await {
                    | Ok(()) => bytes = next_bytes,
                    | Err(source) => {
                        break Err(DownloadError::Io {
                            path: partial.to_path_buf(),
                            source,
                        });
                    }
                }
            }
            | Some(Err(_)) => {
                break Err(HttpError::Request {
                    url: diagnostic.to_string(),
                    kind: "response stream failed",
                }
                .into());
            }
            | None => break Ok(bytes),
        }
    };
    match written {
        | Ok(bytes) => file.flush().await.map(|()| bytes).map_err(|source| DownloadError::Io {
            path: partial.to_path_buf(),
            source,
        }),
        | Err(error) => Err(error),
    }
}
