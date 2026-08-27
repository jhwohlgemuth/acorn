//! Plan and apply repeatable processing to research activity data files.
//!
//! The workflow keeps provider-specific enrichment behind [`Enrichment`] and
//! separates transformation planning from filesystem writes. Callers can use
//! [`Workflow::plan`] for previews or [`Workflow::run`] to plan and apply in one
//! operation.

use crate::io::{read_file, write_file, ApiResult, FromPath, InputOutput};
use crate::prelude::{Path, PathBuf, String, Vec};
use crate::schema::research_activity::ResearchActivity;
use crate::util::{LinkedData, MimeType};
use crate::validation::Validate;
use async_trait::async_trait;
use bon::Builder;
use color_eyre::eyre::eyre;

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
}

impl Default for Options {
    fn default() -> Self {
        Self::init().build()
    }
}

/// A research activity data source and its optional bucket identity.
#[derive(Builder, Clone, Debug, Eq, PartialEq)]
#[builder(start_fn = init, on(String, into))]
pub struct Source {
    /// Local path to the research activity data file.
    pub path: PathBuf,
    /// Bucket name or identifier containing the source.
    pub bucket: Option<String>,
    /// Stable bucket-relative object key, when known.
    pub key: Option<String>,
}

/// Provider-neutral capability for enriching one research activity.
#[async_trait]
pub trait Enrichment: Send + Sync {
    /// Return enriched data without writing it to storage.
    async fn enrich(&self, source: &Source, data: ResearchActivity) -> ApiResult<ResearchActivity>;
}

/// One planned change to a research activity data source.
#[derive(Clone, Debug)]
pub struct Change {
    /// Source represented by this change.
    pub source: Source,
    /// Parsed and transformed research activity data.
    pub data: ResearchActivity,
    /// Serialized content in the source file's format.
    pub content: String,
    /// Whether applying the plan will rewrite the source file.
    pub changed: bool,
    /// Optional JSON-LD companion artifact.
    pub linked: Option<Artifact>,
}

/// A generated workflow artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artifact {
    /// Destination path for the artifact.
    pub path: PathBuf,
    /// Serialized artifact content.
    pub content: String,
}

/// A complete, validated set of changes prepared before any writes occur.
#[derive(Clone, Debug, Default)]
pub struct Plan {
    /// Planned source changes and companion artifacts.
    pub changes: Vec<Change>,
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

/// Service for planning and applying research activity data workflows.
#[derive(Clone, Copy, Debug, Default)]
pub struct Workflow;

impl Workflow {
    /// Plan processing for all sources without writing any files.
    ///
    /// # Errors
    ///
    /// Returns an error when a source cannot be read, parsed, transformed,
    /// serialized, or validated, or when enrichment is enabled without an
    /// available capability.
    pub async fn plan(sources: &[Source], options: Options, enrichment: Option<&dyn Enrichment>) -> ApiResult<Plan> {
        if options.enrich && options.offline {
            return Err(eyre!("Enrichment cannot run while the workflow is offline"));
        }
        if options.enrich && enrichment.is_none() {
            return Err(eyre!("Enrichment is enabled but no enrichment capability was supplied"));
        }
        let mut changes = Vec::with_capacity(sources.len());
        for source in sources {
            let original = match read_file(source.path.clone()) {
                | Ok(value) => value,
                | Err(why) => return Err(why),
            };
            let mut data = match ResearchActivity::read(source.path.clone()) {
                | Ok(value) => value,
                | Err(why) => return Err(why),
            };
            if options.check {
                match validate(&data, &source.path) {
                    | Ok(()) => {}
                    | Err(why) => return Err(why),
                }
            }
            if options.format {
                data = data.format_with(Some(source.path.clone()));
            }
            if options.enrich {
                let capability = match enrichment {
                    | Some(value) => value,
                    | None => return Err(eyre!("Enrichment capability is unavailable")),
                };
                data = match capability.enrich(source, data).await {
                    | Ok(value) => value,
                    | Err(why) => return Err(why),
                };
                if options.format {
                    data = data.format_with(Some(source.path.clone()));
                }
            }
            if options.check {
                match validate(&data, &source.path) {
                    | Ok(()) => {}
                    | Err(why) => return Err(why),
                }
            }
            let mime = MimeType::from_path(&source.path);
            let content = match data.serialize_as(&mime) {
                | Ok(value) => value,
                | Err(why) => return Err(why),
            };
            let linked = if options.link {
                match serde_json::to_string_pretty(&data.with_context()) {
                    | Ok(content) => Some(Artifact {
                        path: source.path.with_extension("jsonld"),
                        content,
                    }),
                    | Err(why) => return Err(eyre!("Failed to serialize linked research activity data — {why}")),
                }
            } else {
                None
            };
            changes.push(Change {
                source: source.clone(),
                changed: original != content,
                data,
                content,
                linked,
            });
        }
        Ok(Plan { changes })
    }

    /// Apply a previously prepared plan.
    ///
    /// # Errors
    ///
    /// Returns an error when a planned source or companion artifact cannot be
    /// written.
    pub fn apply(plan: Plan, options: Options) -> ApiResult<Report> {
        let mut report = Report {
            processed: plan.changes.len(),
            changed: plan.changes.iter().filter(|change| change.changed).count(),
            linked: plan.changes.iter().filter(|change| change.linked.is_some()).count(),
            ..Report::default()
        };
        if options.dry_run {
            return Ok(report);
        }
        for change in plan.changes {
            if change.changed {
                match write_file(change.source.path, change.content) {
                    | Ok(()) => report.written = report.written.saturating_add(1),
                    | Err(why) => return Err(why),
                }
            }
            if let Some(artifact) = change.linked {
                match write_file(artifact.path, artifact.content) {
                    | Ok(()) => report.written = report.written.saturating_add(1),
                    | Err(why) => return Err(why),
                }
            }
        }
        Ok(report)
    }

    /// Plan and apply processing for all sources.
    ///
    /// # Errors
    ///
    /// Returns any planning, validation, enrichment, serialization, or write
    /// error encountered by the workflow.
    pub async fn run(sources: &[Source], options: Options, enrichment: Option<&dyn Enrichment>) -> ApiResult<Report> {
        let apply_options = options.clone();
        match Self::plan(sources, options, enrichment).await {
            | Ok(plan) => Self::apply(plan, apply_options),
            | Err(why) => Err(why),
        }
    }
}

fn validate(data: &ResearchActivity, path: &Path) -> ApiResult<()> {
    data.validate()
        .map_err(|why| eyre!("Research activity data at {} is invalid — {why}", path.display()))
}

#[cfg(test)]
mod tests;
