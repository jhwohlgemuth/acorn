//! Model download planning, resolution, and metadata filtering.
use super::{download_remote_model, materialize_local_model};
use crate::cli::CommandOptions;
use acorn::io::api::harbor::{self, Artifact as HarborArtifact, Options as HarborOptions};
use acorn::io::api::huggingface::{
    self, CandidateSelection, Downloaded, HuggingFaceError, HuggingFaceRepository, ModelInfoExtension, RepositoryResolution,
};
use acorn::io::api::models_dev;
use acorn::io::config::{AuthenticationRequirement, FilterSet, ModelEntry, ModelEntryOptions, RegistryKind, RegistryProfile};
use acorn::io::database::schema::ModelRow;
use acorn::io::database::Row;
use acorn::io::oci::{OciFile, OciReference, OciReferenceValue, OciResolution, OciTransport, OciTransportOptions, OrasTransport};
use acorn::io::{ApiResult, Source, SourceAction};
#[cfg(feature = "tui")]
use acorn::prelude::io;
use acorn::prelude::{create_dir_all, HashMap, HashSet, Path, PathBuf, String, Vec};
use acorn::schema::agent::{
    ModelDetails, ModelSelector, ModelSelectors, OciModelDetails, OciModelExtraction, OciModelLayer, Quantization, Weight, WeightGroups, Weights,
};
use acorn::schema::hardware::memory::Memory;
use acorn::schema::OneOrMany;
use acorn::util::constants::app::{DEFAULT_HUGGINGFACE_MODEL_REVISION, OCI_PREFIX};
use acorn::util::{merge, Label, MimeType, StringConversion};
use acorn::{Location, Repository};
use bon::Builder;
use color_eyre::eyre::{self, eyre};
use core::fmt;
use futures::{future::join_all, stream, StreamExt};
#[cfg(feature = "tui")]
use is_terminal::IsTerminal;
use itertools::Itertools;
use owo_colors::OwoColorize;
use tracing::{error, info, warn};

