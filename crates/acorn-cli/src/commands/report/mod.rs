//! Scheduled bounded artifact creation and immutable package publication.
use crate::{cli::arguments::Report, commands::export::updates};
use acorn::io::{
    api::{
        gitlab::{self, Package, PackageLifecycle, PackageOptions, ReportManifest, ReportStatus},
        powerautomate::{PowerAutomateCallbackResolver, PowerAutomateConfig},
        webhooks::{
            store::{CallbackWorker, OperationQueue},
            ArtifactEvent, WebhookRuntime,
        },
        Configuration,
    },
    config::application::{ApplicationConfiguration, Folder, FolderRegistry},
    http::ReqwestHttpService,
    ApiResult, InputOutput,
};
use acorn_host::fs::SafePath;
use acorn_schema::{
    research_activity::{LogbookUpdates, ReportingWindow, ResearchActivity},
    validation::Validate,
};
use bon::Builder;
use color_eyre::eyre::eyre;
use jiff::SignedDuration;
use serde::Serialize;
use std::fs::{create_dir_all, read, write};

/// Machine-readable outcome for one configured project.
#[derive(Builder, Clone, Debug, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectReportResult {
    error: Option<String>,
    project_id: String,
    status: String,
    through: String,
    version: Option<String>,
}
async fn deliver_callbacks(queue: &OperationQueue, config: &PowerAutomateConfig, routing: &FolderRegistry) -> ApiResult<()> {
    let callback_worker = CallbackWorker::new(
        queue.clone(),
        ReqwestHttpService::default(),
        PowerAutomateCallbackResolver::new(config.callback_destination.clone(), routing),
        SignedDuration::from_mins(5),
    );
    let worker = callback_worker.with_runtime(WebhookRuntime::default());
    loop {
        match worker.run_once().await {
            | Ok(true) => {}
            | Ok(false) => break Ok(()),
            | Err(why) => break Err(why),
        }
    }
}
fn enqueue_manifest_event(queue: &OperationQueue, config: &PowerAutomateConfig, project_id: &str, manifest: &ReportManifest) -> ApiResult<()> {
    let event = match manifest.status {
        | ReportStatus::Published => ArtifactEvent::ArtifactReady {
            checksum: manifest.checksum.clone().unwrap_or_default(),
            package_url: manifest.authenticated_url.clone(),
            project_id: project_id.to_string(),
            through: manifest.through.clone(),
        },
        | ReportStatus::NoChanges => ArtifactEvent::ReportNoChanges {
            project_id: project_id.to_string(),
            through: manifest.through.clone(),
        },
    };
    event.enqueue(queue, &config.callback_destination, &manifest.version).map(|_| ())
}
fn prepare(arguments: &Report, offline: bool) -> ApiResult<(PowerAutomateConfig, FolderRegistry, String, Vec<Folder>)> {
    ApplicationConfiguration::load(&arguments.config)
        .and_then(|configuration| {
            configuration
                .validate()
                .map_err(|why| eyre!("Invalid application configuration — {why}"))
                .map(|()| configuration)
        })
        .and_then(|configuration| {
            let routing = configuration.registry();
            configuration
                .power_automate()
                .cloned()
                .map(|automation| (automation, routing))
                .ok_or_else(|| eyre!("Automated artifact configuration is required"))
        })
        .and_then(|(automation, routing)| {
            arguments
                .commit
                .as_deref()
                .filter(|commit| !commit.trim().is_empty())
                .ok_or_else(|| eyre!("Report source commit is required"))
                .map(str::to_string)
                .map(|commit| (automation, routing, commit))
        })
        .and_then(|(automation, routing, commit)| match offline && !arguments.dry_run {
            | true => Err(eyre!("Report publication cannot run offline unless --dry-run is used")),
            | false => Ok((automation, routing, commit)),
        })
        .and_then(|(automation, routing, commit)| {
            let pipeline = arguments.pipeline_id.as_deref().is_some_and(|value| !value.trim().is_empty());
            let job = arguments.job_id.as_deref().is_some_and(|value| !value.trim().is_empty());
            match arguments.dry_run || pipeline && job {
                | true => Ok((automation, routing, commit)),
                | false => Err(eyre!("Report publication requires GitLab pipeline and job identifiers")),
            }
        })
        .and_then(|(automation, routing, commit)| {
            let selected = routing
                .folders()
                .filter(|folder| arguments.projects.is_empty() || arguments.projects.contains(&folder.id))
                .cloned()
                .collect::<Vec<_>>();
            match selected.is_empty() {
                | true => Err(eyre!("No configured automated-artifact folders matched the report selection")),
                | false => Ok((automation, routing, commit, selected)),
            }
        })
}
fn prepare_project(
    arguments: &Report,
    commit: &str,
    project_id: &str,
    relative_path: &str,
    from: Option<&str>,
) -> ApiResult<(ReportingWindow, LogbookUpdates, std::path::PathBuf)> {
    ReportingWindow::new(from, &arguments.through)
        .map_err(|why| eyre!(why))
        .and_then(|window| {
            SafePath::new(relative_path)
                .map_err(|why| eyre!(why))
                .and_then(|path| path.materialize_under(&arguments.root).map_err(Into::into))
                .and_then(|path| ResearchActivity::read(path).map_err(|why| eyre!("Read report source for project '{project_id}' — {why}")))
                .and_then(|activity| activity.updates_within(&window).map_err(|why| eyre!(why)))
                .map(|updates| (window, updates))
        })
        .and_then(|(window, updates)| {
            SafePath::new(project_id)
                .map_err(|why| eyre!(why))
                .and_then(|project| project.materialize_under(&arguments.output).map_err(Into::into))
                .map(|project_output| {
                    let commit = commit.chars().filter(char::is_ascii_hexdigit).take(12).collect::<String>();
                    let through = window.right.replace([':', '-'], "");
                    (window, updates, project_output.join(format!("{through}-{commit}")))
                })
        })
        .and_then(|(window, updates, project_output)| {
            create_dir_all(&project_output)
                .map_err(|why| eyre!("Create report output directory {} — {why}", project_output.display()))
                .map(|()| (window, updates, project_output))
        })
}
async fn publish_manifest(
    arguments: &Report,
    config: &PowerAutomateConfig,
    package: &Package,
    queue: &OperationQueue,
    project_id: &str,
    manifest: ReportManifest,
    pdf: Option<&[u8]>,
) -> ApiResult<ReportManifest> {
    match arguments.dry_run {
        | true => Ok(manifest),
        | false => match package.publish(&manifest, pdf).await {
            | Ok(()) => enqueue_manifest_event(queue, config, project_id, &manifest).map(|()| manifest),
            | Err(why) => Err(why),
        },
    }
}
fn replay_manifest(arguments: &Report, config: &PowerAutomateConfig, queue: &OperationQueue, manifest: ReportManifest) -> ApiResult<ReportManifest> {
    match arguments.dry_run {
        | true => Ok(manifest),
        | false => enqueue_manifest_event(queue, config, &manifest.project_id, &manifest).map(|()| manifest),
    }
}
async fn report_project(
    arguments: &Report,
    config: &PowerAutomateConfig,
    options: &gitlab::Options,
    queue: &OperationQueue,
    commit: &str,
    offline: bool,
) -> ApiResult<ReportManifest> {
    match options
        .project_id()
        .and_then(|project_id| options.path().map(|path| (project_id, path)))
        .and_then(|(project_id, path)| {
            Package::new(options.clone(), PackageOptions::new(&config.package, project_id)).map(|package| (package, path, project_id))
        }) {
        | Ok((package, relative_path, project_id)) => {
            let previous = match arguments.dry_run {
                | true => Ok(None),
                | false => package.download().await,
            };
            match previous {
                | Ok(Some(manifest)) if arguments.from.is_none() && manifest.through == arguments.through => {
                    replay_manifest(arguments, config, queue, manifest)
                }
                | Ok(previous) => {
                    let from = arguments.from.clone().or_else(|| previous.map(|manifest| manifest.through));
                    match prepare_project(arguments, commit, project_id, relative_path, from.as_deref()) {
                        | Ok((window, updates, project_output)) => {
                            let pdf_path = project_output.join("report.pdf");
                            let pdf = match updates.entries.is_empty() {
                                | true => Ok(None),
                                | false => updates::write_pdf(&updates, &pdf_path, offline, arguments.chrome_path.as_deref())
                                    .await
                                    .and_then(|()| read(&pdf_path).map(Some).map_err(|why| eyre!("Read generated report PDF — {why}"))),
                            };
                            match pdf {
                                | Ok(pdf) => match write_manifest(
                                    arguments,
                                    config,
                                    &package,
                                    commit,
                                    project_id,
                                    &window,
                                    &updates,
                                    &project_output,
                                    pdf.as_deref(),
                                ) {
                                    | Ok(manifest) => {
                                        publish_manifest(arguments, config, &package, queue, project_id, manifest, pdf.as_deref()).await
                                    }
                                    | Err(why) => Err(why),
                                },
                                | Err(why) => Err(why),
                            }
                        }
                        | Err(why) => Err(why),
                    }
                }
                | Err(why) => Err(why),
            }
        }
        | Err(why) => Err(why),
    }
}
/// Run bounded reporting for every selected project and fail after independent projects finish.
pub(crate) async fn run(arguments: &Report, offline: bool) -> ApiResult<()> {
    match prepare(arguments, offline) {
        | Ok((automation, routing, commit, selected)) => run_selected(arguments, &automation, &routing, &commit, selected, offline).await,
        | Err(why) => Err(why),
    }
}
async fn run_selected(
    arguments: &Report,
    automation: &PowerAutomateConfig,
    routing: &FolderRegistry,
    commit: &str,
    selected: Vec<Folder>,
    offline: bool,
) -> ApiResult<()> {
    let options = gitlab::Options::from_env().with_identifier(automation.gitlab_project.clone());
    let queue = OperationQueue::configured();
    let mut results = Vec::new();
    for folder in selected {
        let path = folder.path.clone().unwrap_or_else(|| format!("{}/index.json", folder.id));
        let project_options = options.clone().with_path(path).with_project_id(folder.id.clone());
        let result = report_project(arguments, automation, &project_options, &queue, commit, offline).await;
        match result {
            | Ok(manifest) => results.push(
                ProjectReportResult::init()
                    .project_id(folder.id)
                    .status(manifest.status.to_string())
                    .through(manifest.through)
                    .version(manifest.version)
                    .build(),
            ),
            | Err(why) => {
                let message = why.to_string();
                if !arguments.dry_run {
                    let _ = ArtifactEvent::Failed {
                        message: message.clone(),
                        project_id: folder.id.clone(),
                    }
                    .enqueue(
                        &queue,
                        &automation.callback_destination,
                        &format!("{}:{}:failed", folder.id, arguments.through),
                    );
                }
                results.push(
                    ProjectReportResult::init()
                        .error(message)
                        .project_id(folder.id)
                        .status("failed")
                        .through(arguments.through.clone())
                        .build(),
                );
            }
        }
    }
    let callbacks = match arguments.dry_run {
        | true => Ok(()),
        | false => deliver_callbacks(&queue, automation, routing).await,
    };
    let output = serde_json::to_string_pretty(&results).map_err(|why| eyre!("Failed to encode report results — {why}"));
    match output {
        | Ok(output) => {
            println!("{output}");
            let failed = results.iter().filter_map(|result| result.error.as_ref()).cloned().collect::<Vec<_>>();
            match (failed.is_empty(), callbacks) {
                | (true, Ok(())) => Ok(()),
                | (false, _) => Err(eyre!(
                    "{} automated-artifact project report(s) failed — {}",
                    failed.len(),
                    failed.join("; ")
                )),
                | (true, Err(why)) => Err(why),
            }
        }
        | Err(why) => Err(why),
    }
}
#[allow(clippy::too_many_arguments)]
fn write_manifest(
    arguments: &Report,
    config: &PowerAutomateConfig,
    package: &Package,
    commit: &str,
    project_id: &str,
    window: &ReportingWindow,
    updates: &LogbookUpdates,
    project_output: &std::path::Path,
    pdf: Option<&[u8]>,
) -> ApiResult<ReportManifest> {
    let entry_identifiers = updates.entries.iter().map(|entry| entry.identifier.clone()).collect::<Vec<_>>();
    ReportManifest::new(
        config.package.clone(),
        project_id.to_string(),
        commit.to_string(),
        window,
        entry_identifiers,
        arguments.pipeline_id.clone().unwrap_or_else(|| "local".to_string()),
        arguments.job_id.clone().unwrap_or_else(|| "local".to_string()),
        pdf,
        String::new(),
    )
    .and_then(|draft| {
        let file_name = match draft.status {
            | ReportStatus::Published => "report.pdf",
            | ReportStatus::NoChanges => "manifest.json",
        };
        package
            .url(&draft.version, file_name)
            .map(|authenticated_url| ReportManifest { authenticated_url, ..draft })
    })
    .and_then(|manifest| {
        let manifest_path = project_output.join("manifest.json");
        serde_json::to_vec_pretty(&manifest)
            .map_err(|why| eyre!("Encode report manifest — {why}"))
            .map(|mut bytes| {
                bytes.push(b'\n');
                bytes
            })
            .and_then(|bytes| write(&manifest_path, bytes).map_err(|why| eyre!("Write report manifest {} — {why}", manifest_path.display())))
            .map(|()| manifest)
    })
}

#[cfg(test)]
mod tests;
