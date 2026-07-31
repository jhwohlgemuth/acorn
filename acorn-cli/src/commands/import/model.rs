use crate::cli::Void;
use crate::commands::download::model::configured_models;
use crate::commands::download::model::plan::{resolve_plans, ModelDownloadPlan};
use acorn::io::api::huggingface;
use acorn::io::config::ApplicationConfiguration;
use acorn::io::database::schema::Table;
use acorn::io::database::{Database, Operations, PersistStatus};
use acorn::io::{ApiResult, Source};
use acorn::prelude::{PathBuf, String, Vec};
use acorn::schema::agent::Weights;
use acorn::util::Label;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::eyre;
use futures::future::join_all;
use tracing::info;

/// Import model metadata into the local database.
#[allow(clippy::too_many_arguments)]
pub async fn run(
    model: &[String],
    config: &Option<PathBuf>,
    database_path: &Option<PathBuf>,
    no_local_database: bool,
    offline: bool,
    no_fallback: bool,
    search_limit: usize,
    interactive: bool,
    verbose: &Verbosity,
) -> Void {
    let quiet = verbose.is_silent();
    match (model.is_empty() && config.is_none(), no_local_database) {
        | (true, true) => {
            if !quiet {
                println!(
                    "=> {} Model catalog persistence skipped because the local database is disabled",
                    Label::CAUTION
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
                let (configured_entries, _) = configured_models(Some(configuration));
                let entries = match config.is_some() {
                    | true => configured_entries,
                    | false => Vec::new(),
                };
                match resolve_plans(model, &entries, None, database_path, true, offline).await {
                    | Ok(plans) => {
                        let options = ModelImportOptions {
                            no_local_database,
                            offline,
                            no_fallback,
                            search_limit,
                            interactive,
                            quiet,
                            database_path: database_path.clone(),
                        };
                        let results = join_all(plans.into_iter().map(|plan| plan.import(&options))).await;
                        results.into_iter().collect::<ApiResult<Vec<_>>>().map(|imported| {
                            if !quiet {
                                println!(
                                    "=> {} Resolved metadata for {} model{}{}",
                                    Label::pass(),
                                    imported.len(),
                                    if imported.len() == 1 { "" } else { "s" },
                                    if no_local_database { " (persistence skipped)" } else { "" }
                                );
                            }
                        })
                    }
                    | Err(why) => Err(why),
                }
            }
        },
    }
}
struct ModelImportOptions {
    no_local_database: bool,
    offline: bool,
    no_fallback: bool,
    search_limit: usize,
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
                match huggingface::repository_tree(identifier, &self.revision).await {
                    | Ok(files) => {
                        let huggingface_options = huggingface::Options::init()
                            .identifier(identifier)
                            .interactive(options.interactive)
                            .no_fallback(options.no_fallback)
                            .quiet(options.quiet)
                            .search_limit(options.search_limit)
                            .build();
                        #[cfg(feature = "tui")]
                        let huggingface_options = huggingface::Options {
                            selector: crate::commands::download::model::plan::run_gguf_picker,
                            ..huggingface_options
                        };
                        match files.resolve(&huggingface_options).await {
                            | Ok(repository) => {
                                let weights = Weights::from(repository);
                                if weights.0.is_empty() {
                                    Err(eyre!("No GGUF model files found for '{identifier}'"))
                                } else if options.no_local_database {
                                    if !options.quiet {
                                        info!(
                                            "=> {} Resolved {} GGUF file{} for '{}' without database persistence",
                                            Label::run(),
                                            weights.0.len(),
                                            if weights.0.len() == 1 { "" } else { "s" },
                                            identifier
                                        );
                                    }
                                    Ok(identifier.to_string())
                                } else {
                                    weights.persist(identifier, options.database_path.clone()).map(|_| identifier.to_string())
                                }
                            }
                            | Err(why) => Err(why),
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
