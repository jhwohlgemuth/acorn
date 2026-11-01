use crate::{
    cli::{resolve_paths, CommandOptions},
    commands::preflight,
    io::interactive::{PromptOutcome, Viewport},
};
#[cfg(feature = "exec")]
use acorn::exec::config::ExecutionOrigin;
#[cfg(feature = "exec")]
use acorn::io::workflow::WorkflowAction;
use acorn::{
    fail,
    io::{
        config::application::{ApplicationConfiguration, Hook},
        enrichment::{EnrichmentMetadata, Pipeline},
        slice_path,
        vocabulary::write_vocabulary_report_audits,
        workflow::{self, Change, EnrichmentSelection, Workflow, WorkflowExtension},
        FromPath, InputOutput,
    },
    util::{constants::app::DISPLAY_PATH_CHARACTER_COUNT, StringConversion},
};
use acorn_core::{options::CommonRuntime, util::MimeType, Location};
use acorn_diff::unified_diff;
use acorn_host::terminal::Label;
use acorn_schema::{
    enrichment::Provider,
    research_activity::ResearchActivity,
    standard::Standard,
    standard::{cff::Cff, datacite, dublin_core::DublinCoreDocument, invenio},
    validation::Validate,
    vocabulary::VocabularyReport,
};
use clap_verbosity_flag::log::Level;
use color_eyre::eyre::{eyre, Report, Result};
use owo_colors::OwoColorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::{read_to_string, write},
    io::{self, IsTerminal},
    path::{Path, PathBuf},
};
use strum::IntoEnumIterator;
use tracing::info;

