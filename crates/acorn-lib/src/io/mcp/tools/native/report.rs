//! Shared report preparation and bounded native report operations.
use super::super::{
    ArtifactPolicy, BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, InputLimits, RoutedInputPolicy, ToolRegistry,
};
use super::NativeEffect;
use crate::error::ApiResult;
use crate::io::api::gitlab::{Package, PackageLifecycle, ReportManifest, ReportStatus};
use crate::io::InputOutput;
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::SemanticVersion;
use acorn_host::fs::SafePath;
use acorn_schema::agent::tools::ToolEffects;
use acorn_schema::research_activity::{LogbookUpdates, ReportingWindow, ResearchActivity};
use color_eyre::eyre::eyre;
use core::future::Future;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};

/// Maximum byte length accepted for report identifiers and commits.
const MAX_REPORT_IDENTIFIER_BYTES: usize = 128;

/// Maximum byte length accepted for report window timestamps.
const MAX_REPORT_TIMESTAMP_BYTES: usize = 64;

/// Prepared project entries and exact artifact directory.
pub struct PreparedProjectReport {
    /// Filtered local project updates.
    pub updates: LogbookUpdates,
    /// Exact artifact directory.
    pub output: PathBuf,
    /// Normalized reporting window.
    pub window: ReportingWindow,
}

/// One configured project report request under a bounded checkout root.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReportInput {
    /// Local application configuration beneath the permitted root.
    pub config: PathBuf,
    /// Stable project identifier declared by the configuration.
    pub project_id: String,
    /// Inclusive lower bound for selected updates.
    pub from: Option<String>,
    /// Exclusive upper bound for selected updates.
    pub through: String,
    /// Exact merged source commit.
    pub commit: String,
    /// Exact output directory relative to the permitted root.
    pub destination: PathBuf,
    /// GitLab pipeline ID for explicit publication.
    pub pipeline_id: Option<String>,
    /// GitLab job ID for explicit publication.
    pub job_id: Option<String>,
    /// Render full timestamps in PDFs.
    #[serde(default)]
    pub full_timestamp: bool,
    /// Permit an existing output directory.
    #[serde(default)]
    pub overwrite: bool,
}

/// Inputs needed to persist a deterministic immutable-package manifest.
pub struct ReportManifestRequest<'a> {
    /// Permitted root for routed report artifacts; direct commands may omit it.
    pub allowed_root: Option<&'a Path>,
    /// Exact source commit.
    pub commit: &'a str,
    /// GitLab job identifier, or a local placeholder.
    pub job_id: &'a str,
    /// Exact artifact directory.
    pub output: &'a Path,
    /// Configured GitLab package name.
    pub package_name: &'a str,
    /// Package URL resolver from trusted GitLab configuration.
    pub package: &'a Package,
    /// GitLab pipeline identifier, or a local placeholder.
    pub pipeline_id: &'a str,
    /// Configured stable project identifier.
    pub project_id: &'a str,
    /// Selected local project updates.
    pub updates: &'a LogbookUpdates,
    /// Normalized reporting window.
    pub window: &'a ReportingWindow,
}

/// Bounded report artifact and publication receipt.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReportOutput {
    /// Stable project identifier.
    pub project_id: String,
    /// Report status.
    pub status: ReportStatus,
    /// Immutable package version.
    pub version: String,
    /// Canonical output directory.
    pub destination: PathBuf,
    /// Local report artifact paths, excluding credentials and publication URLs.
    pub artifacts: Vec<String>,
}

/// Inputs for one configured project report projection.
pub struct ReportProjectRequest<'a> {
    /// Exact source commit used in the artifact name and manifest.
    pub commit: &'a str,
    /// Inclusive lower bound for selected logbook entries.
    pub from: Option<&'a str>,
    /// Output root for selected reports.
    pub output: &'a Path,
    /// Configured stable project identifier.
    pub project_id: &'a str,
    /// Configured project index path relative to the checkout root.
    pub relative_path: &'a str,
    /// Repository checkout root.
    pub root: &'a Path,
    /// Exclusive upper bound for selected logbook entries.
    pub through: &'a str,
}

