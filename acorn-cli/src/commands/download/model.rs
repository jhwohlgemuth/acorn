use crate::cli::arguments::OutputFormat;
use crate::cli::{CommandOptions, Void};
use acorn::io::api::huggingface::{self, CandidateSelection, HuggingFaceError, HuggingFaceRepository};
use acorn::io::api::models_dev;
use acorn::io::config::{ApplicationConfiguration, AuthenticationRequirement, FilterSet, ModelEntry, ModelEntryOptions};
use acorn::io::database::schema::ModelRow;
use acorn::io::database::Row;
use acorn::io::download::{DownloadItem, DownloadItems};
use acorn::io::{home_directory, ApiResult, InputOutput, Source, SourceAction};
#[cfg(feature = "tui")]
use acorn::prelude::io;
use acorn::prelude::{create_dir_all, HashSet, Path, PathBuf, String, Vec};
use acorn::schema::agent::{ModelDetails, Weights};
use acorn::util::constants::app::DEFAULT_HUGGINGFACE_MODEL_REVISION;
use acorn::util::{merge, regex_join, suffix, Label, StringConversion};
use bon::Builder;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{self, eyre};
use core::fmt;
use fluent_uri::Uri;
use futures::future::join_all;
#[cfg(feature = "tui")]
use is_terminal::IsTerminal;
use itertools::Itertools;
use owo_colors::OwoColorize;
use serde::Deserialize;
#[cfg(feature = "tui")]
use tracing::warn;
use tracing::{error, info};

