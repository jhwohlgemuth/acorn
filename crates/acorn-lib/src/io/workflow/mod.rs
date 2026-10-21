//! Plan and apply repeatable processing to supported metadata standards.
//!
//! The workflow keeps provider-specific enrichment behind [`Enrich`](crate::io::enrichment::Enrich) and separates transformation planning from filesystem writes.
//! Callers can use [`Workflow::plan`] for previews or [`Workflow::run`] to plan and apply in one operation.
#[cfg(feature = "exec")]
use crate::analyzer::host::render::CheckRender;
#[cfg(feature = "analysis")]
use crate::analyzer::Check;
#[cfg(feature = "exec")]
use crate::exec::capability::FileProposal;
#[cfg(feature = "exec")]
use crate::exec::config::ExecutionOrigin;
#[cfg(feature = "exec")]
use crate::exec::hooks::{HookAuditRecord, HookOptions};
#[cfg(feature = "exec")]
use crate::io::config::application::ApplicationConfiguration;
use crate::{
    fail,
    io::{enrichment::Enrich, read_file, ApiResult, FromPath, InputOutput},
    util::{constants::app::MAX_ENRICHMENT_CANDIDATES, print_changes_with_color, StringConversion},
    LocationExt, With,
};
use acorn_core::{
    options::{CommonRuntime, RuntimeOptions},
    prelude::{Box, String, Vec},
    util::MimeType,
    Location,
};
use acorn_host::terminal::Label;
use acorn_schema::{
    enrichment::Provider,
    validation::Validate,
    vocabulary::{VocabularyFormat, VocabularyReport},
};
use color_eyre::eyre::eyre;
use core::marker::PhantomData;
use futures::stream::{self, StreamExt, TryStreamExt};
use owo_colors::OwoColorize;
#[cfg(feature = "exec")]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
#[cfg(feature = "exec")]
use std::path::Path;
use std::path::PathBuf;
use tracing::warn;

/// Pure logbook-entry repository proposal workflow.
pub mod logbook;
/// Runtime registry for named repository workflows.
pub mod registry;
/// Transport-neutral repository workflow data contracts.
pub mod repository;
mod transaction;

pub(crate) use transaction::Artifacts;

pub use logbook::LogbookProposal;
pub use registry::RepositoryWorkflowRegistry;
pub use repository::{
    FileChange, FileChangeKind, ProcessRepositoryRequest, ProcessRepositoryResponse, RepositoryFileInput, WorkflowAction, WorkflowFileReport,
};