#[derive(Clone, Debug)]
pub(crate) enum ModelDownloadError {
    UnknownModel(String),
    MultipleMatches(String),
    MissingAuth(String),
    ModelList(String),
    Configuration(String),
    RegistryConfiguration(String),
    Whitelist(String),
    NoModelsSpecified,
    NoWhitelistMatches,
    NoFilterMatches,
    OfflineRemoteModels(String),
    NoModelsFound,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct DownloadOptions {
    pub(crate) offline: bool,
    pub(crate) limit: usize,
    pub(crate) minimum_download_count: u64,
    pub(crate) interactive: bool,
}
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init)]
pub(crate) struct ModelDownloadPlan {
    pub(crate) selector: Source,
    #[builder(default = String::from(DEFAULT_HUGGINGFACE_MODEL_REVISION))]
    pub(crate) revision: String,
    #[builder(default)]
    pub(crate) auth: AuthenticationRequirement,
    #[builder(default = Vec::new())]
    pub(crate) filter: Vec<String>,
    #[builder(default = Vec::new())]
    pub(crate) ignore: Vec<String>,
    #[builder(default = Vec::new())]
    pub(crate) quantization: Vec<Quantization>,
    pub(crate) gpu_memory: Option<Memory>,
    #[builder(default = Vec::new())]
    pub(crate) required_paths: Vec<String>,
    pub(crate) action: Option<SourceAction>,
    pub(crate) details: Option<ModelDetails>,
    #[builder(default)]
    pub(crate) oci_options: OciTransportOptions,
    pub(crate) registry_profile: Option<RegistryProfile>,
}
pub(crate) struct ModelDownloadPlans(pub(crate) Vec<ModelDownloadPlan>);
struct OciPlanResolution {
    reference: OciReference,
    resolution: OciResolution,
    harbor: Option<HarborArtifact>,
}
pub(crate) struct ResolvedModelDownload {
    plan: ModelDownloadPlan,
    resolution: RepositoryResolution<ModelDetails>,
    oci: Option<OciPlanResolution>,
}
impl fmt::Display for ModelDownloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::UnknownModel(value) => write!(formatter, "Unknown model selector: {value}"),
            | Self::MultipleMatches(value) => write!(formatter, "Multiple model matches found: {value}"),
            | Self::MissingAuth(value) => write!(formatter, "Model download requires authentication: {value}"),
            | Self::ModelList(value) | Self::Configuration(value) | Self::RegistryConfiguration(value) | Self::Whitelist(value) => {
                formatter.write_str(value)
            }
            | Self::NoModelsSpecified => write!(formatter, "No models specified — provide model IDs, --model-file, or --config"),
            | Self::NoWhitelistMatches => write!(formatter, "No models were found or matched the whitelist"),
            | Self::NoFilterMatches => write!(formatter, "No models matched --filter/--ignore"),
            | Self::OfflineRemoteModels(value) => write!(formatter, "Offline mode cannot download remote models: {value}"),
            | Self::NoModelsFound => write!(formatter, "No models found"),
        }
    }
}
impl core::error::Error for ModelDownloadError {}
impl ResolvedModelDownload {
    pub(super) fn plan(&self) -> &ModelDownloadPlan {
        &self.plan
    }
    pub(super) async fn execute(&self, options: CommandOptions) -> ApiResult<ModelDetails> {
        match &self.oci {
            | Some(resolved) => self.plan.download_from_oci(options, resolved),
            | None => self
                .plan
                .execute(options)
                .await
                .map(|identifier| self.plan.details.clone().unwrap_or_else(|| (&self.plan).into()).with_id(&identifier)),
        }
    }
}
impl fmt::Display for ModelDownloadPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.selector, formatter)
    }
}
impl ModelDownloadPlan {
    fn new(selector: Source) -> Self {
        let action = match &selector {
            | Source::Local { action, .. } => *action,
            | _ => None,
        };
        Self::init().selector(selector).maybe_action(action).build()
    }
    fn from_entry(entry: &ModelEntryOptions, oci_options: OciTransportOptions, profile: Option<RegistryProfile>) -> Self {
        let config_action = match (&entry.copy, &entry.symlink) {
            | (Some(true), _) => Some(SourceAction::Copy),
            | (_, Some(true)) => Some(SourceAction::Symlink),
            | _ => None,
        };
        Self::init()
            .selector(Source::from(&entry.source).with_name(entry.name.as_str()).with_action(config_action))
            .revision(entry.revision.clone().unwrap_or_else(|| DEFAULT_HUGGINGFACE_MODEL_REVISION.to_string()))
            .auth(entry.auth.clone().unwrap_or_default())
            .filter(entry.filter.clone().unwrap_or_default())
            .ignore(entry.ignore.clone().unwrap_or_default())
            .quantization(entry.quantization.clone().map(OneOrMany::into_vec).unwrap_or_default())
            .maybe_gpu_memory(entry.gpu_memory.clone())
            .maybe_action(config_action)
            .oci_options(oci_options)
            .maybe_registry_profile(profile)
            .build()
    }
    fn with_action(self, action: Option<SourceAction>) -> Self {
        match action {
            | Some(value) => Self {
                selector: self.selector.with_action(Some(value)),
                action: Some(value),
                ..self
            },
            | None => self,
        }
    }
    fn with_details(self, details: ModelDetails) -> Self {
        Self {
            details: Some(details),
            ..self
        }
    }
    pub(super) fn is_remote(&self) -> bool {
        self.selector.is_remote()
    }
    fn is_oci(&self) -> bool {
        self.selector.identifier().starts_with(OCI_PREFIX)
    }
    pub(crate) async fn matches(&self, whitelist: &HashSet<String>, offline: bool) -> ApiResult<bool> {
        let name = self.name();
        let rejected = || {
            warn!(
                "=> {}{} {}",
                Label::rejected(),
                self.selector.identifier().yellow(),
                "(did not match whitelist)".dimmed()
            );
            false
        };
        if whitelist.is_empty() || whitelist.contains(&name) || whitelist.contains(&self.selector.identifier()) {
            Ok(true)
        } else if offline || !self.is_remote() || Location::from(&self.selector.identifier()).is_http_or_https() || self.is_oci() {
            Ok(rejected())
        } else {
            let identifier = self.selector.identifier();
            match huggingface::parse_identifier(&identifier) {
                | Ok((owner, name)) => {
                    let info = huggingface::fetch_model_info(owner, name).await;
                    match info {
                        | Ok(model) => match whitelist.iter().any(|base| model.is_declared_derivative_of(base)) {
                            | true => Ok(true),
                            | false => Ok(rejected()),
                        },
                        | Err(why) if huggingface::model_is_unavailable(&why) => {
                            warn!(
                                "=> {} {} {}",
                                Label::not_found(),
                                identifier.yellow(),
                                "(unavailable on Hugging Face)".dimmed()
                            );
                            Ok(false)
                        }
                        | Err(why) => Err(why),
                    }
                }
                | Err(e) => Err(e),
            }
        }
    }
    pub(crate) fn name(&self) -> String {
        self.selector.name()
    }
    async fn execute(&self, options: CommandOptions) -> ApiResult<String> {
        if self.auth.is_required() && self.selector.is_remote() && !huggingface::has_auth_token() {
            Err(eyre!("{}", ModelDownloadError::MissingAuth(self.name())))
        } else {
            match options.selector.as_ref() {
                | Some(Source::Local { path, .. }) if !path.exists() => {
                    error!("=> {} Local model path does not exist: {}", Label::fail(), path.display());
                    Err(eyre!("Local model path does not exist: {}", path.display()))
                }
                | Some(Source::Local { .. }) => materialize_local_model(options.clone()).map(|_| {
                    if !options.quiet {
                        info!(
                            "=> {} Resolved local model '{}' → {}",
                            Label::pass(),
                            self.name().cyan(),
                            options
                                .output
                                .as_ref()
                                .map(|root| root.join(self.name()))
                                .map(|path| path.display().to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                                .dimmed()
                        );
                    }
                    self.name()
                }),
                | Some(Source::Remote { identifier, .. }) if Location::from(identifier).is_http_or_https() => download_remote_model(options).await,
                | Some(Source::Remote { .. }) => self.download_from_huggingface(options.clone()).await.map(|resolution| {
                    let identifier = resolution.resolved().to_string();
                    if !options.no_local_database {
                        if let Err(why) = self.persist_downloaded_weights(&resolution, options.database_path.clone()) {
                            warn!("=> {} Model '{identifier}' weight metadata was not persisted — {why}", Label::CAUTION);
                        }
                    }
                    if !options.quiet {
                        let name = self.name();
                        let destination = options
                            .output
                            .as_ref()
                            .map(|root| root.join(&identifier))
                            .map(|path| path.display().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        info!("=> {} Resolved model '{}' → {}", Label::pass(), name.cyan(), destination.dimmed());
                        if !options.no_local_database {
                            let row = ModelRow::init().maybe_model_id(Some(identifier.to_string())).build();
                            match row.select(options.database_path.clone(), |r| r.model_id.as_deref() == Some(identifier.as_str())) {
                                | Ok(Some(row)) => {
                                    let name = row.name.as_deref().unwrap_or("unknown");
                                    info!(
                                        "=> {} Validated model '{identifier}' against local database (name: {name})",
                                        Label::pass()
                                    );
                                }
                                | Ok(None) => info!("=> {} Model '{identifier}' not in local database — proceeding anyway", Label::CAUTION,),
                                | Err(why) => info!("=> {} Model '{identifier}' database lookup failed — {why}", Label::CAUTION,),
                            }
                        }
                    }
                    identifier
                }),
                | Some(Source::Unsupported(value)) => Err(eyre!("Unsupported model source — {value}")),
                | None => Err(eyre!("Model selector is not configured")),
            }
        }
    }
    pub(super) async fn resolve_oci(&self) -> ApiResult<(OciReference, OciResolution, Option<HarborArtifact>)> {
        let resolved = OciReference::parse(&self.selector.identifier()).and_then(|reference| {
            OrasTransport::new(self.oci_options.clone()).and_then(|transport| {
                transport
                    .resolve(&reference, &self.filter, &self.ignore, &self.quantization)
                    .map(|resolution| (reference, resolution))
            })
        });
        match resolved {
            | Ok((reference, resolution)) => self
                .harbor_metadata(&reference, &resolution)
                .await
                .map(|harbor| (reference, resolution, harbor)),
            | Err(why) => Err(why),
        }
    }
    async fn harbor_metadata(&self, reference: &OciReference, resolution: &OciResolution) -> ApiResult<Option<HarborArtifact>> {
        match &self.registry_profile {
            | Some(profile) if profile.kind == RegistryKind::Harbor => {
                let selected = match reference.reference() {
                    | OciReferenceValue::Tag(value) | OciReferenceValue::Digest(value) => value,
                };
                let artifact = match HarborOptions::from_repository(profile, reference.repository()).map(|options| options.with_reference(selected)) {
                    | Ok(options) => harbor::artifact(&options).await,
                    | Err(why) => Err(why),
                };
                match artifact {
                    | Ok(artifact) if artifact.digest == resolution.resolved_digest => Ok(Some(artifact)),
                    | Ok(_) => Err(eyre!("Harbor and OCI Distribution resolved different artifact digests")),
                    | Err(why) if !self.auth.is_required() => {
                        warn!(
                            "=> {} Harbor metadata is unavailable; using generic OCI resolution — {why}",
                            Label::CAUTION
                        );
                        Ok(None)
                    }
                    | Err(why) => Err(why),
                }
            }
            | _ => Ok(None),
        }
    }
    pub(super) fn oci_details(&self, reference: &OciReference, resolution: &OciResolution, harbor: Option<&HarborArtifact>) -> ModelDetails {
        let identifier = reference.with_digest(&resolution.resolved_digest);
        let weights = resolution
            .files
            .iter()
            .map(|OciFile { path, installed_size, .. }| Weight {
                label: path.clone(),
                url: format!("{identifier}#{path}"),
                is_open: Some(!self.auth.is_required()),
                quantization: Quantization::from_gguf_filename(path),
                size: *installed_size,
            })
            .collect();
        ModelDetails {
            id: Some(identifier),
            name: Some(self.name()),
            fallback: Some(resolution.requested_reference.clone()),
            open_weights: Some(!self.auth.is_required()),
            oci: Some(OciModelDetails {
                registry: resolution.registry.clone(),
                repository: resolution.repository.clone(),
                requested_reference: resolution.requested_reference.clone(),
                resolved_digest: resolution.resolved_digest.clone(),
                artifact_type: resolution.artifact_type.clone(),
                total_size: resolution.total_size,
                transfer_size: resolution.transfer_size,
                installed_size: resolution.installed_size,
                layers: resolution
                    .layers
                    .iter()
                    .map(|layer| OciModelLayer {
                        media_type: MimeType::OciLayer(layer.media_type.clone()),
                        digest: layer.digest.clone(),
                        transfer_size: layer.transfer_size,
                        role: layer.role,
                        path: layer.path.clone(),
                        extraction: OciModelExtraction::from(Into::<&str>::into(layer.extraction)),
                        inventory_deferred: layer.inventory_deferred,
                    })
                    .collect(),
                files: resolution.files.iter().map(|file| file.path.clone()).collect(),
                harbor_tags: harbor.map(|artifact| artifact.tags.iter().map(|tag| tag.name.clone()).collect()),
                harbor_labels: harbor.map(|artifact| artifact.labels.iter().map(|label| label.name.clone()).collect()),
                harbor_push_time: harbor.and_then(|artifact| artifact.push_time.clone()),
            }),
            weights: Some(Weights(weights)),
            ..Default::default()
        }
    }
    fn download_from_oci(&self, options: CommandOptions, resolved: &OciPlanResolution) -> ApiResult<ModelDetails> {
        match options.output {
            | Some(root) => {
                let destination = resolved.reference.destination(&root, &resolved.resolution.resolved_digest);
                OrasTransport::new(self.oci_options.clone())
                    .and_then(|transport| transport.pull(&resolved.reference, &resolved.resolution, &destination))
                    .map(|()| self.oci_details(&resolved.reference, &resolved.resolution, resolved.harbor.as_ref()))
            }
            | None => Err(eyre!("Model output directory is not configured")),
        }
    }
    async fn download_from_huggingface(&self, options: CommandOptions) -> ApiResult<RepositoryResolution<Downloaded>> {
        match (options.selector.clone(), options.output.clone()) {
            | (Some(Source::Remote { identifier, .. }), Some(root)) => {
                let destination = root.join(&identifier);
                match create_dir_all(&destination) {
                    | Ok(()) => match huggingface::repository_tree(&identifier, &self.revision).await {
                        | Ok(files) => {
                            let files = match self.required_paths.is_empty() {
                                | true => files,
                                | false => {
                                    let identifier = files.identifier.clone();
                                    let revision = files.revision.clone();
                                    let selected = files.into_iter().filter(|file| self.required_paths.contains(&file.path)).collect();
                                    huggingface::HuggingFaceRepositoryFiles::new(identifier, revision, selected)
                                }
                            };
                            let CommandOptions {
                                filter,
                                ignore,
                                offline,
                                no_fallback,
                                search_limit,
                                minimum_download_count,
                                interactive,
                                quiet,
                                skip_verify_checksum,
                                ..
                            } = options.clone();
                            let options = huggingface::Options::init()
                                .identifier(&identifier)
                                .maybe_filter(filter)
                                .maybe_ignore(ignore)
                                .output(destination.to_absolute_path())
                                .quiet(quiet)
                                .offline(offline)
                                .no_fallback(no_fallback)
                                .search_limit(search_limit)
                                .minimum_download_count(minimum_download_count)
                                .interactive(interactive)
                                .skip_verify_checksum(skip_verify_checksum)
                                .build();
                            #[cfg(feature = "tui")]
                            let options = huggingface::Options {
                                selector: run_gguf_picker,
                                ..options
                            };
                            let is_gguf_error = |e: &eyre::Report| e.downcast_ref::<HuggingFaceError>() == Some(&HuggingFaceError::NoGgufModelFiles);
                            match files.download(&options).await {
                                | Err(why) if files.should_use_fallback(&options) && is_gguf_error(&why) => {
                                    Self::download_gguf_fallback(&root, options).await
                                }
                                | result => result.map(|downloaded| downloaded.into_resolution(&identifier)),
                            }
                        }
                        | Err(why) if options.no_fallback => Err(why),
                        | Err(_) => {
                            let CommandOptions {
                                filter,
                                ignore,
                                offline,
                                no_fallback,
                                search_limit,
                                minimum_download_count,
                                interactive,
                                quiet,
                                skip_verify_checksum,
                                ..
                            } = options;
                            let options = huggingface::Options::init()
                                .identifier(&identifier)
                                .maybe_filter(filter)
                                .maybe_ignore(ignore)
                                .output(destination.to_absolute_path())
                                .quiet(quiet)
                                .offline(offline)
                                .no_fallback(no_fallback)
                                .search_limit(search_limit)
                                .minimum_download_count(minimum_download_count)
                                .interactive(interactive)
                                .skip_verify_checksum(skip_verify_checksum)
                                .build();
                            #[cfg(feature = "tui")]
                            let options = huggingface::Options {
                                selector: run_gguf_picker,
                                ..options
                            };
                            Self::download_gguf_fallback(&root, options).await
                        }
                    },
                    | Err(why) => {
                        error!("=> {} Create output directory for '{identifier}' — {why}", Label::fail());
                        Err(eyre!(why))
                    }
                }
            }
            | (_, None) => Err(eyre!("Model output directory is not configured")),
            | (None, _) => Err(eyre!("Model selector is not configured")),
            | _ => Err(eyre!("Model selector is not remote")),
        }
    }
    async fn download_gguf_fallback(root: &Path, options: huggingface::Options) -> ApiResult<RepositoryResolution<Downloaded>> {
        let huggingface::Options {
            identifier,
            filter,
            ignore,
            quiet,
            offline,
            search_limit,
            minimum_download_count,
            interactive,
            skip_verify_checksum,
            ..
        } = &options;
        match identifier.as_deref() {
            | Some(identifier) => {
                if !quiet {
                    info!(
                        "=> {} GGUF fallback: searching for quantization repositories derived from '{}'",
                        Label::run(),
                        identifier
                    );
                }
                let search_options = huggingface::SearchOptions::init()
                    .identifier(identifier)
                    .limit(*search_limit)
                    .minimum_download_count(*minimum_download_count)
                    .interactive(*interactive)
                    .build();
                match huggingface::search(&search_options).await {
                    | Ok(candidates) => match candidates.select(&options) {
                        | Ok(fallback_identifier) => {
                            let destination = root.join(&fallback_identifier);
                            match create_dir_all(&destination) {
                                | Ok(()) => match huggingface::repository_tree(&fallback_identifier, DEFAULT_HUGGINGFACE_MODEL_REVISION).await {
                                    | Ok(files) => {
                                        let fallback_options = huggingface::Options::init()
                                            .identifier(&fallback_identifier)
                                            .maybe_filter(filter.clone())
                                            .maybe_ignore(ignore.clone())
                                            .output(destination.to_absolute_path())
                                            .quiet(*quiet)
                                            .offline(*offline)
                                            .no_fallback(true)
                                            .search_limit(*search_limit)
                                            .minimum_download_count(*minimum_download_count)
                                            .interactive(*interactive)
                                            .skip_verify_checksum(*skip_verify_checksum)
                                            .build();
                                        files
                                            .download(&fallback_options)
                                            .await
                                            .map(|downloaded| RepositoryResolution::new(identifier, fallback_identifier, downloaded))
                                    }
                                    | Err(why) => Err(why),
                                },
                                | Err(why) => Err(eyre!(why)),
                            }
                        }
                        | Err(why) => Err(why),
                    },
                    | Err(why) => Err(why),
                }
            }
            | None => Err(eyre!("Model identifier is not configured for GGUF fallback")),
        }
    }
    fn persist_downloaded_weights(&self, resolution: &RepositoryResolution<Downloaded>, database_path: Option<PathBuf>) -> ApiResult<()> {
        let repository = resolution.value();
        let lookup = |model_id: &str| {
            ModelRow::init()
                .maybe_model_id(Some(model_id.to_string()))
                .build()
                .select(database_path.clone(), |row| row.model_id.as_deref() == Some(model_id))
        };
        let row = lookup(resolution.requested()).and_then(|row| match row {
            | Some(row) => Ok(Some(row)),
            | None => lookup(resolution.resolved()),
        });
        match row {
            | Err(why) => Err(why),
            | Ok(Some(row)) => {
                let existing = row
                    .weights
                    .as_deref()
                    .and_then(|weights| serde_json::from_str::<Weights>(weights).ok())
                    .unwrap_or_default();
                let existing_count = existing.0.len();
                let weights = repository.merge_weights(existing);
                if weights.0.len() == existing_count {
                    Ok(())
                } else {
                    serde_json::to_string(&weights)
                        .map_err(|why| eyre!("Failed to serialize downloaded model weights — {why}"))
                        .map(|weights| ModelRow {
                            weights: Some(weights),
                            ..row
                        })
                        .and_then(|row| row.update_weights(database_path).map(|_| ()))
                }
            }
            | Ok(None) => Ok(()),
        }
    }
    fn from_details(details: ModelDetails) -> Option<Self> {
        Option::<Source>::from(details.clone()).map(|source| Self::new(source).with_details(details))
    }
    fn constrain(
        self,
        quantization: Vec<Quantization>,
        filter: &[String],
        ignore: &[String],
        gpu_memory: Option<Memory>,
        database_path: &Option<PathBuf>,
    ) -> Option<Self> {
        let identifier = self.selector.identifier();
        let row = ModelRow::init()
            .model_id(identifier.clone())
            .build()
            .select(database_path.clone(), |candidate| {
                candidate.model_id.as_deref() == Some(identifier.as_str())
            });
        match row {
            | Ok(Some(row)) => {
                let groups = row
                    .weights
                    .as_deref()
                    .and_then(|weights| serde_json::from_str::<Weights>(weights).ok())
                    .map(Weights::groups)
                    .unwrap_or_default();
                match groups.0.is_empty() {
                    | true => {
                        warn!(
                            "=> {} Model '{}' eligibility could not be evaluated because GGUF file metadata is missing; proceeding",
                            Label::CAUTION,
                            self.name()
                        );
                        Some(self)
                    }
                    | false => self.constrain_with_groups(&groups, &quantization, gpu_memory.as_ref(), filter, ignore),
                }
            }
            | Ok(None) | Err(_) => {
                warn!(
                    "=> {} Model '{}' eligibility could not be evaluated because persisted metadata is unavailable; proceeding",
                    Label::CAUTION,
                    self.name()
                );
                Some(self)
            }
        }
    }
    fn constrain_with_groups(
        self,
        groups: &WeightGroups,
        quantization: &[Quantization],
        gpu_memory: Option<&Memory>,
        filter: &[String],
        ignore: &[String],
    ) -> Option<Self> {
        groups.select(quantization, gpu_memory).and_then(|group| {
            let merged_filter = merge(&self.filter, Some(filter));
            let merged_ignore = merge(&self.ignore, Some(ignore));
            let filtered = FilterSet::filter(group.paths.clone(), &merged_filter, &merged_ignore, |path| path.clone(), |_| true);
            match filtered {
                | Ok(paths) if paths.len() == group.paths.len() => {
                    let name = self.name();
                    Some(Self {
                        selector: Source::from(group.repository.as_str()).with_name(name.as_str()).with_action(self.action),
                        revision: group.revision.clone(),
                        required_paths: group.paths.clone(),
                        ..self
                    })
                }
                | Ok(_) => {
                    warn!(
                        "=> {} Model '{}' was rejected because --filter/--ignore removed a required GGUF shard",
                        Label::rejected(),
                        self.name()
                    );
                    None
                }
                | Err(why) => {
                    warn!("=> {} Model '{}' was rejected by file filters — {why}", Label::rejected(), self.name());
                    None
                }
            }
        })
    }
}
impl From<&ModelDownloadPlan> for ModelDetails {
    fn from(plan: &ModelDownloadPlan) -> Self {
        let is_open = Some(!plan.auth.is_required());
        Self {
            id: Some(plan.selector.identifier()),
            name: Some(plan.name()),
            weights: Some(Weights::from_source(&plan.selector, is_open)),
            ..Default::default()
        }
    }
}
impl From<ResolvedModelDownload> for ModelDetails {
    fn from(value: ResolvedModelDownload) -> Self {
        value.resolution.into()
    }
}
impl ModelDownloadPlans {
    pub(super) fn filter(self, filter: &[String], ignore: &[String]) -> ApiResult<Self> {
        FilterSet::compile(ignore, filter).map(|filters| {
            Self(
                self.0
                    .into_iter()
                    .filter(|plan| plan.is_oci() || filters.matches(&plan.selector.identifier()))
                    .collect(),
            )
        })
    }
    pub(super) fn constrain(
        self,
        cli_quantization: &[Quantization],
        filter: &[String],
        ignore: &[String],
        no_local_database: bool,
        cli_gpu_memory: &Option<Memory>,
        database_path: &Option<PathBuf>,
    ) -> Self {
        Self(
            self.0
                .into_iter()
                .filter_map(|plan| {
                    let quantization = match cli_quantization.is_empty() {
                        | true => plan.quantization.clone(),
                        | false => cli_quantization.to_vec(),
                    };
                    let gpu_memory = cli_gpu_memory.clone().or_else(|| plan.gpu_memory.clone());
                    match plan.is_oci() {
                        | true => {
                            let constraints = ModelDownloadPlan {
                                filter: merge(&plan.filter, Some(filter)),
                                ignore: merge(&plan.ignore, Some(ignore)),
                                quantization,
                                ..plan
                            };
                            Some(constraints)
                        },
                        | false => match (quantization.is_empty(), gpu_memory.as_ref(), no_local_database) {
                            | (true, None, _) => Some(plan),
                            | (_, _, true) => {
                                warn!(
                                    "=> {} Model '{plan}' eligibility is unknown because the local database is disabled; proceeding without metadata filtering",
                                    Label::CAUTION,
                                );
                                Some(plan)
                            }
                            | _ => plan.constrain(quantization, filter, ignore, gpu_memory, database_path),
                        },
                    }
                })
                .collect(),
        )
    }
    pub(crate) async fn resolve(&self, options: DownloadOptions) -> Vec<ResolvedModelDownload> {
        let DownloadOptions {
            offline,
            limit,
            minimum_download_count,
            interactive,
        } = options;
        let results = stream::iter(self.0.iter().cloned())
            .then(|plan| async move {
                match &plan.selector {
                    | Source::Remote { identifier, .. } if !offline && plan.is_oci() => match plan.resolve_oci().await {
                        | Ok((reference, resolution, harbor)) => {
                            let details = plan.oci_details(&reference, &resolution, harbor.as_ref());
                            let requested = identifier.clone();
                            Some(ResolvedModelDownload {
                                plan,
                                resolution: RepositoryResolution::new(requested, reference.with_digest(&resolution.resolved_digest), details),
                                oci: Some(OciPlanResolution {
                                    reference,
                                    resolution,
                                    harbor,
                                }),
                            })
                        }
                        | Err(why) => {
                            error!("=> {} Resolve OCI model {} ({why})", Label::fail(), plan.name().yellow());
                            None
                        }
                    },
                    | Source::Remote { identifier, .. } if !offline && !Location::from(identifier).is_http_or_https() => {
                        let base = plan.details.clone().unwrap_or_else(|| ModelDetails::from(&plan));
                        let search_options = huggingface::SearchOptions::init()
                            .identifier(identifier)
                            .limit(limit)
                            .minimum_download_count(minimum_download_count)
                            .interactive(interactive)
                            .build();
                        let resolution = match huggingface::parse_identifier(identifier) {
                            | Ok((provider, name)) => match huggingface::fetch_model_info(provider, name).await {
                                | Ok(info) if info.has_gguf_files() => Some(RepositoryResolution::direct(identifier, base)),
                                | Ok(_) => match base.resolve_fallback(&search_options, offline).await {
                                    | Ok(resolved) => Some(resolved),
                                    | Err(why) => {
                                        let reason = format!("({why})");
                                        error!("=> {} Resolve fallback for {} {}", Label::fail(), plan.name().yellow(), reason.dimmed());
                                        None
                                    }
                                },
                                | Err(why) if huggingface::model_is_unavailable(&why) => {
                                    let reason = "(unavailable on Hugging Face)";
                                    warn!("=> {} {} {}", Label::not_found(), identifier.yellow(), reason.dimmed());
                                    base.resolve_fallback(&search_options, offline).await.ok()
                                }
                                | Err(why) => {
                                    let reason = format!("({why})");
                                    error!("=> {} Resolve {} {}", Label::fail(), plan.name().yellow(), reason.dimmed());
                                    None
                                }
                            },
                            | Err(why) => {
                                let reason = format!("({why})");
                                error!("=> {} Resolve {} {}", Label::fail(), plan.name().yellow(), reason.dimmed());
                                None
                            }
                        };
                        resolution.map(|resolution| ResolvedModelDownload { plan, resolution, oci: None })
                    }
                    | _ => Some(ResolvedModelDownload {
                        resolution: RepositoryResolution::direct(plan.selector.identifier(), ModelDetails::from(&plan)),
                        plan,
                        oci: None,
                    }),
                }
            })
            .collect::<Vec<_>>()
            .await;
        results.into_iter().flatten().collect()
    }
}
fn profile_options(
    entry: &ModelEntryOptions,
    profiles: &HashMap<String, RegistryProfile>,
) -> ApiResult<(OciTransportOptions, Option<RegistryProfile>)> {
    match &entry.source {
        | Repository::OCI {
            location,
            registry: Some(name),
        } => profiles
            .get(name)
            .ok_or_else(|| eyre!("OCI model '{entry}' references unknown registry profile '{name}'"))
            .and_then(|profile| {
                OciReference::parse(&location.uri().unwrap_or_default()).and_then(|reference| {
                    let profile_registry = Location::from(&profile.endpoint).registry();
                    if profile_registry.as_deref() == Some(reference.registry()) {
                        Ok((OciTransportOptions::from(profile), Some(profile.clone())))
                    } else {
                        Err(eyre!("OCI registry profile '{name}' endpoint does not match {}", reference.registry()))
                    }
                })
            }),
        | _ => Ok((OciTransportOptions::default(), None)),
    }
}
/// Resolve positional selectors against config entries, merge with entry-based plans, and deduplicate by identifier.
pub(crate) async fn resolve_plans(
    positional: &ModelSelectors,
    entries: &[ModelEntry],
    profiles: &HashMap<String, RegistryProfile>,
    action: Option<SourceAction>,
    database_path: &Option<PathBuf>,
    no_local_database: bool,
    offline: bool,
) -> ApiResult<Vec<ModelDownloadPlan>> {
    let entry_options = entries
        .iter()
        .filter_map(|entry| match entry {
            | ModelEntry::Entry(options) => Some(options),
            | ModelEntry::Selector(_) => None,
        })
        .collect::<Vec<_>>();
    let future_plans = positional
        .iter()
        .map(|selector| resolve_positional_plan(selector, &entry_options, profiles, action, database_path, no_local_database, offline));
    let configured = entries
        .iter()
        .map(|entry| match entry {
            | ModelEntry::Selector(value) => {
                Ok(ModelSelector::new(value).map(|selector| ModelDownloadPlan::new(Source::from(selector.as_str())).with_action(action)))
            }
            | ModelEntry::Entry(options) => profile_options(options, profiles)
                .map(|(oci_options, profile)| Some(ModelDownloadPlan::from_entry(options, oci_options, profile).with_action(action))),
        })
        .collect::<ApiResult<Vec<_>>>();
    match configured {
        | Ok(plans) => Ok(join_all(future_plans)
            .await
            .into_iter()
            .filter_map(Result::ok)
            .chain(plans.into_iter().flatten())
            .unique_by(|plan| plan.selector.identifier())
            .collect()),
        | Err(why) => Err(why),
    }
}
async fn resolve_positional_plan(
    selector: &ModelSelector,
    entry_options: &[&ModelEntryOptions],
    profiles: &HashMap<String, RegistryProfile>,
    action: Option<SourceAction>,
    database_path: &Option<PathBuf>,
    no_local_database: bool,
    offline: bool,
) -> ApiResult<ModelDownloadPlan> {
    let selector = selector.as_str();
    let fallback = || ModelDownloadPlan::new(Source::from(selector));
    let aliases = entry_options
        .iter()
        .filter(|entry| entry.name.eq_ignore_ascii_case(selector))
        .collect::<Vec<_>>();
    let exact = entry_options.iter().find(|entry| entry.name == selector);
    let aliases = aliases.as_slice();
    let is_remote = Location::from(selector).is_http_or_https() || selector.starts_with(OCI_PREFIX);
    match (exact, aliases, is_remote) {
        | (Some(entry), _, _) => {
            profile_options(entry, profiles).map(|(options, profile)| ModelDownloadPlan::from_entry(entry, options, profile).with_action(action))
        }
        | (_, [entry], _) => {
            profile_options(entry, profiles).map(|(options, profile)| ModelDownloadPlan::from_entry(entry, options, profile).with_action(action))
        }
        | (_, [_, ..], _) => Err(eyre!("{}", ModelDownloadError::MultipleMatches(selector.to_string()))),
        | (_, _, true) => Ok(fallback()),
        | _ if no_local_database => Ok(fallback().with_action(action)),
        | _ => {
            let row = ModelRow::init().maybe_model_id(Some(selector.to_string())).build();
            let resolved = row
                .select(database_path.clone(), |candidate| candidate.model_id.as_deref() == Some(selector))
                .and_then(|match_row| match match_row {
                    | Some(row) => {
                        let name = row.name.or(row.model_id);
                        let weights = row.weights.and_then(|weights| serde_json::from_str::<Weights>(&weights).ok());
                        let has_file_metadata = weights.as_ref().is_some_and(Weights::has_file_metadata);
                        let plan = match (has_file_metadata, weights) {
                            | (true, Some(weights)) => Some(ModelDownloadPlan::new(Source::from(selector)).with_details(ModelDetails {
                                id: Some(selector.to_string()),
                                name,
                                weights: Some(weights),
                                ..Default::default()
                            })),
                            | (_, Some(weights)) => weights.to_source(name).map(ModelDownloadPlan::new),
                            | (_, None) => None,
                        };
                        match plan {
                            | Some(plan) => Ok(Some(plan)),
                            | None => Err(eyre!("{}", ModelDownloadError::UnknownModel(selector.to_string()))),
                        }
                    }
                    | None => Ok(None),
                });
            let plan = match resolved {
                | Ok(Some(plan)) => plan,
                | Ok(None) if !offline => {
                    let resolved = models_dev::download_cached().await.map(|catalog| {
                        catalog
                            .models
                            .get(selector)
                            .cloned()
                            .or_else(|| catalog.models.values().find(|details| details.id.as_deref() == Some(selector)).cloned())
                            .and_then(ModelDownloadPlan::from_details)
                    });
                    match resolved {
                        | Ok(Some(plan)) => plan,
                        | _ => fallback(),
                    }
                }
                | _ => fallback(),
            };
            Ok(plan.with_action(action))
        }
    }
}
#[cfg(feature = "tui")]
pub(crate) fn run_gguf_picker(candidates: huggingface::Candidates, options: &huggingface::Options) -> ApiResult<String> {
    if !io::stdout().is_terminal() {
        let candidate = candidates.first().ok_or_else(|| eyre!("GGUF search returned no candidates"))?;
        if !options.quiet {
            warn!(
                "=> {} Interactive GGUF picker requires a terminal; using most downloaded: '{candidate}'",
                Label::CAUTION
            );
        }
        Ok(candidate.id.clone())
    } else {
        let state = acorn_tui::State::new(acorn_tui::GgufPickerData {
            candidates: candidates.into_iter().map(acorn_tui::Candidate::from).collect(),
            base_model: options.identifier.clone().unwrap_or_default(),
            result: None,
        });
        acorn_tui::run_with(acorn_tui::Screen::GgufPicker, Some(state))?.ok_or_else(|| eyre!("GGUF repository selection cancelled"))
    }
}
#[cfg(test)]
mod metadata_filter_tests {
    use super::*;
    use acorn::schema::agent::Weight;

    fn weight(path: &str, quantization: Quantization, size: Option<u64>) -> Weight {
        Weight {
            label: path.to_string(),
            url: format!("https://huggingface.co/acme/model-GGUF/resolve/v1/{path}"),
            is_open: None,
            quantization: Some(quantization),
            size,
        }
    }
    #[test]
    fn test_weight_groups_aggregate_split_shards() {
        let groups = Weights(vec![
            weight("model-Q4_K_M-00001-of-00002.gguf", Quantization::Q4kM, Some(10)),
            weight("model-Q4_K_M-00002-of-00002.gguf", Quantization::Q4kM, Some(20)),
        ])
        .groups();
        assert_eq!(groups.0.len(), 1);
        assert_eq!(groups.0.first().map(|group| group.paths.len()), Some(2));
        assert_eq!(groups.0.first().and_then(|group| group.size), Some(30));
    }
    #[test]
    fn test_select_weight_group_uses_ordered_exact_allowlist() {
        let groups = Weights(vec![
            weight("model-Q4_K_M.gguf", Quantization::Q4kM, Some(10)),
            weight("model-Q5_K_M.gguf", Quantization::Q5kM, Some(20)),
        ])
        .groups();
        let selected = groups.select(&[Quantization::Q5kM, Quantization::Q4kM], None).unwrap();
        assert_eq!(selected.quantization, Quantization::Q5kM);
    }
    #[test]
    fn test_select_weight_group_memory_only_defaults_to_q4_k_m() {
        let groups = Weights(vec![
            weight("model-Q4_K_M.gguf", Quantization::Q4kM, Some(10)),
            weight("model-Q5_K_M.gguf", Quantization::Q5kM, Some(5)),
        ])
        .groups();
        let memory = "1GB".parse::<Memory>().unwrap();
        let selected = groups.select(&[], Some(&memory)).unwrap();
        assert_eq!(selected.quantization, Quantization::Q4kM);
    }
    #[test]
    fn test_select_weight_group_rejects_aggregate_oversize_and_allows_unknown_size() {
        let oversized = Weights(vec![
            weight("a-Q4_K_M.gguf", Quantization::Q4kM, Some(800_000_000)),
            weight("b-Q4_K_M.gguf", Quantization::Q4kM, Some(800_000_000)),
        ])
        .groups();
        let memory = "1GB".parse::<Memory>().unwrap();
        assert!(oversized.select(&[Quantization::Q4kM], Some(&memory)).is_none());
        let unknown = Weights(vec![weight("model-Q4_K_M.gguf", Quantization::Q4kM, None)]).groups();
        assert!(unknown.select(&[Quantization::Q4kM], Some(&memory)).is_some());
    }
    #[test]
    fn test_selected_group_rewrites_repository_revision_shards_and_honors_filters() {
        let groups = Weights(vec![
            weight("model-Q4_K_M-00001-of-00002.gguf", Quantization::Q4kM, Some(10)),
            weight("model-Q4_K_M-00002-of-00002.gguf", Quantization::Q4kM, Some(20)),
            weight("model-Q5_K_M.gguf", Quantization::Q5kM, Some(40)),
        ])
        .groups();
        let plan = ModelDownloadPlan::new(Source::from("acme/base"));
        let selected = plan
            .clone()
            .constrain_with_groups(&groups, &[Quantization::Q4kM], Some(&"1GB".parse().unwrap()), &[], &[])
            .unwrap();
        assert_eq!(selected.selector.identifier(), "acme/model-GGUF");
        assert_eq!(selected.revision, "v1");
        assert_eq!(selected.required_paths.len(), 2);
        assert!(plan
            .constrain_with_groups(&groups, &[Quantization::Q4kM], None, &[], &["00002".to_string()])
            .is_none());
    }
    #[test]
    fn test_plan_filter_defers_oci_patterns_to_manifest_files() {
        let plans = ModelDownloadPlans(vec![
            ModelDownloadPlan::new(Source::from("oci://registry.example.org/ai/qwen:v1")),
            ModelDownloadPlan::new(Source::from("acme/qwen")),
        ]);
        let filtered = plans.filter(&["\\.gguf$".to_string()], &["tokenizer".to_string()]).unwrap();
        assert_eq!(filtered.0.len(), 1);
        assert_eq!(
            filtered.0.first().map(|plan| plan.selector.identifier()).as_deref(),
            Some("oci://registry.example.org/ai/qwen:v1")
        );
    }
    #[tokio::test]
    async fn test_optional_harbor_enrichment_falls_back_to_oci() {
        let profile = RegistryProfile {
            kind: RegistryKind::Harbor,
            endpoint: "http://127.0.0.1:1".to_string(),
            plain_http: true,
            ..Default::default()
        };
        let plan = ModelDownloadPlan::init()
            .selector(Source::from("oci://127.0.0.1:5000/ai/qwen:v1"))
            .auth(AuthenticationRequirement::Optional)
            .maybe_registry_profile(Some(profile.clone()))
            .build();
        let reference = OciReference::parse("oci://127.0.0.1:5000/ai/qwen:v1").unwrap();
        let resolution = OciResolution {
            registry: "127.0.0.1:5000".to_string(),
            repository: "ai/qwen".to_string(),
            requested_reference: "v1".to_string(),
            resolved_digest: format!("sha256:{}", "a".repeat(64)),
            artifact_type: None,
            total_size: 0,
            transfer_size: 0,
            installed_size: Some(0),
            layers: Vec::new(),
            files: Vec::new(),
            filter: Vec::new(),
            ignore: Vec::new(),
            quantization: Vec::new(),
        };
        assert!(plan.harbor_metadata(&reference, &resolution).await.unwrap().is_none());
        let required = ModelDownloadPlan {
            auth: AuthenticationRequirement::Required,
            ..plan
        };
        assert!(required.harbor_metadata(&reference, &resolution).await.is_err());
    }
    #[tokio::test]
    async fn test_no_local_database_skips_plan_lookup_and_constraint_lookup() {
        let invalid_database_path = PathBuf::from(".");
        let plans = resolve_plans(
            &ModelSelectors::from(vec!["acme/base".to_string()]),
            &[],
            &Default::default(),
            None,
            &Some(invalid_database_path.clone()),
            true,
            true,
        )
        .await
        .unwrap();
        let selected = ModelDownloadPlans(plans).constrain(
            &[Quantization::Q4kM],
            &[],
            &[],
            true,
            &Some("1GB".parse().unwrap()),
            &Some(invalid_database_path),
        );
        assert_eq!(selected.0.len(), 1);
        assert_eq!(selected.0.first().map(|plan| plan.selector.identifier()).as_deref(), Some("acme/base"));
    }
}
