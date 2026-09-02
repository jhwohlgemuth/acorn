use super::*;
#[cfg(feature = "analysis")]
use crate::analyzer::{Check, CheckCategory};
use acorn_schema::schema::research_activity::{ResearchActivity, ResearchActivityMetadata, ResearchOutput};
use acorn_schema::schema::standard::cff::Cff;
use async_trait::async_trait;
use tempfile::tempdir;

struct OutputEnrichment;
struct CffEnrichment;
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
                    outputs: Some(vec![output]),
                    ..data.meta
                },
                ..data
            },
            conflicts: Vec::new(),
            failures: Vec::new(),
        })
    }
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
fn source_file() -> (tempfile::TempDir, Location) {
    let temporary = tempdir().expect("temporary workflow directory should be created");
    let path = temporary.path().join("activity.json");
    ResearchActivity::default()
        .write_json(path.clone())
        .expect("research activity fixture should be written");
    let source = Location::from(&path.display().to_string());
    (temporary, source)
}

#[test]
fn test_default_options_only_check() {
    let options = Options::default();
    assert!(options.check);
    assert!(!options.format);
    assert!(!options.enrich);
    assert!(!options.link);
    assert!(!options.dry_run);
    assert!(!options.offline);
    assert!(!options.no_color);
    assert!(!options.show_changes);
}
#[tokio::test]
async fn test_plan_enriches_without_writing_source() {
    let (_temporary, source) = source_file();
    let path = source.local_path().expect("source should have a local path");
    let before = read_file(path.clone()).expect("source should be readable");
    let options = Options::init().enrich(true).build();
    let plan = Workflow::<ResearchActivity>::plan(core::slice::from_ref(&source), options, Some(&OutputEnrichment))
        .await
        .expect("workflow plan should succeed");
    let after = read_file(path).expect("source should remain readable");
    assert_eq!(before, after);
    assert_eq!(plan.changes.len(), 1);
    let change = plan.changes.first().expect("plan should contain one change");
    assert!(change.is_changed());
    assert_eq!(change.data().and_then(|data| data.meta.outputs.as_ref().map(Vec::len)), Some(1));
}
#[tokio::test]
async fn test_dry_run_applies_no_changes() {
    let (_temporary, source) = source_file();
    let path = source.local_path().expect("source should have a local path");
    let before = read_file(path.clone()).expect("source should be readable");
    let plan_options = Options::init().enrich(true).link(true).build();
    let plan = Workflow::<ResearchActivity>::plan(core::slice::from_ref(&source), plan_options, Some(&OutputEnrichment))
        .await
        .expect("workflow plan should succeed");
    let report = Workflow::<ResearchActivity>::apply(plan, Options::init().dry_run(true).build()).expect("dry run should succeed");
    assert_eq!(report.processed, 1);
    assert_eq!(report.changed, 1);
    assert_eq!(report.linked, 1);
    assert_eq!(report.written, 0);
    assert_eq!(read_file(path.clone()).expect("source should remain readable"), before);
    assert!(!path.with_extension("jsonld").exists());
}
#[tokio::test]
async fn test_run_writes_enrichment_and_linked_artifact() {
    let (_temporary, source) = source_file();
    let options = Options::init().enrich(true).link(true).build();
    let report = Workflow::<ResearchActivity>::run(core::slice::from_ref(&source), options, Some(&OutputEnrichment))
        .await
        .expect("workflow run should succeed");
    let path = source.local_path().expect("source should have a local path");
    let written = ResearchActivity::read(path.clone()).expect("updated source should parse");
    assert_eq!(written.meta.outputs.as_ref().map(Vec::len), Some(1));
    assert!(path.with_extension("jsonld").exists());
    assert_eq!(report.processed, 1);
    assert_eq!(report.changed, 1);
    assert_eq!(report.linked, 1);
    assert_eq!(report.written, 2);
}
#[tokio::test]
async fn test_enrichment_requires_capability_and_online_mode() {
    let (_temporary, source) = source_file();
    let missing = Workflow::<ResearchActivity>::plan(core::slice::from_ref(&source), Options::init().enrich(true).build(), None).await;
    let offline = Workflow::<ResearchActivity>::plan(&[source], Options::init().enrich(true).offline(true).build(), Some(&OutputEnrichment)).await;
    assert!(missing.is_err());
    assert!(offline.is_err());
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
    let options = Options::init().check(false).enrich(true).build();
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
