use super::{DownloadItem, DownloadItems};
use crate::cli::{CommandOptions, Void};
use acorn::io::api::huggingface::{self, RepositoryFile};
use acorn::io::config::{ApplicationConfiguration, FilterSet, ModelEntry, ModelEntryOptions};
use acorn::io::database::schema::ModelRow;
use acorn::io::database::Row;
use acorn::io::{home_directory, symlink, ApiResult, InputOutput, Source, SourceAction};
use acorn::prelude::{copy, create_dir_all, HashSet, PathBuf, String, Vec};
use acorn::util::{regex_join, suffix, Label, StringConversion};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::eyre;
use fluent_uri::Uri;
use owo_colors::OwoColorize;
use tracing::{error, info};

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
    let selectors = match filter_by_patterns(
        resolve_selectors(model, &entries, action),
        filter,
        ignore,
        Source::identifier,
        |selector| whitelist.is_empty() || whitelist.contains(&selector.name()),
    ) {
        | Ok(value) => value,
        | Err(why) => {
            error!("=> {} Filter model selectors — {why}", Label::fail());
            return Err(why);
        }
    };
    if offline && selectors.iter().any(Source::is_remote) {
        let remote_names: Vec<String> = selectors.iter().filter(|source| source.is_remote()).map(Source::identifier).collect();
        error!(
            "=> {} Offline mode cannot download remote models: {}",
            Label::fail(),
            remote_names.join(", ")
        );
        Err(eyre!("Offline mode cannot download remote models: {}", remote_names.join(", ")))
    } else if selectors.is_empty() {
        let msg = if selectors.is_empty() {
            "No models specified — provide model IDs or --config"
        } else if whitelist.is_empty() {
            "No models matched --filter/--ignore"
        } else {
            "No models matched --whitelist"
        };
        error!("=> {} {msg}", Label::fail());
        Err(eyre!(msg))
    } else {
        match output.clone().map_or_else(|| home_directory(".models"), Ok) {
            | Ok(root) => {
                let mut success_count = 0usize;
                let mut failure_count = 0usize;
                for selector in &selectors {
                    let options = CommandOptions::init()
                        .selector(selector.clone())
                        .maybe_action(match selector {
                            | Source::Local { action, .. } => *action,
                            | _ => None,
                        })
                        .output(root.clone())
                        .maybe_database_path(database_path.clone())
                        .quiet(quiet)
                        .maybe_filter(regex_join(filter))
                        .maybe_ignore(regex_join(ignore))
                        .build();
                    match resolve_model(options).await {
                        | Ok(_) => success_count = success_count.saturating_add(1),
                        | Err(why) => {
                            failure_count = failure_count.saturating_add(1);
                            error!("=> {} Download model '{}' — {why}", Label::fail(), selector.name());
                        }
                    }
                }
                if !quiet {
                    print_models(&selectors);
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
pub(crate) struct FileSelectionPolicy<'a> {
    pub(crate) extension: &'a str,
    pub(crate) preferred_marker: &'a str,
    pub(crate) no_match_message: &'a str,
}
/// Resolve a single model selector
async fn resolve_model(options: CommandOptions) -> ApiResult<()> {
    match options.selector.as_ref() {
        | Some(Source::Local { path, .. }) if !path.exists() => {
            error!("=> {} Local model path does not exist: {}", Label::fail(), path.display());
            Err(eyre!("Local model path does not exist: {}", path.display()))
        }
        | Some(Source::Local { .. }) => resolve_local_model(options),
        | Some(Source::Remote { identifier, .. })
            if Uri::parse(identifier.as_str())
                .ok()
                .is_some_and(|uri| matches!(uri.scheme().as_str(), "http" | "https")) =>
        {
            resolve_direct_url_model(options).await
        }
        | Some(Source::Remote { .. }) => resolve_huggingface_model(options).await,
        | Some(Source::Unsupported(value)) => Err(eyre!("Unsupported model source — {value}")),
        | None => Err(eyre!("Model selector is not configured")),
    }
}
fn resolve_local_model(options: CommandOptions) -> ApiResult<()> {
    match options.selector.clone() {
        | Some(Source::Local { .. }) => {
            let name = options.selector.as_ref().map(Source::name).unwrap_or_else(|| "model".to_string());
            let destination = match options.output.clone() {
                | Some(root) => Ok(root.join(&name)),
                | None => Err(eyre!("Model output directory is not configured")),
            };
            match destination {
                | Ok(destination) => match create_dir_all(&destination) {
                    | Ok(_) => {
                        if !options.quiet {
                            info!(
                                "=> {} Resolved local model '{}' → {}",
                                Label::run(),
                                name.cyan(),
                                destination.display().to_string().dimmed()
                            );
                        }
                        match options.selector.clone() {
                            | Some(Source::Local { path, .. }) => match options.action.unwrap_or_default() {
                                | SourceAction::Reference => {
                                    if !options.quiet {
                                        info!(
                                            "=> {} Local model '{}' referenced in place at {}",
                                            Label::pass(),
                                            options.selector.as_ref().map(Source::name).unwrap_or_else(|| "model".to_string()),
                                            path.display()
                                        );
                                    }
                                    Ok(())
                                }
                                | SourceAction::Copy | SourceAction::Symlink => match path.file_name().map(|value| value.to_os_string()) {
                                    | Some(file_name) => materialize_local_file(options, path, destination.join(file_name)),
                                    | None => Err(eyre!("Local model path has no file name")),
                                },
                            },
                            | Some(_) => Err(eyre!("Model selector is not local")),
                            | None => Err(eyre!("Model selector is not configured")),
                        }
                    }
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
fn materialize_local_file(options: CommandOptions, path: PathBuf, target: PathBuf) -> ApiResult<()> {
    match options.action.unwrap_or_default() {
        | SourceAction::Copy if target.exists() => {
            if !options.quiet {
                info!(
                    "=> {} Skipping copy — '{}' already exists at {}",
                    Label::pass(),
                    options.selector.as_ref().map(Source::name).unwrap_or_else(|| "model".to_string()),
                    target.display()
                );
            }
            Ok(())
        }
        | SourceAction::Copy => match copy(&path, &target) {
            | Err(why) => {
                error!(
                    "=> {} Copy local model '{}' to {} — {why}",
                    Label::fail(),
                    options.selector.as_ref().map(Source::name).unwrap_or_else(|| "model".to_string()),
                    target.display()
                );
                Err(why.into())
            }
            | Ok(_) => {
                if !options.quiet {
                    info!(
                        "=> {} Copied local model '{}' → {}",
                        Label::pass(),
                        options.selector.as_ref().map(Source::name).unwrap_or_else(|| "model".to_string()),
                        target.display()
                    );
                }
                Ok(())
            }
        },
        | SourceAction::Symlink if target.exists() || target.symlink_metadata().map(|m| m.is_symlink()).unwrap_or(false) => {
            if !options.quiet {
                info!(
                    "=> {} Skipping symlink — '{}' already exists at {}",
                    Label::pass(),
                    options.selector.as_ref().map(Source::name).unwrap_or_else(|| "model".to_string()),
                    target.display()
                );
            }
            Ok(())
        }
        | SourceAction::Symlink => match symlink(&path, &target) {
            | Err(why) => {
                error!(
                    "=> {} Symlink local model '{}' to {} — {why}",
                    Label::fail(),
                    options.selector.as_ref().map(Source::name).unwrap_or_else(|| "model".to_string()),
                    target.display()
                );
                Err(why)
            }
            | Ok(_) => {
                if !options.quiet {
                    info!(
                        "=> {} Symlinked local model '{}' → {}",
                        Label::pass(),
                        options.selector.as_ref().map(Source::name).unwrap_or_else(|| "model".to_string()),
                        target.display()
                    );
                }
                Ok(())
            }
        },
        | SourceAction::Reference => Ok(()),
    }
}
async fn resolve_direct_url_model(options: CommandOptions) -> ApiResult<()> {
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
            };
            DownloadItems::new(&destination, vec![item], options.quiet).download().await
        }
        | (_, None) => Err(eyre!("Model output directory is not configured")),
        | (None, _) => Err(eyre!("Model selector is not configured")),
        | _ => Err(eyre!("Model selector is not remote")),
    }
}
async fn resolve_huggingface_model(options: CommandOptions) -> ApiResult<()> {
    match (options.selector.clone(), options.output.clone()) {
        | (Some(Source::Remote { identifier, .. }), Some(root)) => {
            let destination = root.join(&identifier);
            match create_dir_all(&destination) {
                | Ok(_) => {
                    if !options.quiet {
                        info!(
                            "=> {} Resolved model '{}' → {}",
                            Label::run(),
                            identifier.cyan(),
                            destination.display().to_string().dimmed()
                        );
                        let model_row = ModelRow::init().maybe_model_id(Some(identifier.to_string())).build();
                        match model_row.select(options.database_path.clone(), |r| r.model_id.as_deref() == Some(identifier.as_str())) {
                            | Ok(Some(row)) => info!(
                                "=> {} Validated model '{identifier}' against local database (name: {})",
                                Label::pass(),
                                row.name.as_deref().unwrap_or("unknown")
                            ),
                            | Ok(None) => info!("=> {} Model '{identifier}' not in local database — proceeding anyway", Label::CAUTION,),
                            | Err(why) => info!("=> {} Model '{identifier}' database lookup failed — {why}", Label::CAUTION,),
                        }
                        info!("=> {} Listing Hugging Face files for {}", Label::run(), identifier.cyan());
                    }
                    match huggingface::repository_tree(&identifier, "main").await {
                        | Ok(files) => match select_huggingface_files(
                            files,
                            &options.filter.clone().into_iter().collect::<Vec<_>>(),
                            &options.ignore.clone().into_iter().collect::<Vec<_>>(),
                        ) {
                            | Ok(selected) => {
                                let items = selected
                                    .into_iter()
                                    .map(|file| DownloadItem {
                                        url: format!("https://huggingface.co/{identifier}/resolve/main/{}", file.path),
                                        path: file.path,
                                        size: file.size,
                                    })
                                    .collect::<Vec<_>>();
                                DownloadItems::new(&destination, items, options.quiet).download().await
                            }
                            | Err(why) => Err(why),
                        },
                        | Err(why) => Err(why),
                    }
                }
                | Err(why) => {
                    error!("=> {} Create output directory for '{identifier}' — {why}", Label::fail());
                    Err(why.into())
                }
            }
        }
        | (_, None) => Err(eyre!("Model output directory is not configured")),
        | (None, _) => Err(eyre!("Model selector is not configured")),
        | _ => Err(eyre!("Model selector is not remote")),
    }
}
/// Merge positional selectors with config selectors (positional first, deduplicated)
fn resolve_selectors(positional: &[String], entries: &[ModelEntry], action: Option<SourceAction>) -> Vec<Source> {
    positional
        .iter()
        .map(|s| Source::from(s.as_str()))
        .chain(entries.iter().filter_map(|entry| match entry {
            | ModelEntry::Selector(s) => (!s.trim().is_empty()).then(|| Source::from(s.trim())),
            | ModelEntry::Entry(ModelEntryOptions { copy, name, source, symlink }) => {
                let config_action = match (&copy, &symlink) {
                    | (Some(true), _) => Some(SourceAction::Copy),
                    | (_, Some(true)) => Some(SourceAction::Symlink),
                    | _ => None,
                };
                Some(Source::from(source).with_name(name.as_str()).with_action(config_action))
            }
        }))
        .fold((HashSet::new(), Vec::new()), |(mut seen, mut selectors), selector| {
            let selector = match action {
                | Some(value) => selector.with_action(Some(value)),
                | None => selector,
            };
            if seen.insert(selector.identifier()) {
                selectors.push(selector);
            }
            (seen, selectors)
        })
        .1
}
fn select_huggingface_files(files: Vec<RepositoryFile>, filter: &[String], ignore: &[String]) -> ApiResult<Vec<RepositoryFile>> {
    match filter_by_patterns(files, filter, ignore, |file| file.path.clone(), |_| true) {
        | Ok(filtered) => {
            if !filter.is_empty() {
                match filtered.is_empty() {
                    | true => Err(eyre!("No model files matched --filter/--ignore")),
                    | false => Ok(filtered),
                }
            } else {
                let policy = FileSelectionPolicy {
                    extension: ".gguf",
                    preferred_marker: "Q4_K_M",
                    no_match_message: "No GGUF model files found; use a GGUF repository or provide --filter for another format",
                };
                select_preferred_file(
                    filtered
                        .into_iter()
                        .filter(|file| file.path.to_ascii_lowercase().ends_with(policy.extension))
                        .collect::<Vec<_>>(),
                    &policy,
                )
            }
        }
        | Err(why) => Err(why),
    }
}
pub(crate) fn filter_by_patterns<T>(
    items: Vec<T>,
    filter: &[String],
    ignore: &[String],
    value: impl Fn(&T) -> String,
    keep: impl Fn(&T) -> bool,
) -> ApiResult<Vec<T>> {
    match FilterSet::compile(ignore, filter) {
        | Ok(FilterSet { ignore, filter }) => Ok(items
            .into_iter()
            .filter(|item| {
                let value = value(item);
                let ignored = ignore.iter().any(|pattern| pattern.is_match(&value).unwrap_or(false));
                let filtered = filter.is_empty() || filter.iter().any(|pattern| pattern.is_match(&value).unwrap_or(false));
                !ignored && filtered && keep(item)
            })
            .collect()),
        | Err(why) => Err(why),
    }
}
pub(crate) fn select_preferred_file(files: Vec<RepositoryFile>, policy: &FileSelectionPolicy<'_>) -> ApiResult<Vec<RepositoryFile>> {
    match files.len() {
        | 0 => Err(eyre!("{}", policy.no_match_message)),
        | 1 => Ok(files),
        | _ => {
            let preferred = files
                .iter()
                .filter(|file| file.path.to_ascii_uppercase().contains(policy.preferred_marker))
                .cloned()
                .collect::<Vec<_>>();
            match preferred.len() {
                | 1 => Ok(preferred),
                | _ => Err(eyre!(
                    "Multiple model file candidates found; use --filter to choose one: {}",
                    files.iter().map(|file| file.path.clone()).collect::<Vec<_>>().join(", ")
                )),
            }
        }
    }
}
/// Print the list of models to be downloaded.
fn print_models(selectors: &[Source]) {
    let local_count = selectors.iter().filter(|source| source.is_local()).count();
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
    for selector in selectors {
        match selector {
            | Source::Local { path, .. } => {
                info!("  {} {} ({})", "→".cyan(), selector.name().green(), path.display().to_string().dimmed());
            }
            | Source::Remote { identifier, .. } => {
                let name = selector.name();
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