#[derive(Deserialize)]
#[serde(untagged)]
enum WhitelistFile {
    Names(Vec<String>),
    Models(Vec<ModelDetails>),
}
pub(crate) struct Whitelist(pub(crate) Vec<String>);
impl From<&[String]> for Whitelist {
    fn from(values: &[String]) -> Self {
        Self(values.to_vec())
    }
}
impl From<Vec<String>> for Whitelist {
    fn from(values: Vec<String>) -> Self {
        Self(values)
    }
}
impl TryFrom<String> for Whitelist {
    type Error = eyre::Report;
    fn try_from(content: String) -> Result<Self, Self::Error> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            Err(eyre!("Whitelist file cannot be empty"))
        } else {
            match serde_norway::from_str::<WhitelistFile>(trimmed) {
                | Ok(file) => file.names().map(Self),
                | Err(why) if trimmed.starts_with('[') || trimmed.lines().any(|line| line.trim_start().starts_with("- ")) => {
                    Err(eyre!("Failed to parse whitelist file as JSON or YAML — {why}"))
                }
                | Err(_) => Ok(Self(
                    trimmed.lines().map(str::trim).filter(|line| !line.is_empty()).map(String::from).collect(),
                )),
            }
        }
    }
}
impl Whitelist {
    pub(crate) fn parse(content: String) -> ApiResult<Self> {
        Self::try_from(content)
    }
    pub(crate) async fn resolve(self, source: &Option<String>, offline: bool) -> ApiResult<Self> {
        match source {
            | Some(source) => match Source::read(source, offline).await.and_then(Self::parse) {
                | Ok(file_whitelist) => Ok(Self(self.0.into_iter().chain(file_whitelist.0).collect())),
                | Err(why) => {
                    error!("=> {} Read model whitelist file — {why}", Label::fail());
                    Err(why)
                }
            },
            | None => Ok(self),
        }
    }
}
impl WhitelistFile {
    fn names(self) -> ApiResult<Vec<String>> {
        match self {
            | Self::Names(names) => Ok(names),
            | Self::Models(models) => models
                .into_iter()
                .map(|details| {
                    details
                        .name
                        .or(details.id)
                        .ok_or_else(|| eyre!("Model details in whitelist file must define 'name' or 'id'"))
                })
                .collect(),
        }
    }
}
/// Download model weights for use with acorn research harnesses and local inference
#[allow(clippy::too_many_arguments)]
pub async fn run(
    model: &[String],
    filter: &[String],
    ignore: &[String],
    config: &Option<PathBuf>,
    output: &Option<PathBuf>,
    database_path: &Option<PathBuf>,
    verbose: &Verbosity,
    offline: bool,
    copy: bool,
    symlink: bool,
    skip_verify_checksum: bool,
    whitelist: &[String],
    no_fallback: bool,
    gguf_search_limit: usize,
    interactive: bool,
    dry_run: bool,
    raw: bool,
    format: &OutputFormat,
) -> Void {
    let quiet = verbose.is_silent();
    let action = SourceAction::from_options(copy, symlink);
    let entries = ApplicationConfiguration::resolve(config)
        .and_then(|path| ApplicationConfiguration::read(path).ok())
        .and_then(|ApplicationConfiguration { models, .. }| models)
        .unwrap_or_default();
    let whitelist = whitelist
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<HashSet<_>>();
    let planned = match resolve_plans(model, &entries, action, database_path, offline).await {
        | Ok(value) => value,
        | Err(why) => {
            error!("=> {} Resolve model selectors — {why}", Label::fail());
            return Err(why);
        }
    };
    let plans = FilterSet::filter(
        planned,
        filter,
        ignore,
        |plan| plan.selector.identifier(),
        |plan| plan.matches_whitelist(&whitelist),
    );
    let selectors = match plans {
        | Ok(value) => value,
        | Err(why) => {
            error!("=> {} Filter model selectors — {why}", Label::fail());
            return Err(why);
        }
    };
    if offline && selectors.iter().any(ModelDownloadPlan::is_remote) {
        let remote_names: Vec<String> = selectors
            .iter()
            .filter(|plan| plan.is_remote())
            .map(|plan| plan.selector.identifier())
            .collect();
        error!(
            "=> {} Offline mode cannot download remote models: {}",
            Label::fail(),
            remote_names.join(", ")
        );
        Err(eyre!("Offline mode cannot download remote models: {}", remote_names.join(", ")))
    } else if selectors.is_empty() {
        let msg = if model.is_empty() && entries.is_empty() {
            "No models specified — provide model IDs or --config"
        } else if !whitelist.is_empty() {
            "No models matched --whitelist"
        } else {
            "No models matched --filter/--ignore"
        };
        error!("=> {} {msg}", Label::fail());
        Err(eyre!(msg))
    } else if dry_run {
        let models = resolve_models(&selectors, offline, gguf_search_limit).await;
        if raw {
            output_raw(&models, format)
        } else {
            info!(
                "=> {} {} model{} resolved",
                Label::run(),
                models.len().cyan().bold(),
                suffix(models.len())
            );
            for details in models {
                let (id, context) = details.report();
                match context {
                    | Some(ctx) => info!("=> {} {} ({})", Label::using(), id.yellow(), ctx.dimmed()),
                    | None => info!("  {} {}", "→".cyan(), id.green()),
                }
            }
            Ok(())
        }
    } else {
        match output.clone().map_or_else(|| home_directory(".models"), Ok) {
            | Ok(root) => {
                let mut success_count = 0usize;
                let mut failure_count = 0usize;
                let mut results: Vec<ModelDetails> = Vec::new();
                for plan in &selectors {
                    let filter = merge(&plan.filter, Some(filter));
                    let ignore = merge(&plan.ignore, Some(ignore));
                    let options = CommandOptions::init()
                        .selector(plan.selector.clone())
                        .maybe_action(match &plan.selector {
                            | Source::Local { action, .. } => *action,
                            | _ => plan.action,
                        })
                        .output(root.clone())
                        .maybe_database_path(database_path.clone())
                        .offline(offline)
                        .no_fallback(no_fallback)
                        .gguf_search_limit(gguf_search_limit)
                        .interactive(interactive)
                        .quiet(quiet || raw)
                        .skip_verify_checksum(skip_verify_checksum)
                        .maybe_filter(regex_join(&filter))
                        .maybe_ignore(regex_join(&ignore))
                        .build();
                    match plan.execute(options).await {
                        | Ok(resolved_id) => {
                            success_count = success_count.saturating_add(1);
                            let details = plan.details.clone().unwrap_or_else(|| plan.into()).with_id(&resolved_id);
                            results.push(details);
                        }
                        | Err(why) => {
                            failure_count = failure_count.saturating_add(1);
                            error!("=> {} Download model '{}' — {why}", Label::fail(), plan.name());
                        }
                    }
                }
                if raw {
                    output_raw(&results, format)
                } else {
                    if !quiet {
                        let local_count = selectors.iter().filter(|plan| plan.selector.is_local()).count();
                        let remote_count = selectors.len().saturating_sub(local_count);
                        let parts = [(local_count, "local"), (remote_count, "remote")]
                            .into_iter()
                            .filter(|(count, _)| *count > 0)
                            .map(|(count, label)| format!("{} {label}", count.cyan().bold()))
                            .collect::<Vec<_>>();
                        info!(
                            "=> {} {} model{} to download ({})",
                            Label::run(),
                            selectors.len().cyan().bold(),
                            suffix(selectors.len()),
                            parts.join(", ")
                        );
                        for plan in selectors {
                            info!("  {} {}", "→".cyan(), plan.selector.green());
                        }
                    }
                    super::log_activity("download model", root.to_absolute_string(), database_path, failure_count == 0);
                    if failure_count > 0 {
                        Err(eyre!("Failed to download {failure_count} model{}", suffix(failure_count)))
                    } else {
                        if !quiet {
                            println!(
                                "\n\n   => {} Resolved {} model{} to {}\n",
                                Label::pass(),
                                success_count.green(),
                                suffix(success_count),
                                root.display()
                            );
                        }
                        Ok(())
                    }
                }
            }
            | Err(why) => Err(why),
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) enum ModelDownloadError {
    UnknownModel(String),
    MultipleMatches(String),
    MissingAuth(String),
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
    pub(crate) action: Option<SourceAction>,
    pub(crate) details: Option<ModelDetails>,
}
impl fmt::Display for ModelDownloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::UnknownModel(value) => write!(formatter, "Unknown model selector: {value}"),
            | Self::MultipleMatches(value) => write!(formatter, "Multiple model matches found: {value}"),
            | Self::MissingAuth(value) => write!(formatter, "Model download requires authentication: {value}"),
        }
    }
}
impl core::error::Error for ModelDownloadError {}
impl ModelDownloadPlan {
    fn new(selector: Source) -> Self {
        let action = match &selector {
            | Source::Local { action, .. } => *action,
            | _ => None,
        };
        Self::init().selector(selector).maybe_action(action).build()
    }
    fn from_entry(entry: &ModelEntryOptions) -> Self {
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
            .maybe_action(config_action)
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
    fn is_remote(&self) -> bool {
        self.selector.is_remote()
    }
    fn matches_whitelist(&self, whitelist: &HashSet<String>) -> bool {
        whitelist.is_empty() || whitelist.contains(&self.name())
    }
    pub(crate) fn name(&self) -> String {
        self.selector.name()
    }
    async fn execute(&self, options: CommandOptions) -> ApiResult<String> {
        if matches!(self.auth, AuthenticationRequirement::Required) && self.selector.is_remote() && !huggingface::has_auth_token() {
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
                | Some(Source::Remote { identifier, .. }) if is_http_or_https(identifier) => download_remote_model(options).await,
                | Some(Source::Remote { .. }) => self.download_from_huggingface(options.clone()).await.map(|identifier| {
                    if !options.quiet {
                        let name = self.name();
                        let destination = options
                            .output
                            .as_ref()
                            .map(|root| root.join(&identifier))
                            .map(|path| path.display().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        info!("=> {} Resolved model '{}' → {}", Label::pass(), name.cyan(), destination.dimmed());
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
                    identifier
                }),
                | Some(Source::Unsupported(value)) => Err(eyre!("Unsupported model source — {value}")),
                | None => Err(eyre!("Model selector is not configured")),
            }
        }
    }
    async fn download_from_huggingface(&self, options: CommandOptions) -> ApiResult<String> {
        match (options.selector.clone(), options.output.clone()) {
            | (Some(Source::Remote { identifier, .. }), Some(root)) => {
                let destination = root.join(&identifier);
                match create_dir_all(&destination) {
                    | Ok(()) => match huggingface::repository_tree(&identifier, &self.revision).await {
                        | Ok(files) => {
                            let CommandOptions {
                                filter,
                                ignore,
                                offline,
                                no_fallback,
                                gguf_search_limit,
                                interactive,
                                quiet,
                                skip_verify_checksum,
                                ..
                            } = options.clone();
                            let options = huggingface::Options::init()
                                .identifier(&identifier)
                                .maybe_filter(filter)
                                .maybe_ignore(ignore)
                                .output(destination.to_absolute_string())
                                .quiet(quiet)
                                .offline(offline)
                                .no_fallback(no_fallback)
                                .search_limit(gguf_search_limit)
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
                                    download_gguf_fallback(&root, options).await
                                }
                                | result => result,
                            }
                        }
                        | Err(why) => Err(why),
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
    fn from_details(details: ModelDetails) -> Option<Self> {
        Option::<Source>::from(details.clone()).map(|source| Self::new(source).with_details(details))
    }
}
impl From<&ModelDownloadPlan> for ModelDetails {
    fn from(plan: &ModelDownloadPlan) -> Self {
        let is_open = Some(plan.auth != AuthenticationRequirement::Required);
        Self {
            id: Some(plan.selector.identifier()),
            name: Some(plan.name()),
            weights: Some(Weights::from_source(&plan.selector, is_open)),
            ..Default::default()
        }
    }
}
async fn download_gguf_fallback(root: &Path, options: huggingface::Options) -> ApiResult<String> {
    let huggingface::Options {
        identifier,
        filter,
        ignore,
        quiet,
        offline,
        search_limit: gguf_search_limit,
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
            match huggingface::search(identifier, None, Some(*gguf_search_limit)).await {
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
                                        .output(destination.to_absolute_string())
                                        .quiet(*quiet)
                                        .offline(*offline)
                                        .no_fallback(true)
                                        .search_limit(*gguf_search_limit)
                                        .interactive(*interactive)
                                        .skip_verify_checksum(*skip_verify_checksum)
                                        .build();
                                    files.download(&fallback_options).await
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
async fn download_remote_model(options: CommandOptions) -> ApiResult<String> {
    let CommandOptions {
        selector,
        output,
        quiet,
        skip_verify_checksum,
        ..
    } = options;
    match (selector, output) {
        | (Some(Source::Remote { identifier, .. }), Some(root)) => {
            let filename = match Uri::parse(identifier.as_str()) {
                | Ok(uri) => uri
                    .path()
                    .to_string()
                    .rsplit('/')
                    .next()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
                    .unwrap_or_else(|| "model.gguf".to_string()),
                | Err(_) => "model.gguf".to_string(),
            };
            let name = PathBuf::from(&filename)
                .file_stem()
                .and_then(|value| value.to_str())
                .map(str::to_string)
                .unwrap_or_else(|| "model".to_string());
            let destination = root.join(&name);
            let item = DownloadItem {
                url: identifier,
                path: filename,
                size: None,
                sha: None,
            };
            DownloadItems::new(&destination, vec![item], quiet, skip_verify_checksum)
                .download()
                .await
                .map(|_| name)
        }
        | (_, None) => Err(eyre!("Model output directory is not configured")),
        | (None, _) => Err(eyre!("Model selector is not configured")),
        | _ => Err(eyre!("Model selector is not remote")),
    }
}
fn materialize_local_model(options: CommandOptions) -> ApiResult<String> {
    match options.selector.clone() {
        | Some(Source::Local { .. }) => {
            let name = options.selector.as_ref().map(Source::name).unwrap_or_else(|| "model".to_string());
            let destination = match options.output.clone() {
                | Some(root) => Ok(root.join(&name)),
                | None => Err(eyre!("Model output directory is not configured")),
            };
            match destination {
                | Ok(destination) => match create_dir_all(&destination) {
                    | Ok(_) => match options.selector.clone() {
                        | Some(Source::Local { path, .. }) => {
                            let action = options.action.unwrap_or_default();
                            let target = match action {
                                | SourceAction::Reference => Ok(destination),
                                | SourceAction::Copy | SourceAction::Symlink => path
                                    .file_name()
                                    .map(|value| destination.join(value))
                                    .ok_or_else(|| eyre!("Local model path has no file name")),
                            };
                            target.and_then(|target| action.materialize(&path, &target, &name))
                        }
                        | Some(_) => Err(eyre!("Model selector is not local")),
                        | None => Err(eyre!("Model selector is not configured")),
                    },
                    | Err(why) => {
                        error!("=> {} Create output directory for '{name}' — {why}", Label::fail());
                        Err(why.into())
                    }
                },
                | Err(why) => Err(why),
            }
        }
        | Some(_) => Err(eyre!("Model selector is not local")),
        | None => Err(eyre!("Model selector is not configured")),
    }
}
fn output_raw(models: &[ModelDetails], format: &OutputFormat) -> Void {
    let output = match format {
        | OutputFormat::Yaml => serde_norway::to_string(models).map_err(|why| eyre!("{format} serialization failed — {why}")),
        | OutputFormat::Json => serde_json::to_string_pretty(models).map_err(|why| eyre!("{format} serialization failed — {why}")),
    };
    output.map(|serialized| {
        println!("{serialized}");
    })
}
/// Resolve positional selectors against config entries, merge with entry-based plans, and deduplicate by identifier.
pub(crate) async fn resolve_plans(
    positional: &[String],
    entries: &[ModelEntry],
    action: Option<SourceAction>,
    database_path: &Option<PathBuf>,
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
        .map(|selector| resolve_positional_plan(selector, &entry_options, action, database_path, offline));
    let plans = join_all(future_plans)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .chain(entries.iter().filter_map(|entry| match entry {
            | ModelEntry::Selector(s) => (!s.trim().is_empty()).then(|| ModelDownloadPlan::new(Source::from(s.trim())).with_action(action)),
            | ModelEntry::Entry(options) => Some(ModelDownloadPlan::from_entry(options).with_action(action)),
        }))
        .unique_by(|plan| plan.selector.identifier())
        .collect();
    Ok(plans)
}
async fn resolve_models(plans: &[ModelDownloadPlan], offline: bool, limit: usize) -> Vec<ModelDetails> {
    let results = join_all(plans.iter().map(|plan| {
        let plan = plan.clone();
        async move {
            match &plan.selector {
                | Source::Remote { identifier, .. } if !offline => {
                    let base = plan.details.clone().unwrap_or_else(|| ModelDetails::from(&plan));
                    match huggingface::search(identifier, None, Some(limit)).await {
                        | Ok(candidates) => {
                            let options = huggingface::Options::init()
                                .identifier(identifier.clone())
                                .offline(offline)
                                .search_limit(limit)
                                .quiet(true)
                                .build();
                            match candidates.select(&options) {
                                | Ok(resolved) if resolved != *identifier => base.with_id(&resolved).with_fallback(identifier),
                                | Ok(_) => base,
                                | Err(why) => {
                                    error!("=> {} Resolve fallback for '{}' — {why}", Label::fail(), plan.name());
                                    base
                                }
                            }
                        }
                        | Err(why) => {
                            error!("=> {} Search candidates for '{}' — {why}", Label::fail(), plan.name());
                            base
                        }
                    }
                }
                | _ => ModelDetails::from(&plan),
            }
        }
    }))
    .await;
    results
}
async fn resolve_positional_plan(
    selector: &str,
    entry_options: &[&ModelEntryOptions],
    action: Option<SourceAction>,
    database_path: &Option<PathBuf>,
    offline: bool,
) -> ApiResult<ModelDownloadPlan> {
    let trimmed = selector.trim();
    let fallback = || ModelDownloadPlan::new(Source::from(trimmed));
    let aliases = entry_options
        .iter()
        .filter(|entry| entry.name.eq_ignore_ascii_case(trimmed))
        .collect::<Vec<_>>();
    match (
        entry_options.iter().find(|entry| entry.name == trimmed),
        aliases.as_slice(),
        is_http_or_https(trimmed),
    ) {
        | (Some(entry), _, _) => Ok(ModelDownloadPlan::from_entry(entry).with_action(action)),
        | (_, [entry], _) => Ok(ModelDownloadPlan::from_entry(entry).with_action(action)),
        | (_, [_, ..], _) => Err(eyre!("{}", ModelDownloadError::MultipleMatches(trimmed.to_string()))),
        | (_, _, true) => Ok(fallback()),
        | _ => {
            let row = ModelRow::init().maybe_model_id(Some(selector.to_string())).build();
            let resolved = row
                .select(database_path.clone(), |candidate| candidate.model_id.as_deref() == Some(selector))
                .and_then(|match_row| match match_row {
                    | Some(row) => {
                        let name = row.name.or(row.model_id);
                        let row_source = row
                            .weights
                            .and_then(|weights| serde_json::from_str::<Weights>(&weights).ok())
                            .and_then(|weights| weights.to_source(name));
                        match row_source {
                            | Some(source) => Ok(Some(ModelDownloadPlan::new(source))),
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
                            .get(trimmed)
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
fn run_gguf_picker(candidates: huggingface::Candidates, options: &huggingface::Options) -> ApiResult<String> {
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
        acorn_tui::run_tui_with(acorn_tui::Screen::GgufPicker, Some(state))?.ok_or_else(|| eyre!("GGUF repository selection cancelled"))
    }
}
fn is_http_or_https(value: &str) -> bool {
    Uri::parse(value)
        .ok()
        .is_some_and(|uri| matches!(uri.scheme().as_str(), "http" | "https"))
}