impl ReportInput {
    pub(super) fn validate(&self, root: &Path, publish: bool) -> ApiResult<(Self, PathBuf)> {
        let bounded = !self.project_id.is_empty()
            && self.project_id.len() <= MAX_REPORT_IDENTIFIER_BYTES
            && self.through.len() <= MAX_REPORT_TIMESTAMP_BYTES
            && self.from.as_ref().is_none_or(|from| from.len() <= MAX_REPORT_TIMESTAMP_BYTES)
            && self.commit.len() <= MAX_REPORT_IDENTIFIER_BYTES
            && self.commit.chars().all(|character| character.is_ascii_hexdigit())
            && self
                .pipeline_id
                .as_ref()
                .is_none_or(|id| !id.is_empty() && id.len() <= MAX_REPORT_IDENTIFIER_BYTES)
            && self
                .job_id
                .as_ref()
                .is_none_or(|id| !id.is_empty() && id.len() <= MAX_REPORT_IDENTIFIER_BYTES);
        match (bounded, publish && (self.pipeline_id.is_none() || self.job_id.is_none())) {
            | (false, _) => Err(eyre!("Report request exceeds the bounded input policy")),
            | (_, true) => Err(eyre!("Report publication requires pipeline and job identifiers")),
            | (true, false) => SafePath::new(&self.project_id)
                .map_err(|why| eyre!("Invalid report project identifier — {why}"))
                .and_then(|_| RoutedInputPolicy::new(root, InputLimits::default()))
                .and_then(|inputs| inputs.resolve_path(&self.config))
                .and_then(|config| {
                    ArtifactPolicy::new(root, true, self.overwrite)
                        .and_then(|artifacts| artifacts.destination(&self.destination, &[]))
                        .and_then(|destination| {
                            root.canonicalize().map_err(|why| eyre!("Resolve report root — {why}")).and_then(|base| {
                                match destination == base || config.starts_with(&destination) {
                                    | true => Err(eyre!("Report output must be below the permitted root and separate from configuration")),
                                    | false => Ok((config, destination)),
                                }
                            })
                        })
                })
                .map(|(config, destination)| (Self { config, ..self.clone() }, destination)),
        }
    }
}

impl ReportManifestRequest<'_> {
    /// Write the same final manifest as the direct report command.
    pub fn write(self, pdf: Option<&[u8]>) -> ApiResult<ReportManifest> {
        let entry_identifiers = self
            .updates
            .entries
            .iter()
            .map(|entry| entry.identifier().to_string())
            .collect::<Vec<_>>();
        let manifest = ReportManifest::new(
            self.package_name.to_string(),
            self.project_id.to_string(),
            self.commit.to_string(),
            self.window,
            entry_identifiers,
            self.pipeline_id.to_string(),
            self.job_id.to_string(),
            pdf,
            String::new(),
        );
        manifest
            .and_then(|draft| {
                let file_name = match draft.status {
                    | ReportStatus::Published => "report.pdf",
                    | ReportStatus::NoChanges => "manifest.json",
                };
                self.package
                    .url(&draft.version, file_name)
                    .map(|authenticated_url| ReportManifest { authenticated_url, ..draft })
            })
            .and_then(|manifest| {
                let path = self.output.join("manifest.json");
                let checked = match self.allowed_root {
                    | None => Ok(path),
                    | Some(root) => root
                        .canonicalize()
                        .map_err(|why| eyre!("Resolve permitted report root — {why}"))
                        .and_then(|base| {
                            self.output
                                .canonicalize()
                                .map_err(|why| eyre!("Resolve report manifest directory — {why}"))
                                .and_then(|directory| {
                                    let canonical_path = directory.join("manifest.json");
                                    canonical_path
                                        .strip_prefix(&base)
                                        .map_err(|why| eyre!("Report manifest lies outside the permitted root — {why}"))
                                        .and_then(|relative| {
                                            ArtifactPolicy::new(&base, true, true).and_then(|policy| policy.destination(relative, &[]))
                                        })
                                })
                        }),
                };
                checked.and_then(|path| {
                    serde_json::to_vec_pretty(&manifest)
                        .map_err(|why| eyre!("Encode report manifest — {why}"))
                        .map(|mut bytes| {
                            bytes.push(b'\n');
                            bytes
                        })
                        .and_then(|bytes| write(&path, bytes).map_err(|why| eyre!("Write report manifest {} — {why}", path.display())))
                        .map(|()| manifest)
                })
            })
    }
}

impl ReportOutput {
    fn validate_destination(self, destination: &Path) -> ApiResult<Self> {
        let artifacts_are_local = self.artifacts.iter().all(|artifact| Path::new(artifact).starts_with(destination));
        match self.destination == destination && artifacts_are_local {
            | true => Ok(self),
            | false => Err(eyre!("Report operation returned an unexpected artifact destination")),
        }
    }
}

