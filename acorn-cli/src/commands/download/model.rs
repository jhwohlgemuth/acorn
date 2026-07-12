use crate::cli::Void;
use acorn::io::config::{ApplicationConfiguration, ModelEntry, ModelEntryOptions};
use acorn::io::database::schema::ModelRow;
use acorn::io::database::Row;
use acorn::io::{home_directory, symlink, ApiResult, InputOutput, Source, SourceAction};
use acorn::prelude::{copy, create_dir_all, HashSet, Path, PathBuf};
use acorn::util::{suffix, Label, StringConversion};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::eyre;
use owo_colors::OwoColorize;
use tracing::{error, info};

/// Download model weights for use with acorn research harnesses and local inference
#[allow(clippy::too_many_arguments)]
pub async fn run(
    model: &[String],
    _filter: &[String],
    _ignore: &[String],
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
    let cli_action = SourceAction::from_options(copy, symlink);
    let config_entries = ApplicationConfiguration::resolve(config)
        .and_then(|path| ApplicationConfiguration::read(path).ok())
        .and_then(|content| content.models)
        .unwrap_or_default();
    let whitelist = whitelist
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<HashSet<_>>();
    let selectors = resolve_selectors(model, &config_entries, cli_action)
        .into_iter()
        .filter(|selector| whitelist.is_empty() || whitelist.contains(&selector.name()))
        .collect::<Vec<_>>();
    let has_remote = selectors.iter().any(Source::is_remote);
    if offline && has_remote {
        let remote_names: Vec<String> = selectors.iter().filter(|source| source.is_remote()).map(Source::identifier).collect();
        error!(
            "=> {} Offline mode cannot download remote models: {}",
            Label::fail(),
            remote_names.join(", ")
        );
        Err(eyre!("Offline mode cannot download remote models: {}", remote_names.join(", ")))
    } else if selectors.is_empty() {
        let msg = if whitelist.is_empty() {
            "No models specified — provide model IDs or --config"
        } else {
            "No models matched --whitelist"
        };
        error!("=> {} {msg}", Label::fail());
        Err(eyre!(msg))
    } else {
        match output.clone().map_or_else(|| home_directory(".models"), Ok) {
            | Ok(root) => {
                let (successes, failures): (Vec<_>, Vec<_>) = selectors
                    .iter()
                    .map(|selector| resolve_model(selector, &root, database_path, quiet))
                    .partition(|result| result.is_ok());
                let success_count = successes.len();
                let failure_count = failures.len();
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
/// Resolve a single model selector
fn resolve_model(selector: &Source, root: &Path, database_path: &Option<PathBuf>, quiet: bool) -> ApiResult<()> {
    match selector {
        | Source::Local { path, .. } if !path.exists() => {
            error!("=> {} Local model path does not exist: {}", Label::fail(), path.display());
            Err(eyre!("Local model path does not exist: {}", path.display()))
        }
        | Source::Local { path, action, .. } => {
            let name = selector.name();
            let action = action.unwrap_or_default();
            let destination = root.join(&name);
            match create_dir_all(&destination) {
                | Ok(_) => {
                    if !quiet {
                        info!(
                            "=> {} Resolved local model '{}' → {}",
                            Label::run(),
                            name.cyan(),
                            destination.display().to_string().dimmed()
                        );
                    }
                    match action {
                        | SourceAction::Reference => {
                            if !quiet {
                                info!("=> {} Local model '{}' referenced in place at {}", Label::pass(), name, path.display());
                            }
                            Ok(())
                        }
                        | SourceAction::Copy | SourceAction::Symlink => match path.file_name() {
                            | None => Err(eyre!("Local model path has no file name")),
                            | Some(file_name) => {
                                let target = destination.join(file_name);
                                match action {
                                    | SourceAction::Copy if target.exists() => {
                                        if !quiet {
                                            info!("=> {} Skipping copy — '{}' already exists at {}", Label::pass(), name, target.display());
                                        }
                                        Ok(())
                                    }
                                    | SourceAction::Copy => match copy(path, &target) {
                                        | Err(why) => {
                                            error!("=> {} Copy local model '{}' to {} — {why}", Label::fail(), name, target.display());
                                            Err(why.into())
                                        }
                                        | Ok(_) => {
                                            if !quiet {
                                                info!("=> {} Copied local model '{}' → {}", Label::pass(), name, target.display());
                                            }
                                            Ok(())
                                        }
                                    },
                                    | SourceAction::Symlink
                                        if target.exists() || target.symlink_metadata().map(|m| m.is_symlink()).unwrap_or(false) =>
                                    {
                                        if !quiet {
                                            info!(
                                                "=> {} Skipping symlink — '{}' already exists at {}",
                                                Label::pass(),
                                                name,
                                                target.display()
                                            );
                                        }
                                        Ok(())
                                    }
                                    | SourceAction::Symlink => match symlink(path, &target) {
                                        | Err(why) => {
                                            error!("=> {} Symlink local model '{}' to {} — {why}", Label::fail(), name, target.display());
                                            Err(why)
                                        }
                                        | Ok(_) => {
                                            if !quiet {
                                                info!("=> {} Symlinked local model '{}' → {}", Label::pass(), name, target.display());
                                            }
                                            Ok(())
                                        }
                                    },
                                    | SourceAction::Reference => Ok(()),
                                }
                            }
                        },
                    }
                }
                | Err(why) => {
                    error!("=> {} Create output directory for '{name}' — {why}", Label::fail());
                    Err(why.into())
                }
            }
        }
        | Source::Remote { identifier, .. } => {
            let destination = root.join(identifier);
            match create_dir_all(&destination) {
                | Ok(_) => {
                    if !quiet {
                        info!(
                            "=> {} Resolved model '{}' → {}",
                            Label::run(),
                            identifier.cyan(),
                            destination.display().to_string().dimmed()
                        );
                        let model_row = ModelRow::init().maybe_model_id(Some(identifier.to_string())).build();
                        match model_row.select(database_path.clone(), |r| r.model_id.as_deref() == Some(identifier)) {
                            | Ok(Some(row)) => info!(
                                "=> {} Validated model '{identifier}' against local database (name: {})",
                                Label::pass(),
                                row.name.as_deref().unwrap_or("unknown")
                            ),
                            | Ok(None) => info!("=> {} Model '{identifier}' not in local database — proceeding anyway", Label::CAUTION,),
                            | Err(why) => info!("=> {} Model '{identifier}' database lookup failed — {why}", Label::CAUTION,),
                        }
                    }
                    Ok(())
                }
                | Err(why) => {
                    error!("=> {} Create output directory for '{identifier}' — {why}", Label::fail());
                    Err(why.into())
                }
            }
        }
        | Source::Unsupported(value) => Err(eyre!("Unsupported model source — {value}")),
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
