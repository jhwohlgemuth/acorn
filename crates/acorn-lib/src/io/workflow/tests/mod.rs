use super::transaction::{Artifacts, StagedFile, StagedFiles};
use super::*;
#[cfg(feature = "analysis")]
use crate::analyzer::{Check, CheckCategory};
use acorn_core::options::CommonRuntime;
use acorn_schema::research_activity::{ResearchActivity, ResearchActivityMetadata, ResearchOutput};
use acorn_schema::standard::cff::Cff;
use acorn_schema::OneOrMany;
use async_trait::async_trait;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use std::fs;
use tempfile::tempdir;

struct CffEnrichment;
struct OutputEnrichment;
#[derive(Default)]
struct TrackingEnrichment {
    active: AtomicUsize,
    maximum: AtomicUsize,
}
#[async_trait]
impl Enrich<(Location, Cff)> for CffEnrichment {
    type Output = ApiResult<EnrichmentResult<Cff>>;
    async fn enrich(&self, (_, data): (Location, Cff)) -> Self::Output {
        Ok(EnrichmentResult {
            data: Cff {
                title: "Enriched CFF".to_string(),
                ..data
            },
            conflicts: Vec::new(),
            failures: Vec::new(),
        })
    }
}
#[async_trait]
impl Enrich<(Location, ResearchActivity)> for OutputEnrichment {
    type Output = ApiResult<EnrichmentResult<ResearchActivity>>;
    async fn enrich(&self, (_, data): (Location, ResearchActivity)) -> Self::Output {
        let output = ResearchOutput::init()
            .identifier("https://openalex.org/W2741809807".to_string())
            .doi("10.1038/nature12373".to_string())
            .title("Enriched output".to_string())
            .kind("article".to_string())
            .build();
        Ok(EnrichmentResult {
            data: ResearchActivity {
                meta: ResearchActivityMetadata {
                    outputs: Some(vec![output].into()),
                    ..data.meta
                },
                ..data
            },
            conflicts: Vec::new(),
            failures: Vec::new(),
        })
    }
}
impl TrackingEnrichment {
    fn maximum(&self) -> usize {
        self.maximum.load(Ordering::SeqCst)
    }
}
#[async_trait]
impl Enrich<(Location, ResearchActivity)> for TrackingEnrichment {
    type Output = ApiResult<EnrichmentResult<ResearchActivity>>;
    async fn enrich(&self, (source, data): (Location, ResearchActivity)) -> Self::Output {
        let active = self.active.fetch_add(1, Ordering::SeqCst).saturating_add(1);
        self.maximum.fetch_max(active, Ordering::SeqCst);
        let delay = source
            .local_path()
            .and_then(|path| {
                path.file_stem()
                    .and_then(|value| value.to_str())
                    .and_then(|value| value.parse::<u64>().ok())
            })
            .unwrap_or_default();
        tokio::time::sleep(Duration::from_millis(delay)).await;
        self.active.fetch_sub(1, Ordering::SeqCst);
        Ok(EnrichmentResult {
            data,
            conflicts: Vec::new(),
            failures: Vec::new(),
        })
    }
}
fn options(common: CommonRuntime, extension: WorkflowExtension) -> Options {
    Options::init().common(common).extension(extension).build()
}
fn source_file() -> (tempfile::TempDir, Location) {
    let temporary = tempdir().expect("temporary workflow directory should be created");
    let path = temporary.path().join("activity.json");
    ResearchActivity::default()
        .write_json(path.clone())
        .expect("research activity fixture should be written");
    let source = Location::from(&path.display().to_string());
    (temporary, source)
}
#[cfg(feature = "analysis")]
#[test]
fn test_change_preserves_check_for_future_fix_resolution() {
    let change: Change<ResearchActivity> = Change::from(
        Check::init()
            .category(CheckCategory::Quality)
            .message("Normalize the output title")
            .build(),
    );
    assert!(matches!(change, Change::Check(_)));
    assert!(!change.is_changed());
    assert!(change.data().is_none());
}
#[test]
fn test_default_options_only_check() {
    let options = Options::default();
    assert!(options.extension.check);
    assert!(!options.extension.format);
    assert!(!options.extension.enrich);
    assert!(!options.extension.link);
    assert!(!options.common.dry_run);
    assert!(!options.common.offline);
    assert!(!options.common.no_color);
    assert!(!options.extension.show_changes);
}
#[tokio::test]
async fn test_dry_run_applies_no_changes() {
    let (_temporary, source) = source_file();
    let path = source.local_path().expect("source should have a local path");
    let before = read_file(path.clone()).expect("source should be readable");
    let plan_options = options(
        CommonRuntime::default(),
        WorkflowExtension {
            enrich: true,
            link: true,
            ..WorkflowExtension::default()
        },
    );
    let plan = Workflow::<ResearchActivity>::plan(core::slice::from_ref(&source), plan_options, Some(&OutputEnrichment))
        .await
        .expect("workflow plan should succeed");
    let report = Workflow::<ResearchActivity>::apply(
        plan,
        options(
            CommonRuntime {
                dry_run: true,
                ..CommonRuntime::default()
            },
            WorkflowExtension::default(),
        ),
    )
    .expect("dry run should succeed");
    assert_eq!(report.processed, 1);
    assert_eq!(report.changed, 1);
    assert_eq!(report.linked, 1);
    assert_eq!(report.written, 0);
    assert_eq!(read_file(path.clone()).expect("source should remain readable"), before);
    assert!(!path.with_extension("jsonld").exists());
}
#[tokio::test]
async fn test_enrichment_requires_capability_and_online_mode() {
    let (_temporary, source) = source_file();
    let extension = WorkflowExtension {
        enrich: true,
        ..WorkflowExtension::default()
    };
    let missing =
        Workflow::<ResearchActivity>::plan(core::slice::from_ref(&source), options(CommonRuntime::default(), extension.clone()), None).await;
    let offline = Workflow::<ResearchActivity>::plan(
        &[source],
        options(
            CommonRuntime {
                offline: true,
                ..CommonRuntime::default()
            },
            extension,
        ),
        Some(&OutputEnrichment),
    )
    .await;
    assert!(missing.is_err());
    assert!(offline.is_err());
}
#[tokio::test]
async fn test_parallel_enrichment_honors_limit_and_preserves_order() {
    let temporary = tempdir().expect("temporary workflow directory should be created");
    let sources = [50_u64, 20, 1]
        .into_iter()
        .map(|delay| {
            let path = temporary.path().join(format!("{delay}.json"));
            ResearchActivity::default()
                .write_json(path.clone())
                .expect("research activity fixture should be written");
            Location::from(&path.display().to_string())
        })
        .collect::<Vec<_>>();
    let enrichment = TrackingEnrichment::default();
    let plan = Workflow::<ResearchActivity>::plan(
        &sources,
        options(
            CommonRuntime {
                threads: 2,
                ..CommonRuntime::default()
            },
            WorkflowExtension {
                enrich: true,
                ..WorkflowExtension::default()
            },
        ),
        Some(&enrichment),
    )
    .await
    .expect("parallel workflow plan should succeed");
    let planned_sources = plan
        .changes
        .iter()
        .filter_map(|change| match change {
            | Change::Document { source, .. } => Some(source.to_string()),
            #[cfg(feature = "analysis")]
            | Change::Check(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(enrichment.maximum(), 2);
    assert_eq!(planned_sources, sources.iter().map(ToString::to_string).collect::<Vec<_>>());
}
#[tokio::test]
async fn test_parallel_enrichment_treats_zero_threads_as_one() {
    let temporary = tempdir().expect("temporary workflow directory should be created");
    let sources = [20_u64, 10]
        .into_iter()
        .map(|delay| {
            let path = temporary.path().join(format!("{delay}.json"));
            ResearchActivity::default()
                .write_json(path.clone())
                .expect("research activity fixture should be written");
            Location::from(&path.display().to_string())
        })
        .collect::<Vec<_>>();
    let enrichment = TrackingEnrichment::default();
    let plan = Workflow::<ResearchActivity>::plan(
        &sources,
        options(
            CommonRuntime {
                threads: 0,
                ..CommonRuntime::default()
            },
            WorkflowExtension {
                enrich: true,
                ..WorkflowExtension::default()
            },
        ),
        Some(&enrichment),
    )
    .await;
    assert!(plan.is_ok());
    assert_eq!(enrichment.maximum(), 1);
}
#[tokio::test]
async fn test_plan_enriches_without_writing_source() {
    let (_temporary, source) = source_file();
    let path = source.local_path().expect("source should have a local path");
    let before = read_file(path.clone()).expect("source should be readable");
    let options = options(
        CommonRuntime::default(),
        WorkflowExtension {
            enrich: true,
            ..WorkflowExtension::default()
        },
    );
    let plan = Workflow::<ResearchActivity>::plan(core::slice::from_ref(&source), options, Some(&OutputEnrichment))
        .await
        .expect("workflow plan should succeed");
    let after = read_file(path).expect("source should remain readable");
    assert_eq!(before, after);
    assert_eq!(plan.changes.len(), 1);
    let change = plan.changes.first().expect("plan should contain one change");
    assert!(change.is_changed());
    assert_eq!(change.data().and_then(|data| data.meta.outputs.as_ref().map(OneOrMany::len)), Some(1));
}
#[test]
fn test_repository_accepts_every_registered_flat_file_format() {
    let activity = serde_json::to_string(&ResearchActivity::default()).unwrap();
    let files = vec![
        RepositoryFileInput {
            content: "cff-version: 1.2.0\nmessage: Cite this work.\ntitle: Example\nauthors:\n  - family-names: Doe\n".to_string(),
            format: Some(MimeType::Cff),
            path: "CITATION.cff".to_string(),
        },
        RepositoryFileInput {
            content: "{\"value\":1}".to_string(),
            format: Some(MimeType::Json),
            path: "metadata.json".to_string(),
        },
        RepositoryFileInput {
            content: "// note\n{\"value\":1}".to_string(),
            format: Some(MimeType::Jsonc),
            path: "metadata.jsonc".to_string(),
        },
        RepositoryFileInput {
            content: "# Research notes\n".to_string(),
            format: Some(MimeType::Markdown),
            path: "notes.md".to_string(),
        },
        RepositoryFileInput {
            content: activity,
            format: Some(MimeType::Json),
            path: "activity.rad.json".to_string(),
        },
        RepositoryFileInput {
            content: "value: 1\n".to_string(),
            format: Some(MimeType::Yaml),
            path: "metadata.yaml".to_string(),
        },
    ];
    let input = RepositoryInput {
        actions: vec![WorkflowAction::Format, WorkflowAction::Validate],
        files,
        project: "30".to_string(),
        revision: "abc123".to_string(),
    };
    let result = RepositoryWorkflowRegistry::acorn().unwrap().process("repository-quality", input).unwrap();
    assert_eq!(result.reports.len(), 6);
    assert_eq!(result.reports.iter().map(|report| report.format.clone()).collect::<Vec<_>>().len(), 6);
}
#[test]
fn test_repository_processes_supported_flat_files_deterministically() {
    let input = RepositoryInput {
        actions: vec![WorkflowAction::Format, WorkflowAction::Validate],
        files: vec![
            RepositoryFileInput {
                content: "cff-version: 1.2.0\nmessage: Cite this work.\ntitle: Example\nauthors:\n  - family-names: Doe\n".to_string(),
                format: Some(MimeType::Cff),
                path: "CITATION.cff".to_string(),
            },
            RepositoryFileInput {
                content: "{\"z\":1,\"a\":2}".to_string(),
                format: Some(MimeType::Json),
                path: "metadata.json".to_string(),
            },
        ],
        project: "30".to_string(),
        revision: "abc123".to_string(),
    };
    let result = RepositoryWorkflowRegistry::acorn().unwrap().process("repository-quality", input).unwrap();
    assert_eq!(result.base_revision, "abc123");
    assert_eq!(result.reports.first().map(|report| report.path.as_str()), Some("CITATION.cff"));
    assert_eq!(result.reports.get(1).map(|report| report.path.as_str()), Some("metadata.json"));
    assert_eq!(result.files.len(), 2);
}
#[test]
fn test_repository_rejects_unknown_workflows_unsafe_paths_and_unconfigured_enrichment() {
    let registry = RepositoryWorkflowRegistry::acorn().unwrap();
    let base = RepositoryInput {
        actions: vec![WorkflowAction::Validate],
        files: vec![RepositoryFileInput {
            content: "# Example\n".to_string(),
            format: Some(MimeType::Markdown),
            path: "../README.md".to_string(),
        }],
        project: "30".to_string(),
        revision: "abc123".to_string(),
    };
    assert!(registry.process("missing", base.clone()).is_err());
    assert!(registry.process("repository-quality", base.clone()).is_err());
    let enrichment = RepositoryInput {
        actions: vec![WorkflowAction::Enrich],
        files: vec![RepositoryFileInput {
            content: "# Example\n".to_string(),
            format: Some(MimeType::Markdown),
            path: "README.md".to_string(),
        }],
        ..base
    };
    assert!(registry.process("repository-quality", enrichment).is_err());
}
#[tokio::test]
async fn test_run_writes_enrichment_and_linked_artifact() {
    let (_temporary, source) = source_file();
    let options = options(
        CommonRuntime::default(),
        WorkflowExtension {
            enrich: true,
            link: true,
            ..WorkflowExtension::default()
        },
    );
    let report = Workflow::<ResearchActivity>::run(core::slice::from_ref(&source), options, Some(&OutputEnrichment))
        .await
        .expect("workflow run should succeed");
    let path = source.local_path().expect("source should have a local path");
    let written = ResearchActivity::read(path.clone()).expect("updated source should parse");
    assert_eq!(written.meta.outputs.as_ref().map(OneOrMany::len), Some(1));
    assert!(path.with_extension("jsonld").exists());
    assert_eq!(report.processed, 1);
    assert_eq!(report.changed, 1);
    assert_eq!(report.linked, 1);
    assert_eq!(report.written, 2);
}
#[test]
fn test_transaction_rejects_conflicting_destinations_before_writing() {
    let temporary = tempdir().expect("temporary directory");
    let target = temporary.path().join("target.txt");
    let result = Artifacts::from(vec![
        Artifact {
            content: "first".to_string(),
            path: target.clone(),
        },
        Artifact {
            content: "second".to_string(),
            path: target.clone(),
        },
    ])
    .apply();
    assert!(result.is_err());
    assert!(!target.exists());
}
#[test]
fn test_transaction_replaces_all_staged_files() {
    let temporary = tempdir().expect("temporary directory");
    let first = temporary.path().join("first.txt");
    let second = temporary.path().join("second.txt");
    fs::write(&first, "old").expect("existing file");
    let written = Artifacts::from(vec![
        Artifact {
            content: "new".to_string(),
            path: first.clone(),
        },
        Artifact {
            content: "created".to_string(),
            path: second.clone(),
        },
    ])
    .apply()
    .expect("transaction");
    assert_eq!(written, 2);
    assert_eq!(fs::read_to_string(first).expect("replacement"), "new");
    assert_eq!(fs::read_to_string(second).expect("creation"), "created");
}
#[test]
fn test_transaction_rolls_back_a_committed_replacement() {
    let temporary = tempdir().expect("temporary directory");
    let first = temporary.path().join("first.txt");
    let first_stage = temporary.path().join("first.stage");
    let first_backup = temporary.path().join("first.backup");
    let second = temporary.path().join("second.txt");
    let missing_stage = temporary.path().join("missing.stage");
    fs::write(&first, "old").expect("existing file");
    fs::write(&first_stage, "new").expect("first stage");
    let result = StagedFiles::from(vec![
        StagedFile {
            backup: Some(first_backup.clone()),
            stage: first_stage,
            target: first.clone(),
        },
        StagedFile {
            backup: None,
            stage: missing_stage,
            target: second.clone(),
        },
    ])
    .commit();
    assert!(result.is_err());
    assert_eq!(fs::read_to_string(first).expect("rolled back file"), "old");
    assert!(!first_backup.exists());
    assert!(!second.exists());
}
#[tokio::test]
async fn test_workflow_processes_and_enriches_non_rad_standard() {
    let temporary = tempdir().expect("temporary workflow directory should be created");
    let path = temporary.path().join("CITATION.cff");
    Cff {
        title: "Original CFF".to_string(),
        ..Cff::default()
    }
    .write_cff(path.clone())
    .expect("CFF fixture should be written");
    let source = Location::from(&path.display().to_string());
    let options = options(
        CommonRuntime::default(),
        WorkflowExtension {
            check: false,
            enrich: true,
            ..WorkflowExtension::default()
        },
    );
    let report = Workflow::<Cff>::run(core::slice::from_ref(&source), options, Some(&CffEnrichment))
        .await
        .expect("generic workflow should process CFF");
    let written = Cff::read(path).expect("updated CFF should parse");
    assert_eq!(written.title, "Enriched CFF");
    assert_eq!(report.processed, 1);
    assert_eq!(report.changed, 1);
    assert_eq!(report.written, 1);
    assert_eq!(report.linked, 0);
}
