//! Transfer primitives for bucket synchronization
//!
//! Contains `TransferManifest`, `TransferItem`, `TransferPolicy` and associated helpers.
use crate::{
    io::{
        adapter::migrate::Migration,
        config::{
            bucket::{
                source::{BucketSourceAdapter, BucketSourceIdentity, BucketSourceItem},
                BucketExtension, BucketOptions,
            },
            is_filtered_path, is_ignored_path, FilterSet,
        },
        database::{schema::Table, Database, Provenance, ResearchActivityCandidate},
        read_file, with_progress, ApiResult, FromPath, InputOutput, ProgressType,
    },
    util::{constants::app::SUPPORTED_RAD_FILETYPES, is_filetype},
};
use acorn_core::{
    options::{CommonRuntime, RuntimeOptions},
    prelude::HashMap,
    util::{suffix, MimeType},
};
use acorn_host::{
    fs::{FsError, SafePath, TemporaryDirectory},
    terminal::Label,
};
use acorn_schema::{
    pid::{Identifier, PID},
    research_activity::ResearchActivity,
};
use color_eyre::eyre::{eyre, Report};
use core::iter::once;
use jiff::Timestamp;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::{
    fs::read,
    path::{Path, PathBuf},
    sync::Mutex,
};

/// Policy controlling which bucket sources a transfer accepts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferPolicy {
    /// Require a remote bucket source.
    Download,
    /// Permit local and remote sources.
    Import,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransferItem {
    pub(crate) source: BucketSourceItem,
    pub(crate) destination: SafePath,
}
/// Items successfully transferred from one configured bucket.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferManifest {
    bucket: Option<String>,
    items: Vec<TransferManifestItem>,
    source: BucketSourceIdentity,
}
/// One item successfully transferred from a configured bucket source.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferManifestItem {
    key: String,
    output_path: PathBuf,
    size: Option<u64>,
    source_path: PathBuf,
    version: Option<String>,
}
impl TransferItem {
    async fn apply<A>(self, adapter: &A, output: &Path, staging: Option<&TemporaryDirectory>, clobber: bool, write_lock: &Mutex<()>) -> ApiResult<()>
    where
        A: BucketSourceAdapter + Sync,
    {
        match (adapter.is_local(), clobber, staging) {
            | (true, true, Some(staging)) => match self.destination.materialize_under(staging.path()) {
                | Ok(staged) => match adapter.retrieve(&self.source, &staged).await {
                    | Ok(()) => self
                        .destination
                        .write_file(output, true, write_lock, || async {
                            read(&staged).map_err(|source| FsError::Io { path: staged, source })
                        })
                        .await
                        .map_err(Report::from),
                    | Err(why) => Err(why),
                },
                | Err(why) => Err(why.into()),
            },
            | (false, true, _) => {
                let target = match write_lock.lock() {
                    | Ok(_guard) => self.destination.prepare(output).map_err(Report::from),
                    | Err(why) => Err(eyre!("Failed to lock bucket output — {why}")),
                };
                match target {
                    | Ok(target) => adapter.retrieve(&self.source, &target).await,
                    | Err(why) => Err(why),
                }
            }
            | _ => match self.destination.materialize_under(output) {
                | Ok(target) => adapter.retrieve(&self.source, &target).await,
                | Err(why) => Err(why.into()),
            },
        }
    }
    pub(crate) fn collect(items: Vec<BucketSourceItem>, flatten: bool) -> ApiResult<Vec<Self>> {
        let items = items
            .into_iter()
            .map(|source| Self {
                destination: source.path().clone(),
                source,
            })
            .map(|item| match flatten {
                | true => item.flatten(),
                | false => Ok(item),
            })
            .collect::<ApiResult<Vec<_>>>();
        match items {
            | Ok(items) => items
                .iter()
                .try_fold(HashMap::<SafePath, SafePath>::new(), |mut destinations, item| {
                    let destination = &item.destination;
                    match destinations.insert(destination.clone(), item.source.path().clone()) {
                        | Some(source) => Err(eyre!(
                            "Output path collision for '{}' — '{}' and '{}'",
                            destination.as_path().display(),
                            source.as_path().display(),
                            item.source.path().as_path().display()
                        )),
                        | None => Ok(destinations),
                    }
                })
                .map(|_| items),
            | Err(why) => Err(why),
        }
    }
    fn flatten(self) -> ApiResult<Self> {
        match self.source.path().as_path().file_name().map(PathBuf::from).map(SafePath::new) {
            | Some(Ok(destination)) => Ok(Self { destination, ..self }),
            | Some(Err(why)) => Err(why.into()),
            | None => Err(eyre!(
                "Cannot flatten bucket path without a filename — {}",
                self.source.path().as_path().display()
            )),
        }
    }
    fn manifest_item(&self) -> TransferManifestItem {
        TransferManifestItem {
            key: self.source.key().to_string(),
            output_path: self.destination.as_path().to_path_buf(),
            size: self.source.size(),
            source_path: self.source.path().as_path().to_path_buf(),
            version: self.source.version().map(str::to_string),
        }
    }
}
impl TransferManifest {
    pub(crate) fn empty(bucket: Option<String>, source: BucketSourceIdentity) -> Self {
        Self {
            bucket,
            items: Vec::new(),
            source,
        }
    }
    /// Return the configured bucket name.
    pub fn bucket(&self) -> Option<&str> {
        self.bucket.as_deref()
    }
    /// Return the reported count of transferred JSON and image files.
    pub fn count(&self) -> usize {
        let paths = self.items.iter().map(|item| item.output_path.display().to_string()).collect::<Vec<_>>();
        count_json_files(&paths).saturating_add(count_image_files(&paths))
    }
    /// Iterate over relative paths written beneath the output directory.
    pub fn files(&self) -> impl Iterator<Item = &Path> {
        self.items.iter().map(|item| item.output_path.as_path())
    }
    /// Return transferred items in source discovery order.
    pub fn items(&self) -> &[TransferManifestItem] {
        &self.items
    }
    /// Return the credential-free source identity.
    pub const fn source(&self) -> &BucketSourceIdentity {
        &self.source
    }
    pub(crate) fn with_source_locator(self, locator: &str) -> Self {
        Self {
            source: self.source.with_locator(locator),
            ..self
        }
    }
    pub(crate) fn canonicalize(self, options: &BucketOptions) -> ApiResult<Self> {
        let RuntimeOptions { extension, output, .. } = options;
        let BucketExtension { canonicalize, .. } = extension;
        match canonicalize {
            | false => Ok(self),
            | true => {
                let output = output.as_deref().unwrap_or_else(|| Path::new(""));
                let canonicalized = self
                    .items
                    .iter()
                    .filter(|item| is_filetype(SUPPORTED_RAD_FILETYPES)(item.output_path.as_path()))
                    .try_fold((), |(), item| {
                        let path = output.join(&item.output_path);
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
                self.items
                    .iter()
                    .filter(|item| is_filetype(SUPPORTED_RAD_FILETYPES)(item.output_path.as_path()))
                    .filter(|item| {
                        let path = output.join(&item.output_path);
                        MimeType::from_path(&path) != MimeType::Markdown || read_file(&path).is_ok_and(ResearchActivity::is_markdown)
                    })
                    .try_fold((), |(), item| {
                        let path = output.join(&item.output_path);
                        match strict {
                            | true => ResearchActivity::read_strict(path.clone()),
                            | false => ResearchActivity::read(path.clone()),
                        }
                        .and_then(|rad| {
                            serde_json::to_value(&rad)
                                .map_err(Report::from)
                                .and_then(|rad_json| database.create_or_enrich(self.candidate(&rad, rad_json, item)).map(|_| ()))
                        })
                        .map_err(|why| eyre!("Failed to ingest transferred RAD {} — {why}", path.display()))
                    })
            }
        }
    }
    fn candidate(&self, rad: &ResearchActivity, rad_json: serde_json::Value, item: &TransferManifestItem) -> ResearchActivityCandidate {
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
        let rad_key = format!("rad:{}:{}", self.source.locator(), rad.meta.identifier);
        let prov = Provenance::Bucket {
            bucket: self.bucket.clone(),
            item: serde_json::to_value(item).unwrap_or_default(),
            observed_at: Timestamp::now().to_string(),
            schema_version: 1,
            source: serde_json::to_value(&self.source).unwrap_or_default(),
        };
        ResearchActivityCandidate::new(
            rad_json,
            pid_keys.chain(once(rad_key)).collect(),
            vec![serde_json::to_value(prov).unwrap_or_default()],
        )
    }
}
impl TransferManifestItem {
    /// Return the opaque source key used for retrieval.
    pub fn key(&self) -> &str {
        &self.key
    }
    /// Return the relative path written beneath the output directory.
    pub fn output_path(&self) -> &Path {
        &self.output_path
    }
    /// Return the provider-declared item size.
    pub const fn size(&self) -> Option<u64> {
        self.size
    }
    /// Return the original provider-reported virtual path.
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }
    /// Return the exact source version used for retrieval.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
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
pub(crate) async fn transfer_bucket_files<A>(name: Option<String>, adapter: &A, options: &BucketOptions) -> ApiResult<TransferManifest>
where
    A: BucketSourceAdapter + Sync,
{
    match (FilterSet::compile(options), adapter.items().await.and_then(BucketSourceItem::validate)) {
        | (Ok(filters), Ok(items)) => {
            let selected = items
                .into_iter()
                .filter(|item| {
                    let path = item.path().as_path().display().to_string();
                    !is_ignored_path(&path, &filters.ignore) && is_filtered_path(&path, &filters.filter)
                })
                .collect::<Vec<_>>();
            match TransferItem::collect(selected, options.extension.flatten) {
                | Ok(items) => transfer_selected(name, adapter, items, options).await,
                | Err(why) => Err(why),
            }
        }
        | (Err(why), _) | (_, Err(why)) => Err(why),
    }
}
async fn transfer_selected<A>(name: Option<String>, adapter: &A, items: Vec<TransferItem>, options: &BucketOptions) -> ApiResult<TransferManifest>
where
    A: BucketSourceAdapter + Sync,
{
    let CommonRuntime { quiet, threads, .. } = &options.common;
    let source_paths = items
        .iter()
        .map(|item| item.source.path().as_path().display().to_string())
        .collect::<Vec<_>>();
    let total_data = count_json_files(&source_paths);
    let total_images = count_image_files(&source_paths);
    let verb = match adapter.is_local() {
        | true => "Copying",
        | false => "Downloading",
    };
    let message = move |item: &TransferItem| format!("{verb} {}", item.source.path().as_path().display());
    let finish_name = name.clone();
    let finish_message = |_| operations_complete_message(finish_name, total_data, total_images);
    let progress_type = match quiet {
        | true => ProgressType::Silent,
        | false => ProgressType::Bar,
    };
    let manifest_items = items.iter().map(TransferItem::manifest_item).collect::<Vec<_>>();
    let output = options.output.clone().unwrap_or_default();
    let staging = match (adapter.is_local(), options.extension.clobber, items.is_empty()) {
        | (true, true, false) => TemporaryDirectory::create("acorn-bucket-transfer").map(Some).map_err(Report::from),
        | _ => Ok(None),
    };
    match staging {
        | Ok(staging) => {
            let write_lock = Mutex::new(());
            let operation = |item: TransferItem| item.apply(adapter, &output, staging.as_ref(), options.extension.clobber, &write_lock);
            with_progress(items, message, operation, finish_message, Some(*threads), progress_type)
                .await
                .map(|_| TransferManifest {
                    bucket: name,
                    items: manifest_items,
                    source: adapter.source_identity(),
                })
        }
        | Err(why) => Err(why),
    }
}
