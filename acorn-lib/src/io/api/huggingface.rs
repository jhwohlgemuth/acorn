//! Hugging Face Hub API helpers.
use super::{Endpoint, IntoHeaders, Params, RemoteResource, ResponseContent, TreeEntry};
use crate::io::api::{Param, RepositoryFileMetadata};
use crate::io::config::FilterSet;
use crate::io::download::{DownloadItem, DownloadItems};
use crate::io::{first_env_var, http, ApiResult, Source};
use crate::prelude::{HashMap, Path};
use crate::schema::agent::{Quantization, Weight, Weights};
use crate::util::constants::app::{
    DEFAULT_HUGGINGFACE_DOMAIN, DEFAULT_HUGGINGFACE_MODEL_REVISION, DEFAULT_HUGGINGFACE_SEARCH_LIMIT, DEFAULT_HUGGINGFACE_SEARCH_TERM,
};
use crate::util::constants::env::HUGGINGFACE_TOKEN_VARIABLE_NAMES;
use crate::util::{glob_matches, regex_to_glob, Label};
use crate::{Location, Repository};
use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use async_trait::async_trait;
use axum::http::HeaderMap;
use bon::Builder;
use color_eyre::eyre::{eyre, Report};
use core::{cmp::Reverse, fmt};
use futures::{future::join_all, TryStreamExt};
use hf_hub::{repository::ModelInfo, HFClient, HFError};
use serde::Serialize;
use serde_json::Value;
use tracing::{info, warn};

