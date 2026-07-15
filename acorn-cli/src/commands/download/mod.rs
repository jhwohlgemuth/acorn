use self::model::ModelDownloadError;
use crate::cli::CommandOptions;
use crate::commands::preflight;
use acorn::io::api::huggingface;
use acorn::io::config::{ApplicationConfiguration, Bucket, BucketOptions};
use acorn::io::database::schema::{ActivityRow, Table};
use acorn::io::database::{Database, Operations};
use acorn::io::{apply_progress_style, create_progress_bar, file_checksum, finish_progress_bar, http, ApiResult, InputOutput, ProgressType};
use acorn::prelude::{create_dir_all, remove_file, Path, PathBuf, Vec};
use acorn::util::constants::app::DEFAULT_CONFIG_FILENAMES;
use acorn::util::{suffix, Label, StringConversion};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{eyre, Report, Result};
use fluent_uri::Uri;
use owo_colors::OwoColorize;
use std::fs::rename;
use std::path::Component;
use tracing::error;
use tracing::info;

pub mod model;

#[derive(Clone, Debug)]
struct DownloadItem {
    url: String,
    path: String,
    size: Option<u64>,
    sha: Option<String>,
}
#[derive(Clone, Debug)]
struct DownloadItems {
    destination: PathBuf,
    items: Vec<DownloadItem>,
    quiet: bool,
    skip_verify_checksum: bool,
}
#[derive(Clone, Debug)]
struct DownloadTask {
    item: DownloadItem,
    part: PathBuf,
    target: PathBuf,
    quiet: bool,
    skip_verify_checksum: bool,
}
impl DownloadItem {
    fn local_path(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }
    fn has_unsafe_path(&self) -> bool {
        self.local_path()
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    }
    fn target(&self, destination: &Path) -> PathBuf {
        destination.join(self.local_path())
    }
    fn partial_path(&self, destination: &Path) -> PathBuf {
        let target = self.target(destination);
        target.with_extension(format!(
            "{}part",
            target
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| format!("{value}."))
                .unwrap_or_default()
        ))
    }
    fn is_huggingface(&self) -> bool {
        Uri::parse(self.url.as_str())
            .ok()
            .and_then(|uri| uri.authority().map(|authority| authority.host().to_string()))
            .is_some_and(|host| host == "huggingface.co" || host.ends_with(".huggingface.co"))
    }
    fn error_message(&self) -> &'static str {
        match self.is_huggingface() {
            | true => "Failed to download Hugging Face file",
            | false => "Failed to download model file",
        }
    }
    fn auth_error_message(&self) -> Option<String> {
        match (self.is_huggingface(), huggingface::has_auth_token()) {
            | (true, true) => {
                Some("Hugging Face denied access to this model file; verify your token has access to the repository and gated/Xet files".to_string())
            }
            | _ => Some("Model download requires authentication; set HF_TOKEN, HF_API_TOKEN, or HUGGINGFACE_HUB_TOKEN".to_string()),
        }
    }
    fn is_complete_at(&self, target: &Path) -> bool {
        target.exists()
            && self
                .size
                .is_none_or(|size| target.metadata().map(|metadata| metadata.len() == size).unwrap_or(false))
    }
    fn verify_size(&self, path: &Path) -> ApiResult<()> {
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
    fn verify_checksum(&self, path: &Path, skip_verify_checksum: bool) -> ApiResult<()> {
        match (skip_verify_checksum, self.sha.as_ref()) {
            | (true, _) => Ok(()),
            | (false, Some(expected)) => match file_checksum(path.to_path_buf(), None) {
                | Some(actual) if actual.checksum_value.eq_ignore_ascii_case(expected) => Ok(()),
                | Some(actual) => Err(eyre!(
                    "{}",
                    ModelDownloadError::IncompleteDownload(format!(
                        "Checksum mismatch for {} (expected {expected}, got {})",
                        self.path, actual.checksum_value
                    ))
                )),
                | None => Err(eyre!(
                    "{}",
                    ModelDownloadError::IncompleteDownload(format!("Failed to compute SHA-256 for {}", self.path))
                )),
            },
            | (false, None) => Ok(()),
        }
    }
}
impl DownloadTask {
    fn new(item: DownloadItem, destination: &Path, quiet: bool, skip_verify_checksum: bool) -> Self {
        Self {
            part: item.partial_path(destination),
            target: item.target(destination),
            item,
            quiet,
            skip_verify_checksum,
        }
    }
    async fn resume_from(&self) -> ApiResult<Option<u64>> {
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
    async fn download(self) -> ApiResult<()> {
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
                    | Ok(_) => match self.resume_from().await {
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
    async fn stream(self, resume_from: Option<u64>) -> ApiResult<()> {
        let progress_type = match (self.quiet, self.item.size) {
            | (true, _) => ProgressType::Silent,
            | (false, Some(_)) => ProgressType::Bar,
            | (false, None) => ProgressType::Spinner,
        };
        let progress = create_progress_bar(
            self.item.size.and_then(|value| usize::try_from(value).ok()).unwrap_or_default(),
            progress_type,
        );
        match self.item.size {
            | Some(_) => apply_progress_style(&progress, "  {spinner:.green} {bytes:>10}/{total_bytes:<10} [{bar:40.green}] {msg}"),
            | None => apply_progress_style(&progress, "  {spinner:.green} {bytes:>10} {msg}"),
        }
        progress.set_message(self.target.display().to_string());
        if !self.quiet {
            info!("=> {} Downloading {}", Label::run(), self.target.display());
        }
        let auth_error_message = self.item.auth_error_message();
        let result = http::download_with_progress(
            &self.item.url,
            &self.part,
            |downloaded, total| {
                if let Some(total) = total {
                    progress.set_length(total);
                }
                progress.set_position(downloaded);
            },
            self.item.is_huggingface().then(huggingface::auth_headers),
            Some(self.item.error_message()),
            auth_error_message.as_deref(),
            resume_from,
        );
        match result.await {
            | Ok(_) => {
                if !self.quiet {
                    finish_progress_bar(&progress, format!("{}Downloaded {}", Label::CHECKMARK, self.target.display()));
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
impl DownloadItems {
    fn new(destination: &Path, items: Vec<DownloadItem>, quiet: bool, skip_verify_checksum: bool) -> Self {
        Self {
            destination: destination.to_path_buf(),
            items,
            quiet,
            skip_verify_checksum,
        }
    }
    async fn download(self) -> ApiResult<()> {
        let DownloadItems {
            destination,
            items,
            quiet,
            skip_verify_checksum,
        } = self;
        let results = futures::future::join_all(items.into_iter().map(|item| {
            let destination = destination.clone();
            async move { DownloadTask::new(item, &destination, quiet, skip_verify_checksum).download().await }
        }))
        .await;
        results.into_iter().find(|result| result.is_err()).unwrap_or(Ok(()))
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn run(
    config: &Option<PathBuf>,
    url: &[String],
    filter: &[String],
    ignore: &[String],
    output: &Option<PathBuf>,
    database_path: &Option<PathBuf>,
    threads: usize,
    verbose: &Verbosity,
    offline: bool,
) -> Result<(), Report> {
    let quiet = verbose.is_silent();
    preflight!(&CommandOptions::init()
        .offline(offline)
        .quiet(quiet)
        .threads(threads)
        .supported(true)
        .build());
    let options = BucketOptions::init()
        .filter(filter.to_vec())
        .ignore(ignore.to_vec())
        .quiet(quiet)
        .threads(threads)
        .build();
    let result: Result<(), Report> = if let Some(path) = ApplicationConfiguration::resolve(config) {
        match ApplicationConfiguration::read(path.clone()) {
            | Ok(content) => {
                let mut total: usize = 0;
                let mut branch_result: Result<(), Report> = Ok(());
                if let Some(buckets) = content.buckets {
                    if let Some(output_path) = output.clone() {
                        let bucket_options = options.clone().with_output(output_path);
                        for bucket in buckets {
                            let is_local = offline || bucket.code_repository.is_local();
                            let bucket_result = if is_local {
                                bucket.copy_files(&bucket_options).await
                            } else {
                                bucket.download_files(&bucket_options).await
                            };
                            match bucket_result {
                                | Ok(count) => {
                                    total = total.saturating_add(count);
                                }
                                | Err(why) => {
                                    if is_local {
                                        error!("=> {} Copy data — {why}", Label::fail());
                                    } else {
                                        error!("=> {} Download data — {why}", Label::fail());
                                    }
                                    branch_result = Err(why);
                                    break;
                                }
                            }
                        }
                    }
                }
                if branch_result.is_ok() {
                    if let Some(path) = output {
                        log_activity("download", path.to_absolute_string(), database_path, true);
                    }
                    if !quiet {
                        println!("\n\n   => {} Processed {} item{}\n", Label::pass(), total.green(), suffix(total));
                    }
                }
                branch_result
            }
            | Err(why) => {
                error!("=> {} Download data — {why}", Label::fail());
                log_activity("download", path.file_name_with_parent(), database_path, false);
                return Err(eyre!("Failed to process download configuration"));
            }
        }
    } else if !url.is_empty() {
        let mut total: usize = 0;
        let mut branch_result: Result<(), Report> = Ok(());
        if let Some(path) = output.clone() {
            let bucket_options = options.clone().with_output(path.clone());
            for value in url {
                if branch_result.is_ok() {
                    let bucket = Bucket::from(value.as_ref());
                    let is_local = offline || bucket.code_repository.is_local();
                    let bucket_result = if is_local {
                        bucket.copy_files(&bucket_options).await
                    } else {
                        bucket.download_files(&bucket_options).await
                    };
                    match bucket_result {
                        | Ok(count) => {
                            total = total.saturating_add(count);
                        }
                        | Err(why) => {
                            if is_local {
                                error!("=> {} Copy data — {why}", Label::fail());
                            } else {
                                error!("=> {} Download data — {why}", Label::fail());
                            }
                            branch_result = Err(why);
                        }
                    }
                }
            }
            if branch_result.is_ok() {
                log_activity("download", path.to_absolute_string(), database_path, true);
                if !quiet {
                    println!("\n\n   => {} Processed {} item{}\n", Label::pass(), total.green(), suffix(total));
                }
            }
        }
        branch_result
    } else if !quiet {
        println!(
            "\n\n   => {} No URL or configuration file found. Provide a URL or create one of: {}\n",
            Label::fail(),
            DEFAULT_CONFIG_FILENAMES.join(", ")
        );
        Ok(())
    } else {
        error!("=> {} No URL or configuration file found for download command", Label::fail());
        Ok(())
    };
    result
}

fn log_activity(command: &str, path: impl Into<String>, database_path: &Option<PathBuf>, success: bool) {
    let database = Database::<Table>::from_path(database_path.clone());
    let activity = ActivityRow::init()
        .command(command)
        .executed_at(chrono::Utc::now())
        .user_path(path)
        .success(success)
        .build();
    if let Err(why) = database.insert(activity) {
        error!("=> {} Save {command} activity — {why}", Label::fail());
    }
}

#[cfg(test)]
mod tests;
