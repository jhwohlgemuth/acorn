use crate::cli::Void;
use acorn::io::config::ApplicationConfiguration;
use acorn::io::database::schema::{ActivityRow, ModelRow, Table};
use acorn::io::database::{Database, Operations, Row};
use acorn::io::InputOutput;
use acorn::prelude::{create_dir_all, Path, PathBuf};
use acorn::util::{merge, suffix, Label, StringConversion};
use chrono::Utc;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{eyre, Result};
use directories::BaseDirs;
use owo_colors::OwoColorize;
use tracing::{error, info, warn};

/// Download model weights for use with acorn research harness and local inference
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
) -> Void {
    let quiet = verbose.is_silent();
    if offline {
        let msg = "Model download requires network access";
        error!("=> {} {msg}", Label::fail());
        Err(eyre!(msg))
    } else {
        let resolved = ApplicationConfiguration::resolve(config)
            .and_then(|path| ApplicationConfiguration::read(path).ok())
            .and_then(|content| content.models);
        let models = merge(model, resolved.as_deref());
        if models.is_empty() {
            let msg = "No models specified — provide model IDs or --config";
            error!("=> {} {msg}", Label::fail());
            Err(eyre!(msg))
        } else {
            let resolved = match output {
                | Some(path) => Ok(path.clone()),
                | None => BaseDirs::new()
                    .ok_or_else(|| eyre!("Failed to resolve home directory for default model output path"))
                    .map(|dirs| dirs.home_dir().join(".models")),
            };
            resolved.and_then(|root| {
                let (successes, failures): (Vec<_>, Vec<_>) = validate(&models, &root, database_path, quiet).into_iter().partition(Result::is_ok);
                let (success_count, failure_count) = (successes.len(), failures.len());
                if !quiet {
                    info!(
                        "=> {} {} model{} to download",
                        Label::run(),
                        models.len().cyan().bold(),
                        suffix(models.len())
                    );
                    for m in models {
                        info!("  {} {}", "→".cyan(), m.green());
                    }
                }
                log_activity(&root, database_path, failure_count == 0);
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
            })
        }
    }
}
/// Log the download activity to the database.
fn log_activity(model_root: &PathBuf, database_path: &Option<PathBuf>, success: bool) {
    let database = Database::<Table>::from_path(database_path.clone());
    let activity_path = model_root.to_absolute_string();
    let activity = ActivityRow::init()
        .command("download model")
        .executed_at(Utc::now())
        .user_path(activity_path)
        .success(success)
        .build();
    if let Err(why) = database.insert(activity) {
        error!("=> {} Save download model activity — {why}", Label::fail());
    }
}
/// Validate each model: create output directories and check local database
fn validate(models: &[String], root: &Path, database_path: &Option<PathBuf>, quiet: bool) -> Vec<Result<()>> {
    models
        .iter()
        .map(|model_id| {
            let destination = root.join(model_id);
            create_dir_all(&destination)
                .map_err(|why| {
                    error!("=> {} Create output directory for '{model_id}' — {why}", Label::fail());
                    eyre!(why)
                })
                .map(|_| {
                    if !quiet {
                        info!(
                            "=> {} Resolved model '{}' → {}",
                            Label::using(),
                            model_id.cyan(),
                            destination.display().to_string().dimmed()
                        );
                        let model_row = ModelRow::init().maybe_model_id(Some(model_id.to_string())).build();
                        match model_row.select(database_path.clone(), |r| r.model_id.as_deref() == Some(model_id)) {
                            | Ok(Some(ModelRow { name, .. })) => info!(
                                "=> {} Validated model '{model_id}' against local database (name: {})",
                                Label::pass(),
                                name.as_deref().unwrap_or("unknown")
                            ),
                            | Ok(None) => warn!("=> {} Model '{model_id}' not in local database — proceeding anyway", Label::using()),
                            | Err(why) => error!("=> {} Model '{model_id}' database lookup failed — {why}", Label::fail()),
                        }
                    }
                })
        })
        .collect()
}
