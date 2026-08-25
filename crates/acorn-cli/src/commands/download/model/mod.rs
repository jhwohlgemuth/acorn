//! Model download command orchestration and execution.
pub mod plan;

use self::plan::{resolve_plans, DownloadOptions, ModelDownloadError, ModelDownloadPlan, ModelDownloadPlans};
use super::log_activity;
use crate::cli::arguments::{DownloadModel, ModelResolutionArgs, ModelSourceArgs, ModelSyncArgs, OutputFormat};
use crate::cli::{CommandOptions, Void};
use acorn::io::config::{ApplicationConfiguration, ModelEntry};
use acorn::io::download::{DownloadItem, DownloadItems};
use acorn::io::{first_env_var, home_directory, sync, ApiResult, ModelListFile, Source, SourceAction};
use acorn::prelude::{create_dir_all, HashSet, PathBuf, String, Vec};
use acorn::schema::agent::{ModelDetails, ModelSelectors};
use acorn::schema::OneOrMany;
use acorn::util::constants::app::DEFAULT_MODELS_DIRECTORY;
use acorn::util::constants::env::MODEL_WHITELIST;
use acorn::util::{merge, regex_join, suffix, Label, StringConversion};
use acorn::Location;
use color_eyre::eyre::{self, eyre};
use fluent_uri::Uri;
use futures::{stream, StreamExt, TryStreamExt};
use itertools::Itertools;
use owo_colors::OwoColorize;
use tracing::{error, info};

