//! Hugging Face Hub API helpers.
use super::{Endpoint, IntoHeaders, Params, RemoteResource, ResponseContent, TreeEntry};
use crate::io::api::{Param, RepositoryFileMetadata};
use crate::io::config::FilterSet;
use crate::io::download::{DownloadItem, DownloadItems};
use crate::io::sync::Shard;
use crate::io::{first_env_var, http, ApiResult, Source};
use crate::prelude::{HashMap, Path};
use crate::schema::agent::{ModelDetails, ModelSelector, Quantization, Weight, Weights, FALLBACK_MODEL_SUFFIXES};
use crate::util::constants::app::{
    DEFAULT_HUGGINGFACE_DOMAIN, DEFAULT_HUGGINGFACE_MINIMUM_DOWNLOAD_COUNT, DEFAULT_HUGGINGFACE_MODEL_REVISION, DEFAULT_HUGGINGFACE_SEARCH_LIMIT,
    DEFAULT_HUGGINGFACE_SEARCH_TERM,
};
use crate::util::constants::env::HUGGINGFACE_TOKEN_VARIABLE_NAMES;
use crate::util::{glob_matches, regex_to_glob, strip_suffixes, to_ascii_alphanumeric, Label};
use crate::{Location, Repository};
use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use async_trait::async_trait;
use axum::http::HeaderMap;
use bon::Builder;
use color_eyre::eyre::{eyre, Report};
use core::{cmp::Reverse, fmt, ops::Deref};
use futures::{future::join_all, TryStreamExt};
use hf_hub::{repository::ModelInfo, HFClient, HFError};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn};

