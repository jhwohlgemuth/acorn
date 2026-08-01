//! File download utilities
use crate::io::api::huggingface;
use crate::io::{apply_progress_style, create_progress_bar, file_checksum, finish_progress_bar, http, ApiResult, ProgressType};
use crate::prelude::{create_dir_all, remove_file, Path, PathBuf, Vec};
use crate::util::constants::{app::DEFAULT_HUGGINGFACE_DOMAIN, env::HUGGINGFACE_TOKEN_VARIABLE_NAMES};
use crate::util::Label;
use color_eyre::eyre::eyre;
use fluent_uri::Uri;
use futures::future::join_all;
use std::fs::rename;
use std::path::Component;
use tracing::info;

/// A single downloadable file with URL, path, expected size, and optional SHA-256 checksum.
#[derive(Clone, Debug)]
pub struct DownloadItem {
    /// URL to download from
    pub url: String,
    /// Relative path for the downloaded file
    pub path: String,
    /// Expected file size in bytes (used for progress bar and resume validation)
    pub size: Option<u64>,
    /// Expected SHA-256 checksum (hex-encoded, lowercase)
    pub sha: Option<String>,
}
/// A batch of [`DownloadItem`]s destined for a single output directory.
#[derive(Clone, Debug)]
pub struct DownloadItems {
    /// Target directory for all downloaded files
    pub destination: PathBuf,
    /// Items to download
    pub items: Vec<DownloadItem>,
    /// Suppress progress output
    pub quiet: bool,
    /// Skip SHA-256 verification after download
    pub skip_verify_checksum: bool,
}
/// Internal task that pairs a [`DownloadItem`] with its resolved paths.
#[derive(Clone, Debug)]
pub struct DownloadTask {
    /// The item to download
    pub item: DownloadItem,
    /// Temporary `.part` file path (atomic-write pattern)
    pub part: PathBuf,
    /// Final destination path
    pub target: PathBuf,
    /// Suppress progress output
    pub quiet: bool,
    /// Skip SHA-256 verification after download
    pub skip_verify_checksum: bool,
}
impl DownloadItem {
    const PART_EXTENSION: &'static str = "part";
    const MODEL_ERROR_MESSAGE: &'static str = "Failed to download model file";
    const HUGGINGFACE_HOST: &'static str = DEFAULT_HUGGINGFACE_DOMAIN;
    const HUGGINGFACE_ERROR_MESSAGE: &'static str = "Failed to download Hugging Face file";
    const HUGGINGFACE_AUTH_ERROR_MESSAGE: &'static str =
        "Hugging Face denied access to this file; verify your token has access to the repository and gated/Xet files";

