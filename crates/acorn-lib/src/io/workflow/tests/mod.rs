use super::*;
use crate::schema::research_activity::{ResearchActivityMetadata, ResearchOutput};
use tempfile::tempdir;

struct OutputEnrichment;

#[async_trait]
impl Enrichment for OutputEnrichment {
    async fn enrich(&self, _source: &Source, data: ResearchActivity) -> ApiResult<ResearchActivity> {
        let output = ResearchOutput::init()
            .identifier("https://openalex.org/W2741809807".to_string())
            .doi("10.1038/nature12373".to_string())
            .title("Enriched output".to_string())
            .kind("article".to_string())
            .build();
        Ok(ResearchActivity {
            meta: ResearchActivityMetadata {
                outputs: Some(vec![output]),
                ..data.meta
            },
            ..data
        })
    }
}

fn source_file() -> (tempfile::TempDir, Source) {
    let temporary = tempdir().expect("temporary workflow directory should be created");
    let path = temporary.path().join("activity.json");
    ResearchActivity::default()
        .write_json(path.clone())
        .expect("research activity fixture should be written");
    let source = Source::init().path(path).bucket("example").key("activity.json").build();
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
}

#[tokio::test]
async fn test_plan_enriches_without_writing_source() {
    let (_temporary, source) = source_file();
    let before = read_file(source.path.clone()).expect("source should be readable");
    let options = Options::init().enrich(true).build();
    let plan = Workflow::plan(core::slice::from_ref(&source), options, Some(&OutputEnrichment))
        .await
        .expect("workflow plan should succeed");
    let after = read_file(source.path).expect("source should remain readable");
    assert_eq!(before, after);
    assert_eq!(plan.changes.len(), 1);
    let change = plan.changes.first().expect("plan should contain one change");
    assert!(change.changed);
    assert_eq!(change.data.meta.outputs.as_ref().map(Vec::len), Some(1));
}

#[tokio::test]
async fn test_dry_run_applies_no_changes() {
    let (_temporary, source) = source_file();
    let before = read_file(source.path.clone()).expect("source should be readable");
    let plan_options = Options::init().enrich(true).link(true).build();
    let plan = Workflow::plan(core::slice::from_ref(&source), plan_options, Some(&OutputEnrichment))
        .await
        .expect("workflow plan should succeed");
    let report = Workflow::apply(plan, Options::init().dry_run(true).build()).expect("dry run should succeed");
    assert_eq!(report.processed, 1);
    assert_eq!(report.changed, 1);
    assert_eq!(report.linked, 1);
    assert_eq!(report.written, 0);
    assert_eq!(read_file(source.path.clone()).expect("source should remain readable"), before);
    assert!(!source.path.with_extension("jsonld").exists());
}

#[tokio::test]
async fn test_run_writes_enrichment_and_linked_artifact() {
    let (_temporary, source) = source_file();
    let options = Options::init().enrich(true).link(true).build();
    let report = Workflow::run(core::slice::from_ref(&source), options, Some(&OutputEnrichment))
        .await
        .expect("workflow run should succeed");
    let written = ResearchActivity::read(source.path.clone()).expect("updated source should parse");
    assert_eq!(written.meta.outputs.as_ref().map(Vec::len), Some(1));
    assert!(source.path.with_extension("jsonld").exists());
    assert_eq!(report.processed, 1);
    assert_eq!(report.changed, 1);
    assert_eq!(report.linked, 1);
    assert_eq!(report.written, 2);
}

#[tokio::test]
async fn test_enrichment_requires_capability_and_online_mode() {
    let (_temporary, source) = source_file();
    let missing = Workflow::plan(core::slice::from_ref(&source), Options::init().enrich(true).build(), None).await;
    let offline = Workflow::plan(&[source], Options::init().enrich(true).offline(true).build(), Some(&OutputEnrichment)).await;
    assert!(missing.is_err());
    assert!(offline.is_err());
}