impl ReportProjectRequest<'_> {
    /// Select updates using the direct report command's source and output rules.
    pub fn prepare(self) -> ApiResult<PreparedProjectReport> {
        ReportingWindow::new(self.from, self.through)
            .map_err(|why| eyre!(why))
            .and_then(|window| {
                SafePath::new(self.relative_path)
                    .map_err(|why| eyre!(why))
                    .and_then(|path| path.materialize_under(self.root).map_err(Into::into))
                    .and_then(|path| path.canonicalize().map_err(|why| eyre!("Resolve report source — {why}")))
                    .and_then(|path| RoutedInputPolicy::new(self.root, InputLimits::default()).and_then(|policy| policy.resolve_path(&path)))
                    .and_then(|path| {
                        ResearchActivity::read(path).map_err(|why| eyre!("Read report source for project '{}' — {why}", self.project_id))
                    })
                    .and_then(|activity| activity.updates_within(&window).map_err(|why| eyre!(why)))
                    .map(|updates| {
                        let entries = updates.entries.into_iter().filter(|entry| !entry.global()).collect();
                        (window, LogbookUpdates { entries, ..updates })
                    })
            })
            .and_then(|(window, updates)| {
                SafePath::new(self.project_id)
                    .map_err(|why| eyre!(why))
                    .and_then(|project| project.materialize_under(self.output).map_err(Into::into))
                    .map(|project_output| {
                        let commit = self.commit.chars().filter(char::is_ascii_hexdigit).take(12).collect::<String>();
                        let through = window.right.replace([':', '-'], "");
                        PreparedProjectReport {
                            window,
                            updates,
                            output: project_output.join(format!("{through}-{commit}")),
                        }
                    })
            })
            .and_then(|prepared| {
                create_dir_all(&prepared.output)
                    .map_err(|why| eyre!("Create report output directory {} — {why}", prepared.output.display()))
                    .map(|()| prepared)
            })
    }
}

impl ToolRegistry {
    /// Register local report generation and explicit GitLab publication.
    /// The callback must enforce the supplied offline policy.
    pub fn register_report_capabilities<F, Fut>(self, root: impl AsRef<Path>, offline: bool, execute: F) -> ApiResult<Self>
    where
        F: Fn(ReportInput, PathBuf, PathBuf, bool) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ApiResult<ReportOutput>> + Send + 'static,
    {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let local_effects = ToolEffects::from(NativeEffect::RemoteWrite);
        let publish_effects = ToolEffects {
            open_world: true,
            ..local_effects
        };
        let local_root = root.clone();
        let local_execute = execute.clone();
        let register = self.register_capability(CapabilityDefinition::new::<ReportInput, ReportOutput>(
            format!("{APPLICATION}.report::local"),
            version,
            "Generate one bounded configured project report into an exact local directory.",
            local_effects,
        ));
        register
            .and_then(|registry| {
                let binding = BindingDefinition::new::<ReportInput, ReportOutput>(
                    format!("native:{APPLICATION}.report::local"),
                    format!("{APPLICATION}.report::local"),
                    version,
                    BindingKind::Native,
                    local_effects,
                    BindingAvailability::Ready,
                );
                let binding = if offline { binding.with_offline_support() } else { binding };
                registry.register_capability_binding::<ReportInput, ReportOutput, _, _>(binding, move |input| {
                    let root = local_root.clone();
                    let execute = local_execute.clone();
                    async move {
                        match input.validate(&root, false) {
                            | Ok((input, destination)) => execute(input, root, destination.clone(), false)
                                .await
                                .and_then(|output| output.validate_destination(&destination)),
                            | Err(why) => Err(why),
                        }
                    }
                })
            })
            .and_then(|registry| {
                registry.register_capability(CapabilityDefinition::new::<ReportInput, ReportOutput>(
                    format!("{APPLICATION}.report::publish"),
                    version,
                    "Publish one configured project report to the trusted GitLab destination.",
                    publish_effects,
                ))
            })
            .and_then(|registry| {
                registry.register_capability_binding::<ReportInput, ReportOutput, _, _>(
                    BindingDefinition::new::<ReportInput, ReportOutput>(
                        format!("native:{APPLICATION}.report::publish"),
                        format!("{APPLICATION}.report::publish"),
                        version,
                        BindingKind::Native,
                        publish_effects,
                        BindingAvailability::Ready,
                    ),
                    move |input| {
                        let root = root.clone();
                        let execute = execute.clone();
                        async move {
                            match input.validate(&root, true) {
                                | Ok((input, destination)) => execute(input, root, destination.clone(), true)
                                    .await
                                    .and_then(|output| output.validate_destination(&destination)),
                                | Err(why) => Err(why),
                            }
                        }
                    },
                )
            })
    }
}