mod acre;
struct FormatResult {
    audit: Option<VocabularyAudit>,
    feedback: String,
}
struct StandardOptions<'a> {
    command: &'a CommandOptions,
    configuration: &'a ApplicationConfiguration,
    enrich: bool,
    explicit_configuration: bool,
    path: &'a Option<PathBuf>,
    providers: &'a [Provider],
    root: &'a Path,
    standard: Standard,
}
struct VocabularyAudit {
    after: String,
    before: String,
    path: PathBuf,
    reports: Vec<VocabularyReport>,
    source: String,
}
impl StandardOptions<'_> {
    fn config_is_unsupported(&self) -> Option<Report> {
        match (self.explicit_configuration, self.configuration.has_hook(Hook::Format)) {
            #[cfg(not(feature = "exec"))]
            | (true, _) => Some(eyre!("--config requires the `exec` feature")),
            #[cfg(not(feature = "exec"))]
            | (false, true) => Some(eyre!("configured Rhai format hooks require the `exec` feature")),
            | _ => None,
        }
    }
}
fn finish_format_results(results: Vec<Result<FormatResult, Report>>) -> Result<(), Report> {
    let audits = results
        .iter()
        .filter_map(|result| result.as_ref().ok().and_then(|result| result.audit.as_ref()))
        .collect::<Vec<_>>();
    write_vocabulary_report_audits(audits.iter().map(|audit| {
        (
            audit.source.as_str(),
            audit.before.as_str(),
            audit.after.as_str(),
            audit.reports.as_slice(),
        )
    }))
    .map_err(Report::msg)
    .or_else(|audit_error| {
        audits
            .iter()
            .try_for_each(|audit| write(&audit.path, &audit.before))
            .map_err(|rollback_error| eyre!("{audit_error}; rollback failed — {rollback_error}"))
            .and_then(|()| Err(audit_error))
    })
    .and_then(|_| results.into_iter().collect::<Result<Vec<_>, _>>().map(|_| ()))
}
async fn format<T>(path: &Option<PathBuf>, options: &CommandOptions) -> Result<(), Report>
where
    T: InputOutput + Serialize,
{
    resolve_paths(path, options).await.and_then(|paths| {
        let viewport = match options.interactive {
            | true => Viewport::open().map(Some),
            | false => Ok(None),
        };
        viewport.and_then(|mut viewport| {
            viewport
                .as_mut()
                .map_or(Ok(()), |viewport| viewport.push("Formatting selected files"))
                .and_then(|()| {
                    let results = paths.par_iter().map(|path| format_file::<T>(path, options)).collect::<Vec<_>>();
                    viewport
                        .as_mut()
                        .map_or(Ok(()), |viewport| {
                            paths.iter().zip(&results).try_for_each(|(path, result)| {
                                viewport.push(match result {
                                    | Ok(result) => result.feedback.clone(),
                                    | Err(why) => format!("{}: failed — {why}", path.display()),
                                })
                            })
                        })
                        .and_then(|()| finish_format_results(results))
                })
        })
    })
}
fn format_diff(old: &str, new: &str, path: &Path, color: bool) -> String {
    let before_label = format!("a/{}", path.display());
    let after_label = format!("b/{}", path.display());
    let diff = unified_diff(old, new, &before_label, &after_label);
    match color {
        | false => diff,
        | true => diff
            .split_inclusive('\n')
            .map(|line| match line {
                | line if line.starts_with("--- ") || line.starts_with('-') => line.red().to_string(),
                | line if line.starts_with("+++ ") || line.starts_with('+') => line.green().to_string(),
                | line if line.starts_with("@@") => line.cyan().to_string(),
                | line => line.dimmed().to_string(),
            })
            .collect(),
    }
}
fn format_file<T>(path: &Path, options: &CommandOptions) -> Result<FormatResult, Report>
where
    T: InputOutput + Serialize,
{
    read_to_string(path)
        .map_err(|why| eyre!("Read original metadata — {why}"))
        .and_then(|original| T::read(path.to_path_buf()).map(|data| (data, original)))
        .map_err(|why| {
            if !options.interactive {
                fail!("Read metadata — {why}");
            }
            eyre!("Read metadata — {why}")
        })
        .and_then(|(data, original)| {
            let mime = MimeType::from_path(path);
            data.serialize_as(&mime)
                .map_err(|why| {
                    if !options.interactive {
                        fail!("Serialize original content — {why}");
                    }
                    eyre!("Serialize original content — {why}")
                })
                .and_then(|old_content| {
                    let formatted = data.format_with_vocabulary_report(options.fuzzy, Some(path.to_path_buf()));
                    formatted
                        .value
                        .serialize_as(&mime)
                        .map_err(|why| {
                            if !options.interactive {
                                fail!("Serialize formatted content — {why}");
                            }
                            eyre!("Serialize formatted content — {why}")
                        })
                        .map(|new_content| (old_content, new_content, formatted.reports, original))
                })
                .and_then(|(old_content, new_content, reports, original)| {
                    let feedback = match old_content == new_content {
                        | true => format!("No format changes for {}", path.display()),
                        | false if options.dry_run => format!(
                            "Dry run: {}\n{}",
                            path.display(),
                            format_diff(&old_content, &new_content, path, !options.no_color)
                        ),
                        | false => format!("Formatted {}", path.display()),
                    };
                    write_formatted_content(path, options, &old_content, &new_content).map(|applied| FormatResult {
                        audit: match applied {
                            | false => None,
                            | true => Some(VocabularyAudit {
                                after: new_content,
                                before: original,
                                path: path.to_path_buf(),
                                reports,
                                source: path.display().to_string(),
                            }),
                        },
                        feedback,
                    })
                })
        })
}
pub async fn run(config: &Option<PathBuf>, enrich: bool, providers: &[Provider], standard: Standard, options: &CommandOptions) -> Result<(), Report> {
    match options.interactive && !(io::stdin().is_terminal() && io::stdout().is_terminal()) {
        | true => Err(eyre!("--interactive requires terminal input and output")),
        | false => run_validated(config, enrich, providers, standard, options).await,
    }
}
async fn run_as<T>(options: StandardOptions<'_>) -> Result<(), Report>
where
    T: Clone + DeserializeOwned + EnrichmentMetadata + InputOutput + Send + Serialize + Sync + Validate + 'static,
{
    let config_is_unsupported = options.config_is_unsupported();
    let StandardOptions {
        command,
        configuration,
        enrich,
        explicit_configuration,
        path,
        providers,
        root,
        standard,
    } = options;
    let selected = match (standard, providers.is_empty()) {
        | (Standard::ResearchActivityData, true) => Ok(Provider::iter().collect()),
        | (Standard::ResearchActivityData, false) => Ok(providers.to_vec()),
        | (_, true) => Ok(vec![Provider::Orcid]),
        | (_, false) if providers.iter().all(|provider| matches!(provider, Provider::Orcid)) => Ok(providers.to_vec()),
        | _ => Err(eyre!("Only the ORCID provider supports enrichment for the {standard} standard")),
    };
    let workflow = match (config_is_unsupported, selected) {
        | (Some(why), _) | (_, Err(why)) => Err(why),
        | (None, Ok(selected)) => match enrich || configuration.has_hook(Hook::Format) || command.interactive {
            | true => {
                run_workflow::<T>(
                    command,
                    Pipeline::new(selected),
                    enrich,
                    configuration,
                    root,
                    explicit_configuration,
                    path,
                )
                .await
            }
            | false => format::<T>(path, command).await,
        },
    };
    workflow
}
async fn run_format_only<T>(options: StandardOptions<'_>) -> Result<(), Report>
where
    T: InputOutput + Serialize,
{
    let has_unsupported_hook = options.explicit_configuration || options.configuration.has_hook(Hook::Format);
    match (options.enrich, has_unsupported_hook) {
        | (true, _) => Err(eyre!("Enrichment is not supported for the {} standard", options.standard)),
        | (false, true) => Err(eyre!("Format hooks are not supported for the {} standard", options.standard)),
        | (false, false) => format::<T>(options.path, options.command).await,
    }
}
async fn run_validated(
    config: &Option<PathBuf>,
    enrich: bool,
    providers: &[Provider],
    standard: Standard,
    options: &CommandOptions,
) -> Result<(), Report> {
    preflight!(options);
    match standard {
        | Standard::ApplicationConfigurationResearchEnablement => acre::run(&options.path, options, config, enrich, providers).await,
        | _ => match enrich && options.offline {
            | true => Err(eyre!("--enrich cannot be used with --offline")),
            | false => match ApplicationConfiguration::load_with_path(config) {
                | Err(why) => Err(why),
                | Ok((configuration, resolved)) => {
                    let root = ApplicationConfiguration::root(&resolved);
                    let standard_options = StandardOptions {
                        command: options,
                        configuration: &configuration,
                        enrich,
                        explicit_configuration: config.is_some(),
                        path: &options.path,
                        providers,
                        root: &root,
                        standard,
                    };
                    match standard {
                        | Standard::CitationFileFormat => run_as::<Cff>(standard_options).await,
                        | Standard::Datacite => run_as::<datacite::Record>(standard_options).await,
                        | Standard::DublinCore => run_format_only::<DublinCoreDocument>(standard_options).await,
                        | Standard::Invenio => run_as::<invenio::Record>(standard_options).await,
                        | Standard::ResearchActivityData => run_as::<ResearchActivity>(standard_options).await,
                        | _ => Err(eyre!("Formatting is not supported for the {standard} standard")),
                    }
                }
            },
        },
    }
}
async fn run_workflow<T>(
    options: &CommandOptions,
    pipeline: Pipeline,
    enrich: bool,
    configuration: &ApplicationConfiguration,
    root: &Path,
    explicit_configuration: bool,
    path: &Option<PathBuf>,
) -> Result<(), Report>
where
    T: Clone + DeserializeOwned + EnrichmentMetadata + InputOutput + Send + Serialize + Sync + Validate + 'static,
{
    let workflow_options = workflow::Options::init()
        .common(CommonRuntime {
            dry_run: options.dry_run,
            no_color: options.no_color,
            offline: options.offline,
            threads: options.threads,
            ..CommonRuntime::default()
        })
        .extension(WorkflowExtension {
            check: false,
            enrich,
            format: true,
            fuzzy: options.fuzzy,
            show_changes: options.verbose.log_level().is_some(),
            ..WorkflowExtension::default()
        })
        .build();
    match resolve_paths(path, options).await {
        | Ok(paths) => {
            let sources = paths.iter().map(|path| Location::from(&path.display().to_string())).collect::<Vec<_>>();
            let viewport = match options.interactive {
                | true => Viewport::open().map(Some),
                | false => Ok(None),
            };
            match viewport {
                | Err(why) => Err(why),
                | Ok(mut viewport) => {
                    let planned = match viewport.as_mut() {
                        | Some(viewport) => {
                            let options = workflow_options.clone();
                            let operation = Workflow::<T>::plan(&sources, options, Some(&pipeline));
                            viewport.spin("Planning format", operation).await
                        }
                        | None => Workflow::<T>::plan(&sources, workflow_options.clone(), Some(&pipeline))
                            .await
                            .map(PromptOutcome::Selected),
                    };
                    let reviewed = planned.and_then(|outcome| match outcome {
                        | PromptOutcome::Cancelled => Err(eyre!("Interactive formatting cancelled")),
                        | PromptOutcome::Selected(mut plan) => viewport
                            .as_mut()
                            .map_or(Ok(()), |viewport| {
                                plan.changes
                                    .iter()
                                    .try_for_each(|change| match change {
                                        | Change::Document {
                                            changed,
                                            content,
                                            original,
                                            source,
                                            ..
                                        } => viewport.push(match (*changed, options.dry_run) {
                                            | (false, _) => format!("No format changes for {source}"),
                                            | (true, true) => format!(
                                                "Dry run: {source}\n{}",
                                                format_diff(original, content, &PathBuf::from(source.to_string()), !options.no_color)
                                            ),
                                            | (true, false) => format!("Planned format changes for {source}"),
                                        }),
                                        | _ => Ok(()),
                                    })
                                    .and_then(|()| {
                                        (0..plan.changes.len()).try_for_each(|index| loop {
                                            match plan.changes.get(index).and_then(|change| change.decisions().first()).cloned() {
                                                | None => break Ok(()),
                                                | Some(decision) => {
                                                    let mut choices = decision
                                                        .candidates
                                                        .iter()
                                                        .map(|candidate| {
                                                            format!("{} — {} [{}]", candidate.label, candidate.key.value, candidate.evidence_source)
                                                        })
                                                        .collect::<Vec<_>>();
                                                    choices.push("Skip — leave this field unchanged".to_string());
                                                    let prompt = decision.summary();
                                                    let outcome =
                                                        viewport.choose(&prompt, &choices, options.no_color).and_then(|outcome| match outcome {
                                                            | PromptOutcome::Cancelled => Err(eyre!("Interactive formatting cancelled")),
                                                            | PromptOutcome::Selected(selected) if selected == decision.candidates.len() => {
                                                                plan.apply_decision(index, EnrichmentSelection::Skip)
                                                            }
                                                            | PromptOutcome::Selected(selected) => decision
                                                                .candidates
                                                                .get(selected)
                                                                .ok_or_else(|| eyre!("Selected enrichment candidate is unavailable"))
                                                                .and_then(|candidate| {
                                                                    plan.apply_decision(index, EnrichmentSelection::Choose(candidate.key.clone()))
                                                                }),
                                                        });
                                                    match outcome {
                                                        | Ok(()) => {}
                                                        | Err(why) => break Err(why),
                                                    }
                                                }
                                            }
                                        })
                                    })
                            })
                            .map(|()| plan),
                    });
                    drop(viewport);
                    match reviewed {
                        #[cfg(feature = "exec")]
                        | Ok(plan) => {
                            let hooks = match enrich {
                                | true => vec![WorkflowAction::Enrich, WorkflowAction::Format],
                                | false => vec![WorkflowAction::Format],
                            };
                            plan.apply(
                                workflow_options,
                                configuration,
                                root,
                                &hooks,
                                ExecutionOrigin::Local,
                                explicit_configuration,
                            )
                        }
                        #[cfg(not(feature = "exec"))]
                        | Ok(plan) => {
                            let _ = (configuration, explicit_configuration, root);
                            plan.apply_enriched(workflow_options)
                        }
                        | Err(why) => Err(why),
                    }
                }
            }
        }
        | Err(why) => Err(why),
    }
}
fn write_formatted_content(path: &Path, options: &CommandOptions, old_content: &str, new_content: &str) -> Result<bool, Report> {
    match old_content == new_content {
        | true => {
            let display_path = slice_path(path, DISPLAY_PATH_CHARACTER_COUNT);
            if !options.interactive {
                info!("=> {} No format changes for {display_path}", Label::skip());
            }
            Ok(false)
        }
        | false => {
            let CommandOptions {
                dry_run, no_color, verbose, ..
            } = options;
            if *dry_run && !options.interactive {
                let absolute_path = path.to_path_buf().to_absolute_path();
                match no_color {
                    | true => println!("{} Format {absolute_path}", Label::DRY_RUN),
                    | false => println!("{} Format {}", Label::dry_run(), absolute_path.yellow()),
                }
            }
            let should_show_changes = matches!(
                verbose.log_level(),
                Some(Level::Warn) | Some(Level::Info) | Some(Level::Debug) | Some(Level::Trace)
            );
            if (*dry_run || should_show_changes) && !options.interactive {
                print!("{}", format_diff(old_content, new_content, path, !options.no_color));
            }
            match *dry_run {
                | true => Ok(false),
                | false => write(path, new_content.as_bytes())
                    .map_err(|why| {
                        fail!("Write — {why}");
                        eyre!("Write — {why}")
                    })
                    .map(|_| {
                        if !options.interactive {
                            let label = match no_color {
                                | true => path.display().to_string(),
                                | false => path.display().cyan().to_string(),
                            };
                            info!("=> {} Formatted {label}", Label::pass());
                        }
                        true
                    }),
            }
        }
    }
}

#[cfg(test)]
mod tests;