/// Download model weights with optional persisted-metadata constraints.
pub(crate) struct Request<'a> {
    pub(crate) arguments: &'a DownloadModel,
    pub(crate) whitelist: &'a [String],
    pub(crate) database_path: &'a Option<PathBuf>,
    pub(crate) offline: bool,
    pub(crate) no_local_database: bool,
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
            match serde_norway::from_str::<ModelListFile>(trimmed) {
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
    pub(crate) async fn from_configuration(values: Option<OneOrMany<String>>, offline: bool) -> ApiResult<Self> {
        match values {
            | Some(OneOrMany::One(value)) if Location::from(&value).is_http_or_https() => Self(Vec::new()).resolve(&Some(value), offline).await,
            | Some(values) => Ok(Self(values.into_vec())),
            | None => Ok(Self(Vec::new())),
        }
    }
    pub(crate) async fn prioritized(cli: &[String], configured: Option<OneOrMany<String>>, offline: bool) -> ApiResult<Self> {
        match cli {
            | [_, ..] => Ok(Self(cli.to_vec())),
            | [] => match Self::from_configuration(configured, offline).await {
                | Ok(whitelist) if !whitelist.0.is_empty() => Ok(whitelist),
                | Ok(_) => match first_env_var(&[MODEL_WHITELIST]) {
                    | Some(value) => {
                        let values = value
                            .split(',')
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(String::from)
                            .collect::<Vec<_>>();
                        let values = match values.as_slice() {
                            | [value] => Some(OneOrMany::One(value.clone())),
                            | [_, ..] => Some(OneOrMany::Many(values)),
                            | [] => None,
                        };
                        Self::from_configuration(values, offline).await
                    }
                    | None => Ok(Self(Vec::new())),
                },
                | Err(why) => Err(why),
            },
        }
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
pub async fn execute(request: Request<'_>) -> Void {
    let Request {
        arguments,
        whitelist,
        database_path,
        offline,
        no_local_database,
    } = request;
    let DownloadModel {
        source: ModelSourceArgs { model, model_file, config },
        resolution:
            ModelResolutionArgs {
                no_fallback,
                interactive,
                search_limit,
                minimum_download_count,
            },
        synchronization: ModelSyncArgs {
            sync: sync_target,
            force,
            dry_run,
        },
        filter,
        ignore,
        quantization,
        gpu_memory,
        local_dir: output,
        copy,
        symlink,
        skip_verify_checksum,
        raw,
        format,
        verbose,
        ..
    } = arguments;
    let model = model.as_slice();
    let filter = filter.as_slice();
    let ignore = ignore.as_slice();
    let quantization = quantization.as_slice();
    let quiet = verbose.is_silent();
    let use_configured_models = model.is_empty() && model_file.is_none();
    let action = SourceAction::from_options(*copy, *symlink);
    let prepared = match ModelSelectors::from(model).resolve(model_file, offline).await {
        | Ok(model) => match ApplicationConfiguration::load(config) {
            | Ok(configuration) => {
                let (configured_entries, configured_whitelist) = configuration.model_entries_and_whitelist();
                match configuration.registry_lookup() {
                    | Ok(registry_profiles) => {
                        let entries = match use_configured_models {
                            | true => configured_entries,
                            | false => Vec::new(),
                        };
                        match Whitelist::prioritized(whitelist, configured_whitelist, offline).await {
                            | Ok(whitelist) => Ok((
                                model,
                                configuration,
                                entries,
                                registry_profiles,
                                whitelist
                                    .0
                                    .iter()
                                    .map(|value| value.trim().to_string())
                                    .filter(|value| !value.is_empty())
                                    .collect::<HashSet<_>>(),
                            )),
                            | Err(why) => {
                                let why = ModelDownloadError::Whitelist(why.to_string());
                                error!("=> {} Resolve model whitelist — {why}", Label::fail());
                                Err(eyre!(why))
                            }
                        }
                    }
                    | Err(why) => {
                        let why = ModelDownloadError::RegistryConfiguration(why.to_string());
                        error!("=> {} Validate OCI registry configuration — {why}", Label::fail());
                        Err(eyre!(why))
                    }
                }
            }
            | Err(why) => {
                let why = ModelDownloadError::Configuration(why.to_string());
                error!("=> {} Read application configuration — {why}", Label::fail());
                Err(eyre!(why))
            }
        },
        | Err(why) => {
            let why = ModelDownloadError::ModelList(why.to_string());
            error!("=> {} Read model list file — {why}", Label::fail());
            Err(eyre!(why))
        }
    };
    match prepared {
        | Ok((model, configuration, entries, registry_profiles, whitelist)) => {
            match resolve_plans(&model, &entries, &registry_profiles, action, database_path, no_local_database, offline).await {
                | Ok(planned) => match ModelDownloadPlans(planned).filter(filter, ignore) {
                    | Ok(plans) => {
                        let plans = plans.0.into_iter().sorted_by_key(|plan| plan.selector.identifier());
                        let futures =
                            stream::iter(plans).then(|plan| async { plan.matches(&whitelist, offline).await.map(|matches| matches.then_some(plan)) });
                        let whitelist_result = futures.try_collect::<Vec<_>>().await.map_err(|why| {
                            error!("=> {} Validate model whitelist — {why}", Label::fail());
                            why
                        });
                        match whitelist_result {
                            | Ok(value) => {
                                let selectors = value.into_iter().flatten().collect::<Vec<_>>();
                                let cli_constrained = !quantization.is_empty() || gpu_memory.is_some();
                                let config_constrained = selectors.iter().any(|plan| !plan.quantization.is_empty() || plan.gpu_memory.is_some());
                                let constrained = cli_constrained || config_constrained;
                                let selectors = ModelDownloadPlans(selectors).constrain(
                                    quantization,
                                    filter,
                                    ignore,
                                    no_local_database,
                                    gpu_memory,
                                    database_path,
                                );
                                let no_selectors = selectors.0.is_empty();
                                let has_remote = selectors.0.iter().any(ModelDownloadPlan::is_remote);
                                match (no_selectors, constrained, offline && has_remote) {
                                    | (true, true, _) => {
                                        let why = ModelDownloadError::NoModelsFound;
                                        error!("=> {} {why}", Label::fail());
                                        Err(eyre!(why))
                                    }
                                    | (_, _, true) => {
                                        let remote_names: Vec<String> = selectors
                                            .0
                                            .iter()
                                            .filter(|plan| plan.is_remote())
                                            .map(|plan| plan.selector.identifier())
                                            .collect();
                                        let why = ModelDownloadError::OfflineRemoteModels(remote_names.join(", "));
                                        error!("=> {} {why}", Label::fail());
                                        Err(eyre!(why))
                                    }
                                    | (true, false, false) => {
                                        let why = match (model.is_empty() && entries.is_empty(), whitelist.is_empty()) {
                                            | (true, _) => ModelDownloadError::NoModelsSpecified,
                                            | (false, false) => ModelDownloadError::NoWhitelistMatches,
                                            | (false, true) => ModelDownloadError::NoFilterMatches,
                                        };
                                        error!("=> {} {why}", Label::fail());
                                        Err(eyre!(why))
                                    }
                                    | (false, _, false) => {
                                        let options = DownloadOptions {
                                            offline,
                                            limit: *search_limit,
                                            minimum_download_count: *minimum_download_count,
                                            interactive: *interactive,
                                        };
                                        let models = selectors.resolve(options).await;
                                        match (models.is_empty(), *dry_run) {
                                            | (true, _) => {
                                                let why = ModelDownloadError::NoModelsFound;
                                                error!("=> {} {why}", Label::fail());
                                                Err(eyre!(why))
                                            }
                                            | (false, true) => {
                                                let models = models.into_iter().map(ModelDetails::from).collect::<Vec<_>>();
                                                let rendered = match *raw {
                                                    | true => output_raw(&models, format),
                                                    | false => {
                                                        info!(
                                                            "=> {} Resolved {} model{}",
                                                            Label::run(),
                                                            models.len().cyan().bold(),
                                                            suffix(models.len())
                                                        );
                                                        for details in &models {
                                                            let (id, context) = details.report();
                                                            match context {
                                                                | Some(ctx) => info!("=> {} {} {}", Label::using(), id.green(), ctx.dimmed()),
                                                                | None => info!("  {} {}", "→".cyan(), id.green()),
                                                            }
                                                        }
                                                        Ok(())
                                                    }
                                                };
                                                rendered.and_then(|()| {
                                                    sync_target.as_ref().map_or(Ok(()), |target| {
                                                        let entries = models
                                                            .iter()
                                                            .filter_map(|details| details.id.clone())
                                                            .unique()
                                                            .map(ModelEntry::Selector)
                                                            .collect::<Vec<_>>();
                                                        let options = sync::Options::init()
                                                            .entries(&entries)
                                                            .opencode(target.is_opencode())
                                                            .llama_swap(target.is_llama_swap())
                                                            .vscode(target.is_vscode())
                                                            .goose(target.is_goose())
                                                            .force(*force)
                                                            .assume_models(*force)
                                                            .dry_run(true)
                                                            .maybe_models_dir(output.as_deref())
                                                            .build();
                                                        configuration.sync_and_update(config, options)
                                                    })
                                                })
                                            }
                                            | (false, false) => match output.clone().map_or_else(|| home_directory(DEFAULT_MODELS_DIRECTORY), Ok) {
                                                | Ok(root) => {
                                                    let mut success_count = 0usize;
                                                    let mut failure_count = 0usize;
                                                    let mut results: Vec<ModelDetails> = Vec::new();
                                                    for model in &models {
                                                        let plan = model.plan();
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
                                                            .no_local_database(no_local_database)
                                                            .no_fallback(*no_fallback)
                                                            .search_limit(*search_limit)
                                                            .minimum_download_count(*minimum_download_count)
                                                            .interactive(*interactive)
                                                            .quiet(quiet || *raw)
                                                            .skip_verify_checksum(*skip_verify_checksum)
                                                            .maybe_filter(regex_join(&filter))
                                                            .maybe_ignore(regex_join(&ignore))
                                                            .build();
                                                        match model.execute(options).await {
                                                            | Ok(details) => {
                                                                success_count = success_count.saturating_add(1);
                                                                results.push(details);
                                                            }
                                                            | Err(why) => {
                                                                failure_count = failure_count.saturating_add(1);
                                                                error!("=> {} Download model '{}' — {why}", Label::fail(), plan.name());
                                                            }
                                                        }
                                                    }
                                                    match *raw {
                                                        | true => output_raw(&results, format),
                                                        | false => {
                                                            if !quiet {
                                                                let local_count =
                                                                    models.iter().filter(|model| model.plan().selector.is_local()).count();
                                                                let remote_count = models.len().saturating_sub(local_count);
                                                                let parts = [(local_count, "local"), (remote_count, "remote")]
                                                                    .into_iter()
                                                                    .filter(|(count, _)| *count > 0)
                                                                    .map(|(count, label)| format!("{} {label}", count.cyan().bold()))
                                                                    .collect::<Vec<_>>();
                                                                info!(
                                                                    "=> {} {} model{} to download ({})",
                                                                    Label::run(),
                                                                    models.len().cyan().bold(),
                                                                    suffix(models.len()),
                                                                    parts.join(", ")
                                                                );
                                                                for model in &models {
                                                                    info!("  {} {}", "→".cyan(), model.plan().green());
                                                                }
                                                            }
                                                            if !no_local_database {
                                                                log_activity(
                                                                    "download model",
                                                                    root.to_absolute_path(),
                                                                    database_path,
                                                                    failure_count == 0,
                                                                );
                                                            }
                                                            match failure_count {
                                                                | 0 => sync_target
                                                                    .as_ref()
                                                                    .map_or(Ok(()), |target| {
                                                                        let entries = results
                                                                            .iter()
                                                                            .filter_map(|details| details.id.clone())
                                                                            .unique()
                                                                            .map(ModelEntry::Selector)
                                                                            .collect::<Vec<_>>();
                                                                        let options = sync::Options::init()
                                                                            .entries(&entries)
                                                                            .opencode(target.is_opencode())
                                                                            .llama_swap(target.is_llama_swap())
                                                                            .vscode(target.is_vscode())
                                                                            .goose(target.is_goose())
                                                                            .force(*force)
                                                                            .assume_models(*force)
                                                                            .models_dir(&root)
                                                                            .build();
                                                                        configuration.sync_and_update(config, options)
                                                                    })
                                                                    .map(|()| {
                                                                        if !quiet {
                                                                            println!(
                                                                                "\n\n   => {} Resolved {} model{} to {}\n",
                                                                                Label::pass(),
                                                                                success_count.green(),
                                                                                suffix(success_count),
                                                                                root.display()
                                                                            );
                                                                        }
                                                                    }),
                                                                | _ => {
                                                                    Err(eyre!("Failed to download {failure_count} model{}", suffix(failure_count)))
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                | Err(why) => Err(why),
                                            },
                                        }
                                    }
                                }
                            }
                            | Err(why) => {
                                error!("=> {} Validate model whitelist — {why}", Label::fail());
                                Err(why)
                            }
                        }
                    }
                    | Err(why) => {
                        error!("=> {} Filter model selectors — {why}", Label::fail());
                        Err(why)
                    }
                },
                | Err(why) => {
                    error!("=> {} Resolve model selectors — {why}", Label::fail());
                    Err(why)
                }
            }
        }
        | Err(why) => Err(why),
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