/// Candidate repositories returned by model search.
pub type Candidates = Vec<Candidate>;
/// Function used to select from multiple interactive candidates.
pub type CandidateSelector = fn(Candidates, &Options) -> ApiResult<String>;
/// Select a repository from model search candidates.
pub trait CandidateSelection {
    /// Build eligible fallback candidates from Hugging Face model metadata.
    fn fallback(models: Vec<ModelInfo>, options: &SearchOptions) -> Self;
    /// Select a candidate using the selector configured in `options` when interaction is required.
    fn select(self, options: &Options) -> ApiResult<String>;
    /// Fallback selector used when the interactive picker is unavailable.
    ///
    /// Picks the most-downloaded candidate and warns the user.
    fn select_interactively(self, options: &Options) -> ApiResult<String>;
}
/// Trait for selecting and filtering files from a Hugging Face repository.
#[async_trait]
pub trait HuggingFaceRepository {
    /// Resolve SHA-256 checksums for repository files from `.sha256` sidecar files
    async fn checksums(&self, identifier: &str, revision: &str) -> ApiResult<HashMap<String, String>>;
    /// Download files and return the selected repository metadata
    async fn download(&self, options: &Options) -> ApiResult<Downloaded>;
    /// Filter repository files using glob patterns for include/exclude.
    fn filter(&self, filter: Option<&str>, ignore: Option<&str>) -> ApiResult<Self>
    where
        Self: Sized;
    /// Select preferred files based on a marker pattern, erroring on ambiguity.
    fn select(&self, policy: &FileSelectionPolicy<'_>) -> ApiResult<Self>
    where
        Self: Sized;
    /// Determine whether a failed download should trigger GGUF repository fallback discovery.
    fn should_use_fallback(&self, options: &Options) -> bool;
    /// Try to filter repository files using glob patterns, returning `None` when regex fallback is required.
    fn try_glob(&self, filter: Option<&str>, ignore: Option<&str>) -> Option<Self>
    where
        Self: Sized;
}
/// Extension trait for Hugging Face model metadata
pub trait ModelInfoExtension {
    /// Determine whether a Hugging Face model repository contains GGUF files
    fn has_gguf_files(&self) -> bool;
    /// Determine whether a Hugging Face model is an eligible fallback for an identifier.
    fn is_fallback_for(&self, identifier: &str) -> bool;
    /// Determine whether a Hugging Face model declares a given model as its exact base model.
    ///
    /// For example, a GGUF repository declaring `openai/gpt-oss-20b` is a derivative of
    /// `openai/gpt-oss-20b`.
    fn is_declared_derivative_of(&self, identifier: &str) -> bool;
    /// Determine whether a Hugging Face model declares a decorated variant of a given base model.
    ///
    /// For example, a GGUF repository declaring `nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-BF16`
    /// is a variant of `nvidia/nemotron-3-super-120b-a12b`.
    fn is_declared_variant_of(&self, identifier: &str) -> bool;
}
/// Errors produced by Hugging Face repository selection and download helpers
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HuggingFaceError {
    /// No GGUF model files were present in the repository tree
    NoGgufModelFiles,
    /// No GGUF quantization repository declares the requested base model
    NoGgufQuantizationRepository {
        /// Requested Hugging Face base-model identifier
        identifier: Box<str>,
    },
    /// The requested Hugging Face base-model identifier is invalid
    InvalidBaseModelIdentifier {
        /// Invalid Hugging Face base-model identifier
        identifier: Box<str>,
    },
    /// The Hugging Face client could not be initialized
    ClientInitializationFailed {
        /// Underlying client initialization error
        reason: Box<str>,
    },
    /// The Hugging Face model search could not be configured
    ModelSearchConfigurationFailed {
        /// Underlying search configuration error
        reason: Box<str>,
    },
    /// The Hugging Face model search failed while reading results
    ModelSearchFailed {
        /// Underlying model search error
        reason: Box<str>,
    },
}
/// GGUF repository metadata returned by fallback discovery.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Candidate {
    /// Hugging Face repository identifier
    pub id: String,
    /// Downloads during the repository's reported period
    pub downloads: u64,
    /// Repository like count
    pub likes: Option<u64>,
    /// Quantization formats detected in repository filenames
    pub quantizations: Vec<String>,
}
/// A repository-backed value together with its requested and resolved identifiers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryResolution<T> {
    requested: String,
    resolved: String,
    value: T,
}
/// Hugging Face repository files selected and downloaded for a model
#[derive(Builder, Clone, Debug, Eq, PartialEq)]
#[builder(start_fn = init, on(String, into))]
pub struct Downloaded {
    /// Hugging Face repository identifier
    pub identifier: String,
    /// Repository revision used for the download
    pub revision: String,
    /// Repository-relative paths downloaded from the repository
    pub files: Vec<String>,
}
/// Policy for selecting a preferred file when multiple candidates exist.
#[derive(Clone, Debug)]
pub struct FileSelectionPolicy<'a> {
    /// Substring to match in the filename (case-insensitive) to identify the preferred file.
    pub preferred_marker: &'a str,
    /// Error message to display when no files match.
    pub no_match_message: &'a str,
}
/// Options for Hugging Face model fallback searches.
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct SearchOptions {
    /// Base model identifier used to find related GGUF repositories.
    pub identifier: String,
    /// Hugging Face search filter.
    #[builder(default = DEFAULT_HUGGINGFACE_SEARCH_TERM.to_string())]
    pub term: String,
    /// Maximum number of repositories considered during fallback discovery.
    #[builder(default = DEFAULT_HUGGINGFACE_SEARCH_LIMIT)]
    pub limit: usize,
    /// Minimum number of downloads required for a fallback repository.
    #[builder(default = DEFAULT_HUGGINGFACE_MINIMUM_DOWNLOAD_COUNT)]
    pub minimum_download_count: u64,
    /// Whether fallback candidates should be selected interactively.
    #[builder(default)]
    pub interactive: bool,
}
/// Options for HuggingFace API requests
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct Options {
    /// Authentication token
    pub token: Option<String>,
    /// Model registry domain (default: "huggingface.co")
    #[builder(default = String::from("huggingface.co"))]
    pub domain: String,
    /// Model repository identifier
    pub identifier: Option<String>,
    /// Model repository revision (branch, tag, or commit)
    #[builder(default = String::from(DEFAULT_HUGGINGFACE_MODEL_REVISION))]
    pub revision: String,
    /// Repository path used for tree requests
    pub path: Option<String>,
    /// Regex pattern of files to include at a given path desginated by `path`
    pub filter: Option<String>,
    /// Regex pattern of files to ignore at a given path desginated by `path`
    pub ignore: Option<String>,
    /// Flag used to suppress output
    #[builder(default)]
    pub quiet: bool,
    /// Whether network-backed fallback discovery is disabled by offline mode
    #[builder(default)]
    pub offline: bool,
    /// Whether automatic GGUF quantization repository discovery is disabled
    #[builder(default)]
    pub no_fallback: bool,
    /// Maximum number of repositories considered during GGUF fallback discovery
    #[builder(default = DEFAULT_HUGGINGFACE_SEARCH_LIMIT)]
    pub search_limit: usize,
    /// Minimum number of downloads required for a GGUF fallback repository
    #[builder(default = DEFAULT_HUGGINGFACE_MINIMUM_DOWNLOAD_COUNT)]
    pub minimum_download_count: u64,
    /// Whether multiple GGUF fallback repositories should be selected interactively
    #[builder(default)]
    pub interactive: bool,
    /// Selector used when multiple repositories require interactive selection
    #[builder(default = select_first)]
    pub selector: CandidateSelector,
    /// Custom API parameters to include in every request
    #[builder(default = vec![])]
    pub custom_params: Vec<Param>,
    /// Skip SHA-256 checksum verification after download
    #[builder(default)]
    pub skip_verify_checksum: bool,
    /// Output directory for downloaded files
    pub output: Option<String>,
}
/// File metadata returned by the Hugging Face repository tree API
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HuggingFaceRepositoryFile {
    /// Repository-relative path
    pub path: String,
    /// File size in bytes, when provided by Hugging Face
    pub size: Option<u64>,
}
/// Repository identity and files returned by the Hugging Face tree API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HuggingFaceRepositoryFiles {
    /// Repository-relative file metadata.
    pub files: Vec<HuggingFaceRepositoryFile>,
    /// Hugging Face repository identifier.
    pub identifier: String,
    /// Repository revision used for the tree request.
    pub revision: String,
}
impl RepositoryFileMetadata for HuggingFaceRepositoryFile {
    fn path(&self) -> &str {
        &self.path
    }
    fn size(&self) -> Option<u64> {
        self.size
    }
}
impl HuggingFaceRepositoryFiles {
    /// Create repository file metadata with its repository identity.
    pub fn new(identifier: impl Into<String>, revision: impl Into<String>, files: Vec<HuggingFaceRepositoryFile>) -> Self {
        Self {
            files,
            identifier: identifier.into(),
            revision: revision.into(),
        }
    }
    /// Returns a complete split GGUF shard set as one logical candidate.
    /// Repository listings expose each shard as a separate file, which can otherwise look like multiple candidates.
    fn complete_shard_set(&self) -> Option<Self> {
        let shards = self
            .iter()
            .filter_map(|file| Shard::parts(&file.path.to_ascii_lowercase()))
            .collect::<Vec<_>>();
        shards.first().and_then(|(key, _, count)| {
            let indexes = shards
                .iter()
                .filter(|(candidate_key, _, candidate_count)| candidate_key == key && candidate_count == count)
                .map(|(_, index, _)| *index)
                .collect::<BTreeSet<_>>();
            let expected = (1..=*count).collect::<BTreeSet<_>>();
            match indexes == expected && usize::try_from(*count).ok() == Some(shards.len()) {
                | true => Some(self.clone()),
                | false => None,
            }
        })
    }
    fn candidate_report(&self) -> String {
        match self.len() {
            | 0 => "no GGUF files".to_string(),
            | 1 | 2 => self.iter().map(|file| file.path.as_str()).collect::<Vec<_>>().join(", "),
            | count => format!("{count} GGUF files"),
        }
    }
    fn parse(content: &str, options: &Options) -> ApiResult<Self> {
        #[derive(Deserialize)]
        struct ApiError {
            #[serde(alias = "message")]
            error: String,
        }
        match options.identifier.as_deref() {
            | Some(identifier) => serde_json::from_str::<Vec<TreeEntry>>(content)
                .map(|entries| {
                    let files = entries
                        .into_iter()
                        .filter(TreeEntry::is_file)
                        .map(HuggingFaceRepositoryFile::from)
                        .collect();
                    Self::new(identifier, &options.revision, files)
                })
                .map_err(|why| {
                    serde_json::from_str::<ApiError>(content).map_or_else(
                        |_| eyre!("Failed to parse Hugging Face repository file list for '{identifier}' — {why}"),
                        |response| eyre!("Hugging Face API rejected repository '{identifier}' — {}", response.error),
                    )
                }),
            | None => Err(eyre!("Missing Hugging Face repository identifier")),
        }
    }
    /// Resolve the configured repository or a selected GGUF fallback repository.
    pub async fn resolve(options: &Options) -> ApiResult<RepositoryResolution<Self>> {
        let Options {
            identifier,
            no_fallback,
            revision,
            search_limit,
            minimum_download_count,
            interactive,
            ..
        } = options;
        match identifier.as_deref() {
            | Some(identifier) => match repository_tree(identifier, revision).await {
                | Ok(repository) if repository.files.iter().any(|file| Quantization::from_gguf_filename(&file.path).is_some()) => {
                    Ok(RepositoryResolution::new(identifier, identifier, repository))
                }
                | Ok(_) if *no_fallback => Err(eyre!("No GGUF model files found for '{identifier}'")),
                | Err(why) if *no_fallback => Err(why),
                | Ok(_) | Err(_) => {
                    let search_options = SearchOptions::init()
                        .identifier(identifier)
                        .limit(*search_limit)
                        .minimum_download_count(*minimum_download_count)
                        .interactive(*interactive)
                        .build();
                    match search(&search_options).await {
                        | Ok(candidates) => match candidates.select(options) {
                            | Ok(resolved) => match repository_tree(&resolved, DEFAULT_HUGGINGFACE_MODEL_REVISION).await {
                                | Ok(repository) => Ok(RepositoryResolution::new(identifier, resolved, repository)),
                                | Err(why) => Err(why),
                            },
                            | Err(why) => Err(why),
                        },
                        | Err(why) => Err(why),
                    }
                }
            },
            | None => Err(eyre!("Missing Hugging Face repository identifier")),
        }
    }
}
impl Deref for HuggingFaceRepositoryFiles {
    type Target = [HuggingFaceRepositoryFile];
    fn deref(&self) -> &Self::Target {
        &self.files
    }
}
impl IntoIterator for HuggingFaceRepositoryFiles {
    type Item = HuggingFaceRepositoryFile;
    type IntoIter = alloc::vec::IntoIter<HuggingFaceRepositoryFile>;
    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
    }
}
impl From<HuggingFaceRepositoryFiles> for Weights {
    fn from(repository: HuggingFaceRepositoryFiles) -> Self {
        Weights(
            repository
                .files
                .into_iter()
                .filter_map(|file| {
                    Quantization::from_gguf_filename(&file.path).map(|quantization| Weight {
                        label: file.path.clone(),
                        url: format!(
                            "https://{DEFAULT_HUGGINGFACE_DOMAIN}/{}/resolve/{}/{}",
                            repository.identifier, repository.revision, file.path
                        ),
                        is_open: None,
                        quantization: Some(quantization),
                        size: file.size,
                    })
                })
                .collect(),
        )
    }
}
impl fmt::Display for Candidate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.id)
    }
}
impl From<ModelInfo> for Candidate {
    fn from(model: ModelInfo) -> Self {
        let quantizations = model
            .siblings
            .unwrap_or_default()
            .iter()
            .filter_map(|sibling| Quantization::from_gguf_filename(&sibling.rfilename).map(|quantization| quantization.to_string()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Self {
            id: model.id,
            downloads: model.downloads.unwrap_or_default(),
            likes: model.likes,
            quantizations,
        }
    }
}
impl CandidateSelection for Candidates {
    fn fallback(models: Vec<ModelInfo>, options: &SearchOptions) -> Self {
        let SearchOptions {
            identifier,
            minimum_download_count,
            interactive,
            ..
        } = options;
        let mut candidates = models
            .into_iter()
            .filter(|model| model.is_fallback_for(identifier))
            .map(Candidate::from)
            .filter(|candidate| !candidate.quantizations.is_empty())
            .collect::<Vec<_>>();
        candidates.sort_by_key(|candidate| Reverse(candidate.downloads));
        let rejected = |candidate: &&Candidate| candidate.downloads < *minimum_download_count;
        let report = |candidate: &Candidate| {
            let Candidate { id, downloads, .. } = candidate;
            let context = format!("{} {identifier}", "fallback from".italic());
            let reason = format!("({downloads} below minimum {minimum_download_count} popularity)");
            warn!("=> {}{} {} {}", Label::rejected(), id.yellow(), context.dimmed(), reason.dimmed(),);
        };
        match interactive {
            | true => candidates.iter().filter(rejected).for_each(report),
            | false => candidates.first().filter(rejected).into_iter().for_each(report),
        }
        candidates
            .into_iter()
            .filter(|candidate| candidate.downloads >= *minimum_download_count)
            .collect()
    }
    fn select(self, options: &Options) -> ApiResult<String> {
        let base_model = options.identifier.as_deref().unwrap_or_default();
        match self.as_slice() {
            | [] => Err(eyre!(HuggingFaceError::NoGgufQuantizationRepository {
                identifier: base_model.into()
            })),
            | [candidate] => Ok(candidate.to_string()),
            | [candidate, ..] if !options.interactive => {
                if !options.quiet {
                    info!(
                        "=> {} {} {} {}",
                        Label::using(),
                        candidate.green(),
                        format!("{} {base_model}", "fallback for".italic().dimmed()),
                        "(using most popular)".dimmed(),
                    );
                }
                Ok(candidate.to_string())
            }
            | _ => (options.selector)(self, options),
        }
    }
    fn select_interactively(self, options: &Options) -> ApiResult<String> {
        let candidate = self.first().ok_or_else(|| eyre!("GGUF search returned no candidates"))?;
        if !options.quiet {
            let reason = "(using most popular)";
            warn!("=> {} {} {}", Label::using(), candidate.green(), reason.dimmed());
        }
        Ok(candidate.to_string())
    }
}
impl Downloaded {
    /// Wrap this download as a direct repository resolution.
    pub fn into_resolution(self, identifier: impl Into<String>) -> RepositoryResolution<Self> {
        RepositoryResolution::direct(identifier, self)
    }
    /// Merge downloaded GGUF quantizations into existing model weights.
    pub fn merge_weights(&self, existing: Weights) -> Weights {
        let downloaded = self
            .files
            .iter()
            .filter_map(|path| {
                let Self { identifier, revision, .. } = self;
                Quantization::from_gguf_filename(path).map(|quantization| Weight {
                    label: quantization.to_string(),
                    url: format!("https://{DEFAULT_HUGGINGFACE_DOMAIN}/{identifier}/resolve/{revision}/{path}"),
                    is_open: None,
                    quantization: Some(quantization),
                    size: None,
                })
            })
            .filter(|candidate| {
                !existing
                    .0
                    .iter()
                    .any(|weight| weight.url == candidate.url && weight.quantization == candidate.quantization)
            })
            .collect::<Vec<_>>();
        Weights(existing.0.into_iter().chain(downloaded).collect())
    }
}
impl fmt::Display for HuggingFaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::NoGgufModelFiles => {
                write!(
                    formatter,
                    "No GGUF model files found; use a GGUF repository or provide --filter for another format"
                )
            }
            | Self::NoGgufQuantizationRepository { identifier } => {
                write!(formatter, "no GGUF quantization repo found for {identifier}")
            }
            | Self::InvalidBaseModelIdentifier { identifier } => {
                write!(formatter, "invalid Hugging Face base model identifier: {identifier}")
            }
            | Self::ClientInitializationFailed { reason } => {
                write!(formatter, "failed to initialize Hugging Face client: {reason}")
            }
            | Self::ModelSearchConfigurationFailed { reason } => {
                write!(formatter, "failed to configure Hugging Face model search: {reason}")
            }
            | Self::ModelSearchFailed { reason } => {
                write!(formatter, "failed to search Hugging Face models: {reason}")
            }
        }
    }
}
impl core::error::Error for HuggingFaceError {}
impl From<TreeEntry> for HuggingFaceRepositoryFile {
    fn from(entry: TreeEntry) -> Self {
        HuggingFaceRepositoryFile {
            path: entry.path,
            size: entry.size,
        }
    }
}
#[async_trait]
impl HuggingFaceRepository for HuggingFaceRepositoryFiles {
    /// Resolve SHA-256 checksums for repository files by fetching `.sha256` sidecar files.
    async fn checksums(&self, identifier: &str, revision: &str) -> ApiResult<HashMap<String, String>> {
        let candidates: Vec<_> = self
            .iter()
            .filter_map(|file| target_path_from_sidecar(&file.path).map(|target| (file.path.clone(), target)))
            .collect();
        let digests = join_all(candidates.into_iter().map(|(path, target)| async move {
            let url = format!("https://huggingface.co/{identifier}/resolve/{revision}/{path}");
            let content = match http::get(url).headers(auth_headers()).send().await {
                | Ok(response) => response.text().await.ok(),
                | Err(_) => None,
            };
            (target, content.as_deref().and_then(extract_sha256))
        }))
        .await;
        Ok(digests.into_iter().filter_map(|(target, digest)| digest.map(|d| (target, d))).collect())
    }
    async fn download(&self, options: &Options) -> ApiResult<Downloaded> {
        let Options {
            identifier,
            revision,
            filter,
            ignore,
            quiet,
            skip_verify_checksum,
            output,
            ..
        } = options;
        match (identifier.as_deref(), output.as_deref()) {
            | (Some(identifier), Some(output)) => match self.checksums(identifier, revision).await {
                | Ok(lookup) => match self.filter(filter.as_deref(), ignore.as_deref()) {
                    | Ok(files) => {
                        let selected_files = files.iter().map(|file| file.path.clone()).collect();
                        let items = files
                            .into_iter()
                            .map(|HuggingFaceRepositoryFile { path, size, .. }| {
                                let sha = lookup.get(&path).cloned();
                                let url = format!("https://huggingface.co/{identifier}/resolve/{revision}/{path}");
                                DownloadItem { path, sha, size, url }
                            })
                            .collect::<Vec<_>>();
                        DownloadItems::new(Path::new(output), items, *quiet, *skip_verify_checksum)
                            .download()
                            .await
                            .map(|()| {
                                Downloaded::init()
                                    .identifier(identifier)
                                    .revision(revision.as_str())
                                    .files(selected_files)
                                    .build()
                            })
                    }
                    | Err(why) => Err(why),
                },
                | Err(why) => Err(why),
            },
            | (None, _) => Err(eyre!("Missing Hugging Face repository identifier")),
            | (_, None) => Err(eyre!("Missing output directory")),
        }
    }
    fn filter(&self, filter: Option<&str>, ignore: Option<&str>) -> ApiResult<Self> {
        let filter_vec = filter.into_iter().map(String::from).collect::<Vec<_>>();
        let ignore_vec = ignore.into_iter().map(String::from).collect::<Vec<_>>();
        let filtered = match self.try_glob(filter, ignore) {
            | Some(result) => Ok(result),
            | None => FilterSet::filter(
                self.files.clone(),
                &filter_vec,
                &ignore_vec,
                |file: &HuggingFaceRepositoryFile| file.path.clone(),
                |_| true,
            )
            .map(|files| Self::new(&self.identifier, &self.revision, files)),
        };
        filtered.and_then(|files| {
            if filter.is_some() {
                match files.is_empty() {
                    | true => Err(eyre!("No model files matched --filter/--ignore")),
                    | false => Ok(files),
                }
            } else {
                let policy = FileSelectionPolicy {
                    preferred_marker: "Q4_K_M",
                    no_match_message: "No GGUF model files found; use a GGUF repository or provide --filter for another format",
                };
                files.select(&policy)
            }
        })
    }
    fn select(&self, policy: &FileSelectionPolicy<'_>) -> ApiResult<Self> {
        let Self { identifier, revision, .. } = self;
        let gguf_files = self
            .iter()
            .filter(|file| file.path.to_ascii_lowercase().ends_with(".gguf"))
            .cloned()
            .collect();
        let files = Self::new(identifier, revision, gguf_files);
        match files.len() {
            | 0 => Err(eyre!(HuggingFaceError::NoGgufModelFiles)),
            | 1 => Ok(files),
            | _ => {
                let preferred_files = files
                    .iter()
                    .filter(|file| file.path.to_ascii_uppercase().contains(policy.preferred_marker))
                    .cloned()
                    .collect();
                let preferred = Self::new(identifier, revision, preferred_files);
                match preferred.len() {
                    | 1 => Ok(preferred),
                    | _ => preferred.complete_shard_set().ok_or_else(|| {
                        eyre!(
                            "Multiple model file candidates found ({}); use --filter to choose one",
                            files.candidate_report()
                        )
                    }),
                }
            }
        }
    }
    fn should_use_fallback(&self, options: &Options) -> bool {
        let Options {
            identifier,
            offline,
            no_fallback,
            filter,
            ..
        } = options;
        let is_huggingface_repo_id = identifier
            .as_deref()
            .and_then(|value| value.split_once('/'))
            .is_some_and(|(owner, name)| !owner.is_empty() && !name.is_empty() && !name.contains('/'));
        let contains_gguf_files = self.iter().any(|file| file.path.to_ascii_lowercase().ends_with(".gguf"));
        is_huggingface_repo_id && filter.is_none() && !(contains_gguf_files || *offline || *no_fallback)
    }
    fn try_glob(&self, filter: Option<&str>, ignore: Option<&str>) -> Option<Self> {
        match (filter.map(regex_to_glob), ignore.map(regex_to_glob)) {
            | (Some(None), _) | (_, Some(None)) => None,
            | (filter_opt, ignore_opt) => {
                let filter_glob = filter_opt.flatten();
                let ignore_glob = ignore_opt.flatten();
                let filtered = self
                    .iter()
                    .filter(|file| {
                        let ignored = ignore_glob.as_ref().is_some_and(|pattern| glob_matches(&file.path, pattern));
                        let kept = filter_glob.as_ref().is_none_or(|pattern| glob_matches(&file.path, pattern));
                        !ignored && kept
                    })
                    .cloned()
                    .collect();
                Some(Self::new(&self.identifier, &self.revision, filtered))
            }
        }
    }
}
impl ModelInfoExtension for ModelInfo {
    fn has_gguf_files(&self) -> bool {
        self.siblings
            .as_ref()
            .is_some_and(|siblings| siblings.iter().any(|file| file.rfilename.to_ascii_lowercase().ends_with(".gguf")))
    }
    fn is_fallback_for(&self, identifier: &str) -> bool {
        self.id.eq_ignore_ascii_case(identifier) || self.is_declared_derivative_of(identifier) || self.is_declared_variant_of(identifier)
    }
    fn is_declared_derivative_of(&self, identifier: &str) -> bool {
        let declares_base_model = self
            .base_models
            .as_ref()
            .is_some_and(|models| models.iter().any(|value| value.eq_ignore_ascii_case(identifier)));
        let declares_card_base_model = self
            .card_data
            .as_ref()
            .and_then(|value| value.get("base_model"))
            .is_some_and(|value| match value {
                | Value::String(value) => value.eq_ignore_ascii_case(identifier),
                | Value::Array(values) => values
                    .iter()
                    .filter_map(Value::as_str)
                    .any(|value| value.eq_ignore_ascii_case(identifier)),
                | _ => false,
            });
        let has_quantized_base_model_tag = self
            .tags
            .as_ref()
            .is_some_and(|tags| tags.iter().any(|tag| is_quantized_base_model_tag(tag, identifier)));
        declares_base_model || declares_card_base_model || has_quantized_base_model_tag
    }
    fn is_declared_variant_of(&self, identifier: &str) -> bool {
        let matches = |value: &str| variant_matches(value, identifier);
        let declares_base_model = self.base_models.as_ref().is_some_and(|models| models.iter().any(|value| matches(value)));
        let declares_card_base_model = self
            .card_data
            .as_ref()
            .and_then(|value| value.get("base_model"))
            .is_some_and(|value| match value {
                | Value::String(value) => matches(value),
                | Value::Array(values) => values.iter().filter_map(Value::as_str).any(matches),
                | _ => false,
            });
        let has_quantized_base_model_tag = self.tags.as_ref().is_some_and(|tags| {
            tags.iter().any(|tag| {
                let mut parts = tag.splitn(3, ':');
                match (parts.next(), parts.next(), parts.next()) {
                    | (Some(kind), Some(relation), Some(value)) => {
                        kind.eq_ignore_ascii_case("base_model") && relation.eq_ignore_ascii_case("quantized") && matches(value)
                    }
                    | _ => false,
                }
            })
        });
        declares_base_model || declares_card_base_model || has_quantized_base_model_tag
    }
}
impl ModelDetails {
    /// Resolve this model to a GGUF fallback repository.
    pub async fn resolve_fallback(self, search_options: &SearchOptions, offline: bool) -> ApiResult<RepositoryResolution<Self>> {
        match search(search_options).await {
            | Ok(candidates) => {
                let options = Options::init()
                    .identifier(&search_options.identifier)
                    .offline(offline)
                    .search_limit(search_options.limit)
                    .minimum_download_count(search_options.minimum_download_count)
                    .interactive(search_options.interactive)
                    .quiet(true)
                    .build();
                candidates
                    .select(&options)
                    .map(|resolved| RepositoryResolution::new(&search_options.identifier, resolved, self))
            }
            | Err(why) => Err(why),
        }
    }
}
impl From<RepositoryResolution<ModelDetails>> for ModelDetails {
    fn from(resolution: RepositoryResolution<ModelDetails>) -> Self {
        let (requested, resolved, details) = resolution.into_parts();
        let details = details.with_id(&resolved);
        match requested == resolved {
            | true => details,
            | false => details.with_fallback(&requested),
        }
    }
}
impl<T> RepositoryResolution<T> {
    /// Create a repository resolution from requested and resolved identifiers.
    pub fn new(requested: impl Into<String>, resolved: impl Into<String>, value: T) -> Self {
        Self {
            requested: requested.into(),
            resolved: resolved.into(),
            value,
        }
    }
    /// Create a direct repository resolution from one identifier.
    pub fn direct(identifier: impl Into<String>, value: T) -> Self {
        let requested = identifier.into();
        let resolved = requested.clone();
        Self::new(requested, resolved, value)
    }
    /// Return whether fallback discovery selected a different repository.
    pub fn is_fallback(&self) -> bool {
        self.requested != self.resolved
    }
    /// Return the originally requested repository identifier.
    pub fn requested(&self) -> &str {
        &self.requested
    }
    /// Return the repository identifier that supplied the resolved value.
    pub fn resolved(&self) -> &str {
        &self.resolved
    }
    /// Return the resolved value.
    pub fn value(&self) -> &T {
        &self.value
    }
    /// Transform the resolved value while preserving repository identity.
    pub fn map<U>(self, transform: impl FnOnce(T) -> U) -> RepositoryResolution<U> {
        let (requested, resolved, value) = self.into_parts();
        RepositoryResolution::new(requested, resolved, transform(value))
    }
    /// Try to transform the resolved value while preserving repository identity.
    pub fn try_map<U, E>(self, transform: impl FnOnce(T) -> Result<U, E>) -> Result<RepositoryResolution<U>, E> {
        let (requested, resolved, value) = self.into_parts();
        transform(value).map(|value| RepositoryResolution::new(requested, resolved, value))
    }
    /// Consume the resolution into its identifiers and value.
    pub fn into_parts(self) -> (String, String, T) {
        (self.requested, self.resolved, self.value)
    }
}
impl From<&Source> for Weights {
    fn from(source: &Source) -> Self {
        match source {
            | Source::Remote { identifier, .. } => Self(vec![Weight {
                label: "Hugging Face".to_string(),
                url: format!("https://{DEFAULT_HUGGINGFACE_DOMAIN}/{identifier}"),
                is_open: None,
                quantization: None,
                size: None,
            }]),
            | Source::Local { path, .. } => Self(vec![Weight {
                label: "Local".to_string(),
                url: path.display().to_string(),
                is_open: Some(true),
                quantization: None,
                size: None,
            }]),
            | Source::Unsupported(_) => Self::default(),
        }
    }
}
impl Weights {
    /// Set the open-weight flag for the primary source.
    pub fn open(mut self, is_open: Option<bool>) -> Self {
        if let Some(weight) = self.0.first_mut() {
            weight.is_open = is_open;
        }
        self
    }
    /// Construct weights from a source location with explicit open-weight control.
    ///
    /// For remote Hugging Face sources, `is_open` controls the open-weight flag.
    /// For local sources, `is_open` is always `Some(true)`, regardless of the parameter.
    pub fn from_source(source: &Source, is_open: Option<bool>) -> Self {
        let weights = Self::from(source);
        // Only override is_open for Remote sources; Local sources are always Some(true)
        match source {
            | Source::Remote { .. } => weights.open(is_open),
            | _ => weights,
        }
    }
    /// Resolve the first non-empty model weight URL into a downloadable source.
    pub fn to_source(self, name: Option<String>) -> Option<Source> {
        self.0.into_iter().find(|weight| !weight.url.trim().is_empty()).map(|weight| {
            let location = Location::from(weight.url.as_str());
            let is_repository = location.host().is_some_and(|host| host.eq_ignore_ascii_case(DEFAULT_HUGGINGFACE_DOMAIN))
                && location
                    .path()
                    .is_some_and(|path| path.split('/').filter(|segment| !segment.is_empty()).count() == 2);
            let source = match is_repository {
                | true => Source::from(&Repository::HuggingFace { location }),
                | false => Source::from(weight.url.as_str()),
            };
            source.with_name(name.unwrap_or(weight.label))
        })
    }
}
/// Build Hugging Face authorization headers using configured environment token values.
pub fn auth_headers() -> HeaderMap {
    Params::new()
        .with_auth(first_env_var(&HUGGINGFACE_TOKEN_VARIABLE_NAMES).unwrap_or_default().as_str(), None)
        .build()
        .into_headers()
}
/// Fetch model metadata from Hugging Face API with base model and tag expansion
pub async fn fetch_model_info(provider: &str, name: &str) -> ApiResult<ModelInfo> {
    let client = HFClient::new().map_err(|why| {
        eyre!(HuggingFaceError::ClientInitializationFailed {
            reason: why.to_string().into()
        })
    });
    match client {
        | Ok(client) => client
            .model(provider, name)
            .info()
            .expand(vec![
                "baseModels".to_string(),
                "cardData".to_string(),
                "siblings".to_string(),
                "tags".to_string(),
            ])
            .send()
            .await
            .map_err(|why| eyre!(why).wrap_err(format!("Failed to read Hugging Face metadata for '{provider}/{name}'"))),
        | Err(e) => Err(e),
    }
}
/// Return whether a non-empty Hugging Face token is configured
pub fn has_auth_token() -> bool {
    first_env_var(&HUGGINGFACE_TOKEN_VARIABLE_NAMES).is_some()
}
/// Return whether a Hugging Face error means a model is unavailable to the caller
pub fn model_is_unavailable(error: &Report) -> bool {
    error.downcast_ref::<HFError>().is_some_and(|source| match source {
        | HFError::RepoNotFound { .. } | HFError::AuthRequired { .. } | HFError::Forbidden { .. } => true,
        | HFError::Http { context } => matches!(context.status.as_u16(), 401 | 403 | 404),
        | _ => false,
    })
}
/// Validate and parse a Hugging Face model identifier into (owner, name)
pub fn parse_identifier(identifier: &str) -> ApiResult<(&str, &str)> {
    match identifier.split_once('/') {
        | Some((owner, name)) if !owner.is_empty() && !name.is_empty() && !name.contains('/') => Ok((owner, name)),
        | _ => Err(eyre!("Invalid Hugging Face model identifier — {identifier}")),
    }
}
/// List files in a Hugging Face model repository at `revision`
pub async fn repository_tree(identifier: &str, revision: &str) -> ApiResult<HuggingFaceRepositoryFiles> {
    let template = "huggingface::api";
    let action = "tree";
    let options = Options::init().identifier(identifier).revision(revision).build();
    let params = Params::new()
        .with_auth(first_env_var(&HUGGINGFACE_TOKEN_VARIABLE_NAMES).unwrap_or_default().as_str(), None)
        .with_template("identifier", options.identifier.as_deref())
        .with_template("revision", Some(&options.revision))
        .with_keyvalue("recursive", Some("1"))
        .build();
    match Endpoint::from_template(template) {
        | Ok(endpoint) => match endpoint.invoke(action, Some(params)).await {
            | Ok(ResponseContent::Json(content)) => HuggingFaceRepositoryFiles::parse(&content, &options),
            | Ok(_) => Err(eyre!("Failed to list Hugging Face model files — response was not JSON")),
            | Err(why) => Err(eyre!("Failed to list Hugging Face model files — {why}")),
        },
        | Err(why) => Err(eyre!("Failed to configure Hugging Face API — {why}")),
    }
}
/// Find repositories matching the search options, sorted by downloads.
pub async fn search(options: &SearchOptions) -> ApiResult<Candidates> {
    let basename = ModelSelector::new(&options.identifier)
        .map(|selector| selector.fallback_search_name())
        .filter(|value| !value.is_empty());
    match (basename, HFClient::new()) {
        | (Some(basename), Ok(client)) => {
            let response = client
                .list_models()
                .search(&basename)
                .filter(&options.term)
                .sort("downloads")
                .full(true)
                .card_data(true)
                .limit(options.limit)
                .send();
            match response {
                | Ok(stream) => match stream.try_collect::<Vec<ModelInfo>>().await {
                    | Ok(models) => {
                        let candidates = Candidates::fallback(models, options);
                        match candidates.is_empty() {
                            | true => Err(eyre!(HuggingFaceError::NoGgufQuantizationRepository {
                                identifier: options.identifier.clone().into()
                            })),
                            | false => Ok(candidates),
                        }
                    }
                    | Err(why) => Err(eyre!(HuggingFaceError::ModelSearchFailed {
                        reason: why.to_string().into()
                    })),
                },
                | Err(why) => Err(eyre!(HuggingFaceError::ModelSearchConfigurationFailed {
                    reason: why.to_string().into()
                })),
            }
        }
        | (None, _) => Err(eyre!(HuggingFaceError::InvalidBaseModelIdentifier {
            identifier: options.identifier.clone().into()
        })),
        | (_, Err(why)) => Err(eyre!(HuggingFaceError::ClientInitializationFailed {
            reason: why.to_string().into()
        })),
    }
}
fn extract_sha256(content: &str) -> Option<String> {
    content
        .split_whitespace()
        .find(|token| token.len() == 64 && token.chars().all(|character| character.is_ascii_hexdigit()))
        .map(|value| value.to_ascii_lowercase())
}
fn is_quantized_base_model_tag(tag: &str, identifier: &str) -> bool {
    let mut parts = tag.splitn(3, ':');
    match (parts.next(), parts.next(), parts.next()) {
        | (Some(kind), Some(relation), Some(value)) => {
            kind.eq_ignore_ascii_case("base_model") && relation.eq_ignore_ascii_case("quantized") && value.eq_ignore_ascii_case(identifier)
        }
        | _ => false,
    }
}
fn select_first(candidates: Candidates, _options: &Options) -> ApiResult<String> {
    candidates
        .into_iter()
        .next()
        .map(|candidate| candidate.to_string())
        .ok_or_else(|| eyre!("GGUF search returned no candidates"))
}
fn target_path_from_sidecar(path: &str) -> Option<String> {
    path.strip_suffix(".sha256")
        .or_else(|| path.strip_suffix(".sha256sum"))
        .map(ToString::to_string)
}
fn variant_matches(declared: &str, requested: &str) -> bool {
    match (declared.split_once('/'), requested.split_once('/')) {
        | (Some((declared_owner, declared_name)), Some((requested_owner, requested_name))) => {
            let same_owner = declared_owner.eq_ignore_ascii_case(requested_owner);
            let owner_is_meta = requested_owner.eq_ignore_ascii_case("meta") && declared_owner.eq_ignore_ascii_case("meta-llama");
            let requested_name = to_ascii_alphanumeric(strip_suffixes(FALLBACK_MODEL_SUFFIXES, requested_name));
            let name_matches = to_ascii_alphanumeric(declared_name).contains(&requested_name);
            (same_owner || owner_is_meta) && !requested_name.is_empty() && name_matches
        }
        | _ => false,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{
        extract_sha256, target_path_from_sidecar, Candidate, CandidateSelection, Candidates, Downloaded, HuggingFaceRepositoryFiles, ModelInfo,
        ModelInfoExtension, RepositoryResolution, SearchOptions, Value,
    };
    use crate::schema::agent::{ModelDetails, Weight, Weights};
    use serde_json::json;

    fn model(value: Value) -> ModelInfo {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn test_has_gguf_files() {
        let gguf = model(json!({"id": "mozilla/test-llama", "siblings": [{"rfilename": "tiny-llama.gguf"}]}));
        assert!(gguf.has_gguf_files());
        let non_gguf = model(json!({"id": "openai/gpt-oss-20b", "siblings": [{"rfilename": "model.safetensors"}]}));
        assert!(!non_gguf.has_gguf_files());
    }
    #[test]
    fn test_is_declared_derivative() {
        let candidate = model(json!({"id": "community/quantized", "baseModels": ["OpenAI/GPT-OSS-2B"]}));
        assert!(candidate.is_declared_derivative_of("openai/gpt-oss-2b"));
        let candidate = model(json!({"id": "community/quantized", "tags": ["base_model:quantized:openai/gpt-oss-2b"]}));
        assert!(candidate.is_declared_derivative_of("openai/gpt-oss-2b"));
        let candidate = model(json!({
            "id": "community/quantized",
            "baseModels": ["other/model"],
            "tags": ["base_model:quantized:other/model"]
        }));
        assert!(!candidate.is_declared_derivative_of("openai/gpt-oss-2b"));
    }
    #[test]
    fn test_is_declared_variant_accepts_decorated_base_model_names() {
        let candidate = model(json!({
            "id": "unsloth/NVIDIA-Nemotron-3-Super-120B-A12B-GGUF",
            "cardData": {"base_model": ["nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-BF16"]},
            "tags": ["base_model:quantized:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-BF16"]
        }));
        assert!(candidate.is_declared_variant_of("nvidia/nemotron-3-super-120b-a12b"));
        assert!(!candidate.is_declared_variant_of("other/nemotron-3-super-120b-a12b"));
        assert!(!candidate.is_declared_variant_of("nvidia/different-model"));
    }
    #[test]
    fn test_is_declared_variant_accepts_meta_publisher_alias_and_catalog_suffixes() {
        let candidate = model(json!({
            "id": "unsloth/Llama-4-Maverick-17B-128E-Instruct-GGUF",
            "tags": ["base_model:quantized:meta-llama/Llama-4-Maverick-17B-128E-Instruct"]
        }));
        assert!(candidate.is_declared_variant_of("meta/llama-4-maverick-17b-128e-instruct"));
        assert!(candidate.is_declared_variant_of("meta/llama-4-maverick-17b-128e-instruct-fp8"));
        assert!(candidate.is_declared_variant_of("meta/llama-4-maverick-17b-128e-instruct-maas"));
        assert!(!candidate.is_declared_variant_of("other/llama-4-maverick-17b-128e-instruct"));
    }
    #[test]
    fn test_is_declared_variant_accepts_nvidia_version_separator_aliases() {
        let ultra = model(json!({
            "id": "bartowski/nvidia_Llama-3_1-Nemotron-Ultra-253B-v1-GGUF",
            "tags": ["base_model:quantized:nvidia/Llama-3_1-Nemotron-Ultra-253B-v1"]
        }));
        let super_model = model(json!({
            "id": "bartowski/nvidia_Llama-3_3-Nemotron-Super-49B-v1_5-GGUF",
            "tags": ["base_model:quantized:nvidia/Llama-3_3-Nemotron-Super-49B-v1_5"]
        }));
        assert!(ultra.is_declared_variant_of("nvidia/llama-3.1-nemotron-ultra-253b"));
        assert!(super_model.is_declared_variant_of("nvidia/llama-3.3-nemotron-super-49b-v1.5"));
        assert!(!super_model.is_declared_variant_of("nvidia/llama-nemotron-rerank-vl-1b-v2"));
    }
    #[test]
    fn test_gguf_candidate_includes_sorted_unique_quantizations() {
        let candidate = Candidate::from(model(json!({
            "id": "community/quantized",
            "downloads": 42,
            "likes": 7,
            "siblings": [
                {"rfilename": "model-Q5_K_M.gguf"},
                {"rfilename": "model-Q4_K_M.gguf"},
                {"rfilename": "model-Q4_K_M-00001-of-00002.gguf"}
            ]
        })));
        assert_eq!(candidate.downloads, 42);
        assert_eq!(candidate.likes, Some(7));
        assert_eq!(candidate.quantizations, vec!["Q4_K_M", "Q5_K_M"]);
        assert_eq!(candidate.to_string(), "community/quantized");
    }
    #[test]
    fn test_gguf_candidate_excludes_unrecognized_quantizations() {
        let candidate = Candidate::from(model(json!({
            "id": "community/unsupported",
            "siblings": [{"rfilename": "model-tq1_0.gguf"}]
        })));
        assert!(candidate.quantizations.is_empty());
    }
    #[test]
    fn test_fallback_candidates_apply_inclusive_minimum_download_count() {
        let candidate = |id: &str, downloads: Option<u64>| {
            model(json!({
                "id": id,
                "downloads": downloads,
                "tags": ["base_model:quantized:acme/base"],
                "siblings": [{"rfilename": "model-Q4_K_M.gguf"}]
            }))
        };
        let options = SearchOptions::init().identifier("acme/base").minimum_download_count(100).build();
        let candidates = Candidates::fallback(
            vec![
                candidate("acme/above-GGUF", Some(101)),
                candidate("acme/boundary-GGUF", Some(100)),
                candidate("acme/below-GGUF", Some(99)),
                candidate("acme/missing-GGUF", None),
            ],
            &options,
        );
        assert_eq!(
            candidates.iter().map(|candidate| candidate.id.as_str()).collect::<Vec<_>>(),
            vec!["acme/above-GGUF", "acme/boundary-GGUF"]
        );
    }
    #[test]
    fn test_weights_to_source_uses_hugging_face_repository_identifier() {
        let source = Weights(vec![Weight {
            label: "Hugging Face".to_string(),
            url: "https://huggingface.co/openai/gpt-oss-20b".to_string(),
            is_open: Some(true),
            quantization: None,
            size: None,
        }])
        .to_source(Some("GPT OSS 20B".to_string()))
        .unwrap();
        assert_eq!(source.identifier(), "openai/gpt-oss-20b");
        assert_eq!(source.name(), "GPT OSS 20B");
    }
    #[test]
    fn test_weights_to_source_keeps_direct_hugging_face_file_url() {
        let url = "https://huggingface.co/openai/gpt-oss-20b/resolve/main/model.gguf".to_string();
        let source = Weights(vec![Weight {
            label: "GGUF".to_string(),
            url: url.to_string(),
            is_open: Some(true),
            quantization: None,
            size: None,
        }])
        .to_source(None)
        .unwrap();
        assert_eq!(source.identifier(), url);
        assert_eq!(source.name(), "GGUF");
    }
    #[test]
    fn test_repository_tree_reports_api_error_message() {
        let options = super::Options::init().identifier("missing/model").revision("main").build();
        let error = HuggingFaceRepositoryFiles::parse(r#"{"error":"Repository not found"}"#, &options).unwrap_err();
        assert_eq!(
            error.to_string(),
            "Hugging Face API rejected repository 'missing/model' — Repository not found"
        );
    }
    #[test]
    fn test_repository_tree_parse_error_identifies_repository() {
        let options = super::Options::init().identifier("broken/model").revision("main").build();
        let error = HuggingFaceRepositoryFiles::parse(r#"{"unexpected":true}"#, &options).unwrap_err();
        assert!(error
            .to_string()
            .starts_with("Failed to parse Hugging Face repository file list for 'broken/model'"));
    }
    #[test]
    fn test_repository_resolution_identifies_direct_and_fallback_values() {
        let direct = RepositoryResolution::direct("acme/model", 1);
        assert!(!direct.is_fallback());
        assert_eq!(direct.requested(), "acme/model");
        assert_eq!(direct.resolved(), "acme/model");
        assert_eq!(*direct.value(), 1);
        let fallback = RepositoryResolution::new("acme/model", "community/model-GGUF", 2);
        assert!(fallback.is_fallback());
        assert_eq!(fallback.requested(), "acme/model");
        assert_eq!(fallback.resolved(), "community/model-GGUF");
        assert_eq!(fallback.into_parts(), ("acme/model".to_string(), "community/model-GGUF".to_string(), 2));
    }
    #[test]
    fn test_repository_resolution_transforms_values_without_losing_identity() {
        let mapped = RepositoryResolution::new("acme/model", "community/model-GGUF", 2).map(|value| value.to_string());
        assert_eq!(mapped.requested(), "acme/model");
        assert_eq!(mapped.resolved(), "community/model-GGUF");
        assert_eq!(mapped.value(), "2");
        let mapped = RepositoryResolution::direct("acme/model", 2).try_map(|value| if value > 0 { Ok(value * 2) } else { Err("invalid") });
        assert_eq!(
            mapped.map(RepositoryResolution::into_parts),
            Ok(("acme/model".to_string(), "acme/model".to_string(), 4))
        );
    }
    #[test]
    fn test_downloaded_into_resolution_uses_one_identifier() {
        let downloaded = Downloaded::init().identifier("acme/model").revision("main").files(Vec::new()).build();
        let resolution = downloaded.into_resolution("acme/model");
        assert!(!resolution.is_fallback());
        assert_eq!(resolution.requested(), "acme/model");
        assert_eq!(resolution.resolved(), "acme/model");
    }
    #[test]
    fn test_model_details_from_repository_resolution_preserves_fallback() {
        let resolution = RepositoryResolution::new("acme/model", "community/model-GGUF", ModelDetails::default());
        let details = ModelDetails::from(resolution);
        assert_eq!(details.id.as_deref(), Some("community/model-GGUF"));
        assert_eq!(details.fallback.as_deref(), Some("acme/model"));
    }
    #[test]
    fn test_target_path_from_sidecar() {
        assert_eq!(target_path_from_sidecar("model.gguf.sha256"), Some("model.gguf".to_string()));
        assert_eq!(
            target_path_from_sidecar("nested/model.gguf.sha256sum"),
            Some("nested/model.gguf".to_string())
        );
    }
    #[test]
    fn test_extract_sha256() {
        let digest = "ABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCD";
        assert_eq!(
            extract_sha256(format!("{digest}  model.gguf").as_str()).as_deref(),
            Some("abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd"),
        );
        let digest = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        assert_eq!(
            extract_sha256(format!("SHA256 ({digest}) = {digest} model.gguf").as_str()).as_deref(),
            Some(digest),
        );
        assert_eq!(extract_sha256("not-a-sha model.gguf"), None);
        assert_eq!(extract_sha256("0123456789abcdef"), None);
    }
}