/// Options shared by workflow interfaces.
pub type Options = RuntimeOptions<WorkflowExtension>;
/// One planned change to a supported data source.
#[derive(Clone, Debug, With)]
pub enum Change<T> {
    /// A fully planned document rewrite and its optional companion artifact.
    Document {
        /// Whether applying the plan will rewrite the source file.
        changed: bool,
        /// Non-fatal enrichment conflicts encountered while producing this change.
        conflicts: Vec<String>,
        /// Unresolved enrichment choices retained for interactive review.
        decisions: Vec<EnrichmentDecision>,
        /// Serialized content in the source file's format.
        content: String,
        /// Parsed and transformed standard data.
        data: Box<T>,
        /// Provider failures encountered while producing this change.
        failures: Vec<String>,
        /// Optional JSON-LD companion artifact.
        linked: Option<Artifact>,
        /// Exact source content retained for transactional audit rollback.
        original: String,
        /// Source represented by this change.
        source: Location,
        /// Controlled-vocabulary resolutions that contributed to this rewrite.
        vocabulary_reports: Vec<VocabularyReport>,
    },
    /// A check retained as the input to a future automated fix.
    #[cfg(feature = "analysis")]
    Check(Check),
}
/// Explicit outcome of reviewing one enrichment decision.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnrichmentSelection {
    /// Use the candidate identified by the caller.
    Choose(EnrichmentCandidateKey),
    /// Leave the field unchanged.
    Skip,
}
/// A generated workflow artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artifact {
    /// Destination path for the artifact.
    pub path: PathBuf,
    /// Serialized artifact content.
    pub content: String,
}
/// One selectable value backed by provider evidence.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnrichmentCandidate {
    /// Stable candidate identity.
    pub key: EnrichmentCandidateKey,
    /// Human-readable candidate label.
    pub label: String,
    /// Provider record or lookup that supplied the candidate.
    pub evidence_source: String,
}
/// Stable identity for one proposed enrichment value.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnrichmentCandidateKey {
    /// Field to replace when this value is selected.
    pub field: String,
    /// Enrichment provider that proposed the value.
    pub provider: Provider,
    /// Exact source document being enriched.
    pub source: Location,
    /// Proposed replacement value.
    pub value: String,
}
/// An unresolved field and its bounded, source-attributed choices.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnrichmentDecision {
    /// Values available for interactive review, in provider order.
    pub candidates: Vec<EnrichmentCandidate>,
    /// Exact field to update.
    pub field: String,
    /// Original field value retained for stale-source checks.
    pub original: String,
    /// Provider that reported ambiguity.
    pub provider: Provider,
    /// Why this field needs a decision.
    pub reason: String,
    /// Exact source document being enriched.
    pub source: Location,
}
/// Result of applying one or more enrichment providers.
#[derive(bon::Builder, Clone, Debug, Default)]
#[builder(on(String, into))]
pub struct EnrichmentResult<T> {
    /// Enriched metadata.
    pub data: T,
    /// Existing-value or ambiguous-match conflicts that were left unchanged.
    #[builder(default)]
    pub conflicts: Vec<String>,
    /// Unresolved choices in deterministic provider order.
    #[builder(default)]
    pub decisions: Vec<EnrichmentDecision>,
    /// Non-fatal provider failures encountered after other providers continued.
    #[builder(default)]
    pub failures: Vec<String>,
}
/// A complete, validated set of changes prepared before any writes occur.
#[derive(Clone, Debug, Default)]
pub struct Plan<T> {
    /// Additional host-owned files proposed by executable hooks.
    pub additional: Vec<Artifact>,
    /// Audit provenance for executable hooks.
    #[cfg(feature = "exec")]
    pub audits: Vec<HookAuditRecord>,
    /// Planned source changes and companion artifacts.
    pub changes: Vec<Change<T>>,
}
/// Summary of a workflow application.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Report {
    /// Audit provenance for executable hooks.
    #[cfg(feature = "exec")]
    pub audits: Vec<HookAuditRecord>,
    /// Number of source files processed.
    pub processed: usize,
    /// Number of source files whose content differed after processing.
    pub changed: usize,
    /// Number of source files and companion artifacts written.
    pub written: usize,
    /// Number of JSON-LD companion artifacts planned.
    pub linked: usize,
}
/// Service for planning and applying workflows for a supported standard.
#[derive(Clone, Copy, Debug)]
pub struct Workflow<T>(PhantomData<fn() -> T>);
/// Workflow-specific runtime options.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WorkflowExtension {
    /// Validate input and transformed research activity data.
    pub check: bool,
    /// Invoke the supplied enrichment capability.
    pub enrich: bool,
    /// Normalize research activity data before output.
    pub format: bool,
    /// Controlled-vocabulary fuzzy matching profile.
    pub fuzzy: acorn_schema::vocabulary::VocabularyFuzzyMode,
    /// Create a JSON-LD companion file.
    pub link: bool,
    /// Render changes even when the workflow will write them.
    pub show_changes: bool,
}
#[cfg(feature = "exec")]
impl ApplicationConfiguration {
    /// Run configured Rhai quality gates for a planned workflow.
    ///
    /// Scripts are resolved and evaluated after proposal validation but before application. Any
    /// resolution, compilation, execution, or validation failure aborts with no partial application.
    pub fn check_scripts<T>(&self, changes: &[Change<T>], project_root: &Path, hook: WorkflowAction, origin: ExecutionOrigin) -> ApiResult<Vec<Check>>
    where
        T: serde::Serialize,
    {
        self.check_script_hooks(changes, project_root, &[hook], origin, false, false)
    }
    /// Run configured Rhai quality gates for an ordered set of workflow hooks.
    ///
    /// Every matching script is resolved and statically compiled before any script is evaluated.
    pub fn check_script_hooks<T>(
        &self,
        changes: &[Change<T>],
        project_root: &Path,
        hooks: &[WorkflowAction],
        origin: ExecutionOrigin,
        explicit_configuration: bool,
        offline: bool,
    ) -> ApiResult<Vec<Check>>
    where
        T: serde::Serialize,
    {
        self.validate().map_err(|why| eyre!("{why}")).and_then(|()| {
            self.prepare_scripts(project_root, hooks, origin, explicit_configuration)
                .and_then(|relevant| match relevant.is_empty() {
                    | true => Ok(Vec::new()),
                    | false => changes
                        .iter()
                        .enumerate()
                        .filter_map(|(document_index, change)| match change {
                            | Change::Document { source, data, .. } => Some((document_index, source, data.as_ref())),
                            #[cfg(feature = "analysis")]
                            | Change::Check(_) => None,
                        })
                        .try_fold(Vec::new(), |checks, (document_index, source, data)| {
                            let source = source.uri().unwrap_or_else(|| source.to_string());
                            relevant.iter().try_fold(checks, |checks, script| {
                                hooks
                                    .iter()
                                    .filter(|hook| script.resolved.entry.hooks.iter().any(|candidate| candidate == *hook))
                                    .try_fold(checks, |checks, hook| {
                                        script
                                            .resolved
                                            .evaluate_hook(
                                                &source,
                                                data,
                                                HookOptions::init()
                                                    .maybe_authorization(script.authorization.as_ref())
                                                    .document_index(document_index)
                                                    .hook(*hook)
                                                    .maybe_manifest_source(script.manifest_source.as_deref())
                                                    .offline(offline)
                                                    .build(),
                                            )
                                            .map(|execution| checks.into_iter().chain(execution.checks).collect())
                                    })
                            })
                        }),
                })
        })
    }
}
#[cfg(feature = "exec")]
impl From<FileProposal> for Artifact {
    fn from(proposal: FileProposal) -> Self {
        Self {
            content: proposal.content,
            path: proposal.path,
        }
    }
}
impl<T> Change<T>
where
    T: InputOutput + Serialize,
{
    /// Render the changes produced by enrichment without applying them.
    pub fn render_enriched(&self, options: &Options) -> ApiResult<()> {
        let CommonRuntime { dry_run, no_color, .. } = &options.common;
        let WorkflowExtension { show_changes, .. } = &options.extension;
        match self {
            | Self::Document { source, content, .. } => match source.local_path() {
                | Some(path) => T::read(path.clone()).and_then(|data| {
                    InputOutput::serialize_as(&data, &MimeType::from_path(&path)).map(|old_content| {
                        if *dry_run {
                            let absolute = path.to_absolute_path();
                            match no_color {
                                | true => println!("{} Enrich and format {absolute}", Label::DRY_RUN),
                                | false => println!("{} Enrich and format {}", Label::dry_run(), absolute.yellow()),
                            }
                        }
                        if *dry_run || *show_changes {
                            print_changes_with_color(&old_content, content, !no_color);
                        }
                    })
                }),
                | None => Ok(()),
            },
            #[cfg(feature = "analysis")]
            | Self::Check(_) => Ok(()),
        }
    }
    /// Return whether this change rewrites its source document.
    pub fn is_changed(&self) -> bool {
        match self {
            | Self::Document { changed, .. } => *changed,
            #[cfg(feature = "analysis")]
            | Self::Check(_) => false,
        }
    }
    /// Return the transformed standard data, when this is a document change.
    pub fn data(&self) -> Option<&T> {
        match self {
            | Self::Document { data, .. } => Some(data),
            #[cfg(feature = "analysis")]
            | Self::Check(_) => None,
        }
    }
    /// Return non-fatal enrichment failures for this change.
    pub fn failures(&self) -> &[String] {
        match self {
            | Self::Document { failures, .. } => failures,
            #[cfg(feature = "analysis")]
            | Self::Check(_) => &[],
        }
    }
    /// Return non-fatal enrichment conflicts for this change.
    pub fn conflicts(&self) -> &[String] {
        match self {
            | Self::Document { conflicts, .. } => conflicts,
            #[cfg(feature = "analysis")]
            | Self::Check(_) => &[],
        }
    }
    /// Return unresolved enrichment decisions for this change.
    pub fn decisions(&self) -> &[EnrichmentDecision] {
        match self {
            | Self::Document { decisions, .. } => decisions,
            #[cfg(feature = "analysis")]
            | Self::Check(_) => &[],
        }
    }
    fn has_linked_artifact(&self) -> bool {
        match self {
            | Self::Document { linked, .. } => linked.is_some(),
            #[cfg(feature = "analysis")]
            | Self::Check(_) => false,
        }
    }
}
#[cfg(feature = "analysis")]
impl<T> From<Check> for Change<T> {
    fn from(value: Check) -> Self {
        Self::Check(value)
    }
}
impl EnrichmentDecision {
    /// Validate a bounded decision and its candidate identities.
    pub fn new(
        provider: Provider,
        source: Location,
        field: String,
        original: String,
        reason: String,
        candidates: Vec<EnrichmentCandidate>,
    ) -> ApiResult<Self> {
        let valid_keys = candidates
            .iter()
            .all(|candidate| candidate.key.provider == provider && candidate.key.source == source && candidate.key.field == field);
        let unique = candidates
            .iter()
            .enumerate()
            .all(|(index, candidate)| candidates.iter().skip(index.saturating_add(1)).all(|other| other.key != candidate.key));
        match (field.is_empty(), candidates.len() > MAX_ENRICHMENT_CANDIDATES, valid_keys, unique) {
            | (false, false, true, true) => Ok(Self {
                candidates,
                field,
                original,
                provider,
                reason,
                source,
            }),
            | _ => Err(eyre!("Invalid enrichment decision or candidate identities")),
        }
    }
    /// Check that a choice belongs to this decision and its source value is unchanged.
    pub fn validate_selection(&self, selection: &EnrichmentSelection, current: &str) -> ApiResult<Option<&EnrichmentCandidate>> {
        match selection {
            | EnrichmentSelection::Skip => Ok(None),
            | EnrichmentSelection::Choose(_) if current != self.original => Err(eyre!("Enrichment source value changed before selection")),
            | EnrichmentSelection::Choose(key) => self
                .candidates
                .iter()
                .find(|candidate| &candidate.key == key)
                .map(Some)
                .ok_or_else(|| eyre!("Selected enrichment candidate is not in this decision")),
        }
    }
}
impl<T> Plan<T> {
    /// Keep planned companion artifacts while suppressing source-document rewrites.
    pub fn without_source_changes(self) -> Self {
        Self {
            additional: self.additional,
            #[cfg(feature = "exec")]
            audits: self.audits,
            changes: self
                .changes
                .into_iter()
                .map(|change| match change {
                    | Change::Document { .. } => change.with_changed(false),
                    #[cfg(feature = "analysis")]
                    | Change::Check(_) => change,
                })
                .collect(),
        }
    }
}
#[cfg(feature = "exec")]
impl<T> Plan<T>
where
    T: DeserializeOwned + InputOutput + Validate + Serialize,
{
    fn evaluate_script_hooks(
        mut self,
        configuration: &ApplicationConfiguration,
        project_root: &Path,
        hooks: &[WorkflowAction],
        origin: ExecutionOrigin,
        explicit_configuration: bool,
        offline: bool,
    ) -> ApiResult<Self> {
        configuration
            .prepare_scripts(project_root, hooks, origin, explicit_configuration)
            .and_then(|scripts| {
                hooks.iter().try_for_each(|hook| {
                    self.changes.iter_mut().enumerate().try_for_each(|(document_index, change)| match change {
                        | Change::Document {
                            source,
                            data,
                            content,
                            changed,
                            ..
                        } => scripts
                            .iter()
                            .filter(|script| script.resolved.entry.hooks.iter().any(|candidate| candidate == hook))
                            .try_for_each(|script| {
                                let source_name = source.uri().unwrap_or_else(|| source.to_string());
                                script
                                    .resolved
                                    .evaluate_hook(
                                        &source_name,
                                        data.as_ref(),
                                        HookOptions::init()
                                            .maybe_authorization(script.authorization.as_ref())
                                            .document_index(document_index)
                                            .hook(*hook)
                                            .maybe_manifest_source(script.manifest_source.as_deref())
                                            .offline(offline)
                                            .build(),
                                    )
                                    .and_then(|execution| {
                                        execution.checks.iter().for_each(|check| check.report(None));
                                        let failures = execution.checks.iter().filter(|check| check.is_failure()).count();
                                        match failures {
                                            | 0 => Ok(execution),
                                            | _ => Err(eyre!("configured Rhai quality gates found {failures} failure(s)")),
                                        }
                                    })
                                    .and_then(|execution| {
                                        let mutation_granted = script
                                            .authorization
                                            .as_ref()
                                            .is_some_and(|authorization| authorization.capabilities.mutation);
                                        match (execution.document, mutation_granted) {
                                            | (Some(_), false) => Err(eyre!("script document replacement requires the `mutation` capability")),
                                            | (Some(document), true) => serde_json::from_str::<T>(&document)
                                                .map_err(|why| eyre!("script returned an invalid typed document — {why}"))
                                                .and_then(|replacement| {
                                                    replacement
                                                        .validate()
                                                        .map_err(|why| eyre!("script document validation failed — {why}"))
                                                        .map(|()| replacement)
                                                })
                                                .and_then(|replacement| {
                                                    let format = source
                                                        .local_path()
                                                        .map(|path| MimeType::from_path(&path))
                                                        .unwrap_or_else(|| MimeType::from(source_name.as_str()));
                                                    replacement.serialize_as(&format).map(|serialized| (replacement, serialized))
                                                })
                                                .map(|(replacement, serialized)| {
                                                    **data = replacement;
                                                    *content = serialized;
                                                    *changed = true;
                                                }),
                                            | (None, _) => Ok(()),
                                        }
                                        .and_then(|()| {
                                            execution.files.into_iter().try_for_each(|proposal| {
                                                let proposal = Artifact::from(proposal);
                                                match self.additional.iter().find(|artifact| artifact.path == proposal.path) {
                                                    | Some(artifact) if artifact.content == proposal.content => Ok(()),
                                                    | Some(_) => Err(eyre!("conflicting file proposal for `{}`", proposal.path.display())),
                                                    | None => {
                                                        self.additional.push(proposal);
                                                        Ok(())
                                                    }
                                                }
                                            })
                                        })
                                        .map(|()| self.audits.push(execution.audit))
                                    })
                            }),
                        #[cfg(feature = "analysis")]
                        | Change::Check(_) => Ok(()),
                    })
                })
            })
            .map(|()| self)
    }
}
impl<T> Plan<T>
where
    T: InputOutput + Validate + Serialize + Clone + Send + Sync + 'static,
{
    /// Run configured quality gates before rendering and applying this plan.
    #[cfg(feature = "exec")]
    pub fn apply(
        self,
        options: Options,
        configuration: &ApplicationConfiguration,
        project_root: &Path,
        hooks: &[WorkflowAction],
        origin: ExecutionOrigin,
        explicit_configuration: bool,
    ) -> ApiResult<()>
    where
        T: DeserializeOwned,
    {
        self.evaluate_script_hooks(configuration, project_root, hooks, origin, explicit_configuration, options.common.offline)
            .and_then(|plan| plan.apply_enriched(options))
    }
    /// Render and apply an enrichment plan, reporting provider conflicts and failures.
    pub fn apply_enriched(self, options: Options) -> ApiResult<()> {
        let failures = self
            .changes
            .iter()
            .flat_map(|change| change.failures().iter().cloned())
            .collect::<Vec<_>>();
        let conflicts = self
            .changes
            .iter()
            .flat_map(|change| change.conflicts().iter().cloned())
            .collect::<Vec<_>>();
        self.changes
            .iter()
            .try_for_each(|change| change.render_enriched(&options))
            .and_then(|()| Workflow::<T>::apply(self, options).map(|_| ()))
            .and_then(|()| {
                conflicts.iter().for_each(|conflict| warn!("Enrichment conflict — {conflict}"));
                failures.iter().for_each(|failure| {
                    fail!("Enrichment provider — {failure}");
                });
                match failures.is_empty() {
                    | true => Ok(()),
                    | false => Err(eyre!("Enrichment completed with {} provider failure(s)", failures.len())),
                }
            })
    }
}
impl<T> Default for Workflow<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<T> Workflow<T>
where
    T: InputOutput + Validate + Serialize + Clone + Send + Sync + 'static,
{
    /// Plan processing for all sources without writing any files
    pub async fn plan(
        sources: &[Location],
        options: Options,
        enrichment: Option<&dyn Enrich<(Location, T), Output = ApiResult<EnrichmentResult<T>>>>,
    ) -> ApiResult<Plan<T>> {
        let RuntimeOptions { common, extension, .. } = &options;
        let CommonRuntime { offline, threads, .. } = common;
        let WorkflowExtension { enrich, .. } = extension;
        let concurrency = match enrich {
            | true => (*threads).max(1),
            | false => 1,
        };
        match (*enrich, *offline, enrichment.is_some()) {
            | (true, true, _) => Err(eyre!("Enrichment cannot run while the workflow is offline")),
            | (true, false, false) => Err(eyre!("Enrichment is enabled but no enrichment capability was supplied")),
            | _ => stream::iter(sources)
                .map(|source| Self::plan_source(source, &options, enrichment))
                .buffered(concurrency)
                .try_collect()
                .await
                .map(|changes| Plan {
                    additional: Vec::new(),
                    #[cfg(feature = "exec")]
                    audits: Vec::new(),
                    changes,
                }),
        }
    }
    /// Apply a previously prepared plan
    pub fn apply(plan: Plan<T>, options: Options) -> ApiResult<Report> {
        let report = Report {
            #[cfg(feature = "exec")]
            audits: plan.audits.clone(),
            processed: plan.changes.len(),
            changed: plan.changes.iter().filter(|change| change.is_changed()).count(),
            linked: plan.changes.iter().filter(|change| change.has_linked_artifact()).count(),
            ..Report::default()
        };
        let vocabulary_audits = plan
            .changes
            .iter()
            .filter_map(|change| match change {
                | Change::Document {
                    changed: true,
                    content,
                    original,
                    source,
                    vocabulary_reports,
                    ..
                } if !vocabulary_reports.is_empty() => Some((source.to_string(), original.clone(), content.clone(), vocabulary_reports.clone())),
                | _ => None,
            })
            .collect::<Vec<_>>();
        let vocabulary_rollbacks = plan
            .changes
            .iter()
            .filter_map(|change| match change {
                | Change::Document {
                    changed: true,
                    original,
                    source,
                    vocabulary_reports,
                    ..
                } if !vocabulary_reports.is_empty() => source.local_path().map(|path| Artifact {
                    content: original.clone(),
                    path,
                }),
                | _ => None,
            })
            .collect::<Vec<_>>();
        match options.common.dry_run {
            | true => Ok(report),
            | false => plan
                .changes
                .into_iter()
                .try_fold(plan.additional, |mut files, change| match change {
                    | Change::Document {
                        source,
                        content,
                        changed,
                        linked,
                        ..
                    } => match changed {
                        | true => source
                            .local_path()
                            .ok_or_else(|| eyre!("Workflow source is not an available local file: {source}"))
                            .map(|path| files.push(Artifact { path, content })),
                        | false => Ok(()),
                    }
                    .map(|()| {
                        files.extend(linked);
                        files
                    }),
                    #[cfg(feature = "analysis")]
                    | Change::Check(_) => Err(eyre!("A check must be resolved into a document change before it can be applied")),
                })
                .and_then(|files| transaction::Artifacts::from(files).apply())
                .and_then(|written| {
                    crate::io::vocabulary::write_vocabulary_report_audits(
                        vocabulary_audits
                            .iter()
                            .map(|(source, before, after, reports)| (source.as_str(), before.as_str(), after.as_str(), reports.as_slice())),
                    )
                    .map_err(|why| eyre!(why))
                    .map(|_| Report { written, ..report })
                    .or_else(|audit_error| {
                        Artifacts::from(vocabulary_rollbacks.clone())
                            .apply()
                            .map_err(|rollback_error| eyre!("{audit_error}; rollback failed — {rollback_error}"))
                            .and_then(|_| Err(audit_error))
                    })
                }),
        }
    }
    /// Plan and apply processing for all sources
    pub async fn run(
        sources: &[Location],
        options: Options,
        enrichment: Option<&dyn Enrich<(Location, T), Output = ApiResult<EnrichmentResult<T>>>>,
    ) -> ApiResult<Report> {
        let apply_options = options.clone();
        Self::plan(sources, options, enrichment)
            .await
            .and_then(|plan| Self::apply(plan, apply_options))
    }
    async fn plan_source(
        source: &Location,
        options: &Options,
        enrichment: Option<&dyn Enrich<(Location, T), Output = ApiResult<EnrichmentResult<T>>>>,
    ) -> ApiResult<Change<T>> {
        let WorkflowExtension {
            check,
            enrich,
            format,
            fuzzy,
            link,
            ..
        } = &options.extension;
        let prepared = source
            .local_path()
            .ok_or_else(|| eyre!("Workflow source is not an available local file: {source}"))
            .and_then(|path| {
                read_file(path.clone()).and_then(|content| {
                    T::read(path.clone()).map(|data| {
                        let formatted = match format {
                            | true => InputOutput::format_with_vocabulary_report(data, *fuzzy, Some(path.clone())),
                            | false => VocabularyFormat {
                                reports: Vec::new(),
                                value: data,
                            },
                        };
                        (path, content, formatted.value, formatted.reports)
                    })
                })
            })
            .and_then(|(path, content, data, reports)| match check {
                | true => data
                    .validate()
                    .map_err(|why| eyre!("Workflow data at {} is invalid — {why}", path.display()))
                    .map(|()| (path, content, data, reports)),
                | false => Ok((path, content, data, reports)),
            });
        match prepared {
            | Ok((path, original, data, mut vocabulary_reports)) => {
                let enriched = match (*enrich, enrichment) {
                    | (true, Some(capability)) => capability.enrich((source.clone(), data)).await,
                    | _ => Ok(EnrichmentResult::builder().data(data).build()),
                };
                enriched
                    .map(|result| {
                        let EnrichmentResult {
                            data,
                            conflicts,
                            decisions,
                            failures,
                        } = result;
                        let formatted = match format {
                            | true => InputOutput::format_with_vocabulary_report(data, *fuzzy, Some(path.clone())),
                            | false => VocabularyFormat {
                                reports: Vec::new(),
                                value: data,
                            },
                        };
                        vocabulary_reports.extend(formatted.reports);
                        (
                            EnrichmentResult::builder()
                                .data(formatted.value)
                                .conflicts(conflicts)
                                .decisions(decisions)
                                .failures(failures)
                                .build(),
                            vocabulary_reports,
                        )
                    })
                    .and_then(|(result, vocabulary_reports)| match check {
                        | true => result
                            .data
                            .validate()
                            .map(|()| (result, vocabulary_reports))
                            .map_err(|why| eyre!("Workflow data at {} is invalid — {why}", path.display())),
                        | false => Ok((result, vocabulary_reports)),
                    })
                    .and_then(|(result, vocabulary_reports)| {
                        let EnrichmentResult {
                            data,
                            conflicts,
                            decisions,
                            failures,
                        } = result;
                        InputOutput::serialize_as(&data, &MimeType::from_path(&path)).and_then(|content| {
                            let linked = match link {
                                | true => data.linked_content().map(|content| {
                                    content.map(|content| Artifact {
                                        path: path.with_extension("jsonld"),
                                        content,
                                    })
                                }),
                                | false => Ok(None),
                            };
                            linked.map(|linked| Change::Document {
                                changed: original != content,
                                conflicts,
                                decisions,
                                content,
                                data: Box::new(data),
                                failures,
                                linked,
                                original,
                                source: source.clone(),
                                vocabulary_reports,
                            })
                        })
                    })
            }
            | Err(why) => Err(why),
        }
    }
}
impl Default for WorkflowExtension {
    fn default() -> Self {
        Self {
            check: true,
            enrich: false,
            format: false,
            fuzzy: acorn_schema::vocabulary::VocabularyFuzzyMode::Strict,
            link: false,
            show_changes: false,
        }
    }
}

#[cfg(test)]
mod tests;