    /// Returns the relative local path as a [`PathBuf`].
    pub fn local_path(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }
    /// Returns `true` if the path contains `..`, `/`, or a drive prefix.
    pub fn has_unsafe_path(&self) -> bool {
        self.local_path()
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    }
    /// Resolves the full target path under the given destination directory.
    pub fn target(&self, destination: &Path) -> PathBuf {
        destination.join(self.local_path())
    }
    /// Returns the partial download path (appends `.part` extension).
    pub fn partial_path(&self, destination: &Path) -> PathBuf {
        let target = self.target(destination);
        target.with_extension(format!(
            "{}{}",
            target
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| format!("{value}."))
                .unwrap_or_default(),
            Self::PART_EXTENSION
        ))
    }
    /// Returns `true` if the URL points to huggingface.co or a subdomain.
    pub fn is_huggingface(&self) -> bool {
        Uri::parse(self.url.as_str())
            .ok()
            .and_then(|uri| uri.authority().map(|authority| authority.host().to_string()))
            .is_some_and(|host| host == Self::HUGGINGFACE_HOST || host.ends_with(&format!(".{}", Self::HUGGINGFACE_HOST)))
    }
    /// Returns a human-readable error message for download failures.
    pub fn error_message(&self) -> &'static str {
        match self.is_huggingface() {
            | true => Self::HUGGINGFACE_ERROR_MESSAGE,
            | false => Self::MODEL_ERROR_MESSAGE,
        }
    }
    /// Returns an auth-related error message, if applicable.
    pub fn auth_error_message(&self) -> Option<String> {
        match (self.is_huggingface(), huggingface::has_auth_token()) {
            | (true, true) => Some(Self::HUGGINGFACE_AUTH_ERROR_MESSAGE.into()),
            | _ => Some(format!(
                "Model download requires authentication; set {}",
                HUGGINGFACE_TOKEN_VARIABLE_NAMES.join(", ")
            )),
        }
    }
    /// Returns `true` if the file at `target` exists and matches the expected size.
    pub fn is_complete_at(&self, target: &Path) -> bool {
        target.exists()
            && self
                .size
                .is_none_or(|size| target.metadata().map(|metadata| metadata.len() == size).unwrap_or(false))
    }
    /// Verifies the downloaded file size matches the expected size.
    pub fn verify_size(&self, path: &Path) -> ApiResult<()> {
        match self.size {
            | Some(size) => match path.metadata() {
                | Ok(metadata) if metadata.len() == size => Ok(()),
                | Ok(metadata) => Err(eyre!(
                    "Downloaded model size did not match expected size (expected {size}, got {})",
                    metadata.len()
                )),
                | Err(why) => Err(eyre!("Failed to inspect downloaded model file {} — {why}", path.display())),
            },
            | None => Ok(()),
        }
    }
    /// Verifies the SHA-256 checksum of the downloaded file.
    pub fn verify_checksum(&self, path: &Path, skip_verify_checksum: bool) -> ApiResult<()> {
        match (skip_verify_checksum, self.sha.as_ref()) {
            | (false, Some(expected)) => match file_checksum(path.to_path_buf(), None) {
                | Some(actual) if actual.checksum_value.eq_ignore_ascii_case(expected) => Ok(()),
                | Some(actual) => Err(eyre!(
                    "Model download is incomplete (checksum mismatch for {} (expected {expected}, got {}))",
                    self.path,
                    actual.checksum_value
                )),
                | None => Err(eyre!("Model download is incomplete (failed to compute SHA-256 for {})", self.path)),
            },
            | _ => Ok(()),
        }
    }
}
impl DownloadItems {
    /// Creates a new batch of download items.
    pub fn new(destination: &Path, items: Vec<DownloadItem>, quiet: bool, skip_verify_checksum: bool) -> Self {
        Self {
            destination: destination.to_path_buf(),
            items,
            quiet,
            skip_verify_checksum,
        }
    }
    /// Downloads all items concurrently and returns the first error encountered.
    pub async fn download(self) -> ApiResult<()> {
        let DownloadItems {
            destination,
            items,
            quiet,
            skip_verify_checksum,
        } = self;
        let results = join_all(items.into_iter().map(|item| {
            let destination = destination.clone();
            async move { DownloadTask::new(item, &destination, quiet, skip_verify_checksum).download().await }
        }))
        .await;
        results.into_iter().find(|result| result.is_err()).unwrap_or(Ok(()))
    }
}
impl DownloadTask {
    /// Creates a new download task with resolved paths.
    pub fn new(item: DownloadItem, destination: &Path, quiet: bool, skip_verify_checksum: bool) -> Self {
        Self {
            part: item.partial_path(destination),
            target: item.target(destination),
            item,
            quiet,
            skip_verify_checksum,
        }
    }
    /// Executes the download, including size and checksum verification.
    pub async fn download(self) -> ApiResult<()> {
        match self.item.has_unsafe_path() {
            | true => Err(eyre!("Model file path is unsafe: {}", self.item.path)),
            | false if self.item.is_complete_at(&self.target) => {
                if !self.quiet {
                    println!("=> {} Skipping existing model file {}", Label::pass(), self.target.display());
                }
                Ok(())
            }
            | false => {
                let parent_ready = match self.part.parent() {
                    | Some(parent) => create_dir_all(parent).map_err(|why| eyre!("Failed to create model output directory — {why}")),
                    | None => Err(eyre!("Model output path has no parent directory")),
                };
                match parent_ready {
                    | Ok(_) => match self.resume().await {
                        | Ok(resume_from) => match self
                            .clone()
                            .stream(resume_from)
                            .await
                            .and_then(|_| self.item.verify_size(&self.part))
                            .and_then(|_| self.item.verify_checksum(&self.part, self.skip_verify_checksum))
                        {
                            | Ok(_) => rename(&self.part, &self.target)
                                .map_err(|why| eyre!("Failed to finalize model download {} — {why}", self.target.display())),
                            | Err(why) => {
                                if self.part.exists() {
                                    let _ = remove_file(&self.part);
                                }
                                Err(why)
                            }
                        },
                        | Err(why) => Err(why),
                    },
                    | Err(why) => Err(why),
                }
            }
        }
    }
    async fn resume(&self) -> ApiResult<Option<u64>> {
        match self.part.exists() {
            | false => Ok(None),
            | true => {
                let metadata = self
                    .part
                    .metadata()
                    .map_err(|why| eyre!("Failed to inspect partial download {} — {why}", self.part.display()));
                match metadata {
                    | Ok(data) => {
                        let part_size = data.len();
                        match self.item.size {
                            | Some(expected) if part_size > 0 && part_size < expected => {
                                let headers = self.item.is_huggingface().then(huggingface::auth_headers);
                                let supports_ranges = http::supports_byte_ranges(&self.item.url, headers).await.unwrap_or(false);
                                if supports_ranges {
                                    Ok(Some(part_size))
                                } else {
                                    remove_file(&self.part)
                                        .map_err(|why| eyre!("Failed to remove stale partial download {} — {why}", self.part.display()))
                                        .map(|_| None)
                                }
                            }
                            | _ => remove_file(&self.part)
                                .map_err(|why| eyre!("Failed to remove stale partial download {} — {why}", self.part.display()))
                                .map(|_| None),
                        }
                    }
                    | Err(why) => Err(why),
                }
            }
        }
    }
    async fn stream(self, resume_from: Option<u64>) -> ApiResult<()> {
        let Self {
            quiet, item, part, target, ..
        } = self;
        let progress_type = match (quiet, item.size) {
            | (true, _) => ProgressType::Silent,
            | (false, Some(_)) => ProgressType::Bar,
            | (false, None) => ProgressType::Spinner,
        };
        let progress = create_progress_bar(item.size.and_then(|value| usize::try_from(value).ok()).unwrap_or_default(), progress_type);
        match item.size {
            | Some(_) => apply_progress_style(&progress, "  {spinner:.green} {bytes:>10}/{total_bytes:<10} [{bar:40.green}] {msg}"),
            | None => apply_progress_style(&progress, "  {spinner:.green} {bytes:>10} {msg}"),
        }
        progress.set_message(target.display().to_string());
        if !quiet {
            info!("=> {} Downloading {}", Label::run(), target.display());
        }
        let auth_error_message = item.auth_error_message();
        let result = http::download_with_progress(
            &item.url,
            &part,
            |downloaded, total| {
                if let Some(total) = total {
                    progress.set_length(total);
                }
                progress.set_position(downloaded);
            },
            item.is_huggingface().then(huggingface::auth_headers),
            Some(item.error_message()),
            auth_error_message.as_deref(),
            resume_from,
        );
        match result.await {
            | Ok(_) => {
                if !quiet {
                    finish_progress_bar(&progress, format!("{}Downloaded {}", Label::CHECKMARK, target.display()));
                }
                Ok(())
            }
            | Err(why) => {
                progress.finish_and_clear();
                Err(why)
            }
        }
    }
}
