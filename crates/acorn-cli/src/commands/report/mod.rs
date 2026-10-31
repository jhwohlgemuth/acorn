//! Scheduled bounded artifact creation and immutable package publication.
use self::powerpoint::PowerPointReporter;
#[cfg(feature = "pdf")]
use crate::commands::export::{html, pdf};
use crate::{
    cli::{
        arguments::{Report, ReportFormat},
        parse::resolve_report_bounds,
    },
    commands::export::updates::LogbookUpdatesExport,
};
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
use core::cmp::Reverse;
use jiff::{SignedDuration, Timestamp};
use serde::Serialize;
use std::fs::{create_dir_all, read, write};
use std::path::{Path, PathBuf};

mod powerpoint;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReportSelection {
    Pdf,
    Powerpoint,
    Both,
}

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

impl Report {
    fn through(&self) -> &str {
        self.through.as_deref().unwrap_or_default()
    }
    fn selected_updates(&self, selected: &[Folder]) -> ApiResult<(ReportingWindow, Vec<LogbookUpdates>)> {
        ReportingWindow::new(self.from.as_deref(), self.through())
            .map_err(|why| eyre!(why))
            .and_then(|window| {
                selected
                    .iter()
                    .map(|folder| {
                        folder
                            .source_path(&self.root)
                            .map(|(_, path)| path)
                            .and_then(|path| {
                                ResearchActivity::read(path).map_err(|why| eyre!("Read report source for project '{}' — {why}", folder.id))
                            })
                            .and_then(|activity| activity.updates_within(&window).map_err(|why| eyre!(why)))
                    })
                    .collect::<ApiResult<Vec<_>>>()
                    .map(|updates| (window, updates))
            })
    }
    fn title_updates(window: &ReportingWindow, projects: &[LogbookUpdates]) -> LogbookUpdates {
        let mut entries = projects
            .iter()
            .flat_map(|project| project.entries.iter().filter(|entry| entry.global).cloned())
            .collect::<Vec<_>>();
        entries.sort_by_cached_key(|entry| Reverse(entry.timestamp.parse::<Timestamp>().ok()));
        LogbookUpdates {
            identifier: "report".to_string(),
            title: "General updates".to_string(),
            subtitle: None,
            keywords: Vec::new(),
            principal_investigators: Vec::new(),
            sponsors: Vec::new(),
            since: window.left.clone().unwrap_or_else(|| "full-history".to_string()),
            through: Some(window.right.clone()),
            entries,
        }
    }
    fn global_updates(&self, selected: &[Folder]) -> ApiResult<LogbookUpdates> {
        self.selected_updates(selected)
            .map(|(window, projects)| Self::title_updates(&window, &projects))
    }
    #[cfg(feature = "pdf")]
    async fn write_combined_pdf(&self, selected: &[Folder], offline: bool) -> ApiResult<()> {
        let content = self.selected_updates(selected).and_then(|(window, projects)| {
            let title = Self::title_updates(&window, &projects);
            let projects = projects
                .into_iter()
                .map(|updates| LogbookUpdates {
                    entries: updates.entries.into_iter().filter(|entry| !entry.global).collect(),
                    ..updates
                })
                .filter(|updates| !updates.entries.is_empty())
                .collect::<Vec<_>>();
            html::combined_html(&title, &projects, self.full_timestamp)
        });
        let content = content.and_then(|content| {
            create_dir_all(&self.output)
                .map_err(|why| eyre!("Create report output directory — {why}"))
                .map(|()| content)
        });
        match content {
            | Ok(content) => match pdf::initialize_page(offline, self.chrome_path.as_deref()).await {
                | Ok(session) => {
                    let result =
                        pdf::render_html_to_pdf(session.page(), &content, &self.output.join("report.pdf"), pdf::PdfMargins::Stylesheet).await;
                    session.close().await;
                    result
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    #[cfg(not(feature = "pdf"))]
    async fn write_combined_pdf(&self, _selected: &[Folder], _offline: bool) -> ApiResult<()> {
        Err(eyre!("Combined report export requires the 'pdf' feature. Rebuild with --features pdf"))
    }
    async fn write_title_page(&self, selected: &[Folder], offline: bool) -> ApiResult<()> {
        let updates = self.global_updates(selected).and_then(|updates| match updates.entries.is_empty() {
            | true => Ok(None),
            | false => create_dir_all(&self.output)
                .map_err(|why| eyre!("Create report output directory — {why}"))
                .map(|()| Some(updates)),
        });
        match updates {
            | Ok(Some(updates)) => {
                updates
                    .write_title_pdf(
                        &self.output.join("title-page.pdf"),
                        offline,
                        self.chrome_path.as_deref(),
                        self.full_timestamp,
                    )
                    .await
            }
            | Ok(None) => Ok(()),
            | Err(why) => Err(why),
        }
    }
    #[cfg(feature = "pdf")]
    async fn write_legend_page(&self, offline: bool) -> ApiResult<()> {
        let content = create_dir_all(&self.output)
            .map_err(|why| eyre!("Create report output directory — {why}"))
            .and_then(|()| html::legend_html());
        match content {
            | Ok(content) => match pdf::initialize_page(offline, self.chrome_path.as_deref()).await {
                | Ok(session) => {
                    let result = pdf::render_html_to_pdf(
                        session.page(),
                        &content,
                        &self.output.join("legend-page.pdf"),
                        pdf::PdfMargins::Stylesheet,
                    )
                    .await;
                    session.close().await;
                    result
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn report_deck(&self, folder: &Folder, commit: &str, from: Option<&str>) -> ApiResult<Option<PathBuf>> {
        folder.source_path(&self.root).and_then(|(relative_path, index_path)| {
            prepare_project(self, commit, &folder.id, &relative_path, from).and_then(|(_, updates, project_output)| {
                ResearchActivity::read(&index_path)
                    .map_err(|why| eyre!("Read report source for project '{}' — {why}", folder.id))
                    .and_then(|activity| {
                        let reference = report_reference_path(self.reference.as_deref(), &index_path);
                        activity
                            .render(&updates, &index_path, &reference, &project_output.join("report.pptx"))
                            .map_err(|why| eyre!("Render PowerPoint for project '{}' using {} — {why}", folder.id, reference.display()))
                    })
            })
        })
    }
    fn report_commit(&self) -> ApiResult<&str> {
        self.commit
            .as_deref()
            .filter(|commit| !commit.trim().is_empty())
            .ok_or_else(|| eyre!("Report source commit is required"))
    }
    fn run_local_decks(&self, selected: Vec<Folder>) -> ApiResult<()> {
        self.report_commit().and_then(|commit| {
            let results = selected
                .into_iter()
                .map(|folder| {
                    let result = self.report_deck(&folder, commit, self.from.as_deref());
                    let (status, error) = match result {
                        | Ok(Some(_)) => ("created", None),
                        | Ok(None) => ("no-changes", None),
                        | Err(why) => ("failed", Some(why.to_string())),
                    };
                    ProjectReportResult {
                        error,
                        project_id: folder.id,
                        status: status.to_string(),
                        through: self.through().to_string(),
                        version: None,
                    }
                })
                .collect::<Vec<_>>();
            print_results(&results, "PowerPoint")
        })
    }
}

impl ReportSelection {
    fn lower_bound(self, explicit: Option<&str>, previous: Option<&ReportManifest>, replay: bool) -> Option<String> {
        explicit.map(str::to_string).or_else(|| match (self, replay) {
            | (Self::Pdf | Self::Both, true) => previous.and_then(|manifest| manifest.from.clone()),
            | (Self::Pdf | Self::Both, false) => previous.map(|manifest| manifest.through.clone()),
            | (Self::Powerpoint, _) => None,
        })
    }
}

impl TryFrom<Vec<ReportFormat>> for ReportSelection {
    type Error = color_eyre::Report;

    fn try_from(formats: Vec<ReportFormat>) -> Result<Self, Self::Error> {
        match formats.as_slice() {
            | [] | [ReportFormat::Pdf] => Ok(Self::Pdf),
            | [ReportFormat::Powerpoint] => Ok(Self::Powerpoint),
            | [ReportFormat::Pdf, ReportFormat::Powerpoint] | [ReportFormat::Powerpoint, ReportFormat::Pdf] => Ok(Self::Both),
            | _ => Err(eyre!("Report formats must contain each of pdf and powerpoint at most once")),
        }
    }
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

fn prepare_combined(arguments: &Report) -> ApiResult<Vec<Folder>> {
    ApplicationConfiguration::load(&arguments.config)
        .and_then(|configuration| {
            configuration
                .validate()
                .map_err(|why| eyre!("Invalid application configuration — {why}"))
                .map(|()| configuration)
        })
        .and_then(|configuration| {
            let selected = configuration
                .registry()
                .folders()
                .filter(|folder| arguments.projects.is_empty() || arguments.projects.contains(&folder.id))
                .cloned()
                .collect::<Vec<_>>();
            match selected.is_empty() {
                | true => Err(eyre!("No configured folders matched the report selection")),
                | false => Ok(selected),
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
    ReportingWindow::new(from, arguments.through())
        .map_err(|why| eyre!(why))
        .and_then(|window| {
            SafePath::new(relative_path)
                .map_err(|why| eyre!(why))
                .and_then(|path| path.materialize_under(&arguments.root).map_err(Into::into))
                .and_then(|path| ResearchActivity::read(path).map_err(|why| eyre!("Read report source for project '{project_id}' — {why}")))
                .and_then(|activity| activity.updates_within(&window).map_err(|why| eyre!(why)))
                .map(|updates| {
                    let entries = updates.entries.into_iter().filter(|entry| !entry.global).collect();
                    (window, LogbookUpdates { entries, ..updates })
                })
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

fn print_results(results: &[ProjectReportResult], artifact: &str) -> ApiResult<()> {
    serde_json::to_string_pretty(results)
        .map_err(|why| eyre!("Failed to encode report results — {why}"))
        .and_then(|output| {
            println!("{output}");
            let failed = results.iter().filter_map(|result| result.error.as_ref()).collect::<Vec<_>>();
            if failed.is_empty() {
                Ok(())
            } else {
                Err(eyre!(
                    "{} {artifact} project report(s) failed — {}",
                    failed.len(),
                    failed.into_iter().cloned().collect::<Vec<_>>().join("; ")
                ))
            }
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
    let package = options
        .project_id()
        .and_then(|project_id| options.path().map(|path| (project_id, path)))
        .and_then(|(project_id, path)| {
            Package::new(options.clone(), PackageOptions::new(&config.package, project_id)).map(|package| (package, path, project_id))
        });
    match package {
        | Ok((package, relative_path, project_id)) => {
            let previous = match arguments.dry_run {
                | true => Ok(None),
                | false => package.download().await,
            };
            match previous {
                | Ok(Some(manifest)) if arguments.from.is_none() && manifest.through == arguments.through() => {
                    replay_manifest(arguments, config, queue, manifest)
                }
                | Ok(previous) => {
                    let from = ReportSelection::Pdf.lower_bound(arguments.from.as_deref(), previous.as_ref(), false);
                    match prepare_project(arguments, commit, project_id, relative_path, from.as_deref()) {
                        | Ok((window, updates, project_output)) => {
                            let pdf_path = project_output.join("report.pdf");
                            let pdf = match updates.entries.is_empty() {
                                | true => Ok(None),
                                | false => updates
                                    .write_pdf(&pdf_path, offline, arguments.chrome_path.as_deref(), arguments.full_timestamp)
                                    .await
                                    .and_then(|()| read(&pdf_path).map(Some).map_err(|why| eyre!("Read generated report PDF — {why}"))),
                            };
                            match pdf {
                                | Ok(pdf) => {
                                    let manifest = write_manifest(
                                        arguments,
                                        config,
                                        &package,
                                        commit,
                                        project_id,
                                        &window,
                                        &updates,
                                        &project_output,
                                        pdf.as_deref(),
                                    );
                                    match manifest {
                                        | Ok(manifest) => {
                                            publish_manifest(arguments, config, &package, queue, project_id, manifest, pdf.as_deref()).await
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
                | Err(why) => Err(why),
            }
        }
        | Err(why) => Err(why),
    }
}

fn report_reference_path(reference: Option<&Path>, index_path: &Path) -> PathBuf {
    let parent = index_path.parent().unwrap_or_else(|| Path::new("."));
    match reference {
        | Some(path) if path.is_absolute() => path.to_path_buf(),
        | Some(path) => parent.join(path),
        | None => parent.join("reference.pptx"),
    }
}

/// Run bounded reporting for every selected project and fail after independent projects finish.
pub(crate) async fn run(arguments: &Report, offline: bool) -> ApiResult<()> {
    let year = Timestamp::now().to_zoned(jiff::tz::TimeZone::UTC).year();
    match ReportSelection::try_from(arguments.formats.clone()) {
        | Err(why) => Err(why),
        | Ok(selection) => match resolve_report_bounds(arguments.from.as_deref(), arguments.through.as_deref(), year) {
            | Err(why) => Err(eyre!(why)),
            | Ok((from, through)) => {
                let normalized = Report {
                    from,
                    through: Some(through),
                    ..arguments.clone()
                };
                let arguments = &normalized;
                match (arguments.combine, selection) {
                    | (true, ReportSelection::Pdf) => match prepare_combined(arguments) {
                        | Ok(selected) => arguments.write_combined_pdf(&selected, offline).await,
                        | Err(why) => Err(why),
                    },
                    | (true, ReportSelection::Powerpoint) => match prepare_combined(arguments) {
                        | Ok(selected) => arguments.run_local_decks(selected),
                        | Err(why) => Err(why),
                    },
                    | (true, ReportSelection::Both) => match prepare_combined(arguments) {
                        | Ok(selected) => match arguments.report_commit() {
                            | Ok(_) => {
                                let pdf = arguments.write_combined_pdf(&selected, offline).await;
                                let decks = arguments.run_local_decks(selected);
                                match (pdf, decks) {
                                    | (Ok(()), Ok(())) => Ok(()),
                                    | (Err(why), Ok(())) | (Ok(()), Err(why)) => Err(why),
                                    | (Err(pdf), Err(decks)) => Err(eyre!("Combined PDF failed: {pdf}; PowerPoint decks failed: {decks}")),
                                }
                            }
                            | Err(why) => Err(why),
                        },
                        | Err(why) => Err(why),
                    },
                    | (false, ReportSelection::Powerpoint) => match prepare_combined(arguments) {
                        | Ok(selected) => arguments.run_local_decks(selected),
                        | Err(why) => Err(why),
                    },
                    | (false, ReportSelection::Pdf | ReportSelection::Both) => match prepare(arguments, offline) {
                        | Ok((automation, routing, commit, selected)) => match arguments.write_title_page(&selected, offline).await {
                            | Ok(()) => {
                                #[cfg(feature = "pdf")]
                                let legend = arguments.write_legend_page(offline).await;
                                #[cfg(not(feature = "pdf"))]
                                let legend: ApiResult<()> = Ok(());
                                match legend {
                                    | Ok(()) => run_selected(arguments, &automation, &routing, &commit, selected, selection, offline).await,
                                    | Err(why) => Err(why),
                                }
                            }
                            | Err(why) => Err(why),
                        },
                        | Err(why) => Err(why),
                    },
                }
            }
        },
    }
}

async fn run_selected(
    arguments: &Report,
    automation: &PowerAutomateConfig,
    routing: &FolderRegistry,
    commit: &str,
    selected: Vec<Folder>,
    selection: ReportSelection,
    offline: bool,
) -> ApiResult<()> {
    let options = gitlab::Options::from_env().with_identifier(automation.gitlab_project.clone());
    let queue = OperationQueue::configured();
    let mut results = Vec::new();
    for folder in selected {
        let pdf_result = match folder.source_path(&arguments.root) {
            | Ok((path, _)) => {
                let project_options = options.clone().with_path(path).with_project_id(folder.id.clone());
                report_project(arguments, automation, &project_options, &queue, commit, offline).await
            }
            | Err(why) => Err(why),
        };
        match pdf_result {
            | Ok(manifest) => {
                let deck = match selection {
                    | ReportSelection::Both => arguments.report_deck(&folder, commit, manifest.from.as_deref()).map(|_| ()),
                    | _ => Ok(()),
                };
                match deck {
                    | Ok(()) => results.push(
                        ProjectReportResult::init()
                            .project_id(folder.id)
                            .status(manifest.status.to_string())
                            .through(manifest.through)
                            .version(manifest.version)
                            .build(),
                    ),
                    | Err(why) => results.push(
                        ProjectReportResult::init()
                            .error(why.to_string())
                            .project_id(folder.id)
                            .status("powerpoint-failed")
                            .through(manifest.through)
                            .version(manifest.version)
                            .build(),
                    ),
                }
            }
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
                        &format!("{}:{}:failed", folder.id, arguments.through()),
                    );
                }
                results.push(
                    ProjectReportResult::init()
                        .error(message)
                        .project_id(folder.id)
                        .status("failed")
                        .through(arguments.through().to_string())
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
