use super::super::{build_analysis_context, should_sync_analyzer, sync_analyzer_before_check};
use acorn::analyzer::host::context::{AnalysisContext, AnalysisOptions};
use acorn::analyzer::readability::ReadabilityType;
use acorn::analyzer::{CheckCategory, OutputFormat, Standard};
use jiff::SignedDuration;

#[test]
fn test_check_context_builder_assigns_every_option_to_its_owner() {
    let context = build_analysis_context(
        &[CheckCategory::Schema, CheckCategory::Link],
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        Standard::Datacite,
        ReadabilityType::ARI,
        false,
        true,
        true,
        SignedDuration::from_secs(7),
    );
    assert_eq!(context.analysis.readability_metric, ReadabilityType::ARI);
    assert_eq!(context.analysis.skip, ["schema", "link"]);
    assert_eq!(context.analysis.standard, Standard::Datacite);
    assert!(context.execution.disable_website_checks);
    assert!(context.execution.ignore_sync_failure);
    assert!(context.execution.offline);
    assert!(context.execution.quiet);
    assert!(!context.execution.skip_sync);
    assert!(context.execution.skip_verify_checksum);
    assert_eq!(context.execution.sync_retry_interval, SignedDuration::from_secs(7));
    assert!(context.report.all);
    assert!(context.report.exit_on_first_error);
    assert_eq!(context.report.format, OutputFormat::Console);
    assert!(context.report.no_fail);
    assert!(context.report.quiet);
    assert!(context.report.terse);
}

#[test]
fn test_check_context_builder_keeps_non_raw_output_visible() {
    let context = build_analysis_context(
        &[],
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        Standard::ResearchActivityData,
        ReadabilityType::FKGL,
        false,
        false,
        false,
        SignedDuration::from_secs(1),
    );
    assert!(!context.execution.quiet);
    assert!(!context.report.quiet);
}

#[test]
fn test_should_sync_analyzer_reflects_prose_selection() {
    assert!(should_sync_analyzer(&AnalysisContext::default()));
    let skipped = AnalysisContext::init()
        .analysis(AnalysisOptions::init().skip(vec!["prose".to_string()]).build())
        .build();
    assert!(!should_sync_analyzer(&skipped));
}

#[tokio::test]
async fn test_sync_analyzer_skips_resolution_when_prose_is_omitted() {
    let context = AnalysisContext::init()
        .analysis(
            AnalysisOptions::init()
                .skip(vec!["prose".to_string()])
                .standard(Standard::Datacite)
                .build(),
        )
        .build();
    let resolved = sync_analyzer_before_check(&context)
        .await
        .expect("skipped prose should avoid analyzer resolution");
    assert_eq!(resolved.analysis.standard, Standard::Datacite);
    assert!(resolved.execution.skip_sync);
}
