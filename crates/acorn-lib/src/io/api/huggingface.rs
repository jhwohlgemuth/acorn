//! Hugging Face Hub API helpers.
use super::{Endpoint, IntoHeaders, Params, RemoteResource, TreeEntry};
use crate::io::api::{Param, RepositoryFileMetadata};
use crate::io::config::FilterSet;
use crate::io::download::{DownloadItem, DownloadItems};
use crate::io::http::HttpResponseError;
use crate::io::sync::Shard;
use crate::io::{first_env_var, http, ApiResult, Source};
use crate::param;
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
use futures::future::join_all;
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
#[derive(Deserialize)]
#[serde(untagged)]
enum BaseModelReference {
    Identifier(String),
    Metadata { id: String },
}
#[derive(Deserialize)]
#[serde(untagged)]
enum BaseModels {
    References(Vec<BaseModelReference>),
    Relationship {
        #[serde(default)]
        models: Vec<BaseModelReference>,
    },
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Hugging Face model metadata used for direct and fallback repository resolution.
pub struct ModelInfo {
    /// Repository identifier in `owner/name` form.
    pub id: String,
    /// Models declared as ancestors of this repository.
    #[serde(default, deserialize_with = "deserialize_base_models")]
    pub base_models: Option<Vec<String>>,
    /// Parsed model-card metadata.
    #[serde(default)]
    pub card_data: Option<Value>,
    /// Downloads reported by Hugging Face.
    #[serde(default)]
    pub downloads: Option<u64>,
    /// Likes reported by Hugging Face.
    #[serde(default)]
    pub likes: Option<u64>,
    /// Files published by the repository.
    #[serde(default)]
    pub siblings: Option<Vec<ModelSibling>>,
    /// Model and relationship tags computed by Hugging Face.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}
/// File reference embedded in Hugging Face model metadata.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModelSibling {
    /// Repository-relative filename.
    pub rfilename: String,
}
/// Options for HuggingFace API requests
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct Options {
    /// Authentication token
    pub token: Option<String>,
    /// Model registry domain (default: "huggingface.co")
    #[builder(default = DEFAULT_HUGGINGFACE_DOMAIN.to_string())]
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
/// A repository-backed value together with its requested and resolved identifiers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryResolution<T> {
    requested: String,
    resolved: String,
    value: T,
}
/// Options for Hugging Face model fallback searches.
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct SearchOptions {
    /// Base model identifier used to find related GGUF repositories.
    pub identifier: String,
    /// Model name used for the Hugging Face text search.
    pub basename: String,
    /// Model registry domain.
    #[builder(default = DEFAULT_HUGGINGFACE_DOMAIN.to_string())]
    pub domain: String,
    /// Authentication token.
    pub token: Option<String>,
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
    /// Custom API parameters to include in the search request.
    #[builder(default = vec![])]
    pub custom_params: Vec<Param>,
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
impl core::error::Error for HuggingFaceError {}
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
impl From<TreeEntry> for HuggingFaceRepositoryFile {
    fn from(entry: TreeEntry) -> Self {
        HuggingFaceRepositoryFile {
            path: entry.path,
            size: entry.size,
        }
    }
}
impl RepositoryFileMetadata for HuggingFaceRepositoryFile {
    fn path(&self) -> &str {
        &self.path
    }
    fn size(&self) -> Option<u64> {
        self.size
    }
}
impl Deref for HuggingFaceRepositoryFiles {
    type Target = [HuggingFaceRepositoryFile];
    fn deref(&self) -> &Self::Target {
        &self.files
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
                                DownloadItem {
                                    path,
                                    sha,
                                    size,
                                    url,
                                    headers: None,
                                }
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
                let candidates = match preferred.is_empty() {
                    | false => preferred,
                    | true => {
                        let repository_models = files.repository_model_files();
                        match repository_models.is_empty() {
                            | true => files.clone(),
                            | false => repository_models,
                        }
                    }
                };
                match candidates.len() {
                    | 1 => Ok(candidates),
                    | _ => candidates.complete_shard_set().ok_or_else(|| {
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
    fn repository_model_files(&self) -> Self {
        let repository_name = self.identifier.rsplit('/').next().unwrap_or_default().to_ascii_lowercase();
        let model_name = repository_name.strip_suffix("-gguf").unwrap_or(&repository_name);
        let prefix = format!("{model_name}-");
        let files = self
            .iter()
            .filter(|file| {
                file.path
                    .rsplit('/')
                    .next()
                    .is_some_and(|filename| filename.to_ascii_lowercase().starts_with(&prefix))
            })
            .cloned()
            .collect();
        Self::new(&self.identifier, &self.revision, files)
    }
    fn candidate_report(&self) -> String {
        match self.len() {
            | 0 => "no GGUF files".to_string(),
            | 1 | 2 => self.iter().map(|file| file.path.as_str()).collect::<Vec<_>>().join(", "),
            | count => format!("{count} GGUF files"),
        }
    }
    /// Resolve the configured repository or a selected GGUF fallback repository.
    pub async fn resolve(options: &Options) -> ApiResult<RepositoryResolution<Self>> {
        let Options {
            identifier,
            no_fallback,
            search_limit,
            minimum_download_count,
            interactive,
            ..
        } = options;
        match identifier.as_deref() {
            | Some(identifier) => match repository_tree(options).await {
                | Ok(repository) if repository.files.iter().any(|file| Quantization::from_gguf_filename(&file.path).is_some()) => {
                    Ok(RepositoryResolution::new(identifier, identifier, repository))
                }
                | Ok(_) if *no_fallback => Err(eyre!("No GGUF model files found for '{identifier}'")),
                | Err(why) if *no_fallback => Err(why),
                | Ok(_) | Err(_) => {
                    let basename = ModelSelector::new(identifier)
                        .map(|selector| selector.fallback_search_name())
                        .unwrap_or_default();
                    let search_options = SearchOptions::init()
                        .identifier(identifier)
                        .basename(basename)
                        .domain(&options.domain)
                        .maybe_token(options.token.clone())
                        .limit(*search_limit)
                        .minimum_download_count(*minimum_download_count)
                        .interactive(*interactive)
                        .custom_params(options.custom_params.clone())
                        .build();
                    match search(&search_options).await {
                        | Ok(candidates) => match candidates.select(options) {
                            | Ok(resolved) => {
                                let fallback_options = Options {
                                    identifier: Some(resolved.clone()),
                                    revision: DEFAULT_HUGGINGFACE_MODEL_REVISION.to_string(),
                                    ..options.clone()
                                };
                                match repository_tree(&fallback_options).await {
                                    | Ok(repository) => Ok(RepositoryResolution::new(identifier, resolved, repository)),
                                    | Err(why) => Err(why),
                                }
                            }
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
impl IntoIterator for HuggingFaceRepositoryFiles {
    type Item = HuggingFaceRepositoryFile;
    type IntoIter = alloc::vec::IntoIter<HuggingFaceRepositoryFile>;
    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
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
impl ModelInfo {
    /// Fetch model metadata with base models, files, card data, and tags expanded.
    pub async fn fetch(options: &Options) -> ApiResult<Self> {
        match options.identifier.as_deref() {
            | Some(identifier) => match Endpoint::from_template("huggingface::api").map(|endpoint| endpoint.with_domain(&options.domain)) {
                | Ok(endpoint) => {
                    let token = request_token(options.token.as_deref());
                    let params: Vec<Param> = vec![
                        param!(TemplateValue, "identifier", identifier),
                        param!(KeyValuePair, "expand", "baseModels"),
                        param!(KeyValuePair, "expand", "cardData"),
                        param!(KeyValuePair, "expand", "siblings"),
                        param!(KeyValuePair, "expand", "tags"),
                    ]
                    .into_iter()
                    .chain((!token.is_empty()).then(|| {
                        let authorization = format!("Bearer {token}");
                        param!(Header, "Authorization", authorization.as_str())
                    }))
                    .chain(options.custom_params.iter().cloned())
                    .collect();
                    let response = endpoint.invoke_response("model", Some(params)).await;
                    endpoint
                        .handle_response::<Self>(response)
                        .map(|response| response.value)
                        .map_err(|why| why.wrap_err(format!("Failed to read Hugging Face metadata for '{identifier}'")))
                }
                | Err(why) => Err(why),
            },
            | None => Err(eyre!(HuggingFaceError::InvalidBaseModelIdentifier { identifier: "".into() })),
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
                | true => Source::from(&Repository::HuggingFace {
                    location: Box::new(location),
                }),
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
fn deserialize_base_models<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<BaseModels>::deserialize(deserializer).map(|value| {
        value.map(|base_models| {
            let references = match base_models {
                | BaseModels::References(references) | BaseModels::Relationship { models: references } => references,
            };
            references
                .into_iter()
                .map(|reference| match reference {
                    | BaseModelReference::Identifier(identifier) | BaseModelReference::Metadata { id: identifier } => identifier,
                })
                .collect()
        })
    })
}
pub(super) fn extract_sha256(content: &str) -> Option<String> {
    content
        .split_whitespace()
        .find(|token| token.len() == 64 && token.chars().all(|character| character.is_ascii_hexdigit()))
        .map(|value| value.to_ascii_lowercase())
}
/// Return whether a non-empty Hugging Face token is configured
pub fn has_auth_token() -> bool {
    first_env_var(&HUGGINGFACE_TOKEN_VARIABLE_NAMES).is_some()
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
/// Return whether a Hugging Face error means a model is unavailable to the caller
pub fn model_is_unavailable(error: &Report) -> bool {
    error
        .downcast_ref::<HttpResponseError>()
        .is_some_and(|source| matches!(source.status_code(), 401 | 403 | 404))
}
pub(super) fn next_page(headers: &HeaderMap, endpoint: &Endpoint) -> ApiResult<Option<String>> {
    match headers
        .get("link")
        .and_then(|value| value.to_str().ok())
        .and_then(|links| links.split(',').find(|link| link.contains("rel=\"next\"")))
    {
        | Some(link) => match link.split_once('<').and_then(|(_, value)| value.split_once('>').map(|(url, _)| url)) {
            | Some(url) => {
                let expected = Endpoint::from(Location::from(endpoint.base().as_str()));
                let actual = Endpoint::from(Location::from(url));
                match actual.domain.eq_ignore_ascii_case(&expected.domain) && actual.scheme == expected.scheme && actual.port == expected.port {
                    | true => Ok(Some(url.to_string())),
                    | false => Err(eyre!(HuggingFaceError::ModelSearchFailed {
                        reason: "pagination URL did not match the configured Hugging Face origin".into()
                    })),
                }
            }
            | None => Err(eyre!(HuggingFaceError::ModelSearchFailed {
                reason: "pagination Link header did not contain a valid target".into()
            })),
        },
        | None => Ok(None),
    }
}
/// Validate and parse a Hugging Face model identifier into (owner, name)
pub fn parse_identifier(identifier: &str) -> ApiResult<(&str, &str)> {
    match identifier.split_once('/') {
        | Some((owner, name)) if !owner.is_empty() && !name.is_empty() && !name.contains('/') => Ok((owner, name)),
        | _ => Err(eyre!("Invalid Hugging Face model identifier — {identifier}")),
    }
}
/// List files in a Hugging Face model repository at `revision`
pub async fn repository_tree(options: &Options) -> ApiResult<HuggingFaceRepositoryFiles> {
    let template = "huggingface::api";
    let action = "tree";
    let token = request_token(options.token.as_deref());
    let params: Vec<Param> = options
        .identifier
        .as_deref()
        .map(|identifier| param!(TemplateValue, "identifier", identifier))
        .into_iter()
        .chain([
            param!(TemplateValue, "revision", options.revision.as_str()),
            param!(KeyValuePair, "recursive", "1"),
        ])
        .chain((!token.is_empty()).then(|| {
            let authorization = format!("Bearer {token}");
            param!(Header, "Authorization", authorization.as_str())
        }))
        .chain(options.custom_params.iter().cloned())
        .collect();
    match Endpoint::from_template(template).map(|endpoint| endpoint.with_domain(&options.domain)) {
        | Ok(endpoint) => {
            let response = endpoint.invoke_response(action, Some(params)).await;
            match (options.identifier.as_deref(), endpoint.handle_response::<Vec<TreeEntry>>(response)) {
                | (Some(identifier), Ok(response)) => {
                    let files = response
                        .value
                        .into_iter()
                        .filter(TreeEntry::is_file)
                        .map(HuggingFaceRepositoryFile::from)
                        .collect();
                    Ok(HuggingFaceRepositoryFiles::new(identifier, &options.revision, files))
                }
                | (None, _) => Err(eyre!("Missing Hugging Face repository identifier")),
                | (_, Err(why)) => Err(why.wrap_err("Failed to list Hugging Face model files")),
            }
        }
        | Err(why) => Err(eyre!("Failed to configure Hugging Face API — {why}")),
    }
}
fn request_token(configured: Option<&str>) -> String {
    configured
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| first_env_var(&HUGGINGFACE_TOKEN_VARIABLE_NAMES))
        .unwrap_or_default()
}
/// Find repositories matching the search options, sorted by downloads.
pub async fn search(options: &SearchOptions) -> ApiResult<Candidates> {
    let template = "huggingface::api";
    let action = "models";
    match options.basename.trim() {
        | "" => Err(eyre!(HuggingFaceError::InvalidBaseModelIdentifier {
            identifier: options.identifier.clone().into()
        })),
        | basename => match Endpoint::from_template(template).map(|endpoint| endpoint.with_domain(&options.domain)) {
            | Ok(endpoint) => {
                let token = request_token(options.token.as_deref());
                let limit = options.limit;
                let limit_value = (limit < 1000).then(|| limit.to_string());
                let params: Vec<Param> = vec![
                    param!(KeyValuePair, "search", basename),
                    param!(KeyValuePair, "filter", options.term.as_str()),
                    param!(KeyValuePair, "sort", "downloads"),
                    param!(KeyValuePair, "full", "true"),
                    param!(KeyValuePair, "cardData", "true"),
                ]
                .into_iter()
                .chain(limit_value.as_deref().map(|value| param!(KeyValuePair, "limit", value)))
                .chain((!token.is_empty()).then(|| {
                    let authorization = format!("Bearer {token}");
                    param!(Header, "Authorization", authorization.as_str())
                }))
                .chain(options.custom_params.iter().cloned())
                .collect();
                let headers = params.clone().into_headers();
                let mut models = Vec::new();
                let mut visited = BTreeSet::new();
                let mut response = endpoint.invoke_response(action, Some(params)).await;
                let pagination = loop {
                    match endpoint.handle_response::<Vec<ModelInfo>>(response) {
                        | Ok(page) => {
                            let remaining = limit.saturating_sub(models.len());
                            let page_is_empty = page.value.is_empty();
                            models.extend(page.value.into_iter().take(remaining));
                            match next_page(&page.headers, &endpoint) {
                                | Ok(Some(next)) if models.len() < limit && !page_is_empty && visited.insert(next.clone()) => {
                                    response = http::get(next).headers(headers.clone()).send().await;
                                }
                                | Ok(Some(_)) if models.len() < limit => {
                                    break Err(eyre!(HuggingFaceError::ModelSearchFailed {
                                        reason: "pagination repeated a URL or returned an empty page".into()
                                    }));
                                }
                                | Ok(_) => break Ok(()),
                                | Err(why) => break Err(why),
                            }
                        }
                        | Err(why) => break Err(why.wrap_err("Failed to search Hugging Face models")),
                    }
                };
                match pagination {
                    | Ok(()) => {
                        let candidates = Candidates::fallback(models, options);
                        match candidates.is_empty() {
                            | true => Err(eyre!(HuggingFaceError::NoGgufQuantizationRepository {
                                identifier: options.identifier.clone().into()
                            })),
                            | false => Ok(candidates),
                        }
                    }
                    | Err(why) => Err(why),
                }
            }
            | Err(why) => Err(why),
        },
    }
}
fn select_first(candidates: Candidates, _options: &Options) -> ApiResult<String> {
    candidates
        .into_iter()
        .next()
        .map(|candidate| candidate.to_string())
        .ok_or_else(|| eyre!("GGUF search returned no candidates"))
}
pub(super) fn target_path_from_sidecar(path: &str) -> Option<String> {
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
