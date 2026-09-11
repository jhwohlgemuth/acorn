//! Transfer primitives for bucket synchronization
//!
//! Contains `TransferManifest`, `TransferItem`, `TransferPolicy` and associated helpers.
use crate::io::adapter::migrate::Migration;
use crate::io::config::bucket::{BucketExtension, BucketOptions};
use crate::io::database::schema::Table;
use crate::io::database::{Database, Provenance, ResearchActivityCandidate};
use crate::io::{read_file, with_progress, ApiResult, FromPath, InputOutput, ProgressType};
use crate::util::constants::app::SUPPORTED_RAD_FILETYPES;
use crate::util::{is_filetype, Label};
use acorn_core::options::{CommonRuntime, RuntimeOptions};
use acorn_core::prelude::HashMap;
use acorn_core::util::{suffix, MimeType};
use acorn_core::Repository;
use acorn_schema::pid::{Identifier, PID};
use acorn_schema::research_activity::ResearchActivity;
use color_eyre::eyre::{eyre, Report};
use core::future::Future;
use core::iter::once;
use jiff::Timestamp;
use owo_colors::OwoColorize;
use std::path::{Component, Path, PathBuf};

/// Policy controlling which bucket sources a transfer accepts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferPolicy {
    /// Require a remote repository source.
    Download,
    /// Permit local and remote sources.
    Import,
}
pub(crate) struct DownloadOptions<'a> {
    pub(crate) repository: &'a Repository,
    pub(crate) output: &'a Path,
    pub(crate) source: &'a Path,
    pub(crate) destination: &'a Path,
    pub(crate) branch: Option<&'a str>,
    pub(crate) clobber: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransferItem {
    pub(crate) source: PathBuf,
    pub(crate) destination: PathBuf,
}
/// Files successfully transferred from one configured bucket.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferManifest {
    /// Configured bucket name.
    pub bucket: Option<String>,
    /// Repository location used for the transfer.
    pub repository: String,
    /// Relative paths written beneath the output directory.
    pub files: Vec<PathBuf>,
}
impl From<&Path> for TransferItem {
    fn from(source: &Path) -> Self {
        Self {
            source: source.to_path_buf(),
            destination: source.to_path_buf(),
        }
    }
}
impl TransferItem {
    pub(crate) fn collect(paths: Vec<String>, flatten: bool) -> ApiResult<Vec<Self>> {
        let items = paths
            .into_iter()
            .map(PathBuf::from)
            .map(|path| Self::from(path.as_path()))
            .map(|item| match flatten {
                | true => item.flatten(),
                | false => Ok(item),
            })
            .collect::<ApiResult<Vec<_>>>();
        match items {
            | Ok(items) => items
                .iter()
                .try_fold(HashMap::<PathBuf, PathBuf>::new(), |mut destinations, item| {
                    let destination = &item.destination;
                    let safe = !destination.as_os_str().is_empty() && destination.components().all(|part| matches!(part, Component::Normal(_)));
                    match safe {
                        | false => Err(eyre!("Output path is unsafe — '{}'", item.destination.display())),
                        | true => match destinations.insert(item.destination.clone(), item.source.clone()) {
                            | Some(source) => Err(eyre!(
                                "Output path collision for '{}' — '{}' and '{}'",
                                item.destination.display(),
                                source.display(),
                                item.source.display()
                            )),
                            | None => Ok(destinations),
                        },
                    }
                })
                .map(|_| items),
            | Err(why) => Err(why),
        }
    }
    fn flatten(self) -> ApiResult<Self> {
        match self.source.file_name().map(PathBuf::from) {
            | Some(destination) => Ok(Self { destination, ..self }),
            | None => Err(eyre!("Cannot flatten repository path without a filename — {}", self.source.display())),
        }
    }
}
impl TransferManifest {
    /// Return the legacy reported count of transferred JSON and image files.
    pub fn count(&self) -> usize {
        let paths = self.files.iter().map(|path| path.display().to_string()).collect::<Vec<_>>();
        count_json_files(&paths).saturating_add(count_image_files(&paths))
    }
    pub(crate) fn canonicalize(self, options: &BucketOptions) -> ApiResult<Self> {
        let RuntimeOptions { extension, output, .. } = options;
        let BucketExtension { canonicalize, .. } = extension;
        match canonicalize {
            | false => Ok(self),
            | true => {
                let output = output.as_deref().unwrap_or_else(|| Path::new(""));
                let canonicalized = self
                    .files
                    .iter()
                    .filter(is_filetype(SUPPORTED_RAD_FILETYPES))
                    .try_fold((), |(), relative| {
                        let path = output.join(relative);
                        ResearchActivity::migrate(path.clone())
                            .map(|_| ())
                            .map_err(|why| eyre!("Failed to canonicalize transferred RAD {} — {why}", path.display()))
                    });
                canonicalized.map(|()| self)
            }
        }
    }
    /// Ingest transferred RAD files into the canonical research activity table.
    pub fn ingest(&self, options: &BucketOptions) -> ApiResult<()> {
        let RuntimeOptions { extension, output, .. } = options;
        let BucketExtension {
            database_path,
            no_local_database,
            strict,
            ..
        } = extension;
        match no_local_database {
            | true => Ok(()),
            | false => {
                let output = output.as_deref().unwrap_or_else(|| Path::new(""));
                let database = Database::<Table>::from_path(database_path.as_ref());
                self.files
                    .iter()
                    .filter(is_filetype(SUPPORTED_RAD_FILETYPES))
                    .filter(|relative| {
                        let path = output.join(relative);
                        MimeType::from_path(&path) != MimeType::Markdown || read_file(&path).is_ok_and(ResearchActivity::is_markdown)
                    })
                    .try_fold((), |(), relative| {
                        let path = output.join(relative);
                        match strict {
                            | true => ResearchActivity::read_strict(path.clone()),
                            | false => ResearchActivity::read(path.clone()),
                        }
                        .and_then(|rad| {
                            serde_json::to_value(&rad)
                                .map_err(Report::from)
                                .and_then(|rad_json| database.create_or_enrich(self.candidate(&rad, rad_json, relative)).map(|_| ()))
                        })
                        .map_err(|why| eyre!("Failed to ingest transferred RAD {} — {why}", path.display()))
                    })
            }
        }
    }
    fn candidate(&self, rad: &ResearchActivity, rad_json: serde_json::Value, relative: &Path) -> ResearchActivityCandidate {
        let pairs = [
            (PID::DOI, rad.meta.doi.as_ref()),
            (PID::Handle, rad.meta.handle.as_ref()),
            (PID::ISBN, rad.meta.books.as_ref()),
            (PID::Patent, rad.meta.patents.as_ref()),
            (PID::RAID, rad.meta.raid.as_ref()),
            (PID::SWHID, rad.meta.swhid.as_ref()),
        ];
        let pid_keys = pairs.into_iter().flat_map(|(kind, values)| {
            values.into_iter().flatten().filter_map(move |value| {
                Identifier::init()
                    .kind(kind.clone())
                    .value(value)
                    .build()
                    .normalized()
                    .map(|identifier| format!("{}:{}", identifier.kind.as_str(), identifier.value))
            })
        });
        let rad_key = format!("rad:{}:{}", self.repository, rad.meta.identifier);
        let prov = Provenance::Bucket {
            bucket: self.bucket.clone(),
            repository: self.repository.clone(),
            relative_path: relative.display().to_string(),
            observed_at: Timestamp::now().to_string(),
        };
        ResearchActivityCandidate::new(
            rad_json,
            pid_keys.chain(once(rad_key)).collect(),
            vec![serde_json::to_value(prov).unwrap_or_default()],
        )
    }
}
pub(crate) fn count_image_files(paths: &[String]) -> usize {
    paths.iter().filter(|&x| has_image_extension(x)).count()
}
pub(crate) fn count_json_files(paths: &[String]) -> usize {
    paths.iter().filter(|&path| path.to_lowercase().ends_with(".json")).count()
}
#[allow(clippy::ptr_arg)]
pub(crate) fn has_image_extension(path: &String) -> bool {
    path.to_lowercase().ends_with(".png") || path.to_lowercase().ends_with(".jpg")
}
pub(crate) fn operations_complete_message(name: Option<String>, json_count: usize, image_count: usize) -> String {
    let total = json_count.saturating_add(image_count);
    let message = if json_count != image_count {
        let recommendation = if json_count > image_count {
            "Do you need to add some images?"
        } else {
            "Do you need to add some JSON files?"
        };
        format!(
            " ({} data file{}, {} image{} - {})",
            json_count.yellow(),
            suffix(json_count),
            image_count.yellow(),
            suffix(image_count),
            recommendation.italic(),
        )
    } else {
        "".to_string()
    };
    let bucket_description = match name {
        | Some(value) => format!("{} bucket", value.to_uppercase().cyan()),
        | None => "<URL>".cyan().to_string(),
    };
    format!(
        "{}Obtained {} file{} from {bucket_description}{}",
        if total > 0 { Label::CHECKMARK } else { Label::CAUTION },
        if total > 0 {
            total.green().to_string()
        } else {
            total.yellow().to_string()
        },
        suffix(total),
        message,
    )
}
pub(crate) async fn transfer_bucket_files<F, Fut>(
    name: Option<String>,
    repository: String,
    items: Vec<TransferItem>,
    options: &BucketOptions,
    verb: &'static str,
    operation: F,
) -> ApiResult<TransferManifest>
where
    F: Fn(TransferItem) -> Fut,
    Fut: Future<Output = ApiResult<()>>,
{
    let CommonRuntime { quiet, threads, .. } = &options.common;
    let source_paths = items.iter().map(|item| item.source.display().to_string()).collect::<Vec<_>>();
    let total_data = count_json_files(&source_paths);
    let total_images = count_image_files(&source_paths);
    let message = move |item: &TransferItem| format!("{verb} {}", item.source.display());
    let finish_name = name.clone();
    let finish_message = |_| operations_complete_message(finish_name, total_data, total_images);
    let progress_type = match quiet {
        | true => ProgressType::Silent,
        | false => ProgressType::Bar,
    };
    let files = items.iter().map(|item| item.destination.clone()).collect::<Vec<_>>();
    with_progress(items, message, operation, finish_message, Some(*threads), progress_type)
        .await
        .map(|_| TransferManifest {
            bucket: name,
            repository,
            files,
        })
}