/// Candidate repositories returned by model search.
pub type Candidates = Vec<Candidate>;
/// Function used to select from multiple interactive candidates.
pub type CandidateSelector = fn(Candidates, &Options) -> ApiResult<String>;
/// Repository files returned by the Hugging Face repository tree API
pub type HuggingFaceRepositoryFiles = Vec<HuggingFaceRepositoryFile>;
/// Select a repository from model search candidates.
pub trait CandidateSelection {
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
    /// Determine whether a Hugging Face model declares a given model as its base model
    fn is_declared_derivative_of(&self, identifier: &str) -> bool;
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
impl RepositoryFileMetadata for HuggingFaceRepositoryFile {
    fn path(&self) -> &str {
        &self.path
    }
    fn size(&self) -> Option<u64> {
        self.size
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
                        "=> {} Multiple GGUF repositories found; using most downloaded: '{candidate}' (non-interactive mode)",
                        Label::CAUTION
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
            warn!(
                "=> {} Interactive GGUF picker is unavailable; using most downloaded: '{candidate}'",
                Label::CAUTION
            );
        }
        Ok(candidate.to_string())
    }
}
impl Downloaded {
    /// Merge downloaded GGUF quantizations into existing model weights.
    pub fn merge_weights(&self, existing: Weights) -> Weights {
        let downloaded = self
            .files
            .iter()
            .filter_map(|path| {
                Quantization::from_gguf_filename(path).map(|quantization| Weight {
                    label: quantization.to_string(),
                    url: format!(
                        "https://{DEFAULT_HUGGINGFACE_DOMAIN}/{}/resolve/{}/{}",
                        self.identifier, self.revision, path
                    ),
                    is_open: None,
                    quantization: Some(quantization),
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
                self.clone(),
                &filter_vec,
                &ignore_vec,
                |file: &HuggingFaceRepositoryFile| file.path.clone(),
                |_| true,
            ),
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
    fn select(&self, policy: &FileSelectionPolicy<'_>) -> ApiResult<Vec<HuggingFaceRepositoryFile>> {
        let gguf_files: Vec<&HuggingFaceRepositoryFile> = self.iter().filter(|file| file.path.to_ascii_lowercase().ends_with(".gguf")).collect();
        match gguf_files.len() {
            | 0 => Err(eyre!(HuggingFaceError::NoGgufModelFiles)),
            | 1 => Ok(gguf_files.into_iter().cloned().collect()),
            | _ => {
                let preferred: Vec<&HuggingFaceRepositoryFile> = gguf_files
                    .into_iter()
                    .filter(|file| file.path.to_ascii_uppercase().contains(policy.preferred_marker))
                    .collect();
                match preferred.len() {
                    | 1 => Ok(preferred.into_iter().cloned().collect()),
                    | _ => Err(eyre!(
                        "Multiple model file candidates found; use --filter to choose one: {}",
                        self.iter().map(|file| file.path.clone()).collect::<Vec<_>>().join(", ")
                    )),
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
                Some(filtered)
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
}
impl From<&Source> for Weights {
    fn from(source: &Source) -> Self {
        match source {
            | Source::Remote { identifier, .. } => Self(vec![Weight {
                label: "Hugging Face".to_string(),
                url: format!("https://{DEFAULT_HUGGINGFACE_DOMAIN}/{identifier}"),
                is_open: None,
                quantization: None,
            }]),
            | Source::Local { path, .. } => Self(vec![Weight {
                label: "Local".to_string(),
                url: path.display().to_string(),
                is_open: Some(true),
                quantization: None,
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
    let params = Params::new()
        .with_auth(first_env_var(&HUGGINGFACE_TOKEN_VARIABLE_NAMES).unwrap_or_default().as_str(), None)
        .with_template("identifier", Some(identifier))
        .with_template("revision", Some(revision))
        .with_keyvalue("recursive", Some("1"))
        .build();
    match Endpoint::from_template(template) {
        | Ok(endpoint) => match endpoint.invoke(action, Some(params)).await {
            | Ok(ResponseContent::Json(content)) => serde_json::from_str::<Vec<TreeEntry>>(&content)
                .map(|entries| {
                    entries
                        .into_iter()
                        .filter(TreeEntry::is_file)
                        .map(HuggingFaceRepositoryFile::from)
                        .collect()
                })
                .map_err(|why| eyre!("Failed to parse Hugging Face repository file list — {why}")),
            | Ok(_) => Err(eyre!("Failed to list Hugging Face model files — response was not JSON")),
            | Err(why) => Err(eyre!("Failed to list Hugging Face model files — {why}")),
        },
        | Err(why) => Err(eyre!("Failed to configure Hugging Face API — {why}")),
    }
}
/// Find repositories matching `term` that declare `identifier` as their source, sorted by downloads.
pub async fn search(identifier: &str, term: Option<&str>, limit: Option<usize>) -> ApiResult<Candidates> {
    let term = term.unwrap_or(DEFAULT_HUGGINGFACE_SEARCH_TERM);
    let limit = limit.unwrap_or(DEFAULT_HUGGINGFACE_SEARCH_LIMIT);
    let basename = identifier.rsplit('/').next().filter(|value| !value.is_empty());
    match (basename, HFClient::new()) {
        | (Some(basename), Ok(client)) => {
            let response = client
                .list_models()
                .search(basename)
                .filter(term)
                .sort("downloads")
                .full(true)
                .card_data(true)
                .limit(limit)
                .send();
            match response {
                | Ok(stream) => match stream.try_collect::<Vec<ModelInfo>>().await {
                    | Ok(models) => {
                        let mut candidates = models
                            .into_iter()
                            .filter(|model| model.id.eq_ignore_ascii_case(identifier) || model.is_declared_derivative_of(identifier))
                            .map(Candidate::from)
                            .collect::<Vec<_>>();
                        candidates.sort_by_key(|candidate| Reverse(candidate.downloads));
                        match candidates.is_empty() {
                            | true => Err(eyre!(HuggingFaceError::NoGgufQuantizationRepository {
                                identifier: identifier.into()
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
            identifier: identifier.into()
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{extract_sha256, target_path_from_sidecar, Candidate, ModelInfo, ModelInfoExtension, Value};
    use crate::schema::agent::{Weight, Weights};
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
    fn test_weights_to_source_uses_hugging_face_repository_identifier() {
        let source = Weights(vec![Weight {
            label: "Hugging Face".to_string(),
            url: "https://huggingface.co/openai/gpt-oss-20b".to_string(),
            is_open: Some(true),
            quantization: None,
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
        }])
        .to_source(None)
        .unwrap();
        assert_eq!(source.identifier(), url);
        assert_eq!(source.name(), "GGUF");
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
