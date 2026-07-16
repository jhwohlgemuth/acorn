use crate::cli::{CommandOptions, Void};
use acorn::io::api::huggingface::{self, HuggingFaceRepository};
use acorn::io::api::models_dev;
use acorn::io::config::{ApplicationConfiguration, AuthenticationRequirement, FilterSet, ModelEntry, ModelEntryOptions};
use acorn::io::database::schema::ModelRow;
use acorn::io::database::Row;
use acorn::io::download::{DownloadItem, DownloadItems};
use acorn::io::source::source_from_weights;
use acorn::io::{home_directory, ApiResult, InputOutput, Source, SourceAction};
use acorn::prelude::{create_dir_all, HashSet, PathBuf, String, Vec};
use acorn::schema::agent::Weight;
use acorn::util::{merge, regex_join, suffix, Label, StringConversion};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::eyre;
use core::fmt;
use fluent_uri::Uri;
use futures::future::join_all;
use itertools::Itertools;
use owo_colors::OwoColorize;
use tracing::{error, info};

const DEFAULT_MODEL_REVISION: &str = "main";

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
    } else {
        match output.clone().map_or_else(|| home_directory(".models"), Ok) {
            | Ok(root) => {
                let mut success_count = 0usize;
                let mut failure_count = 0usize;
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
                        .quiet(quiet)
                        .skip_verify_checksum(skip_verify_checksum)
                        .maybe_filter(regex_join(&filter))
                        .maybe_ignore(regex_join(&ignore))
                        .build();
                    match plan.execute(options).await {
                        | Ok(_) => success_count = success_count.saturating_add(1),
                        | Err(why) => {
                            failure_count = failure_count.saturating_add(1);
                            error!("=> {} Download model '{}' — {why}", Label::fail(), plan.name());
                        }
                    }
                }
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
                        match &plan.selector {
                            | Source::Local { path, .. } => {
                                info!("  {} {} ({})", "→".cyan(), plan.name().green(), path.display().to_string().dimmed());
                            }
                            | Source::Remote { identifier, .. } => {
                                let name = plan.name();
                                if name == *identifier {
                                    info!("  {} {}", "→".cyan(), identifier.green());
                                } else {
                                    info!("  {} {} ({})", "→".cyan(), name.green(), identifier.dimmed());
                                }
                            }
                            | Source::Unsupported(value) => info!("  {} {}", "→".cyan(), value.green()),
                        }
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
#[derive(Clone, Debug)]
pub(crate) struct ModelDownloadPlan {
    pub(crate) selector: Source,
    pub(crate) revision: String,
    pub(crate) auth: AuthenticationRequirement,
    pub(crate) filter: Vec<String>,
    pub(crate) ignore: Vec<String>,
    pub(crate) action: Option<SourceAction>,
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
        Self {
            action: match &selector {
                | Source::Local { action, .. } => *action,
                | _ => None,
            },
            selector,
            revision: DEFAULT_MODEL_REVISION.to_string(),
            auth: AuthenticationRequirement::Optional,
            filter: Vec::new(),
            ignore: Vec::new(),
        }
    }
    fn from_entry(entry: &ModelEntryOptions) -> Self {
        let config_action = match (&entry.copy, &entry.symlink) {
            | (Some(true), _) => Some(SourceAction::Copy),
            | (_, Some(true)) => Some(SourceAction::Symlink),
            | _ => None,
        };
        Self {
            selector: Source::from(&entry.source).with_name(entry.name.as_str()).with_action(config_action),
            revision: entry.revision.clone().unwrap_or_else(|| DEFAULT_MODEL_REVISION.to_string()),
            auth: entry.auth.clone().unwrap_or_default(),
            filter: entry.filter.clone().unwrap_or_default(),
            ignore: entry.ignore.clone().unwrap_or_default(),
            action: config_action,
        }
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
    fn is_remote(&self) -> bool {
        self.selector.is_remote()
    }
    fn matches_whitelist(&self, whitelist: &HashSet<String>) -> bool {
        whitelist.is_empty() || whitelist.contains(&self.name())
    }
    pub(crate) fn name(&self) -> String {
        self.selector.name()
    }
    async fn execute(&self, options: CommandOptions) -> ApiResult<()> {
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
                            let options = huggingface::Options::init()
                                .identifier(&identifier)
                                .maybe_output(Some(destination.to_absolute_string()))
                                .maybe_filter(options.filter.clone())
                                .maybe_ignore(options.ignore.clone())
                                .skip_verify_checksum(options.skip_verify_checksum)
                                .quiet(options.quiet)
                                .build();
                            files.download(options).await
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
}
async fn download_remote_model(options: CommandOptions) -> ApiResult<()> {
    match (options.selector.clone(), options.output.clone()) {
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
            let destination = PathBuf::from(&filename)
                .file_stem()
                .and_then(|value| value.to_str())
                .map(|name| root.join(name))
                .unwrap_or_else(|| root.join("model"));
            let item = DownloadItem {
                url: identifier,
                path: filename,
                size: None,
                sha: None,
            };
            DownloadItems::new(&destination, vec![item], options.quiet, false).download().await
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
                            .and_then(|weights| serde_json::from_str::<Vec<Weight>>(&weights).ok())
                            .and_then(|weights| source_from_weights(weights, name));
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
                            .and_then(Option::<Source>::from)
                            .map(ModelDownloadPlan::new)
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
fn is_http_or_https(value: &str) -> bool {
    Uri::parse(value)
        .ok()
        .is_some_and(|uri| matches!(uri.scheme().as_str(), "http" | "https"))
}
