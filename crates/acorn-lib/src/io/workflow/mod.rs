//! Plan and apply repeatable processing to supported metadata standards.
//!
//! The workflow keeps provider-specific enrichment behind [`Enrich`](crate::io::enrichment::Enrich) and separates transformation planning from filesystem writes.
//! Callers can use [`Workflow::plan`] for previews or [`Workflow::run`] to plan and apply in one operation.
#[cfg(feature = "analysis")]
use crate::analyzer::Check;
use crate::io::enrichment::Enrich;
use crate::io::{read_file, write_file, ApiResult, FromPath, InputOutput};
use crate::util::{print_changes_with_color, Label, StringConversion};
use crate::LocationExt;
use acorn_core::prelude::{Box, String, Vec};
use acorn_core::util::MimeType;
use acorn_core::Location;
use acorn_schema::validation::Validate;
use bon::Builder;
use color_eyre::eyre::eyre;
use core::marker::PhantomData;
use futures::stream::{self, StreamExt, TryStreamExt};
use owo_colors::OwoColorize;
use serde::Serialize;
use std::path::PathBuf;
use tracing::warn;

/// One planned change to a supported data source.
#[derive(Clone, Debug)]
pub enum Change<T> {
    /// A fully planned document rewrite and its optional companion artifact.
    Document {
        /// Source represented by this change.
        source: Location,
        /// Parsed and transformed standard data.
        data: Box<T>,
        /// Serialized content in the source file's format.
        content: String,
        /// Whether applying the plan will rewrite the source file.
        changed: bool,
        /// Optional JSON-LD companion artifact.
        linked: Option<Artifact>,
        /// Provider failures encountered while producing this change.
        failures: Vec<String>,
        /// Non-fatal enrichment conflicts encountered while producing this change.
        conflicts: Vec<String>,
    },
    /// A check retained as the input to a future automated fix.
    #[cfg(feature = "analysis")]
    Check(Check),
}
/// Result of applying one or more enrichment providers.
#[derive(Clone, Debug, Default)]
pub struct EnrichmentResult<T> {
    /// Enriched metadata.
    pub data: T,
    /// Existing-value or ambiguous-match conflicts that were left unchanged.
    pub conflicts: Vec<String>,
    /// Non-fatal provider failures encountered after other providers continued.
    pub failures: Vec<String>,
}
/// A generated workflow artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artifact {
    /// Destination path for the artifact.
    pub path: PathBuf,
    /// Serialized artifact content.
    pub content: String,
}
/// Options shared by workflow interfaces.
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init)]
pub struct Options {
    /// Validate input and transformed research activity data.
    #[builder(default = true)]
    pub check: bool,
    /// Normalize research activity data before output.
    #[builder(default)]
    pub format: bool,
    /// Invoke the supplied enrichment capability.
    #[builder(default)]
    pub enrich: bool,
    /// Create a JSON-LD companion file.
    #[builder(default)]
    pub link: bool,
    /// Preview planned changes without writing them.
    #[builder(default)]
    pub dry_run: bool,
    /// Prohibit network-dependent enrichment.
    #[builder(default)]
    pub offline: bool,
    /// Disable color in rendered change previews.
    #[builder(default)]
    pub no_color: bool,
    /// Render changes even when the workflow will write them.
    #[builder(default)]
    pub show_changes: bool,
}
/// A complete, validated set of changes prepared before any writes occur.
#[derive(Clone, Debug, Default)]
pub struct Plan<T> {
    /// Planned source changes and companion artifacts.
    pub changes: Vec<Change<T>>,
}
/// Summary of a workflow application.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Report {
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
impl<T> Default for Workflow<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<T> Change<T>
where
    T: InputOutput + Serialize,
{
    /// Render the changes produced by enrichment without applying them.
    pub fn render_enriched(&self, dry_run: bool, no_color: bool, show_changes: bool) -> ApiResult<()> {
        match self {
            | Self::Document { source, content, .. } => match source.local_path() {
                | Some(path) => T::read(path.clone()).and_then(|data| {
                    InputOutput::serialize_as(&data, &MimeType::from_path(&path)).map(|old_content| {
                        if dry_run {
                            let absolute = path.to_absolute_path();
                            match no_color {
                                | true => println!("{} Enrich and format {absolute}", Label::DRY_RUN),
                                | false => println!("{} Enrich and format {}", Label::dry_run(), absolute.yellow()),
                            }
                        }
                        if dry_run || show_changes {
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
    fn has_linked_artifact(&self) -> bool {
        match self {
            | Self::Document { linked, .. } => linked.is_some(),
            #[cfg(feature = "analysis")]
            | Self::Check(_) => false,
        }
    }
}
impl<T> Plan<T>
where
    T: InputOutput + Validate + Serialize + Clone + Send + Sync + 'static,
{
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
            .try_for_each(|change| change.render_enriched(options.dry_run, options.no_color, options.show_changes))
            .and_then(|()| Workflow::<T>::apply(self, options).map(|_| ()))
            .and_then(|()| {
                conflicts.iter().for_each(|conflict| warn!("Enrichment conflict — {conflict}"));
                failures.iter().for_each(|failure| {
                    crate::fail!("Enrichment provider — {}", failure);
                });
                match failures.is_empty() {
                    | true => Ok(()),
                    | false => Err(eyre!("Enrichment completed with {} provider failure(s)", failures.len())),
                }
            })
    }
}
#[cfg(feature = "analysis")]
impl<T> From<Check> for Change<T> {
    fn from(value: Check) -> Self {
        Self::Check(value)
    }
}
impl Default for Options {
    fn default() -> Self {
        Self::init().build()
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
        match (options.enrich, options.offline, enrichment.is_some()) {
            | (true, true, _) => Err(eyre!("Enrichment cannot run while the workflow is offline")),
            | (true, false, false) => Err(eyre!("Enrichment is enabled but no enrichment capability was supplied")),
            | _ => stream::iter(sources)
                .then(|source| Self::plan_source(source, &options, enrichment))
                .try_collect()
                .await
                .map(|changes| Plan { changes }),
        }
    }
    /// Apply a previously prepared plan
    pub fn apply(plan: Plan<T>, options: Options) -> ApiResult<Report> {
        let report = Report {
            processed: plan.changes.len(),
            changed: plan.changes.iter().filter(|change| change.is_changed()).count(),
            linked: plan.changes.iter().filter(|change| change.has_linked_artifact()).count(),
            ..Report::default()
        };
        match options.dry_run {
            | true => Ok(report),
            | false => plan.changes.into_iter().try_fold(report, |report, change| match change {
                | Change::Document {
                    source,
                    content,
                    changed,
                    linked,
                    ..
                } => source
                    .local_path()
                    .ok_or_else(|| eyre!("Workflow source is not an available local file: {source}"))
                    .and_then(|path| match changed {
                        | true => write_file(path, content).map(|()| 1),
                        | false => Ok(0),
                    })
                    .and_then(|source_written| {
                        linked
                            .map(|artifact| write_file(artifact.path, artifact.content).map(|()| 1))
                            .transpose()
                            .map(|linked_written| Report {
                                written: report
                                    .written
                                    .saturating_add(source_written)
                                    .saturating_add(linked_written.unwrap_or_default()),
                                ..report
                            })
                    }),
                #[cfg(feature = "analysis")]
                | Change::Check(_) => Err(eyre!("A check must be resolved into a document change before it can be applied")),
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
        let prepared = source
            .local_path()
            .ok_or_else(|| eyre!("Workflow source is not an available local file: {source}"))
            .and_then(|path| {
                read_file(path.clone()).and_then(|content| {
                    T::read(path.clone()).map(|data| {
                        let data = match options.format {
                            | true => InputOutput::format_with(data, Some(path.clone())),
                            | false => data,
                        };
                        (path, content, data)
                    })
                })
            })
            .and_then(|(path, content, data)| match options.check {
                | true => data
                    .validate()
                    .map_err(|why| eyre!("Workflow data at {} is invalid — {why}", path.display()))
                    .map(|()| (path, content, data)),
                | false => Ok((path, content, data)),
            });
        match prepared {
            | Ok((path, original, data)) => {
                let enriched = match (options.enrich, enrichment) {
                    | (true, Some(capability)) => capability.enrich((source.clone(), data)).await,
                    | _ => Ok(EnrichmentResult {
                        data,
                        conflicts: Vec::new(),
                        failures: Vec::new(),
                    }),
                };
                enriched
                    .map(|result| match options.format {
                        | true => EnrichmentResult {
                            data: InputOutput::format_with(result.data, Some(path.clone())),
                            ..result
                        },
                        | false => result,
                    })
                    .and_then(|result| match options.check {
                        | true => result
                            .data
                            .validate()
                            .map(|()| result)
                            .map_err(|why| eyre!("Workflow data at {} is invalid — {why}", path.display())),
                        | false => Ok(result),
                    })
                    .and_then(|result| {
                        let EnrichmentResult { data, conflicts, failures } = result;
                        InputOutput::serialize_as(&data, &MimeType::from_path(&path)).and_then(|content| {
                            let linked = match options.link {
                                | true => data.linked_content().map(|content| {
                                    content.map(|content| Artifact {
                                        path: path.with_extension("jsonld"),
                                        content,
                                    })
                                }),
                                | false => Ok(None),
                            };
                            linked.map(|linked| Change::Document {
                                source: source.clone(),
                                changed: original != content,
                                data: Box::new(data),
                                content,
                                conflicts,
                                failures,
                                linked,
                            })
                        })
                    })
            }
            | Err(why) => Err(why),
        }
    }
}
#[cfg(test)]
mod tests;
