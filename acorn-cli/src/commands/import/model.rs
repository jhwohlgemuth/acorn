use crate::cli::arguments::SyncTarget;
use crate::cli::Void;
#[cfg(feature = "tui")]
use crate::commands::download::model::plan::run_gguf_picker;
use crate::commands::download::model::plan::{resolve_plans, ModelDownloadPlan};
use acorn::io::api::huggingface;
use acorn::io::config::{ApplicationConfiguration, ModelEntry};
use acorn::io::database::schema::Table;
use acorn::io::database::{Database, Operations, PersistStatus};
use acorn::io::{sync, ApiResult, Source};
use acorn::prelude::{PathBuf, String, Vec};
use acorn::schema::agent::{ModelSelectors, Weights};
use acorn::util::{suffix, Label};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::eyre;
use futures::future::join_all;
use itertools::Itertools;
use owo_colors::OwoColorize;
use tracing::{error, info, warn};

/// Import model metadata into the local database
#[allow(clippy::too_many_arguments)]
pub async fn run(
    model: &[String],
    model_file: &Option<String>,
    sync_target: &Option<SyncTarget>,
    force: bool,
    dry_run: bool,
    config: &Option<PathBuf>,
    database_path: &Option<PathBuf>,
    no_local_database: bool,
    offline: bool,
    no_fallback: bool,
    search_limit: usize,
    minimum_download_count: u64,
    interactive: bool,
    verbose: &Verbosity,
) -> Void {
    let quiet = verbose.is_silent();
    let persistence_disabled = no_local_database || dry_run;
    match ModelSelectors::from(model).resolve(model_file, offline).await {
        | Ok(model) => match (model.is_empty() && config.is_none(), persistence_disabled) {
            | (true, true) => {
                if !quiet {
                    println!(
                        "=> {} Model catalog persistence {}",
                        Label::skip(),
                        if dry_run { "(dry run)" } else { "(local database is disabled)" }.dimmed()
                    );
                }
                Ok(())
            }
            | (true, false) => match Database::<Table>::from_path(database_path.clone()).populate(Table::Models).await {
                | Ok(PersistStatus::Downloaded(count)) => {
                    if !quiet {
                        println!("=> {} Imported {count} model records", Label::pass());
                    }
                    Ok(())
                }
                | Ok(PersistStatus::AlreadyExists) => {
                    if !quiet {
                        println!("=> {} Model records already exist in the local database", Label::pass());
                    }
                    Ok(())
                }
                | Err(why) => Err(why),
            },
            | (false, _) => match ApplicationConfiguration::load(config) {
                | Err(why) => Err(why),
                | Ok(configuration) => {
                    let (configured_entries, _) = configuration.model_entries_and_whitelist();
                    let entries = match config.is_some() {
                        | true => configured_entries,
                        | false => Vec::new(),
                    };
                    match resolve_plans(&model, &entries, None, database_path, true, offline).await {
                        | Ok(plans) => {
                            let options = ModelImportOptions {
                                no_local_database: persistence_disabled,
                                offline,
                                no_fallback,
                                search_limit,
                                minimum_download_count,
                                interactive,
                                quiet,
                                database_path: database_path.clone(),
                            };
                            let results = join_all(plans.into_iter().map(|plan| plan.import(&options))).await;
                            collect_imported(results, model_file.is_some()).and_then(|imported| {
                                if !quiet {
                                    println!(
                                        "=> {} Resolved metadata for {} model{}{}",
                                        Label::pass(),
                                        imported.len(),
                                        if imported.len() == 1 { "" } else { "s" },
                                        if persistence_disabled { " (persistence skipped)" } else { "" }
                                    );
                                }
                                sync_target.as_ref().map_or(Ok(()), |target| {
                                    let entries = imported.into_iter().unique().map(ModelEntry::Selector).collect::<Vec<_>>();
                                    let options = sync::Options::init()
                                        .entries(&entries)
                                        .opencode(target.is_opencode())
                                        .llama_swap(target.is_llama_swap())
                                        .force(force)
                                        .dry_run(dry_run)
                                        .build();
                                    configuration.sync_and_update(config, options)
                                })
                            })
                        }
                        | Err(why) => Err(why),
                    }
                }
            },
        },
        | Err(why) => {
            error!("=> {} Read model list file — {why}", Label::fail());
            Err(why)
        }
    }
}
struct ModelImportOptions {
    no_local_database: bool,
    offline: bool,
    no_fallback: bool,
    search_limit: usize,
    minimum_download_count: u64,
    interactive: bool,
    quiet: bool,
    database_path: Option<PathBuf>,
}
impl ModelDownloadPlan {
    async fn import(self, options: &ModelImportOptions) -> ApiResult<String> {
        let is_huggingface_repository = match &self.selector {
            | Source::Remote { identifier, .. } => !identifier.starts_with("http://") && !identifier.starts_with("https://"),
            | _ => false,
        };
        match &self.selector {
            | Source::Remote { identifier, .. } if is_huggingface_repository && !options.offline => {
                let huggingface_options = huggingface::Options::init()
                    .identifier(identifier)
                    .revision(self.revision.clone())
                    .interactive(options.interactive)
                    .no_fallback(options.no_fallback)
                    .quiet(options.quiet)
                    .search_limit(options.search_limit)
                    .minimum_download_count(options.minimum_download_count)
                    .build();
                #[cfg(feature = "tui")]
                let huggingface_options = huggingface::Options {
                    selector: run_gguf_picker,
                    ..huggingface_options
                };
                match huggingface::HuggingFaceRepositoryFiles::resolve(&huggingface_options).await {
                    | Ok(resolution) => {
                        let (requested, resolved, weights) = resolution.map(Weights::from).into_parts();
                        if weights.0.is_empty() {
                            Err(eyre!("No GGUF model files found for '{requested}'"))
                        } else if options.no_local_database {
                            if !options.quiet {
                                let count = weights.0.len();
                                info!(
                                    "=> {} Resolved {count} GGUF file{} for '{requested}' without database persistence",
                                    Label::run(),
                                    suffix(count),
                                );
                            }
                            Ok(resolved)
                        } else {
                            weights.persist(&requested, options.database_path.clone()).map(|_| resolved)
                        }
                    }
                    | Err(why) => Err(why),
                }
            }
            | Source::Remote { identifier, .. } if options.offline => Err(eyre!("Offline mode cannot import remote model metadata: {identifier}")),
            | _ => Err(eyre!("Model metadata import supports only Hugging Face repository identifiers")),
        }
    }
}
pub(super) fn collect_imported(results: Vec<ApiResult<String>>, best_effort: bool) -> ApiResult<Vec<String>> {
    match best_effort {
        | true => Ok(results
            .into_iter()
            .filter_map(|result| match result {
                | Ok(identifier) => Some(identifier),
                | Err(why) => {
                    let reason = format!("({why})");
                    warn!("=> {} Could not import model metadata {}", Label::skip(), reason.dimmed());
                    None
                }
            })
            .collect()),
        | false => results.into_iter().collect(),
    }
}
