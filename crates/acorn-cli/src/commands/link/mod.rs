use crate::cli::{resolve_paths, CommandOptions};
#[cfg(feature = "exec")]
use acorn::exec::config::ExecutionOrigin;
use acorn::io::config::application::ApplicationConfiguration;
#[cfg(not(feature = "exec"))]
use acorn::io::config::application::Hook;
#[cfg(feature = "exec")]
use acorn::io::workflow::WorkflowAction;
use acorn::io::workflow::{self, Workflow, WorkflowExtension};
use acorn_core::options::CommonRuntime;
use acorn_core::Location;
use acorn_schema::research_activity::ResearchActivity;
use clap_verbosity_flag::Verbosity;
#[cfg(not(feature = "exec"))]
use color_eyre::eyre::eyre;
use color_eyre::eyre::{Report, Result};
use std::path::PathBuf;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    ignore: &Option<String>,
    config: &Option<PathBuf>,
    dry_run: bool,
    merge_request: bool,
    verbose: &Verbosity,
) -> Result<(), Report> {
    let options = CommandOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .maybe_ignore(ignore.clone())
        .merge_request(merge_request)
        .build();

    match ApplicationConfiguration::load_with_path(config) {
        | Err(why) => Err(why),
        | Ok((configuration, resolved)) => {
            #[cfg(feature = "exec")]
            let supported = Ok::<(), Report>(());
            #[cfg(not(feature = "exec"))]
            let supported = match config.is_some() || configuration.has_hook(Hook::Link) {
                | true => Err(eyre!("configured Rhai link hooks require the `exec` feature")),
                | false => Ok(()),
            };
            let root = ApplicationConfiguration::root(&resolved);
            match supported {
                | Err(why) => Err(why),
                | Ok(()) => match resolve_paths(path, &options).await {
                    | Err(why) => Err(why),
                    | Ok(paths) => {
                        let sources = paths.iter().map(|path| Location::from(&path.display().to_string())).collect::<Vec<_>>();
                        let workflow_options = workflow::Options::init()
                            .common(CommonRuntime {
                                dry_run,
                                ..CommonRuntime::default()
                            })
                            .extension(WorkflowExtension {
                                check: true,
                                format: true,
                                link: true,
                                show_changes: verbose.log_level().is_some(),
                                ..WorkflowExtension::default()
                            })
                            .build();
                        let apply_options = workflow_options.clone();
                        Workflow::<ResearchActivity>::plan(&sources, workflow_options, None)
                            .await
                            .map(|plan| plan.without_source_changes())
                            .and_then(|plan| {
                                #[cfg(feature = "exec")]
                                {
                                    plan.apply(
                                        apply_options,
                                        &configuration,
                                        &root,
                                        &[WorkflowAction::Link],
                                        ExecutionOrigin::Local,
                                        config.is_some(),
                                    )
                                }
                                #[cfg(not(feature = "exec"))]
                                {
                                    let _ = (configuration, root);
                                    plan.apply_enriched(apply_options)
                                }
                            })
                    }
                },
            }
        }
    }
}
#[cfg(test)]
mod tests;
